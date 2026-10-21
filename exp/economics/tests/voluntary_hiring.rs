use economics_compute_smoke::{
    activities::{Target, WorkOrder},
    agency,
    compute::Backend,
    employment::recruitment::{Mandate, Worker},
    minting::*,
    model::*,
    opportunities::{Action, PERSON_TYPE},
    simulation::Simulation,
};
fn fixture() -> (World, State) {
    let (mut w, mut s) = agency::scenario::public_work().unwrap();
    let t = &mut w.employment[0];
    t.from = 1;
    t.through = 12;
    w.employment_offers.insert(t.id);
    w.agency.get_mut(&ISSUER).unwrap().config.hiring = Some(Mandate {
        offers: [1].into(),
        payroll: Amount::new(COIN, 4),
        horizon: 2,
    });
    w.recruitment.workers.insert(
        WORKER,
        Worker {
            horizon: 2,
            reserves: vec![Amount::new(COIN, 100)],
        },
    );
    w.activities.orders.push(WorkOrder {
        agent: ISSUER,
        definition: GATHER,
        priority: 0,
        target: Target::Stock(Amount::new(FIREWOOD, 4)),
    });
    s.phase = Phase::Acquire;
    s.balances.insert((WORKER, HOURS), 2);
    s.balances.insert((ISSUER, HOURS), 0);
    (w, s)
}
fn acquire(w: World, s: State) -> Simulation {
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    sim
}
#[test]
fn voluntary_job_delivers_hours_then_pays_at_close() {
    let (w, s) = fixture();
    let cash = s.balance(WORKER, COIN);
    let mut sim = acquire(w, s);
    let b = sim.ledger.last().unwrap().employment.as_ref().unwrap();
    println!("{b:?}");
    assert_eq!(b.recruitment[0].selected, 2);
    assert!(!b.recruitment[0].hypothetical);
    assert_eq!(sim.state.balance(WORKER, COIN), cash);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.settled, 0);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.settled, 2);
}
#[test]
fn no_work_no_worker_consent_no_funds_or_no_permission_do_not_hire() {
    for case in 0..4 {
        let (mut w, mut s) = fixture();
        match case {
            0 => w.activities.orders.clear(),
            1 => w.recruitment.workers.clear(),
            2 => {
                s.balances.insert((ISSUER, COIN), 0);
            }
            _ => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(PERSON_TYPE, Action::CapacityTrade));
            }
        }
        let sim = acquire(w, s);
        assert!(sim.state.employment.earned.is_empty(), "case {case}");
        assert_eq!(sim.state.balance(WORKER, HOURS), 2);
    }
}
#[test]
fn essential_personal_capacity_can_refuse_profitable_paid_work() {
    let (mut w, s) = fixture();
    const NEED: u32 = 9001;
    const SELF_CARE: u32 = 9001;
    w.resources.push(Resource {
        id: NEED,
        name: "essential care".into(),
        kind: ResourceKind::Fulfillment,
    });
    w.participants
        .iter_mut()
        .find(|p| p.agent == WORKER)
        .unwrap()
        .needs
        .push(Requirement {
            resource: NEED,
            quantity: 1,
            priority: 0,
        });
    w.definitions.push(ProcessDefinition {
        id: SELF_CARE,
        name: "self care".into(),
        execution: Execution::Consumption,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "care".into(),
            months: 1,
            entry_inputs: vec![Amount::new(FIREWOOD, 1)],
            monthly_services: vec![],
        }],
        outputs: vec![Amount::new(NEED, 1)],
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::Process(SELF_CARE)));
    let sim = acquire(w, s);
    let d = &sim
        .ledger
        .last()
        .unwrap()
        .employment
        .as_ref()
        .unwrap()
        .recruitment[0];
    println!("{d:?}");
    assert_eq!(d.selected, 0);
    assert!(
        d.alternatives
            .iter()
            .any(|a| a.projection.as_ref().is_ok_and(|p| p.completed > 0))
    );
    assert_eq!(sim.state.balance(WORKER, HOURS), 2);
}
#[test]
fn competing_jobs_cannot_reuse_hours_or_overbuy_an_indivisible_task() {
    let (mut w, s) = fixture();
    let mut second = w.employment[0].clone();
    second.id = 2;
    w.employment.push(second);
    w.employment_offers.insert(2);
    w.agency
        .get_mut(&ISSUER)
        .unwrap()
        .config
        .hiring
        .as_mut()
        .unwrap()
        .offers
        .insert(2);
    let sim = acquire(w, s);
    let delivered: i32 = sim
        .ledger
        .last()
        .unwrap()
        .employment
        .as_ref()
        .unwrap()
        .receipts
        .iter()
        .map(|r| r.delivered)
        .sum();
    assert_eq!(delivered, 2);
    assert_eq!(sim.state.employment.earned.len(), 1);
}
#[test]
fn excessive_wage_budget_and_unusable_partial_work_are_rejected() {
    for hours in [1, 2] {
        let (mut w, mut s) = fixture();
        s.balances.insert((WORKER, HOURS), hours);
        if hours == 2 {
            w.employment[0].wage_per_unit.quantity = 3;
        }
        let sim = acquire(w, s);
        assert!(sim.state.employment.earned.is_empty());
        assert_eq!(sim.state.balance(WORKER, HOURS), hours);
    }
}
#[test]
fn altered_consent_receipt_cannot_commit() {
    let (w, s) = fixture();
    let sim = acquire(w.clone(), s.clone());
    let mut batch = sim.ledger[0].clone();
    batch.employment.as_mut().unwrap().recruitment[0].selected = 0;
    let mut untouched = s.clone();
    assert!(
        economics_compute_smoke::settlement::commit(
            &w,
            &mut untouched,
            &batch,
            Backend::Reference,
            economics_compute_smoke::settlement::DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(untouched, s);
}

#[test]
fn other_denominations_do_not_consume_the_coin_payroll_budget() {
    let (mut w, mut s) = fixture();
    let mut grain_job = w.employment[0].clone();
    grain_job.id = 2;
    grain_job.worker = SUPPLIER;
    grain_job.capacity = Amount::new(HOURS, 1);
    grain_job.wage_per_unit = Amount::new(WHEAT, 100);
    w.employment.push(grain_job);
    s.balances.insert((SUPPLIER, HOURS), 1);
    s.balances.insert((ISSUER, WHEAT), 100);
    w.storage.capacities.insert(ISSUER, 200);
    w.storage.capacities.insert(SUPPLIER, 200);
    // Use a separate capacity so unrelated preaccepted delivery cannot satisfy
    // this job's productive requirement or reduce its useful quantity.
    const OTHER: u32 = 9900;
    w.resources.push(Resource {
        id: OTHER,
        name: "other service".into(),
        kind: ResourceKind::Capacity,
    });
    w.employment[1].capacity.resource = OTHER;
    s.balances.insert((SUPPLIER, OTHER), 1);
    let sim = acquire(w, s);
    let d = &sim
        .ledger
        .last()
        .unwrap()
        .employment
        .as_ref()
        .unwrap()
        .recruitment[0];
    assert_eq!(d.affordable, 2);
    assert_eq!(d.selected, 2);
}

#[test]
fn hypothetical_future_consent_is_visible_and_cannot_override_live_refusal() {
    let (mut w, s) = fixture();
    // Worker can keep current hours for care, while a mechanical forecast only
    // checks useful production and funded payroll.
    const NEED: u32 = 9001;
    const CARE: u32 = 9001;
    w.resources.push(Resource {
        id: NEED,
        name: "essential care".into(),
        kind: ResourceKind::Fulfillment,
    });
    w.participants
        .iter_mut()
        .find(|p| p.agent == WORKER)
        .unwrap()
        .needs
        .push(Requirement {
            resource: NEED,
            quantity: 1,
            priority: 0,
        });
    w.definitions.push(ProcessDefinition {
        id: CARE,
        name: "care".into(),
        execution: Execution::Consumption,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "care".into(),
            months: 1,
            entry_inputs: vec![Amount::new(FIREWOOD, 1)],
            monthly_services: vec![],
        }],
        outputs: vec![Amount::new(NEED, 1)],
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::Process(CARE)));
    let (hypothesis, state) =
        economics_compute_smoke::forecast::ForecastContext::new(&w, &s).into_parts();
    let forecast = acquire(hypothesis, state);
    let predicted = &forecast.ledger[0].employment.as_ref().unwrap().recruitment[0];
    assert!(predicted.hypothetical);
    assert_eq!(predicted.selected, 2);
    assert!(
        predicted
            .alternatives
            .iter()
            .all(|a| a.worker_accepts.is_none())
    );
    let live = acquire(w, s);
    let actual = &live.ledger[0].employment.as_ref().unwrap().recruitment[0];
    assert!(!actual.hypothetical);
    assert_eq!(actual.requested, 2);
    assert_eq!(actual.selected, 0);
    assert!(live.state.employment.earned.is_empty());
    assert_eq!(live.state.balance(WORKER, HOURS), 2);
}

