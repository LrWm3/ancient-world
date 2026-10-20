//! Dated process-flow scoring. Targets are preferences, never work entitlements.
use super::{
    completions,
    objectives::{self, Metric, Objective, Scope},
};
use crate::model::*;
use std::collections::BTreeSet;

pub(super) fn shortfall(
    rows: &[completions::Completion],
    scope: &BTreeSet<AgentId>,
    definition: DefinitionId,
    target: u32,
    from: u32,
    through: u32,
) -> i128 {
    (from..=through)
        .map(|month| {
            scope
                .iter()
                .map(|agent| {
                    let completed = rows
                        .iter()
                        .filter(|c| {
                            c.month == month && c.operator == *agent && c.definition == definition
                        })
                        .map(|c| c.process)
                        .collect::<BTreeSet<_>>()
                        .len() as i128;
                    (i128::from(target) - completed).max(0)
                })
                .sum::<i128>()
        })
        .sum()
}

/// At Open, process goals observe only the previous completed month. Month one
/// has no prior interval, so contributes no shortfall. Other goals keep their
/// existing opening-state semantics. Current members define live beneficiaries;
/// projection callers separately freeze their opening beneficiaries.
pub fn observe(
    w: &World,
    s: &State,
    ledger: &[Batch],
    agent: AgentId,
    goals: &[Objective],
) -> Result<Vec<i128>, String> {
    let rows = goals
        .iter()
        .any(|o| matches!(o.metric, Metric::ProcessShortfall { .. }))
        .then(|| {
            completions::observe_all(ledger, s.month.saturating_sub(1), s.month.saturating_sub(1))
        });
    goals
        .iter()
        .map(|o| {
            if let Metric::ProcessShortfall { definition, target } = o.metric {
                if s.month <= 1 {
                    return Ok(0);
                }
                if !ledger
                    .iter()
                    .any(|b| b.month == s.month - 1 && b.phase == Phase::Close)
                {
                    return Err("process observation requires previous committed Close".into());
                }
                let scope = match &o.scope {
                    Scope::Organization => [agent].into(),
                    Scope::Agents(ids) => ids.clone(),
                    Scope::Members => objectives::members(w, s, agent),
                };
                Ok(shortfall(
                    rows.as_ref().unwrap(),
                    &scope,
                    definition,
                    target,
                    s.month - 1,
                    s.month - 1,
                ))
            } else {
                Ok(objectives::measure(w, s, &[], agent, std::slice::from_ref(o))?[0])
            }
        })
        .collect()
}
