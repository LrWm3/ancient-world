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
        follows_assignment: false,
        tender: economics_compute_smoke::recovery::GuaranteeTender::Native,
        security: economics_compute_smoke::recovery::RecourseSecurity::Unsecured,
        id,
        claim: economics_compute_smoke::recovery::GuaranteedClaim::Loan(loan),
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
fn liquidation_discovery_and_common_acceptance_use_funded_bids_and_existing_custody() {
    use economics_compute_smoke::{
        financial_reporting::Audit,
        offers::{self, Id, Request, Terms},
    };
    for funded in [false, true] {
        let (mut w, mut s) = fixture();
        proceeding(&mut w, true);
        s.balances
            .insert((BUYER, TOKEN), if funded { 8 } else { 0 });
        let mut sim = distressed(w, s, Backend::CubeCpu);
        assert!(
            economics_compute_smoke::recovery::market::discover(&sim.world, &sim.state, BUYER)
                .is_empty()
        );
        let mut audit =
            Audit::with_assets(&sim.world, &sim.state, TOKEN, [(PLOT, 10)].into()).unwrap();
        while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
            audit.step(&mut sim).unwrap();
        }
        let inventory =
            economics_compute_smoke::recovery::market::discover(&sim.world, &sim.state, BUYER);
        assert_eq!(inventory.len(), 1);
        assert_eq!(inventory[0].listing.minimum_price, 6);
        assert!(
            economics_compute_smoke::recovery::market::discover(&sim.world, &sim.state, ESTATE)
                .is_empty()
        );
        assert!(
            offers::discover(&sim.world, &sim.state, BUYER)
                .iter()
                .any(|o| matches!(&o.terms, Terms::Liquidation { bid, .. } if bid.id == 1))
        );
        let request = Request::new(Id::LiquidationBid(1), BUYER);
        let before = sim.clone();
        let preview = offers::prepare(&sim, std::slice::from_ref(&request));
        assert_eq!(preview.is_ok(), funded);
        assert_eq!(sim.state, before.state);
        assert!(offers::prepare(&sim, &[Request::new(Id::LiquidationBid(1), OTHER)]).is_err());
        if funded {
            let mut accepted = sim.clone();
            offers::accept(&mut accepted, &[request]).unwrap();
            audit.step(&mut sim).unwrap();
            assert_eq!(sim.state, accepted.state);
            assert_eq!(sim.ledger, accepted.ledger);
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 8);
            assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(BUYER));
            assert!(
                economics_compute_smoke::recovery::market::discover(&sim.world, &sim.state, OTHER)
                    .is_empty()
            );
            assert!(
                offers::accept(&mut sim, &[Request::new(Id::LiquidationBid(1), BUYER)]).is_err()
            );
            let mut resumed = (sim.clone(), audit.clone());
            while sim.state.month <= 5 {
                audit.step(&mut sim).unwrap();
            }
            resumed.0.backend = Backend::Reference;
            while resumed.0.state.month <= 5 {
                resumed.1.step(&mut resumed.0).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.0.state, &resumed.0.ledger, &resumed.1)
            );
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Closed
            );
        } else {
            assert!(offers::accept(&mut sim, &[request]).is_err());
            assert_eq!(sim.state, before.state);
            audit.step(&mut sim).unwrap();
            assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
        }
    }
}

#[test]
fn common_bid_requests_cannot_override_competing_funded_priority_or_duplicate_a_sale() {
    use economics_compute_smoke::offers::{self, Id, Request};
    let (mut w, s) = fixture();
    proceeding(&mut w, true);
    let mut duplicate = w.recovery.bids[0].clone();
    duplicate.id = 2;
    let mut unfunded = duplicate.clone();
    unfunded.id = 3;
    unfunded.buyer = OTHER;
    unfunded.price = 9;
    w.recovery.bids.extend([duplicate, unfunded]);
    let mut sim = distressed(w, s, Backend::Reference);
    while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
        sim.step().unwrap();
    }
    assert!(offers::prepare(&sim, &[Request::new(Id::LiquidationBid(2), BUYER)]).is_err());
    assert!(offers::prepare(&sim, &[Request::new(Id::LiquidationBid(3), OTHER)]).is_err());
    let prepared = offers::prepare(&sim, &[Request::new(Id::LiquidationBid(1), BUYER)]).unwrap();
    let mut reordered = sim.clone();
    reordered.world.recovery.bids.reverse();
    assert_eq!(
        offers::prepare(&reordered, &[Request::new(Id::LiquidationBid(1), BUYER)]).unwrap(),
        prepared
    );
    assert_eq!(
        prepared
            .credit
            .as_ref()
            .unwrap()
            .recovery
            .iter()
            .filter(|r| matches!(r, Receipt::Sold { .. }))
            .count(),
        1
    );
}

#[test]
fn winding_household_cannot_exercise_a_liquidation_bid_even_with_reserved_cash() {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, Agreement},
        offers::{self, Id, Request},
    };
    const HOME: AgentId = 10000;
    for winding in [false, true] {
        let (mut w, mut s) = fixture();
        proceeding(&mut w, true);
        let (baseline, _) = scenario::baseline();
        let mut member = baseline.participants[0].clone();
        member.agent = BUYER;
        member.needs.clear();
        member.capacity.quantity = 0;
        w.participants.push(member);
        let mut governance = Governance::contributed(BUYER);
        governance.constitution.allow_dissolution = true;
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: vec![BUYER],
                governance,
                formed: 1,
                dwelling_process: None,
                admission: None,
                membership: vec![],
                asset_sales: vec![],
                equipment_retirements: vec![],
                support: vec![],
            },
        )
        .unwrap();
        s.balances.insert((BUYER, TOKEN), 0);
        s.balances.insert((HOME, TOKEN), 8);
        w.recovery.bids[0].buyer = HOME;
        // Previously consented future lending prevents distribution while the
        // optional asset bid is tested against identical eight-coin balances.
        let mut future = advance(30, STATE_AGENT);
        future.debtor = HOME;
        future.month = 6;
        future.principal = 1;
        w.lending.push(future);
        let mut sim = distressed(w, s, Backend::CubeCpu);
        if winding {
            households::dissolution::request(&mut sim.world, &sim.state, HOME, BUYER).unwrap();
        }
        while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
            sim.step().unwrap();
        }
        assert_eq!(sim.state.balance(HOME, TOKEN), 8);
        assert_eq!(
            offers::prepare(&sim, &[Request::new(Id::LiquidationBid(1), HOME)]).is_ok(),
            !winding
        );
        assert_eq!(
            economics_compute_smoke::recovery::market::discover(&sim.world, &sim.state, HOME)
                .is_empty(),
            winding
        );
        sim.step().unwrap();
        assert_eq!(
            credit::owner(&sim.world, &sim.state, PLOT),
            Some(if winding { PERSON } else { HOME })
        );
        assert_eq!(sim.state.balance(HOME, TOKEN), if winding { 8 } else { 0 });
    }
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
        sim.state
            .credit
            .loans
            .values()
            .filter(|l| l.debtor == PERSON && l.denomination == TOKEN)
            .map(|l| l.principal)
            .sum::<i32>(),
        20
    );
    assert_eq!(
        sim.state
            .credit
            .loans
            .values()
            .filter(|l| l.creditor == BUYER && l.denomination == TOKEN)
            .map(|l| l.principal)
            .sum::<i32>(),
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
        assert_eq!(sim.state.credit.recovery.proceedings[&1].cash, 8);
        assert_eq!(sim.world.recovery.proceedings[0].debtor, PERSON);
        assert_eq!(sim.world.recovery.proceedings[0].estate, ESTATE);
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
fn new_borrowing_is_rejected_during_a_stay_and_land_claims_are_admissible() {
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
    credit::validate(&w, &s).unwrap();
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
        sim.state
            .credit
            .loans
            .values()
            .filter(|l| l.debtor == PERSON && l.denomination == TOKEN)
            .map(|l| l.principal)
            .sum::<i32>(),
        20
    );
    assert!(
        !agreements::for_agent(&sim.world, &sim.state, OTHER)
            .unwrap()
            .iter()
            .any(|v| v.identity() == Identity::Guarantee(1))
    );
}

