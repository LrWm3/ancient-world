//! Signed stock-market choices, independent of live holdings and officer identity.
use crate::{
    marketplace::{MarketId, Side},
    minting::orders::{self, Quote},
    model::*,
};
use std::collections::{BTreeMap, BTreeSet};

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
