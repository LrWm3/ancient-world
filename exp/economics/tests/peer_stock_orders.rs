use economics_compute_smoke::{
    compute::Backend,
    employment,
    marketplace::{Market, Side},
    minting::{
        self,
        orders::{Quote, peer_stocks},
        *,
    },
    model::*,
    settlement,
    simulation::Simulation,
};

fn fixture() -> (World, State) {
    let (mut w, mut s) = minting::order_scenario("normal").unwrap();
    w.scheduled_starts.clear();
    w.resources.push(Resource {
        id: ORE,
        name: "ore".into(),
        kind: ResourceKind::Stock,
    });
    w.storage.weights.insert(ORE, 1);
    for r in [ORE, FIREWOOD] {
        w.marketplaces[0].markets.push(Market {
            id: r,
            goods: Amount::new(r, 1),
            payment: COIN,
            price_tick: 1,
        });
    }
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.month = 0;
    p.peer_stocks = Some(peer_stocks::Policy {
        claim_months: 1,
        quotes: vec![
            Quote {
                agent: SUPPLIER,
                market: ORE,
                side: Side::Buy,
                limit: 2,
                holding: 2,
                max_lots: None,
            },
            Quote {
                agent: WORKER,
                market: ORE,
                side: Side::Sell,
                limit: 2,
                holding: 0,
                max_lots: None,
            },
        ],
    });
    s.balances.insert((SUPPLIER, COIN), 8);
    s.balances.insert((WORKER, ORE), 4);
    (w, s)
}
fn at_acquire(w: World, s: State) -> Simulation {
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim
}
fn plan(s: &Simulation) -> minting::orders::Plan {
    minting::evaluate(&s.world, &s.state)
        .unwrap()
        .unwrap()
        .plan
        .unwrap()
}

#[test]
fn stock_target_stops_and_restarts_without_a_mint_date_and_enforces_admission() {
    let (w, s) = fixture();
    let mut sim = at_acquire(w, s);
    assert_eq!(plan(&sim).deals.len(), 2);
    sim.state.balances.insert((SUPPLIER, ORE), 2);
    assert!(plan(&sim).orders.iter().all(|o| o.side != Side::Buy));
    sim.state.balances.insert((SUPPLIER, ORE), 1);
    assert_eq!(plan(&sim).deals.len(), 1);
    sim.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(
            economics_compute_smoke::opportunities::PERSON_TYPE,
            economics_compute_smoke::opportunities::Action::StockTrade,
        ));
    assert!(plan(&sim).deals.is_empty());
}

#[test]
fn several_bids_share_cash_after_earned_payroll_protection() {
    let (mut w, mut s) = fixture();
    s.balances.insert((SUPPLIER, COIN), 0);
    s.balances.insert((WORKER, FIREWOOD), 4);
    w.employment.push(employment::Terms {
        id: 900,
        employer: SUPPLIER,
        worker: WORKER,
        from: 1,
        through: 1,
        capacity: Amount::new(HOURS, 1),
        wage_per_unit: Amount::new(COIN, 3),
        on_arrears: employment::ArrearsPolicy::Continue,
        rank: 0,
    });
    let p = w
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
        .as_mut()
        .unwrap();
    let extra: Vec<_> = p
        .quotes
        .iter()
        .cloned()
        .map(|mut q| {
            q.market = FIREWOOD;
            q
        })
        .collect();
    p.quotes.extend(extra);
    let mut first = Simulation::new(w, s, Backend::Reference).unwrap();
    first.run_months(1).unwrap();
    assert_eq!(
        first.state.employment.earned[&(900, 1)].claim.outstanding(),
        3
    );
    // An opening-state funding fixture, retaining the actual earned arrears.
    first.state.balances.insert((SUPPLIER, COIN), 5);
    let sim = at_acquire(first.world, first.state);
    let p = plan(&sim);
    assert!(
        p.purchases
            .iter()
            .all(|b| b.protected_cash == 3 && b.submitted_lots == 1)
    );
    assert_eq!(p.deals.len(), 1);
    assert_eq!(
        p.deals[0].market, FIREWOOD,
        "stable market order allocates scarce cash"
    );
    let mut cpu = Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
    cpu.step().unwrap();
    assert_eq!(cpu.state.balance(SUPPLIER, COIN), 3);
}

#[test]
fn incoming_sales_do_not_fund_purchases_and_unmatched_mint_preserves_peer_trades() {
    let (mut w, mut s) = fixture();
    s.balances.insert((SUPPLIER, COIN), 0);
    s.balances.insert((SUPPLIER, FIREWOOD), 1);
    s.balances.insert((WORKER, COIN), 4);
    let p = w
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
        .as_mut()
        .unwrap();
    p.quotes.extend([
        Quote {
            agent: SUPPLIER,
            market: FIREWOOD,
            side: Side::Sell,
            limit: 2,
            holding: 0,
            max_lots: None,
        },
        Quote {
            agent: WORKER,
            market: FIREWOOD,
            side: Side::Buy,
            limit: 2,
            holding: 1,
            max_lots: None,
        },
    ]);
    let sim = at_acquire(w, s);
    let p = plan(&sim);
    assert_eq!(p.deals.len(), 1);
    assert_eq!(p.deals[0].market, FIREWOOD);
    assert_eq!(
        p.purchases
            .iter()
            .find(|b| b.agent == SUPPLIER)
            .unwrap()
            .submitted_lots,
        0
    );

    let (mut w, mut s) = fixture();
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .month = 1;
    w.scheduled_starts.push(ScheduledStart {
        month: 1,
        agent: ISSUER,
        definition: MINT,
    });
    s.balances.insert((ISSUER, COIN), 6);
    s.balances.insert((SUPPLIER, METAL), 0);
    let sim = at_acquire(w, s);
    let p = plan(&sim);
    assert!(p.reason.starts_with("input package unmatched"));
    assert_eq!(p.deals.len(), 2);
    assert!(
        p.deals
            .iter()
            .all(|d| d.market == ORE && d.buyer == SUPPLIER)
    );
}

