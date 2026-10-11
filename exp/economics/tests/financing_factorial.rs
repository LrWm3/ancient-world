use economics_compute_smoke::{
    acceptance::{
        self, Run, WORKSHOP,
        continuation::{self, Case},
        financing::{self, Liquidity, Preferences, Stocks},
    },
    agency::forwards::execution::Status,
    compute::Backend,
};

const MONTHS: u32 = 48;
fn factorial(stocks: Stocks) {
    for workshop in [Liquidity::TerminalReserve, Liquidity::MonthlyExposure] {
        for town in [Liquidity::TerminalReserve, Liquidity::MonthlyExposure] {
            let (w, s) = financing::with_preferences(
                Case::Baseline,
                true,
                stocks,
                Preferences { workshop, town },
            )
            .unwrap();
            let mut run = Run::new(w, s, Backend::Reference).unwrap();
            run.through(MONTHS).unwrap();
            let e = run.evidence().unwrap();
            let ready = acceptance::governed::check(&run).unwrap();
            println!(
                "FACTORIAL {stocks:?} workshop={workshop:?} town={town:?} ready={} mints={} workshop={} town={} food={} wages={} forwards={:?} failures={:?}",
                ready.ready(),
                e.completed_mints,
                e.workshop_batches,
                e.municipal_batches,
                e.people_food_deficit,
                e.wage_arrears,
                run.sim.world.agency[&WORKSHOP]
                    .startup_forwards
                    .iter()
                    .map(|a| (a.month, a.status.clone()))
                    .collect::<Vec<_>>(),
                ready.failures
            );
            if stocks == Stocks::MetalOnly {
                let exposure = workshop == Liquidity::MonthlyExposure;
                let expected = if exposure {
                    (true, 41, 40, 42, 0, 0)
                } else {
                    (false, 1, 0, 4, 61, 2)
                };
                assert_eq!(
                    (
                        ready.ready(),
                        e.completed_mints,
                        e.workshop_batches,
                        e.municipal_batches,
                        e.people_food_deficit,
                        e.wage_arrears
                    ),
                    expected
                );
                let c = &run.sim.world.agency[&WORKSHOP];
                assert_eq!(c.startup_forwards.len(), 1);
                let a = &c.startup_forwards[0];
                let signed = if exposure { 5 } else { 2 };
                assert_eq!(
                    (a.month, a.proposal.terms.month, a.proposal.terms.due),
                    (signed, signed + 1, signed + 3)
                );
                assert_eq!(a.status, Status::Activated(signed + 2));
                assert_eq!(
                    run.sim.state.exchange.forwards[&a.proposal.terms.id].delivered,
                    2
                );
                let first = &c.forward_reviews[0];
                assert_eq!(first.month, 2);
                let assessment = first.assessment.as_ref().unwrap();
                let buyer = &assessment.comparisons[&WORKSHOP];
                let (baseline, proposed) = if exposure {
                    (11949, 11973)
                } else {
                    (1000, 1000)
                };
                assert_eq!(buyer.baseline, [baseline]);
                assert_eq!(buyer.proposed, [proposed]);
                assert_eq!(assessment.mutually_beneficial(), !exposure);
                if exposure {
                    assert!(ready.failures.is_empty());
                } else {
                    assert_eq!(
                        ready.failures,
                        [
                            "missing paid operating labor",
                            "next operating payment not covered",
                            "nonpositive operating income",
                            "operating reserves declining",
                            "recurring production or market demand absent",
                            "replacement ore not delivered",
                            "unmet nutrition",
                            "unpaid earned wages",
                            "wages entered arrears"
                        ]
                    );
                }
            }
            if stocks == Stocks::Both {
                let exposure = workshop == Liquidity::MonthlyExposure;
                assert_eq!(
                    (
                        ready.ready(),
                        e.completed_mints,
                        e.workshop_batches,
                        e.municipal_batches,
                        e.people_food_deficit,
                        e.wage_arrears
                    ),
                    if exposure {
                        (false, 41, 40, 40, 0, 0)
                    } else {
                        (true, 42, 41, 40, 0, 0)
                    }
                );
                let agreements = &run.sim.world.agency[&WORKSHOP].startup_forwards;
                assert_eq!(agreements.len(), 1);
                let a = &agreements[0];
                assert_eq!(
                    (a.month, a.proposal.terms.month, a.proposal.terms.due),
                    (7, 8, 10)
                );
                assert_eq!(a.status, Status::Activated(9));
                assert_eq!(
                    run.sim.state.exchange.forwards[&a.proposal.terms.id].delivered,
                    2
                );
                if exposure {
                    assert_eq!(
                        ready.failures,
                        ["recurring production or market demand absent"]
                    );
                } else {
                    assert!(ready.failures.is_empty());
                }
                for from in [13, 25, 37] {
                    let p = continuation::period(&run, from, from + 11).unwrap();
                    let batches = if exposure && from == 13 { 11 } else { 12 };
                    assert_eq!(
                        (
                            p.workshop_batches,
                            p.municipal_batches,
                            p.metal_sales,
                            p.ore_delivered
                        ),
                        (batches, 12, batches, 24)
                    );
                    assert_eq!(
                        (
                            p.workshop_hours,
                            p.municipal_hours,
                            p.workshop_wages,
                            p.municipal_wages
                        ),
                        (12, 12, 12, 12)
                    );
                }
            }
            for from in [13, 25, 37] {
                println!(
                    "PERIOD {stocks:?} {workshop:?}/{town:?} {from}: {:?}",
                    continuation::period(&run, from, from + 11).unwrap()
                );
            }
            assert!(run.sim.state.balances.values().all(|v| *v >= 0));
        }
    }
}
#[test]
fn metal_only_preferences() {
    factorial(Stocks::MetalOnly);
}
#[test]
fn both_stock_preferences() {
    factorial(Stocks::Both);
}
