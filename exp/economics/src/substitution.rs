//! Deterministic whole-lot substitution; all recipes draw from one stock budget.
use crate::model::*;
use std::collections::BTreeMap;

pub fn recipes(world: &World, need: ResourceId) -> Vec<&ProcessDefinition> {
    let mut definitions: Vec<_> = world
        .definitions
        .iter()
        .filter(|d| {
            d.enabled && d.execution == Execution::Consumption && d.outputs[0].resource == need
        })
        .collect();
    // Higher yield per unit input first; stable IDs resolve equal yields.
    definitions.sort_by(|a, b| {
        let ai = &a.stages[0].entry_inputs[0];
        let bi = &b.stages[0].entry_inputs[0];
        (i64::from(b.outputs[0].quantity) * i64::from(ai.quantity))
            .cmp(&(i64::from(a.outputs[0].quantity) * i64::from(bi.quantity)))
            .then(a.id.cmp(&b.id))
    });
    definitions
}

pub fn stocks(state: &State, agent: AgentId) -> BTreeMap<ResourceId, i128> {
    state
        .balances
        .iter()
        .filter(|((a, _), _)| *a == agent)
        .map(|((_, r), q)| (*r, i128::from(*q)))
        .collect()
}

pub fn earmarks(world: &World, state: &State, agent: AgentId) -> BTreeMap<ResourceId, i128> {
    world
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Stock)
        .map(|r| {
            (
                r.id,
                crate::commitments::projected_claims(
                    world,
                    state,
                    agent,
                    r.id,
                    u64::from(world.horizon),
                )
                .values()
                .sum(),
            )
        })
        .collect()
}

/// Spend unencumbered stocks first, then release earmarks if needed for food.
/// Returns whole recipe lots and unfulfilled demand. No stock is counted twice,
/// including when multiple recipes consume the same stock or needs share inputs.
pub fn allocate(
    world: &World,
    need: ResourceId,
    desired: i128,
    stocks: &mut BTreeMap<ResourceId, i128>,
    earmarks: &BTreeMap<ResourceId, i128>,
) -> (Vec<(DefinitionId, i128)>, i128) {
    let recipes = recipes(world, need);
    let mut remaining = desired.max(0);
    let mut allocations = Vec::new();
    for release in [false, true] {
        for d in &recipes {
            let input = &d.stages[0].entry_inputs[0];
            let stock = stocks.entry(input.resource).or_default();
            let protected = if release {
                0
            } else {
                *earmarks.get(&input.resource).unwrap_or(&0)
            };
            let available = (*stock - protected).max(0);
            let output = i128::from(d.outputs[0].quantity);
            let lots =
                ((remaining + output - 1) / output).min(available / i128::from(input.quantity));
            if lots > 0 {
                *stock -= lots * i128::from(input.quantity);
                remaining = (remaining - lots * output).max(0);
                allocations.push((d.id, lots));
            }
        }
    }
    (allocations, remaining)
}
