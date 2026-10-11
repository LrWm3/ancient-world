//! Run with: cargo +1.92.0 run --release --example reproduction
use economics_compute_smoke::{
    compute::Backend,
    demography::{
        SupportOrder,
        scenario::{MONTHS, scenario},
    },
    financial_reporting::{Audit, Opening},
    scenario::{GRAIN, NUTRITION, TOKEN},
    simulation::Simulation,
};
const OVERLAPPING_MATURITY_MONTHS: u32 = 12;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args
        .iter()
        .any(|a| !matches!(a.as_str(), "--support-matrix" | "--overlapping-cohorts"))
    {
        return Err("usage: reproduction [--support-matrix] [--overlapping-cohorts]".into());
    }
    let matrix = args.iter().any(|a| a == "--support-matrix");
    let overlap = args.iter().any(|a| a == "--overlapping-cohorts");
    println!(
        "supply,reproduction,support_planning,support_order,maturity_months,months,births,living,adults,children,deaths,food_deficit,child_food_deficit,food_remaining,support_requested,support_delivered,support_shortfall"
    );
    for scarce in [false, true] {
        for enabled in [false, true] {
            let policies = if matrix {
                vec![
                    (false, SupportOrder::OldestFirst),
                    (false, SupportOrder::Rotating),
                    (true, SupportOrder::OldestFirst),
                    (true, SupportOrder::Rotating),
                ]
            } else {
                vec![(false, SupportOrder::OldestFirst)]
            };
            for (planning, order) in policies {
                let (mut w, s) = scenario(enabled, scarce);
                let c = w.demography.as_mut().unwrap();
                c.plan_support = planning;
                c.support_order = order;
                if overlap {
                    c.maturity_months = OVERLAPPING_MATURITY_MONTHS;
                }
                let maturity = c.maturity_months;
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
                let (requested, delivered) = sim
                    .ledger
                    .iter()
                    .filter_map(|b| b.demography.as_ref())
                    .flat_map(|d| &d.support)
                    .fold((0i64, 0i64), |(q, d), r| {
                        (q + i64::from(r.requested), d + i64::from(r.delivered))
                    });
                println!(
                    "{},{enabled},{planning},{order:?},{maturity},{MONTHS},{},{living},{},{children},{},{deficit},{child_deficit},{food},{requested},{delivered},{}",
                    if scarce { "scarce" } else { "abundant" },
                    sim.state.demography.births.len(),
                    living - children,
                    sim.state.terminal.len(),
                    requested - delivered
                );
            }
        }
    }
    Ok(())
}
