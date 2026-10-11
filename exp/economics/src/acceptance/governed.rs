//! Recurring acceptance with discovered work and stock-quote programs. Consent,
//! constitutional options, employment and reserve objectives remain supplied.
use super::{continuation as c, *};
use crate::agency::{
    self,
    markets::Options,
    objectives::{Metric, Objective, Scope},
    organization,
};

const OPERATING_CASH_TARGET: i32 = 1000;
const MAX_ORE_LOTS: i32 = 2;
const ORE_LOT_PRICE: i32 = 2;
const ORE_PURCHASE_TARGET: i32 = 4;

pub fn scenario(case: c::Case) -> Result<(World, State), String> {
    let (mut w, s) = c::spot_scenario(case)?;
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
        .retain(|q| ![WORKSHOP, TOWN].contains(&q.agent));
    for (agent, side, holding) in [
        (WORKSHOP, Side::Buy, ORE_PURCHASE_TARGET),
        (TOWN, Side::Sell, 0),
    ] {
        let previous = w.agency.remove(&agent).unwrap();
        let mut agreement = previous.operating_agreement().unwrap().clone();
        if agent != WORKSHOP || case != c::Case::NoReplacementOrders {
            agreement.stock_markets.insert(
                ORE,
                Options {
                    side,
                    limits: [ORE_LOT_PRICE].into(),
                    holdings: [holding].into(),
                    max_lots: MAX_ORE_LOTS,
                },
            );
        }
        let mut config = previous.config;
        config.programs.clear();
        let cash = Objective {
            scope: Scope::Organization,
            metric: Metric::Reserve {
                resource: COIN,
                target: OPERATING_CASH_TARGET,
            },
        };
        if side == Side::Buy {
            // Preserve productive inventory before optimizing liquid cash. The
            // forecast does not invent the state's future discretionary buys.
            config.objectives.push(Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: ORE,
                    target: ORE_PURCHASE_TARGET,
                },
            });
            config.objectives.push(cash);
        } else {
            // A production buffer is not a sale floor: value realized receipts.
            config.objectives.insert(0, cash);
        }
        organization::install_discovering_initial(
            &mut w,
            &s,
            agreement.clone(),
            &agreement.members,
            config,
        )?;
    }
    Ok((w, s))
}

pub fn check(run: &Run) -> Result<c::Readiness, String> {
    let mut result = c::check(run)?;
    for agent in [WORKSHOP, TOWN] {
        let Some(controller) = run.sim.world.agency.get(&agent) else {
            result
                .failures
                .push(format!("organization {agent} has no operating controller"));
            continue;
        };
        if !controller.history.iter().any(|d| d.accepted.as_ref().is_some_and(|p| p.commands.iter().any(|c| matches!(c, agency::Command::WorkTargets(orders) if orders.iter().any(|o| o.agent == agent))))) {
            result.failures.push(format!("organization {agent} did not choose production targets"));
        }
        if !controller.discover_programs
            || !controller.config.programs.is_empty()
            || !controller.history.iter().any(|d| {
                d.accepted.as_ref().is_some_and(|p| {
                    p.commands
                        .iter()
                        .any(|c| matches!(c, agency::Command::PeerStockQuotes(q) if !q.is_empty()))
                })
            })
        {
            result.failures.push(format!(
                "organization {agent} did not discover and choose stock trading"
            ));
        }
    }
    Ok(result)
}
