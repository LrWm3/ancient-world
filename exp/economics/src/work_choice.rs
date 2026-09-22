//! Bounded remaining-value choice over continuing agreements and posted processes.
//! Forecasts use ordinary resource allocation and settlement, not a second crop model.
use crate::{compute::Backend, model::*, offers, simulation::Simulation};
use std::collections::BTreeMap;

const MAX_HORIZON_MONTHS: u32 = 12;
const MAX_ALTERNATIVES: usize = 4;
const CHOICE_HORIZON_MONTHS: u32 = 4;
const WOOD_LABOR: i32 = 2;
const WOOD_OUTPUT: i32 = 4;
const MATURE_GROWTH_MONTHS: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub agent: AgentId,
    pub horizon: u32,
    /// Subjective stock values, not market quotes or realized coin income.
    pub values: BTreeMap<ResourceId, i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub continue_active: bool,
    /// Repeat at most once each month, after any continuing work.
    pub alternative: Option<DefinitionId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Forecast {
    pub plan: Plan,
    pub net_value: i64,
    pub labor: i64,
    pub first_work: Vec<Receipt>,
    pub ending_stocks: BTreeMap<ResourceId, i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub agent: AgentId,
    pub month: u32,
    pub through: u32,
    pub forecasts: Vec<Forecast>,
    pub selected: usize,
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let Some(c) = &world.work_choice else {
        return Ok(());
    };
    if c.horizon == 0
        || c.horizon > MAX_HORIZON_MONTHS
        || !world
            .participants
            .iter()
            .any(|p| p.agent == c.agent && p.needs.is_empty())
        || c.values.is_empty()
        || c.values.iter().any(|(r, v)| {
            *v < 0
                || !world
                    .resources
                    .iter()
                    .any(|x| x.id == *r && x.kind == ResourceKind::Stock)
        })
        || state
            .processes
            .values()
            .filter(|p| p.operator == c.agent && p.status == Status::Active)
            .count()
            > 1
        || world.priority != Priority::ContinuingFirst
        || world.decision_horizon.is_some()
        || state.pending_production.is_some()
        || !world.households.is_empty()
        || world.competition.is_some()
        || world.pool_market.is_some()
        || world.market.is_some()
        || world.negotiation.is_some()
        || !world.pools.is_empty()
        || !world.techniques.is_empty()
        || !world.condition_rules.is_empty()
        || world.activities != Default::default()
    {
        return Err("remaining-value choice requires one active process, a stock-valuing agent and independent work budgets".into());
    }
    // Missing valuations must not silently turn future stock inputs into free
    // resources. Zero is allowed, but must be an explicit preference.
    if world
        .definitions
        .iter()
        .filter(|d| d.execution == Execution::Productive)
        .any(|d| {
            d.outputs
                .iter()
                .chain(d.stages.iter().flat_map(|s| &s.entry_inputs))
                .any(|a| !c.values.contains_key(&a.resource))
        })
    {
        return Err("remaining-value choice requires explicit input and output valuations".into());
    }
    if alternatives(world, state, c.agent).len() > MAX_ALTERNATIVES {
        return Err("too many one-month alternatives for remaining-value search".into());
    }
    Ok(())
}
fn alternatives(world: &World, state: &State, agent: AgentId) -> Vec<DefinitionId> {
    offers::discover(world, state, agent)
        .into_iter()
        .filter_map(|o| match o.id {
            offers::Id::Process(id)
                if world.definition(id).execution == Execution::Productive
                    && world.definition(id).duration() == 1 =>
            {
                Some(id)
            }
            _ => None,
        })
        .collect()
}

