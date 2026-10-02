//! Bounded reciprocal spot offers after independently announced order intentions.
//! No private counterparty plan enters an assessment, and no future promise is made.
use super::{
    Policy,
    persons::{self, Persons},
};
use crate::{
    composition::{self, Metrics, Scope, SearchOptions, market::Counterparties},
    cooperation::Delivery,
    model::*,
    offers::Request,
    planning::Score,
    simulation::Simulation,
    town_market::{self, OrderSelections},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub actor: AgentId,
    pub outside: Score,
    pub offered: Score,
    pub acceptable: bool,
    pub outside_search: Metrics,
    pub offered_search: Metrics,
    pub requests: Vec<Request>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub month: u32,
    /// Public preliminary intentions only. Their trial settlement is not published.
    pub announced: OrderSelections,
    pub announcement_search: BTreeMap<AgentId, Metrics>,
    pub proposer: AgentId,
    pub terms: Vec<Delivery>,
    pub assessments: Vec<Assessment>,
    pub accepted: bool,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controller {
    /// Ordinary independent planning supplies announcements and the fallback.
    /// Its history covers fallback execution only; this driver's history covers offers.
    pub persons: Persons,
    pub history: Vec<Round>,
}
impl Controller {
    pub fn new(persons: Persons) -> Self {
        Self {
            persons,
            history: vec![],
        }
    }
    pub fn step(&mut self, sim: &mut Simulation) -> Result<(), String> {
        let config = sim
            .world
            .town_market
            .as_ref()
            .ok_or("posted spot offers require a town book")?;
        if sim.world.participants.len() != town_market::CONDITIONAL_SPOT_PARTIES
            || town_market::listings(config).len() != town_market::CONDITIONAL_SPOT_DELIVERIES
            || config.adaptive
            || self
                .persons
                .controllers
                .values()
                .any(|c| c.policy != Policy::Monthly)
        {
            return Err(
                "posted spot pilot requires two people, two fixed-side listings and monthly review"
                    .into(),
            );
        }
        if sim.state.phase != Phase::Acquire {
            return self.persons.step(sim);
        }
        // Independent announcement phase. Keep the actual state untouched until
        // acceptance; a declined offer publishes this validated ordinary fallback.
        let mut fallback = sim.clone();
        let mut planners = self.persons.clone();
        planners.step(&mut fallback)?;
        let book = planners
            .history
            .last()
            .ok_or("missing announcement round")?;
        let announced: OrderSelections = book
            .exchanges
            .iter()
            .map(|(&a, x)| (a, x.submit.clone()))
            .collect();
        let mut receipt = Round {
            month: sim.state.month,
            announced: announced.clone(),
            announcement_search: planners
                .controllers
                .iter()
                .filter_map(|(&a, c)| c.history.last()?.search.clone().map(|m| (a, m)))
                .collect(),
            proposer: *announced.keys().next().ok_or("missing proposer")?,
            terms: vec![],
            assessments: vec![],
            accepted: false,
            reason: "no fully executable reciprocal offer".into(),
        };
        let mut proposed: OrderSelections = sim
            .world
            .participants
            .iter()
            .map(|p| (p.agent, Default::default()))
            .collect();
        for l in town_market::listings(config) {
            for t in &l.traders {
                proposed
                    .get_mut(&t.trader.agent)
                    .ok_or("nonparticipant trader")?
                    .insert((l.market, t.side));
            }
        }
        let quoted = town_market::evaluate_selections(&sim.world, &sim.state, &proposed)?;
        let terms = town_market::spot_deliveries(&quoted);
        let quoted = town_market::evaluate_conditional(&sim.world, &sim.state, &proposed, &terms);
        if let Ok(quoted) = quoted {
            receipt.terms = terms;
            let mut requests = vec![];
            // Stable proposer then recipient; each sees public masks/terms and
            // evaluates only its own work. No joint score or joint forecast.
            for (&actor, controller) in &self.persons.controllers {
                let branch = persons::local(sim, actor);
                let outside = Counterparties::Announced {
                    actor,
                    orders: announced.clone(),
                };
                let offered = Counterparties::Announced {
                    actor,
                    orders: proposed.clone(),
                };
                let assess = |hypothesis, required_orders| {
                    composition::choose_limited(
                        &branch,
                        &Scope::Person(actor),
                        controller.strategy,
                        controller.budget,
                        composition::calibration::Scoring::PrivateBuffers,
                        SearchOptions {
                            land_limit: controller.land_limit,
                            persistent_orders: true,
                            counterparties: Some(hypothesis),
                            required_orders,
                        },
                        &mut vec![],
                    )
                };
                let baseline = assess(&outside, None)?;
                let selection = assess(&offered, Some(&proposed[&actor]))?;
                let acceptable = if actor == receipt.proposer {
                    selection.score < baseline.score
                } else {
                    selection.score <= baseline.score
                };
                requests.extend(selection.requests.clone());
                receipt.assessments.push(Assessment {
                    actor,
                    outside: baseline.score,
                    offered: selection.score,
                    acceptable,
                    outside_search: baseline.metrics,
                    offered_search: selection.metrics,
                    requests: selection.requests,
                });
                if !acceptable {
                    break;
                }
            }
            receipt.reason = "declined by an independent assessment".into();
            if receipt.assessments.len() == town_market::CONDITIONAL_SPOT_PARTIES
                && receipt.assessments.iter().all(|a| a.acceptable)
            {
                let mut acquisition = Batch::empty(&sim.state);
                acquisition.transactions = quoted.transactions.clone();
                acquisition.town_market = Some(town_market::Boundary::Market(quoted));
                match persons::prepare(sim, &requests, Some(&acquisition)) {
                    Ok(batch) => {
                        crate::settlement::commit(
                            &sim.world,
                            &mut sim.state,
                            &batch,
                            sim.backend,
                            sim.effect_limit,
                        )?;
                        sim.ledger.push(batch);
                        receipt.accepted = true;
                        receipt.reason = "reciprocal spot package accepted and settled".into();
                    }
                    Err(reason) => receipt.reason = format!("work package rejected: {reason}"),
                }
            }
        }
        if !receipt.accepted {
            *sim = fallback;
            self.persons = planners;
        }
        self.history.push(receipt);
        Ok(())
    }
}
