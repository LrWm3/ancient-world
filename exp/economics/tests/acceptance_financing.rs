use economics_compute_smoke::{
    acceptance::{self, Run, TOWN, WORKSHOP, continuation as c, financing},
    compute::Backend,
    minting::ORE,
};

#[test]
fn signed_startup_forward_is_compared_without_adding_working_inventory() {
    for enabled in [false, true] {
        let (w, s) = financing::with_authority(c::Case::Baseline, enabled).unwrap();
        assert_eq!(s.balance(TOWN, ORE), 0);
        assert_eq!(s.balance(WORKSHOP, ORE), 4);
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(c::MONTHS).unwrap();
        let evidence = run.evidence().unwrap();
        let report = acceptance::governed::check(&run).unwrap();
        println!(
            "enabled={enabled}: {evidence:?}; {report:?}; agreements={:?}; attempts={:?}",
            run.sim.world.agency[&WORKSHOP].startup_forwards,
            run.sim.world.agency[&WORKSHOP].forward_reviews
        );
        assert_eq!(report.ready(), enabled, "{report:?}");
        if enabled {
            assert_eq!(
                (
                    evidence.completed_mints,
                    evidence.workshop_batches,
                    evidence.municipal_batches,
                    evidence.people_food_deficit,
                    evidence.wage_arrears
                ),
                (42, 41, 40, 0, 0)
            );
            let agreements = &run.sim.world.agency[&WORKSHOP].startup_forwards;
            assert_eq!(agreements.len(), 1);
            let a = &agreements[0];
            assert_eq!(
                (a.month, a.proposal.terms.month, a.proposal.terms.due),
                (7, 8, 10)
            );
            let contract = &run.sim.state.exchange.forwards[&a.proposal.terms.id];
            assert_eq!(contract.delivered, 2);
        } else {
            assert_eq!(
                (
                    evidence.completed_mints,
                    evidence.workshop_batches,
                    evidence.municipal_batches,
                    evidence.people_food_deficit,
                    evidence.wage_arrears
                ),
                (4, 2, 0, 37, 2)
            );
            assert!(!report.ready());
        }
    }
}
