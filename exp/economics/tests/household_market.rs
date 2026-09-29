use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::{Audit, Opening},
    household_governance::{self, Policy, PolicyChange},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
    marketplace::Side,
    model::*,
    negotiation::GRAIN_MARKET,
    process_accounting::Costs,
    scenario::{GRAIN, NUTRITION, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
    town_market::{self, OrderReason},
};
fn fixture() -> (World, State) {
    households::market::scenario().unwrap()
}

#[test]
fn zip_collective_orders_preserve_learning_across_reordered_cpu_continuation() {
    use economics_compute_smoke::{negotiation::QuotePolicy, zip};
    let (mut w, s) = fixture();
    for entry in &mut w.town_market.as_mut().unwrap().traders {
        entry.trader.policy = QuotePolicy::Zip(zip::Config::default());
    }
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    reference.run_months(3).unwrap();
    w.agents.reverse();
    w.participants.reverse();
    w.resources.reverse();
    w.town_market.as_mut().unwrap().traders.reverse();
    let mut cpu = Simulation::new(w.clone(), s, Backend::CubeCpu).unwrap();
    cpu.step().unwrap();
    cpu.step().unwrap();
    let mut resumed = Simulation::new(w, cpu.state.clone(), Backend::CubeCpu).unwrap();
    while cpu.state.month <= 3 {
        cpu.step().unwrap();
        resumed.step().unwrap();
        assert_eq!(resumed.state, cpu.state);
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| *r == GRAIN && **q > 0)
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            processes: Some(Costs::default()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
#[test]
fn collective_buys_feed_real_members_and_reconcile_separate_books_on_cpu() {
    let (w, s) = fixture();
    let run = |backend| {
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        a.step(&mut sim).unwrap();
        let before = (sim.state.clone(), a.clone());
        a.step(&mut sim).unwrap();
        assert_eq!(sim.state.balance(HOME, GRAIN), 2);
        assert_eq!(sim.state.balance(HOME, TOKEN), 60);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
        assert_eq!(
            sim.state.town_market.history[0].markets[&GRAIN_MARKET].volume,
            2
        );
        let checkpoint = (sim.clone(), a.clone());
        through(&mut a, &mut sim, 3);
        assert_eq!(sim.state.balance(HOME, TOKEN), 20);
        let people = &sim.world.households[0].adults;
        for month in [1, 2] {
            assert_eq!(
                sim.reports
                    .iter()
                    .filter(|r| r.month == month && people.contains(&r.agent))
                    .map(|r| r.deficit(NUTRITION))
                    .sum::<i32>(),
                0
            );
        }
        assert_eq!(
            sim.reports
                .iter()
                .filter(|r| r.month == 3 && people.contains(&r.agent))
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>(),
            2
        );
        for id in [HOME, PERSON, 91, 89, 92] {
            let report = a.book().statements(id, 1, 3).unwrap();
            assert_eq!(report.assets, report.liabilities + report.equity);
        }
        let (mut resumed, mut ra) = checkpoint;
        through(&mut ra, &mut resumed, 3);
        assert_eq!(
            (sim.state.clone(), sim.ledger.clone(), a.clone()),
            (resumed.state, resumed.ledger, ra)
        );
        assert_ne!(sim.state, before.0);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn member_private_food_reduces_collective_demand_without_becoming_sale_inventory() {
    let (mut w, mut s) = fixture();
    for id in w.households[0].adults.clone() {
        s.balances.insert((id, GRAIN), 1);
    }
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    sim.step().unwrap();
    sim.step().unwrap();
    let r = &sim.state.town_market.history[0];
    assert!(r.orders.iter().all(|o| o.agent != HOME));
    assert_eq!(
        r.order_receipts
            .iter()
            .find(|r| r.agent == HOME)
            .unwrap()
            .reason,
        OrderReason::NoNeedImprovement
    );
    w.town_market.as_mut().unwrap().adaptive = true;
    for t in &mut w.town_market.as_mut().unwrap().traders {
        t.trader.opening_quote = t.trader.limit;
    }
    for id in w.households[0].adults.clone() {
        s.balances.insert((id, GRAIN), 8);
        w.storage.capacities.insert(id, 20);
    }
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    sim.step().unwrap();
    assert!(
        sim.state.town_market.history[0]
            .orders
            .iter()
            .all(|o| o.agent != HOME)
    );
    assert_eq!(sim.state.balance(HOME, GRAIN), 0);
}
#[test]
fn authorized_policy_change_enables_buying_at_its_dated_boundary() {
    let (mut w, s) = fixture();
    w.households[0].governance.charter.initial_policy = Policy::NetOutput;
    household_governance::schedule(
        &mut w,
        &s,
        HOME,
        PolicyChange {
            month: 2,
            authorized_by: PERSON,
            policy: Policy::NeedsFirst,
        },
    )
    .unwrap();
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 2);
    assert_eq!(
        sim.state.town_market.history[0]
            .order_receipts
            .iter()
            .find(|r| r.agent == HOME)
            .unwrap()
            .reason,
        OrderReason::PurchasePolicy
    );
    assert_eq!(
        sim.state.town_market.history[1].markets[&GRAIN_MARKET].volume,
        2
    );
}
#[test]
fn permissions_locality_and_private_money_do_not_become_collective_funding() {
    for mode in 0..4 {
        let (mut w, mut s) = fixture();
        match mode {
            0 => {
                s.town_market.positions.insert(HOME, 1000);
            }
            1 => {
                w.marketplaces[0]
                    .allowed_types
                    .remove(&economics_compute_smoke::opportunities::HOUSEHOLD_TYPE);
            }
            2 => {
                w.transaction_policy.as_mut().unwrap().permissions.remove(&(
                    economics_compute_smoke::opportunities::HOUSEHOLD_TYPE,
                    economics_compute_smoke::opportunities::Action::StockTrade,
                ));
            }
            _ => {
                s.balances.insert((HOME, TOKEN), 0);
                s.balances.insert((PERSON, TOKEN), 1000);
            }
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        sim.step().unwrap();
        sim.step().unwrap();
        assert_eq!(sim.state.balance(HOME, GRAIN), 0);
        assert_eq!(
            sim.state.town_market.history[0].markets[&GRAIN_MARKET].volume,
            0
        );
    }
}
#[test]
fn household_market_replay_checks_orders_effects_and_pooling_before_publication() {
    let (w, s) = fixture();
    let mut sim = Simulation::new(w.clone(), s, Backend::Reference).unwrap();
    sim.step().unwrap();
    let opening = sim.state.clone();
    sim.step().unwrap();
    let good = sim.ledger.last().unwrap();
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let mut valid = opening.clone();
        commit(&w, &mut valid, good, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        assert_eq!(valid, sim.state);
        for mode in 0..4 {
            let mut forged = good.clone();
            match mode {
                0 => forged.transactions[0].effects[0].delta += 1,
                1 => {
                    if let Some(town_market::Boundary::Market(r)) = &mut forged.town_market {
                        r.orders[0].quote += 1;
                    }
                }
                2 => forged.household.as_mut().unwrap().before.push(Effect {
                    account: (HOME, TOKEN),
                    delta: 1,
                }),
                _ => {
                    if let Some(town_market::Boundary::Market(r)) = &mut forged.town_market {
                        r.order_receipts
                            .iter_mut()
                            .find(|r| r.agent == HOME)
                            .unwrap()
                            .household
                            .as_mut()
                            .unwrap()
                            .leader = None;
                    }
                }
            }
            let mut unchanged = opening.clone();
            assert!(commit(&w, &mut unchanged, &forged, backend, DEFAULT_EFFECT_LIMIT).is_err());
            assert_eq!(unchanged, opening);
        }
        let mut unchanged = opening.clone();
        assert!(commit(&w, &mut unchanged, good, backend, 0).is_err());
        assert_eq!(unchanged, opening);
    }
}

#[test]
fn observer_reports_collective_policy_without_changing_results() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let (w, s) = fixture();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut plain = sim.clone();
    let mut observer = Observer::new(
        vec![],
        "household",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 1).unwrap();
    plain.run_months(1).unwrap();
    assert_eq!(sim.state, plain.state);
    assert_eq!(sim.ledger, plain.ledger);
    let lines = String::from_utf8(observer.finish().unwrap()).unwrap();
    let rows: Vec<serde_json::Value> = lines
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let receipt = rows
        .iter()
        .find(|r| r["kind"] == "order_generation" && r["agent"] == HOME)
        .unwrap();
    assert_eq!(receipt["household"]["governor"], PERSON);
    assert_eq!(receipt["household"]["policy"], "NeedsFirst");
}
#[test]
fn member_and_household_cannot_duplicate_orders_for_the_same_needs() {
    let (mut w, s) = fixture();
    let mut entry = w
        .town_market
        .as_ref()
        .unwrap()
        .traders
        .iter()
        .find(|e| e.side == Side::Buy)
        .unwrap()
        .clone();
    entry.trader.agent = PERSON;
    w.town_market.as_mut().unwrap().traders.push(entry);
    assert!(
        Simulation::new(w, s, Backend::Reference)
            .err()
            .unwrap()
            .contains("collective town account")
    );
}

#[test]
fn collective_sells_only_stock_above_member_consumption_reserves() {
    let (mut w, mut s) = fixture();
    w.storage.capacities.clear();
    s.balances.insert((HOME, GRAIN), 6);
    s.balances.insert((89, GRAIN), 0);
    s.balances.insert((92, GRAIN), 2);
    s.balances.insert((89, TOKEN), 100);
    for e in &mut w.town_market.as_mut().unwrap().traders {
        if e.trader.agent == HOME {
            e.side = Side::Sell;
        }
        if e.trader.agent == 89 {
            e.side = Side::Buy;
            e.trader.limit = 60;
            e.trader.opening_quote = 60;
        }
    }
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    a.step(&mut sim).unwrap();
    a.step(&mut sim).unwrap();
    let order = sim.state.town_market.history[0]
        .orders
        .iter()
        .find(|o| o.agent == HOME)
        .unwrap();
    assert_eq!(order.protected[&(HOME, GRAIN)], 2); // Two current rations already reserved to members.
    assert_eq!(sim.state.balance(HOME, GRAIN), 2);
    assert_eq!(sim.state.balance(HOME, TOKEN), 155);
    through(&mut a, &mut sim, 2);
    let receipt = sim.state.town_market.history[1]
        .order_receipts
        .iter()
        .find(|r| r.agent == HOME)
        .unwrap();
    // The remaining two rations moved to members before the second market opens.
    assert_eq!(receipt.reason, OrderReason::InsufficientOpeningStock);
    assert_eq!(
        sim.reports
            .iter()
            .filter(|r| [PERSON, 91].contains(&r.agent))
            .map(|r| r.deficit(NUTRITION))
            .sum::<i32>(),
        0
    );
}

fn two_goods() -> (World, State) {
    use economics_compute_smoke::scenario::{FUEL, USE_FUEL, WARMTH};
    let (mut w, mut s) = fixture();
    let (catalog, _) = economics_compute_smoke::scenario::with_warmth(false);
    w.resources.extend(
        catalog
            .resources
            .into_iter()
            .filter(|r| [FUEL, WARMTH].contains(&r.id)),
    );
    w.definitions
        .extend(catalog.definitions.into_iter().filter(|d| d.id == USE_FUEL));
    w.transaction_policy.as_mut().unwrap().permissions.insert((
        economics_compute_smoke::opportunities::PERSON_TYPE,
        economics_compute_smoke::opportunities::Action::Process(USE_FUEL),
    ));
    for p in &mut w.participants {
        p.needs.push(Requirement {
            resource: WARMTH,
            quantity: 1,
            priority: 1,
        });
    }
    w.marketplaces[0]
        .markets
        .push(economics_compute_smoke::marketplace::Market {
            id: 2,
            goods: Amount::new(FUEL, 2),
            payment: TOKEN,
            price_tick: 1,
        });
    let c = w.town_market.as_mut().unwrap();
    c.additional.push(town_market::Listing {
        market: 2,
        traders: c.traders.clone(),
        match_limit: None,
    });
    for id in [89, 92] {
        s.balances.insert((id, FUEL), 10);
    }
    w.storage.capacities.clear();
    w.storage.weights.insert(FUEL, 1);
    (w, s)
}
#[test]
fn collective_books_share_money_and_member_contributed_storage() {
    for storage_bound in [false, true] {
        let (mut w, mut s) = two_goods();
        if storage_bound {
            for id in [PERSON, 91] {
                w.storage.capacities.insert(id, 2);
            }
        } else {
            s.balances.insert((HOME, TOKEN), 60);
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        sim.step().unwrap();
        sim.step().unwrap();
        let r = &sim.state.town_market.history[0];
        assert_eq!(r.markets[&GRAIN_MARKET].volume, 2);
        assert_eq!(r.markets[&2].volume, 0);
        assert_eq!(
            r.orders
                .iter()
                .filter(|o| o.agent == HOME && o.side == Side::Buy)
                .count(),
            2
        );
        assert_eq!(
            sim.state.balance(HOME, TOKEN),
            if storage_bound { 60 } else { 20 }
        );
    }
}

#[test]
fn vacant_governance_and_permission_revocation_prevent_collective_orders() {
    for revoke in [false, true] {
        let (mut w, s) = fixture();
        if !revoke {
            w.households[0].governance = household_governance::Governance::elected(PERSON, 1);
            w.households[0].governance.charter.initial_policy = Policy::NeedsFirst;
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        if !revoke {
            sim.run_months(1).unwrap();
        }
        sim.step().unwrap();
        assert_eq!(
            sim.state
                .town_market
                .admission
                .as_ref()
                .unwrap()
                .eligible
                .contains(&HOME),
            revoke
        );
        if revoke {
            sim.world
                .transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(
                    economics_compute_smoke::opportunities::HOUSEHOLD_TYPE,
                    economics_compute_smoke::opportunities::Action::StockTrade,
                ));
        }
        sim.step().unwrap();
        assert!(
            sim.state
                .town_market
                .history
                .last()
                .unwrap()
                .orders
                .iter()
                .all(|o| o.agent != HOME)
        );
    }
}

#[test]
fn trading_and_contributed_work_share_the_same_real_members_and_outputs() {
    use economics_compute_smoke::{
        activities::{Target, WorkOrder},
        opportunities::{Action, PERSON_TYPE},
        scenario::{LABOR, SEED},
    };
    let (mut w, mut s) = fixture();
    let base = economics_compute_smoke::scenario::baseline().0;
    w.resources
        .push(base.resources.into_iter().find(|r| r.id == SEED).unwrap());
    let process = 99;
    w.definitions.push(ProcessDefinition {
        id: process,
        name: "collective food work".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 1,
            entry_inputs: vec![Amount::new(SEED, 1)],
            monthly_services: vec![Amount::new(LABOR, 6)],
        }],
        outputs: vec![Amount::new(GRAIN, 2)],
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::Process(process)));
    for p in &mut w.participants {
        if [PERSON, 91].contains(&p.agent) {
            p.capacity.quantity = 5;
        }
    }
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: process,
        priority: 0,
        target: Target::Stock(Amount::new(GRAIN, 20)),
    });
    s.balances.insert((PERSON, SEED), 1);
    let run = |backend| {
        let mut a = Audit::with_opening(
            &w,
            &s,
            TOKEN,
            Opening {
                inventory: s
                    .balances
                    .iter()
                    .filter(|((_, r), q)| (*r == GRAIN || *r == SEED) && **q > 0)
                    .map(|(key, q)| (*key, i128::from(*q)))
                    .collect(),
                processes: Some(Costs::default()),
                ..Default::default()
            },
        )
        .unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        through(&mut a, &mut sim, 1);
        assert_eq!(
            sim.state.town_market.history[0].markets[&GRAIN_MARKET].volume,
            2
        );
        assert!(
            sim.state
                .processes
                .values()
                .any(|p| p.definition == process && p.status == Status::Completed)
        );
        let labor: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .collect();
        assert!(labor.iter().any(|r| r.granted > 0));
        assert_eq!(
            sim.state.balance(HOME, GRAIN)
                + sim.state.balance(PERSON, GRAIN)
                + sim.state.balance(91, GRAIN),
            2
        );
        assert_eq!(
            sim.reports
                .iter()
                .filter(|r| [PERSON, 91].contains(&r.agent))
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>(),
            0
        );
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
