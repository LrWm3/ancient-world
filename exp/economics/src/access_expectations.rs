//! Quantity-weighted, bounded memory of realized access to shared resource accounts.
use crate::model::{Account, AgentId};
use std::collections::BTreeMap;

pub const MEMORY_MONTHS: u32 = 6;
const PRIOR_UNITS: u64 = 2;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Optimistic,
    Learned,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Estimate {
    pub received: u64,
    pub requested: u64,
}
impl Estimate {
    /// Carry fractional units across forecast months rather than truncating a
    /// one-unit flow permanently to zero. This is an expectation, not a grant.
    pub fn portion(self, quantity: i32, carry: &mut u64) -> Result<i32, String> {
        if quantity < 0
            || self.requested == 0
            || self.received > self.requested
            || *carry >= self.requested
        {
            return Err("invalid expected access fraction".into());
        }
        let scaled = quantity as u128 * u128::from(self.received) + u128::from(*carry);
        *carry = (scaled % u128::from(self.requested)) as u64;
        Ok((scaled / u128::from(self.requested)) as i32)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub month: u32,
    pub agent: AgentId,
    pub account: Account,
    pub requested: u32,
    pub received: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Memory {
    pub observations: Vec<Observation>,
    pub through: Option<u32>,
}
impl Memory {
    /// Called once after successful Productive settlement, including empty rounds.
    pub fn record(&mut self, month: u32, rows: Vec<Observation>) -> Result<(), String> {
        if self.through.is_some_and(|m| month <= m) {
            return Err("duplicate or stale access observation".into());
        }
        let mut keys = std::collections::BTreeSet::new();
        if rows.iter().any(|r| {
            r.month != month
                || r.requested == 0
                || r.received > r.requested
                || !keys.insert((r.agent, r.account))
        }) {
            return Err("invalid access observation".into());
        }
        self.observations
            .retain(|r| month.saturating_sub(r.month) < MEMORY_MONTHS);
        self.observations.extend(rows);
        self.observations
            .sort_by_key(|r| (r.month, r.agent, r.account));
        self.through = Some(month);
        Ok(())
    }
    pub fn estimate(&self, month: u32, agent: AgentId, account: Account) -> Estimate {
        let mut e = Estimate {
            received: PRIOR_UNITS,
            requested: PRIOR_UNITS,
        };
        for r in &self.observations {
            if r.agent == agent
                && r.account == account
                && r.month < month
                && month - r.month <= MEMORY_MONTHS
            {
                e.received += u64::from(r.received);
                e.requested += u64::from(r.requested);
            }
        }
        e
    }
    pub fn snapshot(
        &self,
        month: u32,
        agents: impl Iterator<Item = AgentId>,
        accounts: &[Account],
        mode: Mode,
    ) -> BTreeMap<(AgentId, Account), Estimate> {
        agents
            .flat_map(|agent| accounts.iter().map(move |account| (agent, *account)))
            .map(|key| {
                let e = match mode {
                    Mode::Optimistic => Estimate {
                        received: 1,
                        requested: 1,
                    },
                    Mode::Learned => self.estimate(month, key.0, key.1),
                };
                (key, e)
            })
            .collect()
    }
}
