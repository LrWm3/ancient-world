//! Equal-horizon, 24-month continuation comparison. CSV goes to ignored output/.
#[path = "../tests/support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    composition::{
        Budget, Strategy,
        calibration::{self, Scoring},
        continuation::{Controller, Policy, Reason},
    },
    compute::Backend,
    model::*,
    scenario::*,
};
use std::time::Instant;

const RUN_MONTHS: u32 = 24;
const SEARCH_EXPANSIONS: usize = 256;
const SEARCH_FORECASTS: usize = 32;
const SHOCK_MONTH: u32 = 6;

fn main() -> Result<(), String> {
    println!(
        "case,strategy,horizon,shock,policy,food,warmth,crops,aborted,arrears,pooled_grain,searches,repairs,start_set_changes,forecasts,search_months,projection_months,projection_steps,replay_steps,cheap_steps,elapsed_ms,harvest_months,terminal_months,impaired_months,active_crops,stored_grain,buffer_gap"
    );
    for case in ["B3", "B5", "B5-one-seed"] {
        if std::env::var("CASE").is_ok_and(|v| v != case) {
            continue;
        }
        for strategy in [Strategy::Beam, Strategy::BestFirst] {
            for months in [6, 12, 24] {
                for shock in ["none", "labor", "stock"] {
                    for policy in [
                        Policy::Monthly,
                        Policy::RetainRepair,
                        Policy::ScheduledReview,
                    ] {
                        let (mut sim, scope) = fixtures::fixture(case, Backend::Reference)?;
                        sim.world.priority = Priority::ContinuingFirst;
                        if shock == "labor" {
                            sim.world
                                .capacity_overrides
                                .insert((SHOCK_MONTH, PERSON), 0);
                        }
                        let mut controller = Controller::new(
                            scope,
                            strategy,
                            Budget {
                                expansions: SEARCH_EXPANSIONS,
                                forecasts: SEARCH_FORECASTS,
                                months,
                            },
                            policy,
                        );
                        let begin = Instant::now();
                        while sim.state.month <= RUN_MONTHS {
                            if shock == "stock"
                                && sim.state.month == SHOCK_MONTH
                                && sim.state.phase == Phase::Acquire
                            {
                                // Observed external stock loss; no prospective warning.
                                // This intervention is outside the financial-audit fixture.
                                for ((_, r), q) in &mut sim.state.balances {
                                    if *r == GRAIN {
                                        *q = 0;
                                    }
                                }
                            }
                            controller.step(&mut sim)?;
                        }
                        let elapsed = begin.elapsed().as_millis();
                        let o = calibration::observe(&sim, Scoring::PrivateBuffers)?;
                        let harvests: Vec<_> = o
                            .completions
                            .iter()
                            .filter(|(_, _, d)| [GROW, 99].contains(d))
                            .map(|(m, _, _)| m.to_string())
                            .collect();
                        let h = &controller.history;
                        let m: Vec<_> = h.iter().filter_map(|r| r.search.as_ref()).collect();
                        println!(
                            "{case},{strategy:?},{months},{shock},{policy:?},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{elapsed},{},{},{},{},{},{}",
                            o.deficits.get(&NUTRITION).unwrap_or(&0),
                            o.deficits.get(&WARMTH).unwrap_or(&0),
                            harvests.len(),
                            o.score.broken_commitments,
                            o.arrears.values().sum::<i64>(),
                            o.pooled.get(&(fixtures::HOME, GRAIN)).unwrap_or(&0),
                            m.len(),
                            h.iter()
                                .filter(|r| r.reason == Reason::ObservationChanged)
                                .count(),
                            h.windows(2).filter(|w| w[0].starts != w[1].starts).count(),
                            m.iter().map(|m| m.forecasts).sum::<usize>(),
                            m.iter().map(|m| m.forecast_months).sum::<u64>(),
                            h.iter().map(|r| r.projection_months).sum::<u32>(),
                            h.iter().map(|r| r.projection_steps).sum::<usize>(),
                            h.iter().map(|r| r.replay_steps).sum::<usize>(),
                            h.iter().map(|r| r.cheap_preview_steps).sum::<usize>(),
                            harvests.join(";"),
                            o.score.terminal_months,
                            o.score.impaired_months,
                            sim.state
                                .processes
                                .values()
                                .filter(|p| [GROW, 99].contains(&p.definition)
                                    && p.status == Status::Active)
                                .count(),
                            sim.state
                                .balances
                                .iter()
                                .filter(|((_, r), _)| *r == GRAIN)
                                .map(|(_, q)| i64::from(*q))
                                .sum::<i64>(),
                            o.score.buffer_gap
                        );
                        eprintln!(
                            "{case} {strategy:?} horizon={months} {shock} {policy:?}: {:?}",
                            h.iter()
                                .map(|r| (r.month, r.reason, r.observation_changed))
                                .collect::<Vec<_>>()
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
