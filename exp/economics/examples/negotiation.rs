use economics_compute_smoke::{
    compute::Backend,
    negotiation::{self, QuotePolicy},
    scenario::{GRAIN, TOKEN},
    simulation::Simulation,
};

fn main() -> Result<(), String> {
    let (catalog, _) = negotiation::scenario();
    for venue in &catalog.marketplaces {
        println!(
            "Marketplace {} requires agent type {} (person).",
            venue.agent, venue.required_type
        );
        for market in &venue.markets {
            let name = |id| &catalog.resources.iter().find(|r| r.id == id).unwrap().name;
            println!(
                "Market {} facilitates {} {} for {}; price tick {} per lot.\n",
                market.id,
                market.goods.quantity,
                name(market.goods.resource),
                name(market.payment),
                market.price_tick
            );
        }
    }
    println!(
        "| Case | Quotes (bid/ask, payment ticks per lot) | Outcome | Buyer grain | Buyer coins | Seller grain | Seller coins |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    for case in [
        "concessions",
        "fixed quotes",
        "nonoverlapping limits",
        "round limit",
        "unfunded buyer",
        "full storage",
    ] {
        let (mut world, mut state) = negotiation::scenario();
        let s = world.negotiation.as_mut().unwrap();
        match case {
            "fixed quotes" => {
                s.buyer.policy = QuotePolicy::Fixed;
                s.seller.policy = QuotePolicy::Fixed;
            }
            "nonoverlapping limits" => s.buyer.limit = 25,
            "round limit" => s.max_rounds = 2,
            "unfunded buyer" => {
                state.balances.insert((88, TOKEN), 39);
            }
            "full storage" => {
                world.storage.capacities.insert(88, 0);
            }
            _ => {}
        }
        let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
        sim.run_months(1)?;
        let r = sim
            .ledger
            .iter()
            .find_map(|b| b.negotiation.as_ref())
            .ok_or("missing receipt")?;
        let quotes = r
            .quotes
            .iter()
            .map(|q| format!("{}/{}", q.bid, q.ask))
            .collect::<Vec<_>>()
            .join(" → ");
        println!(
            "| {case} | {quotes} | {:?} | {} | {} | {} | {} |",
            r.outcome,
            sim.state.balance(88, GRAIN),
            sim.state.balance(88, TOKEN),
            sim.state.balance(89, GRAIN),
            sim.state.balance(89, TOKEN)
        );
    }
    Ok(())
}
