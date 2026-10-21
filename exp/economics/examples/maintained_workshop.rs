use economics_compute_smoke::{
    acceptance::{
        continuation,
        maintained_workshop::{self as m, Case},
    },
    compute::Backend,
};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let months = args
        .iter()
        .find_map(|v| v.parse::<u32>().ok())
        .unwrap_or(48);
    let case = if args.iter().any(|v| v == "--supplied") {
        Case::Supplied
    } else if args.iter().any(|v| v == "--no-authority") {
        Case::NoAuthority
    } else if args.iter().any(|v| v == "--no-repair") {
        Case::NoRepair
    } else if args.iter().any(|v| v == "--no-condition-goal") {
        Case::NoConditionGoal
    } else {
        Case::Discovered
    };
    let backend = if args.iter().any(|v| v == "--cpu") {
        Backend::CubeCpu
    } else {
        Backend::Reference
    };
    let mut run = m::run(case, backend)?;
    for month in 1..=months {
        run.through(month)?;
        if args.iter().any(|v| v == "--details") {
            use economics_compute_smoke::{
                acceptance::WORKSHOP,
                minting::{COIN, ORE},
            };
            println!(
                "stocks ore={} cash={} orders={:?}",
                run.sim.state.balance(WORKSHOP, ORE),
                run.sim.state.balance(WORKSHOP, COIN),
                run.sim
                    .world
                    .activities
                    .orders
                    .iter()
                    .filter(|o| o.agent == WORKSHOP)
                    .collect::<Vec<_>>()
            );
            if let Some(d) = run.sim.world.agency[&WORKSHOP].history.last() {
                println!(
                    "governance month={} accepted={:?} reason={}",
                    d.month, d.accepted, d.reason
                );
                for a in &d.alternatives {
                    println!(
                        "alternative program={:?} losses={:?} failure={:?}",
                        a.program, a.losses, a.failure
                    );
                }
            }
            for (&actor, c) in &run.sim.world.agency {
                for a in c.forward_reviews.iter().filter(|a| a.month == month) {
                    if let Ok(result) = &a.assessment
                        && result
                            .comparisons
                            .values()
                            .all(|c| c.proposed <= c.baseline)
                    {
                        println!(
                            "forward accepted={} owner={actor} proposal={:?}",
                            result.mutually_beneficial(),
                            a.proposal
                        );
                        for (party, comparison) in &result.comparisons {
                            println!(
                                "forward party={party} baseline={:?} proposed={:?} self_funded={:?}",
                                comparison.baseline,
                                comparison.proposed,
                                comparison
                                    .self_funded
                                    .as_ref()
                                    .map(|o| (o.month, &o.losses))
                            );
                        }
                    }
                }
            }
            for b in run.sim.ledger.iter().filter(|b| b.month == month) {
                if let Some(e) = &b.employment {
                    for d in &e.recruitment {
                        println!("hiring {d:?}");
                    }
                }
            }
        }
        let repairs = run
            .sim
            .state
            .processes
            .values()
            .filter(|p| {
                p.definition == m::REPAIR
                    && p.status == economics_compute_smoke::model::Status::Completed
            })
            .count();
        println!(
            "Month {month}: condition={} repairs={repairs} {:?}",
            run.sim.state.equipment[&m::MACHINE].remaining_uses,
            run.evidence()?
        );
    }
    for start in (13..=months).step_by(12) {
        println!(
            "{:?}",
            continuation::period(&run, start, (start + 11).min(months))?
        );
    }
    if months == 48 {
        let readiness = m::check(&run)?;
        println!("{readiness:?}");
        if !readiness.ready() {
            return Err("maintained workshop acceptance failed".into());
        }
    }
    Ok(())
}
