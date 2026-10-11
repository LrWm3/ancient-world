use economics_compute_smoke::{
    acceptance::{
        WORKSHOP,
        buffers::{self, Policy},
        continuation::Case,
        financing::{self, Liquidity, Preferences, Stocks},
    },
    agency::{
        self,
        objectives::{Metric, Scope},
    },
    minting::ORE,
};

#[test]
fn buffers_change_only_workshop_goals_not_catalogs_endowments_or_authority() {
    let preferences = Preferences {
        workshop: Liquidity::MonthlyExposure,
        town: Liquidity::TerminalReserve,
    };
    for stocks in Stocks::ALL {
        for enabled in [false, true] {
            let (base, state) =
                financing::with_preferences(Case::Baseline, enabled, stocks, preferences).unwrap();
            for policy in [Policy::Off, Policy::InputsFirst, Policy::CashFirst] {
                let (mut w, s) =
                    buffers::scenario(Case::Baseline, enabled, stocks, preferences, policy)
                        .unwrap();
                assert_eq!(s, state);
                assert_eq!(
                    agency::discovery::programs(&w, WORKSHOP).unwrap(),
                    agency::discovery::programs(&base, WORKSHOP).unwrap()
                );
                assert_eq!(
                    agency::discovery_sales::opportunities(&w, WORKSHOP).unwrap(),
                    agency::discovery_sales::opportunities(&base, WORKSHOP).unwrap()
                );
                let config = &mut w.agency.get_mut(&WORKSHOP).unwrap().config;
                if policy != Policy::Off {
                    let at = usize::from(policy == Policy::CashFirst);
                    let goal = config.objectives.remove(at);
                    assert_eq!(goal.scope, Scope::Organization);
                    assert_eq!(
                        goal.metric,
                        Metric::ReserveExposure {
                            resource: ORE,
                            target: 2
                        }
                    );
                }
                assert_eq!(w, base);
            }
        }
    }
}
