//! Controlled fiscal/storage comparison; generated output belongs under output/.
use economics_compute_smoke::{compute::Backend, model::*, scenario::*, simulation::Simulation};

fn main() -> Result<(), String> {
    println!(
        "# Storage and currency comparison\n\n60 months per arm; CubeCL CPU checked against reference.\n"
    );
    println!(
        "| Scenario | Tax collected | Grain sold | Tokens state / person | State grain | Harvests | Food / warmth deficits |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
    for &name in CURRENCY_SCENARIOS {
        let (world, initial) = named(name)?;
        let mut cpu = Simulation::new(world.clone(), initial.clone(), Backend::CubeCpu)?;
        cpu.run_months(60)?;
        let mut reference = Simulation::new(world, initial, Backend::Reference)?;
        reference.run_months(60)?;
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
        let tax: i32 = cpu.state.obligations.values().map(|o| o.paid).sum();
        let sold = cpu
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter(|t| t.stock_trade.is_some())
            .count();
        let harvests = cpu
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter_map(|t| t.process.as_ref())
            .filter(|p| p.after.definition == GROW && p.after.status == Status::Completed)
            .count();
        assert_eq!(cpu.state.balance(STATE_AGENT, GRAIN), tax + sold as i32);
        println!(
            "| {name} | {tax} | {sold} | {} / {} | {} | {harvests} | {} / {} |",
            cpu.state.balance(STATE_AGENT, TOKEN),
            cpu.state.balance(PERSON, TOKEN),
            cpu.state.balance(STATE_AGENT, GRAIN),
            cpu.reports
                .iter()
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>(),
            cpu.reports.iter().map(|r| r.deficit(WARMTH)).sum::<i32>()
        );
    }
    Ok(())
}
