use economics_compute_smoke::{
    accounting::Account as A,
    compute::Backend,
    crafts::*,
    equipment::DurableAsset,
    financial_reporting::{Audit, Opening},
    model::*,
    process_accounting::{Costs, Output},
    scenario::*,
    simulation::Simulation,
    trading_scenario::{self, STOCK_UNIT},
};
use std::collections::BTreeMap;
fn work_fixture(rate: u32) -> Simulation {
    let (mut w, mut s) = trading_scenario::scenario(1, rate).unwrap();
    w.activities.orders.clear();
    w.condition_rules.clear();
    for p in &mut w.participants {
        for n in &mut p.needs {
            n.quantity = 0;
        }
    }
    let buyer = PERSON + 3;
    let provider = PERSON + 2;
    s.equipment.insert(
        9000,
        DurableAsset {
            id: 9000,
            owner: provider,
            attached_to: None,
            kind: COMB,
            remaining_uses: 24,
            last_used_month: None,
        },
    );
    s.equipment.insert(
        9001,
        DurableAsset {
            id: 9001,
            owner: buyer,
            attached_to: Some(PLOT + 3),
            kind: HERD,
            remaining_uses: 24,
            last_used_month: None,
        },
    );
    Simulation::new(w, s, Backend::Reference).unwrap()
}

fn opening(sim: &Simulation, enabled: bool, price: i128) -> Opening {
    let w = &sim.world;
    let s = &sim.state;
    Opening {
        assets: w
            .assets
            .iter()
            .map(|a| (a.id, 0))
            .chain(s.equipment.keys().map(|id| (*id, 24)))
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
            .map(|(k, q)| (*k, i128::from(*q)))
            .collect(),
        dues: Some(economics_compute_smoke::dues_accounting::Valuation(
            w.agreements.iter().map(|a| (a.id, 1)).collect(),
        )),
        processes: Some(Costs {
            earned_royalty_values: enabled.then(|| BTreeMap::from([(MILK, price), (WOOL, price)])),
            output_weights: BTreeMap::from([(
                HUSBANDRY,
                BTreeMap::from([(Output::Stock(MILK), 1), (Output::Stock(WOOL), 1)]),
            )]),
            ..Default::default()
        }),
        ..Default::default()
    }
}
fn through(a: &mut Audit, s: &mut Simulation, month: u32) {
    while s.state.month <= month {
        a.step(s)
            .unwrap_or_else(|e| panic!("{e}: {} {:?}", s.state.month, s.state.phase));
    }
}
#[test]
fn earned_shares_delivery_idle_and_continuation_on_cpu() {
    for rate in [0, 25, 100] {
        let mut reference = work_fixture(rate);
        reference.world.scheduled_starts.push(ScheduledStart {
            month: 2,
            agent: PERSON + 3,
            definition: HUSBANDRY,
        });
        let mut cpu = reference.clone();
        cpu.backend = Backend::CubeCpu;
        let mut a = Audit::with_opening(
            &reference.world,
            &reference.state,
            TOKEN,
            opening(&reference, true, 3),
        )
        .unwrap();
        let mut b = a.clone();
        through(&mut a, &mut reference, 1);
        through(&mut b, &mut cpu, 1);
        assert_eq!(a, b);
        let provider = PERSON + 2;
        let buyer = PERSON + 3;
        assert_eq!(
            a.book()
                .balances()
                .get(&(buyer, A::Tangible(9000)))
                .copied()
                .unwrap_or(0),
            0
        );
        let report = a.book().statements(provider, 1, 1).unwrap();
        assert_eq!(report.expenses[&A::CostOfSales], 24);
        assert!(!report.income.contains_key(&A::ServiceIncome));
        let mut checkpoint = cpu.clone();
        let mut resumed = b.clone();
        through(&mut a, &mut reference, 3);
        through(&mut b, &mut cpu, 3);
        through(&mut resumed, &mut checkpoint, 3);
        assert_eq!(a, b);
        assert_eq!(b, resumed);
        assert_eq!(cpu.state, reference.state);
        let buyer_report = a.book().statements(buyer, 1, 3).unwrap();
        // 100 hay + one tick of herd wear: joint costs 50 milk, 51 wool.
        let royalty_basis = 50 * i128::from(rate) / 100 + 51 * i128::from(rate) / 100;
        assert_eq!(
            buyer_report
                .expenses
                .get(&A::CostOfSales)
                .copied()
                .unwrap_or(0),
            royalty_basis
        );
        let report = a.book().statements(provider, 1, 3).unwrap();
        let earned = i128::from(3 * STOCK_UNIT) * i128::from(rate) / 100 * 3;
        assert_eq!(
            report.income.get(&A::ServiceIncome).copied().unwrap_or(0),
            earned
        );
        assert_eq!(
            buyer_report
                .expenses
                .get(&A::ServiceExpense)
                .copied()
                .unwrap_or(0),
            earned
        );
        assert_eq!(
            buyer_report.income.get(&A::Sales).copied().unwrap_or(0),
            earned
        );
        assert!(report.cash_flows.values().all(|v| *v == 0));
        for (resource, produced) in [(MILK, 2 * STOCK_UNIT), (WOOL, STOCK_UNIT)] {
            assert_eq!(
                cpu.state.balance(provider, resource),
                produced * rate as i32 / 100
            );
        }
        a.finalize_through(3).unwrap();
    }
}
#[test]
fn missing_policy_and_value_overflow_reject_without_publication() {
    for (enabled, price) in [(false, 3), (true, i128::MAX)] {
        let mut sim = work_fixture(25);
        sim.world.scheduled_starts.push(ScheduledStart {
            month: 2,
            agent: PERSON + 3,
            definition: HUSBANDRY,
        });
        let mut a =
            Audit::with_opening(&sim.world, &sim.state, TOKEN, opening(&sim, enabled, price))
                .unwrap();
        loop {
            assert!(sim.state.month <= 2);
            let old = a.clone();
            let state = sim.state.clone();
            if let Err(e) = a.step(&mut sim) {
                assert!(e.contains("royalty"), "{e}");
                assert_eq!(a, old);
                assert_eq!(sim.state, state);
                break;
            }
        }
    }
}

