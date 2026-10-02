//! Delivery-only agreements assessed by independent composition searches.
use super::{
    Policy,
    persons::{self, Persons},
    posted::Assessment,
};
use crate::{
    composition::{self, Scope, SearchOptions, market::Counterparties},
    cooperation::{self, Contract, Delivery, INDEPENDENT_PARTIES, TERM_MONTHS},
    finance::Transfer,
    marketplace::Side,
    model::*,
    simulation::Simulation,
    town_market,
};
use std::collections::BTreeMap;

const LISTINGS: usize = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub month: u32,
    pub terms: Option<Contract>,
    pub assessments: Vec<Assessment>,
    pub accepted: bool,
    pub continuing: bool,
    pub failure: Option<String>,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controller {
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
            .ok_or("schedule planner needs a town book")?;
        if sim.world.participants.len() != INDEPENDENT_PARTIES
            || town_market::listings(config).len() != LISTINGS
            || config.adaptive
            || sim.world.production_market.is_some()
            || !sim.world.households.is_empty()
            || sim.world.priority != Priority::ContinuingFirst
            || self.persons.controllers.keys().copied().collect::<Vec<_>>() != {
                let mut ids: Vec<_> = sim.world.participants.iter().map(|p| p.agent).collect();
                ids.sort();
                ids
            }
            || self
                .persons
                .controllers
                .values()
                .any(|c| c.policy != Policy::Monthly || c.budget.months != TERM_MONTHS)
        {
            return Err("schedule pilot needs two independent monthly persons and two fixed-side listings with six-month forecasts".into());
        }
        if sim.state.phase != Phase::Acquire {
            return self.persons.step(sim);
        }
        let current = cooperation::active_independent(&sim.state).cloned();
        let mut receipt = Round {
            month: sim.state.month,
            terms: None,
            assessments: vec![],
            accepted: false,
            continuing: current.is_some(),
            failure: None,
            reason: String::new(),
        };
        let candidate = match current.clone() {
            Some(c) => Some(c),
            None => match proposal(sim) {
                Ok(c) => Some(c),
                Err(e) => {
                    receipt.reason = format!("no compatible dated offer: {e}");
                    None
                }
            },
        };
        if let Some(contract) = candidate {
            receipt.terms = Some(contract.clone());
            let round = cooperation::evaluate_schedule(
                &sim.world,
                &sim.state,
                current.is_none().then_some(&contract),
                &if current.is_none() {
                    contract.parties().into_iter().collect()
                } else {
                    Default::default()
                },
            );
            if let Ok(round) = round {
                receipt.failure = round.cooperation.as_ref().and_then(|b| b.failure.clone());
                let mut requests = vec![];
                let empty = Default::default();
                for (&actor, controller) in &self.persons.controllers {
                    let branch = persons::local(sim, actor);
                    let promise = Counterparties::Schedule {
                        actor,
                        contract: contract.clone(),
                    };
                    let options = |counterparties, required_orders| SearchOptions {
                        land_limit: controller.land_limit,
                        persistent_orders: true,
                        counterparties,
                        required_orders,
                    };
                    let choose = |options| {
                        composition::choose_limited(
                            &branch,
                            &Scope::Person(actor),
                            controller.strategy,
                            controller.budget,
                            composition::calibration::Scoring::PrivateBuffers,
                            options,
                            &mut vec![],
                        )
                    };
                    // Outside is the existing ordinary forecast, not another person's private plan.
                    let baseline = choose(options(None, None))?;
                    let selection = choose(options(
                        receipt.failure.is_none().then_some(&promise),
                        Some(&empty),
                    ))?;
                    let acceptable = current.is_some()
                        || (selection.score.broken_commitments == 0
                            && if Some(&actor) == self.persons.controllers.keys().next() {
                                selection.score < baseline.score
                            } else {
                                selection.score <= baseline.score
                            });
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
                if receipt.assessments.len() == INDEPENDENT_PARTIES
                    && receipt.assessments.iter().all(|a| a.acceptable)
                {
                    let mut acquisition = Batch::empty(&sim.state);
                    acquisition.transactions = round.transactions.clone();
                    acquisition.town_market = Some(town_market::Boundary::Market(round));
                    let batch = match persons::prepare(sim, &requests, Some(&acquisition)) {
                        Ok(b) => Ok(b),
                        Err(e) if current.is_some() => {
                            receipt.reason = format!(
                                "work rejected after actual delivery: {e}; continuing work only"
                            );
                            persons::prepare(sim, &[], Some(&acquisition))
                        }
                        Err(e) => Err(e),
                    };
                    match batch {
                        Ok(batch) => {
                            crate::settlement::commit(
                                &sim.world,
                                &mut sim.state,
                                &batch,
                                sim.backend,
                                sim.effect_limit,
                            )?;
                            sim.ledger.push(batch);
                            receipt.accepted = current.is_none();
                            if receipt.reason.is_empty() {
                                receipt.reason =
                                    "delivery schedule executed with independently selected work"
                                        .into();
                            }
                            self.history.push(receipt);
                            return Ok(());
                        }
                        Err(e) if current.is_some() => return Err(e),
                        Err(e) => receipt.reason = format!("proposed work rejected: {e}"),
                    }
                } else {
                    receipt.reason = "independent assessment declined".into();
                }
            } else {
                let reason = round.unwrap_err();
                if current.is_some() {
                    return Err(reason);
                }
                receipt.reason = format!("schedule unavailable: {reason}");
            }
        }
        let mut fallback = sim.clone();
        let mut persons = self.persons.clone();
        persons.step(&mut fallback)?;
        *sim = fallback;
        self.persons = persons;
        self.history.push(receipt);
        Ok(())
    }
}

