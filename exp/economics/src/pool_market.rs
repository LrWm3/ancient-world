//! Recurring environmental collection offers. Quantities are whole process lots;
//! private services and shared stock must be reserved together.
use crate::{
    allocation::{Claim, Context, Policy, Receipt},
    compute::Backend,
    model::*,
    offers,
    resolution::{self, Mechanism},
    simulation::{Request, Simulation},
};
use std::collections::BTreeMap;

pub const RUN_MONTHS: u32 = 36;
const WOOD_PER_LOT: i32 = 2;
const POOL_CAPACITY: i32 = 8;
const INITIAL_FUEL: i32 = 1;
const AMPLE_REGENERATION: i32 = 4;
const SUFFICIENT_REGENERATION: i32 = 2;
const SCARCE_REGENERATION: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub definition: DefinitionId,
    pub consumer: DefinitionId,
    pub account: Account,
    pub seed: u64,
    pub policy: Policy,
    pub mechanism: Mechanism,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Supply {
    pub mechanism: Mechanism,
    pub month: u32,
    pub definition: DefinitionId,
    pub account: Account,
    pub available_stock: i32,
    pub units_per_lot: i32,
}
pub fn supply(world: &World, state: &State) -> Option<Supply> {
    world.pool_market.as_ref().map(|c| Supply {
        mechanism: c.mechanism,
        month: state.month,
        definition: c.definition,
        account: c.account,
        available_stock: state.balance(c.account.0, c.account.1),
        units_per_lot: world.definition(c.definition).stages[0].entry_inputs[0].quantity,
    })
}

