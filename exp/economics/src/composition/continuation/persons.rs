//! Independent person proposals, explicit whole-package admission, one settlement.
//! Forecasts assume access to observed shared stocks, never to another person's labor.
use super::{Controller, Policy};
use crate::{
    allocation::{self, Claim, Context, Outcome, Receipt},
    composition::{Budget, Scope, Strategy, expectations},
    compute::Backend,
    forecast::ForecastContext,
    model::*,
    offers::{self, Id, Request},
    simulation::Simulation,
    town_market,
};
use std::collections::{BTreeMap, BTreeSet};

const PACKAGE_POOL: u64 = 0;
const NEW_PLOTS_PER_REVIEW: usize = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub month: u32,
    pub exchanges: BTreeMap<AgentId, Exchange>,
    pub context: Context,
    pub policy: allocation::Policy,
    pub proposals: BTreeMap<AgentId, Vec<Request>>,
    /// Equally scored packages actually explored by each person's own search.
    pub alternatives: BTreeMap<AgentId, Vec<Vec<Request>>>,
    pub accepted: BTreeMap<AgentId, Vec<Request>>,
    pub admission: Vec<Receipt>,
    pub fallback: BTreeMap<AgentId, Vec<Request>>,
    pub fallback_rejections: Vec<(AgentId, Id, String)>,
}

/// Conditional forecast fills versus the jointly cleared live book. Quantities
/// are native goods units by listing and side, never credited from a prediction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exchange {
    pub expectations: expectations::Snapshot,
    pub submit: BTreeSet<(crate::marketplace::MarketId, crate::marketplace::Side)>,
    pub expected: BTreeMap<(crate::marketplace::MarketId, crate::marketplace::Side), i32>,
    pub actual: BTreeMap<(crate::marketplace::MarketId, crate::marketplace::Side), i32>,
}

fn fills(
    round: &town_market::Round,
    actor: AgentId,
) -> BTreeMap<(crate::marketplace::MarketId, crate::marketplace::Side), i32> {
    let mut result = BTreeMap::new();
    for attempt in &round.attempts {
        if !matches!(
            attempt.round.outcome,
            crate::negotiation::Outcome::Traded { .. }
        ) {
            continue;
        }
        let session = &attempt.session;
        let side = if session.buyer.agent == actor {
            crate::marketplace::Side::Buy
        } else if session.seller.agent == actor {
            crate::marketplace::Side::Sell
        } else {
            continue;
        };
        *result.entry((session.market, side)).or_default() += session.goods.quantity;
    }
    result
}

/// Checkpoint this coordinator alongside Simulation. Person review schedules and
/// rejected intentions survive continuation independently of the physical ledger.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Persons {
    pub controllers: BTreeMap<AgentId, Controller>,
    pub counterparty_policy: expectations::Policy,
    pub order_forecast: crate::composition::market::OrderForecast,
    pub policy: allocation::Policy,
    pub seed: u64,
    pub history: Vec<Round>,
}

/// Other persons remain named asset owners/counterparties. Market forecasts keep
/// their ordinary consumption/orders, but never schedule their productive work.
fn local(sim: &Simulation, agent: AgentId) -> Simulation {
    let mut local = sim.clone();
    local.backend = Backend::Reference;
    local.ledger.clear();
    local.reports.clear();
    local.world.competition = None;
    if local.world.town_market.is_some() {
        // Explicit conditional forecast: peers keep current needs/stocks and
        // ordinary order generation, but produce nothing new. Live peers search
        // independently; their accepted work never comes from this branch.
        local
            .world
            .capacity_overrides
            .retain(|(_, a), _| *a == agent);
        for p in &mut local.world.participants {
            if p.agent != agent {
                p.capacity.quantity = 0;
                local
                    .state
                    .balances
                    .insert((p.agent, p.capacity.resource), 0);
            }
        }
    } else {
        local.world.participants.retain(|p| p.agent == agent);
        local.world.condition_rules.retain(|r| r.subject == agent);
        local.state.conditions.retain(|(a, _), _| *a == agent);
        local.state.terminal.retain(|a, _| *a == agent);
    }
    local.state.processes.retain(|_, p| p.operator == agent);
    local.state.pending_production = None;
    local
}

