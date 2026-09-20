//! Human-readable controlled specialization results; raw traces belong in output/.
use economics_compute_smoke::{
    compute::Backend, crafts::*, model::*, scenario::*, simulation::Simulation,
};

fn main() -> Result<(), String> {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "specialized-activities".into());
    if !SCENARIOS.contains(&name.as_str()) {
        return Err("expected a specialized activities scenario".into());
    }
    let (w, s) = named(&name)?;
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu)?;
    let mut reference = Simulation::new(w, s, Backend::Reference)?;
    cpu.run_months(ACTIVITY_MONTHS)?;
    reference.run_months(ACTIVITY_MONTHS)?;
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    println!(
        "# Specialized activities CPU audit: {name}\n\n36 months; full CPU/reference equality. Starting food, fuel, materials and coins are explicit endowments.\n"
    );
    println!(
        "| Person | Food / warmth / shelter deficit | Grain | Coins | Homes | Terminal |\n| --- | ---: | ---: | ---: | ---: | --- |"
    );
    for p in &cpu.world.participants {
        let deficit = |id| {
            cpu.reports
                .iter()
                .filter(|r| r.agent == p.agent)
                .map(|r| r.deficit(id))
                .sum::<i32>()
        };
        println!(
            "| {} | {} / {} / {} | {} | {} | {} | {:?} |",
            p.agent,
            deficit(NUTRITION),
            deficit(WARMTH),
            deficit(SHELTER),
            cpu.state.balance(p.agent, GRAIN),
            cpu.state.balance(p.agent, TOKEN),
            cpu.state
                .equipment
                .values()
                .filter(|a| a.owner == p.agent && a.kind == HOUSE)
                .count(),
            cpu.state.terminal.get(&p.agent)
        );
    }
    println!("\n| Activity | Completed |\n| --- | ---: |");
    for d in cpu
        .world
        .definitions
        .iter()
        .filter(|d| d.id >= MINE_START && d.execution == Execution::Productive)
    {
        let count = cpu
            .state
            .processes
            .values()
            .filter(|p| p.definition == d.id && p.status == Status::Completed)
            .count();
        println!("| {} | {count} |", d.name);
    }
    println!(
        "\n| Contract | Annual resource | Settled / owed | Native paid |\n| --- | --- | ---: | ---: |"
    );
    for a in &cpu.world.agreements {
        let o: Vec<_> = cpu
            .state
            .obligations
            .values()
            .filter(|o| o.agreement == a.id)
            .collect();
        let name = &cpu
            .world
            .resources
            .iter()
            .find(|r| r.id == a.payment.resource)
            .unwrap()
            .name;
        println!(
            "| {} | {name} | {} / {} | {} |",
            a.id,
            o.iter().map(|o| o.paid).sum::<i32>(),
            o.iter().map(|o| o.owed).sum::<i32>(),
            o.iter().map(|o| o.in_kind_paid).sum::<i32>()
        );
    }
    let total_tokens: i32 = cpu
        .state
        .balances
        .iter()
        .filter(|((_, r), _)| *r == TOKEN)
        .map(|(_, q)| q)
        .sum();
    println!(
        "\nTotal coins: {total_tokens}; state coins: {}. Homes remaining service months: {:?}.\n",
        cpu.state.balance(STATE_AGENT, TOKEN),
        cpu.state
            .equipment
            .values()
            .filter(|a| a.kind == HOUSE)
            .map(|a| a.remaining_uses)
            .collect::<Vec<_>>()
    );
    Ok(())
}
