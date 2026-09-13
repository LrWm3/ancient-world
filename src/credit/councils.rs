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
const INSTITUTION_REQUEST_TAG: u64 = 1 << 62;

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
    /// Separate experimental opt-in; archived council-only policies stay unchanged.
    #[serde(default)]
    pub institution_lenders: bool,
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
            institution_lenders: false,
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
        // A local institution uses the same actual treasury adapter as explicit
        // loans. Keep a year of operating needs; never treat donations as promised.
        let mut institution_councils = std::collections::BTreeMap::new();
        if policy.institution_lenders {
            if let Some(culture) = &self.culture {
                for institution in &culture.institutions {
                    if !institution.operational()
                        || self.sites[institution.site as usize].abandoned
                        || institution
                            .capacity
                            .as_ref()
                            .and_then(|c| c.building.as_ref())
                            .is_some_and(|b| {
                                culture.artifacts.get(b.artifact as usize).is_none_or(|a| {
                                    a.destroyed
                                        || a.lost
                                        || a.site != Some(institution.site)
                                        || a.owner
                                            != crate::culture::Owner::Institution(institution.id)
                                })
                            })
                        || !institution.members.contains(&institution.leader)
                        || !culture
                            .site_people(self, institution.site)
                            .contains(&institution.leader)
                    {
                        continue;
                    }
                    let reserve = culture
                        .institution_operating_target(self, institution)
                        .max(policy.reserve_floor);
                    let offered = (institution.treasury - reserve).max(0.) * policy.surplus_share;
                    if offered > 0. {
                        institution_councils
                            .insert(institution.id, self.controller(institution.site));
                        offers.push(Offer {
                            lender: Account::Institution(institution.id),
                            month: self.month,
                            cash: institution.treasury,
                            operating_reserve: reserve,
                            offered_principal: offered,
                            minimum_annual_rate: policy.annual_rate,
                        });
                    }
                }
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
            let lenders: Vec<_> = offers
                .iter()
                .filter(|o| match o.lender {
                    Account::Council(lender) => contacts.contains(&(id, lender)),
                    Account::Institution(lender) => institution_councils.get(&lender) == Some(&id),
                    _ => false,
                })
                .collect();
            review.contacted_lenders = lenders.len();
            review.outcome = ReviewOutcome::NoContactedLender;
            if lenders.is_empty() {
                continue;
            }
            review.outcome = ReviewOutcome::Submitted;
            for offer in &lenders {
                let (tag, lender) = match offer.lender {
                    Account::Council(lender) => (0, lender),
                    Account::Institution(lender) => (INSTITUTION_REQUEST_TAG, lender),
                    _ => unreachable!(),
                };
                requests.push(Request {
                    id: COUNCIL_REQUEST_NAMESPACE | tag | (u64::from(id) << 32) | u64::from(lender),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires hardware GPU"]
    fn local_institution_lends_only_available_surplus_with_present_leadership() {
        use crate::{
            catalog::Catalog,
            config::Config,
            culture::{Institution, InstitutionKind},
            gpu::{ContextGpu, Generator},
            household_economy::council_allocation,
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 13;
        let leader = h.culture.as_ref().unwrap().site_people(h, 0)[0];
        assert!(h.sites[0].economy.finance[0] >= 200.);
        h.sites[0].economy.finance[0] -= 200.;
        h.culture.as_mut().unwrap().institutions.push(Institution {
            capacity: None,
            id: 0,
            name: "Local lending fixture".into(),
            kind: InstitutionKind::Merchant,
            site: 0,
            tradition: None,
            members: vec![leader],
            leader,
            treasury: 200.,
            active: true,
            founded: 0,
            knowledge: Default::default(),
            property: vec![],
            dues: 200.,
            expenses: 0.,
        });
        let society = h.society.as_mut().unwrap();
        // Keep all cash with an existing owner; remove foreign council offers.
        for council in &mut society.councils {
            h.sites[council.civilization as usize].economy.finance[0] += council.treasury as f32;
            council.treasury = 0.;
        }
        society.routes.iter_mut().for_each(|r| r.open = false);
        society
            .household_economy
            .as_mut()
            .unwrap()
            .council_allocations = vec![council_allocation::Receipt {
            month: 12,
            council: 0,
            policy: Default::default(),
            treasury: 0.,
            administration_forecast: 0.,
            relief_requested: 4.,
            relief_ceiling: 0.,
            relief_granted: 0.,
            relief_paid: 0.,
        }];
        h.credit.council_policy.enabled = true;
        h.credit.council_policy.institution_lenders = true;
        h.credit
            .tax_observations
            .push(super::super::taxes::Observation {
                month: 12,
                council: 0,
                collected: 2000.,
                support_requested: 0.,
                road_requested: Some(0.),
            });
        let opening = h.clone();
        for intervention in 0..8 {
            let mut control = opening.clone();
            match intervention {
                0 => control.credit.council_policy.institution_lenders = false,
                1 => control.culture.as_mut().unwrap().institutions[0].active = false,
                2 => control.culture.as_mut().unwrap().institutions[0]
                    .members
                    .clear(),
                3 => control.credit.council_policy.reserve_floor = 200.,
                4 => control.credit.tax_observations[0].support_requested = 2000.,
                _ => {
                    let culture = control.culture.as_mut().unwrap();
                    let mut capacity = crate::institution_capacity::Capacity::new(13);
                    capacity.readiness = 1.;
                    capacity.building = Some(crate::institution_capacity::MeetingPlace::new(0));
                    culture.institutions[0].capacity = Some(capacity);
                    let artifact = &mut culture.artifacts[0];
                    artifact.owner = if intervention == 5 {
                        crate::culture::Owner::Community(0)
                    } else {
                        crate::culture::Owner::Institution(0)
                    };
                    artifact.site = Some(if intervention == 7 { 1 } else { 0 });
                    artifact.lost = intervention == 6;
                    // Cached readiness alone would incorrectly permit this offer.
                    assert!(culture.institutions[0].operational());
                }
            }
            assert_eq!(control.council_credit_month().unwrap(), 0);
            assert_eq!(
                control.culture.as_ref().unwrap().institutions[0].treasury,
                200.
            );
        }
        let money = h.money_residual();
        assert_eq!(h.council_credit_month().unwrap(), 1);
        let loan = &h.credit.loans[0];
        assert_eq!(loan.terms.lender, Account::Institution(0));
        assert_eq!(loan.terms.borrower, Account::Council(0));
        assert!(loan.original_principal >= 4.);
        assert!(h.culture.as_ref().unwrap().institutions[0].treasury >= 100.);
        assert!((h.money_residual() - money).abs() < 1e-7);
        h.validate_credit().unwrap();
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        assert_eq!(resumed.council_credit_month().unwrap(), 0);
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        // Removing this feature stops offers, not servicing the existing claim.
        for world in [&mut *h, &mut resumed] {
            world.credit.council_policy.institution_lenders = false;
            world.credit.servicing_policy.available_cash_share = 1.;
            world.credit.servicing_policy.protected_cash.clear();
            world.month = 25;
            world.service_credit_month().unwrap();
        }
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert!(h.credit.loans[0].outstanding_principal < h.credit.loans[0].original_principal);
        assert!((h.money_residual() - money).abs() < 1e-7);
    }
}
