use economics_compute_smoke::{
    compute::Backend, model::*, plots, scenario::*, simulation::Simulation,
};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let months = args
        .first()
        .map(|s| s.parse::<u32>())
        .transpose()
        .map_err(|_| "months")?
        .unwrap_or(72);
    for enabled in [false, true] {
        if !enabled && args.get(2).is_some_and(|a| a == "only-expanded") {
            continue;
        }
        let (w, s) = plots::scenario(enabled)?;
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference)?;
        sim.run_months(months)?;
        println!(
            "Expansion {enabled}, months {months}: accepted {}",
            sim.state.accepted_agreements.len()
        );
        for b in &sim.ledger {
            if let Some(r) = &b.plot_request {
                println!("month {}: {:?}", b.month, r);
            }
        }
        let grain: i64 = sim
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter(|t| {
                t.process.as_ref().is_some_and(|p| {
                    p.after.status == Status::Completed && p.after.definition == GROW
                })
            })
            .flat_map(|t| &t.effects)
            .filter(|e| e.account.1 == GRAIN && e.delta > 0)
            .map(|e| i64::from(e.delta))
            .sum();
        let tax: i64 = sim
            .state
            .obligations
            .values()
            .filter(|o| sim.state.accepted_agreements.contains_key(&o.agreement))
            .map(|o| i64::from(o.paid))
            .sum();
        let deficits: i64 = sim
            .reports
            .iter()
            .flat_map(|r| r.needs.values())
            .map(|n| i64::from(n.deficit))
            .sum();
        let arrears: i64 = sim
            .state
            .obligations
            .values()
            .map(|o| i64::from(o.owed - o.paid))
            .sum();
        println!(
            "grain ticks {grain}; extra tax ticks {tax}; deficits {deficits}; arrears {arrears}; terminal {}",
            sim.state.terminal.len()
        );
        println!("Terminal details: {:?}", sim.state.terminal);
        if enabled && args.get(1).is_some_and(|a| a == "cpu") {
            let mut cpu = Simulation::new(w, s, Backend::CubeCpu)?;
            cpu.run_months(months)?;
            assert_eq!(sim.state, cpu.state);
            assert_eq!(sim.ledger, cpu.ledger);
            assert_eq!(sim.reports, cpu.reports);
            println!("CPU/reference identical");
        }
    }
    Ok(())
}
