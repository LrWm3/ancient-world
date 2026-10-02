//! Matched counterparty expectation comparison; endowments and search stay fixed.
#[path = "../tests/support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    allocation::Policy as Allocation,
    composition::{
        Budget, Strategy,
        continuation::{Policy, persons::Persons},
        expectations::Policy as Counterparties,
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
const SHORT_MEMORY_MONTHS: u32 = 1;
const LONG_MEMORY_MONTHS: u32 = 3;
fn main() -> Result<(), String> {
    println!(
        "strategy,counterparties,trading,opening_coins,person,food,warmth,terminal_months,crops,wood,coins,grain_volume,wood_volume,expected_unfilled,unexpected_filled,search_months"
    );
    for strategy in [Strategy::Beam, Strategy::BestFirst] {
        for counterparties in [
            Counterparties::Ordinary,
            Counterparties::RecentSubmission {
                memory_months: SHORT_MEMORY_MONTHS,
            },
            Counterparties::RecentSubmission {
                memory_months: LONG_MEMORY_MONTHS,
            },
        ] {
            for coins in [1, 2, 6] {
                for trading in [true, false] {
                    let mut sim = fixtures::trading_persons(Backend::Reference, trading)?;
                    for person in [PERSON, 89] {
                        sim.state.balances.insert((person, TOKEN), coins);
                    }
                    let mut p = Persons::new(
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
                    p.counterparty_policy = counterparties;
                    while sim.state.month <= RUN_MONTHS {
                        p.step(&mut sim)?;
                    }
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
                        let unfilled: i32 = p
                            .history
                            .iter()
                            .map(|r| {
                                let x = &r.exchanges[&person];
                                x.expected
                                    .iter()
                                    .map(|(key, q)| {
                                        (q - x.actual.get(key).copied().unwrap_or(0)).max(0)
                                    })
                                    .sum::<i32>()
                            })
                            .sum();
                        let unexpected: i32 = p
                            .history
                            .iter()
                            .map(|r| {
                                let x = &r.exchanges[&person];
                                x.actual
                                    .iter()
                                    .map(|(key, q)| {
                                        (q - x.expected.get(key).copied().unwrap_or(0)).max(0)
                                    })
                                    .sum::<i32>()
                            })
                            .sum();
                        let work: u64 = p.controllers[&person]
                            .history
                            .iter()
                            .filter_map(|r| r.search.as_ref())
                            .map(|m| m.forecast_months)
                            .sum();
                        println!(
                            "{strategy:?},{counterparties:?},{trading},{coins},{person},{},{},{},{},{},{},{},{},{unfilled},{unexpected},{work}",
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
