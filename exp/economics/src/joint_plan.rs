//! Bounded receding-horizon production and sale policies, committed one month at a time.
use crate::{
    borrowing, compute::Backend, credit, forecast::ForecastContext, model::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;
const MAX_HORIZON: u32 = 24;
const MAX_PRODUCERS: usize = 4;
const MAX_CURRENT_LOTS: i32 = 2;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub horizon_months: u32,
    pub need_limits: BTreeMap<ResourceId, i64>,
    pub future_reserves: Vec<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    Ordinary,
    Wait,
    Produce(DefinitionId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alternative {
    pub lots: i32,
    pub work: Work,
    pub future_reserve: u32,
    pub deficits: BTreeMap<ResourceId, i64>,
    pub terminal: bool,
    pub missed_payment: bool,
    pub closing_coins: i32,
    pub closing_debt: i32,
    pub buffer_gap: u64,
    pub failures: usize,
    pub starts: Vec<(u32, DefinitionId)>,
    pub sales: Vec<(u32, i32)>,
    pub admissible: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub through: u32,
    pub alternatives: Vec<Alternative>,
    pub selected: usize,
    pub feasible: bool,
}
pub fn validate(w: &World, seller: AgentId, p: &Policy) -> Result<(), String> {
    let producers: Vec<_> = w
        .definitions
        .iter()
        .filter(|d| d.enabled && d.execution == Execution::Productive)
        .collect();
    let duration = producers.iter().map(|d| d.duration()).max().unwrap_or(1);
    let needs = &w
        .participants
        .iter()
        .find(|a| a.agent == seller)
        .ok_or("missing joint planner")?
        .needs;
    if w.priority != Priority::ContinuingFirst
        || w.work_choice.is_some()
        || w.decision_horizon.is_some()
        || w.participants.len() != 1
        || producers.len() > MAX_PRODUCERS
        || p.horizon_months < duration.saturating_mul(2)
        || p.horizon_months > MAX_HORIZON
        || p.need_limits.is_empty()
        || !crate::forecast::needs::valid_limits(needs, &p.need_limits)
        || p.future_reserves.is_empty()
        || p.future_reserves.len() > 2
        || p.future_reserves.iter().any(|n| *n > MAX_HORIZON)
        || w.credit
            .as_ref()
            .and_then(|c| c.stock_sales.as_ref())
            .is_none_or(|s| s.max_lots_per_month > MAX_CURRENT_LOTS)
    {
        return Err("joint planning requires one participant, bounded needs, two production cycles and bounded sale/work candidates".into());
    }
    Ok(())
}
pub(crate) fn choose(
    w: &World,
    s: &State,
    after: &credit::Book,
    limit: i32,
    p: &Policy,
) -> Result<(Decision, Box<Batch>), String> {
    let c = w.credit.as_ref().unwrap();
    let sale = c.stock_sales.as_ref().unwrap();
    let bid = w.bids.iter().find(|b| b.id == sale.bid).unwrap();
    let end = s
        .month
        .checked_add(p.horizon_months)
        .ok_or("joint horizon overflow")?;
    let mut work = vec![Work::Ordinary, Work::Wait];
    let mut ids: Vec<_> = w
        .definitions
        .iter()
        .filter(|d| d.enabled && d.execution == Execution::Productive)
        .map(|d| d.id)
        .collect();
    ids.sort();
    work.extend(ids.into_iter().map(Work::Produce));
    let mut reserves = p.future_reserves.clone();
    reserves.sort();
    reserves.dedup();
    let mut alternatives = Vec::new();
    let mut plans = Vec::new();
    for lots in 0..=limit {
        for &choice in &work {
            for &reserve in &reserves {
                let (mut world, state) = ForecastContext::new(w, s).into_parts();
                let cfg = world.credit.as_mut().unwrap();
                cfg.purchase_policy = if after.loans.contains_key(&c.application.offer) {
                    borrowing::Policy::Scripted
                } else {
                    borrowing::Policy::Decline
                };
                let policy = cfg.stock_sales.as_mut().unwrap();
                policy.joint = None;
                policy.forecast = None;
                policy.max_lots_per_month = lots.max(1);
                policy.purchase_budget = s
                    .credit
                    .stock_spent
                    .checked_add(
                        lots.checked_mul(bid.payment.quantity)
                            .ok_or("joint sale overflow")?,
                    )
                    .ok_or("joint budget overflow")?;
                let mut sim = Simulation::new(world, state, Backend::Reference)?;
                sim.step()?;
                if sim
                    .ledger
                    .last()
                    .unwrap()
                    .credit
                    .as_ref()
                    .unwrap()
                    .stock_sale
                    .as_ref()
                    .unwrap()
                    .sold_lots
                    != lots
                {
                    return Err("joint forecast quantity mismatch".into());
                }
                let policy = sim
                    .world
                    .credit
                    .as_mut()
                    .unwrap()
                    .stock_sales
                    .as_mut()
                    .unwrap();
                policy.purchase_budget = sale.purchase_budget;
                policy.max_lots_per_month = sale.max_lots_per_month;
                policy.reserve_months = reserve;
                let mut first = None;
                while sim.state.month < end {
                    if sim.state.phase == Phase::Productive {
                        let mut b = Batch::empty(&sim.state);
                        let selected = if choice == Work::Wait && sim.state.month != s.month {
                            Work::Ordinary
                        } else {
                            choice
                        };
                        if let Work::Produce(id) = selected
                            && !sim.state.processes.values().any(|x| {
                                x.operator == sale.seller
                                    && x.status == Status::Active
                                    && x.definition == id
                            })
                        {
                            sim.world.scheduled_starts.push(ScheduledStart {
                                month: sim.state.month,
                                agent: sale.seller,
                                definition: id,
                            });
                        }
                        sim.productive_with(&mut b, selected != Work::Ordinary)?;
                        sim.world.scheduled_starts.clear();
                        if first.is_none() {
                            first = Some(Box::new(b.clone()));
                        }
                        sim.state.pending_production = Some(Box::new(b));
                    }
                    sim.step()?;
                }
                let mut deficits = BTreeMap::new();
                for r in sim.reports.iter().filter(|r| r.agent == sale.seller) {
                    crate::forecast::needs::accumulate(
                        &mut deficits,
                        r.needs.iter().map(|(r, n)| (*r, n.deficit)),
                    );
                }
                let terminal = sim.state.terminal.contains_key(&sale.seller);
                let missed_payment = sim
                    .ledger
                    .iter()
                    .filter_map(|b| b.credit.as_ref())
                    .flat_map(|b| &b.events)
                    .any(|e| matches!(e, credit::Event::Arrears { .. }));
                let failures = sim
                    .state
                    .processes
                    .values()
                    .filter(|x| {
                        x.status == Status::Aborted
                            && !s
                                .processes
                                .get(&x.id)
                                .is_some_and(|old| old.status == Status::Aborted)
                    })
                    .count();
                let closing_debt = sim
                    .state
                    .credit
                    .loans
                    .get(&c.application.offer)
                    .map(|l| l.debt())
                    .transpose()?
                    .unwrap_or(0);
                let admissible = !terminal
                    && !missed_payment
                    && failures == 0
                    && crate::forecast::needs::within_limits(&deficits, &p.need_limits);
                let starts = sim
                    .state
                    .processes
                    .values()
                    .filter(|x| {
                        x.operator == sale.seller
                            && x.start >= s.month
                            && w.definition(x.definition).execution == Execution::Productive
                            && !s.processes.contains_key(&x.id)
                    })
                    .map(|x| (x.start, x.definition))
                    .collect();
                let sales = sim
                    .ledger
                    .iter()
                    .filter_map(|b| {
                        b.credit
                            .as_ref()
                            .and_then(|c| c.stock_sale.as_ref())
                            .filter(|r| r.sold_lots > 0)
                            .map(|r| (b.month, r.sold_lots))
                    })
                    .collect();
                alternatives.push(Alternative {
                    lots,
                    work: choice,
                    future_reserve: reserve,
                    deficits,
                    terminal,
                    missed_payment,
                    closing_coins: sim.state.balance(sale.seller, bid.payment.resource),
                    closing_debt,
                    buffer_gap: crate::planning::score(&sim).buffer_gap,
                    failures,
                    starts,
                    sales,
                    admissible,
                });
                plans.push(first.ok_or("missing joint work plan")?);
            }
        }
    }
    let feasible = alternatives.iter().any(|a| a.admissible);
    let needs = &w.participants[0].needs;
    let selected = alternatives
        .iter()
        .enumerate()
        .filter(|(_, a)| if feasible { a.admissible } else { a.lots == 0 })
        .min_by_key(|(i, a)| {
            (
                a.terminal,
                crate::forecast::needs::score(needs, &a.deficits),
                a.missed_payment,
                a.failures,
                a.buffer_gap,
                i64::from(a.closing_debt) - i64::from(a.closing_coins),
                *i,
            )
        })
        .map(|(i, _)| i)
        .ok_or("no joint fallback")?;
    let plan = plans.swap_remove(selected);
    Ok((
        Decision {
            through: end - 1,
            alternatives,
            selected,
            feasible,
        },
        plan,
    ))
}
