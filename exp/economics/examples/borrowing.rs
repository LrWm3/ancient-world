use economics_compute_smoke::{borrowing, compute::Backend, simulation::Simulation};
fn main() -> Result<(), String> {
    for case in ["affordable", "unaffordable", "unhelpful"] {
        let (w, s) = borrowing::scenario(case)?;
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        sim.run_months(12)?;
        let d = sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .find_map(|b| b.decision.as_ref())
            .ok_or("missing decision")?;
        println!(
            "{case}: accept={} reason={:?}\ndecline={:?}\npurchase={:?}",
            d.accept, d.reason, d.decline, d.purchase
        );
    }
    Ok(())
}
