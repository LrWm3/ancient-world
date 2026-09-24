use economics_compute_smoke::{
    accounting::Account as A,
    compute::Backend,
    financial_reporting::Audit,
    model::*,
    scenario::{self, GRAIN, GROW, PERSON, PLOT, SEED, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
fn fixture() -> (World, State) {
    let (mut w, s) = scenario::baseline();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    (w, s)
}
fn audit(
    w: &World,
    s: &State,
    weights: BTreeMap<DefinitionId, BTreeMap<ResourceId, u32>>,
) -> Audit {
    Audit::with_processes(
        w,
        s,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        BTreeMap::from([((PERSON, GRAIN), 10), ((PERSON, SEED), 12)]),
        weights,
    )
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
#[test]
fn planting_capitalizes_seed_harvest_releases_work_and_consumption_expenses_cost_on_cpu() {
    let (w, s) = fixture();
    let mut a = audit(&w, &s, BTreeMap::new());
    let mut b = a.clone();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut reference, 1);
    through(&mut b, &mut cpu, 1);
    assert_eq!(a, b);
    let report = a.book().statements(PERSON, 1, 1).unwrap();
    assert_eq!(
        report
            .trial_balance
            .iter()
            .filter(|(k, _)| matches!(k, A::WorkInProgress(_)))
            .map(|(_, v)| *v)
            .sum::<i128>(),
        12
    );
    assert_eq!(report.expenses[&A::ConsumptionExpense], 2);
    assert_eq!(report.net_income, -2);
    let mut resumed = a.clone();
    let mut checkpoint = reference.clone();
    through(&mut a, &mut reference, 6);
    through(&mut resumed, &mut checkpoint, 6);
    through(&mut b, &mut cpu, 6);
    assert_eq!(a, resumed);
    assert_eq!(a, b);
    assert_eq!(reference.state, cpu.state);
    let report = a.book().statements(PERSON, 1, 6).unwrap();
    assert!(
        report
            .trial_balance
            .iter()
            .filter(|(k, _)| matches!(k, A::WorkInProgress(_)))
            .all(|(_, v)| *v == 0)
    );
    assert_eq!(report.income.values().sum::<i128>(), 0);
    assert_eq!(report.cash_flows.values().sum::<i128>(), 0);
    assert_eq!(report.assets + report.expenses.values().sum::<i128>(), 22);
    assert!(reference.state.balance(PERSON, GRAIN) > 0);
}
#[test]
fn missed_work_writes_off_only_capitalized_cost() {
    let (mut w, s) = fixture();
    w.capacity_overrides.insert((2, PERSON), 0);
    let mut a = audit(&w, &s, BTreeMap::new());
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 2);
    let report = a.book().statements(PERSON, 1, 2).unwrap();
    assert_eq!(report.expenses[&A::ProductionLoss], 12);
    assert_eq!(report.expenses[&A::ConsumptionExpense], 4);
    assert_eq!(report.assets, 6);
}
#[test]
fn joint_output_requires_explicit_shares_and_failure_publishes_neither_side() {
    let (mut w, s) = fixture();
    w.definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .outputs
        .push(Amount::new(SEED, 1));
    let mut a = audit(&w, &s, BTreeMap::new());
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    through(&mut a, &mut sim, 5);
    a.step(&mut sim).unwrap(); // Open month 6.
    let old = a.clone();
    let state = sim.state.clone();
    assert!(a.step(&mut sim).is_err());
    assert_eq!(a, old);
    assert_eq!(sim.state, state);
    let mut a = audit(
        &w,
        &s,
        BTreeMap::from([(GROW, BTreeMap::from([(GRAIN, 3), (SEED, 1)]))]),
    );
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 5);
    a.step(&mut sim).unwrap();
    a.step(&mut sim).unwrap(); // Productive: cost 12 -> grain 9 + seed 3.
    let report = a.book().statements(PERSON, 1, 6).unwrap();
    assert_eq!(report.trial_balance[&A::Inventory(GRAIN)], 9);
    assert_eq!(report.trial_balance[&A::Inventory(SEED)], 3);
}

