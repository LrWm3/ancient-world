//! Inspect independent founding liquidity preferences using existing live receipts.
use economics_compute_smoke::{
    acceptance::{
        self, Run, TOWN, WORKSHOP,
        continuation::{self, Case},
        financing::{self, Liquidity, Preferences, Stocks},
    },
    compute::Backend,
    minting::COIN,
    model::Phase,
};
const MONTHS: u32 = 48;
const DETAIL_MONTHS: u32 = 12;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: liquidity_preferences <both|metal> <terminal|exposure workshop> <terminal|exposure town>".into());
    }
    let stocks = match args[0].as_str() {
        "both" => Stocks::Both,
        "metal" => Stocks::MetalOnly,
        _ => return Err("unknown stocks".into()),
    };
    fn preference(s: &str) -> Result<Liquidity, String> {
        match s {
            "terminal" => Ok(Liquidity::TerminalReserve),
            "exposure" => Ok(Liquidity::MonthlyExposure),
            _ => Err("unknown preference".into()),
        }
    }
    let preferences = Preferences {
        workshop: preference(&args[1])?,
        town: preference(&args[2])?,
    };
    let (w, s) = financing::with_preferences(Case::Baseline, true, stocks, preferences)?;
    let mut run = Run::new(w, s, Backend::Reference)?;
    while run.sim.state.month <= MONTHS {
        run.audit.step(&mut run.sim)?;
        if run.sim.state.phase == Phase::Open {
            let month = run.sim.state.month - 1;
            for agent in [WORKSHOP, TOWN] {
                println!(
                    "CLOSE month={month} agent={agent} coins={}",
                    run.sim
                        .state
                        .balances
                        .get(&(agent, COIN))
                        .copied()
                        .unwrap_or(0)
                );
            }
        }
    }
    println!("SETTINGS stocks={stocks:?} preferences={preferences:?}");
    println!("EVIDENCE {:?}", run.evidence()?);
    println!("READINESS {:?}", acceptance::governed::check(&run)?);
    for from in [13, 25, 37] {
        println!(
            "PERIOD {from}: {:?}",
            continuation::period(&run, from, from + 11)?
        );
    }
    for agent in [WORKSHOP, TOWN] {
        let controller = &run.sim.world.agency[&agent];
        for agreement in &controller.startup_forwards {
            let t = &agreement.proposal.terms;
            println!(
                "ACTUAL signed={} scheduled_admission={} due={} status={:?} contract={:?}",
                agreement.month,
                t.month,
                t.due,
                agreement.status,
                run.sim.state.exchange.forwards.get(&t.id)
            );
        }
        for agreement in &controller.startup_forwards {
            println!("ACTUAL_PROGRAMS {:?}", agreement.programs);
        }
        for attempt in controller
            .forward_reviews
            .iter()
            .filter(|a| a.month <= DETAIL_MONTHS)
        {
            println!(
                "PROPOSAL month={} buyer={} seller={} due={}",
                attempt.month,
                attempt.proposal.terms.buyer,
                attempt.proposal.terms.seller,
                attempt.proposal.terms.due
            );
            println!(
                "  programs buyer={} seller={}",
                attempt.proposal.buyer_program, attempt.proposal.seller_program
            );
            match &attempt.assessment {
                Err(e) => println!("  error={e}"),
                Ok(a) => {
                    println!("  mutually_beneficial={}", a.mutually_beneficial());
                    for (party, c) in &a.comparisons {
                        println!(
                            "  party={party} horizon={} baseline={:?} proposed={:?} admitted={} delivered={}",
                            c.horizon, c.baseline, c.proposed, c.admitted, c.delivered_on_time
                        );
                        for (label, path) in [
                            ("baseline", &c.baseline_path),
                            ("proposed", &c.proposed_path),
                        ] {
                            println!(
                                "    {label}_coins={:?}",
                                path.closes
                                    .iter()
                                    .map(|m| (m.month, m.balance(*party, COIN)))
                                    .collect::<Vec<_>>()
                            );
                        }
                    }
                }
            }
        }
        for d in &controller.history {
            println!(
                "DECISION agent={agent} month={} effective={} chosen={:?} observed={:?} reason={}",
                d.month, d.effective_month, d.chosen, d.observed, d.reason
            );
            if d.month > DETAIL_MONTHS {
                continue;
            }
            for a in &d.alternatives {
                println!(
                    "  program={:?} commands={:?} losses={:?} failure={:?}",
                    a.program,
                    a.program
                        .and_then(|id| d.catalog.get(&id))
                        .map(|p| &p.commands),
                    a.losses,
                    a.failure
                );
                if let Some(p) = &a.projection {
                    println!(
                        "    sales={:?} closing_stocks={:?}",
                        p.sales, p.closing_stocks
                    );
                }
            }
        }
    }
    Ok(())
}
