//! Fixed shared storage capacity; resource weights are catalog data, not money values.
use crate::model::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Storage {
    /// Space per stock unit. Omitted resources use no capacity in this pilot.
    pub weights: BTreeMap<ResourceId, i32>,
    /// Omitted owners retain unconstrained storage (legacy scenarios).
    pub capacities: BTreeMap<AgentId, i32>,
}

pub fn usage(world: &World, balances: &BTreeMap<Account, i32>) -> BTreeMap<AgentId, i128> {
    let mut used = BTreeMap::new();
    for ((owner, resource), quantity) in balances {
        *used.entry(*owner).or_default() +=
            i128::from(*quantity) * i128::from(*world.storage.weights.get(resource).unwrap_or(&0));
    }
    used
}

pub fn apply(world: &World, used: &mut BTreeMap<AgentId, i128>, effects: &[Effect]) {
    for e in effects {
        *used.entry(e.account.0).or_default() += i128::from(e.delta)
            * i128::from(*world.storage.weights.get(&e.account.1).unwrap_or(&0));
    }
}

pub fn fits(world: &World, used: &BTreeMap<AgentId, i128>, effects: &[Effect]) -> bool {
    let mut after = used.clone();
    apply(world, &mut after, effects);
    world.storage.capacities.iter().all(|(owner, cap)| {
        world.households.iter().any(|h| h.adults.contains(owner))
            || after.get(owner).copied().unwrap_or(0) <= i128::from(*cap)
    }) && world
        .households
        .iter()
        .all(|h| shared_room(world, &after, h) >= 0)
}

pub fn room(
    world: &World,
    used: &BTreeMap<AgentId, i128>,
    owner: AgentId,
    resource: ResourceId,
) -> i32 {
    let weight = *world.storage.weights.get(&resource).unwrap_or(&0);
    if weight <= 0 {
        return i32::MAX;
    }
    if let Some(h) = world
        .households
        .iter()
        .find(|h| h.agent == owner || h.adults.contains(&owner))
    {
        let private = if owner == h.agent {
            0
        } else {
            match world.storage.capacities.get(&owner) {
                Some(cap) => i128::from(cap - cap / crate::households::POOL_DIVISOR),
                None => return i32::MAX,
            }
        };
        let private_room = (private - used.get(&owner).copied().unwrap_or(0)).max(0);
        return ((private_room + shared_room(world, used, h).max(0)) / i128::from(weight))
            .min(i128::from(i32::MAX)) as i32;
    }
    match world.storage.capacities.get(&owner) {
        Some(cap) => ((i128::from(*cap) - used.get(&owner).copied().unwrap_or(0)).max(0)
            / i128::from(weight))
        .min(i128::from(i32::MAX)) as i32,
        None => i32::MAX,
    }
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    for (r, w) in &world.storage.weights {
        if *w < 0
            || !world
                .resources
                .iter()
                .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
        {
            return Err("invalid storage weight".into());
        }
    }
    for (owner, capacity) in &world.storage.capacities {
        if *capacity < 0 || !world.agents.iter().any(|a| a.id == *owner) {
            return Err("invalid storage capacity".into());
        }
    }
    if !fits(world, &usage(world, &state.balances), &[]) {
        return Err("storage capacity exceeded".into());
    }
    Ok(())
}

fn shared_room(
    world: &World,
    used: &BTreeMap<AgentId, i128>,
    h: &crate::households::Agreement,
) -> i128 {
    let mut remaining = -used.get(&h.agent).copied().unwrap_or(0);
    for m in &h.adults {
        let Some(cap) = world.storage.capacities.get(m) else {
            return i128::from(i32::MAX) * i128::from(i32::MAX);
        };
        let shared = cap / crate::households::POOL_DIVISOR;
        remaining += i128::from(shared)
            - (used.get(m).copied().unwrap_or(0) - i128::from(cap - shared)).max(0);
    }
    remaining
}
