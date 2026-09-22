use economics_compute_smoke::{
    compute::Backend,
    model::*,
    scenario::{GRAIN, GROW, RAW_WOOD, SEED, STATE_AGENT},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    work_choice,
};
fn scenario(case: &str, backend: Backend) -> Simulation {
    let (w, s) = work_choice::scenario(case).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn after_seizure(s: &mut Simulation) {
    s.run_months(2).unwrap();
    for _ in 0..3 {
        s.step().unwrap();
    }
    assert_eq!((s.state.month, s.state.phase), (3, Phase::Productive));
    assert!(
        s.state.processes.values().any(|p| p.definition == GROW
            && p.operator == STATE_AGENT
            && p.status == Status::Active)
    );
}
#[test]
fn mature_crop_wins_but_remaining_work_makes_early_crop_lose_to_wood() {
    for (case, keep, value, grain, seed, wood) in [
        ("mature", true, 17, 8, 1, 16),
        ("expensive", false, 16, 0, 0, 24),
        ("no-capacity", false, 0, 0, 0, 0),
    ] {
        let mut s = scenario(case, Backend::CubeCpu);
        after_seizure(&mut s);
        let before = s.state.clone();
        let b = work_choice::evaluate(&s).unwrap();
        assert_eq!(s.state, before);
        let d = b.work_choice.as_ref().unwrap();
        let chosen = &d.forecasts[d.selected];
        assert_eq!(chosen.plan.continue_active, keep, "{case}: {d:?}");
        assert_eq!(chosen.net_value, value, "{case}");
        if case == "no-capacity" {
            assert_eq!(chosen.plan.alternative, None);
        }
        s.run_months(4).unwrap();
        assert_eq!(s.state.balance(STATE_AGENT, GRAIN), grain, "{case}");
        assert_eq!(s.state.balance(STATE_AGENT, SEED), seed, "{case}");
        assert_eq!(s.state.balance(STATE_AGENT, RAW_WOOD), wood, "{case}");
        assert_eq!(s.state.credit.loans[&1].principal, 2160);
        let crop = s
            .state
            .processes
            .values()
            .find(|p| p.definition == GROW)
            .unwrap();
        assert_eq!(
            crop.status,
            if keep {
                Status::Completed
            } else {
                Status::Aborted
            }
        );
    }
}
#[test]
fn remaining_inputs_and_output_values_change_the_choice_without_role_checks() {
    let mut s = scenario("mature", Backend::Reference);
    after_seizure(&mut s);
    // Harvest requires inputs not yet paid: profitable output is not feasibility.
    s.world
        .definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .stages[2]
        .entry_inputs = vec![Amount::new(SEED, 2)];
    let b = work_choice::evaluate(&s).unwrap();
    let d = b.work_choice.unwrap();
    assert!(!d.forecasts[d.selected].plan.continue_active);
    // Supplying those inputs makes execution possible, but their opportunity cost
    // lowers finishing value from 17 to 15, below 16 from wood.
    s.state.balances.insert((STATE_AGENT, SEED), 2);
    let d = work_choice::evaluate(&s).unwrap().work_choice.unwrap();
    assert!(!d.forecasts[d.selected].plan.continue_active);
    s.world
        .work_choice
        .as_mut()
        .unwrap()
        .values
        .insert(GRAIN, 2);
    let d = work_choice::evaluate(&s).unwrap().work_choice.unwrap();
    assert!(d.forecasts[d.selected].plan.continue_active);
}
#[test]
fn missing_or_tampered_decision_cannot_publish_work_or_abort_crop() {
    let mut s = scenario("mature", Backend::CubeCpu);
    after_seizure(&mut s);
    let before = s.state.clone();
    for mode in 0..3 {
        let mut b = work_choice::evaluate(&s).unwrap();
        match mode {
            0 => b.work_choice = None,
            1 => b.work_choice.as_mut().unwrap().selected = 0,
            _ => b.transactions.clear(),
        }
        assert!(
            settlement::commit(
                &s.world,
                &mut s.state,
                &b,
                Backend::CubeCpu,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(s.state, before);
    }
}
#[test]
fn reorder_resume_batch_and_reference_preserve_choices_and_outcomes() {
    for case in ["mature", "expensive", "no-capacity"] {
        let mut cpu = scenario(case, Backend::CubeCpu);
        cpu.run_months(6).unwrap();
        let mut reference = scenario(case, Backend::Reference);
        reference.world.agents.reverse();
        reference.world.participants.reverse();
        reference.world.definitions.reverse();
        for _ in 0..6 {
            reference.run_months(1).unwrap();
        }
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        let mut prefix = scenario(case, Backend::CubeCpu);
        after_seizure(&mut prefix);
        let mut resumed =
            Simulation::new(prefix.world.clone(), prefix.state.clone(), Backend::CubeCpu).unwrap();
        resumed.run_months(4).unwrap();
        assert_eq!(cpu.state, resumed.state);
        assert_eq!(&cpu.ledger[prefix.ledger.len()..], resumed.ledger);
    }
}
#[test]
fn forecast_ignores_future_fixture_shocks_and_horizon_limits_are_explicit() {
    let mut s = scenario("mature", Backend::Reference);
    after_seizure(&mut s);
    let b = work_choice::evaluate(&s).unwrap();
    s.world.capacity_overrides.insert((4, STATE_AGENT), 0);
    assert_eq!(b, work_choice::evaluate(&s).unwrap());
    let mut shocked = s.clone();
    shocked.run_months(2).unwrap();
    assert_eq!(
        shocked
            .state
            .processes
            .values()
            .find(|p| p.definition == GROW)
            .unwrap()
            .status,
        Status::Aborted
    );
    let mut unvalued = s.clone();
    unvalued
        .world
        .work_choice
        .as_mut()
        .unwrap()
        .values
        .remove(&SEED);
    assert!(work_choice::evaluate(&unvalued).is_err());
    s.world.work_choice.as_mut().unwrap().horizon = 1;
    let d = work_choice::evaluate(&s).unwrap().work_choice.unwrap();
    assert!(!d.forecasts[d.selected].plan.continue_active);
    s.world.work_choice.as_mut().unwrap().horizon = 0;
    assert!(Simulation::new(s.world, s.state, Backend::Reference).is_err());
}
