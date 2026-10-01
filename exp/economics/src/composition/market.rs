//! Current need-order adapters. Submission is intent, never a guaranteed fill.
use crate::{
    model::*,
    offers,
    simulation::Simulation,
    town_market::{self, OrderSelection},
};

pub fn prepare(
    sim: &Simulation,
    requests: &[offers::Request],
    selection: Option<&OrderSelection>,
) -> Result<Batch, String> {
    match selection {
        None => offers::prepare(sim, requests),
        Some(selected) => {
            if requests.iter().any(|r| r.agent != selected.actor) {
                return Err("market search cannot direct another actor's work".into());
            }
            let round = town_market::evaluate_selected(&sim.world, &sim.state, selected)?;
            let mut batch = Batch::empty(&sim.state);
            batch.transactions = round.transactions.clone();
            batch.town_market = Some(town_market::Boundary::Market(round));
            offers::prepare_with_acquisition(sim, requests, Some(batch))
        }
    }
}

/// Counterparties retain their own ordinary execution policy. Only this actor's
/// outcomes score the search; their resources are never added to its budget.
pub(super) fn actor_score(branch: &Simulation, actor: AgentId) -> crate::planning::Score {
    let mut view = branch.clone();
    view.world.participants.retain(|p| p.agent == actor);
    view.reports.retain(|r| r.agent == actor);
    for b in &mut view.ledger {
        b.receipts.retain(|r| r.agent == actor);
    }
    let mut score = crate::planning::score(&view);
    score.broken_commitments = branch
        .state
        .processes
        .values()
        .filter(|p| p.operator == actor && p.status == Status::Aborted)
        .count() as u64;
    score
}
