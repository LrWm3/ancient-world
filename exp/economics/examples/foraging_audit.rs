//! Controlled substitution and crop-contention comparison; generated output is ignored.
use economics_compute_smoke::{compute::Backend, model::*, scenario::*, simulation::Simulation};

fn main() -> Result<(), String> {
    println!("# Foraging comparison\n\nNine executed months per arm; deterministic CubeCL CPU.\n");
    println!(
        "| Scenario | Start | Decision months | Forages | Harvests | Food / warmth deficits | Ending grain | Ending wild supply | Terminal |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    );
    for &name in FORAGING_SCENARIOS {
        let (world, initial) = named(name)?;
        let horizon = world.decision_horizon.unwrap_or(world.horizon);
        let mut cpu = Simulation::new(world.clone(), initial.clone(), Backend::CubeCpu)?;
        cpu.run_months(SCENARIO_MONTHS)?;
        let mut reference = Simulation::new(world, initial.clone(), Backend::Reference)?;
        reference.run_months(SCENARIO_MONTHS)?;
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
        let count = |definition| {
            cpu.ledger
                .iter()
                .flat_map(|b| &b.transactions)
                .filter_map(|t| t.process.as_ref())
                .filter(|p| p.after.definition == definition && p.after.status == Status::Completed)
                .count()
        };
        let forages = count(FORAGE);
        let growth: i32 = cpu
            .ledger
            .iter()
            .filter(|b| b.phase == Phase::Open)
            .flat_map(|b| &b.transactions)
            .flat_map(|t| &t.effects)
            .filter(|e| e.account == (STATE_AGENT, WILD_SUPPLY))
            .map(|e| e.delta)
            .sum();
        assert_eq!(
            initial.balance(STATE_AGENT, WILD_SUPPLY) + growth - forages as i32,
            cpu.state.balance(STATE_AGENT, WILD_SUPPLY)
        );
        println!(
            "| {name} | {} | {horizon} | {forages} | {} | {} / {} | {} | {} | {:?} |",
            initial.month,
            count(GROW),
            cpu.reports
                .iter()
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>(),
            cpu.reports.iter().map(|r| r.deficit(WARMTH)).sum::<i32>(),
            cpu.state.balance(PERSON, GRAIN),
            cpu.state.balance(STATE_AGENT, WILD_SUPPLY),
            cpu.state.terminal
        );
    }
    Ok(())
}
