use economics_compute_smoke::{
    accounting::Account as A,
    compute::Backend,
    employment::{ArrearsPolicy, Reason, Terms},
    financial_reporting::{Audit, Opening},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
    model::*,
    opportunities::{Action, HOUSEHOLD_TYPE, PERSON_TYPE},
    scenario::{LABOR, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const WORKER: AgentId = 89;
fn fixture(cash: i32, budget: i32) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    s.balances.clear();
    s.balances.insert((HOME, TOKEN), cash);
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == WORKER { 3 } else { 0 };
    }
    w.households[0].governance.charter.hiring_budget = Some(Amount::new(TOKEN, budget));
    let law = w.transaction_policy.as_mut().unwrap();
    law.permissions.extend([
        (HOUSEHOLD_TYPE, Action::CapacityTrade),
        (PERSON_TYPE, Action::CapacityTrade),
    ]);
    w.employment.push(Terms {
        id: 1,
        employer: HOME,
        worker: WORKER,
        from: 1,
        through: 1,
        capacity: Amount::new(LABOR, 3),
        wage_per_unit: Amount::new(TOKEN, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            services: Some(Default::default()),
            processes: Some(Default::default()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim)
            .unwrap_or_else(|e| panic!("{e} at {} {:?}", sim.state.month, sim.state.phase));
    }
}
fn value(a: &Audit, agent: AgentId, account: A) -> i128 {
    a.book()
        .balances()
        .get(&(agent, account))
        .copied()
        .unwrap_or(0)
}
#[test]
fn household_hiring_is_bounded_by_charter_cash_and_whole_hours() {
    for (cash, budget, hours) in [(5, 6, 2), (6, 3, 1), (0, 6, 0), (6, 0, 0), (6, 6, 3)] {
        let (w, s) = fixture(cash, budget);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut a, &mut sim, 1);
            let r = sim
                .ledger
                .iter()
                .filter_map(|b| b.employment.as_ref())
                .flat_map(|b| &b.receipts)
                .find(|r| r.requested == 3)
                .unwrap();
            assert_eq!(r.delivered, hours);
            if hours < 3 {
                assert_eq!(r.reason, Reason::HiringBudget);
            }
            assert_eq!(sim.state.balance(WORKER, TOKEN), hours * 2);
            assert_eq!(sim.state.balance(HOME, TOKEN), cash - hours * 2);
            assert_eq!(
                value(&a, HOME, A::PurchasedCapacity(LABOR)),
                i128::from(hours * 2)
            );
            through(&mut a, &mut sim, 2);
            assert_eq!(sim.state.balance(HOME, LABOR), 0);
            assert_eq!(value(&a, HOME, A::ServiceExpense), i128::from(hours * 2));
            let mut replay = s.clone();
            for b in &sim.ledger {
                commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(replay, sim.state);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
#[test]
fn competing_hires_share_one_budget_and_permissions_still_apply() {
    let (mut w, s) = fixture(6, 6);
    let mut second = w.employment[0].clone();
    second.id = 2;
    second.worker = 92;
    w.participants
        .iter_mut()
        .find(|p| p.agent == 92)
        .unwrap()
        .capacity
        .quantity = 3;
    w.employment.push(second);
    w.employment.reverse();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.employment.earned[&(1, 1)].delivered, 3);
    assert!(!sim.state.employment.earned.contains_key(&(2, 1)));
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(HOUSEHOLD_TYPE, Action::CapacityTrade));
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert!(sim.state.employment.earned.is_empty());
}
#[test]
fn hiring_requires_explicit_matching_charter_and_outside_worker() {
    let (w, s) = fixture(6, 6);
    for case in 0..3 {
        let mut w = w.clone();
        match case {
            0 => w.households[0].governance.charter.hiring_budget = None,
            1 => {
                w.households[0]
                    .governance
                    .charter
                    .hiring_budget
                    .as_mut()
                    .unwrap()
                    .quantity = -1
            }
            _ => w.employment[0].worker = PERSON,
        }
        assert!(Simulation::new(w, s.clone(), Backend::Reference).is_err());
    }
}
