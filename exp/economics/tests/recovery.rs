use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Advance, LoanOffer},
    finance::CollectionPolicy,
    model::*,
    recovery::{Bid, Guarantee, Listing, ProceedingTerms, Receipt, Stage},
    scenario::{self, PERSON, PLOT, STATE_AGENT, TOKEN},
    settlement,
    simulation::Simulation,
};

const OTHER: AgentId = 97;
const BUYER: AgentId = 98;
const ESTATE: AgentId = 99;
fn advance(id: u32, creditor: AgentId) -> Advance {
    Advance {
        id,
        debtor: PERSON,
        terms: LoanOffer {
            creditor,
            denomination: TOKEN,
            max_principal: 10,
            monthly_rate_bps: 0,
            term_months: 1,
            grace_months: 10,
        },
        principal: 10,
        month: 1,
        collateral: None,
        priority: 0,
    }
}
fn fixture() -> (World, State) {
    let (mut w, mut s) = scenario::baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights.clear();
    w.agreements.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coins".into(),
        kind: ResourceKind::Stock,
    });
    for id in [OTHER, BUYER, ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = PERSON;
    w.collection_policy = CollectionPolicy::Proportional;
    w.lending = vec![advance(10, STATE_AGENT), advance(11, OTHER)];
    s.balances.clear();
    for id in [STATE_AGENT, OTHER] {
        s.balances.insert((id, TOKEN), 10);
    }
    s.balances.insert((BUYER, TOKEN), 8);
    (w, s)
}
fn guarantee(id: u32, loan: u32, cap: i32) -> Guarantee {
    Guarantee {
        id,
        loan,
        guarantor: BUYER,
        cap,
        from: 1,
        through: 12,
        delay_months: 0,
        recourse: 100 + id,
        priority: 0,
    }
}
fn proceeding(w: &mut World, discharge: bool) {
    w.lending[0].terms.grace_months = 1;
    w.lending[0].collateral = Some(credit::Collateral {
        asset: PLOT,
        priority: 0,
        pledged: true,
        settlement: credit::CollateralSettlement::FixedValue { value: 10 },
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![Listing {
            asset: PLOT,
            minimum_price: 6,
        }],
        discharge_deficiency: discharge,
    });
    w.recovery.bids.push(Bid {
        id: 1,
        proceeding: 1,
        buyer: BUYER,
        asset: PLOT,
        month: 3,
        price: 8,
    });
}
fn distressed(w: World, s: State, backend: Backend) -> Simulation {
    let mut sim = Simulation::new(w, s, backend).unwrap();
    sim.run_months(1).unwrap();
    // A controlled external loss, identical between policies/backends.
    sim.state.balances.insert((PERSON, TOKEN), 0);
    sim
}
#[test]
fn guarantees_share_finite_funds_and_create_one_collectible_recourse_claim() {
    let (mut w, s) = fixture();
    w.recovery.guarantees = vec![guarantee(1, 10, 6), guarantee(2, 11, 6)];
    let mut sim = distressed(w, s, Backend::CubeCpu);
    let mut reordered = sim.clone();
    reordered.world.recovery.guarantees.reverse();
    sim.run_months(1).unwrap();
    reordered.run_months(1).unwrap();
    assert_eq!(sim.state, reordered.state);
    assert_eq!(sim.ledger, reordered.ledger);
    assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], 6);
    assert_eq!(sim.state.credit.recovery.paid_guarantees[&2], 2);
    assert_eq!(sim.state.credit.loans[&10].principal, 4);
    assert_eq!(sim.state.credit.loans[&11].principal, 8);
    assert_eq!(sim.state.credit.loans[&101].principal, 6);
    assert_eq!(sim.state.credit.loans[&102].principal, 2);
    assert_eq!(sim.state.balance(BUYER, TOKEN), 0);
    assert_eq!(
        credit::balance_sheet(&sim.world, &sim.state, PERSON, TOKEN).principal_payable,
        20
    );
    assert_eq!(
        credit::balance_sheet(&sim.world, &sim.state, BUYER, TOKEN).principal_receivable,
        8
    );
    // Partial guarantee does not create cash for the debtor. Recourse becomes due later.
    assert_eq!(sim.state.credit.loans[&101].opened, 2);
    sim.world
        .claim_priorities
        .insert(economics_compute_smoke::finance::ContractId::Loan(101), 0);
    for id in [10, 11, 102] {
        sim.world
            .claim_priorities
            .insert(economics_compute_smoke::finance::ContractId::Loan(id), 1);
    }
    sim.state.balances.insert((PERSON, TOKEN), 6);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.credit.loans[&101].status, credit::Status::Repaid);
    // Repayment receipts cannot finance another guarantee in the same Due window.
    assert_eq!(sim.state.credit.recovery.paid_guarantees[&2], 2);
    assert_eq!(sim.state.balance(BUYER, TOKEN), 6);
}
#[test]
fn liquidation_uses_real_proceeds_and_preserves_deficiency_or_explicitly_discharges() {
    for discharge in [false, true] {
        let (mut w, s) = fixture();
        proceeding(&mut w, discharge);
        let mut sim = distressed(w, s, Backend::CubeCpu);
        sim.run_months(2).unwrap(); // default in 2, authorized stay + sale in 3
        assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(BUYER));
        assert_eq!(sim.state.balance(ESTATE, TOKEN), 8);
        assert_eq!(sim.state.credit.loans[&10].principal, 10); // no same-month recycling
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Active
        );
        assert_eq!(
            credit::balance_sheet(&sim.world, &sim.state, ESTATE, TOKEN).equity(),
            0
        );
        assert_eq!(
            credit::balance_sheet(&sim.world, &sim.state, PERSON, TOKEN).estate_cash,
            8
        );
        let mut resumed = sim.clone();
        let mut reference =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
        sim.run_months(2).unwrap();
        resumed.run_months(1).unwrap();
        resumed.run_months(1).unwrap();
        reference.run_months(2).unwrap();
        assert_eq!(sim.state, resumed.state);
        assert_eq!(sim.ledger, resumed.ledger);
        assert_eq!(sim.state, reference.state);
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 8);
        assert_eq!(sim.state.balance(OTHER, TOKEN), 0);
        assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Closed
        );
        assert_eq!(
            sim.state.credit.loans[&10].principal,
            if discharge { 0 } else { 2 }
        );
        assert_eq!(
            sim.state.credit.loans[&11].principal,
            if discharge { 0 } else { 10 }
        );
    }
}
#[test]
fn an_unfunded_bid_leaves_the_asset_and_claims_unsold_and_stayed() {
    let (mut w, s) = fixture();
    proceeding(&mut w, false);
    w.recovery.bids[0].price = 20;
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(4).unwrap();
    assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
    assert_eq!(sim.state.credit.recovery.proceedings[&1].cash, 0);
    assert!(sim.state.credit.recovery.proceedings[&1].sold.is_empty());
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert_eq!(sim.state.credit.loans[&10].principal, 10);
    assert_eq!(sim.state.credit.loans[&10].status, credit::Status::Stayed);
}
#[test]
fn a_tampered_estate_sale_cannot_publish_title_cash_or_receipts() {
    let (mut w, s) = fixture();
    proceeding(&mut w, false);
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(1).unwrap();
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Acquire);
    let mut batch = Batch::empty(&sim.state);
    batch.credit = credit::evaluate(&sim.world, &sim.state).unwrap();
    batch.transactions = batch.credit.as_ref().unwrap().transactions.clone();
    batch
        .credit
        .as_mut()
        .unwrap()
        .after
        .recovery
        .proceedings
        .get_mut(&1)
        .unwrap()
        .cash += 1;
    let before = sim.state.clone();
    assert!(
        settlement::commit(&sim.world, &mut sim.state, &batch, Backend::Reference, 4096).is_err()
    );
    assert_eq!(sim.state, before);
}
#[test]
fn arrears_alone_do_not_open_a_case_and_solvent_authorizations_are_rejected() {
    let (w, s) = fixture();
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(2).unwrap();
    assert!(sim.state.credit.recovery.proceedings.is_empty());
    let (mut w, s) = fixture();
    proceeding(&mut w, false);
    let mut solvent = Simulation::new(w, s, Backend::Reference).unwrap();
    solvent.run_months(2).unwrap();
    solvent.step().unwrap();
    let boundary = credit::evaluate(&solvent.world, &solvent.state)
        .unwrap()
        .unwrap();
    assert!(
        boundary
            .recovery
            .contains(&Receipt::OpeningRejected { proceeding: 1 })
    );
    assert!(boundary.after.recovery.proceedings.is_empty());
}
#[test]
fn guarantee_recursion_and_custody_spending_are_explicitly_rejected() {
    let (mut w, s) = fixture();
    w.recovery.guarantees = vec![guarantee(1, 10, 5), guarantee(2, 101, 5)];
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
    let (mut w, s) = fixture();
    proceeding(&mut w, false);
    w.lending.push(advance(12, ESTATE));
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
}

