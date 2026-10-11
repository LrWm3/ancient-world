use economics_compute_smoke::{
    acceptance::{self, Run, TOWN, WORKSHOP, continuation as c, governed as g},
    agency::Command,
    compute::Backend,
};

#[test]
fn organizations_discover_operating_programs_and_replenish_without_initial_quotes() {
    let (w, s) = g::scenario(c::Case::Baseline).unwrap();
    let stock = w
        .minting
        .as_ref()
        .unwrap()
        .order_policy
        .as_ref()
        .unwrap()
        .peer_stocks
        .as_ref()
        .unwrap();
    assert!(stock.quotes.is_empty());
    for agent in [WORKSHOP, TOWN] {
        assert!(w.agency[&agent].config.programs.is_empty());
        assert!(w.activities.orders.iter().all(|o| o.agent != agent));
    }
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(c::MONTHS).unwrap();
    for agent in [WORKSHOP, TOWN] {
        let d = &run.sim.world.agency[&agent].history[0];
        assert!(
            d.accepted.as_ref().is_some_and(|p| p
                .commands
                .iter()
                .any(|c| matches!(c, Command::PeerStockQuotes(q) if !q.is_empty()))),
            "{agent}: {d:?}"
        );
    }
    for agent in [WORKSHOP, TOWN] {
        for d in run.sim.world.agency[&agent]
            .history
            .iter()
            .filter(|d| !d.alternatives.is_empty())
        {
            println!(
                "agent={agent} month={} choice={:?} alternatives={:?}",
                d.month, d.chosen, d.alternatives
            );
        }
    }
    let report = g::check(&run).unwrap();
    println!("governed: {:?}; {:?}", run.evidence().unwrap(), report);
    assert!(report.ready(), "{:?}", report.failures);
    assert!(acceptance::accepted_deals(&run.sim).any(|d| d.buyer == WORKSHOP && d.seller == TOWN));
}
