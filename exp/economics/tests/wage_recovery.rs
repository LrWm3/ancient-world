use economics_compute_smoke::{
    accounting::Account,
    compute::Backend,
    employment::{ArrearsPolicy, Reason, Terms},
    finance::ContractId,
    financial_reporting::{Audit, Opening},
    minting::{self, COIN, HOURS, ISSUER, WORKER},
    model::*,
    recovery::{ProceedingTerms, Receipt, Stage},
    simulation::Simulation,
};
const ESTATE: AgentId = 999;
fn fixture() -> (World, State) {
    let (mut w, mut s) = minting::scenario("normal").unwrap();
    w.minting = None;
    w.transaction_policy = None;
    w.scheduled_starts.clear();
    w.capacity_overrides.clear();
    w.priority = Priority::ContinuingFirst;
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == WORKER { 2 } else { 0 };
    }
    s.balances.clear();
    w.agents.push(Agent {
        id: ESTATE,
        name: "custody".into(),
    });
    w.employment.push(Terms {
        id: 1,
        employer: ISSUER,
        worker: WORKER,
        from: 1,
        through: 6,
        capacity: Amount::new(HOURS, 2),
        wage_per_unit: Amount::new(COIN, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: ISSUER,
        authority: ISSUER,
        estate: ESTATE,
        denomination: COIN,
        opening_month: 3,
        earliest_close: 3,
        assets: vec![],
        discharge_deficiency: true,
    });
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(w, s, COIN, Opening::default()).unwrap()
}
fn through(a: &mut Audit, s: &mut Simulation, month: u32) {
    while s.state.month <= month {
        a.step(s)
            .unwrap_or_else(|e| panic!("{e}: {} {:?}", s.state.month, s.state.phase));
    }
}
fn receipts(sim: &Simulation) -> impl Iterator<Item = &Receipt> {
    sim.ledger
        .iter()
        .filter_map(|b| b.credit.as_ref())
        .flat_map(|b| &b.recovery)
}
#[test]
fn wages_alone_open_recovery_stay_collection_and_block_discharge() {
    let (w, s) = fixture();
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 4);
    reference.run_months(4).unwrap();
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert_eq!(sim.state.employment.earned.len(), 1);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
    assert!(receipts(&sim).any(|r| matches!(r, Receipt::Admitted {claims, ..} if claims.iter().any(|c| c.contract == ContractId::Wages(1) && c.remaining.quantity == 4))));
    assert!(receipts(&sim).any(|r| matches!(r, Receipt::ClosureDeferred {claims, ..} if claims.iter().any(|c| c.contract == ContractId::Wages(1)))));
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.employment.as_ref())
            .flat_map(|b| &b.receipts)
            .any(|r| r.reason == Reason::CollectionStayed)
    );
    let balances = a.book().balances();
    assert_eq!(balances[&(WORKER, Account::WagesReceivable(1, 1))], 4);
    assert_eq!(balances[&(ISSUER, Account::WagesPayable(1, 1))], -4);
}
#[test]
fn unearned_or_not_yet_overdue_payroll_cannot_open_proceeding() {
    for month in [1, 2] {
        let (mut w, s) = fixture();
        w.recovery.proceedings[0].opening_month = month;
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut sim, 3);
        assert!(sim.state.credit.recovery.proceedings.is_empty());
        assert!(receipts(&sim).any(|r| matches!(r, Receipt::OpeningRejected { .. })));
    }
}