fn land_estate(alternative: bool) -> Simulation {
    use economics_compute_smoke::{
        activities::CoinPayment, commitments::Agreement, currency::Issuance,
    };
    let (mut w, s) = fixture();
    w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = STATE_AGENT;
    w.rights = scenario::baseline().0.rights;
    w.rights[0].through = 60;
    w.agreements.push(Agreement {
        id: 1,
        right: 1,
        creditor: STATE_AGENT,
        debtor: PERSON,
        activated: 1,
        payment: Amount::new(scenario::GRAIN, 2),
    });
    w.issuance.push(Issuance {
        agreement: 1,
        token: TOKEN,
        collected_per_token: 2,
    });
    if alternative {
        w.activities.coin_payments.insert(
            1,
            CoinPayment {
                resource: TOKEN,
                coins_per_unit: 2,
            },
        );
    }
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 14,
        earliest_close: 15,
        assets: vec![],
        discharge_deficiency: true,
    });
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(12).unwrap();
    assert_eq!(sim.state.month, 14);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 0);
    sim
}

#[test]
fn land_tender_shares_estate_cash_in_whole_units_and_blocks_premature_discharge() {
    use economics_compute_smoke::{finance::ContractId, recovery_claims};
    let mut sim = land_estate(true);
    sim.state.balances.insert((PERSON, TOKEN), 12);
    let mut cpu = Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    cpu.run_months(1).unwrap();
    assert_eq!(sim.state.balance(ESTATE, TOKEN), 12);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 0);
    let saved = sim.state.clone();
    let mut resumed = Simulation::new(sim.world.clone(), saved, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    cpu.run_months(1).unwrap();
    resumed.run_months(1).unwrap();
    assert_eq!(sim.state, cpu.state);
    assert_eq!(sim.state, resumed.state);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 1);
    assert_eq!(sim.state.obligations[&(1, 13)].in_kind_paid, 0);
    assert_eq!(sim.state.credit.loans[&10].principal, 5);
    assert_eq!(sim.state.credit.loans[&11].principal, 5);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert_eq!(
        recovery_claims::outstanding(&sim.world, &sim.state, PERSON)[0].remaining,
        Amount::new(scenario::GRAIN, 1)
    );
    assert!(
        sim.ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .all(|t| !t.cause.starts_with("issue currency"))
    );
    // Change only the explicit rank: the final two coins now cure land arrears.
    sim.world.claim_priorities.insert(ContractId::Land(1), 0);
    sim.world.claim_priorities.insert(ContractId::Loan(10), 1);
    sim.world.claim_priorities.insert(ContractId::Loan(11), 1);
    sim.state.balances.insert((PERSON, TOKEN), 2);
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 2);
    assert_eq!(sim.state.obligations[&(1, 13)].in_kind_paid, 0);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    assert_eq!(
        sim.state.credit.loans[&10].status,
        credit::Status::Discharged
    );
}

#[test]
fn native_land_receipts_keep_storage_issuance_and_clear_arrears_timing() {
    use economics_compute_smoke::commitments;
    let mut sim = land_estate(false);
    sim.world.storage.weights.insert(scenario::GRAIN, 1);
    sim.world.storage.capacities.insert(STATE_AGENT, 1);
    sim.state.balances.insert((PERSON, scenario::GRAIN), 2);
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 1);
    assert_eq!(sim.state.obligations[&(1, 13)].in_kind_paid, 1);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert!(!commitments::can_start(&sim.world, &sim.state, 1));
    // A genuine new unit of receiving space permits the original arrears retry.
    while sim.state.phase != Phase::ClearArrears {
        sim.step().unwrap();
    }
    sim.world.storage.capacities.insert(STATE_AGENT, 2);
    let mut preview = sim.clone();
    preview.step().unwrap();
    let mut bad = preview.ledger.last().unwrap().clone();
    bad.transactions[0].effects[0].delta -= 1;
    let opening = sim.state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &bad,
            Backend::Reference,
            settlement::DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(sim.state, opening);
    sim.step().unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].in_kind_paid, 2);
    assert!(commitments::can_start(&sim.world, &sim.state, 1));
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 1);
    // The next Due can close; the original fixed annual calendar still continues.
    sim.run_months(10).unwrap();
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    assert!(sim.state.obligations.contains_key(&(1, 25)));
}

fn with_accepted_forward(mut sim: Simulation, due: u32) -> Simulation {
    use economics_compute_smoke::{
        currency::Bid,
        equipment::DurableAsset,
        exchange::{Delivery, Market, ToolRequest},
        forward::{Contract, Policy, Price, Projection, Purchase},
    };
    use std::collections::BTreeMap;
    let id = 9000;
    let goods = scenario::GRAIN;
    for agent in [PERSON, BUYER] {
        sim.world.participants.push(Participant {
            agent,
            capacity: Amount::new(scenario::LABOR, 0),
            needs: vec![],
        });
    }
    let mut definition = scenario::baseline().0.definitions[0].clone();
    definition.enabled = false;
    definition.outputs.clear();
    sim.world.activities.outcomes.insert(
        definition.id,
        economics_compute_smoke::activities::Outcome::Create(1),
    );
    sim.world.activities.kinds.insert(
        1,
        economics_compute_smoke::activities::DurableKind {
            name: "accepted tool".into(),
            lifetime: 4,
            attached: false,
            monthly_decay: 0,
        },
    );
    sim.world.definitions.push(definition);
    let c = Contract {
        id,
        debtor: PERSON,
        creditor: OTHER,
        issued: due - 12,
        due,
        goods: Amount::new(goods, 4),
        advance: Amount::new(TOKEN, 2),
        price: Price { goods: 2, coins: 1 },
        delivered: 0,
        substituted: 0,
        relief: vec![],
    };
    sim.world.bids.push(Bid {
        id: 1,
        buyer: OTHER,
        goods: Amount::new(goods, 1),
        payment: Amount::new(TOKEN, 1),
    });
    sim.world.market = Some(Market {
        capture_percent: 25,
        cash: Some(Policy {
            lender: OTHER,
            coin: TOKEN,
            months: 12,
            enabled: true,
            prices: BTreeMap::from([(goods, Price { goods: 1, coins: 1 })]),
            advance_prices: BTreeMap::from([(goods, Price { goods: 2, coins: 1 })]),
            protected: BTreeMap::new(),
        }),
        tools: vec![ToolRequest {
            buyer: PERSON,
            provider: BUYER,
            kind: 1,
        }],
        ..Default::default()
    });
    sim.state.equipment.insert(
        id,
        DurableAsset {
            id,
            owner: PERSON,
            kind: 1,
            remaining_uses: 4,
            attached_to: None,
            last_used_month: None,
        },
    );
    sim.state.exchange.forwards.insert(id, c.clone());
    sim.state.exchange.contracts.insert(
        id,
        Delivery {
            asset: id,
            buyer: PERSON,
            provider: BUYER,
            capture_percent: 0,
            purchase: Some(Purchase {
                price: Amount::new(TOKEN, 2),
                projection: Projection {
                    incremental_value: 4,
                    from: due - 12,
                    through: due - 1,
                    assisted: BTreeMap::from([(goods, 8)]),
                    surplus: BTreeMap::from([(goods, 4)]),
                },
                advance: Some(c),
            }),
        },
    );
    Simulation::new(sim.world, sim.state, Backend::Reference).unwrap()
}

