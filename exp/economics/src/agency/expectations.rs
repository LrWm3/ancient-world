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
