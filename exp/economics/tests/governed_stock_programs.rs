use economics_compute_smoke::{
    acceptance::{TOWN, WORKSHOP, WORKSHOP_TYPE, continuation as c},
    agency::{self, Command, Program, markets::Options, organization},
    compute::Backend,
    marketplace::Side,
    minting::{COIN, ISSUER, ORE, orders::Quote},
    model::*,
    opportunities::Action,
    simulation::Simulation,
};

fn fixture() -> (World, State) {
    let (mut w, s) = c::spot_scenario(c::Case::Baseline).unwrap();
    let previous = w.agency.remove(&WORKSHOP).unwrap();
    let mut a = previous.operating_agreement().unwrap().clone();
    a.stock_markets.insert(
        ORE,
        Options {
            side: Side::Buy,
            limits: [2].into(),
            holdings: [4].into(),
            max_lots: 2,
        },
    );
    let mut config = previous.config;
    config
        .programs
        .get_mut(&1)
        .unwrap()
        .commands
        .push(Command::PeerStockQuotes(vec![Quote {
            agent: WORKSHOP,
            market: ORE,
            side: Side::Buy,
            limit: 2,
            holding: 4,
            max_lots: Some(2),
        }]));
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
        .as_mut()
        .unwrap()
        .quotes
        .retain(|q| q.agent != WORKSHOP);
    organization::install_initial(&mut w, &s, a.clone(), &a.members, config).unwrap();
    (w, s)
}
fn quotes(w: &World) -> &economics_compute_smoke::minting::orders::peer_stocks::Policy {
    w.minting
        .as_ref()
        .unwrap()
        .order_policy
        .as_ref()
        .unwrap()
        .peer_stocks
        .as_ref()
        .unwrap()
}
fn restore(sim: &mut Simulation) {
    let mut new = Simulation::new(sim.world.clone(), sim.state.clone(), sim.backend).unwrap();
    new.ledger = sim.ledger.clone();
    new.reports = sim.reports.clone();
    new.effect_limit = sim.effect_limit;
    *sim = new;
}

#[test]
fn market_program_activates_next_open_and_preserves_peers_and_obligations() {
    let (w, s) = fixture();
    let opening = quotes(&w).clone();
    let jobs = w.employment.clone();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        sim.run_months(1).unwrap();
        let accepted = sim.world.agency[&WORKSHOP].history[0].clone();
        assert!(accepted.chosen.is_some(), "{accepted:?}");
        assert_eq!(quotes(&sim.world), &opening);
        restore(&mut sim);
        sim.step().unwrap();
        assert_eq!(quotes(&sim.world).claim_months, opening.claim_months);
        assert_eq!(
            quotes(&sim.world).quotes.iter().find(|q| q.agent == TOWN),
            opening.quotes.iter().find(|q| q.agent == TOWN)
        );
        assert_eq!(
            quotes(&sim.world)
                .quotes
                .iter()
                .filter(|q| q.agent == WORKSHOP)
                .count(),
            1
        );
        assert_eq!(sim.world.employment, jobs);
        assert_eq!(sim.world.agency[&WORKSHOP].history[0], accepted);
        restore(&mut sim);
        while sim.state.month <= 3 {
            sim.step().unwrap();
        }
        assert!(
            economics_compute_smoke::acceptance::accepted_deals(&sim)
                .any(|d| d.market == ORE && d.buyer == WORKSHOP && d.seller == TOWN)
        );
        sim
    };
    let reference = run(Backend::Reference);
    let cpu = run(Backend::CubeCpu);
    assert_eq!(reference.world, cpu.world);
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.ledger, cpu.ledger);
}

#[test]
fn law_loss_suppresses_execution_without_rewriting_accepted_intent() {
    let (w, s) = fixture();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    let decision = sim.world.agency[&WORKSHOP].history[0].clone();
    assert!(decision.chosen.is_some());
    sim.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(WORKSHOP_TYPE, Action::StockTrade));
    sim.run_months(2).unwrap();
    assert_eq!(sim.world.agency[&WORKSHOP].history[0], decision);
    assert!(
        quotes(&sim.world)
            .quotes
            .iter()
            .any(|q| q.agent == WORKSHOP)
    );
    assert!(
        sim.ledger
            .iter()
            .filter(|b| b.month >= 2)
            .filter_map(|b| b.minting.as_ref())
            .flat_map(|b| &b.deals)
            .all(|d| d.buyer != WORKSHOP)
    );
}

#[test]
fn explicit_stop_removes_only_discretionary_quotes_and_keeps_payroll() {
    use agency::objectives::{Metric, Objective, Scope};
    let (mut w, mut s) = fixture();
    s.balances.insert((TOWN, ORE), 4);
    let Command::WorkTargets(work) = w.agency[&WORKSHOP].config.programs[&1].commands[0].clone()
    else {
        panic!()
    };
    w.activities.orders.extend(work);
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.month = 0;
    p.additional_months.clear();
    p.peer_stocks.as_mut().unwrap().quotes.push(Quote {
        agent: WORKSHOP,
        market: ORE,
        side: Side::Buy,
        limit: 2,
        holding: 4,
        max_lots: Some(2),
    });
    w.scheduled_starts.retain(|a| a.agent != ISSUER);
    w.agency.get_mut(&ISSUER).unwrap().autonomous = false;
    let config = &mut w.agency.get_mut(&WORKSHOP).unwrap().config;
    config.objectives = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Reserve {
            resource: COIN,
            target: 100,
        },
    }];
    config.programs = [(
        1,
        Program {
            name: "stop purchasing".into(),
            commands: vec![Command::PeerStockQuotes(vec![])],
        },
    )]
    .into();
    let jobs = w.employment.clone();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(
        sim.world.agency[&WORKSHOP].history[0].chosen,
        Some(1),
        "{:?}",
        sim.world.agency[&WORKSHOP].history[0]
    );
    sim.step().unwrap();
    assert!(
        quotes(&sim.world)
            .quotes
            .iter()
            .all(|q| q.agent != WORKSHOP)
    );
    assert!(quotes(&sim.world).quotes.iter().any(|q| q.agent == TOWN));
    assert_eq!(sim.world.employment, jobs);
}