#[test]
fn equal_rank_unsecured_creditors_share_real_proceeds_and_surplus_returns_to_debtor() {
    for price in [8, 30] {
        let (mut w, mut s) = fixture();
        proceeding(&mut w, false);
        w.lending[0].collateral = None;
        w.recovery.bids[0].price = price;
        s.balances.insert((BUYER, TOKEN), price);
        let mut sim = distressed(w, s, Backend::Reference);
        sim.run_months(3).unwrap();
        let per_creditor = if price == 8 { 4 } else { 10 };
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), per_creditor);
        assert_eq!(sim.state.balance(OTHER, TOKEN), per_creditor);
        assert_eq!(sim.state.balance(PERSON, TOKEN), (price - 20).max(0));
        assert_eq!(sim.state.credit.recovery.proceedings[&1].cash, 0);
    }
}

#[test]
fn custody_receipts_wait_for_the_next_boundary_and_expired_guarantees_do_not_call() {
    let (mut w, s) = fixture();
    proceeding(&mut w, false);
    w.lending[0].collateral = None;
    w.recovery.proceedings[0].assets.clear();
    w.recovery.bids.clear();
    let mut g = guarantee(1, 10, 6);
    g.through = 1;
    w.recovery.guarantees.push(g);
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, TOKEN), 9);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.balance(ESTATE, TOKEN), 9);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
    assert_eq!(sim.state.balance(OTHER, TOKEN), 0);
    assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
    assert!(!sim.state.credit.loans.contains_key(&101));
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 5);
    assert_eq!(sim.state.balance(OTHER, TOKEN), 4);
    assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
}

