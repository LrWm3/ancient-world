use economics_compute_smoke::{
    acceptance::{
        TOWN, WORKSHOP,
        maintained_workshop::{self as m, Case},
    },
    activities::Target,
    agency::{
        self, Command,
        objectives::{self, Metric, Objective, Scope},
    },
    minting::REFINE,
};

#[test]
fn condition_reserve_uses_owned_live_equipment_without_imputing_output_or_cash() {
    let (w, mut s) = m::scenario(Case::Discovered).unwrap();
    let goal = Objective {
        scope: Scope::Organization,
        metric: Metric::DurableReserve {
            kind: m::MACHINE_KIND,
            target: 4,
        },
    };
    let loss =
        |s: &_| objectives::measure(&w, s, &[], WORKSHOP, std::slice::from_ref(&goal)).unwrap()[0];
    assert_eq!(loss(&s), 0);
    s.equipment.get_mut(&m::MACHINE).unwrap().remaining_uses = 1;
    assert_eq!(loss(&s), 3);
    let mut foreign = s.equipment[&m::MACHINE].clone();
    foreign.id += 1;
    foreign.owner = TOWN;
    foreign.remaining_uses = 4;
    s.equipment.insert(foreign.id, foreign);
    assert_eq!(loss(&s), 3);
    s.equipment.get_mut(&m::MACHINE).unwrap().remaining_uses = 0;
    assert_eq!(loss(&s), 4);
    s.equipment.get_mut(&m::MACHINE).unwrap().remaining_uses = 4;
    s.equipment.get_mut(&m::MACHINE).unwrap().last_used_month = Some(s.month);
    assert_eq!(loss(&s), 0, "condition is not same-month availability");
    let bad = Objective {
        scope: Scope::Organization,
        metric: Metric::DurableReserve {
            kind: u32::MAX,
            target: 1,
        },
    };
    assert!(objectives::validate(&w, &[bad]).is_err());
}

#[test]
fn maintenance_discovery_keeps_productive_work_and_stable_bounded_catalog() {
    let (mut w, _) = m::scenario(Case::Discovered).unwrap();
    let catalog = agency::discovery::programs(&w, WORKSHOP).unwrap();
    let bundles: Vec<_> = catalog
        .values()
        .flat_map(|p| &p.commands)
        .filter_map(|c| {
            if let Command::WorkTargets(orders) = c {
                Some(orders)
            } else {
                None
            }
        })
        .collect();
    assert!(
        bundles
            .iter()
            .any(|orders| orders.len() == 1 && orders[0].definition == REFINE)
    );
    assert!(bundles.iter().any(|orders| orders.len() == 2
        && orders[0].definition == m::REPAIR
        && orders[0].target
            == Target::Maintain {
                kind: m::MACHINE_KIND,
                below: 1
            }
        && orders[0].priority < orders[1].priority
        && orders[1].definition == REFINE));
    assert!(catalog.len() <= 16);
    w.definitions.reverse();
    assert_eq!(catalog, agency::discovery::programs(&w, WORKSHOP).unwrap());
    let old = w.agency.remove(&WORKSHOP).unwrap();
    let mut agreement = old.operating_agreement().unwrap().clone();
    agreement.processes.remove(&m::REPAIR);
    let (_, s) = m::scenario(Case::Discovered).unwrap();
    agency::organization::install_discovering_initial(
        &mut w,
        &s,
        agreement.clone(),
        &agreement.members,
        old.config,
    )
    .unwrap();
    assert!(agency::discovery::programs(&w, WORKSHOP).unwrap().values().flat_map(|p| &p.commands)
        .all(|c| !matches!(c, Command::WorkTargets(o) if o.iter().any(|o| o.definition == m::REPAIR))));
}

#[test]
fn dated_upstream_work_retains_a_repair_alternative_without_current_sales_interest() {
    let (w, s) = m::scenario(Case::Discovered).unwrap();
    let requests = agency::work_requests::Snapshot {
        month: s.month,
        actor: WORKSHOP,
        requests: vec![],
    };
    let catalog = agency::discovery::programs_from_requests(&w, WORKSHOP, &requests).unwrap();
    assert!(catalog.values().any(|p|
        p.commands.contains(&Command::StartProcess(REFINE)) && p.commands.iter().any(|c|
            matches!(c, Command::WorkTargets(orders) if orders.len() == 1 && orders[0].definition == m::REPAIR))));
}

