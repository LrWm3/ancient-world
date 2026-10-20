use economics_compute_smoke::{
    acceptance::{
        WORKSHOP,
        continuation::Case,
        financing::{Liquidity, Preferences, Stocks},
        production_preferences::{self, Priority},
    },
    compute::Backend,
    minting::REFINE,
    simulation::Simulation,
};
use std::collections::BTreeSet;

#[test]
fn bilateral_scores_use_each_branchs_committed_monthly_work() {
    let (w, s) = production_preferences::scenario(
        Case::Baseline,
        Stocks::MetalOnly,
        Preferences {
            workshop: Liquidity::MonthlyExposure,
            town: Liquidity::TerminalReserve,
        },
        Priority::ProductionFirst,
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(6).unwrap();
    let reviews = &sim.world.agency[&WORKSHOP].forward_reviews;
    let mut checked = 0;
    for review in reviews {
        let Ok(assessment) = &review.assessment else {
            continue;
        };
        let c = &assessment.comparisons[&WORKSHOP];
        for (loss, rows) in [
            (&c.baseline, &c.baseline_completions),
            (&c.proposed, &c.proposed_completions),
        ] {
            let months: BTreeSet<_> = rows
                .iter()
                .filter(|r| r.definition == REFINE)
                .map(|r| r.month)
                .collect();
            assert_eq!(loss[0], i128::from(c.horizon) - months.len() as i128);
            checked += 1;
        }
    }
    assert!(checked > 0);
    for decision in &sim.world.agency[&WORKSHOP].history {
        if decision.month == 1 {
            assert_eq!(decision.observed[0], 0);
            continue;
        }
        let rows = economics_compute_smoke::agency::completions::observe(
            &sim.ledger,
            WORKSHOP,
            decision.month - 1,
            decision.month - 1,
        );
        let completed = rows.iter().filter(|r| r.definition == REFINE).count();
        assert_eq!(decision.observed[0], if completed == 0 { 1 } else { 0 });
    }
}
