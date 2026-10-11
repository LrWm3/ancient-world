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
