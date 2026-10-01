//! Experimental bounded composition over authoritative offer adapters.
//! Search owns no rights or allocation. Forecasts and publication use settlement.
use crate::{
    agreements::{PaymentTerms, ProductionTerms},
    compute::Backend,
    forecast::ForecastContext,
    model::*,
    offers::{self, Id, Request, Terms},
    planning::Score,
    simulation::Simulation,
};
use std::collections::{BTreeMap, BTreeSet};

pub mod calibration;
pub mod continuation;
pub mod market;

const BEAM_WIDTH: usize = 8;
const MAX_PACKAGE: usize = 6;
const OUTPUT_WEIGHT: i64 = 100;
const PREREQUISITE_WEIGHT: i64 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    Beam,
    BestFirst,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub expansions: usize,
    pub forecasts: usize,
    pub months: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metrics {
    pub expansions: usize,
    pub rejected: usize,
    pub forecasts: usize,
    pub forecast_months: u64,
    pub peak_frontier: usize,
    pub exhausted: bool,
    pub pruned: usize,
    pub rejections: BTreeMap<String, usize>,
}

/// Explicit search mandate. Household membership alone is not private consent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    Person(AgentId),
    Household {
        agent: AgentId,
        consenting_members: BTreeSet<AgentId>,
    },
}
impl Scope {
    fn actors(&self, sim: &Simulation) -> Result<Vec<AgentId>, String> {
        match self {
            Self::Person(a)
                if sim
                    .world
                    .households
                    .iter()
                    .any(|h| crate::households::membership::current(h).contains(a)) =>
            {
                Err("use an explicit household mandate for a household member".into())
            }
            Self::Person(a) => Ok(vec![*a]),
            Self::Household {
                agent,
                consenting_members,
            } => {
                let h = sim
                    .world
                    .households
                    .iter()
                    .find(|h| h.agent == *agent)
                    .ok_or("unknown household mandate")?;
                if sim.world.households.len() != 1
                    || sim
                        .world
                        .participants
                        .iter()
                        .any(|p| !crate::households::membership::current(h).contains(&p.agent))
                {
                    return Err(
                        "household composition requires an isolated household forecast branch"
                            .into(),
                    );
                }
                if consenting_members.is_empty()
                    || !consenting_members
                        .iter()
                        .all(|a| crate::households::membership::current(h).contains(a))
                {
                    return Err("household search requires consenting adult members".into());
                }
                Ok(consenting_members.iter().copied().collect())
            }
        }
    }
}

/// A dated description is a view, not an executable promise. Terms carry authority,
/// asset requirements and failure rules; storage/capacity remain in the snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    pub request: Request,
    pub observed_month: u32,
    pub terms: Terms,
    pub requirements: Vec<(u32, Vec<Amount>)>,
    pub outputs: Vec<(u32, Vec<Amount>)>,
    pub payments: Vec<PaymentTerms>,
}

pub fn describe(context: &ForecastContext, agent: AgentId) -> Result<Vec<Description>, String> {
    let w = context.world();
    let s = context.state();
    offers::discover(w, s, agent)
        .into_iter()
        .filter(|o| match o.id {
            Id::Membership(_) | Id::Land(_) => true,
            Id::Process(id) => w.definition(id).execution == Execution::Productive,
            _ => false,
        })
        .map(|o| {
            let mut d = Description {
                request: Request::new(o.id, agent),
                observed_month: s.month,
                terms: o.terms,
                requirements: vec![],
                outputs: vec![],
                payments: vec![],
            };
            match &d.terms {
                Terms::Production(p) | Terms::Collection { production: p, .. } => {
                    d.requirements = p.schedule(s.month)?;
                    let end = s
                        .month
                        .checked_add(p.duration())
                        .and_then(|m| m.checked_sub(1))
                        .ok_or("opportunity date overflow")?;
                    d.outputs.push((end, p.outputs.clone()));
                }
                Terms::Land(a) => {
                    // Acceptance binds an open template's debtor and activation date.
                    let mut a = a.clone();
                    a.debtor = agent;
                    a.activated = s.month;
                    d.payments = a.contract(w, s)?.payments;
                }
                _ => {}
            }
            Ok(d)
        })
        .collect()
}

