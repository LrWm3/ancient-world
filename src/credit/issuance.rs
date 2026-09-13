//! Dated shared-currency experiment. Issuance is a declared external money source.
use super::{
    accounts::{quote, Balance},
    CurrencyId, SHARED_CURRENCY,
};
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const ISSUE_AMOUNT: f64 = 25.;
const PER_ISSUE_CAP: f64 = 25.;
const ROLLING_ANNUAL_CAP: f64 = 100.;
const LIFETIME_CAP: f64 = 250.;
const COOLDOWN_MONTHS: u32 = 12;
const INTERVAL_MONTHS: u32 = 12;
const AUTHORIZATION_DURATION_MONTHS: u32 = 120;
const ANNUAL_WINDOW_MONTHS: u32 = 12;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Limits {
    pub per_issue: f64,
    pub annual: f64,
    pub lifetime: f64,
    pub cooldown_months: u32,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            per_issue: PER_ISSUE_CAP,
            annual: ROLLING_ANNUAL_CAP,
            lifetime: LIFETIME_CAP,
            cooldown_months: COOLDOWN_MONTHS,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub issuers: Vec<u32>,
    pub authorized_month: u32,
    pub start_month: u32,
    pub end_month: u32,
    pub interval_months: u32,
    pub amount: f64,
    pub limits: Limits,
}
impl Schedule {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.issuers.is_empty()
                && self.issuers.iter().collect::<BTreeSet<_>>().len() == self.issuers.len(),
            "invalid authorized issuers"
        );
        ensure!(
            self.start_month > self.authorized_month
                && self.end_month >= self.start_month
                && self.interval_months > 0,
            "invalid issuance dates"
        );
        ensure!(
            [
                self.amount,
                self.limits.per_issue,
                self.limits.annual,
                self.limits.lifetime
            ]
            .iter()
            .all(|x| x.is_finite() && *x >= 0.),
            "invalid issuance limits"
        );
        Ok(())
    }
    fn due(&self, month: u32) -> bool {
        month >= self.start_month
            && month <= self.end_month
            && (month - self.start_month) % self.interval_months == 0
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub month: u32,
    pub issuer: u32,
    pub leader: Option<u32>,
    pub currency: CurrencyId,
    pub requested: f64,
    pub issued: f64,
    pub opening_treasury: f64,
    pub closing_treasury: f64,
    pub annual_before: f64,
    pub lifetime_before: f64,
    pub permitted: f64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Issuance {
    pub enabled: bool,
    pub schedule: Option<Schedule>,
    pub processed_month: Option<u32>,
    pub receipts: Vec<Receipt>,
}
impl Issuance {
    pub fn total_issued(&self) -> f64 {
        self.receipts.iter().map(|r| r.issued).sum()
    }
    fn allowance(&self, issuer: u32, month: u32, schedule: &Schedule) -> (f64, f64, f64) {
        let previous = self
            .receipts
            .iter()
            .filter(|r| r.issuer == issuer && r.month < month);
        let lifetime: f64 = previous.clone().map(|r| r.issued).sum();
        let annual: f64 = previous
            .clone()
            .filter(|r| month - r.month < ANNUAL_WINDOW_MONTHS)
            .map(|r| r.issued)
            .sum();
        let cooling = previous
            .filter(|r| r.issued > 0.)
            .any(|r| month - r.month < schedule.limits.cooldown_months);
        let permitted = if cooling {
            0.
        } else {
            schedule
                .amount
                .min(schedule.limits.per_issue)
                .min((schedule.limits.annual - annual).max(0.))
                .min((schedule.limits.lifetime - lifetime).max(0.))
        };
        (annual, lifetime, permitted)
    }
    pub fn validate(&self, month: u32) -> Result<()> {
        ensure!(
            self.processed_month.is_none_or(|m| m <= month),
            "future issuance boundary"
        );
        let Some(schedule) = &self.schedule else {
            ensure!(
                self.receipts.is_empty() && !self.enabled,
                "issuance lacks authorization"
            );
            return Ok(());
        };
        schedule.validate()?;
        ensure!(
            schedule.authorized_month <= month,
            "future issuance authorization"
        );
        ensure!(self.total_issued().is_finite(), "nonfinite issued supply");
        let mut dates = BTreeSet::new();
        for r in &self.receipts {
            let (annual, lifetime, permitted) = self.allowance(r.issuer, r.month, schedule);
            ensure!(
                schedule.issuers.contains(&r.issuer)
                    && r.currency == SHARED_CURRENCY
                    && r.month <= month
                    && schedule.due(r.month)
                    && self.processed_month.is_some_and(|m| m >= r.month)
                    && dates.insert((r.month, r.issuer))
                    && r.requested == schedule.amount
                    && r.annual_before == annual
                    && r.lifetime_before == lifetime
                    && r.permitted == permitted
                    && r.issued <= permitted
                    && [r.issued, r.opening_treasury, r.closing_treasury]
                        .iter()
                        .all(|x| x.is_finite() && *x >= 0.)
                    && r.closing_treasury - r.opening_treasury == r.issued,
                "invalid issuance receipt"
            );
        }
        Ok(())
    }
}
impl History {
    /// First activation establishes the finite experiment window. Re-enabling
    /// cannot renew that window, erase previous issuance, or bank missed allowances.
    pub fn configure_shared_issuance(&mut self, enabled: bool) -> Result<()> {
        if enabled && self.credit.issuance.schedule.is_none() {
            let issuers: Vec<_> = self
                .society
                .as_ref()
                .map(|s| s.councils.iter().map(|c| c.civilization).collect())
                .unwrap_or_default();
            ensure!(!issuers.is_empty(), "shared issuance requires councils");
            let start = self
                .month
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("issuance date overflow"))?;
            let end = self
                .month
                .checked_add(AUTHORIZATION_DURATION_MONTHS)
                .ok_or_else(|| anyhow::anyhow!("issuance date overflow"))?;
            self.credit.issuance.schedule = Some(Schedule {
                issuers,
                authorized_month: self.month,
                start_month: start,
                end_month: end,
                interval_months: INTERVAL_MONTHS,
                amount: ISSUE_AMOUNT,
                limits: Default::default(),
            });
        }
        self.credit.issuance.enabled = enabled;
        Ok(())
    }
    /// Open, before ordinary treasury obligations. This does not change policy
    /// spending shares, forgive debt, issue a loan, or create goods and labor.
    pub fn shared_issuance_month(&mut self) -> Result<()> {
        self.credit.issuance.validate(self.month)?;
        if self.credit.issuance.processed_month == Some(self.month) {
            return Ok(());
        }
        ensure!(
            self.credit
                .issuance
                .processed_month
                .is_none_or(|m| m < self.month),
            "issuance moved backwards"
        );
        let due = self.credit.issuance.enabled
            && self
                .credit
                .issuance
                .schedule
                .as_ref()
                .is_some_and(|s| s.due(self.month));
        if due {
            let schedule = self.credit.issuance.schedule.as_ref().unwrap().clone();
            // Preflight every treasury before committing any issuer this month.
            let mut batch = Vec::new();
            if let Some(society) = &self.society {
                for (index, council) in society.councils.iter().enumerate() {
                    let issuer = council.civilization;
                    ensure!(
                        issuer as usize == index,
                        "invalid issuance council identity"
                    );
                    if !schedule.issuers.contains(&issuer) {
                        continue;
                    }
                    let Some(civilization) = self
                        .civilizations
                        .get(issuer as usize)
                        .filter(|c| c.id == issuer)
                    else {
                        continue;
                    };
                    let (annual, lifetime, permitted) = self
                        .credit
                        .issuance
                        .allowance(issuer, self.month, &schedule);
                    let (_, closing, issued) = quote(
                        Balance::Double(permitted),
                        Balance::Double(council.treasury),
                        permitted,
                    )?;
                    batch.push(Receipt {
                        month: self.month,
                        issuer,
                        leader: self
                            .people
                            .get(civilization.leader as usize)
                            .filter(|p| p.id == civilization.leader)
                            .map(|p| p.id),
                        currency: SHARED_CURRENCY,
                        requested: schedule.amount,
                        issued,
                        opening_treasury: council.treasury,
                        closing_treasury: closing.value(),
                        annual_before: annual,
                        lifetime_before: lifetime,
                        permitted,
                    });
                }
            }
            ensure!(
                (self.credit.issuance.total_issued() + batch.iter().map(|r| r.issued).sum::<f64>())
                    .is_finite(),
                "issued supply overflow"
            );
            for receipt in batch {
                self.society.as_mut().unwrap().councils[receipt.issuer as usize].treasury =
                    receipt.closing_treasury;
                if receipt.issued > 0. {
                    let site = self
                        .sites
                        .iter()
                        .find(|s| s.civilization == receipt.issuer && !s.abandoned)
                        .map(|s| s.id);
                    self.event("currency_issued", site, None, format!("Council {} issued {:.2} shared currency into its treasury under the authorization dated month {}", receipt.issuer, receipt.issued, schedule.authorized_month));
                }
                self.credit.issuance.receipts.push(receipt);
            }
        }
        self.credit.issuance.processed_month = Some(self.month);
        Ok(())
    }
}
