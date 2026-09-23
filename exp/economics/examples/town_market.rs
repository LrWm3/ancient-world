use economics_compute_smoke::negotiation::GRAIN_MARKET;
use economics_compute_smoke::{
    compute::Backend, scenario::NUTRITION, simulation::Simulation, town_market,
};
fn main() -> Result<(), String> {
    let (w, s) = town_market::scenario();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
    sim.run_months(6)?;
    println!(
        "| Month | Price per lot | Grain traded | Unfilled buy | Unfilled sell | Total food deficit |"
    );
    println!("| --- | --- | --- | --- | --- | --- |");
    for r in &sim.state.town_market.history {
        let deficit: i32 = sim
            .reports
            .iter()
            .filter(|p| p.month == r.month)
            .map(|p| p.deficit(NUTRITION))
            .sum();
        println!(
            "| {} | {:?} | {} | {} | {} | {} |",
            r.month,
            r.markets[&GRAIN_MARKET].posted_price,
            r.markets[&GRAIN_MARKET].volume,
            r.markets[&GRAIN_MARKET].unfilled_buy,
            r.markets[&GRAIN_MARKET].unfilled_sell,
            deficit
        );
    }
    Ok(())
}
