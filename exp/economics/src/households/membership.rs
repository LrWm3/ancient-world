//! Explicit adult accession and voluntary exit at the unopened monthly boundary.
//! Founders, legal receipts and past authority remain historical records.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Join {
        person: AgentId,
        /// Existing living members and the entrant explicitly consent.
        signatories: Vec<AgentId>,
        admission: crate::laws::households::Admission,
    },
    /// The named member requests exit; no household veto or property payout.
    Leave { person: AgentId },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub month: u32,
    pub action: Action,
}

pub fn roster_at(a: &Agreement, month: u32) -> Vec<AgentId> {
    if month < a.formed {
        return vec![];
    }
    let mut people = a.adults.clone();
    for change in a.membership.iter().filter(|c| c.month <= month) {
        match change.action {
            Action::Join { person, .. } => people.push(person),
            Action::Leave { person } => people.retain(|id| *id != person),
        }
    }
    people
}
/// All accepted changes have already taken effect. Storage intentionally includes
/// terminal members until estate handling supplies a distinct capacity disposition.
pub fn current(a: &Agreement) -> Vec<AgentId> {
    roster_at(a, u32::MAX)
}
pub fn ever_member(a: &Agreement, person: AgentId) -> bool {
    a.adults.contains(&person)
        || a.membership
            .iter()
            .any(|c| matches!(c.action, Action::Join { person: id, .. } if id == person))
}

fn boundary(a: &Agreement, s: &State) -> Result<(), String> {
    if s.phase != Phase::Open
        || s.pending_production.is_some()
        || s.month <= a.formed
        || a.membership.last().is_some_and(|c| c.month >= s.month)
        || a.governance
            .changes
            .iter()
            .any(|c| c.issued_month == s.month)
        || a.governance
            .ballots
            .iter()
            .any(|c| c.issued_month == s.month)
    {
        return Err("membership changes require Open before monthly work or governance instructions; one change per household per month".into());
    }
    Ok(())
}

/// Admit an adult on unchanged founding terms. This first slice uses the existing
/// household recognition/founding permission for accession, with fresh evidence
/// for all signatories. Recruitment discovery and negotiated terms are separate.
pub fn join(
    world: &mut World,
    state: &State,
    household: AgentId,
    person: AgentId,
    signatories: Vec<AgentId>,
) -> Result<(), String> {
    let a = world
        .households
        .iter()
        .find(|a| a.agent == household)
        .ok_or("unknown household")?;
    boundary(a, state)?;
    if parent(world, state, person).is_some() || state.terminal.contains_key(&person) {
        return Err("entrant must be a living unaffiliated adult".into());
    }
    let mut terms = a.clone();
    terms.adults = members(a, state).collect();
    terms.adults.push(person);
    terms.formed = state.month;
    terms.membership.clear();
    let admission = crate::laws::households::admit(world, state, &terms)?;
    accept(
        world,
        state,
        household,
        Action::Join {
            person,
            signatories,
            admission,
        },
    )
}

/// Explicit voluntary exit. Existing personal and household property, processes,
/// contracts and claims do not move. Storage must fit the reduced pool first.
pub fn leave(
    world: &mut World,
    state: &State,
    household: AgentId,
    person: AgentId,
) -> Result<(), String> {
    let a = world
        .households
        .iter()
        .find(|a| a.agent == household)
        .ok_or("unknown household")?;
    boundary(a, state)?;
    if !members(a, state).any(|id| id == person) {
        return Err("exit requires a living member's instruction".into());
    }
    accept(world, state, household, Action::Leave { person })
}

fn accept(
    world: &mut World,
    state: &State,
    household: AgentId,
    action: Action,
) -> Result<(), String> {
    let mut candidate = world.clone();
    candidate
        .households
        .iter_mut()
        .find(|a| a.agent == household)
        .unwrap()
        .membership
        .push(Change {
            month: state.month,
            action,
        });
    crate::settlement::validate_world(&candidate, state)?;
    *world = candidate;
    Ok(())
}

pub(super) fn validate(world: &World, state: &State) -> Result<(), String> {
    let mut dates = BTreeSet::new();
    for a in &world.households {
        dates.insert(a.formed);
        let mut previous = a.formed;
        let mut roster = a.adults.clone();
        for c in &a.membership {
            if c.month <= previous || c.month > state.month {
                return Err("invalid household membership date".into());
            }
            previous = c.month;
            dates.insert(c.month);
            let living = |id: &AgentId| state.terminal.get(id).is_none_or(|t| t.month >= c.month);
            match &c.action {
                Action::Join {
                    person,
                    signatories,
                    admission,
                } => {
                    let mut expected: BTreeSet<_> = roster.iter().copied().filter(living).collect();
                    if expected.is_empty()
                        || roster.contains(person)
                        || !living(person)
                        || !world.participants.iter().any(|p| p.agent == *person)
                    {
                        return Err("invalid household entrant".into());
                    }
                    expected.insert(*person);
                    if signatories.iter().copied().collect::<BTreeSet<_>>() != expected
                        || signatories.len() != expected.len()
                    {
                        return Err("household accession requires all living members and entrant to consent".into());
                    }
                    let mut terms = a.clone();
                    terms.formed = c.month;
                    terms.adults = expected.into_iter().collect();
                    terms.admission = Some(admission.clone());
                    crate::laws::households::validate_admission(world, &terms)?;
                    roster.push(*person);
                }
                Action::Leave { person } => {
                    if !roster.contains(person) || !living(person) {
                        return Err("invalid household exit".into());
                    }
                    roster.retain(|id| id != person);
                }
            }
            let count = roster.iter().filter(|id| living(id)).count();
            let rules = a.admission.as_ref().and_then(|r| r.rules.as_ref());
            let min = rules.map_or(1, |r| r.min_adults);
            let max = rules.map_or(FOUNDING_ADULT_LIMIT, |r| r.max_adults);
            if count < min || count > max {
                return Err("membership change exceeds household adult bounds; last-member exit requires dissolution".into());
            }
        }
    }
    // No adult may belong to overlapping households, including historical periods.
    for month in dates {
        let mut occupied = BTreeSet::new();
        for a in &world.households {
            for person in roster_at(a, month) {
                if !occupied.insert(person) {
                    return Err("overlapping household memberships".into());
                }
            }
        }
    }
    Ok(())
}
