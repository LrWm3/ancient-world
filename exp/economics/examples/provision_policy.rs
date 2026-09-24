use economics_compute_smoke::{
    compute::Backend,
    minting::{self, COIN, MINT, NUTRITION, REST, provisioning::ProvisionGoal},
    model::Status,
    simulation::Simulation,
    telemetry::{Config, Observer},
};
const MONTHS: u32 = 6;
fn main() -> Result<(), String> {
    let directory =
        std::env::var("TELEMETRY_DIR").map_err(|_| "set TELEMETRY_DIR under ignored output/")?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    for case in ["adequate", "tight", "empty", "endowed", "recovery"] {
        for goal in [ProvisionGoal::FullBuffer, ProvisionGoal::Incremental] {
            let (w, s) = minting::provision_policy_scenario(case, goal)?;
            let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
            let name = format!("{case}-{goal:?}");
            let file = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
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
            observer.run_months(&mut sim, MONTHS)?;
            observer.finish()?;
            let deficit: i32 = sim.reports.iter().map(|r| r.deficit(NUTRITION)).sum();
            let completed = |id| {
                sim.state
                    .processes
                    .values()
                    .filter(|p| p.definition == id && p.status == Status::Completed)
                    .count()
            };
            let supply: i32 = sim
                .state
                .balances
                .iter()
                .filter(|((_, r), _)| *r == COIN)
                .map(|(_, v)| v)
                .sum();
            println!(
                "{name}: food_deficit={deficit} mint_batches={} leisure_sessions={} coins={supply}",
                completed(MINT),
                completed(REST)
            );
        }
    }
    Ok(())
}