#[test]
fn future_forward_is_not_accelerated_bought_out_or_discarded_at_estate_closure() {
    let mut sim = with_accepted_forward(land_estate(true), 16);
    // Supply the land in kind and keep enough grain for the future forward.
    sim.state.balances.insert((PERSON, scenario::GRAIN), 6);
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 0);
    assert_eq!(sim.state.balance(PERSON, scenario::GRAIN), 4);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert!(
        sim.ledger
            .iter()
            .flat_map(|b| b.credit.iter())
            .flat_map(|b| &b.recovery)
            .any(|r| matches!(r,Receipt::ClosureDeferred {claims,..}
            if claims.iter().any(|c| c.due == 16)))
    );
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 0);
    let mut cpu = Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
    let saved = sim.state.clone();
    let mut preview = sim.clone();
    preview.step().unwrap();
    let mut bad = preview.ledger.last().unwrap().clone();
    let tx = bad
        .transactions
        .iter_mut()
        .find(|t| t.forward.is_some())
        .unwrap();
    tx.effects[0].delta -= 1;
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &bad,
            Backend::Reference,
            settlement::DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(sim.state, saved);
    sim.step().unwrap();
    cpu.step().unwrap();
    assert_eq!(sim.state, cpu.state);
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 4);
    assert_eq!(sim.state.balance(OTHER, scenario::GRAIN), 4);
    sim.run_months(2).unwrap();
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 4);
}

#[test]
fn nonloan_arrears_can_open_a_case_but_future_delivery_alone_cannot() {
    let mut land = land_estate(false);
    land.world.lending.clear();
    land.state.credit = Default::default();
    let mut land = Simulation::new(land.world, land.state, Backend::Reference).unwrap();
    land.run_months(1).unwrap();
    assert_eq!(
        land.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );

    for due in [13, 16] {
        let mut sim = with_accepted_forward(land_estate(false), due);
        sim.world.lending.clear();
        sim.state.credit = Default::default();
        sim.world.agreements.clear();
        sim.world.issuance.clear();
        sim.state.obligations.clear();
        sim = Simulation::new(sim.world, sim.state, Backend::Reference).unwrap();
        sim.run_months(1).unwrap();
        assert_eq!(
            sim.state.credit.recovery.proceedings.contains_key(&1),
            due == 13
        );
    }
}

#[test]
fn forward_performance_remains_partial_and_storage_bounded_while_estate_waits() {
    use economics_compute_smoke::finance::FailureRule;
    let mut sim = with_accepted_forward(land_estate(true), 16);
    sim.world.storage.weights.insert(scenario::GRAIN, 1);
    sim.world.storage.capacities.insert(OTHER, 2);
    sim.world
        .market
        .as_mut()
        .unwrap()
        .cash
        .as_mut()
        .unwrap()
        .protected
        .insert(scenario::GRAIN, 2);
    sim.state.balances.insert((PERSON, scenario::GRAIN), 6);
    sim.run_months(4).unwrap();
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 2);
    assert!(
        sim.state.exchange.forwards[&9000]
            .claim()
            .blocks(FailureRule::BlockNewAdvance)
    );
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert_eq!(sim.state.balance(OTHER, scenario::GRAIN), 2);
    sim.world
        .market
        .as_mut()
        .unwrap()
        .cash
        .as_mut()
        .unwrap()
        .protected
        .clear();
    sim.world.storage.capacities.insert(OTHER, 4);
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 4);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
}

#[test]
fn admission_receipts_filter_nonloan_creditors_and_reject_forged_closure() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let mut sim = with_accepted_forward(land_estate(true), 16);
    // OTHER is a forward creditor only, so observer inclusion cannot come from a loan.
    sim.state.credit.loans.remove(&11);
    sim.world.lending.retain(|a| a.id != 11);
    sim.state.balances.insert((PERSON, scenario::GRAIN), 2);
    let mut plain = sim.clone();
    let mut observer = Observer::new(
        vec![],
        "admission",
        Config {
            settlement: true,
            agents: [OTHER].into_iter().collect(),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 2).unwrap();
    plain.run_months(2).unwrap();
    assert_eq!(sim.state, plain.state);
    assert_eq!(sim.ledger, plain.ledger);
    let log = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert!(
        log.lines()
            .any(|line| line.contains("ClosureDeferred") && line.contains("Forward(9000)"))
    );
    let case = sim.state.credit.recovery.proceedings.get_mut(&1).unwrap();
    case.stage = Stage::Closed;
    case.closed = Some(15);
    for loan in sim.state.credit.loans.values_mut() {
        loan.status = credit::Status::Enforced;
    }
    // Even a coherent-looking closing date cannot discard an accepted future forward.
    assert!(
        credit::validate(&sim.world, &sim.state)
            .unwrap_err()
            .contains("invalid proceeding")
    );
}

fn delivery_terms(
    month: u32,
    action: economics_compute_smoke::delivery_relief::Action,
) -> economics_compute_smoke::delivery_relief::Terms {
    economics_compute_smoke::delivery_relief::Terms {
        id: 1,
        proceeding: 1,
        contract: 9000,
        debtor: PERSON,
        creditor: OTHER,
        month,
        expected_due: 13,
        expected_remaining: 4,
        action,
    }
}

#[test]
fn explicit_delivery_writeoff_closes_estate_without_fictitious_delivery_or_issuance() {
    use economics_compute_smoke::delivery_relief::Action;
    let mut sim = with_accepted_forward(land_estate(true), 13);
    sim.state.balances.insert((PERSON, scenario::GRAIN), 2); // Only real land dues.
    sim.world
        .recovery
        .delivery_relief
        .push(delivery_terms(15, Action::WriteOff { quantity: 4 }));
    sim.run_months(1).unwrap();
    let before = sim.state.balances.clone();
    let mut cpu = Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    cpu.run_months(1).unwrap();
    assert_eq!(sim.state, cpu.state);
    assert_eq!(sim.state.balances, before);
    let c = &sim.state.exchange.forwards[&9000];
    assert_eq!(
        (
            c.goods.quantity,
            c.delivered,
            c.written_off(),
            c.claim().outstanding()
        ),
        (4, 0, 4, 0)
    );
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    assert_eq!(sim.state.balance(OTHER, scenario::GRAIN), 0);
    assert!(
        sim.ledger
            .iter()
            .flat_map(|b| &b.credit)
            .flat_map(|c| &c.recovery)
            .any(|r| matches!(
                r,
                Receipt::DeliveryRelief {
                    applied: true,
                    written_off: 4,
                    remaining: 0,
                    ..
                }
            ))
    );
    let mut resumed =
        Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    resumed.run_months(1).unwrap();
    resumed.run_months(1).unwrap();
    assert_eq!(sim.state, resumed.state);
    assert_eq!(sim.state.exchange.forwards[&9000].relief.len(), 1);
}

