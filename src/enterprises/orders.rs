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
}

impl History {
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
        });
        Ok(id)
    }
}

pub(super) fn validate(enterprises: &Enterprises, h: &History) -> Result<()> {
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
