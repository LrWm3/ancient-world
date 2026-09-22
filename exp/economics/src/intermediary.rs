//! Bounded experiment: independent conditional forecasts, joint input reservation,
//! then one fallback decision against the retained work. Uses the existing phases.
use crate::{
    access_expectations::{Estimate, Memory, Mode, Observation},
    allocation::{Claim, Context, Outcome, Policy},
    compute::Backend,
    model::*,
    planning::Score,
    resolution::{self, Mechanism, Resolution},
    simulation::{Request, Simulation},
};
use std::collections::{BTreeMap, BTreeSet};

pub const MAKE_TOOL: DefinitionId = 700;
pub const TOOL_KIND: u32 = 700;
pub const RUN_MONTHS: u32 = 12;
const FORECAST_MONTHS: u32 = 12;
const MAX_PARTICIPANTS: usize = 4;
const MAX_ACTIONS: usize = 16;
const RESOLUTION_SCOPE: u64 = 0;
const TOOL_WOOD: i32 = 2;
const TOOL_LABOR: i32 = 2;
const TOOL_HARVEST_LABOR: i32 = 1;
const TOOL_LIFETIME: u32 = 2;
const TOOL_MULTIPLIER: u32 = 2;
const OPENING_WOOD: i32 = 3;
const WOOD_REGENERATION: i32 = 1;
const OPENING_FOOD: i32 = 5;
const OPENING_FUEL: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection {
    pub action: Option<DefinitionId>,
    pub score: Score,
    pub immediate_buffer_gap: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub agent: AgentId,
    /// Conditional forecasts: none of these reserve resources or promise delivery.
    pub alternatives: Vec<Projection>,
    pub selected: Option<DefinitionId>,
    pub fallback: Option<DefinitionId>,
    pub fallback_alternatives: Vec<Projection>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub expectations: BTreeMap<(AgentId, Account), Estimate>,
    pub month: u32,
    pub policy: Policy,
    pub decisions: Vec<Decision>,
    pub resolution: Resolution,
}
#[derive(Clone, Debug)]
pub struct Experiment {
    pub access_mode: Mode,
    pub access_memory: Memory,
    pub simulation: Simulation,
    pub policy: Policy,
    pub seed: u64,
    pub rounds: Vec<Round>,
}
impl Experiment {
    pub fn step(&mut self) -> Result<(), String> {
        if self.simulation.state.phase != Phase::Acquire {
            if self.simulation.state.phase == Phase::Productive {
                // Stage both physical settlement and learning so an observation
                // error cannot leave the experiment half advanced.
                let mut next = self.clone();
                next.simulation.step()?;
                let round = next.rounds.last().ok_or("missing intermediary decision")?;
                let batch = next.simulation.ledger.last().unwrap();
                let rows = observations(&next.simulation.world, round, batch)?;
                next.access_memory.record(batch.month, rows)?;
                *self = next;
                return Ok(());
            }
            return self.simulation.step();
        }
        let (batch, round) = prepare_with_access(
            &self.simulation,
            self.policy,
            self.seed,
            self.access_mode,
            &self.access_memory,
        )?;
        crate::settlement::commit(
            &self.simulation.world,
            &mut self.simulation.state,
            &batch,
            self.simulation.backend,
            self.simulation.effect_limit,
        )?;
        self.simulation.ledger.push(batch);
        self.rounds.push(round);
        Ok(())
    }
    pub fn run_months(&mut self, months: u32) -> Result<(), String> {
        let end = self
            .simulation
            .state
            .month
            .checked_add(months)
            .ok_or("month overflow")?;
        while self.simulation.state.month < end {
            self.step()?;
        }
        Ok(())
    }
}

