//! Upfront tool purchases or legacy output shares, plus bounded posted stock bids.
use crate::{activities::Outcome, equipment::TechniqueUse, model::*};
use std::collections::{BTreeMap, BTreeSet};

pub const DEFAULT_CAPTURE_PERCENT: u32 = 25;
const PERCENT_DENOMINATOR: i64 = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolRequest {
    pub buyer: AgentId,
    pub provider: AgentId,
    pub kind: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Market {
    pub seller_prices: BTreeMap<(AgentId, ResourceId), crate::forward::Price>,
    pub plots: Option<crate::plots::Policy>,
    pub cash: Option<crate::forward::Policy>,
    pub capture_percent: u32,
    pub tools: Vec<ToolRequest>,
    /// Posted stock bids stop at this buyer stock target.
    pub targets: BTreeMap<u32, i32>,
    /// Only these accounts offer stock; this much opening stock is protected.
    pub reserves: BTreeMap<Account, i32>,
    pub payment_caps: BTreeMap<Account, i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    pub purchase: Option<crate::forward::Purchase>,
    pub asset: AssetId,
    pub provider: AgentId,
    pub buyer: AgentId,
    pub capture_percent: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Royalty {
    pub provider: AgentId,
    pub operator: AgentId,
    pub process: u64,
    pub amounts: Vec<Amount>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExchangeState {
    pub forwards: BTreeMap<AssetId, crate::forward::Contract>,
    pub contracts: BTreeMap<AssetId, Delivery>,
    /// The first supplied tool used binds one provider for this process.
    pub contributors: BTreeMap<u64, AssetId>,
    pub earned: BTreeMap<Account, i64>,
}

pub fn missing(state: &State, r: &ToolRequest) -> bool {
    !state.terminal.contains_key(&r.buyer)
        && !state
            .equipment
            .values()
            .any(|a| a.owner == r.buyer && a.kind == r.kind && a.remaining_uses > 0)
}

pub fn wants(world: &World, state: &State, provider: AgentId, kind: u32) -> Option<bool> {
    let market = world.market.as_ref()?;
    let requests: Vec<_> = market
        .tools
        .iter()
        .filter(|r| r.provider == provider && r.kind == kind)
        .collect();
    if requests.is_empty() {
        return None;
    }
    Some(
        requests.iter().any(|r| missing(state, r))
            && !state
                .equipment
                .values()
                .any(|a| a.owner == provider && a.kind == kind && a.remaining_uses > 0),
    )
}

fn contributor(
    state: &State,
    p: &ProcessInstance,
    usage: Option<&TechniqueUse>,
) -> Option<AssetId> {
    state.exchange.contributors.get(&p.id).copied().or_else(|| {
        usage.and_then(|u| u.asset).filter(|id| {
            state
                .exchange
                .contracts
                .get(id)
                .is_some_and(|c| c.capture_percent > 0)
        })
    })
}

/// Return the complete output allocation. Recycling the process's own material
/// inputs is not earned output. No royalty is charged on durable/fulfillment outputs.
pub fn outputs(
    world: &World,
    state: &State,
    p: &ProcessInstance,
    usage: Option<&TechniqueUse>,
) -> Result<(Vec<Effect>, Option<Royalty>), String> {
    let d = world.definition(p.definition);
    let contract = contributor(state, p, usage).and_then(|id| state.exchange.contracts.get(&id));
    let mut effects = Vec::new();
    let mut amounts = Vec::new();
    for a in &d.outputs {
        let recycled: i64 = d
            .stages
            .iter()
            .flat_map(|s| &s.entry_inputs)
            .filter(|v| v.resource == a.resource)
            .map(|v| i64::from(v.quantity))
            .sum();
        let multiplier = usage
            .map(|u| {
                world
                    .techniques
                    .iter()
                    .find(|t| t.id == u.technique)
                    .expect("validated technique")
                    .output_multiplier
            })
            .unwrap_or(1);
        let base = i64::from(a.quantity);
        let earned = (base - recycled).max(0) * i64::from(multiplier);
        let produced =
            i32::try_from(base.min(recycled) + earned).map_err(|_| "technique output overflow")?;
        let share = if let Some(c) = contract {
            let numerator = earned * i64::from(c.capture_percent);
            if numerator % PERCENT_DENOMINATOR != 0 {
                return Err("stock precision cannot represent output share".into());
            }
            i32::try_from(numerator / PERCENT_DENOMINATOR).map_err(|_| "output share overflow")?
        } else {
            0
        };
        if share > 0 {
            let c = contract.unwrap();
            effects.push(Effect {
                account: (c.provider, a.resource),
                delta: share,
            });
            amounts.push(Amount::new(a.resource, share));
        }
        if produced > share {
            effects.push(Effect {
                account: (p.beneficiary, a.resource),
                delta: produced - share,
            });
        }
    }
    let receipt = contract.filter(|_| !amounts.is_empty()).map(|c| Royalty {
        provider: c.provider,
        operator: p.operator,
        process: p.id,
        amounts,
    });
    Ok((effects, receipt))
}

pub fn record(state: &mut State, t: &Transaction) {
    if let Some(crate::forward::Event::Delivery { contract, quantity }) = &t.forward {
        state.exchange.forwards.get_mut(contract).unwrap().delivered += quantity;
    }
    if let Some(p) = &t.process {
        if p.after.status != Status::Aborted
            && let Some(id) = contributor(state, &p.after, t.technique_use.as_ref())
        {
            state.exchange.contributors.insert(p.after.id, id);
        }
        if p.after.status != Status::Active {
            state.exchange.contributors.remove(&p.after.id);
        }
    }
    if let Some(r) = &t.royalty {
        for a in &r.amounts {
            *state
                .exchange
                .earned
                .entry((r.provider, a.resource))
                .or_default() += i64::from(a.quantity);
        }
    }
    if let Some(d) = &t.delivery {
        state.equipment.get_mut(&d.asset).unwrap().owner = d.buyer;
        state.exchange.contracts.insert(d.asset, d.clone());
        if let Some(purchase) = &d.purchase
            && let Some(contract) = &purchase.advance
        {
            state
                .exchange
                .forwards
                .insert(contract.id, contract.clone());
        }
    }
}

/// Reserve all transfers against opening stocks. Receipts within this batch
/// cannot finance another bid, and each durable can be delivered only once.
pub fn resolve(world: &World, state: &State) -> Result<Vec<Transaction>, String> {
    resolve_with(
        world,
        state,
        state.balances.clone(),
        crate::storage::usage(world, &state.balances),
    )
}
pub(crate) fn resolve_with(
    world: &World,
    state: &State,
    mut available: BTreeMap<Account, i32>,
    mut stored: BTreeMap<AgentId, i128>,
) -> Result<Vec<Transaction>, String> {
    let Some(market) = &world.market else {
        return Ok(vec![]);
    };
    let mut result = crate::forward::settle(world, state, &mut available, &mut stored)?;
    let mut purchased = BTreeSet::new();
    let mut quoted_state = state.clone();
    for transaction in &result {
        record(&mut quoted_state, transaction);
        for effect in &transaction.effects {
            *quoted_state.balances.entry(effect.account).or_default() += effect.delta;
        }
    }
    let mut reserved = BTreeSet::new();
    let mut requests = market.tools.clone();
    requests.sort_by_key(|r| (r.buyer, r.kind, r.provider));
    for r in requests {
        if !missing(state, &r)
            || state.terminal.contains_key(&r.provider)
            // Proceeding configurations admit this driver for existing delivery
            // servicing only; replacement purchases need another acquisition adapter.
            || !world.recovery.proceedings.is_empty()
        {
            continue;
        }
        if let Some(a) = state.equipment.values().find(|a| {
            a.owner == r.provider
                && a.kind == r.kind
                && a.remaining_uses > 0
                && a.attached_to.is_none()
                && a.last_used_month != Some(state.month)
                && !state.exchange.contracts.contains_key(&a.id)
                && !reserved.contains(&a.id)
        }) {
            if market.cash.is_some() {
                if purchased.contains(&r.buyer) {
                    continue;
                }
                let transaction =
                    crate::forward::purchase(world, &quoted_state, &r, a, &mut available)?;
                if transaction.delivery.is_some() {
                    reserved.insert(a.id);
                    purchased.insert(r.buyer);
                    record(&mut quoted_state, &transaction);
                }
                result.push(transaction);
                continue;
            }
            reserved.insert(a.id);
            result.push(Transaction {
                cause: "tool delivery against output share".into(),
                effects: Vec::new(),
                process: None,
                technique_use: None,
                trade: None,
                stock_trade: None,
                forward: None,
                royalty: None,
                delivery: Some(Delivery {
                    purchase: None,
                    asset: a.id,
                    provider: r.provider,
                    buyer: r.buyer,
                    capture_percent: market.capture_percent,
                }),
            });
        }
    }
    let household_protected = crate::commitments::protected_stock(world, state)?;
    let mut received: BTreeMap<Account, i32> = BTreeMap::new();
    let mut bids: Vec<_> = world.bids.iter().collect();
    bids.sort_by_key(|b| b.id);
    for bid in bids {
        let Some(target) = market.targets.get(&bid.id) else {
            continue;
        };
        for (&(seller, resource), reserve) in &market.reserves {
            if resource != bid.goods.resource || seller == bid.buyer {
                continue;
            }
            let reserve = &(*reserve).max(
                household_protected
                    .get(&(seller, resource))
                    .copied()
                    .unwrap_or(0),
            );
            let bid = crate::currency::terms(world, bid, seller)?;
            loop {
                let buyer_account = (bid.buyer, resource);
                let paid_account = (seller, bid.payment.resource);
                if market.payment_caps.get(&paid_account).is_some_and(|cap| {
                    state.balance(seller, bid.payment.resource)
                        + received.get(&paid_account).copied().unwrap_or(0)
                        + bid.payment.quantity
                        > *cap
                }) {
                    break;
                }
                if state.balance(bid.buyer, resource)
                    + received.get(&buyer_account).copied().unwrap_or(0)
                    + bid.goods.quantity
                    > *target
                    || available.get(&(seller, resource)).copied().unwrap_or(0) - reserve
                        < bid.goods.quantity
                            + crate::forward::pledged(&quoted_state, seller, resource)
                    || available
                        .get(&(bid.buyer, bid.payment.resource))
                        .copied()
                        .unwrap_or(0)
                        < bid.payment.quantity
                {
                    break;
                }
                let Ok(t) = crate::currency::transaction(
                    world,
                    state,
                    crate::currency::StockTrade {
                        bid: bid.id,
                        seller,
                    },
                ) else {
                    break;
                };
                let stored_effects =
                    crate::households::with_contributions(world, state, &t.effects)?;
                if !crate::storage::fits(world, &stored, &stored_effects) {
                    break;
                }
                for e in &t.effects {
                    if e.delta < 0 {
                        *available.entry(e.account).or_default() += e.delta;
                    }
                }
                for e in &t.effects {
                    if e.delta > 0 {
                        *received.entry(e.account).or_default() += e.delta;
                    }
                }
                crate::storage::apply(world, &mut stored, &stored_effects);
                result.push(t);
            }
        }
    }
    Ok(result)
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    crate::forward::validate(world, state)?;
    crate::plots::validate(world, state)?;
    let Some(market) = &world.market else {
        return if state.exchange == ExchangeState::default() {
            Ok(())
        } else {
            Err("exchange state without market".into())
        };
    };
    if market.capture_percent > PERCENT_DENOMINATOR as u32
        || world.priority == Priority::ConsequenceAware
    {
        return Err("invalid output share or unsupported market forecast policy".into());
    }
    for d in world
        .definitions
        .iter()
        .filter(|d| d.execution == Execution::Productive)
    {
        for a in &d.outputs {
            let recycled: i64 = d
                .stages
                .iter()
                .flat_map(|s| &s.entry_inputs)
                .filter(|v| v.resource == a.resource)
                .map(|v| i64::from(v.quantity))
                .sum();
            if (i64::from(a.quantity) - recycled).max(0) * i64::from(market.capture_percent)
                % PERCENT_DENOMINATOR
                != 0
            {
                return Err("stock precision cannot represent output share".into());
            }
        }
    }
    for (&(seller, resource), price) in &market.seller_prices {
        if price.goods <= 0
            || price.coins <= 0
            || !world.agents.iter().any(|a| a.id == seller)
            || !world
                .resources
                .iter()
                .any(|r| r.id == resource && r.kind == ResourceKind::Stock)
        {
            return Err("invalid posted seller price".into());
        }
    }
    for (&(owner, resource), cap) in &market.payment_caps {
        if *cap < 0
            || !world.agents.iter().any(|a| a.id == owner)
            || !world
                .resources
                .iter()
                .any(|r| r.id == resource && r.kind == ResourceKind::Stock)
        {
            return Err("invalid seller payment cap".into());
        }
    }
    let mut requests = BTreeSet::new();
    for r in &market.tools {
        if r.provider == r.buyer
            || !requests.insert((r.buyer, r.kind))
            || ![r.buyer, r.provider]
                .iter()
                .all(|id| world.participants.iter().any(|p| p.agent == *id))
            || world
                .activities
                .kinds
                .get(&r.kind)
                .is_none_or(|k| k.attached)
            || !world
                .activities
                .outcomes
                .values()
                .any(|v| *v == Outcome::Create(r.kind))
        {
            return Err("invalid tool request".into());
        }
    }
    for (id, target) in &market.targets {
        if *target < 0 || !world.bids.iter().any(|b| b.id == *id) {
            return Err("invalid market stock target".into());
        }
    }
    for (&(owner, resource), reserve) in &market.reserves {
        if *reserve < 0
            || !world.agents.iter().any(|a| a.id == owner)
            || !world
                .resources
                .iter()
                .any(|r| r.id == resource && r.kind == ResourceKind::Stock)
        {
            return Err("invalid sale reserve".into());
        }
    }
    for (&id, c) in &state.exchange.contracts {
        if id != c.asset
            || c.capture_percent
                != if market.cash.is_some() {
                    0
                } else {
                    market.capture_percent
                }
            || !state.equipment.get(&id).is_some_and(|a| {
                a.owner == c.buyer
                    && market
                        .tools
                        .iter()
                        .any(|r| r.buyer == c.buyer && r.provider == c.provider && r.kind == a.kind)
            })
        {
            return Err("invalid tool output-share contract".into());
        }
    }
    for (&id, asset) in &state.exchange.contributors {
        if !state.processes.get(&id).is_some_and(|p| {
            p.status == Status::Active
                && state
                    .exchange
                    .contracts
                    .get(asset)
                    .is_some_and(|c| c.buyer == p.operator)
        }) {
            return Err("invalid process contributor".into());
        }
    }
    Ok(())
}

/// The simple need planner must not promise away the provider's share. For a new
/// candidate, conservatively assume an available contracted technique will be used.
pub fn retained_outputs(
    world: &World,
    state: &State,
    agent: AgentId,
    definition: &ProcessDefinition,
    process: Option<&ProcessInstance>,
) -> Vec<Amount> {
    let contracted = process
        .and_then(|p| contributor(state, p, None))
        .and_then(|id| state.exchange.contracts.get(&id))
        .or_else(|| {
            state.equipment.values().find_map(|a| {
                (a.owner == agent
                    && world.techniques.iter().any(|t| {
                        t.definition == definition.id
                            && t.equipment_kind == Some(a.kind)
                            && a.remaining_uses >= t.wear
                            && crate::equipment::eligible(t, state, agent)
                    }))
                .then(|| state.exchange.contracts.get(&a.id))
                .flatten()
            })
        });
    definition
        .outputs
        .iter()
        .map(|a| {
            let recycled: i64 = definition
                .stages
                .iter()
                .flat_map(|s| &s.entry_inputs)
                .filter(|v| v.resource == a.resource)
                .map(|v| i64::from(v.quantity))
                .sum();
            let share = contracted.map_or(0, |c| {
                (i64::from(a.quantity) - recycled).max(0) * i64::from(c.capture_percent)
                    / PERCENT_DENOMINATOR
            });
            Amount::new(a.resource, a.quantity - share as i32)
        })
        .collect()
}
