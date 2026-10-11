use economics_compute_smoke::{
    acceptance::{TOWN, WORKSHOP, continuation::Case, financing},
    agency::{self, objectives::Metric},
};
#[test]
fn exposure_goals_keep_the_same_signed_program_catalog_and_sale_opportunities() {
    let (w, _) = financing::scenario(Case::Baseline).unwrap();
    let mut variant = w.clone();
    for agent in [WORKSHOP, TOWN] {
        let c = &mut variant.agency.get_mut(&agent).unwrap().config;
        for goal in c
            .objectives
            .iter_mut()
            .chain(c.preferences.values_mut().flatten())
        {
            if let Metric::Reserve { resource, target } = goal.metric {
                goal.metric = Metric::ReserveExposure { resource, target };
            }
        }
        assert_eq!(
            agency::discovery::programs(&w, agent).unwrap(),
            agency::discovery::programs(&variant, agent).unwrap()
        );
        assert_eq!(
            agency::discovery_sales::opportunities(&w, agent).unwrap(),
            agency::discovery_sales::opportunities(&variant, agent).unwrap()
        );
    }
}

#[test]
fn ordinary_projection_detail_is_opt_in_and_scoring_uses_completed_months() {
    use economics_compute_smoke::{
        acceptance::financing::{Liquidity, Stocks},
        compute::Backend,
        simulation::Simulation,
    };
    for policy in [Liquidity::TerminalReserve, Liquidity::MonthlyExposure] {
        let (w, s) = financing::with_liquidity(Case::Baseline, true, Stocks::Both, policy).unwrap();
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.step().unwrap();
        let c = &sim.world.agency[&WORKSHOP];
        let decision = c.history.last().unwrap();
        let mut inspected = 0;
        for alternative in &decision.alternatives {
            if let Some(projection) = &alternative.projection {
                inspected += 1;
                match policy {
                    Liquidity::TerminalReserve => assert!(projection.path.closes.is_empty()),
                    Liquidity::MonthlyExposure => {
                        assert_eq!(projection.path.closes.len(), c.config.horizon as usize);
                        let Metric::ReserveExposure { resource, target } =
                            c.config.objectives[0].metric
                        else {
                            panic!("wrong objective")
                        };
                        let expected: i128 = projection
                            .path
                            .closes
                            .iter()
                            .map(|m| {
                                (i128::from(target) - i128::from(m.balance(WORKSHOP, resource)))
                                    .max(0)
                            })
                            .sum();
                        assert_eq!(alternative.losses.as_ref().unwrap()[0], expected);
                    }
                }
            }
        }
        assert!(inspected > 0);
    }
}
