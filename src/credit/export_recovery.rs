//! Opt-in use of newly received late export cash after ordinary Open collection.
use super::{recovery, Account, RepaymentSource, Status};
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const RECOVERY_PROCEEDS_SHARE: f64 = 0.25;
const RECOVERY_OPERATING_CASH_FLOOR: f64 = 100.;
const RECOVERY_REQUEST_NAMESPACE: u64 = 1 << 63;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    pub enabled: bool,
    pub proceeds_share: f64,
    pub operating_cash_floor: f64,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            enabled: false,
            proceeds_share: RECOVERY_PROCEEDS_SHARE,
            operating_cash_floor: RECOVERY_OPERATING_CASH_FLOOR,
        }
    }
}
impl Policy {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.proceeds_share.is_finite()
                && (0. ..=1.).contains(&self.proceeds_share)
                && self.operating_cash_floor.is_finite()
                && self.operating_cash_floor >= 0.,
            "invalid export recovery policy"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub policy: Policy,
    pub observed_month: Option<u32>,
    /// Cumulative seller cash at the last observation, keyed by stable payment ID.
    pub observed_paid: Vec<f64>,
}

impl History {
    /// Once in Open after live-debt servicing. Disabled observations still advance,
    /// so enabling this policy cannot collect historical proceeds retrospectively.
    pub fn recover_delayed_exports_month(&mut self) -> Result<()> {
        let state = &self.credit.export_recovery;
        let policy = state.policy.clone();
        policy.validate()?;
        if state.observed_month == Some(self.month) {
            return Ok(());
        }
        ensure!(
            state.observed_month.is_none_or(|m| m < self.month),
            "export recovery moved backwards"
        );
        self.validate_export_payments()?;
        let paid: Vec<_> = self.export_payments.iter().map(|p| p.seller_paid).collect();
        let mut proceeds = BTreeMap::<(Account, RepaymentSource), f64>::new();
        if state.observed_month.is_some() && policy.enabled {
            for (index, payment) in self.export_payments.iter().enumerate() {
                let previous = state.observed_paid.get(index).copied().unwrap_or(0.);
                ensure!(
                    payment.seller_paid >= previous,
                    "export payment moved backwards"
                );
                if payment.seller_paid > previous {
                    let source = RepaymentSource::Export {
                        contract: payment.contract,
                        payment_month: payment.expected_month,
                    };
                    *proceeds
                        .entry((Account::Town(payment.seller), source))
                        .or_default() += payment.seller_paid - previous;
                }
            }
        }
        ensure!(
            proceeds.values().all(|v| v.is_finite() && *v >= 0.),
            "export recovery proceeds overflow"
        );
        // First observation is a declared baseline for old archives, not a windfall.
        if proceeds.is_empty() {
            self.credit.export_recovery.observed_paid = paid;
            self.credit.export_recovery.observed_month = Some(self.month);
            return Ok(());
        }
        self.validate_credit_recoveries()?;
        self.credit.servicing_policy.validate()?;
        let costs = self.commercial_input_costs();
        let mut claims = BTreeMap::new();
        let mut eligible = Vec::new();
        for loan in &self.credit.loans {
            let key = (loan.terms.borrower, loan.terms.source);
            if loan.status != Status::Defaulted
                || !proceeds.contains_key(&key)
                || loan.accrued_through_month >= self.month
            {
                continue;
            }
            let Account::Town(site) = loan.terms.borrower else {
                continue;
            };
            if self.sites[site as usize].abandoned {
                continue;
            }
            let mut loss = loan
                .entries
                .iter()
                .filter(|e| matches!(e.kind, super::EntryKind::WriteOff))
                .map(|e| e.principal + e.interest)
                .sum::<f64>();
            for r in self
                .credit
                .recoveries
                .iter()
                .filter(|r| r.request.loan == loan.id)
            {
                loss -= r.transfer.amount();
            }
            if loss > 0. {
                *claims.entry(key).or_insert(0.) += loss;
                eligible.push((loan.id, key, loss));
            }
        }
        let mut proposed = Vec::new();
        let mut demand = BTreeMap::<Account, f64>::new();
        for (loan, key, loss) in eligible {
            let allowance = loss * (proceeds[&key] * policy.proceeds_share / claims[&key]).min(1.);
            *demand.entry(key.0).or_default() += allowance;
            proposed.push((loan, key.0, allowance));
        }
        let mut cash = BTreeMap::new();
        for &account in demand.keys() {
            let Account::Town(site) = account else {
                unreachable!()
            };
            let explicit = self
                .credit
                .servicing_policy
                .protected_cash
                .iter()
                .find(|(a, _)| *a == account)
                .map_or(0., |(_, amount)| *amount);
            let reserve = costs[site as usize]
                .max(policy.operating_cash_floor)
                .max(explicit);
            let live_debt: f64 = self
                .credit
                .loans
                .iter()
                .filter(|l| {
                    l.terms.borrower == account
                        && matches!(l.status, Status::Performing | Status::Arrears)
                })
                .map(|l| l.total_due())
                .sum();
            let available =
                (self.settlement_balance(account)?.value() - reserve - live_debt).max(0.);
            ensure!(
                reserve.is_finite() && live_debt.is_finite() && available.is_finite(),
                "invalid export recovery cash boundary"
            );
            cash.insert(account, available);
        }
        ensure!(
            !self.credit.recoveries.iter().any(
                |r| r.request.month == self.month && r.request.id >= RECOVERY_REQUEST_NAMESPACE
            ),
            "incomplete automatic recovery batch"
        );
        // All allowances use the same snapshot. Incoming recovery payments cannot
        // fund another proposal merely because its borrower sorts later.
        for (loan, borrower, allowance) in proposed {
            let allowance =
                allowance * (cash[&borrower] / demand[&borrower].max(f64::MIN_POSITIVE)).min(1.);
            if allowance <= 0.
                || self
                    .settlement_balance(
                        self.credit
                            .ownership
                            .owner_at(&self.credit.loans[loan as usize], self.month)?,
                    )
                    .is_err()
            {
                continue;
            }
            let index = self.credit.recoveries.len() as u64;
            ensure!(
                index < RECOVERY_REQUEST_NAMESPACE,
                "recovery namespace exhausted"
            );
            self.recover_defaulted_credit(recovery::Request {
                id: RECOVERY_REQUEST_NAMESPACE | index,
                month: self.month,
                loan,
                allowance,
                reason: recovery::Reason::DelayedProceeds,
            })?;
        }
        self.credit.export_recovery.observed_paid = paid;
        self.credit.export_recovery.observed_month = Some(self.month);
        Ok(())
    }

    pub(crate) fn validate_export_recovery(&self) -> Result<()> {
        let s = &self.credit.export_recovery;
        s.policy.validate()?;
        ensure!(
            s.observed_month.is_some() || s.observed_paid.is_empty(),
            "uninitialized recovery has payment observations"
        );
        ensure!(
            s.observed_month.is_none_or(|m| m <= self.month)
                && s.observed_paid.len() <= self.export_payments.len()
                && s.observed_paid
                    .iter()
                    .enumerate()
                    .all(|(i, v)| v.is_finite()
                        && *v >= 0.
                        && *v <= self.export_payments[i].seller_paid),
            "invalid export recovery observation"
        );
        Ok(())
    }
}
