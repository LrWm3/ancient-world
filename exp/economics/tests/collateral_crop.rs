use economics_compute_smoke::{
    compute::Backend,
    credit::{self, CollateralSettlement, Event},
    model::*,
    scenario::{GRAIN, LABOR, PERSON, PLOT, SEED, STATE_AGENT},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};

fn scenario(maintain: bool, backend: Backend) -> Simulation {
    let (w, s) = credit::crop_scenario(maintain).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn reach_seizure(sim: &mut Simulation) {
    sim.run_months(2).unwrap();
    sim.step().unwrap();
    assert_eq!((sim.state.month, sim.state.phase), (3, Phase::Due));
}
fn receipt(sim: &Simulation) -> Batch {
    let mut b = Batch::empty(&sim.state);
    b.credit = credit::evaluate(&sim.world, &sim.state).unwrap();
    b.transactions = b.credit.as_ref().unwrap().transactions.clone();
    b
}

#[test]
fn seizure_changes_control_but_not_crop_progress_or_fixed_debt_credit() {
    let mut sim = scenario(true, Backend::CubeCpu);
    reach_seizure(&mut sim);
    let original = sim.state.processes.values().next().unwrap().clone();
    assert_eq!((original.stage, original.elapsed), (1, 1));
    assert_eq!(sim.state.balance(PERSON, SEED), 0);
    let labor = sim.state.balance(PERSON, LABOR);
    sim.step().unwrap();
    let p = &sim.state.processes[&original.id];
    let mut expected = original;
    expected.operator = STATE_AGENT;
    expected.beneficiary = STATE_AGENT;
    expected.goal = None;
    assert_eq!(p, &expected);
    assert_eq!(sim.state.balance(PERSON, LABOR), labor);
    assert_eq!(sim.state.balance(PERSON, SEED), 0);
    assert_eq!(sim.state.balance(STATE_AGENT, SEED), 0);
    assert_eq!(
        credit::owner(&sim.world, &sim.state, PLOT),
        Some(STATE_AGENT)
    );
    assert_eq!(sim.state.credit.loans[&1].principal, 2160);
    assert_eq!(
        sim.state.credit.loans[&1]
            .collateral
            .as_ref()
            .unwrap()
            .settlement,
        CollateralSettlement::FixedValue { value: 6000 }
    );
    sim.run_months(4).unwrap();
    assert_eq!(sim.state.processes[&expected.id].status, Status::Completed);
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 8);
    assert_eq!(sim.state.balance(STATE_AGENT, SEED), 1);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    assert_eq!(sim.state.credit.loans[&1].principal, 2160);
}

#[test]
fn neglect_fails_at_execution_not_at_transfer_and_does_not_reprice_collateral() {
    let mut sim = scenario(false, Backend::CubeCpu);
    reach_seizure(&mut sim);
    sim.step().unwrap();
    assert_eq!(
        sim.state.processes.values().next().unwrap().status,
        Status::Active
    );
    sim.step().unwrap(); // Acquire
    sim.step().unwrap(); // Productive: no state labor available
    assert_eq!(
        sim.state.processes.values().next().unwrap().status,
        Status::Aborted
    );
    sim.run_months(4).unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 0);
    assert_eq!(sim.state.balance(STATE_AGENT, SEED), 0);
    assert_eq!(sim.state.credit.loans[&1].principal, 2160);
    assert_eq!(
        sim.ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.events)
            .filter(|e| matches!(e, Event::Enforced { .. }))
            .count(),
        1
    );
}

#[test]
fn tampered_or_missing_attachment_cannot_partially_seize_plot() {
    let mut sim = scenario(true, Backend::CubeCpu);
    reach_seizure(&mut sim);
    let before = sim.state.clone();
    for mode in 0..3 {
        let mut b = receipt(&sim);
        let c = b.credit.as_mut().unwrap();
        match mode {
            0 => c.attachments.clear(),
            1 => c.attachments[0].after.elapsed = 0,
            _ => c.attachments[0].after.beneficiary = PERSON,
        }
        assert!(
            settlement::commit(
                &sim.world,
                &mut sim.state,
                &b,
                Backend::CubeCpu,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(sim.state, before);
    }
}

#[test]
fn monthly_checkpoint_and_reference_agree_for_maintenance_and_neglect() {
    for maintain in [true, false] {
        let mut cpu = scenario(maintain, Backend::CubeCpu);
        cpu.run_months(6).unwrap();
        let mut reference = scenario(maintain, Backend::Reference);
        for _ in 0..6 {
            reference.run_months(1).unwrap();
        }
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        let mut partial = scenario(maintain, Backend::CubeCpu);
        reach_seizure(&mut partial);
        partial.step().unwrap();
        let mut resumed = Simulation::new(
            partial.world.clone(),
            partial.state.clone(),
            Backend::CubeCpu,
        )
        .unwrap();
        resumed.run_months(4).unwrap();
        assert_eq!(cpu.state, resumed.state);
        assert_eq!(&cpu.ledger[partial.ledger.len()..], resumed.ledger);
    }
}

#[test]
fn completed_outputs_stay_with_borrower_and_deferred_seizure_moves_no_crop() {
    let mut completed = scenario(true, Backend::CubeCpu);
    completed.world.definitions[0].stages.truncate(2);
    completed.world.definitions[0].stages[1].months = 1;
    reach_seizure(&mut completed);
    let p = completed.state.processes.values().next().unwrap().clone();
    assert_eq!(p.status, Status::Completed);
    completed.step().unwrap();
    assert_eq!(completed.state.processes[&p.id], p);
    assert_eq!(completed.state.balance(PERSON, GRAIN), 8);
    assert_eq!(completed.state.balance(PERSON, SEED), 1);
    assert!(
        completed
            .ledger
            .last()
            .unwrap()
            .credit
            .as_ref()
            .unwrap()
            .attachments
            .is_empty()
    );

    let mut deferred = scenario(true, Backend::CubeCpu);
    deferred.world.credit.as_mut().unwrap().offers[0]
        .collateral
        .settlement = CollateralSettlement::FixedValue { value: 200000 };
    reach_seizure(&mut deferred);
    let processes = deferred.state.processes.clone();
    deferred.step().unwrap();
    assert_eq!(deferred.state.processes, processes);
    assert_eq!(
        credit::owner(&deferred.world, &deferred.state, PLOT),
        Some(PERSON)
    );
    let boundary = deferred.ledger.last().unwrap().credit.as_ref().unwrap();
    assert!(boundary.attachments.is_empty());
    assert!(
        boundary
            .events
            .iter()
            .any(|e| matches!(e, Event::EnforcementDeferred { .. }))
    );
}
