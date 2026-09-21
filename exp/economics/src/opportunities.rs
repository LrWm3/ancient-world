//! Shared discovery interface over physical processes and posted access agreements.
//! Terms stay in their owning catalogs; permissions never replace feasibility.
use crate::{commitments::Agreement, model::*};
use std::collections::{BTreeMap, BTreeSet};

pub type AgentType = u32;
pub const PERSON_TYPE: AgentType = 1;
pub const STATE_TYPE: AgentType = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Process(DefinitionId),
    LandAccess,
    Membership,
    EquipmentTrade,
    StockTrade,
}

/// A state's explicit allow-list. Unclassified types and unlisted actions are denied.
/// Absent policy preserves earlier experimental scenarios.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub authority: AgentId,
    pub membership_offers: Vec<crate::membership::Offer>,
    pub membership_permissions: BTreeSet<(crate::membership::Role, Action)>,
    pub agent_types: BTreeMap<AgentId, AgentType>,
    pub permissions: BTreeSet<(AgentType, Action)>,
}

pub fn permits(world: &World, state: &State, agent: AgentId, action: Action) -> bool {
    world.transaction_policy.as_ref().is_none_or(|p| {
        p.agent_types
            .get(&agent)
            .is_some_and(|kind| p.permissions.contains(&(*kind, action)))
            || state.memberships.values().any(|m| {
                m.member == agent
                    && m.contract().permits(
                        state.month,
                        &crate::agreements::Grant::Membership {
                            organization: p.authority,
                            role: m.role,
                        },
                        crate::agreements::Use::Start,
                    )
                    && p.membership_permissions.contains(&(m.role, action))
            })
    })
}

/// Visible prerequisites can be obtained; visibility is not authorization.
fn discoverable(world: &World, state: &State, agent: AgentId, action: Action) -> bool {
    world.transaction_policy.as_ref().is_none_or(|p| {
        let Some(kind) = p.agent_types.get(&agent) else {
            return false;
        };
        permits(world, state, agent, action)
            || (p.permissions.contains(&(*kind, Action::Membership))
                && p.membership_offers.iter().any(|o| {
                    o.organization == p.authority
                        && o.eligible_type == *kind
                        && p.membership_permissions.contains(&(o.role, action))
                }))
    })
}

#[derive(Clone, Copy, Debug)]
pub enum Opportunity<'a> {
    Environment(&'a ProcessDefinition),
    StateAccess(&'a Agreement),
    Membership(&'a crate::membership::Offer),
}

/// Offers are discoverable before inputs/rights have been acquired. Acceptance and
/// execution recheck availability; discovery itself reserves nothing.
pub fn discover<'a>(world: &'a World, state: &State, agent: AgentId) -> Vec<Opportunity<'a>> {
    let mut rows = Vec::new();
    let mut definitions: Vec<_> = world.definitions.iter().collect();
    definitions.sort_by_key(|d| d.id);
    for d in definitions {
        if d.enabled && discoverable(world, state, agent, Action::Process(d.id)) {
            rows.push(Opportunity::Environment(d));
        }
    }
    let mut access: Vec<_> = world.access_offers.iter().collect();
    access.sort_by_key(|a| a.id);
    for a in access {
        if (a.debtor == agent || world.open_access_offers.contains(&a.id))
            && discoverable(world, state, agent, Action::LandAccess)
            && world
                .transaction_policy
                .as_ref()
                .is_none_or(|p| a.creditor == p.authority)
        {
            rows.push(Opportunity::StateAccess(a));
        }
    }
    if let Some(p) = &world.transaction_policy {
        let mut offers: Vec<_> = p
            .membership_offers
            .iter()
            .filter(|o| {
                p.agent_types.get(&agent) == Some(&o.eligible_type)
                    && discoverable(world, state, agent, Action::Membership)
            })
            .collect();
        offers.sort_by_key(|o| o.id);
        rows.extend(offers.into_iter().map(Opportunity::Membership));
    }
    rows
}

pub fn processes<'a>(
    world: &'a World,
    state: &State,
    agent: AgentId,
) -> Vec<&'a ProcessDefinition> {
    if world.transaction_policy.is_none() {
        return world.definitions.iter().collect();
    }
    crate::offers::discover(world, state, agent)
        .into_iter()
        .filter_map(|offer| match offer.id {
            crate::offers::Id::Process(id) if permits(world, state, agent, Action::Process(id)) => {
                Some(world.definition(id))
            }
            _ => None,
        })
        .collect()
}

