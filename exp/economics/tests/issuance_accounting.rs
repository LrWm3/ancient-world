use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    compute::Backend,
    financial_reporting::Audit,
    issuance_accounting::Policy,
    minting::{self, *},
    model::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;

fn audit(w: &World, s: &State) -> Audit {
    let costs = s
        .balances
        .iter()
        .filter(|((_, r), q)| {
            *r != COIN
                && **q > 0
                && w.resources
                    .iter()
                    .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
        })
        .map(|(k, q)| (*k, i128::from(*q)))
        .collect();
    Audit::with_processes(w, s, COIN, BTreeMap::new(), costs, BTreeMap::new()).unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
#[test]
fn physical_coin_creation_is_equity_not_sales_and_real_costs_remain_visible() {
    let (w, s) = minting::scenario("normal").unwrap();
    let mut a = audit(&w, &s)
        .with_issuance_policy(Policy::NonRedeemableEquity)
        .unwrap();
    let mut b = a.clone();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 1);
    through(&mut b, &mut cpu, 1);
    assert_eq!(a, b);
    let mut resumed = a.clone();
    let mut checkpoint = sim.clone();
    through(&mut a, &mut sim, 2);
    through(&mut b, &mut cpu, 2);
    through(&mut resumed, &mut checkpoint, 2);
    assert_eq!(a, b);
    assert_eq!(a, resumed);
    assert_eq!(sim.state, cpu.state);
    let issuer = a.book().statements(ISSUER, 1, 2).unwrap();
    assert_eq!(issuer.income[&A::Sales], 6);
    assert_eq!(issuer.expenses[&A::CostOfSales], 6);
    assert_eq!(issuer.expenses[&A::ServiceExpense], 4);
    assert_eq!(issuer.expenses[&A::ProductionExpense], 2);
    assert_eq!(issuer.net_income, -6);
    assert_eq!(issuer.issuance_change, 10);
    assert_eq!(issuer.capital_change, 0);
    assert_eq!(issuer.cash_flows[&Flow::Operating], 0);
    assert_eq!(issuer.cash_flows[&Flow::Issuance], 10);
    assert_eq!(issuer.closing_cash, 10);
    assert_eq!(issuer.equity, 10);
    let worker = a.book().statements(WORKER, 1, 2).unwrap();
    assert_eq!(worker.income[&A::ServiceIncome], 4);
    assert_eq!(worker.net_income, 4);
    assert!(
        issuer
            .markdown(ISSUER, COIN)
            .contains("| Monetary issuance | 10 |")
    );
    through(&mut a, &mut sim, 4);
    let later = a.book().statements(ISSUER, 3, 4).unwrap();
    assert_eq!(later.issuance_change, 0);
    assert_eq!(later.opening_equity, 10);
    assert_eq!(later.equity, 10);
}
#[test]
fn failed_packages_neither_issue_money_nor_record_unpaid_labor() {
    for case in ["treasury", "metal", "labor"] {
        let (w, s) = minting::scenario(case).unwrap();
        let mut a = audit(&w, &s)
            .with_issuance_policy(Policy::NonRedeemableEquity)
            .unwrap();
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        through(&mut a, &mut sim, 2);
        let r = a.book().statements(ISSUER, 1, 2).unwrap();
        assert_eq!(r.issuance_change, 0, "{case}");
        assert!(!r.expenses.contains_key(&A::ServiceExpense));
        assert!(!r.expenses.contains_key(&A::ProductionExpense));
    }
}
#[test]
fn paid_but_unused_period_services_are_expensed_and_do_not_roll_into_next_month() {
    let (mut w, s) = minting::scenario("normal").unwrap();
    w.scheduled_starts.retain(|v| v.definition != MINT);
    let mut a = audit(&w, &s)
        .with_issuance_policy(Policy::NonRedeemableEquity)
        .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 3);
    let r = a.book().statements(ISSUER, 1, 3).unwrap();
    assert_eq!(r.issuance_change, 0);
    assert_eq!(r.expenses[&A::ServiceExpense], 4);
    assert_eq!(r.trial_balance[&A::Inventory(METAL)], 2);
    assert!(!r.expenses.contains_key(&A::ProductionExpense));
    assert_eq!(sim.state.balance(ISSUER, HOURS), 0);
}
#[test]
fn issuer_convention_is_required_and_forged_output_cannot_publish() {
    let (w, s) = minting::scenario("normal").unwrap();
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    a.step(&mut sim).unwrap();
    let old = a.clone();
    let state = sim.state.clone();
    assert!(a.step(&mut sim).is_err());
    assert_eq!(a, old);
    assert_eq!(sim.state, state);
    assert!(a.with_issuance_policy(Policy::NonRedeemableEquity).is_err());
    let (w, s) = minting::scenario("normal").unwrap();
    let mut a = audit(&w, &s)
        .with_issuance_policy(Policy::NonRedeemableEquity)
        .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.month < 2 || sim.state.phase != Phase::Productive {
        a.step(&mut sim).unwrap();
    }
    let mut preview = sim.clone();
    preview.step().unwrap();
    let mut bad = preview.ledger.last().unwrap().clone();
    let t = bad
        .transactions
        .iter_mut()
        .find(|t| {
            t.process
                .as_ref()
                .is_some_and(|p| p.after.definition == MINT)
        })
        .unwrap();
    t.effects
        .iter_mut()
        .find(|e| e.account == (ISSUER, COIN))
        .unwrap()
        .delta += 1;
    let old = a.clone();
    assert!(
        a.record(&sim.world, &sim.state, &bad, &preview.state)
            .is_err()
    );
    assert_eq!(a, old);
    a.step(&mut sim).unwrap();
}
#[test]
fn repeated_input_production_and_low_yield_keep_issuance_separate_from_profit() {
    for case in ["normal", "ore", "labor", "low_yield"] {
        let (w, s) = minting::repeated_scenario(case).unwrap();
        let mut a = audit(&w, &s)
            .with_issuance_policy(Policy::NonRedeemableEquity)
            .unwrap();
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        through(&mut a, &mut sim, 6);
        let expected: i128 = sim
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter(|t| {
                t.process
                    .as_ref()
                    .is_some_and(|p| p.after.definition == MINT)
            })
            .flat_map(|t| &t.effects)
            .filter(|e| e.account == (ISSUER, COIN))
            .map(|e| i128::from(e.delta))
            .sum();
        assert!(expected > 0);
        let r = a.book().statements(ISSUER, 1, 6).unwrap();
        assert_eq!(r.issuance_change, expected);
        assert_eq!(r.income.get(&A::MonetaryIssuance), None);
        assert_eq!(
            r.opening_equity + r.net_income + r.issuance_change,
            r.equity
        );
    }
}

#[test]
fn provisioning_and_leisure_variants_reconcile_expiring_stock_costs() {
    for case in ["adequate", "scarce", "empty", "endowed"] {
        let (w, s) = minting::provision_scenario(case).unwrap();
        let mut a = audit(&w, &s)
            .with_issuance_policy(Policy::NonRedeemableEquity)
            .unwrap();
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut sim, 12);
    }
}