#[test]
fn unfunded_top_bid_releases_the_asset_for_a_funded_bid_and_discharge_is_not_repayment() {
    let (mut w, s) = fixture();
    proceeding(&mut w, true);
    let mut unfunded = w.recovery.bids[0].clone();
    unfunded.id = 2;
    unfunded.price = 30;
    w.recovery.bids.push(unfunded);
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(2).unwrap();
    assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(BUYER));
    sim.step().unwrap();
    let boundary = credit::evaluate(&sim.world, &sim.state).unwrap().unwrap();
    assert!(boundary.recovery.contains(&Receipt::WrittenOff {
        proceeding: 1,
        loan: 10,
        principal: 2,
        interest: 0
    }));
    assert!(boundary.recovery.contains(&Receipt::WrittenOff {
        proceeding: 1,
        loan: 11,
        principal: 10,
        interest: 0
    }));
    sim.step().unwrap();
    assert_eq!(
        sim.state.credit.loans[&10].status,
        credit::Status::Discharged
    );
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 8);
}

#[test]
fn recovery_observers_report_actual_guarantees_distributions_and_writeoffs() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let (mut w, s) = fixture();
    proceeding(&mut w, true);
    let mut sim = distressed(w, s, Backend::Reference);
    let mut plain = sim.clone();
    let mut observer = Observer::new(
        vec![],
        "recovery",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 3).unwrap();
    plain.run_months(3).unwrap();
    assert_eq!(sim.state, plain.state);
    assert_eq!(sim.ledger, plain.ledger);
    let bytes = observer.finish().unwrap();
    let rows: Vec<serde_json::Value> = String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let distributions: i64 = rows
        .iter()
        .filter(|r| r["kind"] == "estate_recovery" && r["detail"]["event"] == "Distributed")
        .map(|r| r["detail"]["paid"].as_i64().unwrap())
        .sum();
    let losses: i64 = rows
        .iter()
        .filter(|r| r["kind"] == "estate_recovery" && r["detail"]["event"] == "WrittenOff")
        .map(|r| r["detail"]["principal"].as_i64().unwrap())
        .sum();
    assert_eq!(distributions, 8);
    assert_eq!(losses, 12);
    let receipts: Vec<_> = rows
        .iter()
        .filter(|r| r["kind"] == "estate_recovery" && r["detail"]["event"] == "Distributed")
        .map(|r| &r["detail"])
        .collect();
    assert!(
        receipts
            .iter()
            .any(|r| r["requested"].as_i64().unwrap() > 0 && r["allocated"] == 0 && r["paid"] == 0)
    );
    assert!(receipts.iter().all(|r| {
        let requested = r["requested"].as_i64().unwrap();
        let allocated = r["allocated"].as_i64().unwrap();
        let paid = r["paid"].as_i64().unwrap();
        0 <= paid && paid <= allocated && allocated <= requested
    }));
    let (mut w, s) = fixture();
    w.recovery.guarantees.push(guarantee(1, 10, 6));
    let mut sim = distressed(w, s, Backend::Reference);
    let mut observer = Observer::new(
        vec![],
        "guarantees",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 1).unwrap();
    let log = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert!(
        log.lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .any(|r| r["kind"] == "guarantee_payment" && r["paid"] == 6 && r["recourse"] == 101)
    );
}

