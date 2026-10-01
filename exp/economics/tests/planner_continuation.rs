#[path = "support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    composition::{
        self, Budget, Scope, Strategy,
        continuation::{Controller, Policy, Reason},
    },
    compute::Backend,
    model::*,
    scenario::*,
    simulation::Simulation,
};

fn budget() -> Budget {
    Budget {
        expansions: 256,
        forecasts: 32,
        months: 12,
    }
}
fn setup(case: &str, backend: Backend, policy: Policy) -> (Simulation, Controller) {
    let (mut sim, scope) = fixtures::fixture(case, backend).unwrap();
    sim.world.priority = Priority::ContinuingFirst;
    (
        sim,
        Controller::new(scope, Strategy::BestFirst, budget(), policy),
    )
}
fn advance(sim: &mut Simulation, controller: &mut Controller, end: u32) {
    while sim.state.month < end {
        controller.step(sim).unwrap();
    }
}

#[test]
fn stable_continuation_matches_selected_forecast_and_reviews_at_horizon() {
    for case in ["B3", "B5", "B5-one-seed"] {
        let (mut sim, mut retain) = setup(case, Backend::Reference, Policy::RetainRepair);
        fixtures::acquire(&mut sim).unwrap();
        let (_, scope) = fixtures::fixture(case, Backend::Reference).unwrap();
        let selected = composition::choose(&sim, &scope, Strategy::BestFirst, budget()).unwrap();
        let (_, _, forecast) = composition::forecast_package(&sim, &selected.requests, 12).unwrap();
        let mut periodic_sim = sim.clone();
        let mut periodic = Controller::new(
            scope,
            Strategy::BestFirst,
            budget(),
            Policy::ScheduledReview,
        );
        advance(&mut sim, &mut retain, 13);
        advance(&mut periodic_sim, &mut periodic, 13);
        assert_eq!(sim.state, forecast.state, "{case}");
        assert_eq!(sim.reports, forecast.reports, "{case}");
        assert_eq!(sim.state, periodic_sim.state);
        assert_eq!(
            retain.history.iter().filter(|r| r.search.is_some()).count(),
            1
        );
        assert!(
            retain.history[1..]
                .iter()
                .all(|r| r.reason == Reason::Retained && !r.observation_changed)
        );
        assert_eq!(retain.history[0].projection_months, 12);
        assert_eq!(
            retain.history[0].projection_steps,
            retain.history[0].replay_steps
        );
        assert!(retain.history[0].projection_steps > 12);
        fixtures::acquire(&mut sim).unwrap();
        retain.step(&mut sim).unwrap();
        assert_eq!(retain.history.last().unwrap().reason, Reason::HorizonEnded);
    }
}

#[test]
fn monthly_control_matches_existing_composition_search() {
    let (mut sim, mut controller) = setup("B3", Backend::Reference, Policy::Monthly);
    let mut direct = sim.clone();
    advance(&mut sim, &mut controller, 13);
    while direct.state.month < 13 {
        if direct.state.phase == Phase::Acquire {
            composition::choose(
                &direct,
                &Scope::Person(PERSON),
                Strategy::BestFirst,
                budget(),
            )
            .unwrap()
            .accept(&mut direct)
            .unwrap();
        } else {
            direct.step().unwrap();
        }
    }
    assert_eq!(sim.state, direct.state);
    assert_eq!(sim.ledger, direct.ledger);
    assert_eq!(controller.history.len(), 12);
    assert!(
        controller
            .history
            .iter()
            .all(|r| r.search.is_some() && r.projection_steps == 0)
    );
}

