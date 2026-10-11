//! Initial, signed operating authority for existing organizations. This adapter
//! does not found agents, transfer ownership, or grant legal process permissions.
use super::{Config, Controller};
use crate::{
    governance::Leadership,
    model::*,
    opportunities::{self, Action, PERSON_TYPE},
};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agreement {
    pub organization: AgentId,
    pub members: BTreeSet<AgentId>,
    pub founder: AgentId,
    pub leadership: Leadership,
    pub term_months: u32,
    /// Constitutional bounds on the operating programs, additional to state law.
    pub processes: BTreeSet<DefinitionId>,
}

/// Unanimous consent is supplied explicitly; an employee or owner does not
/// automatically become a member or governor. Initial setup only, before Open.
pub fn install_initial(
    w: &mut World,
    s: &State,
    agreement: Agreement,
    signatures: &BTreeSet<AgentId>,
    config: Config,
) -> Result<(), String> {
    if s.month != 1
        || s.phase != Phase::Open
        || s.terminal.contains_key(&agreement.organization)
        || w.agency.contains_key(&agreement.organization)
        || &agreement.members != signatures
        || !w.agents.iter().any(|a| a.id == agreement.organization)
        || w.households
            .iter()
            .any(|a| a.agent == agreement.organization)
        || w.state_governance
            .as_ref()
            .is_some_and(|g| g.state == agreement.organization)
    {
        return Err("operating agreement requires unanimous initial organization consent".into());
    }
    let kind = *w
        .transaction_policy
        .as_ref()
        .and_then(|p| p.agent_types.get(&agreement.organization))
        .ok_or("operating organization needs a legal type")?;
    if agreement.members.iter().any(|id| {
        s.terminal.contains_key(id)
            || !opportunities::permits(w, s, *id, Action::GovernOrganization(kind))
    }) {
        return Err("operating agreement signer lacks legal authority".into());
    }
    let mut candidate = w.clone();
    let mut controller = Controller::new(config);
    controller.organization = Some(agreement.clone());
    candidate.agency.insert(agreement.organization, controller);
    crate::settlement::validate_world(&candidate, s)?;
    *w = candidate;
    Ok(())
}

pub(crate) fn validate(w: &World, agent: AgentId, a: &Agreement) -> Result<(), String> {
    if a.organization != agent
        || a.members.is_empty()
        || !a.members.contains(&a.founder)
        || a.term_months == 0
        || a.leadership == Leadership::Elected
        || a.members.contains(&agent)
        || a.members.iter().any(|id| {
            !w.agents.iter().any(|p| p.id == *id)
                || !w
                    .transaction_policy
                    .as_ref()
                    .is_some_and(|p| p.agent_types.get(id) == Some(&PERSON_TYPE))
        })
        || a.processes.iter().any(|id| {
            !w.definitions
                .iter()
                .any(|d| d.id == *id && d.execution == Execution::Productive)
        })
        || w.households.iter().any(|h| h.agent == agent)
        || w.state_governance
            .as_ref()
            .is_some_and(|g| g.state == agent)
    {
        return Err("invalid static operating agreement".into());
    }
    Ok(())
}

fn death(w: &World, s: &State, id: AgentId) -> Option<u32> {
    s.terminal
        .get(&id)
        .or_else(|| w.governance_observation.as_ref()?.deaths.get(&id))
        .map(|d| d.month)
}

/// Calendar rotation has a fixed roster. A deceased holder leaves a vacancy for
/// that term; death does not rewrite earlier authorizations or advance the queue.
pub(crate) fn at_open(w: &World, s: &State, a: &Agreement, month: u32) -> Option<AgentId> {
    if month == 0 || death(w, s, a.organization).is_some_and(|m| m < month) {
        return None;
    }
    let members: Vec<_> = a.members.iter().copied().collect();
    let anchor = members.iter().position(|id| *id == a.founder)?;
    let index = match a.leadership {
        Leadership::FixedFounder => anchor,
        Leadership::Rotating => {
            (anchor + ((month - 1) / a.term_months) as usize % members.len()) % members.len()
        }
        Leadership::Elected => return None,
    };
    let holder = members[index];
    death(w, s, holder)
        .is_none_or(|m| m >= month)
        .then_some(holder)
}

pub(crate) fn current(w: &World, s: &State, a: &Agreement) -> Option<AgentId> {
    let holder = at_open(w, s, a, s.month)?;
    (death(w, s, holder).is_none() && death(w, s, a.organization).is_none()).then_some(holder)
}
