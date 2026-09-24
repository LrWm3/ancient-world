use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    compute::Backend,
    financial_reporting::Audit,
    inventory_accounting::{Holding, Inventory},
    model::*,
    negotiation,
    scenario::{GRAIN, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;

#[test]
fn negotiated_sale_has_revenue_cost_and_no_buyer_profit_on_cpu() {
    let (w, s) = negotiation::scenario();
    let seller = w.negotiation.as_ref().unwrap().seller.agent;
    let buyer = w.negotiation.as_ref().unwrap().buyer.agent;
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut a = Audit::with_inventory(
        &w,
        &s,
        TOKEN,
        BTreeMap::new(),
        BTreeMap::from([((seller, GRAIN), 14)]),
    )
    .unwrap();
    let mut b = a.clone();
    while reference.state.month == 1 {
        a.step(&mut reference).unwrap();
        b.step(&mut cpu).unwrap();
    }
    assert_eq!(a, b);
    assert_eq!(reference.state, cpu.state);
    let sold = a.book().statements(seller, 1, 1).unwrap();
    assert_eq!(
        (
            sold.income[&A::Sales],
            sold.expenses[&A::CostOfSales],
            sold.net_income
        ),
        (40, 14, 26)
    );
    assert_eq!(sold.cash_flows[&Flow::Operating], 40);
    let bought = a.book().statements(buyer, 1, 1).unwrap();
    assert_eq!(bought.net_income, 0);
    assert_eq!(bought.trial_balance[&A::Inventory(GRAIN)], 40);
    assert_eq!(bought.cash_flows[&Flow::Operating], -40);
    let mut resumed = a.clone();
    let mut continuation = reference.clone();
    while reference.state.month == 2 {
        a.step(&mut reference).unwrap();
        resumed.step(&mut continuation).unwrap();
    }
    assert_eq!(a, resumed);
    assert_eq!(a.book().statements(seller, 2, 2).unwrap().net_income, 0);
}
fn trade(seller: u32, buyer: u32, quantity: i32, price: i32) -> Transaction {
    Transaction {
        cause: "test spot sale".into(),
        effects: vec![
            Effect {
                account: (seller, GRAIN),
                delta: -quantity,
            },
            Effect {
                account: (buyer, GRAIN),
                delta: quantity,
            },
            Effect {
                account: (buyer, TOKEN),
                delta: -price,
            },
            Effect {
                account: (seller, TOKEN),
                delta: price,
            },
        ],
        process: None,
        technique_use: None,
        trade: None,
        stock_trade: None,
        forward: None,
        delivery: None,
        royalty: None,
    }
}
#[test]
fn split_lots_and_order_share_opening_average_cost_and_depletion_clears_rounding() {
    let opening = Inventory(BTreeMap::from([(
        (1, GRAIN),
        Holding {
            quantity: 3,
            cost: 10,
        },
    )]));
    let first = trade(1, 2, 1, 8);
    let second = trade(1, 3, 1, 9);
    let (a, lines) = opening.settle(&[&first, &second], TOKEN).unwrap();
    let (b, _) = opening.settle(&[&second, &first], TOKEN).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        a.0[&(1, GRAIN)],
        Holding {
            quantity: 1,
            cost: 4
        }
    );
    assert_eq!(
        lines
            .iter()
            .find(|l| l.account == A::CostOfSales)
            .unwrap()
            .debit,
        6
    );
    let combined = trade(1, 2, 2, 17);
    assert_eq!(
        opening.settle(&[&combined], TOKEN).unwrap().0.0[&(1, GRAIN)],
        a.0[&(1, GRAIN)]
    );
    let last = trade(1, 2, 1, 5);
    let (closed, lines) = a.settle(&[&last], TOKEN).unwrap();
    assert!(!closed.0.contains_key(&(1, GRAIN)));
    assert_eq!(
        lines
            .iter()
            .find(|l| l.account == A::CostOfSales)
            .unwrap()
            .debit,
        4
    );
    assert_eq!(
        closed.0[&(2, GRAIN)],
        Holding {
            quantity: 2,
            cost: 13
        }
    );
    assert!(
        opening
            .settle(&[&first, &trade(2, 3, 1, 9)], TOKEN)
            .is_err()
    );
}
#[test]
fn missing_cost_and_unrecognized_stock_change_fail_atomically() {
    let (w, s) = negotiation::scenario();
    assert!(Audit::new(&w, &s, TOKEN).is_err());
    let seller = w.negotiation.as_ref().unwrap().seller.agent;
    assert!(
        Audit::with_inventory(
            &w,
            &s,
            TOKEN,
            BTreeMap::new(),
            BTreeMap::from([((seller, GRAIN), -1)])
        )
        .is_err()
    );
    let mut a = Audit::with_inventory(
        &w,
        &s,
        TOKEN,
        BTreeMap::new(),
        BTreeMap::from([((seller, GRAIN), 0)]),
    )
    .unwrap();
    let old = a.clone();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.state.balances.insert((seller, GRAIN), 3);
    let state = sim.state.clone();
    assert!(a.step(&mut sim).is_err());
    assert_eq!(a, old);
    assert_eq!(sim.state, state);
}

