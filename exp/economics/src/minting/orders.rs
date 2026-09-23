//! Opening-state orders for dated production targets; whole lots and fixed limits.
use super::*;
use crate::marketplace::{MarketId, Side};

const MAX_LOTS: i32 = 64;
const MAX_QUOTES: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub month: u32,
    /// Later independent attempts; missed targets are not automatically retried.
    pub additional_months: BTreeSet<u32>,
    pub sale_market: MarketId,
    pub sale_limit: i32,
    pub input_limits: BTreeMap<MarketId, i32>,
    pub quotes: Vec<Quote>,
}
/// A buyer fills a stock target; a seller protects a reserve. Quotes are per lot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quote {
    pub agent: AgentId,
    pub market: MarketId,
    pub side: Side,
    pub limit: i32,
    pub holding: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub agent: AgentId,
    pub market: MarketId,
    pub side: Side,
    pub limit: i32,
    pub lots: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub target_month: Option<u32>,
    pub required_funding: i32,
    pub orders: Vec<Order>,
    pub deals: Vec<Deal>,
    pub reason: String,
}

pub fn validate(w: &World, c: &Config, p: &Policy) -> Result<(), String> {
    let venue = marketplace::venue(w, c.venue).ok_or("missing order venue")?;
    let market = |id| {
        venue
            .markets
            .iter()
            .find(|m| m.id == id)
            .ok_or("unlisted order market")
    };
    let valid_price = |id, price| -> Result<(), String> {
        let m = market(id)?;
        if m.goods.quantity <= 0
            || m.price_tick <= 0
            || price <= 0
            || price % m.price_tick != 0
            || m.payment != c.coin
        {
            return Err("invalid mint order terms".into());
        }
        Ok(())
    };
    let targets: BTreeSet<_> = std::iter::once(p.month)
        .chain(p.additional_months.iter().copied())
        .collect();
    let mut starts: Vec<_> = w
        .scheduled_starts
        .iter()
        .filter(|s| s.agent == c.issuer && s.definition == c.definition)
        .map(|s| s.month)
        .collect();
    starts.sort_unstable();
    if p.additional_months.iter().any(|m| *m <= p.month) {
        return Err("additional mint targets must follow the first target".into());
    }
    if !c.deals.is_empty()
        || p.month == 0
        || p.quotes.len() > MAX_QUOTES
        || starts != targets.into_iter().collect::<Vec<_>>()
    {
        return Err(
            "generated orders require matching dated mint starts and no scripted deals".into(),
        );
    }
    valid_price(p.sale_market, p.sale_limit)?;
    let sale = market(p.sale_market)?;
    if !w
        .resources
        .iter()
        .any(|r| r.id == sale.goods.resource && r.kind == ResourceKind::Stock)
    {
        return Err("funding sales require stock".into());
    }
    let d = w.definition(c.definition);
    let mut inputs = BTreeMap::<ResourceId, i32>::new();
    for a in d.stages[0]
        .entry_inputs
        .iter()
        .chain(&d.stages[0].monthly_services)
    {
        let n = inputs.entry(a.resource).or_default();
        *n = n.checked_add(a.quantity).ok_or("recipe size overflow")?;
    }
    let mut listed = BTreeSet::new();
    let mut budget = 0i32;
    for (&id, &price) in &p.input_limits {
        valid_price(id, price)?;
        let m = market(id)?;
        let qty = *inputs
            .get(&m.goods.resource)
            .ok_or("bid is not a recipe input")?;
        if qty <= 0
            || qty % m.goods.quantity != 0
            || qty / m.goods.quantity > MAX_LOTS
            || !listed.insert(m.goods.resource)
            || m.goods.resource == sale.goods.resource
        {
            return Err("mint input markets require distinct bounded whole recipe lots".into());
        }
        budget = budget
            .checked_add(
                price
                    .checked_mul(qty / m.goods.quantity)
                    .ok_or("mint budget overflow")?,
            )
            .ok_or("mint budget overflow")?;
    }
    if listed.len() != inputs.len() {
        return Err("missing recipe input listing".into());
    }
    let mut keys = BTreeSet::new();
    for q in &p.quotes {
        valid_price(q.market, q.limit)?;
        if q.agent == c.issuer
            || !w.agents.iter().any(|a| a.id == q.agent)
            || q.holding < 0
            || !keys.insert((q.agent, q.market))
            || q.side == Side::Sell && q.market == p.sale_market
            || q.side == Side::Buy && q.market != p.sale_market
            || q.side == Side::Sell && !p.input_limits.contains_key(&q.market)
        {
            return Err("invalid mint counterparty quote policy".into());
        }
    }
    Ok(())
}

