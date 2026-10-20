//! Dated, nonbinding requests derived from published procurement intent.
//! A request is neither an order fill, a reservation nor a funding commitment.
use crate::{marketplace::Side, model::*, opportunities::Action};

pub mod routes;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Policy {
    #[default]
    Static,
    Observe,
    Respond,
}

const MAX_REQUESTS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Source {
    StandingStockTarget {
        target: i32,
        ceiling: i32,
    },
    ScheduledInput {
        definition: DefinitionId,
        needed: i32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Request {
    pub requester: AgentId,
    pub venue: AgentId,
    pub market: u32,
    pub resource: ResourceId,
    pub payment: ResourceId,
    pub lot_quantity: i32,
    pub lots: i32,
    pub limit: i32,
    pub observed_stock: i32,
    /// A production deadline only for a dated procurement plan. Standing stock
    /// interest has no invented delivery promise or deadline.
    pub due: Option<u32>,
    pub source: Source,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Snapshot {
    pub month: u32,
    pub actor: AgentId,
    /// Refreshed at every Open; never reused as current demand next month.
    pub requests: Vec<Request>,
}
impl Snapshot {
    pub fn observe(w: &World, s: &State, actor: AgentId) -> Result<Self, String> {
        if s.phase != Phase::Open || s.month == 0 {
            return Err("work requests require an opening boundary".into());
        }
        let mut snapshot = Self {
            month: s.month,
            actor,
            requests: vec![],
        };
        let Some(c) = &w.minting else {
            return Ok(snapshot);
        };
        let Some(p) = &c.order_policy else {
            return Ok(snapshot);
        };
        if !crate::minting::eligible(w, s, c.venue, actor) {
            return Ok(snapshot);
        }
        let venue = crate::marketplace::venue(w, c.venue).ok_or("missing request venue")?;
        for q in p.peer_stocks.iter().flat_map(|p| &p.quotes) {
            if q.side != Side::Buy
                || q.agent == actor
                || !crate::minting::eligible(w, s, c.venue, q.agent)
                || super::markets::validate_quotes(w, q.agent, std::slice::from_ref(q)).is_err()
            {
                continue;
            }
            let m = venue
                .markets
                .iter()
                .find(|m| m.id == q.market)
                .ok_or("missing request market")?;
            let Some(ceiling) = q.max_lots else { continue };
            let held = s.balance(q.agent, m.goods.resource);
            let lots = crate::minting::orders::required_lots(q.holding, held, m.goods.quantity)
                .min(ceiling);
            if lots > 0 {
                snapshot.requests.push(Request {
                    requester: q.agent,
                    venue: c.venue,
                    market: m.id,
                    resource: m.goods.resource,
                    payment: m.payment,
                    lot_quantity: m.goods.quantity,
                    lots,
                    limit: q.limit,
                    observed_stock: held,
                    due: None,
                    source: Source::StandingStockTarget {
                        target: q.holding,
                        ceiling,
                    },
                });
            }
        }
        // The first still-live date is the same procurement window used by the
        // existing issuer adapter. Later dates cannot spend this window's stock.
        let due = std::iter::once(p.month)
            .filter(|m| *m > 0)
            .chain(p.additional_months.iter().copied())
            .find(|m| *m >= s.month);
        if let Some(due) = due.filter(|_| {
            c.issuer != actor
                && crate::minting::eligible(w, s, c.venue, c.issuer)
                && crate::opportunities::permits(w, s, c.issuer, Action::Process(c.definition))
        }) {
            let d = w.definition(c.definition);
            if d.enabled {
                for (&id, &limit) in &p.input_limits {
                    let m = venue
                        .markets
                        .iter()
                        .find(|m| m.id == id)
                        .ok_or("missing input request market")?;
                    let needed = d.stages[0]
                        .entry_inputs
                        .iter()
                        .chain(&d.stages[0].monthly_services)
                        .filter(|a| a.resource == m.goods.resource)
                        .try_fold(0_i32, |n, a| n.checked_add(a.quantity))
                        .ok_or("request quantity overflow")?;
                    let capacity = w
                        .resources
                        .iter()
                        .any(|r| r.id == m.goods.resource && r.kind == ResourceKind::Capacity);
                    let held = if capacity && due > s.month {
                        0
                    } else {
                        s.balance(c.issuer, m.goods.resource)
                    };
                    let lots =
                        crate::minting::orders::required_lots(needed, held, m.goods.quantity);
                    if lots > 0 {
                        snapshot.requests.push(Request {
                            requester: c.issuer,
                            venue: c.venue,
                            market: id,
                            resource: m.goods.resource,
                            payment: m.payment,
                            lot_quantity: m.goods.quantity,
                            lots,
                            limit,
                            observed_stock: held,
                            due: Some(due),
                            source: Source::ScheduledInput {
                                definition: d.id,
                                needed,
                            },
                        });
                    }
                }
            }
        }
        snapshot
            .requests
            .sort_by_key(|r| (r.venue, r.market, r.requester, r.due));
        if snapshot.requests.len() > MAX_REQUESTS {
            return Err("work request budget exceeded".into());
        }
        Ok(snapshot)
    }
}

impl Snapshot {
    /// Recheck immutable schema and frozen provenance, not today's stock or quotes.
    /// This is historical evidence validation, not proof of future funding.
    pub fn validate(&self, w: &World, actor: AgentId, month: u32) -> Result<(), String> {
        if self.actor != actor
            || self.month != month
            || month == 0
            || self.requests.len() > MAX_REQUESTS
        {
            return Err("invalid work request boundary".into());
        }
        let mut previous = None;
        for r in &self.requests {
            let key = (r.venue, r.market, r.requester, r.due);
            let m = crate::marketplace::venue(w, r.venue)
                .and_then(|v| v.markets.iter().find(|m| m.id == r.market))
                .ok_or("unknown request listing")?;
            if previous.is_some_and(|p| p >= key)
                || r.requester == actor
                || !w.agents.iter().any(|a| a.id == r.requester)
                || m.goods.resource != r.resource
                || m.payment != r.payment
                || m.goods.quantity != r.lot_quantity
                || r.lot_quantity <= 0
                || r.limit <= 0
                || r.limit % m.price_tick != 0
                || r.lots <= 0
                || r.observed_stock < 0
                || r.due.is_some_and(|d| d < month)
            {
                return Err("invalid frozen work request".into());
            }
            let expected = match r.source {
                Source::StandingStockTarget { target, ceiling } => {
                    let options = w
                        .agency
                        .get(&r.requester)
                        .and_then(|c| c.operating_agreement())
                        .and_then(|a| a.stock_markets.get(&r.market))
                        .ok_or("unsigned work request")?;
                    if r.due.is_some()
                        || options.side != Side::Buy
                        || !options.holdings.contains(&target)
                        || !options.limits.contains(&r.limit)
                        || ceiling <= 0
                        || ceiling > options.max_lots
                    {
                        return Err("work request exceeds signed target".into());
                    }
                    crate::minting::orders::required_lots(target, r.observed_stock, r.lot_quantity)
                        .min(ceiling)
                }
                Source::ScheduledInput { definition, needed } => {
                    let c = w
                        .minting
                        .as_ref()
                        .ok_or("missing procurement request adapter")?;
                    let d = w.definition(c.definition);
                    let actual: i64 = d.stages[0]
                        .entry_inputs
                        .iter()
                        .chain(&d.stages[0].monthly_services)
                        .filter(|a| a.resource == r.resource)
                        .map(|a| i64::from(a.quantity))
                        .sum();
                    if c.issuer != r.requester
                        || c.venue != r.venue
                        || c.definition != definition
                        || r.due.is_none()
                        || i64::from(needed) != actual
                    {
                        return Err("invalid dated input request".into());
                    }
                    crate::minting::orders::required_lots(needed, r.observed_stock, r.lot_quantity)
                }
            };
            if r.lots != expected {
                return Err("inconsistent request quantity".into());
            }
            previous = Some(key);
        }
        Ok(())
    }
}

/// Keep own goals; replace speculative sale quantities with bounded visible
/// interest. Multiple requests are alternatives, never promises to every seller.
pub fn sales(
    w: &World,
    snapshot: &Snapshot,
) -> Result<Vec<super::discovery_sales::Opportunity>, String> {
    snapshot.validate(w, snapshot.actor, snapshot.month)?;
    let mut result = vec![];
    for sale in super::discovery_sales::opportunities(w, snapshot.actor)? {
        let options = &w.agency[&snapshot.actor]
            .operating_agreement()
            .unwrap()
            .stock_markets[&sale.market];
        let first_output = snapshot
            .month
            .checked_add(1)
            .and_then(|m| m.checked_add(w.definition(sale.definition).duration()))
            .ok_or("request work date overflow")?;
        for request in &snapshot.requests {
            if request.market != sale.market
                || request.resource != sale.resource
                || !options.limits.iter().any(|p| *p <= request.limit)
                || request.due.is_some_and(|d| first_output > d)
            {
                continue;
            }
            let mut candidate = sale.clone();
            candidate.max_lots = candidate.max_lots.min(request.lots);
            candidate.target = i32::try_from(
                i64::from(candidate.floor)
                    + i64::from(candidate.max_lots) * i64::from(candidate.lot_quantity),
            )
            .map_err(|_| "request target overflow")?;
            if !result.contains(&candidate) {
                result.push(candidate);
            }
            if result.len() > super::MAX_PROGRAMS {
                return Err("request sale budget exceeded".into());
            }
        }
    }
    result.sort_by_key(|o| (o.market, o.definition, o.resource, o.target));
    Ok(result)
}
