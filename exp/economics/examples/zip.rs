use economics_compute_smoke::{
    compute::Backend,
    negotiation::{self, Outcome, QuotePolicy},
    scenario::{GRAIN, TOKEN},
    simulation::Simulation,
    zip,
};

const MONTHS: u32 = 12;
const TOTAL_GRAIN: i32 = 24;
const TOTAL_COINS: i32 = 600;
const SHOCK_MONTH: u32 = 7;
const SHOCK_BUYER_LIMIT: i32 = 80;
const SHOCK_SELLER_LIMIT: i32 = 50;
const INCOMPATIBLE_BUYER_LIMIT: i32 = 25;
const CONCESSION_TICKS: i32 = 5;

fn main() -> Result<(), String> {
    println!(
        "| Scenario | Policy | Trades / 12 | Quote rounds | Prices | Buyer surplus | Seller surplus |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    for case in ["overlap", "limits shift", "no overlap"] {
        for (name, policy) in [
            ("fixed", QuotePolicy::Fixed),
            (
                "concede",
                QuotePolicy::Concede {
                    ticks: CONCESSION_TICKS,
                },
            ),
            ("ZIP seed 7", QuotePolicy::Zip(zip::Config::default())),
            (
                "ZIP seed 19",
                QuotePolicy::Zip(zip::Config {
                    seed: 19,
                    ..Default::default()
                }),
            ),
        ] {
            let (mut w, mut s) = negotiation::scenario();
            w.storage.capacities.insert(88, TOTAL_GRAIN);
            s.balances.insert((89, GRAIN), TOTAL_GRAIN);
            s.balances.insert((88, TOKEN), TOTAL_COINS);
            let session = w.negotiation.as_mut().unwrap();
            session.max_rounds = negotiation::MAX_QUOTE_ROUNDS;
            session.buyer.policy = policy;
            session.seller.policy = policy;
            if case == "no overlap" {
                session.buyer.limit = INCOMPATIBLE_BUYER_LIMIT;
            }
            let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
            let mut prices = vec![];
            let mut rounds = 0;
            let mut buyer_surplus = 0;
            let mut seller_surplus = 0;
            for month in 1..=MONTHS {
                let session = sim.world.negotiation.as_mut().unwrap();
                session.month = month;
                if case == "limits shift" && month == SHOCK_MONTH {
                    session.buyer.limit = SHOCK_BUYER_LIMIT;
                    session.seller.limit = SHOCK_SELLER_LIMIT;
                }
                let (max, min) = (session.buyer.limit, session.seller.limit);
                sim.run_months(1)?;
                let r = sim.state.marketplaces[&negotiation::MARKETPLACE]
                    .history
                    .last()
                    .unwrap();
                rounds += r.quotes.len();
                if let Outcome::Traded { price } = r.outcome {
                    prices.push(price);
                    buyer_surplus += max - price;
                    seller_surplus += price - min;
                }
            }
            println!(
                "| {case} | {name} | {} | {rounds} | {} | {buyer_surplus} | {seller_surplus} |",
                prices.len(),
                prices
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    Ok(())
}
