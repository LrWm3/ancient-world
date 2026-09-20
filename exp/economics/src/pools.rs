//! Finite shared stock pools; fixed regeneration occurs once at Open.
use crate::model::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pool {
    pub account: Account,
    pub capacity: i32,
    pub monthly_regeneration: i32,
}
/// Catalog permission: this process consumes an input from the shared account.
/// All operators of this definition are eligible; future access rules can narrow it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolInput {
    pub definition: DefinitionId,
    pub account: Account,
}

pub fn input_account(
    world: &World,
    definition: DefinitionId,
    operator: AgentId,
    resource: ResourceId,
) -> Account {
    world
        .pool_inputs
        .iter()
        .find(|i| i.definition == definition && i.account.1 == resource)
        .map(|i| i.account)
        .unwrap_or((operator, resource))
}

pub fn regeneration(world: &World, state: &State) -> Vec<Effect> {
    let mut pools: Vec<_> = world.pools.iter().collect();
    pools.sort_by_key(|p| p.account);
    pools
        .into_iter()
        .filter_map(|p| {
            let delta =
                (p.capacity - state.balance(p.account.0, p.account.1)).min(p.monthly_regeneration);
            (delta > 0).then_some(Effect {
                account: p.account,
                delta,
            })
        })
        .collect()
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let mut accounts = BTreeSet::new();
    for p in &world.pools {
        if !accounts.insert(p.account)
            || p.capacity < 0
            || p.monthly_regeneration < 0
            || state.balance(p.account.0, p.account.1) > p.capacity
            || !world.agents.iter().any(|a| a.id == p.account.0)
            || !world
                .resources
                .iter()
                .any(|r| r.id == p.account.1 && r.kind == ResourceKind::Stock)
        {
            return Err("invalid shared pool".into());
        }
    }
    let mut inputs = BTreeSet::new();
    for i in &world.pool_inputs {
        if !inputs.insert((i.definition, i.account.1))
            || !accounts.contains(&i.account)
            || !world.definitions.iter().any(|d| {
                d.id == i.definition
                    && d.execution == Execution::Productive
                    && d.stages
                        .iter()
                        .any(|s| s.entry_inputs.iter().any(|a| a.resource == i.account.1))
            })
        {
            return Err("invalid shared pool input".into());
        }
    }
    Ok(())
}
