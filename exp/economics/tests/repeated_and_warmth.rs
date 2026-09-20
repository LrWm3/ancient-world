use economics_compute_smoke::{
    compute::Backend,
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
use std::collections::BTreeMap;

fn run(name: &str, backend: Backend) -> Simulation {
    let (world, initial) = named(name).unwrap();
    let mut sim = Simulation::new(world, initial, backend).unwrap();
    sim.run_months(REPEATED_MONTHS).unwrap();
    sim
}
fn total_deficit(sim: &Simulation, resource: ResourceId) -> i32 {
    sim.reports.iter().map(|r| r.deficit(resource)).sum()
}
fn harvests(sim: &Simulation) -> Vec<u32> {
    sim.state
        .processes
        .values()
        .filter(|p| p.definition == GROW && p.status == Status::Completed)
        .map(|p| p.reserved_through)
        .collect()
}
fn work(sim: &Simulation) -> i32 {
    -sim.ledger
        .iter()
        .filter(|b| b.phase == Phase::Productive)
        .flat_map(|b| &b.transactions)
        .flat_map(|t| &t.effects)
        .filter(|e| e.account.1 == LABOR)
        .map(|e| e.delta)
        .sum::<i32>()
}

#[test]
fn repeated_harvests_replenish_seed_without_accumulating_or_spending_it_early() {
    let sim = run("repeated-harvests", Backend::Reference);
    assert_eq!(harvests(&sim), [6, 14, 22, 30, 38, 46, 54]);
    let starts: Vec<_> = sim
        .state
        .processes
        .values()
        .filter(|p| p.definition == GROW)
        .map(|p| p.start)
        .collect();
    assert_eq!(starts, [1, 9, 17, 25, 33, 41, 49, 57]);
    assert_eq!(total_deficit(&sim, NUTRITION), 0);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 5 + 7 * 8 - 60);
    assert_eq!(work(&sim), 61);
    let (world, mut state) = repeated();
    for batch in &sim.ledger {
        commit(
            &world,
            &mut state,
            batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
        let active = state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Active)
            .count();
        assert_eq!(state.balance(PERSON, SEED) + active as i32, 1);
        assert!(active <= 1);
    }
    // The seed is in the eighth crop at the cutoff, not lost or stockpiled.
    assert_eq!(state.balance(PERSON, SEED), 0);
    assert_eq!(
        state
            .processes
            .values()
            .find(|p| p.status == Status::Active)
            .unwrap()
            .reserved_through,
        62
    );
}

#[test]
fn repeated_production_still_needs_initial_seed_and_a_long_enough_forecast() {
    for name in ["repeated-no-seed", "repeated-short-horizon"] {
        let sim = run(name, Backend::Reference);
        assert!(harvests(&sim).is_empty());
        assert_eq!(total_deficit(&sim, NUTRITION), 55);
        assert_eq!(work(&sim), 0);
    }
}

#[test]
fn food_priority_meets_both_needs_with_shared_labor_and_finite_wood() {
    let sim = run("warmth-food-first", Backend::Reference);
    assert_eq!(harvests(&sim), [6, 14, 22, 30, 38, 46, 54]);
    assert_eq!(total_deficit(&sim, NUTRITION), 0);
    assert_eq!(total_deficit(&sim, WARMTH), 0);
    assert_eq!(work(&sim), 93); // 61 crop work + 32 fuel preparation
    assert_eq!(sim.state.balance(PERSON, RAW_WOOD), 40 - 32);
    assert_eq!(sim.state.balance(PERSON, FUEL), 1 + 2 * 32 - 60);
    for row in &sim.reports {
        assert_eq!(row.fulfilled(NUTRITION), 1);
        assert_eq!(row.fulfilled(WARMTH), 1);
        assert!((0..=2).contains(&row.balances[&LABOR]));
    }
    for batch in sim.ledger.iter().filter(|b| b.phase == Phase::Productive) {
        let used: i32 = batch.receipts.iter().map(|r| r.completed).sum();
        assert!(used <= 2);
    }
}

