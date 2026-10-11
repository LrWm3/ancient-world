//! Reproducible aged-world diagnostic-copy comparison; not a throughput benchmark.
use economics_compute_smoke::{
    acceptance::{
        continuation::Case,
        financing::{self, Liquidity, Stocks},
    },
    compute::Backend,
    forecast::ForecastContext,
    model::{State, World},
    simulation::Simulation,
};
use std::time::Instant;

const AGE_MONTHS: u32 = 12;
const ROLLOUT_MONTHS: u32 = 12;
const REPEATS: usize = 2;

fn rollout(w: World, s: State) -> Result<(Simulation, u128), String> {
    let clock = Instant::now();
    let mut sim = Simulation::new(w, s, Backend::Reference)?;
    sim.run_months(ROLLOUT_MONTHS)?;
    Ok((sim, clock.elapsed().as_micros()))
}

fn main() -> Result<(), String> {
    for policy in [Liquidity::TerminalReserve, Liquidity::MonthlyExposure] {
        let (w, s) = financing::with_liquidity(Case::Baseline, true, Stocks::Both, policy)?;
        let mut live = Simulation::new(w, s, Backend::Reference)?;
        live.run_months(AGE_MONTHS)?;
        let projections = live
            .world
            .agency
            .values()
            .flat_map(|c| &c.history)
            .flat_map(|d| &d.alternatives)
            .filter(|a| a.projection.is_some())
            .count();
        let attempts: usize = live
            .world
            .agency
            .values()
            .map(|c| c.forward_reviews.len())
            .sum();
        for sample in 0..REPEATS {
            let clock = Instant::now();
            let (compact, s) = ForecastContext::new(&live.world, &live.state).into_parts();
            let context_us = clock.elapsed().as_micros();
            let mut detailed = compact.clone();
            // Rehydrate only the exact payloads removed by the optimization.
            for (&agent, original) in &live.world.agency {
                let c = detailed.agency.get_mut(&agent).unwrap();
                c.history = original.history.clone();
                c.forward_reviews = original.forward_reviews.clone();
            }
            // Alternate ordering to avoid always favoring one warm branch.
            let (small, small_us, full, full_us) = if sample % 2 == 0 {
                let (small, small_us) = rollout(compact, s.clone())?;
                let (full, full_us) = rollout(detailed, s)?;
                (small, small_us, full, full_us)
            } else {
                let (full, full_us) = rollout(detailed, s.clone())?;
                let (small, small_us) = rollout(compact, s)?;
                (small, small_us, full, full_us)
            };
            assert_eq!(small.state, full.state);
            assert_eq!(small.ledger, full.ledger);
            assert_eq!(small.reports, full.reports);
            assert_eq!(
                ForecastContext::new(&small.world, &small.state),
                ForecastContext::new(&full.world, &full.state)
            );
            println!(
                "{policy:?} sample={sample} age={AGE_MONTHS} horizon={ROLLOUT_MONTHS} projections={projections} attempts={attempts} context_us={context_us} compact_us={small_us} detailed_us={full_us}"
            );
        }
    }
    Ok(())
}
