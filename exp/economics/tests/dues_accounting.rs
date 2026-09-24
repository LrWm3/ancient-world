use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    activities::CoinPayment,
    commitments::{Agreement, Obligation},
    compute::Backend,
    financial_reporting::Audit,
    model::*,
    scenario::{self, GRAIN, PERSON, PLOT, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
fn fixture(coins: i32, alternate: bool) -> (World, State) {
    let (mut w, mut s) = scenario::baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights[0].through = 36;
    w.resources.push(Resource {
        id: TOKEN,
        name: "coins".into(),
        kind: ResourceKind::Stock,
    });
    w.agreements.push(Agreement {
        id: 1,
        right: 1,
        creditor: STATE_AGENT,
        debtor: PERSON,
        activated: 1,
        payment: Amount::new(GRAIN, 2),
    });
    if alternate {
        w.activities.coin_payments.insert(
            1,
            CoinPayment {
                resource: TOKEN,
                coins_per_unit: 2,
            },
        );
    }
    s.month = 13;
    s.balances = BTreeMap::from([((PERSON, GRAIN), 1), ((PERSON, TOKEN), coins)]);
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    let costs = s
        .balances
        .iter()
        .filter(|((_, r), q)| *r == GRAIN && **q > 0)
        .map(|(k, q)| (*k, i128::from(*q) * 2))
        .collect();
    Audit::with_dues(
        w,
        s,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        costs,
        BTreeMap::from([(1, 3)]),
    )
    .unwrap()
}
fn through(a: &mut Audit, s: &mut Simulation, month: u32) {
    while s.state.month <= month {
        a.step(s).unwrap();
    }
}
#[test]
fn native_and_alternative_tender_separate_dues_cost_disposal_and_settlement_result_on_cpu() {
    let (w, s) = fixture(4, true);
    let mut a = audit(&w, &s);
    let mut b = a.clone();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut reference, 13);
    through(&mut b, &mut cpu, 13);
    assert_eq!(a, b);
    assert_eq!(reference.state, cpu.state);
    let debtor = a.book().statements(PERSON, 13, 13).unwrap();
    assert_eq!(debtor.expenses[&A::DuesExpense], 6);
    assert_eq!(debtor.income[&A::DisposalGain], 1);
    assert_eq!(debtor.income[&A::SettlementGain], 1);
    assert_eq!(
        (debtor.assets, debtor.liabilities, debtor.net_income),
        (2, 0, -4)
    );
    assert_eq!(debtor.cash_flows[&Flow::Operating], -2);
    let creditor = a.book().statements(STATE_AGENT, 13, 13).unwrap();
    assert_eq!(creditor.income[&A::DuesIncome], 6);
    assert_eq!(creditor.expenses[&A::SettlementLoss], 1);
    assert_eq!(creditor.trial_balance[&A::Inventory(GRAIN)], 3);
    assert_eq!(creditor.net_income, 5);
}
#[test]
fn unpaid_claim_is_symmetric_survives_checkpoint_and_is_not_charged_again() {
    let (w, s) = fixture(0, false);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 13);
    assert_eq!(
        a.book().statements(PERSON, 13, 13).unwrap().trial_balance[&A::DuesPayable(1, 13)],
        -3
    );
    assert_eq!(
        a.book()
            .statements(STATE_AGENT, 13, 13)
            .unwrap()
            .trial_balance[&A::DuesReceivable(1, 13)],
        3
    );
    let mut copy = a.clone();
    let mut resumed = sim.clone();
    through(&mut a, &mut sim, 25);
    through(&mut copy, &mut resumed, 25);
    assert_eq!(a, copy);
    assert_eq!(a.book().statements(PERSON, 14, 24).unwrap().net_income, 0);
    assert_eq!(
        a.book().statements(PERSON, 25, 25).unwrap().expenses[&A::DuesExpense],
        6
    );
    assert_eq!(a.book().statements(PERSON, 13, 25).unwrap().liabilities, 9);
}
#[test]
fn existing_arrears_open_as_liability_not_current_expense_and_native_cash_is_one_tick() {
    let (mut w, mut s) = fixture(5, false);
    w.agreements[0].payment = Amount::new(TOKEN, 2);
    s.balances.remove(&(PERSON, GRAIN));
    s.obligations.insert(
        (1, 13),
        Obligation {
            agreement: 1,
            due: 13,
            owed: 2,
            paid: 0,
            in_kind_paid: 0,
        },
    );
    s.month = 14;
    let mut a = Audit::with_dues(
        &w,
        &s,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 14);
    let d = a.book().statements(PERSON, 14, 14).unwrap();
    assert_eq!((d.opening_equity, d.equity, d.net_income), (3, 3, 0));
    assert_eq!(d.cash_flows[&Flow::Operating], -2);
}
#[test]
fn missing_native_valuation_and_unaccounted_issuance_fail_atomically() {
    let (mut w, s) = fixture(0, false);
    let mut a = Audit::with_dues(
        &w,
        &s,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        BTreeMap::from([((PERSON, GRAIN), 2)]),
        BTreeMap::new(),
    )
    .unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    a.step(&mut sim).unwrap();
    let old = a.clone();
    let state = sim.state.clone();
    assert!(a.step(&mut sim).is_err());
    assert_eq!(a, old);
    assert_eq!(sim.state, state);
    w.issuance
        .push(economics_compute_smoke::currency::Issuance {
            agreement: 1,
            token: TOKEN,
            collected_per_token: 1,
        });
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    a.step(&mut sim).unwrap();
    let old = a.clone();
    let state = sim.state.clone();
    assert!(a.step(&mut sim).is_err());
    assert_eq!(a, old);
    assert_eq!(sim.state, state);
}

