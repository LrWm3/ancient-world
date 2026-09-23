//! Bounded current-sale comparison using ordinary production and consumption.
use crate::{
    borrowing, compute::Backend, forecast::ForecastContext, model::*, simulation::Simulation,
};
use std::collections::BTreeMap;
const MAX_HORIZON_MONTHS: u32 = 24;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub horizon_months: u32,
    pub need_limits: BTreeMap<ResourceId, i64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alternative {
    pub lots: i32,
    pub deficits: BTreeMap<ResourceId, i64>,
    pub terminal: bool,
    pub admissible: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub through: u32,
    pub alternatives: Vec<Alternative>,
    pub selected_lots: i32,
    /// False means waiting also violates the limits; waiting does not repair scarcity.
    pub feasible: bool,
}
pub fn validate(world: &World, seller: AgentId, policy: &Policy) -> Result<(), String> {
    let participant = world
        .participants
        .iter()
        .find(|p| p.agent == seller)
        .ok_or("missing sale planner")?;
    let duration = world
        .definitions
        .iter()
        .filter(|d| d.enabled && d.execution == Execution::Productive)
        .map(|d| d.duration())
        .max()
        .unwrap_or(1);
    if policy.horizon_months < duration
        || policy.horizon_months > MAX_HORIZON_MONTHS
        || policy.need_limits.is_empty()
        || !crate::forecast::needs::valid_limits(&participant.needs, &policy.need_limits)
    {
        return Err("sale forecast requires a bounded production-length horizon and limits on positive needs".into());
    }
    Ok(())
}
pub(crate) fn choose(
    world: &World,
    state: &State,
    after: &crate::credit::Book,
    limit: i32,
    policy: &Policy,
) -> Result<Decision, String> {
    let c = world.credit.as_ref().ok_or("missing credit")?;
    let sale = c.stock_sales.as_ref().ok_or("missing sale policy")?;
    let bid = world
        .bids
        .iter()
        .find(|b| b.id == sale.bid)
        .ok_or("missing sale bid")?;
    let end = state
        .month
        .checked_add(policy.horizon_months)
        .ok_or("sale horizon overflow")?;
    let mut alternatives = Vec::new();
    for lots in 0..=limit {
        let (mut w, s) = ForecastContext::new(world, state).into_parts();
        let credit = w.credit.as_mut().unwrap();
        // Reproduce the already chosen purchase branch without recursive borrowing.
        credit.purchase_policy = if after.loans.contains_key(&c.application.offer) {
            borrowing::Policy::Scripted
        } else {
            borrowing::Policy::Decline
        };
        let p = credit.stock_sales.as_mut().unwrap();
        p.forecast = None;
        p.purchase_budget = state
            .credit
            .stock_spent
            .checked_add(
                lots.checked_mul(bid.payment.quantity)
                    .ok_or("sale forecast value overflow")?,
            )
            .ok_or("sale forecast budget overflow")?;
        p.max_lots_per_month = lots.max(1);
        let mut sim = Simulation::new(w, s, Backend::Reference)?;
        sim.step()?;
        let actual = sim
            .ledger
            .last()
            .and_then(|b| b.credit.as_ref())
            .and_then(|c| c.stock_sale.as_ref())
            .map_or(0, |r| r.sold_lots);
        if actual != lots {
            return Err("sale forecast did not execute the proposed quantity".into());
        }
        // No hypothetical subsequent sale income: each later live boundary replans.
        sim.world
            .credit
            .as_mut()
            .unwrap()
            .stock_sales
            .as_mut()
            .unwrap()
            .purchase_budget = sim.state.credit.stock_spent;
        while sim.state.month < end {
            sim.step()?;
        }
        let mut deficits = BTreeMap::new();
        for report in sim.reports.iter().filter(|r| r.agent == sale.seller) {
            crate::forecast::needs::accumulate(
                &mut deficits,
                report.needs.iter().map(|(r, n)| (*r, n.deficit)),
            );
        }
        let terminal = sim.state.terminal.contains_key(&sale.seller);
        let admissible =
            !terminal && crate::forecast::needs::within_limits(&deficits, &policy.need_limits);
        alternatives.push(Alternative {
            lots,
            deficits,
            terminal,
            admissible,
        });
    }
    let selected = alternatives
        .iter()
        .rev()
        .find(|a| a.admissible)
        .map(|a| a.lots);
    Ok(Decision {
        through: end - 1,
        alternatives,
        selected_lots: selected.unwrap_or(0),
        feasible: selected.is_some(),
    })
}
