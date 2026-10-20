//! Matched founding-policy comparison; keeps resources, law and finance unchanged.
use super::{
    WORKSHOP,
    continuation::Case,
    financing::{Liquidity, Preferences, Stocks},
    production_preferences::{self, Priority},
};
use crate::{agency::publication::Policy, model::*};

pub fn scenario(case: Case, stocks: Stocks, enabled: bool) -> Result<(World, State), String> {
    let (mut w, s) = production_preferences::scenario(
        case,
        stocks,
        Preferences {
            workshop: Liquidity::MonthlyExposure,
            town: Liquidity::TerminalReserve,
        },
        Priority::ProductionFirst,
    )?;
    if enabled {
        w.agency.get_mut(&WORKSHOP).unwrap().config.publication =
            Policy::ProductiveAsksOnEqualProjection;
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}
