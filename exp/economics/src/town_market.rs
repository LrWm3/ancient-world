//! One monthly book: opening admission, need orders, price priority, atomic settlement.
use crate::{
    acquisition::Resources,
    marketplace::{self, Side},
    model::*,
    need_orders,
    negotiation::{self, Outcome, QuotePolicy, Session, Trader},
};
use std::collections::{BTreeMap, BTreeSet};

const MAX_TRADERS: usize = 32;
const SECOND_BUYER: AgentId = 91;
const SECOND_SELLER: AgentId = 92;
const TOWN: AgentId = 93;
const EXAMPLE_REACH: u32 = 2;
const EXAMPLE_STOCK: i32 = 10;
const EXAMPLE_COINS: i32 = 100;
const EXAMPLE_BID: i32 = 50;
const EXAMPLE_ASK: i32 = 30;
const EXAMPLE_PRICE_STEP: i32 = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub side: Side,
    pub trader: Trader,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub venue: AgentId,
    pub market: marketplace::MarketId,
    pub town: AgentId,
    /// One-dimensional abstract distance, not travel or transport.
    pub reach: u32,
    pub reserve: need_orders::Policy,
    /// One whole-lot order per registered participant per month.
    pub traders: Vec<Entry>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Book {
    pub positions: BTreeMap<AgentId, i32>,
    pub admission: Option<Admission>,
    pub history: Vec<Round>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    pub month: u32,
    pub eligible: BTreeSet<AgentId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub agent: AgentId,
    pub side: Side,
    pub quote: i32,
    pub protected: BTreeMap<Account, i128>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub session: Session,
    pub round: negotiation::Round,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub month: u32,
    pub orders: Vec<Order>,
    pub attempts: Vec<Attempt>,
    pub transactions: Vec<Transaction>,
    /// Last completed match price this month; None is not a zero or a stale price.
    pub posted_price: Option<i32>,
    pub volume: i32,
    pub unfilled_buy: i32,
    pub unfilled_sell: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Boundary {
    Admission(Admission),
    Market(Round),
}

fn session(
    c: &Config,
    m: &marketplace::Market,
    buyer: Trader,
    seller: Trader,
    month: u32,
) -> Session {
    Session {
        marketplace: c.venue,
        market: c.market,
        month,
        buyer,
        seller,
        goods: m.goods.clone(),
        payment: m.payment,
        max_rounds: 1,
    }
}
fn catalog<'a>(world: &'a World, c: &Config) -> Result<&'a marketplace::Market, String> {
    marketplace::venue(world, c.venue)
        .and_then(|v| v.markets.iter().find(|m| m.id == c.market))
        .ok_or("unknown town market".into())
}
fn pair(world: &World, c: &Config, t: &Entry, month: u32) -> Result<Session, String> {
    let other = c
        .traders
        .iter()
        .filter(|e| e.side != t.side)
        .min_by_key(|e| e.trader.agent)
        .ok_or("market requires both sides")?;
    let (b, s) = if t.side == Side::Buy {
        (&t.trader, &other.trader)
    } else {
        (&other.trader, &t.trader)
    };
    Ok(session(c, catalog(world, c)?, b.clone(), s.clone(), month))
}
fn template(world: &World, s: Session) -> World {
    let mut w = world.clone();
    w.town_market = None;
    w.negotiation = Some(s);
    w.need_orders = None;
    w
}
pub fn validate(world: &World) -> Result<(), String> {
    let Some(c) = &world.town_market else {
        return Ok(());
    };
    if world.negotiation.is_some()
        || world.need_orders.is_some()
        || world.credit.is_some()
        || world.market.is_some()
        || world.pool_market.is_some()
        || world.competition.is_some()
        || !world.households.is_empty()
        || !world.offers.is_empty()
        || !world.bids.is_empty()
        || !world.access_offers.is_empty()
        || !world.agreements.is_empty()
        || world.work_choice.is_some()
    {
        return Err("town market requires an isolated acquisition driver".into());
    }
    if !(2..=MAX_TRADERS).contains(&c.traders.len()) || !world.agents.iter().any(|a| a.id == c.town)
    {
        return Err("invalid town market participants or town".into());
    }
    let mut ids = BTreeSet::new();
    for t in &c.traders {
        if matches!(t.trader.policy, QuotePolicy::Concede { .. }) {
            return Err("town quote policy must be Fixed or Zip".into());
        }
        if !ids.insert(t.trader.agent) {
            return Err("duplicate town trader".into());
        }
        let s = pair(world, c, t, 1)?;
        let mut w = template(world, s.clone());
        w.need_orders = Some(c.reserve.clone());
        negotiation::validate(&w)?;
        if marketplace::supported(&w, &s).is_none() {
            return Err("unsupported town market terms".into());
        }
    }
    Ok(())
}
pub(crate) fn validate_state(world: &World, state: &State) -> Result<(), String> {
    let book = &state.town_market;
    let Some(c) = &world.town_market else {
        return if book == &Book::default() {
            Ok(())
        } else {
            Err("town market state without configuration".into())
        };
    };
    if book
        .positions
        .keys()
        .any(|id| !world.agents.iter().any(|a| a.id == *id))
        || book.admission.as_ref().is_some_and(|a| {
            a.month == 0
                || a.month > state.month
                || a.eligible
                    .iter()
                    .any(|id| !c.traders.iter().any(|t| t.trader.agent == *id))
        })
        || (state.phase != Phase::Open
            && book
                .admission
                .as_ref()
                .is_none_or(|a| a.month != state.month))
        || book.history.windows(2).any(|r| r[0].month >= r[1].month)
        || book.history.iter().any(|r| {
            r.month == 0
                || r.month > state.month
                || (r.month == state.month && matches!(state.phase, Phase::Open | Phase::Acquire))
        })
    {
        return Err("invalid town market state boundary".into());
    }
    Ok(())
}

