//! Collection-linked issuance and fixed-price, finite stock exchanges.
use crate::model::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issuance {
    pub agreement: u32,
    pub token: ResourceId,
    pub collected_per_token: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bid {
    pub id: u32,
    pub buyer: AgentId,
    pub goods: Amount,
    pub payment: Amount,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StockTrade {
    pub bid: u32,
    pub seller: AgentId,
}

/// Seller-specific posted prices override the common peer bid price.
pub fn terms(world: &World, bid: &Bid, seller: AgentId) -> Result<Bid, String> {
    let mut result = bid.clone();
    if let Some(p) = world
        .market
        .as_ref()
        .and_then(|m| m.seller_prices.get(&(seller, bid.goods.resource)))
    {
        if p.goods <= 0 || p.coins <= 0 {
            return Err("invalid seller price".into());
        }
        let numerator = i64::from(bid.goods.quantity) * i64::from(p.coins);
        if numerator % i64::from(p.goods) != 0 {
            return Err("seller price exceeds quantity precision".into());
        }
        result.payment.quantity =
            i32::try_from(numerator / i64::from(p.goods)).map_err(|_| "seller price overflow")?;
    }
    Ok(result)
}

pub fn transaction(world: &World, state: &State, trade: StockTrade) -> Result<Transaction, String> {
    let bid = world
        .bids
        .iter()
        .find(|b| b.id == trade.bid)
        .ok_or("unknown stock bid")?;
    let bid = terms(world, bid, trade.seller)?;
    if state.phase != Phase::Acquire
        || trade.seller == bid.buyer
        || !world.agents.iter().any(|a| a.id == trade.seller)
        || state.terminal.contains_key(&trade.seller)
        || state.terminal.contains_key(&bid.buyer)
        || state.balance(trade.seller, bid.goods.resource) < bid.goods.quantity
        || state.balance(bid.buyer, bid.payment.resource) < bid.payment.quantity
    {
        return Err("unavailable or unfunded stock bid".into());
    }
    let effects = vec![
        Effect {
            account: (trade.seller, bid.goods.resource),
            delta: -bid.goods.quantity,
        },
        Effect {
            account: (bid.buyer, bid.goods.resource),
            delta: bid.goods.quantity,
        },
        Effect {
            account: (bid.buyer, bid.payment.resource),
            delta: -bid.payment.quantity,
        },
        Effect {
            account: (trade.seller, bid.payment.resource),
            delta: bid.payment.quantity,
        },
    ];
    if !crate::storage::fits(
        world,
        &crate::storage::usage(world, &state.balances),
        &effects,
    ) {
        return Err("stock exchange exceeds storage".into());
    }
    Ok(Transaction {
        cause: format!("fill stock bid {}", bid.id),
        effects,
        process: None,
        technique_use: None,
        trade: None,
        stock_trade: Some(trade),
        forward: None,
        delivery: None,
        royalty: None,
    })
}

pub fn validate(world: &World) -> Result<(), String> {
    let stock = |id| {
        world
            .resources
            .iter()
            .any(|r| r.id == id && r.kind == ResourceKind::Stock)
    };
    let mut agreements = BTreeSet::new();
    for rule in &world.issuance {
        if !agreements.insert(rule.agreement)
            || !stock(rule.token)
            || rule.collected_per_token <= 0
            || *world.storage.weights.get(&rule.token).unwrap_or(&0) != 0
            || !world
                .agreements
                .iter()
                .chain(&world.access_offers)
                .any(|a| a.id == rule.agreement && a.payment.resource != rule.token)
        {
            return Err("invalid collection-linked currency rule".into());
        }
    }
    let mut bids = BTreeSet::new();
    for bid in &world.bids {
        if !bids.insert(bid.id)
            || !world.agents.iter().any(|a| a.id == bid.buyer)
            || !stock(bid.goods.resource)
            || !stock(bid.payment.resource)
            || bid.goods.resource == bid.payment.resource
            || bid.goods.quantity <= 0
            || bid.payment.quantity <= 0
        {
            return Err("invalid stock bid".into());
        }
    }
    Ok(())
}