/// Plans only order/omit requests. The existing offer dispatcher allocates the
/// shared opening budgets and validates inputs, capacity, rights and storage.
fn work(sim: &Simulation, c: &Config, plan: &Plan) -> Result<Batch, String> {
    let mut batch = Batch::empty(&sim.state);
    let mut requests: Vec<_> = sim
        .productive_requests(&mut batch, false, None)?
        .into_iter()
        .filter(|r| r.agent != c.agent)
        .map(|r| offers::Request {
            offer: offers::Id::Process(r.definition),
            agent: r.agent,
            continuing: r.existing,
            need: r.need,
        })
        .collect();
    batch.receipts.retain(|r| r.agent != c.agent);
    for p in sim
        .state
        .processes
        .values()
        .filter(|p| p.operator == c.agent && p.status == Status::Active)
    {
        if plan.continue_active {
            requests.push(offers::Request {
                offer: offers::Id::Process(p.definition),
                agent: c.agent,
                continuing: Some(p.id),
                need: p.goal,
            });
        } else {
            let d = sim.world.definition(p.definition);
            let mut after = p.clone();
            crate::agreements::ProductionTerms::from_definition(d).fail(&mut after);
            batch.transactions.push(Transaction {
                cause: "remaining-value policy declines continuing work".into(),
                effects: vec![],
                process: Some(ProcessChange {
                    before: Some(p.clone()),
                    after,
                }),
                technique_use: None,
                trade: None,
                stock_trade: None,
                forward: None,
                delivery: None,
                royalty: None,
            });
            batch.receipts.push(Receipt {
                agent: c.agent,
                need: p.goal,
                definition: Some(p.definition),
                reason: Reason::NoBeneficialProcess,
                requested: d.stages[p.stage]
                    .monthly_services
                    .iter()
                    .try_fold(0i32, |sum, a| sum.checked_add(a.quantity))
                    .ok_or("service total overflow")?,
                allocated: 0,
                completed: 0,
            });
        }
    }
    if let Some(id) = plan.alternative {
        requests.push(offers::Request::new(offers::Id::Process(id), c.agent));
    }
    offers::resolve(sim, &requests, &mut batch)?;
    Ok(batch)
}
fn forecast(sim: &Simulation, c: &Config, plan: Plan) -> Result<Forecast, String> {
    let mut f = sim.clone();
    f.world.work_choice = None; // bounded explicit rollout; no recursive search
    f.backend = Backend::Reference;
    f.ledger.clear();
    f.reports.clear();
    // Future fixture shocks and discretionary transfers are not observations.
    f.world.capacity_overrides.clear();
    f.world
        .scheduled_starts
        .retain(|s| s.month == sim.state.month);
    if let Some(credit) = &mut f.world.credit {
        credit.transfers.retain(|t| t.month <= sim.state.month);
    }
    let participant = f
        .world
        .participants
        .iter_mut()
        .find(|p| p.agent == c.agent)
        .unwrap();
    participant.capacity.quantity = sim.state.balance(c.agent, participant.capacity.resource);
    let end = sim
        .state
        .month
        .checked_add(c.horizon)
        .ok_or("forecast month overflow")?;
    let mut first_work = None;
    let mut labor = 0i64;
    while f.state.month < end {
        if f.state.phase == Phase::Productive {
            let b = work(&f, c, &plan)?;
            first_work.get_or_insert_with(|| {
                b.receipts
                    .iter()
                    .filter(|r| r.agent == c.agent)
                    .cloned()
                    .collect()
            });
            for e in b.transactions.iter().flat_map(|t| &t.effects) {
                if e.account.0 == c.agent
                    && e.delta < 0
                    && f.world
                        .resources
                        .iter()
                        .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Capacity)
                {
                    labor += -i64::from(e.delta);
                }
            }
            crate::settlement::commit(
                &f.world,
                &mut f.state,
                &b,
                Backend::Reference,
                f.effect_limit,
            )?;
            f.ledger.push(b);
        } else {
            f.step()?;
        }
    }
    let ending_stocks: BTreeMap<_, _> = c
        .values
        .keys()
        .map(|r| (*r, f.state.balance(c.agent, *r)))
        .collect();
    let mut net_value = 0i64;
    for (r, value) in &c.values {
        let delta = i64::from(ending_stocks[r]) - i64::from(sim.state.balance(c.agent, *r));
        net_value = net_value
            .checked_add(
                delta
                    .checked_mul(i64::from(*value))
                    .ok_or("value overflow")?,
            )
            .ok_or("value overflow")?;
    }
    Ok(Forecast {
        plan,
        net_value,
        labor,
        first_work: first_work.unwrap_or_default(),
        ending_stocks,
    })
}
pub fn evaluate(sim: &Simulation) -> Result<Batch, String> {
    validate(&sim.world, &sim.state)?;
    if sim.state.phase != Phase::Productive {
        return Err("work choice outside productive boundary".into());
    }
    let c = sim
        .world
        .work_choice
        .as_ref()
        .ok_or("missing work choice configuration")?;
    let active = sim
        .state
        .processes
        .values()
        .any(|p| p.operator == c.agent && p.status == Status::Active);
    let mut plans = vec![Plan {
        continue_active: false,
        alternative: None,
    }];
    let alternatives = alternatives(&sim.world, &sim.state, c.agent);
    for keep in [false, true] {
        if keep && !active {
            continue;
        }
        if keep {
            plans.push(Plan {
                continue_active: true,
                alternative: None,
            });
        }
        for id in &alternatives {
            plans.push(Plan {
                continue_active: keep,
                alternative: Some(*id),
            });
        }
    }
    let forecasts: Vec<_> = plans
        .into_iter()
        .map(|p| forecast(sim, c, p))
        .collect::<Result<_, _>>()?;
    let selected = (0..forecasts.len())
        .min_by_key(|i| {
            (
                std::cmp::Reverse(forecasts[*i].net_value),
                forecasts[*i].labor,
                *i,
            )
        })
        .unwrap();
    let mut batch = work(sim, c, &forecasts[selected].plan)?;
    batch.work_choice = Some(Decision {
        agent: c.agent,
        month: sim.state.month,
        through: sim.state.month + c.horizon - 1,
        forecasts,
        selected,
    });
    Ok(batch)
}
pub fn validate_batch(
    world: &World,
    state: &State,
    batch: &Batch,
    limit: usize,
) -> Result<(), String> {
    if world.work_choice.is_some() && state.phase == Phase::Productive {
        let mut sim = Simulation::new(world.clone(), state.clone(), Backend::Reference)?;
        sim.effect_limit = limit;
        if evaluate(&sim)? != *batch {
            return Err("altered remaining-value decision or work".into());
        }
    } else if batch.work_choice.is_some() {
        return Err("unexpected work choice receipt".into());
    }
    Ok(())
}

