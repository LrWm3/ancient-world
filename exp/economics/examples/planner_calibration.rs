//! Matched opening forecast, fixed continuation, and realized monthly replanning.
#[path = "../tests/support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    composition::{
        self, Budget, Strategy,
        calibration::{self, Scoring},
    },
    compute::Backend,
    model::*,
    scenario::*,
};

fn main() -> Result<(), String> {
    println!(
        "case,strategy,horizon,scoring,path,food,warmth,crops,aborted,pooled_grain,buffer,changed_packages"
    );
    for case in ["B3", "B5", "B5-one-seed"] {
        if std::env::var("CASE").is_ok_and(|c| c != case) {
            continue;
        }
        for strategy in [Strategy::Beam, Strategy::BestFirst] {
            for horizon in [6, 12, 24] {
                for scoring in [
                    Scoring::PrivateBuffers,
                    Scoring::HouseholdPrivateBuffers,
                    Scoring::HouseholdBuffers,
                ] {
                    let (mut sim, scope) = fixtures::fixture(case, Backend::Reference)?;
                    fixtures::acquire(&mut sim)?;
                    sim.ledger.clear();
                    sim.reports.clear();
                    let budget = Budget {
                        expansions: 256,
                        forecasts: 32,
                        months: horizon,
                    };
                    let selection =
                        composition::choose_with_scoring(&sim, &scope, strategy, budget, scoring)?;
                    let (_, _, forecast) =
                        composition::forecast_package(&sim, &selection.requests, horizon)?;
                    let mut fixed = sim.clone();
                    fixed.world.priority = Priority::ContinuingFirst;
                    fixed.world.competition = None;
                    // The fixed-policy forecast changes continuation configuration,
                    // so commit the same validated opening batch, not a stale selection.
                    economics_compute_smoke::settlement::commit(
                        &fixed.world,
                        &mut fixed.state,
                        &selection.batch,
                        fixed.backend,
                        fixed.effect_limit,
                    )?;
                    fixed.ledger.push(selection.batch.clone());
                    let end = sim.state.month + horizon;
                    while fixed.state.month < end {
                        fixed.step()?;
                    }
                    assert_eq!(fixed.state, forecast.state);
                    assert_eq!(fixed.reports, forecast.reports);
                    selection.accept(&mut sim)?;
                    let mut previous = selection.requests;
                    let mut changes = 0;
                    while sim.state.month < end {
                        if sim.state.phase == Phase::Acquire {
                            let next = composition::choose_with_scoring(
                                &sim, &scope, strategy, budget, scoring,
                            )?;
                            changes += usize::from(next.requests != previous);
                            previous = next.requests.clone();
                            next.accept(&mut sim)?;
                        } else {
                            sim.step()?;
                        }
                    }
                    for (name, result) in [
                        ("forecast", &forecast),
                        ("fixed", &fixed),
                        ("replanned", &sim),
                    ] {
                        let o = calibration::observe(result, scoring)?;
                        let crops = result
                            .state
                            .processes
                            .values()
                            .filter(|p| {
                                [GROW, 99].contains(&p.definition) && p.status == Status::Completed
                            })
                            .count();
                        println!(
                            "{case},{strategy:?},{horizon},{scoring:?},{name},{},{},{crops},{},{},{},{}",
                            o.deficits.get(&NUTRITION).unwrap_or(&0),
                            o.deficits.get(&WARMTH).unwrap_or(&0),
                            o.score.broken_commitments,
                            o.pooled.get(&(fixtures::HOME, GRAIN)).unwrap_or(&0),
                            o.score.buffer_gap,
                            if name == "replanned" { changes } else { 0 }
                        );
                        eprintln!("{case} {strategy:?} h={horizon} {scoring:?} {name}: {o:?}");
                    }
                }
            }
        }
    }
    Ok(())
}
