//! Retained operator cash, early repayment and residual owner distributions.
use super::{Account, Status};
use crate::civilization::History;
use anyhow::{ensure, Result};
use std::collections::BTreeMap;

impl super::state::Credit {
    pub(crate) fn operator_has_debt(&self, id: u32) -> bool {
        self.loans.iter().any(|loan| {
            loan.terms.borrower == Account::Operator(id)
                && matches!(loan.status, Status::Performing | Status::Arrears)
        })
    }
}

impl History {
    /// Called after Open collection and after each operator closure window.
    /// Equal-ranking claims share an opening cash snapshot, including unmatured
    /// loans whose existing terms permit early repayment. Unpaid claims retain
    /// their original maturity/default rules; closure never fabricates a loss.
    pub(crate) fn settle_operator_estates(&mut self) -> Result<()> {
        let Some(enterprises) = &self.enterprises else {
            return Ok(());
        };
        let closed: BTreeMap<_, _> = enterprises
            .firms
            .iter()
            .filter(|f| f.closed.is_some() && f.cash > 0.)
            .map(|f| (f.id, f.cash))
            .collect();
        if closed.is_empty() {
            return Ok(());
        }
        let mut loans = self.credit.loans.clone();
        let mut claims = BTreeMap::<u32, f64>::new();
        for loan in &mut loans {
            if let Account::Operator(id) = loan.terms.borrower {
                if closed.contains_key(&id) {
                    loan.accrue_to(self.month)?;
                    if matches!(loan.status, Status::Performing | Status::Arrears) {
                        *claims.entry(id).or_default() += loan.total_due();
                    }
                }
            }
        }
        ensure!(
            claims.values().all(|v| v.is_finite()),
            "estate claims overflow"
        );
        let mut plans = Vec::new();
        for loan in &loans {
            if let Account::Operator(id) = loan.terms.borrower {
                if let Some(cash) = closed.get(&id) {
                    if matches!(loan.status, Status::Performing | Status::Arrears)
                        && claims[&id] > 0.
                        && self.settlement_balance(loan.terms.lender).is_ok()
                    {
                        plans.push((loan.id, loan.total_due() * (cash / claims[&id]).min(1.)));
                    }
                }
            }
        }
        self.credit.loans = loans;
        for (id, allowance) in plans {
            if allowance > 0. {
                self.pay_credit_loan(id, allowance)?;
            }
        }
        // Incoming repayments to closed creditors remain their cash until here.
        // Do not distribute it if the same estate still owes any live claim.
        let releasable: Vec<_> = closed
            .keys()
            .copied()
            .filter(|id| !self.credit.operator_has_debt(*id))
            .collect();
        for id in releasable {
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
        Ok(())
    }
}