pub fn generate(w: &World, s: &State, c: &Config, p: &Policy) -> Result<Plan, String> {
    let venue = marketplace::venue(w, c.venue).ok_or("missing venue")?;
    let market = |id| {
        venue
            .markets
            .iter()
            .find(|m| m.id == id)
            .ok_or("unlisted market")
    };
    let target = std::iter::once(p.month)
        .chain(p.additional_months.iter().copied())
        .find(|m| *m >= s.month);
    let mut plan = Plan {
        target_month: target,
        required_funding: 0,
        orders: vec![],
        deals: vec![],
        reason: String::new(),
    };
    let Some(target_month) = target else {
        plan.reason = "target dates passed".into();
        return Ok(plan);
    };
    if !marketplace::eligible(w, s, c.venue, c.issuer)
        || !opportunities::permits(w, s, c.issuer, Action::Process(c.definition))
    {
        plan.reason = "issuer admission or mint permission denied".into();
        return Ok(plan);
    }
    let d = w.definition(c.definition);
    if !d.enabled {
        plan.reason = "mint process disabled".into();
        return Ok(plan);
    }
    let mut bids = vec![];
    for (&id, &limit) in &p.input_limits {
        let m = market(id)?;
        let quantity: i32 = d.stages[0]
            .entry_inputs
            .iter()
            .chain(&d.stages[0].monthly_services)
            .filter(|a| a.resource == m.goods.resource)
            .map(|a| a.quantity)
            .sum();
        // Current capacity cannot be carried into the target month.
        let capacity = w
            .resources
            .iter()
            .any(|r| r.id == m.goods.resource && r.kind == ResourceKind::Capacity);
        let owned = if capacity && s.month < target_month {
            0
        } else {
            s.balance(c.issuer, m.goods.resource)
        };
        let deficit = (quantity - owned).max(0);
        let lots =
            (i64::from(deficit) + i64::from(m.goods.quantity) - 1) / i64::from(m.goods.quantity);
        let lots = i32::try_from(lots).map_err(|_| "order size overflow")?;
        plan.required_funding = plan
            .required_funding
            .checked_add(lots.checked_mul(limit).ok_or("funding overflow")?)
            .ok_or("funding overflow")?;
        if lots > 0 {
            bids.push(Order {
                agent: c.issuer,
                market: id,
                side: Side::Buy,
                limit,
                lots,
            });
        }
    }
    for q in &p.quotes {
        if !marketplace::eligible(w, s, c.venue, q.agent) {
            continue;
        }
        let m = market(q.market)?;
        let held = s.balance(q.agent, m.goods.resource);
        let quantity = match q.side {
            Side::Buy => (q.holding - held).max(0),
            Side::Sell => (held - q.holding).max(0),
        };
        let lots = (quantity / m.goods.quantity).min(MAX_LOTS);
        if lots > 0 {
            plan.orders.push(Order {
                agent: q.agent,
                market: q.market,
                side: q.side,
                limit: q.limit,
                lots,
            });
        }
    }
    let gap = (plan.required_funding - s.balance(c.issuer, c.coin)).max(0);
    if gap > 0 && s.month < target_month {
        let m = market(p.sale_market)?;
        let lots = ((i64::from(gap) + i64::from(p.sale_limit) - 1) / i64::from(p.sale_limit))
            .min(i64::from(
                s.balance(c.issuer, m.goods.resource) / m.goods.quantity,
            ))
            .min(i64::from(MAX_LOTS)) as i32;
        if lots > 0 {
            plan.orders.push(Order {
                agent: c.issuer,
                market: p.sale_market,
                side: Side::Sell,
                limit: p.sale_limit,
                lots,
            });
        }
        plan.reason = "raising opening funds before target month".into();
    } else if gap > 0 {
        plan.reason = "insufficient opening funds at target date".into();
    } else if s.month < target_month {
        plan.reason = "funded; waiting for dated inputs".into();
    } else {
        plan.orders.extend(bids);
        plan.reason = "matching complete input package".into();
    }
    plan.orders.sort_by_key(|o| (o.market, o.side, o.agent));
    let mut remaining: Vec<i32> = plan.orders.iter().map(|o| o.lots).collect();
    let mut resources = Resources::opening(w, s);
    let mut next_id = 1;
    let mut shortages = vec![];
    for (i, order) in plan
        .orders
        .iter()
        .enumerate()
        .filter(|(_, o)| o.agent == c.issuer)
    {
        let mut candidates: Vec<_> = plan
            .orders
            .iter()
            .enumerate()
            .filter(|(_, other)| {
                other.agent != c.issuer
                    && other.market == order.market
                    && other.side != order.side
                    && match order.side {
                        Side::Buy => other.limit <= order.limit,
                        Side::Sell => other.limit >= order.limit,
                    }
            })
            .collect();
        candidates.sort_by_key(|(_, o)| {
            (
                match order.side {
                    Side::Buy => i64::from(o.limit),
                    Side::Sell => -i64::from(o.limit),
                },
                o.agent,
            )
        });
        let mut last_rejection = None;
        for (j, other) in candidates {
            while remaining[i] > 0 && remaining[j] > 0 {
                let (buyer, seller, price) = match order.side {
                    Side::Buy => (c.issuer, other.agent, other.limit),
                    Side::Sell => (other.agent, c.issuer, order.limit),
                };
                let deal = Deal {
                    id: next_id,
                    month: s.month,
                    package: if s.month == target_month { 1 } else { next_id },
                    market: order.market,
                    buyer,
                    seller,
                    price,
                };
                let mut staged = resources.clone();
                let valid = transaction(w, s, c, &deal).and_then(|t| staged.reserve(w, &[t]));
                if let Err(reason) = valid {
                    last_rejection = Some(reason);
                    break;
                }
                resources = staged;
                plan.deals.push(deal);
                next_id += 1;
                remaining[i] -= 1;
                remaining[j] -= 1;
            }
        }
        if remaining[i] > 0 {
            shortages.push(format!(
                "market {} lacks {} lots: {}",
                order.market,
                remaining[i],
                last_rejection
                    .as_deref()
                    .unwrap_or("insufficient quantity at crossing quotes")
            ));
        }
    }
    if s.month == target_month
        && plan
            .orders
            .iter()
            .enumerate()
            .any(|(i, o)| o.agent == c.issuer && remaining[i] > 0)
    {
        plan.deals.clear();
        plan.reason = format!("input package unmatched: {}", shortages.join("; "));
    }
    Ok(plan)
}
