use economics_compute_smoke::{
    acceptance::{
        self, Run, WORKSHOP,
        buffers::{self, Policy},
        continuation::{self, Case},
        financing::{Liquidity, Preferences, Stocks},
    },
    compute::Backend,
};

fn compare(stocks: Stocks) {
    for policy in [Policy::Off, Policy::InputsFirst, Policy::CashFirst] {
        let (w, s) = buffers::scenario(
            Case::Baseline,
            true,
            stocks,
            Preferences {
                workshop: Liquidity::MonthlyExposure,
                town: Liquidity::TerminalReserve,
            },
            policy,
        )
        .unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(48).unwrap();
        let e = run.evidence().unwrap();
        let ready = acceptance::governed::check(&run).unwrap();
        println!(
            "BUFFER {stocks:?} {policy:?}: ready={} mints={} workshop={} town={} food={} wages={} forwards={:?} failures={:?}",
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
        let expected = match (stocks, policy) {
            (Stocks::Both, Policy::InputsFirst) => (false, 2, 0, 0, 52, 2, None),
            (Stocks::Both, _) => (false, 41, 40, 40, 0, 0, Some((7, 9))),
            (Stocks::MetalOnly, Policy::InputsFirst) => (false, 1, 0, 3, 61, 2, Some((2, 4))),
            (Stocks::MetalOnly, _) => (true, 41, 40, 42, 0, 0, Some((5, 7))),
            (Stocks::Neither, Policy::InputsFirst) => (false, 0, 1, 4, 67, 2, Some((1, 3))),
            (Stocks::OreOnly | Stocks::Neither, _) => (false, 0, 0, 0, 67, 2, None),
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
            (
                expected.0, expected.1, expected.2, expected.3, expected.4, expected.5
            ),
            "{stocks:?} {policy:?}"
        );
        assert_eq!(
            run.sim.world.agency[&WORKSHOP]
                .startup_forwards
                .iter()
                .map(|a| (a.month, a.status.clone()))
                .collect::<Vec<_>>(),
            expected
                .6
                .into_iter()
                .map(|(signed, activated)| (
                    signed,
                    economics_compute_smoke::agency::forwards::execution::Status::Activated(
                        activated
                    )
                ))
                .collect::<Vec<_>>()
        );
        for from in [13, 25, 37] {
            println!(
                "PERIOD {stocks:?} {policy:?} {from}: {:?}",
                continuation::period(&run, from, from + 11).unwrap()
            );
        }
        assert!(run.sim.state.balances.values().all(|b| *b >= 0));
    }
}
#[test]
fn both() {
    compare(Stocks::Both);
}
#[test]
fn metal_only() {
    compare(Stocks::MetalOnly);
}
#[test]
fn ore_only() {
    compare(Stocks::OreOnly);
}
#[test]
fn neither() {
    compare(Stocks::Neither);
}
