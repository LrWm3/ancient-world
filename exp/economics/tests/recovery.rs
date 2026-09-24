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
