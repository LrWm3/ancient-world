//! Bounded receding-horizon policy portfolio. Forecasts use ordinary settlement,
//! not independently summed promises of output. No agent-kind/resource-ID branches.
use crate::{compute::Backend, model::*, settlement::commit, simulation::Simulation};
use std::collections::BTreeSet;

const MAX_FORECAST_PARTICIPANTS: usize = 4;
const MAX_FORECAST_REQUIREMENTS: usize = 4;
const MAX_POSTED_OFFERS: usize = 4;
const MAX_SUBSTITUTE_PRODUCERS: usize = 4;
const SCORE_SCALE: u64 = 1000;

/// Lexicographic: survival first, then impairment, normalized deprivation,
/// terminal inventory coverage, and finally productive work cost. Lower is better.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    pub terminal_months: u64,
    pub impaired_months: u64,
    pub deprivation: u64,
    pub buffer_gap: u64,
    pub work: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Forecast {
    pub offer: Option<u32>,
    pub access_offer: Option<u32>,
    pub stock_bid: Option<u32>,
    /// Buyer for equipment, or sellers accepting the stock bid.
    pub trade_agents: Vec<AgentId>,
    pub priority: Priority,
    pub defer_new: bool,
    pub preferred_process: Option<DefinitionId>,
    pub score: Score,
    pub outcomes: Vec<MonthReport>,
    pub first_work: Vec<Receipt>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub month: u32,
    pub through: u32,
    pub alternatives: Vec<Forecast>,
    pub selected: usize,
}

fn substitute_producers(world: &World) -> BTreeSet<DefinitionId> {
    world
        .participants
        .iter()
        .flat_map(|p| &p.needs)
        .filter(|n| n.quantity > 0)
        .filter(|n| crate::substitution::recipes(world, n.resource).len() > 1)
        .flat_map(|n| crate::substitution::recipes(world, n.resource))
        .flat_map(|c| {
            world
                .definitions
                .iter()
                .filter(move |d| {
                    d.enabled
                        && d.execution == Execution::Productive
                        && d.outputs
                            .iter()
                            .any(|a| a.resource == c.stages[0].entry_inputs[0].resource)
                })
                .map(|d| d.id)
        })
        .collect()
}

pub fn validate(world: &World) -> Result<(), String> {
    if substitute_producers(world).len() > MAX_SUBSTITUTE_PRODUCERS {
        return Err("at most four substitute producers are supported".into());
    }
    if world.decision_horizon == Some(0) {
        return Err("zero decision horizon".into());
    }
    if world.priority == Priority::ConsequenceAware
        && (world.participants.is_empty()
            || world.participants.len() > MAX_FORECAST_PARTICIPANTS
            || world
                .participants
                .iter()
                .any(|p| p.needs.len() > MAX_FORECAST_REQUIREMENTS)
            || world.offers.len() + world.access_offers.len() + world.bids.len()
                > MAX_POSTED_OFFERS)
    {
        return Err(
            "consequence forecast supports one to four participants, at most four requirements each and four offers".into(),
        );
    }
    Ok(())
}

