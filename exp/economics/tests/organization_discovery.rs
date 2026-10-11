use economics_compute_smoke::{
    acceptance::{WORKSHOP, continuation as c},
    agency::{self, Command, organization},
    compute::Backend,
    minting::{METAL, REFINE},
    model::*,
    simulation::Simulation,
};

fn fixture() -> (World, State) {
    let (mut w, s) = c::spot_scenario(c::Case::Baseline).unwrap();
    let previous = w.agency.remove(&WORKSHOP).unwrap();
    let a = previous.operating_agreement().unwrap().clone();
    let mut config = previous.config;
    config.programs.clear();
    organization::install_discovering_initial(&mut w, &s, a.clone(), &a.members, config).unwrap();
    (w, s)
}

#[test]
fn discovers_persistent_own_work_from_signed_recipes_and_reserve_objectives() {
    let (mut w, s) = fixture();
    let catalog = agency::discovery::programs(&w, WORKSHOP).unwrap();
    assert_eq!(catalog.len(), 1);
    let Command::WorkTargets(orders) = &catalog[&0].commands[0] else {
        panic!()
    };
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].agent, WORKSHOP);
    assert_eq!(orders[0].definition, REFINE);
    assert_eq!(
        orders[0].target,
        economics_compute_smoke::activities::Target::Stock(Amount::new(METAL, 4))
    );
    w.definitions.reverse();
    w.agents.reverse();
    assert_eq!(agency::discovery::programs(&w, WORKSHOP).unwrap(), catalog);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(3).unwrap();
    assert!(sim.world.agency[&WORKSHOP].config.programs.is_empty());
    assert_eq!(sim.world.agency[&WORKSHOP].history[0].catalog, catalog);
    assert!(
        sim.world
            .activities
            .orders
            .iter()
            .any(|o| o.agent == WORKSHOP && o.definition == REFINE)
    );
    assert_eq!(
        agency::discovery::programs(&sim.world, WORKSHOP).unwrap(),
        catalog,
        "holdings must not rewrite history"
    );
}

#[test]
fn discovery_cannot_gain_process_authority_from_an_objective() {
    let (mut w, s) = fixture();
    let previous = w.agency.remove(&WORKSHOP).unwrap();
    let mut a = previous.operating_agreement().unwrap().clone();
    a.processes.clear();
    organization::install_discovering_initial(&mut w, &s, a.clone(), &a.members, previous.config)
        .unwrap();
    assert!(
        agency::discovery::programs(&w, WORKSHOP)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn discovery_respects_declared_objective_priority_for_shared_recipe_outputs() {
    use agency::objectives::{Metric, Objective, Scope};
    let (mut w, _) = fixture();
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .config
        .objectives
        .insert(
            0,
            Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: METAL,
                    target: 6,
                },
            },
        );
    let catalog = agency::discovery::programs(&w, WORKSHOP).unwrap();
    let Command::WorkTargets(orders) = &catalog[&0].commands[0] else {
        panic!()
    };
    assert_eq!(
        orders[0].target,
        economics_compute_smoke::activities::Target::Stock(Amount::new(METAL, 6))
    );
}
