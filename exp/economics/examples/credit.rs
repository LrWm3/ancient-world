use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Event},
    scenario::{PERSON, PLOT, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
const MONTHS: u32 = 6;
fn main() -> Result<(), String> {
    println!("All monetary amounts below are coin ticks (100 per coin).");
    println!(
        "| Case | Status | Principal | Interest owed | Interest paid | Owner | Buyer coins | Buyer equity | Lender equity |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    for case in ["repaid", "downpayment", "recovered", "default", "surplus"] {
        let (w, s) = credit::scenario(case)?;
        let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
        sim.run_months(MONTHS)?;
        let loan = sim.state.credit.loans.get(&1);
        let interest_paid: i32 = sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.events)
            .filter_map(|e| {
                if let Event::Paid { interest, .. } = e {
                    Some(*interest)
                } else {
                    None
                }
            })
            .sum();
        let borrower = credit::balance_sheet(&sim.world, &sim.state, PERSON, TOKEN);
        let lender = credit::balance_sheet(&sim.world, &sim.state, STATE_AGENT, TOKEN);
        println!(
            "| {case} | {:?} | {} | {} | {interest_paid} | {} | {} | {} | {} |",
            loan.map(|l| l.status),
            loan.map_or(0, |l| l.principal),
            loan.map_or(0, |l| l.interest),
            credit::owner(&sim.world, &sim.state, PLOT).unwrap(),
            borrower.coins,
            borrower.equity(),
            lender.equity()
        );
        for event in sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.events)
        {
            if matches!(event, Event::Enforced { .. } | Event::Rejected { .. }) {
                eprintln!("{case}: {event:?}");
            }
        }
    }
    println!("\n| Crop control | Crop status | State grain | State seed | Remaining debt ticks |");
    println!("| --- | --- | --- | --- | --- |");
    for maintain in [true, false] {
        let (world, state) = credit::crop_scenario(maintain)?;
        let mut sim = Simulation::new(world, state, Backend::CubeCpu)?;
        sim.run_months(MONTHS)?;
        println!(
            "| {} | {:?} | {} | {} | {} |",
            if maintain { "maintain" } else { "neglect" },
            sim.state.processes.values().next().unwrap().status,
            sim.state
                .balance(STATE_AGENT, economics_compute_smoke::scenario::GRAIN),
            sim.state
                .balance(STATE_AGENT, economics_compute_smoke::scenario::SEED),
            sim.state.credit.loans[&1].principal
        );
    }
    Ok(())
}
