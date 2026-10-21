//! Same finite economy, but catalog labor terms require fresh mutual consent.
use super::{
    SMITH, TOWN, TOWN_EMPLOYMENT, WORKSHOP, WORKSHOP_EMPLOYMENT, continuation::Case,
    financing::Stocks, working_capital,
};
use crate::{
    employment::recruitment::{Mandate, Worker},
    minting::{COIN, ISSUER, WHEAT},
    model::*,
};
const HORIZON: u32 = 4;
const STARTUP_FOOD_ALLOCATION: i32 = 3;
const PAYROLL_BUDGET: i32 = 1;
const WORKER_CASH_TARGET: i32 = 1000;
const WORKER_FOOD_TARGET: i32 = 3;
pub fn scenario() -> Result<(World, State), String> {
    with_startup_food(true)
}

/// Explicit initial distribution: conserve total grain, bridge the loss of idle wages.
pub fn with_startup_food(buffered: bool) -> Result<(World, State), String> {
    let (mut w, mut s) = working_capital::scenario(Case::Baseline, Stocks::Neither, true)?;
    for (actor, offer) in [(WORKSHOP, WORKSHOP_EMPLOYMENT), (TOWN, TOWN_EMPLOYMENT)] {
        w.employment_offers.insert(offer);
        w.agency.get_mut(&actor).unwrap().config.hiring = Some(Mandate {
            offers: [offer].into(),
            payroll: Amount::new(COIN, PAYROLL_BUDGET),
            horizon: HORIZON,
        });
    }
    w.recruitment.workers.insert(
        SMITH,
        Worker {
            horizon: HORIZON,
            reserves: vec![
                Amount::new(WHEAT, WORKER_FOOD_TARGET),
                Amount::new(COIN, WORKER_CASH_TARGET),
            ],
        },
    );
    if buffered {
        let remaining = s
            .balance(ISSUER, WHEAT)
            .checked_sub(STARTUP_FOOD_ALLOCATION)
            .filter(|q| *q >= 0)
            .ok_or("insufficient opening food allocation")?;
        s.balances.insert((ISSUER, WHEAT), remaining);
        s.balances.insert(
            (SMITH, WHEAT),
            s.balance(SMITH, WHEAT) + STARTUP_FOOD_ALLOCATION,
        );
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}
