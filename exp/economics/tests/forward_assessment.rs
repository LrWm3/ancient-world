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

#[test]
fn published_work_waits_for_admission_and_rejection_leaves_no_work() {
    use economics_compute_smoke::{
        agency::forwards::execution::{self, Status},
        compute::Backend,
        minting::COIN,
        simulation::Simulation,
    };
    for funded in [true, false] {
        let (mut w, mut s) = scenario();
        let p = forwards::candidates(&w, &s).unwrap().remove(0);
        execution::publish(&mut w, &s, &p).unwrap();
        assert!(
            w.activities
                .orders
                .iter()
                .all(|o| ![acceptance::WORKSHOP, acceptance::TOWN].contains(&o.agent))
        );
        if !funded {
            s.balances.insert((acceptance::WORKSHOP, COIN), 0);
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        while sim.state.month < 3 {
            sim.step().unwrap();
        }
        assert_eq!(
            sim.world.agency[&acceptance::WORKSHOP].startup_forwards[0].status,
            Status::Pending
        );
        assert_eq!(
            sim.state.exchange.forwards.contains_key(&p.terms.id),
            funded
        );
        sim.step().unwrap();
        assert_eq!(
            sim.world.agency[&acceptance::WORKSHOP].startup_forwards[0].status,
            if funded {
                Status::Activated(3)
            } else {
                Status::AdmissionRejected(3)
            }
        );
        assert_eq!(
            sim.world
                .activities
                .orders
                .iter()
                .any(|o| o.agent == acceptance::TOWN),
            funded
        );
        for agent in [acceptance::WORKSHOP, acceptance::TOWN] {
            let expected =
                &sim.world.agency[&acceptance::WORKSHOP].startup_forwards[0].programs[&agent];
            assert_eq!(
                economics_compute_smoke::agency::accepted_programs(&sim.world, agent)
                    .any(|p| p == expected),
                funded
            );
        }
    }
}

#[test]
fn publication_seals_both_mandates_and_respects_tenure_after_activation() {
    use economics_compute_smoke::{
        agency::forwards::execution, compute::Backend, simulation::Simulation,
    };
    let (mut w, s) = scenario();
    for agent in [acceptance::WORKSHOP, acceptance::TOWN] {
        w.agency.get_mut(&agent).unwrap().config.minimum_tenure = 6;
    }
    let p = forwards::candidates(&w, &s).unwrap().remove(0);
    execution::publish(&mut w, &s, &p).unwrap();
    for agent in [acceptance::WORKSHOP, acceptance::TOWN] {
        let mut changed = w.clone();
        changed.agency.get_mut(&agent).unwrap().config.horizon = 5;
        assert!(Simulation::new(changed, s.clone(), Backend::Reference).is_err());
    }
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.month < 5 {
        sim.step().unwrap();
    }
    for agent in [acceptance::WORKSHOP, acceptance::TOWN] {
        assert!(
            sim.world.agency[&agent]
                .history
                .iter()
                .filter(|d| d.month >= 3)
                .all(|d| d.accepted.is_none() && d.reason == "retained within minimum tenure")
        );
    }
}
