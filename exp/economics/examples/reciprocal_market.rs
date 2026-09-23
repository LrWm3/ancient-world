use economics_compute_smoke::{
    compute::Backend,
    model::Status,
    negotiation::GRAIN_MARKET,
    production_market::{self, WOOD_MARKET},
    scenario::{LABOR, NUTRITION, TOKEN, WARMTH},
    simulation::Simulation,
};
fn main() -> Result<(), String> {
    let months = std::env::var("MONTHS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(48);
    for mode in ["none", "grain", "both", "directed"] {
        if std::env::var("CASE").is_ok_and(|v| v != mode) {
            continue;
        }
        let (mut w, s) = if mode == "both" || mode == "directed" {
            production_market::reciprocal_scenario(true)
        } else {
            production_market::scenario(mode == "grain")
        };
        w.production_market.as_mut().unwrap().demand =
            production_market::DemandSignal::IncludeUnfilledBids;
        if mode == "directed" {
            use production_market::{Choice, Policy, Purchases, Work};
            // Diagnostic only: establish whether chosen complementary work can
            // exchange through the same books without relying on search.
            let policies = w
                .participants
                .iter()
                .map(|p| {
                    let grower = [88, 89].contains(&p.agent);
                    (
                        p.agent,
                        Choice {
                            work: Work::Produce(if grower {
                                economics_compute_smoke::scenario::GROW
                            } else {
                                economics_compute_smoke::scenario::PREPARE_FUEL
                            }),
                            buy: Purchases::Market(if grower { WOOD_MARKET } else { GRAIN_MARKET }),
                        },
                    )
                })
                .collect();
            w.production_market.as_mut().unwrap().policy = Policy::Fixed(policies);
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        for _ in 0..months {
            sim.run_months(1)?;
            if std::env::var_os("DETAIL").is_some() {
                println!(
                    "month {} balances {:?} decisions {:?}",
                    sim.state.month - 1,
                    sim.state.balances,
                    sim.state.town_market.history.last().unwrap().planning
                );
            }
            if sim.state.month % 12 == 1 {
                let r = sim.state.town_market.history.last().unwrap();
                let volume = |market| {
                    sim.state
                        .town_market
                        .history
                        .iter()
                        .filter_map(|r| r.markets.get(&market))
                        .map(|m| m.volume)
                        .sum::<i32>()
                };
                let deficit = |need| sim.reports.iter().map(|r| r.deficit(need)).sum::<i32>();
                let labor = sim
                    .ledger
                    .iter()
                    .flat_map(|b| &b.transactions)
                    .flat_map(|t| &t.effects)
                    .filter(|e| e.account.1 == LABOR && e.delta < 0)
                    .map(|e| -i64::from(e.delta))
                    .sum::<i64>();
                let coins: Vec<_> = sim
                    .world
                    .participants
                    .iter()
                    .map(|p| (p.agent, sim.state.balance(p.agent, TOKEN)))
                    .collect();
                println!(
                    "{mode} month={} food={} warmth={} grain={} wood={} labor={labor} coins={coins:?} failed={}",
                    r.month,
                    deficit(NUTRITION),
                    deficit(WARMTH),
                    volume(GRAIN_MARKET),
                    volume(WOOD_MARKET),
                    sim.state
                        .processes
                        .values()
                        .filter(|p| p.status == Status::Aborted)
                        .count()
                );
            }
        }
        for r in &sim.state.town_market.history {
            println!("{mode} month={} books={:?}", r.month, r.markets);
        }
    }
    Ok(())
}
