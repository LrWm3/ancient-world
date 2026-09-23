use economics_compute_smoke::{
    compute::Backend,
    marketplace::Side,
    model::*,
    negotiation::{GRAIN_MARKET, Outcome},
    production_market::{self as pm, Choice, Policy, Purchases, WOOD_MARKET, Work},
    scenario::{FUEL, GRAIN, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    town_market::{self, Boundary, ClearingPriority},
};
fn fixture() -> (World, State) {
    let (mut w, mut s) = pm::reciprocal_scenario(true);
    // Equal-price two-unit test lots isolate competition for one opening budget.
    w.marketplaces[0]
        .markets
        .iter_mut()
        .find(|m| m.id == WOOD_MARKET)
        .unwrap()
        .goods
        .quantity = 2;
    for t in &mut w.town_market.as_mut().unwrap().additional[0].traders {
        t.trader.limit = 4;
        t.trader.opening_quote = 4;
    }
    let choices = [88, 89, 91, 92]
        .into_iter()
        .map(|a| {
            (
                a,
                Choice {
                    work: Work::Wait,
                    buy: if a == 88 {
                        Purchases::All
                    } else {
                        Purchases::None
                    },
                },
            )
        })
        .collect();
    w.production_market.as_mut().unwrap().policy = Policy::Fixed(choices);
    for a in [88, 89, 91, 92] {
        s.balances.insert((a, GRAIN), if a == 89 { 8 } else { 0 });
        s.balances.insert((a, FUEL), if a == 89 { 8 } else { 0 });
        s.balances.insert((a, TOKEN), if a == 88 { 8 } else { 0 });
    }
    (w, s)
}
fn opening(w: World, s: State) -> Simulation {
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    sim
}
#[test]
fn books_share_money_and_explicit_priority_resolves_competing_purchases() {
    let (w, mut s) = fixture();
    s.balances.insert((88, TOKEN), 4);
    let mut orders = None;
    for (priority, winner, loser) in [
        (ClearingPriority::MarketId, GRAIN_MARKET, WOOD_MARKET),
        (ClearingPriority::ReverseMarketId, WOOD_MARKET, GRAIN_MARKET),
    ] {
        let mut w = w.clone();
        w.town_market.as_mut().unwrap().priority = priority;
        let mut sim = opening(w, s.clone());
        sim.step().unwrap();
        let r = &sim.state.town_market.history[0];
        assert_eq!(
            (r.markets[&winner].volume, r.markets[&loser].volume),
            (2, 0)
        );
        assert_eq!(r.markets[&loser].posted_price, None);
        assert_eq!(sim.state.balance(88, TOKEN), 0);
        assert_eq!(sim.state.balance(89, TOKEN), 4);
        assert!(
            r.attempts
                .iter()
                .any(|a| a.session.market == loser
                    && a.round.outcome == Outcome::InsufficientPayment)
        );
        let mut current = r.orders.clone();
        current.sort_by_key(|o| (o.market, o.agent));
        if let Some(o) = &orders {
            assert_eq!(&current, o);
        } else {
            orders = Some(current);
        }
    }
}
#[test]
fn receipts_and_beliefs_do_not_mix_resource_volumes_or_prices() {
    let (mut w, s) = fixture();
    for t in &mut w.town_market.as_mut().unwrap().additional[0].traders {
        t.trader.limit = 2;
        t.trader.opening_quote = 2;
    }
    let mut sim = opening(w, s);
    sim.step().unwrap();
    let r = &sim.state.town_market.history[0];
    assert_eq!(r.markets[&GRAIN_MARKET].posted_price, Some(4));
    assert_eq!(r.markets[&WOOD_MARKET].posted_price, Some(2));
    assert_eq!(sim.state.balance(88, TOKEN), 2);
    assert!(
        pm::belief(&sim.world, &sim.state)
            .values()
            .all(|b| b.price.is_none())
    );
    sim.run_months(1).unwrap();
    let beliefs = pm::belief(&sim.world, &sim.state);
    assert_eq!(beliefs[&GRAIN_MARKET].price, Some(4));
    assert_eq!(beliefs[&WOOD_MARKET].price, Some(2));
    assert!(beliefs.values().all(|b| b.lots_per_month == 1));
}
#[test]
fn storage_is_shared_and_sales_cannot_finance_same_boundary_purchases() {
    let (mut w, s) = fixture();
    w.storage.capacities.insert(88, 3); // seed plus one two-unit lot
    let mut sim = opening(w, s);
    sim.step().unwrap();
    let r = &sim.state.town_market.history[0];
    assert_eq!(
        (
            r.markets[&GRAIN_MARKET].volume,
            r.markets[&WOOD_MARKET].volume
        ),
        (2, 0)
    );
    assert!(
        r.attempts
            .iter()
            .any(|a| a.round.outcome == Outcome::InsufficientStorage)
    );
    let (mut w, mut s) = fixture();
    s.balances.insert((88, GRAIN), 8);
    s.balances.insert((88, TOKEN), 0);
    s.balances.insert((89, GRAIN), 0);
    s.balances.insert((89, TOKEN), 4);
    if let Policy::Fixed(p) = &mut w.production_market.as_mut().unwrap().policy {
        p.get_mut(&88).unwrap().buy = Purchases::Market(WOOD_MARKET);
        p.get_mut(&89).unwrap().buy = Purchases::Market(GRAIN_MARKET);
    }
    let mut sim = opening(w, s);
    sim.step().unwrap();
    assert_eq!(sim.state.balance(88, TOKEN), 4);
    assert_eq!(
        sim.state.town_market.history[0].markets[&WOOD_MARKET].volume,
        0
    );
    // Proceeds become usable next month, without extending either private limit.
    sim.run_months(1).unwrap();
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(
        sim.state.town_market.history[1].markets[&WOOD_MARKET].volume,
        2
    );
}
#[test]
fn both_market_legs_commit_atomically_and_replay_is_rejected() {
    let (w, s) = fixture();
    let sim = opening(w, s);
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert_eq!(r.transactions.len(), 2);
    let mut original = Batch::empty(&sim.state);
    original.transactions = r.transactions.clone();
    original.town_market = Some(Boundary::Market(r));
    for mode in 0..3 {
        let mut b = original.clone();
        match mode {
            0 => {
                b.transactions.pop();
            }
            1 => {
                if let Some(Boundary::Market(r)) = &mut b.town_market {
                    r.markets.get_mut(&WOOD_MARKET).unwrap().volume += 2;
                }
            }
            _ => b.transactions[1].effects[0].delta += 1,
        }
        let mut s = sim.state.clone();
        assert!(
            settlement::commit(
                &sim.world,
                &mut s,
                &b,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(s, sim.state);
    }
    let mut s = sim.state.clone();
    settlement::commit(
        &sim.world,
        &mut s,
        &original,
        Backend::CubeCpu,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    let settled = s.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut s,
            &original,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s, settled);
}
#[test]
fn invalid_duplicate_goods_currency_and_participants_are_rejected() {
    for mode in 0..4 {
        let (mut w, s) = fixture();
        match mode {
            0 => w.town_market.as_mut().unwrap().additional[0].market = GRAIN_MARKET,
            1 => w.marketplaces[0].markets[1].goods.resource = GRAIN,
            2 => w.marketplaces[0].markets[1].payment = GRAIN,
            _ => {
                w.town_market.as_mut().unwrap().additional[0].traders.pop();
            }
        }
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}
#[test]
fn a_person_can_buy_food_and_offer_wood_from_one_opening_state() {
    let (mut w, mut s) = fixture();
    s.balances.insert((88, FUEL), 4);
    if let Policy::Fixed(p) = &mut w.production_market.as_mut().unwrap().policy {
        p.get_mut(&88).unwrap().buy = Purchases::Market(GRAIN_MARKET);
    }
    let sim = opening(w, s);
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert!(
        r.orders
            .iter()
            .any(|o| o.agent == 88 && o.market == GRAIN_MARKET && o.side == Side::Buy)
    );
    assert!(
        r.orders
            .iter()
            .any(|o| o.agent == 88 && o.market == WOOD_MARKET && o.side == Side::Sell)
    );
}
#[test]
fn cpu_reordered_catalogs_and_acquire_checkpoint_match_reference() {
    let (w, s) = pm::reciprocal_scenario(true);
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    reference.run_months(4).unwrap();
    let mut w = w;
    w.participants.reverse();
    w.definitions.reverse();
    w.resources.reverse();
    w.marketplaces[0].markets.reverse();
    w.town_market.as_mut().unwrap().traders.reverse();
    w.town_market.as_mut().unwrap().additional[0]
        .traders
        .reverse();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    for _ in 0..4 {
        cpu.step().unwrap();
        let resumed =
            Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu).unwrap();
        cpu.state = resumed.state;
        cpu.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    assert_eq!(
        cpu.state
            .balances
            .iter()
            .filter(|((_, r), _)| *r == TOKEN)
            .map(|(_, q)| *q)
            .sum::<i32>(),
        96
    );
    assert!(
        cpu.state
            .town_market
            .history
            .iter()
            .filter_map(|r| r.planning.as_ref())
            .all(|d| d.people.iter().all(|p| p.alternatives.len() == 16))
    );
}

#[test]
fn unfilled_bids_signal_interest_but_cannot_create_funding_or_a_price() {
    let (w, mut s) = fixture();
    s.balances.insert((88, TOKEN), 0);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    let beliefs = pm::belief(&sim.world, &sim.state);
    for id in [GRAIN_MARKET, WOOD_MARKET] {
        assert_eq!(beliefs[&id].interested_lots, 1);
        assert_eq!(beliefs[&id].lots_per_month, 0);
        assert_eq!(beliefs[&id].price, None);
    }
    sim.world.production_market.as_mut().unwrap().policy = Policy::Plan;
    sim.step().unwrap();
    let d = pm::choose(&sim.world, &sim.state).unwrap().unwrap();
    assert!(
        d.people
            .iter()
            .flat_map(|p| &p.alternatives)
            .all(|f| f.closing_coins == 0
                && f.sales.is_empty()
                && f.purchases.is_empty()
                && f.stock_value == 0)
    );
}

#[test]
fn directed_complementary_work_recurs_without_exhausting_buyer_money() {
    let (mut w, s) = pm::reciprocal_scenario(true);
    w.production_market.as_mut().unwrap().policy = Policy::Fixed(
        w.participants
            .iter()
            .map(|p| {
                let grower = [88, 89].contains(&p.agent);
                (
                    p.agent,
                    Choice {
                        work: Work::Produce(if grower {
                            economics_compute_smoke::scenario::GROW
                        } else {
                            economics_compute_smoke::scenario::PREPARE_FUEL
                        }),
                        buy: Purchases::Market(if grower { WOOD_MARKET } else { GRAIN_MARKET }),
                    },
                )
            })
            .collect(),
    );
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(12).unwrap();
    let coins: Vec<_> = [88, 89, 91, 92]
        .iter()
        .map(|a| sim.state.balance(*a, TOKEN))
        .collect();
    sim.run_months(12).unwrap();
    assert_eq!(
        coins,
        [88, 89, 91, 92]
            .iter()
            .map(|a| sim.state.balance(*a, TOKEN))
            .collect::<Vec<_>>()
    );
    assert!(coins.iter().all(|c| *c > 0));
    assert_eq!(coins.iter().sum::<i32>(), 96);
    assert!(
        sim.reports
            .iter()
            .all(|r| r.needs.values().all(|n| n.deficit == 0))
    );
    for id in [GRAIN_MARKET, WOOD_MARKET] {
        assert!(
            sim.state
                .town_market
                .history
                .iter()
                .filter(|r| r.month > 12)
                .any(|r| r.markets[&id].volume > 0)
        );
    }
    assert!(
        !sim.state
            .processes
            .values()
            .any(|p| p.status == Status::Aborted)
    );
}