/// One public menu: reciprocal existing lots, next month through month six.
/// Equal posted limit/quote required; price negotiation and adaptive roles are separate.
pub fn proposal(sim: &Simulation) -> Result<Contract, String> {
    let config = sim.world.town_market.as_ref().ok_or("missing town book")?;
    let through = sim
        .state
        .month
        .checked_add(TERM_MONTHS - 1)
        .ok_or("schedule overflow")?;
    let venue = crate::marketplace::venue(&sim.world, config.venue).ok_or("missing venue")?;
    let mut deliveries = vec![];
    for listing in town_market::listings(config) {
        if listing.adaptive
            || listing.match_limit == Some(0)
            || listing.traders.len() != INDEPENDENT_PARTIES
        {
            return Err("schedule listing unavailable".into());
        }
        let seller = listing
            .traders
            .iter()
            .find(|t| t.side == Side::Sell)
            .ok_or("missing seller")?;
        let buyer = listing
            .traders
            .iter()
            .find(|t| t.side == Side::Buy)
            .ok_or("missing buyer")?;
        let price = seller.trader.opening_quote;
        if price != seller.trader.limit
            || price != buyer.trader.limit
            || price != buyer.trader.opening_quote
        {
            return Err("pilot requires an agreed fixed lot price".into());
        }
        let market = venue
            .markets
            .iter()
            .find(|m| m.id == listing.market)
            .ok_or("missing market")?;
        for month in sim.state.month + 1..=through {
            deliveries.push(Delivery {
                month,
                market: listing.market,
                goods: Transfer {
                    from: seller.trader.agent,
                    to: buyer.trader.agent,
                    amount: market.goods.clone(),
                },
                payment: Transfer {
                    from: buyer.trader.agent,
                    to: seller.trader.agent,
                    amount: Amount::new(market.payment, price),
                },
            });
        }
    }
    deliveries.sort_by_key(|d| (d.month, d.market));
    Ok(Contract {
        independent: true,
        start: sim.state.month,
        through,
        choices: BTreeMap::new(),
        deliveries,
    })
}