#[test]
fn planning_observer_exposes_demand_consent_and_conditional_assumptions() {
    use economics_compute_smoke::telemetry::{Config, Observer, PlanningDetail};
    let (w, s) = fixture();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut bytes = Vec::new();
    {
        let mut observer = Observer::new(
            &mut bytes,
            "voluntary",
            Config {
                planning: PlanningDetail::Alternatives,
                ..Default::default()
            },
        )
        .unwrap();
        observer.step(&mut sim).unwrap();
    }
    let rows: Vec<serde_json::Value> = String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|v| serde_json::from_str(v).unwrap())
        .collect();
    let row = rows
        .iter()
        .find(|v| v["kind"] == "voluntary_hiring")
        .unwrap();
    assert_eq!(row["employer_demand"], 2);
    assert_eq!(row["mutually_accepted"], 2);
    assert_eq!(row["hypothetical"], false);
    assert_eq!(row["valuation"], "DatedJob");
    assert_eq!(row["alternatives"][0]["employer_accepts"], true);
    assert_eq!(row["alternatives"][0]["worker_accepts"], true);
    assert_eq!(row["protected_duties"].as_array().unwrap().len(), 2);
}

#[test]
fn worker_reserves_reject_duplicates_nonstocks_and_negative_targets() {
    for reserves in [
        vec![Amount::new(COIN, 1), Amount::new(COIN, 2)],
        vec![Amount::new(HOURS, 1)],
        vec![Amount::new(COIN, -1)],
    ] {
        let (mut w, s) = fixture();
        w.recruitment.workers.get_mut(&WORKER).unwrap().reserves = reserves;
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
    let (mut w, s) = fixture();
    for id in 10000..10009 {
        w.resources.push(Resource {
            id,
            name: format!("stock {id}"),
            kind: ResourceKind::Stock,
        });
    }
    w.recruitment.workers.get_mut(&WORKER).unwrap().reserves =
        (10000..10009).map(|id| Amount::new(id, 1)).collect();
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
}
