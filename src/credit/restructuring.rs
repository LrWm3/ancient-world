//! Dated consent and receipt coverage for the single permitted loan extension.
use super::{underwriting::Evidence, Account, Loan, Status, MONTHS_PER_YEAR};
use crate::civilization::History;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

const MAX_EXTENSION_MONTHS: u32 = 12;
const MIN_RECEIPT_COVERAGE: f64 = 1.5;
const MAX_FORECAST_LOSS_FRACTION: f64 = 0.25;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub month: u32,
    pub loan: u64,
    pub revised_maturity: u32,
    /// Same original receivable identity, with a separately revised arrival date.
    pub evidence: Evidence,
    pub expected_payment_month: u32,
    pub lender_consent: Option<Account>,
    pub borrower_consent: Option<Account>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Accepted,
    NoConsent,
    NotEligible,
    InvalidEvidence,
    InsufficientCoverage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub opening: Loan,
    pub proposal: Proposal,
    pub decision: Decision,
    pub original_maturity: u32,
    pub projected_due: f64,
    pub competing_claims: f64,
    pub net_receipts: f64,
}

fn projected_due(loan: &Loan, month: u32) -> f64 {
    loan.total_due()
        + loan.outstanding_principal
            * loan.terms.annual_simple_rate
            * f64::from(month.saturating_sub(loan.accrued_through_month))
            / MONTHS_PER_YEAR
}

/// Portfolio is a completed boundary. Every competing claim to the same original
/// source remains reserved, including other borrowers and earlier extensions.
pub fn resolve(month: u32, loan: &Loan, portfolio: &[Loan], proposal: Proposal) -> Result<Receipt> {
    loan.validate()?;
    let evidence = &proposal.evidence;
    ensure!(
        proposal.month == month && proposal.loan == loan.id,
        "stale restructuring proposal"
    );
    ensure!(
        [
            evidence.expected_receipts,
            evidence.operating_costs,
            evidence.expected_loss_fraction,
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= 0.)
            && evidence.expected_loss_fraction <= 1.,
        "invalid restructuring forecast"
    );
    let due = projected_due(loan, proposal.revised_maturity);
    let mut competing = 0.;
    for other in portfolio {
        other.validate()?;
        ensure!(
            other.accrued_through_month <= month,
            "future restructuring portfolio"
        );
        if other.id != loan.id
            && other.terms.source == evidence.source
            && matches!(other.status, Status::Performing | Status::Arrears)
        {
            competing += projected_due(
                other,
                other
                    .terms
                    .maturity_month
                    .max(proposal.expected_payment_month),
            );
        }
    }
    let net = (evidence.expected_receipts - evidence.operating_costs).max(0.);
    ensure!(
        due.is_finite() && competing.is_finite(),
        "restructuring coverage overflow"
    );
    let decision = decide(month, loan, &proposal, due, competing, net);
    Ok(Receipt {
        opening: loan.clone(),
        proposal,
        decision,
        original_maturity: loan.terms.maturity_month,
        projected_due: due,
        competing_claims: competing,
        net_receipts: net,
    })
}

fn decide(
    month: u32,
    loan: &Loan,
    proposal: &Proposal,
    due: f64,
    competing: f64,
    net: f64,
) -> Decision {
    let evidence = &proposal.evidence;
    if proposal.lender_consent != Some(loan.terms.lender)
        || proposal.borrower_consent != Some(loan.terms.borrower)
    {
        Decision::NoConsent
    } else if loan.status != Status::Arrears
        || loan.restructured
        || loan.accrued_through_month != month
        || proposal.revised_maturity <= month
        || proposal.revised_maturity - month > MAX_EXTENSION_MONTHS
    {
        Decision::NotEligible
    } else if evidence.source != loan.terms.source
        || evidence.beneficiary != loan.terms.borrower
        || evidence.observed_month != month
        || evidence.expected_loss_fraction > MAX_FORECAST_LOSS_FRACTION
        || proposal.expected_payment_month <= month
        || proposal.expected_payment_month >= proposal.revised_maturity
    {
        Decision::InvalidEvidence
    } else if net / MIN_RECEIPT_COVERAGE < due + competing {
        Decision::InsufficientCoverage
    } else {
        Decision::Accepted
    }
}