/// Backward closure includes consumption recipes and terminates on seed cycles.
fn relevant(w: &World, actors: &[AgentId]) -> BTreeSet<DefinitionId> {
    let mut wanted: BTreeSet<_> = w
        .participants
        .iter()
        .filter(|p| actors.contains(&p.agent))
        .flat_map(|p| {
            p.needs
                .iter()
                .filter(|n| n.quantity > 0)
                .map(|n| n.resource)
        })
        .collect();
    let mut found = BTreeSet::new();
    loop {
        let before = (wanted.len(), found.len());
        for d in &w.definitions {
            if d.enabled && d.outputs.iter().any(|o| wanted.contains(&o.resource)) {
                found.insert(d.id);
                wanted.extend(
                    d.stages
                        .iter()
                        .flat_map(|s| &s.entry_inputs)
                        .map(|a| a.resource),
                );
            }
        }
        if before == (wanted.len(), found.len()) {
            return found;
        }
    }
}

fn production(d: &Description) -> Option<&ProductionTerms> {
    match &d.terms {
        Terms::Production(p) | Terms::Collection { production: p, .. } => Some(p),
        _ => None,
    }
}

/// Optimistic ordering hint. Never authorizes resources and is not an admissible
/// optimality bound. Full rollouts, rather than this scalar, select the winner.
fn hint(sim: &Simulation, descriptions: &[Description], indices: &[usize]) -> i64 {
    let mut score = 0;
    let mut work = BTreeMap::<(AgentId, u32, ResourceId), i64>::new();
    for &i in indices {
        let d = &descriptions[i];
        if let Some(p) = production(d) {
            score += OUTPUT_WEIGHT - i64::from(p.duration());
            for (month, amounts) in &d.requirements {
                for a in amounts {
                    if sim
                        .world
                        .resources
                        .iter()
                        .any(|r| r.id == a.resource && r.kind == ResourceKind::Capacity)
                    {
                        *work
                            .entry((d.request.agent, *month, a.resource))
                            .or_default() += i64::from(a.quantity);
                    } else {
                        let available = sim.state.balance(d.request.agent, a.resource);
                        score -= i64::from((a.quantity - available).max(0));
                    }
                }
            }
        } else {
            score += PREREQUISITE_WEIGHT;
        }
    }
    for ((agent, _, resource), quantity) in work {
        let capacity = sim
            .world
            .participants
            .iter()
            .find(|p| p.agent == agent && p.capacity.resource == resource)
            .map_or(0, |p| i64::from(p.capacity.quantity));
        score -= (quantity - capacity).max(0) * OUTPUT_WEIGHT;
    }
    -score // Lower sorts first.
}

#[derive(Clone, Debug)]
struct Node {
    indices: Vec<usize>,
    hint: i64,
}

/// Current acceptance plus ordinary fixed-policy continuation. Kept public for
/// finite test oracles; callers cannot provide raw transactions to this entry.
pub fn forecast_package(
    sim: &Simulation,
    requests: &[Request],
    months: u32,
) -> Result<(Batch, Score, Simulation), String> {
    forecast_orders(sim, requests, None, months)
}

