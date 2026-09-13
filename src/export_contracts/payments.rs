//! Buyer-funded delivery escrow; resolution never creates sale income before arrival.
use crate::{
    civilization::History,
    credit::{
        accounts::{quote, Balance},
        CurrencyId, SHARED_CURRENCY,
    },
    economy::Cargo,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const PAYMENT_LEDGER_RELATIVE_TOLERANCE: f64 = 1e-10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Timing {
    #[default]
    Dispatch,
    Delivery,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Payment {
    pub id: u64,
    pub contract: u64,
    pub buyer: u32,
    pub seller: u32,
    pub currency: CurrencyId,
    pub dispatched_month: u32,
    pub expected_month: u32,
    pub original_kg: f32,
    pub funded: f64,
    pub escrow: f64,
    pub resolution_month: Option<u32>,
    pub delivered_kg: Option<f32>,
    pub seller_target: Option<f64>,
    pub seller_paid: f64,
    pub refunded: f64,
}
impl History {
    pub(crate) fn hold_export_payment(
        &mut self,
        contract: u64,
        buyer: u32,
        seller: u32,
        kg: f32,
        cost: f32,
        expected_month: u32,
    ) -> u64 {
        let id = self.export_payments.len() as u64;
        self.export_payments.push(Payment {
            id,
            contract,
            buyer,
            seller,
            currency: SHARED_CURRENCY,
            dispatched_month: self.month,
            expected_month,
            original_kg: kg,
            funded: f64::from(cost),
            escrow: f64::from(cost),
            resolution_month: None,
            delivered_kg: None,
            seller_target: None,
            seller_paid: 0.,
            refunded: 0.,
        });
        id
    }
    /// The contract's delivery condition is measured once. Spoiled/lost quantities
    /// refund the buyer; already dispatch-paid cargo has no deferred claim to erase.
    pub(crate) fn resolve_export_payment(&mut self, cargo: &Cargo, delivered_kg: f32) {
        let Some(id) = cargo.export_payment else {
            return;
        };
        let payment = &mut self.export_payments[id as usize];
        if payment.resolution_month.is_some() {
            return;
        }
        let delivered = delivered_kg.clamp(0., payment.original_kg);
        payment.delivered_kg = Some(delivered);
        payment.seller_target =
            Some(payment.funded * f64::from(delivered) / f64::from(payment.original_kg));
        payment.resolution_month = Some(self.month);
    }
    /// Retry representable residuals after resolution. A tiny remainder remains
    /// owned escrow, never discarded to force a perfectly round payout.
    pub(crate) fn settle_export_payments(&mut self) -> Result<()> {
        for p in &mut self.export_payments {
            let Some(target) = p.seller_target else {
                continue;
            };
            for (site, due, sale) in [
                (p.seller, (target - p.seller_paid).max(0.), true),
                (p.buyer, (p.funded - target - p.refunded).max(0.), false),
            ] {
                let account = &mut self.sites[site as usize].economy;
                let (remaining, received, paid) = quote(
                    Balance::Double(p.escrow),
                    Balance::Single(account.finance[0]),
                    due,
                )?;
                p.escrow = remaining.value();
                account.finance[0] = received.value() as f32;
                if sale {
                    p.seller_paid += paid;
                    account.finance[2] += paid as f32;
                } else {
                    p.refunded += paid;
                }
            }
        }
        Ok(())
    }
    pub fn validate_export_payments(&self) -> Result<()> {
        let mut in_transit = BTreeSet::new();
        for cargo in &self.cargo {
            if let Some(id) = cargo.export_payment {
                ensure!(
                    in_transit.insert(id)
                        && self
                            .export_payments
                            .get(id as usize)
                            .is_some_and(|p| p.id == id
                                && p.resolution_month.is_none()
                                && p.buyer == cargo.to
                                && p.seller == cargo.from
                                && cargo.kg.is_finite()
                                && cargo.kg >= 0.
                                && cargo.kg <= p.original_kg
                                && f64::from(cargo.paid) == p.funded
                                && self
                                    .export_identities
                                    .get(p.contract as usize)
                                    .is_some_and(|identity| identity.good == cargo.good)),
                    "invalid cargo payment reference"
                );
            }
        }
        for (id, p) in self.export_payments.iter().enumerate() {
            ensure!(
                p.id == id as u64
                    && p.currency == SHARED_CURRENCY
                    && p.dispatched_month <= self.month
                    && p.expected_month > p.dispatched_month
                    && p.original_kg.is_finite()
                    && p.original_kg > 0.
                    && p.funded.is_finite()
                    && p.funded > 0.
                    && [p.escrow, p.seller_paid, p.refunded]
                        .iter()
                        .all(|x| x.is_finite() && *x >= 0.)
                    && self
                        .export_identities
                        .get(p.contract as usize)
                        .is_some_and(|i| i.id == p.contract
                            && i.buyer == p.buyer
                            && i.seller == p.seller)
                    && (p.escrow + p.seller_paid + p.refunded - p.funded).abs()
                        <= PAYMENT_LEDGER_RELATIVE_TOLERANCE * p.funded.max(1.),
                "invalid delivery escrow ledger"
            );
            match (p.resolution_month, p.delivered_kg, p.seller_target) {
                (None, None, None) => ensure!(
                    in_transit.contains(&p.id) && p.seller_paid == 0. && p.refunded == 0.,
                    "orphan pending delivery payment"
                ),
                (Some(month), Some(kg), Some(target)) => ensure!(
                    month >= p.dispatched_month
                        && month <= self.month
                        && kg.is_finite()
                        && (0. ..=p.original_kg).contains(&kg)
                        && target == p.funded * f64::from(kg) / f64::from(p.original_kg)
                        && p.seller_paid <= target
                        && p.refunded <= p.funded - target,
                    "invalid resolved delivery payment"
                ),
                _ => anyhow::bail!("partial delivery resolution"),
            }
        }
        Ok(())
    }
}