pub fn scenario(case: &str) -> Result<(World, State), String> {
    use crate::scenario::{GRAIN, GROW, LABOR, RAW_WOOD, SEED, STATE_AGENT};
    let (mut world, state) = crate::credit::crop_scenario(case != "no-capacity")?;
    match case {
        "mature" => {
            world
                .definitions
                .iter_mut()
                .find(|d| d.id == GROW)
                .unwrap()
                .stages[1]
                .months = MATURE_GROWTH_MONTHS
        }
        "expensive" | "no-capacity" => {}
        _ => return Err("unknown inherited-work scenario".into()),
    }
    world.resources.push(Resource {
        id: RAW_WOOD,
        name: "wood".into(),
        kind: ResourceKind::Stock,
    });
    world.definitions.push(ProcessDefinition {
        id: crate::scenario::PREPARE_FUEL,
        name: "collect wood".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "collect".into(),
            months: 1,
            entry_inputs: vec![],
            monthly_services: vec![Amount::new(LABOR, WOOD_LABOR)],
        }],
        outputs: vec![Amount::new(RAW_WOOD, WOOD_OUTPUT)],
    });
    world.work_choice = Some(Config {
        agent: STATE_AGENT,
        horizon: CHOICE_HORIZON_MONTHS,
        values: BTreeMap::from([(GRAIN, 1), (SEED, 1), (RAW_WOOD, 1)]),
    });
    Ok((world, state))
}