#[test]
fn simultaneous_purchase_does_not_reprice_opening_sale_or_overflow_transient_quantity() {
    let opening = Inventory(BTreeMap::from([
        (
            (1, GRAIN),
            Holding {
                quantity: i32::MAX,
                cost: 10,
            },
        ),
        (
            (2, GRAIN),
            Holding {
                quantity: 1,
                cost: 2,
            },
        ),
    ]));
    let sale = trade(1, 3, i32::MAX, 30);
    let buy = trade(2, 1, 1, 7);
    let (a, _) = opening.settle(&[&buy, &sale], TOKEN).unwrap();
    let (b, _) = opening.settle(&[&sale, &buy], TOKEN).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        a.0[&(1, GRAIN)],
        Holding {
            quantity: 1,
            cost: 7
        }
    );
    assert_eq!(
        a.0[&(3, GRAIN)],
        Holding {
            quantity: i32::MAX,
            cost: 30
        }
    );
}

#[test]
fn posted_stock_bid_uses_same_accounting_and_forged_price_is_atomic() {
    use economics_compute_smoke::{currency, settlement};
    let (mut w, s) = negotiation::scenario();
    let seller = w.negotiation.as_ref().unwrap().seller.agent;
    let buyer = w.negotiation.as_ref().unwrap().buyer.agent;
    w.negotiation = None;
    w.transaction_policy = None; // Legacy posted bids do not support the governance driver.
    w.bids.push(currency::Bid {
        id: 1,
        buyer,
        goods: Amount::new(GRAIN, 2),
        payment: Amount::new(TOKEN, 40),
    });
    let mut a = Audit::with_inventory(
        &w,
        &s,
        TOKEN,
        BTreeMap::new(),
        BTreeMap::from([((seller, GRAIN), 14)]),
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    a.step(&mut sim).unwrap();
    let before = sim.state.clone();
    let mut batch = Batch::empty(&before);
    batch.transactions.push(
        currency::transaction(&sim.world, &before, currency::StockTrade { bid: 1, seller })
            .unwrap(),
    );
    let mut after = before.clone();
    settlement::commit(
        &sim.world,
        &mut after,
        &batch,
        Backend::Reference,
        settlement::DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    let old = a.clone();
    let mut forged = batch.clone();
    forged.transactions[0].effects[2].delta += 1;
    assert!(a.record(&sim.world, &before, &forged, &after).is_err());
    assert_eq!(a, old);
    a.record(&sim.world, &before, &batch, &after).unwrap();
    assert_eq!(a.book().statements(seller, 1, 1).unwrap().net_income, 26);
    let saved = a.clone();
    assert!(a.record(&sim.world, &before, &batch, &after).is_err());
    assert_eq!(a, saved);
}

#[test]
fn zip_prices_change_revenue_not_opening_inventory_cost() {
    use economics_compute_smoke::{negotiation::QuotePolicy, zip};
    let (mut w, mut s) = negotiation::scenario();
    let session = w.negotiation.as_mut().unwrap();
    let (seller, buyer) = (session.seller.agent, session.buyer.agent);
    session.buyer.policy = QuotePolicy::Zip(zip::Config::default());
    session.seller.policy = QuotePolicy::Zip(zip::Config::default());
    session.max_rounds = negotiation::MAX_QUOTE_ROUNDS;
    s.balances.insert((seller, GRAIN), 24);
    s.balances.insert((buyer, TOKEN), 600);
    w.storage.capacities.insert(buyer, 24);
    let mut a = Audit::with_inventory(
        &w,
        &s,
        TOKEN,
        BTreeMap::new(),
        BTreeMap::from([((seller, GRAIN), 120)]),
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while sim.state.month <= 6 {
        a.step(&mut sim).unwrap();
    }
    let quantity = 24 - sim.state.balance(seller, GRAIN);
    assert!(quantity > 0);
    let proceeds = i128::from(sim.state.balance(seller, TOKEN));
    let report = a.book().statements(seller, 1, 6).unwrap();
    assert_eq!(report.income[&A::Sales], proceeds);
    assert_eq!(report.expenses[&A::CostOfSales], i128::from(quantity) * 5);
    assert_eq!(report.net_income, proceeds - i128::from(quantity) * 5);
    assert_eq!(
        a.book().statements(buyer, 1, 6).unwrap().trial_balance[&A::Inventory(GRAIN)],
        proceeds
    );
}
