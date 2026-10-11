use economics_compute_smoke::{
    acceptance::{self, WORKSHOP, WORKSHOP_TYPE, continuation as c},
    agency::{Command, markets::Options, organization},
    compute::Backend,
    marketplace::Side,
    minting::{METAL, ORE, orders::Quote},
    model::*,
    opportunities::Action,
    simulation::Simulation,
};
fn fixture() -> (World, State) {
    let (mut w, s) = c::spot_scenario(c::Case::Baseline).unwrap();
    let previous = w.agency.remove(&WORKSHOP).unwrap();
    let mut a = previous.operating_agreement().unwrap().clone();
    let mut config = previous.config;
    let mut quotes = vec![];
    for (market, side, limit, holding, max_lots) in
        [(ORE, Side::Buy, 2, 4, 2), (METAL, Side::Sell, 4, 0, 1)]
    {
        a.stock_markets.insert(
            market,
            Options {
                side,
                limits: [limit].into(),
                holdings: [holding].into(),
                max_lots,
            },
        );
        quotes.push(Quote {
            agent: WORKSHOP,
            market,
            side,
            limit,
            holding,
            max_lots: Some(max_lots),
        });
    }
    config
        .programs
        .get_mut(&1)
        .unwrap()
        .commands
        .push(Command::PeerStockQuotes(quotes));
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.quotes.retain(|q| q.agent != WORKSHOP);
    p.peer_stocks
        .as_mut()
        .unwrap()
        .quotes
        .retain(|q| q.agent != WORKSHOP);
    organization::install_initial(&mut w, &s, a.clone(), &a.members, config).unwrap();
    (w, s)
}
#[test]
fn governed_input_sales_activate_next_open_and_obey_live_access() {
    for revoke in [false, true] {
        let (w, s) = fixture();
        let jobs = w.employment.clone();
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            sim.run_months(1).unwrap();
            let decision = sim.world.agency[&WORKSHOP].history[0].clone();
            assert!(decision.accepted.is_some(), "{decision:?}");
            assert!(
                !sim.world
                    .minting
                    .as_ref()
                    .unwrap()
                    .order_policy
                    .as_ref()
                    .unwrap()
                    .quotes
                    .iter()
                    .any(|q| q.agent == WORKSHOP)
            );
            if revoke {
                sim.world
                    .transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(WORKSHOP_TYPE, Action::StockTrade));
            }
            let mut restored =
                Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
            restored.ledger = sim.ledger;
            restored.reports = sim.reports;
            sim = restored;
            sim.run_months(3).unwrap();
            assert_eq!(sim.world.agency[&WORKSHOP].history[0], decision);
            assert_eq!(sim.world.employment, jobs);
            let sales = acceptance::accepted_deals(&sim)
                .filter(|d| d.seller == WORKSHOP && d.market == METAL)
                .count();
            assert_eq!(sales > 0, !revoke);
            assert!(
                sim.world
                    .minting
                    .as_ref()
                    .unwrap()
                    .order_policy
                    .as_ref()
                    .unwrap()
                    .quotes
                    .iter()
                    .any(|q| q.agent == WORKSHOP && q.market == METAL)
            );
            sim
        };
        let reference = run(Backend::Reference);
        let cpu = run(Backend::CubeCpu);
        assert_eq!(reference.world, cpu.world);
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
    }
}
