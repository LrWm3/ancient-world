//! A bounded market-income objective for the existing contributed-labor allocator.
use super::*;
use crate::{forecast::ForecastContext, marketplace::MarketId, negotiation::Outcome};

pub mod scenario;

const DEMAND_OBSERVATION_MONTHS: u32 = 6;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Forecast {
    pub observed_through: u32,
    pub through: u32,
    pub demand_caps: BTreeMap<MarketId, u32>,
    pub opening_coins: i32,
    pub closing_coins: i32,
    pub net_coins: i64,
    pub sales: BTreeMap<MarketId, i32>,
    pub purchases: BTreeMap<MarketId, i32>,
}

/// Compare the same cash objective for labor allocation and voluntary support.
/// A static reserve target permits leisure once needs and the buffer are covered.
pub(super) fn improves(household: &Agreement, baseline: &Forecast, candidate: &Forecast) -> bool {
    if let Some(target) = &household.governance.charter.cash_target {
        candidate.closing_coins.min(target.quantity) > baseline.closing_coins.min(target.quantity)
    } else {
        candidate.net_coins > baseline.net_coins
    }
}

/// One next-book hypothesis, not a receivable. Observe current balances and
/// completed book history; never execute another Productive phase recursively.
pub(super) fn project(
    world: &World,
    state: &State,
    productive: &Batch,
    household: AgentId,
) -> Result<Forecast, String> {
    let (world, opening) = ForecastContext::new(world, state).into_parts();
    let mut sim = needs::after_consumption(&world, &opening, productive)?;
    // A one-book income comparison holds the current accepted person choices.
    // Re-entering Plan at that book would recursively forecast this allocation.
    if world
        .production_market
        .as_ref()
        .is_some_and(|c| matches!(c.policy, crate::production_market::Policy::Plan))
    {
        let decision = state
            .town_market
            .history
            .iter()
            .find(|r| r.month == state.month)
            .and_then(|r| r.planning.as_ref());
        let choices = crate::production_market::choices(&world, decision);
        sim.world.production_market.as_mut().unwrap().policy =
            crate::production_market::Policy::Fixed(choices);
    }
    let config = world
        .town_market
        .as_ref()
        .ok_or("income forecast needs a town book")?;
    let venue = crate::marketplace::venue(&world, config.venue).unwrap();
    let payment = venue
        .markets
        .iter()
        .find(|m| m.id == config.market)
        .unwrap()
        .payment;
    let through = state
        .month
        .checked_add(1)
        .ok_or("income forecast date overflow")?;
    // Bound each predicted book by actual recent trades or unfilled bids. Neither
    // signal creates a buyer, liquidity or a commitment to repeat the demand.
    let caps: BTreeMap<_, _> = crate::town_market::listings(config)
        .iter()
        .map(|listing| {
            let lot = venue
                .markets
                .iter()
                .find(|m| m.id == listing.market)
                .unwrap()
                .goods
                .quantity;
            let demand = state
                .town_market
                .history
                .iter()
                .filter(|r| {
                    r.month <= state.month
                        && r.month >= through.saturating_sub(DEMAND_OBSERVATION_MONTHS)
                })
                .filter_map(|r| r.markets.get(&listing.market))
                .map(|r| {
                    ((i64::from(r.volume) + i64::from(r.unfilled_buy)) / i64::from(lot)) as u32
                })
                .max()
                .unwrap_or(0);
            (
                listing.market,
                listing.match_limit.map_or(demand, |cap| cap.min(demand)),
            )
        })
        .collect();
    let book = sim.world.town_market.as_mut().unwrap();
    book.match_limit = Some(caps[&book.market]);
    for listing in &mut book.additional {
        listing.match_limit = Some(caps[&listing.market]);
    }
    while sim.state.month < through || sim.state.phase != Phase::Acquire {
        if matches!(sim.state.phase, Phase::Productive | Phase::Consumption) {
            return Err("income forecast crossed its next-market boundary".into());
        }
        sim.step()?;
    }
    sim.step()?;
    let mut sales = BTreeMap::new();
    let mut purchases = BTreeMap::new();
    let receipt = sim
        .state
        .town_market
        .history
        .last()
        .ok_or("missing forecast town book")?;
    for attempt in &receipt.attempts {
        if matches!(attempt.round.outcome, Outcome::Traded { .. }) {
            let s = &attempt.session;
            if s.seller.agent == household {
                *sales.entry(s.market).or_default() += s.goods.quantity;
            }
            if s.buyer.agent == household {
                *purchases.entry(s.market).or_default() += s.goods.quantity;
            }
        }
    }
    let opening_coins = state.balance(household, payment);
    let closing_coins = sim.state.balance(household, payment);
    Ok(Forecast {
        observed_through: state.month,
        through,
        demand_caps: caps,
        opening_coins,
        closing_coins,
        net_coins: i64::from(closing_coins) - i64::from(opening_coins),
        sales,
        purchases,
    })
}