pub fn admission(world: &World, state: &State) -> Result<Admission, String> {
    let c = world.town_market.as_ref().ok_or("missing town market")?;
    let eligible = c
        .traders
        .iter()
        .filter(|t| {
            let a = t.trader.agent;
            !state.terminal.contains_key(&a)
                && marketplace::eligible(world, state, c.venue, a)
                && state
                    .town_market
                    .positions
                    .get(&c.town)
                    .zip(state.town_market.positions.get(&a))
                    .is_some_and(|(town, p)| town.abs_diff(*p) <= c.reach)
        })
        .map(|t| t.trader.agent)
        .collect();
    Ok(Admission {
        month: state.month,
        eligible,
    })
}

pub fn evaluate(world: &World, state: &State) -> Result<Round, String> {
    validate(world)?;
    if state.phase != Phase::Acquire {
        return Err("town matching requires Acquire".into());
    }
    let c = world.town_market.as_ref().ok_or("missing town market")?;
    let m = catalog(world, c)?;
    let admitted = state
        .town_market
        .admission
        .as_ref()
        .filter(|a| a.month == state.month)
        .ok_or("missing current opening admission")?;
    let mut resources = Resources::opening(world, state);
    let mut orders = Vec::new();
    for t in &c.traders {
        if !admitted.eligible.contains(&t.trader.agent)
            || state.terminal.contains_key(&t.trader.agent)
            || !marketplace::eligible(world, state, c.venue, t.trader.agent)
        {
            continue;
        }
        let s = pair(world, c, t, state.month)?;
        let d = need_orders::generate_for(world, state, &resources, &c.reserve, &s)?;
        let exists = if t.side == Side::Buy {
            d.buy.is_some()
        } else {
            d.sell.is_some()
        };
        if exists {
            let quote = marketplace::learning(state, &s, t.side).map_or_else(
                || marketplace::opening(state, &s, t.side),
                |l| l.quote(t.trader.limit, m.price_tick, t.side),
            );
            orders.push(Order {
                agent: t.trader.agent,
                side: t.side,
                quote,
                protected: d
                    .protected
                    .into_iter()
                    .filter(|((a, _), _)| *a == t.trader.agent)
                    .collect(),
            });
        }
    }
    orders.sort_by_key(|o| (o.side, o.agent));
    let mut buyers: Vec<_> = orders.iter().filter(|o| o.side == Side::Buy).collect();
    let mut sellers: Vec<_> = orders.iter().filter(|o| o.side == Side::Sell).collect();
    buyers.sort_by_key(|o| (std::cmp::Reverse(o.quote), o.agent));
    sellers.sort_by_key(|o| (o.quote, o.agent));
    let mut filled = BTreeSet::new();
    let mut pricing_state = state.clone();
    let mut result = Round {
        month: state.month,
        orders: vec![],
        attempts: vec![],
        transactions: vec![],
        posted_price: None,
        volume: 0,
        unfilled_buy: 0,
        unfilled_sell: 0,
    };
    for buyer in &buyers {
        for seller in &sellers {
            if filled.contains(&seller.agent) {
                continue;
            }
            // Quotes are fixed for this monthly book. An uncrossed best remaining
            // ask means no lower-ranked ask can cross this buyer's bid.
            let trader = |id| {
                c.traders
                    .iter()
                    .find(|e| e.trader.agent == id)
                    .unwrap()
                    .trader
                    .clone()
            };
            let s = session(c, m, trader(buyer.agent), trader(seller.agent), state.month);
            let w = template(world, s.clone());
            let mut spendable = resources.clone();
            for (o, r) in [(buyer, m.payment), (seller, m.goods.resource)] {
                let account = (o.agent, r);
                let balance = spendable.available.entry(account).or_default();
                *balance = (i128::from(*balance) - o.protected.get(&account).copied().unwrap_or(0))
                    .max(0) as i32;
            }
            let mut round =
                negotiation::evaluate_with(&w, state, &spendable)?.ok_or("missing town match")?;
            // Orders keep their opening quotes for this book. Learning accumulates
            // public events in match order and affects only subsequent books.
            round.buyer_learning = marketplace::learning(&pricing_state, &s, Side::Buy);
            round.seller_learning = marketplace::learning(&pricing_state, &s, Side::Sell);
            for event in &round.events {
                negotiation::observe(
                    &s,
                    m.price_tick,
                    &mut round.buyer_learning,
                    &mut round.seller_learning,
                    event,
                );
            }
            marketplace::record(&mut pricing_state, &s, &round);
            if let Outcome::Traded { price } = round.outcome {
                let transactions = negotiation::transactions(&w, state, &Some(round.clone()))?;
                resources.reserve(world, &transactions)?;
                result.transactions.extend(transactions);
                result.volume = result
                    .volume
                    .checked_add(m.goods.quantity)
                    .ok_or("market volume overflow")?;
                result.posted_price = Some(price);
                filled.insert(buyer.agent);
                filled.insert(seller.agent);
            }
            result.attempts.push(Attempt { session: s, round });
            if filled.contains(&buyer.agent) || buyer.quote < seller.quote {
                break;
            }
        }
    }
    result.unfilled_buy =
        i32::try_from(buyers.iter().filter(|o| !filled.contains(&o.agent)).count())
            .unwrap()
            .checked_mul(m.goods.quantity)
            .ok_or("order volume overflow")?;
    result.unfilled_sell = i32::try_from(
        sellers
            .iter()
            .filter(|o| !filled.contains(&o.agent))
            .count(),
    )
    .unwrap()
    .checked_mul(m.goods.quantity)
    .ok_or("order volume overflow")?;
    result.orders = orders;
    Ok(result)
}
pub(crate) fn validate_batch(world: &World, state: &State, batch: &Batch) -> Result<(), String> {
    let expected = if world.town_market.is_none() {
        None
    } else {
        match state.phase {
            Phase::Open => Some(Boundary::Admission(admission(world, state)?)),
            Phase::Acquire => Some(Boundary::Market(evaluate(world, state)?)),
            _ => None,
        }
    };
    if batch.town_market != expected {
        return Err("missing or altered town market receipt".into());
    }
    if let Some(Boundary::Market(r)) = expected
        && batch.transactions != r.transactions
    {
        return Err("altered town market settlement".into());
    }
    Ok(())
}
pub(crate) fn record(state: &mut State, boundary: &Option<Boundary>) {
    match boundary {
        Some(Boundary::Admission(a)) => state.town_market.admission = Some(a.clone()),
        Some(Boundary::Market(r)) => {
            for a in &r.attempts {
                marketplace::record(state, &a.session, &a.round)
            }
            state.town_market.history.push(r.clone());
        }
        None => {}
    }
}

