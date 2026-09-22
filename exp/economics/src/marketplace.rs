//! Marketplace components attached to ordinary agents; no membership lifecycle.
use crate::{
    model::*,
    negotiation::{QuotePolicy, Round, Session},
    opportunities, zip,
};
use std::collections::{BTreeMap, BTreeSet};

pub type MarketId = u32;
pub const MARKETPLACE_TYPE: opportunities::AgentType = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Market {
    pub id: MarketId,
    pub goods: Amount,
    pub payment: ResourceId,
    /// Smallest quote increment, in payment units per whole lot.
    pub price_tick: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marketplace {
    pub agent: AgentId,
    pub required_type: opportunities::AgentType,
    /// Explicit catalog of facilitated exchanges, not owned inventory.
    pub markets: Vec<Market>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Buy,
    Sell,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pricing {
    pub policy: QuotePolicy,
    pub last_quote: i32,
    pub learning: Option<zip::Learning>,
    pub month: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Memory {
    pub pricing: BTreeMap<(AgentId, MarketId, Side), Pricing>,
    pub history: Vec<Round>,
}

pub fn venue(world: &World, agent: AgentId) -> Option<&Marketplace> {
    world.marketplaces.iter().find(|m| m.agent == agent)
}
pub fn eligible(world: &World, state: &State, marketplace: AgentId, participant: AgentId) -> bool {
    venue(world, marketplace).is_some_and(|m| {
        !state.terminal.contains_key(&marketplace)
            && !state.terminal.contains_key(&participant)
            && participant != marketplace
            && world
                .transaction_policy
                .as_ref()
                .is_some_and(|p| p.agent_types.get(&participant) == Some(&m.required_type))
            && opportunities::permits(world, state, participant, opportunities::Action::StockTrade)
    })
}
/// Discoverable trade catalog for an eligible participant. Does not reserve stock.
pub fn discover<'a>(
    world: &'a World,
    state: &State,
    marketplace: AgentId,
    participant: AgentId,
) -> Vec<&'a Market> {
    if !eligible(world, state, marketplace, participant) {
        return vec![];
    }
    let mut rows: Vec<_> = venue(world, marketplace).unwrap().markets.iter().collect();
    rows.sort_by_key(|m| m.id);
    rows
}
pub fn supported<'a>(world: &'a World, s: &Session) -> Option<&'a Market> {
    venue(world, s.marketplace)?.markets.iter().find(|m| {
        m.id == s.market
            && m.goods == s.goods
            && m.payment == s.payment
            && [&s.buyer, &s.seller].iter().all(|t| {
                t.opening_quote % m.price_tick == 0
                    && t.limit % m.price_tick == 0
                    && match t.policy {
                        QuotePolicy::Fixed | QuotePolicy::Zip(_) => true,
                        QuotePolicy::Concede { ticks } => ticks % m.price_tick == 0,
                    }
            })
    })
}
pub fn opening(state: &State, s: &Session, side: Side) -> i32 {
    let t = match side {
        Side::Buy => &s.buyer,
        Side::Sell => &s.seller,
    };
    let quote = state
        .marketplaces
        .get(&s.marketplace)
        .and_then(|m| m.pricing.get(&(t.agent, s.market, side)))
        .filter(|p| p.policy == t.policy)
        .map_or(t.opening_quote, |p| p.last_quote);
    match side {
        Side::Buy => quote.min(t.limit),
        Side::Sell => quote.max(t.limit),
    }
}
pub fn learning(state: &State, s: &Session, side: Side) -> Option<zip::Learning> {
    let t = match side {
        Side::Buy => &s.buyer,
        Side::Sell => &s.seller,
    };
    let QuotePolicy::Zip(config) = t.policy else {
        return None;
    };
    Some(
        state
            .marketplaces
            .get(&s.marketplace)
            .and_then(|m| m.pricing.get(&(t.agent, s.market, side)))
            .filter(|p| p.policy == t.policy)
            .and_then(|p| p.learning.clone())
            .unwrap_or_else(|| {
                zip::Learning::new(
                    config,
                    t.limit,
                    t.opening_quote,
                    &[
                        u64::from(s.marketplace),
                        u64::from(s.market),
                        u64::from(t.agent),
                        match side {
                            Side::Buy => 0,
                            Side::Sell => 1,
                        },
                    ],
                )
            }),
    )
}
/// Called only on staged state after the negotiation batch has been revalidated.
pub(crate) fn record(state: &mut State, s: &Session, round: &Round) {
    let memory = state.marketplaces.entry(s.marketplace).or_default();
    if let Some(q) = round.quotes.last() {
        for (t, side, quote) in [(&s.buyer, Side::Buy, q.bid), (&s.seller, Side::Sell, q.ask)] {
            memory.pricing.insert(
                (t.agent, s.market, side),
                Pricing {
                    learning: match side {
                        Side::Buy => round.buyer_learning.clone(),
                        Side::Sell => round.seller_learning.clone(),
                    },
                    policy: t.policy,
                    last_quote: quote,
                    month: round.month,
                },
            );
        }
    }
    memory.history.push(round.clone());
}
pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let mut venues = BTreeSet::new();
    for m in &world.marketplaces {
        if !venues.insert(m.agent) || !world.agents.iter().any(|a| a.id == m.agent) {
            return Err("invalid marketplace agent".into());
        }
        let mut markets = BTreeSet::new();
        for market in &m.markets {
            if !markets.insert(market.id)
                || market.goods.quantity <= 0
                || market.price_tick <= 0
                || market.goods.resource == market.payment
                || [market.goods.resource, market.payment].iter().any(|id| {
                    !world
                        .resources
                        .iter()
                        .any(|r| r.id == *id && r.kind == ResourceKind::Stock)
                })
            {
                return Err("invalid marketplace trade catalog".into());
            }
        }
    }
    for (id, memory) in &state.marketplaces {
        let m = venue(world, *id).ok_or("unknown marketplace memory")?;
        for ((agent, market, side), p) in &memory.pricing {
            let terms = m
                .markets
                .iter()
                .find(|m| m.id == *market)
                .ok_or("unknown pricing market")?;
            if match (&p.policy, &p.learning) {
                (QuotePolicy::Zip(config), Some(l)) => !config.valid() || !l.valid(*side),
                (QuotePolicy::Zip(_), None) => true,
                (_, Some(_)) => true,
                _ => false,
            } || !world.agents.iter().any(|a| a.id == *agent)
                || p.last_quote <= 0
                || p.last_quote % terms.price_tick != 0
                || p.month == 0
                || p.month > state.month
            {
                return Err("invalid marketplace pricing memory".into());
            }
        }
        if memory
            .history
            .iter()
            .any(|r| r.marketplace != *id || r.month > state.month)
        {
            return Err("invalid marketplace history".into());
        }
    }
    Ok(())
}
