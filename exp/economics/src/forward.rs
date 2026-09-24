//! Prepaid commodity forwards finance atomic, upfront tool purchases.
use crate::finance::{self, Condition, FailureRule, Transfer};
use crate::{
    compute::Backend,
    equipment::DurableAsset,
    exchange::{Delivery, ToolRequest},
    model::*,
    simulation::Simulation,
};
use std::collections::{BTreeMap, BTreeSet};

pub const PROJECTION_MONTHS: u32 = 12;
const MAX_PROJECTION_MONTHS: u32 = 24;
const PERCENT: i64 = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Price {
    pub goods: i32,
    pub coins: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub lender: AgentId,
    pub coin: ResourceId,
    pub months: u32,
    pub enabled: bool,
    pub prices: BTreeMap<ResourceId, Price>,
    pub advance_prices: BTreeMap<ResourceId, Price>,
    pub protected: BTreeMap<ResourceId, i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection {
    pub incremental_value: i32,
    pub from: u32,
    pub through: u32,
    /// Realized in a hypothetical local rollout using the offered tool.
    pub assisted: BTreeMap<ResourceId, i32>,
    /// Positive stock growth after local inputs, consumption and existing taxes.
    pub surplus: BTreeMap<ResourceId, i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    pub id: AssetId,
    pub debtor: AgentId,
    pub creditor: AgentId,
    pub issued: u32,
    pub due: u32,
    pub goods: Amount,
    pub price: Price,
    pub advance: Amount,
    pub delivered: i32,
    /// Accepted relief applied at Due; never recorded as physical delivery.
    pub relief: Vec<crate::delivery_relief::Applied>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Purchase {
    pub price: Amount,
    pub projection: Projection,
    pub advance: Option<Contract>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    NoProjectedUse,
    NotEconomic,
    NoCoinFunding,
    AdvancesDisabled,
    ExistingForward,
    NoProjectedSurplus,
    TreasuryShortfall,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Rejected {
        buyer: AgentId,
        asset: AssetId,
        price: i32,
        reason: Reason,
    },
    Delivery {
        contract: AssetId,
        quantity: i32,
    },
}

impl Contract {
    pub fn effective_due(&self) -> u32 {
        self.relief
            .iter()
            .fold(self.due, |due, r| match r.terms.action {
                crate::delivery_relief::Action::Extend { due } => due,
                crate::delivery_relief::Action::WriteOff { .. } => due,
            })
    }
    pub fn written_off(&self) -> i32 {
        self.relief
            .iter()
            .map(|r| match r.terms.action {
                crate::delivery_relief::Action::WriteOff { quantity } => quantity,
                _ => 0,
            })
            .sum()
    }
    pub fn claim(&self) -> finance::Obligation {
        finance::Obligation {
            transfer: Transfer {
                from: self.debtor,
                to: self.creditor,
                amount: self.goods.clone(),
            },
            settled: self.delivered + self.written_off(),
            condition: Condition::OnOrAfterMonth(self.effective_due()),
            failure: FailureRule::BlockNewAdvance,
        }
    }
}

pub fn policy(world: &World) -> Option<&Policy> {
    world.market.as_ref().and_then(|m| m.cash.as_ref())
}
pub fn pledged(state: &State, debtor: AgentId, resource: ResourceId) -> i32 {
    state
        .exchange
        .forwards
        .values()
        .filter(|c| c.debtor == debtor && c.goods.resource == resource)
        .map(|c| c.claim().outstanding())
        .sum()
}

fn transaction(cause: &str) -> Transaction {
    Transaction {
        cause: cause.into(),
        effects: Vec::new(),
        process: None,
        technique_use: None,
        trade: None,
        stock_trade: None,
        delivery: None,
        royalty: None,
        forward: None,
    }
}
fn rejected(buyer: AgentId, asset: AssetId, price: i32, reason: Reason) -> Transaction {
    let mut t = transaction("tool purchase declined");
    t.forward = Some(Event::Rejected {
        buyer,
        asset,
        price,
        reason,
    });
    t
}

/// A bounded one-agent projection, not a perfect observation of future world state.
/// Shared pools are observed at opening; competition and later shocks can disappoint.
pub fn project(
    world: &World,
    state: &State,
    buyer: AgentId,
    asset: AssetId,
) -> Result<Projection, String> {
    let config = policy(world).ok_or("missing forward policy")?;
    let mut opening = state.clone();
    opening
        .equipment
        .get_mut(&asset)
        .ok_or("missing offered tool")?
        .owner = buyer;
    let (w, s) = local(world, &opening, buyer);
    let from = s.month;
    let mut manual = s.clone();
    manual.equipment.remove(&asset);
    let mut control = Simulation::new(w.clone(), manual, Backend::Reference)?;
    control.run_months(config.months)?;
    let mut sim = Simulation::new(w, s, Backend::Reference)?;
    sim.run_months(config.months)?;
    let mut assisted = BTreeMap::<ResourceId, i32>::new();
    let mut touched = BTreeSet::new();
    for b in &sim.ledger {
        for t in &b.transactions {
            let Some(change) = &t.process else {
                continue;
            };
            if t.technique_use.as_ref().and_then(|u| u.asset) == Some(asset) {
                touched.insert(change.after.id);
            }
            if change.after.status == Status::Completed && touched.contains(&change.after.id) {
                let d = world.definition(change.after.definition);
                for output in &d.outputs {
                    let recycled: i32 = d
                        .stages
                        .iter()
                        .flat_map(|s| &s.entry_inputs)
                        .filter(|a| a.resource == output.resource)
                        .map(|a| a.quantity)
                        .sum();
                    let produced: i32 = t
                        .effects
                        .iter()
                        .filter(|e| e.account.1 == output.resource && e.delta > 0)
                        .map(|e| e.delta)
                        .sum();
                    let net = (produced - recycled).max(0);
                    if net > 0 && config.prices.contains_key(&output.resource) {
                        let value = assisted.entry(output.resource).or_default();
                        *value = value.checked_add(net).ok_or("projected output overflow")?;
                    }
                }
            }
        }
    }
    let surplus = config
        .prices
        .keys()
        .filter_map(|&r| {
            let growth = (sim.state.balance(buyer, r) - state.balance(buyer, r)).max(0);
            (growth > 0).then_some((r, growth))
        })
        .collect();
    let incremental = config
        .prices
        .iter()
        .map(|(r, p)| {
            (i64::from(sim.state.balance(buyer, *r)) - i64::from(control.state.balance(buyer, *r)))
                * i64::from(p.coins)
                / i64::from(p.goods)
        })
        .sum::<i64>()
        + i64::from(sim.state.balance(buyer, config.coin))
        - i64::from(control.state.balance(buyer, config.coin));
    Ok(Projection {
        incremental_value: i32::try_from(incremental).map_err(|_| "incremental value overflow")?,
        from,
        through: sim.state.month - 1,
        assisted,
        surplus,
    })
}

pub fn quote(policy: &Policy, projection: &Projection, percent: u32) -> Result<i32, String> {
    let value = projection.assisted.iter().try_fold(0i64, |sum, (r, q)| {
        let p = policy.prices.get(r).ok_or("unknown projected commodity")?;
        if *q < 0 || p.goods <= 0 || p.coins <= 0 {
            return Err("invalid projected value");
        }
        sum.checked_add(i64::from(*q) * i64::from(p.coins) / i64::from(p.goods))
            .ok_or("projected value overflow")
    })?;
    i32::try_from(
        value
            .checked_mul(i64::from(percent))
            .ok_or("quote overflow")?
            / PERCENT,
    )
    .map_err(|_| "quote overflow".into())
}

pub fn purchase(
    world: &World,
    state: &State,
    request: &ToolRequest,
    asset: &DurableAsset,
    available: &mut BTreeMap<Account, i32>,
) -> Result<Transaction, String> {
    let config = policy(world).ok_or("missing forward policy")?;
    let buyer = request.buyer;
    let cash = available.get(&(buyer, config.coin)).copied().unwrap_or(0);
    let treasury = available
        .get(&(config.lender, config.coin))
        .copied()
        .unwrap_or(0);
    if cash == 0 && treasury == 0 {
        return Ok(rejected(buyer, asset.id, 0, Reason::NoCoinFunding));
    }
    let projection = project(world, state, buyer, asset.id)?;
    let price = quote(
        config,
        &projection,
        world.market.as_ref().unwrap().capture_percent,
    )?;
    if price == 0 {
        return Ok(rejected(buyer, asset.id, price, Reason::NoProjectedUse));
    }
    let own = price.min(cash);
    let gap = price - own;
    let mut advance = None;
    if gap > 0 {
        if !config.enabled {
            return Ok(rejected(buyer, asset.id, price, Reason::AdvancesDisabled));
        }
        if state
            .exchange
            .forwards
            .values()
            .any(|c| c.debtor == buyer && c.claim().blocks(FailureRule::BlockNewAdvance))
        {
            return Ok(rejected(buyer, asset.id, price, Reason::ExistingForward));
        }
        if treasury < gap {
            return Ok(rejected(buyer, asset.id, price, Reason::TreasuryShortfall));
        }
        let mut candidates: Vec<_> = projection.surplus.iter().collect();
        candidates.sort_by_key(|(resource, _)| **resource);
        for (&resource, &surplus) in candidates {
            let rate = &config.advance_prices[&resource];
            let numerator = i64::from(gap) * i64::from(rate.goods);
            if numerator % i64::from(rate.coins) != 0 {
                continue;
            }
            let quantity = i32::try_from(numerator / i64::from(rate.coins))
                .map_err(|_| "forward quantity overflow")?;
            if quantity > surplus {
                continue;
            }
            // Outstanding receipts reserve finite creditor storage, alongside
            // physical stocks; the real delivery still rechecks current space.
            let outstanding: i64 = state
                .exchange
                .forwards
                .values()
                .filter(|c| c.creditor == config.lender)
                .map(|c| {
                    i64::from(c.claim().outstanding())
                        * i64::from(*world.storage.weights.get(&c.goods.resource).unwrap_or(&0))
                })
                .sum();
            let used = crate::storage::usage(world, &state.balances);
            let added = i64::from(quantity)
                * i64::from(*world.storage.weights.get(&resource).unwrap_or(&0));
            if world
                .storage
                .capacities
                .get(&config.lender)
                .is_some_and(|cap| {
                    used.get(&config.lender).copied().unwrap_or(0) + i128::from(outstanding + added)
                        > i128::from(*cap)
                })
            {
                continue;
            }
            advance = Some(Contract {
                id: asset.id,
                debtor: buyer,
                creditor: config.lender,
                issued: state.month,
                due: state
                    .month
                    .checked_add(config.months)
                    .ok_or("forward due overflow")?,
                goods: Amount::new(resource, quantity),
                price: rate.clone(),
                advance: Amount::new(config.coin, gap),
                delivered: 0,
                relief: vec![],
            });
            break;
        }
        if advance.is_none() {
            return Ok(rejected(buyer, asset.id, price, Reason::NoProjectedSurplus));
        }
    }
    let financing_cost = advance
        .as_ref()
        .map(|c| {
            let spot = &config.prices[&c.goods.resource];
            i64::from(c.goods.quantity) * i64::from(spot.coins) / i64::from(spot.goods)
        })
        .unwrap_or(0);
    if i64::from(projection.incremental_value) <= i64::from(own) + financing_cost {
        return Ok(rejected(buyer, asset.id, price, Reason::NotEconomic));
    }
    let mut t = transaction("upfront tool purchase; advance paid to provider on buyer's behalf");
    if own > 0 {
        t.effects.push(Effect {
            account: (buyer, config.coin),
            delta: -own,
        });
    }
    if gap > 0 {
        t.effects.push(Effect {
            account: (config.lender, config.coin),
            delta: -gap,
        });
    }
    t.effects.push(Effect {
        account: (request.provider, config.coin),
        delta: price,
    });
    *available.entry((buyer, config.coin)).or_default() -= own;
    *available.entry((config.lender, config.coin)).or_default() -= gap;
    t.delivery = Some(Delivery {
        asset: asset.id,
        provider: request.provider,
        buyer,
        capture_percent: 0,
        purchase: Some(Purchase {
            price: Amount::new(config.coin, price),
            projection,
            advance,
        }),
    });
    Ok(t)
}

/// Due commodity delivery precedes new purchases; annual taxes have already settled.
/// An unmet balance remains a dated obligation, never a fictional stock receipt.
pub fn settle(
    world: &World,
    state: &State,
    available: &mut BTreeMap<Account, i32>,
    stored: &mut BTreeMap<AgentId, i128>,
) -> Result<Vec<Transaction>, String> {
    let Some(config) = policy(world) else {
        return Ok(Vec::new());
    };
    let household_protected = crate::commitments::protected_stock(world, state)?;
    let mut contracts: Vec<_> = state
        .exchange
        .forwards
        .values()
        .filter(|c| c.effective_due() <= state.month)
        .collect();
    contracts.sort_by_key(|c| {
        (
            world
                .claim_priorities
                .get(&finance::ContractId::Forward(c.id))
                .copied()
                .unwrap_or(finance::DEFAULT_CLAIM_RANK),
            c.effective_due(),
            c.id,
        )
    });
    let mut result = Vec::new();
    let mut execution = finance::Execution::from_parts(available.clone(), stored.clone());
    for c in contracts {
        let account = (c.debtor, c.goods.resource);
        let protected = config
            .protected
            .get(&c.goods.resource)
            .copied()
            .unwrap_or(0)
            .max(household_protected.get(&account).copied().unwrap_or(0));
        let claim = c.claim();
        let payment = execution.pay_protected(world, state.month, &claim, protected)?;
        let quantity = payment.paid;
        if quantity == 0 {
            continue;
        }
        let mut t = transaction("deliver prepaid forward commodity");
        t.forward = Some(Event::Delivery {
            contract: c.id,
            quantity,
        });
        t.effects = payment.effects;
        result.push(t);
    }
    *available = execution.available;
    *stored = execution.stored;
    Ok(result)
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let Some(config) = policy(world) else {
        return if state.exchange.forwards.is_empty()
            && state
                .exchange
                .contracts
                .values()
                .all(|d| d.purchase.is_none())
        {
            Ok(())
        } else {
            Err("cash contracts without policy".into())
        };
    };
    let stock = |id| {
        world
            .resources
            .iter()
            .any(|r| r.id == id && r.kind == ResourceKind::Stock)
    };
    if !world.agents.iter().any(|a| a.id == config.lender)
        || !stock(config.coin)
        || config.months == 0
        || config.months > MAX_PROJECTION_MONTHS
        || *world.storage.weights.get(&config.coin).unwrap_or(&0) != 0
    {
        return Err("invalid forward policy".into());
    }
    if config.protected.iter().any(|(r, q)| !stock(*r) || *q < 0) {
        return Err("invalid forward protected stock".into());
    }
    for (&resource, price) in &config.prices {
        if !stock(resource)
            || resource == config.coin
            || price.goods <= 0
            || price.coins <= 0
            || !world.bids.iter().any(|b| {
                b.buyer == config.lender
                    && b.goods.resource == resource
                    && b.payment.resource == config.coin
                    && i64::from(b.goods.quantity) * i64::from(price.coins)
                        == i64::from(b.payment.quantity) * i64::from(price.goods)
            })
        {
            return Err("valuation price must equal a state spot purchase price".into());
        }
    }
    if config.advance_prices.len() != config.prices.len()
        || config
            .advance_prices
            .iter()
            .any(|(r, p)| !config.prices.contains_key(r) || p.goods <= 0 || p.coins <= 0)
    {
        return Err("invalid forward advance prices".into());
    }
    for (&id, c) in &state.exchange.forwards {
        crate::delivery_relief::validate_history(world, state, c)?;
        if id != c.id
            || c.creditor != config.lender
            || c.goods.quantity <= 0
            || c.advance.resource != config.coin
            || c.advance.quantity <= 0
            || c.delivered < 0
            || c.delivered > c.goods.quantity
            || c.issued > state.month
            || Some(c.due) != c.issued.checked_add(config.months)
            || config.advance_prices.get(&c.goods.resource) != Some(&c.price)
            || i64::from(c.goods.quantity) * i64::from(c.price.coins)
                != i64::from(c.advance.quantity) * i64::from(c.price.goods)
            || !state
                .exchange
                .contracts
                .get(&id)
                .and_then(|d| d.purchase.as_ref())
                .is_some_and(|p| {
                    p.advance.as_ref().is_some_and(|a| {
                        let mut expected = a.clone();
                        expected.delivered = c.delivered;
                        expected.relief = c.relief.clone();
                        expected == *c
                    })
                })
        {
            return Err("invalid prepaid production contract".into());
        }
    }
    for d in state.exchange.contracts.values() {
        let p = d
            .purchase
            .as_ref()
            .ok_or("cash delivery without purchase")?;
        if p.advance.as_ref().is_some_and(|a| {
            a.id != d.asset
                || a.debtor != d.buyer
                || a.issued != p.projection.from
                || a.delivered != 0
                || !a.relief.is_empty()
                || a.advance.quantity > p.price.quantity
                || p.projection
                    .surplus
                    .get(&a.goods.resource)
                    .copied()
                    .unwrap_or(0)
                    < a.goods.quantity
                || !state.exchange.forwards.contains_key(&a.id)
        }) || d.capture_percent != 0
            || p.price.resource != config.coin
            || p.price.quantity <= 0
            || quote(
                config,
                &p.projection,
                world.market.as_ref().unwrap().capture_percent,
            )? != p.price.quantity
            || p.projection.through.checked_add(1) != p.projection.from.checked_add(config.months)
        {
            return Err("invalid upfront quote".into());
        }
    }
    Ok(())
}

/// Isolate observed local production and accepted rights for bounded forecasts.
pub(crate) fn local(world: &World, state: &State, buyer: AgentId) -> (World, State) {
    let mut w = world.clone();
    let mut s = state.clone();
    w.market = None;
    s.exchange = Default::default();
    w.bids.clear();
    w.offers.clear();
    s.filled_offers.clear();
    let accepted = s.accepted_agreements.clone();
    w.rights.retain(|r| {
        !w.access_offers.iter().any(|o| o.right == r.id)
            || accepted.values().any(|a| a.right == r.id)
    });
    for a in accepted.values() {
        if let Some(r) = w.rights.iter_mut().find(|r| r.id == a.right) {
            r.from = a.activated;
        }
        w.agreements.push(a.clone());
    }
    w.access_offers.clear();
    s.accepted_agreements.clear();
    // Individual underwriting excludes future household support. Preserve any
    // shared space already occupied at this observed boundary; stripping the
    // household must not make the starting snapshot spuriously over capacity.
    let used = crate::storage::usage(world, &state.balances);
    for home in &world.households {
        for member in &home.adults {
            if let Some(cap) = w.storage.capacities.get_mut(member) {
                let occupied = used.get(member).copied().unwrap_or(0);
                *cap = (*cap).max(i32::try_from(occupied).unwrap_or(i32::MAX));
            }
        }
    }
    w.households.clear();
    s.household_remainders.clear();
    w.participants.retain(|p| p.agent == buyer);
    w.condition_rules.retain(|r| r.subject == buyer);
    s.conditions.retain(|(agent, _), _| *agent == buyer);
    s.terminal.retain(|agent, _| *agent == buyer);
    w.activities.orders.retain(|o| o.agent == buyer);
    w.agreements.retain(|a| a.debtor == buyer);
    w.issuance
        .retain(|r| w.agreements.iter().any(|a| a.id == r.agreement));
    w.activities
        .coin_payments
        .retain(|id, _| w.agreements.iter().any(|a| a.id == *id));
    s.obligations
        .retain(|(id, _), _| w.agreements.iter().any(|a| a.id == *id));
    // Do not look ahead at scripted future capacities or starts.
    w.capacity_overrides.clear();
    w.scheduled_starts.clear();
    s.pending_production = None;
    s.equipment.retain(|_, a| a.owner == buyer);
    // Retain an ID watermark so hypothetical output assets cannot collide with
    // existing assets. Other agents' work is excluded from this local forecast.
    let watermark = s.processes.keys().next_back().copied();
    s.processes.retain(|id, p| {
        (p.operator == buyer && p.status == Status::Active) || Some(*id) == watermark
    });
    if let Some(id) = watermark
        && let Some(p) = s.processes.get_mut(&id)
        && p.operator != buyer
    {
        p.goal = None;
        if p.status == Status::Active {
            p.status = Status::Aborted;
        }
    }
    (w, s)
}
