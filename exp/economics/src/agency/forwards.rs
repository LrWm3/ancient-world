//! Opt-in startup delivery authority. Spot-market consent never implies credit
//! exposure: both organizations must separately sign these finite terms.
use super::organization::Agreement;
use crate::{marketplace::MarketId, model::*};

const MIN_DELIVERY_MONTHS: u32 = 2;
const MAX_DELIVERY_MONTHS: u32 = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// One market lot, prepaid at this exact total price in its payment resource.
    pub prepayment: i32,
    /// Months from acceptance to delivery; admission and work begin next month.
    pub delivery_months: u32,
}

pub(crate) fn validate_options(w: &World, agreement: &Agreement) -> Result<(), String> {
    for (&market, option) in &agreement.forwards {
        let listing = listing(w, market).ok_or("forward authority needs a stock listing")?;
        if !agreement.stock_markets.contains_key(&market)
            || option.prepayment <= 0
            || !(MIN_DELIVERY_MONTHS..=MAX_DELIVERY_MONTHS).contains(&option.delivery_months)
            || !w
                .resources
                .iter()
                .any(|r| r.id == listing.goods.resource && r.kind == ResourceKind::Stock)
            || !w
                .resources
                .iter()
                .any(|r| r.id == listing.payment && r.kind == ResourceKind::Stock)
            || listing.goods.resource == listing.payment
            || w.storage
                .weights
                .get(&listing.payment)
                .copied()
                .unwrap_or(0)
                != 0
        {
            return Err("invalid signed startup forward authority".into());
        }
    }
    Ok(())
}

fn listing(w: &World, market: MarketId) -> Option<&crate::marketplace::Market> {
    let mint = w.minting.as_ref()?;
    crate::marketplace::venue(w, mint.venue)?
        .markets
        .iter()
        .find(|m| m.id == market)
}
