//! Monthly Open collection against one completed cash snapshot.
use super::{Account, Status};
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const DEFAULT_COLLECTION_CASH_SHARE: f64 = 0.25;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    /// Further protects ordinary spending after explicit account reserves.
    pub available_cash_share: f64,
    pub protected_cash: Vec<(Account, f64)>,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            available_cash_share: DEFAULT_COLLECTION_CASH_SHARE,
            protected_cash: Vec::new(),
        }
    }
}
impl Policy {
    pub fn validate(&self) -> Result<()> {
        let mut accounts = std::collections::BTreeSet::new();
        ensure!(
            self.protected_cash.iter().all(|(a, _)| accounts.insert(*a)),
            "duplicate protected cash account"
        );
        ensure!(
            self.available_cash_share.is_finite()
                && (0. ..=1.).contains(&self.available_cash_share),
            "invalid collection share"
        );
        ensure!(
            self.protected_cash
                .iter()
                .map(|(_, x)| x)
                .all(|x| x.is_finite() && *x >= 0.),
            "invalid protected cash"
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub month: u32,
    pub loan: u64,
    pub due: f64,
    pub opening_cash: Option<f64>,
    pub protected_cash: f64,
    pub allowance: f64,
    pub paid: f64,
    pub accounts_available: bool,
    pub defaulted: bool,
    #[serde(default)]
    pub precision_settled: f64,
    /// Affordable claim retained because the exact transfer adapter cannot move it.
    #[serde(default)]
    pub precision_blocked: bool,
}

impl History {
    /// After arrivals and economy preparation, before new work reservations.
    /// Interest advances monthly even before maturity. Payments start at maturity;
    /// all same-borrower obligations share the opening allowance proportionally.
    pub fn service_credit_month(&mut self) -> Result<()> {
        if self.credit.loans.is_empty() || self.credit.serviced_month == Some(self.month) {
            return Ok(());
        }
        ensure!(
            self.credit.serviced_month.is_none_or(|m| m < self.month),
            "credit servicing moved backwards"
        );
        ensure!(
            !self
                .credit
                .service_receipts
                .iter()
                .any(|r| r.month == self.month),
            "incomplete credit service batch requires recovery"
        );
        self.credit.servicing_policy.validate()?;
        let mut accrued = self.credit.loans.clone();
        for loan in &mut accrued {
            loan.accrue_to(self.month)?;
        }
        let mut due_by_account = BTreeMap::<Account, f64>::new();
        for loan in &accrued {
            if loan.status == Status::Arrears {
                *due_by_account.entry(loan.terms.borrower).or_default() += loan.total_due();
            }
        }
        ensure!(
            due_by_account.values().all(|x| x.is_finite()),
            "collection total overflow"
        );
        let policy = &self.credit.servicing_policy;
        let mut plans = Vec::new();
        for loan in &accrued {
            if loan.status != Status::Arrears {
                continue;
            }
            let borrower = loan.terms.borrower;
            let cash = self.settlement_balance(borrower).ok().map(|b| b.value());
            let available = cash.is_some() && self.settlement_balance(loan.terms.lender).is_ok();
            let reserve = policy
                .protected_cash
                .iter()
                .find(|(a, _)| *a == borrower)
                .map_or(0., |(_, v)| *v);
            let budget = (cash.unwrap_or(0.) - reserve).max(0.) * policy.available_cash_share;
            let due = loan.total_due();
            let allowance = if available {
                due * (budget / due_by_account[&borrower]).min(1.)
            } else {
                0.
            };
            plans.push(Receipt {
                month: self.month,
                loan: loan.id,
                due,
                opening_cash: cash,
                protected_cash: reserve,
                allowance,
                paid: 0.,
                accounts_available: available,
                defaulted: false,
                precision_settled: 0.,
                precision_blocked: false,
            });
        }
        self.credit.loans = accrued;
        // Receipts are committed one by one. Unexpected transfer failures surface
        // to the caller; they must not be converted into successful collection.
        for mut receipt in plans {
            if receipt.allowance > 0. {
                receipt.paid = self.pay_credit_loan(receipt.loan, receipt.allowance)?;
            }
            let current = &self.credit.loans[receipt.loan as usize];
            let remainder = current.total_due();
            if current.status == Status::Arrears
                && remainder > 0.
                && remainder <= (receipt.allowance - receipt.paid).max(0.)
            {
                let from = self.settlement_balance(current.terms.borrower)?;
                let to = self.settlement_balance(current.terms.lender)?;
                if from.value() >= remainder {
                    let (_, _, transferable) = super::accounts::quote(from, to, remainder)?;
                    if transferable == 0. {
                        if current.precision_residue() {
                            receipt.precision_settled = self.credit.loans[receipt.loan as usize]
                                .settle_precision_residue(self.month)?;
                        } else {
                            // Keep the claim and retry. Transfer precision is not
                            // evidence of insolvency and does not relax write-off caps.
                            receipt.precision_blocked = true;
                        }
                    }
                }
            }
            let loan = &mut self.credit.loans[receipt.loan as usize];
            if loan.status == Status::Arrears
                && !receipt.precision_blocked
                && self.month.saturating_sub(loan.terms.maturity_month) >= loan.terms.grace_months
            {
                loan.write_off(self.month)?;
                receipt.defaulted = true;
            }
            self.credit.service_receipts.push(receipt);
        }
        self.credit.serviced_month = Some(self.month);
        Ok(())
    }
}