/// Backward reachability is only candidate discovery, never proof of feasibility.
/// Seed cycles terminate through visited resource IDs. Ordinary dated rollouts
/// still evaluate stocks, future work, taxes, need consequences and settlement.
pub fn relevant_access(world: &World, state: &State, agent: AgentId) -> BTreeSet<u32> {
    let rows = discover(world, state, agent);
    let mut wanted: BTreeSet<_> = world
        .participants
        .iter()
        .filter(|p| p.agent == agent)
        .flat_map(|p| {
            p.needs
                .iter()
                .filter(|n| n.quantity > 0)
                .map(|n| n.resource)
        })
        .collect();
    let mut processes = BTreeSet::new();
    loop {
        let previous = (wanted.len(), processes.len());
        for row in &rows {
            if let Opportunity::Environment(d) = row
                && d.outputs.iter().any(|a| wanted.contains(&a.resource))
            {
                processes.insert(d.id);
                wanted.extend(
                    d.stages
                        .iter()
                        .flat_map(|s| &s.entry_inputs)
                        .map(|a| a.resource),
                );
            }
        }
        if previous == (wanted.len(), processes.len()) {
            break;
        }
    }
    rows.iter()
        .filter_map(|row| {
            let Opportunity::StateAccess(a) = row else {
                return None;
            };
            let right = world.rights.iter().find(|r| r.id == a.right)?;
            let asset = world.assets.iter().find(|s| s.id == right.asset)?;
            processes
                .iter()
                .any(|id| world.definition(*id).asset_kind == Some(asset.kind))
                .then_some(a.id)
        })
        .collect()
}

pub fn validate(world: &World) -> Result<(), String> {
    if let Some(p) = &world.transaction_policy {
        let known = |id| world.agents.iter().any(|a| a.id == id);
        if !known(p.authority) || p.agent_types.keys().any(|id| !known(*id)) {
            return Err("invalid transaction policy authority/agent".into());
        }
        if p.permissions.iter().chain(p.membership_permissions.iter()).any(|(_, action)| matches!(action, Action::Process(id) if !world.definitions.iter().any(|d| d.id == *id))) {
            return Err("unknown permitted process".into());
        }
        // This first governed marketplace deliberately supports physical and
        // state-access offers only. Other resolvers need bilateral rules first.
        if world.market.is_some()
            || !world.offers.is_empty()
            || !world.bids.is_empty()
            || !world.households.is_empty()
        {
            return Err(
                "governed marketplace currently supports processes and state access only".into(),
            );
        }
    }
    Ok(())
}

pub const RUN_MONTHS: u32 = 36;
const FORECAST_MONTHS: u32 = 18;
const MONTHLY_LABOR: i32 = 3;
const WOOD_CAPACITY: i32 = 12;
const WOOD_REGENERATION: i32 = 1;
const ANNUAL_TAX: i32 = 2;

pub fn scenario() -> Result<(World, State), String> {
    use crate::scenario::*;
    let (mut world, mut state) = named("offer-useful")?;
    world
        .definitions
        .retain(|d| [GROW, CONSUME, PREPARE_FUEL, USE_FUEL].contains(&d.id));
    world.participants[0].capacity.quantity = MONTHLY_LABOR;
    world.decision_horizon = Some(FORECAST_MONTHS);
    world.access_offers[0].payment.quantity = ANNUAL_TAX;
    // Collection draws finite environmental wood, rather than a personal stock
    // that eventually runs out. The resulting fuel satisfies the warmth need.
    world
        .definitions
        .iter_mut()
        .find(|d| d.id == PREPARE_FUEL)
        .unwrap()
        .name = "collect firewood".into();
    state.balances.remove(&(PERSON, RAW_WOOD));
    state
        .balances
        .insert((STATE_AGENT, RAW_WOOD), WOOD_CAPACITY);
    world.pools.push(crate::pools::Pool {
        account: (STATE_AGENT, RAW_WOOD),
        capacity: WOOD_CAPACITY,
        monthly_regeneration: WOOD_REGENERATION,
    });
    world.pool_inputs.push(crate::pools::PoolInput {
        definition: PREPARE_FUEL,
        account: (STATE_AGENT, RAW_WOOD),
    });
    world.transaction_policy = Some(Policy {
        authority: STATE_AGENT,
        membership_offers: vec![],
        membership_permissions: Default::default(),
        agent_types: BTreeMap::from([(PERSON, PERSON_TYPE), (STATE_AGENT, STATE_TYPE)]),
        permissions: [
            Action::LandAccess,
            Action::Process(GROW),
            Action::Process(PREPARE_FUEL),
            Action::Process(CONSUME),
            Action::Process(USE_FUEL),
        ]
        .into_iter()
        .map(|a| (PERSON_TYPE, a))
        .collect(),
    });
    Ok((world, state))
}