#[test]
fn warmth_buffer_priority_can_abort_a_feasible_harvest_and_lose_the_seed() {
    let sim = run("warmth-first", Backend::Reference);
    assert_eq!(total_deficit(&sim, WARMTH), 0);
    assert_eq!(total_deficit(&sim, NUTRITION), 55);
    assert!(harvests(&sim).is_empty());
    let crop = sim
        .state
        .processes
        .values()
        .find(|p| p.definition == GROW)
        .unwrap();
    assert_eq!(
        (crop.start, crop.reserved_through, crop.status),
        (6, 11, Status::Aborted)
    );
    assert_eq!(sim.state.balance(PERSON, SEED), 0);
    assert_eq!(work(&sim), 38);
    assert_eq!(sim.reports[9].balances[&FUEL], 5); // enough warmth stock before failed harvest
    let harvest = sim
        .ledger
        .iter()
        .find(|b| b.month == 11 && b.phase == Phase::Productive)
        .unwrap();
    let refused = harvest
        .receipts
        .iter()
        .find(|r| r.definition == Some(GROW))
        .unwrap();
    assert_eq!(
        (refused.requested, refused.allocated, refused.completed),
        (2, 0, 0)
    );
    assert_eq!(refused.reason, Reason::InsufficientCapacity);
    assert_eq!(sim.reports[10].balances[&LABOR], 1); // unfillable partial grant stays unused
    assert!(
        !sim.state
            .processes
            .values()
            .any(|p| p.status == Status::Active)
    );
}

#[test]
fn priority_comparison_uses_identical_opening_resources_and_requests() {
    let (world, state) = named("warmth-first").unwrap();
    let mut warm_first = Simulation::new(world, state, Backend::Reference).unwrap();
    warm_first.run_months(10).unwrap();
    warm_first.step().unwrap(); // Open month 11
    let mut food_first = warm_first.clone();
    for need in &mut food_first.world.participants[0].needs {
        need.priority = u32::from(need.resource != NUTRITION);
    }
    assert_eq!(warm_first.state, food_first.state);
    warm_first.step().unwrap();
    food_first.step().unwrap();
    let requests = |sim: &Simulation| -> BTreeMap<_, _> {
        sim.ledger
            .last()
            .unwrap()
            .receipts
            .iter()
            .filter_map(|r| r.definition.map(|id| (id, r.requested)))
            .collect()
    };
    assert_eq!(requests(&warm_first), requests(&food_first));
    assert_eq!(
        requests(&warm_first),
        BTreeMap::from([(GROW, 2), (PREPARE_FUEL, 1)])
    );
    assert_eq!(warm_first.state.balance(PERSON, GRAIN), 0);
    assert_eq!(food_first.state.balance(PERSON, GRAIN), 8);
    assert_eq!(food_first.state.balance(PERSON, SEED), 1);
    // Neither branch reruns production or borrows next month's capacity.
    assert_eq!(warm_first.state.phase, Phase::Consumption);
    assert_eq!(food_first.state.phase, Phase::Consumption);
}

#[test]
fn controls_separate_priority_capacity_inactive_demand_and_missing_inputs() {
    for name in ["warmth-abundant-food-first", "warmth-abundant-warmth-first"] {
        let sim = run(name, Backend::Reference);
        assert_eq!(total_deficit(&sim, NUTRITION), 0);
        assert_eq!(total_deficit(&sim, WARMTH), 0);
        assert_eq!(harvests(&sim).len(), 7);
    }
    let protected = run("warmth-protect-active", Backend::Reference);
    assert_eq!(harvests(&protected), [11, 19, 27, 35, 43, 51, 59]);
    assert_eq!(total_deficit(&protected, NUTRITION), 5);
    assert_eq!(total_deficit(&protected, WARMTH), 0);
    assert!(
        !protected
            .state
            .processes
            .values()
            .any(|p| p.status == Status::Aborted)
    );
    let inactive = run("warmth-inactive", Backend::Reference);
    assert_eq!(harvests(&inactive).len(), 7);
    assert_eq!(inactive.state.balance(PERSON, RAW_WOOD), 40);
    assert_eq!(inactive.state.balance(PERSON, FUEL), 1);
    assert!(
        !inactive
            .state
            .processes
            .values()
            .any(|p| p.definition == PREPARE_FUEL || p.definition == USE_FUEL)
    );
    let missing = run("warmth-no-wood", Backend::Reference);
    assert_eq!(total_deficit(&missing, NUTRITION), 0);
    assert_eq!(total_deficit(&missing, WARMTH), 59);
    assert!(
        missing
            .ledger
            .iter()
            .flat_map(|b| &b.receipts)
            .any(|r| r.definition == Some(PREPARE_FUEL) && r.reason == Reason::MissingStock)
    );
}

