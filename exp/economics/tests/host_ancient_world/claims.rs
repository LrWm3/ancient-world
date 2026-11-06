use super::*;
use ancient_world::credit::{
    Account, RepaymentSource, SHARED_CURRENCY, Status, Terms,
    ownership::Request as AssignmentRequest, recovery,
};

fn source(month: u32) -> History {
    let mut h = stock_history(month, 2.);
    h.sites.push(Site {
        id: 1,
        name: "external town".into(),
        ..h.sites[0].clone()
    });
    h
}

fn loan(h: &mut History, lender: Account, borrower: Account, principal: f64, maturity: u32) -> u64 {
    let id = h
        .commit_credit_loan(
            Terms {
                lender,
                borrower,
                currency: SHARED_CURRENCY,
                source: RepaymentSource::AnnualTax {
                    council: 0,
                    collection_month: maturity,
                },
                annual_simple_rate: 0.,
                maturity_month: maturity,
                grace_months: 0,
            },
            principal,
        )
        .unwrap()
        .unwrap();
    h.validate_credit().unwrap();
    id
}

fn next(h: &History) -> History {
    let mut next = h.clone();
    next.month += 1;
    next.participation = history(next.month, 0.5, 0.5).participation;
    next
}

fn assign(h: &mut History, loan: u64, from: Account, to: Account) {
    h.assign_credit_claim(AssignmentRequest {
        id: 17,
        loan,
        month: h.month,
        from,
        to,
        owner_consent: Some(from),
        recipient_consent: Some(to),
        cause: None,
    })
    .unwrap();
    h.validate_credit().unwrap();
}

#[test]
fn unsupported_debt_and_receivable_block_bootstrap_without_changing_source() {
    for (lender, borrower) in [
        (Account::Council(0), Account::Town(0)),
        (Account::Town(0), Account::Town(1)),
    ] {
        let mut h = source(10);
        loan(&mut h, lender, borrower, 4.125, 14);
        let before = serde_json::to_value(&h).unwrap();
        let report = stock_bindings().inspect_claims(&h).unwrap();
        assert_eq!(report.loans.len(), 1);
        let row = &report.loans[0];
        assert_eq!(row.outstanding.principal, 4.125);
        assert_eq!(row.outstanding.interest, 0.);
        assert_eq!(row.maturity_month, 14);
        assert_eq!(row.accrued_through_month, 10);
        assert_eq!(
            row.creditor_agent,
            (lender == Account::Town(0)).then_some(STATE_AGENT)
        );
        assert_eq!(
            row.borrower_agent,
            (borrower == Account::Town(0)).then_some(STATE_AGENT)
        );
        assert!(report.require_liquid_only().is_err());
        assert!(
            stock_bindings()
                .start(&h, &next(&h), bindings(), stock_world(), Backend::Reference)
                .unwrap_err()
                .contains("cannot omit source credit claims [0]")
        );
        assert_eq!(serde_json::to_value(&h).unwrap(), before);
    }
}

#[test]
fn pending_household_assignment_is_visible_before_it_becomes_the_current_owner() {
    let mut h = source(10);
    let id = loan(&mut h, Account::Town(1), Account::Council(0), 4., 14);
    assign(&mut h, id, Account::Town(1), Account::Household(0));
    let opening_cash = h
        .society
        .as_ref()
        .unwrap()
        .household_economy
        .as_ref()
        .unwrap()
        .accounts[0]
        .cash;
    let report = stock_bindings().inspect_claims(&h).unwrap();
    let row = &report.loans[0];
    assert_eq!(row.creditor, Account::Town(1));
    assert_eq!(row.creditor_agent, None);
    assert_eq!(row.next_creditor, Account::Household(0));
    assert_eq!(row.next_creditor_agent, Some(WALLET_AGENT));
    assert_eq!(row.assignments[0].effective_month, 11);
    assert!(report.require_liquid_only().is_err());
    let later = next(&h);
    let later_report = stock_bindings().inspect_claims(&later).unwrap();
    assert_eq!(later_report.loans[0].creditor_agent, Some(WALLET_AGENT));
    assert_eq!(later_report.loans[0].original_lender, Account::Town(1));
    assert_eq!(
        later
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .accounts[0]
            .cash,
        opening_cash
    );
    let roundtrip: History = serde_json::from_slice(&serde_json::to_vec(&later).unwrap()).unwrap();
    assert_eq!(
        stock_bindings().inspect_claims(&roundtrip).unwrap(),
        later_report
    );
    let mut bad = serde_json::to_value(&h).unwrap();
    bad["credit"]["ownership"]["assignments"][0]["request"]["from"] =
        serde_json::json!({"Town": 0});
    let bad: History = serde_json::from_value(bad).unwrap();
    assert!(stock_bindings().inspect_claims(&bad).is_err());
}

