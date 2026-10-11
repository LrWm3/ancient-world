use economics_compute_smoke::{
    acceptance::{Run, WORKSHOP, continuation as c, governed as g},
    agency::expectations::{Policy, Snapshot},
    compute::Backend,
    minting::ISSUER,
    model::Status,
};

#[test]
fn observations_require_two_completed_accepted_peer_starts_and_fresh_history() {
    let (w, s) = g::scenario(c::Case::Baseline).unwrap();
    let policy = Policy::RecentSuccessfulStarts { memory_months: 6 };
    assert!(
        Snapshot::observe(&w, &s, WORKSHOP, policy)
            .unwrap()
            .evidence
            .is_empty()
    );
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(6).unwrap();
    let w = &run.sim.world;
    let s = &run.sim.state;
    let snapshot = Snapshot::observe(w, s, WORKSHOP, policy).unwrap();
    let evidence = snapshot
        .evidence
        .iter()
        .find(|e| e.agent == ISSUER)
        .unwrap();
    assert_eq!(evidence.interval, 1);
    assert!(evidence.latest < s.month);
    assert!(
        Snapshot::observe(w, s, ISSUER, policy)
            .unwrap()
            .evidence
            .iter()
            .all(|e| e.agent != ISSUER)
    );
    assert!(
        Snapshot::observe(w, s, WORKSHOP, Policy::AcceptedOnly)
            .unwrap()
            .evidence
            .is_empty()
    );
    let mut failed = s.clone();
    for p in failed
        .processes
        .values_mut()
        .filter(|p| p.operator == ISSUER && p.start == evidence.latest)
    {
        p.status = Status::Aborted;
    }
    assert!(
        Snapshot::observe(w, &failed, WORKSHOP, policy)
            .unwrap()
            .evidence
            .is_empty()
    );
    let mut old = s.clone();
    old.month += 7;
    assert!(
        Snapshot::observe(w, &old, WORKSHOP, policy)
            .unwrap()
            .evidence
            .is_empty()
    );
    let mut unpublished = w.clone();
    unpublished.agency.get_mut(&ISSUER).unwrap().history.clear();
    assert!(
        Snapshot::observe(&unpublished, s, WORKSHOP, policy)
            .unwrap()
            .evidence
            .is_empty()
    );
    for memory_months in [0, 25, u32::MAX] {
        assert!(
            Policy::RecentSuccessfulStarts { memory_months }
                .validate()
                .is_err()
        );
    }
    assert_eq!(snapshot, Snapshot::observe(w, s, WORKSHOP, policy).unwrap());
}

#[test]
fn hypothetical_starts_are_private_future_dated_deduplicated_and_goal_bounded() {
    use economics_compute_smoke::{
        forecast::ForecastContext,
        minting::{COIN, MINT},
        model::*,
    };
    let (w, s) = g::scenario(c::Case::Baseline).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(6).unwrap();
    let original = run.sim.clone();
    let snapshot = Snapshot::observe(
        &original.world,
        &original.state,
        WORKSHOP,
        Policy::RecentSuccessfulStarts { memory_months: 6 },
    )
    .unwrap();
    assert!(
        snapshot
            .apply_hypotheses(&mut run.sim.world, &run.sim.state)
            .is_err()
    );
    let (mut w, mut s) = ForecastContext::new(&original.world, &original.state).into_parts();
    assert!(snapshot.apply_hypotheses(&mut w, &s).unwrap().is_empty());
    s.month += 1;
    let state_before = s.clone();
    let history = w.agency[&ISSUER].history.clone();
    let starts = snapshot.apply_hypotheses(&mut w, &s).unwrap();
    assert_eq!(
        starts,
        vec![ScheduledStart {
            month: s.month,
            agent: ISSUER,
            definition: MINT
        }]
    );
    assert!(snapshot.apply_hypotheses(&mut w, &s).unwrap().is_empty());
    assert_eq!(s, state_before);
    assert_eq!(w.agency[&ISSUER].history, history);
    assert_eq!(original.world, run.sim.world);
    assert_eq!(original.state, run.sim.state);
    s.month += 1;
    s.balances.insert((ISSUER, COIN), 1000);
    assert!(snapshot.apply_hypotheses(&mut w, &s).unwrap().is_empty());
    s.balances.insert((ISSUER, COIN), 0);
    s.month += 8;
    assert!(snapshot.apply_hypotheses(&mut w, &s).unwrap().is_empty());
}

#[test]
fn organization_alternatives_share_observations_but_execute_their_own_hypotheses() {
    let (mut w, s) = g::scenario(c::Case::Baseline).unwrap();
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .config
        .peer_expectations = Policy::RecentSuccessfulStarts { memory_months: 6 };
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(12).unwrap();
    let before = run.sim.state.clone();
    run.audit.step(&mut run.sim).unwrap();
    let decision = run.sim.world.agency[&WORKSHOP].history.last().unwrap();
    assert_eq!(decision.month, 13);
    assert!(
        !decision
            .peer_expectations
            .as_ref()
            .unwrap()
            .evidence
            .is_empty()
    );
    assert!(decision.alternatives.iter().all(|a| a.losses.is_some()));
    assert!(decision.alternatives.iter().any(|a| {
        a.projection
            .as_ref()
            .unwrap()
            .peer_attempts
            .iter()
            .any(|p| p.completed)
    }));
    assert!(
        decision
            .alternatives
            .iter()
            .flat_map(|a| &a.projection.as_ref().unwrap().peer_attempts)
            .all(|a| a.month > 13 && a.agent != WORKSHOP)
    );
    // Only the actual Open ran; private month 14+ completions cannot leak.
    assert_eq!(run.sim.state.processes, before.processes);
    assert_eq!(
        run.sim.world.agency[&ISSUER].history.last().unwrap().month,
        13
    );
}
