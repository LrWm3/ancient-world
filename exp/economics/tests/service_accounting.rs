use economics_compute_smoke::{
    accounting::{Account as A, Book, Flow},
    compute::Backend,
    financial_reporting::{Audit, Opening},
    issuance_accounting::Policy,
    minting::{self, *},
    model::*,
    opportunities::{Action, STATE_TYPE},
    simulation::Simulation,
};
use std::collections::BTreeMap;
const MAKE: DefinitionId = 900;
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        COIN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| {
                    *r != COIN
                        && **q > 0
                        && w.resources
                            .iter()
                            .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
                })
                .map(|(k, q)| (*k, i128::from(*q)))
                .collect(),
            processes: Some(Default::default()),
            services: Some(Default::default()),
            issuance: Some(Policy::NonRedeemableEquity),
            ..Default::default()
        },
    )
    .unwrap()
}
fn fixture(own: i32, required: i32, finish: bool) -> (World, State) {
    let (mut w, s) = minting::scenario("normal").unwrap();
    w.scheduled_starts.retain(|s| s.definition != MINT);
    w.definitions.push(ProcessDefinition {
        id: MAKE,
        name: "two-month paid work".into(),
        enabled: true,
        execution: Execution::Productive,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 2,
            entry_inputs: vec![Amount::new(METAL, 2)],
            monthly_services: vec![Amount::new(HOURS, required)],
        }],
        outputs: vec![Amount::new(FIREWOOD, 2)],
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((STATE_TYPE, Action::Process(MAKE)));
    w.scheduled_starts.push(ScheduledStart {
        month: 2,
        agent: ISSUER,
        definition: MAKE,
    });
    w.capacity_overrides.insert((2, ISSUER), own);
    w.capacity_overrides
        .insert((3, ISSUER), if finish { required } else { 0 });
    (w, s)
}
fn through(a: &mut Audit, s: &mut Simulation, month: u32) {
    while s.state.month <= month {
        a.step(s)
            .unwrap_or_else(|e| panic!("{e}: {} {:?}", s.state.month, s.state.phase));
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
fn paid_hours_become_wip_then_inventory_on_cpu_and_resume() {
    let (w, s) = fixture(0, 2, true);
    let mut a = audit(&w, &s);
    let mut b = a.clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    while !(cpu.state.month == 2 && cpu.state.phase == Phase::Productive) {
        a.step(&mut cpu).unwrap();
        b.step(&mut reference).unwrap();
    }
    assert_eq!(value(&a, ISSUER, A::PurchasedCapacity(HOURS)), 4);
    assert_eq!(value(&a, WORKER, A::ServiceIncome), -4);
    assert_eq!(value(&a, ISSUER, A::ServiceExpense), 0);
    let saved = Book::from_json(&a.book().to_json().unwrap()).unwrap();
    assert_eq!(saved.balances(), a.book().balances());
    let mut resumed = a.clone();
    let mut checkpoint = cpu.clone();
    through(&mut a, &mut cpu, 2);
    through(&mut b, &mut reference, 2);
    let p = cpu
        .state
        .processes
        .values()
        .find(|p| p.definition == MAKE)
        .unwrap();
    assert_eq!(value(&a, ISSUER, A::WorkInProgress(p.id)), 6);
    assert_eq!(value(&a, ISSUER, A::PurchasedCapacity(HOURS)), 0);
    through(&mut a, &mut cpu, 3);
    through(&mut b, &mut reference, 3);
    through(&mut resumed, &mut checkpoint, 3);
    assert_eq!(a, b);
    assert_eq!(a, resumed);
    assert_eq!(cpu.state, reference.state);
    assert_eq!(value(&a, ISSUER, A::Inventory(FIREWOOD)), 6);
    assert_eq!(value(&a, ISSUER, A::ServiceExpense), 0);
    assert_eq!(value(&a, ISSUER, A::ProductionExpense), 0);
    let worker = a.book().statements(WORKER, 2, 3).unwrap();
    assert_eq!(worker.cash_flows[&Flow::Operating], 4);
    a.finalize_through(3).unwrap();
}
#[test]
fn mixed_free_and_bought_hours_allocate_proportionally_and_unused_cost_expires() {
    let (w, s) = fixture(2, 2, true);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 2);
    assert_eq!(value(&a, ISSUER, A::PurchasedCapacity(HOURS)), 2);
    let p = sim
        .state
        .processes
        .values()
        .find(|p| p.definition == MAKE)
        .unwrap();
    assert_eq!(value(&a, ISSUER, A::WorkInProgress(p.id)), 4);
    assert_eq!(sim.state.balance(ISSUER, HOURS), 2);
    a.step(&mut sim).unwrap(); // Same quantity regenerated; old paid cost still expires.
    assert_eq!(sim.state.balance(ISSUER, HOURS), 2);
    assert_eq!(value(&a, ISSUER, A::PurchasedCapacity(HOURS)), 0);
    assert_eq!(value(&a, ISSUER, A::ServiceExpense), 2);
    through(&mut a, &mut sim, 4);
    assert_eq!(value(&a, ISSUER, A::Inventory(FIREWOOD)), 4);
    assert_eq!(value(&a, ISSUER, A::ServiceExpense), 2);
}
#[test]
fn missed_work_loses_material_and_used_labor_cost_without_extra_wages() {
    let (w, s) = fixture(0, 2, false);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 3);
    assert_eq!(value(&a, ISSUER, A::ProductionLoss), 6);
    assert_eq!(value(&a, ISSUER, A::Inventory(FIREWOOD)), 0);
    assert_eq!(value(&a, WORKER, A::ServiceIncome), -4);
}
#[test]
fn productive_service_cost_can_be_capitalized_into_a_durable() {
    use economics_compute_smoke::activities::{DurableKind, Outcome};
    let (mut w, s) = fixture(0, 2, true);
    w.definitions
        .iter_mut()
        .find(|d| d.id == MAKE)
        .unwrap()
        .outputs
        .clear();
    w.activities.kinds.insert(
        42,
        DurableKind {
            name: "workshop".into(),
            lifetime: 2,
            attached: false,
            monthly_decay: 1,
        },
    );
    w.activities.outcomes.insert(MAKE, Outcome::Create(42));
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 3);
    let id = sim
        .state
        .equipment
        .values()
        .find(|a| a.kind == 42)
        .unwrap()
        .id;
    assert_eq!(value(&a, ISSUER, A::Tangible(id)), 6);
    through(&mut a, &mut sim, 5);
    assert_eq!(value(&a, ISSUER, A::Tangible(id)), 0);
    assert_eq!(value(&a, ISSUER, A::Depreciation), 6);
}
#[test]
fn minting_and_idle_capacity_are_expensed_at_actual_use_or_expiration() {
    for run in [false, true] {
        let (mut w, s) = minting::scenario("normal").unwrap();
        if !run {
            w.scheduled_starts.retain(|s| s.definition != MINT);
        }
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut sim, 2);
        assert_eq!(value(&a, ISSUER, A::ServiceExpense), 0);
        assert_eq!(
            value(&a, ISSUER, A::ProductionExpense),
            if run { 6 } else { 0 }
        );
        assert_eq!(
            value(&a, ISSUER, A::PurchasedCapacity(HOURS)),
            if run { 0 } else { 4 }
        );
        through(&mut a, &mut sim, 4);
        assert_eq!(
            value(&a, ISSUER, A::ServiceExpense),
            if run { 0 } else { 4 }
        );
        assert_eq!(
            value(&a, ISSUER, A::MonetaryIssuance),
            if run { -10 } else { 0 }
        );
    }
}

