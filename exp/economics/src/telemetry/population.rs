//! Committed person lifecycle receipts; never run demographic forecasts here.
use super::Config;
use crate::model::*;
use serde_json::{Value, json};

pub(super) fn batch(config: &Config, world: &World, batch: &Batch) -> Vec<Value> {
    let mut rows = vec![];
    let Some(d) = &batch.demography else {
        return rows;
    };
    if !config.settlement {
        return rows;
    }
    let selected = |parent, child| {
        config.agents.is_empty()
            || config.agents.contains(&parent)
            || config.agents.contains(&child)
    };
    for b in &d.births {
        if selected(b.parent, b.child) {
            rows.push(json!({"kind":"person_birth", "month":batch.month,
                "batch":batch.id, "parent":b.parent, "child":b.child, "adult_at":b.adult_at,
                "name":world.agents.iter().find(|a|a.id==b.child).map(|a|a.name.as_str())}));
        }
    }
    for child in &d.matured {
        let b = &d.after.births[child];
        if selected(b.parent, *child) {
            rows.push(json!({"kind":"person_matured", "month":batch.month,
                "batch":batch.id,"parent":b.parent,"child":child}));
        }
    }
    for support in &d.support {
        if selected(support.parent, support.child) {
            rows.push(json!({"kind":"dependent_support", "month":batch.month,
                "batch":batch.id,"parent":support.parent,"child":support.child,
                "resource":world.demography.as_ref().map(|c|c.support.resource),
                "at_birth":d.births.iter().any(|b|b.child==support.child),
                "requested":support.requested,"delivered":support.delivered,
                "shortfall":support.requested-support.delivered}));
        }
    }
    rows
}

/// Completed-month stock and flow measures for the registered birth cohort only.
/// Founders and unrelated participant types are deliberately outside this scope.
pub(super) fn closing(
    config: &Config,
    sim: &crate::simulation::Simulation,
    month: u32,
) -> Option<Value> {
    sim.world.demography.as_ref()?;
    let selected = |parent, child| {
        config.agents.is_empty()
            || config.agents.contains(&parent)
            || config.agents.contains(&child)
    };
    let cohort: Vec<_> = sim
        .state
        .demography
        .births
        .values()
        .filter(|b| b.month <= month && selected(b.parent, b.child))
        .collect();
    let terminal = cohort
        .iter()
        .filter(|b| sim.state.terminal.contains_key(&b.child))
        .count();
    let living_dependents = cohort
        .iter()
        .filter(|b| !sim.state.terminal.contains_key(&b.child) && month < b.adult_at)
        .count();
    let living_adults = cohort.len() - terminal - living_dependents;
    let complete = sim
        .ledger
        .iter()
        .any(|b| b.month == month && b.phase == Phase::Open && b.demography.is_some());
    let mut births = 0;
    let mut maturities = 0;
    let mut requested = 0i64;
    let mut delivered = 0i64;
    for d in sim
        .ledger
        .iter()
        .rev()
        .take_while(|b| b.month >= month)
        .filter(|b| b.month == month)
        .filter_map(|b| b.demography.as_ref())
    {
        births += d
            .births
            .iter()
            .filter(|b| selected(b.parent, b.child))
            .count();
        maturities += d
            .matured
            .iter()
            .filter(|id| {
                let b = &d.after.births[id];
                selected(b.parent, b.child)
            })
            .count();
        for r in d.support.iter().filter(|r| selected(r.parent, r.child)) {
            requested += i64::from(r.requested);
            delivered += i64::from(r.delivered);
        }
    }
    Some(
        json!({"kind":"population", "month":month,"scope":"registered_birth_cohort","registered":cohort.len(),
        "living_dependents":living_dependents,"living_adults":living_adults,"terminal":terminal,
        "flows_complete":complete,"births":complete.then_some(births),"maturities":complete.then_some(maturities),"support_requested_including_birth_buffers":complete.then_some(requested),
        "support_delivered_including_birth_buffers":complete.then_some(delivered),"support_shortfall":complete.then_some(requested-delivered)}),
    )
}