#[test]
fn extension_defers_delivery_and_partial_writeoff_leaves_real_residual() {
    use economics_compute_smoke::delivery_relief::Action;
    let mut sim = with_accepted_forward(land_estate(true), 13);
    sim.state.balances.insert((PERSON, scenario::GRAIN), 4);
    sim.world
        .recovery
        .delivery_relief
        .push(delivery_terms(14, Action::Extend { due: 17 }));
    let mut partial = delivery_terms(18, Action::WriteOff { quantity: 1 });
    partial.id = 2;
    partial.expected_due = 17;
    partial.expected_remaining = 2;
    sim.world.recovery.delivery_relief.push(partial);
    sim.run_months(3).unwrap(); // Through month 16, grain exists but is not due.
    let c = &sim.state.exchange.forwards[&9000];
    assert_eq!((c.due, c.effective_due(), c.delivered), (13, 17, 0));
    assert_eq!(
        economics_compute_smoke::forward::pledged(&sim.state, PERSON, scenario::GRAIN),
        4
    );
    sim.run_months(2).unwrap(); // Actual delivery 2 at 17, then forgive only 1 at 18.
    let c = &sim.state.exchange.forwards[&9000];
    assert_eq!(
        (c.delivered, c.written_off(), c.claim().outstanding()),
        (2, 1, 1)
    );
    assert_eq!(sim.state.balance(OTHER, scenario::GRAIN), 2);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert_eq!(
        economics_compute_smoke::forward::pledged(&sim.state, PERSON, scenario::GRAIN),
        1
    );
    sim.state.balances.insert((PERSON, scenario::GRAIN), 1);
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 3);
    assert_eq!(sim.state.balance(OTHER, scenario::GRAIN), 3);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
}

#[test]
fn stale_or_wrong_party_delivery_relief_is_not_applied() {
    use economics_compute_smoke::delivery_relief::Action;
    for wrong_party in [false, true] {
        let mut sim = with_accepted_forward(land_estate(true), 13);
        sim.state
            .balances
            .insert((PERSON, scenario::GRAIN), if wrong_party { 2 } else { 3 });
        let mut terms = delivery_terms(15, Action::WriteOff { quantity: 4 });
        if wrong_party {
            terms.creditor = STATE_AGENT;
        }
        sim.world.recovery.delivery_relief.push(terms);
        sim.run_months(2).unwrap();
        let c = &sim.state.exchange.forwards[&9000];
        assert!(c.relief.is_empty());
        assert_eq!(c.claim().outstanding(), if wrong_party { 4 } else { 3 });
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Active
        );
        assert!(
            sim.ledger
                .iter()
                .flat_map(|b| &b.credit)
                .flat_map(|c| &c.recovery)
                .any(|r| matches!(
                    r,
                    Receipt::DeliveryRelief {
                        applied: false,
                        written_off: 0,
                        ..
                    }
                ))
        );
    }
}

#[test]
fn invalid_or_forged_delivery_relief_cannot_publish_partial_state() {
    use economics_compute_smoke::delivery_relief::Action;
    let mut sim = with_accepted_forward(land_estate(true), 13);
    sim.state.balances.insert((PERSON, scenario::GRAIN), 2);
    for action in [
        Action::WriteOff { quantity: 5 },
        Action::WriteOff { quantity: 0 },
        Action::Extend { due: 14 },
    ] {
        let mut bad = sim.world.clone();
        bad.recovery
            .delivery_relief
            .push(delivery_terms(14, action));
        assert!(Simulation::new(bad, sim.state.clone(), Backend::Reference).is_err());
    }
    sim.world
        .recovery
        .delivery_relief
        .push(delivery_terms(14, Action::WriteOff { quantity: 4 }));
    while sim.state.phase != Phase::Due {
        sim.step().unwrap();
    }
    let opening = sim.state.clone();
    let mut preview = sim.clone();
    preview.step().unwrap();
    let mut forged = preview.ledger.last().unwrap().clone();
    forged
        .credit
        .as_mut()
        .unwrap()
        .forward_changes
        .get_mut(&9000)
        .unwrap()
        .delivered = 4;
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &forged,
            Backend::Reference,
            settlement::DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(sim.state, opening);
    sim.step().unwrap();
    let mut forged_state = sim.state.clone();
    forged_state
        .exchange
        .forwards
        .get_mut(&9000)
        .unwrap()
        .relief[0]
        .terms
        .creditor = STATE_AGENT;
    assert!(Simulation::new(sim.world.clone(), forged_state, Backend::Reference).is_err());
    let mut forged_state = sim.state.clone();
    forged_state
        .exchange
        .forwards
        .get_mut(&9000)
        .unwrap()
        .delivered = 1;
    assert!(Simulation::new(sim.world.clone(), forged_state, Backend::Reference).is_err());
}

#[test]
fn delivery_relief_logs_creditor_outcome_without_changing_execution() {
    use economics_compute_smoke::{
        delivery_relief::Action,
        telemetry::{Config, Observer},
    };
    let mut sim = with_accepted_forward(land_estate(true), 13);
    sim.state.credit.loans.remove(&11);
    sim.world.lending.retain(|a| a.id != 11);
    sim.state.balances.insert((PERSON, scenario::GRAIN), 2);
    sim.world
        .recovery
        .delivery_relief
        .push(delivery_terms(15, Action::WriteOff { quantity: 4 }));
    let mut plain = sim.clone();
    let mut observer = Observer::new(
        vec![],
        "delivery-relief",
        Config {
            settlement: true,
            agents: [OTHER].into_iter().collect(),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 2).unwrap();
    plain.run_months(2).unwrap();
    assert_eq!(sim.state, plain.state);
    assert_eq!(sim.ledger, plain.ledger);
    let log = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert!(log.lines().any(|line| line.contains("DeliveryRelief")
        && line.contains("\"written_off\":4")
        && line.contains("\"applied\":true")));
}

#[test]
fn fulfilled_deliveries_or_inactive_cases_cannot_generate_writeoffs() {
    use economics_compute_smoke::delivery_relief::{Action, Rejection};
    for inactive in [false, true] {
        let mut sim = with_accepted_forward(land_estate(true), 13);
        if inactive {
            // No admissible arrears at the authorized opening month.
            sim.world.lending.clear();
            sim.state.credit = Default::default();
            sim.world.agreements.clear();
            sim.world.issuance.clear();
            sim.world.activities.coin_payments.clear();
            sim.state.obligations.clear();
            sim.state
                .exchange
                .forwards
                .get_mut(&9000)
                .unwrap()
                .delivered = 4;
        } else {
            sim.state.balances.insert((PERSON, scenario::GRAIN), 6);
        }
        sim.world
            .recovery
            .delivery_relief
            .push(delivery_terms(15, Action::WriteOff { quantity: 4 }));
        sim.run_months(2).unwrap();
        assert!(sim.state.exchange.forwards[&9000].relief.is_empty());
        assert_eq!(sim.state.exchange.forwards[&9000].delivered, 4);
        let expected = if inactive {
            Rejection::InactiveProceeding
        } else {
            Rejection::TermsMismatch
        };
        assert!(sim.ledger.iter().flat_map(|b| &b.credit).flat_map(|c| &c.recovery)
            .any(|r| matches!(r, Receipt::DeliveryRelief { applied: false, rejection: Some(reason), .. } if *reason == expected)));
    }
}

#[test]
fn competing_liens_share_only_their_assets_realized_proceeds() {
    use economics_compute_smoke::financial_reporting::Audit;
    for policy in [CollectionPolicy::Stable, CollectionPolicy::Proportional] {
        for junior_rank in [0, 1] {
            for funded in [false, true] {
                let run = |backend, reverse| {
                    let (mut w, mut s) = fixture();
                    proceeding(&mut w, false);
                    w.collection_policy = policy;
                    for (i, loan) in w.lending.iter_mut().enumerate() {
                        loan.terms.grace_months = 0;
                        // Ordinary collection reverses seniority deliberately.
                        loan.priority = if i == 0 { 9 } else { 0 };
                        loan.collateral = Some(credit::Collateral {
                            asset: PLOT,
                            priority: if i == 0 { 0 } else { junior_rank },
                            pledged: true,
                            settlement: credit::CollateralSettlement::AuthorizedLiquidation,
                        });
                    }
                    if reverse {
                        w.lending.reverse();
                    }
                    s.balances
                        .insert((BUYER, TOKEN), if funded { 8 } else { 0 });
                    let mut sim = distressed(w, s, backend);
                    assert_eq!(sim.state.credit.loans.len(), 2);
                    let mut audit =
                        Audit::with_assets(&sim.world, &sim.state, TOKEN, [(PLOT, 10)].into())
                            .unwrap();
                    while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
                        audit.step(&mut sim).unwrap();
                    }
                    // Neither default nor opening an estate transfers title or
                    // reduces secured debt using an appraisal.
                    assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
                    assert!(sim.state.credit.loans.values().all(|l| l.principal == 10));
                    let (mut resumed, mut ra) = (sim.clone(), audit.clone());
                    audit.step(&mut sim).unwrap();
                    let expected = if !funded {
                        (0, 0)
                    } else if junior_rank == 0 && policy == CollectionPolicy::Proportional {
                        (4, 4)
                    } else {
                        (8, 0)
                    };
                    let case = &sim.state.credit.recovery.proceedings[&1];
                    assert_eq!(case.cash, if funded { 8 } else { 0 });
                    assert_eq!(
                        (
                            case.secured.get(&10).copied().unwrap_or(0),
                            case.secured.get(&11).copied().unwrap_or(0)
                        ),
                        expected
                    );
                    assert!(sim.state.credit.loans.values().all(|l| l.principal == 10));
                    assert_eq!(
                        credit::owner(&sim.world, &sim.state, PLOT),
                        Some(if funded { BUYER } else { PERSON })
                    );
                    while sim.state.month < 5 {
                        audit.step(&mut sim).unwrap();
                    }
                    while resumed.state.month < 5 {
                        ra.step(&mut resumed).unwrap();
                    }
                    assert_eq!(
                        (&sim.state, &sim.ledger, &audit),
                        (&resumed.state, &resumed.ledger, &ra)
                    );
                    assert_eq!(
                        (
                            sim.state.balance(STATE_AGENT, TOKEN),
                            sim.state.balance(OTHER, TOKEN)
                        ),
                        expected
                    );
                    assert_eq!(sim.state.credit.loans[&10].principal, 10 - expected.0);
                    assert_eq!(sim.state.credit.loans[&11].principal, 10 - expected.1);
                    assert!(
                        sim.state.credit.loans.values().all(|l| l
                            .collateral
                            .as_ref()
                            .unwrap()
                            .pledged
                            != funded)
                    );
                    for a in &sim.world.agents {
                        let report = audit.book().statements(a.id, 2, 4).unwrap();
                        assert_eq!(report.assets, report.liabilities + report.equity);
                    }
                    (sim.state, sim.ledger, audit)
                };
                let expected = run(Backend::Reference, false);
                assert_eq!(expected, run(Backend::Reference, true));
                assert_eq!(expected, run(Backend::CubeCpu, false));
            }
        }
    }
}

