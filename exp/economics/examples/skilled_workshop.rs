use economics_compute_smoke::{
    acceptance::{SMITH, WORKSHOP_EMPLOYMENT, maintained_workshop, skilled_workshop},
    compute::Backend,
};
const ACCEPTANCE_MONTHS: u32 = 48;
const REPORT_MONTHS: u32 = 12;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let months = args
        .iter()
        .find_map(|v| v.parse::<u32>().ok())
        .unwrap_or(ACCEPTANCE_MONTHS);
    let benefit = !args.iter().any(|v| v == "--no-benefit");
    let backend = if args.iter().any(|v| v == "--cpu") {
        Backend::CubeCpu
    } else {
        Backend::Reference
    };
    let mut run = skilled_workshop::run(benefit, backend)?;
    for month in 1..=months {
        run.through(month)?;
        if args.iter().any(|v| v == "--details")
            || month == months
            || month.is_multiple_of(REPORT_MONTHS)
        {
            println!("{:?}", run.evidence()?);
            println!(
                "practice={} workshop_hours={} worker_cash={}",
                run.sim
                    .state
                    .practice
                    .get(&(SMITH, skilled_workshop::COMPETENCY))
                    .copied()
                    .unwrap_or(0),
                run.sim
                    .state
                    .employment
                    .earned
                    .iter()
                    .filter(|((id, _), _)| *id == WORKSHOP_EMPLOYMENT)
                    .map(|(_, e)| e.delivered)
                    .sum::<i32>(),
                run.sim
                    .state
                    .balance(SMITH, economics_compute_smoke::minting::COIN)
            );
        }
        if args.iter().any(|v| v == "--details") {
            for ((m, process, stage), work) in &run.sim.state.employment.work {
                if *m == month {
                    println!("work process={process} stage={stage} {work:?}");
                }
            }
        }
    }
    if months == ACCEPTANCE_MONTHS {
        let readiness = maintained_workshop::check(&run)?;
        println!("{readiness:?}");
        if benefit && !readiness.ready() {
            return Err("skilled-workshop acceptance failed".into());
        }
    }
    Ok(())
}
