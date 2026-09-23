use economics_compute_smoke::{
    compute::Backend,
    model::Status,
    negotiation::GRAIN_MARKET,
    production_market::{self, WOOD_MARKET},
    scenario::{LABOR, NUTRITION, TOKEN, WARMTH},
    simulation::Simulation,
    telemetry::{Config, Observer, PlanningDetail},
};
const DEFAULT_MONTHS: u32 = 48;

// All configuration stays in the runner; economic code knows nothing of telemetry.
fn observer(mode: &str) -> Result<Option<Observer<std::io::BufWriter<std::fs::File>>>, String> {
    let Some(directory) = std::env::var_os("TELEMETRY_DIR") else {
        return Ok(None);
    };
    let mut config = Config::default();
    if let Ok(value) = std::env::var("TELEMETRY_PLANNING") {
        config.planning = match value.as_str() {
            "off" => PlanningDetail::Off,
            "selected" => PlanningDetail::Selected,
            "alternatives" => PlanningDetail::Alternatives,
            _ => return Err("TELEMETRY_PLANNING must be off, selected, or alternatives".into()),
        };
    }
    if let Ok(value) = std::env::var("TELEMETRY_SETTLEMENT") {
        config.settlement = match value.as_str() {
            "true" => true,
            "false" => false,
            _ => return Err("TELEMETRY_SETTLEMENT must be true or false".into()),
        };
    }
    if let Ok(value) = std::env::var("TELEMETRY_MODE") {
        match value.as_str() {
            "metrics" => config.logs = false,
            "logs" => config.metrics = false,
            "both" => {}
            _ => return Err("TELEMETRY_MODE must be metrics, logs, or both".into()),
        }
    }
    if let Ok(value) = std::env::var("TELEMETRY_AGENTS") {
        config.agents = value
            .split(',')
            .map(|s| {
                s.trim().parse().map_err(|_| {
                    "TELEMETRY_AGENTS must contain comma-separated agent IDs".to_string()
                })
            })
            .collect::<Result<_, _>>()?;
    }
    for (name, target) in [
        ("TELEMETRY_FIRST_MONTH", &mut config.first_month),
        ("TELEMETRY_EVERY", &mut config.metric_every),
    ] {
        if let Ok(value) = std::env::var(name) {
            *target = value.parse().map_err(|_| format!("invalid {name}"))?;
        }
    }
    if let Ok(value) = std::env::var("TELEMETRY_LAST_MONTH") {
        config.last_month = Some(value.parse().map_err(|_| "invalid TELEMETRY_LAST_MONTH")?);
    }
    if let Ok(value) = std::env::var("TELEMETRY_LOG_LIMIT") {
        config.log_limit = value.parse().map_err(|_| "invalid TELEMETRY_LOG_LIMIT")?;
    }
    let directory = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(format!("{mode}.jsonl")))
        .map_err(|e| e.to_string())?;
    Observer::new(std::io::BufWriter::new(file), mode, config).map(Some)
}

fn main() -> Result<(), String> {
    let months = std::env::var("MONTHS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MONTHS);
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
        if let Ok(value) = std::env::var("ORDER_HORIZON") {
            let horizon = match value.as_str() {
                "legacy" => economics_compute_smoke::town_market::OrderHorizon::Legacy,
                _ => economics_compute_smoke::town_market::OrderHorizon::Aligned(
                    value
                        .parse()
                        .map_err(|_| "ORDER_HORIZON must be legacy or a positive month count")?,
                ),
            };
            w.town_market.as_mut().unwrap().order_horizon = horizon;
        }
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
        let mut telemetry = observer(mode)?;
        for _ in 0..months {
            let result = if let Some(observer) = &mut telemetry {
                observer.run_months(&mut sim, 1)
            } else {
                sim.run_months(1)
            };
            if let Err(error) = result {
                if let Some(observer) = telemetry.take()
                    && let Err(output_error) = observer.finish()
                {
                    return Err(format!("{error}; {output_error}"));
                }
                return Err(error);
            }
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
        if let Some(observer) = telemetry {
            observer.finish()?;
        }
        for r in &sim.state.town_market.history {
            println!("{mode} month={} books={:?}", r.month, r.markets);
        }
    }
    Ok(())
}
