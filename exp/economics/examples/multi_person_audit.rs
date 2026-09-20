//! Four-person comparison with fixed versus proportionally scaled shared resources.
use economics_compute_smoke::{compute::Backend, model::*, scenario::*, simulation::Simulation};

fn main() -> Result<(), String> {
    println!(
        "# Four-person CPU comparison\n\n60 months; full state, ledger and reports checked against the reference.\n"
    );
    for &name in MULTI_PERSON_SCENARIOS {
        let (world, initial) = named(name)?;
        let mut cpu = Simulation::new(world.clone(), initial.clone(), Backend::CubeCpu)?;
        cpu.run_months(REPEATED_MONTHS)?;
        let mut reference = Simulation::new(world, initial, Backend::Reference)?;
        reference.run_months(REPEATED_MONTHS)?;
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
        println!(
            "## {name}\n\n| Person | Harvests | Forages | Food / warmth deficit | Tax paid / owed | Tokens | Terminal |\n| --- | ---: | ---: | ---: | ---: | ---: | --- |"
        );
        for p in &cpu.world.participants {
            let count = |d| {
                cpu.state
                    .processes
                    .values()
                    .filter(|v| {
                        v.operator == p.agent && v.definition == d && v.status == Status::Completed
                    })
                    .count()
            };
            let deficit = |r| {
                cpu.reports
                    .iter()
                    .filter(|v| v.agent == p.agent)
                    .map(|v| v.deficit(r))
                    .sum::<i32>()
            };
            let obligations: Vec<_> = cpu
                .state
                .obligations
                .values()
                .filter(|o| {
                    cpu.world
                        .agreements
                        .iter()
                        .any(|a| a.id == o.agreement && a.debtor == p.agent)
                })
                .collect();
            println!(
                "| {} | {} | {} | {} / {} | {} / {} | {} | {:?} |",
                p.agent,
                count(GROW),
                count(FORAGE),
                deficit(NUTRITION),
                deficit(WARMTH),
                obligations.iter().map(|o| o.paid).sum::<i32>(),
                obligations.iter().map(|o| o.owed).sum::<i32>(),
                cpu.state.balance(p.agent, TOKEN),
                cpu.state.terminal.get(&p.agent)
            );
        }
        let sold = cpu
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter(|t| t.stock_trade.is_some())
            .count();
        let tokens: i32 = cpu
            .state
            .balances
            .iter()
            .filter(|((_, r), _)| *r == TOKEN)
            .map(|(_, q)| q)
            .sum();
        println!(
            "\nState grain: {}; state tokens: {}; total tokens issued: {tokens}; grain purchases: {sold}.\n",
            cpu.state.balance(STATE_AGENT, GRAIN),
            cpu.state.balance(STATE_AGENT, TOKEN)
        );
    }
    Ok(())
}