fn work(sim: &Simulation, requests: &[Request]) -> Result<Batch, String> {
    let mut b = Batch::empty(&sim.state);
    sim.resolve_work_unallocated(requests.to_vec(), &mut b)?;
    Ok(b)
}
fn completed(b: &Batch, agent: AgentId, definition: DefinitionId) -> bool {
    b.transactions.iter().any(|t| {
        t.process.as_ref().is_some_and(|p| {
            p.before.is_none()
                && p.after.operator == agent
                && p.after.definition == definition
                && p.after.status != Status::Aborted
        })
    })
}
fn request(agent: AgentId, definition: DefinitionId) -> Request {
    Request {
        agent,
        definition,
        existing: None,
        need: None,
    }
}

/// One-step actions are discovered by their connection to needs or productive
/// techniques, not by farmer/tool-maker agent types or scenario resource IDs.
fn candidates(sim: &Simulation, agent: AgentId) -> Vec<DefinitionId> {
    let participant = sim
        .world
        .participants
        .iter()
        .find(|p| p.agent == agent)
        .unwrap();
    let mut ids = BTreeSet::new();
    for need in &participant.needs {
        if let Some(d) = sim.plan(participant, need, None).0 {
            ids.insert(d);
        }
    }
    for offer in crate::offers::discover(&sim.world, &sim.state, agent) {
        let crate::offers::Id::Process(id) = offer.id else {
            continue;
        };
        let Some(crate::activities::Outcome::Create(kind)) = sim.world.activities.outcomes.get(&id)
        else {
            continue;
        };
        if sim
            .state
            .equipment
            .values()
            .any(|a| a.owner == agent && a.kind == *kind && a.remaining_uses > 0)
        {
            continue;
        }
        let useful =
            sim.world
                .techniques
                .iter()
                .filter(|t| t.equipment_kind == Some(*kind))
                .any(|t| {
                    let producer = sim.world.definition(t.definition);
                    participant.needs.iter().any(|n| {
                        crate::substitution::recipes(&sim.world, n.resource)
                            .iter()
                            .any(|consumer| {
                                producer.outputs.iter().any(|o| {
                                    o.resource == consumer.stages[0].entry_inputs[0].resource
                                })
                            })
                    })
                });
        if useful && sim.world.definition(id).duration() == 1 {
            ids.insert(id);
        }
    }
    ids.into_iter().collect()
}

fn project(
    sim: &Simulation,
    agent: AgentId,
    requests: &[Request],
    expectations: &BTreeMap<(AgentId, Account), Estimate>,
) -> Result<(Score, u64), String> {
    let mut f = sim.clone();
    f.backend = Backend::Reference;
    f.world.competition = None;
    f.world.priority = Priority::ContinuingFirst;
    f.world.capacity_overrides.clear();
    f.world.scheduled_starts.clear();
    f.ledger.clear();
    f.reports.clear();
    let b = work(&f, requests)?;
    crate::settlement::commit(&f.world, &mut f.state, &b, f.backend, f.effect_limit)?;
    f.ledger.push(b);
    // Future shared access is an expectation. Other agents' unknown future
    // choices are not a reservation; first-boundary joint work is already fixed.
    f.world.participants.retain(|p| p.agent == agent);
    f.world.condition_rules.retain(|r| r.subject == agent);
    f.state.conditions.retain(|(a, _), _| *a == agent);
    f.state.processes.retain(|_, p| p.operator == agent);
    f.state.terminal.retain(|a, _| *a == agent);
    let immediate_buffer_gap = crate::planning::score(&f).buffer_gap;
    // Only future speculative access is discounted. Current candidate work was
    // settled above against the real opening stock, including retained grants.
    let mut flows = Vec::new();
    for pool in &f.world.pools {
        let estimate = expectations[&(agent, pool.account)];
        let mut carry = 0;
        let stock = f.state.balance(pool.account.0, pool.account.1);
        f.state
            .balances
            .insert(pool.account, estimate.portion(stock, &mut carry)?);
        flows.push((pool.account, pool.monthly_regeneration, estimate, carry));
    }
    let end = f
        .state
        .month
        .checked_add(FORECAST_MONTHS)
        .ok_or("forecast month overflow")?;
    while f.state.month < end {
        if f.state.phase == Phase::Open {
            for (account, regeneration, estimate, carry) in &mut flows {
                f.world
                    .pools
                    .iter_mut()
                    .find(|p| p.account == *account)
                    .unwrap()
                    .monthly_regeneration = estimate.portion(*regeneration, carry)?;
            }
        }
        f.step()?;
    }
    for b in &mut f.ledger {
        b.receipts.retain(|r| r.agent == agent);
    }
    let mut score = crate::planning::score(&f);
    score.broken_commitments = f
        .state
        .processes
        .values()
        .filter(|p| p.status == Status::Aborted)
        .count() as u64;
    Ok((score, immediate_buffer_gap))
}

