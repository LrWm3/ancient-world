//! Voluntary initial asks with exactly the sale experiment's finite stocks.
use super::{TOWN, WORKSHOP, continuation as c, sales};
use crate::{
    agency::publication::Policy,
    minting::{METAL, ORE},
    model::*,
};

pub fn scenario(case: c::Case) -> Result<(World, State), String> {
    with_startup(
        case,
        sales::Startup::SupplyChainInventory,
        Policy::StockedAsksOnEqualProjection,
    )
}

/// Startup selects inherited inventory/review settings; even `PostedMetal` has
/// its supplied ask removed. Only agents may publish organization asks here.
pub fn with_startup(
    case: c::Case,
    startup: sales::Startup,
    policy: Policy,
) -> Result<(World, State), String> {
    let (mut w, s) = sales::with_startup(case, startup)?;
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.quotes
        .retain(|q| q.agent != WORKSHOP || q.market != METAL);
    p.peer_stocks
        .as_mut()
        .unwrap()
        .quotes
        .retain(|q| q.agent != TOWN || q.market != ORE);
    for agent in [WORKSHOP, TOWN] {
        w.agency.get_mut(&agent).unwrap().config.publication = policy;
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}
