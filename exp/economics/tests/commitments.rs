use economics_compute_smoke::{
    commitments,
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
fn anniversaries_are_fixed_and_independent_of_harvests() {
    let mut sim = new("annual-access", Backend::Reference);
    sim.run_months(12).unwrap();
    assert_eq!(sim.state.month, 13);
    assert!(sim.state.obligations.is_empty());
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 0);
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Due);
    let grain = sim.state.balance(PERSON, GRAIN);
    sim.step().unwrap();
    let o = &sim.state.obligations[&(1, 13)];
    assert_eq!((o.owed, o.paid), (1, 1));
    assert_eq!(sim.state.balance(PERSON, GRAIN), grain - 1);
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 1);
    sim.run_months(48).unwrap();
    assert_eq!(
        sim.state
            .obligations
            .keys()
            .map(|k| k.1)
            .collect::<Vec<_>>(),
        [13, 25, 37, 49]
    );
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 4);

    let mut no_harvest = new("annual-access", Backend::Reference);
    no_harvest.state.month = 13;
    no_harvest.state.balances.insert((PERSON, GRAIN), 0);
    no_harvest.state.balances.insert((PERSON, SEED), 0);
    no_harvest.step().unwrap();
    no_harvest.step().unwrap();
    assert_eq!(no_harvest.state.obligations[&(1, 13)].paid, 0);
    assert!(!commitments::can_start(
        &no_harvest.world,
        &no_harvest.state,
        no_harvest.world.rights[0].id
    ));
}

#[test]
fn arrears_block_only_new_work_and_harvest_cures_after_production() {
    let mut sim = new("annual-arrears", Backend::Reference);
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Productive);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 0);
    let mut blocked = sim.clone();
    blocked.state.processes.clear();
    blocked.state.balances.insert((PERSON, SEED), 1);
    blocked.step().unwrap();
    assert!(
        !blocked
            .state
            .processes
            .values()
            .any(|p| p.definition == GROW)
    );
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::ClearArrears);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 8);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 0);
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 7);
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 1);
    assert!(commitments::can_start(
        &sim.world,
        &sim.state,
        sim.world.rights[0].id
    ));
    assert_eq!(sim.state.phase, Phase::Consumption);
    assert!(
        sim.state
            .processes
            .values()
            .filter(|p| p.definition == GROW)
            .all(|p| p.status == Status::Completed)
    );
    sim.world.scheduled_starts.push(ScheduledStart {
        month: 14,
        agent: PERSON,
        definition: GROW,
    });
    while sim.state.month < 15 {
        sim.step().unwrap();
    }
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.definition == GROW && p.start == 14)
    );
}

#[test]
fn partial_payments_oldest_first_and_failed_settlement_are_atomic() {
    let mut sim = new("annual-access", Backend::Reference);
    sim.state.month = 37;
    sim.state.balances.insert((PERSON, GRAIN), 2);
    sim.step().unwrap();
    let opening = sim.state.clone();
    let expected = commitments::evaluate(&sim.world, &sim.state).unwrap();
    assert_eq!(expected.obligations[&(1, 13)].paid, 1);
    assert_eq!(expected.obligations[&(1, 25)].paid, 1);
    assert_eq!(expected.obligations[&(1, 37)].paid, 0);
    let mut batch = Batch::empty(&sim.state);
    assert!(settle(&mut sim, &batch).is_err());
    batch.transactions = expected.transactions.clone();
    batch.commitments = Some(expected.clone());
    batch.transactions[0].effects[0].delta = 0;
    assert!(settle(&mut sim, &batch).is_err());
    assert_eq!(sim.state, opening);
    batch.transactions = expected.transactions;
    settle(&mut sim, &batch).unwrap();
    let paid = sim.state.clone();
    assert!(settle(&mut sim, &batch).is_err());
    assert_eq!(sim.state, paid);

    let mut partial = new("annual-access", Backend::Reference);
    partial.world.agreements[0].payment.quantity = 3;
    partial.state.month = 13;
    partial.state.balances.insert((PERSON, GRAIN), 2);
    partial.step().unwrap();
    partial.step().unwrap();
    assert_eq!(partial.state.obligations[&(1, 13)].paid, 2);
    assert_eq!(partial.state.balance(PERSON, GRAIN), 0);
}

#[test]
fn due_forecasts_cpu_replay_batching_and_checkpoints_agree() {
    for &name in AGREEMENT_SCENARIOS {
        let mut cpu = new(name, Backend::CubeCpu);
        cpu.run_months(60).unwrap();
        let mut reference = new(name, Backend::Reference);
        reference.run_months(60).unwrap();
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
        let mut replay = new(name, Backend::Reference);
        let mut saw_future_due = false;
        let mut checkpoints = Vec::new();
        for batch in &cpu.ledger {
            if let Some(d) = &batch.decision {
                let f = &d.alternatives[d.selected];
                saw_future_due |= f
                    .outcomes
                    .iter()
                    .any(|r| r.obligations.values().any(|o| o.due > batch.month));
                for row in f.outcomes.iter().filter(|r| r.month == batch.month) {
                    assert_eq!(
                        Some(row),
                        cpu.reports
                            .iter()
                            .find(|r| r.month == row.month && r.agent == row.agent)
                    );
                }
            }
            if batch.month == 13 {
                checkpoints.push(replay.state.clone());
            }
            settle(&mut replay, batch).unwrap();
        }
        assert_eq!(replay.state, cpu.state);
        if name == "annual-access" {
            assert!(saw_future_due);
        }
        for state in checkpoints {
            let mut resumed =
                Simulation::new(cpu.world.clone(), state, Backend::Reference).unwrap();
            while resumed.state.month < cpu.state.month {
                resumed.step().unwrap();
            }
            assert_eq!(resumed.state, cpu.state);
        }
        let mut monthly = new(name, Backend::Reference);
        for _ in 0..60 {
            monthly.run_months(1).unwrap();
        }
        assert_eq!(monthly.state, cpu.state);
        assert_eq!(monthly.ledger, cpu.ledger);
    }
}

#[test]
fn expired_access_retains_old_debt_but_stops_new_billing() {
    let mut sim = new("annual-access", Backend::Reference);
    sim.world.rights[0].through = 13;
    sim.state.month = 25;
    sim.state.balances.insert((PERSON, GRAIN), 0);
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.obligations.len(), 1);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 0);
    let (mut w, s) = named("annual-access").unwrap();
    w.agreements[0].payment.quantity = 0;
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
}
