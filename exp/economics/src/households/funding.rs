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

/// Own accepted claims plus the member land dues and forwards already covered by ordinary
/// household support. Protect current private consumption before counting cover.
/// This prepares collective stocks; it never assumes or transfers a member's debt.
pub(super) fn claims(
    w: &World,
    s: &State,
    agent: AgentId,
    months: u32,
) -> Result<BTreeMap<ResourceId, i128>, String> {
    let mut result = crate::need_orders::accepted_claims(w, s, agent, months)?;
    let Some(h) = w.households.iter().find(|h| h.agent == agent) else {
        return Ok(result);
    };
    for member in members(h, s) {
        let mut available = crate::substitution::stocks(s, member);
        crate::need_orders::consume_person(w, s, member, 1, &mut available, true);
        for resource in w.resources.iter().filter(|r| r.kind == ResourceKind::Stock) {
            let due: i128 =
                crate::commitments::projected_claims(w, s, member, resource.id, u64::from(months))
                    .values()
                    .sum();
            let gap = (due - available.get(&resource.id).copied().unwrap_or(0)).max(0);
            if gap > 0 {
                *result.entry(resource.id).or_default() += gap;
            }
        }
    }
    Ok(result)
}

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