#[test]
fn seller_floor_storage_and_tampered_evidence_are_enforced() {
    let (mut w, s) = fixture();
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
        .as_mut()
        .unwrap()
        .quotes[1]
        .holding = 3;
    let sim = at_acquire(w, s);
    assert_eq!(plan(&sim).deals.len(), 1);
    let mut constrained = sim.clone();
    constrained.world.storage.capacities.insert(SUPPLIER, 0);
    assert!(plan(&constrained).deals.is_empty());
    let mut execution = sim.clone();
    execution.step().unwrap();
    let mut forged = execution.ledger.last().unwrap().clone();
    forged
        .minting
        .as_mut()
        .unwrap()
        .plan
        .as_mut()
        .unwrap()
        .purchases[0]
        .protected_cash += 1;
    let mut state = sim.state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut state,
            &forged,
            Backend::Reference,
            sim.effect_limit
        )
        .is_err()
    );
    assert_eq!(state, sim.state);
}

#[test]
fn peer_purchase_observer_names_the_actual_market() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let (w, s) = fixture();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut observer = Observer::new(
        Vec::new(),
        "peer-stock",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 1).unwrap();
    let data = observer.finish().unwrap();
    let rows: Vec<serde_json::Value> = std::str::from_utf8(&data)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(
        rows.iter()
            .any(|r| r["kind"] == "stock_purchase" && r["market"] == ORE && r["settled_lots"] == 2)
    );
}

#[test]
fn invalid_or_overlapping_peer_markets_cannot_be_installed() {
    for case in [
        "duplicate",
        "issuer",
        "capacity",
        "overlap",
        "horizon",
        "price",
        "provisioning",
    ] {
        let (mut w, s) = fixture();
        let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
        let policy = p.peer_stocks.as_mut().unwrap();
        match case {
            "duplicate" => policy.quotes.push(policy.quotes[0].clone()),
            "issuer" => policy.quotes[0].agent = ISSUER,
            "capacity" => {
                w.resources.iter_mut().find(|r| r.id == ORE).unwrap().kind = ResourceKind::Capacity
            }
            "overlap" => {
                w.marketplaces[0]
                    .markets
                    .iter_mut()
                    .find(|m| m.id == ORE)
                    .unwrap()
                    .goods
                    .resource = METAL
            }
            "horizon" => policy.claim_months = 0,
            "price" => policy.quotes[0].limit = 0,
            "provisioning" => {
                let (other, _) = minting::provision_scenario("adequate").unwrap();
                p.provisioning = other.minting.unwrap().order_policy.unwrap().provisioning;
            }
            _ => unreachable!(),
        }
        assert!(Simulation::new(w, s, Backend::Reference).is_err(), "{case}");
    }
}

#[test]
fn legacy_and_peer_bids_share_the_longest_applicable_cash_protection() {
    let (mut w, mut s) = fixture();
    s.balances.insert((SUPPLIER, COIN), 10);
    w.storage.weights.insert(METAL, 0);
    s.balances.insert((SUPPLIER, WHEAT), 0);
    let terms = economics_compute_smoke::forward::direct::Terms {
        id: 900,
        seller: SUPPLIER,
        buyer: ISSUER,
        month: 1,
        due: 4,
        goods: Amount::new(COIN, 6),
        prepayment: Amount::new(METAL, 1),
    };
    s.exchange.forwards.insert(900, terms.contract());
    w.prepaid_deliveries.push(terms);
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.public_sale = Some(minting::orders::StockSales {
        reserve: 0,
        claim_months: 4,
    });
    p.quotes
        .retain(|q| q.agent == SUPPLIER && q.side == Side::Buy);
    let sim = at_acquire(w, s);
    let p = plan(&sim);
    assert_eq!(
        p.purchases
            .iter()
            .find(|b| b.market == WHEAT)
            .unwrap()
            .protected_cash,
        6
    );
    assert_eq!(
        p.purchases
            .iter()
            .find(|b| b.market == ORE)
            .unwrap()
            .protected_cash,
        0
    );
    assert_eq!(p.deals.len(), 1);
    assert_eq!(p.deals[0].market, WHEAT);
    assert_eq!(p.deals[0].price, 3);
    assert_eq!(
        p.purchases
            .iter()
            .find(|b| b.market == ORE)
            .unwrap()
            .matched_lots,
        0
    );
}
