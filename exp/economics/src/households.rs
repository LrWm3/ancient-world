//! Agreement-formed collective agents. Transfers are reserved before a phase and
//! collected after it; both sub-boundaries are part of the replayable batch.
use crate::{
    compute::{self, Backend},
    model::*,
    simulation::Simulation,
};
use std::collections::{BTreeMap, BTreeSet};

pub const FOUNDING_ADULT_LIMIT: usize = 4;
pub const GROWN_CHILD_ADULT_SLOTS: usize = 4;
pub const POOL_DIVISOR: i32 = 2;
const NEED_BENEFIT: i64 = 1_000_000;
const INPUT_BENEFIT: i64 = 1_000;
const PAYMENT_BENEFIT: i64 = 1;
const HOUSEHOLD_ID_BASE: u32 = 10_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agreement {
    pub id: u32,
    pub agent: AgentId,
    pub governance: crate::household_governance::Governance,
    /// All signatories are adults; labor ordering is selected by charter.
    pub adults: Vec<AgentId>,
    pub formed: u32,
    /// A non-rival occupancy service, produced by one member's actual dwelling.
    pub dwelling_process: Option<DefinitionId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Purpose {
    Need(ResourceId),
    Input(DefinitionId),
    Obligation,
    Labor,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub household: AgentId,
    pub member: AgentId,
    pub resource: ResourceId,
    pub quantity: i32,
    pub minimum: i32,
    pub individual_benefit: i64,
    pub collective_benefit: i64,
    pub sequence: u64,
    pub purpose: Purpose,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reservation {
    pub request: Request,
    pub allocated: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaborDecision {
    pub household: AgentId,
    pub recipient: Option<AgentId>,
    pub baseline_value: i64,
    pub projected_value: i64,
    pub granted: i32,
    pub policy: crate::household_governance::Policy,
    pub leader: Option<AgentId>,
    pub tie_break: crate::household_governance::TieBreak,
    pub contributions: Vec<LaborContribution>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaborContribution {
    pub member: AgentId,
    pub resource: ResourceId,
    pub available: i32,
    pub reserved: i32,
    pub directed: i32,
    pub returned: i32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Boundary {
    pub governance: Vec<crate::household_governance::Authority>,
    pub remainders: BTreeMap<Account, i32>,
    pub labor: Vec<LaborDecision>,
    pub reservations: Vec<Reservation>,
    pub before: Vec<Effect>,
    pub after: Vec<Effect>,
    /// Historical agent/agreement records survive dissolution; no orphan agent
    /// can plan, reserve or receive new contributions without a living adult.
    pub inactive: Vec<AgentId>,
}

pub fn members<'a>(a: &'a Agreement, state: &'a State) -> impl Iterator<Item = AgentId> + 'a {
    a.adults
        .iter()
        .copied()
        .filter(|id| !state.terminal.contains_key(id))
}
pub fn parent(world: &World, state: &State, person: AgentId) -> Option<AgentId> {
    world
        .households
        .iter()
        .find(|a| a.formed <= state.month && members(a, state).any(|m| m == person))
        .map(|a| a.agent)
}

/// Formation is an explicit, unanimously signed initial agreement. No goods or
/// capacity are minted and no child/adulthood transition is implemented yet.
pub fn form(world: &mut World, state: &State, agreement: Agreement) -> Result<(), String> {
    if agreement
        .adults
        .iter()
        .any(|id| state.terminal.contains_key(id))
    {
        return Err("only living adults can sign formation".into());
    }
    if agreement.formed != state.month {
        return Err("formation must be dated at the current boundary".into());
    }
    let mut candidate = world.clone();
    if candidate.agents.iter().any(|a| a.id == agreement.agent) {
        return Err("household agent already exists".into());
    }
    candidate.agents.push(Agent {
        id: agreement.agent,
        name: format!("household {}", agreement.id),
    });
    candidate.households.push(agreement);
    crate::settlement::validate_world(&candidate, state)?;
    *world = candidate;
    Ok(())
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    if !world.households.is_empty()
        && (world.priority == Priority::ConsequenceAware || state.pending_production.is_some())
    {
        return Err("households currently require fixed individual priorities without a pending forecast plan".into());
    }
    for (&(member, resource), &remainder) in &state.household_remainders {
        if !(0..POOL_DIVISOR).contains(&remainder)
            || !world.households.iter().any(|a| a.adults.contains(&member))
            || !world
                .resources
                .iter()
                .any(|r| r.id == resource && r.kind == ResourceKind::Stock)
        {
            return Err("invalid household fractional contribution".into());
        }
    }
    let mut ids = BTreeSet::new();
    let mut agents = BTreeSet::new();
    let mut adults = BTreeSet::new();
    for a in &world.households {
        crate::household_governance::validate(world, state, a)?;
        if !ids.insert(a.id)
            || !agents.insert(a.agent)
            || a.adults.is_empty()
            || a.adults.len() > FOUNDING_ADULT_LIMIT
            || a.formed == 0
            || a.formed > state.month
            || !world.agents.iter().any(|x| x.id == a.agent)
            || world.participants.iter().any(|p| p.agent == a.agent)
            || a.adults
                .iter()
                .any(|id| !adults.insert(*id) || !world.participants.iter().any(|p| p.agent == *id))
        {
            return Err("invalid household formation agreement".into());
        }
        if a.dwelling_process.is_some_and(|id| {
            !world.definitions.iter().any(|d| {
                d.id == id
                    && d.execution == Execution::Productive
                    && d.outputs.len() == 1
                    && world.activities.required.contains_key(&id)
                    && world.activities.perishable.contains(&d.outputs[0].resource)
            })
        }) {
            return Err("household dwelling must produce a perishable occupancy service".into());
        }
        if a.adults
            .iter()
            .any(|id| world.households.iter().any(|h| h.agent == *id))
        {
            return Err("nested household membership is unsupported".into());
        }
        if world.storage.capacities.contains_key(&a.agent) {
            return Err("household storage is contributed by members".into());
        }
    }
    Ok(())
}

/// Demand-capped reservations: collective benefit, then submitted order. Repeated
/// aliases for a member/resource reserve the greatest requested total, not its sum.
pub fn allocate(
    world: &World,
    state: &State,
    mut requests: Vec<Request>,
) -> Result<(Vec<Reservation>, Vec<Effect>), String> {
    requests.sort_by_key(|r| (std::cmp::Reverse(r.collective_benefit), r.sequence));
    let mut budget = state.balances.clone();
    let mut totals = BTreeMap::<Account, i32>::new();
    let mut used = crate::storage::usage(world, &state.balances);
    let mut receipts = vec![];
    let mut effects = vec![];
    for r in requests {
        if r.quantity < 0
            || r.minimum <= 0
            || parent(world, state, r.member) != Some(r.household)
            || !world
                .resources
                .iter()
                .any(|s| s.id == r.resource && s.kind == ResourceKind::Stock)
        {
            return Err("invalid household resource request".into());
        }
        let already = *totals.get(&(r.member, r.resource)).unwrap_or(&0);
        let demand = (r.quantity - already).max(0);
        let mut q = demand.min(*budget.get(&(r.household, r.resource)).unwrap_or(&0));
        if r.individual_benefit <= 0 || r.collective_benefit <= 0 || q < r.minimum {
            q = 0;
        }
        if q > 0 {
            let transfer = transfer(r.household, r.member, r.resource, q);
            if crate::storage::fits(world, &used, &transfer) {
                *budget.entry((r.household, r.resource)).or_default() -= q;
                *totals.entry((r.member, r.resource)).or_default() += q;
                crate::storage::apply(world, &mut used, &transfer);
                effects.extend(transfer);
            } else {
                q = 0;
            }
        }
        receipts.push(Reservation {
            request: r,
            allocated: q,
        });
    }
    Ok((receipts, effects))
}
fn transfer(from: AgentId, to: AgentId, r: ResourceId, q: i32) -> Vec<Effect> {
    vec![
        Effect {
            account: (from, r),
            delta: -q,
        },
        Effect {
            account: (to, r),
            delta: q,
        },
    ]
}

fn requests(world: &World, state: &State) -> Result<Vec<Request>, String> {
    let mut result = vec![];
    let planner = (state.phase == Phase::Productive).then(|| Simulation {
        world: world.clone(),
        state: state.clone(),
        ledger: vec![],
        reports: vec![],
        backend: Backend::Reference,
        effect_limit: crate::settlement::DEFAULT_EFFECT_LIMIT,
    });
    let protected = crate::commitments::protected_stock(world, state)?;
    for a in &world.households {
        for member in members(a, state) {
            let p = world
                .participants
                .iter()
                .find(|p| p.agent == member)
                .unwrap();
            let mut stocks = crate::substitution::stocks(state, member);
            let mut needs: Vec<_> = p.needs.iter().collect();
            needs.sort_by_key(|n| (n.priority, n.resource));
            for n in needs {
                let (_, missing) = crate::substitution::allocate(
                    world,
                    n.resource,
                    i128::from((n.quantity - state.balance(member, n.resource)).max(0)),
                    &mut stocks,
                    &BTreeMap::new(),
                );
                if missing == 0 {
                    continue;
                }
                for d in crate::substitution::recipes(world, n.resource) {
                    let input = &d.stages[0].entry_inputs[0];
                    if state.balance(a.agent, input.resource) == 0 {
                        continue;
                    }
                    let lots = (missing + i128::from(d.outputs[0].quantity) - 1)
                        / i128::from(d.outputs[0].quantity);
                    let quantity = i32::try_from(lots * i128::from(input.quantity))
                        .map_err(|_| "household need overflow")?;
                    let deprivation = state
                        .conditions
                        .get(&(member, n.resource))
                        .map(|c| i64::from(c.deprivation))
                        .unwrap_or(0);
                    let benefit = NEED_BENEFIT * (1 + deprivation) / (1 + i64::from(n.priority));
                    result.push(Request {
                        household: a.agent,
                        member,
                        resource: input.resource,
                        quantity,
                        minimum: input.quantity,
                        individual_benefit: benefit,
                        collective_benefit: benefit,
                        sequence: result.len() as u64,
                        purpose: Purpose::Need(n.resource),
                    });
                    break;
                }
            }
            if state.phase == Phase::Productive {
                let mut definitions = BTreeSet::new();
                for need in &p.needs {
                    if let Some(id) = planner.as_ref().unwrap().plan(p, need, None).0 {
                        definitions.insert(id);
                    }
                }
                for order in world
                    .activities
                    .orders
                    .iter()
                    .filter(|o| o.agent == member && crate::activities::wants(world, state, o))
                {
                    definitions.insert(order.definition);
                }
                for process in state
                    .processes
                    .values()
                    .filter(|p| p.operator == member && p.status == Status::Active)
                {
                    definitions.insert(process.definition);
                }
                for id in definitions {
                    let d = world.definition(id);
                    let active = state.processes.values().find(|p| {
                        p.operator == member && p.definition == id && p.status == Status::Active
                    });
                    if !d.enabled || active.is_some_and(|p| p.elapsed > 0) {
                        continue;
                    }
                    let stage = active.map(|p| p.stage).unwrap_or(0);
                    for input in &d.stages[stage].entry_inputs {
                        let quantity =
                            (input.quantity - state.balance(member, input.resource)).max(0);
                        if quantity > 0 {
                            result.push(Request {
                                household: a.agent,
                                member,
                                resource: input.resource,
                                quantity,
                                minimum: quantity,
                                individual_benefit: INPUT_BENEFIT,
                                collective_benefit: INPUT_BENEFIT,
                                sequence: result.len() as u64,
                                purpose: Purpose::Input(id),
                            });
                        }
                    }
                }
            }
            if matches!(
                state.phase,
                Phase::Due | Phase::ClearArrears | Phase::Acquire
            ) {
                for r in world
                    .resources
                    .iter()
                    .filter(|r| r.kind == ResourceKind::Stock)
                {
                    let owed: i128 =
                        crate::commitments::projected_claims(world, state, member, r.id, 1)
                            .values()
                            .sum();
                    let reserve = if state.phase == Phase::Acquire {
                        crate::forward::policy(world)
                            .and_then(|p| p.protected.get(&r.id))
                            .copied()
                            .unwrap_or(0)
                    } else {
                        0
                    };
                    let needs_requested = result
                        .iter()
                        .filter(|req: &&Request| {
                            req.member == member
                                && req.resource == r.id
                                && matches!(req.purpose, Purpose::Need(_))
                        })
                        .map(|r| r.quantity)
                        .max()
                        .unwrap_or(0);
                    let reserve = reserve.max(
                        protected
                            .get(&(member, r.id))
                            .copied()
                            .unwrap_or(0)
                            .saturating_add(needs_requested),
                    );
                    let quantity = i32::try_from(
                        (owed + i128::from(reserve) - i128::from(state.balance(member, r.id)))
                            .max(0),
                    )
                    .map_err(|_| "household payment overflow")?;
                    if owed > 0 && quantity > 0 {
                        result.push(Request {
                            household: a.agent,
                            member,
                            resource: r.id,
                            quantity,
                            minimum: 1,
                            individual_benefit: PAYMENT_BENEFIT,
                            collective_benefit: PAYMENT_BENEFIT,
                            sequence: result.len() as u64,
                            purpose: Purpose::Obligation,
                        });
                    }
                }
                if matches!(state.phase, Phase::Due | Phase::ClearArrears) {
                    let settlement = crate::commitments::evaluate(world, state)?;
                    for o in settlement.obligations.values().filter(|o| o.owed > o.paid) {
                        let Some(contract) = crate::commitments::active(world, state)
                            .find(|c| c.id == o.agreement && c.debtor == member)
                        else {
                            continue;
                        };
                        let Some(coin) = world.activities.coin_payments.get(&o.agreement) else {
                            continue;
                        };
                        // Native pooled payment takes precedence. Request coins
                        // only for the residual that the common commodity cannot cover.
                        let residual =
                            (o.owed - o.paid - state.balance(a.agent, contract.payment.resource))
                                .max(0);
                        let quantity = residual
                            .checked_mul(coin.coins_per_unit)
                            .ok_or("household coin payment overflow")?;
                        if quantity > 0 {
                            result.push(Request {
                                household: a.agent,
                                member,
                                resource: coin.resource,
                                quantity,
                                minimum: coin.coins_per_unit,
                                individual_benefit: PAYMENT_BENEFIT,
                                collective_benefit: PAYMENT_BENEFIT,
                                sequence: result.len() as u64,
                                purpose: Purpose::Obligation,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(result)
}

/// Checked gather for the household sub-boundaries. Repeated incoming resources
/// do not fund outgoing allocations in this same sub-boundary.
fn apply(
    world: &World,
    state: &mut State,
    effects: &[Effect],
    backend: Backend,
) -> Result<(), String> {
    let mut grouped = BTreeMap::<Account, Vec<i32>>::new();
    for e in effects {
        grouped.entry(e.account).or_default().push(e.delta);
    }
    let mut keys = vec![];
    let mut opening = vec![];
    let mut offsets = vec![0];
    let mut deltas = vec![];
    for (key, values) in grouped {
        if !world.agents.iter().any(|a| a.id == key.0)
            || !world.resources.iter().any(|r| r.id == key.1)
        {
            return Err("unknown household account".into());
        }
        let start = state.balance(key.0, key.1);
        if values
            .iter()
            .filter(|v| **v < 0)
            .map(|v| -i64::from(*v))
            .sum::<i64>()
            > i64::from(start)
        {
            return Err("household overspending".into());
        }
        let mut q = start;
        for v in &values {
            q = q.checked_add(*v).ok_or("household balance overflow")?;
        }
        if q < 0 {
            return Err("negative household balance".into());
        }
        keys.push(key);
        opening.push(start);
        deltas.extend(values);
        offsets.push(u32::try_from(deltas.len()).map_err(|_| "household buffer overflow")?);
    }
    if !crate::storage::fits(
        world,
        &crate::storage::usage(world, &state.balances),
        effects,
    ) {
        return Err("household storage exceeded".into());
    }
    let values = compute::apply(backend, &opening, &offsets, &deltas)?;
    for (key, value) in keys.into_iter().zip(values) {
        state.balances.insert(key, value);
    }
    Ok(())
}

fn prepare(world: &World, state: &State) -> Result<(State, Boundary), String> {
    let mut staged = state.clone();
    let mut b = Boundary {
        inactive: world
            .households
            .iter()
            .filter(|a| members(a, state).next().is_none())
            .map(|a| a.agent)
            .collect(),
        ..Default::default()
    };
    if state.phase == Phase::Open {
        b.governance = world
            .households
            .iter()
            .map(|a| crate::household_governance::authority(a, state))
            .collect();
        b.governance.sort_by_key(|a| a.household);
    }
    if !matches!(state.phase, Phase::Open | Phase::Close) {
        (b.reservations, b.before) = allocate(world, state, requests(world, state)?)?;
        apply(world, &mut staged, &b.before, Backend::Reference)?;
    }
    if state.phase == Phase::Productive && staged.pending_production.is_none() {
        let (labor, decisions) = labor(world, &staged)?;
        b.labor = decisions;
        apply(world, &mut staged, &labor, Backend::Reference)?;
        b.before.extend(labor);
    }
    Ok((staged, b))
}

/// Storage reservations include the mandatory contribution before execution.
pub(crate) fn with_contributions(
    world: &World,
    state: &State,
    effects: &[Effect],
) -> Result<Vec<Effect>, String> {
    let mut result = effects.to_vec();
    // Reserve an upper bound for indivisible ticks. Final settlement carries
    // fractional shares across receipts and months instead of losing them.
    for ((member, r), q) in incomes(world, state, effects)? {
        result.extend(transfer(
            member,
            parent(world, state, member).unwrap(),
            r,
            q / POOL_DIVISOR + i32::from(q % POOL_DIVISOR != 0),
        ));
    }
    Ok(result)
}
fn incomes(
    world: &World,
    state: &State,
    effects: &[Effect],
) -> Result<BTreeMap<Account, i32>, String> {
    let mut net = BTreeMap::<Account, i32>::new();
    for e in effects {
        let q = net.entry(e.account).or_default();
        *q = q
            .checked_add(e.delta)
            .ok_or("household collection overflow")?;
    }
    let mut result = BTreeMap::new();
    for ((member, r), quantity) in net {
        if parent(world, state, member).is_some()
            && quantity > 0
            && world
                .resources
                .iter()
                .any(|a| a.id == r && a.kind == ResourceKind::Stock)
            && !world.activities.perishable.contains(&r)
        {
            result.insert((member, r), quantity);
        }
    }
    Ok(result)
}

fn collect(
    world: &World,
    opening: &State,
    closed: &State,
    batch: &Batch,
) -> Result<(Vec<Effect>, BTreeMap<Account, i32>), String> {
    let mut effects = vec![];
    let mut gained = BTreeMap::<Account, i32>::new();
    let mut remainders = opening.household_remainders.clone();
    for t in &batch.transactions {
        if t.process.is_some()
            || t.stock_trade.is_some()
            || t.delivery.is_some()
            || t.trade.is_some()
        {
            for (key, quantity) in incomes(world, opening, &t.effects)? {
                let q = gained.entry(key).or_default();
                *q = q.checked_add(quantity).ok_or("household income overflow")?;
            }
        }
    }
    for (key, quantity) in gained {
        let numerator = i64::from(quantity) + i64::from(*remainders.get(&key).unwrap_or(&0));
        let share = (numerator / i64::from(POOL_DIVISOR)) as i32;
        remainders.insert(key, (numerator % i64::from(POOL_DIVISOR)) as i32);
        if share > 0 {
            effects.extend(transfer(
                key.0,
                parent(world, opening, key.0).unwrap(),
                key.1,
                share,
            ));
        }
    }
    if batch.phase == Phase::Productive {
        for a in &world.households {
            let Some(id) = a.dwelling_process else {
                continue;
            };
            let d = world.definition(id);
            let service = &d.outputs[0];
            if batch.transactions.iter().any(|t| {
                t.process.as_ref().is_some_and(|p| {
                    p.after.definition == id
                        && p.after.status == Status::Completed
                        && members(a, opening).any(|m| m == p.after.operator)
                })
            }) {
                for m in members(a, opening) {
                    let q = (service.quantity - closed.balance(m, service.resource)).max(0);
                    if q > 0 {
                        effects.push(Effect {
                            account: (m, service.resource),
                            delta: q,
                        });
                    }
                }
            }
        }
    }
    Ok((effects, remainders))
}

fn probe(world: &World, state: &State) -> Result<Batch, String> {
    let sim = Simulation {
        world: world.clone(),
        state: state.clone(),
        ledger: vec![],
        reports: vec![],
        backend: Backend::Reference,
        effect_limit: crate::settlement::DEFAULT_EFFECT_LIMIT,
    };
    let mut batch = Batch::empty(state);
    sim.productive_with(&mut batch, false)?;
    Ok(batch)
}
fn work_value(world: &World, batch: &Batch, people: &BTreeSet<AgentId>) -> i64 {
    batch
        .transactions
        .iter()
        .filter_map(|t| t.process.as_ref())
        .filter(|p| {
            people.contains(&p.after.operator)
                && people.contains(&p.after.beneficiary)
                && p.after.status != Status::Aborted
        })
        .map(|p| {
            let d = world.definition(p.after.definition);
            // A bounded net-output progress proxy in current spot value;
            // unquoted resources use par value. No future market sale is assumed.
            let value = |a: &Amount| -> i64 {
                crate::forward::policy(world)
                    .and_then(|p| p.prices.get(&a.resource))
                    .map(|p| i64::from(a.quantity) * i64::from(p.coins) / i64::from(p.goods))
                    .unwrap_or(i64::from(a.quantity))
            };
            let output = d.outputs.iter().map(value).sum::<i64>();
            let inputs = d
                .stages
                .iter()
                .flat_map(|s| &s.entry_inputs)
                .map(value)
                .sum::<i64>();
            let net = if world.activities.outcomes.contains_key(&d.id) {
                (output - inputs).max(1)
            } else {
                (output - inputs).max(0)
            };
            net * INPUT_BENEFIT / i64::from(d.duration()).max(1)
        })
        .sum()
}
fn labor(world: &World, state: &State) -> Result<(Vec<Effect>, Vec<LaborDecision>), String> {
    let mut decisions = vec![];
    let mut staged = state.clone();
    let mut effects = vec![];
    for a in &world.households {
        if let crate::household_governance::Contribution::Percent(percent) =
            a.governance.charter.contribution
        {
            let (chosen, decision) = contributed_labor(world, &staged, a, percent)?;
            apply(world, &mut staged, &chosen, Backend::Reference)?;
            effects.extend(chosen);
            decisions.push(decision);
            continue;
        }
        let people: BTreeSet<_> = members(a, state).collect();
        if people.len() < 2 {
            continue;
        }
        let baseline = probe(world, &staged)?;
        let base_value = work_value(world, &baseline, &people);
        let mut spent = BTreeMap::<Account, i32>::new();
        for e in baseline
            .transactions
            .iter()
            .flat_map(|t| &t.effects)
            .filter(|e| e.delta < 0)
        {
            *spent.entry(e.account).or_default() -= e.delta;
        }
        let mut best: Option<(i64, AgentId, Vec<Effect>)> = None;
        for member in crate::household_governance::ordered(a, state) {
            let p = world
                .participants
                .iter()
                .find(|p| p.agent == member)
                .unwrap();
            let resource = p.capacity.resource;
            let donors: Vec<_> = members(a, state)
                .filter(|m| *m != member)
                .map(|m| {
                    (
                        m,
                        (staged.balance(m, resource)
                            - spent.get(&(m, resource)).copied().unwrap_or(0))
                        .max(0),
                    )
                })
                .filter(|(_, q)| *q > 0)
                .collect();
            if donors.is_empty() {
                continue;
            }
            let mut trial = staged.clone();
            let mut proposed = vec![];
            for &(m, q) in &donors {
                proposed.extend(transfer(m, member, resource, q));
            }
            apply(world, &mut trial, &proposed, Backend::Reference)?;
            let plan = probe(world, &trial)?;
            let preserves_members = baseline
                .transactions
                .iter()
                .filter_map(|t| t.process.as_ref())
                .filter(|p| {
                    people.contains(&p.after.operator)
                        && p.after.operator != member
                        && p.after.status != Status::Aborted
                })
                .all(|p| {
                    plan.transactions
                        .iter()
                        .filter_map(|t| t.process.as_ref())
                        .any(|q| {
                            q.after.operator == p.after.operator
                                && q.after.definition == p.after.definition
                                && q.before.as_ref().map(|p| p.id)
                                    == p.before.as_ref().map(|p| p.id)
                                && q.after.asset == p.after.asset
                                && q.after.status == p.after.status
                                && q.after.stage == p.after.stage
                                && q.after.elapsed == p.after.elapsed
                        })
                });
            if !preserves_members {
                continue;
            }
            let value = work_value(world, &plan, &people) - base_value;
            if value <= 0 || best.as_ref().is_some_and(|(v, _, _)| *v >= value) {
                continue;
            }
            let used: i32 = plan
                .transactions
                .iter()
                .flat_map(|t| &t.effects)
                .filter(|e| e.account == (member, resource) && e.delta < 0)
                .map(|e| -e.delta)
                .sum();
            let mut required = (used - staged.balance(member, resource)).max(0);
            let mut actual = vec![];
            for (m, q) in donors {
                let grant = q.min(required);
                if grant > 0 {
                    actual.extend(transfer(m, member, resource, grant));
                    required -= grant;
                }
            }
            if !actual.is_empty() {
                best = Some((value, member, actual));
            }
        }
        let mut decision = LaborDecision {
            household: a.agent,
            recipient: None,
            baseline_value: base_value,
            projected_value: base_value,
            granted: 0,
            policy: a.governance.policy(state.month),
            leader: crate::household_governance::leader(a, state),
            tie_break: a.governance.charter.tie_break,
            contributions: vec![],
        };
        if let Some((gain, member, chosen)) = best {
            apply(world, &mut staged, &chosen, Backend::Reference)?;
            decision.recipient = Some(member);
            decision.projected_value += gain;
            decision.granted = chosen.iter().filter(|e| e.delta > 0).map(|e| e.delta).sum();
            effects.extend(chosen);
        }
        decisions.push(decision);
    }
    Ok((effects, decisions))
}

/// Fixed contributions are reserved before private productive planning. Only
/// demand-capped directed hours leave a donor; all other reservations are released.
fn contributed_labor(
    world: &World,
    state: &State,
    a: &Agreement,
    percent: u32,
) -> Result<(Vec<Effect>, LaborDecision), String> {
    use crate::household_governance::{self as governance, Policy};
    let people: BTreeSet<_> = members(a, state).collect();
    let order = governance::ordered(a, state);
    let baseline = probe(world, state)?;
    let base_value = work_value(world, &baseline, &people);
    let policy = a.governance.policy(state.month);
    let contributions: Vec<_> = order
        .iter()
        .map(|&member| {
            let resource = world
                .participants
                .iter()
                .find(|p| p.agent == member)
                .unwrap()
                .capacity
                .resource;
            let available = state.balance(member, resource);
            let reserved =
                (i64::from(available) * i64::from(percent) / i64::from(governance::PERCENT)) as i32;
            LaborContribution {
                member,
                resource,
                available,
                reserved,
                directed: 0,
                returned: reserved,
            }
        })
        .collect();
    let mut decision = LaborDecision {
        household: a.agent,
        recipient: None,
        baseline_value: base_value,
        projected_value: base_value,
        granted: 0,
        policy,
        leader: crate::household_governance::leader(a, state),
        tie_break: a.governance.charter.tie_break,
        contributions: contributions.clone(),
    };
    let mut reserved_state = state.clone();
    for c in &contributions {
        reserved_state
            .balances
            .insert((c.member, c.resource), c.available - c.reserved);
    }
    let mut best_effects = vec![];
    for &member in &order {
        let resource = world
            .participants
            .iter()
            .find(|p| p.agent == member)
            .unwrap()
            .capacity
            .resource;
        let pool = contributions
            .iter()
            .filter(|c| c.resource == resource)
            .try_fold(0_i32, |q, c| {
                q.checked_add(c.reserved)
                    .ok_or("household labor pool overflow")
            })?;
        if pool == 0 {
            continue;
        }
        let mut trial = reserved_state.clone();
        let available = trial
            .balance(member, resource)
            .checked_add(pool)
            .ok_or("household labor grant overflow")?;
        trial.balances.insert((member, resource), available);
        let plan = probe(world, &trial)?;
        // A limited mandate conservatively covers the recipient's whole funded
        // plan; verify again after returning unused reservations.
        let in_scope = |batch: &Batch| {
            !a.governance
                .constitution
                .activities
                .as_ref()
                .is_some_and(|allowed| {
                    batch
                        .transactions
                        .iter()
                        .filter_map(|t| t.process.as_ref())
                        .any(|p| {
                            p.after.operator == member
                                && p.after.status != Status::Aborted
                                && !allowed.contains(&p.after.definition)
                        })
                })
        };
        if !in_scope(&plan) {
            continue;
        }
        let spent: i64 = plan
            .transactions
            .iter()
            .flat_map(|t| &t.effects)
            .filter(|e| e.account == (member, resource) && e.delta < 0)
            .map(|e| -i64::from(e.delta))
            .sum();
        let mut required =
            i32::try_from((spent - i64::from(reserved_state.balance(member, resource))).max(0))
                .map_err(|_| "household labor demand overflow")?
                .min(pool);
        if required == 0 {
            continue;
        }
        let grant = required;
        let mut receipts = contributions.clone();
        // The recipient's own contribution returns first; donor ties follow the
        // same explicit ordering used for recipient selection.
        let mut donors: Vec<_> = (0..receipts.len()).collect();
        donors.sort_by_key(|&i| receipts[i].member != member);
        let mut proposed = vec![];
        for i in donors {
            let c = &mut receipts[i];
            if c.resource != resource {
                continue;
            }
            c.directed = c.reserved.min(required);
            c.returned = c.reserved - c.directed;
            required -= c.directed;
            if c.member != member && c.directed > 0 {
                proposed.extend(transfer(c.member, member, resource, c.directed));
            }
        }
        let mut final_state = state.clone();
        apply(world, &mut final_state, &proposed, Backend::Reference)?;
        let final_plan = probe(world, &final_state)?;
        if !in_scope(&final_plan) {
            continue;
        }
        let same_work = |p: &ProcessChange, q: &ProcessChange| {
            q.after.operator == p.after.operator
                && q.after.definition == p.after.definition
                && q.before.as_ref().map(|p| p.id) == p.before.as_ref().map(|p| p.id)
                && q.after.asset == p.after.asset
                && q.after.status == p.after.status
                && q.after.stage == p.after.stage
                && q.after.elapsed == p.after.elapsed
        };
        let retains = |source: &Batch, committed_only: bool| {
            source
                .transactions
                .iter()
                .filter_map(|t| t.process.as_ref())
                .filter(|p| {
                    people.contains(&p.after.operator)
                        && p.after.status != Status::Aborted
                        && if committed_only {
                            p.before.is_some()
                        } else {
                            p.after.operator == member
                        }
                })
                .all(|p| {
                    final_plan
                        .transactions
                        .iter()
                        .filter_map(|t| t.process.as_ref())
                        .any(|q| same_work(p, q))
                })
        };
        if !retains(&plan, false)
            || (policy == Policy::PreserveCommittedWork && !retains(&baseline, true))
        {
            continue;
        }
        let value = work_value(world, &final_plan, &people);
        if value <= decision.projected_value {
            continue;
        }
        decision.recipient = Some(member);
        decision.projected_value = value;
        decision.granted = grant;
        decision.contributions = receipts;
        best_effects = proposed;
    }
    Ok((best_effects, decision))
}

pub(crate) fn step(sim: &mut Simulation) -> Result<(), String> {
    let (prepared, mut receipt) = prepare(&sim.world, &sim.state)?;
    let mut staged = Simulation {
        world: sim.world.clone(),
        state: prepared.clone(),
        ledger: vec![],
        reports: vec![],
        backend: sim.backend,
        effect_limit: sim.effect_limit,
    };
    staged.step_core()?;
    let mut batch = staged.ledger.pop().unwrap();
    (receipt.after, receipt.remainders) = collect(&sim.world, &prepared, &staged.state, &batch)?;
    batch.household = Some(receipt);
    commit(
        &sim.world,
        &mut sim.state,
        &batch,
        sim.backend,
        sim.effect_limit,
    )?;
    sim.ledger.push(batch);
    sim.reports.extend(staged.reports);
    Ok(())
}

pub fn commit(
    world: &World,
    state: &mut State,
    batch: &Batch,
    backend: Backend,
    limit: usize,
) -> Result<(), String> {
    let (_, _, closed) = settled_boundaries(world, state, batch, backend, limit)?;
    *state = closed;
    Ok(())
}

/// Verified ordered views for settlement and accounting: allocated, core-settled,
/// and collected. All remain candidates until the caller publishes the result.
pub(crate) fn settled_boundaries(
    world: &World,
    state: &State,
    batch: &Batch,
    backend: Backend,
    limit: usize,
) -> Result<(State, State, State), String> {
    let (prepared, mut expected) = prepare(world, state)?;
    let receipt = batch
        .household
        .as_ref()
        .ok_or("missing household boundary")?;
    let count = batch
        .transactions
        .iter()
        .map(|t| t.effects.len())
        .sum::<usize>()
        .checked_add(receipt.before.len())
        .and_then(|n| n.checked_add(receipt.after.len()))
        .ok_or("household buffer overflow")?;
    if count > limit {
        return Err("effect buffer capacity exceeded before publication".into());
    }
    if receipt.before != expected.before
        || receipt.reservations != expected.reservations
        || receipt.inactive != expected.inactive
        || receipt.labor != expected.labor
        || receipt.governance != expected.governance
    {
        return Err("altered household reservations".into());
    }
    let mut staged = state.clone();
    apply(world, &mut staged, &expected.before, backend)?;
    let mut core = batch.clone();
    core.household = None;
    crate::settlement::commit_core(world, &mut staged, &core, backend, limit)?;
    (expected.after, expected.remainders) = collect(world, &prepared, &staged, &core)?;
    if receipt.after != expected.after || receipt.remainders != expected.remainders {
        return Err("altered household contributions".into());
    }
    let core_settled = staged.clone();
    apply(world, &mut staged, &expected.after, backend)?;
    staged.household_remainders = expected.remainders;
    crate::settlement::validate_world(world, &staged)?;
    Ok((prepared, core_settled, staged))
}

pub fn scenario() -> Result<(World, State), String> {
    let (mut w, s) =
        crate::trading_scenario::cash_scenario(crate::trading_scenario::DEFAULT_PROVIDERS, true)?;
    let people: Vec<_> = w.participants.iter().map(|p| p.agent).collect();
    for (i, chunk) in people.chunks(FOUNDING_ADULT_LIMIT).enumerate() {
        form(
            &mut w,
            &s,
            Agreement {
                id: i as u32,
                agent: HOUSEHOLD_ID_BASE + i as u32,
                adults: chunk.to_vec(),
                governance: crate::household_governance::Governance::contributed(chunk[0]),
                formed: s.month,
                dwelling_process: Some(crate::crafts::OCCUPY_HOME),
            },
        )?;
    }
    Ok((w, s))
}
