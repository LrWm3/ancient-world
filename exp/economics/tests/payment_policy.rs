use economics_compute_smoke::{
    commitments::{self, PaymentPolicy},
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
fn run(name: &str, backend: Backend) -> Simulation {
    let mut sim = new(name, backend);
    sim.run_months(60).unwrap();
    sim
}
fn settle(sim: &mut Simulation, b: &Batch) -> Result<(), String> {
    commit(
        &sim.world,
        &mut sim.state,
        b,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
}

#[test]
fn identical_opening_due_allocations_protect_only_current_essential_stock() {
    let mut debt = new("payment-trap-debt", Backend::Reference);
    let mut protected = new("payment-trap-protected", Backend::Reference);
    assert_eq!(debt.state, protected.state);
    debt.step().unwrap();
    protected.step().unwrap();
    assert_eq!(debt.state, protected.state);
    let d = commitments::evaluate(&debt.world, &debt.state).unwrap();
    let p = commitments::evaluate(&protected.world, &protected.state).unwrap();
    assert_eq!(d.obligations[&(1, 13)].owed, p.obligations[&(1, 13)].owed);
    assert_eq!(
        (d.obligations[&(1, 13)].paid, p.obligations[&(1, 13)].paid),
        (1, 0)
    );
    assert_eq!(p.protected[&(PERSON, GRAIN)], 1);
    protected.step().unwrap();
    assert_eq!(protected.state.balance(PERSON, GRAIN), 1);
    assert_eq!(protected.state.balance(STATE_AGENT, GRAIN), 0);
    protected.step().unwrap();
    assert!(
        protected
            .ledger
            .last()
            .unwrap()
            .receipts
            .iter()
            .any(|r| r.reason == Reason::UnpaidObligation)
    );
    protected.step().unwrap(); // ClearArrears cannot spend the same protected food.
    assert_eq!(protected.state.balance(PERSON, GRAIN), 1);
    protected.step().unwrap();
    assert_eq!(protected.state.balance(PERSON, GRAIN), 0);
    assert_eq!(protected.state.balance(PERSON, NUTRITION), 1);

    let mut abundant = new("payment-trap-protected", Backend::Reference);
    abundant.state.balances.insert((PERSON, GRAIN), 10);
    abundant.step().unwrap();
    let p = commitments::evaluate(&abundant.world, &abundant.state).unwrap();
    abundant.world.payment_policy = PaymentPolicy::DebtFirst;
    let d = commitments::evaluate(&abundant.world, &abundant.state).unwrap();
    assert_eq!(p.obligations, d.obligations);
    assert_eq!(p.transactions, d.transactions);
}

#[test]
fn generic_need_recipes_fulfillment_and_inactive_controls_bound_protection() {
    let mut sim = new("payment-protected", Backend::Reference);
    sim.state.balances.insert((PERSON, GRAIN), 10);
    // Two essential requirements competing for one stock add their consumption demand.
    sim.world
        .definitions
        .iter_mut()
        .find(|d| d.id == USE_FUEL)
        .unwrap()
        .stages[0]
        .entry_inputs = vec![Amount::new(GRAIN, 1)];
    assert_eq!(
        commitments::protected_stock(&sim.world, &sim.state).unwrap()[&(PERSON, GRAIN)],
        2
    );
    sim.state.balances.insert((PERSON, NUTRITION), 1);
    assert_eq!(
        commitments::protected_stock(&sim.world, &sim.state).unwrap()[&(PERSON, GRAIN)],
        1
    );
    sim.world.participants[0]
        .needs
        .iter_mut()
        .find(|n| n.resource == WARMTH)
        .unwrap()
        .quantity = 0;
    assert!(
        commitments::protected_stock(&sim.world, &sim.state)
            .unwrap()
            .is_empty()
    );
    sim.state.balances.insert((PERSON, NUTRITION), 0);
    sim.world
        .condition_rules
        .retain(|r| r.provision != NUTRITION);
    assert!(
        commitments::protected_stock(&sim.world, &sim.state)
            .unwrap()
            .is_empty()
    );
    let mut missing = new("payment-protected", Backend::Reference);
    missing
        .world
        .definitions
        .iter_mut()
        .find(|d| d.id == CONSUME)
        .unwrap()
        .enabled = false;
    assert!(
        !commitments::protected_stock(&missing.world, &missing.state)
            .unwrap()
            .contains_key(&(PERSON, GRAIN))
    );
    // Integer lot rounding and caps preserve available stock, not imaginary supply.
    let mut lots = new("payment-protected", Backend::Reference);
    let d = lots
        .world
        .definitions
        .iter_mut()
        .find(|d| d.id == CONSUME)
        .unwrap();
    d.stages[0].entry_inputs[0].quantity = 2;
    d.outputs[0].quantity = 3;
    lots.world.participants[0]
        .needs
        .iter_mut()
        .find(|n| n.resource == NUTRITION)
        .unwrap()
        .quantity = 4;
    assert_eq!(
        commitments::protected_stock(&lots.world, &lots.state).unwrap()[&(PERSON, GRAIN)],
        4
    );
    lots.state.balances.insert((PERSON, GRAIN), 1);
    assert_eq!(
        commitments::protected_stock(&lots.world, &lots.state).unwrap()[&(PERSON, GRAIN)],
        1
    );
}

#[test]
fn forged_policy_or_reserve_is_rejected_and_harvest_can_cure_after_protection() {
    let mut sim = new("annual-arrears", Backend::Reference);
    sim.world.payment_policy = PaymentPolicy::ProtectEssentials;
    sim.step().unwrap();
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::ClearArrears);
    let expected = commitments::evaluate(&sim.world, &sim.state).unwrap();
    assert_eq!(expected.protected[&(PERSON, GRAIN)], 1);
    assert_eq!(expected.obligations[&(1, 13)].paid, 1);
    let mut b = Batch::empty(&sim.state);
    b.transactions = expected.transactions.clone();
    b.commitments = Some(expected.clone());
    let opening = sim.state.clone();
    b.commitments.as_mut().unwrap().policy = PaymentPolicy::DebtFirst;
    assert!(settle(&mut sim, &b).is_err());
    assert_eq!(sim.state, opening);
    b.commitments = Some(expected);
    b.commitments.as_mut().unwrap().protected.clear();
    assert!(settle(&mut sim, &b).is_err());
    assert_eq!(sim.state, opening);
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 7);
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 1);
}

