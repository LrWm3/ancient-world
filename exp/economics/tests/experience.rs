use economics_compute_smoke::{
    compute::Backend,
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn new(name: &str, backend: Backend) -> Simulation {
    let (w, s) = named(name).unwrap();
    Simulation::new(w, s, backend).unwrap()
}

fn settle(sim: &mut Simulation, batch: &Batch) -> Result<(), String> {
    commit(
        &sim.world,
        &mut sim.state,
        batch,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
}

#[test]
fn four_completed_harvests_unlock_only_subsequent_work_and_preserve_tools() {
    for name in ["experience-manual", "experience-tool"] {
        let mut sim = new(name, Backend::Reference);
        sim.run_months(60).unwrap();
        assert!(sim.state.terminal.is_empty());
        assert!(
            sim.reports
                .iter()
                .all(|r| r.deficit(NUTRITION) == 0 && r.deficit(WARMTH) == 0)
        );
        let harvests: Vec<_> = sim
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter(|t| {
                t.process.as_ref().is_some_and(|p| {
                    p.after.definition == GROW && p.after.status == Status::Completed
                })
            })
            .collect();
        assert!(harvests.len() > 4);
        assert_eq!(sim.state.practice[&(PERSON, 1)], harvests.len() as u32);
        assert!(
            harvests[..4]
                .iter()
                .all(|t| t.technique_use.as_ref().is_none_or(|u| u.technique != 2))
        );
        assert!(harvests[4..].iter().all(|t| {
            t.technique_use
                .as_ref()
                .is_some_and(|u| u.technique == 2 && u.asset.is_none())
        }));
        if name == "experience-tool" {
            assert_eq!(sim.state.equipment[&TOOL].remaining_uses, 2);
            assert!(
                harvests[..4]
                    .iter()
                    .all(|t| t.technique_use.as_ref().unwrap().asset == Some(TOOL))
            );
        } else {
            assert!(harvests[..4].iter().all(|t| {
                t.effects
                    .iter()
                    .any(|e| e.account == (PERSON, LABOR) && e.delta == -2)
            }));
        }
    }
}

#[test]
fn idle_failed_and_partial_work_do_not_award_practice() {
    let mut no_seed = new("experience-no-seed", Backend::Reference);
    no_seed.run_months(60).unwrap();
    assert!(no_seed.state.practice.is_empty());
    assert!(!no_seed.state.terminal.is_empty());

    let mut failed = new("experience-tool", Backend::Reference);
    failed.world.capacity_overrides.insert((6, PERSON), 0);
    failed.run_months(1).unwrap();
    assert!(failed.state.practice.is_empty());
    assert!(
        failed
            .state
            .processes
            .values()
            .any(|p| p.status == Status::Aborted)
    );
    let mut partial = new("experience-manual", Backend::Reference);
    // Award planting practice only when its deliberately multi-month stage completes.
    partial.world.practice_rules[0].stage = 0;
    partial
        .world
        .definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .stages[0]
        .months = 2;
    partial.world.horizon = 12;
    partial.run_months(1).unwrap();
    assert!(partial.state.practice.is_empty());
    partial.run_months(1).unwrap();
    assert_eq!(partial.state.practice[&(PERSON, 1)], 1);
}

#[test]
fn competency_validation_overflow_and_forged_eligibility_are_atomic() {
    let mut sim = new("experience-tool", Backend::Reference);
    sim.step().unwrap();
    sim.step().unwrap();
    let mut work = *sim.state.pending_production.clone().unwrap();
    sim.state.pending_production = None;
    let opening = sim.state.clone();
    work.transactions
        .iter_mut()
        .find(|t| t.technique_use.is_some())
        .unwrap()
        .technique_use = Some(economics_compute_smoke::equipment::TechniqueUse {
        technique: 2,
        asset: None,
    });
    assert!(settle(&mut sim, &work).is_err());
    assert_eq!(sim.state, opening);

    let mut sim = new("experience-tool", Backend::Reference);
    sim.state.practice.insert((PERSON, 1), u32::MAX);
    sim.world.priority = Priority::ContinuingFirst;
    sim.world.offers.clear();
    sim.step().unwrap();
    let opening = sim.state.clone();
    assert!(sim.step().is_err());
    assert_eq!(sim.state, opening);

    let (mut w, s) = named("experience-manual").unwrap();
    w.practice_rules[0].points = 0;
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
}

#[test]
fn forecasts_include_learning_and_cpu_replay_and_continuation_agree() {
    for &name in EXPERIENCE_SCENARIOS {
        let mut reference = new(name, Backend::Reference);
        reference.run_months(60).unwrap();
        let mut cpu = new(name, Backend::CubeCpu);
        cpu.run_months(60).unwrap();
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
        let mut replay = new(name, Backend::Reference);
        let mut crossing_checkpoint = None;
        let mut predicted_learning = false;
        for batch in &cpu.ledger {
            if let Some(d) = &batch.decision {
                let selected = &d.alternatives[d.selected];
                for r in selected.outcomes.iter().filter(|r| r.month == batch.month) {
                    assert_eq!(
                        Some(r),
                        cpu.reports
                            .iter()
                            .find(|a| a.month == r.month && a.agent == r.agent)
                    );
                }
                if replay
                    .state
                    .practice
                    .get(&(PERSON, 1))
                    .copied()
                    .unwrap_or(0)
                    < 4
                    && selected
                        .outcomes
                        .iter()
                        .any(|r| r.practice.get(&(PERSON, 1)).copied().unwrap_or(0) >= 4)
                {
                    predicted_learning = true;
                }
            }
            settle(&mut replay, batch).unwrap();
            if replay.state.practice.get(&(PERSON, 1)) == Some(&4) && crossing_checkpoint.is_none()
            {
                crossing_checkpoint = Some(replay.state.clone());
            }
        }
        assert_eq!(replay.state, cpu.state);
        if name != "experience-no-seed" {
            assert!(predicted_learning);
            let mut resumed = Simulation::new(
                cpu.world.clone(),
                crossing_checkpoint.unwrap(),
                Backend::Reference,
            )
            .unwrap();
            while resumed.state.month < cpu.state.month {
                resumed.step().unwrap();
            }
            assert_eq!(resumed.state, cpu.state);
        }
        let mut monthly = new(name, Backend::Reference);
        monthly.world.techniques.reverse();
        for _ in 0..60 {
            monthly.run_months(1).unwrap();
        }
        assert_eq!(monthly.state, cpu.state);
        assert_eq!(monthly.ledger, cpu.ledger);
    }
}
