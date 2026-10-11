use economics_compute_smoke::{
    compute::Backend,
    forward::direct::Terms,
    marketplace::Side,
    minting::{self, *},
    model::*,
    simulation::Simulation,
};
fn fixture(backend: Backend) -> Simulation {
    let (mut w, mut s) = minting::order_scenario("normal").unwrap();
    // Four units are sought, while only one two-unit lot is unclaimed.
    w.definitions
        .iter_mut()
        .find(|d| d.id == MINT)
        .unwrap()
        .stages[0]
        .entry_inputs
        .iter_mut()
        .find(|a| a.resource == METAL)
        .unwrap()
        .quantity = 4;
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.month = 1;
    w.scheduled_starts
        .iter_mut()
        .filter(|s| s.agent == ISSUER && s.definition == MINT)
        .for_each(|s| s.month = 1);
    p.private_sales = Some(6);
    let q = p.quotes.iter_mut().find(|q| q.market == METAL).unwrap();
    q.max_lots = Some(2);
    s.balances.insert((SUPPLIER, METAL), 6);
    s.balances.insert((ISSUER, COIN), 20);
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim
}
fn add_claim(sim: &mut Simulation, quantity: i32) {
    let t = Terms {
        id: 9910,
        seller: SUPPLIER,
        buyer: ISSUER,
        month: 1,
        due: 2,
        goods: Amount::new(METAL, quantity),
        prepayment: Amount::new(COIN, 1),
    };
    sim.world.prepaid_deliveries.push(t.clone());
    sim.state.exchange.forwards.insert(t.id, t.contract());
}
#[test]
fn issuer_inputs_preserve_existing_claims_and_recompute_new_claims_at_clearing() {
    let mut sim = fixture(Backend::Reference);
    let plan = |sim: &Simulation| {
        minting::evaluate(&sim.world, &sim.state)
            .unwrap()
            .unwrap()
            .plan
            .unwrap()
    };
    assert_eq!(
        plan(&sim)
            .orders
            .iter()
            .find(|o| o.market == METAL && o.side == Side::Sell)
            .unwrap()
            .lots,
        2
    );
    add_claim(&mut sim, 4);
    let protected = plan(&sim);
    let budget = protected
        .sales
        .iter()
        .find(|b| b.agent == SUPPLIER && b.market == METAL)
        .unwrap();
    assert_eq!(
        (
            budget.opening_available,
            budget.protected_stock,
            budget.feasible_lots
        ),
        (6, 4, 1)
    );
    assert_eq!(
        protected
            .orders
            .iter()
            .find(|o| o.market == METAL && o.side == Side::Sell)
            .unwrap()
            .lots,
        1
    );
    let t = sim.world.prepaid_deliveries.last_mut().unwrap();
    t.goods.quantity = 6;
    sim.state.exchange.forwards.insert(t.id, t.contract());
    let exhausted = plan(&sim);
    assert!(
        !exhausted
            .orders
            .iter()
            .any(|o| o.market == METAL && o.side == Side::Sell)
    );
}

#[test]
fn a_funded_input_fill_leaves_the_accepted_delivery_stock_intact() {
    let run = |backend| {
        let mut sim = fixture(backend);
        add_claim(&mut sim, 4);
        sim.state.balances.insert((ISSUER, METAL), 2);
        sim.step().unwrap();
        assert_eq!(sim.state.balance(SUPPLIER, METAL), 4);
        assert_eq!(sim.state.balance(ISSUER, METAL), 4);
        assert!(
            sim.ledger
                .last()
                .unwrap()
                .minting
                .as_ref()
                .unwrap()
                .receipts
                .iter()
                .any(|r| r.accepted)
        );
        sim
    };
    let reference = run(Backend::Reference);
    let cpu = run(Backend::CubeCpu);
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
}

#[test]
fn default_horizon_signed_floor_and_failed_package_have_truthful_budgets() {
    let mut sim = fixture(Backend::Reference);
    sim.world
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .private_sales = None;
    add_claim(&mut sim, 4);
    let plan = |sim: &Simulation| {
        minting::evaluate(&sim.world, &sim.state)
            .unwrap()
            .unwrap()
            .plan
            .unwrap()
    };
    // Month two is outside the one-month fallback at the month-one boundary.
    assert_eq!(plan(&sim).sales[0].feasible_lots, 2);
    let t = sim.world.prepaid_deliveries.last_mut().unwrap();
    t.due = 1;
    sim.state.exchange.forwards.insert(t.id, t.contract());
    assert_eq!(plan(&sim).sales[0].feasible_lots, 1);
    assert_eq!(plan(&sim).sales[0].matched_lots, 0);
    let quote = sim
        .world
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes
        .iter_mut()
        .find(|q| q.market == METAL)
        .unwrap();
    quote.holding = 5;
    let floor = plan(&sim);
    assert_eq!(
        (floor.sales[0].protected_stock, floor.sales[0].feasible_lots),
        (5, 0)
    );
    // A partial input package is rolled back, not reported as a metal fill.
    assert_eq!(floor.sales[0].matched_lots, 0);
    assert!(floor.deals.iter().all(|d| d.market != METAL));
}

#[test]
fn legacy_provisioning_does_not_publish_discarded_fixed_route_budgets() {
    let (w, s) = minting::provision_scenario("adequate").unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(3).unwrap();
    for plan in sim
        .ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .filter_map(|b| b.plan.as_ref())
    {
        assert!(plan.sales.is_empty());
    }
}
