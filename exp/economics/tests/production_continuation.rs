use economics_compute_smoke::{
    acceptance::{
        Run, continuation as c,
        financing::{Liquidity, Preferences, Stocks},
        production_preferences::{self, Priority},
    },
    compute::Backend,
    model::Phase,
    simulation::Simulation,
};
const MONTHS: u32 = 24;
#[test]
fn both_stock_flow_continuation() {
    compare(Stocks::Both, false);
}
#[test]
fn metal_only_flow_continuation() {
    compare(Stocks::MetalOnly, false);
}
#[test]
fn ore_only_productive_publication_continuation() {
    compare(Stocks::OreOnly, true);
}
#[test]
fn neither_stock_productive_publication_continuation() {
    compare(Stocks::Neither, true);
}
fn compare(stocks: Stocks, productive: bool) {
    let (mut w, s) = production_preferences::scenario(
        c::Case::Baseline,
        stocks,
        Preferences {
            workshop: Liquidity::MonthlyExposure,
            town: Liquidity::TerminalReserve,
        },
        Priority::ProductionFirst,
    )
    .unwrap();
    if productive {
        w.agency
            .get_mut(&economics_compute_smoke::acceptance::WORKSHOP)
            .unwrap()
            .config
            .publication =
            economics_compute_smoke::agency::publication::Policy::ProductiveAsksOnEqualProjection;
    }
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
    while cpu.sim.state.month <= MONTHS {
        cpu.audit.step(&mut cpu.sim).unwrap();
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
    assert_eq!(reordered.sim.ledger, reference.sim.ledger);
    assert_eq!(reordered.audit, reference.audit);
}
