use economics_compute_smoke::{
    acceptance::{self, continuation as c, publication, sales::Startup},
    agency::{
        forwards::{self, Options, assessment},
        organization,
        publication::Policy,
    },
    minting::ORE,
    model::*,
};

fn scenario() -> (World, State) {
    let (mut w, mut s) = publication::with_startup(
        c::Case::Baseline,
        Startup::MetalInventory,
        Policy::StockedAsksOnEqualProjection,
    )
    .unwrap();
    s.balances.insert((acceptance::WORKSHOP, ORE), 0);
    for agent in [acceptance::WORKSHOP, acceptance::TOWN] {
        let controller = w.agency.remove(&agent).unwrap();
        let mut agreement = controller.operating_agreement().unwrap().clone();
        agreement.forwards.insert(
            ORE,
            Options {
                prepayment: 2,
                delivery_months: 3,
            },
        );
        let signatures = agreement.members.clone();
        organization::install_discovering_initial(
            &mut w,
            &s,
            agreement,
            &signatures,
            controller.config,
        )
        .unwrap();
    }
    (w, s)
}

#[test]
fn bilateral_projection_reports_each_governors_actual_losses_without_publication() {
    let (w, s) = scenario();
    let before = (w.clone(), s.clone());
    let candidates = forwards::candidates(&w, &s).unwrap();
    assert!(!candidates.is_empty());
    for p in candidates {
        let result = assessment::assess(&w, &s, &p).unwrap();
        println!("{result:?}; mutual={}", result.mutually_beneficial());
        assert_eq!(result.comparisons.len(), 2);
        assert!(
            result
                .comparisons
                .values()
                .all(|c| c.admitted && c.delivered_on_time == 2)
        );
    }
    assert_eq!(before, (w, s));
}

#[test]
fn changing_cash_preference_does_not_turn_a_delivery_into_consent() {
    let (mut w, s) = scenario();
    use economics_compute_smoke::{
        agency::objectives::{Metric, Objective, Scope},
        minting::COIN,
    };
    w.agency
        .get_mut(&acceptance::WORKSHOP)
        .unwrap()
        .config
        .horizon = 4;
    let founder = w.agency[&acceptance::WORKSHOP]
        .operating_agreement()
        .unwrap()
        .founder;
    w.agency
        .get_mut(&acceptance::WORKSHOP)
        .unwrap()
        .config
        .preferences
        .insert(
            founder,
            vec![Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: COIN,
                    target: 1000,
                },
            }],
        );
    let p = forwards::candidates(&w, &s).unwrap().remove(0);
    let result = assessment::assess(&w, &s, &p).unwrap();
    assert_eq!(result.comparisons[&acceptance::WORKSHOP].governor, founder);
    assert_eq!(result.comparisons[&acceptance::WORKSHOP].proposed.len(), 1);
    assert!(!result.mutually_beneficial());
}
