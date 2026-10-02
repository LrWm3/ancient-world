//! One monthly book: opening admission, need orders, price priority, atomic settlement.
use crate::{
    acquisition::Resources,
    marketplace::{self, Side},
    model::*,
    need_orders,
    negotiation::{self, Outcome, QuotePolicy, Session, Trader},
};
use std::collections::{BTreeMap, BTreeSet};

const MAX_LISTINGS: usize = 2;
pub(crate) const CONDITIONAL_SPOT_PARTIES: usize = 2;
pub(crate) const CONDITIONAL_SPOT_DELIVERIES: usize = 2;
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
/// A second listed good shares admission, money and storage with the first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listing {
    pub market: marketplace::MarketId,
    pub traders: Vec<Entry>,
    pub match_limit: Option<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearingPriority {
    MarketId,
    ReverseMarketId,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrderHorizon {
    /// Existing buying horizon (planner or one month) and separate reserve policy.
    #[default]
    Legacy,
    /// One horizon for both demand and protected stock; production planning is unchanged.
    Aligned(u32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub order_horizon: OrderHorizon,
    pub additional: Vec<Listing>,
    pub priority: ClearingPriority,
    /// Opt-in need-based side selection instead of the registered side.
    pub adaptive: bool,
    /// Scoped cap on completed lots, also used by observation-limited forecasts.
    pub match_limit: Option<u32>,
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
    pub market: marketplace::MarketId,
    pub agent: AgentId,
    pub side: Side,
    pub quote: i32,
    pub protected: BTreeMap<Account, i128>,
}
/// First decisive gate, not an exhaustive set of counterfactual failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderReason {
    Submitted,
    NotAdmitted,
    Inactive,
    Ineligible,
    PurchasePolicy,
    PlannerWithheld,
    OtherSideSelected,
    NoNeedImprovement,
    InsufficientOpeningStock,
    ProtectedStock,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderReceipt {
    pub household: Option<crate::household_governance::Authority>,
    pub market: marketplace::MarketId,
    pub agent: AgentId,
    pub side: Side,
    pub reason: OrderReason,
    pub resource: ResourceId,
    pub lot: i32,
    pub buy_months: u32,
    pub reserve_months: u32,
    /// Only populated when need/reserve evaluation actually ran.
    pub available: Option<i32>,
    pub protected: Option<i128>,
    pub deficits_before: Option<BTreeMap<ResourceId, i64>>,
    pub deficits_after: Option<BTreeMap<ResourceId, i64>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub session: Session,
    pub round: negotiation::Round,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderSelection {
    pub actor: AgentId,
    pub submit: BTreeSet<(marketplace::MarketId, Side)>,
}
/// Explicit submission masks collected from independently deciding participants.
pub type OrderSelections = BTreeMap<AgentId, BTreeSet<(marketplace::MarketId, Side)>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub conditional: Option<Vec<crate::cooperation::Delivery>>,
    pub selections: Option<Box<OrderSelections>>,
    pub selection: Option<Box<OrderSelection>>,
    pub cooperation: Option<Box<crate::cooperation::Boundary>>,
    pub planning: Option<Box<crate::production_market::Decision>>,
    pub month: u32,
    pub orders: Vec<Order>,
    pub order_receipts: Vec<OrderReceipt>,
    pub attempts: Vec<Attempt>,
    pub transactions: Vec<Transaction>,
    pub markets: BTreeMap<marketplace::MarketId, MarketResult>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MarketResult {
    /// Last completed match this month; no trade means None.
    pub posted_price: Option<i32>,
    pub volume: i32,
    pub unfilled_buy: i32,
    pub unfilled_sell: i32,
}
/// Canonical listing order is independent of catalog/vector order. Allocation
/// priority is explicit and changes only the order of reservations in Acquire.
pub fn listings(c: &Config) -> Vec<Config> {
    let mut primary = c.clone();
    primary.additional.clear();
    let mut rows = vec![primary.clone()];
    for l in &c.additional {
        let mut row = primary.clone();
        row.market = l.market;
        row.traders = l.traders.clone();
        row.match_limit = l.match_limit;
        rows.push(row);
    }
    rows.sort_by_key(|r| r.market);
    if c.priority == ClearingPriority::ReverseMarketId {
        rows.reverse();
    }
    rows
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
        .filter(|e| e.trader.agent != t.trader.agent && (c.adaptive || e.side != t.side))
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
    // Accepted land bills use Due/ClearArrears alongside acquisition; land-offer
    // discovery remains a separate driver.
    if world.negotiation.is_some()
        || world.need_orders.is_some()
        || world.credit.is_some()
        || world.market.is_some()
        || world.pool_market.is_some()
        || world.competition.is_some()
        || !world.offers.is_empty()
        || !world.bids.is_empty()
        || !world.access_offers.is_empty()
        || world.work_choice.is_some()
    {
        return Err("town market requires an isolated acquisition driver".into());
    }
    if !(2..=MAX_TRADERS).contains(&c.traders.len()) || !world.agents.iter().any(|a| a.id == c.town)
    {
        return Err("invalid town market participants or town".into());
    }
    if matches!(c.order_horizon, OrderHorizon::Aligned(months) if !(1..=need_orders::MAX_RESERVE_MONTHS).contains(&months))
    {
        return Err("aligned order horizon must be within reserve horizon bounds".into());
    }
    if c.additional.len() + 1 > MAX_LISTINGS {
        return Err("too many town listings".into());
    }
    let mut markets = BTreeSet::new();
    let mut goods = BTreeSet::new();
    let payment = catalog(world, c)?.payment;
    let participants: BTreeSet<_> = c.traders.iter().map(|t| t.trader.agent).collect();
    crate::households::market::validate(world)?;
    for row in listings(c) {
        let c = &row;
        let m = catalog(world, c)?;
        if !markets.insert(c.market)
            || !goods.insert(m.goods.resource)
            || m.payment != payment
            || c.traders
                .iter()
                .map(|t| t.trader.agent)
                .collect::<BTreeSet<_>>()
                != participants
        {
            return Err(
                "town listings require distinct goods, common payment and participants".into(),
            );
        }
        let mut ids = BTreeSet::new();
        for t in &c.traders {
            if c.adaptive
                && (t.trader.opening_quote != t.trader.limit
                    || t.trader.policy != QuotePolicy::Fixed)
            {
                return Err(
                    "adaptive sides initially require fixed quotes at supplied limits".into(),
                );
            }
            if matches!(t.trader.policy, QuotePolicy::Concede { .. }) {
                return Err("town quote policy must be Fixed or Zip".into());
            }
            if !ids.insert(t.trader.agent) {
                return Err("duplicate town trader".into());
            }
            let s = pair(world, c, t, 1)?;
            let mut w = template(world, s.clone());
            w.need_orders = Some(c.reserve.clone());
            negotiation::validate_terms(&w)?;
            if marketplace::supported(&w, &s).is_none() {
                return Err("unsupported town market terms".into());
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_state(world: &World, state: &State) -> Result<(), String> {
    crate::cooperation::validate_state(world, state)?;
    let book = &state.town_market;
    let Some(c) = &world.town_market else {
        return if book == &Book::default() {
            Ok(())
        } else {
            Err("town market state without configuration".into())
        };
    };
    let ids: BTreeSet<_> = listings(c).iter().map(|l| l.market).collect();
    for r in &book.history {
        if let Some(terms) = &r.conditional {
            let consents = conditional_consents(world, r.month, terms)?;
            if r.selection.is_some()
                || r.cooperation.is_some()
                || r.selections.as_deref() != Some(&consents)
                || spot_deliveries(r) != *terms
            {
                return Err("inconsistent conditional spot receipt".into());
            }
        }
    }
    if book.history.iter().any(|r| {
        r.markets.keys().copied().collect::<BTreeSet<_>>() != ids
            || r.markets.values().any(|m| {
                m.volume < 0
                    || m.unfilled_buy < 0
                    || m.unfilled_sell < 0
                    || (m.volume == 0) != m.posted_price.is_none()
                    || m.posted_price.is_some_and(|p| p <= 0)
            })
            || r.orders.iter().any(|o| !ids.contains(&o.market))
    }) {
        return Err("invalid town market observations".into());
    }
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
                && crate::recovery::active(world, &state.credit, a).is_none()
                && crate::households::market::active(world, state, a)
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
    evaluate_with(world, state, &Resources::opening(world, state), state)
}

pub(crate) fn evaluate_with(
    world: &World,
    state: &State,
    opening: &Resources,
    planning_state: &State,
) -> Result<Round, String> {
    evaluate_selected_with(world, state, opening, planning_state, None, None)
}

/// Opt-in submission of a subset of one actor's ordinary eligible need orders.
/// Counterparty orders, quotes, reserve policy and clearing remain authoritative.
pub fn evaluate_selected(
    world: &World,
    state: &State,
    selection: &OrderSelection,
) -> Result<Round, String> {
    evaluate_selected_with(
        world,
        state,
        &Resources::opening(world, state),
        state,
        Some(selection),
        None,
    )
}

/// Clear only the masks supplied by all participants. An empty mask withholds
/// that actor's orders; there is no automatic counterparty submission here.
pub fn evaluate_selections(
    world: &World,
    state: &State,
    selections: &OrderSelections,
) -> Result<Round, String> {
    let participants: BTreeSet<_> = world.participants.iter().map(|p| p.agent).collect();
    if selections.keys().copied().collect::<BTreeSet<_>>() != participants
        || world.town_market.as_ref().is_some_and(|c| {
            c.traders
                .iter()
                .any(|t| !selections.contains_key(&t.trader.agent))
        })
    {
        return Err("explicit town book requires every participant's order mask".into());
    }
    evaluate_selected_with(
        world,
        state,
        &Resources::opening(world, state),
        state,
        None,
        Some(selections),
    )
}
fn evaluate_selected_with(
    world: &World,
    state: &State,
    opening: &Resources,
    planning_state: &State,
    selection: Option<&OrderSelection>,
    selections: Option<&OrderSelections>,
) -> Result<Round, String> {
    validate(world)?;
    let mut masks = selections.cloned().unwrap_or_default();
    if let Some(selection) = selection {
        masks.insert(selection.actor, selection.submit.clone());
    }
    for (&actor, submit) in &masks {
        if crate::acquisition::shared(world)
            || world.production_market.is_some()
            || !world.households.is_empty()
        {
            return Err("explicit town orders require a plain individual town book".into());
        }
        let c = world.town_market.as_ref().ok_or("missing town market")?;
        if !world.participants.iter().any(|p| p.agent == actor)
            || submit.iter().any(|(market, _)| {
                !listings(c).iter().any(|l| {
                    l.market == *market && l.traders.iter().any(|t| t.trader.agent == actor)
                })
            })
        {
            return Err("unknown explicit order actor or listing".into());
        }
    }
    if state.phase != Phase::Acquire {
        return Err("town matching requires Acquire".into());
    }
    if let Some(round) = crate::cooperation::evaluate_with(world, state, opening, planning_state)? {
        return Ok(round);
    }
    let c = world.town_market.as_ref().ok_or("missing town market")?;
    // Rollouts must repeat the complete acquisition from its opening state.
    // Quoted holdings can include a new advance that is not yet spendable here.
    let planning = crate::production_market::choose(world, planning_state)?;
    let choices = crate::production_market::choices(world, planning.as_ref());
    let mut resources = opening.clone();
    if resources.pooling.is_none() {
        resources.pooling = Some(crate::households::income_reservations::Reservations::new(
            world,
            state,
            resources.storage.clone(),
        ));
    }
    let mut pricing_state = state.clone();
    let mut result = Round {
        conditional: None,
        selections: selections.cloned().map(Box::new),
        selection: selection.cloned().map(Box::new),
        cooperation: None,
        month: state.month,
        planning: planning.map(Box::new),
        orders: vec![],
        order_receipts: vec![],
        attempts: vec![],
        transactions: vec![],
        markets: BTreeMap::new(),
    };
    // Every order observes the same opening holdings, before either good settles.
    let books = listings(c)
        .into_iter()
        .map(|c| {
            let (mut orders, mut receipts) = orders(world, state, &c, &choices, opening)?;
            if !masks.is_empty() {
                orders.retain(|o| {
                    masks
                        .get(&o.agent)
                        .is_none_or(|submit| submit.contains(&(o.market, o.side)))
                });
                for r in &mut receipts {
                    if r.reason == OrderReason::Submitted
                        && masks
                            .get(&r.agent)
                            .is_some_and(|submit| !submit.contains(&(r.market, r.side)))
                    {
                        r.reason = OrderReason::PlannerWithheld;
                    }
                }
            }
            result.order_receipts.extend(receipts);
            Ok((c, orders))
        })
        .collect::<Result<Vec<_>, String>>()?;
    for (c, orders) in books {
        clear(
            world,
            state,
            &c,
            orders,
            &mut resources,
            &mut pricing_state,
            &mut result,
        )?;
    }
    Ok(result)
}
fn orders(
    world: &World,
    state: &State,
    c: &Config,
    choices: &BTreeMap<AgentId, crate::production_market::Choice>,
    resources: &Resources,
) -> Result<(Vec<Order>, Vec<OrderReceipt>), String> {
    let m = catalog(world, c)?;
    let admitted = state
        .town_market
        .admission
        .as_ref()
        .filter(|a| a.month == state.month)
        .ok_or("missing current opening admission")?;
    let (buy_months, reserve_months) = match c.order_horizon {
        OrderHorizon::Legacy => (
            world.production_market.as_ref().map_or(1, |p| p.horizon),
            c.reserve.reserve_months,
        ),
        OrderHorizon::Aligned(months) => (months, months),
    };
    let reserve = need_orders::Policy { reserve_months };
    let mut orders = Vec::new();
    let mut receipts = Vec::new();
    for t in &c.traders {
        let agent = t.trader.agent;
        let gate = if !admitted.eligible.contains(&agent) {
            Some(OrderReason::NotAdmitted)
        } else if state.terminal.contains_key(&agent)
            || !crate::households::market::active(world, state, agent)
            || crate::recovery::active(world, &state.credit, agent).is_some()
        {
            Some(OrderReason::Inactive)
        } else if !marketplace::eligible(world, state, c.venue, agent) {
            Some(OrderReason::Ineligible)
        } else {
            None
        };
        let sides = if c.adaptive {
            vec![Side::Buy, Side::Sell]
        } else {
            vec![t.side]
        };
        let mut submitted = false;
        for side in sides {
            let mut receipt = OrderReceipt {
                household: world
                    .households
                    .iter()
                    .find(|h| h.agent == agent)
                    .map(|h| crate::household_governance::authority(h, state)),
                market: c.market,
                agent,
                side,
                reason: OrderReason::Submitted,
                resource: m.goods.resource,
                lot: m.goods.quantity,
                buy_months,
                reserve_months,
                available: None,
                protected: None,
                deficits_before: None,
                deficits_after: None,
            };
            let skipped = gate.or_else(|| {
                if submitted {
                    Some(OrderReason::OtherSideSelected)
                } else if side == Side::Buy
                    && (choices.get(&agent).is_some_and(|p| !p.buy.allows(c.market))
                        || !crate::households::market::buys(world, state, agent))
                {
                    Some(OrderReason::PurchasePolicy)
                } else {
                    None
                }
            });
            if let Some(reason) = skipped {
                receipt.reason = reason;
                receipts.push(receipt);
                continue;
            }
            let entry = Entry {
                side,
                trader: t.trader.clone(),
            };
            let s = pair(world, c, &entry, state.month)?;
            let d = need_orders::generate_for_horizon(
                world, state, resources, &reserve, &s, buy_months,
            )?;
            let exists = if side == Side::Buy {
                d.buy.is_some()
            } else {
                d.sell.is_some()
            };
            let available = resources
                .available
                .get(&(agent, m.goods.resource))
                .copied()
                .unwrap_or(0);
            let protected = d
                .protected
                .get(&(agent, m.goods.resource))
                .copied()
                .unwrap_or(0);
            receipt.available = Some(available);
            receipt.protected = Some(protected);
            if side == Side::Buy {
                receipt.deficits_before = Some(d.buyer_deficits.clone());
                receipt.deficits_after = Some(d.buyer_after_purchase.clone());
            }
            if exists {
                let quote = marketplace::learning(state, &s, side).map_or_else(
                    || marketplace::opening(state, &s, side),
                    |l| l.quote(t.trader.limit, m.price_tick, side),
                );
                orders.push(Order {
                    market: c.market,
                    agent,
                    side,
                    quote,
                    protected: d
                        .protected
                        .into_iter()
                        .filter(|((a, _), _)| *a == agent)
                        .collect(),
                });
                submitted = true;
            } else {
                receipt.reason = if side == Side::Buy {
                    OrderReason::NoNeedImprovement
                } else if available < m.goods.quantity {
                    OrderReason::InsufficientOpeningStock
                } else {
                    OrderReason::ProtectedStock
                };
            }
            receipts.push(receipt);
        }
    }
    orders.sort_by_key(|o| (o.side, o.agent));
    receipts.sort_by_key(|r| (r.side, r.agent));
    Ok((orders, receipts))
}
fn clear(
    world: &World,
    state: &State,
    c: &Config,
    orders: Vec<Order>,
    resources: &mut Resources,
    pricing_state: &mut State,
    result: &mut Round,
) -> Result<(), String> {
    let m = catalog(world, c)?;
    let mut observation = MarketResult::default();
    let mut completed = 0;
    let mut buyers: Vec<_> = orders.iter().filter(|o| o.side == Side::Buy).collect();
    let mut sellers: Vec<_> = orders.iter().filter(|o| o.side == Side::Sell).collect();
    buyers.sort_by_key(|o| (std::cmp::Reverse(o.quote), o.agent));
    sellers.sort_by_key(|o| (o.quote, o.agent));
    let mut filled = BTreeSet::new();
    let limit = if world.production_market.as_ref().is_some_and(|p| !p.trading) {
        Some(0)
    } else {
        c.match_limit
    };
    for buyer in &buyers {
        if limit.is_some_and(|n| completed >= n) {
            break;
        }
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
            let mut round = negotiation::evaluate_matching(&w, state, &spendable)?
                .ok_or("missing town match")?;
            // Orders keep their opening quotes for this book. Learning accumulates
            // public events in match order and affects only subsequent books.
            round.buyer_learning = marketplace::learning(pricing_state, &s, Side::Buy);
            round.seller_learning = marketplace::learning(pricing_state, &s, Side::Sell);
            for event in &round.events {
                negotiation::observe(
                    &s,
                    m.price_tick,
                    &mut round.buyer_learning,
                    &mut round.seller_learning,
                    event,
                );
            }
            marketplace::record(pricing_state, &s, &round);
            if let Outcome::Traded { price } = round.outcome {
                let transactions = negotiation::transactions(&w, state, &Some(round.clone()))?;
                resources.reserve(world, &transactions)?;
                result.transactions.extend(transactions);
                observation.volume = observation
                    .volume
                    .checked_add(m.goods.quantity)
                    .ok_or("market volume overflow")?;
                observation.posted_price = Some(price);
                completed += 1;
                filled.insert(buyer.agent);
                filled.insert(seller.agent);
            }
            result.attempts.push(Attempt { session: s, round });
            if filled.contains(&buyer.agent) || buyer.quote < seller.quote {
                break;
            }
        }
    }
    observation.unfilled_buy =
        i32::try_from(buyers.iter().filter(|o| !filled.contains(&o.agent)).count())
            .unwrap()
            .checked_mul(m.goods.quantity)
            .ok_or("order volume overflow")?;
    observation.unfilled_sell = i32::try_from(
        sellers
            .iter()
            .filter(|o| !filled.contains(&o.agent))
            .count(),
    )
    .unwrap()
    .checked_mul(m.goods.quantity)
    .ok_or("order volume overflow")?;
    result.orders.extend(orders);
    result.markets.insert(c.market, observation);
    Ok(())
}
pub(crate) fn validate_batch(world: &World, state: &State, batch: &Batch) -> Result<(), String> {
    if state.phase == Phase::Acquire
        && world.town_market.is_some()
        && crate::acquisition::shared(world)
    {
        // The shared resolver validates credit and town trades together.
        return crate::acquisition::validate_batch(world, state, batch);
    }
    let expected = if world.town_market.is_none() {
        None
    } else {
        match state.phase {
            Phase::Open => Some(Boundary::Admission(admission(world, state)?)),
            Phase::Acquire => {
                let round = batch.town_market.as_ref().and_then(|b| match b {
                    Boundary::Market(r) => Some(r),
                    _ => None,
                });
                Some(Boundary::Market(match round {
                    Some(r) if r.selection.is_some() && r.selections.is_some() => {
                        return Err("conflicting town order masks".into());
                    }
                    Some(r)
                        if r.cooperation
                            .as_ref()
                            .is_some_and(|b| !b.consents.is_empty()) =>
                    {
                        let b = r.cooperation.as_ref().unwrap();
                        crate::cooperation::evaluate_schedule(
                            world,
                            state,
                            b.terms.as_ref(),
                            &b.consents,
                        )?
                    }
                    Some(r) if r.conditional.is_some() => evaluate_conditional(
                        world,
                        state,
                        r.selections
                            .as_deref()
                            .ok_or("conditional exchange requires all consents")?,
                        r.conditional.as_ref().unwrap(),
                    )?,
                    Some(r) if r.selections.is_some() => {
                        evaluate_selections(world, state, r.selections.as_ref().unwrap())?
                    }
                    Some(r) if r.selection.is_some() => {
                        evaluate_selected(world, state, r.selection.as_ref().unwrap())?
                    }
                    _ => evaluate(world, state)?,
                }))
            }
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
        order_horizon: OrderHorizon::Legacy,
        additional: vec![],
        priority: ClearingPriority::MarketId,
        adaptive: false,
        match_limit: None,
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

/// Two reciprocal spot deliveries: both settle in this book or neither does.
/// Supplied masks are each participant's consent. No future obligation is created.
pub fn evaluate_conditional(
    world: &World,
    state: &State,
    selections: &OrderSelections,
    terms: &[crate::cooperation::Delivery],
) -> Result<Round, String> {
    if &conditional_consents(world, state.month, terms)? != selections {
        return Err("conditional spot exchange lacks exact party consents".into());
    }
    let mut round = evaluate_selections(world, state, selections)?;
    let completed = spot_deliveries(&round);
    let mut expected = terms.to_vec();
    expected.sort_by_key(|d| d.market);
    if completed != expected {
        return Err("conditional spot package not fully funded or matched".into());
    }
    round.conditional = Some(expected);
    Ok(round)
}

/// Public price/quantity terms of a quoted book, not its participants' work plans.
pub fn spot_deliveries(round: &Round) -> Vec<crate::cooperation::Delivery> {
    let mut result: Vec<_> = round
        .attempts
        .iter()
        .filter_map(|a| {
            let Outcome::Traded { price } = a.round.outcome else {
                return None;
            };
            Some(crate::cooperation::Delivery {
                month: round.month,
                market: a.session.market,
                goods: crate::finance::Transfer {
                    from: a.session.seller.agent,
                    to: a.session.buyer.agent,
                    amount: a.session.goods.clone(),
                },
                payment: crate::finance::Transfer {
                    from: a.session.buyer.agent,
                    to: a.session.seller.agent,
                    amount: Amount::new(a.session.payment, price),
                },
            })
        })
        .collect();
    result.sort_by_key(|d| d.market);
    result
}

fn conditional_consents(
    world: &World,
    month: u32,
    terms: &[crate::cooperation::Delivery],
) -> Result<OrderSelections, String> {
    if terms.len() != CONDITIONAL_SPOT_DELIVERIES
        || world.participants.len() != CONDITIONAL_SPOT_PARTIES
    {
        return Err("conditional spot pilot requires two reciprocal deliveries and people".into());
    }
    let a = &terms[0];
    let b = &terms[1];
    if a.market == b.market || a.goods.from != b.goods.to || a.goods.to != b.goods.from {
        return Err("conditional spot terms must be reciprocal".into());
    }
    let mut required: OrderSelections = world
        .participants
        .iter()
        .map(|p| (p.agent, BTreeSet::new()))
        .collect();
    for d in terms {
        if d.month != month || d.goods.from != d.payment.to || d.goods.to != d.payment.from {
            return Err("invalid conditional spot date or consideration".into());
        }
        required
            .get_mut(&d.goods.from)
            .ok_or("unknown conditional seller")?
            .insert((d.market, Side::Sell));
        required
            .get_mut(&d.goods.to)
            .ok_or("unknown conditional buyer")?
            .insert((d.market, Side::Buy));
    }
    Ok(required)
}
