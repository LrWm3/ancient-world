use economics_compute_smoke::{
    accounting::Account as A,
    activities::{Target, WorkOrder},
    compute::Backend,
    employment::{ArrearsPolicy, Terms},
    financial_reporting::{Audit, Opening},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
    model::*,
    opportunities::{Action, PERSON_TYPE},
    scenario::{GRAIN, LABOR, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const WORKER: AgentId = 89;
const MEMBER: AgentId = 91;
const MAKE: DefinitionId = 901;
fn fixture() -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    s.balances.clear();
    s.balances.insert((PERSON, TOKEN), 10);
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = match p.agent {
            PERSON | WORKER => 5,
            MEMBER => 1,
            _ => 0,
        };
    }
    let law = w.transaction_policy.as_mut().unwrap();
    law.permissions.extend([
        (PERSON_TYPE, Action::CapacityTrade),
        (PERSON_TYPE, Action::Process(MAKE)),
    ]);
    w.employment.push(Terms {
        id: 1,
        employer: PERSON,
        worker: WORKER,
        from: 1,
        through: 3,
        capacity: Amount::new(LABOR, 5),
        wage_per_unit: Amount::new(TOKEN, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    w.definitions.push(ProcessDefinition {
        id: MAKE,
        name: "member output".into(),
        enabled: true,
        execution: Execution::Productive,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 1,
            entry_inputs: vec![],
            monthly_services: vec![Amount::new(LABOR, 2)],
        }],
        outputs: vec![Amount::new(GRAIN, 4)],
    });
    w.activities.orders.push(WorkOrder {
        agent: MEMBER,
        definition: MAKE,
        priority: 0,
        target: Target::Stock(Amount::new(GRAIN, 100)),
    });
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| *r == GRAIN && **q > 0)
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            exchange_values: [(GRAIN, 1)].into(),
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
fn value(a: &Audit, who: AgentId, account: A) -> i128 {
    a.book()
        .balances()
        .get(&(who, account))
        .copied()
        .unwrap_or(0)
}
fn replay(w: &World, s: &State, sim: &Simulation) {
    let mut replay = s.clone();
    for b in &sim.ledger {
        commit(w, &mut replay, b, sim.backend, DEFAULT_EFFECT_LIMIT).unwrap();
    }
    assert_eq!(replay, sim.state);
}
#[test]
fn private_hiring_does_not_increase_own_labor_contribution_and_cost_follows_allocated_hours() {
    let (w, s) = fixture();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut a, &mut sim, 1);
        let labor = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .next()
            .unwrap();
        let c = labor
            .contributions
            .iter()
            .find(|c| c.member == PERSON)
            .unwrap();
        assert_eq!((c.available, c.reserved, c.directed), (10, 1, 1));
        assert_eq!(labor.recipient, Some(MEMBER));
        assert_eq!(sim.state.balance(WORKER, TOKEN), 10);
        assert_eq!(sim.state.balance(HOME, TOKEN), 0);
        assert_eq!(value(&a, PERSON, A::PurchasedCapacity(LABOR)), 9);
        assert_eq!(value(&a, PERSON, A::TransferExpense), 1);
        assert_eq!(value(&a, MEMBER, A::TransferIncome), -1);
        assert_eq!(
            value(&a, MEMBER, A::Inventory(GRAIN)) + value(&a, HOME, A::Inventory(GRAIN)),
            1
        );
        assert_eq!(sim.state.balance(MEMBER, GRAIN), 2);
        assert_eq!(sim.state.balance(HOME, GRAIN), 2);
        let (mut resumed, mut ra) = (sim.clone(), a.clone());
        a.step(&mut sim).unwrap();
        ra.step(&mut resumed).unwrap();
        assert_eq!(value(&a, PERSON, A::ServiceExpense), 9);
        assert_eq!(value(&a, PERSON, A::PurchasedCapacity(LABOR)), 0);
        assert_eq!(sim.state, resumed.state);
        assert_eq!(a, ra);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn purchased_hours_alone_create_no_member_labor_entitlement_and_internal_hires_reject() {
    let (mut w, s) = fixture();
    w.participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .capacity
        .quantity = 0;
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    let c = sim
        .ledger
        .iter()
        .filter_map(|b| b.household.as_ref())
        .flat_map(|h| &h.labor)
        .flat_map(|d| &d.contributions)
        .find(|c| c.member == PERSON)
        .unwrap();
    assert_eq!((c.available, c.reserved, c.directed), (5, 0, 0));
    assert_eq!(sim.state.balance(MEMBER, GRAIN), 0);
    w.employment[0].worker = MEMBER;
    assert!(
        Simulation::new(w, s, Backend::Reference)
            .unwrap_err()
            .contains("internal household")
    );
}
