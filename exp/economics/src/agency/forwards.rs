//! Opt-in startup delivery authority. Spot-market consent never implies credit
//! exposure: both organizations must separately sign these finite terms.
use super::organization::Agreement;
use crate::{marketplace::MarketId, model::*};

pub mod assessment;
pub mod execution;
pub mod review;

const MAX_PROPOSALS: usize = 64;
const MIN_DELIVERY_MONTHS: u32 = 3;
const MAX_DELIVERY_MONTHS: u32 = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// One market lot, prepaid at this exact total price in its payment resource.
    pub prepayment: i32,
    /// Months from publication to delivery; admission is next month; conditional work starts the following Open.
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub market: MarketId,
    pub terms: crate::forward::direct::Terms,
    /// Each party's own discovered operating program, not a joint objective.
    pub buyer_program: u32,
    pub seller_program: u32,
}

/// Bounded, stable discovery. A signed buy target can request an intermediate
/// even when the buyer's ultimate objective is cash. Productive sellers need no
/// opening stock, but this is only a candidate, never a feasibility assertion.
pub fn candidates(w: &World, s: &State) -> Result<Vec<Proposal>, String> {
    use crate::marketplace::Side;
    super::validate(w)?;
    let id = w
        .prepaid_deliveries
        .iter()
        .map(|t| t.id)
        .chain(s.exchange.forwards.keys().copied())
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("forward ID exhausted")?;
    let mut result = vec![];
    for (&buyer, controller) in &w.agency {
        if !controller.discover_programs {
            continue;
        }
        let Some(buy) = controller.operating_agreement() else {
            continue;
        };
        for (&market, option) in &buy.forwards {
            let bounds = &buy.stock_markets[&market];
            let listing = listing(w, market).ok_or("unknown forward market")?;
            if bounds.side != Side::Buy
                || s.balance(buyer, listing.goods.resource) >= *bounds.holdings.last().unwrap()
                || s.balance(buyer, listing.payment) < option.prepayment
                || w.prepaid_deliveries.iter().any(|t| {
                    [t.buyer, t.seller].contains(&buyer)
                        && t.goods.resource == listing.goods.resource
                })
            {
                continue;
            }
            for (&seller, controller) in &w.agency {
                if !controller.discover_programs {
                    continue;
                }
                let Some(sell) = controller.operating_agreement() else {
                    continue;
                };
                if seller == buyer
                    || sell.forwards.get(&market) != Some(option)
                    || sell.stock_markets[&market].side != Side::Sell
                    || w.prepaid_deliveries.iter().any(|t| {
                        [t.buyer, t.seller].contains(&seller)
                            && t.goods.resource == listing.goods.resource
                    })
                {
                    continue;
                }
                for buyer_program in productive_programs(w, buyer, market, Side::Buy)? {
                    for seller_program in productive_programs(w, seller, market, Side::Sell)? {
                        if result.len() == MAX_PROPOSALS {
                            return Err("startup forward catalog exceeds bounded search".into());
                        }
                        result.push(Proposal {
                            market,
                            terms: crate::forward::direct::Terms {
                                id,
                                buyer,
                                seller,
                                month: s.month.checked_add(1).ok_or("forward date overflow")?,
                                due: s
                                    .month
                                    .checked_add(option.delivery_months)
                                    .ok_or("forward date overflow")?,
                                goods: listing.goods.clone(),
                                prepayment: Amount::new(listing.payment, option.prepayment),
                            },
                            buyer_program,
                            seller_program,
                        });
                    }
                }
            }
        }
    }
    Ok(result)
}

fn productive_programs(
    w: &World,
    agent: AgentId,
    market: MarketId,
    side: crate::marketplace::Side,
) -> Result<Vec<u32>, String> {
    use super::Command;
    let resource = listing(w, market)
        .ok_or("unknown forward listing")?
        .goods
        .resource;
    Ok(super::discovery::programs(w, agent)?.into_iter().filter_map(|(id, p)| {
        let quotes = p.commands.iter().any(|c| matches!(c, Command::PeerStockQuotes(q) if q.iter().any(|q| q.market == market && q.side == side)));
        let work = p.commands.iter().any(|c| matches!(c, Command::WorkTargets(orders) if orders.iter().any(|o| {
            let d = w.definition(o.definition);
            if side == crate::marketplace::Side::Sell {
                d.outputs.iter().any(|a| a.resource == resource)
            } else {
                d.stages.iter().flat_map(|stage| &stage.entry_inputs).any(|a| a.resource == resource)
            }
        })));
        (quotes && work).then_some(id)
    }).collect())
}
