//! Opt-in working-cash requests from planned production and funded receivables.
use super::{
    underwriting::{Offer, Request},
    Account, Terms, SHARED_CURRENCY,
};
use crate::{
    civilization::History,
    economy::{FOOD, GOODS},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const OPERATING_CASH_FLOOR: f64 = 100.;
const LENDER_SURPLUS_SHARE: f64 = 0.25;
const ANNUAL_REQUIRED_RETURN: f64 = 0.06;
const SHIPMENT_LOSS_ASSUMPTION: f64 = 0.05;
const MONTHS_PER_YEAR: f64 = 12.;
const COMMERCIAL_REQUEST_NAMESPACE: u64 = 1 << 62;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    pub enabled: bool,
    pub operating_cash_floor: f64,
    pub surplus_share: f64,
    pub annual_required_return: f64,
    pub expected_loss_fraction: f64,
    pub underwriting: super::underwriting::Policy,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            enabled: false,
            operating_cash_floor: OPERATING_CASH_FLOOR,
            surplus_share: LENDER_SURPLUS_SHARE,
            annual_required_return: ANNUAL_REQUIRED_RETURN,
            expected_loss_fraction: SHIPMENT_LOSS_ASSUMPTION,
            underwriting: Default::default(),
        }
    }
}
impl Policy {
    pub fn validate(&self) -> Result<()> {
        self.underwriting.validate()?;
        ensure!(
            self.operating_cash_floor.is_finite() && self.operating_cash_floor >= 0.,
            "invalid commercial reserve"
        );
        ensure!(
            [self.surplus_share, self.annual_required_return]
                .iter()
                .all(|x| x.is_finite() && (0. ..=1.).contains(x))
                && self.expected_loss_fraction.is_finite()
                && (0. ..1.).contains(&self.expected_loss_fraction),
            "invalid commercial lending policy"
        );
        Ok(())
    }
}
impl History {
    /// Cost of missing non-food recipe inputs after planned local output. This is
    /// an opening quote, not a reservation or a promise of delivery/production.
    pub fn commercial_input_costs(&self) -> Vec<f64> {
        let Some(catalog) = &self.economy_catalog else {
            return vec![0.; self.sites.len()];
        };
        self.sites
            .iter()
            .map(|site| {
                if site.abandoned {
                    return 0.;
                }
                let mut net = [0_f64; GOODS];
                for (i, recipe) in catalog.recipes.iter().enumerate() {
                    let batches = f64::from(site.economy.orders[i]);
                    for (g, needed) in net.iter_mut().enumerate() {
                        *needed +=
                            batches * (f64::from(recipe.input[g]) - f64::from(recipe.output[g]));
                    }
                }
                net.iter()
                    .enumerate()
                    .filter(|(g, _)| *g != FOOD)
                    .map(|(g, needed)| {
                        (needed - f64::from(site.economy.goods[g])).max(0.)
                            * f64::from(site.economy.prices[g])
                    })
                    .sum()
            })
            .collect()
    }
    /// After production planning, before enterprise funding. Trading counterparties
    /// lend only existing surplus; no household debt or new commercial payee is created.
    pub fn commercial_credit_month(&mut self) -> Result<usize> {
        let policy = self.credit.commercial_policy.clone();
        policy.validate()?;
        if !policy.enabled || self.credit.commercial_decided_month == Some(self.month) {
            return Ok(0);
        }
        ensure!(
            self.credit
                .commercial_decided_month
                .is_none_or(|m| m < self.month),
            "commercial credit moved backwards"
        );
        let costs = self.commercial_input_costs();
        ensure!(
            costs.iter().all(|x| x.is_finite() && *x >= 0.),
            "invalid commercial input quote"
        );
        let mut evidence = self.export_credit_evidence(policy.expected_loss_fraction)?;
        let mut requests = Vec::new();
        let mut offers = Vec::new();
        let mut offered = BTreeSet::new();
        // Split each town's one funding gap across its eligible receipts. The
        // resolver then applies shared borrower, lender and pledged-source caps.
        let mut source_counts = vec![0_usize; self.sites.len()];
        for e in &evidence {
            if let Account::Town(id) = e.beneficiary {
                source_counts[id as usize] += 1;
            }
        }
        for (index, e) in evidence.iter_mut().enumerate() {
            let Account::Town(seller) = e.beneficiary else {
                continue;
            };
            let super::RepaymentSource::Export {
                contract,
                payment_month,
            } = e.source
            else {
                continue;
            };
            let buyer = self.export_identities[contract as usize].buyer;
            if self.sites[buyer as usize].abandoned {
                continue;
            }
            let gap =
                (costs[seller as usize] - self.credit_account_cash(Account::Town(seller))?).max(0.);
            if gap == 0. {
                continue;
            }
            // The next production commitment remains senior to repayment.
            e.operating_costs += costs[seller as usize];
            let lender = Account::Town(buyer);
            if offered.insert(buyer) {
                let cash = self.credit_account_cash(lender)?;
                let reserve = costs[buyer as usize].max(policy.operating_cash_floor);
                offers.push(Offer {
                    lender,
                    month: self.month,
                    cash,
                    operating_reserve: reserve,
                    offered_principal: (cash - reserve).max(0.) * policy.surplus_share,
                    minimum_annual_rate: policy.annual_required_return,
                });
            }
            let Some(maturity) = payment_month.checked_add(1) else {
                continue;
            };
            let years = f64::from(maturity - self.month) / MONTHS_PER_YEAR;
            // Expected principal-and-interest recovery must cover the lender's
            // annual return; a short risky bridge requires a higher annual quote.
            let rate = (policy.annual_required_return + policy.expected_loss_fraction / years)
                / (1. - policy.expected_loss_fraction);
            if rate > 1. {
                continue;
            }
            requests.push(Request {
                id: COMMERCIAL_REQUEST_NAMESPACE | index as u64,
                month: self.month,
                principal: gap / source_counts[seller as usize] as f64,
                terms: Terms {
                    lender,
                    borrower: e.beneficiary,
                    currency: SHARED_CURRENCY,
                    source: e.source,
                    annual_simple_rate: rate,
                    maturity_month: maturity,
                    grace_months: Terms::default_grace_months(),
                },
            });
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
        self.credit.commercial_decided_month = Some(self.month);
        Ok(count)
    }
}
