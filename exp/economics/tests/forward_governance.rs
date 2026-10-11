use economics_compute_smoke::{
    acceptance::{
        TOWN, WORKSHOP,
        continuation::Case,
        financing::{self, Stocks},
    },
    agency::{
        self,
        forwards::{
            self,
            execution::{self, Status},
        },
        organization,
    },
    compute::Backend,
    minting::COIN,
    simulation::Simulation,
};

#[test]
fn consent_survives_scheduled_governor_rotation_before_activation() {
    let (mut w, s) = financing::with_stocks(Case::Baseline, true, Stocks::MetalOnly).unwrap();
    let c = w.agency.remove(&TOWN).unwrap();
    let mut agreement = c.operating_agreement().unwrap().clone();
    agreement.term_months = 2;
    let signatures = agreement.members.clone();
    organization::install_discovering_initial(&mut w, &s, agreement, &signatures, c.config)
        .unwrap();
    let p = forwards::candidates(&w, &s).unwrap().remove(0);
    execution::publish(&mut w, &s, &p).unwrap();
    let signatory = w.agency[&WORKSHOP].startup_forwards[0].governors[&TOWN];
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    assert_ne!(
        agency::governor(&sim.world, &sim.state, TOWN),
        Some(signatory)
    );
    sim.step().unwrap();
    let receipt = &sim.world.agency[&WORKSHOP].startup_forwards[0];
    assert_eq!(receipt.status, Status::Activated(3));
    assert_eq!(receipt.governors[&TOWN], signatory);
    assert!(sim.state.exchange.forwards.contains_key(&p.terms.id));
    Simulation::new(sim.world, sim.state, Backend::Reference).unwrap();
}

#[test]
fn failed_admission_uses_the_signed_one_shot_authority_even_after_cash_recovers() {
    let (mut w, mut s) = financing::with_stocks(Case::Baseline, true, Stocks::MetalOnly).unwrap();
    let p = forwards::candidates(&w, &s).unwrap().remove(0);
    execution::publish(&mut w, &s, &p).unwrap();
    s.balances.insert((WORKSHOP, COIN), 0);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    sim.step().unwrap();
    assert_eq!(
        sim.world.agency[&WORKSHOP].startup_forwards[0].status,
        Status::AdmissionRejected(3)
    );
    sim.state.balances.insert((WORKSHOP, COIN), 100);
    assert!(
        forwards::candidates(&sim.world, &sim.state)
            .unwrap()
            .is_empty()
    );
    assert!(!sim.state.exchange.forwards.contains_key(&p.terms.id));
    assert!(sim.world.prepaid_deliveries.contains(&p.terms));
}
