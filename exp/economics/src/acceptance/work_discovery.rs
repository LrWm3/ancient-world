//! Matched discovery comparison; request policy alone varies at founding.
use super::{TOWN, WORKSHOP, continuation::Case, financing::Stocks, startup_publication};
use crate::{agency::work_requests::Policy, model::*};

pub fn scenario(case: Case, stocks: Stocks, policy: Policy) -> Result<(World, State), String> {
    let (mut w, s) = startup_publication::scenario(case, stocks, true)?;
    for actor in [WORKSHOP, TOWN] {
        w.agency.get_mut(&actor).unwrap().config.work_requests = policy;
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}

/// Separate publication ablation; announcing interest alone remains available.
pub fn with_input_bids(
    case: Case,
    stocks: Stocks,
    enabled: bool,
) -> Result<(World, State), String> {
    let (mut w, s) = scenario(case, stocks, Policy::Upstream)?;
    if enabled {
        w.agency.get_mut(&WORKSHOP).unwrap().config.publication =
            crate::agency::publication::Policy::InputBidsOnEqualProjection;
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}
