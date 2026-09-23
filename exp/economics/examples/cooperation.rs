use economics_compute_smoke::{
    calibration,
    compute::Backend,
    cooperation::Discovery,
    production_market::Policy,
    scenario::{FUEL, GRAIN, NUTRITION, TOKEN, WARMTH},
    simulation::Simulation,
    telemetry::{Config, Observer, PlanningDetail},
};
const DEFAULT_MONTHS: u32 = 24;
const FAILED_HARVEST_MONTH: u32 = 3;
const LOW_FOOD_GRAIN: i32 = 2;
fn main() -> Result<(), String> {
    let months = std::env::var("MONTHS").map_or(Ok(DEFAULT_MONTHS), |v| {
        v.parse().map_err(|_| "invalid MONTHS")
    })?;
    let directory =
        std::env::var("TELEMETRY_DIR").map_err(|_| "set TELEMETRY_DIR under ignored output/")?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let shock = std::env::var("SHOCK").unwrap_or_else(|_| "none".into());
    if !["none", "harvest"].contains(&shock.as_str()) {
        return Err("unknown SHOCK".into());
    }
    let variant = std::env::var("VARIANT").unwrap_or_else(|_| "normal".into());
    if !["normal", "low-food"].contains(&variant.as_str()) {
        return Err("unknown VARIANT".into());
    }
    for mode in [Discovery::Mutual, Discovery::Posted] {
        let (mut w, mut s) = calibration::scenario(true);
        if variant == "low-food" {
            s.balances
                .insert((calibration::WOOD_PERSON, GRAIN), LOW_FOOD_GRAIN);
        }
        w.production_market.as_mut().unwrap().policy = Policy::Cooperate(mode);
        if shock == "harvest" {
            w.capacity_overrides
                .insert((FAILED_HARVEST_MONTH, calibration::CROP_PERSON), 0);
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        let name = format!("{mode:?}-{variant}-{shock}");
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::Path::new(&directory).join(format!("{name}.jsonl")))
            .map_err(|e| e.to_string())?;
        let mut observer = Observer::new(
            std::io::BufWriter::new(file),
            name,
            Config {
                settlement: true,
                planning: PlanningDetail::Selected,
                ..Config::default()
            },
        )?;
        for _ in 0..months {
            observer.run_months(&mut sim, 1)?;
            let r = sim.state.town_market.history.last().unwrap();
            let c = r.cooperation.as_ref().unwrap();
            println!(
                "{mode:?} month={} event={} offers={} projections={} deliveries={} failure={:?} choices={:?}",
                r.month,
                c.event,
                c.offers.len(),
                c.projections,
                c.completed.len(),
                c.failure,
                c.active.as_ref().map(|c| &c.choices)
            );
        }
        observer.finish()?;
        for p in &sim.world.participants {
            let deficits = |resource| {
                sim.reports
                    .iter()
                    .filter(|r| r.agent == p.agent)
                    .map(|r| r.deficit(resource))
                    .sum::<i32>()
            };
            println!(
                "{mode:?} agent={} food={} warmth={} grain={} fuel={} coins={}",
                p.agent,
                deficits(NUTRITION),
                deficits(WARMTH),
                sim.state.balance(p.agent, GRAIN),
                sim.state.balance(p.agent, FUEL),
                sim.state.balance(p.agent, TOKEN)
            );
        }
    }
    Ok(())
}
