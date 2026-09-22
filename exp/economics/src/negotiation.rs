//! Bounded bilateral quote discovery, separate from physical settlement.
use crate::{
    finance,
    marketplace::{self, Side},
    model::*,
    opportunities, storage,
};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_QUOTE_ROUNDS: u32 = 64;
const BUYER: AgentId = 88;
const SELLER: AgentId = 89;
const GRAIN_LOT: i32 = 2;
const OPENING_COINS: i32 = 100;
const BUYER_STORAGE: i32 = 2;
const BUYER_LIMIT: i32 = 50;
const SELLER_LIMIT: i32 = 30;
const OPENING_BID: i32 = 20;
const OPENING_ASK: i32 = 60;
const CONCESSION_TICKS: i32 = 5;
const EXAMPLE_ROUNDS: u32 = 10;
pub const MARKETPLACE: AgentId = 90;
pub const GRAIN_MARKET: marketplace::MarketId = 1;
const PRICE_TICK: i32 = 1;

/// A first quote policy, not ZIP. Prices are integer payment units per whole lot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuotePolicy {
    Fixed,
    Concede { ticks: i32 },
}
impl QuotePolicy {
    fn next(self, quote: i32, limit: i32, buying: bool) -> i32 {
        let Self::Concede { ticks } = self else {
            return quote;
        };
        if buying {
            quote.saturating_add(ticks).min(limit)
        } else {
            quote.saturating_sub(ticks).max(limit)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trader {
    pub agent: AgentId,
    /// Private maximum payment for buyers; minimum receipt for sellers.
    pub limit: i32,
    pub opening_quote: i32,
    pub policy: QuotePolicy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub marketplace: AgentId,
    pub market: marketplace::MarketId,
    pub month: u32,
    pub buyer: Trader,
    pub seller: Trader,
    pub goods: Amount,
    pub payment: ResourceId,
    pub max_rounds: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quotes {
    pub round: u32,
    pub bid: i32,
    pub ask: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Traded { price: i32 },
    NoAgreement,
    UnsupportedMarket,
    Ineligible,
    InsufficientGoods,
    InsufficientPayment,
    InsufficientStorage,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub marketplace: AgentId,
    pub market: marketplace::MarketId,
    pub buyer: AgentId,
    pub seller: AgentId,
    pub month: u32,
    pub quotes: Vec<Quotes>,
    pub outcome: Outcome,
}

pub fn validate(world: &World) -> Result<(), String> {
    let Some(s) = &world.negotiation else {
        return Ok(());
    };
    let stock = |id| {
        world
            .resources
            .iter()
            .any(|r| r.id == id && r.kind == ResourceKind::Stock)
    };
    let trader = |t: &Trader| {
        world.agents.iter().any(|a| a.id == t.agent)
            && t.limit > 0
            && t.opening_quote > 0
            && match t.policy {
                QuotePolicy::Fixed => true,
                QuotePolicy::Concede { ticks } => ticks > 0,
            }
    };
    if marketplace::venue(world, s.marketplace).is_none()
        || s.month == 0
        || s.max_rounds == 0
        || s.max_rounds > MAX_QUOTE_ROUNDS
        || !trader(&s.buyer)
        || !trader(&s.seller)
        || s.buyer.agent == s.seller.agent
        || s.buyer.opening_quote > s.buyer.limit
        || s.seller.opening_quote < s.seller.limit
        || s.goods.quantity <= 0
        || !stock(s.goods.resource)
        || !stock(s.payment)
        || s.goods.resource == s.payment
    {
        return Err("invalid negotiation terms".into());
    }
    // This pilot owns one Acquire window. Composition with other acquisition
    // drivers needs a shared reservation boundary before it can be enabled.
    if world.market.is_some()
        || world.competition.is_some()
        || world.pool_market.is_some()
        || !world.households.is_empty()
        || !world.offers.is_empty()
        || !world.bids.is_empty()
        || !world.access_offers.is_empty()
    {
        return Err("negotiation pilot requires an isolated acquisition driver".into());
    }
    Ok(())
}

fn effects(state: &State, s: &Session, price: i32) -> Result<Vec<Effect>, String> {
    let mut result = finance::exchange_payment(
        s.seller.agent,
        s.buyer.agent,
        s.goods.clone(),
        state.balance(s.seller.agent, s.goods.resource),
    )?;
    result.extend(finance::exchange_payment(
        s.buyer.agent,
        s.seller.agent,
        Amount::new(s.payment, price),
        state.balance(s.buyer.agent, s.payment),
    )?);
    Ok(result)
}

/// Reads one immutable acquisition boundary. Quotes reserve nothing; only a
/// mutually acceptable, funded whole-lot exchange emits effects.
pub fn evaluate(world: &World, state: &State) -> Result<Option<Round>, String> {
    validate(world)?;
    marketplace::validate(world, state)?;
    let Some(s) = world
        .negotiation
        .as_ref()
        .filter(|s| s.month == state.month && state.phase == Phase::Acquire)
    else {
        return Ok(None);
    };
    let mut result = Round {
        marketplace: s.marketplace,
        market: s.market,
        buyer: s.buyer.agent,
        seller: s.seller.agent,
        month: state.month,
        quotes: vec![],
        outcome: Outcome::NoAgreement,
    };
    if [s.buyer.agent, s.seller.agent]
        .iter()
        .any(|a| !marketplace::eligible(world, state, s.marketplace, *a))
    {
        result.outcome = Outcome::Ineligible;
        return Ok(Some(result));
    }
    let Some(market) = marketplace::supported(world, s) else {
        result.outcome = Outcome::UnsupportedMarket;
        return Ok(Some(result));
    };
    let mut bid = marketplace::opening(state, s, Side::Buy);
    let mut ask = marketplace::opening(state, s, Side::Sell);
    for round in 1..=s.max_rounds {
        result.quotes.push(Quotes { round, bid, ask });
        if bid >= ask {
            // Midpoint rounded down to a payment tick. Always within both quotes.
            let price = ask + ((bid - ask) / market.price_tick / 2) * market.price_tick;
            result.outcome = if state.balance(s.seller.agent, s.goods.resource) < s.goods.quantity {
                Outcome::InsufficientGoods
            } else if state.balance(s.buyer.agent, s.payment) < price {
                Outcome::InsufficientPayment
            } else if !storage::fits(
                world,
                &storage::usage(world, &state.balances),
                &effects(state, s, price)?,
            ) {
                Outcome::InsufficientStorage
            } else {
                Outcome::Traded { price }
            };
            break;
        }
        // Each side uses only its own limit and policy after the public rejection.
        // Neither side's update reads the other's private limit.
        let next_bid = s.buyer.policy.next(bid, s.buyer.limit, true);
        let next_ask = s.seller.policy.next(ask, s.seller.limit, false);
        if (next_bid, next_ask) == (bid, ask) {
            break;
        }
        (bid, ask) = (next_bid, next_ask);
    }
    Ok(Some(result))
}

pub fn transactions(
    world: &World,
    state: &State,
    round: &Option<Round>,
) -> Result<Vec<Transaction>, String> {
    let Some(Round {
        outcome: Outcome::Traded { price },
        ..
    }) = round
    else {
        return Ok(vec![]);
    };
    let s = world
        .negotiation
        .as_ref()
        .ok_or("missing negotiation terms")?;
    Ok(vec![Transaction {
        cause: format!(
            "marketplace {} market {}: negotiated stock exchange {} -> {} at {}",
            s.marketplace, s.market, s.seller.agent, s.buyer.agent, price
        ),
        effects: effects(state, s, *price)?,
        process: None,
        technique_use: None,
        trade: None,
        stock_trade: None,
        forward: None,
        delivery: None,
        royalty: None,
    }])
}

pub fn validate_batch(world: &World, state: &State, batch: &Batch) -> Result<(), String> {
    let expected = evaluate(world, state)?;
    if batch.negotiation != expected {
        return Err("missing or altered negotiation receipt".into());
    }
    if world.negotiation.is_some()
        && state.phase == Phase::Acquire
        && batch.transactions != transactions(world, state, &expected)?
    {
        return Err("exchange differs from negotiated terms".into());
    }
    Ok(())
}

/// Controlled one-lot exchange: one buyer and one seller, no state price setter.
pub fn scenario() -> (World, State) {
    let (mut world, mut state) = crate::scenario::baseline();

    world.agents.push(Agent {
        id: SELLER,
        name: "grain seller".into(),
    });
    world.agents.push(Agent {
        id: MARKETPLACE,
        name: "grain marketplace".into(),
    });
    world.transaction_policy = Some(opportunities::Policy {
        authority: crate::scenario::STATE_AGENT,
        membership_offers: vec![],
        membership_permissions: BTreeSet::new(),
        agent_types: BTreeMap::from([
            (BUYER, opportunities::PERSON_TYPE),
            (SELLER, opportunities::PERSON_TYPE),
            (MARKETPLACE, marketplace::MARKETPLACE_TYPE),
            (crate::scenario::STATE_AGENT, opportunities::STATE_TYPE),
        ]),
        permissions: BTreeSet::from([(
            opportunities::PERSON_TYPE,
            opportunities::Action::StockTrade,
        )]),
    });
    world.marketplaces.push(marketplace::Marketplace {
        agent: MARKETPLACE,
        required_type: opportunities::PERSON_TYPE,
        markets: vec![marketplace::Market {
            id: GRAIN_MARKET,
            goods: Amount::new(crate::scenario::GRAIN, GRAIN_LOT),
            payment: crate::scenario::TOKEN,
            price_tick: PRICE_TICK,
        }],
    });
    world.assets.clear();
    world.rights.clear();
    world.definitions.clear();
    world.participants.clear();
    state.balances.clear();
    world.resources.retain(|r| r.id == crate::scenario::GRAIN);
    world.resources.push(Resource {
        id: crate::scenario::TOKEN,
        name: "coin ticks".into(),
        kind: ResourceKind::Stock,
    });
    world.storage.weights.insert(crate::scenario::GRAIN, 1);
    world.storage.capacities.insert(BUYER, BUYER_STORAGE);
    state
        .balances
        .insert((BUYER, crate::scenario::TOKEN), OPENING_COINS);
    state
        .balances
        .insert((SELLER, crate::scenario::GRAIN), GRAIN_LOT);
    world.negotiation = Some(Session {
        marketplace: MARKETPLACE,
        market: GRAIN_MARKET,
        month: 1,
        buyer: Trader {
            agent: BUYER,
            limit: BUYER_LIMIT,
            opening_quote: OPENING_BID,
            policy: QuotePolicy::Concede {
                ticks: CONCESSION_TICKS,
            },
        },
        seller: Trader {
            agent: SELLER,
            limit: SELLER_LIMIT,
            opening_quote: OPENING_ASK,
            policy: QuotePolicy::Concede {
                ticks: CONCESSION_TICKS,
            },
        },
        goods: Amount::new(crate::scenario::GRAIN, GRAIN_LOT),
        payment: crate::scenario::TOKEN,
        max_rounds: EXAMPLE_ROUNDS,
    });
    (world, state)
}
