//! One negotiated spot trade, with explicit opening inventory cost.
use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::Audit,
    negotiation,
    scenario::{GRAIN, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
const OPENING_GRAIN_COST: i128 = 14;
fn main() -> Result<(), String> {
    let (world, state) = negotiation::scenario();
    let terms = world.negotiation.as_ref().ok_or("missing market")?;
    let (seller, buyer) = (terms.seller.agent, terms.buyer.agent);
    let mut audit = Audit::with_inventory(
        &world,
        &state,
        TOKEN,
        BTreeMap::new(),
        BTreeMap::from([((seller, GRAIN), OPENING_GRAIN_COST)]),
    )?;
    let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
    while sim.state.month == 1 {
        audit.step(&mut sim)?;
    }
    for agent in [seller, buyer] {
        println!("{}", audit.book().statements(agent, 1, 1)?.markdown(TOKEN));
    }
    Ok(())
}
