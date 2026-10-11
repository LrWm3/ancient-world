//! Cash-first matched comparison, changing only the workshop's peer forecast.
use super::{WORKSHOP, continuation as c, governed as g};
use crate::{agency::expectations::Policy, model::*};

const PEER_MEMORY_MONTHS: u32 = 6;
pub const POLICY: Policy = Policy::RecentSuccessfulStarts {
    memory_months: PEER_MEMORY_MONTHS,
};

pub fn scenario(case: c::Case) -> Result<(World, State), String> {
    scenario_with(case, POLICY)
}
pub fn scenario_with(case: c::Case, policy: Policy) -> Result<(World, State), String> {
    let (mut w, s) = g::scenario(case)?;
    let config = &mut w.agency.get_mut(&WORKSHOP).unwrap().config;
    config.objectives.rotate_right(1);
    config.peer_expectations = policy;
    Ok((w, s))
}
