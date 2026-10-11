//! Run the mixed-organization acceptance fixture on CubeCL CPU.
use economics_compute_smoke::{acceptance, compute::Backend};
fn main() -> Result<(), String> {
    let governed = std::env::args().any(|a| a == "--governed");
    if std::env::args().any(|a| a == "--continuation-matrix") {
        use acceptance::continuation as c;
        println!(
            "case,ready,food_deficit,wage_arrears,last_year_workshop_income,last_year_town_income"
        );
        for case in c::Case::ALL {
            let (w, s) = if governed {
                acceptance::governed::scenario(case)?
            } else if std::env::args().any(|a| a == "--spot") {
                c::spot_scenario(case)?
            } else {
                c::scenario(case)?
            };
            let mut run = acceptance::Run::new(w, s, Backend::CubeCpu)?;
            run.through(c::MONTHS)?;
            let evidence = run.evidence()?;
            let report = if governed {
                acceptance::governed::check(&run)?
            } else {
                c::check(&run)?
            };
            let year = report.periods.last().unwrap();
            println!(
                "{case:?},{},{},{},{},{}",
                report.ready(),
                evidence.people_food_deficit,
                evidence.wage_arrears,
                year.workshop_income,
                year.municipal_income
            );
            if report.ready() != (case == c::Case::Baseline) {
                return Err(format!(
                    "unexpected acceptance outcome: {case:?}: {:?}",
                    report.failures
                ));
            }
        }
        return Ok(());
    }
    if !std::env::args().any(|a| {
        matches!(
            a.as_str(),
            "--legacy" | "--matrix" | "--tight-food" | "--no-sales-target"
        )
    }) {
        use acceptance::continuation as c;
        let (w, s) = if governed {
            acceptance::governed::scenario(c::Case::Baseline)?
        } else if std::env::args().any(|a| a == "--spot") {
            c::spot_scenario(c::Case::Baseline)?
        } else {
            c::scenario(c::Case::Baseline)?
        };
        let mut run = acceptance::Run::new(w, s, Backend::CubeCpu)?;
        run.through(c::MONTHS)?;
        let report = if governed {
            acceptance::governed::check(&run)?
        } else {
            c::check(&run)?
        };
        println!("{:#?}\n{report:#?}", run.evidence()?);
        if !report.ready() {
            return Err(format!("acceptance failed: {:?}", report.failures));
        }
        println!(
            "Acceptance passed: {} months, including three recurring operating cycles.",
            c::MONTHS
        );
        return Ok(());
    }
    if std::env::args().any(|a| a == "--matrix") {
        println!(
            "case,food_deficit,smith_deficit,mints,metal_lots,home_food_lots,private_food_lots,smith_food_lots,municipal_work,wage_arrears,workshop_income,storage_rejections,timely_forward"
        );
        for case in acceptance::Case::ALL {
            let (w, s) = acceptance::scenario_for(case)?;
            let mut run = acceptance::Run::new(w, s, Backend::CubeCpu)?;
            run.through(acceptance::RUN_MONTHS)?;
            let e = run.evidence()?;
            println!(
                "{case:?},{},{},{},{},{},{},{},{},{},{},{},{}",
                e.people_food_deficit,
                e.smith_food_deficit,
                e.completed_mints,
                e.workshop_metal_sales,
                e.household_food_sales,
                e.grower_food_sales,
                e.smith_food_purchases,
                e.municipal_batches,
                e.wage_arrears,
                e.workshop_net_income,
                e.grower_storage_rejections,
                e.household_on_time_delivery
            );
        }
        return Ok(());
    }
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
