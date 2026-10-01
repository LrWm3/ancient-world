//! Multi-person continuation and contention, with per-person outcomes.
#[path = "../tests/support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    allocation::{Outcome, Policy as Allocation},
    composition::{
        Budget, Strategy,
        continuation::{Policy, persons::Persons},
    },
    compute::Backend,
    model::*,
    scenario::*,
};
const RUN_MONTHS: u32 = 24;
const FORECAST_MONTHS: u32 = 12;
const SEARCH_EXPANSIONS: usize = 256;
const SEARCH_FORECASTS: usize = 32;
const SHOCK_MONTH: u32 = 6;
const SEED: u64 = 7;

fn main() -> Result<(), String> {
    println!(
        "people,plots,allocation,review,shock,person,crops,food,warmth,terminal_months,aborted,active,arrears,searches,denied,fallbacks,search_months,projection_months,replay_steps,harvest_months"
    );
    for people in [2, 4] {
        for plots in [1, people] {
            for allocation in [Allocation::StablePriority, Allocation::Lottery] {
                for review in [
                    Policy::Monthly,
                    Policy::RetainRepair,
                    Policy::ScheduledReview,
                ] {
                    for shock in [false, true] {
                        let mut sim = fixtures::persons(people, plots, Backend::Reference)?;
                        if shock {
                            sim.world
                                .capacity_overrides
                                .insert((SHOCK_MONTH, PERSON), 0);
                        }
                        let mut persons = Persons::new(
                            sim.world.participants.iter().map(|p| p.agent),
                            Strategy::BestFirst,
                            Budget {
                                expansions: SEARCH_EXPANSIONS,
                                forecasts: SEARCH_FORECASTS,
                                months: FORECAST_MONTHS,
                            },
                            review,
                            allocation,
                            SEED,
                        )?;
                        while sim.state.month <= RUN_MONTHS {
                            persons.step(&mut sim)?;
                        }
                        for (&person, controller) in &persons.controllers {
                            let reports: Vec<_> =
                                sim.reports.iter().filter(|r| r.agent == person).collect();
                            let food: i64 = reports
                                .iter()
                                .map(|r| i64::from(r.deficit(NUTRITION)))
                                .sum();
                            let warmth: i64 =
                                reports.iter().map(|r| i64::from(r.deficit(WARMTH))).sum();
                            let terminal = reports.iter().filter(|r| r.terminal.is_some()).count();
                            let harvests: Vec<_> = sim
                                .ledger
                                .iter()
                                .flat_map(|b| {
                                    b.transactions.iter().filter_map(move |t| {
                                        let p = &t.process.as_ref()?.after;
                                        (p.operator == person
                                            && p.definition == GROW
                                            && p.status == Status::Completed)
                                            .then_some(b.month.to_string())
                                    })
                                })
                                .collect();
                            let aborted = sim
                                .state
                                .processes
                                .values()
                                .filter(|p| p.operator == person && p.status == Status::Aborted)
                                .count();
                            let active = sim
                                .state
                                .processes
                                .values()
                                .filter(|p| p.operator == person && p.status == Status::Active)
                                .count();
                            let arrears: i64 = sim
                                .state
                                .obligations
                                .values()
                                .filter(|o| {
                                    sim.state
                                        .accepted_agreements
                                        .get(&o.agreement)
                                        .is_some_and(|a| a.debtor == person)
                                })
                                .map(|o| i64::from(o.outstanding()))
                                .sum();
                            let h = &controller.history;
                            let denied = persons
                                .history
                                .iter()
                                .flat_map(|r| &r.admission)
                                .filter(|r| {
                                    r.claim.id == u64::from(person)
                                        && !matches!(r.outcome, Outcome::Reserved(_))
                                })
                                .count();
                            let fallbacks = persons
                                .history
                                .iter()
                                .filter(|r| r.fallback.contains_key(&person))
                                .count();
                            println!(
                                "{people},{plots},{allocation:?},{review:?},{shock},{person},{},{food},{warmth},{terminal},{aborted},{active},{arrears},{},{denied},{fallbacks},{},{},{},{}",
                                harvests.len(),
                                h.iter().filter(|r| r.search.is_some()).count(),
                                h.iter()
                                    .filter_map(|r| r.search.as_ref())
                                    .map(|m| m.forecast_months)
                                    .sum::<u64>(),
                                h.iter().map(|r| r.projection_months).sum::<u32>(),
                                h.iter().map(|r| r.replay_steps).sum::<usize>(),
                                harvests.join(";")
                            );
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
