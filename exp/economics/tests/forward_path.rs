use economics_compute_smoke::{
    acceptance::{
        WORKSHOP,
        continuation::Case,
        financing::{self, Liquidity, Stocks},
    },
    agency::forwards::{self, assessment},
    minting::COIN,
};

#[test]
fn bilateral_paths_preserve_dates_and_expose_cash_saturation() {
    let mut results = vec![];
    for policy in [Liquidity::TerminalReserve, Liquidity::MonthlyExposure] {
        let (w, s) =
            financing::with_liquidity(Case::Baseline, true, Stocks::MetalOnly, policy).unwrap();
        let p = forwards::candidates(&w, &s).unwrap().remove(0);
        let a = assessment::assess(&w, &s, &p).unwrap();
        for c in a.comparisons.values() {
            assert_eq!(c.baseline_path.closes.len(), c.horizon as usize);
            assert_eq!(c.proposed_path.closes.len(), c.horizon as usize);
            assert_eq!(
                c.proposed_path.closes.last().unwrap().month,
                s.month + c.horizon - 1
            );
        }
        let c = &a.comparisons[&WORKSHOP];
        println!(
            "{policy:?}: mutual={} base={:?} proposed={:?} base_cash={:?} proposed_cash={:?}",
            a.mutually_beneficial(),
            c.baseline,
            c.proposed,
            c.baseline_path
                .closes
                .iter()
                .map(|m| m.balance(WORKSHOP, COIN))
                .collect::<Vec<_>>(),
            c.proposed_path
                .closes
                .iter()
                .map(|m| m.balance(WORKSHOP, COIN))
                .collect::<Vec<_>>()
        );
        results.push(a);
    }
    assert!(results[0].mutually_beneficial());
    assert!(!results[1].mutually_beneficial());
    // The policy must distinguish the paths even when the final cash ties.
    assert_eq!(
        results[0].comparisons[&WORKSHOP].baseline,
        results[0].comparisons[&WORKSHOP].proposed
    );
    assert_ne!(
        results[1].comparisons[&WORKSHOP].baseline,
        results[1].comparisons[&WORKSHOP].proposed
    );
}
