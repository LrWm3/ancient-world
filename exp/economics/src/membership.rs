//! Accepted membership agreements grant scoped permissions without changing agent type.
use crate::{
    model::*,
    opportunities::{Action, AgentType},
};
use std::collections::BTreeSet;

pub type Role = u32;
pub const CITIZEN: Role = 1;

/// A reusable offer: each eligible person may accept once for this organization/role.
/// This first agreement has no upkeep, breach, expiration or exit conditions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub id: u32,
    pub organization: AgentId,
    pub role: Role,
    pub eligible_type: AgentType,
}

/// The accepted agreement is also the authoritative membership relationship.
/// (member, organization, role) identifies it; source offer and date preserve provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agreement {
    pub member: AgentId,
    pub organization: AgentId,
    pub role: Role,
    pub source_offer: u32,
    pub accepted_month: u32,
}

pub fn acceptance(
    world: &World,
    state: &State,
    offer: u32,
    member: AgentId,
) -> Result<Agreement, String> {
    let p = world
        .transaction_policy
        .as_ref()
        .ok_or("membership requires state policy")?;
    let offer = p
        .membership_offers
        .iter()
        .find(|o| o.id == offer)
        .ok_or("unknown membership offer")?;
    if state.phase != Phase::Acquire
        || offer.organization != p.authority
        || p.agent_types.get(&member) != Some(&offer.eligible_type)
        || !crate::opportunities::permits(world, state, member, Action::Membership)
        || state.terminal.contains_key(&member)
        || state.terminal.contains_key(&offer.organization)
        || state
            .memberships
            .contains_key(&(member, offer.organization, offer.role))
    {
        return Err("unavailable membership offer".into());
    }
    Ok(Agreement {
        member,
        organization: offer.organization,
        role: offer.role,
        source_offer: offer.id,
        accepted_month: state.month,
    })
}

/// Candidate prerequisite bundles. At most one new membership is accepted in a
/// boundary in this first slice; ordinary settlement validates each preview.
pub fn candidates(world: &World, state: &State, member: AgentId) -> Vec<(u32, AgentId)> {
    let Some(p) = &world.transaction_policy else {
        return vec![];
    };
    let mut rows: Vec<_> = p
        .membership_offers
        .iter()
        .filter(|o| acceptance(world, state, o.id, member).is_ok())
        .map(|o| (o.id, member))
        .collect();
    rows.sort_unstable();
    rows
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let Some(p) = &world.transaction_policy else {
        return if state.memberships.is_empty() {
            Ok(())
        } else {
            Err("membership without policy".into())
        };
    };
    let mut ids = BTreeSet::new();
    for offer in &p.membership_offers {
        if !ids.insert(offer.id) || offer.organization != p.authority {
            return Err("invalid membership offer authority or duplicate ID".into());
        }
    }
    for (&key, agreement) in &state.memberships {
        if key != (agreement.member, agreement.organization, agreement.role)
            || agreement.accepted_month == 0
            || agreement.accepted_month > state.month
            || !world.agents.iter().any(|a| a.id == agreement.member)
            || !p.membership_offers.iter().any(|o| {
                o.id == agreement.source_offer
                    && o.organization == agreement.organization
                    && o.role == agreement.role
            })
        {
            return Err("invalid accepted membership agreement".into());
        }
    }
    Ok(())
}

pub fn scenario() -> Result<(World, State), String> {
    use crate::{
        opportunities::{Action, PERSON_TYPE},
        scenario::*,
    };
    let (mut w, s) = crate::opportunities::scenario()?;
    let p = w.transaction_policy.as_mut().unwrap();
    for action in [Action::LandAccess, Action::Process(GROW)] {
        p.permissions.remove(&(PERSON_TYPE, action));
        p.membership_permissions.insert((CITIZEN, action));
    }
    p.permissions.insert((PERSON_TYPE, Action::Membership));
    p.membership_offers.push(Offer {
        id: 1,
        organization: STATE_AGENT,
        role: CITIZEN,
        eligible_type: PERSON_TYPE,
    });
    Ok((w, s))
}
