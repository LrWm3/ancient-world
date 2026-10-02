//! Current need-order adapters. Submission is intent, never a guaranteed fill.
use crate::{
    model::*,
    offers,
    simulation::Simulation,
    town_market::{self, OrderSelection},
};

/// What the candidate assumes about its own later order submission. Neither
/// option authorizes a future trade or chooses a counterparty's live actions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrderForecast {
    #[default]
    CurrentBoundaryOnly,
    StandingPolicy,
}

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

/// Joint spot-book acceptance with explicitly supplied masks for every participant.
/// Work is checked against actual fills. Orders and work remain separate intents:
/// callers may admit a smaller work set without undoing an already accepted book.
pub fn prepare_all(
    sim: &Simulation,
    requests: &[offers::Request],
    selections: &town_market::OrderSelections,
) -> Result<Batch, String> {
    let round = town_market::evaluate_selections(&sim.world, &sim.state, selections)?;
    let mut batch = Batch::empty(&sim.state);
    batch.transactions = round.transactions.clone();
    batch.town_market = Some(town_market::Boundary::Market(round));
    offers::prepare_with_acquisition(sim, requests, Some(batch))
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

/// Frozen assumptions, distinct from the independently submitted live masks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Counterparties {
    Observed(super::expectations::Snapshot),
    Announced {
        actor: AgentId,
        orders: town_market::OrderSelections,
    },
}
impl Counterparties {
    fn actor(&self) -> AgentId {
        match self {
            Self::Observed(s) => s.actor,
            Self::Announced { actor, .. } => *actor,
        }
    }
    fn masks(&self, sim: &Simulation) -> Result<town_market::OrderSelections, String> {
        match self {
            Self::Observed(s) => s.masks(&sim.world, sim.state.month),
            Self::Announced { orders, .. } => Ok(orders.clone()),
        }
    }
}

/// Only used within isolated forecast branches. Actual clearing receives the
/// independently selected masks, never this counterparty hypothesis.
pub(super) fn prepare_expected(
    sim: &Simulation,
    requests: &[offers::Request],
    selection: Option<&OrderSelection>,
    expectation: Option<&Counterparties>,
) -> Result<Batch, String> {
    let Some(expectation) = expectation else {
        return prepare(sim, requests, selection);
    };
    if requests.iter().any(|r| r.agent != expectation.actor())
        || selection.is_some_and(|s| s.actor != expectation.actor())
    {
        return Err("forecast exceeds expectation actor mandate".into());
    }
    let mut masks = expectation.masks(sim)?;
    if let Some(selection) = selection {
        masks.insert(selection.actor, selection.submit.clone());
    }
    prepare_all(sim, requests, &masks)
}
