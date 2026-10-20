use economics_compute_smoke::{
    acceptance::{
        self, Run, WORKSHOP,
        continuation::{self, Case},
        financing::{Liquidity, Preferences, Stocks},
        production_preferences::{self, Priority},
    },
    compute::Backend,
};

fn run(stocks: Stocks, priority: Priority, case: Case) {
    let (w, s) = production_preferences::scenario(
        case,
        stocks,
        Preferences {
            workshop: Liquidity::MonthlyExposure,
            town: Liquidity::TerminalReserve,
        },
        priority,
    )
    .unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(48).unwrap();
    let e = run.evidence().unwrap();
    let readiness = acceptance::governed::check(&run).unwrap();
    let (expected, forward) = match (case, stocks, priority) {
        (Case::NoOreSupply, Stocks::MetalOnly, Priority::ProductionFirst) => {
            ((false, 2, 0, 0, 52, 2), None)
        }
        (Case::LowMetalPrice, Stocks::MetalOnly, Priority::ProductionFirst) => {
            ((false, 6, 4, 6, 31, 2), Some((2, 4)))
        }
        (Case::Baseline, Stocks::Both, Priority::Off) => ((false, 41, 40, 40, 0, 0), Some((7, 9))),
        (Case::Baseline, Stocks::Both, Priority::ProductionFirst | Priority::CashFirst) => {
            ((true, 44, 43, 43, 0, 0), None)
        }
        (Case::Baseline, Stocks::MetalOnly, Priority::ProductionFirst) => {
            ((true, 44, 43, 44, 0, 0), Some((3, 5)))
        }
        (Case::Baseline, Stocks::OreOnly, Priority::ProductionFirst) => {
            ((false, 0, 1, 2, 67, 2), None)
        }
        (Case::Baseline, Stocks::Neither, Priority::ProductionFirst) => {
            ((false, 0, 1, 3, 67, 2), Some((1, 3)))
        }
        _ => panic!("unmeasured comparison"),
    };
    assert_eq!(
        (
            readiness.ready(),
            e.completed_mints,
            e.workshop_batches,
            e.municipal_batches,
            e.people_food_deficit,
            e.wage_arrears
        ),
        expected
    );
    assert_eq!(
        run.sim.world.agency[&WORKSHOP]
            .startup_forwards
            .iter()
            .map(|a| (a.month, a.status.clone()))
            .collect::<Vec<_>>(),
        forward
            .into_iter()
            .map(|(signed, activated)| (
                signed,
                economics_compute_smoke::agency::forwards::execution::Status::Activated(activated)
            ))
            .collect::<Vec<_>>()
    );
    println!(
        "FLOW {stocks:?} {priority:?} {case:?}: ready={} mints={} workshop={} town={} food={} wages={} forwards={:?} failures={:?}",
        readiness.ready(),
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
        readiness.failures
    );
    for from in [13, 25, 37] {
        println!(
            "PERIOD {stocks:?} {priority:?} {case:?} {from}: {:?}",
            continuation::period(&run, from, from + 11).unwrap()
        );
    }
    assert!(run.sim.state.balances.values().all(|q| *q >= 0));
    if case == Case::NoOreSupply {
        assert_eq!(e.workshop_batches, 0);
        assert!(!readiness.ready());
    }
}
#[test]
fn both_production_first() {
    run(Stocks::Both, Priority::ProductionFirst, Case::Baseline);
}
#[test]
fn metal_production_first() {
    run(Stocks::MetalOnly, Priority::ProductionFirst, Case::Baseline);
}
#[test]
fn ore_production_first() {
    run(Stocks::OreOnly, Priority::ProductionFirst, Case::Baseline);
}
#[test]
fn neither_production_first() {
    run(Stocks::Neither, Priority::ProductionFirst, Case::Baseline);
}
#[test]
fn both_cash_first() {
    run(Stocks::Both, Priority::CashFirst, Case::Baseline);
}
#[test]
fn both_cash_only() {
    run(Stocks::Both, Priority::Off, Case::Baseline);
}
#[test]
fn no_ore() {
    run(
        Stocks::MetalOnly,
        Priority::ProductionFirst,
        Case::NoOreSupply,
    );
}
#[test]
fn low_price() {
    run(
        Stocks::MetalOnly,
        Priority::ProductionFirst,
        Case::LowMetalPrice,
    );
}
