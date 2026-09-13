//! Read-only monetary views; financial claims are never counted as cash.
use super::{state::Credit, Account, CurrencyId, EntryKind, RepaymentSource, Status};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Amounts {
    pub principal: f64,
    pub interest: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct LoanReport {
    pub id: u64,
    pub currency: CurrencyId,
    pub lender: Account,
    pub borrower: Account,
    pub source: RepaymentSource,
    pub status: Status,
    pub opened_month: u32,
    pub maturity_month: u32,
    pub annual_simple_rate: f64,
    pub original_principal: f64,
    pub due: Amounts,
    pub repaid: Amounts,
    pub default_loss: Amounts,
    pub precision_writeoff: Amounts,
    pub recovered: Amounts,
}
impl Credit {
    /// All contracts, in stable ID order. No accrual, transfers or state mutation.
    /// Loss and recovery stay separate so a recovery cannot conceal a default.
    pub fn loan_reports(&self) -> Vec<LoanReport> {
        let mut recovered = BTreeMap::<u64, Amounts>::new();
        for receipt in &self.recoveries {
            let amount = recovered.entry(receipt.request.loan).or_default();
            amount.principal += receipt.transfer.principal;
            amount.interest += receipt.transfer.interest;
        }
        self.loans
            .iter()
            .map(|loan| {
                let mut repaid = Amounts::default();
                let mut loss = Amounts::default();
                let mut precision = Amounts::default();
                for entry in &loan.entries {
                    let target = match entry.kind {
                        EntryKind::Repayment => &mut repaid,
                        EntryKind::WriteOff => &mut loss,
                        EntryKind::PrecisionWriteOff => &mut precision,
                        _ => continue,
                    };
                    target.principal += entry.principal;
                    target.interest += entry.interest;
                }
                LoanReport {
                    id: loan.id,
                    currency: loan.terms.currency,
                    lender: loan.terms.lender,
                    borrower: loan.terms.borrower,
                    source: loan.terms.source,
                    status: loan.status,
                    opened_month: loan.opened_month,
                    maturity_month: loan.terms.maturity_month,
                    annual_simple_rate: loan.terms.annual_simple_rate,
                    original_principal: loan.original_principal,
                    due: Amounts {
                        principal: loan.outstanding_principal,
                        interest: loan.interest_due,
                    },
                    repaid,
                    default_loss: loss,
                    precision_writeoff: precision,
                    recovered: recovered.remove(&loan.id).unwrap_or_default(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credit::{accounts::Transfer, recovery, Loan, Terms, SHARED_CURRENCY};

    #[test]
    fn report_preserves_default_and_separates_financing_from_recovery() {
        let terms = Terms {
            lender: Account::Council(0),
            borrower: Account::Town(1),
            currency: SHARED_CURRENCY,
            source: RepaymentSource::AnnualTax {
                council: 1,
                collection_month: 12,
            },
            annual_simple_rate: 0.12,
            maturity_month: 12,
            grace_months: 0,
        };
        let mut loan = Loan::record_disbursement(0, terms, 0, 100.).unwrap();
        loan.accrue_to(12).unwrap();
        loan.record_repayment(12, 22.).unwrap();
        loan.write_off(12).unwrap();
        let mut credit = Credit::default();
        credit.loans.push(loan);
        credit.recoveries.push(recovery::Receipt {
            request: recovery::Request {
                id: 0,
                month: 13,
                loan: 0,
                allowance: 5.,
                reason: recovery::Reason::DelayedProceeds,
            },
            transfer: Transfer {
                month: 13,
                from: Account::Town(1),
                to: Account::Council(0),
                currency: SHARED_CURRENCY,
                requested: 5.,
                principal: 5.,
                interest: 0.,
            },
            opening_cash: [5., 0.],
            closing_cash: [0., 5.],
        });
        let before = serde_json::to_value(&credit).unwrap();
        let rows = credit.loan_reports();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.status, Status::Defaulted);
        assert_eq!(r.original_principal, 100.);
        assert_eq!(r.repaid.principal, 10.);
        assert_eq!(r.repaid.interest, 12.);
        assert_eq!(r.default_loss.principal, 90.);
        assert_eq!(r.default_loss.interest, 0.);
        assert_eq!(r.due.principal + r.due.interest, 0.);
        assert_eq!(r.precision_writeoff.principal, 0.);
        assert_eq!(r.recovered.principal, 5.);
        assert_eq!(serde_json::to_value(&credit).unwrap(), before);
        let restored: Credit = serde_json::from_value(before).unwrap();
        assert_eq!(
            serde_json::to_value(restored.loan_reports()).unwrap(),
            serde_json::to_value(rows).unwrap()
        );
    }
}
