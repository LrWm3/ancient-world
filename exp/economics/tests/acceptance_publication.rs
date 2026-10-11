use economics_compute_smoke::{
    acceptance::{self, Run, continuation as c, publication as p, sales::Startup},
    agency::publication::Policy,
    compute::Backend,
};
#[test]
fn voluntary_publication_is_compared_with_identical_unquoted_stock_and_default_retention() {
    for policy in [Policy::Retain, Policy::StockedAsksOnEqualProjection] {
        let (w, s) =
            p::with_startup(c::Case::Baseline, Startup::SupplyChainInventory, policy).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(c::MONTHS).unwrap();
        let report = acceptance::governed::check(&run).unwrap();
        println!("{policy:?}: {:?}\n{report:?}", run.evidence().unwrap());
        let evidence = run.evidence().unwrap();
        assert_eq!(
            report.ready(),
            policy == Policy::StockedAsksOnEqualProjection,
            "{report:?}"
        );
        let expected = if policy == Policy::Retain {
            (0, 0, 0, 67, 2)
        } else {
            (44, 43, 40, 0, 0)
        };
        assert_eq!(
            (
                evidence.completed_mints,
                evidence.workshop_batches,
                evidence.municipal_batches,
                evidence.people_food_deficit,
                evidence.wage_arrears
            ),
            expected
        );
        for id in [acceptance::WORKSHOP, acceptance::TOWN] {
            assert!(
                run.sim.world.agency[&id]
                    .history
                    .iter()
                    .take(1)
                    .flat_map(|d| &d.alternatives)
                    .filter_map(|a| a.projection.as_ref())
                    .all(|p| p.peer_attempts.is_empty())
            );
        }
    }
}

#[test]
fn comparison_changes_only_publication_policy_and_supplies_no_organization_asks_or_work() {
    use economics_compute_smoke::{
        agency::objectives::{Metric, Scope},
        minting::{COIN, METAL, ORE},
    };
    let (mut retained, state) = p::with_startup(
        c::Case::Baseline,
        Startup::SupplyChainInventory,
        Policy::Retain,
    )
    .unwrap();
    let (published, same_state) = p::scenario(c::Case::Baseline).unwrap();
    assert_eq!(state, same_state);
    let orders = retained
        .minting
        .as_ref()
        .unwrap()
        .order_policy
        .as_ref()
        .unwrap();
    assert!(
        orders
            .quotes
            .iter()
            .all(|q| q.agent != acceptance::WORKSHOP)
    );
    assert!(
        orders
            .peer_stocks
            .as_ref()
            .unwrap()
            .quotes
            .iter()
            .all(|q| q.agent != acceptance::TOWN)
    );
    assert_eq!(state.balance(acceptance::WORKSHOP, METAL), 4);
    assert_eq!(state.balance(acceptance::TOWN, ORE), 4);
    for id in [acceptance::WORKSHOP, acceptance::TOWN] {
        let controller = retained.agency.get_mut(&id).unwrap();
        assert!(controller.discover_programs);
        assert!(controller.config.programs.is_empty());
        assert!(controller.config.preferences.is_empty());
        assert_eq!(controller.config.objectives.len(), 1);
        let goal = &controller.config.objectives[0];
        assert_eq!(goal.scope, Scope::Organization);
        assert!(matches!(goal.metric, Metric::Reserve { resource: COIN, target } if target > 0));
        assert!(retained.activities.orders.iter().all(|o| o.agent != id));
        controller.config.publication = Policy::StockedAsksOnEqualProjection;
    }
    assert_eq!(retained, published);
}