#[test]
fn exhausted_tool_stops_earning_and_missing_output_price_is_atomic() {
    let mut sim = work_fixture(25);
    sim.state.equipment.get_mut(&9000).unwrap().remaining_uses = 1;
    for month in [2, 3] {
        sim.world.scheduled_starts.push(ScheduledStart {
            month,
            agent: PERSON + 3,
            definition: HUSBANDRY,
        });
    }
    let mut a = Audit::with_opening(&sim.world, &sim.state, TOKEN, opening(&sim, true, 3)).unwrap();
    through(&mut a, &mut sim, 1);
    let mut no_price = opening(&sim, true, 3);
    no_price
        .processes
        .as_mut()
        .unwrap()
        .earned_royalty_values
        .as_mut()
        .unwrap()
        .remove(&WOOL);
    let mut missing = Audit::with_opening(&sim.world, &sim.state, TOKEN, no_price).unwrap();
    let mut rejected = sim.clone();
    loop {
        let prior = missing.clone();
        let state = rejected.state.clone();
        if let Err(e) = missing.step(&mut rejected) {
            assert!(e.contains("royalty"), "{e}");
            assert_eq!(missing, prior);
            assert_eq!(rejected.state, state);
            break;
        }
        assert!(rejected.state.month <= 2);
    }
    through(&mut a, &mut sim, 2);
    let earned = sim.state.exchange.earned.clone();
    assert!(!earned.is_empty());
    assert_eq!(sim.state.equipment[&9000].remaining_uses, 0);
    through(&mut a, &mut sim, 3);
    assert_eq!(sim.state.exchange.earned, earned);
    assert!(
        sim.ledger
            .iter()
            .filter(|b| b.month == 3)
            .flat_map(|b| &b.transactions)
            .any(|t| t.process.as_ref().is_some_and(
                |p| p.after.definition == HUSBANDRY && p.after.status == Status::Completed
            ))
    );
}

#[test]
fn failed_work_books_sunk_cost_without_royalty_income() {
    let mut sim = work_fixture(25);
    sim.world
        .definitions
        .iter_mut()
        .find(|d| d.id == HUSBANDRY)
        .unwrap()
        .stages[0]
        .months = 2;
    sim.world.scheduled_starts.push(ScheduledStart {
        month: 2,
        agent: PERSON + 3,
        definition: HUSBANDRY,
    });
    let mut a = Audit::with_opening(&sim.world, &sim.state, TOKEN, opening(&sim, true, 3)).unwrap();
    through(&mut a, &mut sim, 2);
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.definition == HUSBANDRY && p.status == Status::Active)
    );
    sim.world
        .participants
        .iter_mut()
        .find(|p| p.agent == PERSON + 3)
        .unwrap()
        .capacity
        .quantity = 0;
    through(&mut a, &mut sim, 3);
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.definition == HUSBANDRY && p.status == Status::Aborted)
    );
    assert!(sim.state.exchange.earned.is_empty());
    let buyer = a.book().statements(PERSON + 3, 1, 3).unwrap();
    assert_eq!(buyer.expenses[&A::ProductionLoss], 101);
    let provider = a.book().statements(PERSON + 2, 1, 3).unwrap();
    assert_eq!(provider.expenses[&A::CostOfSales], 24);
    assert!(!provider.income.contains_key(&A::ServiceIncome));
}
