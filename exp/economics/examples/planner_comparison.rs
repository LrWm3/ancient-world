//! Reproducible P0-P3 controls; stdout is a local artifact, not repository data.
#[path = "../tests/support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    composition::{self, Budget, Strategy},
    compute::Backend,
    model::*,
    scenario::*,
    simulation::Simulation,
};
use std::time::Instant;
fn main() -> Result<(), String> {
    let mode = std::env::var("MODE").unwrap_or("all".into());
    let months = std::env::var("MONTHS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24);
    let horizon = std::env::var("HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12);
    let forecasts = std::env::var("FORECASTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(32);
    let expansions = std::env::var("EXPANSIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(256);
    println!(
        "case,strategy,months,horizon,forecasts_cap,food_deficit,warmth_deficit,crops,failed_crops,arrears,grain,seed,fuel,expansions,forecasts,forecast_months,peak_frontier,exhausted_boundaries,terminal_agents,terminal_months,seconds"
    );
    for &case in fixtures::CASES {
        if std::env::var("CASE").is_ok_and(|c| c != case) {
            continue;
        }
        for (name, strategy) in [
            ("legacy", None),
            ("beam", Some(Strategy::Beam)),
            ("best", Some(Strategy::BestFirst)),
        ] {
            if mode != "all" && mode != name {
                continue;
            }
            let (mut sim, scope) = fixtures::fixture(case, Backend::Reference)?;
            sim.world.decision_horizon = if case.starts_with("B5") {
                None
            } else {
                Some(horizon)
            };
            let started = Instant::now();
            let mut expanded = 0;
            let mut forecasts_run = 0;
            let mut forecast_months = 0;
            let mut peak = 0;
            let mut exhausted = 0;
            let end = sim.state.month + months;
            while sim.state.month < end {
                if sim.state.phase == Phase::Acquire
                    && let Some(strategy) = strategy
                {
                    let selection = composition::choose(
                        &sim,
                        &scope,
                        strategy,
                        Budget {
                            expansions,
                            forecasts,
                            months: horizon,
                        },
                    )?;
                    expanded += selection.metrics.expansions;
                    forecasts_run += selection.metrics.forecasts;
                    forecast_months += selection.metrics.forecast_months;
                    peak = peak.max(selection.metrics.peak_frontier);
                    exhausted += usize::from(selection.metrics.exhausted);
                    if std::env::var_os("DETAIL").is_some() {
                        eprintln!(
                            "{case} {name} month={} {:?} {:?} {:?}",
                            sim.state.month, selection.requests, selection.score, selection.metrics
                        );
                    }
                    selection.accept(&mut sim)?;
                } else {
                    sim.step()?;
                }
            }
            if strategy.is_none() {
                for d in sim.ledger.iter().filter_map(|b| b.decision.as_ref()) {
                    forecasts_run += d.alternatives.len();
                    forecast_months +=
                        u64::from(d.through - d.month + 1) * d.alternatives.len() as u64;
                    exhausted += usize::from(d.search_budget_exhausted);
                }
            }
            let deficit = |r| {
                sim.reports
                    .iter()
                    .map(|p| i64::from(p.deficit(r)))
                    .sum::<i64>()
            };
            let crops = |status| {
                sim.state
                    .processes
                    .values()
                    .filter(|p| [GROW, 99].contains(&p.definition) && p.status == status)
                    .count()
            };
            let balance = |r| {
                sim.world
                    .participants
                    .iter()
                    .map(|p| i64::from(sim.state.balance(p.agent, r)))
                    .sum::<i64>()
                    + sim
                        .world
                        .households
                        .iter()
                        .map(|h| i64::from(sim.state.balance(h.agent, r)))
                        .sum::<i64>()
            };
            println!(
                "{case},{name},{months},{horizon},{forecasts},{},{},{},{},{},{},{},{},{expanded},{forecasts_run},{forecast_months},{peak},{exhausted},{},{},{:.3}",
                deficit(NUTRITION),
                deficit(WARMTH),
                crops(Status::Completed),
                crops(Status::Aborted),
                sim.state
                    .obligations
                    .values()
                    .map(|o| i64::from(o.outstanding()))
                    .sum::<i64>(),
                balance(GRAIN),
                balance(SEED),
                balance(FUEL),
                sim.state.terminal.len(),
                sim.reports.iter().filter(|r| r.terminal.is_some()).count(),
                started.elapsed().as_secs_f64()
            );
        }
    }
    if std::env::var_os("MARKET").is_some() {
        for missing in [false, true] {
            let (w, s) = economics_compute_smoke::production_market::reciprocal_scenario(true);
            let mut sim = Simulation::new(w, s, Backend::Reference)?;
            if missing {
                for a in [91, 92] {
                    sim.state.town_market.positions.insert(a, 100);
                }
            }
            sim.run_months(months)?;
            println!(
                "B4 existing market driver buyer_missing={missing}: food={} warmth={} volumes={:?}",
                sim.reports
                    .iter()
                    .map(|p| p.deficit(NUTRITION))
                    .sum::<i32>(),
                sim.reports.iter().map(|p| p.deficit(WARMTH)).sum::<i32>(),
                sim.state
                    .town_market
                    .history
                    .iter()
                    .flat_map(|r| r.markets.iter().map(|(m, r)| (*m, r.volume)))
                    .collect::<Vec<_>>()
            );
        }
    }
    Ok(())
}
