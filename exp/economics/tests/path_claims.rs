//! Controlled observation boundaries; claim validation/admission is tested by its adapters.
use economics_compute_smoke::{
    acceptance::{TOWN, WORKSHOP, continuation::Case, financing},
    agency::{
        objectives::{self, Metric, Objective, Scope},
        path::{ClaimKind, Path},
    },
    credit::{Loan, Status},
    delivery_relief,
    forward::direct::Terms,
    minting::{COIN, ORE},
    model::*,
};
#[test]
fn path_uses_scheduled_loan_amounts_and_effective_delivery_claims() {
    let (w, mut s) = financing::scenario(Case::Baseline).unwrap();
    s.credit.loans.clear();
    s.credit.loans.insert(
        90,
        Loan {
            id: 90,
            creditor: TOWN,
            debtor: WORKSHOP,
            denomination: COIN,
            original_principal: 4,
            principal: 4,
            interest: 0,
            interest_remainder: 0,
            monthly_rate_bps: 0,
            opened: 1,
            last_accrued: 1,
            term_months: 2,
            grace_months: 1,
            first_unpaid: None,
            status: Status::Active,
            collateral: None,
            priority: 0,
        },
    );
    let terms = Terms {
        id: 91,
        month: 1,
        due: 2,
        seller: TOWN,
        buyer: WORKSHOP,
        goods: Amount::new(ORE, 2),
        prepayment: Amount::new(COIN, 2),
    };
    s.exchange.forwards.insert(91, terms.contract());
    let mut path = Path::new(1);
    s.month = 2;
    path.record(&w, &s).unwrap();
    assert!(path.closes[0].unpaid.is_empty());
    s.month = 3;
    path.record(&w, &s).unwrap();
    assert_eq!(
        path.closes[1]
            .unpaid
            .iter()
            .map(|c| c.quantity)
            .collect::<Vec<_>>(),
        [2, 2]
    );
    let loan = s.credit.loans.get_mut(&90).unwrap();
    loan.principal = 2;
    let c = s.exchange.forwards.get_mut(&91).unwrap();
    c.relief.push(delivery_relief::Applied {
        delivered: 0,
        substituted: 0,
        terms: delivery_relief::Terms {
            id: 1,
            proceeding: 1,
            contract: 91,
            debtor: TOWN,
            creditor: WORKSHOP,
            month: 3,
            expected_due: 2,
            expected_remaining: 2,
            action: delivery_relief::Action::Extend { due: 4 },
        },
    });
    s.month = 4;
    path.record(&w, &s).unwrap();
    assert_eq!(path.closes[2].unpaid.len(), 1);
    assert_eq!(path.closes[2].unpaid[0].kind, ClaimKind::Loan(90));
    s.credit.loans.get_mut(&90).unwrap().principal = 0;
    s.exchange.forwards.get_mut(&91).unwrap().substituted = 1;
    s.month = 5;
    path.record(&w, &s).unwrap();
    assert_eq!(path.closes[3].unpaid.len(), 1);
    assert_eq!(path.closes[3].unpaid[0].quantity, 1);
    assert_eq!(path.closes[3].unpaid[0].kind, ClaimKind::Delivery(91));
}
#[test]
fn frozen_member_exposure_keeps_original_beneficiaries() {
    let (mut w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let goals = [Objective {
        scope: Scope::Members,
        metric: Metric::ReserveExposure {
            resource: COIN,
            target: 10,
        },
    }];
    let frozen = objectives::freeze(&w, &s, TOWN, &goals);
    let Scope::Agents(members) = &frozen[0].scope else {
        panic!("unfrozen scope")
    };
    for &a in members {
        s.balances.insert((a, COIN), 0);
    }
    let mut path = Path::new(1);
    s.month = 2;
    path.record(&w, &s).unwrap();
    // Controlled roster removal: final membership cannot erase opening beneficiaries.
    w.agency.remove(&TOWN);
    assert!(objectives::members(&w, &s, TOWN).is_empty());
    assert_eq!(
        objectives::measure_projection(&w, &s, &[], TOWN, &frozen, &path).unwrap(),
        [10 * members.len() as i128]
    );
    assert!(objectives::measure_projection(&w, &s, &[], TOWN, &goals, &path).is_err());
}
