use economics_compute_smoke::{
    accounting::Account,
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::{Audit, Opening},
    model::*,
    scenario::{self, GRAIN, PERSON, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
fn fixture(extra: i32) -> (World, State, Opening) {
    let (mut w, mut s) = scenario::baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights.clear();
    w.agreements.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "reporting coin".into(),
        kind: ResourceKind::Stock,
    });
    w.storage.weights.insert(GRAIN, 1);
    w.lending.push(Advance {
        id: 1,
        debtor: PERSON,
        month: 1,
        principal: 4,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: GRAIN,
            max_principal: 4,
            monthly_rate_bps: 2500,
            term_months: 1,
            grace_months: 12,
        },
    });
    s.balances.clear();
    s.balances.insert((STATE_AGENT, GRAIN), 6);
    s.balances.insert((PERSON, GRAIN), extra);
    let mut opening = Opening {
        inventory: BTreeMap::from([((STATE_AGENT, GRAIN), 6)]),
        exchange_values: BTreeMap::from([(GRAIN, 3)]),
        assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
        ..Default::default()
    };
    if extra > 0 {
        opening
            .inventory
            .insert((PERSON, GRAIN), i128::from(extra * 2));
    }
    (w, s, opening)
}
#[test]
fn physical_advances_interest_and_repayments_reconcile_without_sales_or_cash() {
    for extra in [0, 1] {
        let (w, s, opening) = fixture(extra);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = Audit::with_opening(&w, &s, TOKEN, opening.clone()).unwrap();
            while sim.state.month <= 1 {
                a.step(&mut sim).unwrap();
            }
            let b = a.book().balances();
            assert_eq!(b[&(STATE_AGENT, Account::LoanReceivable(1))], 12);
            assert_eq!(b[&(PERSON, Account::LoanPayable(1))], -12);
            assert_eq!(b[&(STATE_AGENT, Account::SettlementGain)], -8);
            let mut resumed = Simulation::new(w.clone(), sim.state.clone(), backend).unwrap();
            let mut resumed_a = a.clone();
            while sim.state.month <= 2 {
                a.step(&mut sim).unwrap();
            }
            while resumed.state.month <= 2 {
                resumed_a.step(&mut resumed).unwrap();
            }
            assert_eq!(sim.state, resumed.state);
            assert_eq!(a, resumed_a);
            assert_eq!(sim.state.credit.loans[&1].principal, 1 - extra);
            let b = a.book().balances();
            assert_eq!(b[&(STATE_AGENT, Account::InterestIncome)], -3);
            assert_eq!(b[&(PERSON, Account::InterestExpense)], 3);
            for ((_, account), amount) in b {
                if matches!(
                    account,
                    Account::Sales | Account::CostOfSales | Account::Cash
                ) {
                    assert_eq!(*amount, 0);
                }
            }
            for agent in [STATE_AGENT, PERSON] {
                let statement = a.book().statements(agent, 1, 2).unwrap();
                assert_eq!(statement.closing_cash, 0);
            }
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
    }
}
#[test]
fn missing_commodity_value_rejects_reporting_atomically_before_loan_publication() {
    let (w, s, mut opening) = fixture(0);
    opening.exchange_values.clear();
    let mut a = Audit::with_opening(&w, &s, TOKEN, opening).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Acquire {
        a.step(&mut sim).unwrap();
    }
    let before = sim.state.clone();
    let before_a = a.clone();
    assert!(a.step(&mut sim).is_err());
    assert_eq!(before, sim.state);
    assert_eq!(before_a, a);
}

#[test]
fn physical_loan_guarantee_delivers_stock_and_creates_same_unit_recourse() {
    use economics_compute_smoke::recovery::{Guarantee, GuaranteedClaim};
    let (mut w, mut s, mut opening) = fixture(0);
    w.agents.push(Agent {
        id: 99,
        name: "grain guarantor".into(),
    });
    s.balances.insert((99, GRAIN), 2);
    opening.inventory.insert((99, GRAIN), 2);
    w.recovery.guarantees.push(Guarantee {
        follows_assignment: false,
        tender: economics_compute_smoke::recovery::GuaranteeTender::Native,
        security: economics_compute_smoke::recovery::RecourseSecurity::Unsecured,
        id: 1,
        claim: GuaranteedClaim::Loan(1),
        guarantor: 99,
        cap: 4,
        from: 1,
        through: 12,
        delay_months: 0,
        recourse: 101,
        priority: 0,
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = Audit::with_opening(&w, &s, TOKEN, opening.clone()).unwrap();
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.credit.loans[&1].principal, 0);
        let recourse = &sim.state.credit.loans[&101];
        assert_eq!(
            (recourse.principal, recourse.denomination, recourse.opened),
            (1, GRAIN, 2)
        );
        assert_eq!(sim.state.balance(99, GRAIN), 1);
        assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 7);
        let b = a.book().balances();
        assert_eq!(b[&(99, Account::LoanReceivable(101))], 3);
        assert_eq!(b[&(PERSON, Account::LoanPayable(101))], -3);
        assert_eq!(b[&(99, Account::SettlementGain)], -2);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
}

#[test]
fn coin_proceeding_admits_unsecured_native_arrears_without_converting_or_accruing_them() {
    use economics_compute_smoke::{
        credit::Status,
        recovery::{ProceedingTerms, Stage},
    };
    let (mut w, mut s, opening) = fixture(0);
    w.agents.push(Agent {
        id: 999,
        name: "coin custodian".into(),
    });
    s.balances.insert((PERSON, TOKEN), 10);
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: 999,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 3,
        assets: vec![],
        discharge_deficiency: true,
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = Audit::with_opening(&w, &s, TOKEN, opening.clone()).unwrap();
        while sim.state.month <= 5 {
            a.step(&mut sim).unwrap();
        }
        let loan = &sim.state.credit.loans[&1];
        assert_eq!(
            (loan.principal, loan.interest, loan.last_accrued),
            (1, 0, 5)
        );
        assert_eq!(loan.status, Status::Stayed);
        let case = &sim.state.credit.recovery.proceedings[&1];
        assert_eq!((case.stage, case.cash), (Stage::Active, 0));
        assert_eq!(sim.state.balance(PERSON, TOKEN), 10);
        assert_eq!(a.book().balances()[&(PERSON, Account::LoanPayable(1))], -3);
        assert_eq!(
            a.book().balances()[&(STATE_AGENT, Account::LoanReceivable(1))],
            3
        );
        assert_eq!(a.book().balances()[&(PERSON, Account::InterestExpense)], 3);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
}