/// A comparison view, never executable state. Ignore unrelated private balances,
/// global batch numbers and globally assigned process IDs. Own commitments,
/// rights/offer availability and shared pool observations still invalidate plans.
fn observation(context: &ForecastContext, agent: AgentId) -> (World, State) {
    let (mut w, mut s) = context.clone().into_parts();
    w.capacity_overrides.retain(|(_, a), _| *a == agent);
    let visible: BTreeSet<_> = w
        .access_offers
        .iter()
        .filter(|a| {
            s.accepted_agreements
                .get(&a.id)
                .is_none_or(|a| a.debtor == agent)
        })
        .map(|a| a.id)
        .collect();
    let rights: BTreeSet<_> = w
        .access_offers
        .iter()
        .filter(|a| visible.contains(&a.id))
        .map(|a| a.right)
        .chain(
            w.rights
                .iter()
                .filter(|r| crate::commitments::holder(&w, &s, r) == Some(agent))
                .map(|r| r.id),
        )
        .collect();
    w.access_offers.retain(|a| visible.contains(&a.id));
    w.open_access_offers.retain(|id| visible.contains(id));
    w.rights.retain(|r| rights.contains(&r.id));
    w.agreements.retain(|a| a.debtor == agent);
    s.accepted_agreements.retain(|_, a| a.debtor == agent);
    s.obligations.retain(|(id, _), _| {
        s.accepted_agreements.contains_key(id) || w.agreements.iter().any(|a| a.id == *id)
    });
    let pools: BTreeSet<_> = w.pools.iter().map(|p| p.account).collect();
    s.balances.retain(|a, _| a.0 == agent || pools.contains(a));
    s.memberships.retain(|(member, _, _), _| *member == agent);
    s.practice.retain(|(a, _), _| *a == agent);
    s.next_batch = 0;
    s.processes = s
        .processes
        .into_values()
        .enumerate()
        .map(|(i, mut p)| {
            p.id = i as u64;
            (p.id, p)
        })
        .collect();
    (w, s)
}

fn intentions(batch: &Batch, agent: AgentId) -> Result<Vec<Request>, String> {
    let mut result = vec![];
    for (id, owner) in batch
        .accept_membership
        .into_iter()
        .chain(batch.additional_memberships.iter().copied())
    {
        result.push(Request::new(Id::Membership(id), owner));
    }
    for (id, owner) in batch
        .accept_access
        .map(|id| (id, batch.access_applicant.unwrap_or(agent)))
        .into_iter()
        .chain(batch.additional_access.iter().copied())
    {
        result.push(Request::new(Id::Land(id), owner));
    }
    if let Some(work) = &batch.production_plan {
        result.extend(
            work.transactions
                .iter()
                .filter_map(|t| t.process.as_ref())
                .filter(|p| p.before.is_none())
                .map(|p| Request {
                    offer: Id::Process(p.after.definition),
                    agent: p.after.operator,
                    continuing: None,
                    need: p.after.goal,
                }),
        );
    }
    if result.iter().any(|r| r.agent != agent) {
        return Err("person proposed another person's work".into());
    }
    Ok(result)
}

/// Common preparation checks cumulative rights, stocks, capacity and storage.
/// Even an empty new package must reserve existing work for every person.
fn prepare(
    sim: &Simulation,
    requests: &[Request],
    acquisition: Option<&Batch>,
) -> Result<Batch, String> {
    let ordered: Vec<_> = requests
        .iter()
        .filter(|r| !matches!(r.offer, Id::Process(_)))
        .chain(
            requests
                .iter()
                .filter(|r| matches!(r.offer, Id::Process(_))),
        )
        .cloned()
        .collect();
    let mut batch = offers::prepare_with_acquisition(sim, &ordered, acquisition.cloned())?;
    if batch.production_plan.is_none() {
        let mut preview = sim.clone();
        crate::settlement::commit(
            &preview.world,
            &mut preview.state,
            &batch,
            Backend::Reference,
            preview.effect_limit,
        )?;
        let work_requests = preview
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
            .collect::<Vec<_>>();
        let mut work = Batch::empty(&preview.state);
        offers::resolve(&preview, &work_requests, &mut work)?;
        batch.production_plan = Some(Box::new(work));
    }
    Ok(batch)
}

