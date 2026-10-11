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
