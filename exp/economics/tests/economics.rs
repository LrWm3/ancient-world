use economics_compute_smoke::{
    compute::Backend,
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
use std::collections::BTreeMap;

fn run(name: &str, backend: Backend) -> Simulation {
    let (world, state) = named(name).unwrap();
    let mut sim = Simulation::new(world, state, backend).unwrap();
    sim.run_months(SCENARIO_MONTHS).unwrap();
    sim
}
fn has_reason(sim: &Simulation, reason: Reason) -> bool {
    sim.ledger
        .iter()
        .flat_map(|b| &b.receipts)
        .any(|r| r.reason == reason)
}
fn effect_total(sim: &Simulation, resource: ResourceId, positive: bool) -> i32 {
    sim.ledger
        .iter()
        .filter(|b| b.phase != Phase::Open)
        .flat_map(|b| &b.transactions)
        .flat_map(|t| &t.effects)
        .filter(|e| e.account.1 == resource && (e.delta > 0) == positive)
        .map(|e| e.delta)
        .sum()
}

#[test]
fn baseline_has_exact_timing_balances_and_nonaccumulating_capacities() {
    let sim = run("baseline", Backend::Reference);
    assert_eq!(
        sim.reports
            .iter()
            .map(|r| r.balances[&GRAIN])
            .collect::<Vec<_>>(),
        [4, 3, 2, 1, 0, 7, 6, 5, 4]
    );
    assert!(
        sim.reports
            .iter()
            .all(|r| r.fulfilled(NUTRITION) == 1 && r.deficit(NUTRITION) == 0)
    );
    assert_eq!(sim.state.balance(PERSON, SEED), 0);
    assert_eq!(effect_total(&sim, LABOR, false), -8);
    assert_eq!(effect_total(&sim, SEED, false), -1);
    assert_eq!(effect_total(&sim, GRAIN, true), 8);
    assert_eq!(
        sim.reports
            .iter()
            .map(|r| r.balances[&LABOR])
            .collect::<Vec<_>>(),
        [0, 1, 1, 1, 1, 0, 2, 2, 2]
    );
    let crop: Vec<_> = sim
        .state
        .processes
        .values()
        .filter(|p| p.definition == GROW)
        .collect();
    assert_eq!(crop.len(), 1);
    assert_eq!(crop[0].status, Status::Completed);
    assert_eq!(
        (crop[0].start, crop[0].reserved_through, crop[0].beneficiary),
        (1, 6, PERSON)
    );
    assert_eq!(sim.world.assets[0].owner, STATE_AGENT);
    let output_months: Vec<_> = sim
        .ledger
        .iter()
        .filter(|b| {
            b.transactions
                .iter()
                .flat_map(|t| &t.effects)
                .any(|e| e.account.1 == GRAIN && e.delta > 0)
        })
        .map(|b| b.month)
        .collect();
    assert_eq!(output_months, [6]);
}

#[test]
fn shortages_and_failures_have_distinct_observable_causes() {
    let short = run("short-food", Backend::Reference);
    assert_eq!(
        short
            .reports
            .iter()
            .filter(|r| r.deficit(NUTRITION) > 0)
            .map(|r| r.month)
            .collect::<Vec<_>>(),
        [3, 4, 5]
    );
    assert_eq!(short.state.balance(PERSON, GRAIN), 4);
    for (name, reason) in [
        ("no-seed", Reason::MissingStock),
        ("no-right", Reason::MissingRight),
        ("short-right", Reason::RightTooShort),
        ("disabled", Reason::Disabled),
        ("no-need", Reason::NoDeficit),
    ] {
        let sim = run(name, Backend::Reference);
        assert!(has_reason(&sim, reason), "{name}");
        assert_eq!(effect_total(&sim, GRAIN, true), 0, "{name}");
        assert!(
            !sim.state.processes.values().any(|p| p.definition == GROW),
            "{name}"
        );
    }
    let missed = run("missed-work", Backend::Reference);
    assert_eq!(effect_total(&missed, LABOR, false), -3);
    assert_eq!(effect_total(&missed, GRAIN, true), 0);
    assert_eq!(missed.state.balance(PERSON, SEED), 0);
    assert!(
        missed
            .state
            .processes
            .values()
            .filter(|p| p.definition == GROW)
            .all(|p| p.status == Status::Aborted)
    );
    assert!(
        !missed
            .state
            .processes
            .values()
            .any(|p| p.status == Status::Active)
    );
    assert!(has_reason(&missed, Reason::InsufficientCapacity));
}

#[test]
fn priorities_change_allocations_without_changing_timing_or_wasting_partial_grants() {
    let continuing = run("continuing-first", Backend::Reference);
    let new = run("new-first", Backend::Reference);
    assert_eq!(effect_total(&continuing, GRAIN, true), 8);
    assert_eq!(continuing.state.balance(PERSON, REPAIR_OUTPUT), 0);
    assert_eq!(effect_total(&new, GRAIN, true), 0);
    assert_eq!(new.state.balance(PERSON, REPAIR_OUTPUT), 1);
    assert_eq!(new.reports[5].balances[&LABOR], 1); // unusable partial harvest grant stays unspent
    let refused = new
        .ledger
        .iter()
        .filter(|b| b.month == 6)
        .flat_map(|b| &b.receipts)
        .find(|r| r.definition == Some(GROW))
        .unwrap();
    assert_eq!(
        (refused.requested, refused.allocated, refused.completed),
        (2, 0, 0)
    );
}

#[test]
fn overlapping_rights_and_duplicate_requests_do_not_multiply_land() {
    let (mut world, mut state) = baseline();
    world.agents.push(Agent {
        id: 99,
        name: "Another person".into(),
    });
    world.participants.push(Participant {
        agent: 99,
        ..world.participants[0].clone()
    });
    world.rights.push(UseRight {
        id: 2,
        holder: 99,
        output_owner: 99,
        ..world.rights[0].clone()
    });
    state.balances.insert((99, GRAIN), 5);
    state.balances.insert((99, SEED), 1);
    world.scheduled_starts.push(ScheduledStart {
        month: 1,
        agent: PERSON,
        definition: GROW,
    });
    let mut sim = Simulation::new(world.clone(), state.clone(), Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(
        sim.state
            .processes
            .values()
            .filter(|p| p.status == Status::Active)
            .count(),
        1
    );
    assert_eq!(sim.state.balance(99, SEED), 1);
    assert!(has_reason(&sim, Reason::Occupied));
    world.agents.reverse();
    world.participants.reverse();
    world.rights.reverse();
    world.definitions.reverse();
    world.resources.reverse();
    let mut reordered = Simulation::new(world, state, Backend::Reference).unwrap();
    reordered.run_months(1).unwrap();
    assert_eq!(sim.state, reordered.state);
    assert_eq!(sim.ledger, reordered.ledger);
}

#[test]
fn changing_catalog_ids_names_and_duration_does_not_require_an_occupation_branch() {
    let (mut world, mut state) = baseline();
    world.definitions[0].id = 44;
    world.definitions[0].name = "alternative conversion".into();
    world.definitions[0].stages[1].months = 2;
    world.definitions[0].outputs[0].quantity = 10;
    world.agents[1].name = "Unlabelled decision maker".into();
    state.balances.insert((PERSON, GRAIN), 3);
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    sim.run_months(4).unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 9);
    assert!(sim.reports.iter().all(|r| r.deficit(NUTRITION) == 0));
    assert_eq!(
        sim.state
            .processes
            .values()
            .find(|p| p.definition == 44)
            .unwrap()
            .reserved_through,
        4
    );
}

#[test]
fn cubecl_cpu_matches_reference_for_every_scenario_and_replay() {
    for name in [
        "baseline",
        "short-food",
        "no-seed",
        "no-right",
        "short-right",
        "missed-work",
        "no-need",
        "disabled",
        "continuing-first",
        "new-first",
    ] {
        let cpu = run(name, Backend::CubeCpu);
        let reference = run(name, Backend::Reference);
        assert_eq!(cpu.state, reference.state, "{name}");
        assert_eq!(cpu.ledger, reference.ledger, "{name}");
        assert_eq!(cpu.reports, reference.reports, "{name}");
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
        // Independent raw-effect accounting, without CSR grouping/reduction.
        let (_, initial) = named(name).unwrap();
        let mut balances = initial.balances;
        for batch in &cpu.ledger {
            let mut sums: BTreeMap<Account, i64> = BTreeMap::new();
            for e in batch.transactions.iter().flat_map(|t| &t.effects) {
                *sums.entry(e.account).or_default() += i64::from(e.delta);
            }
            for (key, delta) in sums {
                let value = i64::from(balances.get(&key).copied().unwrap_or(0)) + delta;
                balances.insert(key, i32::try_from(value).unwrap());
            }
        }
        assert_eq!(balances, cpu.state.balances);
    }
}

#[test]
fn monthly_batching_and_every_barrier_checkpoint_preserve_execution() {
    let expected = run("baseline", Backend::Reference);
    let (world, state) = baseline();
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    for _ in 0..SCENARIO_MONTHS {
        sim.run_months(1).unwrap();
    }
    assert_eq!(sim.state, expected.state);
    assert_eq!(sim.ledger, expected.ledger);
    let (world, state) = baseline();
    let mut partial = Simulation::new(world, state, Backend::Reference).unwrap();
    for _ in 0..SCENARIO_MONTHS * 4 {
        let mut resumed = partial.clone();
        while resumed.state.month <= SCENARIO_MONTHS {
            resumed.step().unwrap();
        }
        assert_eq!(resumed.state, expected.state);
        assert_eq!(resumed.ledger, expected.ledger);
        assert_eq!(resumed.reports, expected.reports);
        partial.step().unwrap();
    }
}

#[test]
fn buffer_overflow_and_duplicate_batches_do_not_partially_publish() {
    let (world, state) = baseline();
    let mut sim = Simulation::new(world, state, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    let before = sim.state.clone();
    let ledger_before = sim.ledger.clone();
    sim.effect_limit = 0;
    assert!(sim.step().unwrap_err().contains("capacity"));
    assert_eq!(sim.state, before);
    assert_eq!(sim.ledger, ledger_before);
    sim.effect_limit = DEFAULT_EFFECT_LIMIT;
    sim.step().unwrap();
    let before = sim.state.clone();
    assert!(
        commit(
            &sim.world,
            &mut sim.state,
            sim.ledger.last().unwrap(),
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(sim.state, before);
}

#[test]
fn gross_spending_and_overflow_fail_atomically_even_with_positive_net_effects() {
    let (world, mut state) = baseline();
    let before = state.clone();
    let mut batch = Batch {
        credit: None,
        negotiation: None,
        accept_membership: None,
        household: None,
        maintenance: None,
        decision: None,
        production_plan: None,
        commitments: None,
        accept_access: None,
        access_applicant: None,
        additional_access: vec![],
        additional_memberships: vec![],
        allocation: None,
        pool_market: None,
        plot_request: None,
        id: 0,
        month: 1,
        phase: Phase::Open,
        receipts: vec![],
        transactions: vec![Transaction {
            technique_use: None,
            trade: None,
            stock_trade: None,
            forward: None,
            delivery: None,
            royalty: None,
            cause: "invalid transfer fixture".into(),
            process: None,
            effects: vec![
                Effect {
                    account: (PERSON, GRAIN),
                    delta: 10,
                },
                Effect {
                    account: (PERSON, GRAIN),
                    delta: -6,
                },
                Effect {
                    account: (STATE_AGENT, GRAIN),
                    delta: 6,
                },
            ],
        }],
    };
    assert!(
        commit(
            &world,
            &mut state,
            &batch,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .unwrap_err()
        .contains("gross")
    );
    assert_eq!(state, before);
    batch.transactions[0].effects = vec![Effect {
        account: (PERSON, GRAIN),
        delta: i32::MAX,
    }];
    assert!(
        commit(
            &world,
            &mut state,
            &batch,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .unwrap_err()
        .contains("overflow")
    );
    assert_eq!(state, before);
}

#[test]
fn forged_process_output_or_progress_is_rejected_before_publication() {
    let (world, state) = baseline();
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    sim.step().unwrap();
    let opening = sim.state.clone();
    sim.step().unwrap();
    let mut batch = sim.ledger.last().unwrap().clone();
    batch.transactions[0].effects.push(Effect {
        account: (PERSON, GRAIN),
        delta: 8,
    });
    let mut replay = opening.clone();
    assert!(
        commit(
            &sim.world,
            &mut replay,
            &batch,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, opening);
    batch = sim.ledger.last().unwrap().clone();
    batch.transactions[0].process.as_mut().unwrap().after.stage = 2;
    assert!(
        commit(
            &sim.world,
            &mut replay,
            &batch,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, opening);
}

#[test]
fn output_entitlement_is_distinct_from_use_permission_and_land_ownership() {
    let (mut world, state) = baseline();
    world.rights[0].output_owner = STATE_AGENT;
    // An externally instructed start can produce for a different beneficiary;
    // the person's own need planner must not count that output as its food.
    world.scheduled_starts.push(ScheduledStart {
        month: 1,
        agent: PERSON,
        definition: GROW,
    });
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    sim.run_months(6).unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 8);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    assert_eq!(sim.reports[5].deficit(NUTRITION), 1);
    assert!(has_reason(&sim, Reason::NoBeneficialProcess));
}

#[test]
fn future_harvest_outside_horizon_and_missing_consumption_chain_are_not_hidden() {
    let (mut world, state) = baseline();
    world.horizon = 3;
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    sim.run_months(6).unwrap();
    assert!(!sim.state.processes.values().any(|p| p.definition == GROW));
    assert!(has_reason(&sim, Reason::NoBeneficialProcess));
    let (mut world, state) = baseline();
    world.definitions.retain(|d| d.id != CONSUME);
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert!(has_reason(&sim, Reason::NoKnownChain));
}
