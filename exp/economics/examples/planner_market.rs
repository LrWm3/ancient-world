//! B4b: one planning producer, a finite passive grain seller/wood buyer.
#[path = "../tests/support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    composition::{self, Budget, Strategy},
    compute::Backend,
    model::*,
    production_market::WOOD_MARKET,
    scenario::*,
};
fn main() -> Result<(), String> {
    println!("control,strategy,horizon,food,warmth,terminal,wood_sales,grain_buys,coins,forecasts");
    for control in [
        "present",
        "no_wood_fills",
        "no_counterparty_cash",
        "out_of_reach",
    ] {
        for horizon in [6, 12, 24] {
            for (name, strategy) in [
                ("fixed", None),
                ("beam", Some(Strategy::Beam)),
                ("best", Some(Strategy::BestFirst)),
            ] {
                let (mut sim, scope) =
                    fixtures::wood_market(Backend::Reference, control != "no_wood_fills")?;
                match control {
                    "no_counterparty_cash" => {
                        sim.state.balances.insert((89, TOKEN), 0);
                    }
                    "out_of_reach" => {
                        sim.state.town_market.positions.insert(89, 100);
                    }
                    _ => {}
                }
                let mut forecasts = 0;
                while sim.state.month < 25 {
                    if sim.state.phase == Phase::Acquire
                        && let Some(strategy) = strategy
                    {
                        let selection = composition::choose(
                            &sim,
                            &scope,
                            strategy,
                            Budget {
                                expansions: 256,
                                forecasts: 32,
                                months: horizon,
                            },
                        )?;
                        forecasts += selection.metrics.forecasts;
                        selection.accept(&mut sim)?;
                    } else {
                        sim.step()?;
                    }
                }
                let deficit = |resource| {
                    sim.reports
                        .iter()
                        .filter(|r| r.agent == PERSON)
                        .map(|r| r.deficit(resource))
                        .sum::<i32>()
                };
                let volume = |market| {
                    sim.state
                        .town_market
                        .history
                        .iter()
                        .map(|r| r.markets[&market].volume)
                        .sum::<i32>()
                };
                println!(
                    "{control},{name},{horizon},{},{},{},{},{},{},{forecasts}",
                    deficit(NUTRITION),
                    deficit(WARMTH),
                    sim.state.terminal.contains_key(&PERSON),
                    volume(WOOD_MARKET),
                    volume(sim.world.town_market.as_ref().unwrap().market),
                    sim.state.balance(PERSON, TOKEN)
                );
            }
        }
    }
    Ok(())
}
