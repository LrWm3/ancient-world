use economics_compute_smoke::{
    allocation::{Outcome, Policy},
    compute::Backend,
    intermediary::{self, Experiment, MAKE_TOOL, TOOL_KIND},
    model::*,
    scenario::*,
    settlement,
};
fn acquire(e: &mut Experiment) {
    while e.simulation.state.phase != Phase::Acquire {
        e.step().unwrap();
    }
}
fn harvest(e: &Experiment, agent: u32) -> i32 {
    e.simulation
        .ledger
        .iter()
        .flat_map(|b| &b.transactions)
        .filter(|t| {
            t.process.as_ref().is_some_and(|p| {
                p.after.definition == GROW
                    && p.after.operator == agent
                    && p.after.status == Status::Completed
            })
        })
        .flat_map(|t| &t.effects)
        .filter(|e| e.account == (agent, GRAIN) && e.delta > 0)
        .map(|e| e.delta)
        .sum()
}
#[test]
fn competing_projections_reserve_one_tool_and_loser_replans_before_cpu_execution() {
    let mut e = intermediary::scenario(Backend::CubeCpu).unwrap();
    acquire(&mut e);
    let opening = e.simulation.state.clone();
    let (batch, round) = intermediary::prepare(&e.simulation, e.policy, e.seed).unwrap();
    assert_eq!(e.simulation.state, opening);
    assert!(
        round
            .decisions
            .iter()
            .all(|d| d.selected == Some(MAKE_TOOL))
    );
    assert_eq!(round.decisions[1].fallback, Some(PREPARE_FUEL));
    assert_eq!(round.resolution.receipts[0].outcome, Outcome::Reserved(1));
    assert_eq!(
        round.resolution.receipts[1].outcome,
        Outcome::InsufficientCapacity
    );
    assert_eq!(
        round.resolution.shortfalls[&u64::from(PERSON + 1)][0].account,
        (STATE_AGENT, RAW_WOOD)
    );
    e.step().unwrap(); // admission reserves dated work, never materializes projected tool
    assert!(e.simulation.state.equipment.is_empty());
    let pending = e.simulation.state.pending_production.as_ref().unwrap();
    assert_eq!(Some(pending), batch.production_plan.as_ref());
    let mut forged = *pending.clone();
    forged.transactions.clear();
    let before = e.simulation.state.clone();
    assert!(
        settlement::commit(
            &e.simulation.world,
            &mut e.simulation.state,
            &forged,
            Backend::CubeCpu,
            e.simulation.effect_limit
        )
        .is_err()
    );
    assert_eq!(e.simulation.state, before);
    e.step().unwrap();
    assert_eq!(e.simulation.state.equipment.len(), 1);
    let tool = e.simulation.state.equipment.values().next().unwrap();
    assert_eq!(
        (tool.owner, tool.kind, tool.remaining_uses),
        (PERSON, TOOL_KIND, 2)
    );
    assert_eq!(e.simulation.state.balance(PERSON, LABOR), 0);
    assert_eq!(e.simulation.state.balance(PERSON + 1, LABOR), 1);
    assert_eq!(e.simulation.state.balance(STATE_AGENT, RAW_WOOD), 0);
    assert_eq!(e.simulation.state.balance(PERSON + 1, FUEL), 4);
    assert!(
        e.simulation
            .state
            .processes
            .values()
            .filter(|p| p.definition == GROW)
            .all(|p| p.status == Status::Active)
    );
    let after = e.simulation.state.clone();
    assert!(
        settlement::commit(
            &e.simulation.world,
            &mut e.simulation.state,
            &forged,
            Backend::CubeCpu,
            e.simulation.effect_limit
        )
        .is_err()
    );
    assert_eq!(e.simulation.state, after);
    while e.simulation.state.month < 7 {
        e.step().unwrap();
    }
    assert_eq!(harvest(&e, PERSON), 16);
    assert_eq!(harvest(&e, PERSON + 1), 8);
    assert_eq!(
        e.simulation
            .state
            .equipment
            .values()
            .next()
            .unwrap()
            .remaining_uses,
        1
    );
}

#[test]
fn cold_or_unhelpful_tool_changes_the_choice_and_ample_wood_admits_both() {
    for control in ["cold", "unhelpful", "ample"] {
        let mut e = intermediary::scenario(Backend::Reference).unwrap();
        match control {
            "cold" => {
                for p in &e.simulation.world.participants {
                    e.simulation.state.balances.insert((p.agent, FUEL), 0);
                }
            }
            "unhelpful" => {
                let t = e
                    .simulation
                    .world
                    .techniques
                    .iter_mut()
                    .find(|t| t.equipment_kind == Some(TOOL_KIND))
                    .unwrap();
                t.output_multiplier = 1;
                t.services = vec![Amount::new(LABOR, 2)];
            }
            _ => {
                e.simulation
                    .world
                    .pools
                    .iter_mut()
                    .find(|p| p.account == (STATE_AGENT, RAW_WOOD))
                    .unwrap()
                    .capacity = 4;
                e.simulation
                    .state
                    .balances
                    .insert((STATE_AGENT, RAW_WOOD), 4);
            }
        }
        acquire(&mut e);
        e.step().unwrap();
        let r = &e.rounds[0];
        if control == "ample" {
            assert_eq!(
                r.resolution
                    .receipts
                    .iter()
                    .filter(|r| r.outcome == Outcome::Reserved(1))
                    .count(),
                2
            );
            assert!(r.decisions.iter().all(|d| d.selected == Some(MAKE_TOOL)));
        } else {
            if control == "cold" {
                assert!(r.decisions.iter().all(|d| d.selected == Some(PREPARE_FUEL)));
            }
            assert!(
                r.decisions.iter().all(|d| d.selected != Some(MAKE_TOOL)),
                "{control}: {:?}",
                r.decisions
            );
        }
    }
}

#[test]
fn cpu_reference_checkpoint_order_and_seeded_ranking_preserve_outcomes() {
    let mut cpu = intermediary::scenario(Backend::CubeCpu).unwrap();
    let mut reference = cpu.clone();
    reference.simulation.backend = Backend::Reference;
    reference.simulation.world.participants.reverse();
    reference.simulation.world.agents.reverse();
    cpu.run_months(6).unwrap();
    reference.run_months(6).unwrap();
    assert_eq!(cpu.simulation.state, reference.simulation.state);
    assert_eq!(cpu.simulation.ledger, reference.simulation.ledger);
    assert_eq!(cpu.rounds, reference.rounds);
    let mut checkpoint = cpu.clone();
    cpu.run_months(3).unwrap();
    for _ in 0..3 {
        checkpoint.run_months(1).unwrap();
    }
    assert_eq!(cpu.simulation.state, checkpoint.simulation.state);
    assert_eq!(cpu.simulation.ledger, checkpoint.simulation.ledger);
    assert_eq!(cpu.rounds, checkpoint.rounds);
    let mut e = intermediary::scenario(Backend::Reference).unwrap();
    acquire(&mut e);
    let stable = intermediary::prepare(&e.simulation, Policy::StablePriority, 7)
        .unwrap()
        .1;
    let mut different = false;
    for seed in 0..8 {
        let lottery = intermediary::prepare(&e.simulation, Policy::Lottery, seed)
            .unwrap()
            .1;
        for (a, b) in stable.decisions.iter().zip(&lottery.decisions) {
            assert_eq!(a.alternatives, b.alternatives);
            assert_eq!(a.selected, b.selected);
        }
        different |=
            lottery.resolution.receipts[0].claim.id != stable.resolution.receipts[0].claim.id;
    }
    assert!(different);
}
