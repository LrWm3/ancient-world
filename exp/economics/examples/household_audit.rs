use economics_compute_smoke::{
    compute::Backend, households, model::ResourceKind, simulation::Simulation,
};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let months = args
        .first()
        .map(|s| s.parse::<u32>())
        .transpose()
        .map_err(|_| "months")?
        .unwrap_or(24);
    let (mut w, s) = households::scenario()?;
    if args.iter().any(|s| s == "control") {
        w.households.clear();
    }
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference)?;
    for _ in 0..months {
        reference.run_months(1)?;
        println!("month {} completed", reference.state.month - 1);
    }
    let allocations: i64 = reference
        .ledger
        .iter()
        .filter_map(|b| b.household.as_ref())
        .flat_map(|b| &b.reservations)
        .map(|r| i64::from(r.allocated))
        .sum();
    let labor: i64 = reference
        .ledger
        .iter()
        .filter_map(|b| b.household.as_ref())
        .flat_map(|b| &b.before)
        .filter(|e| {
            e.delta > 0
                && w.resources
                    .iter()
                    .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Capacity)
        })
        .map(|e| i64::from(e.delta))
        .sum();
    let deficits: i64 = reference
        .reports
        .iter()
        .flat_map(|r| r.needs.values())
        .map(|n| i64::from(n.deficit))
        .sum();
    println!(
        "{} households; {} people; allocations {} stock ticks (mixed resources); reassigned labor {} ticks; total need deficit {}; terminal {}; tax arrears {}",
        w.households.len(),
        w.participants.len(),
        allocations,
        labor,
        deficits,
        reference.state.terminal.len(),
        reference
            .state
            .obligations
            .values()
            .map(|o| i64::from(o.owed - o.paid))
            .sum::<i64>()
    );
    for resource in &w.resources {
        let deficit: i64 = reference
            .reports
            .iter()
            .map(|r| i64::from(r.deficit(resource.id)))
            .sum();
        if deficit > 0 {
            println!("need {} deficits {}", resource.name, deficit);
        }
    }
    println!(
        "tools {}; forwards {}; overdue forward ticks {}",
        reference.state.exchange.contracts.len(),
        reference.state.exchange.forwards.len(),
        reference
            .state
            .exchange
            .forwards
            .values()
            .filter(|c| c.due < reference.state.month)
            .map(|c| i64::from(c.claim().outstanding()))
            .sum::<i64>()
    );
    for a in &w.households {
        println!(
            "household {}: {:?}",
            a.agent,
            reference
                .state
                .balances
                .iter()
                .filter(|((id, _), q)| *id == a.agent && **q > 0)
                .collect::<Vec<_>>()
        );
    }
    if args.get(1).is_some_and(|s| s == "cpu") {
        let mut cpu = Simulation::new(w, s, Backend::CubeCpu)?;
        cpu.run_months(months)?;
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        assert_eq!(reference.reports, cpu.reports);
        println!("CPU/reference exact match");
    }
    Ok(())
}
