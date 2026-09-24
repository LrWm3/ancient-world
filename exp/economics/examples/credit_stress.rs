use economics_compute_smoke::{
    compute::Backend,
    credit,
    credit_stress::{self, Case, OBSERVATION_MONTHS},
    financial_reporting::Audit,
    model::ResourceKind,
    scenario::{GRAIN, GROW, NUTRITION, PERSON, PLOT, SEED, STATE_AGENT, TOKEN},
    simulation::Simulation,
    telemetry::{Config, Observer},
};

use std::collections::BTreeMap;
const OPENING_LAND_COST: i128 = 10_000;
const OPENING_STOCK_UNIT_COST: i128 = 1;

fn main() -> Result<(), String> {
    println!(
        "Journal reports use explicit opening land cost 10000 and stock unit cost 1, with equal grain/seed cost shares."
    );
    let directory =
        std::env::var("TELEMETRY_DIR").map_err(|_| "set TELEMETRY_DIR under ignored output/")?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    for case in [Case::Normal, Case::Temporary, Case::Persistent] {
        let name = format!("{case:?}");
        let (w, s) = credit_stress::scenario(case)?;
        let costs = s
            .balances
            .iter()
            .filter(|((_, r), q)| {
                *r != TOKEN
                    && **q > 0
                    && w.resources
                        .iter()
                        .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
            })
            .map(|(k, q)| (*k, i128::from(*q) * OPENING_STOCK_UNIT_COST))
            .collect();
        let mut audit = Audit::with_processes(
            &w,
            &s,
            TOKEN,
            BTreeMap::from([(PLOT, OPENING_LAND_COST)]),
            costs,
            BTreeMap::from([(GROW, BTreeMap::from([(GRAIN, 1), (SEED, 1)]))]),
        )?;
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(std::path::Path::new(&directory).join(format!("{name}.jsonl")))
            .map_err(|e| e.to_string())?;
        let mut observer = Observer::new(
            std::io::BufWriter::new(file),
            &name,
            Config {
                settlement: true,
                ..Config::default()
            },
        )?;
        for month in 1..=OBSERVATION_MONTHS {
            while sim.state.month <= month {
                observer.step_audited(&mut sim, &mut audit)?;
            }
            audit.finalize_through(month)?;
            let borrower = audit.book().finalized_statements(PERSON, month, month)?;
            let lender = audit
                .book()
                .finalized_statements(STATE_AGENT, month, month)?;
            let loan = &sim.state.credit.loans[&1];
            let borrower_cash = sim.state.balance(PERSON, TOKEN);
            let lender_cash = sim.state.balance(STATE_AGENT, TOKEN);
            println!(
                "{name} month={month} owner={:?} status={:?} pledged={} debt={} arrears={:?} grain={} borrower_cash={borrower_cash} lender_cash={lender_cash} borrower_equity={} lender_equity={}",
                credit::owner(&sim.world, &sim.state, PLOT),
                loan.status,
                loan.collateral.as_ref().is_some_and(|c| c.pledged),
                loan.debt()?,
                loan.first_unpaid,
                sim.state.balance(PERSON, GRAIN),
                borrower.equity,
                lender.equity
            );
            for b in sim.ledger.iter().filter(|b| b.month == month) {
                if let Some(c) = &b.credit {
                    for e in &c.events {
                        println!("{name} {month} {:?} {e:?}", b.phase);
                    }
                }
            }
        }
        observer.finish()?;
        let deficit: i32 = sim
            .reports
            .iter()
            .filter(|r| r.agent == PERSON)
            .map(|r| r.deficit(NUTRITION))
            .sum();
        println!(
            "{name} deficit={deficit} stock_spent={} crops={:?}",
            sim.state.credit.stock_spent, sim.state.processes
        );
    }
    Ok(())
}
