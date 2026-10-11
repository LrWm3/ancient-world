//! Frozen, optional hypotheses about peers' recurring discretionary process work.
//! Observations confer neither consent nor a receivable; execution stays ordinary.
use crate::model::*;
use std::collections::BTreeMap;

const MAX_MEMORY_MONTHS: u32 = 24;
const REQUIRED_OBSERVATIONS: usize = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Policy {
    #[default]
    AcceptedOnly,
    RecentSuccessfulStarts {
        memory_months: u32,
    },
}
impl Policy {
    pub fn validate(self) -> Result<(), String> {
        if matches!(self, Self::RecentSuccessfulStarts { memory_months } if !(1..=MAX_MEMORY_MONTHS).contains(&memory_months))
        {
            return Err("invalid peer process observation memory".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Evidence {
    pub agent: AgentId,
    pub definition: DefinitionId,
    pub previous: u32,
    pub latest: u32,
    pub interval: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Snapshot {
    pub as_of: u32,
    pub actor: AgentId,
    pub policy: Policy,
    pub evidence: Vec<Evidence>,
}
impl Snapshot {
    pub fn observe(w: &World, s: &State, actor: AgentId, policy: Policy) -> Result<Self, String> {
        policy.validate()?;
        let mut result = Self {
            as_of: s.month,
            actor,
            policy,
            evidence: vec![],
        };
        let Policy::RecentSuccessfulStarts { memory_months } = policy else {
            return Ok(result);
        };
        for (&peer, controller) in &w.agency {
            if peer == actor {
                continue;
            }
            let mut dates: BTreeMap<DefinitionId, Vec<u32>> = BTreeMap::new();
            for decision in &controller.history {
                if decision.effective_month >= s.month {
                    continue;
                }
                if let Some(program) = &decision.accepted {
                    for command in &program.commands {
                        if let super::Command::StartProcess(definition) = command {
                            dates
                                .entry(*definition)
                                .or_default()
                                .push(decision.effective_month);
                        }
                    }
                }
            }
            for (definition, mut months) in dates {
                // One-period processes avoid inferring completion dates/overlap
                // for long-running work from a terminal status alone.
                if !w
                    .definitions
                    .iter()
                    .any(|d| d.id == definition && d.duration() == 1)
                {
                    continue;
                }
                months.sort_unstable();
                months.dedup();
                if months.len() < REQUIRED_OBSERVATIONS {
                    continue;
                }
                let latest = months[months.len() - 1];
                let previous = months[months.len() - 2];
                if s.month - previous > memory_months
                    || ![previous, latest].iter().all(|month| {
                        s.processes.values().any(|p| {
                            p.operator == peer
                                && p.definition == definition
                                && p.start == *month
                                && p.status == Status::Completed
                        })
                    })
                {
                    continue;
                }
                result.evidence.push(Evidence {
                    agent: peer,
                    definition,
                    previous,
                    latest,
                    interval: latest - previous,
                });
            }
        }
        Ok(result)
    }
}

impl Snapshot {
    /// Apply to a sanitized, isolated forecast only. Does not add decisions,
    /// claims or balances; returns the hypothetical starts for inspection.
    pub fn apply_hypotheses(
        &self,
        w: &mut World,
        s: &State,
    ) -> Result<Vec<ScheduledStart>, String> {
        self.policy.validate()?;
        if w.agency.values().any(|c| c.autonomous) {
            return Err("peer hypotheses require frozen forecast controllers".into());
        }
        let Policy::RecentSuccessfulStarts { memory_months } = self.policy else {
            return Ok(vec![]);
        };
        if s.phase != Phase::Open || s.month <= self.as_of {
            return Ok(vec![]);
        }
        let mut starts = vec![];
        for e in &self.evidence {
            if e.agent == self.actor
                || e.latest >= self.as_of
                || e.previous >= e.latest
                || e.interval != e.latest - e.previous
                || s.month - e.latest > memory_months
                || !(s.month - e.latest).is_multiple_of(e.interval)
            {
                continue;
            }
            let Some(c) = w.agency.get(&e.agent) else {
                continue;
            };
            let Some(d) = w
                .definitions
                .iter()
                .find(|d| d.id == e.definition && d.enabled && d.duration() == 1)
            else {
                continue;
            };
            if super::governor(w, s, e.agent).is_none()
                || !crate::opportunities::permits(
                    w,
                    s,
                    e.agent,
                    crate::opportunities::Action::Process(e.definition),
                )
            {
                continue;
            }
            let needed = c.config.objectives.iter().any(|o| o.scope == super::objectives::Scope::Organization && matches!(o.metric, super::objectives::Metric::Reserve { resource, target } if d.outputs.iter().any(|a| a.resource == resource && a.quantity > 0) && s.balance(e.agent, resource) < target));
            if !needed
                || w.scheduled_starts.iter().any(|a| {
                    a.month == s.month && a.agent == e.agent && a.definition == e.definition
                })
            {
                continue;
            }
            super::schedule_start(w, e.agent, e.definition, s.month)?;
            starts.push(ScheduledStart {
                month: s.month,
                agent: e.agent,
                definition: e.definition,
            });
        }
        Ok(starts)
    }
}

/// A hypothetical attempt and its actual result inside that isolated rollout.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Attempt {
    pub agent: AgentId,
    pub definition: DefinitionId,
    pub month: u32,
    pub completed: bool,
}