impl Persons {
    pub fn new(
        agents: impl IntoIterator<Item = AgentId>,
        strategy: Strategy,
        budget: Budget,
        review: Policy,
        policy: allocation::Policy,
        seed: u64,
    ) -> Result<Self, String> {
        let mut controllers = BTreeMap::new();
        for agent in agents {
            let mut controller = Controller::new(Scope::Person(agent), strategy, budget, review);
            controller.land_limit = Some(NEW_PLOTS_PER_REVIEW);
            if controllers.insert(agent, controller).is_some() {
                return Err("duplicate person controller".into());
            }
        }
        if controllers.is_empty() {
            return Err("person controllers required".into());
        }
        Ok(Self {
            controllers,
            counterparty_policy: expectations::Policy::Ordinary,
            order_forecast: crate::composition::market::OrderForecast::StandingPolicy,
            policy,
            seed,
            history: vec![],
        })
    }

    pub fn step(&mut self, sim: &mut Simulation) -> Result<(), String> {
        self.counterparty_policy.validate()?;
        if !sim.world.households.is_empty() || sim.world.priority != Priority::ContinuingFirst {
            return Err("independent continuation requires persons without households and with ContinuingFirst".into());
        }
        let participants: BTreeSet<_> = sim.world.participants.iter().map(|p| p.agent).collect();
        if participants != self.controllers.keys().copied().collect()
            || participants.len() != sim.world.participants.len()
        {
            return Err("exactly one controller required for each participant".into());
        }
        // Reject unsupported drivers before advancing any phase.
        for (&agent, controller) in &self.controllers {
            if sim.world.town_market.is_some() && controller.policy != Policy::Monthly {
                return Err("multi-person market planning requires monthly review".into());
            }
            let mut branch = local(sim, agent);
            branch.state.phase = Phase::Acquire;
            super::super::validate_search(&branch, &Scope::Person(agent), controller.budget)?;
            if controller.scope != Scope::Person(agent) {
                return Err("controller mandate differs from person".into());
            }
            if controller.land_limit != Some(NEW_PLOTS_PER_REVIEW) {
                return Err("person controller must retain the new land request limit".into());
            }
        }
        if sim.state.phase != Phase::Acquire {
            return sim.step();
        }
        let mut next = self.clone();
        let mut proposals = BTreeMap::new();
        let mut alternatives = BTreeMap::new();
        let mut exchanges = BTreeMap::new();
        let mut selections = town_market::OrderSelections::new();
        for (&agent, controller) in &mut next.controllers {
            controller.order_forecast = self.order_forecast;
            let expectation =
                expectations::Snapshot::observe(&sim.state, agent, self.counterparty_policy)?;
            controller.counterparties = (sim.world.town_market.is_some()
                && self.counterparty_policy != expectations::Policy::Ordinary)
                .then(|| expectation.clone());
            let mut branch = local(sim, agent);
            controller.step_comparing(&mut branch, |a, b| {
                observation(a, agent) == observation(b, agent)
            })?;
            proposals.insert(
                agent,
                intentions(
                    branch.ledger.last().ok_or("missing person proposal")?,
                    agent,
                )?,
            );
            if let Some(town_market::Boundary::Market(round)) =
                &branch.ledger.last().unwrap().town_market
            {
                let selected = if let Some(selected) = &round.selection {
                    if selected.actor != agent {
                        return Err("order mask exceeds person mandate".into());
                    }
                    &selected.submit
                } else {
                    round
                        .selections
                        .as_ref()
                        .and_then(|s| s.get(&agent))
                        .ok_or("missing personal order mask")?
                };
                selections.insert(agent, selected.clone());
                exchanges.insert(
                    agent,
                    Exchange {
                        expectations: expectation,
                        submit: selected.clone(),
                        expected: fills(round, agent),
                        actual: BTreeMap::new(),
                    },
                );
            } else if controller
                .history
                .last()
                .is_some_and(|r| r.search.is_some())
            {
                alternatives.insert(agent, controller.alternatives.clone());
            }
        }
        let acquisition = if sim.world.town_market.is_some() {
            let batch = crate::composition::market::prepare_all(sim, &[], &selections)?;
            let Some(town_market::Boundary::Market(round)) = &batch.town_market else {
                return Err("missing jointly prepared town book".into());
            };
            for (&agent, exchange) in &mut exchanges {
                exchange.actual = fills(round, agent);
            }
            Some(batch)
        } else {
            None
        };
        let context = Context {
            seed: self.seed,
            pool: PACKAGE_POOL,
            round: u64::from(sim.state.month),
        };
        let claims: Vec<_> = proposals
            .keys()
            .map(|a| Claim {
                id: u64::from(*a),
                priority: 0,
                requested: 1,
                minimum: 1,
            })
            .collect();
        let mut accepted = vec![];
        let mut accepted_packages = BTreeMap::new();
        let mut batch = prepare(sim, &accepted, acquisition.as_ref())?;
        let admission = allocation::resolve(
            context,
            &self.policy,
            u32::try_from(claims.len()).map_err(|_| "too many people")?,
            &claims,
            |claim, _| {
                let agent = claim.id as AgentId;
                let mut last = "no feasible package".to_string();
                for proposal in std::iter::once(&proposals[&agent])
                    .chain(alternatives.get(&agent).into_iter().flatten())
                {
                    if proposal.iter().any(|r| r.agent != agent) {
                        return Err("alternative exceeds person mandate".into());
                    }
                    let mut candidate = accepted.clone();
                    candidate.extend(proposal.clone());
                    match prepare(sim, &candidate, acquisition.as_ref()) {
                        Ok(prepared) => {
                            accepted_packages.insert(agent, proposal.clone());
                            accepted = candidate;
                            batch = prepared;
                            return Ok(());
                        }
                        Err(reason) => last = reason,
                    }
                }
                Err(last)
            },
        )?;
        let mut fallback = BTreeMap::new();
        let mut fallback_rejections = vec![];
        for receipt in &admission {
            if matches!(receipt.outcome, Outcome::Reserved(_)) {
                continue;
            }
            let agent = receipt.claim.id as AgentId;
            // Rejection invalidates the proposed acquisition, not a binding contract.
            // Retry at the next Acquire even under scheduled review.
            next.controllers.get_mut(&agent).unwrap().admission_rejected = true;
            let mut preview = sim.clone();
            let mut settled = batch.clone();
            settled.production_plan = None;
            crate::settlement::commit(
                &preview.world,
                &mut preview.state,
                &settled,
                Backend::Reference,
                preview.effect_limit,
            )?;
            let preview = local(&preview, agent);
            let mut ignored = Batch::empty(&preview.state);
            let work = preview.productive_requests(&mut ignored, false, None)?;
            for r in work.into_iter().filter(|r| r.existing.is_none()) {
                let request = Request {
                    offer: Id::Process(r.definition),
                    agent,
                    continuing: None,
                    need: r.need,
                };
                let mut candidate = accepted.clone();
                candidate.push(request.clone());
                match prepare(sim, &candidate, acquisition.as_ref()) {
                    Ok(prepared) => {
                        accepted = candidate;
                        batch = prepared;
                        fallback.entry(agent).or_insert_with(Vec::new).push(request);
                    }
                    Err(reason) => fallback_rejections.push((agent, request.offer, reason)),
                }
            }
        }
        crate::settlement::commit(
            &sim.world,
            &mut sim.state,
            &batch,
            sim.backend,
            sim.effect_limit,
        )?;
        next.history.push(Round {
            month: batch.month,
            exchanges,
            context,
            policy: self.policy,
            proposals,
            alternatives,
            accepted: accepted_packages,
            admission,
            fallback,
            fallback_rejections,
        });
        sim.ledger.push(batch);
        *self = next;
        Ok(())
    }
}