pub fn forecast_orders(
    sim: &Simulation,
    requests: &[Request],
    orders: Option<&crate::town_market::OrderSelection>,
    months: u32,
) -> Result<(Batch, Score, Simulation), String> {
    if months == 0 {
        return Err("forecast needs a positive horizon".into());
    }
    let mut batch = market::prepare(sim, requests, orders)?;
    let (mut w, s) = ForecastContext::new(&sim.world, &sim.state).into_parts();
    w.priority = Priority::ContinuingFirst;
    w.competition = None;
    let mut branch = Simulation::new(w, s, Backend::Reference)?;
    branch.effect_limit = sim.effect_limit;
    // Even empty/prerequisite-only packages mean continue existing work, not
    // permission to auto-start a new crop in the current boundary.
    if batch.production_plan.is_none() {
        let mut preview = branch.clone();
        crate::settlement::commit(
            &preview.world,
            &mut preview.state,
            &batch,
            Backend::Reference,
            preview.effect_limit,
        )?;
        let requests: Vec<_> = preview
            .state
            .processes
            .values()
            .filter(|p| p.status == Status::Active)
            .map(|p| Request {
                offer: Id::Process(p.definition),
                agent: p.operator,
                continuing: Some(p.id),
                need: p.goal,
            })
            .collect();
        let mut work = Batch::empty(&preview.state);
        if preview.world.households.is_empty() {
            offers::resolve(&preview, &requests, &mut work)?;
        } else {
            crate::households::prepare_offers(&preview, &requests, &mut work)?;
        }
        batch.production_plan = Some(Box::new(work));
    }
    crate::settlement::commit(
        &branch.world,
        &mut branch.state,
        &batch,
        Backend::Reference,
        branch.effect_limit,
    )?;
    branch.ledger.push(batch.clone());
    let end = sim
        .state
        .month
        .checked_add(months)
        .ok_or("forecast horizon overflow")?;
    while branch.state.month < end {
        branch.step()?;
    }
    let mut score = crate::planning::score(&branch);
    score.broken_commitments = branch
        .state
        .processes
        .values()
        .filter(|p| p.status == Status::Aborted)
        .count() as u64;
    if let Some(orders) = orders {
        score = market::actor_score(&branch, orders.actor);
    }
    Ok((batch, score, branch))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    opening: ForecastContext,
    pub requests: Vec<Request>,
    pub orders: Option<crate::town_market::OrderSelection>,
    pub batch: Batch,
    pub score: Score,
    pub metrics: Metrics,
    /// Outstanding work/claims are retained, including beyond the forecast horizon.
    pub ending_active: usize,
    pub ending_arrears: i64,
    pub ending_contracts: Vec<crate::agreements::Agreement>,
}
impl Selection {
    pub fn accept(&self, sim: &mut Simulation) -> Result<(), String> {
        if ForecastContext::new(&sim.world, &sim.state) != self.opening {
            return Err("stale composition selection".into());
        }
        crate::settlement::commit(
            &sim.world,
            &mut sim.state,
            &self.batch,
            sim.backend,
            sim.effect_limit,
        )?;
        sim.ledger.push(self.batch.clone());
        Ok(())
    }
}

/// Search is deliberately opt-in. Neither default policy nor the monthly
/// scheduler is changed. The caller chooses the Acquire boundaries to reconsider.
pub fn choose(
    sim: &Simulation,
    scope: &Scope,
    strategy: Strategy,
    budget: Budget,
) -> Result<Selection, String> {
    choose_with_scoring(
        sim,
        scope,
        strategy,
        budget,
        calibration::Scoring::PrivateBuffers,
    )
}

fn validate_search(
    sim: &Simulation,
    scope: &Scope,
    budget: Budget,
) -> Result<Vec<AgentId>, String> {
    if sim.state.phase != Phase::Acquire || budget.forecasts == 0 || budget.months == 0 {
        return Err("composition requires Acquire and positive forecast budget/horizon".into());
    }
    if sim.world.production_market.is_some()
        || sim.world.market.is_some()
        || sim.world.negotiation.is_some()
        || sim.world.credit.is_some()
        || !sim.world.lending.is_empty()
        || sim.world.minting.is_some()
        || sim.world.pool_market.is_some()
        || !sim.world.employment.is_empty()
        || !sim.world.prepaid_deliveries.is_empty()
        || !sim.world.issuance.is_empty()
        || !sim.world.offers.is_empty()
        || !sim.world.bids.is_empty()
    {
        return Err("composition adapter does not support this market/financial driver".into());
    }
    let actors = scope.actors(sim)?;
    if actors
        .iter()
        .any(|a| !sim.world.participants.iter().any(|p| p.agent == *a))
    {
        return Err("composition requires a participating actor".into());
    }
    // An individual forecast never schedules a rival's productive work.
    if matches!(scope, Scope::Person(_))
        && sim.world.participants.len() != 1
        && sim.world.town_market.is_none()
    {
        return Err(
            "individual composition requires an isolated observation/forecast branch".into(),
        );
    }
    if sim.world.town_market.is_some() && !sim.world.households.is_empty() {
        return Err("composition market adapter requires an individual mandate".into());
    }
    let market_actor = sim.world.town_market.as_ref().map(|_| actors[0]);
    if market_actor.is_some_and(|actor| {
        sim.world
            .participants
            .iter()
            .any(|p| p.agent != actor && p.capacity.quantity != 0)
    }) {
        return Err("composition town adapter currently requires passive counterparties".into());
    }
    Ok(actors)
}