fn choose(
    sim: &Simulation,
    agent: AgentId,
    base: &[Request],
    excluded: Option<DefinitionId>,
    expectations: &BTreeMap<(AgentId, Account), Estimate>,
) -> Result<(Option<DefinitionId>, Vec<Projection>), String> {
    let (score, immediate_buffer_gap) = project(sim, agent, base, expectations)?;
    let mut alternatives = vec![Projection {
        action: None,
        score,
        immediate_buffer_gap,
    }];
    let actions = candidates(sim, agent);
    if actions.len() > MAX_ACTIONS {
        return Err("intermediary action budget exceeded".into());
    }
    for id in actions.into_iter().filter(|id| Some(*id) != excluded) {
        let mut trial = base.to_vec();
        trial.push(request(agent, id));
        if !completed(&work(sim, &trial)?, agent, id) {
            continue;
        }
        let (score, immediate_buffer_gap) = project(sim, agent, &trial, expectations)?;
        alternatives.push(Projection {
            action: Some(id),
            score,
            immediate_buffer_gap,
        });
    }
    let best = alternatives
        .iter()
        .min_by_key(|p| (&p.score, p.immediate_buffer_gap, p.action))
        .unwrap()
        .action;
    Ok((best, alternatives))
}

pub fn prepare(sim: &Simulation, policy: Policy, seed: u64) -> Result<(Batch, Round), String> {
    prepare_with_access(sim, policy, seed, Mode::Optimistic, &Memory::default())
}