impl Receipt {
    pub(crate) fn validate(&self, month: u32) -> Result<()> {
        self.opening.validate()?;
        let p = &self.proposal;
        ensure!(
            p.month <= month
                && p.loan == self.opening.id
                && self.opening.accrued_through_month <= p.month
                && self.original_maturity == self.opening.terms.maturity_month,
            "invalid restructuring receipt boundary"
        );
        ensure!(
            [
                self.projected_due,
                self.competing_claims,
                self.net_receipts,
                p.evidence.expected_receipts,
                p.evidence.operating_costs,
                p.evidence.expected_loss_fraction
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.)
                && p.evidence.expected_loss_fraction <= 1.
                && self.projected_due == projected_due(&self.opening, p.revised_maturity)
                && self.net_receipts
                    == (p.evidence.expected_receipts - p.evidence.operating_costs).max(0.),
            "invalid restructuring receipt amounts"
        );
        ensure!(
            self.decision
                == decide(
                    p.month,
                    &self.opening,
                    p,
                    self.projected_due,
                    self.competing_claims,
                    self.net_receipts
                ),
            "invalid restructuring decision"
        );
        Ok(())
    }
}

impl History {
    /// Explicit completed-Open boundary API. Both parties' policy decisions and
    /// current evidence must be provided; there is no automatic refinancing.
    pub fn resolve_credit_restructuring(&mut self, proposal: Proposal) -> Result<Decision> {
        ensure!(
            self.credit.serviced_month == Some(self.month),
            "service debt before renegotiation"
        );
        ensure!(
            !self
                .credit
                .restructurings
                .iter()
                .any(|r| r.proposal.month == self.month && r.proposal.loan == proposal.loan),
            "replayed restructuring decision"
        );
        let loan = self
            .credit
            .loans
            .get(proposal.loan as usize)
            .filter(|l| l.id == proposal.loan)
            .context("missing restructuring loan")?;
        self.credit_account_cash(loan.terms.lender)?;
        self.credit_account_cash(loan.terms.borrower)?;
        let receipt = resolve(self.month, loan, &self.credit.loans, proposal)?;
        let decision = receipt.decision;
        if decision == Decision::Accepted {
            self.credit.loans[receipt.proposal.loan as usize]
                .restructure(self.month, receipt.proposal.revised_maturity)?;
        }
        self.credit.restructurings.push(receipt);
        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credit::{RepaymentSource, Terms, SHARED_CURRENCY};
    fn fixture() -> (Loan, Proposal) {
        let source = RepaymentSource::Export {
            contract: 7,
            payment_month: 2,
        };
        let mut loan = Loan::record_disbursement(
            0,
            Terms {
                lender: Account::Town(0),
                borrower: Account::Town(1),
                currency: SHARED_CURRENCY,
                source,
                annual_simple_rate: 0.12,
                maturity_month: 3,
                grace_months: 3,
            },
            0,
            100.,
        )
        .unwrap();
        loan.accrue_to(3).unwrap();
        let proposal = Proposal {
            month: 3,
            loan: 0,
            revised_maturity: 6,
            evidence: Evidence {
                source,
                beneficiary: loan.terms.borrower,
                observed_month: 3,
                expected_receipts: 200.,
                operating_costs: 10.,
                expected_loss_fraction: 0.1,
            },
            expected_payment_month: 5,
            lender_consent: Some(loan.terms.lender),
            borrower_consent: Some(loan.terms.borrower),
        };
        (loan, proposal)
    }
    #[test]
    fn consent_coverage_original_source_and_competing_claims_bound_extension() {
        let (loan, p) = fixture();
        let result = resolve(3, &loan, &[], p.clone()).unwrap();
        assert_eq!(result.decision, Decision::Accepted);
        assert_eq!(result.projected_due, 106.);
        result.validate(3).unwrap();
        let mut q = p.clone();
        q.lender_consent = None;
        assert_eq!(
            resolve(3, &loan, &[], q).unwrap().decision,
            Decision::NoConsent
        );
        let mut q = p.clone();
        q.evidence.expected_receipts = 110.;
        assert_eq!(
            resolve(3, &loan, &[], q).unwrap().decision,
            Decision::InsufficientCoverage
        );
        let mut other = loan.clone();
        other.id = 1;
        assert_eq!(
            resolve(3, &loan, &[other], p.clone()).unwrap().decision,
            Decision::InsufficientCoverage
        );
        let mut q = p.clone();
        q.evidence.source = RepaymentSource::Export {
            contract: 7,
            payment_month: 5,
        };
        assert_eq!(
            resolve(3, &loan, &[], q).unwrap().decision,
            Decision::InvalidEvidence
        );
        let mut corrupt = result;
        corrupt.projected_due = 0.;
        assert!(corrupt.validate(3).is_err());
        let mut extended = loan;
        extended.restructure(3, 6).unwrap();
        extended.accrue_to(6).unwrap();
        let mut q = p;
        q.month = 6;
        q.evidence.observed_month = 6;
        q.expected_payment_month = 8;
        q.revised_maturity = 9;
        assert_eq!(
            resolve(6, &extended, &[], q).unwrap().decision,
            Decision::NotEligible
        );
    }
}
