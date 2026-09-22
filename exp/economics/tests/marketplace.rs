use economics_compute_smoke::{
    compute::Backend,
    marketplace::{self, Side},
    model::*,
    negotiation::{self, GRAIN_MARKET, MARKETPLACE, Outcome},
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    scenario::{GRAIN, STATE_AGENT, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
fn opening() -> Simulation {
    let (w, s) = negotiation::scenario();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    sim
}
#[test]
fn catalog_is_person_only_even_when_other_types_can_trade() {
    let mut sim = opening();
    let rows = marketplace::discover(&sim.world, &sim.state, MARKETPLACE, 88);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].goods, Amount::new(GRAIN, 2));
    assert_eq!(rows[0].payment, TOKEN);
    assert_eq!(rows[0].price_tick, 1);
    assert!(sim.state.memberships.is_empty());
    for who in [STATE_AGENT, MARKETPLACE] {
        assert!(marketplace::discover(&sim.world, &sim.state, MARKETPLACE, who).is_empty());
    }
    for buyer in [true, false] {
        let mut denied = sim.clone();
        let who = if buyer { 88 } else { 89 };
        let p = denied.world.transaction_policy.as_mut().unwrap();
        p.agent_types.insert(who, STATE_TYPE);
        p.permissions.insert((STATE_TYPE, Action::StockTrade));
        let before = denied.state.balances.clone();
        denied.step().unwrap();
        let r = denied.ledger.last().unwrap().negotiation.as_ref().unwrap();
        assert_eq!(r.outcome, Outcome::Ineligible);
        assert!(r.quotes.is_empty());
        assert_eq!(denied.state.balances, before);
    }
    sim.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .agent_types
        .remove(&88);
    assert!(marketplace::discover(&sim.world, &sim.state, MARKETPLACE, 88).is_empty());
    sim.world.transaction_policy = None;
    assert!(marketplace::discover(&sim.world, &sim.state, MARKETPLACE, 89).is_empty());
}
#[test]
fn undeclared_markets_lots_and_payment_pairs_cannot_trade() {
    for case in 0..3 {
        let mut sim = opening();
        let s = sim.world.negotiation.as_mut().unwrap();
        match case {
            0 => s.market = 999,
            1 => s.goods.quantity = 1,
            2 => {
                s.goods.resource = TOKEN;
                s.payment = GRAIN;
            }
            _ => unreachable!(),
        }
        let before = sim.state.balances.clone();
        sim.step().unwrap();
        let r = sim.ledger.last().unwrap().negotiation.as_ref().unwrap();
        assert_eq!(r.outcome, Outcome::UnsupportedMarket);
        assert!(r.quotes.is_empty());
        assert_eq!(sim.state.balances, before);
    }
}
#[test]
fn history_and_per_side_quotes_persist_with_no_venue_custody() {
    let mut sim = opening();
    sim.state.balances.insert((89, GRAIN), 4);
    sim.world.storage.capacities.insert(88, 4);
    sim.run_months(1).unwrap();
    let memory = &sim.state.marketplaces[&MARKETPLACE];
    assert_eq!(memory.history.len(), 1);
    assert_eq!(memory.history[0].buyer, 88);
    assert_eq!(
        memory.pricing[&(88, GRAIN_MARKET, Side::Buy)].last_quote,
        40
    );
    assert_eq!(
        memory.pricing[&(89, GRAIN_MARKET, Side::Sell)].last_quote,
        40
    );
    assert!(!memory.pricing.contains_key(&(88, GRAIN_MARKET, Side::Sell)));
    assert_eq!(sim.state.balance(MARKETPLACE, GRAIN), 0);
    assert_eq!(sim.state.balance(MARKETPLACE, TOKEN), 0);
    let s = sim.world.negotiation.as_mut().unwrap();
    s.month = 2;
    s.buyer.limit = 35;
    let mut resumed = sim.clone();
    resumed.backend = Backend::Reference;
    sim.run_months(1).unwrap();
    resumed.run_months(1).unwrap();
    assert_eq!(sim.state, resumed.state);
    assert_eq!(sim.ledger, resumed.ledger);
    let memory = &sim.state.marketplaces[&MARKETPLACE];
    let r = &memory.history[1];
    assert_eq!((r.quotes[0].bid, r.quotes[0].ask), (35, 40));
    assert_eq!(r.outcome, Outcome::Traded { price: 35 });
    assert_eq!(sim.state.balance(88, TOKEN), 25);
    assert_eq!(sim.state.balance(88, GRAIN), 4);
    assert!(sim.state.memberships.is_empty());
}
#[test]
fn market_identity_and_changed_access_are_revalidated_atomically() {
    let sim = opening();
    let mut batch = Batch::empty(&sim.state);
    batch.negotiation = negotiation::evaluate(&sim.world, &sim.state).unwrap();
    batch.transactions =
        negotiation::transactions(&sim.world, &sim.state, &batch.negotiation).unwrap();
    for case in 0..3 {
        let mut w = sim.world.clone();
        let mut b = batch.clone();
        let mut state = sim.state.clone();
        match case {
            0 => b.negotiation.as_mut().unwrap().marketplace = STATE_AGENT,
            1 => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(PERSON_TYPE, Action::StockTrade));
            }
            _ => w.marketplaces[0].markets.clear(),
        }
        assert!(
            settlement::commit(&w, &mut state, &b, Backend::CubeCpu, DEFAULT_EFFECT_LIMIT).is_err()
        );
        assert_eq!(state, sim.state);
        assert!(state.marketplaces.is_empty());
    }
}
#[test]
fn catalog_validation_rejects_unknown_agents_duplicate_markets_and_bad_ticks() {
    for case in 0..5 {
        let (mut w, s) = negotiation::scenario();
        let m = &mut w.marketplaces[0];
        match case {
            0 => m.agent = 999,
            1 => m.markets.push(m.markets[0].clone()),
            2 => m.markets[0].price_tick = 0,
            3 => m.markets[0].goods.resource = 999,
            _ => w.negotiation.as_mut().unwrap().marketplace = STATE_AGENT,
        }
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}

#[test]
fn catalog_price_ticks_constrain_quotes_and_the_crossing_price() {
    let mut sim = opening();
    sim.world.marketplaces[0].markets[0].price_tick = 5;
    let s = sim.world.negotiation.as_mut().unwrap();
    s.buyer.opening_quote = 50;
    s.seller.opening_quote = 35;
    sim.step().unwrap();
    assert_eq!(
        sim.state.marketplaces[&MARKETPLACE].history[0].outcome,
        Outcome::Traded { price: 40 }
    );
    let mut invalid = opening();
    invalid.world.marketplaces[0].markets[0].price_tick = 5;
    invalid
        .world
        .negotiation
        .as_mut()
        .unwrap()
        .buyer
        .opening_quote = 21;
    invalid.step().unwrap();
    assert_eq!(
        invalid.state.marketplaces[&MARKETPLACE].history[0].outcome,
        Outcome::UnsupportedMarket
    );
    assert_eq!(invalid.state.balance(88, TOKEN), 100);
}
