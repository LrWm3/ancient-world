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
fn acquire(sim: &mut Simulation) {
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
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
fn acceptance_grants_access_and_dates_payment_from_actual_activation() {
    let mut sim = new("offer-useful", Backend::Reference);
    let right = sim.world.rights[0].id;
    assert!(!commitments::can_start(&sim.world, &sim.state, right));
    sim.world.priority = Priority::ContinuingFirst;
    sim.run_months(1).unwrap(); // No automatic acceptance in static policy.
    assert!(sim.state.accepted_agreements.is_empty());
    assert!(!sim.state.processes.values().any(|p| p.definition == GROW));
    acquire(&mut sim);
    let mut b = Batch::empty(&sim.state);
    b.accept_access = Some(1);
    settle(&mut sim, &b).unwrap();
    assert_eq!(sim.state.accepted_agreements[&1].activated, 2);
    assert!(commitments::can_start(&sim.world, &sim.state, right));
    let after = sim.state.clone();
    assert!(settle(&mut sim, &b).is_err());
    assert_eq!(after, sim.state);
    while sim.state.month < 14 {
        sim.step().unwrap();
    }
    assert!(sim.state.obligations.is_empty());
    sim.step().unwrap();
    sim.step().unwrap();
    assert!(sim.state.obligations.contains_key(&(1, 14)));
    assert!(!sim.state.obligations.contains_key(&(1, 13)));
}

#[test]
fn short_horizon_misses_price_difference_long_horizon_sees_first_bill() {
    let mut short = new("offer-short", Backend::Reference);
    let mut long = new("offer-long", Backend::Reference);
    assert_eq!(short.world.horizon, long.world.horizon);
    for sim in [&mut short, &mut long] {
        acquire(sim);
        sim.step().unwrap();
    }
    assert!(short.state.accepted_agreements.contains_key(&1));
    assert!(long.state.accepted_agreements.contains_key(&2));
    let sd = short.ledger.last().unwrap().decision.as_ref().unwrap();
    let ld = long.ledger.last().unwrap().decision.as_ref().unwrap();
    assert_eq!((sd.through, ld.through), (6, 18));
    let best = |d: &economics_compute_smoke::planning::Decision, id| {
        d.alternatives
            .iter()
            .filter(|a| a.access_offer == Some(id))
            .map(|a| a.score.clone())
            .min()
            .unwrap()
    };
    assert_eq!(best(sd, 1), best(sd, 2)); // Tie breaks by ID, not a claim expensive is better.
    assert!(best(ld, 2) < best(ld, 1));
    assert!(
        ld.alternatives[ld.selected]
            .outcomes
            .iter()
            .any(|r| r.obligations.contains_key(&(2, 13)))
    );
    assert!(
        sd.alternatives
            .iter()
            .all(|a| a.outcomes.iter().all(|r| r.obligations.is_empty()))
    );
    assert!(ld.alternatives.iter().any(|a| a.access_offer.is_none()));
    let mut no_seed = new("offer-no-seed", Backend::Reference);
    acquire(&mut no_seed);
    no_seed.step().unwrap();
    assert!(no_seed.state.accepted_agreements.is_empty());
    assert!(
        no_seed
            .ledger
            .last()
            .unwrap()
            .decision
            .as_ref()
            .unwrap()
            .alternatives
            .iter()
            .any(|a| a.access_offer == Some(1))
    );
}

#[test]
fn incompatible_duplicate_invalid_and_out_of_phase_acceptances_publish_nothing() {
    let mut sim = new("offer-short", Backend::Reference);
    let mut b = Batch::empty(&sim.state);
    b.accept_access = Some(1);
    let opening = sim.state.clone();
    assert!(settle(&mut sim, &b).is_err());
    assert_eq!(sim.state, opening);
    acquire(&mut sim);
    b = Batch::empty(&sim.state);
    b.accept_access = Some(999);
    let opening = sim.state.clone();
    assert!(settle(&mut sim, &b).is_err());
    assert_eq!(sim.state, opening);
    b.accept_access = Some(1);
    settle(&mut sim, &b).unwrap();
    sim.world.priority = Priority::ContinuingFirst;
    while sim.state.month == 1 {
        sim.step().unwrap();
    }
    acquire(&mut sim);
    for id in [1, 2] {
        let opening = sim.state.clone();
        b = Batch::empty(&sim.state);
        b.accept_access = Some(id);
        assert!(settle(&mut sim, &b).is_err());
        assert_eq!(sim.state, opening);
    }
}

#[test]
fn cpu_forecasts_replay_and_acceptance_checkpoints_preserve_contracts() {
    for &name in ACCESS_SCENARIOS {
        let mut reference = new(name, Backend::Reference);
        reference.run_months(60).unwrap();
        let mut cpu = new(name, Backend::CubeCpu);
        cpu.run_months(60).unwrap();
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        assert_eq!(reference.reports, cpu.reports);
        let mut replay = new(name, Backend::Reference);
        let mut checkpoint = None;
        for b in &cpu.ledger {
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
                            .find(|r| r.agent == row.agent && r.month == row.month)
                    );
                }
            }
            settle(&mut replay, b).unwrap();
            if b.accept_access.is_some() {
                checkpoint = Some(replay.state.clone());
            }
        }
        assert_eq!(replay.state, cpu.state);
        if let Some(state) = checkpoint {
            let mut resumed =
                Simulation::new(cpu.world.clone(), state, Backend::Reference).unwrap();
            while resumed.state.month < cpu.state.month {
                resumed.step().unwrap();
            }
            assert_eq!(resumed.state, cpu.state);
        }
        let mut monthly = new(name, Backend::Reference);
        monthly.world.access_offers.reverse();
        monthly.world.rights.reverse();
        for _ in 0..60 {
            monthly.run_months(1).unwrap();
        }
        assert_eq!(monthly.state, cpu.state);
        assert_eq!(monthly.ledger, cpu.ledger);
    }
}
