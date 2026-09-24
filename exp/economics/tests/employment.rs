use economics_compute_smoke::{
    accounting::{Account as A, Book},
    agreements,
    compute::Backend,
    employment::{ArrearsPolicy, Reason, Terms},
    financial_reporting::{Audit, Opening},
    minting::{self, COIN, FIREWOOD, HOURS, ISSUER, SUPPLIER, WORKER},
    model::*,
    simulation::Simulation,
};
const MAKE: u32 = 900;
fn fixture(cash: i32, policy: ArrearsPolicy) -> (World, State) {
    let (mut w, mut s) = minting::scenario("normal").unwrap();
    w.minting = None;
    w.transaction_policy = None;
    w.scheduled_starts.clear();
    w.capacity_overrides.clear();
    w.priority = Priority::ContinuingFirst;
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == WORKER { 2 } else { 0 };
    }
    s.balances.clear();
    s.balances.insert((ISSUER, COIN), cash);
    w.definitions.push(ProcessDefinition {
        id: MAKE,
        name: "paid production".into(),
        enabled: true,
        execution: Execution::Productive,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 1,
            entry_inputs: vec![],
            monthly_services: vec![Amount::new(HOURS, 2)],
        }],
        outputs: vec![Amount::new(FIREWOOD, 2)],
    });
    w.scheduled_starts.push(ScheduledStart {
        month: 1,
        agent: ISSUER,
        definition: MAKE,
    });
    w.employment.push(Terms {
        id: 1,
        employer: ISSUER,
        worker: WORKER,
        from: 1,
        through: 4,
        capacity: Amount::new(HOURS, 2),
        wage_per_unit: Amount::new(COIN, 2),
        on_arrears: policy,
        rank: 0,
    });
    (w, s)
}
fn audit(w: &World, s: &State, capitalize: bool) -> Audit {
    Audit::with_opening(
        w,
        s,
        COIN,
        Opening {
            processes: Some(Default::default()),
            services: capitalize.then(Default::default),
            ..Default::default()
        },
    )
    .unwrap()
}
fn value(a: &Audit, agent: AgentId, account: A) -> i128 {
    a.book()
        .balances()
        .get(&(agent, account))
        .copied()
        .unwrap_or(0)
}
fn through(a: &mut Audit, s: &mut Simulation, month: u32) {
    while s.state.month <= month {
        a.step(s)
            .unwrap_or_else(|e| panic!("{e}: {} {:?}", s.state.month, s.state.phase));
    }
}
#[test]
fn earned_wages_capitalize_and_partial_payment_leaves_matching_arrears() {
    let (w, s) = fixture(3, ArrearsPolicy::SuspendDelivery);
    let mut a = audit(&w, &s, true);
    let mut b = a.clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    while cpu.state.phase != Phase::Productive {
        a.step(&mut cpu).unwrap();
        b.step(&mut reference).unwrap();
    }
    assert_eq!(value(&a, ISSUER, A::PurchasedCapacity(HOURS)), 4);
    assert_eq!(
        economics_compute_smoke::employment::contract(
            &cpu.world.employment[0],
            &cpu.state.employment
        )
        .evaluate(1)
        .status,
        agreements::Status::Active
    );
    assert_eq!(value(&a, WORKER, A::WagesReceivable(1, 1)), 4);
    assert_eq!(value(&a, ISSUER, A::WagesPayable(1, 1)), -4);
    let mut resumed = cpu.clone();
    let mut c = a.clone();
    through(&mut a, &mut cpu, 2);
    through(&mut b, &mut reference, 2);
    through(&mut c, &mut resumed, 2);
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.state, resumed.state);
    assert_eq!(a.book().balances(), b.book().balances());
    assert_eq!(a.book().balances(), c.book().balances());
    assert_eq!(value(&a, ISSUER, A::Inventory(FIREWOOD)), 4);
    assert_eq!(value(&a, ISSUER, A::ServiceExpense), 0);
    assert_eq!(value(&a, WORKER, A::ServiceIncome), -4);
    assert_eq!(value(&a, WORKER, A::WagesReceivable(1, 1)), 1);
    assert_eq!(value(&a, ISSUER, A::WagesPayable(1, 1)), -1);
    assert_eq!(cpu.state.balance(WORKER, COIN), 3);
    assert!(
        cpu.ledger
            .iter()
            .filter_map(|b| b.employment.as_ref())
            .flat_map(|b| &b.receipts)
            .any(|r| r.earned_month == 2 && r.reason == Reason::Arrears)
    );
    let view = agreements::for_agent(&cpu.world, &cpu.state, WORKER)
        .unwrap()
        .into_iter()
        .find(|v| v.identity() == agreements::Identity::Employment(1))
        .unwrap();
    assert_eq!(view.claims().unwrap()[0].outstanding(), 1);
    assert_eq!(
        Book::from_json(&a.book().to_json().unwrap())
            .unwrap()
            .balances(),
        a.book().balances()
    );
}
#[test]
fn later_earnings_pay_old_wages_without_same_boundary_reuse_and_work_resumes() {
    let (mut w, mut s) = fixture(3, ArrearsPolicy::SuspendDelivery);
    s.balances.insert((SUPPLIER, COIN), 2);
    w.capacity_overrides.insert((2, ISSUER), 1);
    let mut t = w.employment[0].clone();
    t.id = 2;
    t.employer = SUPPLIER;
    t.worker = ISSUER;
    t.from = 2;
    t.through = 2;
    t.capacity.quantity = 1;
    w.employment.push(t);
    let mut a = audit(&w, &s, true);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 2);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 1);
    assert_eq!(sim.state.balance(ISSUER, COIN), 2);
    through(&mut a, &mut sim, 3);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
    through(&mut a, &mut sim, 4);
    assert_eq!(sim.state.employment.earned[&(1, 4)].delivered, 2);
    assert_eq!(value(&a, WORKER, A::ServiceIncome), -8);
    assert_eq!(value(&a, WORKER, A::WagesReceivable(1, 1)), 0);
}
#[test]
fn competing_contracts_cap_delivery_and_do_not_earn_for_missing_hours() {
    let (mut w, s) = fixture(0, ArrearsPolicy::Continue);
    w.employment[0].capacity.quantity = 1;
    let mut t = w.employment[0].clone();
    t.id = 2;
    t.capacity.quantity = 2;
    t.employer = SUPPLIER;
    w.employment.push(t);
    w.employment.reverse();
    let mut a = audit(&w, &s, false);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 2);
    assert_eq!(sim.state.employment.earned[&(1, 1)].delivered, 1);
    assert_eq!(sim.state.employment.earned[&(2, 1)].delivered, 1);
    assert_eq!(value(&a, WORKER, A::ServiceIncome), -8);
    assert_eq!(value(&a, ISSUER, A::ServiceExpense), 4);
    assert_eq!(value(&a, SUPPLIER, A::ServiceExpense), 4);
}
#[test]
fn idle_labor_expires_but_expiration_never_cancels_earned_wages() {
    let (mut w, s) = fixture(0, ArrearsPolicy::SuspendDelivery);
    w.scheduled_starts.clear();
    let mut a = audit(&w, &s, true);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 2);
    assert_eq!(value(&a, ISSUER, A::PurchasedCapacity(HOURS)), 0);
    assert_eq!(value(&a, ISSUER, A::ServiceExpense), 4);
    assert_eq!(value(&a, ISSUER, A::WagesPayable(1, 1)), -4);
    through(&mut a, &mut sim, 5);
    assert_eq!(value(&a, WORKER, A::WagesReceivable(1, 1)), 4);
    let v = agreements::for_agent(&sim.world, &sim.state, WORKER)
        .unwrap()
        .into_iter()
        .find(|v| v.identity() == agreements::Identity::Employment(1))
        .unwrap();
    let agreements::View::Agreement(v) = v else {
        panic!()
    };
    assert_eq!(v.evaluate(5).status, agreements::Status::Expired);
    assert_eq!(v.evaluate(5).breaches[0].outstanding, 4);
}
#[test]
fn forged_receipts_missing_boundary_and_duplicate_commit_are_atomic() {
    let (w, s) = fixture(3, ArrearsPolicy::Continue);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut original_audit = audit(&sim.world, &sim.state, true);
    original_audit.step(&mut sim).unwrap();
    let before = sim.state.clone();
    sim.step().unwrap();
    let valid = sim.ledger.last().unwrap().clone();
    let mut limited = before.clone();
    assert!(
        economics_compute_smoke::settlement::commit(
            &sim.world,
            &mut limited,
            &valid,
            Backend::Reference,
            1
        )
        .is_err()
    );
    assert_eq!(limited, before);
    for kind in 0..3 {
        let mut bad = valid.clone();
        if kind == 0 {
            bad.employment = None;
        } else if kind == 1 {
            bad.employment
                .as_mut()
                .unwrap()
                .after
                .earned
                .get_mut(&(1, 1))
                .unwrap()
                .claim
                .transfer
                .amount
                .quantity += 1;
        } else {
            bad.employment.as_mut().unwrap().transactions[0].effects[1].delta += 1;
        }
        let mut state = before.clone();
        assert!(
            economics_compute_smoke::settlement::commit(
                &sim.world,
                &mut state,
                &bad,
                Backend::Reference,
                4096
            )
            .is_err()
        );
        assert_eq!(state, before);
        let mut candidate = original_audit.clone();
        assert!(
            candidate
                .record(&sim.world, &before, &bad, &sim.state)
                .is_err()
        );
        assert_eq!(
            candidate.book().balances(),
            original_audit.book().balances()
        );
    }
    let mut state = sim.state.clone();
    assert!(
        economics_compute_smoke::settlement::commit(
            &sim.world,
            &mut state,
            &valid,
            Backend::Reference,
            4096
        )
        .is_err()
    );
    assert_eq!(state, sim.state);
}
#[test]
fn monthly_availability_and_permissions_bound_earnings() {
    let (mut w, s) = fixture(0, ArrearsPolicy::Continue);
    w.capacity_overrides.insert((1, WORKER), 1);
    w.capacity_overrides.insert((2, WORKER), 0);
    let mut a = audit(&w, &s, false);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 2);
    assert_eq!(value(&a, WORKER, A::ServiceIncome), -2);
    assert!(!sim.state.employment.earned.contains_key(&(1, 2)));
    let (mut w, s) = fixture(0, ArrearsPolicy::Continue);
    w.transaction_policy = minting::scenario("normal").unwrap().0.transaction_policy;
    w.transaction_policy.as_mut().unwrap().permissions.clear();
    let mut a = audit(&w, &s, false);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    assert!(sim.state.employment.earned.is_empty());
}

