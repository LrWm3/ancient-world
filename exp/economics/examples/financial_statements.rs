//! Mortgage report with explicit opening land cost. Generated output stays local.
use economics_compute_smoke::{
    compute::Backend,
    credit,
    financial_reporting::Audit,
    scenario::{PERSON, PLOT, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
const MONTHS: u32 = 5;
const OPENING_LAND_COST: i128 = 10_000;
fn main() -> Result<(), String> {
    let case = std::env::args().nth(1).unwrap_or_else(|| "default".into());
    let (world, state) = credit::scenario(&case)?;
    let mut audit = Audit::with_assets(
        &world,
        &state,
        TOKEN,
        BTreeMap::from([(PLOT, OPENING_LAND_COST)]),
    )?;
    let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
    while sim.state.month <= MONTHS {
        audit.step(&mut sim)?;
    }
    for agent in [PERSON, STATE_AGENT] {
        println!(
            "{}",
            audit.book().statements(agent, 1, MONTHS)?.markdown(TOKEN)
        );
    }
    println!(
        "## Journal and noncash movements\n\nDebit-positive reporting ticks; each batch balances within each agent.\n"
    );
    for e in audit.book().entries() {
        println!(
            "- Month {}, {} (batch {:?}): {}",
            e.month, e.id, e.batch, e.description
        );
        for l in &e.lines {
            println!(
                "  - Agent {}: {:?} {:+}; cash flow {:?}",
                l.agent, l.account, l.debit, l.flow
            );
        }
    }
    Ok(())
}
