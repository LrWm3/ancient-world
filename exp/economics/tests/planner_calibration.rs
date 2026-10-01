#[path = "support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    composition::{
        self, Budget, Strategy,
        calibration::{self, Scoring},
    },
    compute::Backend,
    model::*,
    scenario::*,
};

#[test]
fn pooled_coverage_uses_one_shared_budget_and_does_not_mutate_state() {
    let (mut sim, _) = fixtures::fixture("B5", Backend::Reference).unwrap();
    let original = sim.clone();
    let private = calibration::score(&sim, Scoring::PrivateBuffers).unwrap();
    let shared = calibration::score(&sim, Scoring::HouseholdBuffers).unwrap();
    assert!(shared.buffer_gap < private.buffer_gap);
    assert_eq!(sim.state, original.state);
    assert_eq!(sim.world, original.world);
    let mut fulfilled = sim.clone();
    for p in &fulfilled.world.participants {
        fulfilled.state.balances.insert((p.agent, NUTRITION), 100);
        fulfilled.state.balances.insert((p.agent, WARMTH), 100);
    }
    assert_eq!(
        shared.buffer_gap,
        calibration::score(&fulfilled, Scoring::HouseholdBuffers)
            .unwrap()
            .buffer_gap,
        "last month's fulfillment cannot become stored coverage"
    );
    let hidden = calibration::score(&sim, Scoring::HouseholdPrivateBuffers).unwrap();
    assert!(shared.buffer_gap < hidden.buffer_gap);
    for p in &sim.world.participants {
        sim.state.balances.insert((p.agent, GRAIN), 0);
    }
    sim.state.balances.insert((fixtures::HOME, GRAIN), 1);
    let one = calibration::score(&sim, Scoring::HouseholdBuffers).unwrap();
    sim.state.balances.insert((fixtures::HOME, GRAIN), 2);
    let two = calibration::score(&sim, Scoring::HouseholdBuffers).unwrap();
    sim.state.balances.insert((fixtures::HOME, GRAIN), 0);
    let zero = calibration::score(&sim, Scoring::HouseholdBuffers).unwrap();
    assert!(zero.buffer_gap > one.buffer_gap && one.buffer_gap > two.buffer_gap);
    assert!((zero.buffer_gap - one.buffer_gap).abs_diff(one.buffer_gap - two.buffer_gap) <= 1);
}

#[test]
fn opening_forecast_matches_fixed_execution_but_monthly_replanning_can_diverge() {
    let (mut sim, scope) = fixtures::fixture("B5", Backend::Reference).unwrap();
    fixtures::acquire(&mut sim).unwrap();
    sim.ledger.clear();
    let budget = Budget {
        expansions: 256,
        forecasts: 32,
        months: 12,
    };
    let selected = composition::choose(&sim, &scope, Strategy::BestFirst, budget).unwrap();
    let (_, _, forecast) = composition::forecast_package(&sim, &selected.requests, 12).unwrap();
    let mut fixed = sim.clone();
    selected.accept(&mut fixed).unwrap();
    while fixed.state.month < 13 {
        fixed.step().unwrap();
    }
    assert_eq!(fixed.state, forecast.state);
    assert_eq!(
        calibration::observe(&fixed, Scoring::PrivateBuffers).unwrap(),
        calibration::observe(&forecast, Scoring::PrivateBuffers).unwrap()
    );
    selected.accept(&mut sim).unwrap();
    while sim.state.month < 13 {
        if sim.state.phase == Phase::Acquire {
            composition::choose(&sim, &scope, Strategy::BestFirst, budget)
                .unwrap()
                .accept(&mut sim)
                .unwrap();
        } else {
            sim.step().unwrap();
        }
    }
    assert_ne!(sim.state, forecast.state);
    assert_ne!(
        calibration::observe(&sim, Scoring::PrivateBuffers)
            .unwrap()
            .completions,
        calibration::observe(&forecast, Scoring::PrivateBuffers)
            .unwrap()
            .completions
    );
}
