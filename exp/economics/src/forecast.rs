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