#[test]
fn upkeep_preserves_dated_targets_without_priority_drift_or_live_order_dependence() {
    use economics_compute_smoke::{activities::WorkOrder, minting::METAL, model::Amount};
    let (mut w, _) = m::scenario(Case::Discovered).unwrap();
    let order = WorkOrder {
        agent: WORKSHOP,
        definition: REFINE,
        priority: 1,
        target: Target::Stock(Amount::new(METAL, 2)),
    };
    // A compact dated-history fixture. The integration test below authenticates
    // actual decisions through the ordinary projection and acceptance path.
    let mut previous = agency::Program {
        name: "accepted target".into(),
        commands: vec![Command::WorkTargets(vec![order.clone()])],
    };
    let original = agency::work_requests::Snapshot {
        month: 1,
        actor: WORKSHOP,
        requests: vec![],
    };
    let founding = agency::discovery::programs_from_requests(&w, WORKSHOP, &original).unwrap();
    for month in 1..=3 {
        w.agency
            .get_mut(&WORKSHOP)
            .unwrap()
            .history
            .push(agency::Decision {
                month,
                effective_month: month + 1,
                authorized_by: None,
                chosen: Some(0),
                accepted: Some(previous.clone()),
                catalog: [(0, previous)].into(),
                alternatives: vec![],
                peer_expectations: None,
                sale_opportunities: vec![],
                work_requests: None,
                work_routes: vec![],
                observed: vec![],
                reason: String::new(),
                ballots: vec![],
            });
        let snapshot = agency::work_requests::Snapshot {
            month: month + 1,
            actor: WORKSHOP,
            requests: vec![],
        };
        let catalog = agency::discovery::programs_from_requests(&w, WORKSHOP, &snapshot).unwrap();
        previous = catalog.values().find(|p| p.commands.iter().any(|c|
            matches!(c, Command::WorkTargets(o) if o.len() == 2 && o[0].definition == m::REPAIR && o[1] == order)
        )).expect("same target and priority must survive repeated upkeep selection").clone();
        w.activities.orders = vec![];
        assert_eq!(
            catalog,
            agency::discovery::programs_from_requests(&w, WORKSHOP, &snapshot).unwrap()
        );
        assert_eq!(
            founding,
            agency::discovery::programs_from_requests(&w, WORKSHOP, &original).unwrap()
        );
    }
    let clear = &mut w
        .agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .history
        .last_mut()
        .unwrap()
        .accepted
        .as_mut()
        .unwrap()
        .commands;
    *clear = vec![Command::WorkTargets(vec![])];
    let after_clear = agency::work_requests::Snapshot {
        month: 4,
        actor: WORKSHOP,
        requests: vec![],
    };
    assert_eq!(
        founding,
        agency::discovery::programs_from_requests(&w, WORKSHOP, &after_clear).unwrap()
    );
}

#[test]
fn regenerating_a_real_decision_ignores_later_live_work_orders() {
    use economics_compute_smoke::{compute::Backend, simulation::Simulation};
    let (w, s) = m::scenario(Case::Discovered).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    let first = sim.world.agency[&WORKSHOP].history[0].clone();
    // Later execution can change live orders. Such changes are not evidence
    // of the targets accepted at this earlier request boundary.
    sim.world
        .activities
        .orders
        .push(economics_compute_smoke::activities::WorkOrder {
            agent: WORKSHOP,
            definition: REFINE,
            priority: 0,
            target: Target::Stock(economics_compute_smoke::model::Amount::new(
                economics_compute_smoke::minting::METAL,
                2,
            )),
        });
    assert_eq!(
        first.catalog,
        agency::discovery::programs_from_requests(
            &sim.world,
            WORKSHOP,
            first.work_requests.as_ref().unwrap()
        )
        .unwrap()
    );
    agency::validate_history(&sim.world, &sim.state).unwrap();
    Simulation::new(sim.world, sim.state, Backend::Reference).unwrap();
}