fn delayed_income(w: &mut World, s: &mut State, quantity: i32) {
    use economics_compute_smoke::minting::SUPPLIER;
    w.employment[0].through = 1;
    s.balances.insert((SUPPLIER, COIN), quantity);
    w.capacity_overrides.insert((2, ISSUER), 1);
    w.employment.push(Terms {
        id: 9,
        employer: SUPPLIER,
        worker: ISSUER,
        from: 2,
        through: 2,
        capacity: Amount::new(HOURS, 1),
        wage_per_unit: Amount::new(COIN, quantity),
        on_arrears: ArrearsPolicy::Continue,
        rank: 0,
    });
}
#[test]
fn later_cash_enters_custody_then_pays_real_wages_and_closes() {
    let (mut w, mut s) = fixture();
    delayed_income(&mut w, &mut s, 4);
    let mut a = audit(&w, &s);
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut b = a.clone();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 3);
    assert_eq!(sim.state.balance(ESTATE, COIN), 4);
    assert_eq!(sim.state.balance(WORKER, COIN), 0);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
    let mut resumed = sim.clone();
    let mut c = a.clone();
    through(&mut a, &mut sim, 5);
    through(&mut b, &mut reference, 5);
    through(&mut c, &mut resumed, 5);
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(sim.state, resumed.state);
    assert_eq!(a.book().balances(), c.book().balances());
    assert_eq!(a.book().balances(), b.book().balances());
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
    assert_eq!(sim.state.balance(WORKER, COIN), 4);
    assert_eq!(sim.state.balance(ESTATE, COIN), 0);
    assert_eq!(
        receipts(&sim)
            .filter_map(|r| if let Receipt::WagesDistributed { paid, .. } = r {
                Some(*paid)
            } else {
                None
            })
            .sum::<i32>(),
        4
    );
}
#[test]
fn wages_and_loans_share_custody_by_explicit_priority_and_proportion() {
    use economics_compute_smoke::{
        credit::{Advance, LoanOffer},
        minting::{SUPPLIER, VENUE},
    };
    for (wage_rank, loan_rank, expected_wages, expected_loan) in
        [(0, 0, 3, 3), (0, 1, 4, 2), (1, 0, 2, 4)]
    {
        let (mut w, mut s) = fixture();
        delayed_income(&mut w, &mut s, 6);
        // The first loan funds the first worker. A second worker earns unpaid
        // wages; the later earned income cannot be reused at the same Close.
        w.participants
            .iter_mut()
            .find(|p| p.agent == SUPPLIER)
            .unwrap()
            .capacity
            .quantity = 2;
        let mut second = w.employment[0].clone();
        second.id = 2;
        second.worker = SUPPLIER;
        w.employment.push(second);
        w.lending.push(Advance {
            id: 20,
            debtor: ISSUER,
            terms: LoanOffer {
                creditor: VENUE,
                denomination: COIN,
                max_principal: 4,
                monthly_rate_bps: 0,
                term_months: 1,
                grace_months: 10,
            },
            principal: 4,
            month: 1,
            collateral: None,
            priority: loan_rank,
        });
        s.balances.insert((VENUE, COIN), 4);
        w.claim_priorities.insert(ContractId::Wages(2), wage_rank);
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
        w.employment.reverse();
        let mut reversed = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut sim, 4);
        reversed.run_months(4).unwrap();
        assert_eq!(sim.state, reversed.state);
        assert_eq!(sim.ledger, reversed.ledger);
        assert_eq!(
            sim.state.employment.earned[&(2, 1)].claim.settled,
            expected_wages
        );
        assert_eq!(sim.state.balance(VENUE, COIN), expected_loan);
        assert_eq!(sim.state.balance(ESTATE, COIN), 0);
        if expected_wages < 4 {
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Active
            );
            assert_eq!(sim.state.credit.loans[&20].principal, 4 - expected_loan);
        }
    }
}

#[test]
fn native_physical_wages_are_not_converted_to_estate_cash_or_forgiven() {
    use economics_compute_smoke::minting::FIREWOOD;
    let (mut w, mut s) = fixture();
    w.employment[0].through = 1;
    w.employment[0].wage_per_unit.resource = FIREWOOD;
    s.balances.insert((ISSUER, COIN), 4);
    let mut a = Audit::with_opening(
        &w,
        &s,
        COIN,
        Opening {
            exchange_values: [(FIREWOOD, 1)].into(),
            ..Default::default()
        },
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 4);
    assert_eq!(sim.state.balance(ISSUER, COIN), 4);
    assert_eq!(sim.state.balance(ESTATE, COIN), 0);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert!(!receipts(&sim).any(|r| matches!(r, Receipt::WagesDistributed{paid, ..} if *paid > 0)));
}

