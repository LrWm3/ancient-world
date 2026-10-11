use economics_compute_smoke::{
    acceptance::{self, continuation as c, financing},
    agency::forwards::{self, assessment, execution},
    compute::Backend,
    minting::{COIN, ORE},
    model::*,
    simulation::Simulation,
};
fn scenario() -> (World, State) {
    let (w, mut s) = financing::scenario(c::Case::Baseline).unwrap();
    s.balances.insert((acceptance::WORKSHOP, ORE), 0);
    (w, s)
}

#[test]
fn stored_consent_cannot_be_reused_or_forge_activation() {
    let (mut w, s) = scenario();
    let p = forwards::candidates(&w, &s).unwrap().remove(0);
    execution::publish(&mut w, &s, &p).unwrap();
    let mut duplicate = w.clone();
    let mut agreement = duplicate.agency[&acceptance::WORKSHOP].startup_forwards[0].clone();
    agreement.proposal.terms.id += 1;
    duplicate
        .prepaid_deliveries
        .push(agreement.proposal.terms.clone());
    duplicate
        .agency
        .get_mut(&acceptance::WORKSHOP)
        .unwrap()
        .startup_forwards
        .push(agreement);
    assert!(Simulation::new(duplicate, s.clone(), Backend::Reference).is_err());
    let mut forged = w.clone();
    forged
        .agency
        .get_mut(&acceptance::WORKSHOP)
        .unwrap()
        .startup_forwards[0]
        .status = execution::Status::Activated(1);
    assert!(Simulation::new(forged, s.clone(), Backend::Reference).is_err());
    let mut changed = w.clone();
    changed
        .agency
        .get_mut(&acceptance::WORKSHOP)
        .unwrap()
        .startup_forwards[0]
        .programs
        .clear();
    assert!(Simulation::new(changed, s, Backend::Reference).is_err());
}

#[test]
fn failed_production_and_cash_refusal_cannot_publish() {
    for no_production in [false, true] {
        let (mut w, s) = scenario();
        if no_production {
            w.definitions
                .iter_mut()
                .find(|d| d.id == acceptance::MUNICIPAL_WORK)
                .unwrap()
                .enabled = false;
        } else {
            w.agency
                .get_mut(&acceptance::WORKSHOP)
                .unwrap()
                .config
                .horizon = 4;
        }
        let p = forwards::candidates(&w, &s).unwrap().remove(0);
        let result = assessment::assess(&w, &s, &p).unwrap();
        assert!(!result.mutually_beneficial());
        if no_production {
            assert!(
                result
                    .comparisons
                    .values()
                    .all(|c| c.delivered_on_time == 0)
            );
        }
        let before = w.clone();
        assert!(execution::publish(&mut w, &s, &p).is_err());
        assert_eq!(w, before);
    }
}

#[test]
fn publication_rechecks_funding_law_and_boundary() {
    let (w, s) = scenario();
    let p = forwards::candidates(&w, &s).unwrap().remove(0);
    let mut poor = s.clone();
    poor.balances.insert((acceptance::WORKSHOP, COIN), 1);
    let mut midmonth = s.clone();
    midmonth.phase = Phase::Acquire;
    for state in [poor, midmonth] {
        let mut candidate = w.clone();
        assert!(execution::publish(&mut candidate, &state, &p).is_err());
        assert_eq!(candidate, w);
    }
    let mut prohibited = w.clone();
    prohibited
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(
            acceptance::TOWN_TYPE,
            economics_compute_smoke::opportunities::Action::StockTrade,
        ));
    let before = prohibited.clone();
    assert!(execution::publish(&mut prohibited, &s, &p).is_err());
    assert_eq!(prohibited, before);
}

#[test]
fn reconstruction_rejects_a_pending_program_after_its_activation_open() {
    let (mut w, s) = scenario();
    let p = forwards::candidates(&w, &s).unwrap().remove(0);
    execution::publish(&mut w, &s, &p).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.month < 3 || sim.state.phase == Phase::Open {
        sim.step().unwrap();
    }
    sim.world
        .agency
        .get_mut(&acceptance::WORKSHOP)
        .unwrap()
        .startup_forwards[0]
        .status = execution::Status::Pending;
    assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
}

#[test]
fn only_activated_signed_programs_count_as_authorized_operating_choices() {
    let (mut w, s) = scenario();
    let p = forwards::candidates(&w, &s).unwrap().remove(0);
    execution::publish(&mut w, &s, &p).unwrap();
    for agent in [acceptance::WORKSHOP, acceptance::TOWN] {
        assert!(
            economics_compute_smoke::agency::accepted_programs(&w, agent)
                .next()
                .is_none()
        );
    }
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.month < 3 || sim.state.phase == Phase::Open {
        sim.step().unwrap();
    }
    for agent in [acceptance::WORKSHOP, acceptance::TOWN] {
        let program = &sim.world.agency[&acceptance::WORKSHOP].startup_forwards[0].programs[&agent];
        assert!(
            economics_compute_smoke::agency::accepted_programs(&sim.world, agent)
                .any(|p| p == program)
        );
    }
}