#[test]
fn storage_blocked_harvest_records_failed_work_not_fictitious_spoilage() {
    let (mut w, s) = fixture();
    w.storage.weights = BTreeMap::from([(GRAIN, 1), (SEED, 1)]);
    w.storage.capacities.insert(PERSON, 6);
    let mut a = audit(&w, &s, BTreeMap::new());
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 6);
    let report = a.book().statements(PERSON, 1, 6).unwrap();
    assert_eq!(report.expenses[&A::ProductionLoss], 12);
    assert_eq!(report.expenses[&A::ConsumptionExpense], 10);
    assert_eq!(report.assets, 0);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.definition == GROW && p.status == Status::Aborted)
    );
}

#[test]
fn repeated_harvests_preserve_material_cost_and_open_work_cannot_be_rebased_for_free() {
    let (mut w, s) = fixture();
    w.rights[0].through = 60;
    w.definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .outputs
        .push(Amount::new(SEED, 1));
    let mut a = audit(
        &w,
        &s,
        BTreeMap::from([(GROW, BTreeMap::from([(GRAIN, 3), (SEED, 1)]))]),
    );
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    assert!(
        Audit::with_inventory(
            &sim.world,
            &sim.state,
            TOKEN,
            BTreeMap::from([(PLOT, 0)]),
            BTreeMap::from([((PERSON, GRAIN), 8)])
        )
        .is_err()
    );
    for month in 2..=18 {
        through(&mut a, &mut sim, month);
        let report = a.book().statements(PERSON, 1, month).unwrap();
        assert_eq!(report.assets + report.expenses.values().sum::<i128>(), 22);
        assert_eq!(report.net_income, -report.expenses.values().sum::<i128>());
    }
    assert!(
        sim.state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Completed)
            .count()
            >= 2
    );
}

#[test]
fn shared_input_cost_passes_to_operator_and_regrowth_adds_no_income() {
    use economics_compute_smoke::pools::{Pool, PoolInput};
    let (mut w, mut s) = fixture();
    s.balances.insert((PERSON, SEED), 0);
    s.balances.insert((scenario::STATE_AGENT, SEED), 1);
    w.pools.push(Pool {
        account: (scenario::STATE_AGENT, SEED),
        capacity: 2,
        monthly_regeneration: 1,
    });
    w.pool_inputs.push(PoolInput {
        definition: GROW,
        account: (scenario::STATE_AGENT, SEED),
    });
    let mut a = Audit::with_processes(
        &w,
        &s,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        BTreeMap::from([((PERSON, GRAIN), 10), ((scenario::STATE_AGENT, SEED), 12)]),
        BTreeMap::new(),
    )
    .unwrap();
    let mut b = a.clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut reference, 1);
    through(&mut b, &mut cpu, 1);
    assert_eq!(a, b);
    assert_eq!(reference.state, cpu.state);
    let owner = a.book().statements(scenario::STATE_AGENT, 1, 1).unwrap();
    let operator = a.book().statements(PERSON, 1, 1).unwrap();
    // Open replenishes 1 -> 2 units without additional cost; planting takes half.
    assert_eq!(owner.trial_balance[&A::Inventory(SEED)], 6);
    assert_eq!(owner.expenses[&A::TransferExpense], 6);
    assert_eq!(operator.income[&A::TransferIncome], 6);
    assert_eq!(
        operator
            .trial_balance
            .iter()
            .filter(|(a, _)| matches!(a, A::WorkInProgress(_)))
            .map(|(_, v)| *v)
            .sum::<i128>(),
        6
    );
    through(&mut a, &mut reference, 6);
    through(&mut b, &mut cpu, 6);
    assert_eq!(a, b);
    assert_eq!(reference.state, cpu.state);
    let owner = a.book().statements(scenario::STATE_AGENT, 2, 6).unwrap();
    assert_eq!(owner.net_income, 0);
    assert_eq!(owner.trial_balance[&A::Inventory(SEED)], 6);
    assert_eq!(reference.state.balance(scenario::STATE_AGENT, SEED), 2);
}
