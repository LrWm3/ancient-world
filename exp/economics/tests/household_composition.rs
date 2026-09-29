use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::{Audit, Opening},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME, support::Mandate},
    model::*,
    scenario::{GRAIN, NUTRITION, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const STORED: ResourceId = 100;
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            exchange_values: w
                .resources
                .iter()
                .filter(|r| r.id == STORED)
                .map(|r| (r.id, 3))
                .collect(),
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| {
                    *r != TOKEN
                        && **q > 0
                        && w.resources
                            .iter()
                            .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
                })
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            processes: Some(economics_compute_smoke::process_accounting::Costs {
                work: s
                    .processes
                    .values()
                    .filter(|p| p.status == Status::Active)
                    .map(|p| (p.id, (p.beneficiary, 0)))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        },
    )
    .unwrap()
}
fn stock(w: &mut World, id: ResourceId) {
    w.resources.push(Resource {
        id,
        name: format!("stored {id}"),
        kind: ResourceKind::Stock,
    });
    w.storage.weights.insert(id, 1);
}
fn support_fixture(blocked: bool) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    w.activities.orders.clear();
    w.participants
        .iter_mut()
        .find(|p| p.agent == 91)
        .unwrap()
        .needs[0]
        .quantity = 2;
    stock(&mut w, STORED);
    w.storage.capacities.insert(PERSON, 10);
    w.storage.capacities.insert(91, 2);
    s.balances
        .insert((PERSON, GRAIN), if blocked { 5 } else { 6 });
    s.balances
        .insert((HOME, STORED), if blocked { 6 } else { 5 });
    w.households[0].support.push(Mandate {
        member: PERSON,
        resource: GRAIN,
        from: 1,
        through: 12,
        revoked_from: None,
        reserve_months: 1,
        private_reserve: 2,
        household_target: 4,
        monthly_limit: 4,
    });
    (w, s)
}
#[test]
fn partial_support_uses_shared_storage_and_retains_private_needs() {
    for blocked in [false, true] {
        let (w, s) = support_fixture(blocked);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            while sim.state.month == 1 {
                a.step(&mut sim).unwrap();
            }
            let r = sim
                .ledger
                .iter()
                .filter_map(|b| b.household.as_ref())
                .flat_map(|b| &b.support)
                .next()
                .unwrap();
            assert_eq!(r.accepted, if blocked { 0 } else { 1 });
            assert!(r.offered > r.accepted);
            let food = sim
                .reports
                .iter()
                .find(|r| r.agent == 91)
                .unwrap()
                .deficit(NUTRITION);
            assert_eq!(food, if blocked { 2 } else { 1 });
            let mut replay = s.clone();
            for b in &sim.ledger {
                commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(replay, sim.state);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

fn barter_fixture(room: i32, carry: i32) -> (World, State) {
    use economics_compute_smoke::marketplace::Side;
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.activities.orders.clear();
    stock(&mut w, STORED);
    stock(&mut w, STORED + 1);
    for cap in w.storage.capacities.values_mut() {
        *cap = 100;
    }
    w.storage.capacities.insert(91, 2);
    for v in &mut w.marketplaces {
        for m in &mut v.markets {
            m.payment = STORED;
        }
    }
    let c = w.town_market.as_mut().unwrap();
    let mut seller = c
        .traders
        .iter()
        .find(|e| e.side == Side::Sell)
        .unwrap()
        .clone();
    seller.trader.agent = PERSON;
    c.traders.push(seller);
    for e in &mut c.traders {
        e.trader.limit = 3;
        e.trader.opening_quote = 3;
        if e.trader.agent == 89 {
            e.side = Side::Buy;
        }
    }
    let needs = w
        .participants
        .iter()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .needs
        .clone();
    w.participants
        .iter_mut()
        .find(|p| p.agent == 89)
        .unwrap()
        .needs = needs;
    s.balances.insert((PERSON, GRAIN), 10);
    s.balances.insert((89, GRAIN), 0);
    s.balances.insert((92, GRAIN), 0);
    s.balances.insert((89, STORED), 6);
    s.balances.insert((HOME, TOKEN), 0);
    s.balances.insert((HOME, STORED + 1), 51 - room);
    s.household_remainders.insert((HOME, PERSON, STORED), carry);
    (w, s)
}
#[test]
fn barter_reserves_collective_storage_and_fractional_carry_before_matching() {
    for (room, carry, filled) in [(0, 0, false), (1, 0, true), (1, 1, false), (2, 1, true)] {
        let (w, s) = barter_fixture(room, carry);
        let run = |backend| {
            let mut a = audit(&w, &s);
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.phase != Phase::Acquire {
                a.step(&mut sim).unwrap();
            }
            a.step(&mut sim).unwrap();
            assert_eq!(sim.state.balance(89, GRAIN), if filled { 2 } else { 0 });
            assert_eq!(
                sim.state.balance(HOME, STORED),
                if filled { (3 + carry) / 2 } else { 0 }
            );
            assert_eq!(
                sim.state.household_remainders[&(HOME, PERSON, STORED)],
                if filled { (3 + carry) % 2 } else { carry }
            );
            assert_eq!(
                sim.state.balance(89, STORED)
                    + sim.state.balance(PERSON, STORED)
                    + sim.state.balance(HOME, STORED),
                6
            );
            if !filled {
                assert!(
                    format!("{:?}", sim.state.town_market.history[0].attempts)
                        .contains("InsufficientStorage")
                );
            }
            let mut replay = s.clone();
            for b in &sim.ledger {
                commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(replay, sim.state);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

fn delegated_fixture() -> (World, State) {
    use economics_compute_smoke::{household_governance::Purchasing, marketplace::Side};
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.households[0].governance.charter.purchasing = Purchasing::Members;
    let c = w.town_market.as_mut().unwrap();
    let mut buyer = c
        .traders
        .iter()
        .find(|e| e.side == Side::Buy)
        .unwrap()
        .clone();
    buyer.trader.agent = PERSON;
    c.traders.push(buyer);
    s.balances.insert((PERSON, TOKEN), 100);
    (w, s)
}
#[test]
fn charter_delegates_buys_while_preserving_pooling_and_separate_books() {
    use economics_compute_smoke::town_market::OrderReason;
    let (w, s) = delegated_fixture();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        while sim.state.phase != Phase::Acquire {
            a.step(&mut sim).unwrap();
        }
        a.step(&mut sim).unwrap();
        let round = &sim.state.town_market.history[0];
        assert!(!round.orders.iter().any(|o| o.agent == HOME));
        assert!(round.orders.iter().any(|o| o.agent == PERSON));
        assert_eq!(
            round
                .order_receipts
                .iter()
                .find(|r| r.agent == HOME)
                .unwrap()
                .reason,
            OrderReason::PurchasePolicy
        );
        assert_eq!(sim.state.balance(HOME, TOKEN), 100);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 60);
        assert_eq!(sim.state.balance(HOME, GRAIN), 1);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 1);
        let receipt = round
            .order_receipts
            .iter()
            .find(|r| r.agent == PERSON)
            .unwrap();
        assert_eq!(receipt.deficits_after.as_ref().unwrap()[&NUTRITION], 0);
        while sim.state.month == 1 {
            a.step(&mut sim).unwrap();
        }
        for person in [PERSON, 91] {
            assert_eq!(
                sim.reports
                    .iter()
                    .find(|r| r.agent == person)
                    .unwrap()
                    .deficit(NUTRITION),
                0
            );
        }
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn current_membership_routes_registered_bids_through_exit_and_accession() {
    use economics_compute_smoke::{
        household_governance::Purchasing, marketplace::Side, town_market::OrderReason,
    };
    let (mut w, mut s) = delegated_fixture();
    w.households[0].governance.charter.purchasing = Purchasing::Collective;
    w.town_market
        .as_mut()
        .unwrap()
        .traders
        .iter_mut()
        .find(|e| e.trader.agent == PERSON)
        .unwrap()
        .trader
        .agent = 91;
    s.balances.insert((91, TOKEN), 100);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        for month in 1..=3 {
            if month == 2 {
                households::membership::leave(&mut sim.world, &sim.state, HOME, 91).unwrap();
            }
            if month == 3 {
                households::membership::join(
                    &mut sim.world,
                    &sim.state,
                    HOME,
                    91,
                    vec![PERSON, 91],
                )
                .unwrap();
            }
            let (mut resumed, mut saved) = (sim.clone(), a.clone());
            while sim.state.month == month {
                a.step(&mut sim).unwrap();
                saved.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &a),
                (&resumed.state, &resumed.ledger, &saved)
            );
            let round = sim.state.town_market.history.last().unwrap();
            let receipt = round
                .order_receipts
                .iter()
                .find(|r| r.agent == 91 && r.side == Side::Buy)
                .unwrap();
            assert_eq!(
                receipt.reason,
                if month == 2 {
                    OrderReason::Submitted
                } else {
                    OrderReason::PurchasePolicy
                }
            );
            if month == 2 {
                assert_eq!(sim.state.balance(91, GRAIN), 1);
            } // no pooling while outside
        }
        assert_eq!(sim.state.balance(91, TOKEN), 60);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

fn input_fixture(fund: bool, active: bool, held: i32) -> (World, State) {
    use economics_compute_smoke::{
        activities::{Target, WorkOrder},
        opportunities::{Action, PERSON_TYPE},
        scenario::{LABOR, SEED},
    };
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.households[0].governance.charter.fund_committed_inputs = fund;
    stock(&mut w, SEED);
    for cap in w.storage.capacities.values_mut() {
        *cap = 30;
    }
    for id in [PERSON, 91] {
        s.balances.insert((id, GRAIN), 4);
    }
    s.balances.insert((PERSON, SEED), held);
    s.balances.insert((89, SEED), 10);
    for venue in &mut w.marketplaces {
        for market in &mut venue.markets {
            market.goods = Amount::new(SEED, 2);
        }
    }
    w.participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .capacity
        .quantity = 5;
    w.definitions.push(ProcessDefinition {
        id: 99,
        name: "accepted seed work".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 2,
            entry_inputs: vec![Amount::new(SEED, 2)],
            monthly_services: vec![Amount::new(LABOR, 1)],
        }],
        outputs: vec![Amount::new(GRAIN, 4)],
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::Process(99)));
    if active {
        s.processes.insert(
            900,
            ProcessInstance {
                id: 900,
                definition: 99,
                operator: PERSON,
                beneficiary: PERSON,
                goal: None,
                asset: None,
                right: None,
                start: 1,
                reserved_through: 2,
                stage: 0,
                elapsed: 0,
                status: Status::Active,
            },
        );
    } else {
        w.activities.orders.push(WorkOrder {
            agent: PERSON,
            definition: 99,
            priority: 0,
            target: Target::Stock(Amount::new(GRAIN, 20)),
        });
    }
    (w, s)
}
#[test]
fn collective_market_inputs_complete_member_work_and_reconcile_costs() {
    use economics_compute_smoke::{accounting::Account as A, scenario::SEED};
    let (w, s) = input_fixture(true, true, 0);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        while sim.state.phase != Phase::Acquire {
            a.step(&mut sim).unwrap();
        }
        a.step(&mut sim).unwrap();
        let receipt = sim.state.town_market.history[0]
            .order_receipts
            .iter()
            .find(|r| r.agent == HOME)
            .unwrap();
        assert_eq!(receipt.deficits_before.as_ref().unwrap()[&SEED], 2);
        assert_eq!(receipt.deficits_after.as_ref().unwrap()[&SEED], 0);
        assert_eq!(sim.state.balance(HOME, SEED), 2);
        while sim.state.month == 1 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.balance(HOME, SEED), 0);
        assert_eq!(sim.state.processes[&900].status, Status::Active);
        let (mut resumed, mut saved) = (sim.clone(), a.clone());
        while sim.state.month == 2 {
            a.step(&mut sim).unwrap();
            saved.step(&mut resumed).unwrap();
        }
        assert_eq!((&sim.state, &a), (&resumed.state, &saved));
        assert_eq!(sim.state.processes[&900].status, Status::Completed);
        assert_eq!(sim.state.balance(HOME, TOKEN), 60);
        assert_eq!(sim.state.balance(HOME, GRAIN), 2);
        let f = a.book().statements(HOME, 1, 2).unwrap();
        // Existing three grain (basis 3) and four output (basis 40)
        // share the average-cost pool: two transferred units carry floor(86/7).
        assert_eq!(f.trial_balance[&A::Inventory(GRAIN)], 12);
        for id in [HOME, PERSON, 91, 89] {
            let f = a.book().statements(id, 1, 2).unwrap();
            assert_eq!(f.assets, f.liabilities + f.equity);
        }
        let mut replay = s.clone();
        for b in &sim.ledger {
            commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        }
        assert_eq!(replay, sim.state);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn input_funding_requires_charter_authority_accepted_work_and_a_real_shortfall() {
    for (fund, active, held) in [(false, true, 0), (true, false, 0), (true, true, 2)] {
        let (w, s) = input_fixture(fund, active, held);
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
        let mut a = audit(&w, &s);
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        assert!(
            sim.state
                .town_market
                .history
                .iter()
                .all(|r| r.orders.iter().all(|o| o.agent != HOME))
        );
        assert_eq!(sim.state.balance(HOME, TOKEN), 100);
        if active {
            assert_eq!(
                sim.state.processes[&900].status,
                if held > 0 {
                    Status::Completed
                } else {
                    Status::Aborted
                }
            );
        }
    }
}

#[test]
fn two_barter_listings_share_one_pooling_budget_and_accumulate_odd_proceeds() {
    use economics_compute_smoke::{
        opportunities::{Action, PERSON_TYPE},
        scenario::SEED,
        town_market::Listing,
    };
    for (room, trades) in [(2, 1), (3, 2)] {
        let (mut w, mut s) = barter_fixture(room, 0);
        stock(&mut w, SEED);
        let mut recipe = w
            .definitions
            .iter()
            .find(|d| d.execution == Execution::Consumption)
            .unwrap()
            .clone();
        recipe.id = 300;
        recipe.stages[0].entry_inputs = vec![Amount::new(SEED, 1)];
        w.definitions.push(recipe);
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((PERSON_TYPE, Action::Process(300)));
        w.participants
            .iter_mut()
            .find(|p| p.agent == 89)
            .unwrap()
            .needs[0]
            .quantity = 3;
        let config = w.town_market.as_mut().unwrap();
        config.additional.push(Listing {
            market: 200,
            traders: config.traders.clone(),
            match_limit: None,
        });
        let venue = w
            .marketplaces
            .iter_mut()
            .find(|m| m.agent == config.venue)
            .unwrap();
        let mut market = venue.markets[0].clone();
        market.id = 200;
        market.goods = Amount::new(SEED, 2);
        venue.markets.push(market);
        s.balances.insert((PERSON, SEED), 10);
        let run = |backend| {
            let mut a = audit(&w, &s);
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.phase != Phase::Acquire {
                a.step(&mut sim).unwrap();
            }
            a.step(&mut sim).unwrap();
            assert_eq!(sim.state.town_market.history[0].transactions.len(), trades);
            assert_eq!(
                sim.state.balance(HOME, STORED),
                if trades == 1 { 1 } else { 3 }
            );
            assert_eq!(
                sim.state.household_remainders[&(HOME, PERSON, STORED)],
                if trades == 1 { 1 } else { 0 }
            );
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
