//! Dated, nonbinding requests derived from published procurement intent.
//! A request is neither an order fill, a reservation nor a funding commitment.
use crate::{marketplace::Side, model::*, opportunities::Action};

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
