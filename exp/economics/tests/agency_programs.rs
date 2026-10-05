use economics_compute_smoke::{
    agency,
    compute::Backend,
    minting::*,
    model::*,
    opportunities::{Action, STATE_TYPE},
    simulation::Simulation,
};

fn idle() -> (World, State) {
    let (mut w, s) = agency::scenario::minting().unwrap();
    let config = w.agency.remove(&ISSUER).unwrap().config;
    w.agency
        .insert(ISSUER, agency::Controller::discovering(config));
    w.scheduled_starts.clear();
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .month = 0;
    (w, s)
}

#[test]
fn constitutional_options_and_recipes_replace_supplied_programs_and_start_dates() {
    let (w, s) = idle();
    assert!(w.agency[&ISSUER].config.programs.is_empty());
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    reference.run_months(6).unwrap();
    cpu.run_months(6).unwrap();
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.world, reference.world);
    assert!(
        cpu.state
            .processes
            .values()
            .any(|p| p.definition == MINT && p.status == Status::Completed)
    );
    assert!(cpu.world.agency[&ISSUER].history.iter().any(|d| {
        d.accepted
            .as_ref()
            .is_some_and(|p| p.commands.contains(&agency::Command::StartProcess(MINT)))
    }));
}

#[test]
fn discovered_catalog_does_not_authorize_a_prohibited_process() {
    let (mut w, s) = idle();
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(STATE_TYPE, Action::Process(MINT)));
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    assert!(sim.world.scheduled_starts.is_empty());
    assert!(
        sim.world.agency[&ISSUER]
            .history
            .iter()
            .flat_map(|d| &d.alternatives)
            .any(|a| a.failure.as_deref() == Some("organization process is prohibited"))
    );
}
