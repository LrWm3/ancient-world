//! Opening-state orders for dated production targets; whole lots and fixed limits.
use super::*;
use crate::marketplace::{MarketId, Side};

const MAX_LOTS: i32 = 64;
const MAX_QUOTES: usize = 32;

/// A positive unmet target requires a whole lot, even when smaller than that lot.
pub(crate) fn required_lots(target: i32, held: i32, lot: i32) -> i32 {
    let gap = (i64::from(target) - i64::from(held)).max(0);
    ((gap + i64::from(lot) - 1) / i64::from(lot)) as i32
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub month: u32,
    /// Later independent attempts; missed targets are not automatically retried.
    pub additional_months: BTreeSet<u32>,
    pub sale_market: MarketId,
    pub sale_limit: i32,
    pub input_limits: BTreeMap<MarketId, i32>,
    pub quotes: Vec<Quote>,
    pub provisioning: Option<super::provisioning::Policy>,
    /// Sell surplus independently of mint funding; ordinary clearing still decides fills.
    pub public_sale: Option<StockSales>,
    /// Opt-in peer stock sales with this accepted-claim protection horizon.
    pub private_sales: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StockSales {
    pub reserve: i32,
    pub claim_months: u32,
}
/// A buyer fills a stock target; a seller protects a reserve. Quotes are per lot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quote {
    /// Optional whole-lot authorization ceiling, independent of the retained stock floor.
    pub max_lots: Option<i32>,
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
pub struct PurchaseBudget {
    pub agent: AgentId,
    pub requested_lots: i32,
    pub opening_cash: i32,
    pub protected_cash: i128,
    pub affordable_lots: i32,
    pub submitted_lots: i32,
    pub matched_lots: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub target_month: Option<u32>,
    pub required_funding: i32,
    pub orders: Vec<Order>,
    pub deals: Vec<Deal>,
    /// Procurement status; public stock sales may settle even when minting is idle.
    pub reason: String,
    pub provision: Vec<super::provisioning::Decision>,
    pub purchases: Vec<PurchaseBudget>,
}

pub fn validate(w: &World, c: &Config, p: &Policy) -> Result<(), String> {
    if p.private_sales
        .is_some_and(|months| !(1..=crate::need_orders::MAX_RESERVE_MONTHS).contains(&months))
        || p.private_sales.is_some() && p.provisioning.is_some()
        || p.public_sale.as_ref().is_some_and(|v| {
            v.reserve < 0 || !(1..=crate::need_orders::MAX_RESERVE_MONTHS).contains(&v.claim_months)
        })
        || p.public_sale.is_some() && p.provisioning.is_some()
    {
        return Err("invalid public stock sales policy".into());
    }
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
        .filter(|m| *m > 0)
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
        || (p.month == 0 && !p.additional_months.is_empty())
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
            || q.max_lots.is_some_and(|n| !(0..=MAX_LOTS).contains(&n))
            || !keys.insert((q.agent, q.market))
            || q.side == Side::Sell && q.market == p.sale_market && p.private_sales.is_none()
            || q.side == Side::Buy && q.market != p.sale_market
            || q.side == Side::Sell
                && q.market != p.sale_market
                && !p.input_limits.contains_key(&q.market)
        {
            return Err("invalid mint counterparty quote policy".into());
        }
    }
    if let Some(policy) = &p.provisioning {
        super::provisioning::validate(w, c, p, policy)?;
    }
    Ok(())
}

pub fn generate(w: &World, s: &State, c: &Config, p: &Policy) -> Result<Plan, String> {
    let resources = super::market_resources(w, s, &Resources::opening(w, s));
    generate_with(w, s, c, p, &resources)
}
pub(crate) fn generate_with(
    w: &World,
    s: &State,
    c: &Config,
    p: &Policy,
    opening: &Resources,
) -> Result<Plan, String> {
    if let Some(policy) = &p.provisioning {
        return super::provisioning::generate_with(w, s, c, p, policy, opening);
    }
    let mut plan = generate_fixed(w, s, c, p, opening)?;
    if let Some(policy) = &p.public_sale {
        public_sales(w, s, c, p, policy, opening, &mut plan)?;
    } else if let Some(months) = p.private_sales {
        public_sales(
            w,
            s,
            c,
            p,
            &StockSales {
                reserve: 0,
                claim_months: months,
            },
            opening,
            &mut plan,
        )?;
    } else {
        clear(w, s, c, &mut plan, opening)?;
    }
    Ok(plan)
}