#[test]
fn all_long_scenarios_match_cubecl_cpu_reference_and_ledger_replay() {
    for &name in LONG_SCENARIOS {
        let reference = run(name, Backend::Reference);
        let cpu = run(name, Backend::CubeCpu);
        assert_eq!(cpu.state, reference.state, "{name}");
        assert_eq!(cpu.reports, reference.reports, "{name}");
        assert_eq!(cpu.ledger, reference.ledger, "{name}");
        let (world, mut replay) = named(name).unwrap();
        for batch in &cpu.ledger {
            commit(
                &world,
                &mut replay,
                batch,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT,
            )
            .unwrap();
        }
        assert_eq!(replay, cpu.state, "{name}");
    }
}

#[test]
fn repeated_cycles_survive_monthly_batching_checkpoints_and_reordered_needs() {
    let expected = run("warmth-food-first", Backend::Reference);
    let (mut world, state) = named("warmth-food-first").unwrap();
    world.participants[0].needs.reverse();
    world.definitions.reverse();
    world.resources.reverse();
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    for month in 1..=REPEATED_MONTHS {
        if [1, 6, 9, 14, 54, 57, 60].contains(&month) {
            for _ in 0..4 {
                let mut resumed = sim.clone();
                while resumed.state.month <= REPEATED_MONTHS {
                    resumed.step().unwrap();
                }
                assert_eq!(resumed.state, expected.state);
                assert_eq!(resumed.reports, expected.reports);
                assert_eq!(resumed.ledger, expected.ledger);
                sim.step().unwrap();
            }
        } else {
            sim.run_months(1).unwrap();
        }
    }
    assert_eq!(sim.state, expected.state);
    assert_eq!(sim.ledger, expected.ledger);
}

#[test]
fn duplicate_needs_are_rejected_and_shared_consumption_stock_is_not_double_spent() {
    let (mut world, mut state) = with_warmth(false);
    let duplicate = world.participants[0].needs[0].clone();
    world.participants[0].needs.push(duplicate);
    assert!(Simulation::new(world, state.clone(), Backend::Reference).is_err());
    let (mut world, _) = with_warmth(false);
    world
        .definitions
        .iter_mut()
        .find(|d| d.id == USE_FUEL)
        .unwrap()
        .stages[0]
        .entry_inputs[0]
        .resource = GRAIN;
    // Both needs now consume the same stock. Enter their common settlement phase.
    state.phase = Phase::Consumption;
    state.balances.insert((PERSON, GRAIN), 1);
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    assert_eq!(sim.state.balance(PERSON, NUTRITION), 1);
    assert_eq!(sim.state.balance(PERSON, WARMTH), 0);
}

#[test]
fn an_unrelated_active_crop_does_not_explain_a_full_fuel_buffer() {
    let (world, mut state) = with_warmth(false);
    state.balances.insert((PERSON, FUEL), 7);
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.step().unwrap();
    sim.step().unwrap();
    let receipt = sim
        .ledger
        .last()
        .unwrap()
        .receipts
        .iter()
        .find(|r| r.need == Some(WARMTH) && r.definition.is_none())
        .unwrap();
    assert_eq!(receipt.reason, Reason::NoDeficit);
}
