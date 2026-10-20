use economics_compute_smoke::{
    acceptance::{Run, TOWN, WORKSHOP, continuation::Case, financing::Stocks, work_discovery},
    agency::Command,
    compute::Backend,
    marketplace::Side,
    minting::{COIN, ISSUER, METAL, ORE},
};
fn compare(stocks: Stocks) {
    let mut previous: Option<economics_compute_smoke::acceptance::Evidence> = None;
    for enabled in [false, true] {
        let (w, s) = work_discovery::with_input_bids(Case::Baseline, stocks, enabled).unwrap();
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        if enabled && matches!(stocks, Stocks::Neither) {
            run.through(4).unwrap();
            // First refined output reaches the next acquisition boundary only
            // after issuer cash falls below its eight-coin input package.
            assert_eq!(run.sim.state.month, 5);
            assert_eq!(run.sim.state.balance(WORKSHOP, METAL), 2);
            assert_eq!(run.sim.state.balance(ISSUER, COIN), 7);
        }
        run.through(12).unwrap();
        let e = run.evidence().unwrap();
        println!("INPUT_BIDS {stocks:?} enabled={enabled}: {e:?}");
        if let Some(mut expected) = previous {
            if matches!(stocks, Stocks::MetalOnly) {
                expected.completed_mints += 1;
                expected.issuance += 10;
                expected.workshop_batches += 1;
                expected.municipal_batches += 1;
                expected.municipal_cash += 2;
                expected.workshop_metal_sales += 1;
                expected.workshop_net_income += 2;
            }
            assert_eq!(e, expected);
        }
        previous = Some(e);
        if enabled && matches!(stocks, Stocks::Neither) {
            // Different coordination, despite equal aggregate outcomes: a spot
            // bid elicits upstream work without the original startup forward.
            let controller = &run.sim.world.agency[&WORKSHOP];
            assert!(controller.startup_forwards.is_empty());
            let first = controller.history.iter().find(|d| d.month == 1).unwrap();
            assert!(
                matches!(first.accepted.as_ref().unwrap().commands.as_slice(),
                [Command::PeerStockQuotes(q)] if q.len() == 1 && q[0].market == ORE && q[0].side == Side::Buy)
            );
            let town = run.sim.world.agency[&TOWN]
                .history
                .iter()
                .find(|d| d.month == 2)
                .unwrap();
            assert!(
                town.accepted
                    .as_ref()
                    .unwrap()
                    .commands
                    .iter()
                    .any(|c| matches!(c, Command::WorkTargets(_)))
            );
            let policy = run
                .sim
                .world
                .minting
                .as_ref()
                .unwrap()
                .order_policy
                .as_ref()
                .unwrap();
            assert_eq!(policy.month, 2);
            assert!(policy.additional_months.is_empty());
            assert_eq!(run.sim.state.balance(WORKSHOP, METAL), 4);
        }
        assert!(run.sim.state.balances.values().all(|n| *n >= 0));
    }
}
#[test]
fn neither() {
    compare(Stocks::Neither);
}
#[test]
fn both() {
    compare(Stocks::Both);
}
#[test]
fn metal() {
    compare(Stocks::MetalOnly);
}
#[test]
fn ore() {
    compare(Stocks::OreOnly);
}