#[test]
fn future_labor_loss_is_hidden_then_repairs_only_after_observation() {
    let (clean, _) = setup("B3", Backend::Reference, Policy::RetainRepair);
    for policy in [Policy::RetainRepair, Policy::ScheduledReview] {
        let (mut sim, mut controller) = setup("B3", Backend::Reference, policy);
        sim.world.capacity_overrides.insert((6, PERSON), 0);
        let mut baseline = clean.clone();
        let mut normal =
            Controller::new(Scope::Person(PERSON), Strategy::BestFirst, budget(), policy);
        advance(&mut sim, &mut controller, 6);
        advance(&mut baseline, &mut normal, 6);
        assert_eq!(sim.state, baseline.state);
        assert_eq!(
            controller, normal,
            "unobserved labor loss leaked into planning"
        );
        fixtures::acquire(&mut sim).unwrap();
        controller.step(&mut sim).unwrap();
        let receipt = controller.history.last().unwrap();
        assert!(receipt.observation_changed);
        assert_eq!(
            receipt.reason,
            if policy == Policy::RetainRepair {
                Reason::ObservationChanged
            } else {
                Reason::Continued
            }
        );
        assert_eq!(receipt.search.is_some(), policy == Policy::RetainRepair);
        advance(&mut sim, &mut controller, 25);
        assert!(
            sim.state
                .processes
                .values()
                .any(|p| p.status == Status::Aborted)
        );
        assert!(
            controller
                .history
                .iter()
                .filter_map(|r| r.search.as_ref())
                .all(|m| m.forecasts <= 32 && m.expansions <= 256)
        );
    }
}

#[test]
fn lost_stock_repaired_without_reusing_saved_grants() {
    let (mut sim, mut controller) = setup("B3", Backend::Reference, Policy::RetainRepair);
    advance(&mut sim, &mut controller, 4);
    fixtures::acquire(&mut sim).unwrap();
    // An observed fixture intervention, not an accounting transaction.
    sim.state.balances.insert((PERSON, SEED), 0);
    controller.step(&mut sim).unwrap();
    assert_eq!(
        controller.history.last().unwrap().reason,
        Reason::ObservationChanged
    );
    advance(&mut sim, &mut controller, 13);
    assert!(sim.state.balances.values().all(|q| *q >= 0));
}

#[test]
fn cpu_reference_checkpoint_and_financial_audit_agree() {
    fn run(
        sim: &mut Simulation,
        controller: &mut Controller,
        audit: &mut economics_compute_smoke::financial_reporting::Audit,
        end: u32,
    ) {
        while sim.state.month < end {
            let before = sim.state.clone();
            controller.step(sim).unwrap();
            audit
                .record(&sim.world, &before, sim.ledger.last().unwrap(), &sim.state)
                .unwrap();
        }
    }
    let mut results = vec![];
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let (mut sim, mut controller) = setup("B5", backend, Policy::RetainRepair);
        let mut audit = fixtures::audit(&sim).unwrap();
        run(&mut sim, &mut controller, &mut audit, 4);
        // Preserve the controller together with the simulation, including a dated plan.
        while sim.state.phase != Phase::Productive {
            let before = sim.state.clone();
            controller.step(&mut sim).unwrap();
            audit
                .record(&sim.world, &before, sim.ledger.last().unwrap(), &sim.state)
                .unwrap();
        }
        let (mut resumed, mut saved, mut saved_audit) =
            (sim.clone(), controller.clone(), audit.clone());
        run(&mut sim, &mut controller, &mut audit, 15);
        for end in 5..=15 {
            run(&mut resumed, &mut saved, &mut saved_audit, end);
        }
        assert_eq!(sim.state, resumed.state);
        assert_eq!(sim.ledger, resumed.ledger);
        assert_eq!(controller, saved);
        assert_eq!(audit, saved_audit);
        results.push((sim.state, sim.ledger, controller, audit));
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn unsupported_configuration_and_partial_mandate_fail_without_mutation() {
    let (mut sim, _) = setup("B5", Backend::Reference, Policy::RetainRepair);
    fixtures::acquire(&mut sim).unwrap();
    let original = sim.clone();
    let mut controller = Controller::new(
        Scope::Household {
            agent: fixtures::HOME,
            consenting_members: [PERSON].into(),
        },
        Strategy::BestFirst,
        budget(),
        Policy::RetainRepair,
    );
    let before = controller.clone();
    assert!(
        controller
            .step(&mut sim)
            .unwrap_err()
            .contains("every participant")
    );
    assert_eq!(controller, before);
    assert_eq!(sim.state, original.state);
    assert_eq!(sim.ledger, original.ledger);
    sim.world.priority = Priority::ConsequenceAware;
    assert!(controller.step(&mut sim).unwrap_err().contains("opt-in"));
}
