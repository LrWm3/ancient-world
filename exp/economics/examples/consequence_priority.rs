use economics_compute_smoke::{
    compute::Backend, consequence_priority::Mode, intermediary, model::Status, scenario::*,
};
fn main() -> Result<(), String> {
    println!(
        "| Covered warmth | Mode | Person | Month 2 warmth deficit | Total warmth deficit | Nutrition deficit | Crops completed | Crops aborted | Grain | Tools | Terminal |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"
    );
    for covered in [false, true] {
        for mode in [Mode::Existing, Mode::AvoidHarm] {
            let mut e = intermediary::consequence_scenario(covered, Backend::CubeCpu)?;
            e.allocation_mode = mode;
            e.run_months(intermediary::RUN_MONTHS)?;
            eprintln!(
                "covered={covered} mode={mode:?} first proposals {:?} grants {:?}",
                e.rounds[0]
                    .decisions
                    .iter()
                    .map(|d| (d.agent, d.selected, d.fallback))
                    .collect::<Vec<_>>(),
                e.rounds[0].resolution.receipts
            );
            for (id, r) in &e.rounds[0].assessments {
                eprintln!("claim {id} priority {:?}", r.priority()?);
            }
            for p in &e.simulation.world.participants {
                let deficit = |need, first: bool| {
                    e.simulation
                        .reports
                        .iter()
                        .filter(|r| r.agent == p.agent && (!first || r.month == 2))
                        .map(|r| r.deficit(need))
                        .sum::<i32>()
                };
                let crops = |status| {
                    e.simulation
                        .state
                        .processes
                        .values()
                        .filter(|r| {
                            r.operator == p.agent && r.definition == GROW && r.status == status
                        })
                        .count()
                };
                let grain = e
                    .simulation
                    .ledger
                    .iter()
                    .flat_map(|b| &b.transactions)
                    .filter(|t| {
                        t.process.as_ref().is_some_and(|r| {
                            r.after.definition == GROW && r.after.status == Status::Completed
                        })
                    })
                    .flat_map(|t| &t.effects)
                    .filter(|v| v.account == (p.agent, GRAIN) && v.delta > 0)
                    .map(|v| v.delta)
                    .sum::<i32>();
                let tools = e
                    .simulation
                    .state
                    .equipment
                    .values()
                    .filter(|a| a.owner == p.agent)
                    .count();
                println!(
                    "| {covered} | {mode:?} | {} | {} | {} | {} | {} | {} | {grain} | {tools} | {} |",
                    p.agent,
                    deficit(WARMTH, true),
                    deficit(WARMTH, false),
                    deficit(NUTRITION, false),
                    crops(Status::Completed),
                    crops(Status::Aborted),
                    e.simulation.state.terminal.contains_key(&p.agent)
                );
            }
        }
    }
    Ok(())
}
