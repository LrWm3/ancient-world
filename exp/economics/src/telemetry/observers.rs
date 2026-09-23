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
    if config.settlement
        && let Some(c) = &world.minting
    {
        if let Some(boundary) = &batch.minting {
            for receipt in &boundary.receipts {
                let deals: Vec<_> = c
                    .deals
                    .iter()
                    .filter(|d| receipt.deals.contains(&d.id))
                    .collect();
                if deals
                    .iter()
                    .any(|d| selected(config, d.buyer) || selected(config, d.seller))
                {
                    records.push(json!({"kind":"physical_minting_market","package":receipt.package,
                        "accepted":receipt.accepted,"reason":receipt.reason,
                        "deals":deals.iter().map(|d|json!({"id":d.id,"month":d.month,
                            "market":d.market,"buyer":d.buyer,"seller":d.seller,"price":d.price})).collect::<Vec<_>>()}));
                }
            }
        }
        if selected(config, c.issuer) {
            for p in batch
                .transactions
                .iter()
                .filter_map(|t| t.process.as_ref())
                .filter(|p| {
                    p.after.definition == c.definition && p.after.status == Status::Completed
                })
            {
                records.push(json!({"kind":"physical_coin_issuance","issuer":c.issuer,
                    "process":p.after.id,"coin":c.coin,"quantity":world.definition(c.definition).outputs[0].quantity}));
            }
        }
    }
    if config.settlement
        && let Some(credit) = &batch.credit
    {
        let visible = |loan: &crate::credit::Loan| {
            selected(config, loan.debtor) || selected(config, loan.creditor)
        };
        for loan in credit.after.loans.values().filter(|l| visible(l)) {
            records.push(json!({"kind":"loan_state","loan":loan.id,
                "debtor":loan.debtor,"creditor":loan.creditor,"denomination":loan.denomination,
                "principal":loan.principal,"interest":loan.interest,
                "principal_due":loan.principal_due(batch.month),"first_unpaid":loan.first_unpaid,
                "last_accrued":loan.last_accrued,"status":format!("{:?}",loan.status),
                "asset":loan.collateral.asset,"pledged":loan.collateral.pledged,
                "owner":credit.after.owners.get(&loan.collateral.asset)}));
        }
        for event in &credit.events {
            use crate::credit::Event;
            let (loan, detail) = match event {
                Event::Accrued {
                    loan,
                    opening_principal,
                    interest,
                } => (
                    *loan,
                    json!({"event":"Accrued","opening_principal":opening_principal,"interest":interest}),
                ),
                Event::Paid {
                    loan,
                    interest,
                    principal,
                } => (
                    *loan,
                    json!({"event":"Paid","interest":interest,"principal":principal}),
                ),
                Event::Arrears {
                    loan,
                    amount,
                    since,
                } => (
                    *loan,
                    json!({"event":"Arrears","amount":amount,"since":since}),
                ),
                Event::Enforced {
                    loan,
                    value,
                    debt_credit,
                    surplus,
                    remaining_debt,
                } => (
                    *loan,
                    json!({"event":"Enforced","value":value,"debt_credit":debt_credit,
                        "surplus":surplus,"remaining_debt":remaining_debt}),
                ),
                _ => continue,
            };
            if credit.after.loans.get(&loan).is_some_and(visible) {
                records.push(json!({"kind":"loan_event","loan":loan,"detail":detail}));
            }
        }
        if let Some(sale) = &credit.stock_sale
            && (selected(config, sale.seller)
                || world
                    .bids
                    .iter()
                    .find(|b| b.id == sale.bid)
                    .is_some_and(|b| selected(config, b.buyer)))
        {
            records.push(
                json!({"kind":"credit_stock_sale","seller":sale.seller,"bid":sale.bid,
                "reserve":sale.reserve,"opening_stock":sale.opening_stock,
                "desired_lots":sale.desired_lots,"monthly_limit":sale.monthly_limit,
                "funding_limit":sale.funding_limit,"storage_limit":sale.storage_limit,
                "sold_lots":sale.sold_lots,"goods":sale.goods,"coins":sale.coins}),
            );
        }
        for change in &credit.attachments {
            if selected(config, change.after.operator)
                || change
                    .before
                    .as_ref()
                    .is_some_and(|p| selected(config, p.operator))
            {
                records.push(
                    json!({"kind":"collateral_process_transfer","process":change.after.id,
                    "from":change.before.as_ref().map(|p|p.operator),"to":change.after.operator,
                    "beneficiary":change.after.beneficiary,"stage":change.after.stage,
                    "elapsed":change.after.elapsed,"status":format!("{:?}",change.after.status)}),
                );
            }
        }
    }
    if let Some(town_market::Boundary::Market(round)) = &batch.town_market {
        if let Some(c) = &round.cooperation
            && (config.settlement || config.planning != PlanningDetail::Off)
        {
            let delivery = |d: &crate::cooperation::Delivery| {
                json!({"month":d.month,
                "market":d.market,"seller":d.goods.from,"buyer":d.goods.to,
                "resource":d.goods.amount.resource,"quantity":d.goods.amount.quantity,
                "payment_resource":d.payment.amount.resource,"price":d.payment.amount.quantity})
            };
            let score = |s: &crate::cooperation::Score| {
                json!({"terminal":s.terminal,
                "deficits":s.deficits,"failures":s.failures,"buffer_gap":s.buffer_gap.to_string(),
                "productive_labor":s.productive_labor})
            };
            let relevant = world.participants.iter().any(|p| selected(config, p.agent));
            if relevant {
                records.push(json!({"kind":"cooperation","event":c.event,"failure":c.failure,
                    "agreement":c.agreement,
                    "projections":c.projections,"joint_projections":c.joint_projections,"through":c.active.as_ref().map(|a|a.through),
                    "deliveries":c.active.as_ref().map(|a|a.deliveries.iter().map(&delivery).collect::<Vec<_>>()),
                    "choices":c.active.as_ref().map(|a|a.choices.iter().map(|(id,c)|json!({"agent":id,"work":format!("{:?}",c.work)})).collect::<Vec<_>>()),
                    "offers":c.offers.iter().map(|o|json!({"proposer":o.proposer,"expires":o.expires,"accepted":o.accepted,
                        "proposer_assessment":selected(config,o.proposer).then(||json!({"acceptable":o.proposer_assessment.acceptable,"work":format!("{:?}",o.proposer_assessment.choice.work),"baseline":score(&o.proposer_assessment.baseline),"proposed":score(&o.proposer_assessment.proposed),"closing_coins":o.proposer_assessment.closing_coins})),
                        "replies":o.replies.iter().filter(|a|selected(config,a.agent)).map(|a|json!({"agent":a.agent,"work":format!("{:?}",a.choice.work),"acceptable":a.acceptable,"baseline":score(&a.baseline),"proposed":score(&a.proposed),"closing_coins":a.closing_coins})).collect::<Vec<_>>(),
                        "deliveries":o.deliveries.iter().map(&delivery).collect::<Vec<_>>()})).collect::<Vec<_>>(),
                    "assessments":c.assessments.iter().filter(|a|selected(config,a.agent)).map(|a|json!({"agent":a.agent,"acceptable":a.acceptable,
                        "baseline":score(&a.baseline),"proposed":score(&a.proposed),"closing_coins":a.closing_coins})).collect::<Vec<_>>(),
                    "completed":c.completed.iter().map(delivery).collect::<Vec<_>>()}));
            }
        }
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
                    "assumptions":{"own_choice":"held_over_horizon","other_agents":world.production_market.as_ref().map(|c|format!("{:?}",c.counterparties)),
                        "counterparty_choices":decision.counterparties.iter().filter(|(agent,_)|**agent!=person.agent).map(|(agent,o)|json!({"agent":agent,"month":o.month,"work":format!("{:?}",o.choice.work),"buy":format!("{:?}",o.choice.buy)})).collect::<Vec<_>>(),
                        "demand_signal":world.production_market.as_ref().map(|c|format!("{:?}",c.demand))},
                    "persistence":world.production_market.as_ref().map(|c|format!("{:?}",c.persistence)),
                    "selection_reason":format!("{:?}",person.selection_reason),"retain_through":person.retain_through,
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