#[test]
fn shared_liens_require_compatible_terms_and_explicit_liquidation_authority() {
    use economics_compute_smoke::financial_reporting::Audit;
    let (mut w, s) = fixture();
    for loan in &mut w.lending {
        loan.terms.grace_months = 0;
        loan.collateral = Some(credit::Collateral {
            asset: PLOT,
            priority: 0,
            pledged: true,
            settlement: credit::CollateralSettlement::AuthorizedLiquidation,
        });
    }
    let run = |backend| {
        let mut sim = distressed(w.clone(), s.clone(), backend);
        let mut audit =
            Audit::with_assets(&sim.world, &sim.state, TOKEN, [(PLOT, 10)].into()).unwrap();
        for _ in 0..4 {
            let month = sim.state.month;
            while sim.state.month == month {
                audit.step(&mut sim).unwrap();
            }
        }
        assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
        assert!(sim.state.credit.recovery.proceedings.is_empty());
        assert!(
            sim.state
                .credit
                .loans
                .values()
                .all(|l| l.principal == 10 && l.collateral.as_ref().unwrap().pledged)
        );
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));

    let mut incompatible = w.clone();
    incompatible.lending[1]
        .collateral
        .as_mut()
        .unwrap()
        .settlement = credit::CollateralSettlement::FixedValue { value: 10 };
    let mut sim = Simulation::new(incompatible, s.clone(), Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert!(sim.state.credit.loans.contains_key(&10));
    assert!(!sim.state.credit.loans.contains_key(&11));
    assert_eq!(sim.state.balance(OTHER, TOKEN), 10);

    let mut guarantee_world = w.clone();
    guarantee_world
        .recovery
        .guarantees
        .push(guarantee(1, 10, 5));
    assert!(
        Simulation::new(guarantee_world, s.clone(), Backend::Reference)
            .unwrap_err()
            .contains("lien-subrogation")
    );

    let mut foreign = w;
    proceeding(&mut foreign, false);
    foreign.lending[1].terms.denomination = scenario::GRAIN;
    assert!(
        Simulation::new(foreign, s, Backend::Reference)
            .unwrap_err()
            .contains("denomination")
    );
}