#[test]
fn payroll_priority_is_explicit_and_independent_of_catalog_order() {
    let (mut w, mut s) = fixture(3, ArrearsPolicy::Continue);
    w.participants
        .iter_mut()
        .find(|p| p.agent == SUPPLIER)
        .unwrap()
        .capacity
        .quantity = 2;
    let mut t = w.employment[0].clone();
    t.id = 2;
    t.worker = SUPPLIER;
    t.rank = 0;
    w.employment[0].rank = 1;
    w.employment.push(t);
    s.balances.insert((SUPPLIER, COIN), 0);
    let mut a = audit(&w, &s, false);
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    w.employment.reverse();
    let mut other = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    other.run_months(1).unwrap();
    assert_eq!(sim.state, other.state);
    assert_eq!(sim.ledger, other.ledger);
    assert_eq!(sim.state.balance(SUPPLIER, COIN), 3);
    assert_eq!(sim.state.balance(WORKER, COIN), 0);
    assert_eq!(value(&a, ISSUER, A::WagesPayable(2, 1)), -1);
    assert_eq!(value(&a, ISSUER, A::WagesPayable(1, 1)), -4);
}
#[test]
fn employment_composes_with_cash_service_acquisition_and_telemetry() {
    use economics_compute_smoke::{
        issuance_accounting::Policy,
        telemetry::{Config, Observer},
    };
    let (mut w, s) = minting::scenario("normal").unwrap();
    // Existing minting buys its two hours first. The same worker cannot sell them twice.
    w.employment.push(Terms {
        id: 1,
        employer: ISSUER,
        worker: WORKER,
        from: 2,
        through: 2,
        capacity: Amount::new(HOURS, 2),
        wage_per_unit: Amount::new(COIN, 2),
        on_arrears: ArrearsPolicy::Continue,
        rank: 0,
    });
    w.capacity_overrides.insert((2, WORKER), 3);
    let mut a = Audit::with_opening(
        &w,
        &s,
        COIN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| {
                    *r != COIN
                        && **q > 0
                        && w.resources
                            .iter()
                            .any(|x| x.id == *r && x.kind == ResourceKind::Stock)
                })
                .map(|(k, q)| (*k, i128::from(*q)))
                .collect(),
            processes: Some(Default::default()),
            services: Some(Default::default()),
            issuance: Some(Policy::NonRedeemableEquity),
            ..Default::default()
        },
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut observer = Observer::new(
        Vec::new(),
        "employment",
        Config {
            settlement: true,
            ..Default::default()
        },
    )
    .unwrap();
    while sim.state.month <= 2 {
        observer.step_audited(&mut sim, &mut a).unwrap();
    }
    assert_eq!(sim.state.employment.earned[&(1, 2)].delivered, 1);
    assert_eq!(sim.state.employment.earned[&(1, 2)].claim.outstanding(), 0);
    let bytes = observer.finish().unwrap();
    let logs = String::from_utf8(bytes).unwrap();
    let records: Vec<serde_json::Value> = logs
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(
        records
            .iter()
            .any(|r| r["kind"] == "employment" && r["delivered"] == 1 && r["earned"] == 2)
    );
    assert!(logs.contains("Collection"));
}
#[test]
fn invalid_terms_and_unreportable_wages_fail_before_publication() {
    let (w, s) = fixture(0, ArrearsPolicy::Continue);
    for mode in 0..4 {
        let mut bad = w.clone();
        match mode {
            0 => bad.employment[0].worker = ISSUER,
            1 => bad.employment[0].wage_per_unit.quantity = i32::MAX,
            2 => bad.employment[0].capacity.resource = COIN,
            _ => bad.employment.push(bad.employment[0].clone()),
        }
        assert!(Simulation::new(bad, s.clone(), Backend::Reference).is_err());
    }
    let mut other = w;
    other.employment[0].wage_per_unit.resource = FIREWOOD;
    assert!(Audit::with_opening(&other, &s, COIN, Opening::default()).is_err());
}
