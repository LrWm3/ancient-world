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

#[test]
fn input_priority_compares_replenishment_on_the_same_opening_snapshot() {
    use crate::acceptance::financing::{Liquidity, Preferences, Stocks};
    use crate::agency::objectives::{Metric, Objective, Scope};
    let (w, s) = financing::with_preferences(
        Case::Baseline,
        true,
        Stocks::Both,
        Preferences {
            workshop: Liquidity::MonthlyExposure,
            town: Liquidity::TerminalReserve,
        },
    )
    .unwrap();
    let mut live = Simulation::new(w, s, Backend::Reference).unwrap();
    live.run_months(11).unwrap();
    let before = live.clone();
    let mut config = live.world.agency[&WORKSHOP].config.clone();
    config.programs = discovery::programs(&live.world, WORKSHOP).unwrap();
    let drop_commands = &config.programs[&1].commands;
    assert!(drop_commands.iter().any(|command| matches!(command,
        Command::WorkTargets(orders) if orders.iter().any(|o| o.definition == crate::minting::REFINE))));
    let quotes: Vec<_> = drop_commands
        .iter()
        .filter_map(|command| match command {
            Command::PeerStockQuotes(quotes) => Some(quotes),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(
        quotes
            .iter()
            .any(|q| q.market == crate::minting::METAL && q.side == crate::marketplace::Side::Sell)
    );
    assert!(
        !quotes
            .iter()
            .any(|q| q.market == crate::minting::ORE && q.side == crate::marketplace::Side::Buy)
    );
    let peers = expectations::Snapshot::observe(
        &live.world,
        &live.state,
        WORKSHOP,
        config.peer_expectations,
    )
    .unwrap();
    let mut pairs = vec![];
    for buffered in [false, true] {
        let mut c = config.clone();
        if buffered {
            c.objectives.insert(
                0,
                Objective {
                    scope: Scope::Organization,
                    metric: Metric::ReserveExposure {
                        resource: crate::minting::ORE,
                        target: 2,
                    },
                },
            );
        }
        let retain = forecast(&live.world, &live.state, WORKSHOP, &c, None, &peers);
        let drop_bid = forecast(&live.world, &live.state, WORKSHOP, &c, Some(1), &peers);
        println!(
            "SAME_OPEN buffered={buffered} retain={:?} drop={:?} completions={:?}/{:?}",
            retain.losses,
            drop_bid.losses,
            retain.projection.as_ref().map(|p| &p.completions),
            drop_bid.projection.as_ref().map(|p| &p.completions)
        );
        assert!(retain.failure.is_none() && drop_bid.failure.is_none());
        pairs.push((retain, drop_bid));
    }
    assert_eq!(pairs[0].0.losses, Some(vec![11910]));
    assert_eq!(pairs[0].1.losses, Some(vec![11870]));
    assert_eq!(pairs[1].0.losses, Some(vec![2, 11910]));
    assert_eq!(pairs[1].1.losses, Some(vec![24, 11870]));
    assert!(pairs[0].1.losses < pairs[0].0.losses);
    assert!(pairs[1].0.losses < pairs[1].1.losses);
    assert_eq!(pairs[0].0.projection.as_ref().unwrap().completions.len(), 4);
    assert_eq!(pairs[0].1.projection.as_ref().unwrap().completions.len(), 1);
    // Same actions produce identical flows regardless of how those flows are scored.
    for index in 0..2 {
        let a = if index == 0 { &pairs[0].0 } else { &pairs[0].1 };
        let b = if index == 0 { &pairs[1].0 } else { &pairs[1].1 };
        assert_eq!(a.projection, b.projection);
    }
    assert_eq!(live.world, before.world);
    assert_eq!(live.state, before.state);
}
