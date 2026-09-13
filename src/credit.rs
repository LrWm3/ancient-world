//! Contract accounting for transfers made by existing accounts.
//! This ledger holds claims, never spendable money. Account transfer and policy
//! adapters must commit cash first and pass the amount actually transferred.
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

pub const SHARED_CURRENCY: CurrencyId = CurrencyId(0);
const MONTHS_PER_YEAR: f64 = 12.;
const MAX_ANNUAL_SIMPLE_RATE: f64 = 1.;
const MAX_LOAN_TERM_MONTHS: u32 = 120;
const DEFAULT_GRACE_MONTHS: u32 = 3;
const LEDGER_RELATIVE_TOLERANCE: f64 = 1e-10;

pub mod accounts;
pub mod councils;
pub mod exports;
pub mod servicing;
pub mod state;
pub mod taxes;
pub mod underwriting;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CurrencyId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Account {
    Council(u32),
    Institution(u32),
    Town(u32),
    Operator(u32),
}

/// Stable payment provenance; an estimate is not a second cash balance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RepaymentSource {
    AnnualTax { council: u32, collection_month: u32 },
    Export { contract: u64, payment_month: u32 },
    ServiceOrder { order: u64, payment_month: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Terms {
    pub lender: Account,
    pub borrower: Account,
    pub currency: CurrencyId,
    pub source: RepaymentSource,
    pub annual_simple_rate: f64,
    pub maturity_month: u32,
    pub grace_months: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Performing,
    Arrears,
    Repaid,
    Defaulted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EntryKind {
    Disbursement,
    InterestAccrued,
    Repayment,
    WriteOff,
    Restructuring {
        previous_maturity: u32,
        revised_maturity: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub month: u32,
    pub kind: EntryKind,
    pub principal: f64,
    pub interest: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Loan {
    pub id: u64,
    pub terms: Terms,
    pub opened_month: u32,
    pub accrued_through_month: u32,
    pub original_principal: f64,
    pub outstanding_principal: f64,
    pub interest_due: f64,
    pub status: Status,
    pub restructured: bool,
    pub entries: Vec<Entry>,
}

impl Terms {
    pub fn validate(&self, month: u32) -> Result<()> {
        ensure!(
            self.lender != self.borrower,
            "self lending is not permitted"
        );
        ensure!(
            self.annual_simple_rate.is_finite()
                && (0. ..=MAX_ANNUAL_SIMPLE_RATE).contains(&self.annual_simple_rate),
            "invalid simple interest rate"
        );
        ensure!(
            self.maturity_month > month && self.maturity_month - month <= MAX_LOAN_TERM_MONTHS,
            "invalid loan maturity"
        );
        ensure!(
            self.grace_months <= MAX_LOAN_TERM_MONTHS,
            "invalid grace period"
        );
        Ok(())
    }

    pub fn default_grace_months() -> u32 {
        DEFAULT_GRACE_MONTHS
    }
}

impl Loan {
    /// Call only after a successful account transfer; no cash is minted here.
    pub fn record_disbursement(id: u64, terms: Terms, month: u32, actual: f64) -> Result<Self> {
        terms.validate(month)?;
        ensure!(actual.is_finite() && actual > 0., "invalid disbursement");
        Ok(Self {
            id,
            terms,
            opened_month: month,
            accrued_through_month: month,
            original_principal: actual,
            outstanding_principal: actual,
            interest_due: 0.,
            status: Status::Performing,
            restructured: false,
            entries: vec![Entry {
                month,
                kind: EntryKind::Disbursement,
                principal: actual,
                interest: 0.,
            }],
        })
    }

    /// Monthly stepping retains the same addition order for batched advancement.
    /// Arrears accrue ordinary simple interest, never interest on interest.
    pub fn accrue_to(&mut self, month: u32) -> Result<()> {
        ensure!(
            month >= self.accrued_through_month,
            "cannot accrue backwards"
        );
        if matches!(self.status, Status::Repaid | Status::Defaulted) {
            return Ok(());
        }
        let amount = self.outstanding_principal * self.terms.annual_simple_rate / MONTHS_PER_YEAR;
        let months = month - self.accrued_through_month;
        ensure!(
            (self.interest_due + amount * months as f64).is_finite(),
            "interest overflow"
        );
        while self.accrued_through_month < month {
            self.accrued_through_month += 1;
            self.interest_due += amount;
            if amount > 0. {
                self.entries.push(Entry {
                    month: self.accrued_through_month,
                    kind: EntryKind::InterestAccrued,
                    principal: 0.,
                    interest: amount,
                });
            }
        }
        self.refresh_status(month);
        Ok(())
    }

    pub fn total_due(&self) -> f64 {
        self.outstanding_principal + self.interest_due
    }

    /// Quote before transferring. The pilot pays accrued interest first and
    /// then principal; early repayment has no penalty.
    pub fn repayment_quote(&self, available: f64) -> Result<(f64, f64)> {
        ensure!(
            available.is_finite() && available >= 0.,
            "invalid payment funds"
        );
        ensure!(
            !matches!(self.status, Status::Repaid | Status::Defaulted),
            "loan is closed"
        );
        let interest = available.min(self.interest_due);
        let principal = (available - interest).min(self.outstanding_principal);
        Ok((principal, interest))
    }

    pub fn record_repayment(&mut self, month: u32, actual: f64) -> Result<()> {
        ensure!(
            month == self.accrued_through_month,
            "payment requires current accrual boundary"
        );
        ensure!(
            actual.is_finite() && actual > 0. && actual <= self.total_due(),
            "invalid repayment"
        );
        let (principal, interest) = self.repayment_quote(actual)?;
        self.outstanding_principal -= principal;
        self.interest_due -= interest;
        self.entries.push(Entry {
            month,
            kind: EntryKind::Repayment,
            principal,
            interest,
        });
        self.refresh_status(month);
        Ok(())
    }

    fn refresh_status(&mut self, month: u32) {
        self.status = if self.total_due() == 0. {
            Status::Repaid
        } else if month >= self.terms.maturity_month {
            Status::Arrears
        } else {
            Status::Performing
        };
    }

    /// Consent and supported revised receipts must be established by policy.
    /// Existing interest is not capitalized and the rate is unchanged.
    pub fn restructure(&mut self, month: u32, revised_maturity: u32) -> Result<()> {
        ensure!(
            month == self.accrued_through_month && self.status == Status::Arrears,
            "restructuring requires current arrears"
        );
        ensure!(!self.restructured, "only one restructuring is permitted");
        ensure!(
            revised_maturity > month && revised_maturity - month <= MAX_LOAN_TERM_MONTHS,
            "invalid revised maturity"
        );
        let previous_maturity = self.terms.maturity_month;
        self.terms.maturity_month = revised_maturity;
        self.restructured = true;
        self.status = Status::Performing;
        self.entries.push(Entry {
            month,
            kind: EntryKind::Restructuring {
                previous_maturity,
                revised_maturity,
            },
            principal: 0.,
            interest: 0.,
        });
        Ok(())
    }

    /// Write off claims and liabilities together. No account balance changes.
    pub fn write_off(&mut self, month: u32) -> Result<()> {
        ensure!(
            month == self.accrued_through_month && self.status == Status::Arrears,
            "write-off requires current arrears"
        );
        ensure!(
            month.saturating_sub(self.terms.maturity_month) >= self.terms.grace_months,
            "default grace period has not elapsed"
        );
        self.entries.push(Entry {
            month,
            kind: EntryKind::WriteOff,
            principal: self.outstanding_principal,
            interest: self.interest_due,
        });
        self.outstanding_principal = 0.;
        self.interest_due = 0.;
        self.status = Status::Defaulted;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.original_principal.is_finite()
                && self.original_principal > 0.
                && self.outstanding_principal.is_finite()
                && self.outstanding_principal >= 0.
                && self.interest_due.is_finite()
                && self.interest_due >= 0.,
            "invalid debt balance"
        );
        let mut principal = 0.;
        let mut interest = 0.;
        let mut last = self.opened_month;
        let mut origin_count = 0;
        for e in &self.entries {
            ensure!(
                e.month >= last
                    && e.month <= self.accrued_through_month
                    && e.principal.is_finite()
                    && e.principal >= 0.
                    && e.interest.is_finite()
                    && e.interest >= 0.,
                "invalid debt receipt"
            );
            last = e.month;
            match e.kind {
                EntryKind::Disbursement => {
                    origin_count += 1;
                    principal += e.principal;
                }
                EntryKind::InterestAccrued => interest += e.interest,
                EntryKind::Repayment | EntryKind::WriteOff => {
                    principal -= e.principal;
                    interest -= e.interest;
                }
                EntryKind::Restructuring { .. } => {}
            }
        }
        ensure!(
            origin_count == 1
                && self
                    .entries
                    .first()
                    .is_some_and(|e| matches!(e.kind, EntryKind::Disbursement)
                        && e.principal == self.original_principal),
            "missing or duplicate original financing"
        );
        let tolerance = LEDGER_RELATIVE_TOLERANCE * self.original_principal.max(1.);
        ensure!(
            (principal - self.outstanding_principal).abs() <= tolerance
                && (interest - self.interest_due).abs() <= tolerance,
            "debt ledger does not reconcile"
        );
        ensure!(
            matches!(self.status, Status::Repaid | Status::Defaulted) == (self.total_due() == 0.),
            "debt status does not match balance"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn loan() -> Loan {
        Loan::record_disbursement(
            0,
            Terms {
                lender: Account::Institution(4),
                borrower: Account::Council(2),
                currency: SHARED_CURRENCY,
                source: RepaymentSource::AnnualTax {
                    council: 2,
                    collection_month: 12,
                },
                annual_simple_rate: 0.12,
                maturity_month: 12,
                grace_months: 3,
            },
            0,
            100.,
        )
        .unwrap()
    }
    #[test]
    fn principal_is_financing_and_simple_interest_is_not_compounded() {
        let mut l = loan();
        l.accrue_to(12).unwrap();
        assert_eq!(l.outstanding_principal, 100.);
        assert_eq!(l.interest_due, 12.);
        assert_eq!(l.status, Status::Arrears);
        l.record_repayment(12, 32.).unwrap();
        assert_eq!(l.outstanding_principal, 80.);
        assert_eq!(l.interest_due, 0.);
        l.accrue_to(13).unwrap();
        assert!((l.interest_due - 0.8).abs() < 1e-12);
        l.validate().unwrap();
    }
    #[test]
    fn batch_and_serialized_continuation_do_not_double_accrue() {
        let mut batch = loan();
        batch.accrue_to(12).unwrap();
        let mut resumed = loan();
        resumed.accrue_to(6).unwrap();
        resumed = serde_json::from_str(&serde_json::to_string(&resumed).unwrap()).unwrap();
        for month in 6..=12 {
            resumed.accrue_to(month).unwrap();
        }
        assert_eq!(
            serde_json::to_value(batch).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
    }
    #[test]
    fn restructuring_does_not_capitalize_interest_or_repeat_forever() {
        let mut l = loan();
        l.accrue_to(12).unwrap();
        l.restructure(12, 24).unwrap();
        assert_eq!(l.outstanding_principal, 100.);
        assert_eq!(l.interest_due, 12.);
        l.accrue_to(24).unwrap();
        assert_eq!(l.interest_due, 24.);
        assert!(l.restructure(24, 36).is_err());
        assert!(l.write_off(24).is_err());
        l.accrue_to(27).unwrap();
        l.write_off(27).unwrap();
        assert_eq!(l.status, Status::Defaulted);
        assert_eq!(l.total_due(), 0.);
        l.validate().unwrap();
        assert!(l.write_off(27).is_err());
        assert!(l.record_repayment(27, 1.).is_err());
    }
    #[test]
    fn rejects_invalid_boundary_and_overpayment_without_mutation() {
        let mut l = loan();
        let before = serde_json::to_value(&l).unwrap();
        assert!(l.record_repayment(1, 1.).is_err());
        assert!(l.record_repayment(0, 101.).is_err());
        assert!(l.record_repayment(0, f64::NAN).is_err());
        assert_eq!(before, serde_json::to_value(&l).unwrap());
        l.record_repayment(0, 100.).unwrap();
        l.accrue_to(12).unwrap();
        assert_eq!(l.status, Status::Repaid);
        assert_eq!(l.interest_due, 0.);
        l.validate().unwrap();
    }
}