pub fn choose_with_scoring(
    sim: &Simulation,
    scope: &Scope,
    strategy: Strategy,
    budget: Budget,
    scoring: calibration::Scoring,
) -> Result<Selection, String> {
    let actors = validate_search(sim, scope, budget)?;
    let market_actor = sim.world.town_market.as_ref().map(|_| actors[0]);
    let mut order_actions = vec![];
    if let Some(actor) = market_actor {
        let round = crate::town_market::evaluate(&sim.world, &sim.state)?;
        order_actions = round
            .orders
            .iter()
            .filter(|o| o.agent == actor)
            .map(|o| (o.market, o.side))
            .collect();
        order_actions.sort();
    }
    let context = ForecastContext::new(&sim.world, &sim.state);
    let relevant = relevant(&sim.world, &actors);
    let mut descriptions = Vec::new();
    for actor in actors {
        for d in describe(&context, actor)? {
            if production(&d).is_none_or(|p| relevant.contains(&p.definition)) {
                descriptions.push(d);
            }
        }
    }
    descriptions.sort_by_key(|d| (d.request.offer, d.request.agent));
    let action_count = descriptions.len() + order_actions.len();
    let package = |indices: &[usize]| {
        let requests: Vec<_> = indices
            .iter()
            .filter(|&&i| i < descriptions.len())
            .map(|&i| descriptions[i].request.clone())
            .collect();
        let orders = market_actor.map(|actor| crate::town_market::OrderSelection {
            actor,
            submit: indices
                .iter()
                .filter(|&&i| i >= descriptions.len())
                .map(|&i| order_actions[i - descriptions.len()])
                .collect(),
        });
        (requests, orders)
    };
    let root = Node {
        indices: vec![],
        hint: 0,
    };
    let mut frontier = vec![root];
    let mut metrics = Metrics {
        peak_frontier: 1,
        ..Default::default()
    };
    let mut best: Option<Selection> = None;
    while !frontier.is_empty() {
        if metrics.forecasts == budget.forecasts {
            metrics.exhausted = true;
            break;
        }
        frontier.sort_by_key(|n| (n.hint, n.indices.clone()));
        let count = if strategy == Strategy::Beam {
            frontier.len()
        } else {
            1
        };
        let layer: Vec<_> = frontier.drain(..count).collect();
        for (position, node) in layer.into_iter().enumerate() {
            if metrics.forecasts == budget.forecasts {
                metrics.exhausted = true;
                break;
            }
            let (requests, orders) = package(&node.indices);
            let (batch, _, branch) =
                forecast_orders(sim, &requests, orders.as_ref(), budget.months)?;
            let score = match market_actor {
                Some(actor) => market::actor_score(&branch, actor),
                None => calibration::score(&branch, scoring)?,
            };
            metrics.forecasts += 1;
            metrics.forecast_months += u64::from(budget.months);
            if best.as_ref().is_none_or(|b| score < b.score) {
                best = Some(Selection {
                    opening: context.clone(),
                    requests,
                    orders,
                    batch,
                    score,
                    metrics: Metrics::default(),
                    ending_active: branch
                        .state
                        .processes
                        .values()
                        .filter(|p| p.status == Status::Active)
                        .count(),
                    ending_arrears: branch
                        .state
                        .obligations
                        .values()
                        .map(|o| i64::from(o.outstanding()))
                        .sum(),
                    ending_contracts: branch
                        .state
                        .accepted_agreements
                        .values()
                        .map(|a| a.contract(&branch.world, &branch.state))
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .chain(
                            branch
                                .state
                                .processes
                                .values()
                                .filter(|p| p.status == Status::Active)
                                .map(|p| crate::agreements::process(&branch.world, p)),
                        )
                        .collect(),
                });
            }
            if node.indices.len() == MAX_PACKAGE {
                metrics.exhausted |= node.indices.last().is_some_and(|i| i + 1 < action_count);
                continue;
            }
            let start = node.indices.last().map_or(0, |i| i + 1);
            for i in start..action_count {
                if metrics.expansions == budget.expansions {
                    metrics.exhausted = true;
                    break;
                }
                metrics.expansions += 1;
                let mut indices = node.indices.clone();
                indices.push(i);
                let (requests, orders) = package(&indices);
                match market::prepare(sim, &requests, orders.as_ref()) {
                    Ok(_) => frontier.push(Node {
                        hint: hint(
                            sim,
                            &descriptions,
                            &indices
                                .iter()
                                .copied()
                                .filter(|&i| i < descriptions.len())
                                .collect::<Vec<_>>(),
                        ) - (indices.iter().filter(|&&i| i >= descriptions.len()).count()
                            as i64
                            * PREREQUISITE_WEIGHT),
                        indices,
                    }),
                    Err(e) => {
                        metrics.rejected += 1;
                        *metrics.rejections.entry(e).or_default() += 1;
                    }
                }
                metrics.peak_frontier =
                    metrics.peak_frontier.max(frontier.len() + count - position);
            }
        }
        if strategy == Strategy::Beam && frontier.len() > BEAM_WIDTH {
            frontier.sort_by_key(|n| (n.hint, n.indices.clone()));
            metrics.pruned += frontier.len() - BEAM_WIDTH;
            metrics.exhausted = true;
            frontier.truncate(BEAM_WIDTH);
        }
    }
    let mut selected = best.ok_or("composition did not evaluate its fallback")?;
    selected.metrics = metrics;
    Ok(selected)
}

