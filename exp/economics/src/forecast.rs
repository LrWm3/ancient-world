//! Shared observation boundary for bounded search and hypothetical rollouts.
use crate::model::{Phase, State, World};

/// Owned, sanitized snapshot. Accepted commitments and current balances survive;
/// unpublished fixture events and speculative resale buyers do not. This is the
/// pilot's full-information observation policy, not agent-specific private access.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForecastContext {
    world: World,
    state: State,
}
impl ForecastContext {
    pub fn new(world: &World, state: &State) -> Self {
        let mut world = world.clone();
        // Current capacity is already in State after Open. Never replay overrides.
        world.capacity_overrides.clear();
        world.scheduled_starts.retain(|s| s.month == state.month);
        if let Some(credit) = &mut world.credit {
            // Cash already observed stays in State; future discretionary funding
            // is not a contractual receivable. Accepted loan due dates remain.
            credit.transfers.retain(|t| t.month <= state.month);
            // Bidding is a separate hypothetical decision, never recursive income.
            credit.resale_buyer = None;
        }
        Self {
            world,
            state: state.clone(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    /// Phase is the next live boundary, not a phase silently advanced by inspection.
    pub fn boundary(&self) -> (u32, Phase) {
        (self.state.month, self.state.phase)
    }

    /// Consume an isolated branch before applying explicit domain hypotheses.
    /// Callers own horizon, scoring, policy replacement and hypothetical actions;
    /// modifications to these tables cannot publish to the original simulation.
    pub fn into_parts(self) -> (World, State) {
        (self.world, self.state)
    }
}

/// Shared accounting for planning constraints. Horizons, candidate generation,
/// fallback rules and economic objectives remain the caller's policy.
pub mod needs {
    use crate::model::{Requirement, ResourceId};
    use std::collections::BTreeMap;

    pub type Deficits = BTreeMap<ResourceId, i64>;

    pub fn valid_limits(requirements: &[Requirement], limits: &Deficits) -> bool {
        limits.iter().all(|(resource, maximum)| {
            *maximum >= 0
                && requirements
                    .iter()
                    .any(|n| n.resource == *resource && n.quantity > 0)
        })
    }

    pub fn accumulate(total: &mut Deficits, deficits: impl IntoIterator<Item = (ResourceId, i32)>) {
        for (resource, quantity) in deficits {
            *total.entry(resource).or_default() += i64::from(quantity);
        }
    }

    pub fn within_limits(deficits: &Deficits, limits: &Deficits) -> bool {
        limits
            .iter()
            .all(|(r, maximum)| deficits.get(r).copied().unwrap_or(0) <= *maximum)
    }

    /// Lower priority numbers come first; resource ID breaks equal priorities.
    pub fn ordered(requirements: &[Requirement]) -> Vec<&Requirement> {
        let mut rows: Vec<_> = requirements.iter().collect();
        rows.sort_by_key(|n| (n.priority, n.resource));
        rows
    }

    pub fn score(requirements: &[Requirement], deficits: &Deficits) -> Vec<i64> {
        ordered(requirements)
            .iter()
            .map(|n| deficits.get(&n.resource).copied().unwrap_or(0))
            .collect()
    }

    pub fn first_violation(
        requirements: &[Requirement],
        deficits: &Deficits,
        limits: &Deficits,
    ) -> Option<(ResourceId, i64, i64)> {
        ordered(requirements).into_iter().find_map(|n| {
            let maximum = *limits.get(&n.resource)?;
            let projected = deficits.get(&n.resource).copied().unwrap_or(0);
            (projected > maximum).then_some((n.resource, projected, maximum))
        })
    }
}
