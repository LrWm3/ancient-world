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
