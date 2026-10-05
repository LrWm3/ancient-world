//! CPU demonstration of the same bounded organization policy across four roles.
use economics_compute_smoke::{agency, compute::Backend, model::*, simulation::Simulation};

const MONTHS: u32 = 8;

fn main() -> Result<(), String> {
    for (name, (world, state)) in [
        ("farming law", agency::scenario::farming()?),
        ("coin reserve", agency::scenario::minting()?),
        ("paid public work", agency::scenario::public_work()?),
        ("household priorities", agency::scenario::household()?),
    ] {
        let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
        sim.run_months(MONTHS)?;
        println!("{name}: completed {MONTHS} months");
        for (agent, controller) in &sim.world.agency {
            for d in &controller.history {
                if let Some(program) = &d.accepted {
                    println!(
                        "  agent={agent} decided={} effective={} governor={:?} program={}",
                        d.month, d.effective_month, d.authorized_by, program.name
                    );
                }
            }
            println!(
                "  agent={agent} closing_losses={:?}",
                agency::objectives::measure(
                    &sim.world,
                    &sim.state,
                    &[],
                    *agent,
                    &controller.config.objectives
                )?
            );
        }
        println!(
            "  accepted_land={} completed_processes={} deaths={}",
            sim.state.accepted_agreements.len(),
            sim.state
                .processes
                .values()
                .filter(|p| p.status == Status::Completed)
                .count(),
            sim.state.terminal.len()
        );
    }
    Ok(())
}
