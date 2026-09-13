//! Funded future service fees; escrow is town-owned until completed work earns it.
use super::{Enterprises, History, SERVICE_QUOTE_MULTIPLIER};
use crate::{
    civilization::Site,
    household_economy::{deposit, withdraw},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const MAX_ORDER_HORIZON_MONTHS: u32 = 12;
const ORDER_LEDGER_RELATIVE_TOLERANCE: f64 = 1e-9;
const PROCUREMENT_CASH_RESERVE: f64 = 100.;
const PROCUREMENT_SURPLUS_SHARE: f64 = 0.25;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Procurement {
    pub enabled: bool,
    /// Include due escrowed work in shift demand; existing cash/labor caps still apply.
    #[serde(default)]
    pub contract_staffing: bool,
    /// Cap shifts against current recipe orders, not just past activity.
    #[serde(default)]
    pub demand_staffing: bool,
    #[serde(default)]
    pub staffing_observations: Vec<StaffingObservation>,
    pub cash_reserve: f64,
    pub surplus_share: f64,
    pub last_month: Option<u32>,
    pub claims: Vec<ProcurementClaim>,
}
impl Default for Procurement {
    fn default() -> Self {
        Self {
            enabled: false,
            contract_staffing: false,
            demand_staffing: false,
            staffing_observations: vec![],
            cash_reserve: PROCUREMENT_CASH_RESERVE,
            surplus_share: PROCUREMENT_SURPLUS_SHARE,
            last_month: None,
            claims: vec![],
        }
    }
}
/// Reserve-boundary demand before cash and participant matching. This is a
/// forecast ceiling, not a materials reservation or a completed-work receipt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaffingObservation {
    pub month: u32,
    pub firm: u32,
    pub unconstrained_work: f64,
    pub ordered_work: f64,
    /// None in older records that did not forecast current input feasibility.
    #[serde(default)]
    pub feasible_work: Option<f64>,
    pub requested_work: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcurementClaim {
    pub firm: u32,
    pub requested_fee: f64,
    pub allowance: f64,
    pub funded: f64,
    pub order: Option<u64>,
}
impl Procurement {
    fn validate(&self, month: u32) -> Result<()> {
        ensure!(
            self.cash_reserve.is_finite()
                && self.cash_reserve >= 0.
                && self.surplus_share.is_finite()
                && (0. ..=1.).contains(&self.surplus_share)
                && self.last_month.is_none_or(|m| m <= month),
            "invalid service procurement policy"
        );
        for observation in &self.staffing_observations {
            ensure!(
                observation.month <= month
                    && [
                        observation.unconstrained_work,
                        observation.ordered_work,
                        observation.requested_work
                    ]
                    .iter()
                    .all(|v| v.is_finite() && *v >= 0.)
                    && observation.requested_work <= observation.unconstrained_work
                    && observation.requested_work <= observation.ordered_work
                    && observation.feasible_work.is_none_or(|work| {
                        work.is_finite()
                            && work >= 0.
                            && work <= observation.ordered_work
                            && observation.requested_work <= work
                    }),
                "invalid workshop demand observation"
            );
        }
        for claim in &self.claims {
            ensure!(
                [claim.requested_fee, claim.allowance, claim.funded]
                    .iter()
                    .all(|v| v.is_finite() && *v >= 0.)
                    && claim.funded <= claim.allowance
                    && claim.allowance <= claim.requested_fee,
                "invalid service procurement allowance"
            );
        }
        Ok(())
    }
}

/// Observations at the due-month execution boundary, not inferred shortfall causes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionObservation {
    /// Request after the operator cash cap; not unconstrained labor demand.
    pub requested_labor: f64,
    pub funded_labor: f64,
    pub completed_labor: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceOrder {
    pub id: u64,
    pub firm: u32,
    pub site: u32,
    pub posted: u32,
    pub due: u32,
    pub currency: crate::credit::CurrencyId,
    pub requested_work: f64,
    pub funded_work: f64,
    pub price_per_work: f64,
    pub funded: f64,
    pub escrow: f64,
    pub completed_work: f64,
    pub paid: f64,
    pub refunded: f64,
    pub settled: Option<u32>,
    /// Absent in older archives, on cancellation, or when the due boundary was missed.
    #[serde(default)]
    pub execution: Option<ExecutionObservation>,
}

impl History {
    /// Close-phase commitments for next month, based on the last production plan.
    /// Requests share a site cash envelope before any escrow is withdrawn.
    pub fn procure_workshop_services(&mut self) -> Result<usize> {
        let Some(enterprises) = &self.enterprises else {
            return Ok(0);
        };
        let policy = enterprises.procurement.clone();
        policy.validate(self.month)?;
        if !enterprises.enabled || !policy.enabled || policy.last_month == Some(self.month) {
            return Ok(0);
        }
        let Some(catalog) = &self.economy_catalog else {
            return Ok(0);
        };
        let due = self
            .month
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("service date overflow"))?;
        let costs = self.commercial_input_costs();
        let mut requests = vec![];
        let mut totals = vec![0_f64; self.sites.len()];
        for f in &enterprises.firms {
            let town = &self.sites[f.site as usize];
            if f.closed.is_some()
                || town.abandoned
                || enterprises
                    .orders
                    .iter()
                    .any(|o| o.firm == f.id && o.settled.is_none())
            {
                continue;
            }
            let installed = f64::from(town.economy.workshop_types[f.family as usize][0]);
            if installed <= 0. {
                continue;
            }
            let units = f.leased_units.min(installed);
            let demand =
                super::industrial_order_work(catalog, &town.economy.orders, f.family as usize);
            let work = (demand * units / installed)
                .min(units * f64::from(crate::production::WORKSHOP_WORKER_MONTHS_PER_UNIT));
            let price = f
                .service_rate
                .unwrap_or(f.wage_rate * SERVICE_QUOTE_MULTIPLIER);
            let quote = work * price;
            if quote <= 0. || price <= 0. {
                continue;
            }
            ensure!(quote.is_finite(), "invalid procurement quote");
            totals[f.site as usize] += quote;
            requests.push((f.site, f.family, f.id, price, quote));
        }
        ensure!(
            totals.iter().all(|v| v.is_finite()),
            "procurement demand overflow"
        );
        requests.sort_by_key(|r| (r.0, r.1, r.2));
        let budgets: Vec<f64> = self
            .sites
            .iter()
            .enumerate()
            .map(|(i, s)| {
                (f64::from(s.economy.finance[0]) - costs[i].max(policy.cash_reserve)).max(0.)
                    * policy.surplus_share
            })
            .collect();
        let mut claims = vec![];
        let mut count = 0;
        for (site, _, firm, price, quote) in requests {
            let allowance = quote * (budgets[site as usize] / totals[site as usize]).min(1.);
            let opening = self.sites[site as usize].economy.finance[0];
            // Round retained f32 cash upward so escrow cannot exceed its grant.
            let retained = (f64::from(opening) - allowance).max(0.);
            let mut closing = retained as f32;
            if f64::from(closing) < retained {
                closing = f32::from_bits(closing.to_bits() + 1);
            }
            let fee = (f64::from(opening) - f64::from(closing)).max(0.);
            let mut preview = opening;
            let actual = withdraw(&mut preview, (fee / price) * price);
            let mut claim = ProcurementClaim {
                firm,
                requested_fee: quote,
                allowance,
                funded: 0.,
                order: None,
            };
            if actual > 0. && actual <= allowance {
                let id = self.fund_workshop_order(firm, fee / price, due)?;
                claim.funded = self.enterprises.as_ref().unwrap().orders[id as usize].funded;
                claim.order = Some(id);
                count += 1;
            }
            claims.push(claim);
        }
        let procurement = &mut self.enterprises.as_mut().unwrap().procurement;
        procurement.last_month = Some(self.month);
        procurement.claims = claims;
        procurement.validate(self.month)?;
        Ok(count)
    }
    /// A funded fee is still conditional on future work. Costs retain current
    /// payroll quotes and rent; neither escrow nor loans are operator income.
    pub fn service_order_credit_evidence(
        &self,
        loss: f64,
    ) -> Result<Vec<crate::credit::underwriting::Evidence>> {
        ensure!(
            loss.is_finite() && (0. ..=1.).contains(&loss),
            "invalid service loss assumption"
        );
        let Some(enterprises) = &self.enterprises else {
            return Ok(vec![]);
        };
        validate(enterprises, self)?;
        if !enterprises.enabled {
            return Ok(vec![]);
        }
        let refined = self
            .resolution
            .as_ref()
            .is_some_and(|r| r.workshop_individual);
        let mut evidence = vec![];
        for order in &enterprises.orders {
            let firm = &enterprises.firms[order.firm as usize];
            let town = &self.sites[order.site as usize];
            if order.settled.is_some()
                || order.due < self.month
                || firm.closed.is_some()
                || town.abandoned
            {
                continue;
            }
            let units = firm.leased_units.min(f64::from(
                town.economy.workshop_types[firm.family as usize][0],
            ));
            let work = order
                .funded_work
                .min(units * f64::from(crate::production::WORKSHOP_WORKER_MONTHS_PER_UNIT));
            if work <= 0. {
                continue;
            }
            let reference = super::workshop_reference_wage(town);
            let wage = reference
                * if refined {
                    firm.wage_policy
                        .as_ref()
                        .map_or(1., |p| p.pending.unwrap_or(p.multiplier))
                } else {
                    1.
                };
            let rent = units
                * super::RENT_REFERENCE_WORK_MONTHS_PER_UNIT
                * reference
                * f64::from(order.due - self.month + 1);
            evidence.push(crate::credit::underwriting::Evidence {
                work_funding: Some(crate::credit::underwriting::WorkFunding {
                    opening_cash: firm.cash,
                    fixed_cost: rent,
                    cost_per_work: wage,
                    maximum_work: work,
                }),
                source: crate::credit::RepaymentSource::ServiceOrder {
                    order: order.id,
                    payment_month: order.due,
                },
                beneficiary: crate::credit::Account::Operator(firm.id),
                observed_month: self.month,
                expected_receipts: (work * order.price_per_work).min(order.escrow) * (1. - loss),
                operating_costs: work * wage + rent,
                expected_loss_fraction: loss,
            });
        }
        Ok(evidence)
    }

    /// Explicit opt-in order at a completed history boundary. This reserves money,
    /// not workers or materials, and cannot finance work already performed.
    pub fn fund_workshop_order(&mut self, firm: u32, work: f64, due: u32) -> Result<u64> {
        ensure!(work.is_finite() && work > 0., "invalid service work");
        ensure!(
            due > self.month && due - self.month <= MAX_ORDER_HORIZON_MONTHS,
            "invalid service payment month"
        );
        let enterprises = self
            .enterprises
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no enterprises"))?;
        ensure!(enterprises.enabled, "enterprises disabled");
        let operator = enterprises
            .firms
            .get(firm as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown operator"))?;
        ensure!(
            operator.closed.is_none() && !self.sites[operator.site as usize].abandoned,
            "service operator unavailable"
        );
        ensure!(
            !enterprises
                .orders
                .iter()
                .any(|o| o.firm == firm && o.settled.is_none()),
            "operator already has a pending service order"
        );
        let price = operator
            .service_rate
            .unwrap_or(operator.wage_rate * SERVICE_QUOTE_MULTIPLIER);
        let quote = work * price;
        ensure!(
            price.is_finite() && price > 0. && quote.is_finite(),
            "invalid service quote"
        );
        let pool = &mut self.sites[operator.site as usize].economy.finance[0];
        ensure!(pool.is_finite() && *pool > 0., "no service funding");
        let funded = withdraw(pool, quote);
        ensure!(funded > 0., "service funding below account precision");
        let id = enterprises.orders.len() as u64;
        enterprises.orders.push(ServiceOrder {
            id,
            firm,
            site: operator.site,
            posted: self.month,
            due,
            currency: crate::credit::SHARED_CURRENCY,
            requested_work: work,
            funded_work: funded / price,
            price_per_work: price,
            funded,
            escrow: funded,
            completed_work: 0.,
            paid: 0.,
            refunded: 0.,
            settled: None,
            execution: None,
        });
        Ok(id)
    }
}

pub(super) fn validate(enterprises: &Enterprises, h: &History) -> Result<()> {
    enterprises.procurement.validate(h.month)?;
    let mut observed = std::collections::BTreeSet::new();
    for observation in &enterprises.procurement.staffing_observations {
        ensure!(
            (observation.firm as usize) < enterprises.firms.len()
                && observed.insert(observation.firm),
            "invalid staffing observation firm"
        );
    }
    for claim in &enterprises.procurement.claims {
        ensure!(
            (claim.firm as usize) < enterprises.firms.len(),
            "invalid procurement firm"
        );
        if let Some(id) = claim.order {
            ensure!(
                enterprises
                    .orders
                    .get(id as usize)
                    .is_some_and(|o| o.firm == claim.firm
                        && o.funded == claim.funded
                        && Some(o.posted) == enterprises.procurement.last_month),
                "procurement receipt differs from escrow"
            );
        } else {
            ensure!(claim.funded == 0., "unrecorded procurement funding");
        }
    }
    let mut pending = std::collections::BTreeSet::new();
    for (id, order) in enterprises.orders.iter().enumerate() {
        ensure!(
            order.id == id as u64
                && enterprises
                    .firms
                    .get(order.firm as usize)
                    .is_some_and(|f| f.site == order.site)
                && order.currency == crate::credit::SHARED_CURRENCY,
            "invalid service order identity"
        );
        ensure!(
            order.posted <= h.month
                && order.due > order.posted
                && order.due - order.posted <= MAX_ORDER_HORIZON_MONTHS
                && order
                    .settled
                    .is_none_or(|m| m >= order.posted && m <= h.month),
            "invalid service order date"
        );
        ensure!(
            [
                order.requested_work,
                order.funded_work,
                order.price_per_work,
                order.funded,
                order.escrow,
                order.completed_work,
                order.paid,
                order.refunded
            ]
            .iter()
            .all(|x| x.is_finite() && *x >= 0.)
                && order.price_per_work > 0.,
            "invalid service order amounts"
        );
        if let Some(observed) = &order.execution {
            ensure!(
                order.settled == Some(order.due)
                    && [
                        observed.requested_labor,
                        observed.funded_labor,
                        observed.completed_labor
                    ]
                    .iter()
                    .all(|v| v.is_finite() && *v >= 0.)
                    && observed.completed_labor.min(order.funded_work) == order.completed_work,
                "invalid service order execution observation"
            );
        }
        let tolerance = ORDER_LEDGER_RELATIVE_TOLERANCE * (1. + order.funded);
        ensure!(
            (order.funded - order.escrow - order.paid - order.refunded).abs() <= tolerance
                && (order.funded_work * order.price_per_work - order.funded).abs() <= tolerance
                && (order.completed_work * order.price_per_work - order.paid).abs() <= tolerance
                && order.completed_work <= order.funded_work,
            "service order ledger does not reconcile"
        );
        ensure!(
            order.settled.is_some()
                || (pending.insert(order.firm) && order.paid == 0. && order.refunded == 0.),
            "duplicate or prematurely paid service order"
        );
    }
    Ok(())
}

/// Return work covered by the order and earned money, indexed by firm. Ordinary
/// invoices must omit this work. Unrepresentable f32 refunds remain owned escrow.
pub(super) fn settle(
    enterprises: &mut Enterprises,
    sites: &mut [Site],
    month: u32,
) -> (Vec<f64>, Vec<f64>) {
    let mut covered = vec![0.; enterprises.firms.len()];
    let mut earned = covered.clone();
    for order in &mut enterprises.orders {
        let firm = &enterprises.firms[order.firm as usize];
        let site = &mut sites[order.site as usize];
        if order.settled.is_none()
            && (month >= order.due || firm.closed.is_some() || site.abandoned)
        {
            if month == order.due && firm.closed.is_none() && !site.abandoned {
                let work = f64::from(site.economy.enterprise_used[firm.family as usize]);
                order.execution = Some(ExecutionObservation {
                    requested_labor: firm.last_requested_work,
                    funded_labor: firm.last_funded_work,
                    completed_labor: work,
                });
                order.completed_work = work.min(order.funded_work).max(0.);
                let payment = (order.completed_work * order.price_per_work).min(order.escrow);
                order.escrow -= payment;
                order.paid += payment;
                covered[order.firm as usize] = order.completed_work;
                earned[order.firm as usize] = payment;
            }
            order.settled = Some(month);
        }
        if order.settled.is_some() {
            let refund = deposit(&mut site.economy.finance[0], order.escrow);
            order.escrow -= refund;
            order.refunded += refund;
        }
    }
    (covered, earned)
}
