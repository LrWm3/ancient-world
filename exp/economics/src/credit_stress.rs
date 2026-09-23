//! Production-funded loan controls with unanticipated missed-harvest work.
use crate::{borrowing, model::*, scenario::*, stock_sale};

pub const OBSERVATION_MONTHS: u32 = 24;
const GRACE_MONTHS: u32 = 8;
const INITIAL_SEED: i32 = 3;
const LENDER_LABOR: i32 = 2;
const FIRST_FAILED_HARVEST: u32 = 6;
const SECOND_FAILED_HARVEST: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Normal,
    Temporary,
    Persistent,
}

pub fn scenario(case: Case) -> Result<(World, State), String> {
    let (mut world, mut state) = stock_sale::scenario("funded")?;
    let credit = world.credit.as_mut().unwrap();
    // Isolate post-acceptance servicing; future capacity shocks are not credit advice.
    credit.purchase_policy = borrowing::Policy::Scripted;
    credit.offers[0].loan.grace_months = GRACE_MONTHS;
    state.balances.insert((PERSON, SEED), INITIAL_SEED);
    world.participants.push(Participant {
        agent: STATE_AGENT,
        capacity: Amount::new(LABOR, LENDER_LABOR),
        needs: vec![],
    });
    if case != Case::Normal {
        world
            .capacity_overrides
            .insert((FIRST_FAILED_HARVEST, PERSON), 0);
    }
    if case == Case::Persistent {
        world
            .capacity_overrides
            .insert((SECOND_FAILED_HARVEST, PERSON), 0);
    }
    Ok((world, state))
}
