//! Scoped posted stock bid: retain needs/input reserves, then sell bounded surplus.
use crate::{credit, currency, model::*, storage};
use std::collections::BTreeMap;

const MAX_LOTS_PER_MONTH: i32 = 64;
const MAX_RESERVE_MONTHS: u32 = 24;
const SALE_BID: u32 = 1;
const MONTHLY_LOTS: i32 = 2;
const PURCHASE_BUDGET: i32 = 12_000;
const LIMITED_BUDGET: i32 = 1_200;
const NORMAL_PRICE: i32 = 1_200;
const FOOD_TIGHT_PRICE: i32 = 600;
const RESERVE_MONTHS: u32 = 6;
const BRIDGE_COINS: i32 = 6_500;
const LOAN_MONTHS: u32 = 12;
const FORECAST_MONTHS: u32 = 18;
const CROP_GRAIN: i32 = 12;
const STORE_CAPACITY: i32 = 100;
// Ownership-following cultivation permission, independent of the observation window.
const CULTIVATION_RIGHT_MONTHS: u32 = 120;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub bid: u32,
    pub seller: AgentId,
    pub reserve_months: u32,
    pub max_lots_per_month: i32,
    /// Total posted coin allowance across months; not an escrow or new money.
    pub purchase_budget: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub bid: u32,
    pub seller: AgentId,
    pub reserve: i32,
    pub opening_stock: i32,
    pub desired_lots: i32,
    pub monthly_limit: i32,
    pub funding_limit: i32,
    pub storage_limit: i32,
    pub sold_lots: i32,
    pub goods: i32,
    pub coins: i32,
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let Some(c) = &world.credit else {
        return Ok(());
    };
    let Some(p) = &c.stock_sales else {
        return if state.credit.stock_spent == 0 {
            Ok(())
        } else {
            Err("stock spending without posted-sale policy".into())
        };
    };
    let bid = world
        .bids
        .iter()
        .find(|b| b.id == p.bid)
        .ok_or("missing posted stock bid")?;
    if world.bids.len() != 1
        || p.seller != c.application.buyer
        || p.seller == bid.buyer
        || p.max_lots_per_month <= 0
        || p.max_lots_per_month > MAX_LOTS_PER_MONTH
        || p.reserve_months > MAX_RESERVE_MONTHS
        || p.purchase_budget < 0
        || state.credit.stock_spent < 0
        || state.credit.stock_spent > p.purchase_budget
        || !world.agents.iter().any(|a| a.id == bid.buyer)
        || bid.goods.quantity <= 0
        || bid.payment.quantity <= 0
        || c.offers
            .iter()
            .any(|o| o.loan.denomination != bid.payment.resource)
        || !world
            .resources
            .iter()
            .any(|r| r.id == bid.goods.resource && r.kind == ResourceKind::Stock)
        || !world.participants.iter().any(|a| a.agent == p.seller)
        || c.resale_buyer.is_some()
    {
        return Err("invalid scoped posted-sale policy".into());
    }
    reserve(world, p, bid.goods.resource)?;
    Ok(())
}

/// Direct one-stock consumption recipes only. Keep the cheapest known recipe's
/// stock requirement for each need; additionally keep inputs for one productive
/// process cycle. This pilot does not solve arbitrary substitution networks.
fn reserve(world: &World, policy: &Policy, good: ResourceId) -> Result<i32, String> {
    let participant = world
        .participants
        .iter()
        .find(|p| p.agent == policy.seller)
        .ok_or("missing seller")?;
    let mut monthly = 0_i64;
    for need in &participant.needs {
        let units = world
            .definitions
            .iter()
            .filter(|d| {
                d.enabled
                    && d.execution == Execution::Consumption
                    && d.stages.len() == 1
                    && d.stages[0].entry_inputs.len() == 1
                    && d.stages[0].entry_inputs[0].resource == good
            })
            .filter_map(|d| {
                d.outputs
                    .iter()
                    .find(|o| o.resource == need.resource && o.quantity > 0)
                    .map(|o| {
                        ((i64::from(need.quantity) + i64::from(o.quantity) - 1)
                            / i64::from(o.quantity))
                            * i64::from(d.stages[0].entry_inputs[0].quantity)
                    })
            })
            .min()
            .unwrap_or(0);
        monthly = monthly.checked_add(units).ok_or("need reserve overflow")?;
    }
    let inputs = world
        .definitions
        .iter()
        .filter(|d| d.enabled && d.execution == Execution::Productive)
        .map(|d| {
            d.stages
                .iter()
                .flat_map(|s| &s.entry_inputs)
                .filter(|a| a.resource == good)
                .map(|a| i64::from(a.quantity))
                .sum::<i64>()
        })
        .max()
        .unwrap_or(0);
    let reserve = monthly
        .checked_mul(i64::from(policy.reserve_months))
        .and_then(|n| n.checked_add(inputs))
        .ok_or("stock reserve overflow")?;
    i32::try_from(reserve).map_err(|_| "stock reserve overflow".into())
}

