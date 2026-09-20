use economics_compute_smoke::{
    compute::Backend, exchange::DEFAULT_CAPTURE_PERCENT, scenario::*, simulation::Simulation,
    trading_scenario::*,
};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let counts: Vec<usize> = if let Some(count) = args.first() {
        vec![count.parse().map_err(|_| "provider count")?]
    } else {
        (1..=MAX_PROVIDERS).collect()
    };
    let capture = args
        .get(2)
        .map(|s| s.parse::<u32>())
        .transpose()
        .map_err(|_| "capture percent")?
        .unwrap_or(DEFAULT_CAPTURE_PERCENT);
    println!("72-month provider sweep; stocks in physical units, output share {capture}%.");
    println!(
        "| Providers | Delivered tools | Food deficits | Unpaid tax | Minimum provider net food income/month, final 24 months | Supported |"
    );
    let mut results = Vec::new();
    for count in counts {
        let (w, s) = scenario(count, capture)?;
        let mut sim = Simulation::new(w, s, Backend::Reference)?;
        sim.run_months(TRADING_MONTHS)?;
        let makers = providers(count);
        let assessments = assess(&sim, count);
        let min = assessments
            .iter()
            .map(|a| {
                (a.food_income - a.material_spend) as f64
                    / f64::from(STOCK_UNIT * EVALUATION_MONTHS as i32)
            })
            .fold(f64::INFINITY, f64::min);
        let deficits: i32 = sim.reports.iter().map(|r| r.deficit(NUTRITION)).sum();
        let arrears: i32 = sim
            .state
            .obligations
            .values()
            .map(|o| o.owed - o.paid)
            .sum();
        let supported = assessments.iter().all(|a| a.supported) && sim.state.terminal.is_empty();
        results.push((count, supported));
        println!(
            "| {count} | {} | {deficits} | {:.2} | {min:.3} | {supported} |",
            sim.state.exchange.contracts.len(),
            arrears as f64 / f64::from(STOCK_UNIT)
        );
        for maker in makers {
            let earned: Vec<_> = sim
                .state
                .exchange
                .earned
                .iter()
                .filter(|((owner, _), _)| *owner == maker)
                .map(|((_, resource), q)| (*resource, *q as f64 / f64::from(STOCK_UNIT)))
                .collect();
            println!(
                "provider {maker}: earned {earned:?}; closing grain {:.2}; coins {:.2}",
                sim.state.balance(maker, GRAIN) as f64 / f64::from(STOCK_UNIT),
                sim.state.balance(maker, TOKEN) as f64 / f64::from(STOCK_UNIT)
            );
        }
        if args.get(1).is_some_and(|s| s == "cpu") {
            let (w, s) = scenario(count, capture)?;
            let mut cpu = Simulation::new(w, s, Backend::CubeCpu)?;
            cpu.run_months(TRADING_MONTHS)?;
            assert_eq!(cpu.state, sim.state);
            assert_eq!(cpu.ledger, sim.ledger);
            assert_eq!(cpu.reports, sim.reports);
            println!("CPU/reference state, ledger and reports match exactly.");
        }
    }
    println!(
        "Selected provider count (minimum-one fallback): {}",
        select_provider_count(&results)
    );
    Ok(())
}
