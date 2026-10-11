//! Missing-town-inventory comparison; no added cash, ore, work, or accepted terms.
use super::{TOWN, WORKSHOP, continuation as c, publication, sales::Startup};
use crate::{
    agency::{forwards::Options, organization, publication::Policy},
    minting::{METAL, ORE},
    model::*,
};

const STARTUP_PREPAYMENT: i32 = 2;
const DELIVERY_MONTHS: u32 = 3;

/// Matched ablations of the retained workshop stocks; never adds inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stocks {
    Both,
    OreOnly,
    MetalOnly,
    Neither,
}
impl Stocks {
    pub const ALL: [Self; 4] = [Self::Both, Self::OreOnly, Self::MetalOnly, Self::Neither];
}

pub fn with_stocks(case: c::Case, enabled: bool, stocks: Stocks) -> Result<(World, State), String> {
    let (w, mut s) = with_authority(case, enabled)?;
    if matches!(stocks, Stocks::MetalOnly | Stocks::Neither) {
        s.balances.insert((WORKSHOP, ORE), 0);
    }
    if matches!(stocks, Stocks::OreOnly | Stocks::Neither) {
        s.balances.insert((WORKSHOP, METAL), 0);
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}

pub fn scenario(case: c::Case) -> Result<(World, State), String> {
    with_authority(case, true)
}

pub fn with_authority(case: c::Case, enabled: bool) -> Result<(World, State), String> {
    let (mut w, s) = publication::with_startup(
        case,
        Startup::MetalInventory,
        Policy::StockedAsksOnEqualProjection,
    )?;
    if enabled {
        for agent in [WORKSHOP, TOWN] {
            let controller = w.agency.remove(&agent).unwrap();
            let mut agreement = controller.operating_agreement().unwrap().clone();
            if agreement.stock_markets.contains_key(&ORE) {
                agreement.forwards.insert(
                    ORE,
                    Options {
                        prepayment: STARTUP_PREPAYMENT,
                        delivery_months: DELIVERY_MONTHS,
                    },
                );
            }
            let signatures = agreement.members.clone();
            organization::install_discovering_initial(
                &mut w,
                &s,
                agreement,
                &signatures,
                controller.config,
            )?;
        }
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}
