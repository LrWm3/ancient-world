use economics_compute_smoke::negotiation::GRAIN_MARKET;
use economics_compute_smoke::{
    compute::Backend,
    marketplace::Side,
    model::*,
    negotiation::{Outcome, QuotePolicy},
    scenario::{GRAIN, NUTRITION, PERSON, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    town_market::{self, Boundary},
    zip,
};

fn opening(w: World, s: State, backend: Backend) -> Simulation {
    let mut sim = Simulation::new(w, s, backend).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Acquire);
    sim
}
fn batch(sim: &Simulation) -> Batch {
    let mut b = Batch::empty(&sim.state);
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    b.transactions = r.transactions.clone();
    b.town_market = Some(Boundary::Market(r));
    b
}
#[test]
fn cpu_matches_two_pairs_then_releases_no_new_price_without_trades() {
    let (w, s) = town_market::scenario();
    let mut sim = opening(w, s, Backend::CubeCpu);
    sim.step().unwrap();
    let r = &sim.state.town_market.history[0];
    assert_eq!(
        (
            r.markets[&GRAIN_MARKET].volume,
            r.markets[&GRAIN_MARKET].posted_price,
            r.markets[&GRAIN_MARKET].unfilled_buy,
            r.markets[&GRAIN_MARKET].unfilled_sell
        ),
        (4, Some(40), 0, 0)
    );
    assert_eq!(
        r.attempts
            .iter()
            .map(|a| (a.session.buyer.agent, a.session.seller.agent))
            .collect::<Vec<_>>(),
        vec![(PERSON, 89), (91, 92)]
    );
    for a in [PERSON, 91] {
        assert_eq!(sim.state.balance(a, GRAIN), 2);
        assert_eq!(sim.state.balance(a, TOKEN), 60)
    }
    for a in [89, 92] {
        assert_eq!(sim.state.balance(a, GRAIN), 8);
        assert_eq!(sim.state.balance(a, TOKEN), 40)
    }
    sim.run_months(6).unwrap();
    let r = &sim.state.town_market.history[1];
    assert_eq!(
        (
            r.markets[&GRAIN_MARKET].volume,
            r.markets[&GRAIN_MARKET].posted_price
        ),
        (0, None)
    );
    assert_eq!(
        sim.state
            .town_market
            .history
            .iter()
            .map(|r| r.markets[&GRAIN_MARKET].volume)
            .sum::<i32>(),
        8
    );
    // Seller protection prevents trading away the final months' food.
    assert_eq!(
        sim.reports
            .iter()
            .filter(|r| [89, 92].contains(&r.agent))
            .map(|r| r.deficit(NUTRITION))
            .sum::<i32>(),
        0
    );
}
#[test]
fn scarce_lot_has_one_winner_and_id_ties_ignore_registration_order() {
    let (w, mut s) = town_market::scenario();
    s.balances.insert((92, GRAIN), 2);
    let mut expected = None;
    for reverse in [false, true] {
        let mut w = w.clone();
        for t in &mut w.town_market.as_mut().unwrap().traders {
            if t.side == Side::Buy {
                t.trader.limit = 50;
                t.trader.opening_quote = 50
            }
        }
        if reverse {
            w.town_market.as_mut().unwrap().traders.reverse()
        }
        let mut sim = opening(w, s.clone(), Backend::Reference);
        sim.step().unwrap();
        let r = &sim.state.town_market.history[0];
        assert_eq!(
            (
                r.markets[&GRAIN_MARKET].volume,
                r.markets[&GRAIN_MARKET].unfilled_buy
            ),
            (2, 2)
        );
        assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
        assert_eq!(sim.state.balance(91, GRAIN), 0);
        if let Some(e) = &expected {
            assert_eq!(r, e)
        } else {
            expected = Some(r.clone())
        }
    }
}
#[test]
fn failed_funding_or_storage_leaves_seller_available_to_next_buyer() {
    for storage in [false, true] {
        let (mut w, mut s) = town_market::scenario();
        s.balances.insert((92, GRAIN), 2);
        if storage {
            w.storage.capacities.insert(PERSON, 0);
        } else {
            s.balances.insert((PERSON, TOKEN), 0);
        }
        let mut sim = opening(w, s, Backend::Reference);
        sim.step().unwrap();
        let r = &sim.state.town_market.history[0];
        assert_eq!(r.markets[&GRAIN_MARKET].volume, 2);
        assert_eq!(r.attempts.len(), 2);
        assert_eq!(
            r.attempts[0].round.outcome,
            if storage {
                Outcome::InsufficientStorage
            } else {
                Outcome::InsufficientPayment
            }
        );
        assert_eq!(sim.state.balance(91, GRAIN), 2);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
        assert_eq!(r.markets[&GRAIN_MARKET].posted_price, Some(35));
    }
}
#[test]
fn admission_is_dated_at_open_and_does_not_grant_trade_permission() {
    let (w, mut s) = town_market::scenario();
    s.town_market.positions.insert(PERSON, 3);
    let mut sim = opening(w, s, Backend::Reference);
    sim.state.town_market.positions.insert(PERSON, 0);
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert!(!r.orders.iter().any(|o| o.agent == PERSON));
    sim.run_months(1).unwrap();
    sim.step().unwrap();
    assert!(
        sim.state
            .town_market
            .admission
            .as_ref()
            .unwrap()
            .eligible
            .contains(&PERSON)
    );
    // Moving after Open changes next month's locality, not the current snapshot.
    sim.state.town_market.positions.insert(PERSON, 100);
    assert!(
        town_market::evaluate(&sim.world, &sim.state)
            .unwrap()
            .orders
            .iter()
            .any(|o| o.agent == PERSON)
    );
    sim.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .agent_types
        .remove(&PERSON);
    assert!(
        !town_market::evaluate(&sim.world, &sim.state)
            .unwrap()
            .orders
            .iter()
            .any(|o| o.agent == PERSON)
    );
}
#[test]
fn uncrossed_and_all_failed_books_have_no_price_or_volume() {
    for unfunded in [false, true] {
        let (mut w, mut s) = town_market::scenario();
        if unfunded {
            for a in [PERSON, 91] {
                s.balances.insert((a, TOKEN), 0);
            }
        } else {
            for t in &mut w.town_market.as_mut().unwrap().traders {
                if t.side == Side::Buy {
                    t.trader.limit = 10;
                    t.trader.opening_quote = 10;
                }
            }
        }
        let mut sim = opening(w, s, Backend::Reference);
        sim.step().unwrap();
        let r = &sim.state.town_market.history[0];
        assert_eq!(
            (
                r.markets[&GRAIN_MARKET].volume,
                r.markets[&GRAIN_MARKET].posted_price,
                r.markets[&GRAIN_MARKET].unfilled_buy,
                r.markets[&GRAIN_MARKET].unfilled_sell
            ),
            (0, None, 4, 4)
        );
        assert_eq!(sim.state.balance(89, GRAIN), 10);
    }
}
#[test]
fn forged_receipts_transfers_and_replay_do_not_publish_any_part() {
    let (w, s) = town_market::scenario();
    let sim = opening(w, s, Backend::Reference);
    let original = batch(&sim);
    for mode in 0..4 {
        let mut b = original.clone();
        match mode {
            0 => b.town_market = None,
            1 => {
                if let Some(Boundary::Market(r)) = &mut b.town_market {
                    r.markets.get_mut(&GRAIN_MARKET).unwrap().posted_price = Some(1)
                }
            }
            2 => b.transactions.pop().map(|_| ()).unwrap(),
            _ => b.transactions[0].effects[0].delta += 1,
        }
        let mut state = sim.state.clone();
        assert!(
            settlement::commit(
                &sim.world,
                &mut state,
                &b,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(state, sim.state);
    }
    let mut state = sim.state.clone();
    settlement::commit(
        &sim.world,
        &mut state,
        &original,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    let before = state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut state,
            &original,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(state, before);
}
#[test]
fn zip_and_fixed_continue_and_reorder_identically_on_cpu() {
    for zip in [false, true] {
        let (mut w, s) = town_market::scenario();
        if zip {
            for t in &mut w.town_market.as_mut().unwrap().traders {
                t.trader.policy = QuotePolicy::Zip(zip::Config::default());
            }
        }
        let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
        reference.run_months(6).unwrap();
        w.agents.reverse();
        w.participants.reverse();
        w.resources.reverse();
        w.town_market.as_mut().unwrap().traders.reverse();
        let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        for _ in 0..6 {
            cpu.run_months(1).unwrap();
            cpu = cpu.clone();
        }
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
    }
}
#[test]
fn malformed_and_unsupported_combinations_are_rejected() {
    for mode in 0..5 {
        let (mut w, s) = town_market::scenario();
        let c = w.town_market.as_mut().unwrap();
        match mode {
            0 => c.traders.push(c.traders[0].clone()),
            1 => c.reserve.reserve_months = 0,
            2 => c.traders.retain(|t| t.side == Side::Buy),
            3 => c.market = 999,
            _ => {
                w.need_orders =
                    Some(economics_compute_smoke::need_orders::Policy { reserve_months: 2 })
            }
        };
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}

#[test]
fn last_completed_price_does_not_reprice_earlier_transfers() {
    let (mut w, s) = town_market::scenario();
    let t = &mut w.town_market.as_mut().unwrap().traders[0].trader;
    t.limit = 60;
    t.opening_quote = 60;
    let mut sim = opening(w, s, Backend::Reference);
    sim.step().unwrap();
    let r = &sim.state.town_market.history[0];
    assert_eq!(r.attempts[0].round.outcome, Outcome::Traded { price: 45 });
    assert_eq!(r.attempts[1].round.outcome, Outcome::Traded { price: 40 });
    assert_eq!(r.markets[&GRAIN_MARKET].posted_price, Some(40));
    assert_eq!(sim.state.balance(PERSON, TOKEN), 55);
    assert_eq!(
        [PERSON, 89, 91, 92]
            .iter()
            .map(|a| sim.state.balance(*a, TOKEN))
            .sum::<i32>(),
        200
    );
    assert_eq!(
        [PERSON, 89, 91, 92]
            .iter()
            .map(|a| sim.state.balance(*a, GRAIN))
            .sum::<i32>(),
        20
    );
}

#[test]
fn zip_accumulates_failed_and_completed_events_without_requoting_the_book() {
    let (mut w, mut s) = town_market::scenario();
    for t in &mut w.town_market.as_mut().unwrap().traders {
        t.trader.policy = QuotePolicy::Zip(zip::Config::default());
    }
    s.balances.insert((PERSON, TOKEN), 0);
    s.balances.insert((92, GRAIN), 2);
    let mut sim = opening(w, s, Backend::Reference);
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert_eq!(r.attempts.len(), 2);
    let seller = &r.attempts[0].session.seller;
    let mut expected = economics_compute_smoke::marketplace::learning(
        &sim.state,
        &r.attempts[0].session,
        Side::Sell,
    )
    .unwrap();
    for a in &r.attempts {
        assert_eq!(a.round.quotes[0].ask, 30);
        for event in &a.round.events {
            expected.observe(zip::Config::default(), Side::Sell, seller.limit, 1, event);
        }
    }
    sim.step().unwrap();
    let memory = &sim.state.marketplaces[&r.attempts[0].session.marketplace];
    assert_eq!(
        memory.pricing[&(89, r.attempts[0].session.market, Side::Sell)].learning,
        Some(expected)
    );
    assert_eq!(
        sim.state.town_market.history[0].markets[&GRAIN_MARKET].volume,
        2
    );
}

#[test]
fn every_barrier_checkpoint_preserves_admission_history_and_future_matching() {
    let (w, s) = town_market::scenario();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut checkpoints = vec![];
    while sim.state.month < 4 {
        checkpoints.push(sim.state.clone());
        sim.step().unwrap();
    }
    for state in checkpoints {
        let mut resumed = Simulation::new(sim.world.clone(), state, Backend::Reference).unwrap();
        while resumed.state.month < 4 {
            resumed.step().unwrap();
        }
        assert_eq!(resumed.state, sim.state);
    }
    let mut invalid = sim.state.clone();
    invalid.town_market.admission.as_mut().unwrap().month = 99;
    assert!(Simulation::new(sim.world, invalid, Backend::Reference).is_err());
}

#[test]
fn admission_and_buffer_failures_are_atomic() {
    let (w, s) = town_market::scenario();
    let mut initial = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    initial.step().unwrap();
    let mut b = initial.ledger[0].clone();
    if let Some(Boundary::Admission(a)) = &mut b.town_market {
        a.eligible.remove(&PERSON);
    }
    let mut state = s.clone();
    assert!(
        settlement::commit(&w, &mut state, &b, Backend::Reference, DEFAULT_EFFECT_LIMIT).is_err()
    );
    assert_eq!(state, s);
    let b = batch(&initial);
    let mut state = initial.state.clone();
    assert!(settlement::commit(&w, &mut state, &b, Backend::Reference, 1).is_err());
    assert_eq!(state, initial.state);
}

#[test]
fn order_generation_receipts_explain_omissions_and_reject_forgery() {
    use town_market::OrderReason;
    let (w, mut s) = town_market::scenario();
    s.town_market.positions.insert(PERSON, 3);
    s.balances.insert((89, GRAIN), 0);
    let mut sim = opening(w, s, Backend::Reference);
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    let absent = r.order_receipts.iter().find(|r| r.agent == PERSON).unwrap();
    assert_eq!(absent.reason, OrderReason::NotAdmitted);
    assert!(absent.available.is_none());
    let empty = r.order_receipts.iter().find(|r| r.agent == 89).unwrap();
    assert_eq!(empty.reason, OrderReason::InsufficientOpeningStock);
    assert_eq!(empty.available, Some(0));
    let mut b = batch(&sim);
    if let Some(Boundary::Market(r)) = &mut b.town_market {
        r.order_receipts[0].reason = OrderReason::Submitted;
    }
    let before = sim.state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &b,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(sim.state, before);
}

#[test]
fn adaptive_receipts_distinguish_policy_protection_and_buy_first() {
    use economics_compute_smoke::production_market::{self, Choice, Policy, Purchases, Work};
    use town_market::OrderReason;
    let (mut w, s) = production_market::reciprocal_scenario(true);
    w.production_market.as_mut().unwrap().policy = Policy::Fixed(
        w.participants
            .iter()
            .map(|p| {
                (
                    p.agent,
                    Choice {
                        work: Work::Ordinary,
                        buy: Purchases::Market(GRAIN_MARKET),
                    },
                )
            })
            .collect(),
    );
    let mut sim = opening(w, s, Backend::Reference);
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert_eq!(r.order_receipts.len(), 16);
    let wood: Vec<_> = r
        .order_receipts
        .iter()
        .filter(|r| r.market == production_market::WOOD_MARKET)
        .collect();
    for r in wood.iter().filter(|r| r.side == Side::Buy) {
        assert_eq!(r.reason, OrderReason::PurchasePolicy);
        assert!(r.deficits_before.is_none());
    }
    assert!(wood.iter().any(|r| r.reason == OrderReason::ProtectedStock));
    sim.world.production_market.as_mut().unwrap().policy = Policy::Fixed(
        sim.world
            .participants
            .iter()
            .map(|p| (p.agent, Choice::default()))
            .collect(),
    );
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert!(
        r.order_receipts
            .iter()
            .any(|r| r.reason == OrderReason::OtherSideSelected)
    );
    for row in r
        .order_receipts
        .iter()
        .filter(|r| r.reason == OrderReason::Submitted)
    {
        assert!(
            r.orders
                .iter()
                .any(|o| o.agent == row.agent && o.side == row.side && o.market == row.market)
        );
    }
}

#[test]
fn aligned_horizons_remove_this_stock_surplus_and_demand_overlap() {
    use economics_compute_smoke::{
        production_market::{self, Choice, Policy},
        scenario::FUEL,
    };
    use town_market::{OrderHorizon, OrderReason};
    for (policy, buy, protected, side) in [
        (OrderHorizon::Legacy, 6, 2, Side::Buy),
        (OrderHorizon::Aligned(2), 2, 2, Side::Sell),
        (OrderHorizon::Aligned(6), 6, 3, Side::Buy),
    ] {
        let (mut w, mut s) = production_market::reciprocal_scenario(true);
        w.town_market.as_mut().unwrap().order_horizon = policy;
        w.production_market.as_mut().unwrap().policy = Policy::Fixed(
            w.participants
                .iter()
                .map(|p| (p.agent, Choice::default()))
                .collect(),
        );
        s.balances.insert((PERSON, FUEL), 3);
        let sim = opening(w, s, Backend::Reference);
        assert_eq!(sim.world.production_market.as_ref().unwrap().horizon, 6);
        let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
        let rows: Vec<_> = r
            .order_receipts
            .iter()
            .filter(|r| r.agent == PERSON && r.market == production_market::WOOD_MARKET)
            .collect();
        assert_eq!(rows[0].buy_months, buy);
        assert_eq!(rows[0].protected, Some(protected));
        assert!(
            rows.iter()
                .any(|r| r.side == side && r.reason == OrderReason::Submitted)
        );
        if policy == OrderHorizon::Aligned(2) {
            assert_eq!(rows[0].reason, OrderReason::NoNeedImprovement);
        }
    }
    for bad in [0, 25] {
        let (mut w, s) = town_market::scenario();
        w.town_market.as_mut().unwrap().order_horizon = OrderHorizon::Aligned(bad);
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}

#[test]
fn aligned_horizons_match_cpu_checkpoint_and_reordered_fixed_policy_runs() {
    use economics_compute_smoke::production_market::{self, Choice, Policy};
    for months in [2, 6] {
        let (mut w, s) = production_market::reciprocal_scenario(true);
        w.town_market.as_mut().unwrap().order_horizon = town_market::OrderHorizon::Aligned(months);
        w.production_market.as_mut().unwrap().policy = Policy::Fixed(
            w.participants
                .iter()
                .map(|p| (p.agent, Choice::default()))
                .collect(),
        );
        let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
        reference.run_months(4).unwrap();
        w.participants.reverse();
        w.town_market.as_mut().unwrap().traders.reverse();
        w.town_market.as_mut().unwrap().additional[0]
            .traders
            .reverse();
        let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        for _ in 0..4 {
            cpu.step().unwrap();
            cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                .unwrap()
                .state;
            cpu.run_months(1).unwrap();
        }
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        assert_eq!(reference.reports, cpu.reports);
    }
}
