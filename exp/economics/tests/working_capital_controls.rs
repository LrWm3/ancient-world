//! Matching liquidity does not create resources, legal access or profitable margins.
use economics_compute_smoke::{
    acceptance::{
        self,
        continuation::{self, Case},
        financing::Stocks,
        working_capital,
    },
    compute::Backend,
    minting::ISSUER,
};
fn run(case: Case, stocks: Stocks, consent: bool, months: u32) -> acceptance::Run {
    let (w, s) = working_capital::scenario(case, stocks, consent).unwrap();
    let mut run = acceptance::Run::new(w, s, Backend::Reference).unwrap();
    run.through(months).unwrap();
    println!(
        "{case:?} {stocks:?} consent={consent} {:?}",
        run.evidence().unwrap()
    );
    run
}
#[test]
fn no_lending_consent_keeps_the_48_month_economy_unready() {
    let r = run(Case::Baseline, Stocks::Neither, false, 48);
    assert!(!continuation::check(&r).unwrap().ready());
    assert_eq!(r.evidence().unwrap().completed_mints, 0);
    assert!(r.sim.world.agency[&ISSUER].financed_programs.is_empty());
}
#[test]
fn financing_does_not_create_ore() {
    blocked(Case::NoOreSupply);
}
#[test]
fn financing_does_not_grant_market_access() {
    blocked(Case::NoMetalAccess);
}
#[test]
fn financing_does_not_make_a_loss_making_chain_sustainable() {
    let r = run(Case::LowMetalPrice, Stocks::Neither, true, 48);
    let evidence = r.evidence().unwrap();
    assert!(evidence.workshop_net_income < 0);
    assert!(!continuation::check(&r).unwrap().ready());
}
fn blocked(case: Case) {
    let r = run(case, Stocks::Neither, true, 12);
    assert_eq!(r.evidence().unwrap().completed_mints, 0);
    assert!(r.sim.world.agency[&ISSUER].financed_programs.is_empty());
}
#[test]
fn historical_payroll_failure_cannot_be_cured_out_of_readiness() {
    let r = run(Case::LatePayroll, Stocks::Neither, true, 48);
    assert!(
        continuation::check(&r)
            .unwrap()
            .failures
            .iter()
            .any(|f| f.contains("wages entered arrears"))
    );
}
#[test]
fn financing_preserves_inherited_stock_successes() {
    for stocks in [Stocks::Both, Stocks::OreOnly, Stocks::MetalOnly] {
        let r = run(Case::Baseline, stocks, true, 12);
        let baseline = run(Case::Baseline, stocks, false, 12);
        let e = r.evidence().unwrap();
        assert!(
            e.completed_mints >= baseline.evidence().unwrap().completed_mints,
            "financing regressed {stocks:?}"
        );
        assert_eq!(e.people_food_deficit, 0);
        assert_eq!(e.wage_arrears, 0);
    }
}
