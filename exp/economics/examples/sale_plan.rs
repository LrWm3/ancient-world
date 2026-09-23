use economics_compute_smoke::{
    compute::Backend,
    scenario::{NUTRITION, PERSON, TOKEN},
    simulation::Simulation,
    stock_sale,
};
fn main() -> Result<(), String> {
    for case in ["funded", "limited", "food-tight"] {
        let (w, s) = stock_sale::forecast_scenario(case)?;
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        sim.run_months(18)?;
        let d = sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .find_map(|b| b.decision.as_ref())
            .unwrap();
        println!(
            "{case} accept={} reason={:?} cash={} deficits={}",
            d.accept,
            d.reason,
            sim.state.balance(PERSON, TOKEN),
            sim.reports
                .iter()
                .filter(|r| r.agent == PERSON)
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>()
        );
        for b in &sim.ledger {
            if let Some(r) = b.credit.as_ref().and_then(|c| c.stock_sale.as_ref()) {
                println!(
                    "month {} sold={} decision={:?}",
                    b.month, r.goods, r.decision
                );
            }
        }
    }
    Ok(())
}
