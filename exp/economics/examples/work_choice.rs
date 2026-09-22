use economics_compute_smoke::{
    compute::Backend,
    scenario::{GRAIN, RAW_WOOD, SEED, STATE_AGENT},
    simulation::Simulation,
    work_choice,
};
const MONTHS: u32 = 6;
fn main() -> Result<(), String> {
    println!(
        "| Case | Month 3 continue | Projected net value | State grain | State seed | State wood | Debt ticks |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    for case in ["mature", "expensive", "no-capacity"] {
        let (w, s) = work_choice::scenario(case)?;
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        sim.run_months(MONTHS)?;
        let d = sim
            .ledger
            .iter()
            .find_map(|b| b.work_choice.as_ref().filter(|d| d.month == 3))
            .unwrap();
        let chosen = &d.forecasts[d.selected];
        println!(
            "| {case} | {} | {} | {} | {} | {} | {} |",
            chosen.plan.continue_active,
            chosen.net_value,
            sim.state.balance(STATE_AGENT, GRAIN),
            sim.state.balance(STATE_AGENT, SEED),
            sim.state.balance(STATE_AGENT, RAW_WOOD),
            sim.state.credit.loans[&1].principal
        );
        for f in &d.forecasts {
            eprintln!(
                "{case}: {:?}: value {}, work {}",
                f.plan, f.net_value, f.labor
            );
        }
    }
    Ok(())
}
