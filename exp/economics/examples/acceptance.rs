//! Run the mixed-organization acceptance fixture on CubeCL CPU.
use economics_compute_smoke::{acceptance, compute::Backend};
fn main() -> Result<(), String> {
    let case = if std::env::args().any(|a| a == "--tight-food") {
        acceptance::Case::TightFood
    } else if std::env::args().any(|a| a == "--no-sales-target") {
        acceptance::Case::NoSalesTarget
    } else {
        acceptance::Case::Baseline
    };
    let (w, s) = acceptance::scenario_for(case)?;
    let mut run = acceptance::Run::new(w, s, Backend::CubeCpu)?;
    run.through(acceptance::RUN_MONTHS)?;
    println!("{:#?}", run.evidence()?);
    if std::env::args().any(|a| a == "--details") {
        for b in &run.sim.ledger {
            for r in &b.receipts {
                if r.agent == economics_compute_smoke::agency::integration::GROWER {
                    println!("month={} phase={:?} grower={r:?}", b.month, b.phase);
                }
            }
        }
        for agent in &run.sim.world.agents {
            let report = run
                .audit
                .book()
                .statements(agent.id, 1, acceptance::RUN_MONTHS)?;
            println!(
                "agent={} name={:?} cash={} net_income={} wheat={}",
                agent.id,
                agent.name,
                report.closing_cash,
                report.net_income,
                run.sim
                    .state
                    .balance(agent.id, economics_compute_smoke::minting::WHEAT)
            );
        }
    }
    Ok(())
}