#[test]
fn assigned_away_claim_is_not_still_owned_by_the_original_lender() {
    let mut h = source(10);
    let id = loan(&mut h, Account::Town(0), Account::Council(0), 4., 14);
    assign(&mut h, id, Account::Town(0), Account::Town(1));
    assert!(
        stock_bindings()
            .inspect_claims(&h)
            .unwrap()
            .require_liquid_only()
            .is_err()
    );
    let h = next(&h);
    let report = stock_bindings().inspect_claims(&h).unwrap();
    assert!(report.loans.is_empty());
    assert!(
        stock_bindings()
            .start(&h, &next(&h), bindings(), stock_world(), Backend::Reference)
            .is_ok()
    );
}

#[test]
fn default_loss_and_recovery_remain_distinct_even_after_full_recovery() {
    let mut h = source(1);
    let id = h
        .commit_credit_loan(
            Terms {
                lender: Account::Council(0),
                borrower: Account::Town(0),
                currency: SHARED_CURRENCY,
                source: RepaymentSource::AnnualTax {
                    council: 0,
                    collection_month: 2,
                },
                annual_simple_rate: 0.375,
                maturity_month: 2,
                grace_months: 0,
            },
            4.,
        )
        .unwrap()
        .unwrap();
    h = next(&h);
    h.credit.loans[id as usize].accrue_to(h.month).unwrap();
    let accrued = stock_bindings().inspect_claims(&h).unwrap();
    assert_eq!(accrued.loans[0].outstanding.interest, 0.125);
    assert_eq!(accrued.loans[0].annual_simple_rate, 0.375);
    h.credit.loans[id as usize].write_off(h.month).unwrap();
    h.recover_defaulted_credit(recovery::Request {
        id: 0,
        loan: id,
        month: h.month,
        allowance: 1.,
        reason: recovery::Reason::VoluntarySettlement,
    })
    .unwrap();
    h.validate_credit().unwrap();
    let report = stock_bindings().inspect_claims(&h).unwrap();
    let row = &report.loans[0];
    assert_eq!(row.status, Status::Defaulted);
    assert_eq!(row.outstanding.principal, 0.);
    assert_eq!(row.written_off.principal, 4.);
    assert_eq!(row.written_off.interest, 0.125);
    assert_eq!(row.recovered.interest, 0.125);
    assert_eq!(row.recovered.principal, 0.875);
    assert_eq!(row.unrecovered.principal, 3.125);
    assert_eq!(row.unrecovered.interest, 0.);
    assert!(row.blocks_liquid_import());
    h.recover_defaulted_credit(recovery::Request {
        id: 1,
        loan: id,
        month: h.month,
        allowance: 3.125,
        reason: recovery::Reason::VoluntarySettlement,
    })
    .unwrap();
    let fully = stock_bindings().inspect_claims(&h).unwrap();
    assert_eq!(fully.loans[0].unrecovered.principal, 0.);
    // Default history still affects native policy; it cannot be erased by import.
    assert!(fully.require_liquid_only().is_err());
}

#[test]
fn repaid_and_unrelated_credit_do_not_create_stock_or_block_liquid_import() {
    let mut h = source(10);
    let id = loan(&mut h, Account::Council(0), Account::Town(0), 4., 14);
    assert_eq!(h.pay_credit_loan(id, 4.).unwrap(), 4.);
    loan(&mut h, Account::Town(1), Account::Council(0), 4., 14);
    let (replay, receipt) = stock_bindings()
        .start(&h, &next(&h), bindings(), stock_world(), Backend::Reference)
        .unwrap();
    assert_eq!(receipt.claims.loans.len(), 1);
    assert_eq!(receipt.claims.loans[0].status, Status::Repaid);
    assert_eq!(receipt.claims.loans[0].repaid.principal, 4.);
    assert_eq!(
        replay
            .economy()
            .simulation()
            .state
            .balance(STATE_AGENT, TOKEN),
        37
    );
    assert!(replay.economy().simulation().state.credit.loans.is_empty());
    assert_eq!(receipt.claims.household_shares[0].share, 1.);
    assert_eq!(receipt.claims.total_household_share, 1.);
}

