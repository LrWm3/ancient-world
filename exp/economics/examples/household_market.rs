use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::{Audit, Opening},
    households::market::{EXAMPLE_HOUSEHOLD, scenario},
    negotiation::GRAIN_MARKET,
    process_accounting::Costs,
    scenario::{GRAIN, NUTRITION, TOKEN},
    simulation::Simulation,
};

const RUN_MONTHS: u32 = 3;

fn main() -> Result<(), String> {
    let (world, state) = scenario()?;
    let mut audit = Audit::with_opening(
        &world,
        &state,
        TOKEN,
        Opening {
            inventory: state
                .balances
                .iter()
                .filter(|((_, r), q)| *r == GRAIN && **q > 0)
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            processes: Some(Costs::default()),
            ..Default::default()
        },
    )?;
    let members = world.households[0].adults.clone();
    let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
    println!("| Month | Price per lot | Grain traded | Household coins | Member food deficit |");
    println!("| --- | --- | --- | --- | --- |");
    for month in 1..=RUN_MONTHS {
        while sim.state.month <= month {
            audit.step(&mut sim)?;
        }
        let book = &sim.state.town_market.history.last().unwrap().markets[&GRAIN_MARKET];
        let deficit: i32 = sim
            .reports
            .iter()
            .filter(|r| r.month == month && members.contains(&r.agent))
            .map(|r| r.deficit(NUTRITION))
            .sum();
        println!(
            "| {month} | {:?} | {} | {} | {deficit} |",
            book.posted_price,
            book.volume,
            sim.state.balance(EXAMPLE_HOUSEHOLD, TOKEN),
        );
    }
    for id in members.into_iter().chain([EXAMPLE_HOUSEHOLD]) {
        let report = audit.book().statements(id, 1, RUN_MONTHS)?;
        if report.assets != report.liabilities + report.equity {
            return Err(format!("agent {id} statements do not reconcile"));
        }
    }
    println!(
        "Separate household/member statements reconcile; no member balances are consolidated."
    );
    Ok(())
}
