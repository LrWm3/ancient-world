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
    score.broken_commitments += branch
        .ledger
        .iter()
        .filter_map(|b| b.town_market.as_ref())
        .filter_map(|b| match b {
            town_market::Boundary::Market(r) => r.cooperation.as_deref(),
            _ => None,
        })
        .filter(|b| {
            b.failure.is_some()
                && b.terms
                    .as_ref()
                    .is_some_and(|c| c.parties().contains(&actor))
        })
        .count() as u64;
    score
}

/// Frozen assumptions, distinct from the independently submitted live masks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Counterparties {
    Observed(super::expectations::Snapshot),
    /// Conditional on the counterparty meeting dated terms; own budgets remain real.
    Schedule {
        actor: AgentId,
        contract: crate::cooperation::Contract,
    },
    Announced {
        actor: AgentId,
        orders: town_market::OrderSelections,
    },
}
impl Counterparties {
    fn actor(&self) -> AgentId {
        match self {
            Self::Observed(s) => s.actor,
            Self::Announced { actor, .. } | Self::Schedule { actor, .. } => *actor,
        }
    }
    fn masks(&self, sim: &Simulation) -> Result<town_market::OrderSelections, String> {
        match self {
            Self::Observed(s) => s.masks(&sim.world, sim.state.month),
            Self::Announced { orders, .. } => Ok(orders.clone()),
            Self::Schedule { .. } => Ok(sim
                .world
                .participants
                .iter()
                .map(|p| (p.agent, Default::default()))
                .collect()),
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
    if let Counterparties::Schedule { contract, .. } = expectation {
        if sim.state.month == contract.start
            && crate::cooperation::active_independent(&sim.state).is_none()
        {
            let round = crate::cooperation::evaluate_schedule(
                &sim.world,
                &sim.state,
                Some(contract),
                &contract.parties().into_iter().collect(),
            )?;
            let mut batch = Batch::empty(&sim.state);
            batch.transactions = round.transactions.clone();
            batch.town_market = Some(town_market::Boundary::Market(round));
            return offers::prepare_with_acquisition(sim, requests, Some(batch));
        }
        // The accepted executor owns the book; once ended, do not assume renewal.
        return prepare_all(sim, requests, &expectation.masks(sim)?);
    }
    let mut masks = expectation.masks(sim)?;
    if let Some(selection) = selection {
        masks.insert(selection.actor, selection.submit.clone());
    }
    prepare_all(sim, requests, &masks)
}

/// Conditional projection only: budget the peer's remaining promises without
/// inspecting its productive plan. Never grant goods or credit to the actor.
pub(super) fn promise_view(
    sim: &Simulation,
    expectation: Option<&Counterparties>,
) -> Result<Option<Simulation>, String> {
    let Some(Counterparties::Schedule { actor, contract }) = expectation else {
        return Ok(None);
    };
    let mut branch = sim.clone();
    for p in &mut branch.world.participants {
        if p.agent != *actor {
            p.needs.clear();
            p.capacity.quantity = 0;
            branch.world.storage.capacities.remove(&p.agent);
            branch.state.balances.retain(|(a, _), _| *a != p.agent);
        }
    }
    branch.world.condition_rules.retain(|r| r.subject == *actor);
    branch.state.conditions.retain(|(a, _), _| *a == *actor);
    branch.state.processes.retain(|_, p| p.operator == *actor);
    for d in contract
        .deliveries
        .iter()
        .filter(|d| d.month >= branch.state.month)
    {
        for t in [&d.goods, &d.payment] {
            if t.from != *actor {
                let q = branch
                    .state
                    .balances
                    .entry((t.from, t.amount.resource))
                    .or_default();
                *q = q
                    .checked_add(t.amount.quantity)
                    .ok_or("promise forecast overflow")?;
            }
        }
    }
    Ok(Some(branch))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conditional_resources_are_only_a_peer_promise_not_an_actor_endowment() {
        let (mut w, s) = crate::production_market::reciprocal_scenario(true);
        w.production_market = None;
        // Build explicit terms without referring to either party's private work.
        let actor = crate::scenario::PERSON;
        let contract = crate::cooperation::Contract {
            independent: true,
            start: 1,
            through: 6,
            choices: Default::default(),
            deliveries: vec![crate::cooperation::Delivery {
                month: 2,
                market: 1,
                goods: crate::finance::Transfer {
                    from: 89,
                    to: actor,
                    amount: Amount::new(crate::scenario::GRAIN, 2),
                },
                payment: crate::finance::Transfer {
                    from: actor,
                    to: 89,
                    amount: Amount::new(crate::scenario::TOKEN, 2),
                },
            }],
        };
        let sim = Simulation::new(w, s, crate::compute::Backend::Reference).unwrap();
        let before = sim.state.clone();
        let view = promise_view(&sim, Some(&Counterparties::Schedule { actor, contract }))
            .unwrap()
            .unwrap();
        for (&account, &quantity) in sim.state.balances.iter().filter(|(a, _)| a.0 == actor) {
            assert_eq!(view.state.balances[&account], quantity);
        }
        assert_eq!(view.state.balance(89, crate::scenario::GRAIN), 2);
        assert_eq!(view.state.balance(89, crate::scenario::TOKEN), 0);
        assert!(
            view.world
                .participants
                .iter()
                .find(|p| p.agent == 89)
                .unwrap()
                .needs
                .is_empty()
        );
        assert_eq!(sim.state, before);
    }
}
