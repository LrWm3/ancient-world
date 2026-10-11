//! Opt-in stock targets in the existing Acquire book, independent of mint dates.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub claim_months: u32,
    pub quotes: Vec<Quote>,
}

pub(super) fn validate(
    w: &World,
    c: &Config,
    p: &super::Policy,
    policy: &Policy,
) -> Result<(), String> {
    if p.provisioning.is_some()
        || !(1..=crate::need_orders::MAX_RESERVE_MONTHS).contains(&policy.claim_months)
        || policy.quotes.len() > MAX_QUOTES
    {
        return Err("invalid independent stock policy or incompatible provisioning".into());
    }
    let venue = marketplace::venue(w, c.venue).ok_or("missing stock venue")?;
    let mut keys = BTreeSet::new();
    let mut resources = BTreeMap::new();
    for m in &venue.markets {
        if m.id == p.sale_market || p.input_limits.contains_key(&m.id) {
            resources.insert(m.goods.resource, m.id);
        }
    }
    for q in &policy.quotes {
        let m = venue
            .markets
            .iter()
            .find(|m| m.id == q.market)
            .ok_or("unlisted stock market")?;
        if q.agent == c.issuer
            || !w.agents.iter().any(|a| a.id == q.agent)
            || q.market == p.sale_market
            || p.input_limits.contains_key(&q.market)
            || !w
                .resources
                .iter()
                .any(|r| r.id == m.goods.resource && r.kind == ResourceKind::Stock)
            || m.goods.resource == c.coin
            || m.payment != c.coin
            || m.goods.quantity <= 0
            || m.price_tick <= 0
            || q.limit <= 0
            || q.limit % m.price_tick != 0
            || q.holding < 0
            || q.max_lots.is_some_and(|n| !(0..=MAX_LOTS).contains(&n))
            || !keys.insert((q.agent, q.market))
            || resources
                .insert(m.goods.resource, m.id)
                .is_some_and(|id| id != m.id)
        {
            return Err("invalid independent stock quote or overlapping resource market".into());
        }
    }
    Ok(())
}

pub(super) fn protect_cash(c: &Config, budgets: &[PurchaseBudget], resources: &mut Resources) {
    let mut floors = BTreeMap::<AgentId, i128>::new();
    for budget in budgets {
        let floor = floors.entry(budget.agent).or_default();
        *floor = (*floor).max(budget.protected_cash);
    }
    for (agent, protected) in floors {
        let available = resources.available.entry((agent, c.coin)).or_default();
        *available = (i128::from(*available) - protected).max(0) as i32;
    }
}

pub(super) fn generate(
    w: &World,
    s: &State,
    c: &Config,
    policy: &Policy,
    opening: &Resources,
    plan: &mut Plan,
) -> Result<(), String> {
    let venue = marketplace::venue(w, c.venue).ok_or("missing stock venue")?;
    let mut quotes: Vec<_> = policy.quotes.iter().collect();
    quotes.sort_by_key(|q| (q.market, q.agent));
    for q in quotes {
        let m = venue
            .markets
            .iter()
            .find(|m| m.id == q.market)
            .ok_or("unlisted stock market")?;
        let protected = stock_protection(w, s, q.agent, policy.claim_months)?;
        let eligible = super::super::eligible(w, s, c.venue, q.agent);
        let authorized = q.max_lots.unwrap_or(MAX_LOTS);
        let lots = match q.side {
            Side::Buy => {
                let requested = required_lots(
                    q.holding,
                    s.balance(q.agent, m.goods.resource),
                    m.goods.quantity,
                );
                let cash = opening
                    .available
                    .get(&(q.agent, c.coin))
                    .copied()
                    .unwrap_or(0);
                let protected_cash = protected.get(&c.coin).copied().unwrap_or(0);
                let affordable = ((i128::from(cash) - protected_cash).max(0) / i128::from(q.limit))
                    .min(i128::from(MAX_LOTS)) as i32;
                let lots = if eligible {
                    requested.min(authorized).min(affordable)
                } else {
                    0
                };
                plan.purchases.push(PurchaseBudget {
                    agent: q.agent,
                    market: q.market,
                    requested_lots: requested,
                    opening_cash: cash,
                    protected_cash,
                    affordable_lots: affordable,
                    submitted_lots: lots,
                    matched_lots: 0,
                });
                lots
            }
            Side::Sell => {
                let held = opening
                    .available
                    .get(&(q.agent, m.goods.resource))
                    .copied()
                    .unwrap_or(0);
                let floor = protected
                    .get(&m.goods.resource)
                    .copied()
                    .unwrap_or(0)
                    .max(i128::from(q.holding));
                let feasible = ((i128::from(held) - floor).max(0) / i128::from(m.goods.quantity))
                    .min(i128::from(authorized)) as i32;
                let lots = if eligible { feasible } else { 0 };
                plan.sales.push(SaleBudget {
                    agent: q.agent,
                    market: q.market,
                    opening_available: held,
                    quote_floor: q.holding,
                    authorized_lots: authorized,
                    protected_stock: floor,
                    eligible,
                    feasible_lots: feasible,
                    submitted_lots: lots,
                    matched_lots: 0,
                });
                lots
            }
        };
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
    Ok(())
}
