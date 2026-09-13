//! Opt-in tax-bridge pilot, using current administration and lagged relief demand.
use super::{
    underwriting::{Offer, Request},
    Account, RepaymentSource, Terms, SHARED_CURRENCY,
};
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const OPERATING_RESERVE_MONTHS: f64 = 3.;
const MINIMUM_OPERATING_CASH: f64 = 100.;
const VOLUNTARY_SURPLUS_SHARE: f64 = 0.25;
const ANNUAL_BRIDGE_RATE: f64 = 0.06;
const ANNUAL_OPERATING_MONTHS: f64 = 12.;
const MAX_RELIEF_OBSERVATION_AGE_MONTHS: u32 = 1;
const COUNCIL_REQUEST_NAMESPACE: u64 = 1 << 63;

/// Construction outcomes, before underwriting decides whether any loan is safe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReviewOutcome {
    NoCashGap,
    MissingTaxEvidence,
    InvalidCollectionWindow,
    NoContactedLender,
    Submitted,
}

/// Latest enabled boundary per council. Totals retain counts without a monthly log.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Review {
    pub month: u32,
    pub council: u32,
    pub opening_cash: f64,
    pub monthly_demand: f64,
    pub cash_gap: f64,
    pub expected_taxes: Option<f64>,
    pub annual_commitments: Option<f64>,
    pub monthly_costs_annualized: f64,
    pub contacted_lenders: usize,
    pub outcome: ReviewOutcome,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    pub enabled: bool,
    pub reserve_months: f64,
    pub reserve_floor: f64,
    pub surplus_share: f64,
    pub annual_rate: f64,
    pub underwriting: super::underwriting::Policy,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            enabled: false,
            reserve_months: OPERATING_RESERVE_MONTHS,
            reserve_floor: MINIMUM_OPERATING_CASH,
            surplus_share: VOLUNTARY_SURPLUS_SHARE,
            annual_rate: ANNUAL_BRIDGE_RATE,
            underwriting: Default::default(),
        }
    }
}
impl Policy {
    pub fn validate(&self) -> Result<()> {
        self.underwriting.validate()?;
        ensure!(
            [self.reserve_months, self.reserve_floor]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.),
            "invalid council lending reserve"
        );
        ensure!(
            [self.surplus_share, self.annual_rate]
                .iter()
                .all(|v| v.is_finite() && (0. ..=1.).contains(v)),
            "invalid council lending rate"
        );
        Ok(())
    }
}

