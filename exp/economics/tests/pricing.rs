use economics_compute_smoke::{
    compute::Backend,
    crafts::HOE,
    currency::{self, StockTrade},
    equipment::DurableAsset,
    model::*,
    scenario::*,
    simulation::Simulation,
    trading_scenario::*,
};

#[test]
fn state_spreads_peer_prices_forward_discount_and_taxes_are_distinct() {
    let (w, mut s) = cash_scenario(1, true).unwrap();
    let (old, _) = scenario(1, 25).unwrap();
    assert_eq!(w.agreements, old.agreements);
    assert_eq!(w.issuance, old.issuance);
    assert_eq!(w.activities.coin_payments, old.activities.coin_payments);
    s.phase = Phase::Acquire;
    s.balances.insert((STATE_AGENT, GRAIN), 10000);
    s.balances.insert((STATE_AGENT, TOKEN), 10000);
    let state_bid = w
        .bids
        .iter()
        .find(|b| b.buyer == STATE_AGENT && b.goods.resource == GRAIN)
        .unwrap();
    let person_bid = w
        .bids
        .iter()
        .find(|b| b.buyer == PERSON && b.goods.resource == GRAIN)
        .unwrap();
    let sale = currency::transaction(
        &w,
        &s,
        StockTrade {
            bid: state_bid.id,
            seller: PERSON,
        },
    )
    .unwrap();
    let buy = currency::transaction(
        &w,
        &s,
        StockTrade {
            bid: person_bid.id,
            seller: STATE_AGENT,
        },
    )
    .unwrap();
    assert_eq!(
        sale.effects
            .iter()
            .find(|e| e.account == (PERSON, TOKEN))
            .unwrap()
            .delta,
        75
    );
    assert_eq!(
        buy.effects
            .iter()
            .find(|e| e.account == (PERSON, TOKEN))
            .unwrap()
            .delta,
        -150
    );
    let peer = currency::terms(&w, person_bid, PERSON + 1).unwrap();
    assert_eq!(peer.payment.quantity, 100);
    for t in [sale, buy] {
        for r in [GRAIN, TOKEN] {
            assert_eq!(
                t.effects
                    .iter()
                    .filter(|e| e.account.1 == r)
                    .map(|e| e.delta)
                    .sum::<i32>(),
                0
            );
        }
    }
    let policy = w.market.as_ref().unwrap().cash.as_ref().unwrap();
    assert_eq!(policy.prices[&GRAIN].coins, 75);
    assert_eq!(policy.advance_prices[&GRAIN].coins, 50);
    // Gross 1-coin forward proceeds pledge goods worth 1.5 coins at the spot bid.
    assert_eq!(
        100 * policy.advance_prices[&GRAIN].goods / policy.advance_prices[&GRAIN].coins,
        200
    );
    assert_eq!(
        200 * policy.prices[&GRAIN].coins / policy.prices[&GRAIN].goods,
        150
    );
    s.balances.insert((PERSON, TOKEN), 149);
    assert!(
        currency::transaction(
            &w,
            &s,
            StockTrade {
                bid: person_bid.id,
                seller: STATE_AGENT
            }
        )
        .is_err()
    );
}

#[test]
fn completion_tools_double_harvest_but_never_multiply_recycled_seed() {
    for equipped in [false, true] {
        let (mut w, mut s) = cash_scenario(1, true).unwrap();
        w.market.as_mut().unwrap().tools.clear();
        w.market.as_mut().unwrap().targets.clear();
        w.activities
            .orders
            .retain(|o| o.agent == PERSON && o.definition == GROW);
        for p in &mut w.participants {
            for n in &mut p.needs {
                n.quantity = 0;
            }
        }
        if equipped {
            s.equipment.insert(
                9000,
                DurableAsset {
                    id: 9000,
                    owner: PERSON,
                    kind: HOE,
                    remaining_uses: 24,
                    last_used_month: None,
                    attached_to: None,
                },
            );
        }
        let opening_seed = s.balance(PERSON, SEED);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        sim.run_months(12).unwrap();
        let harvests: Vec<_> = sim
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter(|t| {
                t.process.as_ref().is_some_and(|p| {
                    p.after.definition == GROW && p.after.status == Status::Completed
                })
            })
            .collect();
        assert_eq!(harvests.len(), 2);
        for t in harvests {
            assert_eq!(
                t.effects
                    .iter()
                    .filter(|e| e.account == (PERSON, GRAIN) && e.delta > 0)
                    .map(|e| e.delta)
                    .sum::<i32>(),
                if equipped { 1600 } else { 800 }
            );
            assert_eq!(
                t.effects
                    .iter()
                    .filter(|e| e.account == (PERSON, SEED) && e.delta > 0)
                    .map(|e| e.delta)
                    .sum::<i32>(),
                100
            );
        }
        assert_eq!(sim.state.balance(PERSON, SEED), opening_seed);
    }
}
