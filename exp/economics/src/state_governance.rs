//! Person governors select dated legal policies within a static state constitution.
//! Policies constrain actions through the existing laws interface. They neither
//! grant resources nor replace household governance, rights or contract servicing.
use crate::{
    governance::{self, AcceptedBallot, Ballot, ElectionResult, Leadership, Rules},
    membership::CITIZEN,
    model::*,
    opportunities::{Action, AgentType, PERSON_TYPE, STATE_TYPE},
};
use std::collections::{BTreeMap, BTreeSet};

pub mod formation;
pub mod scenario;

const MAX_TURNOUT_PERCENT: u32 = 100;
pub type PolicyId = u32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegalPolicy {
    pub name: String,
    /// None applies to all classified types. These restrictions are additional
    /// to the base law: choosing an empty set cannot override an existing ban.
    pub prohibited: BTreeSet<(Option<AgentType>, Action)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constitution {
    pub leadership: Leadership,
    /// The permitted policy menu is fixed at founding, not edited by governors.
    pub policies: BTreeMap<PolicyId, LegalPolicy>,
}

/// Static parameters of this constitution. Citizenship, office and ownership
/// remain distinct: the founding governor does not own the state's property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Charter {
    pub founder: AgentId,
    pub term_months: u32,
    pub election: Rules,
    pub initial_policy: PolicyId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyChange {
    pub month: u32,
    pub authorized_by: AgentId,
    pub policy: PolicyId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedPolicy {
    pub issued_month: u32,
    pub change: PolicyChange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Governance {
    pub formation: Option<formation::Agreement>,
    pub state: AgentId,
    pub formed: u32,
    pub constitution: Constitution,
    pub charter: Charter,
    pub ballots: Vec<AcceptedBallot>,
    pub changes: Vec<AcceptedPolicy>,
}

/// A read-only observation, including the provenance of the effective policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Authority {
    pub state: AgentId,
    pub leadership: Leadership,
    pub governor: Option<AgentId>,
    pub term_start: u32,
    pub election: Option<ElectionResult>,
    pub policy: PolicyId,
    pub effective_since: u32,
    pub instruction: Option<AcceptedPolicy>,
}

/// Observed political history survives economic forecasts that omit peer work.
/// Forecast-only data; live worlds read their authoritative State records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicHistory {
    pub memberships:
        BTreeMap<(AgentId, AgentId, crate::membership::Role), crate::membership::Agreement>,
    pub deaths: BTreeMap<AgentId, crate::maintenance::TerminalTransition>,
}

pub(crate) fn preserve_history(w: &mut World, s: &State) {
    if w.state_governance.is_none() && w.agency.is_empty() {
        return;
    }
    let history = w
        .governance_observation
        .get_or_insert_with(|| PublicHistory {
            memberships: Default::default(),
            deaths: Default::default(),
        });
    history.memberships.extend(s.memberships.clone());
    history.deaths.extend(s.terminal.clone());
    for controller in w.agency.values_mut() {
        controller.autonomous = false;
    }
}

fn membership_at(
    w: &World,
    s: &State,
    member: AgentId,
    state: AgentId,
) -> Option<crate::membership::Agreement> {
    let key = (member, state, CITIZEN);
    s.memberships
        .get(&key)
        .or_else(|| w.governance_observation.as_ref()?.memberships.get(&key))
        .cloned()
}

fn death_at(w: &World, s: &State, id: AgentId) -> Option<u32> {
    s.terminal
        .get(&id)
        .or_else(|| w.governance_observation.as_ref()?.deaths.get(&id))
        .map(|t| t.month)
}

fn citizen_at(w: &World, s: &State, g: &Governance, id: AgentId, month: u32) -> bool {
    w.transaction_policy.as_ref().is_some_and(|p| {
        p.agent_types.get(&id) == Some(&PERSON_TYPE)
            && membership_at(w, s, id, g.state).is_some_and(|m| {
                m.accepted_month <= month && death_at(w, s, id).is_none_or(|date| date >= month)
            })
    })
}

fn term_start(g: &Governance, month: u32) -> Option<u32> {
    if month < g.formed || g.charter.term_months == 0 {
        return None;
    }
    Some(g.formed + (month - g.formed) / g.charter.term_months * g.charter.term_months)
}

fn electorate(w: &World, s: &State, g: &Governance, start: u32) -> Vec<AgentId> {
    let mut memberships = w
        .governance_observation
        .as_ref()
        .map(|h| h.memberships.clone())
        .unwrap_or_default();
    memberships.extend(s.memberships.clone());
    memberships
        .values()
        .filter(|m| {
            m.organization == g.state
                && m.role == CITIZEN
                // Citizenship acquired during a regular opening month joins the
                // next term, never rewriting that month's election or rotation.
                && (m.accepted_month < start || start == g.formed)
                && citizen_at(w, s, g, m.member, start)
        })
        .map(|m| m.member)
        .collect()
}

fn election(w: &World, s: &State, g: &Governance, start: u32) -> ElectionResult {
    governance::tally(
        start,
        electorate(w, s, g, start),
        &g.charter.election,
        &g.ballots,
    )
}

/// Historical opening authority. A later death cannot invalidate an already
/// accepted instruction; death during a month prevents new live instructions.
pub(crate) fn governor_at_open(
    w: &World,
    s: &State,
    g: &Governance,
    month: u32,
) -> Option<AgentId> {
    let start = term_start(g, month)?;
    if death_at(w, s, g.state).is_some_and(|date| date < month) {
        return None;
    }
    let winner = match g.constitution.leadership {
        Leadership::FixedFounder => Some(g.charter.founder),
        _ if start == g.formed => Some(g.charter.founder),
        Leadership::Elected => election(w, s, g, start).winner,
        Leadership::Rotating => {
            let eligible = electorate(w, s, g, start);
            // Retain the founder as a calendar anchor, even after death.
            let mut order = eligible.clone();
            order.push(g.charter.founder);
            order.sort_unstable();
            order.dedup();
            let anchor = order.iter().position(|id| *id == g.charter.founder)?;
            let term = ((start - g.formed) / g.charter.term_months) as usize;
            (0..order.len())
                .map(|offset| order[(anchor + term % order.len() + offset) % order.len()])
                .find(|id| eligible.contains(id))
        }
    };
    // No emergency succession in this slice: vacancy lasts until the next term.
    winner.filter(|id| citizen_at(w, s, g, *id, month))
}

fn effective(g: &Governance, month: u32) -> Option<&AcceptedPolicy> {
    g.changes
        .iter()
        .filter(|p| p.issued_month <= month && p.change.month <= month)
        .max_by_key(|p| (p.change.month, p.issued_month))
}

pub fn authority(w: &World, s: &State) -> Option<Authority> {
    let g = w.state_governance.as_ref()?;
    let start = term_start(g, s.month)?;
    let instruction = effective(g, s.month).cloned();
    Some(Authority {
        state: g.state,
        leadership: g.constitution.leadership,
        governor: governor_at_open(w, s, g, s.month)
            .filter(|id| death_at(w, s, *id).is_none() && death_at(w, s, g.state).is_none()),
        term_start: start,
        election: (g.constitution.leadership == Leadership::Elected && start != g.formed)
            .then(|| election(w, s, g, start)),
        policy: instruction
            .as_ref()
            .map_or(g.charter.initial_policy, |p| p.change.policy),
        effective_since: instruction.as_ref().map_or(g.formed, |p| p.change.month),
        instruction,
    })
}

/// One dated policy applies throughout the month, including private forecasts.
/// There is no extra activation pass that could be missed or run twice.
pub(crate) fn prohibition(
    w: &World,
    s: &State,
    kind: AgentType,
    action: Action,
) -> Option<PolicyId> {
    let g = w.state_governance.as_ref()?;
    if s.month < g.formed {
        return None;
    }
    let id = effective(g, s.month).map_or(g.charter.initial_policy, |p| p.change.policy);
    let policy = g.constitution.policies.get(&id)?;
    (policy.prohibited.contains(&(None, action))
        || policy.prohibited.contains(&(Some(kind), action)))
    .then_some(id)
}

/// Atomically accept a governor instruction. The policy cannot affect already
/// planned or completed work in this month. A later governor may supersede a
/// pending effective date; two instructions issued for that date this month reject.
pub fn schedule(w: &mut World, s: &State, change: PolicyChange) -> Result<(), String> {
    if change.month <= s.month
        || authority(w, s).and_then(|a| a.governor) != Some(change.authorized_by)
    {
        return Err("state policy requires a living governor and a future month".into());
    }
    let mut candidate = w.clone();
    candidate
        .state_governance
        .as_mut()
        .ok_or("missing state governance")?
        .changes
        .push(AcceptedPolicy {
            issued_month: s.month,
            change,
        });
    validate(&candidate, s)?;
    *w = candidate;
    Ok(())
}

/// Supplied preferences, one immutable ballot per citizen per future term.
pub fn cast(w: &mut World, s: &State, ballot: Ballot) -> Result<(), String> {
    let g = w
        .state_governance
        .as_ref()
        .ok_or("missing state governance")?;
    if death_at(w, s, g.state).is_some()
        || death_at(w, s, ballot.voter).is_some()
        || ballot
            .candidate
            .is_some_and(|id| death_at(w, s, id).is_some())
    {
        return Err("state ballot requires living participants".into());
    }
    let mut candidate = w.clone();
    candidate
        .state_governance
        .as_mut()
        .unwrap()
        .ballots
        .push(AcceptedBallot {
            issued_month: s.month,
            ballot,
        });
    validate(&candidate, s)?;
    *w = candidate;
    Ok(())
}

pub fn validate(w: &World, s: &State) -> Result<(), String> {
    formation::validate(w, s)?;
    let Some(g) = &w.state_governance else {
        return Ok(());
    };
    let p = w
        .transaction_policy
        .as_ref()
        .ok_or("state governance requires transaction policy")?;
    if p.authority != g.state
        || p.agent_types.get(&g.state) != Some(&STATE_TYPE)
        || !w.agents.iter().any(|a| a.id == g.state)
        || g.formed == 0
        || g.formed > s.month
        || g.charter.term_months == 0
        || !(1..=MAX_TURNOUT_PERCENT).contains(&g.charter.election.minimum_turnout_percent)
        || !citizen_at(w, s, g, g.charter.founder, g.formed)
        || !g.constitution.policies.contains_key(&g.charter.initial_policy)
        || g.constitution.policies.values().any(|policy| {
            policy.name.trim().is_empty()
                || policy.prohibited.iter().any(|(_, action)| {
                    matches!(action, Action::Process(id) if !w.definitions.iter().any(|d| d.id == *id))
                })
        })
    {
        return Err("invalid state constitution, charter or founding authority".into());
    }
    let mut seen = BTreeSet::new();
    for accepted in &g.ballots {
        let b = &accepted.ballot;
        if g.constitution.leadership != Leadership::Elected
            || accepted.issued_month < g.formed
            || accepted.issued_month > s.month
            || b.term_start <= accepted.issued_month
            || term_start(g, b.term_start) != Some(b.term_start)
            || !seen.insert((b.term_start, b.voter))
            || death_at(w, s, g.state).is_some_and(|month| month < accepted.issued_month)
            || !citizen_at(w, s, g, b.voter, accepted.issued_month)
            || b.candidate
                .is_some_and(|id| !citizen_at(w, s, g, id, accepted.issued_month))
        {
            return Err("invalid state election ballot".into());
        }
    }
    let mut dates = BTreeSet::new();
    for accepted in &g.changes {
        let c = &accepted.change;
        if accepted.issued_month < g.formed
            || accepted.issued_month > s.month
            || c.month <= accepted.issued_month
            || !dates.insert((accepted.issued_month, c.month))
            || !g.constitution.policies.contains_key(&c.policy)
            || governor_at_open(w, s, g, accepted.issued_month) != Some(c.authorized_by)
        {
            return Err("invalid state policy instruction".into());
        }
    }
    Ok(())
}