/// Preview a pool round using the same typed process requests as other offers.
/// The configured mechanism allows partial lots or requires each person's complete
/// collection request. Continuing work remains outside those new bundles.
pub fn prepare(sim: &Simulation, requests: &[offers::Request]) -> Result<Batch, String> {
    if sim.state.phase != Phase::Productive {
        return Err("collection preview requires Productive".into());
    }
    let mut work = Vec::new();
    for r in requests {
        let offers::Id::Process(definition) = r.offer else {
            return Err("collection preview expects work requests".into());
        };
        if !sim.world.definitions.iter().any(|d| d.id == definition)
            || !sim.world.agents.iter().any(|a| a.id == r.agent)
            || r.continuing.is_some_and(|id| {
                !sim.state.processes.get(&id).is_some_and(|p| {
                    p.operator == r.agent
                        && p.definition == definition
                        && p.status == Status::Active
                })
            })
        {
            return Err("invalid collection work request".into());
        }
        work.push(Request {
            agent: r.agent,
            definition,
            existing: r.continuing,
            need: r.need,
        });
    }
    let mut batch = Batch::empty(&sim.state);
    resolve(sim, work, &mut batch)?;
    Ok(batch)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Demand {
    pub agent: AgentId,
    pub requested: u32,
    pub feasible: u32,
    pub urgent: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub context: Context,
    pub policy: Policy,
    pub available_stock: i32,
    pub units_per_lot: i32,
    pub requests: Vec<offers::Request>,
    pub demands: Vec<Demand>,
    pub receipts: Vec<Receipt>,
    pub resolution: resolution::Resolution,
}

pub fn validate(world: &World) -> Result<(), String> {
    let Some(c) = &world.pool_market else {
        return Ok(());
    };
    let d = world
        .definitions
        .iter()
        .find(|d| d.id == c.definition)
        .ok_or("unknown collection process")?;
    let consumer = world
        .definitions
        .iter()
        .find(|d| d.id == c.consumer)
        .ok_or("unknown collection consumer")?;
    if d.execution != Execution::Productive
        || d.asset_kind.is_some()
        || d.stages.len() != 1
        || d.stages[0].months != 1
        || d.stages[0].entry_inputs.len() != 1
        || d.stages[0].entry_inputs[0].resource != c.account.1
        || d.stages[0].entry_inputs[0].quantity <= 0
        || d.stages[0].monthly_services.is_empty()
        || d.stages[0].monthly_services.iter().any(|a| a.quantity <= 0)
        || d.outputs.len() != 1
        || d.outputs[0].quantity <= 0
        || consumer.execution != Execution::Consumption
        || consumer.stages.len() != 1
        || consumer.stages[0].months != 1
        || consumer.stages[0].entry_inputs.len() != 1
        || consumer.stages[0].entry_inputs[0].resource != d.outputs[0].resource
        || consumer.stages[0].entry_inputs[0].quantity <= 0
        || consumer.outputs.len() != 1
        || consumer.outputs[0].quantity <= 0
        || !world
            .pool_inputs
            .iter()
            .any(|p| p.definition == d.id && p.account == c.account)
        || !world.households.is_empty()
        || world.market.is_some()
    {
        return Err("pool market requires a single-period collection/consumption chain".into());
    }
    Ok(())
}

/// Forecast stock target uses the configured need buffer; urgent means opening
/// stock cannot supply this month's need. Request size is bounded by own capacity.
pub fn demand(world: &World, state: &State, agent: AgentId) -> Result<(u32, bool), String> {
    let c = world.pool_market.as_ref().ok_or("missing pool market")?;
    let d = world.definition(c.definition);
    let consumer = world.definition(c.consumer);
    let Some(p) = world.participants.iter().find(|p| p.agent == agent) else {
        return Ok((0, false));
    };
    let need = p
        .needs
        .iter()
        .find(|n| n.resource == consumer.outputs[0].resource)
        .map_or(0, |n| n.quantity);
    let per_month = (i128::from(need) + i128::from(consumer.outputs[0].quantity) - 1)
        / i128::from(consumer.outputs[0].quantity)
        * i128::from(consumer.stages[0].entry_inputs[0].quantity);
    let stock = i128::from(state.balance(agent, d.outputs[0].resource));
    let deficit = (per_month * i128::from(world.horizon) - stock).max(0);
    let mut lots =
        (deficit + i128::from(d.outputs[0].quantity) - 1) / i128::from(d.outputs[0].quantity);
    for service in &d.stages[0].monthly_services {
        if service.quantity > 0 {
            lots = lots.min(i128::from(
                state.balance(agent, service.resource) / service.quantity,
            ));
        }
    }
    Ok((
        u32::try_from(lots).map_err(|_| "collection demand overflow")?,
        stock < per_month,
    ))
}

fn completed(batch: &Batch, definition: DefinitionId, agent: AgentId) -> usize {
    batch
        .transactions
        .iter()
        .filter_map(|t| t.process.as_ref())
        .filter(|p| {
            p.before.is_none()
                && p.after.definition == definition
                && p.after.operator == agent
                && p.after.status == Status::Completed
        })
        .count()
}

pub(crate) fn resolve(
    sim: &Simulation,
    requests: Vec<Request>,
    batch: &mut Batch,
) -> Result<(), String> {
    let c = sim
        .world
        .pool_market
        .as_ref()
        .ok_or("missing pool market")?;
    let unit = sim.world.definition(c.definition).stages[0].entry_inputs[0].quantity;
    if requests.len() > sim.effect_limit {
        return Err("pool request limit exceeded".into());
    }
    let opening_requests: Vec<_> = requests
        .iter()
        .map(|r| offers::Request {
            offer: offers::Id::Process(r.definition),
            agent: r.agent,
            continuing: r.existing,
            need: r.need,
        })
        .collect();
    let mut counts = BTreeMap::<AgentId, u32>::new();
    let mut work = Vec::new();
    for r in requests {
        if r.definition == c.definition && r.existing.is_none() {
            *counts.entry(r.agent).or_default() += 1;
        } else {
            work.push(r);
        }
    }
    let mut base = Batch::empty(&sim.state);
    sim.resolve_work_unallocated(work.clone(), &mut base)?;
    let spent: i32 = base
        .transactions
        .iter()
        .flat_map(|t| &t.effects)
        .filter(|e| e.account == c.account && e.delta < 0)
        .map(|e| -e.delta)
        .sum();
    let available_stock = sim.state.balance(c.account.0, c.account.1) - spent;
    let context = Context {
        seed: c.seed,
        pool: (u64::from(c.account.0) << 32) | u64::from(c.account.1),
        round: u64::from(sim.state.month),
    };
    let mut demands = Vec::new();
    let mut claims = Vec::new();
    for (agent, requested) in counts {
        let urgent = demand(&sim.world, &sim.state, agent)?.1;
        // Check private feasibility without letting public scarcity decide order.
        // This shadow stock is never committed or used to produce real grants.
        let mut shadow = sim.clone();
        let extra = i32::try_from(requested)
            .map_err(|_| "collection count overflow")?
            .checked_mul(unit)
            .ok_or("collection stock overflow")?;
        shadow.state.balances.insert(
            c.account,
            sim.state.balance(c.account.0, c.account.1).max(
                spent
                    .checked_add(extra)
                    .ok_or("collection stock overflow")?,
            ),
        );
        let mut trial = work.clone();
        let mut feasible = 0;
        for n in 1..=requested {
            trial.push(Request {
                agent,
                definition: c.definition,
                existing: None,
                need: Some(sim.world.definition(c.consumer).outputs[0].resource),
            });
            let mut checked = Batch::empty(&sim.state);
            shadow.resolve_work_unallocated(trial.clone(), &mut checked)?;
            if completed(&checked, c.definition, agent) != n as usize {
                break;
            }
            feasible = n;
        }
        demands.push(Demand {
            agent,
            requested,
            feasible,
            urgent,
        });
        if feasible > 0 || c.mechanism == Mechanism::ConditionalBundle {
            claims.push(Claim {
                id: u64::from(agent),
                priority: u32::from(!urgent),
                requested: if c.mechanism == Mechanism::ConditionalBundle {
                    requested
                } else {
                    feasible
                },
                minimum: 1,
            });
        }
    }
    let resolution_requests: Vec<_> = claims
        .into_iter()
        .map(|claim| resolution::Request {
            claim,
            inputs: BTreeMap::from([(c.account, unit as u32)]),
        })
        .collect();
    let (resolution, work) = resolution::resolve(
        context,
        &c.policy,
        c.mechanism,
        &BTreeMap::from([(
            c.account,
            u32::try_from(available_stock).map_err(|_| "negative collection availability")?,
        )]),
        &resolution_requests,
        &work,
        |work, claim, granted| {
            let mut trial = work.clone();
            let agent = claim.id as AgentId;
            for _ in 0..granted {
                trial.push(Request {
                    agent,
                    definition: c.definition,
                    existing: None,
                    need: Some(sim.world.definition(c.consumer).outputs[0].resource),
                });
            }
            let mut checked = Batch::empty(&sim.state);
            sim.resolve_work_unallocated(trial.clone(), &mut checked)?;
            if completed(&checked, c.definition, agent) != granted as usize {
                return Err("joint collection reservation failed".into());
            }
            *work = trial;
            Ok(())
        },
    )?;
    let mut resolved = Batch::empty(&sim.state);
    sim.resolve_work_unallocated(work, &mut resolved)?;
    batch.transactions.extend(resolved.transactions);
    batch.receipts.extend(resolved.receipts);
    batch.pool_market = Some(Round {
        context,
        policy: c.policy,
        available_stock,
        units_per_lot: unit,
        requests: opening_requests,
        demands,
        receipts: resolution.receipts.clone(),
        resolution,
    });
    Ok(())
}

pub(crate) fn validate_batch(
    world: &World,
    state: &State,
    batch: &Batch,
    effect_limit: usize,
) -> Result<(), String> {
    let Some(round) = &batch.pool_market else {
        if world.pool_market.is_some()
            && batch.phase == Phase::Productive
            && batch
                .transactions
                .iter()
                .filter_map(|t| t.process.as_ref())
                .any(|p| p.after.definition == world.pool_market.as_ref().unwrap().definition)
        {
            return Err("collection requires a dated allocation receipt".into());
        }
        return Ok(());
    };
    if batch.phase != Phase::Productive || world.pool_market.is_none() {
        return Err("pool allocation outside productive boundary".into());
    }
    let sim = Simulation {
        world: world.clone(),
        state: state.clone(),
        backend: Backend::Reference,
        effect_limit,
        ledger: vec![],
        reports: vec![],
    };
    let requests: Result<Vec<_>, String> = round
        .requests
        .iter()
        .map(|r| {
            let offers::Id::Process(definition) = r.offer else {
                return Err("non-process collection request".into());
            };
            if !world.definitions.iter().any(|d| d.id == definition)
                || !world.agents.iter().any(|a| a.id == r.agent)
                || r.continuing.is_some_and(|id| {
                    !state.processes.get(&id).is_some_and(|p| {
                        p.operator == r.agent
                            && p.definition == definition
                            && p.status == Status::Active
                    })
                })
            {
                return Err("invalid collection work request".into());
            }
            Ok(Request {
                agent: r.agent,
                definition,
                existing: r.continuing,
                need: r.need,
            })
        })
        .collect();
    let mut expected = Batch::empty(state);
    resolve(&sim, requests?, &mut expected)?;
    if expected.transactions != batch.transactions
        || expected.pool_market != batch.pool_market
        || !batch.receipts.ends_with(&expected.receipts)
    {
        return Err("collection differs from dated allocation".into());
    }
    Ok(())
}

pub fn scenario(supply: &str, policy: Policy) -> Result<(World, State), String> {
    use crate::scenario::*;
    let (world, state) = crate::competition::scenario(2, crate::competition::DEFAULT_SEED)?;
    let mut sim = Simulation::new(world, state, Backend::Reference)?;
    // Admit both farmers and reserve their crops before varying the wood supply.
    sim.run_months(1)?;
    let regeneration = match supply {
        "ample" => AMPLE_REGENERATION,
        "sufficient" => SUFFICIENT_REGENERATION,
        "scarce" => SCARCE_REGENERATION,
        _ => return Err("unknown collection supply".into()),
    };
    let c = Config {
        definition: PREPARE_FUEL,
        consumer: USE_FUEL,
        account: (STATE_AGENT, RAW_WOOD),
        seed: crate::competition::DEFAULT_SEED,
        policy,
        mechanism: Mechanism::Immediate,
    };
    sim.world
        .definitions
        .iter_mut()
        .find(|d| d.id == PREPARE_FUEL)
        .unwrap()
        .stages[0]
        .entry_inputs[0]
        .quantity = WOOD_PER_LOT;
    let pool = sim
        .world
        .pools
        .iter_mut()
        .find(|p| p.account == c.account)
        .unwrap();
    pool.capacity = POOL_CAPACITY;
    pool.monthly_regeneration = regeneration;
    sim.state.balances.insert(c.account, 0);
    for p in &sim.world.participants {
        sim.state.balances.insert((p.agent, FUEL), INITIAL_FUEL);
    }
    sim.world.pool_market = Some(c);
    Ok((sim.world, sim.state))
}
