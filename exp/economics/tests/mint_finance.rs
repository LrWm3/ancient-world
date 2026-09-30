use economics_compute_smoke::{
    accounting::Flow,
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::Audit,
    issuance_accounting,
    minting::{self, *},
    model::*,
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    simulation::Simulation,
};
use std::collections::BTreeMap;

fn fixture() -> (World, State) {
    let (mut w, s) = minting::scenario("normal").unwrap();
    let c = w.minting.as_mut().unwrap();
    c.deals.retain(|d| d.month == 2);
    let early: Vec<_> = c
        .deals
        .iter()
        .cloned()
        .map(|mut d| {
            d.id += 10;
            d.month = 1;
            d
        })
        .collect();
    c.deals.extend(early);
    w.lending.push(Advance {
        id: 10,
        debtor: ISSUER,
        terms: LoanOffer {
            creditor: SUPPLIER,
            denomination: COIN,
            max_principal: 6,
            monthly_rate_bps: 0,
            term_months: 12,
            grace_months: 1,
        },
        principal: 6,
        month: 1,
        collateral: None,
        priority: 0,
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .extend([(PERSON_TYPE, Action::Lend), (STATE_TYPE, Action::Borrow)]);
    (w, s)
}
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
    Audit::with_processes(w, s, COIN, BTreeMap::new(), costs, BTreeMap::new())
        .unwrap()
        .with_issuance_policy(issuance_accounting::Policy::NonRedeemableEquity)
        .unwrap()
}

#[test]
fn borrowed_coins_fund_later_physical_issuance_and_repayment_with_reconciled_books() {
    let (w, s) = fixture();
    let mut a = audit(&w, &s);
    let mut b = a.clone();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while reference.state.month <= 1 {
        a.step(&mut reference).unwrap();
    }
    assert_eq!(reference.state.balance(ISSUER, COIN), 6);
    assert_eq!(reference.state.balance(ISSUER, METAL), 0);
    let mint = reference
        .ledger
        .iter()
        .find_map(|b| b.minting.as_ref())
        .unwrap();
    assert!(!mint.receipts[0].accepted);
    let mut resumed = Simulation::new(
        reference.world.clone(),
        reference.state.clone(),
        Backend::Reference,
    )
    .unwrap();
    let mut resumed_audit = a.clone();
    while reference.state.month <= 3 {
        a.step(&mut reference).unwrap();
    }
    while resumed.state.month <= 3 {
        resumed_audit.step(&mut resumed).unwrap();
    }
    cpu.world.agents.reverse();
    cpu.world.minting.as_mut().unwrap().deals.reverse();
    while cpu.state.month <= 3 {
        b.step(&mut cpu).unwrap();
    }
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.state, resumed.state);
    assert_eq!(reference.ledger, cpu.ledger);
    assert_eq!(a, b);
    assert_eq!(a, resumed_audit);
    assert_eq!(reference.state.credit.loans[&10].principal, 5);
    assert_eq!(reference.state.balance(ISSUER, COIN), 9);
    let statement = a.book().statements(ISSUER, 1, 3).unwrap();
    assert_eq!(statement.issuance_change, 10);
    assert_eq!(statement.cash_flows[&Flow::Financing], 5);
    assert_eq!(statement.closing_cash, 9);
}

#[test]
fn lending_reserves_the_same_opening_money_as_mint_purchases() {
    let (mut w, mut s) = fixture();
    w.lending[0].month = 2;
    w.lending[0].debtor = SUPPLIER;
    w.lending[0].terms.creditor = ISSUER;
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .extend([(STATE_TYPE, Action::Lend), (PERSON_TYPE, Action::Borrow)]);
    s.balances.insert((ISSUER, COIN), 6);
    w.minting.as_mut().unwrap().deals.retain(|d| d.month == 2);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.credit.loans[&10].principal, 6);
    assert_eq!(sim.state.balance(ISSUER, COIN), 0);
    assert_eq!(sim.state.balance(ISSUER, METAL), 0);
    assert!(
        !sim.state
            .processes
            .values()
            .any(|p| p.definition == MINT && p.status == Status::Completed)
    );
}
