//! CPU financial statements under an explicit non-redeemable currency convention.
use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::Audit,
    issuance_accounting::Policy,
    minting::{self, COIN, ISSUER, SUPPLIER, WORKER},
    model::ResourceKind,
    simulation::Simulation,
};
use std::collections::BTreeMap;

fn main() -> Result<(), String> {
    let case = std::env::args().nth(1).unwrap_or_else(|| "normal".into());
    let (world, state) = minting::scenario(&case)?;
    let costs = state
        .balances
        .iter()
        .filter(|((_, r), q)| {
            *r != COIN
                && **q > 0
                && world
                    .resources
                    .iter()
                    .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
        })
        .map(|(k, q)| (*k, i128::from(*q)))
        .collect();
    let mut audit = Audit::with_processes(
        &world,
        &state,
        COIN,
        BTreeMap::new(),
        costs,
        BTreeMap::new(),
    )?
    .with_issuance_policy(Policy::NonRedeemableEquity)?;
    let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
    while sim.state.month <= 2 {
        audit.step(&mut sim)?;
    }
    println!(
        "# Physical minting: {case}\n\nNon-redeemable issuance adds equity; delivered monthly services are expensed. Opening stock cost: one coin tick per unit.\n"
    );
    for agent in [ISSUER, SUPPLIER, WORKER] {
        println!("{}", audit.book().statements(agent, 1, 2)?.markdown(COIN));
    }
    Ok(())
}
