//! Opt-in working-cash requests from planned production and funded receivables.
use super::{
    underwriting::{required_annual_rate, Offer, Request},
    Account, Terms, SHARED_CURRENCY,
};
use crate::{
    civilization::History,
    economy::{FOOD, GOODS},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const OPERATING_CASH_FLOOR: f64 = 100.;
const LENDER_SURPLUS_SHARE: f64 = 0.25;
const ANNUAL_REQUIRED_RETURN: f64 = 0.06;
const SHIPMENT_LOSS_ASSUMPTION: f64 = 0.05;
const COMMERCIAL_REQUEST_NAMESPACE: u64 = 1 << 62;
const SERVICE_REQUEST_NAMESPACE: u64 = 1 << 61;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    pub enabled: bool,
    #[serde(default)]
    pub service_orders: bool,
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
            service_orders: false,
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
/// The town has one opening operating requirement, even if several deliveries
/// can repay it. Allocate that shared requirement by expected proceeds; existing
/// source-specific costs remain attached to their source.
fn allocate_operating_costs(
    evidence: &mut [super::underwriting::Evidence],
    costs: &[f64],
) -> Result<()> {
    let mut receipts = BTreeMap::<u32, f64>::new();
    for e in evidence.iter() {
        let Account::Town(site) = e.beneficiary else {
            continue;
        };
        ensure!(
            e.expected_receipts.is_finite()
                && e.expected_receipts >= 0.
                && e.operating_costs.is_finite()
                && e.operating_costs >= 0.
                && costs
                    .get(site as usize)
                    .is_some_and(|c| c.is_finite() && *c >= 0.),
            "invalid commercial operating-cost allocation"
        );
        *receipts.entry(site).or_default() += e.expected_receipts;
    }
    ensure!(
        receipts.values().all(|v| v.is_finite()),
        "commercial receipts overflow"
    );
    for e in evidence {
        let Account::Town(site) = e.beneficiary else {
            continue;
        };
        let total = receipts[&site];
        if total > 0. {
            e.operating_costs += costs[site as usize] * (e.expected_receipts / total);
            ensure!(e.operating_costs.is_finite(), "commercial costs overflow");
        }
    }
    Ok(())
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
        allocate_operating_costs(&mut evidence, &costs)?;
        if policy.service_orders {
            evidence.extend(self.service_order_credit_evidence(policy.expected_loss_fraction)?);
        }
        let mut requests = Vec::new();
        let mut offers = Vec::new();
        let mut offered = BTreeSet::new();
        // Split each town's one funding gap across its eligible receipts. The
        // resolver then applies shared borrower, lender and pledged-source caps.
        for (index, e) in evidence.iter().enumerate() {
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
            // Expected principal-and-interest recovery must cover the lender's
            // annual return; a short risky bridge requires a higher annual quote.
            let rate = required_annual_rate(
                policy.annual_required_return,
                policy.expected_loss_fraction,
                maturity - self.month,
            );
            let terms = Terms {
                lender,
                borrower: e.beneficiary,
                currency: SHARED_CURRENCY,
                source: e.source,
                annual_simple_rate: rate,
                maturity_month: maturity,
                grace_months: Terms::default_grace_months(),
            };
            // Unfinanceable voyage duration/rate is a declined opportunity,
            // not invalid history. Use the same bounds as committed contracts.
            if terms.validate(self.month).is_err() {
                continue;
            }
            requests.push(Request {
                id: COMMERCIAL_REQUEST_NAMESPACE | index as u64,
                month: self.month,
                principal: gap,
                terms,
            });
        }
        // The local paying town can voluntarily bridge its operator from cash
        // left outside escrow. Keep these proposals in the same allocation round
        // as exports so a town cannot offer its surplus twice.
        for e in &evidence {
            let Account::Operator(firm) = e.beneficiary else {
                continue;
            };
            let super::RepaymentSource::ServiceOrder {
                order,
                payment_month,
            } = e.source
            else {
                continue;
            };
            let operator = &self.enterprises.as_ref().unwrap().firms[firm as usize];
            let payer = operator.site;
            let gap = (e.operating_costs - self.credit_account_cash(e.beneficiary)?).max(0.);
            if gap == 0. {
                continue;
            }
            let lender = Account::Town(payer);
            if offered.insert(payer) {
                let cash = self.credit_account_cash(lender)?;
                let reserve = costs[payer as usize].max(policy.operating_cash_floor);
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
            let terms = Terms {
                lender,
                borrower: e.beneficiary,
                currency: SHARED_CURRENCY,
                source: e.source,
                annual_simple_rate: required_annual_rate(
                    policy.annual_required_return,
                    policy.expected_loss_fraction,
                    maturity - self.month,
                ),
                maturity_month: maturity,
                grace_months: Terms::default_grace_months(),
            };
            if terms.validate(self.month).is_err() {
                continue;
            }
            requests.push(Request {
                id: SERVICE_REQUEST_NAMESPACE | order,
                month: self.month,
                principal: gap,
                terms,
            });
        }
        // Divide only among requests that survived duration/counterparty checks.
        // An unusable distant receipt must not consume another source's share of
        // the funding gap. Underwriting still caps each source and lender; this
        // does not reallocate rejected or capacity-limited grants afterward.
        let mut source_counts = BTreeMap::<Account, usize>::new();
        for request in &requests {
            *source_counts.entry(request.terms.borrower).or_default() += 1;
        }
        for request in &mut requests {
            request.principal /= source_counts[&request.terms.borrower] as f64;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credit::{
        underwriting::{resolve, Evidence},
        RepaymentSource,
    };

    fn sources() -> Vec<Evidence> {
        (0..2)
            .map(|contract| Evidence {
                work_funding: None,
                source: RepaymentSource::Export {
                    contract,
                    payment_month: 12,
                },
                beneficiary: Account::Town(0),
                observed_month: 1,
                expected_receipts: 100.,
                operating_costs: 0.,
                expected_loss_fraction: 0.,
            })
            .collect()
    }

    #[test]
    fn shared_cost_is_protected_once_without_double_pledging_proceeds() {
        let mut evidence = sources();
        allocate_operating_costs(&mut evidence, &[150.]).unwrap();
        assert_eq!(
            evidence.iter().map(|e| e.operating_costs).sum::<f64>(),
            150.
        );
        let offers = [Offer {
            lender: Account::Town(1),
            month: 1,
            cash: 1000.,
            operating_reserve: 100.,
            offered_principal: 900.,
            minimum_annual_rate: 0.,
        }];
        let requests: Vec<_> = evidence
            .iter()
            .enumerate()
            .map(|(id, e)| Request {
                id: id as u64,
                month: 1,
                principal: 25.,
                terms: Terms {
                    lender: Account::Town(1),
                    borrower: e.beneficiary,
                    currency: SHARED_CURRENCY,
                    source: e.source,
                    annual_simple_rate: 0.,
                    maturity_month: 13,
                    grace_months: 3,
                },
            })
            .collect();
        let policy = super::super::underwriting::Policy::default();
        let grants = resolve(1, &policy, &[], &offers, &evidence, &requests).unwrap();
        // 200 receipts - 150 operating costs = 50; half covers new debt.
        assert_eq!(grants.iter().map(|g| g.granted).sum::<f64>(), 25.);
        assert!(grants.iter().all(|g| g.granted == 12.5));
        let existing =
            crate::credit::Loan::record_disbursement(0, requests[0].terms.clone(), 1, 12.5)
                .unwrap();
        let repeated = resolve(1, &policy, &[existing], &offers, &evidence, &requests).unwrap();
        assert_eq!(repeated[0].granted, 0.);
        assert_eq!(repeated[1].granted, 12.5);
        let mut exhausted = sources();
        allocate_operating_costs(&mut exhausted, &[250.]).unwrap();
        assert!(resolve(1, &policy, &[], &offers, &exhausted, &requests)
            .unwrap()
            .iter()
            .all(|g| g.granted == 0.));
    }

    #[test]
    fn cost_shares_follow_receipts_and_preserve_local_costs_and_single_source() {
        let mut e = sources();
        e[1].expected_receipts = 300.;
        e[0].operating_costs = 2.;
        allocate_operating_costs(&mut e, &[120.]).unwrap();
        assert_eq!((e[0].operating_costs, e[1].operating_costs), (32., 90.));
        let mut reversed = sources();
        reversed[1].expected_receipts = 300.;
        reversed[0].operating_costs = 2.;
        reversed.reverse();
        allocate_operating_costs(&mut reversed, &[120.]).unwrap();
        reversed.reverse();
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::to_value(reversed).unwrap()
        );
        let mut single = vec![sources().remove(0)];
        allocate_operating_costs(&mut single, &[120.]).unwrap();
        assert_eq!(single[0].operating_costs, 120.);
        let mut missing = sources();
        assert!(allocate_operating_costs(&mut missing, &[]).is_err());
    }
}
