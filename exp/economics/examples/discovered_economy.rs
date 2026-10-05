use economics_compute_smoke::{compute::Backend, discovery::scenario, simulation::Simulation};

fn main() -> Result<(), String> {
    let (w, s) = if std::env::args().any(|a| a == "--surplus") {
        scenario::surplus()?
    } else {
        scenario::scenario()?
    };
    let mut audit = scenario::audit(&w, &s)?;
    let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
    while sim.state.month <= scenario::RUN_MONTHS {
        let month = sim.state.month;
        let phase = sim.state.phase;
        audit
            .step(&mut sim)
            .map_err(|e| format!("month {month} {phase:?}: {e}"))?;
    }
    for receipt in &sim.world.discovery.as_ref().unwrap().receipts {
        println!(
            "month {}: {} accepted={} {:?}",
            receipt.month, receipt.description, receipt.accepted, receipt.comparisons
        );
    }
    println!(
        "memberships={} leases={} households={} processes={} deficit={}",
        sim.state.memberships.len(),
        sim.state.accepted_agreements.len(),
        sim.world.households.len(),
        sim.state.processes.len(),
        sim.reports
            .iter()
            .flat_map(|r| r.needs.values())
            .map(|n| n.deficit)
            .sum::<i32>()
    );
    for (agent, c) in &sim.world.agency {
        println!(
            "agent {agent} chose {:?}",
            c.history
                .iter()
                .filter_map(|d| d.accepted.as_ref().map(|p| (d.month, &p.name)))
                .collect::<Vec<_>>()
        );
    }
    println!("balances: {:?}", sim.state.balances);
    println!(
        "obligations: {:?} forwards: {:?}",
        sim.state.obligations, sim.state.exchange.forwards
    );
    println!("loans: {:?}", sim.state.credit.loans);
    println!("audited {} months on CubeCL CPU", scenario::RUN_MONTHS);
    Ok(())
}
