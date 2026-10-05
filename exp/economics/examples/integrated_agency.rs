//! CPU integration run. Print missed deadlines as well as reconciled closing stocks.
use economics_compute_smoke::{
    agency::integration::*,
    compute::Backend,
    minting::{COIN, ISSUER, NUTRITION, WHEAT},
    simulation::Simulation,
};

fn main() -> Result<(), String> {
    let (w, s) = scenario()?;
    let mut audit = audit(&w, &s)?;
    let mut sim = Simulation::new(w, s, Backend::CubeCpu)?;
    while sim.state.month <= RUN_MONTHS {
        let month = sim.state.month;
        while sim.state.month == month {
            audit.step(&mut sim)?;
        }
        let overdue = sim
            .state
            .exchange
            .forwards
            .values()
            .filter(|c| c.effective_due() <= month)
            .map(|c| c.claim().outstanding())
            .sum::<i32>();
        println!(
            "month={month} state_coins={} household_food={} food_deficit={} loan_principal={:?} overdue_food={overdue}",
            sim.state.balance(ISSUER, COIN),
            sim.state.balance(HOUSEHOLD, WHEAT),
            sim.reports
                .iter()
                .filter(|r| r.month == month)
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>(),
            sim.state.credit.loans.get(&10).map(|l| l.principal)
        );
    }
    for (agent, c) in &sim.world.agency {
        for d in c.history.iter().filter(|d| d.accepted.is_some()) {
            println!(
                "agent={agent} decision_month={} effective_month={} program={}",
                d.month,
                d.effective_month,
                d.accepted.as_ref().unwrap().name
            );
        }
    }
    println!(
        "land_due={} land_paid={}",
        sim.state.obligations[&(77, 13)].owed,
        sim.state.obligations[&(77, 13)].paid
    );
    for b in &sim.ledger {
        for t in &b.transactions {
            if let Some(economics_compute_smoke::forward::Event::Delivery { contract, quantity }) =
                t.forward
            {
                println!(
                    "delivery month={} contract={contract} quantity={quantity}",
                    b.month
                );
            }
        }
    }
    for agent in &sim.world.agents {
        let b = audit.book().statements(agent.id, 1, RUN_MONTHS)?;
        println!(
            "agent={} assets={} liabilities={} equity={} cash={} issuance={}",
            agent.id, b.assets, b.liabilities, b.equity, b.closing_cash, b.issuance_change
        );
    }
    Ok(())
}
