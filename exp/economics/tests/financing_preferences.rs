use economics_compute_smoke::{
    acceptance::{
        TOWN, WORKSHOP,
        continuation::Case,
        financing::{self, Liquidity, Preferences, Stocks},
    },
    agency::{self, objectives::Metric},
    minting::COIN,
};

#[test]
fn founding_preferences_are_independent_and_preserve_all_other_inputs() {
    for enabled in [false, true] {
        for stocks in Stocks::ALL {
            let (base, state) = financing::with_liquidity(
                Case::Baseline,
                enabled,
                stocks,
                Liquidity::TerminalReserve,
            )
            .unwrap();
            for workshop in [Liquidity::TerminalReserve, Liquidity::MonthlyExposure] {
                for town in [Liquidity::TerminalReserve, Liquidity::MonthlyExposure] {
                    let (mut variant, actual_state) = financing::with_preferences(
                        Case::Baseline,
                        enabled,
                        stocks,
                        Preferences { workshop, town },
                    )
                    .unwrap();
                    assert_eq!(actual_state, state);
                    for (agent, policy) in [(WORKSHOP, workshop), (TOWN, town)] {
                        assert_eq!(
                            agency::discovery::programs(&base, agent).unwrap(),
                            agency::discovery::programs(&variant, agent).unwrap()
                        );
                        assert_eq!(
                            agency::discovery_sales::opportunities(&base, agent).unwrap(),
                            agency::discovery_sales::opportunities(&variant, agent).unwrap()
                        );
                        let config = &mut variant.agency.get_mut(&agent).unwrap().config;
                        let mut changed = 0;
                        for goal in config
                            .objectives
                            .iter_mut()
                            .chain(config.preferences.values_mut().flatten())
                        {
                            if let Metric::ReserveExposure { resource, target } = goal.metric {
                                assert_eq!(policy, Liquidity::MonthlyExposure);
                                assert_eq!(resource, COIN);
                                goal.metric = Metric::Reserve { resource, target };
                                changed += 1;
                            }
                        }
                        let expected_changes = if policy == Liquidity::MonthlyExposure {
                            base.agency[&agent]
                                .config
                                .objectives
                                .iter()
                                .chain(base.agency[&agent].config.preferences.values().flatten())
                                .filter(|goal| {
                                    matches!(goal.metric, Metric::Reserve { resource: COIN, .. })
                                })
                                .count()
                        } else {
                            0
                        };
                        assert_eq!(changed, expected_changes);
                    }
                    assert_eq!(variant, base);
                }
            }
        }
    }
}

#[test]
fn symmetric_helper_preserves_existing_fixtures() {
    for stocks in Stocks::ALL {
        for policy in [Liquidity::TerminalReserve, Liquidity::MonthlyExposure] {
            assert_eq!(
                financing::with_liquidity(Case::Baseline, true, stocks, policy).unwrap(),
                financing::with_preferences(
                    Case::Baseline,
                    true,
                    stocks,
                    Preferences {
                        workshop: policy,
                        town: policy
                    }
                )
                .unwrap()
            );
        }
    }
}