#[test]
fn annual_dues_compose_with_material_production_and_consumption_at_existing_boundaries() {
    use economics_compute_smoke::scenario::{GROW, SEED};
    let (mut w, s) = scenario::baseline();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coins".into(),
        kind: ResourceKind::Stock,
    });
    w.rights[0].through = 36;
    w.agreements.push(Agreement {
        id: 1,
        right: 1,
        creditor: STATE_AGENT,
        debtor: PERSON,
        activated: 1,
        payment: Amount::new(GRAIN, 2),
    });
    w.definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .outputs
        .push(Amount::new(SEED, 1));
    let mut a = Audit::with_dues(
        &w,
        &s,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        BTreeMap::from([((PERSON, GRAIN), 10), ((PERSON, SEED), 12)]),
        BTreeMap::from([(1, 3)]),
    )
    .unwrap()
    .with_process_policy(
        &w,
        BTreeMap::from([(GROW, BTreeMap::from([(GRAIN, 3), (SEED, 1)]))]),
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 13);
    let d = a.book().statements(PERSON, 1, 13).unwrap();
    let c = a.book().statements(STATE_AGENT, 1, 13).unwrap();
    assert_eq!(d.expenses[&A::DuesExpense], 6);
    assert_eq!(c.income[&A::DuesIncome], 6);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 2);
    assert_eq!(c.trial_balance[&A::Inventory(GRAIN)], 6);
    assert_eq!(d.liabilities, 0);
    assert!(d.expenses[&A::ConsumptionExpense] > 0);
    assert_eq!(d.cash_flows.values().sum::<i128>(), 0);
    assert!(
        a.clone()
            .with_process_policy(&sim.world, BTreeMap::new())
            .is_err()
    );
}

#[test]
fn lending_and_land_dues_share_reports_without_counting_principal_as_income() {
    use economics_compute_smoke::credit::{Advance, LoanOffer};
    let (mut w, mut s) = fixture(4, true);
    s.balances.insert((STATE_AGENT, TOKEN), 10);
    w.lending.push(Advance {
        id: 10,
        debtor: PERSON,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: 5,
            monthly_rate_bps: 0,
            term_months: 2,
            grace_months: 1,
        },
        principal: 5,
        month: 13,
        collateral: None,
        priority: 0,
    });
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 13);
    let d = a.book().statements(PERSON, 13, 13).unwrap();
    assert_eq!((d.assets, d.liabilities, d.net_income), (7, 5, -4));
    assert_eq!(d.cash_flows[&Flow::Financing], 5);
    assert_eq!(d.cash_flows[&Flow::Operating], -2);
    let c = a.book().statements(STATE_AGENT, 13, 13).unwrap();
    assert_eq!(c.net_income, 5);
    assert_eq!(c.trial_balance[&A::LoanReceivable(10)], 5);
}

#[test]
fn explicit_issuance_policy_counts_only_native_collection_without_duplicate_dues_income() {
    use economics_compute_smoke::{currency::Issuance, issuance_accounting::Policy};
    for native in [1, 2] {
        let (mut w, mut s) = fixture(4, true);
        s.balances.insert((PERSON, GRAIN), native);
        w.issuance.push(Issuance {
            agreement: 1,
            token: TOKEN,
            collected_per_token: 2,
        });
        let mut a = audit(&w, &s)
            .with_issuance_policy(Policy::NonRedeemableEquity)
            .unwrap();
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        through(&mut a, &mut sim, 13);
        let r = a.book().statements(STATE_AGENT, 13, 13).unwrap();
        assert_eq!(r.issuance_change, i128::from(native / 2));
        assert_eq!(r.income[&A::DuesIncome], 6);
        let mut saved = a.clone();
        let mut checkpoint = sim.clone();
        through(&mut a, &mut sim, 14);
        through(&mut saved, &mut checkpoint, 14);
        assert_eq!(a, saved);
        assert_eq!(
            a.book()
                .statements(STATE_AGENT, 14, 14)
                .unwrap()
                .issuance_change,
            0
        );
    }
}