#[test]
fn tampered_estate_wage_payment_and_replay_are_atomic() {
    use economics_compute_smoke::settlement::{DEFAULT_EFFECT_LIMIT, commit};
    let (mut w, mut s) = fixture();
    delayed_income(&mut w, &mut s, 4);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.month < 4 || sim.state.phase != Phase::Due {
        a.step(&mut sim).unwrap();
    }
    let before = sim.state.clone();
    let before_audit = a.clone();
    a.step(&mut sim).unwrap();
    let valid = sim.ledger.last().unwrap().clone();
    for mode in 0..3 {
        let mut bad = valid.clone();
        let c = bad.credit.as_mut().unwrap();
        match mode {
            0 => c.employment = None,
            1 => {
                c.employment
                    .as_mut()
                    .unwrap()
                    .earned
                    .get_mut(&(1, 1))
                    .unwrap()
                    .claim
                    .settled -= 1
            }
            _ => {
                if let Some(Receipt::WagesDistributed { paid, .. }) = c
                    .recovery
                    .iter_mut()
                    .find(|r| matches!(r, Receipt::WagesDistributed{paid, ..} if *paid > 0))
                {
                    *paid += 1;
                }
            }
        }
        let mut candidate = before.clone();
        assert!(
            commit(
                &sim.world,
                &mut candidate,
                &bad,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(candidate, before);
        let mut reporting = before_audit.clone();
        assert!(
            reporting
                .record(&sim.world, &before, &bad, &sim.state)
                .is_err()
        );
        assert_eq!(reporting.book().balances(), before_audit.book().balances());
    }
    let mut candidate = sim.state.clone();
    assert!(
        commit(
            &sim.world,
            &mut candidate,
            &valid,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(candidate, sim.state);
}

#[test]
fn liquidation_wages_pool_to_workers_household_once_and_are_observable() {
    use economics_compute_smoke::{
        households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
        opportunities::{Action, PERSON_TYPE},
        recovery::{Bid, Listing},
        scenario::{LABOR, PERSON, TOKEN},
        telemetry::{Config, Observer},
    };
    const EMPLOYER: AgentId = 89;
    const BUYER: AgentId = 92;
    const ASSET: AssetId = 900;
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    s.balances.clear();
    s.balances.insert((BUYER, TOKEN), 4);
    w.activities.orders.clear();
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == PERSON { 5 } else { 0 };
    }
    w.transaction_policy.as_mut().unwrap().permissions.extend([
        (PERSON_TYPE, Action::CapacityTrade),
        (PERSON_TYPE, Action::AssetTrade),
    ]);
    w.agents.push(Agent {
        id: ESTATE,
        name: "custody".into(),
    });
    w.assets.push(Asset {
        id: ASSET,
        owner: EMPLOYER,
        kind: 1,
    });
    w.employment.push(Terms {
        id: 1,
        employer: EMPLOYER,
        worker: PERSON,
        from: 1,
        through: 1,
        capacity: Amount::new(LABOR, 2),
        wage_per_unit: Amount::new(TOKEN, 2),
        on_arrears: ArrearsPolicy::Continue,
        rank: 0,
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: EMPLOYER,
        authority: w.transaction_policy.as_ref().unwrap().authority,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 3,
        assets: vec![Listing {
            asset: ASSET,
            minimum_price: 4,
        }],
        discharge_deficiency: true,
    });
    w.recovery.bids.push(Bid {
        id: 1,
        proceeding: 1,
        buyer: BUYER,
        asset: ASSET,
        month: 3,
        price: 4,
    });
    let mut a = Audit::with_opening(
        &w,
        &s,
        TOKEN,
        Opening {
            assets: w.assets.iter().map(|asset| (asset.id, 0)).collect(),
            ..Default::default()
        },
    )
    .unwrap();
    let mut b = a.clone();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut observer = Observer::new(
        Vec::new(),
        "estate-wage",
        Config {
            settlement: true,
            ..Default::default()
        },
    )
    .unwrap();
    while sim.state.month <= 5 {
        observer.step_audited(&mut sim, &mut a).unwrap();
    }
    through(&mut b, &mut reference, 5);
    assert_eq!(sim.state, reference.state);
    assert_eq!(a.book().balances(), b.book().balances());
    assert_eq!(sim.state.balance(PERSON, TOKEN), 2);
    assert_eq!(sim.state.balance(HOME, TOKEN), 2);
    assert_eq!(sim.state.balance(BUYER, TOKEN), 0);
    assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    let logs = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert!(
        logs.lines()
            .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
            .any(|r| r["detail"]["event"] == "WagesDistributed" && r["detail"]["paid"] == 4)
    );
}

fn wage_relief(
    id: u32,
    month: u32,
    remaining: i32,
    quantity: i32,
) -> economics_compute_smoke::claim_relief::Terms {
    economics_compute_smoke::claim_relief::Terms {
        id,
        proceeding: 1,
        contract: ContractId::Wages(1),
        original_due: 2,
        debtor: ISSUER,
        creditor: WORKER,
        month,
        expected_due: 2,
        expected_remaining: remaining,
        action: economics_compute_smoke::claim_relief::Action::WriteOff { quantity },
    }
}
#[test]
fn accepted_wage_relief_preserves_actual_work_and_payment_and_allows_closure() {
    let (mut w, mut s) = fixture();
    w.employment[0].through = 1;
    s.balances.insert((ISSUER, COIN), 1);
    w.recovery.claim_relief = vec![wage_relief(1, 3, 3, 2), wage_relief(2, 4, 1, 1)];
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut b = a.clone();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 3);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 1);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    let mut resumed = sim.clone();
    let mut c = a.clone();
    through(&mut a, &mut sim, 5);
    through(&mut b, &mut reference, 5);
    through(&mut c, &mut resumed, 5);
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(sim.state, resumed.state);
    assert_eq!(a.book().balances(), b.book().balances());
    assert_eq!(a.book().balances(), c.book().balances());
    let e = &sim.state.employment.earned[&(1, 1)];
    assert_eq!(e.delivered, 2);
    assert_eq!(e.claim.settled, 1);
    assert_eq!(e.claim.outstanding(), 0);
    assert_eq!(sim.state.balance(WORKER, COIN), 1);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    let balances = a.book().balances();
    assert_eq!(balances[&(WORKER, Account::CreditLoss)], 3);
    assert_eq!(balances[&(ISSUER, Account::DebtRelief)], -3);
    assert_eq!(balances[&(WORKER, Account::ServiceIncome)], -4);
}
#[test]
fn stale_wage_relief_and_unconsented_checkpoint_changes_are_rejected() {
    let (mut w, mut s) = fixture();
    s.balances.insert((ISSUER, COIN), 1);
    w.employment[0].through = 1;
    w.recovery.claim_relief.push(wage_relief(1, 3, 4, 4));
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 3);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 3);
    assert!(receipts(&sim).any(|r| matches!(
        r,
        Receipt::ClaimRelief {
            rejection: Some(_),
            written_off: None,
            ..
        }
    )));
    let mut bad = sim.state.clone();
    bad.employment
        .earned
        .get_mut(&(1, 1))
        .unwrap()
        .claim
        .transfer
        .amount
        .quantity -= 1;
    assert!(Simulation::new(sim.world.clone(), bad, Backend::Reference).is_err());
}

