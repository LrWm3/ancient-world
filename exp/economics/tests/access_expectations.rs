use economics_compute_smoke::{
    access_expectations::{Estimate, MEMORY_MONTHS, Memory, Mode, Observation},
    compute::Backend,
    intermediary,
    model::Phase,
    scenario::*,
};
fn row(month: u32, requested: u32, received: u32) -> Observation {
    Observation {
        month,
        agent: PERSON,
        account: (STATE_AGENT, RAW_WOOD),
        requested,
        received,
    }
}
#[test]
fn losses_successes_expiry_and_inactivity_have_distinct_meanings() {
    let account = (STATE_AGENT, RAW_WOOD);
    let mut m = Memory::default();
    m.record(1, vec![row(1, 2, 0)]).unwrap();
    assert_eq!(
        m.estimate(1, PERSON, account),
        Estimate {
            received: 2,
            requested: 2
        }
    ); // no same-month evidence
    assert_eq!(
        m.estimate(2, PERSON, account),
        Estimate {
            received: 2,
            requested: 4
        }
    );
    m.record(2, vec![row(2, 2, 2)]).unwrap();
    assert_eq!(
        m.estimate(3, PERSON, account),
        Estimate {
            received: 4,
            requested: 6
        }
    );
    assert_eq!(
        m.estimate(3, PERSON + 1, account),
        Estimate {
            received: 2,
            requested: 2
        }
    );
    assert_eq!(
        m.estimate(3, PERSON, (STATE_AGENT, GRAIN)),
        Estimate {
            received: 2,
            requested: 2
        }
    );
    m.record(3, vec![]).unwrap();
    assert_eq!(
        m.estimate(4, PERSON, account),
        m.estimate(3, PERSON, account)
    );
    assert_eq!(
        m.estimate(3 + MEMORY_MONTHS, PERSON, account),
        Estimate {
            received: 2,
            requested: 2
        }
    );
    let before = m.clone();
    assert!(m.record(3, vec![]).is_err());
    assert_eq!(m, before);
    assert!(m.record(4, vec![row(4, 1, 2)]).is_err());
    assert_eq!(m, before);
    assert!(m.record(4, vec![row(4, 1, 1), row(4, 1, 1)]).is_err());
    assert_eq!(m, before);
}
#[test]
fn fractional_access_keeps_one_unit_flows_possible_and_avoids_overflow() {
    let e = Estimate {
        received: 3,
        requested: 4,
    };
    let mut carry = 0;
    let portions: Vec<_> = (0..4).map(|_| e.portion(1, &mut carry).unwrap()).collect();
    assert_eq!(portions, vec![0, 1, 1, 1]);
    assert_eq!(carry, 0);
    let e = Estimate {
        received: u64::MAX - 1,
        requested: u64::MAX,
    };
    assert_eq!(e.portion(i32::MAX, &mut carry).unwrap(), i32::MAX - 1);
    assert!(
        Estimate {
            received: 0,
            requested: 0
        }
        .portion(1, &mut 0)
        .is_err()
    );
}
fn acquire(e: &mut intermediary::Experiment) {
    while e.simulation.state.phase != Phase::Acquire {
        e.step().unwrap();
    }
}
#[test]
fn learning_waits_for_completion_counts_fallback_once_and_changes_only_forecasts() {
    let mut e = intermediary::scenario(Backend::CubeCpu).unwrap();
    e.access_mode = Mode::Learned;
    acquire(&mut e);
    e.step().unwrap();
    assert!(e.access_memory.observations.is_empty());
    let checkpoint = e.clone();
    let mut invalid = e.clone();
    invalid.access_memory.through = Some(invalid.simulation.state.month);
    let untouched = invalid.simulation.state.clone();
    let ledger = invalid.simulation.ledger.clone();
    assert!(invalid.step().is_err());
    assert_eq!(invalid.simulation.state, untouched);
    assert_eq!(invalid.simulation.ledger, ledger);
    e.step().unwrap();
    assert_eq!(e.access_memory.observations.len(), 2);
    let loss = e
        .access_memory
        .observations
        .iter()
        .find(|r| r.agent == PERSON + 1)
        .unwrap();
    assert_eq!((loss.requested, loss.received), (2, 1));
    let mut resumed = checkpoint;
    resumed.step().unwrap();
    assert_eq!(e.access_memory, resumed.access_memory);
    assert_eq!(e.simulation.state, resumed.simulation.state);
    acquire(&mut e);
    let before = e.clone();
    let (_, optimistic) = intermediary::prepare_with_access(
        &e.simulation,
        e.policy,
        e.seed,
        Mode::Optimistic,
        &e.access_memory,
    )
    .unwrap();
    let (_, learned) = intermediary::prepare_with_access(
        &e.simulation,
        e.policy,
        e.seed,
        Mode::Learned,
        &e.access_memory,
    )
    .unwrap();
    assert_eq!(
        learned.expectations[&(PERSON + 1, (STATE_AGENT, RAW_WOOD))],
        Estimate {
            received: 3,
            requested: 4
        }
    );
    assert_eq!(e.access_memory, before.access_memory);
    assert_eq!(e.simulation.state, before.simulation.state);
    let mut changed = optimistic.decisions[1].alternatives != learned.decisions[1].alternatives;
    for _ in 0..7 {
        e.run_months(1).unwrap();
        acquire(&mut e);
        let (_, optimistic) = intermediary::prepare_with_access(
            &e.simulation,
            e.policy,
            e.seed,
            Mode::Optimistic,
            &e.access_memory,
        )
        .unwrap();
        let (_, learned) = intermediary::prepare_with_access(
            &e.simulation,
            e.policy,
            e.seed,
            Mode::Learned,
            &e.access_memory,
        )
        .unwrap();
        changed |= optimistic.decisions[1].alternatives != learned.decisions[1].alternatives;
    }
    assert!(changed);
}
#[test]
fn learned_cpu_reference_order_batching_and_checkpoint_agree() {
    let mut a = intermediary::scenario(Backend::CubeCpu).unwrap();
    a.access_mode = Mode::Learned;
    let mut b = a.clone();
    b.simulation.backend = Backend::Reference;
    b.simulation.world.participants.reverse();
    b.simulation.world.agents.reverse();
    a.run_months(6).unwrap();
    for _ in 0..6 {
        b.run_months(1).unwrap();
    }
    assert_eq!(a.simulation.state, b.simulation.state);
    assert_eq!(a.simulation.ledger, b.simulation.ledger);
    assert_eq!(a.rounds, b.rounds);
    assert_eq!(a.access_memory, b.access_memory);
    let mut resumed = a.clone();
    a.run_months(3).unwrap();
    resumed.run_months(3).unwrap();
    assert_eq!(a.simulation.state, resumed.simulation.state);
    assert_eq!(a.access_memory, resumed.access_memory);
}
#[test]
fn successful_access_control_preserves_tools_and_outcomes() {
    let mut a = intermediary::scenario(Backend::Reference).unwrap();
    let pool = a
        .simulation
        .world
        .pools
        .iter_mut()
        .find(|p| p.account == (STATE_AGENT, RAW_WOOD))
        .unwrap();
    pool.capacity = 6;
    pool.monthly_regeneration = 3;
    a.simulation
        .state
        .balances
        .insert((STATE_AGENT, RAW_WOOD), 6);
    let mut b = a.clone();
    b.access_mode = Mode::Learned;
    a.run_months(12).unwrap();
    b.run_months(12).unwrap();
    assert_eq!(a.simulation.state, b.simulation.state);
    assert_eq!(a.simulation.ledger, b.simulation.ledger);
    assert_eq!(b.simulation.state.equipment.len(), 2);
    assert!(
        b.access_memory
            .observations
            .iter()
            .all(|r| r.requested == r.received)
    );
}