pub fn prepare_with_access(
    sim: &Simulation,
    policy: Policy,
    seed: u64,
    mode: Mode,
    memory: &Memory,
) -> Result<(Batch, Round), String> {
    if sim.world.participants.len() > MAX_PARTICIPANTS
        || sim.state.phase != Phase::Acquire
        || sim.world.pool_market.is_some()
        || !sim.world.households.is_empty()
        || sim.world.market.is_some()
        || !sim.world.activities.orders.is_empty()
    {
        return Err("intermediary experiment requires an unshared Acquire boundary without other market drivers".into());
    }
    let expectations = memory.snapshot(
        sim.state.month,
        sim.world.participants.iter().map(|p| p.agent),
        &sim.world
            .pools
            .iter()
            .map(|p| p.account)
            .collect::<Vec<_>>(),
        mode,
    );
    let mut preview = sim.clone();
    let mut batch = Batch::empty(&sim.state);
    crate::settlement::commit(
        &preview.world,
        &mut preview.state,
        &batch,
        Backend::Reference,
        preview.effect_limit,
    )?;
    let base: Vec<_> = preview
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
    let base_work = work(&preview, &base)?;
    let mut available: BTreeMap<_, u32> = preview
        .state
        .balances
        .iter()
        .map(|(a, q)| {
            Ok((
                *a,
                u32::try_from(*q).map_err(|_| "negative opening budget")?,
            ))
        })
        .collect::<Result<_, String>>()?;
    for e in base_work
        .transactions
        .iter()
        .flat_map(|t| &t.effects)
        .filter(|e| e.delta < 0)
    {
        let q = available.entry(e.account).or_default();
        *q = q
            .checked_sub(e.delta.unsigned_abs())
            .ok_or("base work exceeds budget")?;
    }
    let mut agents: Vec<_> = sim
        .world
        .participants
        .iter()
        .map(|p| p.agent)
        .filter(|a| !sim.state.terminal.contains_key(a))
        .collect();
    agents.sort_unstable();
    let mut decisions = Vec::new();
    let mut claims = Vec::new();
    for agent in agents {
        let (selected, alternatives) = choose(&preview, agent, &base, None, &expectations)?;
        if let Some(id) = selected {
            let d = sim.world.definition(id);
            let mut inputs = BTreeMap::<Account, u32>::new();
            for a in &d.stages[0].entry_inputs {
                let account = crate::pools::input_account(&sim.world, id, agent, a.resource);
                let q = inputs.entry(account).or_default();
                *q = q.checked_add(a.quantity as u32).ok_or("input overflow")?;
            }
            for a in &d.stages[0].monthly_services {
                let q = inputs.entry((agent, a.resource)).or_default();
                *q = q
                    .checked_add(a.quantity as u32)
                    .ok_or("capacity overflow")?;
            }
            claims.push(resolution::Request {
                claim: Claim {
                    id: u64::from(agent),
                    priority: 0,
                    requested: 1,
                    minimum: 1,
                },
                inputs,
            });
        }
        decisions.push(Decision {
            agent,
            alternatives,
            selected,
            fallback: None,
            fallback_alternatives: vec![],
        });
    }
    let (resolution, mut accepted) = resolution::resolve(
        Context {
            seed,
            pool: RESOLUTION_SCOPE,
            round: u64::from(sim.state.month),
        },
        &policy,
        Mechanism::ConditionalBundle,
        &available,
        &claims,
        &base,
        |trial, c, _| {
            let agent = c.id as AgentId;
            let id = decisions
                .iter()
                .find(|d| d.agent == agent)
                .unwrap()
                .selected
                .unwrap();
            trial.push(request(agent, id));
            if !completed(&work(&preview, trial)?, agent, id) {
                return Err("joint process prerequisites failed".into());
            }
            Ok(())
        },
    )?;
    // Same policy order as initial allocation. Retained work never runs again;
    // trial resolution only appends one feasible fallback before actual execution.
    for receipt in &resolution.receipts {
        if matches!(receipt.outcome, Outcome::Reserved(_)) {
            continue;
        }
        let d = decisions
            .iter_mut()
            .find(|d| u64::from(d.agent) == receipt.claim.id)
            .unwrap();
        let (action, alternatives) =
            choose(&preview, d.agent, &accepted, d.selected, &expectations)?;
        d.fallback = action;
        d.fallback_alternatives = alternatives;
        if let Some(id) = action {
            accepted.push(request(d.agent, id));
        }
    }
    batch.production_plan = Some(Box::new(work(&preview, &accepted)?));
    Ok((
        batch,
        Round {
            expectations,
            month: sim.state.month,
            policy,
            decisions,
            resolution,
        },
    ))
}

