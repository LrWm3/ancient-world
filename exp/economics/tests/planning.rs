use economics_compute_smoke::{
    compute::Backend,
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn new(name: &str, backend: Backend) -> Simulation {
    let (world, state) = named(name).unwrap();
    Simulation::new(world, state, backend).unwrap()
}
fn run(name: &str, backend: Backend) -> Simulation {
    let mut sim = new(name, backend);
    sim.run_months(REPEATED_MONTHS).unwrap();
    sim
}
fn first_decision(sim: &mut Simulation) -> Batch {
    assert_eq!(sim.state.phase, Phase::Open);
    sim.step().unwrap();
    sim.step().unwrap();
    sim.ledger.last().unwrap().clone()
}

#[test]
fn forecasting_sustains_repeated_harvests_without_changing_opening_resources() {
    let mut forecast = new("forecast-harvest", Backend::Reference);
    let mut fixed = new("conditions-warmth-first", Backend::Reference);
    assert_eq!(forecast.state, fixed.state);
    let mut same_world = forecast.world.clone();
    same_world.priority = fixed.world.priority;
    assert_eq!(same_world, fixed.world);
    forecast.run_months(60).unwrap();
    fixed.run_months(60).unwrap();
    assert!(forecast.state.terminal.is_empty());
    assert_eq!(fixed.state.terminal[&PERSON].month, 11);
    assert!(
        forecast
            .reports
            .iter()
            .all(|r| r.deficit(NUTRITION) == 0 && r.deficit(WARMTH) == 0)
    );
    assert_eq!(
        forecast
            .state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Completed)
            .map(|p| p.reserved_through)
            .collect::<Vec<_>>(),
        [6, 14, 22, 30, 38, 46, 54]
    );
    let mut replay = new("forecast-harvest", Backend::Reference);
    for b in &forecast.ledger {
        commit(
            &replay.world,
            &mut replay.state,
            b,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
        assert_eq!(
            replay.state.balance(PERSON, SEED)
                + replay
                    .state
                    .processes
                    .values()
                    .filter(|p| p.definition == GROW && p.status == Status::Active)
                    .count() as i32,
            1
        );
    }
}

#[test]
fn severe_cold_reverses_the_choice_but_does_not_restore_lost_seed() {
    let mut forecast = new("forecast-cold", Backend::Reference);
    let fixed = new("static-cold", Backend::Reference);
    assert_eq!(forecast.state, fixed.state);
    let batch = first_decision(&mut forecast);
    assert!(
        batch
            .receipts
            .iter()
            .any(|r| r.definition == Some(PREPARE_FUEL) && r.completed == 1)
    );
    assert!(batch.transactions.iter().any(|t| {
        t.process
            .as_ref()
            .is_some_and(|p| p.after.definition == GROW && p.after.status == Status::Aborted)
    }));
    let d = batch.decision.unwrap();
    let selected = &d.alternatives[d.selected];
    assert_eq!(selected.plan.work.priority, Priority::NeedFirstFor(WARMTH));
    assert_eq!(selected.score.terminal_months, 0);
    assert!(d.alternatives.iter().any(|a| a.score.terminal_months > 0));
    let forecast = run("forecast-cold", Backend::Reference);
    let fixed = run("static-cold", Backend::Reference);
    assert_eq!(
        (
            fixed.state.terminal[&PERSON].month,
            fixed.state.terminal[&PERSON].reason
        ),
        (6, WARMTH)
    );
    assert_eq!(
        (
            forecast.state.terminal[&PERSON].month,
            forecast.state.terminal[&PERSON].reason
        ),
        (31, NUTRITION)
    );
    assert_eq!(forecast.state.balance(PERSON, SEED), 0);
}

#[test]
fn resupply_forecast_retains_recovery_and_cannot_magic_away_scarcity() {
    let mut recovery = new("forecast-resupply", Backend::Reference);
    let initial = first_decision(&mut recovery);
    let decision = initial.decision.unwrap();
    let predicted = &decision.alternatives[decision.selected].outcomes;
    assert_eq!(predicted.iter().map(|r| r.deficit(UPKEEP)).sum::<i32>(), 2);
    assert_eq!(predicted[4].conditions[&UPKEEP].deprivation, 3);
    let recovery = run("forecast-resupply", Backend::Reference);
    let fixed = run("institution-recovery", Backend::Reference);
    assert_eq!(recovery.state, fixed.state);
    assert_eq!(recovery.reports, fixed.reports);
    assert_eq!(
        recovery
            .state
            .processes
            .values()
            .filter(|p| p.definition == 2)
            .count(),
        1
    );
    let mut scarcity = new("forecast-scarcity", Backend::Reference);
    let initial = first_decision(&mut scarcity);
    assert!(
        initial
            .decision
            .unwrap()
            .alternatives
            .iter()
            .all(|a| a.score.terminal_months > 0)
    );
    let scarcity = run("forecast-scarcity", Backend::Reference);
    let fixed = run("static-scarcity", Backend::Reference);
    assert_eq!(scarcity.state, fixed.state);
    assert_eq!(scarcity.state.terminal[&PERSON].month, 6);
    assert!(
        scarcity
            .ledger
            .iter()
            .filter(|b| b.month > 6)
            .all(|b| b.decision.is_none())
    );
}

#[test]
fn forecasts_do_not_read_future_fixture_shocks_but_use_current_observed_capacity() {
    let mut baseline = new("forecast-harvest", Backend::Reference);
    let mut surprise = baseline.clone();
    surprise.world.capacity_overrides.insert((2, PERSON), 0);
    surprise.world.scheduled_starts.push(ScheduledStart {
        month: 3,
        agent: PERSON,
        definition: REPAIR,
    });
    let expected = first_decision(&mut baseline);
    let actual = first_decision(&mut surprise);
    assert_eq!(actual, expected);
    while surprise.state.month <= 2 {
        surprise.step().unwrap();
    }
    let chosen = actual.decision.as_ref().unwrap();
    assert_ne!(
        chosen.alternatives[chosen.selected].outcomes[1],
        surprise.reports[1]
    );
    // Opening overrides are already observed, so a zero current budget cannot fund planting.
    let mut observed = new("forecast-harvest", Backend::Reference);
    observed.world.capacity_overrides.insert((1, PERSON), 0);
    let batch = first_decision(&mut observed);
    assert!(batch.receipts.iter().all(|r| r.completed == 0));
}

#[test]
fn bounded_search_rejects_unsupported_scope_and_failed_forecasts_publish_nothing() {
    let mut sim = new("forecast-harvest", Backend::Reference);
    sim.world
        .participants
        .push(sim.world.participants[0].clone());
    assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
    let mut sim = new("forecast-harvest", Backend::Reference);
    sim.step().unwrap();
    sim.effect_limit = 0;
    let before = sim.clone();
    assert!(sim.step().is_err());
    assert_eq!(sim.state, before.state);
    assert_eq!(sim.ledger, before.ledger);
    assert_eq!(sim.reports, before.reports);
}

#[test]
fn cpu_predictions_match_current_outcomes_and_replay_for_every_new_scenario() {
    for &name in PLANNING_SCENARIOS {
        let reference = run(name, Backend::Reference);
        let cpu = run(name, Backend::CubeCpu);
        assert_eq!(cpu.state, reference.state, "{name}");
        assert_eq!(cpu.ledger, reference.ledger, "{name}");
        assert_eq!(cpu.reports, reference.reports, "{name}");
        let mut replay = new(name, Backend::Reference);
        for batch in &cpu.ledger {
            if let Some(d) = &batch.decision {
                let chosen = &d.alternatives[d.selected];
                assert_eq!(chosen.first_work, batch.receipts);
                assert!(d.alternatives.iter().all(|a| chosen.score <= a.score));
                for predicted in chosen.outcomes.iter().filter(|r| r.month == batch.month) {
                    assert_eq!(
                        Some(predicted),
                        cpu.reports
                            .iter()
                            .find(|r| r.month == predicted.month && r.agent == predicted.agent)
                    );
                }
                assert!(d.alternatives.len() <= 10);
            }
            commit(
                &replay.world,
                &mut replay.state,
                batch,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT,
            )
            .unwrap();
        }
        assert_eq!(replay.state, cpu.state);
    }
}

#[test]
fn replanning_preserves_batching_checkpoints_and_catalog_order() {
    for name in [
        "forecast-harvest",
        "forecast-cold",
        "forecast-resupply",
        "forecast-scarcity",
    ] {
        let expected = run(name, Backend::Reference);
        let mut monthly = new(name, Backend::Reference);
        for _ in 0..60 {
            monthly.run_months(1).unwrap();
        }
        assert_eq!(monthly.state, expected.state);
        assert_eq!(monthly.ledger, expected.ledger);
        let mut checkpoint = new(name, Backend::Reference);
        let end = checkpoint.state.month + 60;
        // Includes forecasts, consequences and Open on both sides of the first harvest.
        for _ in 0..28 {
            if matches!(checkpoint.state.month, 1 | 5 | 6 | 7 | 8) {
                let mut resumed = checkpoint.clone();
                while resumed.state.month < end {
                    resumed.step().unwrap();
                }
                assert_eq!(resumed.state, expected.state);
                assert_eq!(resumed.ledger, expected.ledger);
            }
            checkpoint.step().unwrap();
        }
        let mut reversed = new(name, Backend::Reference);
        reversed.world.definitions.reverse();
        reversed.world.condition_rules.reverse();
        reversed.world.resources.reverse();
        reversed.world.participants[0].needs.reverse();
        reversed.run_months(60).unwrap();
        assert_eq!(reversed.state, expected.state);
        assert_eq!(reversed.ledger, expected.ledger);
    }
}

#[test]
fn dated_rent_admits_early_planting_with_identical_opening_resources() {
    let mut contracted = new("annual-access", Backend::Reference);
    contracted.world.decision_horizon = Some(18);
    contracted.state.month = 7;
    contracted.state.balances.insert((PERSON, GRAIN), 7);
    let mut consumption_only = contracted.clone();
    consumption_only.world.agreements.clear();
    assert_eq!(contracted.state, consumption_only.state);
    contracted.run_months(1).unwrap();
    consumption_only.run_months(1).unwrap();
    assert!(
        contracted
            .ledger
            .iter()
            .flat_map(|b| &b.receipts)
            .any(|r| r.definition == Some(GROW) && r.completed > 0)
    );
    assert!(
        !consumption_only
            .ledger
            .iter()
            .flat_map(|b| &b.receipts)
            .any(|r| r.definition == Some(GROW) && r.completed > 0)
    );
    contracted.run_months(6).unwrap();
    assert_eq!(contracted.state.obligations[&(1, 13)].paid, 1);
    assert!(
        contracted
            .reports
            .iter()
            .all(|r| r.deficit(NUTRITION) == 0 && r.deficit(WARMTH) == 0)
    );
}
