//! A bounded two-link planner and generic staged-process resolver.
use crate::{
    commitments,
    compute::Backend,
    maintenance,
    model::*,
    settlement::{DEFAULT_EFFECT_LIMIT, validate_world},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct Simulation {
    pub world: World,
    pub state: State,
    pub ledger: Vec<Batch>,
    pub reports: Vec<MonthReport>,
    pub backend: Backend,
    pub effect_limit: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct Request {
    pub(crate) agent: AgentId,
    pub(crate) definition: DefinitionId,
    pub(crate) existing: Option<u64>,
    pub(crate) need: Option<ResourceId>,
}

impl Simulation {
    pub fn new(world: World, state: State, backend: Backend) -> Result<Self, String> {
        validate_world(&world, &state)?;
        Ok(Self {
            world,
            state,
            ledger: Vec::new(),
            reports: Vec::new(),
            backend,
            effect_limit: DEFAULT_EFFECT_LIMIT,
        })
    }

    /// One committed visibility barrier. Cloning Simulation gives an in-memory
    /// checkpoint including configuration, pending fixture intents and receipts.
    pub fn step(&mut self) -> Result<(), String> {
        if !self.world.households.is_empty() {
            return crate::households::step(self);
        }
        self.step_core()
    }

    pub(crate) fn step_core(&mut self) -> Result<(), String> {
        let mut batch = Batch {
            credit: None,
            negotiation: None,
            accept_membership: None,
            household: None,
            id: self.state.next_batch,
            month: self.state.month,
            phase: self.state.phase,
            transactions: Vec::new(),
            receipts: Vec::new(),
            maintenance: None,
            decision: None,
            production_plan: None,
            commitments: None,
            accept_access: None,
            access_applicant: None,
            additional_access: vec![],
            additional_memberships: vec![],
            allocation: None,
            pool_market: None,
            plot_request: None,
        };
        if self.world.credit.is_some()
            && matches!(self.state.phase, Phase::Open | Phase::Due | Phase::Acquire)
        {
            batch.credit = crate::credit::evaluate(&self.world, &self.state)?;
            batch.transactions = batch.credit.as_ref().unwrap().transactions.clone();
        } else {
            match self.state.phase {
                Phase::Open => Self::open(&self.world, &self.state, &mut batch),
                Phase::Due | Phase::ClearArrears => {
                    let settlement = crate::commitments::evaluate(&self.world, &self.state)?;
                    batch.transactions = settlement.transactions.clone();
                    batch.commitments = Some(settlement);
                }
                Phase::Acquire => {
                    if self.world.negotiation.is_some() {
                        batch.negotiation = crate::negotiation::evaluate(&self.world, &self.state)?;
                        batch.transactions = crate::negotiation::transactions(
                            &self.world,
                            &self.state,
                            &batch.negotiation,
                        )?;
                    } else if self.world.market.is_some() {
                        batch.transactions = crate::exchange::resolve(&self.world, &self.state)?;
                        batch.plot_request = crate::plots::after_market(
                            &self.world,
                            &self.state,
                            &batch.transactions,
                        )?;
                        batch.accept_access = batch
                            .plot_request
                            .as_ref()
                            .filter(|r| r.reason == crate::plots::Reason::Accepted)
                            .and_then(|r| r.offer);
                    } else if self.world.competition.is_some() {
                        crate::competition::choose(self, &mut batch)?;
                    } else if self.world.priority == Priority::ConsequenceAware {
                        crate::planning::choose(self, &mut batch)?;
                    }
                }
                Phase::Productive => self.productive(&mut batch)?,
                Phase::Consumption => self.consume(&mut batch)?,
                Phase::Close => {
                    batch.maintenance = Some(maintenance::evaluate(&self.world, &self.state)?);
                }
            }
        }
        let mut reports = if self.state.phase == Phase::Close {
            self.month_reports()
        } else {
            Vec::new()
        };
        crate::settlement::commit_core(
            &self.world,
            &mut self.state,
            &batch,
            self.backend,
            self.effect_limit,
        )?;
        for report in &mut reports {
            report.conditions = self
                .state
                .conditions
                .iter()
                .filter(|((owner, _), _)| *owner == report.agent)
                .map(|((_, resource), value)| (*resource, value.clone()))
                .collect();
            report.terminal = self.state.terminal.get(&report.agent).cloned();
        }
        self.ledger.push(batch);
        self.reports.extend(reports);
        Ok(())
    }

    /// Runs complete monthly boundaries even when resuming mid-month.
    pub fn run_months(&mut self, count: u32) -> Result<(), String> {
        let end = self
            .state
            .month
            .checked_add(count)
            .ok_or("month overflow")?;
        while self.state.month < end {
            self.step()?;
        }
        Ok(())
    }

    fn sorted_participants(&self) -> Vec<&Participant> {
        let mut people: Vec<_> = self.world.participants.iter().collect();
        people.sort_by_key(|p| p.agent);
        people
    }

    pub(crate) fn open(world: &World, state: &State, batch: &mut Batch) {
        let mut effects = crate::activities::expiration(world, state);
        effects.extend(crate::pools::regeneration(world, state));
        if !effects.is_empty() {
            batch.transactions.push(Transaction {
                cause: "shared pool regeneration".into(),
                effects,
                process: None,
                technique_use: None,
                trade: None,
                stock_trade: None,
                forward: None,
                delivery: None,
                royalty: None,
            });
        }
        // Regeneration is an explicit source; unused services and previous-period
        // fulfillment expire. Stock accounts never reset here.
        let mut agents: Vec<_> = world.agents.iter().collect();
        agents.sort_by_key(|a| a.id);
        let mut resources: Vec<_> = world.resources.iter().collect();
        resources.sort_by_key(|r| r.id);
        for agent in agents {
            for resource in &resources {
                if resource.kind == ResourceKind::Stock {
                    continue;
                }
                let old = state.balance(agent.id, resource.id);
                let base = world
                    .participants
                    .iter()
                    .find(|p| p.agent == agent.id && p.capacity.resource == resource.id)
                    .map(|p| {
                        world
                            .capacity_overrides
                            .get(&(state.month, p.agent))
                            .copied()
                            .unwrap_or(p.capacity.quantity)
                    })
                    .unwrap_or(0);
                let new = maintenance::capacity(world, state, agent.id, resource.id, base);
                let mut effects = Vec::new();
                if old != 0 {
                    effects.push(Effect {
                        account: (agent.id, resource.id),
                        delta: -old,
                    });
                }
                if new != 0 {
                    effects.push(Effect {
                        account: (agent.id, resource.id),
                        delta: new,
                    });
                }
                if !effects.is_empty() {
                    batch.transactions.push(Transaction {
                        technique_use: None,
                        trade: None,
                        stock_trade: None,
                        forward: None,
                        delivery: None,
                        royalty: None,
                        cause: "period expiration/regeneration".into(),
                        effects,
                        process: None,
                    });
                }
            }
        }
    }

    fn productive(&self, batch: &mut Batch) -> Result<(), String> {
        if let Some(plan) = &self.state.pending_production {
            *batch = *plan.clone();
            return Ok(());
        }
        if self.world.priority == Priority::ConsequenceAware {
            return crate::planning::choose(self, batch);
        }
        self.productive_with(batch, false)
    }

    pub(crate) fn productive_with(&self, batch: &mut Batch, defer_new: bool) -> Result<(), String> {
        self.productive_choice(batch, defer_new, None)
    }

    pub(crate) fn productive_choice(
        &self,
        batch: &mut Batch,
        defer_new: bool,
        preferred: Option<DefinitionId>,
    ) -> Result<(), String> {
        let mut requests: Vec<_> = self
            .state
            .processes
            .values()
            .filter(|p| p.status == Status::Active)
            .map(|p| Request {
                agent: p.operator,
                definition: p.definition,
                existing: Some(p.id),
                need: p.goal,
            })
            .collect();
        for participant in self.sorted_participants() {
            if self.state.terminal.contains_key(&participant.agent) || defer_new {
                continue;
            }
            let mut selected = BTreeSet::new();
            for need in Self::sorted_needs(participant) {
                let (candidate, reason) = self.plan(participant, need, preferred);
                if let Some(definition) = candidate {
                    // A joint-output producer requested by two needs starts once.
                    if selected.insert(definition) {
                        let count = if self
                            .world
                            .pool_market
                            .as_ref()
                            .is_some_and(|c| c.definition == definition)
                        {
                            crate::pool_market::demand(&self.world, &self.state, participant.agent)?
                                .0
                        } else {
                            1
                        };
                        if count as usize > self.effect_limit.saturating_sub(requests.len()) {
                            return Err("productive request capacity exceeded".into());
                        }
                        for _ in 0..count {
                            requests.push(Request {
                                agent: participant.agent,
                                definition,
                                existing: None,
                                need: Some(need.resource),
                            });
                        }
                    }
                } else {
                    batch.receipts.push(Receipt {
                        agent: participant.agent,
                        need: Some(need.resource),
                        definition: None,
                        reason,
                        requested: 0,
                        allocated: 0,
                        completed: 0,
                    });
                }
            }
        }
        if !defer_new {
            for order in &self.world.activities.orders {
                if crate::activities::wants(&self.world, &self.state, order)
                    && !requests.iter().any(|r| {
                        r.agent == order.agent
                            && r.definition == order.definition
                            && r.existing.is_none()
                    })
                {
                    requests.push(Request {
                        agent: order.agent,
                        definition: order.definition,
                        existing: None,
                        need: None,
                    });
                }
            }
        }
        for start in self
            .world
            .scheduled_starts
            .iter()
            .filter(|s| s.month == self.state.month)
        {
            requests.push(Request {
                agent: start.agent,
                definition: start.definition,
                existing: None,
                need: None,
            });
        }
        requests.sort_by_key(|r| {
            let need_rank = self
                .world
                .participants
                .iter()
                .find(|p| p.agent == r.agent)
                .and_then(|p| p.needs.iter().find(|n| Some(n.resource) == r.need))
                .map(|n| n.priority)
                .unwrap_or_else(|| {
                    self.world
                        .activities
                        .orders
                        .iter()
                        .find(|o| o.agent == r.agent && o.definition == r.definition)
                        .map(|o| o.priority)
                        .unwrap_or(u32::MAX)
                });
            let (first, second) = match self.world.priority {
                Priority::ContinuingFirst => (u32::from(r.existing.is_none()), need_rank),
                Priority::NewFirst => (u32::from(r.existing.is_some()), need_rank),
                Priority::NeedFirst => (need_rank, u32::from(r.existing.is_none())),
                Priority::NeedFirstFor(resource) => {
                    (u32::from(r.need != Some(resource)), need_rank)
                }
                Priority::ConsequenceAware => (u32::from(r.existing.is_none()), need_rank),
            };
            (
                u32::from(preferred != Some(r.definition)),
                first,
                second,
                r.agent,
                r.need,
                r.definition,
                r.existing,
            )
        });
        self.resolve(requests, batch)
    }

    fn sorted_needs(participant: &Participant) -> Vec<&Requirement> {
        let mut needs: Vec<_> = participant.needs.iter().collect();
        needs.sort_by_key(|n| (n.priority, n.resource));
        needs
    }

    pub(crate) fn plan(
        &self,
        participant: &Participant,
        need: &Requirement,
        preferred: Option<DefinitionId>,
    ) -> (Option<DefinitionId>, Reason) {
        if need.quantity == 0 {
            return (None, Reason::NoDeficit);
        }
        if crate::substitution::recipes(&self.world, need.resource).len() > 1 {
            return self.plan_substitutes(participant, need, preferred);
        }
        let mut consumers: Vec<_> =
            crate::opportunities::processes(&self.world, &self.state, participant.agent)
                .into_iter()
                .filter(|d| {
                    d.enabled
                        && d.execution == Execution::Consumption
                        && d.outputs[0].resource == need.resource
                })
                .collect();
        consumers.sort_by_key(|d| d.id);
        if consumers.is_empty() {
            return (None, Reason::NoKnownChain);
        }
        let mut candidates = Vec::new();
        let mut any_deficit = false;
        let mut any_chain = false;
        let mut enabled_chain = false;
        let mut active_help = false;
        for consumer in consumers {
            let input = &consumer.stages[0].entry_inputs[0];
            let output = &consumer.outputs[0];
            let lots = (i128::from(need.quantity) + i128::from(output.quantity) - 1)
                / i128::from(output.quantity);
            let demand = lots * i128::from(input.quantity);
            let has_claim = commitments::active(&self.world, &self.state)
                .any(|a| a.debtor == participant.agent && a.payment.resource == input.resource);
            // A dated claim needs a replenishment window including the month
            // after completion: waiting another month may miss its opening Due.
            // Uncontracted needs retain the configured buffer horizon.
            let horizon = if has_claim {
                self.world
                    .definitions
                    .iter()
                    .filter(|d| {
                        d.enabled
                            && d.execution == Execution::Productive
                            && d.outputs.iter().any(|a| a.resource == input.resource)
                    })
                    .map(|d| u64::from(d.duration()) + 1)
                    .max()
                    .unwrap_or(0)
                    .max(u64::from(self.world.horizon))
            } else {
                u64::from(self.world.horizon)
            };
            let claims = commitments::projected_claims(
                &self.world,
                &self.state,
                participant.agent,
                input.resource,
                horizon,
            );
            let base = self.projected_shortfall(
                participant.agent,
                input.resource,
                demand,
                None,
                horizon,
                &claims,
            );
            if base == 0 {
                // Credit active output only if opening stock alone would leave
                // a shortfall for this specific need's input. An unrelated active
                // process must not explain a full fuel buffer, for example.
                active_help |= i128::from(self.state.balance(participant.agent, input.resource))
                    < demand * i128::from(self.world.horizon);
                continue;
            }
            any_deficit = true;
            for producer in
                crate::opportunities::processes(&self.world, &self.state, participant.agent)
            {
                if producer.execution != Execution::Productive
                    || !producer
                        .outputs
                        .iter()
                        .any(|a| a.resource == input.resource)
                {
                    continue;
                }
                any_chain = true;
                if !producer.enabled {
                    continue;
                }
                enabled_chain = true;
                if let Some(kind) = producer.asset_kind {
                    let relevant: Vec<_> = self
                        .world
                        .rights
                        .iter()
                        .filter(|r| {
                            crate::commitments::holder(&self.world, &self.state, r)
                                == Some(participant.agent)
                                && self
                                    .world
                                    .assets
                                    .iter()
                                    .any(|a| a.id == r.asset && a.kind == kind)
                        })
                        .collect();
                    if !relevant.is_empty()
                        && !relevant.iter().any(|r| {
                            crate::commitments::output_owner(&self.world, &self.state, r)
                                == Some(participant.agent)
                        })
                    {
                        continue;
                    }
                }
                let with = self.projected_shortfall(
                    participant.agent,
                    input.resource,
                    demand,
                    Some(producer),
                    horizon,
                    &claims,
                );
                if with < base {
                    candidates.push((
                        (base - with) * i128::from(output.quantity) / i128::from(input.quantity),
                        producer.id,
                    ));
                }
            }
        }
        candidates.sort_by_key(|(score, id)| (std::cmp::Reverse(*score), *id));
        // Prefer a feasible candidate. If none is feasible, resolve the best wish
        // anyway to retain the precise missing-stock/right/capacity receipt.
        let empty = BTreeMap::new();
        let next_id = self
            .state
            .processes
            .keys()
            .next_back()
            .copied()
            .unwrap_or(0)
            .saturating_add(1);
        for &(_, id) in &candidates {
            let request = Request {
                agent: participant.agent,
                definition: id,
                existing: None,
                need: Some(need.resource),
            };
            if self
                .prepare(
                    &request,
                    next_id,
                    &self.state.balances,
                    &empty,
                    &crate::storage::usage(&self.world, &self.state.balances),
                )
                .is_ok()
            {
                return (Some(id), Reason::Selected);
            }
        }
        if let Some(&(_, id)) = candidates.first() {
            return (Some(id), Reason::Selected);
        }
        (
            None,
            if !any_deficit {
                if active_help {
                    Reason::ActiveOutputSufficient
                } else {
                    Reason::NoDeficit
                }
            } else if enabled_chain {
                Reason::NoBeneficialProcess
            } else if any_chain {
                Reason::Disabled
            } else {
                Reason::NoKnownChain
            },
        )
    }

    fn plan_substitutes(
        &self,
        participant: &Participant,
        need: &Requirement,
        preferred: Option<DefinitionId>,
    ) -> (Option<DefinitionId>, Reason) {
        let inputs: BTreeSet<_> = crate::substitution::recipes(&self.world, need.resource)
            .iter()
            .map(|d| d.stages[0].entry_inputs[0].resource)
            .collect();
        let producers: Vec<_> = self
            .world
            .definitions
            .iter()
            .filter(|d| {
                d.execution == Execution::Productive
                    && d.enabled
                    && d.outputs.iter().any(|a| inputs.contains(&a.resource))
            })
            .collect();
        let contracted = commitments::active(&self.world, &self.state)
            .any(|a| a.debtor == participant.agent && inputs.contains(&a.payment.resource));
        let horizon = if contracted {
            producers
                .iter()
                .map(|d| d.duration() + 1)
                .max()
                .unwrap_or(0)
                .max(self.world.horizon)
        } else {
            self.world.horizon
        };
        let base = self.substitute_shortfall(participant, need.resource, None, horizon);
        if base == 0 {
            return (None, Reason::NoDeficit);
        }
        let mut candidates: Vec<_> = producers
            .iter()
            .filter(|d| {
                d.asset_kind.is_none_or(|kind| {
                    let relevant: Vec<_> = self
                        .world
                        .rights
                        .iter()
                        .filter(|r| {
                            crate::commitments::holder(&self.world, &self.state, r)
                                == Some(participant.agent)
                                && self
                                    .world
                                    .assets
                                    .iter()
                                    .any(|a| a.id == r.asset && a.kind == kind)
                        })
                        .collect();
                    relevant.is_empty()
                        || relevant.iter().any(|r| {
                            crate::commitments::output_owner(&self.world, &self.state, r)
                                == Some(participant.agent)
                        })
                })
            })
            .filter_map(|d| {
                let with = self.substitute_shortfall(participant, need.resource, Some(d), horizon);
                (with < base).then_some((base - with, d.id))
            })
            .collect();
        candidates
            .sort_by_key(|(gain, id)| (preferred != Some(*id), std::cmp::Reverse(*gain), *id));
        let empty = BTreeMap::new();
        let next = self
            .state
            .processes
            .keys()
            .next_back()
            .copied()
            .unwrap_or(0)
            .saturating_add(1);
        for &(_, id) in &candidates {
            let r = Request {
                agent: participant.agent,
                definition: id,
                existing: None,
                need: Some(need.resource),
            };
            if self
                .prepare(
                    &r,
                    next,
                    &self.state.balances,
                    &empty,
                    &crate::storage::usage(&self.world, &self.state.balances),
                )
                .is_ok()
            {
                return (Some(id), Reason::Selected);
            }
        }
        candidates
            .first()
            .map(|(_, id)| (Some(*id), Reason::Selected))
            .unwrap_or((
                None,
                if producers.is_empty() {
                    Reason::NoKnownChain
                } else {
                    Reason::NoBeneficialProcess
                },
            ))
    }

    // A shared hypothetical stock budget prevents aliases and overlapping needs
    // from counting one edible stock more than once. Actual feasibility remains
    // the responsibility of the ordinary rollout, not this candidate heuristic.
    fn substitute_shortfall(
        &self,
        participant: &Participant,
        target: ResourceId,
        extra: Option<&ProcessDefinition>,
        horizon: u32,
    ) -> i128 {
        let mut stocks = crate::substitution::stocks(&self.state, participant.agent);
        let claims: BTreeMap<_, _> = self
            .world
            .resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Stock)
            .map(|r| {
                (
                    r.id,
                    commitments::projected_claims(
                        &self.world,
                        &self.state,
                        participant.agent,
                        r.id,
                        u64::from(horizon),
                    ),
                )
            })
            .collect();
        let target_inputs: BTreeSet<_> = crate::substitution::recipes(&self.world, target)
            .iter()
            .map(|d| d.stages[0].entry_inputs[0].resource)
            .collect();
        let mut deficit = 0;
        for month in self.state.month..self.state.month + horizon {
            let mut outputs = Vec::new();
            for p in self.state.processes.values().filter(|p| {
                p.status == Status::Active
                    && p.beneficiary == participant.agent
                    && p.reserved_through == month
            }) {
                outputs.extend(crate::exchange::retained_outputs(
                    &self.world,
                    &self.state,
                    participant.agent,
                    self.world.definition(p.definition),
                    Some(p),
                ));
            }
            if let Some(d) = extra
                && month == self.state.month + d.duration() - 1
            {
                outputs.extend(crate::exchange::retained_outputs(
                    &self.world,
                    &self.state,
                    participant.agent,
                    d,
                    None,
                ));
            }
            for a in outputs {
                *stocks.entry(a.resource).or_default() += i128::from(a.quantity);
            }
            for (resource, dues) in &claims {
                let owed = dues.get(&u64::from(month)).copied().unwrap_or(0);
                let stock = stocks.entry(*resource).or_default();
                let paid = (*stock).min(owed);
                *stock -= paid;
                if target_inputs.contains(resource) {
                    deficit += owed - paid;
                }
            }
            // Preserve upcoming payment stocks where alternative food can cover consumption.
            let earmarks = claims
                .iter()
                .map(|(r, dues)| {
                    (
                        *r,
                        dues.iter()
                            .filter(|(m, _)| **m > u64::from(month))
                            .map(|(_, q)| *q)
                            .sum(),
                    )
                })
                .collect();
            for need in Self::sorted_needs(participant) {
                let (_, unmet) = crate::substitution::allocate(
                    &self.world,
                    need.resource,
                    i128::from(need.quantity),
                    &mut stocks,
                    &earmarks,
                );
                if need.resource == target {
                    deficit += unmet;
                }
            }
        }
        deficit
    }

    fn projected_shortfall(
        &self,
        agent: AgentId,
        resource: ResourceId,
        demand: i128,
        extra: Option<&ProcessDefinition>,
        horizon: u64,
        claims: &BTreeMap<u64, i128>,
    ) -> i128 {
        let mut stock = i128::from(self.state.balance(agent, resource));
        let mut deficit = 0;
        for month in u64::from(self.state.month)..u64::from(self.state.month) + horizon {
            for p in self.state.processes.values().filter(|p| {
                p.status == Status::Active
                    && p.beneficiary == agent
                    && u64::from(p.reserved_through) == month
            }) {
                for a in crate::exchange::retained_outputs(
                    &self.world,
                    &self.state,
                    agent,
                    self.world.definition(p.definition),
                    Some(p),
                ) {
                    if a.resource == resource {
                        stock += i128::from(a.quantity);
                    }
                }
            }
            if let Some(d) = extra
                && month == u64::from(self.state.month) + u64::from(d.duration()) - 1
            {
                for a in crate::exchange::retained_outputs(&self.world, &self.state, agent, d, None)
                {
                    if a.resource == resource {
                        stock += i128::from(a.quantity);
                    }
                }
            }
            // Candidate pressure only, not a second payment resolver. Actual
            // priority, arrears and same-month harvest settlement stay in the
            // ordinary forecast/commit pipeline.
            let required = demand + claims.get(&month).copied().unwrap_or(0);
            let used = stock.min(required);
            stock -= used;
            deficit += required - used;
        }
        deficit
    }

    fn consume(&self, batch: &mut Batch) -> Result<(), String> {
        let mut requests = Vec::new();
        for participant in self.sorted_participants() {
            if self.state.terminal.contains_key(&participant.agent) {
                continue;
            }
            let mut stocks = crate::substitution::stocks(&self.state, participant.agent);
            let earmarks =
                crate::substitution::earmarks(&self.world, &self.state, participant.agent);
            for need in Self::sorted_needs(participant) {
                let remaining =
                    (need.quantity - self.state.balance(participant.agent, need.resource)).max(0);
                let (mut lots, unmet) = crate::substitution::allocate(
                    &self.world,
                    need.resource,
                    i128::from(remaining),
                    &mut stocks,
                    &earmarks,
                );
                // Retain unmet requests and their missing-stock receipts.
                if unmet > 0
                    && let Some(d) =
                        crate::substitution::recipes(&self.world, need.resource).first()
                {
                    let output = i128::from(d.outputs[0].quantity);
                    lots.push((d.id, (unmet + output - 1) / output));
                }
                for (definition, count) in lots {
                    if count > self.effect_limit.saturating_sub(requests.len()) as i128 {
                        return Err("consumption request capacity exceeded".into());
                    }
                    for _ in 0..count {
                        requests.push(Request {
                            agent: participant.agent,
                            definition,
                            existing: None,
                            need: Some(need.resource),
                        });
                    }
                }
            }
        }
        self.resolve(requests, batch)
    }

    fn prepare(
        &self,
        r: &Request,
        id: u64,
        budget: &BTreeMap<Account, i32>,
        reserved: &BTreeMap<AssetId, u64>,
        stored: &BTreeMap<AgentId, i128>,
    ) -> Result<(ProcessInstance, Option<crate::equipment::TechniqueUse>), Reason> {
        let d = self.world.definition(r.definition);
        if !crate::opportunities::permits(
            &self.world,
            &self.state,
            r.agent,
            crate::opportunities::Action::Process(d.id),
        ) {
            return Err(Reason::NotPermitted);
        }
        if self.state.terminal.contains_key(&r.agent) {
            return Err(Reason::Inactive);
        }
        if !d.enabled && r.existing.is_none() {
            return Err(Reason::Disabled);
        }
        let mut p = if let Some(existing) = r.existing {
            self.state.processes[&existing].clone()
        } else {
            ProcessInstance {
                id,
                definition: d.id,
                operator: r.agent,
                beneficiary: r.agent,
                goal: r.need,
                asset: None,
                right: None,
                start: self.state.month,
                reserved_through: self.state.month + d.duration() - 1,
                stage: 0,
                elapsed: 0,
                status: Status::Active,
            }
        };
        if let Some(kind) = d.asset_kind {
            if p.asset.is_none() {
                let mut rights: Vec<_> = self
                    .world
                    .rights
                    .iter()
                    .filter(|right| {
                        crate::commitments::holder(&self.world, &self.state, right) == Some(r.agent)
                            && right.from <= self.state.month
                            && right.through >= self.state.month
                            && self
                                .world
                                .assets
                                .iter()
                                .any(|a| a.id == right.asset && a.kind == kind)
                    })
                    .collect();
                rights.sort_by_key(|right| (right.asset, right.id));
                if rights.is_empty() {
                    return Err(Reason::MissingRight);
                }
                let had_unpaid = rights.iter().any(|r| {
                    crate::commitments::active(&self.world, &self.state).any(|a| {
                        a.right == r.id
                            && a.contract(&self.world, &self.state).is_ok_and(|contract| {
                                !contract.evaluate(self.state.month).breaches.is_empty()
                            })
                    })
                });
                rights.retain(|right| {
                    crate::commitments::can_start(&self.world, &self.state, right.id)
                });
                if rights.is_empty() {
                    return Err(if had_unpaid {
                        Reason::UnpaidObligation
                    } else {
                        Reason::MissingRight
                    });
                }
                rights.retain(|right| right.through >= p.reserved_through);
                if rights.is_empty() {
                    return Err(Reason::RightTooShort);
                }
                rights.retain(|right| {
                    self.world.activities.shared_sites.contains(&d.id)
                        || (!reserved.contains_key(&right.asset)
                            && !self.state.processes.values().any(|active| {
                                active.status == Status::Active
                                    && active.asset == Some(right.asset)
                                    && !self
                                        .world
                                        .activities
                                        .shared_sites
                                        .contains(&active.definition)
                            }))
                });
                let right = rights.first().ok_or(Reason::Occupied)?;
                p.asset = Some(right.asset);
                p.right = Some(right.id);
                p.beneficiary = crate::commitments::output_owner(&self.world, &self.state, right)
                    .ok_or(Reason::MissingRight)?;
            } else if !self.world.rights.iter().any(|right| {
                Some(right.id) == p.right
                    && crate::commitments::holder(&self.world, &self.state, right)
                        == Some(p.operator)
                    && right.from <= self.state.month
                    && right.through >= p.reserved_through
            }) {
                return Err(Reason::MissingRight);
            }
        }
        let bindings = crate::activities::bindings(&self.world, &self.state, &p, reserved)
            .map_err(|_| Reason::MissingEquipment)?;
        // Optional stage equipment is reserved alongside the process's plot.
        // Prefer lower service cost, with manual execution winning equal-cost ties.
        let mut variants = vec![(
            d.stages[p.stage]
                .monthly_services
                .iter()
                .map(|a| i64::from(a.quantity))
                .sum::<i64>(),
            None,
        )];
        for technique in self.world.techniques.iter().filter(|t| {
            t.definition == d.id
                && t.stage == p.stage
                && crate::equipment::eligible(t, &self.state, p.operator)
        }) {
            if technique.equipment_kind.is_none() {
                variants.push((
                    technique
                        .services
                        .iter()
                        .map(|a| i64::from(a.quantity))
                        .sum(),
                    Some(crate::equipment::TechniqueUse {
                        technique: technique.id,
                        asset: None,
                    }),
                ));
            }
            for asset in self.state.equipment.values().filter(|a| {
                a.owner == p.operator
                    && Some(a.kind) == technique.equipment_kind
                    && a.remaining_uses >= technique.wear
                    && a.last_used_month != Some(self.state.month)
                    && !reserved.contains_key(&a.id)
                    && !bindings.contains(&a.id)
                    && crate::activities::attached_access(&self.world, &self.state, a, p.asset)
            }) {
                variants.push((
                    technique
                        .services
                        .iter()
                        .map(|a| i64::from(a.quantity))
                        .sum(),
                    Some(crate::equipment::TechniqueUse {
                        technique: technique.id,
                        asset: Some(asset.id),
                    }),
                ));
            }
        }
        variants.sort_by_key(|(cost, usage)| {
            (
                *cost,
                usage
                    .as_ref()
                    .map(|u| (u.asset.is_some(), u.technique, u.asset)),
            )
        });
        let mut failure = Reason::InsufficientCapacity;
        for (_, usage) in variants {
            let services = usage
                .as_ref()
                .map(|u| crate::equipment::services(&self.world, u));
            let mut feasible = true;
            for (resource, required) in costs(d, &p, services) {
                let available = i64::from(
                    budget
                        .get(&crate::pools::input_account(
                            &self.world,
                            d.id,
                            r.agent,
                            resource,
                        ))
                        .copied()
                        .unwrap_or(0),
                );
                if required > available {
                    let kind = self
                        .world
                        .resources
                        .iter()
                        .find(|v| v.id == resource)
                        .unwrap()
                        .kind;
                    failure = if kind == ResourceKind::Capacity {
                        Reason::InsufficientCapacity
                    } else {
                        Reason::MissingStock
                    };
                    feasible = false;
                    break;
                }
            }
            if feasible {
                let mut effects: Vec<_> = costs(d, &p, services)
                    .into_iter()
                    .map(|(resource, quantity)| Effect {
                        account: crate::pools::input_account(&self.world, d.id, r.agent, resource),
                        delta: -(quantity as i32),
                    })
                    .collect();
                if p.stage + 1 == d.stages.len() && p.elapsed + 1 == d.stages[p.stage].months {
                    let Ok((outputs, _)) =
                        crate::exchange::outputs(&self.world, &self.state, &p, usage.as_ref())
                    else {
                        failure = Reason::InsufficientStorage;
                        continue;
                    };
                    effects.extend(outputs);
                }
                let Ok(stored_effects) =
                    crate::households::with_contributions(&self.world, &self.state, &effects)
                else {
                    return Err(Reason::InsufficientStorage);
                };
                if !crate::storage::fits(&self.world, stored, &stored_effects) {
                    failure = Reason::InsufficientStorage;
                    continue;
                }
                return Ok((p, usage));
            }
        }
        Err(failure)
    }

    fn resolve(&self, requests: Vec<Request>, batch: &mut Batch) -> Result<(), String> {
        let requests = requests
            .into_iter()
            .map(|r| crate::offers::Request {
                offer: crate::offers::Id::Process(r.definition),
                agent: r.agent,
                continuing: r.existing,
                need: r.need,
            })
            .collect::<Vec<_>>();
        crate::offers::resolve(self, &requests, batch)
    }

    pub(crate) fn resolve_work(
        &self,
        requests: Vec<Request>,
        batch: &mut Batch,
    ) -> Result<(), String> {
        if self.world.pool_market.is_some() && self.state.phase == Phase::Productive {
            crate::pool_market::resolve(self, requests, batch)
        } else {
            self.resolve_work_unallocated(requests, batch)
        }
    }

    pub(crate) fn resolve_work_unallocated(
        &self,
        requests: Vec<Request>,
        batch: &mut Batch,
    ) -> Result<(), String> {
        let mut budget = self.state.balances.clone();
        let mut stored = crate::storage::usage(&self.world, &self.state.balances);
        let mut reserved = BTreeMap::new();
        let mut next_id = self
            .state
            .processes
            .keys()
            .next_back()
            .copied()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("instance ID overflow")?;
        for r in requests {
            let d = self.world.definition(r.definition);
            let stage_index = r
                .existing
                .map(|id| self.state.processes[&id].stage)
                .unwrap_or(0);
            let requested = d.stages[stage_index]
                .monthly_services
                .iter()
                .try_fold(0i32, |n, a| n.checked_add(a.quantity))
                .ok_or("service total overflow")?;
            match self.prepare(&r, next_id, &budget, &reserved, &stored) {
                Err(reason) => {
                    if let Some(id) = r.existing {
                        let before = self.state.processes[&id].clone();
                        let mut after = before.clone();
                        crate::agreements::ProductionTerms::from_definition(d).fail(&mut after);
                        batch.transactions.push(Transaction {
                            technique_use: None,
                            trade: None,
                            stock_trade: None,
                            forward: None,
                            delivery: None,
                            royalty: None,
                            cause: format!("abort {}: {reason:?}", d.name),
                            effects: Vec::new(),
                            process: Some(ProcessChange {
                                before: Some(before),
                                after,
                            }),
                        });
                    }
                    batch.receipts.push(Receipt {
                        agent: r.agent,
                        need: r.need,
                        definition: Some(d.id),
                        reason,
                        requested,
                        allocated: 0,
                        completed: 0,
                    });
                }
                Ok((mut p, technique_use)) => {
                    let services = technique_use
                        .as_ref()
                        .map(|u| crate::equipment::services(&self.world, u));
                    let requested = services
                        .unwrap_or(&d.stages[p.stage].monthly_services)
                        .iter()
                        .try_fold(0i32, |n, a| n.checked_add(a.quantity))
                        .ok_or("service total overflow")?;
                    if let Some(asset) = technique_use.as_ref().and_then(|u| u.asset) {
                        reserved.insert(asset, p.id);
                    }
                    let before = r.existing.map(|id| self.state.processes[&id].clone());
                    if before.is_none() {
                        next_id = next_id.checked_add(1).ok_or("instance ID overflow")?;
                    }
                    for asset in
                        crate::activities::bindings(&self.world, &self.state, &p, &reserved)?
                    {
                        reserved.insert(asset, p.id);
                    }
                    if let Some(asset) = p.asset
                        && !self.world.activities.shared_sites.contains(&d.id)
                    {
                        reserved.insert(asset, p.id);
                    }
                    let mut effects = Vec::new();
                    for (resource, cost) in costs(d, &p, services) {
                        let cost = i32::try_from(cost).map_err(|_| "process cost overflow")?;
                        let account =
                            crate::pools::input_account(&self.world, d.id, r.agent, resource);
                        *budget.entry(account).or_default() -= cost;
                        effects.push(Effect {
                            account,
                            delta: -cost,
                        });
                    }
                    let mut royalty = None;
                    let stage_name = d.stages[p.stage].name.clone();
                    p.elapsed += 1;
                    if p.elapsed == d.stages[p.stage].months {
                        if p.stage + 1 == d.stages.len() {
                            p.status = Status::Completed;
                            let (outputs, receipt) = crate::exchange::outputs(
                                &self.world,
                                &self.state,
                                &p,
                                technique_use.as_ref(),
                            )?;
                            effects.extend(outputs);
                            royalty = receipt;
                        } else {
                            p.stage += 1;
                            p.elapsed = 0;
                        }
                    }
                    crate::storage::apply(
                        &self.world,
                        &mut stored,
                        &crate::households::with_contributions(&self.world, &self.state, &effects)?,
                    );
                    batch.transactions.push(Transaction {
                        technique_use,
                        trade: None,
                        stock_trade: None,
                        forward: None,
                        delivery: None,
                        royalty,
                        cause: format!("{} / {stage_name}", d.name),
                        effects,
                        process: Some(ProcessChange { before, after: p }),
                    });
                    // Newly generated output is deliberately not added to budget.
                    batch.receipts.push(Receipt {
                        agent: r.agent,
                        need: r.need,
                        definition: Some(d.id),
                        reason: Reason::Selected,
                        requested,
                        allocated: requested,
                        completed: requested,
                    });
                }
            }
        }
        Ok(())
    }

    fn month_reports(&self) -> Vec<MonthReport> {
        self.sorted_participants()
            .into_iter()
            .map(|p| MonthReport {
                accepted_agreements: self
                    .state
                    .accepted_agreements
                    .iter()
                    .filter(|(_, a)| a.debtor == p.agent)
                    .map(|(k, v)| (*k, v.clone()))
                    .collect(),
                obligations: self
                    .state
                    .obligations
                    .iter()
                    .filter(|(_, o)| {
                        crate::commitments::active(&self.world, &self.state)
                            .any(|a| a.id == o.agreement && a.debtor == p.agent)
                    })
                    .map(|(k, v)| (*k, v.clone()))
                    .collect(),
                month: self.state.month,
                agent: p.agent,
                conditions: BTreeMap::new(),
                terminal: None,
                practice: self
                    .state
                    .practice
                    .iter()
                    .filter(|((a, _), _)| *a == p.agent)
                    .map(|(k, v)| (*k, *v))
                    .collect(),
                equipment: self
                    .state
                    .equipment
                    .iter()
                    .filter(|(_, a)| a.owner == p.agent)
                    .map(|(id, a)| (*id, a.clone()))
                    .collect(),
                balances: self
                    .state
                    .balances
                    .iter()
                    .filter(|((owner, _), _)| *owner == p.agent)
                    .map(|((_, resource), value)| (*resource, *value))
                    .collect(),
                needs: p
                    .needs
                    .iter()
                    .map(|need| {
                        let desired = if self.state.terminal.contains_key(&p.agent) {
                            0
                        } else {
                            need.quantity
                        };
                        let fulfilled = self.state.balance(p.agent, need.resource).min(desired);
                        (
                            need.resource,
                            NeedReport {
                                desired,
                                fulfilled,
                                deficit: desired - fulfilled,
                            },
                        )
                    })
                    .collect(),
            })
            .collect()
    }
}

fn costs(
    d: &ProcessDefinition,
    p: &ProcessInstance,
    services: Option<&[Amount]>,
) -> BTreeMap<ResourceId, i64> {
    let stage = &d.stages[p.stage];
    let mut result = BTreeMap::new();
    for a in services.unwrap_or(&stage.monthly_services) {
        *result.entry(a.resource).or_default() += i64::from(a.quantity);
    }
    if p.elapsed == 0 {
        for a in &stage.entry_inputs {
            *result.entry(a.resource).or_default() += i64::from(a.quantity);
        }
    }
    result
}
