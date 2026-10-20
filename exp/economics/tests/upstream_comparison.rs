use economics_compute_smoke::{
    acceptance::{Run, TOWN, WORKSHOP, continuation::Case, financing::Stocks, work_discovery},
    agency::work_requests::{Policy, Source},
    compute::Backend,
};
fn compare(stocks: Stocks) {
    let mut previous = None;
    for policy in [Policy::RespondWithWorkQuotes, Policy::Upstream] {
        let (w, s) = work_discovery::scenario(Case::Baseline, stocks, policy).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(12).unwrap();
        let e = run.evidence().unwrap();
        let requests: usize = [WORKSHOP, TOWN]
            .iter()
            .flat_map(|a| &run.sim.world.agency[a].history)
            .filter_map(|d| d.work_requests.as_ref())
            .flat_map(|r| &r.requests)
            .filter(|r| matches!(r.source, Source::ProspectiveInput(_)))
            .count();
        println!("UPSTREAM {stocks:?} {policy:?} requests={requests}: {e:?}");
        if let Some(old) = &previous {
            assert_eq!(&e, old);
        }
        previous = Some(e);
        if policy == Policy::Upstream {
            assert_eq!(
                requests,
                match stocks {
                    Stocks::Neither => 2,
                    Stocks::MetalOnly => 3,
                    _ => 0,
                }
            );
        }
        assert!(run.sim.state.balances.values().all(|n| *n >= 0));
    }
}
#[test]
fn both() {
    compare(Stocks::Both);
}
#[test]
fn ore() {
    compare(Stocks::OreOnly);
}
#[test]
fn metal() {
    compare(Stocks::MetalOnly);
}
#[test]
fn neither() {
    compare(Stocks::Neither);
}