pub fn scenario() -> (World, State) {
    use crate::scenario::{GRAIN, PERSON, TOKEN};
    let (mut w, mut s) = need_orders::scenario();
    let base = w.negotiation.take().unwrap();
    let reserve = w.need_orders.take().unwrap();
    let seller = base.seller.agent;
    for (id, name, source) in [
        (SECOND_BUYER, "second buyer", PERSON),
        (SECOND_SELLER, "second seller", seller),
    ] {
        w.agents.push(Agent {
            id,
            name: name.into(),
        });
        let mut p = w
            .participants
            .iter()
            .find(|p| p.agent == source)
            .unwrap()
            .clone();
        p.agent = id;
        w.participants.push(p);
        w.transaction_policy
            .as_mut()
            .unwrap()
            .agent_types
            .insert(id, crate::opportunities::PERSON_TYPE);
        w.storage.capacities.insert(id, EXAMPLE_STOCK);
    }
    w.agents.push(Agent {
        id: TOWN,
        name: "market town".into(),
    });
    let traders = [
        (PERSON, Side::Buy, EXAMPLE_BID),
        (SECOND_BUYER, Side::Buy, EXAMPLE_BID - EXAMPLE_PRICE_STEP),
        (seller, Side::Sell, EXAMPLE_ASK),
        (SECOND_SELLER, Side::Sell, EXAMPLE_ASK + EXAMPLE_PRICE_STEP),
    ]
    .into_iter()
    .map(|(agent, side, limit)| Entry {
        side,
        trader: Trader {
            agent,
            limit,
            opening_quote: limit,
            policy: QuotePolicy::Fixed,
        },
    })
    .collect();
    w.town_market = Some(Config {
        venue: base.marketplace,
        market: base.market,
        town: TOWN,
        reach: EXAMPLE_REACH,
        reserve,
        traders,
    });
    for id in [PERSON, SECOND_BUYER] {
        s.balances.insert((id, TOKEN), EXAMPLE_COINS);
    }
    for id in [seller, SECOND_SELLER] {
        s.balances.insert((id, GRAIN), EXAMPLE_STOCK);
    }
    s.town_market.positions = [PERSON, SECOND_BUYER, seller, SECOND_SELLER, TOWN]
        .into_iter()
        .map(|id| (id, 0))
        .collect();
    (w, s)
}
