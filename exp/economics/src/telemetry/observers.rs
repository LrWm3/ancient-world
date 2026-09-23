//! Read-only projections of domain receipts; no hypothetical execution here.
use super::{Config, PlanningDetail};
use crate::{model::*, negotiation::Outcome, production_market as planning, town_market};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) struct Pending {
    batch: u64,
    month: u32,
    through: u32,
    agent: AgentId,
    forecast: planning::Forecast,
    sales: BTreeMap<u32, i64>,
    purchases: BTreeMap<u32, i64>,
    deficits: BTreeMap<u32, i64>,
}
fn forecast(f: &planning::Forecast) -> Value {
    let work = match f.choice.work {
        planning::Work::Ordinary => json!({"kind":"ordinary"}),
        planning::Work::Wait => json!({"kind":"wait"}),
        planning::Work::Produce(id) => json!({"kind":"produce","definition":id}),
    };
    let purchases = match f.choice.buy {
        planning::Purchases::None => json!({"kind":"none"}),
        planning::Purchases::All => json!({"kind":"all"}),
        planning::Purchases::Market(id) => json!({"kind":"market","market":id}),
    };
    json!({"work":work,"buy":purchases,"deficits":f.deficits,"terminal":f.terminal,
        "failures":f.failures,"buffer_gap":f.buffer_gap.to_string(),"closing_coins":f.closing_coins,
        "sales":f.sales,"purchases":f.purchases,"labor":f.labor,"stock_value":f.stock_value})
}
fn selected(config: &Config, agent: AgentId) -> bool {
    config.agents.is_empty() || config.agents.contains(&agent)
}

pub(super) fn batch(
    config: &Config,
    world: &World,
    batch: &Batch,
    pending: &mut Vec<Pending>,
) -> Vec<Value> {
    let mut records = vec![];
    if let Some(town_market::Boundary::Market(round)) = &batch.town_market {
        if config.planning != PlanningDetail::Off
            && let Some(decision) = &round.planning
        {
            for person in &decision.people {
                if !selected(config, person.agent) {
                    continue;
                }
                let chosen = &person.alternatives[person.selected];
                let beliefs: Vec<_> = decision
                    .belief
                    .iter()
                    .map(|(market, b)| {
                        json!({
                    "market":market,"through":b.through,"price":b.price,
                    "lots_per_month":b.lots_per_month,"interested_lots":b.interested_lots})
                    })
                    .collect();
                let needs: Vec<_> = world
                    .participants
                    .iter()
                    .filter(|p| p.agent == person.agent)
                    .flat_map(|p| &p.needs)
                    .map(|n| {
                        json!({"resource":n.resource,"quantity":n.quantity,
                        "priority":n.priority})
                    })
                    .collect();
                records.push(json!({"kind":"plan","agent":person.agent,"through":decision.through,
                    "planner":"production_market",
                    "assumptions":{"own_choice":"held_over_horizon","other_agents":"ordinary_work_allow_all_purchases",
                        "demand_signal":world.production_market.as_ref().map(|c|format!("{:?}",c.demand))},
                    "selected":person.selected,"candidate_count":person.alternatives.len(),
                    "needs":needs,"beliefs":beliefs,"forecast":forecast(chosen),
                    "ranking":"terminal, priority-ordered deficits, failures, buffer_gap, descending(closing_coins+stock_value), labor, candidate_index"}));
                if config.planning == PlanningDetail::Alternatives {
                    for (index, f) in person.alternatives.iter().enumerate() {
                        records.push(json!({"kind":"plan_alternative","agent":person.agent,"index":index,
                            "selected":index==person.selected,"through":decision.through,"forecast":forecast(f)}));
                    }
                }
                pending.push(Pending {
                    batch: batch.id,
                    month: batch.month,
                    through: decision.through,
                    agent: person.agent,
                    forecast: chosen.clone(),
                    sales: BTreeMap::new(),
                    purchases: BTreeMap::new(),
                    deficits: BTreeMap::new(),
                });
            }
        }
        if config.settlement {
            for (index, r) in round.order_receipts.iter().enumerate() {
                if !selected(config, r.agent) {
                    continue;
                }
                records.push(json!({"kind":"order_generation","index":index,"agent":r.agent,
                    "market":r.market,"side":format!("{:?}",r.side),"reason":format!("{:?}",r.reason),
                    "resource":r.resource,"lot":r.lot,"buy_months":r.buy_months,"reserve_months":r.reserve_months,"available":r.available,
                    "protected":r.protected.map(|q|q.to_string()),
                    "deficits_before":r.deficits_before,"deficits_after":r.deficits_after}));
            }

            for (index, order) in round.orders.iter().enumerate() {
                if !selected(config, order.agent) {
                    continue;
                }
                records.push(json!({"kind":"market_order","index":index,"agent":order.agent,
                    "market":order.market,"side":format!("{:?}",order.side),"quote":order.quote,
                    "protected":order.protected.iter().map(|(&(agent,resource),quantity)|
                        json!({"agent":agent,"resource":resource,"quantity":quantity.to_string()})).collect::<Vec<_>>()}));
            }
            for (index, attempt) in round.attempts.iter().enumerate() {
                let s = &attempt.session;
                if !selected(config, s.buyer.agent) && !selected(config, s.seller.agent) {
                    continue;
                }
                let (outcome, price) = match attempt.round.outcome {
                    Outcome::Traded { price } => ("Traded".to_string(), Some(price)),
                    ref other => (format!("{other:?}"), None),
                };
                records.push(json!({"kind":"market_attempt","index":index,"market":s.market,
                    "buyer":s.buyer.agent,"seller":s.seller.agent,"resource":s.goods.resource,
                    "requested":s.goods.quantity,"completed":if price.is_some(){s.goods.quantity}else{0},
                    "payment_resource":s.payment,"outcome":outcome,"price":price,
                    "quotes":attempt.round.quotes.iter().map(|q|json!({"round":q.round,"bid":q.bid,"ask":q.ask})).collect::<Vec<_>>()}));
            }
        }
    }
    if config.settlement {
        for (index, r) in batch.receipts.iter().enumerate() {
            if !selected(config, r.agent) {
                continue;
            }
            records.push(json!({"kind":"work_receipt","index":index,"agent":r.agent,
                "need":r.need,"definition":r.definition,"reason":format!("{:?}",r.reason),
                "requested":r.requested,"allocated":r.allocated,"completed":r.completed}));
        }
    }
    for record in &mut records {
        record["month"] = json!(batch.month);
        record["batch"] = json!(batch.id);
        record["phase"] = json!(format!("{:?}", batch.phase));
    }
    records
}

