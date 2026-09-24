//! Shared Acquire boundary: credit reservations precede bilateral exchange.
//! Receipts describe one atomic batch; incoming funds cannot finance another leg.
use crate::{credit, model::*, negotiation, storage};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct Resources {
    pub available: BTreeMap<Account, i32>,
    pub holdings: BTreeMap<Account, i32>,
    pub storage: BTreeMap<AgentId, i128>,
}
impl Resources {
    pub fn opening(world: &World, state: &State) -> Self {
        Self {
            available: state.balances.clone(),
            holdings: state.balances.clone(),
            storage: storage::usage(world, &state.balances),
        }
    }
    pub fn reserve(&mut self, world: &World, transactions: &[Transaction]) -> Result<(), String> {
        let mut net = BTreeMap::<Account, i128>::new();
        for t in transactions {
            for e in &t.effects {
                *net.entry(e.account).or_default() += i128::from(e.delta);
                if e.delta < 0 {
                    let balance = self.available.entry(e.account).or_default();
                    *balance = balance
                        .checked_add(e.delta)
                        .filter(|n| *n >= 0)
                        .ok_or("acquisition exceeds opening resources")?;
                }
            }
            storage::apply(world, &mut self.storage, &t.effects);
        }
        for (account, delta) in net {
            let holding = self.holdings.entry(account).or_default();
            *holding = i32::try_from(i128::from(*holding) + delta)
                .map_err(|_| "acquisition holding overflow")?;
        }
        if !storage::fits(world, &self.storage, &[]) {
            return Err("acquisition exceeds storage capacity".into());
        }
        Ok(())
    }
}

/// Credit first is the explicit scoped allocation rule, not a scheduler change.
/// Quote discovery observes opening state and only the remaining spendable budget.
pub fn evaluate(world: &World, state: &State) -> Result<Batch, String> {
    if state.phase != Phase::Acquire {
        return Err("acquisition resolver requires Acquire".into());
    }
    let mut batch = Batch::empty(state);
    batch.credit = credit::evaluate(world, state)?;
    if let Some(c) = &batch.credit {
        batch.transactions = c.transactions.clone();
        batch.production_plan = c.production_plan.clone();
    }
    let mut resources = Resources::opening(world, state);
    resources.reserve(world, &batch.transactions)?;
    // Later underwriting sees the accepted liabilities/control changes, but
    // receives the separately reserved opening budget for actual settlement.
    let mut quoted = state.clone();
    quoted.balances = resources.holdings.clone();
    if let Some(c) = &batch.credit {
        quoted.credit = c.after.clone();
        for change in &c.attachments {
            quoted
                .processes
                .insert(change.after.id, change.after.clone());
        }
    }
    batch.negotiation = negotiation::evaluate_with(world, &quoted, &resources)?;
    let trades = negotiation::transactions(world, state, &batch.negotiation)?;
    resources.reserve(world, &trades)?;
    batch.transactions.extend(trades);
    let trades = crate::exchange::resolve_with(
        world,
        &quoted,
        resources.available.clone(),
        resources.storage.clone(),
    )?;
    resources.reserve(world, &trades)?;
    batch.transactions.extend(trades);
    Ok(batch)
}

pub(crate) fn validate_batch(world: &World, state: &State, batch: &Batch) -> Result<(), String> {
    if state.phase == Phase::Acquire
        && crate::credit::enabled(world)
        && (world.negotiation.is_some() || world.market.is_some())
    {
        let expected = evaluate(world, state)?;
        if batch.credit != expected.credit
            || batch.negotiation != expected.negotiation
            || batch.production_plan != expected.production_plan
            || batch.transactions != expected.transactions
        {
            return Err("missing or altered shared acquisition settlement".into());
        }
        Ok(())
    } else {
        negotiation::validate_batch(world, state, batch)?;
        credit::validate_batch(world, state, batch)
    }
}
