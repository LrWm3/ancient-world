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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Liquidity {
    TerminalReserve,
    MonthlyExposure,
}

/// Initial constitutional preference variant, before any mandate is sealed.
/// Targets, ordering, beneficiaries, money and institutional rights are unchanged.
pub fn with_liquidity(
    case: c::Case,
    enabled: bool,
    stocks: Stocks,
    liquidity: Liquidity,
) -> Result<(World, State), String> {
    with_preferences(
        case,
        enabled,
        stocks,
        Preferences {
            workshop: liquidity,
            town: liquidity,
        },
    )
}

/// Independent founding preferences: these are not runtime policy amendments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub workshop: Liquidity,
    pub town: Liquidity,
}

pub fn with_preferences(
    case: c::Case,
    enabled: bool,
    stocks: Stocks,
    preferences: Preferences,
) -> Result<(World, State), String> {
    let (mut w, s) = with_stocks(case, enabled, stocks)?;
    for (agent, liquidity) in [(WORKSHOP, preferences.workshop), (TOWN, preferences.town)] {
        apply_liquidity(&mut w.agency.get_mut(&agent).unwrap().config, liquidity);
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}

fn apply_liquidity(config: &mut crate::agency::Config, liquidity: Liquidity) {
    use crate::agency::objectives::Metric;
    if liquidity != Liquidity::MonthlyExposure {
        return;
    }
    for o in config
        .objectives
        .iter_mut()
        .chain(config.preferences.values_mut().flatten())
    {
        if let Metric::Reserve { resource, target } = o.metric
            && resource == crate::minting::COIN
        {
            o.metric = Metric::ReserveExposure { resource, target };
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agency::objectives::{Metric, Objective, Scope};
    #[test]
    fn individual_governor_preferences_convert_only_coin_reserves() {
        let goals = vec![
            Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: crate::minting::COIN,
                    target: 3,
                },
            },
            Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: ORE,
                    target: 7,
                },
            },
        ];
        let mut config = crate::agency::scenario::config(goals.clone(), Default::default());
        config.preferences.insert(99, goals);
        let original = config.clone();
        apply_liquidity(&mut config, Liquidity::TerminalReserve);
        assert_eq!(config, original);
        apply_liquidity(&mut config, Liquidity::MonthlyExposure);
        let mut expected = original;
        expected.objectives[0].metric = Metric::ReserveExposure {
            resource: crate::minting::COIN,
            target: 3,
        };
        expected.preferences.get_mut(&99).unwrap()[0].metric =
            expected.objectives[0].metric.clone();
        assert_eq!(config, expected);
        apply_liquidity(&mut config, Liquidity::MonthlyExposure);
        assert_eq!(config, expected);
    }
}
