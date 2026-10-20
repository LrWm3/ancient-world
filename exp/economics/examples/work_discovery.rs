//! Bounded request-discovery comparison, with frozen explanations in decisions.
use economics_compute_smoke::{
    acceptance::{Run, TOWN, WORKSHOP, continuation::Case, financing::Stocks, work_discovery},
    agency::work_requests::Policy,
    compute::Backend,
};
const MONTHS: u32 = 24;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "usage: work_discovery <both|metal|ore|neither> <static|observe|respond|bundled>"
                .into(),
        );
    }
    let stocks = match args[0].as_str() {
        "both" => Stocks::Both,
        "metal" => Stocks::MetalOnly,
        "ore" => Stocks::OreOnly,
        "neither" => Stocks::Neither,
        _ => return Err("unknown stocks".into()),
    };
    let policy = match args[1].as_str() {
        "static" => Policy::Static,
        "observe" => Policy::Observe,
        "respond" => Policy::Respond,
        "bundled" => Policy::RespondWithWorkQuotes,
        _ => return Err("unknown policy".into()),
    };
    let (w, s) = work_discovery::scenario(Case::Baseline, stocks, policy)?;
    let mut run = Run::new(w, s, Backend::Reference)?;
    run.through(MONTHS)?;
    println!("SETTINGS {stocks:?} {policy:?} months={MONTHS}");
    println!("EVIDENCE {:?}", run.evidence()?);
    for agent in [WORKSHOP, TOWN] {
        for d in &run.sim.world.agency[&agent].history {
            println!(
                "DISCOVERY agent={agent} month={} chosen={:?} accepted={:?} opportunities={:?} requests={:?} requirements={:?}",
                d.month, d.chosen, d.accepted, d.sale_opportunities, d.work_requests, d.work_routes
            );
        }
    }
    Ok(())
}
