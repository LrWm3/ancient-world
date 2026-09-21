use economics_compute_smoke::{
    finance::{Condition, FailureRule, Obligation, Transfer, exchange_payment},
    model::Amount,
};

fn claim(failure: FailureRule) -> Obligation {
    Obligation {
        transfer: Transfer {
            from: 1,
            to: 2,
            amount: Amount::new(3, 8),
        },
        settled: 2,
        condition: Condition::OnOrAfterMonth(12),
        failure,
    }
}

#[test]
fn dated_claim_respects_acceptance_due_date_stock_storage_and_prior_payment() {
    let c = claim(FailureRule::CarryArrears);
    assert_eq!(c.payable(11, true, 20, 20), 0);
    assert_eq!(c.payable(12, false, 20, 20), 0);
    assert_eq!(c.payable(12, true, 20, 20), 6);
    assert_eq!(c.payable(13, true, 4, 20), 4);
    assert_eq!(c.payable(13, true, 20, 3), 3);
    assert_eq!(c.payable(13, true, -2, 20), 0);
    assert_eq!(c.payable(13, true, 20, -2), 0);
    let effects = c.payment(3).unwrap();
    assert_eq!(effects[0].account, (1, 3));
    assert_eq!(effects[1].account, (2, 3));
    assert_eq!(effects.iter().map(|e| e.delta).sum::<i32>(), 0);
    assert!(c.payment(7).is_err());
    assert!(c.payment(0).is_err());
}

#[test]
fn acceptance_payment_is_full_or_rejected() {
    assert!(exchange_payment(1, 2, Amount::new(3, 8), 7).is_err());
    assert!(exchange_payment(1, 2, Amount::new(3, 8), 8).is_ok());
    assert!(exchange_payment(1, 1, Amount::new(3, 8), 8).is_err());
    assert!(exchange_payment(1, 2, Amount::new(3, -1), 8).is_err());
    let c = claim(FailureRule::RejectExchange);
    assert!(c.payment(5).is_err());
    assert!(!Condition::OnAcceptance.is_met(0, false));
    assert_eq!(c.payable(12, true, 5, 20), 0);
    assert_eq!(c.payable(12, true, 20, 5), 0);
}

#[test]
fn failure_rules_are_scoped_and_clear_when_settled() {
    let mut c = claim(FailureRule::BlockNewAdvance);
    assert!(c.blocks(FailureRule::BlockNewAdvance));
    assert!(!c.blocks(FailureRule::BlockNewUse));
    c.settled = 8;
    assert!(!c.blocks(FailureRule::BlockNewAdvance));
    assert_eq!(c.payable(12, true, 20, 20), 0);
    c.settled = -1;
    assert_eq!(c.payable(12, true, 20, 20), 0);
    assert!(c.payment(1).is_err());
}
