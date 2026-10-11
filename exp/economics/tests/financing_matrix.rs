use economics_compute_smoke::{
    acceptance::{
        self, Run, WORKSHOP,
        continuation::Case,
        financing::{self, Liquidity, Stocks},
    },
    agency::forwards::execution::Status,
    compute::Backend,
};

const MONTHS: u32 = 48;

fn compare(stocks: Stocks) {
    for policy in [Liquidity::TerminalReserve, Liquidity::MonthlyExposure] {
        let (w, s) = financing::with_liquidity(Case::Baseline, true, stocks, policy).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(MONTHS).unwrap();
        let e = run.evidence().unwrap();
        let ready = acceptance::governed::check(&run).unwrap();
        let agreements = &run.sim.world.agency[&WORKSHOP].startup_forwards;
        println!(
            "MATRIX {stocks:?} {policy:?}: ready={} mints={} workshop={} town={} food={} wages={} agreements={:?} failures={:?}",
            ready.ready(),
            e.completed_mints,
            e.workshop_batches,
            e.municipal_batches,
            e.people_food_deficit,
            e.wage_arrears,
            agreements
                .iter()
                .map(|a| (a.month, a.status.clone()))
                .collect::<Vec<_>>(),
            ready.failures
        );
        // Regression observations, not promises that either preference is best.
        let expected = match (stocks, policy) {
            (Stocks::Both, Liquidity::TerminalReserve) => (true, 42, 41, 40, 0, 0, Some((7, 9))),
            (Stocks::Both, Liquidity::MonthlyExposure) => (false, 41, 40, 40, 0, 0, Some((7, 9))),
            (Stocks::MetalOnly, Liquidity::TerminalReserve) => {
                (false, 1, 0, 4, 61, 2, Some((2, 4)))
            }
            (Stocks::MetalOnly, Liquidity::MonthlyExposure) => {
                (true, 41, 40, 42, 0, 0, Some((5, 7)))
            }
            (Stocks::Neither, Liquidity::TerminalReserve) => (false, 0, 1, 4, 67, 2, Some((1, 3))),
            (Stocks::OreOnly, _) | (Stocks::Neither, Liquidity::MonthlyExposure) => {
                (false, 0, 0, 0, 67, 2, None)
            }
        };
        assert_eq!(
            (
                ready.ready(),
                e.completed_mints,
                e.workshop_batches,
                e.municipal_batches,
                e.people_food_deficit,
                e.wage_arrears,
            ),
            (
                expected.0, expected.1, expected.2, expected.3, expected.4, expected.5
            ),
            "{stocks:?} {policy:?}"
        );
        let expected_agreements: Vec<_> = expected
            .6
            .into_iter()
            .map(|(signed, activated)| (signed, Status::Activated(activated)))
            .collect();
        assert_eq!(
            agreements
                .iter()
                .map(|a| (a.month, a.status.clone()))
                .collect::<Vec<_>>(),
            expected_agreements,
            "{stocks:?} {policy:?}"
        );
        if stocks == Stocks::Both && policy == Liquidity::MonthlyExposure {
            assert_eq!(
                ready.failures,
                ["recurring production or market demand absent"]
            );
        }
        assert!(run.sim.state.balances.values().all(|v| *v >= 0));
    }
}
#[test]
fn both() {
    compare(Stocks::Both);
}
#[test]
fn ore_only() {
    compare(Stocks::OreOnly);
}
#[test]
fn metal_only() {
    compare(Stocks::MetalOnly);
}
#[test]
fn neither() {
    compare(Stocks::Neither);
}
