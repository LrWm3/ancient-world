use economics_compute_smoke::{
    compute::Backend,
    minting::{self, *},
    simulation::Simulation,
    telemetry::{Config, Observer},
};
const RUN_MONTHS: u32 = 3;
const REPEATED_MONTHS: u32 = 6;
fn main() -> Result<(), String> {
    let directory =
        std::env::var("TELEMETRY_DIR").map_err(|_| "set TELEMETRY_DIR under ignored output/")?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let generated = std::env::var("MINT_ORDERS").as_deref() == Ok("true");
    let repeated = std::env::var("MINT_CYCLES").as_deref() == Ok("true");
    let cases = if repeated {
        ["normal", "ore", "labor", "low_yield"]
    } else {
        ["normal", "treasury", "metal", "labor"]
    };
    for case in cases {
        let (w, s) = if repeated {
            minting::repeated_scenario(case)?
        } else if generated {
            minting::order_scenario(case)?
        } else {
            minting::scenario(case)?
        };
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::Path::new(&directory).join(format!("{case}.jsonl")))
            .map_err(|e| e.to_string())?;
        let mut observer = Observer::new(
            std::io::BufWriter::new(file),
            case,
            Config {
                settlement: true,
                ..Config::default()
            },
        )?;
        observer.run_months(
            &mut sim,
            if repeated {
                REPEATED_MONTHS
            } else {
                RUN_MONTHS
            },
        )?;
        observer.finish()?;
        let supply: i32 = sim
            .state
            .balances
            .iter()
            .filter(|((_, r), _)| *r == COIN)
            .map(|(_, v)| v)
            .sum();
        println!(
            "{case}: supply={supply}, coins state/supplier/worker={}/{}/{}, worker firewood={}",
            sim.state.balance(ISSUER, COIN),
            sim.state.balance(SUPPLIER, COIN),
            sim.state.balance(WORKER, COIN),
            sim.state.balance(WORKER, FIREWOOD)
        );
        for b in &sim.ledger {
            if let Some(r) = &b.minting {
                if let Some(plan) = &r.plan {
                    println!(
                        "month={} funding={} reason={} orders={:?}",
                        r.month, plan.required_funding, plan.reason, plan.orders
                    );
                }
                for p in &r.receipts {
                    println!(
                        "month={} package={} accepted={} reason={:?}",
                        r.month, p.package, p.accepted, p.reason
                    );
                }
            }
        }
    }
    Ok(())
}
