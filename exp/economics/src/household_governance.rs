//! Founding templates, static charter parameters and bounded household policy.
use crate::{households::Agreement, model::*};
use std::collections::BTreeSet;
pub mod elections;

pub const PERCENT: i32 = 100;
pub const DEFAULT_LABOR_PERCENT: u32 = 20;
pub const DEFAULT_TERM_MONTHS: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Policy {
    NetOutput,
    PreserveCommittedWork,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TieBreak {
    MemberId,
    Rotating,
    SignatoryOrder,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contribution {
    Percent(u32),
    SpareLabor,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leadership {
    FixedFounder,
    Rotating,
    Elected,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constitution {
    pub leadership: Leadership,
    pub permitted_policies: BTreeSet<Policy>,
    /// None permits member-executable productive activities. A subset can narrow
    /// the mandate, but never grants a worker another member's personal rights.
    pub activities: Option<BTreeSet<DefinitionId>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Charter {
    /// Founding governor; rotating terms start here in the stable adult-ID ring.
    pub leader: AgentId,
    pub term_months: u32,
    pub election: elections::Rules,
    pub contribution: Contribution,
    pub tie_break: TieBreak,
    pub initial_policy: Policy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyChange {
    pub month: u32,
    pub authorized_by: AgentId,
    pub policy: Policy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedPolicy {
    pub issued_month: u32,
    pub change: PolicyChange,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Authority {
    pub household: AgentId,
    pub leader: Option<AgentId>,
    pub leadership: Leadership,
    pub term_start: u32,
    pub election: Option<elections::ElectionResult>,
    pub policy: Policy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Governance {
    pub constitution: Constitution,
    pub charter: Charter,
    /// Accepted dated instructions; leader chooses policy, not individual jobs.
    pub changes: Vec<AcceptedPolicy>,
    pub ballots: Vec<elections::AcceptedBallot>,
}
impl Governance {
    pub fn contributed(leader: AgentId) -> Self {
        Self {
            constitution: Constitution {
                leadership: Leadership::FixedFounder,
                permitted_policies: [Policy::NetOutput, Policy::PreserveCommittedWork]
                    .into_iter()
                    .collect(),
                activities: None,
            },
            charter: Charter {
                leader,
                term_months: DEFAULT_TERM_MONTHS,
                election: elections::Rules::default(),
                contribution: Contribution::Percent(DEFAULT_LABOR_PERCENT),
                tie_break: TieBreak::Rotating,
                initial_policy: Policy::PreserveCommittedWork,
            },
            changes: vec![],
            ballots: vec![],
        }
    }
    pub fn rotating(leader: AgentId, term_months: u32) -> Self {
        let mut g = Self::contributed(leader);
        g.constitution.leadership = Leadership::Rotating;
        g.charter.term_months = term_months;
        g
    }
    pub fn elected(leader: AgentId, term_months: u32) -> Self {
        let mut g = Self::rotating(leader, term_months);
        g.constitution.leadership = Leadership::Elected;
        g
    }
    pub fn legacy(leader: AgentId) -> Self {
        let mut g = Self::contributed(leader);
        g.charter.contribution = Contribution::SpareLabor;
        g.charter.tie_break = TieBreak::SignatoryOrder;
        g.charter.initial_policy = Policy::NetOutput;
        g
    }
    pub fn policy(&self, month: u32) -> Policy {
        self.changes
            .iter()
            .filter(|c| c.issued_month <= month && c.change.month <= month)
            .max_by_key(|c| (c.change.month, c.issued_month))
            .map_or(self.charter.initial_policy, |c| c.change.policy)
    }
}

pub fn validate(world: &World, state: &State, a: &Agreement) -> Result<(), String> {
    let g = &a.governance;
    elections::validate(a, state)?;
    let mut dates = BTreeSet::new();
    if g.charter.term_months == 0
        || !a.adults.contains(&g.charter.leader)
        || !g
            .constitution
            .permitted_policies
            .contains(&g.charter.initial_policy)
        || matches!(g.charter.contribution, Contribution::Percent(p) if p > PERCENT as u32)
        || g.constitution.activities.as_ref().is_some_and(|ids| {
            ids.iter().any(|id| {
                !world
                    .definitions
                    .iter()
                    .any(|d| d.id == *id && d.execution == Execution::Productive)
            })
        })
        || g.changes.iter().any(|c| {
            c.issued_month < a.formed
                || c.issued_month > state.month
                || c.change.month <= c.issued_month
                || !dates.insert((c.issued_month, c.change.month))
                || leader_at_open(a, state, c.issued_month) != Some(c.change.authorized_by)
                || !g.constitution.permitted_policies.contains(&c.change.policy)
        })
    {
        return Err("invalid household constitution, charter or policy authority".into());
    }
    // Legacy is a compatibility policy, not an alternative executor for a limited mandate.
    if g.charter.contribution == Contribution::SpareLabor
        && (g.constitution.activities.is_some()
            || g.charter.initial_policy != Policy::NetOutput
            || !g.changes.is_empty())
    {
        return Err("legacy spare labor requires its original unrestricted output policy".into());
    }
    Ok(())
}

/// Governance changes only at month opening. A terminal transition at Close m
/// affects selection from Open m+1, preserving earlier authority evidence.
fn leader_at_open(a: &Agreement, state: &State, month: u32) -> Option<AgentId> {
    let g = &a.governance;
    if month < a.formed || g.charter.term_months == 0 {
        return None;
    }
    let alive = |id: &AgentId| state.terminal.get(id).is_none_or(|t| t.month >= month);
    if g.constitution.leadership == Leadership::FixedFounder {
        return alive(&g.charter.leader).then_some(g.charter.leader);
    }
    if g.constitution.leadership == Leadership::Elected {
        let term_start =
            a.formed + ((month - a.formed) / g.charter.term_months) * g.charter.term_months;
        let winner = if term_start == a.formed {
            Some(g.charter.leader)
        } else {
            elections::resolve(a, state, term_start).winner
        };
        return winner.filter(alive);
    }
    let mut ring = a.adults.clone();
    ring.sort_unstable();
    let initial = ring.iter().position(|id| *id == g.charter.leader)?;
    let term = (month - a.formed) / g.charter.term_months;
    let start = (initial + term as usize % ring.len()) % ring.len();
    (0..ring.len())
        .map(|offset| ring[(start + offset) % ring.len()])
        .find(alive)
}
/// Living current authority; a death does not confer same-month replacement powers.
pub fn leader(a: &Agreement, state: &State) -> Option<AgentId> {
    leader_at_open(a, state, state.month).filter(|id| !state.terminal.contains_key(id))
}
pub fn authority(a: &Agreement, state: &State) -> Authority {
    let g = &a.governance;
    let term_start = if g.constitution.leadership != Leadership::FixedFounder
        && g.charter.term_months > 0
    {
        a.formed
            + (state.month.saturating_sub(a.formed) / g.charter.term_months) * g.charter.term_months
    } else {
        a.formed
    };
    Authority {
        household: a.agent,
        leader: leader(a, state),
        leadership: g.constitution.leadership,
        term_start,
        election: (g.constitution.leadership == Leadership::Elected && term_start > a.formed)
            .then(|| elections::resolve(a, state, term_start)),
        policy: g.policy(state.month),
    }
}

pub fn ordered(a: &Agreement, state: &State) -> Vec<AgentId> {
    let mut people: Vec<_> = crate::households::members(a, state).collect();
    match a.governance.charter.tie_break {
        TieBreak::SignatoryOrder => {}
        TieBreak::MemberId => people.sort_unstable(),
        TieBreak::Rotating => {
            people.sort_unstable();
            if !people.is_empty() {
                let offset = (state.month - a.formed) as usize % people.len();
                people.rotate_left(offset);
            }
        }
    }
    people
}

/// Accept a policy instruction now for a future month. Founding documents stay fixed.
pub fn schedule(
    world: &mut World,
    state: &State,
    household: AgentId,
    change: PolicyChange,
) -> Result<(), String> {
    let mut candidate = world.clone();
    let a = candidate
        .households
        .iter_mut()
        .find(|a| a.agent == household)
        .ok_or("unknown household")?;
    if change.month <= state.month || leader(a, state) != Some(change.authorized_by) {
        return Err("policy instruction requires a living governor and a future month".into());
    }
    a.governance.changes.push(AcceptedPolicy {
        issued_month: state.month,
        change,
    });
    crate::households::validate(&candidate, state)?;
    *world = candidate;
    Ok(())
}
