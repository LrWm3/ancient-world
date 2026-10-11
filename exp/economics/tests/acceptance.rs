use economics_compute_smoke::{
    acceptance::{self, Run},
    compute::Backend,
};

#[test]
fn acceptance_baseline_reconciles_actual_cross_agent_outcomes() {
    let (w, s) = acceptance::scenario().unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    assert!(run.evidence().is_err());
    run.through(acceptance::RUN_MONTHS).unwrap();
    let e = run.evidence().unwrap();
    assert_eq!(e.months, 14);
    assert_eq!(e.people_food_deficit, 2);
    assert_eq!(e.household_delivered, 2);
    assert_eq!(e.land_dues_paid, 1);
    assert_eq!(e.completed_mints, 4);
    assert_eq!(e.issuance, 40);
}
