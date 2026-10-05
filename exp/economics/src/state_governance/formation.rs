//! Explicit first-state founding. The scenario supplies bootstrap eligibility and
//! immutable terms; signatures express consent, not an autonomous political plan.
use super::{Charter, Constitution, Governance};
use crate::{
    membership::{self, CITIZEN},
    model::*,
    opportunities::{Action, PERSON_TYPE, Policy, STATE_TYPE},
};
use std::collections::BTreeSet;

/// Posted bootstrap terms, available only before a legal authority exists.
/// The initial typed roster is scenario data, not inferred from labor capacity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub agent: Agent,
    pub eligible_founders: BTreeSet<AgentId>,
    pub minimum_founders: usize,
    pub constitution: Constitution,
    pub charter: Charter,
    pub law: Policy,
    pub citizenship_offer: u32,
}

/// A complete founding proposal, retained as the accepted agreement after commit.
/// Every listed founder signs these exact terms; the canonical order grants no
/// priority, ownership, extra capacity or financial claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agreement {
    pub terms: Template,
    pub founders: Vec<AgentId>,
    pub formed: u32,
}

fn check_terms(t: &Template, founders: &[AgentId]) -> Result<(), String> {
    let unique: BTreeSet<_> = founders.iter().copied().collect();
    if t.agent.name.trim().is_empty()
        || t.minimum_founders == 0
        || founders.len() < t.minimum_founders
        || unique.len() != founders.len()
        || !unique.contains(&t.charter.founder)
        || !unique.is_subset(&t.eligible_founders)
        || t.law.authority != t.agent.id
        || t.law.agent_types.get(&t.agent.id) != Some(&STATE_TYPE)
        || t.eligible_founders
            .iter()
            .any(|id| t.law.agent_types.get(id) != Some(&PERSON_TYPE))
        || !t.law.membership_offers.iter().any(|o| {
            o.id == t.citizenship_offer
                && o.organization == t.agent.id
                && o.role == CITIZEN
                && o.eligible_type == PERSON_TYPE
        })
    {
        return Err("invalid state founding terms or signatures".into());
    }
    Ok(())
}

/// Read-only feasibility includes ordinary world, governance and law validation.
/// Nothing is reserved; acceptance repeats all checks against live state.
pub fn propose(w: &World, s: &State, signatures: &[AgentId]) -> Result<Agreement, String> {
    let mut founders = signatures.to_vec();
    founders.sort_unstable();
    let agreement = Agreement {
        terms: w.state_founding.clone().ok_or("no state founding offer")?,
        founders,
        formed: s.month,
    };
    stage(w, s, &agreement)?;
    Ok(agreement)
}

fn stage(w: &World, s: &State, a: &Agreement) -> Result<(World, State), String> {
    if s.phase != Phase::Open
        || a.formed != s.month
        || s.month == 0
        || w.transaction_policy.is_some()
        || w.state_governance.is_some()
        || w.state_founding.as_ref() != Some(&a.terms)
        || w.agents.iter().any(|p| p.id == a.terms.agent.id)
        || a.founders.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(
            "state founding requires a fresh identity and current Open bootstrap offer".into(),
        );
    }
    // Reject pre-existing dangling assets/accounts rather than letting founding
    // retroactively legitimize property attributed to an uncreated identity.
    crate::settlement::validate_world(w, s)?;
    check_terms(&a.terms, &a.founders)?;
    if a.terms
        .eligible_founders
        .iter()
        .any(|id| !w.agents.iter().any(|p| p.id == *id))
        || a.founders.iter().any(|id| s.terminal.contains_key(id))
        || s.terminal.contains_key(&a.terms.agent.id)
        || s.balances.keys().any(|(id, _)| *id == a.terms.agent.id)
    {
        return Err("state founders must be known living persons; state must start empty".into());
    }
    let mut world = w.clone();
    let mut state = s.clone();
    world.agents.push(a.terms.agent.clone());
    world.transaction_policy = Some(a.terms.law.clone());
    world.state_governance = Some(Governance {
        formation: Some(a.clone()),
        state: a.terms.agent.id,
        formed: a.formed,
        constitution: a.terms.constitution.clone(),
        charter: a.terms.charter.clone(),
        ballots: vec![],
        changes: vec![],
    });
    // Citizenship is an explicit founding grant. Founders need not already be
    // citizens, but the new law must permit their membership action. Ordinary
    // later entrants still accept the posted offer through Acquire settlement.
    for &member in &a.founders {
        if !crate::opportunities::permits(&world, &state, member, Action::Membership) {
            return Err("founding law prohibits founder citizenship".into());
        }
        let agreement = membership::Agreement {
            member,
            organization: a.terms.agent.id,
            role: CITIZEN,
            source_offer: a.terms.citizenship_offer,
            accepted_month: a.formed,
        };
        if state
            .memberships
            .insert((member, a.terms.agent.id, CITIZEN), agreement)
            .is_some()
        {
            return Err("founder already has citizenship".into());
        }
    }
    crate::settlement::validate_world(&world, &state)?;
    Ok((world, state))
}

/// Publish identity, initial law, office and citizenship together, or nothing.
/// This does not advance the clock or mint, transfer, reserve or pool resources.
pub fn accept(w: &mut World, s: &mut State, agreement: Agreement) -> Result<(), String> {
    let (world, state) = stage(w, s, &agreement)?;
    *w = world;
    *s = state;
    Ok(())
}

/// Historical terms are not re-admitted against today's policy or mortality.
pub fn validate(w: &World, s: &State) -> Result<(), String> {
    let Some(g) = &w.state_governance else {
        return Ok(());
    };
    let Some(a) = &g.formation else {
        return Ok(());
    };
    check_terms(&a.terms, &a.founders)?;
    let p = w
        .transaction_policy
        .as_ref()
        .ok_or("formed state lacks law")?;
    // Later institutions may add classifications. Founding classifications and
    // the base law stay fixed; governors select the separate dated policy menu.
    let mut base = p.clone();
    base.agent_types
        .retain(|id, _| a.terms.law.agent_types.contains_key(id));
    if a.formed != g.formed
        || a.terms.agent.id != g.state
        || a.terms.constitution != g.constitution
        || a.terms.charter != g.charter
        || base != a.terms.law
        || a.founders.windows(2).any(|pair| pair[0] >= pair[1])
        || a.founders.iter().any(|id| {
            s.memberships.get(&(*id, g.state, CITIZEN)).is_none_or(|m| {
                m.accepted_month != a.formed || m.source_offer != a.terms.citizenship_offer
            })
        })
    {
        return Err(
            "state founding receipt disagrees with constitution, law or citizenship".into(),
        );
    }
    Ok(())
}
