//! Validate into staging, gather balances, then publish one boundary.
use crate::{
    compute::{self, Backend},
    maintenance,
    model::*,
};
use std::collections::{BTreeMap, BTreeSet};

pub const DEFAULT_EFFECT_LIMIT: usize = 4096;
const MAX_PLANNING_HORIZON_MONTHS: u32 = 120;
const MAX_PROCESS_DURATION_MONTHS: u32 = 120;

pub fn validate_world(world: &World, state: &State) -> Result<(), String> {
    crate::opportunities::validate(world)?;
    crate::negotiation::validate(world)?;
    crate::pool_market::validate(world)?;
    crate::membership::validate(world, state)?;
    crate::households::validate(world, state)?;
    fn unique(ids: impl Iterator<Item = u32>) -> bool {
        let mut seen = BTreeSet::new();
        ids.into_iter().all(|id| seen.insert(id))
    }
    if !unique(world.agents.iter().map(|x| x.id))
        || !unique(world.resources.iter().map(|x| x.id))
        || !unique(world.definitions.iter().map(|x| x.id))
        || !unique(world.assets.iter().map(|x| x.id))
        || !unique(world.rights.iter().map(|x| x.id))
        || !unique(world.participants.iter().map(|x| x.agent))
    {
        return Err("duplicate stable ID".into());
    }
    if world.horizon == 0 || world.horizon > MAX_PLANNING_HORIZON_MONTHS {
        return Err("horizon must be 1..=120 months".into());
    }
    let agent = |id| world.agents.iter().any(|a| a.id == id);
    let kind = |id| world.resources.iter().find(|r| r.id == id).map(|r| r.kind);
    for p in &world.participants {
        if !agent(p.agent)
            || p.capacity.quantity < 0
            || kind(p.capacity.resource) != Some(ResourceKind::Capacity)
            || !unique(p.needs.iter().map(|n| n.resource))
            || p.needs
                .iter()
                .any(|n| n.quantity < 0 || kind(n.resource) != Some(ResourceKind::Fulfillment))
        {
            return Err("invalid participant components".into());
        }
    }
    for a in &world.assets {
        if !agent(a.owner) {
            return Err("missing asset owner".into());
        }
    }
    for r in &world.rights {
        if !agent(r.holder)
            || !agent(r.output_owner)
            || !world.assets.iter().any(|a| a.id == r.asset)
            || r.from == 0
            || r.through < r.from
        {
            return Err("invalid right".into());
        }
    }
    for d in &world.definitions {
        let duration: u64 = d.stages.iter().map(|s| u64::from(s.months)).sum();
        if d.stages.is_empty()
            || duration > u64::from(MAX_PROCESS_DURATION_MONTHS)
            || (d.outputs.is_empty() && !world.activities.outcomes.contains_key(&d.id))
            || d.stages.iter().any(|s| s.months == 0)
        {
            return Err("invalid process duration/output".into());
        }
        for s in &d.stages {
            for a in &s.entry_inputs {
                if a.quantity <= 0 || kind(a.resource) != Some(ResourceKind::Stock) {
                    return Err("invalid process stock input".into());
                }
            }
            for a in &s.monthly_services {
                if a.quantity <= 0 || kind(a.resource) != Some(ResourceKind::Capacity) {
                    return Err("invalid process service".into());
                }
            }
        }
        for a in &d.outputs {
            let expected = if d.execution == Execution::Productive {
                ResourceKind::Stock
            } else {
                ResourceKind::Fulfillment
            };
            if a.quantity <= 0 || kind(a.resource) != Some(expected) {
                return Err("invalid process output".into());
            }
        }
        // The first planner deliberately supports one stock -> one fulfillment.
        if d.execution == Execution::Consumption
            && (d.duration() != 1
                || d.stages[0].entry_inputs.len() != 1
                || !d.stages[0].monthly_services.is_empty()
                || d.outputs.len() != 1
                || d.asset_kind.is_some())
        {
            return Err("consumption must be a one-stage stock-to-fulfillment process".into());
        }
    }
    for ((month, who), quantity) in &world.capacity_overrides {
        if *month == 0 || !agent(*who) || *quantity < 0 {
            return Err("invalid capacity override".into());
        }
    }
    for start in &world.scheduled_starts {
        if start.month == 0
            || !agent(start.agent)
            || !world
                .definitions
                .iter()
                .any(|d| d.id == start.definition && d.execution == Execution::Productive)
        {
            return Err("invalid scheduled intent".into());
        }
    }
    crate::planning::validate(world)?;
    validate_state(world, state)
}

