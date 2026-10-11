//! Cash-only organizations discover work from signed sale opportunities.
use super::{continuation as c, expectations, *};
use crate::agency::{
    markets::Options,
    objectives::{Metric, Scope},
    organization,
};

const STARTUP_REVIEW_MONTHS: u32 = 1;
const STARTUP_METAL_UNITS: i32 = 4;
const STARTUP_ORE_UNITS: i32 = 4;
const METAL_SALE_LOTS: i32 = 1;
const METAL_SALE_FLOOR: i32 = 0;

pub fn scenario(case: c::Case) -> Result<(World, State), String> {
    let (mut w, s) = expectations::scenario(case)?;
    // Convert the supplied initial seller quote into an explicitly signed option;
    // thereafter catalogs never depend on mutable live quotes.
    let price = w
        .minting
        .as_ref()
        .unwrap()
        .order_policy
        .as_ref()
        .unwrap()
        .input_limits[&METAL];
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes
        .retain(|q| q.agent != WORKSHOP || q.market != METAL);
    for agent in [WORKSHOP, TOWN] {
        let previous = w.agency.remove(&agent).unwrap();
        let mut a = previous.operating_agreement().unwrap().clone();
        if agent == WORKSHOP {
            a.stock_markets.insert(
                METAL,
                Options {
                    side: Side::Sell,
                    limits: [price].into(),
                    holdings: [METAL_SALE_FLOOR].into(),
                    max_lots: METAL_SALE_LOTS,
                },
            );
        }
        let mut config = previous.config;
        config.objectives.retain(|o| {
            o.scope == Scope::Organization
                && matches!(o.metric, Metric::Reserve { resource: COIN, .. })
        });
        organization::install_discovering_initial(&mut w, &s, a.clone(), &a.members, config)?;
    }
    Ok((w, s))
}

/// Declared working-inventory controls, not endogenous initial market formation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Startup {
    Cold,
    PostedMetal,
    MetalInventory,
    SupplyChainInventory,
}

pub fn with_startup(case: c::Case, startup: Startup) -> Result<(World, State), String> {
    let (mut w, mut s) = scenario(case)?;
    if startup == Startup::Cold {
        return Ok((w, s));
    }
    for agent in [WORKSHOP, TOWN] {
        w.agency.get_mut(&agent).unwrap().config.review_every = STARTUP_REVIEW_MONTHS;
    }
    let price = w
        .minting
        .as_ref()
        .unwrap()
        .order_policy
        .as_ref()
        .unwrap()
        .input_limits[&METAL];
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes
        .push(crate::minting::orders::Quote {
            agent: WORKSHOP,
            market: METAL,
            side: Side::Sell,
            limit: price,
            holding: METAL_SALE_FLOOR,
            max_lots: Some(METAL_SALE_LOTS),
        });
    if matches!(
        startup,
        Startup::MetalInventory | Startup::SupplyChainInventory
    ) {
        s.balances.insert((WORKSHOP, METAL), STARTUP_METAL_UNITS);
    }
    if startup == Startup::SupplyChainInventory {
        s.balances.insert((TOWN, ORE), STARTUP_ORE_UNITS);
        let a = w.agency[&TOWN].operating_agreement().unwrap();
        let o = &a.stock_markets[&ORE];
        let quote = crate::minting::orders::Quote {
            agent: TOWN,
            market: ORE,
            side: o.side,
            limit: *o.limits.first().unwrap(),
            holding: *o.holdings.first().unwrap(),
            max_lots: Some(o.max_lots),
        };
        w.minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap()
            .peer_stocks
            .as_mut()
            .unwrap()
            .quotes
            .push(quote);
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}
