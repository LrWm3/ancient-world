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
    let base: Vec<_> = options
        .iter()
        .map(|(&market, o)| {
            if o.limits.is_empty() || o.holdings.is_empty() || o.max_lots <= 0 {
                return Err("empty or inactive stock authority");
            }
            Ok(Quote {
                agent,
                market,
                side: o.side,
                limit: *o.limits.first().unwrap(),
                holding: *o.holdings.first().unwrap(),
                max_lots: Some(o.max_lots),
            })
        })
        .collect::<Result<_, _>>()?;
    for (&market, o) in options {
        for &limit in &o.limits {
            for &holding in &o.holdings {
                let mut quotes = base.clone();
                let q = quotes.iter_mut().find(|q| q.market == market).unwrap();
                q.limit = limit;
                q.holding = holding;
                let candidate = replace(w, agent, p, options, &quotes)?;
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
    let p = c
        .order_policy
        .as_ref()
        .ok_or("missing stock order policy")?;
    let a = w.agency[&agent]
        .operating_agreement()
        .ok_or("missing stock authority")?;
    let p = replace(w, agent, p, &a.stock_markets, quotes)?;
    orders::validate(w, c, &p)?;
    Ok(p)
}

pub(crate) fn apply(w: &mut World, agent: AgentId, quotes: &[Quote]) -> Result<(), String> {
    validate_quotes(w, agent, quotes)?;
    let p = candidate(w, agent, quotes)?;
    w.minting.as_mut().unwrap().order_policy = Some(p);
    Ok(())
}

/// Reserve schema compatibility, not cash or goods. Every signed quote slot must
/// coexist with peers' slots and still-live quotes before choices are accepted.
/// This conservative union intentionally forbids aliases even if mutually exclusive.
pub(crate) fn validate_joint_authorities(w: &World) -> Result<(), String> {
    let authorities: Vec<_> = w
        .agency
        .values()
        .filter_map(|c| c.operating_agreement())
        .filter(|a| !a.stock_markets.is_empty())
        .collect();
    if authorities.is_empty() {
        return Ok(());
    }
    let c = w.minting.as_ref().ok_or("missing stock market engine")?;
    let mut policy = c.order_policy.clone().ok_or("missing stock order policy")?;
    for a in authorities {
        let mut quotes: BTreeMap<_, _> = policy
            .peer_stocks
            .as_ref()
            .ok_or("missing peer stock book")?
            .quotes
            .iter()
            .filter(|q| q.agent == a.organization)
            .map(|q| (q.market, q.clone()))
            .collect();
        for (&market, o) in &a.stock_markets {
            quotes.insert(
                market,
                Quote {
                    agent: a.organization,
                    market,
                    side: o.side,
                    limit: *o.limits.first().ok_or("empty stock prices")?,
                    holding: *o.holdings.first().ok_or("empty stock holdings")?,
                    max_lots: Some(o.max_lots),
                },
            );
        }
        policy = replace(
            w,
            a.organization,
            &policy,
            &a.stock_markets,
            &quotes.into_values().collect::<Vec<_>>(),
        )?;
    }
    orders::validate(w, c, &policy)
}

/// Same signed program, existing execution routes. Issuer-input stock sales
/// remain contingent on mint procurement; independent stocks use the peer book.
fn replace(
    w: &World,
    agent: AgentId,
    original: &orders::Policy,
    options: &BTreeMap<MarketId, Options>,
    quotes: &[Quote],
) -> Result<orders::Policy, String> {
    let c = w.minting.as_ref().ok_or("missing stock market engine")?;
    let venue = crate::marketplace::venue(w, c.venue).ok_or("missing market venue")?;
    let mut p = original.clone();
    p.quotes
        .retain(|q| q.agent != agent || !options.contains_key(&q.market));
    p.peer_stocks
        .as_mut()
        .ok_or("missing peer stock book")?
        .quotes
        .retain(|q| q.agent != agent);
    for q in quotes {
        if p.input_limits.contains_key(&q.market) {
            let m = venue
                .markets
                .iter()
                .find(|m| m.id == q.market)
                .ok_or("missing input market")?;
            if q.side != Side::Sell
                || !w
                    .resources
                    .iter()
                    .any(|r| r.id == m.goods.resource && r.kind == ResourceKind::Stock)
            {
                return Err("signed issuer-input options require stock sales".into());
            }
            p.quotes.push(q.clone());
        } else {
            p.peer_stocks.as_mut().unwrap().quotes.push(q.clone());
        }
    }
    p.quotes.sort_by_key(|q| (q.market, q.agent));
    p.peer_stocks
        .as_mut()
        .unwrap()
        .quotes
        .sort_by_key(|q| (q.market, q.agent));
    Ok(p)
}

#[cfg(test)]
mod tests;
