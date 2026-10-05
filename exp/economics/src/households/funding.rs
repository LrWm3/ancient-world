//! Optional collective claim preparation using ordinary member-executable work.
use super::*;
use crate::household_governance::Policy;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Gap {
    pub resource: ResourceId,
    pub required: i128,
    pub held: i128,
    pub shortfall: i128,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Forecast {
    /// Includes this month, ending exclusively at month + months.
    pub months: u32,
    pub gaps: Vec<Gap>,
}

pub(super) fn horizon(a: &Agreement, s: &State) -> Option<u32> {
    match a.governance.policy(s.month) {
        Policy::NeedsThenCommitments { months } => Some(months),
        _ => None,
    }
}

/// Accepted land/forward deliveries in the horizon, plus currently earned wages
/// and loan dues. No assumed future loan amortization, wages, offers or sales.
pub(super) use crate::need_orders::accepted_claims as claims;

pub(super) fn project(
    w: &World,
    s: &State,
    batch: &Batch,
    a: &Agreement,
) -> Result<Option<Forecast>, String> {
    let Some(months) = horizon(a, s) else {
        return Ok(None);
    };
    let closing = needs::after_consumption(w, s, batch)?;
    let gaps = claims(w, &closing.state, a.agent, months)?
        .into_iter()
        .map(|(resource, required)| {
            let held = i128::from(closing.state.balance(a.agent, resource));
            Gap {
                resource,
                required,
                held,
                shortfall: (required - held).max(0),
            }
        })
        .collect();
    Ok(Some(Forecast { months, gaps }))
}

/// Resource ID is an explicit stable priority; unlike units are never added.
/// This ranks funding preparation, not creditors at collection.
pub(super) fn compare(a: &Forecast, b: &Forecast) -> std::cmp::Ordering {
    a.gaps
        .iter()
        .map(|g| (g.resource, g.shortfall))
        .cmp(b.gaps.iter().map(|g| (g.resource, g.shortfall)))
}

/// Direct producers only. Existing resolution still checks actual inputs,
/// capacity, storage and rights. No new authority over private stock is created.
pub(crate) fn candidates(
    w: &World,
    s: &State,
    member: AgentId,
) -> Result<Vec<DefinitionId>, String> {
    let Some(parent) = parent(w, s, member) else {
        return Ok(vec![]);
    };
    let a = w.households.iter().find(|a| a.agent == parent).unwrap();
    let Some(months) = horizon(a, s) else {
        return Ok(vec![]);
    };
    if !market::active(w, s, parent) || s.terminal.contains_key(&member) {
        return Ok(vec![]);
    }
    let claims = claims(w, s, parent, months)?;
    let mut definitions: Vec<_> = crate::opportunities::processes(w, s, member)
        .into_iter()
        .filter(|d| {
            d.execution == Execution::Productive
                && a.governance
                    .constitution
                    .activities
                    .as_ref()
                    .is_none_or(|ids| ids.contains(&d.id))
                && d.outputs.iter().any(|output| {
                    claims.get(&output.resource).copied().unwrap_or(0)
                        > i128::from(s.balance(parent, output.resource))
                })
        })
        .map(|d| d.id)
        .collect();
    definitions.sort_unstable();
    Ok(definitions)
}
