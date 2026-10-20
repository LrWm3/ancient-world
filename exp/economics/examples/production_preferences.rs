//! Compare explicit monthly completion preferences under unchanged acceptance gates.
use economics_compute_smoke::{
    acceptance::{
        self, Run, WORKSHOP,
        continuation::{self, Case},
        financing::{Liquidity, Preferences, Stocks},
        production_preferences::{self, Priority},
    },
    compute::Backend,
};
const MONTHS: u32 = 48;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: production_preferences <both|metal|ore|neither> <off|production|cash> [baseline|no-ore|low-price]".into());
    }
    let stocks = match args[0].as_str() {
        "both" => Stocks::Both,
        "metal" => Stocks::MetalOnly,
        "ore" => Stocks::OreOnly,
        "neither" => Stocks::Neither,
        _ => return Err("unknown opening stocks".into()),
    };
    let priority = match args[1].as_str() {
        "off" => Priority::Off,
        "production" => Priority::ProductionFirst,
        "cash" => Priority::CashFirst,
        _ => return Err("unknown priority".into()),
    };
    let case = match args.get(2).map(String::as_str).unwrap_or("baseline") {
        "baseline" => Case::Baseline,
        "no-ore" => Case::NoOreSupply,
        "low-price" => Case::LowMetalPrice,
        _ => return Err("unknown control".into()),
    };
    let (w, s) = production_preferences::scenario(
        case,
        stocks,
        Preferences {
            workshop: Liquidity::MonthlyExposure,
            town: Liquidity::TerminalReserve,
        },
        priority,
    )?;
    let mut run = Run::new(w, s, Backend::Reference)?;
    run.through(MONTHS)?;
    println!("SETTINGS stocks={stocks:?} priority={priority:?} case={case:?} months={MONTHS}");
    println!("EVIDENCE {:?}", run.evidence()?);
    println!("READINESS {:?}", acceptance::governed::check(&run)?);
    for from in [13, 25, 37] {
        println!(
            "PERIOD {from}: {:?}",
            continuation::period(&run, from, from + 11)?
        );
    }
    for a in &run.sim.world.agency[&WORKSHOP].startup_forwards {
        println!("FORWARD signed={} status={:?}", a.month, a.status);
    }
    Ok(())
}
