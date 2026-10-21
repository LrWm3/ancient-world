use economics_compute_smoke::{
    acceptance::{self, continuation, voluntary_employment},
    compute::Backend,
};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let months = args
        .iter()
        .find_map(|v| v.parse::<u32>().ok())
        .unwrap_or(48);
    let details = args.iter().any(|v| v == "--details");
    let backend = if args.iter().any(|v| v == "--cpu") {
        Backend::CubeCpu
    } else {
        Backend::Reference
    };
    let (w, s) = voluntary_employment::with_startup_food(
        !args.iter().any(|v| v == "--original-allocation"),
    )?;
    let mut run = acceptance::Run::new(w, s, backend)?;
    for month in 1..=months {
        run.through(month)?;
        println!("Month {month}: {:?}", run.evidence()?);
    }
    if details {
        for b in &run.sim.ledger {
            if let Some(e) = &b.employment {
                for d in &e.recruitment {
                    println!("{d:?}");
                }
            }
        }
    }
    if months == 48 {
        let readiness = continuation::check(&run)?;
        println!("{readiness:?}");
        if !readiness.ready() {
            return Err("voluntary employment acceptance failed".into());
        }
    }
    Ok(())
}