#[test]
fn claims_below_one_imported_cash_tick_are_not_rounded_away() {
    let mut h = source(10);
    loan(&mut h, Account::Council(0), Account::Town(0), 0.125, 14);
    let report = stock_bindings().inspect_claims(&h).unwrap();
    assert_eq!(report.loans[0].outstanding.principal, 0.125);
    assert!(report.require_liquid_only().is_err()); // Currency tick is 0.25.
}

#[test]
fn source_credit_and_property_evidence_must_validate_before_admission() {
    let mut h = source(10);
    loan(&mut h, Account::Council(0), Account::Town(0), 4., 14);
    let mut cases = Vec::new();
    let mut broken = h.clone();
    broken.credit.cash_receipts.clear();
    cases.push(broken);
    let mut broken = h.clone();
    broken.credit.loans[0].outstanding_principal += 1.;
    cases.push(broken);
    let mut broken = h.clone();
    broken.credit.loans[0].accrued_through_month = 11;
    cases.push(broken);
    let mut broken = h.clone();
    broken.credit.loans[0].terms.annual_simple_rate = f64::NAN;
    cases.push(broken);
    let mut broken = h.clone();
    broken.society.as_mut().unwrap().households[0].share = f64::NAN;
    cases.push(broken);
    let mut broken = h.clone();
    broken.society.as_mut().unwrap().households[0].head = 999;
    cases.push(broken);
    for broken in cases {
        let before = serde_json::to_value(&broken).unwrap();
        assert!(stock_bindings().inspect_claims(&broken).is_err());
        assert_eq!(serde_json::to_value(&broken).unwrap(), before);
    }
    let mut missing = stock_bindings();
    missing.wallets.clear();
    assert!(missing.inspect_claims(&h).is_err());
    let mut merged = stock_bindings();
    merged.wallets[0].agent = STATE_AGENT;
    assert!(merged.inspect_claims(&h).is_err());

    let mut written_off = source(1);
    loan(
        &mut written_off,
        Account::Council(0),
        Account::Town(0),
        4.,
        2,
    );
    written_off = next(&written_off);
    written_off.credit.loans[0].accrue_to(2).unwrap();
    written_off.credit.loans[0].write_off(2).unwrap();
    written_off.credit.loans[0].status = Status::Repaid;
    written_off.validate_credit().unwrap(); // Native balance reconciliation alone accepts this.
    assert!(
        stock_bindings()
            .inspect_claims(&written_off)
            .unwrap_err()
            .contains("default status disagrees")
    );
}

#[test]
fn individually_finite_source_entries_cannot_publish_infinite_claim_totals() {
    use ancient_world::credit::{Entry, EntryKind, accounts::Transfer, state::CashReceipt};
    let mut h = source(10);
    let id = loan(&mut h, Account::Council(0), Account::Town(0), 4., 14);
    h.pay_credit_loan(id, 4.).unwrap();
    // The native validator reconciles each pair but does not check the cumulative
    // repayment total used by the reporting view. No individual field is infinite.
    for _ in 0..2 {
        h.credit.loans[0].entries.extend([
            Entry {
                month: 10,
                kind: EntryKind::InterestAccrued,
                principal: 0.,
                interest: 1e308,
            },
            Entry {
                month: 10,
                kind: EntryKind::Repayment,
                principal: 0.,
                interest: 1e308,
            },
        ]);
        h.credit.cash_receipts.push(CashReceipt {
            loan: id,
            disbursement: false,
            transfer: Transfer {
                month: 10,
                from: Account::Town(0),
                to: Account::Council(0),
                currency: SHARED_CURRENCY,
                requested: 1e308,
                principal: 0.,
                interest: 1e308,
            },
        });
    }
    h.validate_credit().unwrap();
    assert!(h.credit.loan_reports()[0].repaid.interest.is_infinite());
    assert!(
        stock_bindings()
            .inspect_claims(&h)
            .unwrap_err()
            .contains("overflowing source claim total")
    );
    assert!(
        stock_bindings()
            .start(&h, &next(&h), bindings(), stock_world(), Backend::Reference)
            .is_err()
    );
}
