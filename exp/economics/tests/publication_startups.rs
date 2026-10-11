use economics_compute_smoke::{
    acceptance::{self, Run, continuation as c, publication as p, sales::Startup},
    agency::publication::Policy,
    compute::Backend,
};
#[test]
fn partial_inventory_is_not_replaced_by_offer_publication() {
    for startup in [Startup::PostedMetal, Startup::MetalInventory] {
        let (w, s) = p::with_startup(
            c::Case::Baseline,
            startup,
            Policy::StockedAsksOnEqualProjection,
        )
        .unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(c::MONTHS).unwrap();
        let e = run.evidence().unwrap();
        let report = acceptance::governed::check(&run).unwrap();
        println!("{startup:?}: {e:?}; {report:?}");
        assert!(!report.ready());
        assert_eq!(e.municipal_batches, 0);
        assert!(e.people_food_deficit > 0);
    }
}
