//! Explicit post-default settlements. A recovery does not reopen written-off debt.
use super::{accounts::Transfer, EntryKind, Status};
use crate::civilization::History;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// Automatic callers occupy disjoint ranges of the shared recovery receipt ID space.
pub(super) const ESTATE_REQUEST_NAMESPACE: u64 = 1 << 62;
pub(super) const EXPORT_REQUEST_NAMESPACE: u64 = 1 << 63;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Reason {
    DelayedProceeds,
    EstateSurplus,
    VoluntarySettlement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub month: u32,
    pub loan: u64,
    /// Already authorized spend from the original borrower's existing account.
    pub allowance: f64,
    pub reason: Reason,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub request: Request,
    pub transfer: Transfer,
    /// Original borrower, then dated creditor; balances bracket only this transfer.
    pub opening_cash: [f64; 2],
    pub closing_cash: [f64; 2],
}

impl History {
    fn written_off_balance(&self, id: u64) -> Result<(f64, f64, u32)> {
        let loan = self
            .credit
            .loans
            .get(id as usize)
            .filter(|l| l.id == id)
            .context("missing recovery loan")?;
        loan.validate()?;
        ensure!(
            loan.status == Status::Defaulted,
            "recovery requires a defaulted loan"
        );
        let mut losses = loan
            .entries
            .iter()
            .filter(|e| matches!(e.kind, EntryKind::WriteOff));
        let loss = losses.next().context("default missing write-off")?;
        ensure!(losses.next().is_none(), "multiple default write-offs");
        Ok((loss.principal, loss.interest, loss.month))
    }

    /// Voluntary/authorized repayment of a recorded loss. No new interest,
    /// reinstated debt, backdated payments or reset of default exclusion.
    pub fn recover_defaulted_credit(&mut self, request: Request) -> Result<f64> {
        self.validate_credit_recoveries()?;
        ensure!(
            request.month == self.month,
            "recovery requires current month"
        );
        ensure!(
            request.allowance.is_finite() && request.allowance >= 0.,
            "invalid recovery allowance"
        );
        ensure!(
            !self
                .credit
                .recoveries
                .iter()
                .any(|r| r.request.id == request.id),
            "recovery request already committed"
        );
        let (mut principal, mut interest, default_month) =
            self.written_off_balance(request.loan)?;
        ensure!(self.month >= default_month, "recovery precedes default");
        for r in self
            .credit
            .recoveries
            .iter()
            .filter(|r| r.request.loan == request.loan)
        {
            principal -= r.transfer.principal;
            interest -= r.transfer.interest;
        }
        // Same interest-first split as the original contract; limits are the
        // remaining recorded loss, not a restored interest-bearing liability.
        let pay_interest = request.allowance.min(interest);
        let pay_principal = (request.allowance - pay_interest).min(principal);
        let terms = self.credit.loans[request.loan as usize].terms.clone();
        let creditor = self
            .credit
            .ownership
            .owner_at(&self.credit.loans[request.loan as usize], self.month)?;
        let opening_cash = [
            self.settlement_balance(terms.borrower)?.value(),
            self.settlement_balance(creditor)?.value(),
        ];
        let transfer = self.transfer_credit_cash(
            terms.borrower,
            creditor,
            terms.currency,
            pay_principal,
            pay_interest,
        )?;
        let actual = transfer.amount();
        let closing_cash = [
            self.settlement_balance(terms.borrower)
                .expect("preflighted recovery borrower")
                .value(),
            self.settlement_balance(creditor)
                .expect("preflighted recovery lender")
                .value(),
        ];
        self.credit.recoveries.push(Receipt {
            request,
            transfer,
            opening_cash,
            closing_cash,
        });
        if actual > 0. {
            let r = self.credit.recoveries.last().unwrap();
            self.record_credit_event(r.request.loan, "loan_recovery", format!("Loan {} recovered principal {:.6} and interest {:.6} in currency {} through {:?}; the original default remains recorded.", r.request.loan, r.transfer.principal, r.transfer.interest, r.transfer.currency.0, r.request.reason));
        }
        Ok(actual)
    }

    pub(crate) fn validate_credit_recoveries(&self) -> Result<()> {
        let mut ids = BTreeSet::new();
        let mut remaining = std::collections::BTreeMap::new();
        for r in &self.credit.recoveries {
            let q = &r.request;
            let t = &r.transfer;
            ensure!(ids.insert(q.id), "duplicate recovery request");
            let (p, i, month) = self.written_off_balance(q.loan)?;
            let balance = remaining.entry(q.loan).or_insert((p, i, month));
            let loan = &self.credit.loans[q.loan as usize];
            ensure!(
                q.month >= balance.2
                    && q.month <= self.month
                    && t.month == q.month
                    && t.from == loan.terms.borrower
                    && t.to == self.credit.ownership.owner_at(loan, q.month)?
                    && t.currency == loan.terms.currency
                    && [q.allowance, t.requested, t.principal, t.interest]
                        .iter()
                        .all(|v| v.is_finite() && *v >= 0.)
                    && t.amount() <= t.requested
                    && t.requested <= q.allowance
                    && t.principal <= balance.0
                    && t.interest <= balance.1,
                "invalid default recovery receipt"
            );
            ensure!(
                r.opening_cash
                    .iter()
                    .chain(&r.closing_cash)
                    .all(|v| v.is_finite() && *v >= 0.)
                    && r.opening_cash[0] - r.closing_cash[0] == t.amount()
                    && r.closing_cash[1] - r.opening_cash[1] == t.amount(),
                "recovery account deltas disagree with transfer"
            );
            // Verify the split using the actual transfer, including partial cash
            // and representability shortfalls, before reducing the loss ledger.
            let interest = t.amount().min(balance.1);
            ensure!(
                t.interest == interest && t.principal == t.amount() - interest,
                "recovery disagrees with interest-first allocation"
            );
            balance.0 -= t.principal;
            balance.1 -= t.interest;
            balance.2 = q.month;
        }
        Ok(())
    }
}
