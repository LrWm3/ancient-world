//! Run the mixed-organization acceptance fixture on CubeCL CPU.
use economics_compute_smoke::{acceptance, compute::Backend};
fn main() -> Result<(), String> {
    let (w, s) = acceptance::scenario()?;
    let mut run = acceptance::Run::new(w, s, Backend::CubeCpu)?;
    run.through(acceptance::RUN_MONTHS)?;
    println!("{:#?}", run.evidence()?);
    Ok(())
}
