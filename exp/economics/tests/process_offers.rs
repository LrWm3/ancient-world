use economics_compute_smoke::{
    agreements::{self, Consequence, Counterparty},
    compute::Backend,
    membership,
    model::*,
    offers::{self, Id, Request, Terms},
    scenario::*,
    simulation::Simulation,
};

fn sim(backend: Backend) -> Simulation {
    let (w, s) = membership::scenario().unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn acquire(s: &mut Simulation) {
    while s.state.phase != Phase::Acquire {
        s.step().unwrap();
    }
}
fn bundle() -> Vec<Request> {
    vec![
        Request::new(Id::Membership(1), PERSON),
        Request::new(Id::Land(1), PERSON),
        Request::new(Id::Process(GROW), PERSON),
    ]
}

#[test]
fn common_discovery_and_atomic_acceptance_reserve_a_dated_farming_agreement() {
    let mut s = sim(Backend::CubeCpu);
    acquire(&mut s);
    let visible = offers::discover(&s.world, &s.state, PERSON);
    assert!(visible.iter().any(|o| o.id == Id::Membership(1)));
    assert!(visible.iter().any(|o| o.id == Id::Land(1)));
    let farm = visible.iter().find(|o| o.id == Id::Process(GROW)).unwrap();
    let Terms::Production(terms) = &farm.terms else {
        panic!("production terms")
    };
    assert_eq!(terms.duration(), 6);
    assert_eq!(terms.asset_kind, Some(1));
    assert_eq!(
        terms
            .schedule(1)
            .unwrap()
            .iter()
            .map(|(_, costs)| costs
                .iter()
                .filter(|a| a.resource == LABOR)
                .map(|a| a.quantity)
                .sum::<i32>())
            .collect::<Vec<_>>(),
        [2, 1, 1, 1, 1, 2]
    );
    assert!(
        terms.schedule(1).unwrap()[0]
            .1
            .contains(&Amount::new(SEED, 1))
    );
    assert!(terms.outputs.contains(&Amount::new(SEED, 1)));
    assert_eq!(terms.on_unfulfilled, Consequence::AbortWithoutRefund);
    let opening = s.state.clone();
    assert!(offers::feasible(&s, &[Request::new(Id::Process(GROW), PERSON)]).is_err());
    let mut request = bundle();
    request.push(Request::new(Id::Process(PREPARE_FUEL), PERSON));
    assert!(offers::feasible(&s, &request).is_ok());
    assert_eq!(s.state, opening); // Feasibility is not acceptance.
    offers::accept(&mut s, &request).unwrap();
    assert_eq!(s.state.phase, Phase::Productive);
    assert_eq!(s.state.balance(PERSON, SEED), 1); // Reserved work, not backdated execution.
    assert!(s.state.pending_production.is_some());
    s.step().unwrap();
    assert_eq!(s.state.balance(PERSON, SEED), 0);
    assert_eq!(s.state.balance(PERSON, LABOR), 0); // Crop and wood share one opening budget.
    let p = s
        .state
        .processes
        .values()
        .find(|p| p.definition == GROW)
        .unwrap();
    let contract = agreements::process(&s.world, p);
    assert_eq!(contract.grantor, Counterparty::Environment);
    assert_eq!(
        contract.production.as_ref().unwrap().instance.right,
        Some(1)
    );
    assert_eq!(
        contract.evaluate(s.state.month).status,
        agreements::Status::Active
    );
    s.run_months(6).unwrap();
    let first = s
        .state
        .processes
        .values()
        .find(|p| p.definition == GROW && p.start == 1)
        .unwrap();
    assert_eq!(
        agreements::process(&s.world, first).evaluate(6).status,
        agreements::Status::Completed
    );
}

#[test]
fn competing_starts_and_wrong_prerequisite_order_publish_nothing() {
    let mut s = sim(Backend::Reference);
    acquire(&mut s);
    for request in [
        vec![
            Request::new(Id::Land(1), PERSON),
            Request::new(Id::Membership(1), PERSON),
        ],
        {
            let mut r = bundle();
            r.push(Request::new(Id::Process(GROW), PERSON));
            r
        },
        {
            let mut r = bundle();
            r.extend([
                Request::new(Id::Process(PREPARE_FUEL), PERSON),
                Request::new(Id::Process(PREPARE_FUEL), PERSON),
            ]);
            r
        },
    ] {
        let before = s.state.clone();
        let ledger = s.ledger.clone();
        assert!(offers::accept(&mut s, &request).is_err());
        assert_eq!(s.state, before);
        assert_eq!(s.ledger, ledger);
    }
}

#[test]
fn missed_month_aborts_without_refund_or_outputs() {
    let mut s = sim(Backend::CubeCpu);
    s.world.capacity_overrides.insert((2, PERSON), 0);
    acquire(&mut s);
    offers::accept(&mut s, &bundle()).unwrap();
    s.step().unwrap();
    while s.state.month < 2 {
        s.step().unwrap();
    }
    s.run_months(1).unwrap();
    let p = s
        .state
        .processes
        .values()
        .find(|p| p.definition == GROW && p.start == 1)
        .unwrap();
    let evaluation = agreements::process(&s.world, p).evaluate(2);
    assert_eq!(evaluation.status, agreements::Status::Failed);
    assert_eq!(
        evaluation.production_failure,
        Some(Consequence::AbortWithoutRefund)
    );
    assert_eq!(s.state.balance(PERSON, SEED), 0);
    let abort = s
        .ledger
        .iter()
        .flat_map(|b| &b.transactions)
        .find(|t| {
            t.process
                .as_ref()
                .is_some_and(|c| c.after.id == p.id && c.after.status == Status::Aborted)
        })
        .unwrap();
    assert!(abort.effects.is_empty());
}

#[test]
fn currently_feasible_offer_is_rejected_by_planner_if_later_service_is_impossible() {
    let mut s = sim(Backend::Reference);
    s.world
        .definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .stages
        .last_mut()
        .unwrap()
        .monthly_services[0]
        .quantity = 4;
    acquire(&mut s);
    assert!(offers::feasible(&s, &bundle()).is_ok());
    s.step().unwrap();
    let decision = s.ledger.last().unwrap().decision.as_ref().unwrap();
    assert!(!decision.rejection_reasons.is_empty());
    assert!(
        !decision.alternatives[decision.selected]
            .first_work
            .iter()
            .any(|r| r.definition == Some(GROW) && r.reason == Reason::Selected)
    );
    assert_eq!(s.state.balance(PERSON, SEED), 1);
}

#[test]
fn forecasts_report_commitments_and_competing_monthly_work() {
    let mut s = sim(Backend::Reference);
    acquire(&mut s);
    s.step().unwrap();
    let decision = s.ledger.last().unwrap().decision.as_ref().unwrap();
    let assessment = &decision.alternatives[decision.selected].commitments;
    assert!(assessment.processes.iter().any(|a| {
        a.production.as_ref().is_some_and(|p| {
            p.instance.definition == GROW && p.instance.status == Status::Completed
        })
    }));
    assert!(
        assessment
            .capacity
            .iter()
            .any(|r| r.month == 1 && r.agent == PERSON && r.resource == LABOR && r.required == 3)
    );
    assert!(
        assessment
            .work
            .iter()
            .flat_map(|w| &w.receipts)
            .any(|r| r.definition == Some(PREPARE_FUEL) && r.reason == Reason::Selected)
    );
}

#[test]
fn individually_affordable_work_is_not_confused_with_a_sustainable_combination() {
    let mut s = sim(Backend::Reference);
    let needs = s.world.participants[0].needs.clone();
    for need in &mut s.world.participants[0].needs {
        if need.resource != NUTRITION {
            need.quantity = 0;
        }
    }
    let wood = s
        .world
        .definitions
        .iter_mut()
        .find(|d| d.id == PREPARE_FUEL)
        .unwrap();
    wood.stages[0].months = 4;
    wood.stages[0].monthly_services[0].quantity = 2;
    acquire(&mut s);
    offers::accept(&mut s, &bundle()).unwrap();
    while s.state.month < 3 {
        s.step().unwrap();
    }
    s.world.participants[0].needs = needs;
    s.state.balances.insert((PERSON, FUEL), 0);
    acquire(&mut s);
    // Current month: existing crop 1 + proposed collection 2 fits the 3-hour pool.
    assert!(offers::feasible(&s, &[Request::new(Id::Process(PREPARE_FUEL), PERSON)]).is_ok());
    // At harvest: crop 2 + collection 2 exceeds it, although each fits alone.
    let crop = s.world.definition(GROW);
    assert_eq!(crop.stages.last().unwrap().monthly_services[0].quantity, 2);
    s.step().unwrap();
    let decision = s.ledger.last().unwrap().decision.as_ref().unwrap();
    assert!(!decision.rejection_reasons.is_empty());
    let selected = &decision.alternatives[decision.selected];
    assert!(!selected.commitments.processes.iter().any(|a| {
        a.accepted_month == 3
            && a.production
                .as_ref()
                .is_some_and(|p| p.instance.definition == PREPARE_FUEL)
    }));
}
