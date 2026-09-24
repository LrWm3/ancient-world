use economics_compute_smoke::{
    compute::Backend,
    credit,
    credit_stress::{self, Case, OBSERVATION_MONTHS},
    scenario::{GRAIN, NUTRITION, PERSON, PLOT, STATE_AGENT, TOKEN},
    simulation::Simulation,
    telemetry::{Config, Observer},
};

fn main() -> Result<(), String> {
    println!(
        "Operational diagnostics only; this crop-transfer scenario has no complete financial report adapter yet."
    );
    let directory =
        std::env::var("TELEMETRY_DIR").map_err(|_| "set TELEMETRY_DIR under ignored output/")?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    for case in [Case::Normal, Case::Temporary, Case::Persistent] {
        let name = format!("{case:?}");
        let (w, s) = credit_stress::scenario(case)?;
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::Path::new(&directory).join(format!("{name}.jsonl")))
            .map_err(|e| e.to_string())?;
        let mut observer = Observer::new(
            std::io::BufWriter::new(file),
            &name,
            Config {
                settlement: true,
                ..Config::default()
            },
        )?;
        for month in 1..=OBSERVATION_MONTHS {
            observer.run_months(&mut sim, 1)?;
            let loan = &sim.state.credit.loans[&1];
            let borrower_cash = sim.state.balance(PERSON, TOKEN);
            let lender_cash = sim.state.balance(STATE_AGENT, TOKEN);
            println!(
                "{name} month={month} owner={:?} status={:?} pledged={} debt={} arrears={:?} grain={} borrower_cash={borrower_cash} lender_cash={lender_cash}",
                credit::owner(&sim.world, &sim.state, PLOT),
                loan.status,
                loan.collateral.as_ref().is_some_and(|c| c.pledged),
                loan.debt()?,
                loan.first_unpaid,
                sim.state.balance(PERSON, GRAIN)
            );
            for b in sim.ledger.iter().filter(|b| b.month == month) {
                if let Some(c) = &b.credit {
                    for e in &c.events {
                        println!("{name} {month} {:?} {e:?}", b.phase);
                    }
                }
            }
        }
        observer.finish()?;
        let deficit: i32 = sim
            .reports
            .iter()
            .filter(|r| r.agent == PERSON)
            .map(|r| r.deficit(NUTRITION))
            .sum();
        println!(
            "{name} deficit={deficit} stock_spent={} crops={:?}",
            sim.state.credit.stock_spent, sim.state.processes
        );
    }
    Ok(())
}