fn validate_state(world: &World, state: &State) -> Result<(), String> {
    crate::activities::validate(world, state)?;
    crate::exchange::validate(world, state)?;
    crate::storage::validate(world, state)?;
    crate::currency::validate(world)?;
    crate::pools::validate(world, state)?;
    maintenance::validate(world, state)?;
    crate::equipment::validate(world, state)?;
    crate::commitments::validate(world, state)?;
    if state.month == 0
        || state.month > u32::MAX - MAX_PLANNING_HORIZON_MONTHS - MAX_PROCESS_DURATION_MONTHS
    {
        return Err("month out of range".into());
    }
    for (&(owner, resource), &value) in &state.balances {
        if value < 0
            || !world.agents.iter().any(|a| a.id == owner)
            || !world.resources.iter().any(|r| r.id == resource)
        {
            return Err("invalid account or negative balance".into());
        }
    }
    let mut occupied = BTreeSet::new();
    for (&id, p) in &state.processes {
        let d = world
            .definitions
            .iter()
            .find(|d| d.id == p.definition)
            .ok_or("unknown process definition")?;
        if id != p.id
            || !world.agents.iter().any(|a| a.id == p.operator)
            || !world.agents.iter().any(|a| a.id == p.beneficiary)
            || p.goal.is_some_and(|goal| {
                !world.participants.iter().any(|participant| {
                    participant.agent == p.operator
                        && participant.needs.iter().any(|n| n.resource == goal)
                })
            })
            || p.stage >= d.stages.len()
            || p.elapsed > d.stages[p.stage].months
            || p.start.checked_add(d.duration() - 1) != Some(p.reserved_through)
        {
            return Err("invalid process instance".into());
        }
        match (d.asset_kind, p.asset, p.right) {
            (Some(kind), Some(asset), Some(right)) => {
                if !world.assets.iter().any(|a| a.id == asset && a.kind == kind)
                    || !world.rights.iter().any(|r| {
                        r.id == right
                            && (!world.access_offers.iter().any(|a| a.right == right)
                                || state
                                    .accepted_agreements
                                    .values()
                                    .any(|a| a.right == right && a.activated <= p.start))
                            && r.asset == asset
                            && crate::commitments::holder(world, state, r) == Some(p.operator)
                            && crate::commitments::output_owner(world, state, r)
                                == Some(p.beneficiary)
                            && r.from <= p.start
                            && r.through >= p.reserved_through
                    })
                {
                    return Err("invalid process authority".into());
                }
                if p.status == Status::Active
                    && !world.activities.shared_sites.contains(&d.id)
                    && !occupied.insert(asset)
                {
                    return Err("overlapping exclusive occupancy".into());
                }
            }
            (None, None, None) => {}
            _ => return Err("process asset binding mismatch".into()),
        }
    }
    Ok(())
}

/// Replays only committed, trusted operation records. Arithmetic, references,
/// stage identity, authority and structural preconditions are revalidated.
/// This is not an authentication interface for externally supplied transactions.
pub fn commit(
    world: &World,
    state: &mut State,
    batch: &Batch,
    backend: Backend,
    effect_limit: usize,
) -> Result<(), String> {
    if !world.households.is_empty() {
        return crate::households::commit(world, state, batch, backend, effect_limit);
    }
    if batch.household.is_some() {
        return Err("unexpected household receipt".into());
    }
    commit_core(world, state, batch, backend, effect_limit)
}