/// Observed town listings, deliberately distinct from executable common offers.
/// A posted price/volume describes the last completed market, not a future buyer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketDescription {
    pub venue: AgentId,
    pub actor: AgentId,
    pub terms: crate::marketplace::Market,
    pub admitted: bool,
    pub registered: bool,
    pub match_limit: Option<u32>,
    pub last_completed: Option<(u32, crate::town_market::MarketResult)>,
}
pub fn describe_markets(context: &ForecastContext, actor: AgentId) -> Vec<MarketDescription> {
    let w = context.world();
    let s = context.state();
    let Some(config) = &w.town_market else {
        return vec![];
    };
    let Some(venue) = crate::marketplace::venue(w, config.venue) else {
        return vec![];
    };
    let mut rows: Vec<_> = crate::town_market::listings(config)
        .into_iter()
        .filter_map(|listing| {
            let terms = venue
                .markets
                .iter()
                .find(|m| m.id == listing.market)?
                .clone();
            let last_completed = s
                .town_market
                .history
                .iter()
                .rev()
                .filter(|r| r.month < s.month)
                .find_map(|r| r.markets.get(&listing.market).map(|m| (r.month, m.clone())));
            Some(MarketDescription {
                venue: venue.agent,
                actor,
                terms,
                admitted: s
                    .town_market
                    .admission
                    .as_ref()
                    .is_some_and(|a| a.month == s.month && a.eligible.contains(&actor)),
                registered: listing.traders.iter().any(|t| t.trader.agent == actor),
                match_limit: listing.match_limit,
                last_completed,
            })
        })
        .collect();
    rows.sort_by_key(|r| r.terms.id);
    rows
}