#[test]
fn admitted_forward_targets_are_dated_and_override_same_month_ordinary_choices() {
    use economics_compute_smoke::{
        activities::WorkOrder, agency::forwards, minting::METAL, model::Amount,
    };
    let (mut w, _) = m::scenario(Case::Discovered).unwrap();
    let order = WorkOrder {
        agent: WORKSHOP,
        definition: REFINE,
        priority: 0,
        target: Target::Stock(Amount::new(METAL, 2)),
    };
    let program = agency::Program {
        name: "forward operating target".into(),
        commands: vec![Command::WorkTargets(vec![order.clone()])],
    };
    // Synthetic history isolates replacement precedence; forward authority and
    // admission are separately validated by ordinary contract execution tests.
    let mut terms = w.prepaid_deliveries[0].clone();
    terms.buyer = WORKSHOP;
    terms.seller = TOWN;
    let agreement = forwards::execution::Agreement {
        month: 1,
        proposal: forwards::Proposal {
            market: 0,
            terms,
            buyer_program: 0,
            seller_program: 0,
        },
        governors: Default::default(),
        programs: [(WORKSHOP, program)].into(),
        status: forwards::execution::Status::Pending,
    };
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .startup_forwards
        .push(agreement);
    let snapshot = |month| agency::work_requests::Snapshot {
        month,
        actor: WORKSHOP,
        requests: vec![],
    };
    let no_target = agency::discovery::programs_from_requests(&w, WORKSHOP, &snapshot(1)).unwrap();
    assert_eq!(
        no_target,
        agency::discovery::programs_from_requests(&w, WORKSHOP, &snapshot(3)).unwrap()
    );
    w.agency.get_mut(&WORKSHOP).unwrap().startup_forwards[0].status =
        forwards::execution::Status::Activated(3);
    assert_eq!(
        no_target,
        agency::discovery::programs_from_requests(&w, WORKSHOP, &snapshot(2)).unwrap()
    );
    let check_target = |w: &_, month| {
        let catalog =
            agency::discovery::programs_from_requests(w, WORKSHOP, &snapshot(month)).unwrap();
        assert!(
            catalog
                .values()
                .any(|p| p.commands.iter().any(|c| matches!(c,
            Command::WorkTargets(o) if o.len() == 2 && o[0].definition == m::REPAIR
            && o[1].definition == REFINE && o[1].target == order.target)))
        );
    };
    check_target(&w, 3);
    for owner in [WORKSHOP, TOWN] {
        let mut later = w.agency[&WORKSHOP].startup_forwards[0].clone();
        let mut replacement = order.clone();
        replacement.target = Target::Stock(Amount::new(METAL, 3));
        later.programs.get_mut(&WORKSHOP).unwrap().commands =
            vec![Command::WorkTargets(vec![replacement.clone()])];
        w.agency
            .get_mut(&owner)
            .unwrap()
            .startup_forwards
            .push(later);
        let catalog =
            agency::discovery::programs_from_requests(&w, WORKSHOP, &snapshot(3)).unwrap();
        assert!(
            catalog
                .values()
                .any(|p| p.commands.iter().any(|c| matches!(c,
            Command::WorkTargets(o) if o.len() == 2 && o[1].target == replacement.target)))
        );
        w.agency.get_mut(&owner).unwrap().startup_forwards.pop();
    }
    let clear = agency::Program {
        name: "clear work".into(),
        commands: vec![Command::WorkTargets(vec![])],
    };
    w.agency
        .get_mut(&WORKSHOP)
        .unwrap()
        .history
        .push(agency::Decision {
            month: 2,
            effective_month: 3,
            authorized_by: None,
            chosen: Some(0),
            accepted: Some(clear.clone()),
            catalog: [(0, clear)].into(),
            alternatives: vec![],
            peer_expectations: None,
            sale_opportunities: vec![],
            work_requests: None,
            work_routes: vec![],
            observed: vec![],
            reason: String::new(),
            ballots: vec![],
        });
    check_target(&w, 3);
    let d = &mut w.agency.get_mut(&WORKSHOP).unwrap().history[0];
    d.month = 3;
    d.effective_month = 4;
    assert_eq!(
        no_target,
        agency::discovery::programs_from_requests(&w, WORKSHOP, &snapshot(4)).unwrap()
    );
    w.agency.get_mut(&WORKSHOP).unwrap().startup_forwards[0].status =
        forwards::execution::Status::AdmissionRejected(3);
    assert_eq!(
        no_target,
        agency::discovery::programs_from_requests(&w, WORKSHOP, &snapshot(3)).unwrap()
    );
}
