//! Conditional input evidence and bid activation survive every execution boundary.
use economics_compute_smoke::{
    acceptance::{Run, continuation::Case, financing::Stocks, work_discovery},
    compute::Backend,
    model::Phase,
    simulation::Simulation,
};
const MONTHS: u32 = 24;
#[test]
fn metal_only_input_bid_continuation() {
    compare(Stocks::MetalOnly, true);
}
#[test]
fn missing_stocks_input_bid_continuation() {
    compare(Stocks::Neither, true);
}
fn compare(stocks: Stocks, input_bids: bool) {
    let (w, s) = work_discovery::with_input_bids(Case::Baseline, stocks, input_bids).unwrap();
    let mut reference = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut replay_state = s.clone();
    let mut replay_audit = reference.audit.clone();
    while reference.sim.state.month <= MONTHS {
        reference.audit.step(&mut reference.sim).unwrap();
        let batch = reference.sim.ledger.last().unwrap();
        let before = replay_state.clone();
        let mut world = reference.sim.world.clone();
        economics_compute_smoke::demography::replay(
            &mut world,
            &mut replay_state,
            batch,
            Backend::Reference,
            reference.sim.effect_limit,
        )
        .unwrap();
        replay_audit
            .record(&world, &before, batch, &replay_state)
            .unwrap();
        assert_eq!(replay_state, reference.sim.state);
    }
    assert_eq!(replay_audit, reference.audit);
    let mut cpu = Run::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut activation_boundary_checked = false;
    while cpu.sim.state.month <= MONTHS {
        cpu.audit.step(&mut cpu.sim).unwrap();
        // Open activates accepted month-one work and portfolios before Acquire.
        // Reconstruct here as well as every later boundary: no start or quote
        // replacement may be repeated or lost on continuation.
        if cpu.sim.state.month == 2 && cpu.sim.state.phase == Phase::Acquire {
            activation_boundary_checked = true;
        }
        let mut restored = Simulation::new(
            cpu.sim.world.clone(),
            cpu.sim.state.clone(),
            Backend::CubeCpu,
        )
        .unwrap();
        restored.ledger = cpu.sim.ledger;
        restored.reports = cpu.sim.reports;
        restored.effect_limit = cpu.sim.effect_limit;
        cpu.sim = restored;
    }
    assert!(activation_boundary_checked);
    assert_eq!(cpu.sim.world, reference.sim.world);
    assert_eq!(cpu.sim.state, reference.sim.state);
    assert_eq!(cpu.sim.ledger, reference.sim.ledger);
    assert_eq!(cpu.sim.reports, reference.sim.reports);
    assert_eq!(cpu.audit, reference.audit);
    let mut shuffled = w;
    shuffled.agents.reverse();
    shuffled.participants.reverse();
    shuffled.definitions.reverse();
    shuffled.resources.reverse();
    shuffled.employment.reverse();
    for venue in &mut shuffled.marketplaces {
        venue.markets.reverse();
    }
    let mut reordered = Run::new(shuffled, s, Backend::Reference).unwrap();
    reordered.through(MONTHS).unwrap();
    assert_eq!(reordered.sim.state.phase, Phase::Open);
    assert_eq!(reordered.sim.state, reference.sim.state);
    assert_eq!(reordered.sim.world.agency, reference.sim.world.agency);
    assert_eq!(reordered.sim.ledger, reference.sim.ledger);
    assert_eq!(reordered.audit, reference.audit);
}
