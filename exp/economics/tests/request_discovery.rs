use economics_compute_smoke::{
    acceptance::{Run, TOWN, WORKSHOP, continuation::Case, financing::Stocks, startup_publication},
    agency::{
        self, Command,
        work_requests::{Policy, Snapshot},
    },
    compute::Backend,
};
fn fixture(
    policy: Policy,
) -> (
    economics_compute_smoke::model::World,
    economics_compute_smoke::model::State,
) {
    let (mut w, s) = startup_publication::scenario(Case::Baseline, Stocks::OreOnly, true).unwrap();
    for actor in [WORKSHOP, TOWN] {
        w.agency.get_mut(&actor).unwrap().config.work_requests = policy;
    }
    (w, s)
}
#[test]
fn observing_requests_does_not_change_choices_or_execution() {
    let (w, s) = fixture(Policy::Static);
    let mut old = Run::new(w, s, Backend::Reference).unwrap();
    let (w, s) = fixture(Policy::Observe);
    let mut observed = Run::new(w, s, Backend::Reference).unwrap();
    old.through(6).unwrap();
    observed.through(6).unwrap();
    assert_eq!(old.sim.state, observed.sim.state);
    assert_eq!(old.sim.ledger, observed.sim.ledger);
    for actor in [WORKSHOP, TOWN] {
        for (a, b) in old.sim.world.agency[&actor]
            .history
            .iter()
            .zip(&observed.sim.world.agency[&actor].history)
        {
            assert_eq!(a.catalog, b.catalog);
            assert_eq!(a.chosen, b.chosen);
            assert!(b.work_requests.is_some());
        }
    }
}
#[test]
fn demand_candidates_use_frozen_evidence_without_rewriting_previous_choices() {
    let (w, s) = fixture(Policy::Respond);
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(6).unwrap();
    for actor in [WORKSHOP, TOWN] {
        for d in &run.sim.world.agency[&actor].history {
            assert_eq!(
                d.catalog,
                agency::discovery::programs_from_requests(
                    &run.sim.world,
                    actor,
                    d.work_requests.as_ref().unwrap()
                )
                .unwrap()
            );
        }
    }
    let saved = run.sim.world.agency[&TOWN].history.clone();
    run.sim
        .world
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
        .as_mut()
        .unwrap()
        .quotes
        .clear();
    agency::validate_history(&run.sim.world, &run.sim.state).unwrap();
    assert_eq!(saved, run.sim.world.agency[&TOWN].history);
    run.sim.world.agency.get_mut(&TOWN).unwrap().history[0]
        .work_requests
        .as_mut()
        .unwrap()
        .month += 1;
    assert!(agency::validate_history(&run.sim.world, &run.sim.state).is_err());
}
#[test]
fn absent_demand_removes_sale_work_but_preserves_own_goals_and_static_catalogs() {
    use economics_compute_smoke::{
        agency::objectives::{Metric, Objective, Scope},
        minting::{COIN, REFINE},
    };
    let (mut w, s) = fixture(Policy::Respond);
    let c = &mut w.agency.get_mut(&WORKSHOP).unwrap().config;
    c.objectives = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Reserve {
            resource: COIN,
            target: 10,
        },
    }];
    c.preferences.clear();
    let empty = Snapshot {
        month: s.month,
        actor: WORKSHOP,
        requests: vec![],
    };
    let has_work = |p: &std::collections::BTreeMap<u32, agency::Program>| {
        p.values().any(|p| {
            p.commands
                .iter()
                .any(|c| matches!(c, Command::WorkTargets(_)))
        })
    };
    let static_catalog = agency::discovery::programs(&w, WORKSHOP).unwrap();
    assert!(has_work(&static_catalog));
    assert!(!has_work(
        &agency::discovery::programs_from_requests(&w, WORKSHOP, &empty).unwrap()
    ));
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .config
        .objectives
        .push(Objective {
            scope: Scope::Organization,
            metric: Metric::ProcessShortfall {
                definition: REFINE,
                target: 1,
            },
        });
    assert!(
        agency::discovery::programs_from_requests(&w, WORKSHOP, &empty)
            .unwrap()
            .values()
            .any(|p| p
                .commands
                .iter()
                .any(|c| matches!(c, Command::StartProcess(id) if *id == REFINE)))
    );
}
