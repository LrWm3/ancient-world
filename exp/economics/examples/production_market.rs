use economics_compute_smoke::{
    compute::Backend,
    model::{ResourceKind, Status},
    production_market,
    scenario::{LABOR, NUTRITION, TOKEN, WARMTH},
    simulation::Simulation,
};
fn main() -> Result<(), String> {
    for (trading, missing_buyers) in [(false, false), (true, false), (true, true)] {
        let (w, s) = production_market::scenario(trading);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        let months = std::env::var("MONTHS")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(24);
        if missing_buyers {
            let opening = months.min(12);
            sim.run_months(opening)?;
            for a in [91, 92] {
                sim.state.town_market.positions.insert(a, 100);
            }
            sim.run_months(months - opening)?;
        } else {
            sim.run_months(months)?;
        }
        let deficit = |r| sim.reports.iter().map(|p| p.deficit(r)).sum::<i32>();
        let labor = sim
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .flat_map(|t| &t.effects)
            .filter(|e| e.account.1 == LABOR && e.delta < 0)
            .map(|e| -i64::from(e.delta))
            .sum::<i64>();
        println!(
            "trade={trading} missing_buyers={missing_buyers}: nutrition={} warmth={} labor={labor} grain_traded={} completed={} failed={}",
            deficit(NUTRITION),
            deficit(WARMTH),
            sim.state
                .town_market
                .history
                .iter()
                .map(|r| r.volume)
                .sum::<i32>(),
            sim.state
                .processes
                .values()
                .filter(|p| p.status == Status::Completed)
                .count(),
            sim.state
                .processes
                .values()
                .filter(|p| p.status == Status::Aborted)
                .count()
        );
        for p in &sim.world.participants {
            println!(
                "person {} coins {} stocks {:?}",
                p.agent,
                sim.state.balance(p.agent, TOKEN),
                sim.world
                    .resources
                    .iter()
                    .filter(|r| r.kind == ResourceKind::Stock)
                    .map(|r| (&r.name, sim.state.balance(p.agent, r.id)))
                    .collect::<Vec<_>>()
            );
        }
        for r in &sim.state.town_market.history {
            if std::env::var_os("DETAIL").is_some() {
                println!("orders {:?} planning {:?}", r.orders, r.planning);
            }
            let plans = r.planning.as_ref().unwrap();
            println!(
                "month {} price {:?} volume {} choices {:?}",
                r.month,
                r.posted_price,
                r.volume,
                plans
                    .people
                    .iter()
                    .map(|p| (p.agent, p.alternatives[p.selected].choice))
                    .collect::<Vec<_>>()
            );
        }
    }
    Ok(())
}