#[test]
fn a_lien_cannot_take_proceeds_reserved_for_another_asset() {
    use economics_compute_smoke::financial_reporting::Audit;
    let run = |backend| {
        let (mut w, mut s) = fixture();
        proceeding(&mut w, false);
        const SECOND_PLOT: AssetId = 500;
        w.assets.push(Asset {
            id: SECOND_PLOT,
            owner: PERSON,
            kind: 1,
        });
        for (i, loan) in w.lending.iter_mut().enumerate() {
            loan.terms.grace_months = 0;
            loan.collateral = Some(credit::Collateral {
                asset: if i == 0 { PLOT } else { SECOND_PLOT },
                priority: 0,
                pledged: true,
                settlement: credit::CollateralSettlement::AuthorizedLiquidation,
            });
        }
        w.recovery.proceedings[0].assets[0].minimum_price = 4;
        w.recovery.proceedings[0].assets.push(Listing {
            asset: SECOND_PLOT,
            minimum_price: 12,
        });
        w.recovery.bids[0].price = 4;
        w.recovery.bids.push(Bid {
            id: 2,
            proceeding: 1,
            buyer: BUYER,
            asset: SECOND_PLOT,
            month: 3,
            price: 12,
        });
        s.balances.insert((BUYER, TOKEN), 16);
        let mut sim = distressed(w, s, backend);
        let mut audit = Audit::with_assets(
            &sim.world,
            &sim.state,
            TOKEN,
            [(PLOT, 10), (SECOND_PLOT, 10)].into(),
        )
        .unwrap();
        while (sim.state.month, sim.state.phase) != (3, Phase::Productive) {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].secured,
            [(10, 4), (11, 10)].into()
        );
        assert_eq!(sim.state.balance(ESTATE, TOKEN), 16);
        let (mut resumed, mut ra) = (sim.clone(), audit.clone());
        while sim.state.month < 5 {
            audit.step(&mut sim).unwrap();
        }
        while resumed.state.month < 5 {
            ra.step(&mut resumed).unwrap();
        }
        assert_eq!(
            (&sim.state, &sim.ledger, &audit),
            (&resumed.state, &resumed.ledger, &ra)
        );
        assert_eq!(
            (
                sim.state.balance(STATE_AGENT, TOKEN),
                sim.state.balance(OTHER, TOKEN)
            ),
            (6, 10)
        );
        assert_eq!(sim.state.credit.loans[&10].principal, 4);
        assert_eq!(sim.state.credit.loans[&11].principal, 0);
        assert_eq!(sim.state.balance(BUYER, TOKEN), 0);
        assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
        for a in &sim.world.agents {
            let report = audit.book().statements(a.id, 2, 4).unwrap();
            assert_eq!(report.assets, report.liabilities + report.equity);
        }
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn household_liens_preserve_member_claims_through_wind_down_and_discharge() {
    use economics_compute_smoke::{
        financial_reporting::Audit,
        household_governance::Governance,
        households::{self, Agreement, dissolution},
    };
    const HOME: AgentId = 10000;
    for discharge in [false, true] {
        let run = |backend| {
            let (mut w, mut s) = fixture();
            let mut member = scenario::baseline().0.participants[0].clone();
            member.needs.clear();
            member.capacity.quantity = 0;
            w.participants.push(member);
            let mut governance = Governance::contributed(PERSON);
            governance.constitution.allow_dissolution = true;
            households::form(
                &mut w,
                &s,
                Agreement {
                    id: 1,
                    agent: HOME,
                    adults: vec![PERSON],
                    governance,
                    formed: s.month,
                    dwelling_process: None,
                    admission: None,
                    membership: vec![],
                    asset_sales: vec![],
                    equipment_retirements: vec![],
                    support: vec![],
                },
            )
            .unwrap();
            proceeding(&mut w, discharge);
            w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = HOME;
            w.recovery.proceedings[0].debtor = HOME;
            w.lending[1].terms.creditor = PERSON;
            s.balances.insert((PERSON, TOKEN), 17);
            for loan in &mut w.lending {
                loan.debtor = HOME;
                loan.terms.grace_months = 0;
                loan.collateral = Some(credit::Collateral {
                    asset: PLOT,
                    priority: 0,
                    pledged: true,
                    settlement: credit::CollateralSettlement::AuthorizedLiquidation,
                });
            }
            let mut sim = Simulation::new(w, s, backend).unwrap();
            sim.run_months(1).unwrap();
            assert_eq!(sim.state.balance(PERSON, TOKEN), 7);
            assert_eq!(sim.state.balance(HOME, TOKEN), 20);
            // Controlled pre-book household loss; the member's funds and claim
            // remain separate from the household's estate.
            sim.state.balances.insert((HOME, TOKEN), 0);
            dissolution::request(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
            let mut audit =
                Audit::with_assets(&sim.world, &sim.state, TOKEN, [(PLOT, 10)].into()).unwrap();
            while sim.state.month < 3 {
                audit.step(&mut sim).unwrap();
            }
            assert!(dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).is_err());
            let (mut resumed, mut ra) = (sim.clone(), audit.clone());
            while sim.state.month < 5 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month < 5 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &ra)
            );
            assert_eq!(sim.state.balance(PERSON, TOKEN), 11);
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 4);
            assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            assert_eq!(
                sim.state.credit.loans[&11].principal,
                if discharge { 0 } else { 6 }
            );
            assert_eq!(
                dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).is_ok(),
                discharge
            );
            assert_eq!(
                dissolution::finish(&mut resumed.world, &resumed.state, HOME, PERSON).is_ok(),
                discharge
            );
            while sim.state.month < 6 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month < 6 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &ra)
            );
            assert_eq!(
                households::membership::current(&sim.world.households[0]),
                if discharge { vec![] } else { vec![PERSON] }
            );
            for a in &sim.world.agents {
                let report = audit.book().statements(a.id, 2, 5).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn guarantee_inherits_liens_and_realized_proceeds_without_same_month_recourse() {
    guaranteed_liens(false);
}

#[test]
fn secured_relief_does_not_reclaim_proceeds_transferred_to_a_guarantor() {
    guaranteed_liens(true);
}

fn guaranteed_liens(relief: bool) {
    use economics_compute_smoke::{financial_reporting::Audit, recovery::RecourseSecurity};
    for from in [3, 4] {
        for cap in [6, 10] {
            if relief && (from != 4 || cap != 6) {
                continue;
            }
            for policy in [CollectionPolicy::Stable, CollectionPolicy::Proportional] {
                let run = |backend| {
                    let (mut w, s) = fixture();
                    w.lending.truncate(1);
                    proceeding(&mut w, false);
                    w.collection_policy = policy;
                    w.lending[0].collateral.as_mut().unwrap().settlement =
                        credit::CollateralSettlement::AuthorizedLiquidation;
                    let mut g = guarantee(1, 10, cap);
                    g.guarantor = OTHER;
                    g.from = from;
                    g.security = RecourseSecurity::InheritLiquidationLien;
                    w.recovery.guarantees.push(g);
                    if relief {
                        w.recovery.claim_relief.push(
                            economics_compute_smoke::claim_relief::Terms {
                                id: 1,
                                proceeding: 1,
                                contract: economics_compute_smoke::finance::ContractId::Loan(10),
                                original_due: 2,
                                debtor: PERSON,
                                creditor: STATE_AGENT,
                                month: 4,
                                expected_due: 2,
                                expected_remaining: 4,
                                action: economics_compute_smoke::claim_relief::Action::WriteOff {
                                    quantity: 1,
                                },
                            },
                        );
                    }
                    let mut sim = distressed(w, s, backend);
                    let mut books =
                        Audit::with_assets(&sim.world, &sim.state, TOKEN, [(PLOT, 10)].into())
                            .unwrap();
                    while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
                        books.step(&mut sim).unwrap();
                    }
                    if from == 3 {
                        assert_eq!(sim.state.credit.loans[&10].principal, 10 - cap);
                        assert_eq!(sim.state.credit.loans[&101].principal, cap);
                        assert!(
                            sim.state.credit.loans[&101]
                                .collateral
                                .as_ref()
                                .unwrap()
                                .pledged
                        );
                    }
                    let (mut resumed, mut rb) = (sim.clone(), books.clone());
                    books.step(&mut sim).unwrap();
                    let allocations = sim.state.credit.recovery.proceedings[&1].secured.clone();
                    assert_eq!(allocations.values().sum::<i32>(), 8);
                    if from == 4 {
                        assert_eq!(allocations[&10], 8);
                    }
                    while sim.state.month < 5 {
                        books.step(&mut sim).unwrap();
                    }
                    if from == 4 {
                        let reserve = cap.min(8);
                        assert_eq!(sim.state.credit.recovery.proceedings[&1].cash, reserve);
                        assert_eq!(
                            sim.state.credit.recovery.proceedings[&1].secured[&101],
                            reserve
                        );
                        assert_eq!(sim.state.credit.loans[&101].principal, cap);
                        assert_eq!(sim.state.balance(OTHER, TOKEN), 10 - cap);
                        assert_eq!(
                            sim.state.credit.recovery.proceedings[&1].stage,
                            Stage::Active
                        );
                    }
                    while sim.state.month < 6 {
                        books.step(&mut sim).unwrap();
                    }
                    while resumed.state.month < 6 {
                        rb.step(&mut resumed).unwrap();
                    }
                    assert_eq!(
                        (&sim.state, &sim.ledger, &books),
                        (&resumed.state, &resumed.ledger, &rb)
                    );
                    let original_recovery = if from == 4 {
                        8 - cap.min(8)
                    } else {
                        allocations.get(&10).copied().unwrap_or(0)
                    };
                    let recourse_recovery = 8 - original_recovery;
                    assert_eq!(
                        sim.state.balance(STATE_AGENT, TOKEN),
                        cap + original_recovery
                    );
                    assert_eq!(
                        sim.state.balance(OTHER, TOKEN),
                        10 - cap + recourse_recovery
                    );
                    assert_eq!(
                        sim.state.credit.loans[&10].principal,
                        10 - cap - original_recovery - i32::from(relief)
                    );
                    assert_eq!(
                        sim.state.credit.loans[&101].principal,
                        cap - recourse_recovery
                    );
                    assert_eq!(sim.state.credit.recovery.proceedings[&1].cash, 0);
                    assert!(
                        !sim.state.credit.loans[&101]
                            .collateral
                            .as_ref()
                            .unwrap()
                            .pledged
                    );
                    // A forged checkpoint cannot drop the agreed inherited security.
                    let mut bad = sim.state.clone();
                    bad.credit.loans.get_mut(&101).unwrap().collateral = None;
                    assert!(Simulation::new(sim.world.clone(), bad, Backend::Reference).is_err());
                    (sim.state, sim.ledger, books)
                };
                assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
            }
        }
    }
}

