use economics_compute_smoke::{
    acceptance::{SMITH, WORKSHOP, continuation::Case, financing},
    agency::{
        completions::Completion,
        flow,
        objectives::{self, Metric, Objective, Scope},
        path::Path,
    },
    minting::REFINE,
    model::*,
};

fn goal(scope: Scope) -> Objective {
    Objective {
        scope,
        metric: Metric::ProcessShortfall {
            definition: REFINE,
            target: 1,
        },
    }
}

#[test]
fn monthly_shortfall_filters_operators_and_definitions_without_catch_up_credit() {
    let (w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let goals = [goal(Scope::Agents([WORKSHOP, SMITH].into()))];
    objectives::validate(&w, &goals).unwrap();
    let mut path = Path::new(1);
    for month in 2..=4 {
        s.month = month;
        path.record(&w, &s).unwrap();
    }
    assert!(objectives::measure_projection(&w, &s, &[], WORKSHOP, &goals, &path).is_err());
    path.completions = Some(vec![
        Completion {
            month: 2,
            process: 1,
            operator: WORKSHOP,
            definition: REFINE,
        },
        Completion {
            month: 2,
            process: 2,
            operator: WORKSHOP,
            definition: REFINE,
        },
        Completion {
            month: 2,
            process: 1,
            operator: WORKSHOP,
            definition: REFINE,
        },
        Completion {
            month: 3,
            process: 3,
            operator: SMITH,
            definition: REFINE,
        },
        Completion {
            month: 1,
            process: 4,
            operator: WORKSHOP,
            definition: 999,
        },
        Completion {
            month: 1,
            process: 5,
            operator: 999,
            definition: REFINE,
        },
        Completion {
            month: 0,
            process: 6,
            operator: WORKSHOP,
            definition: REFINE,
        },
    ]);
    assert_eq!(
        objectives::measure_projection(&w, &s, &[], WORKSHOP, &goals, &path).unwrap(),
        [4]
    );
    path.completions = Some(vec![]);
    assert_eq!(
        objectives::measure_projection(&w, &s, &[], WORKSHOP, &goals, &path).unwrap(),
        [6]
    );
    path.closes[1].month = 1;
    assert!(objectives::measure_projection(&w, &s, &[], WORKSHOP, &goals, &path).is_err());
    assert!(objectives::measure(&w, &s, &[], WORKSHOP, &goals).is_err());
}

#[test]
fn live_observation_requires_previous_close_and_counts_only_that_month() {
    let (w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let goals = [goal(Scope::Organization)];
    assert_eq!(flow::observe(&w, &s, &[], WORKSHOP, &goals).unwrap(), [0]);
    let mut close = Batch::empty(&s);
    close.phase = Phase::Close;
    s.month = 2;
    assert!(flow::observe(&w, &s, &[], WORKSHOP, &goals).is_err());
    assert_eq!(
        flow::observe(&w, &s, &[close.clone()], WORKSHOP, &goals).unwrap(),
        [1]
    );
    close.month = 0;
    assert!(flow::observe(&w, &s, &[close], WORKSHOP, &goals).is_err());
}

#[test]
fn validates_definition_and_freezes_member_beneficiaries() {
    let (w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let goals = [goal(Scope::Members)];
    let frozen = objectives::freeze(&w, &s, WORKSHOP, &goals);
    assert_eq!(
        frozen[0].scope,
        Scope::Agents(objectives::members(&w, &s, WORKSHOP))
    );
    let mut path = Path::new(1);
    s.month = 2;
    path.record(&w, &s).unwrap();
    assert!(path.record_completions(&[]).is_err());
    let mut close = Batch::empty(&s);
    close.phase = Phase::Close;
    close.month = 1;
    path.record_completions(&[close]).unwrap();
    assert!(objectives::measure_projection(&w, &s, &[], WORKSHOP, &goals, &path).is_err());
    assert!(objectives::measure_projection(&w, &s, &[], WORKSHOP, &frozen, &path).is_ok());
    s.month = 3;
    assert!(path.record(&w, &s).is_err());
    let invalid = [Objective {
        scope: Scope::Organization,
        metric: Metric::ProcessShortfall {
            definition: u32::MAX,
            target: 1,
        },
    }];
    assert!(objectives::validate(&w, &invalid).is_err());
}

#[test]
fn missing_middle_month_cannot_be_finalized_as_zero_work() {
    let (w, mut s) = financing::scenario(Case::Baseline).unwrap();
    let mut path = Path::new(1);
    let mut ledger = vec![];
    for month in 2..=4 {
        s.month = month;
        path.record(&w, &s).unwrap();
        let mut close = Batch::empty(&s);
        close.month = month - 1;
        close.phase = Phase::Close;
        ledger.push(close);
    }
    assert!(
        path.record_completions(&[ledger[0].clone(), ledger[2].clone()])
            .is_err()
    );
    assert!(path.completions.is_none());
    path.record_completions(&ledger).unwrap();
    assert_eq!(path.completions, Some(vec![]));
}

#[test]
fn process_only_discovery_still_requires_signed_operating_authority() {
    use economics_compute_smoke::agency::{self, Command};
    for (allowed, personal) in [(true, false), (false, false), (true, true), (false, true)] {
        let (mut w, s) = financing::scenario(Case::Baseline).unwrap();
        let old = w.agency.remove(&WORKSHOP).unwrap();
        let mut agreement = old.operating_agreement().unwrap().clone();
        agreement.forwards.clear();
        agreement.stock_markets.clear();
        agreement.processes = if allowed {
            [REFINE].into()
        } else {
            Default::default()
        };
        let mut config = old.config;
        config.objectives = vec![goal(Scope::Organization)];
        config.preferences.clear();
        if personal {
            config.preferences.insert(SMITH, config.objectives.clone());
            config.objectives = vec![Objective {
                scope: Scope::Organization,
                metric: Metric::Deaths,
            }];
        }
        config.programs.clear();
        agency::organization::install_discovering_initial(
            &mut w,
            &s,
            agreement.clone(),
            &agreement.members,
            config,
        )
        .unwrap();
        let programs = agency::discovery::programs(&w, WORKSHOP).unwrap();
        let has_process = programs
            .values()
            .flat_map(|p| &p.commands)
            .any(|c| matches!(c, Command::StartProcess(id) if *id == REFINE));
        assert_eq!(has_process, allowed, "personal={personal}");
        let mut reversed = w.clone();
        reversed.definitions.reverse();
        reversed.agents.reverse();
        assert_eq!(
            agency::discovery::programs(&reversed, WORKSHOP).unwrap(),
            programs
        );
    }
}

#[test]
fn personal_stock_goals_discover_recipes_and_refuse_oversized_catalogs() {
    use economics_compute_smoke::agency::{self, Command};
    let (mut w, s) = financing::scenario(Case::Baseline).unwrap();
    let old = w.agency.remove(&WORKSHOP).unwrap();
    let mut agreement = old.operating_agreement().unwrap().clone();
    agreement.forwards.clear();
    agreement.stock_markets.clear();
    let mut config = old.config;
    config.objectives = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Deaths,
    }];
    config.preferences = [(
        SMITH,
        vec![Objective {
            scope: Scope::Organization,
            metric: Metric::Reserve {
                resource: economics_compute_smoke::minting::METAL,
                target: 2,
            },
        }],
    )]
    .into();
    config.programs.clear();
    agency::organization::install_discovering_initial(
        &mut w,
        &s,
        agreement.clone(),
        &agreement.members,
        config,
    )
    .unwrap();
    let programs = agency::discovery::programs(&w, WORKSHOP).unwrap();
    assert!(
        programs
            .values()
            .flat_map(|p| &p.commands)
            .any(|c| matches!(c, Command::StartProcess(id) if *id == REFINE))
    );
    let old = w.agency.remove(&WORKSHOP).unwrap();
    for id in 1000..1017 {
        let mut recipe = w.definition(REFINE).clone();
        recipe.id = id;
        w.definitions.push(recipe);
        agreement.processes.insert(id);
    }
    let error = agency::organization::install_discovering_initial(
        &mut w,
        &s,
        agreement.clone(),
        &agreement.members,
        old.config,
    )
    .unwrap_err();
    assert!(error.contains("bounded search"));
    assert!(
        !w.agency.contains_key(&WORKSHOP),
        "failed installation is atomic"
    );
}

#[test]
fn multimonth_completion_is_scored_when_finished_not_when_started() {
    use economics_compute_smoke::{
        compute::Backend, forecast::ForecastContext, scenario, simulation::Simulation,
    };
    let (w, s) = scenario::repeated();
    let mut live = Simulation::new(w, s, Backend::Reference).unwrap();
    live.run_months(2).unwrap();
    let growing = live
        .state
        .processes
        .values()
        .find(|p| p.definition == scenario::GROW && p.status == Status::Active)
        .unwrap()
        .id;
    let (w, s) = ForecastContext::new(&live.world, &live.state).into_parts();
    let mut branch = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut path = Path::new(branch.state.month);
    for _ in 0..4 {
        branch.run_months(1).unwrap();
        path.record(&branch.world, &branch.state).unwrap();
    }
    path.record_completions(&branch.ledger).unwrap();
    let harvests: Vec<_> = path
        .completions
        .as_ref()
        .unwrap()
        .iter()
        .filter(|c| c.definition == scenario::GROW)
        .collect();
    assert_eq!(harvests.len(), 1);
    assert_eq!((harvests[0].process, harvests[0].month), (growing, 6));
    let goals = [Objective {
        scope: Scope::Organization,
        metric: Metric::ProcessShortfall {
            definition: scenario::GROW,
            target: 1,
        },
    }];
    assert_eq!(
        objectives::measure_projection(
            &branch.world,
            &branch.state,
            &branch.reports,
            scenario::PERSON,
            &goals,
            &path
        )
        .unwrap(),
        [3]
    );
    assert_eq!(live.state.month, 3);
    assert_eq!(live.state.processes[&growing].status, Status::Active);
}

#[test]
fn production_shortfall_triggers_early_review_from_previous_close() {
    use economics_compute_smoke::{
        agency, compute::Backend, minting::MINT, simulation::Simulation,
    };
    for emergency in [false, true] {
        let (mut w, s) = agency::scenario::minting().unwrap();
        let c = &mut w
            .agency
            .get_mut(&economics_compute_smoke::minting::ISSUER)
            .unwrap()
            .config;
        c.objectives.insert(
            0,
            Objective {
                scope: Scope::Organization,
                metric: Metric::ProcessShortfall {
                    definition: MINT,
                    target: 1,
                },
            },
        );
        c.preferences.clear();
        c.review_every = 12;
        c.emergency = emergency.then_some((0, 0));
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(1).unwrap();
        let mut restored =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
        restored.ledger = sim.ledger.clone();
        restored.reports = sim.reports.clone();
        sim.step().unwrap();
        restored.step().unwrap();
        assert_eq!(sim.world, restored.world);
        assert_eq!(sim.state, restored.state);
        let d = sim.world.agency[&economics_compute_smoke::minting::ISSUER]
            .history
            .last()
            .unwrap();
        assert_eq!(d.observed[0], 1);
        assert_eq!(!d.alternatives.is_empty(), emergency);
    }
}
