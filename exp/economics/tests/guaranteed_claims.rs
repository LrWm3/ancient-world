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
