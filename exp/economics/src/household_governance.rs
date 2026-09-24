//! Founding templates, static charter parameters and bounded household policy.
use crate::{households::Agreement, model::*};
use std::collections::BTreeSet;

pub const PERCENT: i32 = 100;
pub const DEFAULT_LABOR_PERCENT: u32 = 20;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constitution {
    pub permitted_policies: BTreeSet<Policy>,
    /// None permits member-executable productive activities. A subset can narrow
    /// the mandate, but never grants a worker another member's personal rights.
    pub activities: Option<BTreeSet<DefinitionId>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Charter {
    pub leader: AgentId,
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
pub struct Governance {
    pub constitution: Constitution,
    pub charter: Charter,
    /// Accepted dated instructions; leader chooses policy, not individual jobs.
    pub changes: Vec<PolicyChange>,
}
impl Governance {
    pub fn contributed(leader: AgentId) -> Self {
        Self {
            constitution: Constitution {
                permitted_policies: [Policy::NetOutput, Policy::PreserveCommittedWork]
                    .into_iter()
                    .collect(),
                activities: None,
            },
            charter: Charter {
                leader,
                contribution: Contribution::Percent(DEFAULT_LABOR_PERCENT),
                tie_break: TieBreak::Rotating,
                initial_policy: Policy::PreserveCommittedWork,
            },
            changes: vec![],
        }
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
            .filter(|c| c.month <= month)
            .max_by_key(|c| c.month)
            .map_or(self.charter.initial_policy, |c| c.policy)
    }
}

pub fn validate(world: &World, a: &Agreement) -> Result<(), String> {
    let g = &a.governance;
    let mut dates = BTreeSet::new();
    if !a.adults.contains(&g.charter.leader)
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
            c.month < a.formed
                || !dates.insert(c.month)
                || c.authorized_by != g.charter.leader
                || !g.constitution.permitted_policies.contains(&c.policy)
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
    if change.month <= state.month || state.terminal.contains_key(&change.authorized_by) {
        return Err("policy instruction requires a living governor and a future month".into());
    }
    a.governance.changes.push(change);
    crate::households::validate(&candidate, state)?;
    *world = candidate;
    Ok(())
}
