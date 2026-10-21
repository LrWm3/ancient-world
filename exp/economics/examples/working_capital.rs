use economics_compute_smoke::{
    acceptance::{
        Run,
        continuation::{self, Case},
        financing::Stocks,
        working_capital,
    },
    compute::Backend,
    minting::ISSUER,
};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let months = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map(|s| s.parse::<u32>())
        .transpose()
        .map_err(|e| e.to_string())?
        .unwrap_or(continuation::MONTHS);
    let enabled = !args.iter().any(|a| a == "--without-finance");
    let backend = if args.iter().any(|a| a == "--cpu") {
        Backend::CubeCpu
    } else {
        Backend::Reference
    };
    let (w, s) = working_capital::scenario(Case::Baseline, Stocks::Neither, enabled)?;
    let mut run = Run::new(w, s, backend)?;
    run.through(months)?;
    println!("EVIDENCE {:?}", run.evidence()?);
    println!(
        "LOANS {:?}",
        run.sim.world.agency[&ISSUER].financed_programs
    );
    if args.iter().any(|a| a == "--details") {
        for a in &run.sim.world.agency[&ISSUER].financing_reviews {
            println!("REVIEW {a:?}");
        }
    }
    if months == continuation::MONTHS {
        let readiness = continuation::check(&run)?;
        println!("READINESS {readiness:?}");
        if !readiness.ready() {
            return Err("working-capital economy is not ready".into());
        }
    }
    Ok(())
}
