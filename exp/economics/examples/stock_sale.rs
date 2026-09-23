use economics_compute_smoke::{compute::Backend, simulation::Simulation, stock_sale};
fn main() -> Result<(), String> {
    for case in ["funded", "limited", "food-tight", "unprotected"] {
        let (mut w, s) = stock_sale::scenario(if case == "unprotected" {
            "food-tight"
        } else {
            case
        })?;
        if case == "unprotected" {
            w.credit
                .as_mut()
                .unwrap()
                .stock_sales
                .as_mut()
                .unwrap()
                .reserve_months = 0;
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        sim.run_months(18)?;
        let d = sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .find_map(|b| b.decision.as_ref())
            .ok_or("missing decision")?;
        println!("{case}: accept={} reason={:?}", d.accept, d.reason);
        println!("decline={:?}\npurchase={:?}", d.decline, d.purchase);
        for b in &sim.ledger {
            if let Some(sale) = b.credit.as_ref().and_then(|b| b.stock_sale.as_ref())
                && sale.goods > 0
            {
                println!("month {} sale: {:?}", b.month, sale);
            }
        }
    }
    Ok(())
}
