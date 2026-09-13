//! Retained closed-account cash, early repayment and residual distributions.
use super::{Account, Status};
use crate::civilization::History;
use anyhow::{ensure, Result};
use std::collections::BTreeMap;

impl super::state::Credit {
    pub(crate) fn operator_has_debt(&self, id: u32) -> bool {
        self.account_has_debt(Account::Operator(id))
    }

    fn account_has_debt(&self, account: Account) -> bool {
        self.loans.iter().any(|loan| {
            loan.terms.borrower == account
                && matches!(loan.status, Status::Performing | Status::Arrears)
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
            return Ok(());
        }
        let mut loans = self.credit.loans.clone();
        let mut claims = BTreeMap::<Account, f64>::new();
        for loan in &mut loans {
            if closed.contains_key(&loan.terms.borrower) {
                loan.accrue_to(self.month)?;
                if matches!(loan.status, Status::Performing | Status::Arrears) {
                    *claims.entry(loan.terms.borrower).or_default() += loan.total_due();
                }
            }
        }
        ensure!(
            claims.values().all(|v| v.is_finite()),
            "estate claims overflow"
        );
        let mut plans = Vec::new();
        for loan in &loans {
            if let Some(cash) = closed.get(&loan.terms.borrower) {
                if matches!(loan.status, Status::Performing | Status::Arrears)
                    && claims[&loan.terms.borrower] > 0.
                    && self
                        .settlement_balance(self.credit.ownership.owner_at(loan, self.month)?)
                        .is_ok()
                {
                    plans.push((
                        loan.id,
                        loan.total_due() * (cash / claims[&loan.terms.borrower]).min(1.),
                    ));
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
        Ok(())
    }
}
