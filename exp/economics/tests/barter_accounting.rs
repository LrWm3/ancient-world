use economics_compute_smoke::{
    accounting::Account as A,
    compute::Backend,
    currency::{self, Bid, StockTrade},
    financial_reporting::{Audit, Opening},
    model::*,
    negotiation,
    scenario::{GRAIN, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
use std::collections::BTreeMap;
const WOOD: ResourceId = 99;
fn fixture() -> (World, State, AgentId, AgentId) {
    let (mut w, mut s) = negotiation::scenario();
    let session = w.negotiation.take().unwrap();
    let (seller, buyer) = (session.seller.agent, session.buyer.agent);
    w.transaction_policy = None;
    w.resources.push(Resource {
        id: WOOD,
        name: "wood".into(),
        kind: ResourceKind::Stock,
    });
    w.bids.push(Bid {
        id: 1,
        buyer,
        goods: Amount::new(GRAIN, 2),
        payment: Amount::new(WOOD, 4),
    });
    s.balances = BTreeMap::from([((seller, GRAIN), 2), ((buyer, WOOD), 4)]);
    (w, s, seller, buyer)
}
fn opening(seller: AgentId, buyer: AgentId, unit: Option<i128>) -> Opening {
    Opening {
        inventory: BTreeMap::from([((seller, GRAIN), 14), ((buyer, WOOD), 8)]),
        exchange_values: unit
            .map(|v| BTreeMap::from([(WOOD, v)]))
            .unwrap_or_default(),
        ..Default::default()
    }
}
fn commit_barter(a: &mut Audit, sim: &mut Simulation, seller: AgentId) {
    assert_eq!(sim.state.phase, Phase::Acquire);
    let before = sim.state.clone();
    let mut batch = Batch::empty(&before);
    batch
        .transactions
        .push(currency::transaction(&sim.world, &before, StockTrade { bid: 1, seller }).unwrap());
    let mut after = before.clone();
    settlement::commit(
        &sim.world,
        &mut after,
        &batch,
        sim.backend,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    a.record(&sim.world, &before, &batch, &after).unwrap();
    sim.state = after;
    sim.ledger.push(batch);
}
#[test]
fn posted_barter_values_both_deliveries_without_cash_on_cpu_and_continuation() {
    let (w, s, seller, buyer) = fixture();
    let mut a = Audit::with_opening(&w, &s, TOKEN, opening(seller, buyer, Some(3))).unwrap();
    let mut b = a.clone();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    a.step(&mut reference).unwrap();
    b.step(&mut cpu).unwrap();
    commit_barter(&mut a, &mut reference, seller);
    commit_barter(&mut b, &mut cpu, seller);
    assert_eq!(a, b);
    assert_eq!(reference.state, cpu.state);
    let sold = a.book().statements(seller, 1, 1).unwrap();
    assert_eq!(sold.income[&A::Sales], 12);
    assert_eq!(sold.expenses[&A::CostOfSales], 14);
    assert_eq!(sold.trial_balance[&A::Inventory(WOOD)], 12);
    assert_eq!(sold.net_income, -2);
    let bought = a.book().statements(buyer, 1, 1).unwrap();
    assert_eq!(bought.income[&A::Sales], 12);
    assert_eq!(bought.expenses[&A::CostOfSales], 8);
    assert_eq!(bought.trial_balance[&A::Inventory(GRAIN)], 12);
    assert_eq!(bought.net_income, 4);
    assert!(sold.cash_flows.is_empty() && bought.cash_flows.is_empty());
    let mut resumed = reference.clone();
    let mut saved = a.clone();
    while reference.state.month <= 2 {
        a.step(&mut reference).unwrap();
        b.step(&mut cpu).unwrap();
        saved.step(&mut resumed).unwrap();
    }
    assert_eq!(a, b);
    assert_eq!(a, saved);
    assert_eq!(reference.state, cpu.state);
    a.finalize_through(2).unwrap();
    assert_eq!(
        a.book()
            .finalized_statements(seller, 2, 2)
            .unwrap()
            .net_income,
        0
    );
}
#[test]
fn missing_or_overflowing_barter_valuation_cannot_publish_a_journal() {
    for unit in [None, Some(i128::MAX)] {
        let (w, s, seller, buyer) = fixture();
        let mut a = Audit::with_opening(&w, &s, TOKEN, opening(seller, buyer, unit)).unwrap();
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        a.step(&mut sim).unwrap();
        let old = a.clone();
        let mut batch = Batch::empty(&sim.state);
        batch.transactions.push(
            currency::transaction(&sim.world, &sim.state, StockTrade { bid: 1, seller }).unwrap(),
        );
        let mut after = sim.state.clone();
        settlement::commit(
            &sim.world,
            &mut after,
            &batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
        assert!(a.record(&sim.world, &sim.state, &batch, &after).is_err());
        assert_eq!(a, old);
    }
}
#[test]
fn barter_and_coin_sales_share_the_same_opening_cost_pool() {
    let (mut w, mut s, seller, buyer) = fixture();
    w.bids[0].goods.quantity = 1;
    let third = economics_compute_smoke::scenario::STATE_AGENT;
    w.bids.push(Bid {
        id: 2,
        buyer: third,
        goods: Amount::new(GRAIN, 1),
        payment: Amount::new(TOKEN, 10),
    });
    s.balances.insert((third, TOKEN), 10);
    let mut a = Audit::with_opening(&w, &s, TOKEN, opening(seller, buyer, Some(3))).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    a.step(&mut sim).unwrap();
    let before = sim.state.clone();
    let mut batch = Batch::empty(&before);
    for bid in [1, 2] {
        batch
            .transactions
            .push(currency::transaction(&sim.world, &before, StockTrade { bid, seller }).unwrap());
    }
    let mut after = before.clone();
    settlement::commit(
        &sim.world,
        &mut after,
        &batch,
        Backend::CubeCpu,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    let mut reversed = batch.clone();
    reversed.transactions.reverse();
    let mut other = a.clone();
    a.record(&sim.world, &before, &batch, &after).unwrap();
    other
        .record(&sim.world, &before, &reversed, &after)
        .unwrap();
    assert_eq!(a.book().balances(), other.book().balances());
    let r = a.book().statements(seller, 1, 1).unwrap();
    assert_eq!(r.income[&A::Sales], 22);
    assert_eq!(r.expenses[&A::CostOfSales], 14);
    assert_eq!(r.closing_cash, 10);
    assert_eq!(r.trial_balance[&A::Inventory(WOOD)], 12);
}