pub(crate) fn commit_core(
    world: &World,
    state: &mut State,
    batch: &Batch,
    backend: Backend,
    effect_limit: usize,
) -> Result<(), String> {
    if (batch.id, batch.month, batch.phase) != (state.next_batch, state.month, state.phase) {
        return Err("duplicate, stale or out-of-order batch".into());
    }
    crate::pool_market::validate_batch(world, state, batch, effect_limit)?;
    crate::negotiation::validate_batch(world, state, batch)?;
    let count = batch
        .transactions
        .iter()
        .try_fold(0usize, |n, t| n.checked_add(t.effects.len()))
        .ok_or("effect count overflow")?;
    if count > effect_limit {
        return Err("effect buffer capacity exceeded before publication".into());
    }
    let expected_maintenance = if batch.phase == Phase::Close {
        Some(maintenance::evaluate(world, state)?)
    } else {
        None
    };
    if batch.maintenance != expected_maintenance {
        return Err("missing, stale or invalid maintenance settlement".into());
    }
    if let Some(plan) = &state.pending_production
        && plan.as_ref() != batch
    {
        return Err("execution differs from dated production plan".into());
    }
    if batch.production_plan.is_some() && batch.phase != Phase::Acquire {
        return Err("production plan outside acquisition boundary".into());
    }
    let expected_commitments = if matches!(batch.phase, Phase::Due | Phase::ClearArrears) {
        Some(crate::commitments::evaluate(world, state)?)
    } else {
        None
    };
    if batch.commitments != expected_commitments
        || expected_commitments
            .as_ref()
            .is_some_and(|s| s.transactions != batch.transactions)
    {
        return Err("missing or altered commitment settlement".into());
    }
    if !matches!(batch.phase, Phase::Due | Phase::ClearArrears) {
        for token in world
            .issuance
            .iter()
            .map(|r| r.token)
            .collect::<BTreeSet<_>>()
        {
            let net: i128 = batch
                .transactions
                .iter()
                .flat_map(|t| &t.effects)
                .filter(|e| e.account.1 == token)
                .map(|e| i128::from(e.delta))
                .sum();
            if net != 0 {
                return Err("currency issuance outside collection settlement".into());
            }
        }
    }
    if batch.phase == Phase::Acquire
        && world.market.is_some()
        && batch.transactions != crate::exchange::resolve(world, state)?
    {
        return Err("exchange differs from reserved opening offers".into());
    }
    if batch.transactions.iter().any(|t| {
        (t.delivery.is_some() || t.forward.is_some())
            && (batch.phase != Phase::Acquire || world.market.is_none())
            || t.royalty.is_some() && t.process.is_none()
    }) {
        return Err("exchange receipt outside authorized boundary".into());
    }
    let expansion = world
        .market
        .as_ref()
        .and_then(|m| m.plots.as_ref())
        .is_some();
    if batch.phase == Phase::Acquire && expansion {
        let expected = crate::plots::after_market(world, state, &batch.transactions)?;
        let accepted = expected
            .as_ref()
            .filter(|r| r.reason == crate::plots::Reason::Accepted)
            .and_then(|r| r.offer);
        if batch.plot_request != expected || batch.accept_access != accepted {
            return Err("plot request differs from opening productivity forecast".into());
        }
    } else if batch.plot_request.is_some() {
        return Err("plot request outside review boundary".into());
    }
    if batch.allocation.is_some() {
        crate::competition::validate_batch(world, state, batch, effect_limit)?;
    }
    if batch.access_applicant.is_some() && batch.accept_access.is_none() {
        return Err("applicant without access acceptance".into());
    }
    let mut staged = state.clone();
    for (offer, agent) in batch
        .accept_membership
        .into_iter()
        .chain(batch.additional_memberships.iter().copied())
    {
        let membership = crate::membership::acceptance(world, &staged, offer, agent)?;
        staged.memberships.insert(
            (membership.member, membership.organization, membership.role),
            membership,
        );
    }
    if let Some(id) = batch.accept_access {
        let agreement = if let Some(applicant) = batch.access_applicant {
            crate::commitments::acceptance_for(world, &staged, id, applicant)?
        } else {
            crate::commitments::acceptance(world, &staged, id)?
        };
        staged.accepted_agreements.insert(id, agreement);
    }
    for &(id, applicant) in &batch.additional_access {
        let agreement = crate::commitments::acceptance_for(world, &staged, id, applicant)?;
        staged.accepted_agreements.insert(id, agreement);
    }
    if let Some(s) = expected_commitments {
        staged.obligations = s.obligations;
    }
    staged.pending_production = batch.production_plan.clone();
    if let Some(settlement) = &batch.maintenance {
        for change in &settlement.changes {
            staged
                .conditions
                .insert((change.subject, change.resource), change.after.clone());
        }
        for transition in &settlement.transitions {
            staged
                .terminal
                .insert(transition.subject, transition.clone());
        }
    }
    if batch.phase == Phase::Open {
        crate::activities::age(world, &mut staged);
        let expiry: Vec<_> = batch
            .transactions
            .iter()
            .flat_map(|t| &t.effects)
            .filter(|e| world.activities.perishable.contains(&e.account.1))
            .cloned()
            .collect();
        if expiry != crate::activities::expiration(world, state) {
            return Err("invalid service ticket expiration".into());
        }
        let actual: Vec<_> = batch
            .transactions
            .iter()
            .flat_map(|t| &t.effects)
            .filter(|e| world.pools.iter().any(|p| p.account == e.account))
            .cloned()
            .collect();
        if actual != crate::pools::regeneration(world, state) {
            return Err("invalid shared pool regeneration".into());
        }
    }
    let mut groups: BTreeMap<Account, Vec<i32>> = BTreeMap::new();
    let mut changed = BTreeSet::new();
    for t in &batch.transactions {
        if let Some(trade) = &t.stock_trade {
            let expected = crate::currency::transaction(world, state, trade.clone())?;
            if t.effects != expected.effects
                || t.process.is_some()
                || t.technique_use.is_some()
                || t.trade.is_some()
            {
                return Err("stock trade differs from posted bid".into());
            }
        }
        if t.trade.is_some() {
            crate::equipment::apply_trade(world, &mut staged, t)?;
        }
        if t.technique_use.is_some() {
            crate::equipment::apply_use(world, state, &mut staged, t)?;
        }
        for e in &t.effects {
            groups.entry(e.account).or_default().push(e.delta);
        }
        if let Some(change) = &t.process {
            let after = &change.after;
            if change.before.is_none()
                && after
                    .right
                    .is_some_and(|r| !crate::commitments::can_start(world, state, r))
            {
                return Err("agreement blocks new process".into());
            }
            if state.terminal.contains_key(&after.operator) && after.status != Status::Aborted {
                return Err("inactive operator cannot execute a process".into());
            }
            if !changed.insert(after.id) || state.processes.get(&after.id) != change.before.as_ref()
            {
                return Err("duplicate or stale process transition".into());
            }
            if change
                .before
                .as_ref()
                .is_some_and(|p| p.status != Status::Active)
                || !matches!(batch.phase, Phase::Productive | Phase::Consumption)
            {
                return Err("process advanced outside its execution boundary".into());
            }
            let definition = world
                .definitions
                .iter()
                .find(|d| d.id == after.definition)
                .ok_or("unknown definition")?;
            let expected = if definition.execution == Execution::Productive {
                Phase::Productive
            } else {
                Phase::Consumption
            };
            if expected != batch.phase {
                return Err("wrong process execution phase".into());
            }
            validate_process_transaction(
                world,
                definition,
                change,
                t,
                state,
                t.technique_use
                    .as_ref()
                    .map(|u| crate::equipment::services(world, u)),
            )?;
            crate::activities::apply(world, &mut staged, change)?;
            crate::equipment::award(world, &mut staged, change)?;
            staged.processes.insert(after.id, after.clone());
        }
        crate::exchange::record(&mut staged, t);
    }
    let mut keys = Vec::new();
    let mut opening = Vec::new();
    let mut offsets = vec![0u32];
    let mut deltas = Vec::new();
    for (key, values) in groups {
        if !world.agents.iter().any(|a| a.id == key.0)
            || !world.resources.iter().any(|r| r.id == key.1)
        {
            return Err("unknown effect account".into());
        }
        let start = state.balance(key.0, key.1);
        let outgoing: i64 = values
            .iter()
            .filter(|&&v| v < 0)
            .map(|&v| -i64::from(v))
            .sum();
        if outgoing > i64::from(start) {
            return Err("gross spending exceeds opening resources".into());
        }
        let mut checked = start;
        for &delta in &values {
            checked = checked
                .checked_add(delta)
                .ok_or("balance prefix overflow")?;
        }
        if checked < 0 {
            return Err("negative closing balance".into());
        }
        keys.push(key);
        opening.push(start);
        deltas.extend(values);
        offsets.push(u32::try_from(deltas.len()).map_err(|_| "effect index overflow")?);
    }
    let balances = compute::apply(backend, &opening, &offsets, &deltas)?;
    if balances.len() != keys.len() {
        return Err("invalid kernel result length".into());
    }
    for (key, value) in keys.into_iter().zip(balances) {
        staged.balances.insert(key, value);
    }
    (staged.month, staged.phase) = state.phase.next(state.month);
    if matches!(state.phase, Phase::Open | Phase::Due)
        && (!world.offers.is_empty()
            || !world.access_offers.is_empty()
            || world
                .transaction_policy
                .as_ref()
                .is_some_and(|p| !p.membership_offers.is_empty())
            || !world.bids.is_empty()
            || world.market.is_some()
            || world.negotiation.is_some())
    {
        staged.phase = Phase::Acquire;
    }
    if state.phase == Phase::Open
        && (!world.agreements.is_empty() || !world.access_offers.is_empty())
    {
        staged.phase = Phase::Due;
    }
    if state.phase == Phase::Productive
        && (!world.agreements.is_empty() || !world.access_offers.is_empty())
    {
        staged.phase = Phase::ClearArrears;
    }
    staged.next_batch = state.next_batch.checked_add(1).ok_or("batch ID overflow")?;
    validate_state(world, &staged)?;
    *state = staged;
    Ok(())
}

