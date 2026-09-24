//! CPU material-cost statements; no prices or labor wages are imputed.
use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::Audit,
    model::{Resource, ResourceKind},
    scenario::{self, GRAIN, PERSON, PLOT, SEED, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
const MONTHS: u32 = 6;
const GRAIN_COST: i128 = 10;
const SEED_COST: i128 = 12;
const STORAGE_LIMIT: i32 = 6;
fn main() -> Result<(), String> {
    let case = std::env::args().nth(1).unwrap_or_else(|| "harvest".into());
    let (mut world, state) = scenario::baseline();
    world.resources.push(Resource {
        id: TOKEN,
        name: "reporting coin".into(),
        kind: ResourceKind::Stock,
    });
    match case.as_str() {
        "harvest" => {}
        "missed-work" => {
            world.capacity_overrides.insert((2, PERSON), 0);
        }
        "storage-blocked" => {
            world.storage.weights = BTreeMap::from([(GRAIN, 1), (SEED, 1)]);
            world.storage.capacities.insert(PERSON, STORAGE_LIMIT);
        }
        _ => return Err("choose harvest, missed-work or storage-blocked".into()),
    }
    let mut audit = Audit::with_processes(
        &world,
        &state,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        BTreeMap::from([((PERSON, GRAIN), GRAIN_COST), ((PERSON, SEED), SEED_COST)]),
        BTreeMap::new(),
    )?;
    let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
    while sim.state.month <= MONTHS {
        audit.step(&mut sim)?;
    }
    println!(
        "{}",
        audit.book().statements(PERSON, 1, MONTHS)?.markdown(TOKEN)
    );
    Ok(())
}
