//! Snapshot-based lending proposals. No account is debited by this resolver.
use super::{Account, Loan, RepaymentSource, Status, Terms, SHARED_CURRENCY};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const MAX_EVIDENCE_AGE_MONTHS: u32 = 24;
const RECEIPT_COVERAGE_FRACTION: f64 = 0.5;
const MAX_REQUEST_PRINCIPAL: f64 = 2000.;
const MAX_BORROWER_PRINCIPAL: f64 = 5000.;
const MAX_LENDER_PRINCIPAL: f64 = 5000.;
const MAX_EXPECTED_LOSS_FRACTION: f64 = 0.25;
const DEFAULT_CREDIT_EXCLUSION_MONTHS: u32 = 60;
const MONTHS_PER_YEAR: f64 = 12.;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    pub evidence_age_months: u32,
    pub receipt_coverage_fraction: f64,
    pub max_request_principal: f64,
    pub max_borrower_principal: f64,
    pub max_lender_principal: f64,
    pub max_expected_loss_fraction: f64,
    pub default_exclusion_months: u32,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            evidence_age_months: MAX_EVIDENCE_AGE_MONTHS,
            receipt_coverage_fraction: RECEIPT_COVERAGE_FRACTION,
            max_request_principal: MAX_REQUEST_PRINCIPAL,
            max_borrower_principal: MAX_BORROWER_PRINCIPAL,
            max_lender_principal: MAX_LENDER_PRINCIPAL,
            max_expected_loss_fraction: MAX_EXPECTED_LOSS_FRACTION,
            default_exclusion_months: DEFAULT_CREDIT_EXCLUSION_MONTHS,
        }
    }
}
impl Policy {
    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(
            [
                self.max_request_principal,
                self.max_borrower_principal,
                self.max_lender_principal
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.),
            "invalid credit exposure limits"
        );
        ensure!(
            [
                self.receipt_coverage_fraction,
                self.max_expected_loss_fraction
            ]
            .iter()
            .all(|v| v.is_finite() && (0. ..=1.).contains(v)),
            "invalid underwriting fractions"
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub source: RepaymentSource,
    pub beneficiary: Account,
    pub observed_month: u32,
    pub expected_receipts: f64,
    pub operating_costs: f64,
    pub expected_loss_fraction: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Offer {
    pub lender: Account,
    pub month: u32,
    pub cash: f64,
    pub operating_reserve: f64,
    /// A voluntary ceiling; cash above the reserve is not automatically offered.
    pub offered_principal: f64,
    pub minimum_annual_rate: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub month: u32,
    pub terms: Terms,
    pub principal: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    Approved,
    UnsupportedCurrency,
    MissingEvidence,
    StaleEvidence,
    WrongBeneficiary,
    Timing,
    Risk,
    NoOffer,
    InsufficientReturn,
    CreditExclusion,
    BorrowingAndLending,
    NoCapacity,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grant {
    pub request: u64,
    pub month: u32,
    pub requested: f64,
    pub eligible: f64,
    pub granted: f64,
    pub pledged_receipts: f64,
    pub decision: Decision,
}

fn payment_month(source: RepaymentSource) -> u32 {
    match source {
        RepaymentSource::AnnualTax {
            collection_month, ..
        } => collection_month,
        RepaymentSource::Export { payment_month, .. }
        | RepaymentSource::ServiceOrder { payment_month, .. } => payment_month,
    }
}
fn active(loan: &Loan) -> bool {
    matches!(loan.status, Status::Performing | Status::Arrears)
}
fn sum(map: &mut BTreeMap<Account, f64>, key: Account, value: f64) {
    *map.entry(key).or_default() += value;
}

/// Proportional scaling against three independent pools (lender, borrower,
/// repayment source). Taking the smallest factor cannot oversubscribe a pool,
/// though it can leave capacity unused. Requests are sorted by ID for reproducible
/// sums; ID order does not grant an earlier applicant first claim on money.
pub fn resolve(
    month: u32,
    policy: &Policy,
    loans: &[Loan],
    offers: &[Offer],
    evidence: &[Evidence],
    requests: &[Request],
) -> Result<Vec<Grant>> {
    policy.validate()?;
    let mut offer_map = BTreeMap::new();
    for offer in offers {
        ensure!(
            [
                offer.cash,
                offer.operating_reserve,
                offer.offered_principal,
                offer.minimum_annual_rate
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.),
            "invalid lender offer"
        );
        ensure!(
            offer_map.insert(offer.lender, offer).is_none(),
            "duplicate lender offer"
        );
    }
    let mut sources = BTreeMap::new();
    for e in evidence {
        ensure!(
            [
                e.expected_receipts,
                e.operating_costs,
                e.expected_loss_fraction
            ]
            .iter()
            .all(|v| v.is_finite() && *v >= 0.)
                && e.expected_loss_fraction <= 1.,
            "invalid receipt evidence"
        );
        ensure!(
            sources.insert(e.source, e).is_none(),
            "conflicting receipt evidence"
        );
    }
    let mut borrower_used = BTreeMap::new();
    let mut lender_used = BTreeMap::new();
    let mut pledged = BTreeMap::new();
    for loan in loans {
        loan.validate()?;
        ensure!(
            loan.accrued_through_month <= month,
            "future portfolio observation"
        );
        if active(loan) {
            sum(
                &mut borrower_used,
                loan.terms.borrower,
                loan.outstanding_principal,
            );
            sum(
                &mut lender_used,
                loan.terms.lender,
                loan.outstanding_principal,
            );
            let future = loan
                .terms
                .maturity_month
                .saturating_sub(loan.accrued_through_month) as f64;
            *pledged.entry(loan.terms.source).or_insert(0.) += loan.total_due()
                + loan.outstanding_principal * loan.terms.annual_simple_rate * future
                    / MONTHS_PER_YEAR;
        }
    }
    let mut ordered: Vec<_> = requests.iter().collect();
    ordered.sort_by_key(|r| r.id);
    ensure!(
        ordered.windows(2).all(|w| w[0].id != w[1].id),
        "duplicate credit request"
    );
    let borrowing: BTreeSet<_> = requests.iter().map(|r| r.terms.borrower).collect();
    let lending: BTreeSet<_> = requests.iter().map(|r| r.terms.lender).collect();
    let mut grants = Vec::new();
    let mut lender_demand = BTreeMap::new();
    let mut borrower_demand = BTreeMap::new();
    let mut source_demand = BTreeMap::new();
    let mut factors = Vec::new();
    for request in &ordered {
        request.terms.validate(month)?;
        ensure!(
            request.principal.is_finite() && request.principal > 0.,
            "invalid requested principal"
        );
        let t = &request.terms;
        let factor =
            1. + t.annual_simple_rate * (t.maturity_month - month) as f64 / MONTHS_PER_YEAR;
        let decision = if t.currency != SHARED_CURRENCY {
            Decision::UnsupportedCurrency
        } else if request.month != month {
            Decision::Timing
        } else if lending.contains(&t.borrower)
            || borrowing.contains(&t.lender)
            || lender_used.contains_key(&t.borrower)
            || borrower_used.contains_key(&t.lender)
        {
            Decision::BorrowingAndLending
        } else if loans.iter().any(|l| {
            l.terms.borrower == t.borrower
                && l.status == Status::Defaulted
                && month.saturating_sub(l.accrued_through_month) < policy.default_exclusion_months
        }) {
            Decision::CreditExclusion
        } else if let Some(e) = sources.get(&t.source) {
            if e.beneficiary != t.borrower {
                Decision::WrongBeneficiary
            } else if e.observed_month > month
                || month - e.observed_month > policy.evidence_age_months
            {
                Decision::StaleEvidence
            } else if payment_month(e.source) <= month
                || payment_month(e.source) >= t.maturity_month
            {
                Decision::Timing
            } else if e.expected_loss_fraction > policy.max_expected_loss_fraction {
                Decision::Risk
            } else if let Some(o) = offer_map.get(&t.lender).filter(|o| o.month == month) {
                if t.annual_simple_rate < o.minimum_annual_rate + e.expected_loss_fraction {
                    Decision::InsufficientReturn
                } else {
                    Decision::Approved
                }
            } else {
                Decision::NoOffer
            }
        } else {
            Decision::MissingEvidence
        };
        let eligible = if decision == Decision::Approved {
            request.principal.min(policy.max_request_principal)
        } else {
            0.
        };
        if eligible > 0. {
            sum(&mut lender_demand, t.lender, eligible);
            sum(&mut borrower_demand, t.borrower, eligible);
            *source_demand.entry(t.source).or_insert(0.) += eligible * factor;
        }
        grants.push(Grant {
            request: request.id,
            month,
            requested: request.principal,
            eligible,
            granted: 0.,
            pledged_receipts: 0.,
            decision,
        });
        factors.push(factor);
    }
    for ((grant, request), factor) in grants.iter_mut().zip(ordered).zip(factors) {
        if grant.eligible == 0. {
            continue;
        }
        let t = &request.terms;
        let o = offer_map[&t.lender];
        let e = sources[&t.source];
        let lender = (o.cash - o.operating_reserve)
            .max(0.)
            .min(o.offered_principal)
            .min(
                (policy.max_lender_principal - lender_used.get(&t.lender).copied().unwrap_or(0.))
                    .max(0.),
            );
        let borrower = (policy.max_borrower_principal
            - borrower_used.get(&t.borrower).copied().unwrap_or(0.))
        .max(0.);
        let source = ((e.expected_receipts - e.operating_costs).max(0.)
            * policy.receipt_coverage_fraction
            - pledged.get(&t.source).copied().unwrap_or(0.))
        .max(0.);
        let scale = (lender / lender_demand[&t.lender])
            .min(borrower / borrower_demand[&t.borrower])
            .min(source / source_demand[&t.source])
            .min(1.);
        grant.granted = grant.eligible * scale;
        grant.pledged_receipts = grant.granted * factor;
        if grant.granted == 0. {
            grant.decision = Decision::NoCapacity;
        }
    }
    Ok(grants)
}

/// A dated decision and its actual funding. Incomplete rounds are retained on a
/// settlement error and cannot be replayed as though no money had moved.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Round {
    pub month: u32,
    pub policy: Policy,
    pub offers: Vec<Offer>,
    pub evidence: Vec<Evidence>,
    pub requests: Vec<Request>,
    pub grants: Vec<Grant>,
    pub loan_ids: Vec<Option<u64>>,
    pub complete: bool,
}

impl crate::civilization::History {
    pub fn fund_credit_requests(
        &mut self,
        policy: Policy,
        mut offers: Vec<Offer>,
        evidence: Vec<Evidence>,
        requests: Vec<Request>,
    ) -> Result<usize> {
        ensure!(!requests.is_empty(), "empty credit round");
        for r in &requests {
            ensure!(
                !self
                    .credit
                    .rounds
                    .iter()
                    .any(|round| round.month == self.month
                        && round.requests.iter().any(|prior| prior.id == r.id)),
                "credit request already resolved this month"
            );
            // Recheck account availability before recording a decision or debiting money.
            self.credit_account_cash(r.terms.borrower)?;
            self.credit_account_cash(r.terms.lender)?;
        }
        for offer in &mut offers {
            let actual = self.credit_account_cash(offer.lender)?;
            ensure!(
                offer.cash.is_finite() && offer.cash >= 0.,
                "invalid offered cash observation"
            );
            offer.cash = offer.cash.min(actual);
        }
        let grants = resolve(
            self.month,
            &policy,
            &self.credit.loans,
            &offers,
            &evidence,
            &requests,
        )?;
        let index = self.credit.rounds.len();
        let loan_ids = vec![None; grants.len()];
        self.credit.rounds.push(Round {
            month: self.month,
            policy,
            offers,
            evidence,
            requests,
            grants,
            loan_ids,
            complete: false,
        });
        for i in 0..self.credit.rounds[index].grants.len() {
            let grant = &self.credit.rounds[index].grants[i];
            if grant.granted <= 0. {
                continue;
            }
            let request = self.credit.rounds[index]
                .requests
                .iter()
                .find(|r| r.id == grant.request)
                .unwrap();
            let (terms, amount) = (request.terms.clone(), grant.granted);
            let id = self.commit_credit_loan(terms, amount)?;
            self.credit.rounds[index].loan_ids[i] = id;
        }
        self.credit.rounds[index].complete = true;
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> RepaymentSource {
        RepaymentSource::AnnualTax {
            council: 0,
            collection_month: 12,
        }
    }
    fn evidence() -> Evidence {
        Evidence {
            source: source(),
            beneficiary: Account::Council(0),
            observed_month: 0,
            expected_receipts: 200.,
            operating_costs: 0.,
            expected_loss_fraction: 0.,
        }
    }
    fn offer(id: u32) -> Offer {
        Offer {
            lender: Account::Institution(id),
            month: 1,
            cash: 100.,
            operating_reserve: 20.,
            offered_principal: 80.,
            minimum_annual_rate: 0.,
        }
    }
    fn request(id: u64, lender: u32) -> Request {
        Request {
            id,
            month: 1,
            principal: 100.,
            terms: Terms {
                lender: Account::Institution(lender),
                borrower: Account::Council(0),
                currency: SHARED_CURRENCY,
                source: source(),
                annual_simple_rate: 0.,
                maturity_month: 13,
                grace_months: 3,
            },
        }
    }
    #[test]
    fn one_receipt_cannot_back_two_full_loans_and_order_does_not_choose_winner() {
        let p = Policy::default();
        let a = request(1, 0);
        let b = request(2, 1);
        let g = resolve(
            1,
            &p,
            &[],
            &[offer(0), offer(1)],
            &[evidence()],
            &[a.clone(), b.clone()],
        )
        .unwrap();
        assert_eq!(
            g.iter().map(|v| v.granted).collect::<Vec<_>>(),
            vec![50., 50.]
        );
        let reversed = resolve(1, &p, &[], &[offer(1), offer(0)], &[evidence()], &[b, a]).unwrap();
        assert_eq!(
            serde_json::to_value(g).unwrap(),
            serde_json::to_value(reversed).unwrap()
        );
    }
    #[test]
    fn lender_reserve_and_existing_pledge_reduce_new_capacity() {
        let r = request(1, 0);
        let g = resolve(
            1,
            &Policy::default(),
            &[],
            &[offer(0)],
            &[evidence()],
            std::slice::from_ref(&r),
        )
        .unwrap();
        assert_eq!(g[0].granted, 80.);
        let loan = Loan::record_disbursement(0, r.terms.clone(), 1, 70.).unwrap();
        let g = resolve(
            1,
            &Policy::default(),
            &[loan],
            &[offer(0)],
            &[evidence()],
            &[r],
        )
        .unwrap();
        assert_eq!(g[0].granted, 30.);
    }
    #[test]
    fn timing_risk_and_real_repayment_beneficiary_gate_credit() {
        let mut e = evidence();
        e.beneficiary = Account::Town(0);
        let decision = |e: Evidence| {
            resolve(
                1,
                &Policy::default(),
                &[],
                &[offer(0)],
                &[e],
                &[request(1, 0)],
            )
            .unwrap()[0]
                .decision
        };
        assert_eq!(decision(e), Decision::WrongBeneficiary);
        let mut e = evidence();
        e.observed_month = 2;
        assert_eq!(decision(e), Decision::StaleEvidence);
        let mut e = evidence();
        e.expected_loss_fraction = 0.5;
        assert_eq!(decision(e), Decision::Risk);
        let mut e = evidence();
        e.expected_receipts = 0.;
        assert_eq!(decision(e), Decision::NoCapacity);
    }
}