// A process cannot skip time, change owners mid-transition, refund sunk inputs,
// or emit an output different from its definition's declared result.
fn validate_process_transaction(
    world: &World,
    definition: &ProcessDefinition,
    change: &ProcessChange,
    transaction: &Transaction,
    state: &State,
    services: Option<&[Amount]>,
) -> Result<(), String> {
    let month = state.month;
    let after = &change.after;
    if after.status != Status::Aborted
        && !crate::opportunities::permits(
            world,
            state,
            after.operator,
            crate::opportunities::Action::Process(definition.id),
        )
    {
        return Err("state policy denies process execution".into());
    }
    let mut expected = if let Some(before) = &change.before {
        before.clone()
    } else {
        if after.start != month || after.status == Status::Aborted {
            return Err("invalid new process boundary".into());
        }
        let mut initial = after.clone();
        initial.stage = 0;
        initial.elapsed = 0;
        initial.status = Status::Active;
        initial
    };
    if expected.definition != definition.id || expected.stage >= definition.stages.len() {
        return Err("invalid process definition/stage transition".into());
    }
    let stage = &definition.stages[expected.stage];
    let prior_months: u32 = definition.stages[..expected.stage]
        .iter()
        .map(|s| s.months)
        .sum();
    if expected
        .start
        .checked_add(prior_months)
        .and_then(|v| v.checked_add(expected.elapsed))
        != Some(month)
    {
        return Err("process advanced at the wrong month".into());
    }
    let mut effects = BTreeMap::<Account, i64>::new();
    if after.status == Status::Aborted {
        crate::agreements::ProductionTerms::from_definition(definition).fail(&mut expected);
    } else {
        for amount in services.unwrap_or(&stage.monthly_services) {
            *effects
                .entry(crate::pools::input_account(
                    world,
                    definition.id,
                    expected.operator,
                    amount.resource,
                ))
                .or_default() -= i64::from(amount.quantity);
        }
        if expected.elapsed == 0 {
            for amount in &stage.entry_inputs {
                *effects
                    .entry(crate::pools::input_account(
                        world,
                        definition.id,
                        expected.operator,
                        amount.resource,
                    ))
                    .or_default() -= i64::from(amount.quantity);
            }
        }
        expected.elapsed += 1;
        if expected.elapsed == stage.months {
            if expected.stage + 1 == definition.stages.len() {
                expected.status = Status::Completed;
                let (outputs, royalty) = crate::exchange::outputs(
                    world,
                    state,
                    &expected,
                    transaction.technique_use.as_ref(),
                )?;
                if transaction.royalty != royalty {
                    return Err("invalid output share receipt".into());
                }
                for effect in outputs {
                    *effects.entry(effect.account).or_default() += i64::from(effect.delta);
                }
            } else {
                expected.stage += 1;
                expected.elapsed = 0;
            }
        }
    }
    if expected.status != Status::Completed && transaction.royalty.is_some() {
        return Err("output share without completed output".into());
    }
    let mut actual = BTreeMap::<Account, i64>::new();
    for effect in &transaction.effects {
        *actual.entry(effect.account).or_default() += i64::from(effect.delta);
    }
    effects.retain(|_, value| *value != 0);
    actual.retain(|_, value| *value != 0);
    if &expected != after || actual != effects {
        return Err("process transition or effects disagree with definition".into());
    }
    Ok(())
}
