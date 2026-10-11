use economics_compute_smoke::{
    acceptance::{Run, WORKSHOP, continuation as c, publication},
    agency::publication::Policy,
    compute::Backend,
};

#[test]
fn publication_obeys_review_dates_and_minimum_tenure() {
    let (mut w, s) = publication::scenario(c::Case::Baseline).unwrap();
    let config = &mut w.agency.get_mut(&WORKSHOP).unwrap().config;
    config.review_every = 3;
    config.minimum_tenure = 6;
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(4).unwrap();
    let h = &run.sim.world.agency[&WORKSHOP].history;
    assert_eq!(
        h[0].reason,
        "publish stocked sale offers on equal projection"
    );
    assert!(h[0].accepted.is_some());
    assert_eq!(h[1].reason, "waiting for review");
    assert_eq!(h[2].reason, "waiting for review");
    assert_eq!(h[3].reason, "retained within minimum tenure");
    assert!(h[1..].iter().all(|d| d.accepted.is_none()));
    assert_eq!(h[0].effective_month, 2);
}

#[test]
fn publication_policy_is_part_of_the_sealed_mandate() {
    let (w, s) = publication::scenario(c::Case::Baseline).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(1).unwrap();
    run.sim
        .world
        .agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .config
        .publication = Policy::Retain;
    let before = run.sim.state.clone();
    let history = run.sim.world.agency[&WORKSHOP].history.clone();
    assert!(run.sim.step().is_err());
    assert_eq!(run.sim.state, before);
    assert_eq!(run.sim.world.agency[&WORKSHOP].history, history);
}
