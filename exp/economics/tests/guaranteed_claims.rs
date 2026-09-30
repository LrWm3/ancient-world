use economics_compute_smoke::{
    accounting::Account,
    compute::Backend,
    employment::{ArrearsPolicy, Terms},
    financial_reporting::{Audit, Opening},
    minting::{self, COIN, HOURS, ISSUER, SUPPLIER, WORKER},
    model::*,
    recovery::{Guarantee, GuaranteedClaim, Receipt},
    simulation::Simulation,
};

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
    s.balances.insert((SUPPLIER, COIN), 3);
    w.employment.push(Terms {
        id: 1,
        employer: ISSUER,
        worker: WORKER,
        from: 1,
        through: 1,
        capacity: Amount::new(HOURS, 2),
        wage_per_unit: Amount::new(COIN, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    w.recovery.guarantees.push(Guarantee {
        id: 1,
        claim: GuaranteedClaim::Wages {
            agreement: 1,
            earned_month: 1,
        },
        guarantor: SUPPLIER,
        cap: 4,
        from: 1,
        through: 8,
        delay_months: 0,
        recourse: 101,
        priority: 0,
    });
    (w, s)
}
fn through(a: &mut Audit, s: &mut Simulation, month: u32) {
    while s.state.month <= month {
        a.step(s)
            .unwrap_or_else(|e| panic!("{} {:?}: {e}", s.state.month, s.state.phase));
    }
}
#[test]
fn earned_wage_guarantee_pays_only_the_residual_and_creates_matching_recourse() {
    let (w, s) = fixture();
    let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
    let mut resumed =
        Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 3);
    reference.run_months(3).unwrap();
    resumed.run_months(2).unwrap();
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(sim.state, resumed.state);
    assert_eq!(sim.state.balance(WORKER, COIN), 3);
    assert_eq!(sim.state.balance(SUPPLIER, COIN), 0);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 1);
    let recourse = &sim.state.credit.loans[&101];
    assert_eq!(
        (recourse.debtor, recourse.creditor, recourse.principal),
        (ISSUER, SUPPLIER, 3)
    );
    assert_eq!(recourse.opened, 2);
    let b = a.book().balances();
    assert_eq!(b[&(WORKER, Account::WagesReceivable(1, 1))], 1);
    assert_eq!(b[&(ISSUER, Account::WagesPayable(1, 1))], -1);
    assert_eq!(b[&(SUPPLIER, Account::LoanReceivable(101))], 3);
    assert_eq!(b[&(ISSUER, Account::LoanPayable(101))], -3);
    assert_eq!(b[&(WORKER, Account::ServiceIncome)], -4);
    assert_eq!(b[&(ISSUER, Account::ServiceExpense)], 4);
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.recovery)
            .any(|r| matches!(
                r,
                Receipt::Guaranteed {
                    paid: 3,
                    requested: 4,
                    ..
                }
            ))
    );
}
#[test]
fn unearned_work_has_no_call_and_ordinary_payment_reduces_coverage() {
    for (hours, employer_cash, expected) in [(0, 0, 0), (2, 1, 3), (2, 4, 0)] {
        let (mut w, mut s) = fixture();
        w.participants
            .iter_mut()
            .find(|p| p.agent == WORKER)
            .unwrap()
            .capacity
            .quantity = hours;
        s.balances.insert((ISSUER, COIN), employer_cash);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(2).unwrap();
        assert_eq!(
            sim.state
                .credit
                .recovery
                .paid_guarantees
                .get(&1)
                .copied()
                .unwrap_or(0),
            expected
        );
        if expected > 0 {
            assert_eq!(sim.state.credit.loans[&101].principal, expected);
        } else {
            assert!(!sim.state.credit.loans.contains_key(&101));
        }
    }
}

fn land_fixture() -> (World, State) {
    let (mut w, s) = fixture();
    w.employment.clear();
    w.assets.push(Asset {
        id: 900,
        owner: WORKER,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: 900,
        holder: ISSUER,
        asset: 900,
        from: 1,
        through: 30,
        output_owner: ISSUER,
    });
    w.agreements
        .push(economics_compute_smoke::commitments::Agreement {
            id: 900,
            right: 900,
            creditor: WORKER,
            debtor: ISSUER,
            activated: 1,
            payment: Amount::new(COIN, 4),
        });
    w.recovery.guarantees[0].claim = GuaranteedClaim::Land {
        agreement: 900,
        due: 13,
    };
    w.recovery.guarantees[0].through = 30;
    (w, s)
}
#[test]
fn land_guarantee_settles_one_original_bill_without_renewing_or_paying_later_bills() {
    let (w, s) = land_fixture();
    let opening = Opening {
        assets: [(900, 0)].into(),
        dues: Some(Default::default()),
        ..Default::default()
    };
    let mut a = Audit::with_opening(&w, &s, COIN, opening).unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 12);
    assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
    through(&mut a, &mut sim, 25);
    reference.run_months(25).unwrap();
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    let first = &sim.state.obligations[&(900, 13)];
    assert_eq!(
        (
            first.owed,
            first.paid,
            first.in_kind_paid,
            first.outstanding()
        ),
        (4, 3, 3, 1)
    );
    assert_eq!(sim.state.obligations[&(900, 25)].outstanding(), 4);
    assert_eq!(sim.state.credit.loans[&101].principal, 3);
    assert_eq!(sim.state.balance(WORKER, COIN), 3);
    let b = a.book().balances();
    assert_eq!(b[&(ISSUER, Account::DuesPayable(900, 13))], -1);
    assert_eq!(b[&(ISSUER, Account::LoanPayable(101))], -3);
    assert_eq!(b[&(WORKER, Account::DuesReceivable(900, 13))], 1);
    assert_eq!(b[&(SUPPLIER, Account::LoanReceivable(101))], 3);
    assert_eq!(b[&(ISSUER, Account::DuesExpense)], 8);
    assert_eq!(b[&(WORKER, Account::DuesIncome)], -8);
}
#[test]
fn guarantee_terms_reject_unidentified_dates_and_unsupported_native_tender() {
    let (mut w, s) = land_fixture();
    w.recovery.guarantees[0].claim = GuaranteedClaim::Land {
        agreement: 900,
        due: 14,
    };
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
    let (mut w, s) = fixture();
    let physical = economics_compute_smoke::minting::FIREWOOD;
    w.employment[0].wage_per_unit.resource = physical;
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
}

