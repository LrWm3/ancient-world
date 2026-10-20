use economics_compute_smoke::{
    acceptance::{Run, TOWN, WORKSHOP, continuation::Case, financing::Stocks, work_discovery},
    agency::work_requests::Policy,
    compute::Backend,
};
fn compare(stocks: Stocks) {
    for policy in [Policy::Static, Policy::Respond] {
        let (w, s) = work_discovery::scenario(Case::Baseline, stocks, policy).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(12).unwrap();
        let e = run.evidence().unwrap();
        let requests: usize = [WORKSHOP, TOWN]
            .iter()
            .flat_map(|a| &run.sim.world.agency[a].history)
            .filter_map(|d| d.work_requests.as_ref())
            .map(|r| r.requests.len())
            .sum();
        let routes: usize = [WORKSHOP, TOWN]
            .iter()
            .flat_map(|a| &run.sim.world.agency[a].history)
            .map(|d| d.work_routes.len())
            .sum();
        let catalogs: usize = [WORKSHOP, TOWN]
            .iter()
            .flat_map(|a| &run.sim.world.agency[a].history)
            .map(|d| d.catalog.len())
            .sum();
        println!(
            "WORK_DISCOVERY {stocks:?} {policy:?}: mints={} workshop={} town={} food={} arrears={} requests={requests} routes={routes} candidates={catalogs}",
            e.completed_mints,
            e.workshop_batches,
            e.municipal_batches,
            e.people_food_deficit,
            e.wage_arrears
        );
        let expected = match (stocks, policy) {
            (Stocks::Both, Policy::Static) => (9, 8, 7, 0, 0),
            (Stocks::OreOnly, Policy::Static) => (9, 10, 10, 0, 0),
            (Stocks::Both | Stocks::OreOnly, Policy::Respond) => (0, 2, 0, 5, 2),
            (Stocks::MetalOnly, _) => (8, 7, 8, 0, 0),
            (Stocks::Neither, _) => (0, 1, 3, 5, 1),
            _ => unreachable!(),
        };
        assert_eq!(
            (
                e.completed_mints,
                e.workshop_batches,
                e.municipal_batches,
                e.people_food_deficit,
                e.wage_arrears
            ),
            expected
        );
        assert!(run.sim.state.balances.values().all(|n| *n >= 0));
        assert!(
            run.sim.world.agency[&WORKSHOP]
                .history
                .iter()
                .all(|d| d.catalog.len() <= 16)
        );
    }
}
#[test]
fn both() {
    compare(Stocks::Both);
}
#[test]
fn metal() {
    compare(Stocks::MetalOnly);
}
#[test]
fn ore() {
    compare(Stocks::OreOnly);
}
#[test]
fn neither() {
    compare(Stocks::Neither);
}
