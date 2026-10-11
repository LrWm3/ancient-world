use economics_compute_smoke::{
    acceptance::{
        self, Run, TOWN, WORKSHOP, continuation as c,
        sales::{self, Startup},
    },
    compute::Backend,
};
#[test]
fn partial_startups_do_not_sustain_the_supply_chain() {
    for startup in [Startup::PostedMetal, Startup::MetalInventory] {
        let (w, s) = sales::with_startup(c::Case::Baseline, startup).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(c::MONTHS).unwrap();
        let e = run.evidence().unwrap();
        let report = acceptance::governed::check(&run).unwrap();
        println!("{startup:?}: {e:?}\n{report:?}");
        for id in [WORKSHOP, TOWN] {
            assert_eq!(run.sim.world.agency[&id].config.objectives.len(), 1);
        }
        if startup == Startup::PostedMetal {
            assert_eq!(e.completed_mints, 1);
            assert_eq!(e.workshop_batches, 0);
            assert!(!report.ready());
        }
        if startup == Startup::MetalInventory {
            assert_eq!(e.completed_mints, 4);
            assert_eq!(e.workshop_batches, 2);
            assert_eq!(e.municipal_batches, 0);
            assert!(!report.ready());
        }
    }
}

#[test]
fn finite_supply_chain_inventory_bootstraps_cash_only_recurring_operation() {
    let (w, s) = sales::with_startup(c::Case::Baseline, Startup::SupplyChainInventory).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(c::MONTHS).unwrap();
    let e = run.evidence().unwrap();
    let report = acceptance::governed::check(&run).unwrap();
    assert!(report.ready(), "{report:?}");
    assert_eq!(
        (e.completed_mints, e.workshop_batches, e.municipal_batches),
        (44, 43, 40)
    );
    assert_eq!((e.people_food_deficit, e.wage_arrears), (0, 0));
    for id in [WORKSHOP, TOWN] {
        assert_eq!(run.sim.world.agency[&id].config.objectives.len(), 1);
    }
}

#[test]
fn startup_terms_have_only_cash_goals_and_no_supplied_organization_work() {
    use economics_compute_smoke::{
        agency::objectives::{Metric, Scope},
        minting::COIN,
    };
    for startup in [
        Startup::Cold,
        Startup::PostedMetal,
        Startup::MetalInventory,
        Startup::SupplyChainInventory,
    ] {
        let (w, _) = sales::with_startup(c::Case::Baseline, startup).unwrap();
        for id in [WORKSHOP, TOWN] {
            let c = &w.agency[&id];
            assert_eq!(c.config.objectives.len(), 1);
            let o = &c.config.objectives[0];
            assert_eq!(o.scope, Scope::Organization);
            assert!(matches!(o.metric, Metric::Reserve { resource: COIN, target } if target > 0));
            assert!(c.config.preferences.is_empty());
            assert!(c.discover_programs);
            assert!(c.config.programs.is_empty());
            assert!(w.activities.orders.iter().all(|o| o.agent != id));
        }
    }
}
