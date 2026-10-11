use economics_compute_smoke::{
    acceptance::{
        WORKSHOP,
        continuation::Case,
        financing::{self, Liquidity, Stocks},
    },
    agency::completions::Completion,
    compute::Backend,
    simulation::Simulation,
};

#[test]
fn bilateral_reviews_retain_dated_party_completions_even_when_consent_fails() {
    let (w, s) = financing::with_liquidity(
        Case::Baseline,
        true,
        Stocks::MetalOnly,
        Liquidity::MonthlyExposure,
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(5).unwrap();
    let mut accepted = false;
    let mut rejected = false;
    let mut evidence = std::collections::BTreeSet::new();
    for review in &sim.world.agency[&WORKSHOP].forward_reviews {
        let Ok(a) = &review.assessment else { continue };
        accepted |= a.mutually_beneficial();
        rejected |= !a.mutually_beneficial();
        for (&party, c) in &a.comparisons {
            for (proposed, rows) in [
                (false, &c.baseline_completions),
                (true, &c.proposed_completions),
            ] {
                if !rows.is_empty() {
                    evidence.insert((a.mutually_beneficial(), proposed));
                }
                assert!(rows.iter().all(|r| r.operator == party
                    && r.month >= review.month
                    && r.month < review.month + c.horizon));
                assert!(rows.windows(2).all(
                    |r: &[Completion]| (r[0].month, r[0].process) < (r[1].month, r[1].process)
                ));
            }
        }
    }
    assert!(accepted && rejected);
    // Before startup the retained baseline completes no work; proposals do.
    assert_eq!(evidence, [(false, true), (true, true)].into());
    // Diagnostics do not alter the known consent divergence or its signing date.
    let agreements = &sim.world.agency[&WORKSHOP].startup_forwards;
    assert_eq!(agreements.len(), 1);
    assert_eq!(agreements[0].month, 5);
}