fn public_sales(
    w: &World,
    s: &State,
    c: &Config,
    p: &Policy,
    policy: &StockSales,
    opening: &Resources,
    plan: &mut Plan,
) -> Result<(), String> {
    // Replace the legacy funding ask and its buyers, never stack authorizations.
    plan.orders.retain(|o| o.market != p.sale_market);
    let venue = marketplace::venue(w, c.venue).ok_or("missing sale venue")?;
    let market = venue
        .markets
        .iter()
        .find(|m| m.id == p.sale_market)
        .unwrap();
    let claims = crate::need_orders::claims(w, s, c.issuer, policy.claim_months)?;
    let held = opening
        .available
        .get(&(c.issuer, market.goods.resource))
        .copied()
        .unwrap_or(0);
    let protected =
        i128::from(policy.reserve) + claims.get(&market.goods.resource).copied().unwrap_or(0);
    let supply = ((i128::from(held) - protected).max(0) / i128::from(market.goods.quantity))
        .min(i128::from(MAX_LOTS)) as i32;
    {
        let mut demand = 0;
        if p.private_sales.is_some() {
            for q in p
                .quotes
                .iter()
                .filter(|q| q.side == Side::Sell && q.market == p.sale_market)
            {
                if !super::eligible(w, s, c.venue, q.agent) {
                    continue;
                }
                let held = opening
                    .available
                    .get(&(q.agent, market.goods.resource))
                    .copied()
                    .unwrap_or(0);
                let lots = ((held - q.holding).max(0) / market.goods.quantity)
                    .min(q.max_lots.unwrap_or(MAX_LOTS));
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
        }
        for q in p
            .quotes
            .iter()
            .filter(|q| q.side == Side::Buy && q.market == p.sale_market)
        {
            if !super::eligible(w, s, c.venue, q.agent)
                || (p.private_sales.is_none() && q.limit < p.sale_limit)
            {
                continue;
            }
            let needed = required_lots(
                q.holding,
                s.balance(q.agent, market.goods.resource),
                market.goods.quantity,
            );
            let cash = opening
                .available
                .get(&(q.agent, c.coin))
                .copied()
                .unwrap_or(0);
            let commitments = crate::need_orders::claims(w, s, q.agent, policy.claim_months)?;
            let protected_cash = commitments.get(&c.coin).copied().unwrap_or(0);
            // Private asks may differ from the public valuation. Reserve each
            // bid at its own limit so every crossing price preserves claims.
            let budget_price = if p.private_sales.is_some() {
                q.limit
            } else {
                p.sale_limit
            };
            let affordable_lots = ((i128::from(cash) - protected_cash).max(0)
                / i128::from(budget_price))
            .min(i128::from(MAX_LOTS)) as i32;
            let lots = needed
                .min(q.max_lots.unwrap_or(MAX_LOTS))
                .min(affordable_lots);
            plan.purchases.push(PurchaseBudget {
                agent: q.agent,
                requested_lots: needed,
                opening_cash: cash,
                protected_cash,
                affordable_lots,
                submitted_lots: lots,
                matched_lots: 0,
            });
            if lots > 0 {
                demand += lots;
                plan.orders.push(Order {
                    agent: q.agent,
                    market: p.sale_market,
                    side: Side::Buy,
                    limit: q.limit,
                    lots,
                });
            }
        }
        let lots = supply.min(demand);
        if lots > 0 && p.public_sale.is_some() && super::eligible(w, s, c.venue, c.issuer) {
            plan.orders.push(Order {
                agent: c.issuer,
                market: p.sale_market,
                side: Side::Sell,
                limit: p.sale_limit,
                lots,
            });
        }
    }
    clear(w, s, c, plan, opening)?;
    for budget in &mut plan.purchases {
        budget.matched_lots = plan
            .deals
            .iter()
            .filter(|d| d.buyer == budget.agent && d.market == p.sale_market)
            .count() as i32;
    }
    Ok(())
}
pub(super) fn generate_fixed(
    w: &World,
    s: &State,
    c: &Config,
    p: &Policy,
    opening: &Resources,
) -> Result<Plan, String> {
    let venue = marketplace::venue(w, c.venue).ok_or("missing venue")?;
    let market = |id| {
        venue
            .markets
            .iter()
            .find(|m| m.id == id)
            .ok_or("unlisted market")
    };
    let target = std::iter::once(p.month)
        .filter(|m| *m > 0)
        .chain(p.additional_months.iter().copied())
        .find(|m| *m >= s.month);
    let mut plan = Plan {
        target_month: target,
        provision: vec![],
        purchases: vec![],
        required_funding: 0,
        orders: vec![],
        deals: vec![],
        reason: String::new(),
    };
    let Some(target_month) = target else {
        plan.reason = "target dates passed".into();
        return Ok(plan);
    };
    if !super::eligible(w, s, c.venue, c.issuer)
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
        let lots = required_lots(quantity, owned, m.goods.quantity);
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
        if !super::eligible(w, s, c.venue, q.agent) {
            continue;
        }
        let m = market(q.market)?;
        let held = s.balance(q.agent, m.goods.resource);
        let lots = match q.side {
            Side::Buy => required_lots(q.holding, held, m.goods.quantity),
            Side::Sell => {
                (opening
                    .available
                    .get(&(q.agent, m.goods.resource))
                    .copied()
                    .unwrap_or(0)
                    - q.holding)
                    .max(0)
                    / m.goods.quantity
            }
        };
        let lots = lots.min(q.max_lots.unwrap_or(MAX_LOTS));
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
    let gap = (plan.required_funding
        - opening
            .available
            .get(&(c.issuer, c.coin))
            .copied()
            .unwrap_or(0))
    .max(0);
    if gap > 0 && s.month < target_month {
        let m = market(p.sale_market)?;
        let lots = ((i64::from(gap) + i64::from(p.sale_limit) - 1) / i64::from(p.sale_limit))
            .min(i64::from(
                opening
                    .available
                    .get(&(c.issuer, m.goods.resource))
                    .copied()
                    .unwrap_or(0)
                    / m.goods.quantity,
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
    Ok(plan)
}

/// Food/stock sales precede the conditional input package, but their proceeds
/// are never added to opening spendable funds. Failed inputs preserve sales.
pub(super) fn clear(
    w: &World,
    s: &State,
    c: &Config,
    plan: &mut Plan,
    opening: &Resources,
) -> Result<(), String> {
    plan.deals.clear();
    plan.orders
        .sort_by_key(|o| (o.side != Side::Sell, o.market, o.limit, o.agent));
    let sale_market = c.order_policy.as_ref().unwrap().sale_market;
    let mut remaining: Vec<i32> = plan.orders.iter().map(|o| o.lots).collect();
    let mut resources = opening.clone();
    let mut next_id = 1;
    let mut shortages = vec![];
    for (i, order) in plan.orders.iter().enumerate().filter(|(_, o)| {
        (o.side == Side::Sell && o.market == sale_market)
            || (o.agent == c.issuer && o.side == Side::Buy)
    }) {
        let mut candidates: Vec<_> = plan
            .orders
            .iter()
            .enumerate()
            .filter(|(_, other)| {
                other.agent != order.agent
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
                    Side::Sell => (other.agent, order.agent, order.limit),
                };
                let deal = Deal {
                    id: next_id,
                    month: s.month,
                    package: if order.side == Side::Buy {
                        u32::MAX
                    } else {
                        next_id
                    },
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
        if order.side == Side::Buy && remaining[i] > 0 {
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
    if !shortages.is_empty() {
        plan.deals.retain(|d| d.market == sale_market);
        plan.reason = format!("input package unmatched: {}", shortages.join("; "));
    }
    Ok(())
}
