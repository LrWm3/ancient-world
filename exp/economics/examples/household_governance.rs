//! Reproducible lawful founding, election and operating-policy comparison.
use economics_compute_smoke::{
    compute::Backend,
    household_governance::{self as governance, scenario},
    scenario::NUTRITION,
    simulation::Simulation,
};
fn main() -> Result<(), String> {
    let (w, s) = scenario::pair()?;
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference)?;
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu)?;
    for month in 1..=scenario::RUN_MONTHS {
        if month == 1 + scenario::TERM_MONTHS {
            for sim in [&mut reference, &mut cpu] {
                scenario::incoming_policy(&mut sim.world, &sim.state)?;
            }
        }
        let authority = governance::authority(&reference.world.households[0], &reference.state);
        reference.run_months(1)?;
        cpu.run_months(1)?;
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        assert_eq!(reference.reports, cpu.reports);
        let unmet: i32 = reference
            .reports
            .iter()
            .filter(|r| r.month == month)
            .map(|r| r.deficit(NUTRITION))
            .sum();
        println!(
            "month {month}: governor {:?}, {:?}, {:?}; unmet nutrition {unmet}",
            authority.leader, authority.policy, authority.tie_break
        );
    }
    println!("CPU/reference state, ledger and reports match at every month");
    Ok(())
}
