use economics_compute_smoke::{
    acceptance::{
        WORKSHOP,
        continuation::Case,
        financing::{self, Liquidity, Stocks},
    },
    compute::Backend,
    forecast::ForecastContext,
    simulation::Simulation,
};

#[test]
fn private_context_omits_old_diagnostics_without_erasing_live_evidence() {
    let (w, s) = financing::with_liquidity(
        Case::Baseline,
        true,
        Stocks::MetalOnly,
        Liquidity::MonthlyExposure,
    )
    .unwrap();
    let mut live = Simulation::new(w, s, Backend::Reference).unwrap();
    live.run_months(5).unwrap();
    let before = live.clone();
    assert!(
        live.world.agency[&WORKSHOP]
            .history
            .iter()
            .flat_map(|d| &d.alternatives)
            .any(|a| a
                .projection
                .as_ref()
                .is_some_and(|p| !p.path.closes.is_empty()))
    );
    assert!(!live.world.agency[&WORKSHOP].forward_reviews.is_empty());
    let context = ForecastContext::new(&live.world, &live.state);
    let mut expected = before.world.clone();
    for c in expected.agency.values_mut() {
        c.autonomous = false;
        c.forward_reviews.clear();
        for d in &mut c.history {
            for a in &mut d.alternatives {
                a.projection = None;
            }
        }
    }
    assert_eq!(context.world().agency, expected.agency);
    assert_eq!(context.state(), &live.state);
    assert_eq!(
        context,
        ForecastContext::new(context.world(), context.state())
    );
    assert_eq!(live.world, before.world);
    assert_eq!(live.state, before.state);
}
