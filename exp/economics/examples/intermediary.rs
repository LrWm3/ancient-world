use economics_compute_smoke::{
    allocation::Outcome, compute::Backend, intermediary, model::Status, scenario::*,
};
fn main() -> Result<(), String> {
    let mut e = intermediary::scenario(Backend::CubeCpu)?;
    e.run_months(intermediary::RUN_MONTHS)?;
    println!("# Intermediary resource experiment\n\nCubeCL CPU; months 2–13; stable priority.\n");
    println!(
        "| Month | Person | Proposed process | Reserved | Fallback process |\n| --- | --- | --- | --- | --- |"
    );
    for r in &e.rounds {
        for d in &r.decisions {
            let reserved = r.resolution.receipts.iter().any(|r| {
                r.claim.id == u64::from(d.agent) && matches!(r.outcome, Outcome::Reserved(_))
            });
            println!(
                "| {} | {} | {:?} | {} | {:?} |",
                r.month, d.agent, d.selected, reserved, d.fallback
            );
        }
    }
    println!("\nProcesses: 1 = cultivate, 4 = collect fuel, 700 = make tool.\n");
    println!(
        "| Harvest month | Person | Grain | Seed | Tool used |\n| --- | --- | --- | --- | --- |"
    );
    for b in &e.simulation.ledger {
        for t in &b.transactions {
            if let Some(p) = &t.process
                && p.after.definition == GROW
                && p.after.status == Status::Completed
            {
                let output = |resource| {
                    t.effects
                        .iter()
                        .filter(|v| v.account == (p.after.operator, resource) && v.delta > 0)
                        .map(|v| v.delta)
                        .sum::<i32>()
                };
                println!(
                    "| {} | {} | {} | {} | {} |",
                    b.month,
                    p.after.operator,
                    output(GRAIN),
                    output(SEED),
                    t.technique_use.is_some()
                );
            }
        }
    }
    for p in &e.simulation.world.participants {
        let deficit = |need| {
            e.simulation
                .reports
                .iter()
                .filter(|r| r.agent == p.agent)
                .map(|r| r.deficit(need))
                .sum::<i32>()
        };
        println!(
            "\nPerson {}: unmet nutrition {}, unmet warmth {}; terminal: {}.",
            p.agent,
            deficit(NUTRITION),
            deficit(WARMTH),
            e.simulation.state.terminal.contains_key(&p.agent)
        );
    }
    Ok(())
}
