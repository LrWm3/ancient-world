//! Persisted contract commits. Underwriting is a separate policy boundary.
use super::{accounts::Transfer, Loan, Status, Terms, SHARED_CURRENCY};
use crate::civilization::History;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CashReceipt {
    pub loan: u64,
    pub disbursement: bool,
    pub transfer: Transfer,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Credit {
    #[serde(default)]
    pub rounds: Vec<super::underwriting::Round>,
    pub loans: Vec<Loan>,
    pub cash_receipts: Vec<CashReceipt>,
}

impl History {
    /// Commit a loan whose consent and underwriting have already been resolved.
    /// No speculative receivable is credited; principal equals actual lender cash.
    pub fn commit_credit_loan(&mut self, terms: Terms, requested: f64) -> Result<Option<u64>> {
        terms.validate(self.month)?;
        ensure!(
            terms.currency == SHARED_CURRENCY,
            "unsupported loan currency"
        );
        ensure!(
            requested.is_finite() && requested > 0.,
            "invalid principal request"
        );
        let id = self.credit.loans.len() as u64;
        let transfer =
            self.transfer_credit_cash(terms.lender, terms.borrower, terms.currency, requested, 0.)?;
        if transfer.amount() == 0. {
            return Ok(None);
        }
        // Terms and amount validity were checked before either account changed.
        let loan = Loan::record_disbursement(id, terms, self.month, transfer.amount())
            .expect("preflighted terms and actual cash transfer");
        self.credit.loans.push(loan);
        self.credit.cash_receipts.push(CashReceipt {
            loan: id,
            disbursement: true,
            transfer,
        });
        Ok(Some(id))
    }

    /// Collect a bounded payment at the current month. Policy determines the
    /// allowance; this method never bypasses account cash or compounds interest.
    pub fn pay_credit_loan(&mut self, id: u64, allowance: f64) -> Result<f64> {
        let mut loan = self
            .credit
            .loans
            .get(id as usize)
            .filter(|l| l.id == id)
            .context("missing loan")?
            .clone();
        loan.accrue_to(self.month)?;
        let (principal, interest) = loan.repayment_quote(allowance)?;
        let transfer = self.transfer_credit_cash(
            loan.terms.borrower,
            loan.terms.lender,
            loan.terms.currency,
            principal,
            interest,
        )?;
        let actual = transfer.amount();
        if actual > 0. {
            loan.record_repayment(self.month, actual)
                .expect("preflighted current repayment");
            self.credit.cash_receipts.push(CashReceipt {
                loan: id,
                disbursement: false,
                transfer,
            });
        }
        self.credit.loans[id as usize] = loan;
        Ok(actual)
    }

    pub fn validate_credit(&self) -> Result<()> {
        let mut funded = std::collections::BTreeSet::new();
        let mut requests = std::collections::BTreeSet::new();
        for round in &self.credit.rounds {
            ensure!(
                round.complete
                    && round.month <= self.month
                    && round.grants.len() == round.loan_ids.len()
                    && round.grants.len() == round.requests.len(),
                "incomplete or invalid credit round"
            );
            for request in &round.requests {
                ensure!(
                    requests.insert((round.month, request.id)),
                    "replayed credit request"
                );
            }
            for (grant, id) in round.grants.iter().zip(&round.loan_ids) {
                let request = round
                    .requests
                    .iter()
                    .find(|r| r.id == grant.request)
                    .context("missing grant request")?;
                ensure!(
                    grant.month == round.month
                        && grant.granted.is_finite()
                        && grant.granted >= 0.
                        && grant.granted <= grant.requested,
                    "invalid credit grant"
                );
                if let Some(id) = id {
                    let loan = self
                        .credit
                        .loans
                        .get(*id as usize)
                        .context("missing funded loan")?;
                    ensure!(
                        funded.insert(*id)
                            && loan.id == *id
                            && loan.opened_month == round.month
                            && loan.original_principal <= grant.granted
                            && loan.terms.lender == request.terms.lender
                            && loan.terms.borrower == request.terms.borrower
                            && loan.terms.source == request.terms.source,
                        "funded loan disagrees with grant"
                    );
                }
            }
        }
        for (id, loan) in self.credit.loans.iter().enumerate() {
            ensure!(
                loan.id == id as u64 && loan.accrued_through_month <= self.month,
                "invalid loan identity or future boundary"
            );
            loan.validate()?;
            ensure!(
                loan.terms.currency == SHARED_CURRENCY,
                "unsupported persisted loan currency"
            );
            let mut disbursements = 0;
            let mut principal_paid = 0.;
            let mut interest_paid = 0.;
            for receipt in self
                .credit
                .cash_receipts
                .iter()
                .filter(|r| r.loan == loan.id)
            {
                let t = &receipt.transfer;
                ensure!(
                    t.currency == loan.terms.currency
                        && t.month >= loan.opened_month
                        && t.month <= self.month
                        && [t.requested, t.principal, t.interest]
                            .iter()
                            .all(|x| x.is_finite() && *x >= 0.)
                        && t.amount() <= t.requested,
                    "invalid credit cash receipt"
                );
                if receipt.disbursement {
                    disbursements += 1;
                    ensure!(
                        t.from == loan.terms.lender
                            && t.to == loan.terms.borrower
                            && t.month == loan.opened_month
                            && t.principal == loan.original_principal
                            && t.interest == 0.,
                        "invalid disbursement provenance"
                    );
                } else {
                    ensure!(
                        t.from == loan.terms.borrower && t.to == loan.terms.lender,
                        "invalid repayment counterparties"
                    );
                    principal_paid += t.principal;
                    interest_paid += t.interest;
                }
            }
            ensure!(
                disbursements == 1,
                "missing or duplicated cash disbursement"
            );
            let mut p = 0.;
            let mut i = 0.;
            for e in &loan.entries {
                if matches!(e.kind, super::EntryKind::Repayment) {
                    p += e.principal;
                    i += e.interest;
                }
            }
            ensure!(
                p == principal_paid && i == interest_paid,
                "cash and debt repayments disagree"
            );
            ensure!(
                loan.status != Status::Repaid || loan.total_due() == 0.,
                "unsettled repaid loan"
            );
        }
        ensure!(
            self.credit.cash_receipts.iter().all(|r| self
                .credit
                .loans
                .get(r.loan as usize)
                .is_some_and(|l| l.id == r.loan)),
            "orphan credit cash receipt"
        );
        Ok(())
    }
}
