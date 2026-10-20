//! Dated startup diagnostics followed by unchanged 48-month acceptance checks.
use economics_compute_smoke::{
    acceptance::{self, Run, TOWN, WORKSHOP, continuation, financing::Stocks, startup_publication},
    compute::Backend,
    minting::{COIN, ISSUER, METAL, ORE},
    model::Phase,
};
const MONTHS: u32 = 48;
const DETAIL_MONTHS: u32 = 8;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: startup_publication <both|metal|ore|neither> <off|on>".into());
    }
    let stocks = match args[0].as_str() {
        "both" => Stocks::Both,
        "metal" => Stocks::MetalOnly,
        "ore" => Stocks::OreOnly,
        "neither" => Stocks::Neither,
        _ => return Err("unknown stocks".into()),
    };
    let enabled = match args[1].as_str() {
        "on" => true,
        "off" => false,
        _ => return Err("unknown publication policy".into()),
    };
    let (w, s) = startup_publication::scenario(continuation::Case::Baseline, stocks, enabled)?;
    let mut run = Run::new(w, s, Backend::Reference)?;
    println!("SETTINGS stocks={stocks:?} productive_publication={enabled} months={MONTHS}");
    while run.sim.state.month <= MONTHS {
        let month = run.sim.state.month;
        let phase = run.sim.state.phase;
        run.audit.step(&mut run.sim)?;
        if month <= DETAIL_MONTHS {
            println!(
                "BOUNDARY month={month} phase={phase:?} closing_balances={:?}",
                [ISSUER, WORKSHOP, TOWN]
                    .map(|a| (a, [COIN, ORE, METAL].map(|r| run.sim.state.balance(a, r))))
            );
            if phase == Phase::Open {
                for a in [ISSUER, WORKSHOP, TOWN] {
                    if let Some(d) = run.sim.world.agency[&a]
                        .history
                        .last()
                        .filter(|d| d.month == month)
                    {
                        println!(
                            "DECISION agent={a} effective={} chosen={:?} reason={} accepted={:?}",
                            d.effective_month, d.chosen, d.reason, d.accepted
                        );
                    }
                }
            }
            if phase == Phase::Acquire {
                println!(
                    "PROCUREMENT month={month} {:?}",
                    run.sim.ledger.last().unwrap().minting
                );
            }
        }
    }
    println!("EVIDENCE {:?}", run.evidence()?);
    println!("READINESS {:?}", acceptance::governed::check(&run)?);
    for from in [13, 25, 37] {
        println!(
            "PERIOD {from}: {:?}",
            continuation::period(&run, from, from + 11)?
        );
    }
    Ok(())
}
