//! Compare declared input-stock preferences with fixed workshop/town liquidity goals.
use economics_compute_smoke::{
    acceptance::{
        self, Run, WORKSHOP,
        buffers::{self, Policy},
        continuation::{self, Case},
        financing::{Liquidity, Preferences, Stocks},
    },
    compute::Backend,
};
const MONTHS: u32 = 48;
const DIAGNOSTIC_MONTH: u32 = 12;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: input_buffers <both|metal|ore|neither> <off|inputs|cash> [baseline|no-ore|no-access|low-price|late-payroll]".into());
    }
    let stocks = match args[0].as_str() {
        "both" => Stocks::Both,
        "metal" => Stocks::MetalOnly,
        "ore" => Stocks::OreOnly,
        "neither" => Stocks::Neither,
        _ => return Err("unknown stocks".into()),
    };
    let policy = match args[1].as_str() {
        "off" => Policy::Off,
        "inputs" => Policy::InputsFirst,
        "cash" => Policy::CashFirst,
        _ => return Err("unknown buffer priority".into()),
    };
    let case = match args.get(2).map(String::as_str).unwrap_or("baseline") {
        "baseline" => Case::Baseline,
        "no-ore" => Case::NoOreSupply,
        "no-access" => Case::NoMetalAccess,
        "low-price" => Case::LowMetalPrice,
        "late-payroll" => Case::LatePayroll,
        _ => return Err("unknown control".into()),
    };
    let preferences = Preferences {
        workshop: Liquidity::MonthlyExposure,
        town: Liquidity::TerminalReserve,
    };
    let (w, s) = buffers::scenario(case, true, stocks, preferences, policy)?;
    let mut run = Run::new(w, s, Backend::Reference)?;
    run.through(MONTHS)?;
    println!(
        "SETTINGS stocks={stocks:?} policy={policy:?} case={case:?} months={MONTHS} preferences={preferences:?}"
    );
    println!("EVIDENCE {:?}", run.evidence()?);
    println!("READINESS {:?}", acceptance::governed::check(&run)?);
    for from in [13, 25, 37] {
        println!(
            "PERIOD {from}: {:?}",
            continuation::period(&run, from, from + 11)?
        );
    }
    let c = &run.sim.world.agency[&WORKSHOP];
    for a in &c.startup_forwards {
        println!(
            "FORWARD signed={} due={} status={:?}",
            a.month, a.proposal.terms.due, a.status
        );
    }
    if let Some(d) = c.history.iter().find(|d| d.month == DIAGNOSTIC_MONTH) {
        println!(
            "DECISION month={} effective={} chosen={:?}",
            d.month, d.effective_month, d.chosen
        );
        for a in &d.alternatives {
            println!(
                "ALTERNATIVE program={:?} losses={:?} failure={:?}",
                a.program, a.losses, a.failure
            );
            if let Some(p) = &a.projection {
                println!(
                    "  completions={:?} sales={:?} stocks={:?}",
                    p.completions, p.sales, p.closing_stocks
                );
            }
        }
    }
    Ok(())
}
