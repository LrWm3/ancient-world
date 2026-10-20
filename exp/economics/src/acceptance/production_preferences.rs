//! Opt-in founding preferences for completed work, without input stock targets.
use super::{
    WORKSHOP, buffers,
    continuation::Case,
    financing::{self, Preferences, Stocks},
};
use crate::{
    agency::objectives::{Metric, Objective, Scope},
    minting::REFINE,
    model::*,
};

const MONTHLY_REFINING_TARGET: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    Off,
    ProductionFirst,
    CashFirst,
}

pub fn scenario(
    case: Case,
    stocks: Stocks,
    preferences: Preferences,
    priority: Priority,
) -> Result<(World, State), String> {
    let (mut w, s) = financing::with_preferences(case, true, stocks, preferences)?;
    let policy = match priority {
        Priority::Off => buffers::Policy::Off,
        Priority::ProductionFirst => buffers::Policy::InputsFirst,
        Priority::CashFirst => buffers::Policy::CashFirst,
    };
    buffers::apply(
        &mut w.agency.get_mut(&WORKSHOP).unwrap().config,
        &[Objective {
            scope: Scope::Organization,
            metric: Metric::ProcessShortfall {
                definition: REFINE,
                target: MONTHLY_REFINING_TARGET,
            },
        }],
        policy,
    );
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}
