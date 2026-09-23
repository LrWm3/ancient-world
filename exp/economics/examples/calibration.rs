use economics_compute_smoke::{
    calibration,
    compute::Backend,
    negotiation::GRAIN_MARKET,
    production_market as pm,
    scenario::{FUEL, GRAIN, NUTRITION, TOKEN, WARMTH},
    simulation::Simulation,
    telemetry::{Config, Observer, PlanningDetail},
};
const DEFAULT_MONTHS: u32 = 24;
const PLAN_HOLD_MONTHS: u32 = 3;
fn main() -> Result<(), String> {
    let mode = std::env::var("CASE").unwrap_or_else(|_| "baseline".into());
    let months = std::env::var("MONTHS")
        .map_or(Ok(DEFAULT_MONTHS), |s| s.parse().map_err(|_| "bad MONTHS"))?;
    let (mut w, s) = calibration::scenario(mode != "autarky" && mode != "isolated-specialists");
    match mode.as_str() {
        "baseline" | "autarky" => {}
        "expectations" => {
            w.production_market.as_mut().unwrap().counterparties =
                pm::CounterpartyExpectation::LastPublishedPlan
        }
        "persistent" => {
            w.production_market.as_mut().unwrap().persistence = pm::Persistence::Hold {
                months: PLAN_HOLD_MONTHS,
            }
        }
        "combined" => {
            let c = w.production_market.as_mut().unwrap();
            c.counterparties = pm::CounterpartyExpectation::LastPublishedPlan;
            c.persistence = pm::Persistence::Hold {
                months: PLAN_HOLD_MONTHS,
            };
        }
        "directed" | "isolated-specialists" => calibration::directed(&mut w),
        _ => return Err("unknown CASE".into()),
    }
    let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
    let directory =
        std::env::var("TELEMETRY_DIR").map_err(|_| "set TELEMETRY_DIR under ignored output/")?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(std::path::Path::new(&directory).join(format!("{mode}.jsonl")))
        .map_err(|e| e.to_string())?;
    let mut observer = Observer::new(
        std::io::BufWriter::new(file),
        &mode,
        Config {
            planning: PlanningDetail::Selected,
            settlement: true,
            ..Config::default()
        },
    )?;
    let result = observer.run_months(&mut sim, months);
    observer.finish()?;
    result?;
    let deficits = |r| {
        sim.reports
            .iter()
            .map(|p| i64::from(p.deficit(r)))
            .sum::<i64>()
    };
    let volume = |m| {
        sim.state
            .town_market
            .history
            .iter()
            .filter_map(|r| r.markets.get(&m))
            .map(|m| i64::from(m.volume))
            .sum::<i64>()
    };
    println!(
        "{mode} months={months} food={} warmth={} grain={} wood={}",
        deficits(NUTRITION),
        deficits(WARMTH),
        volume(GRAIN_MARKET),
        volume(pm::WOOD_MARKET)
    );
    for p in &sim.world.participants {
        println!(
            "{} grain={} fuel={} coins={}",
            p.agent,
            sim.state.balance(p.agent, GRAIN),
            sim.state.balance(p.agent, FUEL),
            sim.state.balance(p.agent, TOKEN)
        );
    }
    Ok(())
}