pub(crate) fn choose(sim: &Simulation, batch: &mut Batch) -> Result<(), String> {
    validate(&sim.world)?;
    let horizon = sim.world.decision_horizon.unwrap_or(sim.world.horizon);
    let end = sim
        .state
        .month
        .checked_add(horizon)
        .ok_or("forecast horizon overflow")?;
    if sim
        .world
        .participants
        .iter()
        .all(|p| sim.state.terminal.contains_key(&p.agent))
    {
        return if sim.state.phase == Phase::Acquire {
            Ok(())
        } else {
            sim.productive_with(batch, true)
        };
    }
    let requirements: BTreeSet<_> = sim
        .world
        .participants
        .iter()
        .flat_map(|p| &p.needs)
        .filter(|n| n.quantity > 0)
        .map(|n| n.resource)
        .collect();
    let mut policies = vec![Priority::ContinuingFirst];
    policies.extend(requirements.into_iter().map(Priority::NeedFirstFor));
    let mut alternatives = Vec::new();
    let mut first_batches = Vec::new();
    let mut offers = vec![(None, None, None, Vec::new())];
    if sim.state.phase == Phase::Acquire {
        let mut actors: Vec<_> = sim
            .world
            .participants
            .iter()
            .map(|p| p.agent)
            .filter(|id| !sim.state.terminal.contains_key(id))
            .collect();
        actors.sort_unstable();
        let mut equipment: Vec<_> = sim.world.offers.iter().collect();
        equipment.sort_by_key(|o| o.id);
        for offer in equipment {
            for &buyer in &actors {
                if crate::equipment::transaction(
                    &sim.world,
                    &sim.state,
                    crate::equipment::Trade {
                        offer: offer.id,
                        buyer,
                    },
                )
                .is_ok()
                {
                    offers.push((Some(offer.id), None, None, vec![buyer]));
                }
            }
        }
        let mut access: Vec<_> = sim.world.access_offers.iter().collect();
        access.sort_by_key(|o| o.id);
        for offer in access {
            if actors.contains(&offer.debtor)
                && crate::commitments::acceptance(&sim.world, &sim.state, offer.id).is_ok()
            {
                offers.push((None, Some(offer.id), None, Vec::new()));
            }
        }
        let mut bids: Vec<_> = sim.world.bids.iter().collect();
        bids.sort_by_key(|b| b.id);
        for bid in bids {
            let sellers: Vec<_> = actors
                .iter()
                .copied()
                .filter(|&seller| {
                    crate::currency::transaction(
                        &sim.world,
                        &sim.state,
                        crate::currency::StockTrade {
                            bid: bid.id,
                            seller,
                        },
                    )
                    .is_ok()
                })
                .collect();
            // All bounded subsets share the same opening treasury budget and space.
            // One lot per seller; masks provide explicit stable-ID tie breaking.
            for mask in 1..(1usize << sellers.len()) {
                let selected = sellers
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, id)| *id)
                    .collect();
                offers.push((None, None, Some(bid.id), selected));
            }
        }
    }
    for (offer, access_offer, stock_bid, trade_agents) in offers {
        let mut acquisition = batch.clone();
        acquisition.accept_access = access_offer;
        if sim.state.phase == Phase::Acquire {
            for &agent in &trade_agents {
                if let Some(offer) = offer {
                    acquisition.transactions.push(crate::equipment::transaction(
                        &sim.world,
                        &sim.state,
                        crate::equipment::Trade {
                            offer,
                            buyer: agent,
                        },
                    )?);
                }
                if let Some(bid) = stock_bid {
                    acquisition.transactions.push(crate::currency::transaction(
                        &sim.world,
                        &sim.state,
                        crate::currency::StockTrade { bid, seller: agent },
                    )?);
                }
            }
            let mut preview = sim.state.clone();
            // Individually feasible trades can jointly overdraw the treasury.
            if commit(
                &sim.world,
                &mut preview,
                &acquisition,
                Backend::Reference,
                sim.effect_limit,
            )
            .is_err()
            {
                continue;
            }
        }
        for &policy in &policies {
            let mut choices = vec![(false, None), (true, None)];
            choices.extend(
                substitute_producers(&sim.world)
                    .into_iter()
                    .map(|id| (false, Some(id))),
            );
            for (defer_new, preferred_process) in choices {
                let mut forecast_world = sim.world.clone();
                forecast_world.priority = policy;
                // The current Open capacity is observed. Future fixture overrides and
                // scheduled intents are not observations and must not leak into forecasts.
                forecast_world.capacity_overrides.clear();
                forecast_world
                    .scheduled_starts
                    .retain(|s| s.month == sim.state.month);
                let mut forecast =
                    Simulation::new(forecast_world, sim.state.clone(), Backend::Reference)?;
                forecast.effect_limit = sim.effect_limit;
                let mut first = acquisition.clone();
                let mut work;
                if sim.state.phase == Phase::Acquire {
                    commit(
                        &forecast.world,
                        &mut forecast.state,
                        &first,
                        Backend::Reference,
                        forecast.effect_limit,
                    )?;
                    forecast.ledger.push(first.clone());
                    work = Batch::empty(&forecast.state);
                } else {
                    work = first.clone();
                }
                forecast.productive_choice(&mut work, defer_new, preferred_process)?;
                commit(
                    &forecast.world,
                    &mut forecast.state,
                    &work,
                    Backend::Reference,
                    forecast.effect_limit,
                )?;
                forecast.ledger.push(work.clone());
                if sim.state.phase == Phase::Acquire {
                    first.production_plan = Some(Box::new(work.clone()));
                } else {
                    first = work.clone();
                }
                while forecast.state.month < end {
                    forecast.step()?;
                }
                alternatives.push(Forecast {
                    offer,
                    access_offer,
                    stock_bid,
                    trade_agents: trade_agents.clone(),
                    priority: policy,
                    defer_new,
                    preferred_process,
                    score: score(&forecast),
                    outcomes: forecast.reports,
                    first_work: work.receipts.clone(),
                });
                first_batches.push(first);
            }
        }
    }
    let selected = alternatives
        .iter()
        .enumerate()
        .min_by_key(|(i, f)| (f.score.clone(), *i))
        .map(|(i, _)| i)
        .ok_or("no forecast alternatives")?;
    *batch = first_batches.swap_remove(selected);
    batch.decision = Some(Decision {
        month: sim.state.month,
        through: end - 1,
        alternatives,
        selected,
    });
    Ok(())
}

