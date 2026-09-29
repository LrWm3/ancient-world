use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::{Audit, Opening},
    households::{
        income::scenario::{FUEL_MARKET, RUN_MONTHS, costs, scenario},
        market::EXAMPLE_HOUSEHOLD,
    },
    negotiation::GRAIN_MARKET,
    scenario::{FUEL, GRAIN, NUTRITION, SEED, TOKEN},
    simulation::Simulation,
};

fn main() -> Result<(), String> {
    let months = std::env::args()
        .nth(1)
        .map(|s| s.parse::<u32>().map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or(RUN_MONTHS);
    if months == 0 {
        return Err("run length must be positive".into());
    }
    let (w, s) = scenario()?;
    let mut audit = Audit::with_opening(
        &w,
        &s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| [GRAIN, FUEL, SEED].contains(r) && **q > 0)
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            processes: Some(costs()),
            ..Default::default()
        },
    )?;
    let people = w.households[0].adults.clone();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
    println!(
        "| Month | Grain bought | Fuel sold | Household coins | Member food deficit | Directed hours |"
    );
    println!("| --- | --- | --- | --- | --- | --- |");
    for month in 1..=months {
        while sim.state.month <= month {
            audit.step(&mut sim)?;
        }
        let book = sim.state.town_market.history.last().unwrap();
        let deficit: i32 = sim
            .reports
            .iter()
            .filter(|r| r.month == month && people.contains(&r.agent))
            .map(|r| r.deficit(NUTRITION))
            .sum();
        let labor: i32 = sim
            .ledger
            .iter()
            .filter(|b| b.month == month)
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .map(|d| d.granted)
            .sum();
        println!(
            "| {month} | {} | {} | {} | {deficit} | {labor} |",
            book.markets[&GRAIN_MARKET].volume,
            book.markets[&FUEL_MARKET].volume,
            sim.state.balance(EXAMPLE_HOUSEHOLD, TOKEN)
        );
    }
    for agent in &sim.world.agents {
        let r = audit.book().statements(agent.id, 1, months)?;
        if r.assets != r.liabilities + r.equity {
            return Err("unbalanced statements".into());
        }
    }
    println!("All separate financial statements reconcile.");
    Ok(())
}
