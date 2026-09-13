//! Retained closed-account cash, early repayment and residual distributions.
use super::{Account, Status};
use crate::civilization::History;
use anyhow::{ensure, Result};
use std::collections::BTreeMap;

const ESTATE_REQUEST_NAMESPACE: u64 = 1 << 63;

impl super::state::Credit {
    /// Preserve default losses as a priority claim for estate asset distributions,
    /// even though servicing no longer treats them as interest-bearing debt.
    fn unrecovered_credit_loss(&self, loan: &super::Loan) -> f64 {
        let loss: f64 = loan
            .entries
            .iter()
            .filter(|e| matches!(e.kind, super::EntryKind::WriteOff))
            .map(|e| e.principal + e.interest)
            .sum();
        let recovered: f64 = self
            .recoveries
            .iter()
            .filter(|r| r.request.loan == loan.id)
            .map(|r| r.transfer.amount())
            .sum();
        (loss - recovered).max(0.)
    }

    pub(crate) fn operator_has_debt(&self, id: u32) -> bool {
        self.account_has_debt(Account::Operator(id))
    }

    pub(super) fn account_has_debt(&self, account: Account) -> bool {
        self.loans.iter().any(|loan| {
            loan.terms.borrower == account
                && (matches!(loan.status, Status::Performing | Status::Arrears)
                    || self.unrecovered_credit_loss(loan) > 0.)
        })
    }
}

impl History {
    /// Called around Open collection and after operator/cultural closure windows.
    /// Equal-ranking claims share an opening cash snapshot, including unmatured
    /// loans whose existing terms permit early repayment. Unpaid claims retain
    /// their original maturity/default rules; closure never fabricates a loss.
    pub(crate) fn settle_credit_estates(&mut self) -> Result<()> {
        let mut closed = BTreeMap::<Account, f64>::new();
        if let Some(enterprises) = &self.enterprises {
            closed.extend(
                enterprises
                    .firms
                    .iter()
                    .filter(|f| f.closed.is_some() && f.cash > 0.)
                    .map(|f| (Account::Operator(f.id), f.cash)),
            );
        }
        if let Some(culture) = &self.culture {
            let traveling: std::collections::BTreeSet<_> = culture
                .relocations
                .iter()
                .filter(|m| m.arrived.is_none())
                .map(|m| m.institution)
                .collect();
            closed.extend(
                culture
                    .institutions
                    .iter()
                    .filter(|n| !n.active && n.treasury > 0. && !traveling.contains(&n.id))
                    .map(|n| (Account::Institution(n.id), n.treasury)),
            );
        }
        if closed.is_empty() {
            return self.schedule_credit_estate_claims();
        }
        let mut loans = self.credit.loans.clone();
        let mut claims = BTreeMap::<Account, f64>::new();
        for loan in &mut loans {
            if closed.contains_key(&loan.terms.borrower) {
                loan.accrue_to(self.month)?;
                let due = if loan.status == Status::Defaulted {
                    self.credit.unrecovered_credit_loss(loan)
                } else {
                    loan.total_due()
                };
                *claims.entry(loan.terms.borrower).or_default() += due;
            }
        }
        ensure!(
            claims.values().all(|v| v.is_finite()),
            "estate claims overflow"
        );
        let mut plans = Vec::new();
        for loan in &loans {
            if let Some(cash) = closed.get(&loan.terms.borrower) {
                let due = if loan.status == Status::Defaulted {
                    self.credit.unrecovered_credit_loss(loan)
                } else {
                    loan.total_due()
                };
                if due > 0.
                    && claims[&loan.terms.borrower] > 0.
                    && self
                        .settlement_balance(self.credit.ownership.owner_at(loan, self.month)?)
                        .is_ok()
                {
                    plans.push((
                        loan.id,
                        due * (cash / claims[&loan.terms.borrower]).min(1.),
                        loan.status == Status::Defaulted,
                    ));
                }
            }
        }
        self.credit.loans = loans;
        for (id, allowance, defaulted) in plans {
            if allowance > 0. {
                if defaulted {
                    self.recover_estate_credit(id, allowance)?;
                } else {
                    self.pay_credit_loan(id, allowance)?;
                }
            }
        }
        // Incoming repayments to closed creditors remain their cash until here.
        // Do not distribute it if the same estate still owes a live claim or an unrecovered default loss.
        let releasable: Vec<_> = closed
            .keys()
            .copied()
            .filter(|account| !self.credit.account_has_debt(*account))
            .collect();
        for account in releasable {
            if let Account::Institution(id) = account {
                let n = &self.culture.as_ref().unwrap().institutions[id as usize];
                let (site, cash, expenses) = (n.site, n.treasury, n.expenses);
                ensure!(
                    (expenses + cash).is_finite(),
                    "institution estate distribution overflow"
                );
                let actual = self
                    .transfer_credit_cash(
                        account,
                        Account::Town(site),
                        super::SHARED_CURRENCY,
                        cash,
                        0.,
                    )?
                    .amount();
                self.culture.as_mut().unwrap().institutions[id as usize].expenses += actual;
                // A precision-blocked remainder stays in the original treasury.
                continue;
            }
            let Account::Operator(id) = account else {
                unreachable!("closed account kind");
            };
            let f = &self.enterprises.as_ref().unwrap().firms[id as usize];
            if f.cash == 0. {
                continue;
            }
            let owner = f.owner as usize;
            let cash = f.cash;
            let a = self
                .society
                .as_ref()
                .and_then(|s| s.household_economy.as_ref())
                .and_then(|e| e.accounts.get(owner))
                .ok_or_else(|| anyhow::anyhow!("missing operator estate beneficiary"))?;
            ensure!(
                (a.cash + cash).is_finite()
                    && (a.capital_returned + cash).is_finite()
                    && (f.liquidation + cash).is_finite(),
                "estate distribution overflow"
            );
            let a = &mut self
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .accounts[owner];
            a.cash += cash;
            a.capital_returned += cash;
            let f = &mut self.enterprises.as_mut().unwrap().firms[id as usize];
            f.liquidation += cash;
            f.cash = 0.;
        }
        self.schedule_credit_estate_claims()
    }
}

