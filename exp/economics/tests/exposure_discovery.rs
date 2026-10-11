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
