//! Shared Acquire boundary: credit, direct prepaid deliveries, then spot exchange.
//! Receipts describe one atomic batch; incoming funds cannot finance another leg.
use crate::{credit, model::*, negotiation, storage};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct Resources {
    pub available: BTreeMap<Account, i32>,
    pub holdings: BTreeMap<Account, i32>,
    pub storage: BTreeMap<AgentId, i128>,
    pub pooling: Option<crate::households::income_reservations::Reservations>,
}
impl Resources {
    pub fn opening(world: &World, state: &State) -> Self {
        Self {
            pooling: None,
            available: state.balances.clone(),
            holdings: state.balances.clone(),
            storage: storage::usage(world, &state.balances),
        }
    }
    pub fn fits(&self, world: &World, effects: &[Effect]) -> Result<bool, String> {
        if !storage::fits(world, &self.storage, effects) {
            return Ok(false);
        }
        match &self.pooling {
            Some(p) => Ok(p.preview(world, effects)?.is_some()),
            None => Ok(true),
        }
    }
    pub fn reserve(&mut self, world: &World, transactions: &[Transaction]) -> Result<(), String> {
        let mut net = BTreeMap::<Account, i128>::new();
        for t in transactions {
            if let Some(p) = &self.pooling {
                self.pooling = Some(
                    p.preview(world, &t.effects)?
                        .ok_or("acquisition exceeds pooled storage capacity")?,
                );
            }
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

pub(crate) fn shared(world: &World) -> bool {
    world.minting.is_some()
        || crate::forward::direct::enabled(world)
        || (credit::enabled(world)
            && (world.negotiation.is_some()
                || world.market.is_some()
                || world.town_market.is_some()))
}

/// Credit, direct forward collection/admission, then spot trades reserve the
/// same opening pool. This is an explicit allocation rule, not a scheduler change.
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
        credit::record(&mut quoted, c);
    }
    let forwards = crate::forward::direct::evaluate(world, &quoted, &mut resources)?;
    batch.transactions.extend(forwards.transactions);
    batch.forward_collections = forwards.receipts;
    quoted.balances = resources.holdings.clone();
    for t in &batch.transactions {
        if t.forward.is_some() {
            crate::exchange::record(&mut quoted, t);
        }
    }
    if world.minting.is_some() {
        batch.minting = crate::minting::evaluate_with(world, &quoted, &resources)?;
        if let Some(m) = &batch.minting {
            resources.reserve(world, &m.transactions)?;
            batch.transactions.extend(m.transactions.clone());
        }
        return Ok(batch);
    }
    if world.town_market.is_some() {
        let round = crate::town_market::evaluate_with(world, &quoted, &resources)?;
        resources.reserve(world, &round.transactions)?;
        batch.transactions.extend(round.transactions.clone());
        batch.town_market = Some(crate::town_market::Boundary::Market(round));
        return Ok(batch);
    }
    batch.negotiation = negotiation::evaluate_with(world, &quoted, &resources)?;
    let trades = negotiation::transactions(world, state, &batch.negotiation)?;
    resources.reserve(world, &trades)?;
    batch.transactions.extend(trades);
    let exchange = if crate::forward::direct::enabled(world) {
        crate::exchange::after_collections
    } else {
        crate::exchange::resolve_with
    };
    let trades = exchange(
        world,
        &quoted,
        resources.available.clone(),
        resources.storage.clone(),
    )?;
    resources.reserve(world, &trades)?;
    batch.transactions.extend(trades);
    batch.plot_request = crate::plots::after_acquisition(world, state, &batch)?;
    batch.accept_access = batch
        .plot_request
        .as_ref()
        .filter(|r| r.reason == crate::plots::Reason::Accepted)
        .and_then(|r| r.offer);
    Ok(batch)
}

pub(crate) fn validate_batch(world: &World, state: &State, batch: &Batch) -> Result<(), String> {
    if state.phase == Phase::Acquire && shared(world) {
        let expected = evaluate(world, state)?;
        if batch.forward_collections != expected.forward_collections
            || batch.credit != expected.credit
            || batch.minting != expected.minting
            || batch.town_market != expected.town_market
            || batch.negotiation != expected.negotiation
            || batch.production_plan != expected.production_plan
            || batch.plot_request != expected.plot_request
            || batch.accept_access != expected.accept_access
            || batch.transactions != expected.transactions
        {
            return Err("missing or altered shared acquisition settlement".into());
        }
        Ok(())
    } else {
        if !batch.forward_collections.is_empty() {
            return Err("forward collection receipts outside shared acquisition".into());
        }
        negotiation::validate_batch(world, state, batch)?;
        credit::validate_batch(world, state, batch)
    }
}
