use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Event},
    resale,
    scenario::{PERSON, PLOT, TOKEN},
    simulation::Simulation,
};
const MONTHS: u32 = 6;
fn main() -> Result<(), String> {
    println!("Amounts are coin ticks (100 per coin).");
    println!("| Case | Status | Sale price | Surplus | Debt | Owner | Borrower coins |");
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    for case in ["funded", "deficiency", "insufficient", "no-buyer"] {
        let (w, s) = resale::scenario(case)?;
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        sim.run_months(MONTHS)?;
        let sale = sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|c| &c.events)
            .find_map(|e| {
                if let Event::Resold { price, surplus, .. } = e {
                    Some((*price, *surplus))
                } else {
                    None
                }
            });
        let l = &sim.state.credit.loans[&1];
        println!(
            "| {case} | {:?} | {:?} | {:?} | {} | {} | {} |",
            l.status,
            sale.map(|s| s.0),
            sale.map(|s| s.1),
            l.debt()?,
            credit::owner(&sim.world, &sim.state, PLOT).unwrap(),
            sim.state.balance(PERSON, TOKEN)
        );
    }
    Ok(())
}