pub(crate) fn settle(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    budgets: &mut BTreeMap<Account, i32>,
) -> Result<(), String> {
    let Some(c) = &world.credit else {
        return Ok(());
    };
    let Some(p) = &c.stock_sales else {
        return Ok(());
    };
    let bid = world
        .bids
        .iter()
        .find(|b| b.id == p.bid)
        .ok_or("missing stock bid")?;
    let reserve = reserve(world, p, bid.goods.resource)?;
    let opening_stock = *budgets.get(&(p.seller, bid.goods.resource)).unwrap_or(&0);
    let desired = opening_stock.saturating_sub(reserve).max(0) / bid.goods.quantity;
    let cash = *budgets
        .get(&(bid.buyer, bid.payment.resource))
        .unwrap_or(&0);
    let remaining_budget = p.purchase_budget - out.after.stock_spent;
    let funding = cash.min(remaining_budget).max(0) / bid.payment.quantity;
    let mut used = storage::usage(world, &state.balances);
    for t in &out.transactions {
        storage::apply(world, &mut used, &t.effects);
    }
    let room = storage::room(world, &used, bid.buyer, bid.goods.resource) / bid.goods.quantity;
    let active =
        !state.terminal.contains_key(&p.seller) && !state.terminal.contains_key(&bid.buyer);
    let limit = if active {
        desired.min(p.max_lots_per_month).min(funding).min(room)
    } else {
        0
    };
    let mut sold = 0_i32;
    for _ in 0..limit {
        let transaction = currency::transaction(
            world,
            state,
            currency::StockTrade {
                bid: bid.id,
                seller: p.seller,
            },
        )?;
        if !storage::fits(world, &used, &transaction.effects) {
            break;
        }
        for e in &transaction.effects {
            // Incoming sale receipts cannot fund another outgoing action at this boundary.
            if e.delta < 0 {
                let budget = budgets.entry(e.account).or_default();
                *budget = budget.checked_add(e.delta).ok_or("sale budget overflow")?;
                if *budget < 0 {
                    return Err("posted sale overspends opening budget".into());
                }
            }
        }
        storage::apply(world, &mut used, &transaction.effects);
        out.transactions.push(transaction);
        sold += 1;
    }
    let coins = sold
        .checked_mul(bid.payment.quantity)
        .ok_or("sale value overflow")?;
    out.after.stock_spent = out
        .after
        .stock_spent
        .checked_add(coins)
        .ok_or("purchase budget overflow")?;
    out.stock_sale = Some(Receipt {
        bid: bid.id,
        seller: p.seller,
        reserve,
        opening_stock,
        desired_lots: desired,
        monthly_limit: p.max_lots_per_month,
        funding_limit: funding,
        storage_limit: room,
        sold_lots: sold,
        goods: sold
            .checked_mul(bid.goods.quantity)
            .ok_or("sale quantity overflow")?,
        coins,
    });
    Ok(())
}

pub fn scenario(case: &str) -> Result<(World, State), String> {
    use crate::scenario::{GRAIN, GROW, PERSON, SEED, STATE_AGENT, TOKEN};
    let (mut w, s) = crate::borrowing::scenario("affordable")?;
    for right in &mut w.rights {
        right.through = CULTIVATION_RIGHT_MONTHS;
    }
    let c = w.credit.as_mut().unwrap();
    c.offers[0].loan.term_months = LOAN_MONTHS;
    c.purchase_policy = crate::borrowing::Policy::Compare(crate::borrowing::Config {
        horizon_months: FORECAST_MONTHS,
    });
    c.endowments
        .iter_mut()
        .find(|e| e.agent == PERSON)
        .unwrap()
        .amount
        .quantity = BRIDGE_COINS;
    c.stock_sales = Some(Policy {
        bid: SALE_BID,
        seller: PERSON,
        reserve_months: RESERVE_MONTHS,
        max_lots_per_month: MONTHLY_LOTS,
        purchase_budget: match case {
            "funded" | "food-tight" => PURCHASE_BUDGET,
            "limited" => LIMITED_BUDGET,
            _ => return Err("unknown stock sale case".into()),
        },
    });
    w.definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .outputs
        .iter_mut()
        .find(|a| a.resource == GRAIN)
        .unwrap()
        .quantity = CROP_GRAIN;
    w.bids.push(currency::Bid {
        id: SALE_BID,
        buyer: STATE_AGENT,
        goods: Amount::new(GRAIN, 1),
        payment: Amount::new(
            TOKEN,
            if case == "food-tight" {
                FOOD_TIGHT_PRICE
            } else {
                NORMAL_PRICE
            },
        ),
    });
    w.storage.weights.insert(GRAIN, 1);
    w.storage.weights.insert(SEED, 1);
    w.storage.capacities.insert(PERSON, STORE_CAPACITY);
    w.storage.capacities.insert(STATE_AGENT, STORE_CAPACITY);
    Ok((w, s))
}