fn score(sim: &Simulation) -> Score {
    let mut score = Score::default();
    for row in &sim.reports {
        if row.terminal.is_some() {
            score.terminal_months += 1;
        }
        let mut impaired = false;
        for rule in &sim.world.condition_rules {
            if rule.subject != row.agent {
                continue;
            }
            let points = row
                .conditions
                .get(&rule.provision)
                .map(|c| c.deprivation)
                .unwrap_or(0);
            impaired |= points >= rule.impaired_at;
            score.deprivation += u64::from(points) * SCORE_SCALE / u64::from(rule.terminal_at);
        }
        score.impaired_months += u64::from(impaired);
    }
    // Compare capped end-of-horizon coverage in period-equivalents, not raw kg vs fuel.
    // This tie-break is deliberately secondary to all condition consequences.
    for participant in &sim.world.participants {
        let mut stocks = crate::substitution::stocks(&sim.state, participant.agent);
        let mut needs: Vec<_> = participant.needs.iter().collect();
        needs.sort_by_key(|n| (n.priority, n.resource));
        let mut deficits = std::collections::BTreeMap::<ResourceId, i128>::new();
        // Fulfillment expires monthly. A two-provision lot cannot feed two
        // separate months when only one provision is required each month.
        for _ in 0..sim.world.horizon {
            for need in &needs {
                if need.quantity == 0 {
                    continue;
                }
                let (_, unmet) = crate::substitution::allocate(
                    &sim.world,
                    need.resource,
                    i128::from(need.quantity),
                    &mut stocks,
                    &Default::default(),
                );
                *deficits.entry(need.resource).or_default() += unmet;
            }
        }
        for need in needs {
            if need.quantity == 0 {
                continue;
            }
            let target = i128::from(need.quantity) * i128::from(sim.world.horizon);
            score.buffer_gap +=
                (deficits[&need.resource] * i128::from(SCORE_SCALE) / target) as u64;
        }
    }
    score.work = sim
        .ledger
        .iter()
        .filter(|b| b.phase == Phase::Productive)
        .flat_map(|b| &b.receipts)
        .map(|r| r.completed as u64)
        .sum();
    score
}