impl History {
    fn recover_estate_credit(&mut self, loan: u64, allowance: f64) -> Result<()> {
        let contract = &self.credit.loans[loan as usize];
        let creditor = self.credit.ownership.owner_at(contract, self.month)?;
        let balances = [
            self.settlement_balance(contract.terms.borrower)?.value(),
            self.settlement_balance(creditor)?.value(),
        ];
        // A precision-blocked attempt need not append another zero receipt at
        // every estate window. New cash, recipient balances or a new month may
        // make the same remaining claim transferable, so allow those retries.
        if self.credit.recoveries.iter().rev().any(|r| {
            r.request.loan == loan
                && r.request.month == self.month
                && r.request.allowance == allowance
                && matches!(r.request.reason, super::recovery::Reason::EstateSurplus)
                && r.transfer.to == creditor
                && r.transfer.amount() == 0.
                && r.closing_cash == balances
        }) {
            return Ok(());
        }
        let mut id = super::recovery::ESTATE_REQUEST_NAMESPACE
            .checked_add(self.credit.recoveries.len() as u64)
            .ok_or_else(|| anyhow::anyhow!("estate recovery IDs exhausted"))?;
        while self.credit.recoveries.iter().any(|r| r.request.id == id) {
            id = id
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("estate recovery IDs exhausted"))?;
        }
        ensure!(
            id < super::recovery::EXPORT_REQUEST_NAMESPACE,
            "estate recovery IDs exhausted"
        );
        self.recover_defaulted_credit(super::recovery::Request {
            id,
            month: self.month,
            loan,
            allowance,
            reason: super::recovery::Reason::EstateSurplus,
        })?;
        Ok(())
    }

    fn schedule_credit_estate_claims(&mut self) -> Result<()> {
        use super::ownership::{Basis, Request};
        let mut plans = Vec::new();
        for loan in &self.credit.loans {
            if !(matches!(loan.status, Status::Performing | Status::Arrears)
                || loan.status == Status::Defaulted
                    && self.credit.unrecovered_credit_loss(loan) > 0.)
            {
                continue;
            }
            if self
                .credit
                .ownership
                .assignments()
                .iter()
                .any(|a| a.request.loan == loan.id && a.effective_month > self.month)
            {
                continue;
            }
            let from = self.credit.ownership.owner_at(loan, self.month)?;
            if self.credit.account_has_debt(from) {
                continue;
            }
            let (to, basis) = match from {
                Account::Operator(id) => {
                    let Some(f) = self
                        .enterprises
                        .as_ref()
                        .and_then(|e| e.firms.get(id as usize))
                    else {
                        continue;
                    };
                    let Some(closed_month) = f.closed else {
                        continue;
                    };
                    if self
                        .society
                        .as_ref()
                        .is_none_or(|s| s.relocation.lost_households.contains(&f.owner))
                    {
                        continue;
                    }
                    (
                        Account::Household(f.owner),
                        Basis::OperatorEstate {
                            household: f.owner,
                            closed_month,
                        },
                    )
                }
                Account::Institution(id) => {
                    let Some(c) = &self.culture else {
                        continue;
                    };
                    let Some(n) = c.institutions.get(id as usize) else {
                        continue;
                    };
                    if n.active
                        || c.relocations
                            .iter()
                            .any(|r| r.institution == id && r.arrived.is_none())
                    {
                        continue;
                    }
                    (
                        Account::Town(n.site),
                        Basis::InstitutionEstate { site: n.site },
                    )
                }
                _ => continue,
            };
            // Assignment is not forgiveness; a debtor inheriting its own claim
            // needs a separately recorded cancellation rule, so retain it here.
            if to == loan.terms.borrower {
                continue;
            }
            let Ok(balance) = self.settlement_balance(to) else {
                continue;
            };
            ensure!(
                balance.value().is_finite() && balance.value() >= 0.,
                "invalid estate successor cash"
            );
            plans.push((loan.id, from, to, basis));
        }
        for (loan, from, to, basis) in plans {
            let mut id = ESTATE_REQUEST_NAMESPACE;
            while self
                .credit
                .ownership
                .assignments()
                .iter()
                .any(|a| a.request.id == id)
            {
                id = id
                    .checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("estate request IDs exhausted"))?;
            }
            let request = Request {
                id,
                loan,
                month: self.month,
                from,
                to,
                owner_consent: None,
                recipient_consent: None,
                cause: None,
            };
            let sequence = self.credit.ownership.assign_with_basis(
                &self.credit.loans,
                self.month,
                request,
                basis,
            )?;
            let effective = self.credit.ownership.assignments()[sequence as usize].effective_month;
            self.record_credit_event(loan, "loan_assigned", format!(
                "Closed estate {from:?} assigned its creditor claim on loan {loan} to {to:?}, effective month {effective}; no cash transferred or borrower obligation changed."));
        }
        Ok(())
    }
}