fn observations(world: &World, round: &Round, batch: &Batch) -> Result<Vec<Observation>, String> {
    if round.month != batch.month {
        return Err("stale access learning round".into());
    }
    let pools: BTreeSet<_> = world.pools.iter().map(|p| p.account).collect();
    let mut rows = Vec::new();
    for d in &round.decisions {
        let mut requested = BTreeMap::<Account, u32>::new();
        let receipt = round
            .resolution
            .receipts
            .iter()
            .find(|r| r.claim.id == u64::from(d.agent));
        // A permission/storage/labor rejection is not evidence of resource access.
        let initial = receipt
            .filter(|r| !matches!(r.outcome, Outcome::Rejected(_)))
            .and(d.selected);
        for id in [initial, d.fallback].into_iter().flatten() {
            let mut action = BTreeMap::<Account, u32>::new();
            for a in &world.definition(id).stages[0].entry_inputs {
                let account = crate::pools::input_account(world, id, d.agent, a.resource);
                if pools.contains(&account) {
                    *action.entry(account).or_default() += a.quantity as u32;
                }
            }
            for (account, q) in action {
                let previous = requested.entry(account).or_default();
                *previous = (*previous).max(q);
            }
        }
        for (account, requested) in requested {
            let received = batch
                .transactions
                .iter()
                .filter(|t| {
                    t.process.as_ref().is_some_and(|p| {
                        p.before.is_none()
                            && p.after.operator == d.agent
                            && [d.selected, d.fallback].contains(&Some(p.after.definition))
                    })
                })
                .flat_map(|t| &t.effects)
                .filter(|e| e.account == account && e.delta < 0)
                .map(|e| e.delta.unsigned_abs())
                .sum();
            rows.push(Observation {
                month: batch.month,
                agent: d.agent,
                account,
                requested,
                received,
            });
        }
    }
    Ok(rows)
}

pub fn scenario(backend: Backend) -> Result<Experiment, String> {
    use crate::scenario::*;
    let (world, state) = crate::competition::scenario(2, crate::competition::DEFAULT_SEED)?;
    let mut sim = Simulation::new(world, state, Backend::Reference)?;
    sim.run_months(1)?;
    sim.world.competition = None;
    sim.world.priority = Priority::ContinuingFirst;
    sim.world.definitions.push(ProcessDefinition {
        id: MAKE_TOOL,
        name: "make wooden cultivation tool".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "shape wood".into(),
            months: 1,
            entry_inputs: vec![Amount::new(RAW_WOOD, TOOL_WOOD)],
            monthly_services: vec![Amount::new(LABOR, TOOL_LABOR)],
        }],
        outputs: vec![],
    });
    sim.world.activities.kinds.insert(
        TOOL_KIND,
        crate::activities::DurableKind {
            name: "cultivation tool".into(),
            lifetime: TOOL_LIFETIME,
            attached: false,
            monthly_decay: 0,
        },
    );
    sim.world
        .activities
        .outcomes
        .insert(MAKE_TOOL, crate::activities::Outcome::Create(TOOL_KIND));
    let harvest = sim.world.definition(GROW).stages.len() - 1;
    sim.world.techniques.push(crate::equipment::Technique {
        id: TOOL_KIND,
        definition: GROW,
        stage: harvest,
        equipment_kind: Some(TOOL_KIND),
        competency: None,
        wear: 1,
        services: vec![Amount::new(LABOR, TOOL_HARVEST_LABOR)],
        output_multiplier: TOOL_MULTIPLIER,
    });
    sim.world.pool_inputs.push(crate::pools::PoolInput {
        definition: MAKE_TOOL,
        account: (STATE_AGENT, RAW_WOOD),
    });
    sim.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((
            crate::opportunities::PERSON_TYPE,
            crate::opportunities::Action::Process(MAKE_TOOL),
        ));
    let pool = sim
        .world
        .pools
        .iter_mut()
        .find(|p| p.account == (STATE_AGENT, RAW_WOOD))
        .unwrap();
    pool.capacity = OPENING_WOOD;
    pool.monthly_regeneration = WOOD_REGENERATION;
    sim.state
        .balances
        .insert((STATE_AGENT, RAW_WOOD), OPENING_WOOD);
    for p in &sim.world.participants {
        sim.state.balances.insert((p.agent, GRAIN), OPENING_FOOD);
        sim.state.balances.insert((p.agent, FUEL), OPENING_FUEL);
    }
    sim = Simulation::new(sim.world, sim.state, backend)?;
    Ok(Experiment {
        access_mode: Mode::Optimistic,
        access_memory: Memory::default(),
        simulation: sim,
        policy: Policy::StablePriority,
        seed: crate::competition::DEFAULT_SEED,
        rounds: vec![],
    })
}
