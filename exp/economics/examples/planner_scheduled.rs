//! Dated reciprocal schedules versus independently selected spot orders.
#[path = "../tests/support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    allocation::Policy as Allocation,
    composition::{
        Budget, Strategy,
        continuation::scheduled,
        continuation::{Policy, persons::Persons},
    },
    compute::Backend,
    model::*,
    scenario::*,
};
const RUN_MONTHS: u32 = 24;
const FORECAST_MONTHS: u32 = 6;
const SEARCH_EXPANSIONS: usize = 256;
const SEARCH_FORECASTS: usize = 32;
const SEED: u64 = 7;
fn main() -> Result<(), String> {
    println!(
        "strategy,scheduled,trading,opening_coins,person,food,warmth,terminal_months,crops,wood,coins,grain_volume,wood_volume,accepted,completed_schedules,failed_schedules,outstanding_deliveries,search_months"
    );
    for strategy in [Strategy::Beam, Strategy::BestFirst] {
        for scheduled in [false, true] {
            for coins in [1, 2, 6] {
                for trading in [true, false] {
                    let mut sim = fixtures::trading_persons(Backend::Reference, trading)?;
                    for person in [PERSON, 89] {
                        sim.state.balances.insert((person, TOKEN), coins);
                    }
                    let p = Persons::new(
                        sim.world.participants.iter().map(|p| p.agent),
                        strategy,
                        Budget {
                            expansions: SEARCH_EXPANSIONS,
                            forecasts: SEARCH_FORECASTS,
                            months: FORECAST_MONTHS,
                        },
                        Policy::Monthly,
                        Allocation::StablePriority,
                        SEED,
                    )?;
                    let mut driver = scheduled::Controller::new(p);
                    while sim.state.month <= RUN_MONTHS {
                        if scheduled {
                            driver.step(&mut sim)?;
                        } else {
                            driver.persons.step(&mut sim)?;
                        }
                    }
                    let p = &driver.persons;
                    let volume = |id| {
                        sim.state
                            .town_market
                            .history
                            .iter()
                            .map(|r| r.markets[&id].volume)
                            .sum::<i32>()
                    };
                    for person in [PERSON, 89] {
                        let reports: Vec<_> =
                            sim.reports.iter().filter(|r| r.agent == person).collect();
                        let deficit =
                            |resource| reports.iter().map(|r| r.deficit(resource)).sum::<i32>();
                        let completions = |id| {
                            sim.state
                                .processes
                                .values()
                                .filter(|x| {
                                    x.operator == person
                                        && x.definition == id
                                        && x.status == Status::Completed
                                })
                                .count()
                        };
                        let accepted = driver.history.iter().filter(|r| r.accepted).count();
                        let completed = sim
                            .state
                            .town_market
                            .history
                            .iter()
                            .filter_map(|r| r.cooperation.as_ref())
                            .filter(|b| b.event == "Completed")
                            .count();
                        let failed = sim
                            .state
                            .town_market
                            .history
                            .iter()
                            .filter_map(|r| r.cooperation.as_ref())
                            .filter(|b| b.failure.is_some())
                            .count();
                        let outstanding =
                            economics_compute_smoke::cooperation::active_independent(&sim.state)
                                .map_or(0, |c| {
                                    c.deliveries
                                        .iter()
                                        .filter(|d| d.month >= sim.state.month)
                                        .count()
                                });
                        let work: u64 = if scheduled {
                            p.controllers
                                .values()
                                .flat_map(|c| &c.history)
                                .filter_map(|r| r.search.as_ref())
                                .map(|m| m.forecast_months)
                                .sum::<u64>()
                                + driver
                                    .history
                                    .iter()
                                    .map(|r| {
                                        r.assessments
                                            .iter()
                                            .map(|a| {
                                                a.outside_search.forecast_months
                                                    + a.offered_search.forecast_months
                                            })
                                            .sum::<u64>()
                                    })
                                    .sum::<u64>()
                        } else {
                            p.controllers
                                .values()
                                .flat_map(|c| &c.history)
                                .filter_map(|r| r.search.as_ref())
                                .map(|m| m.forecast_months)
                                .sum::<u64>()
                        };
                        println!(
                            "{strategy:?},{scheduled},{trading},{coins},{person},{},{},{},{},{},{},{},{},{accepted},{completed},{failed},{outstanding},{work}",
                            deficit(NUTRITION),
                            deficit(WARMTH),
                            reports.iter().filter(|r| r.terminal.is_some()).count(),
                            completions(GROW),
                            completions(PREPARE_FUEL),
                            sim.state.balance(person, TOKEN),
                            volume(1),
                            volume(economics_compute_smoke::production_market::WOOD_MARKET)
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
