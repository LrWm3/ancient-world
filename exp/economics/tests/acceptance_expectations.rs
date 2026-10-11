use economics_compute_smoke::{
    acceptance::{Run, WORKSHOP, continuation as c, expectations as e, governed as g},
    agency::expectations::Policy,
    compute::Backend,
};

#[test]
fn cash_first_comparison_changes_only_peer_expectations_and_records_the_mechanism() {
    let (mut ordinary, state) = e::scenario_with(c::Case::Baseline, Policy::AcceptedOnly).unwrap();
    let (expected, expected_state) =
        e::scenario_with(c::Case::Baseline, e::PROCESS_POLICY).unwrap();
    ordinary
        .agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .config
        .peer_expectations = e::PROCESS_POLICY;
    assert_eq!(ordinary, expected);
    assert_eq!(state, expected_state);
    for policy in [Policy::AcceptedOnly, e::PROCESS_POLICY] {
        let (w, s) = e::scenario_with(c::Case::Baseline, policy).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(c::MONTHS).unwrap();
        let report = g::check(&run).unwrap();
        println!("{policy:?}: {:?}; {:?}", run.evidence().unwrap(), report);
        assert!(!report.ready());
        let review = &run.sim.world.agency[&WORKSHOP].history[12];
        assert_eq!(review.month, 13);
        println!("snapshot {:?}", review.peer_expectations);
        if policy == Policy::AcceptedOnly {
            assert!(
                review.alternatives.iter().all(|a| a
                    .projection
                    .as_ref()
                    .unwrap()
                    .peer_attempts
                    .is_empty())
            );
        } else {
            assert!(review.alternatives.iter().any(|a| {
                a.projection
                    .as_ref()
                    .unwrap()
                    .peer_attempts
                    .iter()
                    .any(|a| a.completed)
            }));
        }
    }
}

#[test]
fn observed_voting_and_forecast_horizon_comparison() {
    for (horizon, policy, ready) in [
        (
            4,
            Policy::RecentSuccessfulStartsAndBallots { memory_months: 6 },
            false,
        ),
        (4, e::POLICY, false),
        (12, Policy::AcceptedOnly, false),
        (
            12,
            Policy::RecentSuccessfulStarts { memory_months: 24 },
            false,
        ),
        (12, e::POLICY, true),
    ] {
        let (mut w, s) = e::scenario_with(c::Case::Baseline, policy).unwrap();
        w.agency.get_mut(&WORKSHOP).unwrap().config.horizon = horizon;
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(c::MONTHS).unwrap();
        let report = g::check(&run).unwrap();
        println!(
            "horizon={horizon} policy={policy:?} ready={} evidence={:?} failures={:?}",
            report.ready(),
            run.evidence().unwrap(),
            report.failures
        );
        assert_eq!(
            report.ready(),
            ready,
            "horizon={horizon} {policy:?}: {:?}",
            report.failures
        );
        let review = &run.sim.world.agency[&WORKSHOP].history[12];
        if ready {
            assert!(review.chosen.is_none(), "retain sustainable replenishment");
            assert!(review.alternatives.iter().any(|a| {
                a.projection
                    .as_ref()
                    .unwrap()
                    .peer_ballots
                    .iter()
                    .any(|v| v.accepted)
            }));
            assert_eq!(run.evidence().unwrap().wage_arrears, 0);
        }
    }
}
