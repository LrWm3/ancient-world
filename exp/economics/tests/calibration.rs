use economics_compute_smoke::{
    calibration,
    compute::Backend,
    model::*,
    production_market::{self as pm, CounterpartyExpectation, Persistence},
    scenario::{FUEL, GRAIN, NUTRITION, TOKEN, WARMTH},
    simulation::Simulation,
    town_market,
};

#[test]
fn directed_exchange_is_viable_with_finite_budgets_beyond_initial_buffers() {
    for trading in [false, true] {
        let (mut w, s) = calibration::scenario(trading);
        assert_eq!(w.participants.len(), 2);
        assert_eq!(
            w.town_market.as_ref().unwrap().order_horizon,
            town_market::OrderHorizon::Aligned(6)
        );
        calibration::directed(&mut w);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        sim.run_months(72).unwrap();
        let deficit = |r| sim.reports.iter().map(|p| p.deficit(r)).sum::<i32>();
        if trading {
            assert_eq!((deficit(NUTRITION), deficit(WARMTH)), (0, 0));
            assert!(
                sim.state
                    .town_market
                    .history
                    .iter()
                    .filter(|r| r.month > 60)
                    .any(|r| r.markets[&pm::WOOD_MARKET].volume > 0)
            );
        } else {
            assert!(deficit(NUTRITION) > 0 && deficit(WARMTH) > 0);
        }
        assert_eq!(
            sim.state.balance(88, TOKEN) + sim.state.balance(91, TOKEN),
            48
        );
        if trading {
            assert!(
                sim.state
                    .processes
                    .values()
                    .all(|p| p.status != Status::Aborted)
            );
        }
        for a in [88, 91] {
            assert!(sim.state.balance(a, GRAIN) >= 0 && sim.state.balance(a, FUEL) >= 0);
        }
    }
}

#[test]
fn variants_use_only_prior_plans_and_preserve_cpu_checkpoint_ordering() {
    let (mut w, s) = calibration::scenario(true);
    let c = w.production_market.as_mut().unwrap();
    c.counterparties = CounterpartyExpectation::LastPublishedPlan;
    c.persistence = Persistence::Hold { months: 3 };
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    reference.run_months(3).unwrap();
    let history = &reference.state.town_market.history;
    assert!(
        history[0]
            .planning
            .as_ref()
            .unwrap()
            .counterparties
            .is_empty()
    );
    for pair in history.windows(2) {
        let previous = pair[0].planning.as_ref().unwrap();
        let current = pair[1].planning.as_ref().unwrap();
        for p in &previous.people {
            let observed = &current.counterparties[&p.agent];
            assert_eq!(observed.month, previous.month);
            assert_eq!(observed.choice, p.alternatives[p.selected].choice);
        }
    }
    w.participants.reverse();
    w.definitions.reverse();
    w.resources.reverse();
    w.town_market.as_mut().unwrap().traders.reverse();
    w.town_market.as_mut().unwrap().additional[0]
        .traders
        .reverse();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    for _ in 0..3 {
        cpu.step().unwrap();
        cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
            .unwrap()
            .state;
        cpu.run_months(1).unwrap();
    }
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.ledger, cpu.ledger);
    assert_eq!(reference.reports, cpu.reports);
    let mut invalid = cpu.state.clone();
    let d = invalid
        .town_market
        .history
        .last_mut()
        .unwrap()
        .planning
        .as_mut()
        .unwrap();
    d.counterparties.values_mut().next().unwrap().month = d.month;
    assert!(Simulation::new(cpu.world.clone(), invalid, Backend::Reference).is_err());
}

#[test]
fn invalid_hold_duration_is_rejected_and_default_remains_monthly() {
    let (mut w, s) = calibration::scenario(true);
    assert_eq!(
        w.production_market.as_ref().unwrap().persistence,
        Persistence::Monthly
    );
    for months in [0, 13] {
        w.production_market.as_mut().unwrap().persistence = Persistence::Hold { months };
        assert!(Simulation::new(w.clone(), s.clone(), Backend::Reference).is_err());
    }
}
