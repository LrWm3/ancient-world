use economics_compute_smoke::{
    compute::Backend,
    need_orders,
    negotiation::QuotePolicy,
    scenario::{NUTRITION, PERSON, TOKEN},
    simulation::Simulation,
    zip,
};
const RUN_MONTHS: u32 = 6;
const SELLER: u32 = 89;
fn main() -> Result<(), String> {
    for learning in [false, true] {
        let (mut w, s) = need_orders::scenario();
        if learning {
            let n = w.negotiation.as_mut().unwrap();
            n.buyer.policy = QuotePolicy::Zip(zip::Config::default());
            n.seller.policy = QuotePolicy::Zip(zip::Config::default());
            n.max_rounds = economics_compute_smoke::negotiation::MAX_QUOTE_ROUNDS;
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        sim.run_months(RUN_MONTHS)?;
        println!("\nPolicy: {}", if learning { "ZIP" } else { "concessions" });
        println!("| Month | Buy order | Sell order | Outcome | Buyer deficit | Seller deficit |");
        println!("| --- | --- | --- | --- | --- | --- |");
        for r in sim.ledger.iter().filter_map(|b| b.negotiation.as_ref()) {
            let o = r.orders.as_ref().ok_or("missing generated orders")?;
            let deficit = |agent| {
                sim.reports
                    .iter()
                    .find(|p| p.agent == agent && p.month == r.month)
                    .unwrap()
                    .deficit(NUTRITION)
            };
            println!(
                "| {} | {} | {} | {:?} | {} | {} |",
                r.month,
                o.buy.is_some(),
                o.sell.is_some(),
                r.outcome,
                deficit(PERSON),
                deficit(SELLER)
            );
        }
        println!(
            "Closing coins: buyer {}, seller {}",
            sim.state.balance(PERSON, TOKEN),
            sim.state.balance(SELLER, TOKEN)
        );
    }
    Ok(())
}
