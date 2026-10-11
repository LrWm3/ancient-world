//! Signed stock-market choices, independent of live holdings and officer identity.
use crate::{
    marketplace::{MarketId, Side},
    minting::orders::{self, Quote},
    model::*,
};
use std::collections::{BTreeMap, BTreeSet};

pub mod discovery;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub side: Side,
    /// Admissible per-lot prices and stock targets (buy) or floors (sell).
    pub limits: BTreeSet<i32>,
    pub holdings: BTreeSet<i32>,
    pub max_lots: i32,
}

pub(crate) fn validate_options(
    w: &World,
    agent: AgentId,
    options: &BTreeMap<MarketId, Options>,
) -> Result<(), String> {
    if options.is_empty() {
        return Ok(());
    }
    let combinations = options.values().try_fold(0_usize, |n, o| {
        n.checked_add(o.limits.len().checked_mul(o.holdings.len())?)
    });
    if combinations.is_none_or(|n| n > super::MAX_PROGRAMS) {
        return Err("stock authority exceeds bounded search".into());
    }
    let c = w
        .minting
        .as_ref()
        .ok_or("stock authority requires a market engine")?;
    let p = c
        .order_policy
        .as_ref()
        .ok_or("stock authority requires order policy")?;
    let mut candidate = p.clone();
    let peer = candidate
        .peer_stocks
        .as_mut()
        .ok_or("stock authority requires peer stock book")?;
    peer.quotes.retain(|q| q.agent != agent);
    for (&market, o) in options {
        if o.limits.is_empty() || o.holdings.is_empty() || o.max_lots <= 0 {
            return Err("empty or inactive stock authority".into());
        }
        peer.quotes.push(Quote {
            agent,
            market,
            side: o.side,
            limit: *o.limits.first().unwrap(),
            holding: *o.holdings.first().unwrap(),
            max_lots: Some(o.max_lots),
        });
    }
    orders::validate(w, c, &candidate)?;
    for (&market, o) in options {
        for &limit in &o.limits {
            for &holding in &o.holdings {
                let q = candidate
                    .peer_stocks
                    .as_mut()
                    .unwrap()
                    .quotes
                    .iter_mut()
                    .find(|q| q.agent == agent && q.market == market)
                    .unwrap();
                q.limit = limit;
                q.holding = holding;
                orders::validate(w, c, &candidate)?;
            }
        }
    }
    Ok(())
}

/// Structural and constitutional validation. Legal eligibility is checked when
/// issuing a choice and again by normal market execution, not granted here.
pub fn validate_quotes(w: &World, agent: AgentId, quotes: &[Quote]) -> Result<(), String> {
    let a = w
        .agency
        .get(&agent)
        .and_then(|c| c.operating_agreement())
        .ok_or("stock program requires a signed operating agreement")?;
    if a.stock_markets.is_empty()
        || quotes.iter().any(|q| {
            q.agent != agent
                || a.stock_markets.get(&q.market).is_none_or(|o| {
                    q.side != o.side
                        || !o.limits.contains(&q.limit)
                        || !o.holdings.contains(&q.holding)
                        || q.max_lots.is_none_or(|n| n <= 0 || n > o.max_lots)
                })
        })
    {
        return Err("stock program exceeds signed authority".into());
    }
    candidate(w, agent, quotes).map(|_| ())
}

fn candidate(w: &World, agent: AgentId, quotes: &[Quote]) -> Result<orders::Policy, String> {
    let c = w.minting.as_ref().ok_or("missing stock market engine")?;
    let mut p = c.order_policy.clone().ok_or("missing stock order policy")?;
    let peer = p.peer_stocks.as_mut().ok_or("missing peer stock book")?;
    peer.quotes.retain(|q| q.agent != agent);
    peer.quotes.extend_from_slice(quotes);
    peer.quotes.sort_by_key(|q| (q.market, q.agent));
    orders::validate(w, c, &p)?;
    Ok(p)
}

pub(crate) fn apply(w: &mut World, agent: AgentId, quotes: &[Quote]) -> Result<(), String> {
    validate_quotes(w, agent, quotes)?;
    let p = candidate(w, agent, quotes)?;
    w.minting.as_mut().unwrap().order_policy = Some(p);
    Ok(())
}