#[test]
fn invalid_opening_cost_and_forged_labor_payment_are_rejected() {
    let (w, s) = fixture(0, 2, true);
    for (resource, cost) in [(HOURS, 4), (METAL, 4), (HOURS, -1)] {
        let result = Audit::with_opening(
            &w,
            &s,
            COIN,
            Opening {
                inventory: s
                    .balances
                    .iter()
                    .filter(|((_, r), q)| *r != COIN && **q > 0)
                    .map(|(k, q)| (*k, i128::from(*q)))
                    .collect(),
                services: Some(economics_compute_smoke::service_accounting::Costs {
                    balances: BTreeMap::from([((ISSUER, resource), cost)]),
                }),
                ..Default::default()
            },
        );
        assert!(result.unwrap_err().contains("capacity"));
    }
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while !(sim.state.month == 2 && sim.state.phase == Phase::Acquire) {
        a.step(&mut sim).unwrap();
    }
    let old = a.clone();
    let mut preview = sim.clone();
    preview.step().unwrap();
    let mut bad = preview.ledger.last().unwrap().clone();
    let labor = bad
        .transactions
        .iter_mut()
        .find(|t| t.effects.iter().any(|e| e.account.1 == HOURS))
        .unwrap();
    for e in &mut labor.effects {
        if e.account.1 == COIN {
            e.delta *= 2;
        }
    }
    assert!(
        a.record(&sim.world, &sim.state, &bad, &preview.state)
            .is_err()
    );
    assert_eq!(a, old);
}

#[test]
fn two_processes_share_one_paid_cost_pool_and_preserve_rounding_remainder() {
    let (mut w, mut s) = fixture(1, 1, true);
    s.balances.insert((ISSUER, COIN), 10);
    w.minting
        .as_mut()
        .unwrap()
        .deals
        .iter_mut()
        .find(|d| d.market == HOURS)
        .unwrap()
        .price = 5;
    let first = w.definitions.iter_mut().find(|d| d.id == MAKE).unwrap();
    first.stages[0].months = 1;
    first.stages[0].entry_inputs.clear();
    let mut second = first.clone();
    second.id = MAKE + 1;
    w.definitions.push(second);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((STATE_TYPE, Action::Process(MAKE + 1)));
    w.scheduled_starts.push(ScheduledStart {
        month: 2,
        agent: ISSUER,
        definition: MAKE + 1,
    });
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 2);
    assert_eq!(sim.state.balance(ISSUER, FIREWOOD), 4);
    // Two of three available hours: floor(5*2/3)=3 used, 2 remain.
    assert_eq!(value(&a, ISSUER, A::Inventory(FIREWOOD)), 3);
    assert_eq!(value(&a, ISSUER, A::PurchasedCapacity(HOURS)), 2);
    through(&mut a, &mut sim, 3);
    assert_eq!(value(&a, ISSUER, A::ServiceExpense), 2);
    assert_eq!(value(&a, WORKER, A::ServiceIncome), -5);
}
#[test]
fn unfunded_packages_create_neither_paid_capacity_nor_wages() {
    for case in ["treasury", "metal", "labor"] {
        let (w, s) = minting::scenario(case).unwrap();
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut sim, 3);
        assert_eq!(value(&a, ISSUER, A::PurchasedCapacity(HOURS)), 0);
        assert_eq!(value(&a, ISSUER, A::ServiceExpense), 0);
        assert_eq!(value(&a, WORKER, A::ServiceIncome), 0);
    }
}
