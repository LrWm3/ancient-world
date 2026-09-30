//! Posted contingent terms use explicit applications; origination checks law once.
//! A contingent cap is not current funding, a reserve, or an underwriting forecast.
use crate::{
    credit, laws,
    model::*,
    recovery::{self, Guarantee},
};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application {
    pub guarantee: u32,
    pub month: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    NotPosted,
    WrongApplicant,
    OutsideTerm,
    AlreadyAccepted,
    Inactive,
    NotPermitted,
}

pub fn accepted_month(world: &World, book: &credit::Book, g: &Guarantee) -> Option<u32> {
    if world.recovery.posted_guarantees.contains(&g.id) {
        book.recovery.accepted_guarantees.get(&g.id).copied()
    } else {
        Some(g.from)
    }
}

pub fn acceptance(world: &World, state: &State, id: u32, agent: AgentId) -> Result<(), Rejection> {
    let g = world
        .recovery
        .guarantees
        .iter()
        .find(|g| g.id == id)
        .filter(|g| world.recovery.posted_guarantees.contains(&g.id))
        .ok_or(Rejection::NotPosted)?;
    if agent != g.guarantor {
        return Err(Rejection::WrongApplicant);
    }
    if !(g.from..=g.through).contains(&state.month) {
        return Err(Rejection::OutsideTerm);
    }
    if state.credit.recovery.accepted_guarantees.contains_key(&id) {
        return Err(Rejection::AlreadyAccepted);
    }
    if state.terminal.contains_key(&agent)
        || recovery::active(world, &state.credit, agent).is_some()
        || !crate::households::market::active(world, state, agent)
        || world.households.iter().any(|h| {
            h.agent == agent && crate::households::dissolution::winding_at(h, state.month).is_some()
        })
    {
        return Err(Rejection::Inactive);
    }
    if !laws::evaluate_agreement(world, state, agent, laws::AgreementForm::Guarantee).allowed {
        return Err(Rejection::NotPermitted);
    }
    Ok(())
}

pub fn discover<'a>(world: &'a World, state: &State, agent: AgentId) -> Vec<&'a Guarantee> {
    let mut offers: Vec<_> = world
        .recovery
        .guarantees
        .iter()
        .filter(|g| acceptance(world, state, g.id, agent).is_ok())
        .collect();
    offers.sort_by_key(|g| g.id);
    offers
}

pub(crate) fn validate(world: &World, state: &State) -> Result<(), String> {
    let config = &world.recovery;
    if config
        .posted_guarantees
        .iter()
        .any(|id| !config.guarantees.iter().any(|g| g.id == *id))
    {
        return Err("unknown posted guarantee".into());
    }
    let mut seen = BTreeSet::new();
    for a in &config.guarantee_applications {
        if !seen.insert(a.guarantee)
            || !config.posted_guarantees.contains(&a.guarantee)
            || config
                .guarantees
                .iter()
                .find(|g| g.id == a.guarantee)
                .is_none_or(|g| !(g.from..=g.through).contains(&a.month))
        {
            return Err("invalid dated guarantee application".into());
        }
    }
    for (&id, &month) in &state.credit.recovery.accepted_guarantees {
        if month > state.month
            || (month == state.month
                && matches!(state.phase, Phase::Open | Phase::Due | Phase::Acquire))
            || !config
                .guarantee_applications
                .iter()
                .any(|a| a.guarantee == id && a.month == month)
        {
            return Err("guarantee acceptance without dated consent".into());
        }
    }
    Ok(())
}

pub(crate) fn apply(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
) -> Result<(), String> {
    let mut applications: Vec<_> = world
        .recovery
        .guarantee_applications
        .iter()
        .filter(|a| a.month == state.month)
        .collect();
    applications.sort_by_key(|a| a.guarantee);
    for a in applications {
        if out
            .after
            .recovery
            .accepted_guarantees
            .contains_key(&a.guarantee)
        {
            continue;
        }
        let g = world
            .recovery
            .guarantees
            .iter()
            .find(|g| g.id == a.guarantee)
            .ok_or("missing posted guarantee terms")?;
        let current = crate::recovery_claims::current(state, out);
        let rejection = acceptance(world, &current, a.guarantee, g.guarantor).err();
        if rejection.is_none() {
            out.after
                .recovery
                .accepted_guarantees
                .insert(a.guarantee, state.month);
        }
        out.recovery.push(recovery::Receipt::GuaranteeAdmission {
            guarantee: a.guarantee,
            rejection,
        });
    }
    Ok(())
}