/// Observe actual committed rounds for all live forecasts, even if this month
/// falls outside export filters. A forecast total is compared only at its horizon.
pub(super) fn outcomes(batch: &Batch, pending: &mut [Pending]) {
    if let Some(town_market::Boundary::Market(round)) = &batch.town_market {
        for p in pending
            .iter_mut()
            .filter(|p| batch.month >= p.month && batch.month <= p.through)
        {
            for a in &round.attempts {
                if !matches!(a.round.outcome, Outcome::Traded { .. }) {
                    continue;
                }
                if a.session.buyer.agent == p.agent {
                    *p.purchases.entry(a.session.market).or_default() +=
                        i64::from(a.session.goods.quantity);
                }
                if a.session.seller.agent == p.agent {
                    *p.sales.entry(a.session.market).or_default() +=
                        i64::from(a.session.goods.quantity);
                }
            }
        }
    }
}
pub(super) fn reports(reports: &[MonthReport], pending: &mut [Pending]) {
    for p in pending {
        for report in reports
            .iter()
            .filter(|r| r.agent == p.agent && r.month >= p.month && r.month <= p.through)
        {
            for (resource, need) in &report.needs {
                *p.deficits.entry(*resource).or_default() += i64::from(need.deficit);
            }
        }
    }
}
pub(super) fn close(month: u32, pending: &mut Vec<Pending>) -> Vec<Value> {
    let mut rows = vec![];
    pending.retain(|p| {
        if p.through>month {return true;}
        rows.push(json!({"kind":"plan_outcome","month":month,"phase":"Close","agent":p.agent,
            "plan_batch":p.batch,"from":p.month,"through":p.through,
            "forecast":{"sales":p.forecast.sales,"purchases":p.forecast.purchases,"deficits":p.forecast.deficits},
            "actual":{"sales":p.sales,"purchases":p.purchases,"deficits":p.deficits},
            "interpretation":"original fixed-choice forecast versus realized path with monthly replanning"}));
        false
    });
    rows
}