#[test]
fn new_borrowing_is_rejected_during_a_stay_and_other_claim_types_are_not_ignored() {
    let (mut w, s) = fixture();
    proceeding(&mut w, false);
    w.recovery.bids.clear();
    let mut later = advance(12, BUYER);
    later.month = 3;
    later.principal = 5;
    w.lending.push(later);
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(2).unwrap();
    assert!(!sim.state.credit.loans.contains_key(&12));
    assert_eq!(sim.state.balance(BUYER, TOKEN), 8);
    let (mut w, s) = scenario::named("annual-access").unwrap();
    w.priority = Priority::ContinuingFirst;
    w.resources.push(Resource {
        id: TOKEN,
        name: "coins".into(),
        kind: ResourceKind::Stock,
    });
    w.agents.push(Agent {
        id: ESTATE,
        name: "estate".into(),
    });
    w.lending.push(advance(10, STATE_AGENT));
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![],
        discharge_deficiency: false,
    });
    assert!(
        credit::validate(&w, &s)
            .unwrap_err()
            .contains("land dues or forward claims")
    );
}

#[test]
fn an_authorized_stay_freezes_interest_and_prevents_due_collateral_enforcement() {
    let (mut w, s) = fixture();
    proceeding(&mut w, false);
    w.lending[0].terms.monthly_rate_bps = 1_000;
    w.recovery.bids.clear();
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(4).unwrap();
    let loan = &sim.state.credit.loans[&10];
    assert_eq!(loan.interest, 1); // accrued in month 2, frozen from authorized month 3
    assert_eq!(loan.last_accrued, 5);
    assert_eq!(loan.status, credit::Status::Stayed);
    assert!(loan.collateral.as_ref().unwrap().pledged);
    assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
}

#[test]
fn resumed_estates_reject_unknown_lien_references_before_execution() {
    let (mut w, s) = fixture();
    proceeding(&mut w, false);
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(2).unwrap();
    sim.state
        .credit
        .recovery
        .proceedings
        .get_mut(&1)
        .unwrap()
        .secured
        .insert(123_456, 0);
    assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
}

#[test]
fn common_contract_inspection_exposes_contingent_guarantees_without_mutating_debt() {
    use economics_compute_smoke::agreements::{self, Identity, View};
    let (mut w, mut s) = fixture();
    w.recovery.guarantees.push(guarantee(1, 10, 6));
    s.balances.insert((BUYER, TOKEN), 0);
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(1).unwrap();
    let before = sim.state.clone();
    for agent in [PERSON, STATE_AGENT, BUYER] {
        let views = agreements::for_agent(&sim.world, &sim.state, agent).unwrap();
        let view = views
            .iter()
            .find(|v| v.identity() == Identity::Guarantee(1))
            .unwrap();
        let View::Guarantee(g) = view else {
            panic!("missing guarantee adapter")
        };
        assert_eq!(g.paid, 0);
        assert_eq!(view.claims().unwrap()[0].outstanding(), 6);
    }
    assert_eq!(sim.state, before);
    assert_eq!(
        credit::balance_sheet(&sim.world, &sim.state, PERSON, TOKEN).principal_payable,
        20
    );
    assert!(
        !agreements::for_agent(&sim.world, &sim.state, OTHER)
            .unwrap()
            .iter()
            .any(|v| v.identity() == Identity::Guarantee(1))
    );
}
