//! Cash-only organizations discover work from signed sale opportunities.
use super::{continuation as c, expectations, *};
use crate::agency::{
    markets::Options,
    objectives::{Metric, Scope},
    organization,
};

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
