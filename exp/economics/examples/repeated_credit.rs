//! Controlled cultivation-right audit; all money, food and work rules are identical.
use economics_compute_smoke::{
    compute::Backend,
    model::Status,
    scenario::{GRAIN, GROW, NUTRITION, PERSON, SEED, TOKEN},
    simulation::Simulation,
    stock_sale,
};
const OBSERVATION_MONTHS: u32 = 60;
const LEGACY_RIGHT_END: u32 = 9;
fn main() -> Result<(), String> {
    for short_right in [true, false] {
        let (mut world, state) = stock_sale::scenario("funded")?;
        if short_right {
            for right in &mut world.rights {
                right.through = LEGACY_RIGHT_END;
            }
        }
        let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
        println!(
            "cultivation right through month {}",
            sim.world.rights[0].through
        );
        for _ in 0..OBSERVATION_MONTHS {
            sim.run_months(1)?;
            let month = sim.state.month - 1;
            let crops: Vec<_> = sim
                .state
                .processes
                .values()
                .filter(|p| {
                    p.definition == GROW && (p.start == month || p.reserved_through == month)
                })
                .map(|p| (p.start, p.reserved_through, p.status))
                .collect();
            let report = sim
                .reports
                .iter()
                .find(|r| r.month == month && r.agent == PERSON)
                .unwrap();
            println!(
                "month {month}: grain={} seed={} deficit={} crops={crops:?}",
                sim.state.balance(PERSON, GRAIN),
                sim.state.balance(PERSON, SEED),
                report.deficit(NUTRITION)
            );
        }
        let harvests: Vec<_> = sim
            .state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Completed)
            .map(|p| p.reserved_through)
            .collect();
        let deficit: i32 = sim
            .reports
            .iter()
            .filter(|r| r.agent == PERSON)
            .map(|r| r.deficit(NUTRITION))
            .sum();
        println!(
            "harvests={harvests:?}, deficit={deficit}, coins={}, state purchases={} ticks",
            sim.state.balance(PERSON, TOKEN),
            sim.state.credit.stock_spent
        );
    }
    Ok(())
}