const OTHER_WORKER: AgentId = 92;
const OTHER_GUARANTOR: AgentId = 93;
fn competing() -> (World, State) {
    let (mut w, mut s) = fixture();
    w.agents.push(Agent {
        id: OTHER_WORKER,
        name: "second worker".into(),
    });
    let mut participant = w
        .participants
        .iter()
        .find(|p| p.agent == WORKER)
        .unwrap()
        .clone();
    participant.agent = OTHER_WORKER;
    w.participants.push(participant);
    let mut job = w.employment[0].clone();
    job.id = 2;
    job.worker = OTHER_WORKER;
    w.employment.push(job);
    let mut g = w.recovery.guarantees[0].clone();
    g.id = 2;
    g.claim = GuaranteedClaim::Wages {
        agreement: 2,
        earned_month: 1,
    };
    g.recourse = 102;
    w.recovery.guarantees.push(g);
    s.balances.insert((SUPPLIER, COIN), 6);
    (w, s)
}
#[test]
fn explicit_guarantee_allocation_compares_identical_requests_and_finite_funds() {
    use economics_compute_smoke::finance::CollectionPolicy::{Proportional, Stable};
    for (policy, high_second, expected) in [
        (Stable, false, (4, 2)),
        (Proportional, false, (3, 3)),
        (Proportional, true, (2, 4)),
    ] {
        let (mut w, s) = competing();
        w.recovery.guarantee_policy = policy;
        if high_second {
            w.recovery.guarantees[0].priority = 1;
        }
        let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
        w.recovery.guarantees.reverse();
        w.employment.reverse();
        let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut sim, 2);
        reference.run_months(2).unwrap();
        assert_eq!(sim.state, reference.state);
        assert_eq!(sim.ledger, reference.ledger);
        assert_eq!(
            (
                sim.state.balance(WORKER, COIN),
                sim.state.balance(OTHER_WORKER, COIN)
            ),
            expected
        );
        assert_eq!(sim.state.balance(SUPPLIER, COIN), 0);
        let receipts: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.recovery)
            .filter_map(|r| {
                if let Receipt::Guaranteed {
                    requested,
                    allocated,
                    paid,
                    ..
                } = r
                {
                    Some((*requested, *allocated, *paid))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(receipts[0].0, 4);
        assert_eq!(receipts[1].0, 4);
        assert_eq!(receipts.iter().map(|r| r.2).sum::<i32>(), 6);
        assert!(
            receipts
                .iter()
                .all(|r| r.1.is_some() == (policy == Proportional))
        );
    }
}
#[test]
fn overlapping_coverage_releases_unneeded_grants_without_double_payment() {
    let (mut w, mut s) = competing();
    w.recovery.guarantee_policy = economics_compute_smoke::finance::CollectionPolicy::Proportional;
    w.agents.push(Agent {
        id: OTHER_GUARANTOR,
        name: "another guarantor".into(),
    });
    s.balances.insert((OTHER_GUARANTOR, COIN), 4);
    s.balances.insert((SUPPLIER, COIN), 4);
    let mut duplicate = w.recovery.guarantees[0].clone();
    duplicate.id = 3;
    duplicate.recourse = 103;
    w.recovery.guarantees[0].guarantor = OTHER_GUARANTOR;
    w.recovery.guarantees.push(duplicate);
    let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    w.recovery.guarantees.reverse();
    let mut reordered = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 2);
    reordered.run_months(2).unwrap();
    assert_eq!(sim.state, reordered.state);
    assert_eq!(sim.ledger, reordered.ledger);
    assert_eq!(sim.state.balance(WORKER, COIN), 4);
    assert_eq!(sim.state.balance(OTHER_WORKER, COIN), 4);
    assert_eq!(sim.state.credit.loans[&101].principal, 4);
    assert_eq!(sim.state.credit.loans[&102].principal, 4);
    assert!(!sim.state.credit.loans.contains_key(&103));
    assert!(
        sim.state
            .employment
            .earned
            .values()
            .all(|e| e.claim.outstanding() == 0)
    );
}
