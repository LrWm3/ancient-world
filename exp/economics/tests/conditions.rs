use economics_compute_smoke::{
    compute::Backend,
    maintenance::{self, Condition},
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn simulation(name: &str, backend: Backend) -> Simulation {
    let (world, state) = named(name).unwrap();
    Simulation::new(world, state, backend).unwrap()
}
fn run(name: &str, backend: Backend) -> Simulation {
    let mut sim = simulation(name, backend);
    sim.run_months(REPEATED_MONTHS).unwrap();
    sim
}
fn capacity_at(sim: &Simulation, month: u32, subject: AgentId, resource: ResourceId) -> i32 {
    sim.reports
        .iter()
        .find(|r| r.month == month && r.agent == subject)
        .unwrap()
        .balances
        .get(&resource)
        .copied()
        .unwrap_or(0)
}

#[test]
fn severity_duration_saturation_and_recovery_are_separate() {
    let (world, _) = named("institution-upkeep").unwrap();
    let rule = &world.condition_rules[0];
    let first = maintenance::advance(rule, &Condition::default(), 10, 8).unwrap();
    assert_eq!(
        first,
        Condition {
            deprivation: 4,
            adverse_months: 1
        }
    );
    let second = maintenance::advance(rule, &first, 10, 8).unwrap();
    assert_eq!(
        second,
        Condition {
            deprivation: 8,
            adverse_months: 2
        }
    );
    // Meeting demand repairs one point; extreme surplus does not bank satisfaction.
    assert_eq!(
        maintenance::advance(rule, &second, 10, 10),
        maintenance::advance(rule, &second, 10, 1000)
    );
    let recovered = maintenance::advance(
        rule,
        &Condition {
            deprivation: 1,
            adverse_months: 4,
        },
        1,
        1,
    )
    .unwrap();
    assert_eq!(recovered, Condition::default());
    let inactive = maintenance::advance(rule, &first, 0, 0).unwrap();
    assert_eq!(inactive.deprivation, 4);
    assert_eq!(inactive.adverse_months, 2);
    assert!(maintenance::advance(rule, &first, -1, 0).is_err());
    assert!(
        maintenance::advance(
            rule,
            &Condition {
                deprivation: u32::MAX,
                adverse_months: 1
            },
            1,
            0
        )
        .is_err()
    );
}

#[test]
fn institution_impairment_and_dissolution_use_next_month_capacity() {
    let sim = run("institution-upkeep", Backend::Reference);
    assert_eq!(sim.reports[2].conditions[&UPKEEP].deprivation, 2);
    assert_eq!(sim.reports[3].conditions[&UPKEEP].deprivation, 4);
    assert_eq!(capacity_at(&sim, 4, INSTITUTION, ADMINISTRATION), 4);
    assert_eq!(capacity_at(&sim, 5, INSTITUTION, ADMINISTRATION), 2);
    let terminal = &sim.state.terminal[&INSTITUTION];
    assert_eq!((terminal.month, terminal.state.as_str()), (8, "dissolved"));
    assert_eq!(capacity_at(&sim, 8, INSTITUTION, ADMINISTRATION), 2);
    assert_eq!(capacity_at(&sim, 9, INSTITUTION, ADMINISTRATION), 0);
    assert_eq!(sim.reports[7].deficit(UPKEEP), 1);
    assert_eq!(sim.reports[8].needs[&UPKEEP].desired, 0);
    assert_eq!(
        sim.state.conditions[&(INSTITUTION, UPKEEP)].adverse_months,
        6
    );
    assert_eq!(
        sim.ledger
            .iter()
            .filter_map(|b| b.maintenance.as_ref())
            .flat_map(|m| &m.transitions)
            .count(),
        1
    );
    assert!(
        sim.ledger
            .iter()
            .filter(|b| b.month > 8)
            .all(|b| b.transactions.is_empty() || b.phase == Phase::Open)
    );
}

#[test]
fn finite_resupply_recovers_condition_and_capability_without_erasing_history() {
    let sim = run("institution-recovery", Backend::Reference);
    assert!(sim.state.terminal.is_empty());
    assert_eq!(
        sim.reports.iter().map(|r| r.deficit(UPKEEP)).sum::<i32>(),
        2
    );
    assert_eq!(capacity_at(&sim, 5, INSTITUTION, ADMINISTRATION), 2);
    assert_eq!(capacity_at(&sim, 6, INSTITUTION, ADMINISTRATION), 4);
    assert_eq!(
        sim.reports[4].conditions[&UPKEEP],
        Condition {
            deprivation: 3,
            adverse_months: 3
        }
    );
    assert_eq!(sim.reports[7].conditions[&UPKEEP], Condition::default());
    assert_eq!(sim.state.balance(INSTITUTION, SUPPLY_RESERVE), 0);
    assert_eq!(sim.state.balance(INSTITUTION, UPKEEP_SUPPLY), 4);
}

#[test]
fn satisfied_controls_remain_healthy_and_bad_priorities_now_have_consequences() {
    for name in ["conditions-warmth-food-first", "institution-supplied"] {
        let sim = run(name, Backend::Reference);
        assert!(sim.state.terminal.is_empty());
        assert!(
            sim.state
                .conditions
                .values()
                .all(|c| c == &Condition::default())
        );
    }
    for (name, month, cause) in [
        ("conditions-warmth-first", 11, NUTRITION),
        ("conditions-repeated-no-seed", 11, NUTRITION),
        ("conditions-warmth-no-wood", 7, WARMTH),
    ] {
        let sim = run(name, Backend::Reference);
        let terminal = &sim.state.terminal[&PERSON];
        assert_eq!(
            (terminal.month, terminal.reason, terminal.state.as_str()),
            (month, cause, "dead")
        );
        assert!(
            sim.state
                .processes
                .values()
                .all(|p| p.status != Status::Active)
        );
        assert_eq!(sim.state.balance(PERSON, LABOR), 0);
        assert!(
            sim.ledger
                .iter()
                .filter(|b| b.month > month)
                .flat_map(|b| &b.transactions)
                .all(|t| t
                    .process
                    .as_ref()
                    .is_none_or(|c| c.after.status == Status::Aborted))
        );
    }
}

#[test]
fn modifiers_combine_by_minimum_and_cannot_restore_terminal_capacity() {
    let (mut world, mut state) = named("conditions-warmth-food-first").unwrap();
    for rule in &world.condition_rules {
        state.conditions.insert(
            (PERSON, rule.provision),
            Condition {
                deprivation: 4,
                adverse_months: 2,
            },
        );
    }
    assert_eq!(maintenance::capacity(&world, &state, PERSON, LABOR, 4), 2);
    world.condition_rules[1].retained_capacity_permille = 250;
    assert_eq!(maintenance::capacity(&world, &state, PERSON, LABOR, 4), 1);
    // Resource matching prevents a labor penalty from modifying an unrelated capacity.
    assert_eq!(maintenance::capacity(&world, &state, PERSON, 999, 4), 4);
    assert_eq!(maintenance::capacity(&world, &state, PERSON, LABOR, 3), 0);
    let dead = run("conditions-warmth-first", Backend::Reference);
    assert_eq!(
        maintenance::capacity(&dead.world, &dead.state, PERSON, LABOR, i32::MAX),
        0
    );
}

#[test]
fn provisions_reference_completed_consumption_not_owned_stock() {
    let sim = run("conditions-warmth-food-first", Backend::Reference);
    for batch in &sim.ledger {
        if let Some(m) = &batch.maintenance {
            assert_eq!(m.provisions.len(), 2);
            for p in &m.provisions {
                let process = &sim.state.processes[&p.source_process];
                assert_eq!(process.status, Status::Completed);
                assert_eq!(process.reserved_through, p.month);
                assert_eq!(
                    sim.world.definition(process.definition).execution,
                    Execution::Consumption
                );
            }
        }
    }
    let mut missing_chain = simulation("conditions-warmth-food-first", Backend::Reference);
    missing_chain
        .world
        .definitions
        .iter_mut()
        .find(|d| d.id == USE_FUEL)
        .unwrap()
        .enabled = false;
    missing_chain.run_months(1).unwrap();
    assert!(missing_chain.state.balance(PERSON, FUEL) > 0);
    assert_eq!(
        missing_chain.state.conditions[&(PERSON, WARMTH)].deprivation,
        2
    );
}

#[test]
fn close_rejects_missing_forged_or_duplicate_effects_atomically() {
    let mut sim = simulation("institution-upkeep", Backend::Reference);
    sim.run_months(4).unwrap();
    let batch = sim.ledger.last().unwrap().clone();
    let mut before = simulation("institution-upkeep", Backend::Reference);
    for b in &sim.ledger[..sim.ledger.len() - 1] {
        commit(
            &before.world,
            &mut before.state,
            b,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
    }
    for variant in 0..4 {
        let mut forged = batch.clone();
        match variant {
            0 => forged.maintenance = None,
            1 => {
                forged.maintenance.as_mut().unwrap().changes[0]
                    .after
                    .deprivation = 0
            }
            2 => {
                let c = forged.maintenance.as_ref().unwrap().changes[0].clone();
                forged.maintenance.as_mut().unwrap().changes.push(c);
            }
            _ => forged.maintenance.as_mut().unwrap().changes[0].supplied = 1,
        }
        let mut state = before.state.clone();
        assert!(
            commit(
                &before.world,
                &mut state,
                &forged,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(state, before.state);
    }
    let state = sim.state.clone();
    assert!(
        commit(
            &sim.world,
            &mut sim.state,
            &batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(sim.state, state);
}

#[test]
fn invalid_rules_and_duplicate_requirement_bindings_are_rejected() {
    let (world, state) = named("institution-upkeep").unwrap();
    for variant in 0..5 {
        let mut bad = world.clone();
        match variant {
            0 => bad.condition_rules.push(bad.condition_rules[0].clone()),
            1 => bad.condition_rules[0].terminal_at = 1,
            2 => bad.condition_rules[0].retained_capacity_permille = 1001,
            3 => bad.condition_rules[0].provision = UPKEEP_SUPPLY,
            _ => bad.condition_rules[0].affected_capacity = UPKEEP,
        }
        assert!(Simulation::new(bad, state.clone(), Backend::Reference).is_err());
    }
}

#[test]
fn all_condition_scenarios_match_cpu_replay_batching_and_boundary_checkpoints() {
    for &name in CONDITION_SCENARIOS {
        let reference = run(name, Backend::Reference);
        let cpu = run(name, Backend::CubeCpu);
        assert_eq!(cpu.state, reference.state, "{name}");
        assert_eq!(cpu.ledger, reference.ledger, "{name}");
        assert_eq!(cpu.reports, reference.reports, "{name}");
        let mut monthly = simulation(name, Backend::Reference);
        for _ in 0..REPEATED_MONTHS {
            monthly.run_months(1).unwrap();
        }
        assert_eq!(monthly.state, reference.state);
        assert_eq!(monthly.reports, reference.reports);
        let (world, mut state) = named(name).unwrap();
        for b in &cpu.ledger {
            commit(
                &world,
                &mut state,
                b,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT,
            )
            .unwrap();
        }
        assert_eq!(state, reference.state);
        let mut incremental = simulation(name, Backend::Reference);
        // Covers pre/post impairment, resupply, terminal transitions and cleanup.
        while incremental.state.month <= 12 {
            let mut resumed = incremental.clone();
            while resumed.state.month <= REPEATED_MONTHS {
                resumed.step().unwrap();
            }
            assert_eq!(resumed.state, reference.state);
            assert_eq!(resumed.reports, reference.reports);
            incremental.step().unwrap();
        }
        let mut reversed = simulation(name, Backend::Reference);
        reversed.world.condition_rules.reverse();
        reversed.world.resources.reverse();
        reversed.world.definitions.reverse();
        for p in &mut reversed.world.participants {
            p.needs.reverse();
        }
        reversed.run_months(REPEATED_MONTHS).unwrap();
        assert_eq!(reversed.ledger, reference.ledger);
        assert_eq!(reversed.state, reference.state);
    }
}

#[test]
fn terminal_operators_cannot_replay_even_capacity_free_consumption() {
    let mut sim = run("conditions-warmth-first", Backend::Reference);
    sim.step().unwrap(); // Open expires fulfillment and supplies zero capacity.
    sim.step().unwrap(); // Productive barrier.
    assert_eq!(sim.state.phase, Phase::Consumption);
    assert!(sim.state.balance(PERSON, FUEL) > 0);
    let before = sim.state.clone();
    let batch = Batch {
        negotiation: None,
        accept_membership: None,
        household: None,
        id: before.next_batch,
        month: before.month,
        phase: Phase::Consumption,
        receipts: vec![],
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
        transactions: vec![Transaction {
            technique_use: None,
            trade: None,
            stock_trade: None,
            forward: None,
            delivery: None,
            royalty: None,
            cause: "invalid consumption after terminal transition".into(),
            effects: vec![
                Effect {
                    account: (PERSON, FUEL),
                    delta: -1,
                },
                Effect {
                    account: (PERSON, WARMTH),
                    delta: 1,
                },
            ],
            process: Some(ProcessChange {
                before: None,
                after: ProcessInstance {
                    id: before.processes.keys().next_back().unwrap() + 1,
                    definition: USE_FUEL,
                    operator: PERSON,
                    beneficiary: PERSON,
                    goal: Some(WARMTH),
                    asset: None,
                    right: None,
                    start: before.month,
                    reserved_through: before.month,
                    stage: 0,
                    elapsed: 1,
                    status: Status::Completed,
                },
            }),
        }],
    };
    assert!(
        commit(
            &sim.world,
            &mut sim.state,
            &batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(sim.state, before);
}
