//! Run with: cargo +1.92.0 run --release --example reproduction
use economics_compute_smoke::{
    compute::Backend,
    demography::scenario::{MONTHS, scenario},
    financial_reporting::{Audit, Opening},
    scenario::{GRAIN, NUTRITION, TOKEN},
    simulation::Simulation,
};
fn main() -> Result<(), String> {
    println!(
        "supply,reproduction,months,births,living,adults,children,deaths,food_deficit,child_food_deficit,food_remaining"
    );
    for scarce in [false, true] {
        for enabled in [false, true] {
            let (w, s) = scenario(enabled, scarce);
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    inventory: s
                        .balances
                        .iter()
                        .filter(|((_, r), q)| *r == GRAIN && **q > 0)
                        .map(|(a, q)| (*a, i128::from(*q)))
                        .collect(),
                    services: Some(Default::default()),
                    processes: Some(Default::default()),
                    ..Default::default()
                },
            )?;
            let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
            while sim.state.month <= MONTHS {
                audit.step(&mut sim)?;
            }
            let living = sim
                .world
                .participants
                .iter()
                .filter(|p| !sim.state.terminal.contains_key(&p.agent))
                .count();
            let children = sim
                .state
                .demography
                .births
                .values()
                .filter(|b| {
                    !sim.state.terminal.contains_key(&b.child)
                        && !sim.state.demography.matured.contains(&b.child)
                })
                .count();
            let deficit: i64 = sim
                .reports
                .iter()
                .map(|r| i64::from(r.deficit(NUTRITION)))
                .sum();
            let child_deficit: i64 = sim
                .reports
                .iter()
                .filter(|r| {
                    sim.state
                        .demography
                        .births
                        .get(&r.agent)
                        .is_some_and(|b| r.month < b.adult_at)
                })
                .map(|r| i64::from(r.deficit(NUTRITION)))
                .sum();
            let food: i64 = sim
                .state
                .balances
                .iter()
                .filter(|((_, r), _)| *r == GRAIN)
                .map(|(_, q)| i64::from(*q))
                .sum();
            println!(
                "{},{enabled},{MONTHS},{},{living},{},{children},{},{deficit},{child_deficit},{food}",
                if scarce { "scarce" } else { "abundant" },
                sim.state.demography.births.len(),
                living - children,
                sim.state.terminal.len()
            );
        }
    }
    Ok(())
}