impl History {
    /// First Reserve operation. Borrowing supplies cash to the existing council
    /// account; ordinary relief/administration still decides and pays for work.
    pub fn council_credit_month(&mut self) -> Result<usize> {
        let policy = self.credit.council_policy.clone();
        policy.validate()?;
        if !policy.enabled || self.credit.council_decided_month == Some(self.month) {
            return Ok(0);
        }
        ensure!(
            self.credit
                .council_decided_month
                .is_none_or(|m| m < self.month),
            "council credit moved backwards"
        );
        let Some(society) = &self.society else {
            return Ok(0);
        };
        let administration = self.administration_forecast();
        let mut demand = vec![0.; society.councils.len()];
        let mut offers = Vec::new();
        for council in &society.councils {
            let id = council.civilization as usize;
            let relief = society
                .household_economy
                .as_ref()
                .and_then(|e| {
                    e.council_allocations.iter().find(|r| {
                        r.council == council.civilization
                            && r.month < self.month
                            && self.month - r.month <= MAX_RELIEF_OBSERVATION_AGE_MONTHS
                    })
                })
                .map_or(0., |r| r.relief_requested);
            demand[id] = administration[id] + relief;
            let reserve = (demand[id] * policy.reserve_months).max(policy.reserve_floor);
            let offered = (council.treasury - reserve).max(0.) * policy.surplus_share;
            if offered > 0. {
                offers.push(Offer {
                    lender: Account::Council(council.civilization),
                    month: self.month,
                    cash: council.treasury,
                    operating_reserve: reserve,
                    offered_principal: offered,
                    minimum_annual_rate: policy.annual_rate,
                });
            }
        }
        // Direct open-route contacts only in this pilot; neither global knowledge
        // nor cash teleported between otherwise isolated councils creates offers.
        let mut contacts = BTreeSet::new();
        for route in &society.routes {
            if !route.open
                || self.sites[route.from as usize].abandoned
                || self.sites[route.to as usize].abandoned
            {
                continue;
            }
            let a = self.controller(route.from);
            let b = self.controller(route.to);
            let hostile = self.politics.as_ref().is_some_and(|p| {
                p.wars.iter().any(|w| {
                    w.ended.is_none()
                        && ((w.attacker == a && w.defender == b)
                            || (w.attacker == b && w.defender == a))
                })
            });
            if a != b && !hostile {
                contacts.insert((a, b));
                contacts.insert((b, a));
            }
        }
        let mut evidence = Vec::new();
        let mut requests = Vec::new();
        let mut reviews = Vec::new();
        for council in &society.councils {
            let id = council.civilization;
            let needed = (demand[id as usize] - council.treasury).max(0.);
            reviews.push(Review {
                month: self.month,
                council: id,
                opening_cash: council.treasury,
                monthly_demand: demand[id as usize],
                cash_gap: needed,
                expected_taxes: None,
                annual_commitments: None,
                monthly_costs_annualized: demand[id as usize] * ANNUAL_OPERATING_MONTHS,
                contacted_lenders: 0,
                outcome: ReviewOutcome::NoCashGap,
            });
            let review = reviews.last_mut().unwrap();
            if needed <= 0. {
                continue;
            }
            review.outcome = ReviewOutcome::MissingTaxEvidence;
            let Some(mut receipt) = self.council_credit_evidence(id) else {
                continue;
            };
            review.expected_taxes = Some(receipt.expected_receipts);
            review.annual_commitments = Some(receipt.operating_costs);
            // Protect the next annual operating budget, not just months until
            // maturity. A shorter term does not erase recurring service needs.
            receipt.operating_costs += review.monthly_costs_annualized;
            review.outcome = ReviewOutcome::InvalidCollectionWindow;
            let RepaymentSource::AnnualTax {
                collection_month, ..
            } = receipt.source
            else {
                continue;
            };
            if collection_month <= self.month {
                continue;
            }
            let Some(maturity) = collection_month.checked_add(1) else {
                continue;
            };
            let lenders: Vec<_> = offers.iter().filter(|o|
                matches!(o.lender, Account::Council(lender) if contacts.contains(&(id, lender))))
                .collect();
            review.contacted_lenders = lenders.len();
            review.outcome = ReviewOutcome::NoContactedLender;
            if lenders.is_empty() {
                continue;
            }
            review.outcome = ReviewOutcome::Submitted;
            for offer in &lenders {
                let Account::Council(lender) = offer.lender else {
                    unreachable!()
                };
                requests.push(Request {
                    id: COUNCIL_REQUEST_NAMESPACE | (u64::from(id) << 32) | u64::from(lender),
                    month: self.month,
                    principal: needed / lenders.len() as f64,
                    terms: Terms {
                        lender: offer.lender,
                        borrower: Account::Council(id),
                        currency: SHARED_CURRENCY,
                        source: receipt.source,
                        annual_simple_rate: policy.annual_rate,
                        maturity_month: maturity,
                        grace_months: Terms::default_grace_months(),
                    },
                });
            }
            evidence.push(receipt);
        }
        let count = if requests.is_empty() {
            0
        } else {
            let round =
                self.fund_credit_requests(policy.underwriting, offers, evidence, requests)?;
            self.credit.rounds[round]
                .loan_ids
                .iter()
                .filter(|id| id.is_some())
                .count()
        };
        for review in &reviews {
            *self
                .credit
                .council_review_counts
                .entry(review.outcome)
                .or_default() += 1;
        }
        self.credit.council_reviews = reviews;
        self.credit.council_decided_month = Some(self.month);
        Ok(count)
    }
}