#[test]
fn household_guarantor_keeps_member_money_separate_and_recovers_before_wind_down() {
    use economics_compute_smoke::{
        financial_reporting::Audit,
        household_governance::Governance,
        households::{self, Agreement, dissolution},
        recovery::RecourseSecurity,
    };
    const HOME: AgentId = 10000;
    for discharge in [false, true] {
        let run = |backend| {
            let (mut w, mut s) = fixture();
            w.lending.truncate(1);
            proceeding(&mut w, discharge);
            w.collection_policy = CollectionPolicy::Stable;
            w.lending[0].collateral.as_mut().unwrap().settlement =
                credit::CollateralSettlement::AuthorizedLiquidation;
            let mut member = scenario::baseline().0.participants[0].clone();
            member.agent = OTHER;
            member.needs.clear();
            member.capacity.quantity = 0;
            w.participants.push(member);
            let mut governance = Governance::contributed(OTHER);
            governance.constitution.allow_dissolution = true;
            households::form(
                &mut w,
                &s,
                Agreement {
                    id: 1,
                    agent: HOME,
                    adults: vec![OTHER],
                    governance,
                    formed: 1,
                    dwelling_process: None,
                    admission: None,
                    membership: vec![],
                    asset_sales: vec![],
                    equipment_retirements: vec![],
                    support: vec![],
                },
            )
            .unwrap();
            s.balances.insert((HOME, TOKEN), 10);
            let mut g = guarantee(1, 10, 6);
            g.guarantor = HOME;
            g.from = 3;
            g.through = 3;
            g.security = RecourseSecurity::InheritLiquidationLien;
            w.recovery.guarantees.push(g);
            let mut sim = distressed(w, s, backend);
            let mut books =
                Audit::with_assets(&sim.world, &sim.state, TOKEN, [(PLOT, 10)].into()).unwrap();
            while sim.state.month < 4 {
                books.step(&mut sim).unwrap();
            }
            assert_eq!(sim.state.balance(OTHER, TOKEN), 10);
            assert_eq!(sim.state.balance(HOME, TOKEN), 4);
            assert_eq!(sim.state.credit.loans[&101].creditor, HOME);
            dissolution::request(&mut sim.world, &sim.state, HOME, OTHER).unwrap();
            assert!(dissolution::finish(&mut sim.world, &sim.state, HOME, OTHER).is_err());
            let (mut resumed, mut rb) = (sim.clone(), books.clone());
            while sim.state.month < 5 {
                books.step(&mut sim).unwrap();
            }
            while resumed.state.month < 5 {
                rb.step(&mut resumed).unwrap();
            }
            assert_eq!(sim.state.balance(HOME, TOKEN), 8);
            assert_eq!(sim.state.balance(OTHER, TOKEN), 10);
            assert_eq!(
                sim.state.credit.loans[&101].principal,
                if discharge { 0 } else { 2 }
            );
            // Residual assets are distributed at the next household Open before exit.
            while sim.state.month < 6 {
                books.step(&mut sim).unwrap();
            }
            while resumed.state.month < 6 {
                rb.step(&mut resumed).unwrap();
            }
            assert_eq!(
                dissolution::finish(&mut sim.world, &sim.state, HOME, OTHER).is_ok(),
                discharge
            );
            assert_eq!(
                dissolution::finish(&mut resumed.world, &resumed.state, HOME, OTHER).is_ok(),
                discharge
            );
            while sim.state.month < 7 {
                books.step(&mut sim).unwrap();
            }
            while resumed.state.month < 7 {
                rb.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &books),
                (&resumed.state, &resumed.ledger, &rb)
            );
            assert_eq!(
                sim.state.balance(HOME, TOKEN),
                if discharge { 0 } else { 8 }
            );
            assert_eq!(
                sim.state.balance(OTHER, TOKEN),
                if discharge { 18 } else { 10 }
            );
            assert_eq!(
                households::membership::current(&sim.world.households[0]).is_empty(),
                discharge
            );
            (sim.state, sim.ledger, books)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn partial_secured_relief_preserves_lien_priority_before_and_after_sale() {
    partial_secured_relief(false);
}

#[test]
fn household_secured_relief_keeps_junior_member_claim_material_after_liquidation() {
    partial_secured_relief(true);
}

fn partial_secured_relief(household: bool) {
    use economics_compute_smoke::{
        accounting::Account,
        claim_relief::{Action, Terms},
        finance::ContractId,
        financial_reporting::Audit,
    };
    for (month, quantity, expected, senior_paid, junior_paid, senior_remaining) in [
        (3, 7, 10, 3, 5, 0),
        (4, 7, 10, 3, 5, 0),
        (3, 10, 10, 0, 8, 0),
        (4, 10, 10, 0, 8, 0),
        (3, 7, 11, 8, 0, 2),
        (5, 1, 2, 8, 0, 1),
        (5, 2, 2, 8, 0, 0),
    ] {
        let (mut w, mut s) = fixture();
        const HOME: AgentId = 800;
        let debtor = if household { HOME } else { PERSON };
        let junior = if household { PERSON } else { OTHER };
        proceeding(&mut w, false);
        w.recovery.proceedings[0].earliest_close = 6;
        if household {
            use economics_compute_smoke::{
                household_governance::Governance,
                households::{self, Agreement},
            };
            let mut person = scenario::baseline().0.participants.remove(0);
            person.needs.clear();
            person.capacity.quantity = 0;
            w.participants.push(person);
            let mut governance = Governance::contributed(PERSON);
            governance.constitution.allow_dissolution = true;
            households::form(
                &mut w,
                &s,
                Agreement {
                    id: 1,
                    agent: HOME,
                    adults: vec![PERSON],
                    governance,
                    formed: 1,
                    dwelling_process: None,
                    admission: None,
                    membership: vec![],
                    asset_sales: vec![],
                    equipment_retirements: vec![],
                    support: vec![],
                },
            )
            .unwrap();
            w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = HOME;
            w.recovery.proceedings[0].debtor = HOME;
            for loan in &mut w.lending {
                loan.debtor = HOME;
            }
            w.lending[1].terms.creditor = PERSON;
            s.balances.insert((PERSON, TOKEN), 10);
            s.balances.insert((OTHER, TOKEN), 0);
        }
        for (rank, loan) in w.lending.iter_mut().enumerate() {
            loan.collateral = Some(credit::Collateral {
                asset: PLOT,
                priority: rank as u32,
                pledged: true,
                settlement: credit::CollateralSettlement::AuthorizedLiquidation,
            });
        }
        // Unsecured collection priority must not capture released lien proceeds.
        w.lending[1].priority = 10;
        let mut unsecured = advance(12, BUYER);
        unsecured.debtor = debtor;
        w.lending.push(unsecured);
        s.balances.insert((BUYER, TOKEN), 18);
        w.recovery.claim_relief.push(Terms {
            id: 1,
            proceeding: 1,
            contract: ContractId::Loan(10),
            original_due: 2,
            debtor,
            creditor: STATE_AGENT,
            month,
            expected_due: 2,
            expected_remaining: expected,
            action: Action::WriteOff { quantity },
        });
        let accepted = expected != 11;
        let before_sale = accepted && month == 3;
        let run = |backend| {
            let mut sim = distressed(w.clone(), s.clone(), backend);
            if household {
                sim.state.balances.insert((HOME, TOKEN), 0);
                economics_compute_smoke::households::dissolution::request(
                    &mut sim.world,
                    &sim.state,
                    HOME,
                    PERSON,
                )
                .unwrap();
            }
            let mut a = Audit::with_opening(
                &sim.world,
                &sim.state,
                TOKEN,
                economics_compute_smoke::financial_reporting::Opening {
                    assets: sim
                        .world
                        .assets
                        .iter()
                        .map(|asset| (asset.id, if asset.id == PLOT { 8 } else { 0 }))
                        .collect(),
                    ..Default::default()
                },
            )
            .unwrap();
            while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
                a.step(&mut sim).unwrap();
            }
            let loan = &sim.state.credit.loans[&10];
            assert_eq!(loan.principal, if before_sale { 10 - quantity } else { 10 });
            assert_eq!(
                loan.collateral.as_ref().unwrap().pledged,
                !(before_sale && quantity == 10)
            );
            assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(debtor));
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
            let (saved, mut ra) = (sim.clone(), a.clone());
            a.step(&mut sim).unwrap();
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1]
                    .secured
                    .get(&10)
                    .copied()
                    .unwrap_or(0),
                if before_sale { 10 - quantity } else { 8 }
            );
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1]
                    .secured
                    .get(&11)
                    .copied()
                    .unwrap_or(0),
                if before_sale { quantity - 2 } else { 0 }
            );
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 8);
            while sim.state.month <= 6 {
                a.step(&mut sim).unwrap();
            }
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), senior_paid);
            assert_eq!(sim.state.balance(junior, TOKEN), junior_paid);
            assert_eq!(sim.state.credit.loans[&10].principal, senior_remaining);
            assert_eq!(sim.state.credit.loans[&11].principal, 10 - junior_paid);
            assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(BUYER));
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
            assert_eq!(sim.state.balance(BUYER, TOKEN), 0);
            assert_eq!(sim.state.credit.loans[&12].principal, 10);
            assert_eq!(
                a.book()
                    .balances()
                    .get(&(STATE_AGENT, Account::CreditLoss))
                    .copied()
                    .unwrap_or(0),
                if accepted { i128::from(quantity) } else { 0 }
            );
            assert_eq!(
                a.book()
                    .balances()
                    .get(&(debtor, Account::DebtRelief))
                    .copied()
                    .unwrap_or(0),
                if accepted { -i128::from(quantity) } else { 0 }
            );
            if accepted {
                let r = &sim.state.credit.recovery.loan_writeoffs[&10][0];
                let retained = if quantity < expected {
                    Some(PLOT)
                } else {
                    None
                };
                assert_eq!(r.retained_collateral, retained);
                let mut bad = sim.state.clone();
                bad.credit.recovery.loan_writeoffs.get_mut(&10).unwrap()[0].retained_collateral =
                    if retained.is_some() { None } else { Some(PLOT) };
                assert!(Simulation::new(w.clone(), bad, backend).is_err());
            }
            if household {
                use economics_compute_smoke::households::dissolution as d;
                assert_eq!(
                    a.book().balances()[&(PERSON, Account::LoanReceivable(11))],
                    i128::from(10 - junior_paid)
                );
                assert_eq!(
                    a.book().balances()[&(HOME, Account::LoanPayable(11))],
                    i128::from(junior_paid - 10)
                );
                assert_eq!(sim.state.balance(HOME, TOKEN), 0);
                assert!(d::blockers(&sim.world, &sim.state, HOME).contains(&d::Blocker::Loan));
                assert!(d::finish(&mut sim.world, &sim.state, HOME, PERSON).is_err());
            }
            let mut resumed = Simulation::new(saved.world, saved.state, backend).unwrap();
            while resumed.state.month <= 6 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!((&sim.state, &a), (&resumed.state, &ra));
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn chained_secured_guarantees_transfer_the_same_lien_and_hold_new_recourse_until_next_month() {
    use economics_compute_smoke::{
        financial_reporting::Audit,
        recovery::{GuaranteedClaim, RecourseSecurity},
    };
    const LAST: AgentId = 801;
    for from in [3, 4] {
        for policy in [CollectionPolicy::Stable, CollectionPolicy::Proportional] {
            let (mut w, mut s) = fixture();
            w.lending.truncate(1);
            proceeding(&mut w, false);
            w.collection_policy = policy;
            w.lending[0].collateral.as_mut().unwrap().settlement =
                credit::CollateralSettlement::AuthorizedLiquidation;
            w.agents.push(Agent {
                id: LAST,
                name: "second guarantor".into(),
            });
            s.balances.insert((LAST, TOKEN), 10);
            let mut first = guarantee(1, 10, 6);
            first.guarantor = OTHER;
            first.from = from;
            first.through = from;
            first.security = RecourseSecurity::InheritLiquidationLien;
            let mut second = guarantee(2, 101, 6);
            second.guarantor = LAST;
            second.from = from + 1;
            second.through = from + 1;
            second.security = RecourseSecurity::InheritLiquidationLien;
            w.recovery.guarantees = vec![first, second];
            let mut invalid = w.clone();
            invalid.recovery.guarantees[1].security = RecourseSecurity::Unsecured;
            assert!(
                Simulation::new(invalid, s.clone(), Backend::Reference)
                    .unwrap_err()
                    .contains("explicit lien inheritance")
            );
            let mut cyclic = w.clone();
            cyclic.recovery.guarantees[0].claim = GuaranteedClaim::Loan(102);
            assert!(Simulation::new(cyclic, s.clone(), Backend::Reference).is_err());
            let run = |backend, reverse| {
                let mut world = w.clone();
                if reverse {
                    world.recovery.guarantees.reverse();
                }
                let mut sim = distressed(world, s.clone(), backend);
                let mut audit =
                    Audit::with_assets(&sim.world, &sim.state, TOKEN, [(PLOT, 10)].into()).unwrap();
                while sim.state.month <= from {
                    audit.step(&mut sim).unwrap();
                }
                assert!(!sim.state.credit.loans.contains_key(&102));
                let reserved = sim.state.credit.recovery.proceedings[&1].secured[&101];
                let original_paid = 8 - reserved;
                let (saved, mut resumed_audit) = (sim.clone(), audit.clone());
                while sim.state.month <= from + 1 {
                    audit.step(&mut sim).unwrap();
                }
                assert_eq!(sim.state.credit.loans[&101].principal, 0);
                assert_eq!(sim.state.credit.loans[&102].principal, 6);
                assert_eq!(
                    sim.state.credit.recovery.proceedings[&1].secured[&102],
                    reserved
                );
                assert_eq!(sim.state.balance(LAST, TOKEN), 4);
                assert_eq!(sim.state.balance(OTHER, TOKEN), 10);
                while sim.state.month <= from + 2 {
                    audit.step(&mut sim).unwrap();
                }
                assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 6 + original_paid);
                assert_eq!(sim.state.balance(LAST, TOKEN), 4 + reserved);
                assert_eq!(sim.state.credit.loans[&102].principal, 6 - reserved);
                assert_eq!(sim.state.credit.loans[&10].principal, 4 - original_paid);
                assert_eq!(
                    sim.state.credit.recovery.proceedings[&1].stage,
                    Stage::Closed
                );
                assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(BUYER));
                let mut bad = sim.state.clone();
                bad.credit
                    .loans
                    .get_mut(&102)
                    .unwrap()
                    .collateral
                    .as_mut()
                    .unwrap()
                    .priority += 1;
                assert!(Simulation::new(sim.world.clone(), bad, backend).is_err());
                let mut resumed = Simulation::new(saved.world, saved.state, backend).unwrap();
                while resumed.state.month <= from + 2 {
                    resumed_audit.step(&mut resumed).unwrap();
                }
                assert_eq!((&sim.state, &audit), (&resumed.state, &resumed_audit));
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference, false), run(Backend::CubeCpu, true));
        }
    }
}
