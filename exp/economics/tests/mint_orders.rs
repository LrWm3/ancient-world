use economics_compute_smoke::{
    compute::Backend,
    marketplace::Side,
    minting::{self, *},
    model::*,
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
fn sim(case: &str, backend: Backend) -> Simulation {
    let (w, s) = minting::order_scenario(case).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn boundary(s: &Simulation, month: u32) -> &minting::Boundary {
    s.ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .find(|b| b.month == month)
        .unwrap()
}
fn minted(s: &Simulation) -> usize {
    s.state
        .processes
        .values()
        .filter(|p| p.definition == MINT && p.status == Status::Completed)
        .count()
}
#[test]
fn generated_cpu_orders_reproduce_fixed_control_without_supplied_deals() {
    let mut s = sim("normal", Backend::CubeCpu);
    assert!(s.world.minting.as_ref().unwrap().deals.is_empty());
    s.run_months(3).unwrap();
    let first = boundary(&s, 1);
    let plan = first.plan.as_ref().unwrap();
    assert_eq!(plan.required_funding, 6);
    assert!(
        plan.orders
            .iter()
            .any(|o| o.agent == ISSUER && o.side == Side::Sell && o.lots == 2)
    );
    assert!(
        !plan
            .orders
            .iter()
            .any(|o| o.agent == ISSUER && o.side == Side::Buy)
    );
    assert_eq!(first.deals.len(), 2);
    let second = boundary(&s, 2);
    assert_eq!(second.deals.len(), 2);
    assert_eq!(second.receipts.len(), 1);
    assert!(second.receipts[0].accepted);
    assert_eq!(minted(&s), 1);
    assert_eq!(s.state.balance(WORKER, FIREWOOD), 0);
    assert!(boundary(&s, 3).deals.is_empty());
    let (w, state) = minting::scenario("normal").unwrap();
    let mut fixed = Simulation::new(w, state, Backend::Reference).unwrap();
    fixed.run_months(3).unwrap();
    assert_eq!(fixed.state.balances, s.state.balances);
}
#[test]
fn partial_stock_and_existing_money_reduce_orders() {
    let mut s = sim("normal", Backend::CubeCpu);
    s.state.balances.insert((ISSUER, METAL), 2);
    s.state.balances.insert((ISSUER, COIN), 1);
    s.run_months(2).unwrap();
    assert_eq!(boundary(&s, 1).plan.as_ref().unwrap().required_funding, 4);
    assert_eq!(boundary(&s, 1).deals.len(), 1);
    assert_eq!(s.state.balance(ISSUER, WHEAT), 3);
    assert_eq!(boundary(&s, 2).deals.len(), 1);
    assert_eq!(boundary(&s, 2).deals[0].market, HOURS);
    assert_eq!(minted(&s), 1);
}
#[test]
fn shortfalls_and_non_crossing_prices_leave_input_package_unspent() {
    for case in ["treasury", "metal", "labor", "price", "demand", "storage"] {
        let mut s = sim(
            if ["treasury", "metal", "labor"].contains(&case) {
                case
            } else {
                "normal"
            },
            Backend::CubeCpu,
        );
        let p = s
            .world
            .minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap();
        match case {
            "price" => {
                p.quotes
                    .iter_mut()
                    .find(|q| q.market == HOURS)
                    .unwrap()
                    .limit = 5
            }
            "demand" => p
                .quotes
                .iter_mut()
                .filter(|q| q.market == WHEAT)
                .for_each(|q| q.holding = 0),
            "storage" => {
                s.world.storage.capacities.insert(ISSUER, 0);
                s.state.balances.remove(&(ISSUER, WHEAT));
                s.state.balances.insert((ISSUER, COIN), 6);
            }
            _ => {}
        }
        s.run_months(2).unwrap();
        assert_eq!(minted(&s), 0, "{case}");
        assert!(boundary(&s, 2).transactions.is_empty(), "{case}");
        assert_eq!(s.state.balance(ISSUER, METAL), 0, "{case}");
        assert_eq!(
            s.state.balance(ISSUER, COIN),
            match case {
                "treasury" => 3,
                "demand" => 0,
                _ => 6,
            },
            "{case}"
        );
        assert_eq!(
            s.state.balance(WORKER, FIREWOOD),
            if case == "labor" { 0 } else { 1 },
            "{case}"
        );
    }
}
#[test]
fn cheaper_ask_sets_trade_price_and_does_not_exceed_bid() {
    let mut s = sim("normal", Backend::CubeCpu);
    s.world
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes
        .iter_mut()
        .find(|q| q.market == HOURS)
        .unwrap()
        .limit = 3;
    s.run_months(2).unwrap();
    assert_eq!(
        boundary(&s, 2)
            .deals
            .iter()
            .find(|d| d.market == HOURS)
            .unwrap()
            .price,
        3
    );
    assert_eq!(s.state.balance(ISSUER, COIN), 11);
    assert_eq!(minted(&s), 1);
}
#[test]
fn funded_state_waits_for_dated_capacity_and_expired_target_does_not_retry() {
    let mut s = sim("normal", Backend::CubeCpu);
    s.state.balances.insert((ISSUER, COIN), 6);
    s.run_months(1).unwrap();
    assert!(boundary(&s, 1).deals.is_empty());
    assert!(
        boundary(&s, 1)
            .plan
            .as_ref()
            .unwrap()
            .reason
            .contains("waiting")
    );
    assert_eq!(s.state.balance(ISSUER, WHEAT), 6);
    s.run_months(3).unwrap();
    assert_eq!(minted(&s), 1);
    assert!(boundary(&s, 3).plan.as_ref().unwrap().orders.is_empty());
}
#[test]
fn cpu_reference_reordering_and_each_phase_restart_agree() {
    for case in ["normal", "treasury", "metal", "labor"] {
        let mut reference = sim(case, Backend::Reference);
        reference.run_months(3).unwrap();
        let mut cpu = sim(case, Backend::CubeCpu);
        cpu.world.agents.reverse();
        cpu.world.participants.reverse();
        cpu.world.marketplaces[0].markets.reverse();
        cpu.world
            .minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap()
            .quotes
            .reverse();
        while cpu.state.month <= 3 {
            cpu.step().unwrap();
            cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                .unwrap()
                .state;
        }
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
    }
}
#[test]
fn altered_order_receipt_rejects_atomically_and_invalid_terms_are_errors() {
    let mut s = sim("normal", Backend::CubeCpu);
    s.step().unwrap();
    let before = s.state.clone();
    let mut b = Batch::empty(&s.state);
    b.minting = minting::evaluate(&s.world, &s.state).unwrap();
    b.transactions = b.minting.as_ref().unwrap().transactions.clone();
    b.minting.as_mut().unwrap().plan.as_mut().unwrap().orders[0].limit += 1;
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &b,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, s.state);
    for bad in ["tick", "date", "listing", "budget"] {
        let (mut w, state) = minting::order_scenario("normal").unwrap();
        let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
        match bad {
            "tick" => p.sale_limit = 0,
            "date" => p.month = 3,
            "listing" => {
                p.input_limits.remove(&METAL);
            }
            _ => {
                p.input_limits.insert(METAL, i32::MAX);
                p.input_limits.insert(HOURS, i32::MAX);
            }
        }
        assert!(
            Simulation::new(w, state, Backend::Reference).is_err(),
            "{bad}"
        );
    }
}
#[test]
fn observer_exposes_generated_orders_and_resolved_counterparties() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let mut s = sim("normal", Backend::CubeCpu);
    let mut o = Observer::new(
        Vec::new(),
        "orders",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    o.run_months(&mut s, 3).unwrap();
    let bytes = o.finish().unwrap();
    let rows: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(
        rows.iter()
            .any(|r| r["kind"] == "physical_minting_orders" && r["required_funding"] == 6)
    );
    assert!(rows.iter().any(|r| r["kind"] == "physical_minting_market"
        && r["deals"].as_array().is_some_and(|a| a.len() == 2)));
    let mut plain = sim("normal", Backend::CubeCpu);
    plain.run_months(3).unwrap();
    assert_eq!(plain.state, s.state);
    assert_eq!(plain.ledger, s.ledger);
}

#[test]
fn chooses_cheapest_available_counterparty_and_respects_seller_reserve() {
    for reserve in [0, 2] {
        let mut s = sim("normal", Backend::CubeCpu);
        s.state.balances.insert((WORKER, METAL), 2);
        s.world
            .minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap()
            .quotes
            .push(minting::orders::Quote {
                agent: WORKER,
                market: METAL,
                side: Side::Sell,
                limit: 1,
                holding: reserve,
            });
        s.run_months(2).unwrap();
        let deal = boundary(&s, 2)
            .deals
            .iter()
            .find(|d| d.market == METAL)
            .unwrap();
        assert_eq!(deal.seller, if reserve == 0 { WORKER } else { SUPPLIER });
        assert_eq!(deal.price, if reserve == 0 { 1 } else { 2 });
        assert_eq!(minted(&s), 1);
    }
}

#[test]
fn generated_orders_obey_laws_and_same_month_revenue_is_not_funding() {
    use economics_compute_smoke::opportunities::{self, Action};
    for case in ["mint", "capacity", "admission", "same_month", "disabled"] {
        let mut s = sim("normal", Backend::CubeCpu);
        match case {
            "mint" => {
                s.world
                    .transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(opportunities::STATE_TYPE, Action::Process(MINT)));
            }
            "capacity" => {
                s.world
                    .transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(opportunities::PERSON_TYPE, Action::CapacityTrade));
            }
            "admission" => {
                s.world.marketplaces[0]
                    .allowed_types
                    .remove(&opportunities::STATE_TYPE);
            }
            "disabled" => {
                s.world
                    .definitions
                    .iter_mut()
                    .find(|d| d.id == MINT)
                    .unwrap()
                    .enabled = false;
            }
            _ => {
                s.world
                    .minting
                    .as_mut()
                    .unwrap()
                    .order_policy
                    .as_mut()
                    .unwrap()
                    .month = 1;
                s.world
                    .scheduled_starts
                    .iter_mut()
                    .find(|p| p.agent == ISSUER)
                    .unwrap()
                    .month = 1;
            }
        }
        s.run_months(2).unwrap();
        assert_eq!(minted(&s), 0, "{case}");
        assert!(boundary(&s, 2).transactions.is_empty(), "{case}");
        if case != "capacity" {
            assert!(boundary(&s, 1).transactions.is_empty(), "{case}");
        }
    }
}