#[test]
fn accepted_wage_extension_preserves_debt_until_new_due_and_blocks_early_closure() {
    use economics_compute_smoke::{claim_relief::Action, finance::Condition};
    let (mut w, mut s) = fixture();
    delayed_income(&mut w, &mut s, 4);
    let mut terms = wage_relief(1, 3, 4, 4);
    terms.action = Action::Extend { due: 5 };
    w.recovery.claim_relief.push(terms);
    let mut a = audit(&w, &s);
    let mut b = a.clone();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 4);
    let e = &sim.state.employment.earned[&(1, 1)];
    assert_eq!(e.claim.condition, Condition::OnOrAfterMonth(5));
    assert_eq!(e.claim.outstanding(), 4);
    assert_eq!(sim.state.balance(ISSUER, COIN), 4);
    assert_eq!(sim.state.balance(ESTATE, COIN), 0);
    assert_eq!(sim.state.balance(WORKER, COIN), 0);
    let mut forged = sim.state.clone();
    let p = forged.credit.recovery.proceedings.get_mut(&1).unwrap();
    p.stage = Stage::Closed;
    p.closed = Some(4);
    assert!(Simulation::new(sim.world.clone(), forged, Backend::Reference).is_err());
    let mut resumed = sim.clone();
    let mut c = a.clone();
    through(&mut a, &mut sim, 5);
    assert_eq!(sim.state.balance(ESTATE, COIN), 4);
    assert_eq!(sim.state.balance(WORKER, COIN), 0);
    through(&mut a, &mut sim, 6);
    through(&mut b, &mut reference, 6);
    through(&mut c, &mut resumed, 6);
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.state, resumed.state);
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(sim.state.balance(WORKER, COIN), 4);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    assert!(
        !a.book()
            .balances()
            .contains_key(&(WORKER, Account::CreditLoss))
    );
}

#[test]
fn physical_wage_writeoff_uses_claim_valuation_without_a_coin_buyout() {
    use economics_compute_smoke::minting::FIREWOOD;
    let (mut w, mut s) = fixture();
    w.employment[0].through = 1;
    w.employment[0].wage_per_unit.resource = FIREWOOD;
    w.recovery.claim_relief.push(wage_relief(1, 3, 4, 4));
    s.balances.insert((ISSUER, COIN), 7);
    let mut a = Audit::with_opening(
        &w,
        &s,
        COIN,
        Opening {
            exchange_values: [(FIREWOOD, 3)].into(),
            ..Default::default()
        },
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 4);
    let e = &sim.state.employment.earned[&(1, 1)];
    assert_eq!(
        (e.delivered, e.claim.settled, e.claim.outstanding()),
        (2, 0, 0)
    );
    assert_eq!(sim.state.balance(ISSUER, COIN), 7);
    assert_eq!(sim.state.balance(WORKER, COIN), 0);
    assert_eq!(sim.state.balance(WORKER, FIREWOOD), 0);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    assert_eq!(a.book().balances()[&(WORKER, Account::CreditLoss)], 12);
    assert_eq!(a.book().balances()[&(ISSUER, Account::DebtRelief)], -12);
}
