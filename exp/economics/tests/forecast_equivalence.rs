use economics_compute_smoke::{
    acceptance::{
        WORKSHOP,
        continuation::Case,
        financing::{self, Liquidity, Stocks},
    },
    agency::{
        self,
        expectations::{Policy, Snapshot},
        forwards::execution::Status as ForwardStatus,
        path::Path,
    },
    compute::Backend,
    forecast::ForecastContext,
    model::*,
    simulation::Simulation,
};

fn compare(live: &Simulation, months: u32) -> Simulation {
    let before = live.clone();
    let (compact, state) = ForecastContext::new(&live.world, &live.state).into_parts();
    let mut detailed = compact.clone();
    for (&agent, original) in &live.world.agency {
        let c = detailed.agency.get_mut(&agent).unwrap();
        c.history = original.history.clone();
        c.forward_reviews = original.forward_reviews.clone();
    }
    let mut compact = Simulation::new(compact, state.clone(), Backend::Reference).unwrap();
    let mut detailed = Simulation::new(detailed, state, Backend::Reference).unwrap();
    let mut paths = [Path::new(live.state.month), Path::new(live.state.month)];
    while compact.state.month < live.state.month + months {
        compact.step().unwrap();
        detailed.step().unwrap();
        assert_eq!(compact.state, detailed.state);
        assert_eq!(compact.ledger, detailed.ledger);
        assert_eq!(compact.reports, detailed.reports);
        if compact.state.phase == Phase::Open {
            paths[0].record(&compact.world, &compact.state).unwrap();
            paths[1].record(&detailed.world, &detailed.state).unwrap();
            for agent in live.world.agency.keys() {
                let policy = Policy::RecentSuccessfulStartsAndBallots { memory_months: 6 };
                assert_eq!(
                    Snapshot::observe(&compact.world, &compact.state, *agent, policy).unwrap(),
                    Snapshot::observe(&detailed.world, &detailed.state, *agent, policy).unwrap()
                );
            }
        }
    }
    assert_eq!(paths[0], paths[1]);
    assert_eq!(
        ForecastContext::new(&compact.world, &compact.state),
        ForecastContext::new(&detailed.world, &detailed.state)
    );
    assert_eq!(live.world, before.world);
    assert_eq!(live.state, before.state);
    compact
}

#[test]
fn pending_and_completed_forwards_survive_private_diagnostic_trimming() {
    let (w, s) = financing::with_liquidity(
        Case::Baseline,
        true,
        Stocks::MetalOnly,
        Liquidity::MonthlyExposure,
    )
    .unwrap();
    let mut live = Simulation::new(w, s, Backend::Reference).unwrap();
    live.run_months(5).unwrap();
    assert_eq!(
        live.world.agency[&WORKSHOP].startup_forwards[0].status,
        ForwardStatus::Pending
    );
    let id = live.world.agency[&WORKSHOP].startup_forwards[0]
        .proposal
        .terms
        .id;
    let branch = compare(&live, 6);
    assert_eq!(
        branch.world.agency[&WORKSHOP].startup_forwards[0].status,
        ForwardStatus::Activated(7)
    );
    assert!(branch.state.exchange.forwards[&id].delivered > 0);
    live.run_months(5).unwrap();
    assert!(live.state.exchange.forwards[&id].delivered > 0);
    compare(&live, 3);
    assert!(live.world.agency.keys().any(|agent| {
        !Snapshot::observe(
            &live.world,
            &live.state,
            *agent,
            Policy::RecentSuccessfulStartsAndBallots { memory_months: 6 },
        )
        .unwrap()
        .evidence
        .is_empty()
    }));
}

#[test]
fn accepted_future_process_start_remains_scheduled_and_executes() {
    let (mut w, s) = agency::scenario::public_work().unwrap();
    assert!(w.minting.is_none());
    w.agency
        .get_mut(&economics_compute_smoke::minting::ISSUER)
        .unwrap()
        .config
        .programs
        .get_mut(&1)
        .unwrap()
        .commands = vec![agency::Command::StartProcess(
        economics_compute_smoke::minting::GATHER,
    )];
    let mut live = Simulation::new(w, s, Backend::Reference).unwrap();
    let start = loop {
        assert!(live.state.month <= 6, "no accepted dated process start");
        live.step().unwrap();
        let accepted = live.world.agency.iter().find_map(|(&agent, c)| {
            c.history.iter().find_map(|d| {
                if d.effective_month <= live.state.month {
                    return None;
                }
                d.accepted
                    .as_ref()?
                    .commands
                    .iter()
                    .find_map(|command| match command {
                        agency::Command::StartProcess(definition) => {
                            Some((agent, *definition, d.effective_month))
                        }
                        _ => None,
                    })
            })
        });
        if let Some(start) = accepted {
            break start;
        }
    };
    let private = ForecastContext::new(&live.world, &live.state);
    assert!(
        private
            .world()
            .scheduled_starts
            .iter()
            .any(|s| (s.agent, s.definition, s.month) == start)
    );
    let branch = compare(&live, 4);
    assert!(
        branch
            .state
            .processes
            .values()
            .any(|p| (p.operator, p.definition, p.start) == start && p.status == Status::Completed)
    );
}
