use super::*;
use crate::{
    acceptance::{WORKSHOP, continuation::Case, financing},
    compute::Backend,
    forecast::ForecastContext,
};

#[test]
fn forecast_completions_match_executed_processes_without_counting_history() {
    let (w, s) = financing::scenario(Case::Baseline).unwrap();
    let mut live = Simulation::new(w, s, Backend::Reference).unwrap();
    live.run_months(10).unwrap();
    let before = live.clone();
    let config = &live.world.agency[&WORKSHOP].config;
    let peers = expectations::Snapshot::observe(
        &live.world,
        &live.state,
        WORKSHOP,
        config.peer_expectations,
    )
    .unwrap();
    let alternative = forecast(&live.world, &live.state, WORKSHOP, config, None, &peers);
    assert!(alternative.failure.is_none(), "{:?}", alternative.failure);
    let projection = alternative.projection.unwrap();
    let old: BTreeSet<_> = live
        .state
        .processes
        .values()
        .filter(|p| p.status == Status::Completed)
        .map(|p| p.id)
        .collect();
    assert!(!old.is_empty());
    let (w, s) = ForecastContext::new(&live.world, &live.state).into_parts();
    let mut branch = Simulation::new(w, s, Backend::Reference).unwrap();
    let end = live.state.month + config.horizon;
    while branch.state.month < end {
        if branch.state.phase == Phase::Open {
            peers
                .apply_ballots(&mut branch.world, &branch.state)
                .unwrap();
            peers
                .apply_hypotheses(&mut branch.world, &branch.state)
                .unwrap();
        }
        branch.step().unwrap();
    }
    let expected: BTreeSet<_> = branch
        .state
        .processes
        .values()
        .filter(|p| p.operator == WORKSHOP && p.status == Status::Completed && !old.contains(&p.id))
        .map(|p| p.id)
        .collect();
    assert!(!expected.is_empty());
    assert_eq!(
        projection
            .completions
            .iter()
            .map(|c| c.process)
            .collect::<BTreeSet<_>>(),
        expected
    );
    assert!(
        projection
            .completions
            .iter()
            .all(|c| c.month >= live.state.month && c.month < end && c.operator == WORKSHOP)
    );
    // Terminal reserve forecasts now have completion evidence without stock paths.
    assert!(projection.path.closes.is_empty());
    assert_eq!(live.world, before.world);
    assert_eq!(live.state, before.state);
    assert_eq!(live.ledger, before.ledger);
}
