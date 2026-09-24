//! One posted collateral sale and one prospective buyer; no auction or ZIP here.
use crate::{
    compute::Backend,
    credit::{self, CollateralSettlement, Event},
    model::*,
    simulation::Simulation,
    work_choice,
};
use std::collections::BTreeMap;

pub const BUYER: AgentId = 99;
const LAND_VALUE_TICKS: i32 = 9_000;
const MINIMUM_PRICE_TICKS: i32 = 7_500;
const FUNDED_COINS: i32 = 10_000;
const DEFICIENCY_COINS: i32 = 8_000;
const INSUFFICIENT_COINS: i32 = 7_000;
const STOCK_VALUE_TICKS: i32 = 100;
const BUYER_LABOR: i32 = 2;
const BID_HORIZON_MONTHS: u32 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingSale {
    pub listed: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buyer {
    pub from_month: u32,
    pub land_value: i32,
    /// Valuations are in loan-denomination ticks for this buyer's bids.
    pub preferences: work_choice::Config,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bid {
    pub buyer: AgentId,
    pub land_value: i32,
    pub with_asset: i64,
    pub without_asset: i64,
    pub crop_value: i32,
    pub willingness: i32,
    pub available_coins: i32,
    pub amount: i32,
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let Some(c) = &world.credit else {
        return Ok(());
    };
    if state.credit.pending_sales.len() > 1 {
        return Err("resale pilot supports one pending listing".into());
    }
    for (&id, listing) in &state.credit.pending_sales {
        let loan = state.credit.loans.get(&id).ok_or("sale without a loan")?;
        if loan.status != credit::Status::PendingSale
            || listing.listed == 0
            || listing.listed > state.month
            || listing.listed != loan.last_accrued
            || !matches!(
                loan.collateral
                    .as_ref()
                    .ok_or("sale requires collateral")?
                    .settlement,
                CollateralSettlement::ResaleProceeds { .. }
            )
            || credit::owner(
                world,
                state,
                loan.collateral
                    .as_ref()
                    .ok_or("sale requires collateral")?
                    .asset,
            ) != Some(loan.creditor)
        {
            return Err("invalid pending collateral sale".into());
        }
    }
    if state.credit.loans.values().any(|l| {
        (l.status == credit::Status::PendingSale) != state.credit.pending_sales.contains_key(&l.id)
    }) {
        return Err("pending loan and sale disagree".into());
    }
    if let Some(b) = &c.resale_buyer {
        let who = b.preferences.agent;
        if b.from_month == 0
            || b.land_value < 0
            || who == c.application.buyer
            || c.offers.iter().any(|o| {
                who == o.loan.creditor
                    || who == o.sale.seller
                    || b.preferences.values.contains_key(&o.loan.denomination)
            })
        {
            return Err("invalid resale buyer or valuation denomination".into());
        }
        let mut valuation_world = world.clone();
        valuation_world.work_choice = Some(b.preferences.clone());
        work_choice::validate(&valuation_world, state)?;
    }
    Ok(())
}

/// Value the current attachment relative to the buyer's best alternative work.
/// Land's longer-term value is supplied separately. These read-only forecasts
/// do not pay for the asset, promise future sales, or create real reservations.
pub fn bid(
    world: &World,
    state: &State,
    loan: &credit::Loan,
    buyer: &Buyer,
    available: i32,
) -> Result<Bid, String> {
    let context = crate::forecast::ForecastContext::new(world, state);
    let mut scores = Vec::new();
    for acquire in [false, true] {
        let (mut w, mut s) = context.clone().into_parts();
        w.work_choice = Some(buyer.preferences.clone());
        s.phase = Phase::Productive; // purchase is before this month's work
        if acquire {
            s.credit.pending_sales.remove(&loan.id);
            s.credit
                .loans
                .get_mut(&loan.id)
                .ok_or("missing valuation loan")?
                .status = credit::Status::Enforced;
            s.credit.owners.insert(
                loan.collateral
                    .as_ref()
                    .ok_or("sale requires collateral")?
                    .asset,
                buyer.preferences.agent,
            );
            for p in s.processes.values_mut().filter(|p| {
                p.status == Status::Active
                    && p.asset == loan.collateral.as_ref().map(|c| c.asset)
                    && p.right.is_some_and(|r| credit::follows_owner(&w, r))
            }) {
                p.operator = buyer.preferences.agent;
                p.beneficiary = buyer.preferences.agent;
                p.goal = None;
            }
        }
        let sim = Simulation::new(w, s, Backend::Reference)?;
        let decision = work_choice::evaluate(&sim)?.work_choice.unwrap();
        scores.push(decision.forecasts[decision.selected].net_value);
    }
    let crop_value = i32::try_from(
        scores[1]
            .checked_sub(scores[0])
            .ok_or("crop value overflow")?
            .max(0),
    )
    .map_err(|_| "crop value overflow")?;
    let willingness = buyer
        .land_value
        .checked_add(crop_value)
        .ok_or("bid overflow")?;
    Ok(Bid {
        buyer: buyer.preferences.agent,
        land_value: buyer.land_value,
        with_asset: scores[1],
        without_asset: scores[0],
        crop_value,
        willingness,
        available_coins: available,
        amount: willingness.min(available),
    })
}

pub(crate) fn settle(
    world: &World,
    state: &State,
    out: &mut credit::Boundary,
    budgets: &mut BTreeMap<Account, i32>,
) -> Result<(), String> {
    // Only listings committed in an earlier month may trade. A rejected bid
    // leaves title, debt, crop and cash untouched; another month may try again.
    for (&id, listing) in &state.credit.pending_sales {
        if listing.listed >= state.month {
            continue;
        }
        let mut loan = out.after.loans[&id].clone();
        let CollateralSettlement::ResaleProceeds { minimum_price } = loan
            .collateral
            .as_ref()
            .ok_or("sale requires collateral")?
            .settlement
        else {
            return Err("wrong sale settlement rule".into());
        };
        let Some(buyer) = world
            .credit
            .as_ref()
            .unwrap()
            .resale_buyer
            .as_ref()
            .filter(|b| {
                b.from_month <= state.month
                    && !state.terminal.contains_key(&b.preferences.agent)
                    && !state
                        .processes
                        .values()
                        .any(|p| p.operator == b.preferences.agent && p.status == Status::Active)
            })
        else {
            out.events.push(Event::ResaleNoBuyer { loan: id });
            continue;
        };
        let cash = budgets
            .get(&(buyer.preferences.agent, loan.denomination))
            .copied()
            .unwrap_or(0);
        let quote = bid(world, state, &loan, buyer, cash)?;
        let price = quote.amount;
        out.events.push(Event::ResaleBid { loan: id, quote });
        if price < minimum_price {
            out.events.push(Event::ResaleRejected {
                loan: id,
                bid: price,
                minimum_price,
            });
            continue;
        }
        let debt_credit = price.min(loan.debt()?);
        let surplus = price - debt_credit;
        // Split the buyer's payment directly, so surplus never depends on the
        // lender spending funds it has only just received in the same boundary.
        credit::transfer(
            out,
            budgets,
            buyer.preferences.agent,
            loan.creditor,
            loan.denomination,
            debt_credit,
        )?;
        credit::transfer(
            out,
            budgets,
            buyer.preferences.agent,
            loan.debtor,
            loan.denomination,
            surplus,
        )?;
        credit::transfer_attachments(
            world,
            state,
            out,
            loan.collateral
                .as_ref()
                .ok_or("sale requires collateral")?
                .asset,
            buyer.preferences.agent,
        );
        out.after.owners.insert(
            loan.collateral
                .as_ref()
                .ok_or("sale requires collateral")?
                .asset,
            buyer.preferences.agent,
        );
        out.after.values.insert(
            loan.collateral
                .as_ref()
                .ok_or("sale requires collateral")?
                .asset,
            price,
        );
        out.after.pending_sales.remove(&id);
        loan.status = credit::Status::Enforced;
        loan.apply_payment(debt_credit);
        out.events.push(Event::Resold {
            loan: id,
            buyer: buyer.preferences.agent,
            price,
            debt_credit,
            surplus,
            remaining_debt: loan.debt()?,
        });
        out.after.loans.insert(id, loan);
    }
    Ok(())
}

pub fn scenario(case: &str) -> Result<(World, State), String> {
    use crate::scenario::{GRAIN, LABOR, RAW_WOOD, SEED, TOKEN};
    let coins = match case {
        "funded" | "no-buyer" => FUNDED_COINS,
        "deficiency" => DEFICIENCY_COINS,
        "insufficient" => INSUFFICIENT_COINS,
        _ => return Err("unknown resale scenario".into()),
    };
    let (mut w, s) = work_choice::scenario("mature")?;
    w.agents.push(Agent {
        id: BUYER,
        name: "Prospective buyer".into(),
    });
    w.participants.push(Participant {
        agent: BUYER,
        capacity: Amount::new(LABOR, BUYER_LABOR),
        needs: vec![],
    });
    let c = w.credit.as_mut().unwrap();
    c.offers[0].collateral.settlement = CollateralSettlement::ResaleProceeds {
        minimum_price: MINIMUM_PRICE_TICKS,
    };
    c.endowments.push(credit::Endowment {
        agent: BUYER,
        amount: Amount::new(TOKEN, coins),
    });
    if case != "no-buyer" {
        c.resale_buyer = Some(Buyer {
            from_month: 1,
            land_value: LAND_VALUE_TICKS,
            preferences: work_choice::Config {
                agent: BUYER,
                horizon: BID_HORIZON_MONTHS,
                values: BTreeMap::from([
                    (GRAIN, STOCK_VALUE_TICKS),
                    (SEED, STOCK_VALUE_TICKS),
                    (RAW_WOOD, STOCK_VALUE_TICKS),
                ]),
            },
        });
    }
    Ok((w, s))
}