#[test]
fn paired_outcomes_forecasts_cpu_replay_and_continuation_are_consistent() {
    for &name in PAYMENT_SCENARIOS {
        let cpu = run(name, Backend::CubeCpu);
        let reference = run(name, Backend::Reference);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
        let mut replay = new(name, Backend::Reference);
        let mut checkpoints = Vec::new();
        for b in &cpu.ledger {
            if b.month == 13 && matches!(b.phase, Phase::Due | Phase::ClearArrears) {
                checkpoints.push(replay.state.clone());
            }
            if let Some(d) = &b.decision {
                for row in d.alternatives[d.selected]
                    .outcomes
                    .iter()
                    .filter(|r| r.month == b.month)
                {
                    assert_eq!(
                        Some(row),
                        cpu.reports
                            .iter()
                            .find(|r| r.month == row.month && r.agent == row.agent)
                    );
                }
            }
            settle(&mut replay, b).unwrap();
        }
        assert_eq!(replay.state, cpu.state);
        for state in checkpoints {
            let mut resumed =
                Simulation::new(cpu.world.clone(), state, Backend::Reference).unwrap();
            while resumed.state.month < cpu.state.month {
                resumed.step().unwrap();
            }
            assert_eq!(resumed.state, cpu.state);
        }
        let mut monthly = new(name, Backend::Reference);
        monthly.world.participants[0].needs.reverse();
        monthly.world.definitions.reverse();
        for _ in 0..60 {
            monthly.run_months(1).unwrap();
        }
        assert_eq!(monthly.state, cpu.state);
        assert_eq!(monthly.ledger, cpu.ledger);
        let deficits: i32 = cpu.reports.iter().map(|r| r.deficit(NUTRITION)).sum();
        let active_arrears = cpu
            .reports
            .iter()
            .filter(|r| r.terminal.is_none() && r.obligations.values().any(|o| o.paid < o.owed))
            .count();
        let blocks = cpu
            .ledger
            .iter()
            .flat_map(|b| &b.receipts)
            .filter(|r| r.reason == Reason::UnpaidObligation)
            .count();
        if name.contains("trap") {
            assert_eq!(deficits, 6);
            let protected = name.ends_with("protected");
            assert_eq!(
                cpu.state.terminal[&PERSON].month,
                if protected { 19 } else { 18 }
            );
            assert_eq!(
                cpu.state.balance(STATE_AGENT, GRAIN),
                if protected { 0 } else { 1 }
            );
            assert_eq!(active_arrears, if protected { 6 } else { 0 });
            assert_eq!(blocks, if protected { 7 } else { 0 });
        } else {
            // Dated candidate pressure now plants before the bill instead of
            // leaving four avoidable food deficits in this feasible fixture.
            assert_eq!(deficits, 0);
            assert_eq!(active_arrears, 0);
            assert_eq!(blocks, 0);
            assert!(cpu.state.terminal.is_empty());
            assert_eq!(cpu.state.balance(STATE_AGENT, GRAIN), 4);
        }
    }
}
