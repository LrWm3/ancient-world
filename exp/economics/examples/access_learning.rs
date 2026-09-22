use economics_compute_smoke::{
    access_expectations::Mode, allocation::Policy, compute::Backend, intermediary, model::Status,
    scenario::*,
};
const AMPLE_WOOD: i32 = 6;
const AMPLE_REGENERATION: i32 = 3;

fn main() -> Result<(), String> {
    println!(
        "| Policy | Seed | Supply | Forecast | Person | Nutrition deficit | Warmth deficit | Tools | Harvest grain | Dead |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"
    );
    for (policy, seed) in [
        (Policy::StablePriority, 7),
        (Policy::Lottery, 7),
        (Policy::Lottery, 19),
    ] {
        for ample in [false, true] {
            for mode in [Mode::Optimistic, Mode::Learned] {
                let mut e = intermediary::scenario(Backend::CubeCpu)?;
                e.policy = policy;
                e.seed = seed;
                e.access_mode = mode;
                if ample {
                    let pool = e
                        .simulation
                        .world
                        .pools
                        .iter_mut()
                        .find(|p| p.account == (STATE_AGENT, RAW_WOOD))
                        .unwrap();
                    pool.capacity = AMPLE_WOOD;
                    pool.monthly_regeneration = AMPLE_REGENERATION;
                    e.simulation
                        .state
                        .balances
                        .insert((STATE_AGENT, RAW_WOOD), AMPLE_WOOD);
                }
                e.run_months(intermediary::RUN_MONTHS)?;
                for p in &e.simulation.world.participants {
                    let deficit = |resource| {
                        e.simulation
                            .reports
                            .iter()
                            .filter(|r| r.agent == p.agent)
                            .map(|r| r.deficit(resource))
                            .sum::<i32>()
                    };
                    let tools = e
                        .simulation
                        .state
                        .equipment
                        .values()
                        .filter(|a| a.owner == p.agent)
                        .count();
                    let grain = e
                        .simulation
                        .ledger
                        .iter()
                        .flat_map(|b| &b.transactions)
                        .filter(|t| {
                            t.process.as_ref().is_some_and(|p| {
                                p.after.definition == GROW && p.after.status == Status::Completed
                            })
                        })
                        .flat_map(|t| &t.effects)
                        .filter(|v| v.account == (p.agent, GRAIN) && v.delta > 0)
                        .map(|v| v.delta)
                        .sum::<i32>();
                    println!(
                        "| {policy:?} | {seed} | {} | {mode:?} | {} | {} | {} | {tools} | {grain} | {} |",
                        if ample { "ample" } else { "scarce" },
                        p.agent,
                        deficit(NUTRITION),
                        deficit(WARMTH),
                        e.simulation.state.terminal.contains_key(&p.agent)
                    );
                }
                if !ample && policy == Policy::StablePriority {
                    for r in &e.rounds {
                        eprintln!(
                            "{mode:?} month {} {:?} estimates {:?}",
                            r.month,
                            r.decisions
                                .iter()
                                .map(|d| (d.agent, d.selected, d.fallback))
                                .collect::<Vec<_>>(),
                            r.expectations
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
