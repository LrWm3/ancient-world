use economics_compute_smoke::{
    acceptance::{SMITH, WORKSHOP, WORKSHOP_EMPLOYMENT, skilled_workshop},
    compute::Backend,
    host::{Boundary, Opening, Settlement, Stage},
    minting::{HOURS, REFINE},
    model::Phase,
};

fn fixture(backend: Backend) -> Settlement {
    let run = skilled_workshop::run(true, backend).unwrap();
    Settlement::attach(
        Boundary {
            site: 7,
            month: 101,
        },
        run.sim,
        run.audit,
    )
    .unwrap()
}

fn input(s: &Settlement) -> Opening {
    Opening {
        boundary: s.boundary(),
        capacities: s.capacity_ceilings(),
    }
}

fn same(a: &Settlement, b: &Settlement) {
    assert_eq!(a.boundary(), b.boundary());
    assert_eq!(a.stage(), b.stage());
    assert_eq!(a.simulation().world, b.simulation().world);
    assert_eq!(a.simulation().state, b.simulation().state);
    assert_eq!(a.simulation().ledger, b.simulation().ledger);
    assert_eq!(a.simulation().reports, b.simulation().reports);
    assert_eq!(a.audit(), b.audit());
}

#[test]
fn dated_stages_match_ordinary_skilled_economy_and_cpu_continuation() {
    let mut direct = skilled_workshop::run(true, Backend::Reference).unwrap();
    let mut staged = fixture(Backend::CubeCpu);
    let mut monthly = fixture(Backend::Reference);
    let mut completed = 0;
    for month in 1..=12 {
        let boundary = staged.boundary();
        assert_eq!(boundary.month, month + 100);
        let open = staged.open(input(&staged)).unwrap();
        assert_eq!(open.local_month, month);
        assert_eq!(staged.simulation().state.phase, Phase::Acquire);
        let reserve = staged.reserve(boundary).unwrap();
        assert_eq!(staged.simulation().state.phase, Phase::Productive);
        assert_eq!(staged.simulation().state.month, month);
        // A pending dated reservation must survive an actual mid-month clone.
        let mut resumed = staged.clone();
        let execute = resumed.execute(boundary).unwrap();
        assert_eq!(resumed.simulation().state.phase, Phase::Close);
        let respond = resumed.respond(boundary).unwrap();
        assert_eq!(respond.first_batch, respond.next_batch);
        let outcome = resumed.close(boundary).unwrap();
        assert_eq!(outcome.stages[..4], [open, reserve, execute, respond]);
        assert_eq!(outcome.stages[4].stage, Stage::Close);
        for pair in outcome.stages.windows(2) {
            assert_eq!(pair[0].next_batch, pair[1].first_batch);
        }
        assert_eq!(outcome, monthly.advance_month(input(&monthly)).unwrap());
        same(&resumed, &monthly);
        direct.through(month).unwrap();
        assert_eq!(resumed.simulation().world, direct.sim.world);
        assert_eq!(resumed.simulation().state, direct.sim.state);
        assert_eq!(resumed.simulation().ledger, direct.sim.ledger);
        assert_eq!(resumed.simulation().reports, direct.sim.reports);
        assert_eq!(resumed.audit(), &direct.audit);
        assert!(outcome.needs.iter().all(|n| n.deficit == 0));
        completed += outcome
            .completions
            .iter()
            .filter(|p| p.operator == WORKSHOP && p.definition == REFINE)
            .count();
        for change in &outcome.stocks {
            assert_eq!(
                change.closing,
                direct.sim.state.balance(change.agent, change.resource)
            );
            assert_ne!(change.opening, change.closing);
        }
        staged = resumed;
    }
    assert_eq!(completed, 8);
    assert_eq!(direct.evidence().unwrap().wage_arrears, 0);
}

#[test]
fn unavailable_worker_cannot_supply_hours_and_recovers_next_month() {
    let mut live = fixture(Backend::Reference);
    for _ in 0..2 {
        live.advance_month(input(&live)).unwrap();
    }
    let mut opening = input(&live);
    opening
        .capacities
        .iter_mut()
        .find(|c| c.agent == SMITH)
        .unwrap()
        .quantity = 0;
    let before = live.simulation().state.practice.clone();
    let stopped = live.advance_month(opening).unwrap();
    assert!(!stopped.completions.iter().any(|p| p.operator == WORKSHOP));
    assert_eq!(live.simulation().state.practice, before);
    assert_eq!(
        live.simulation()
            .state
            .employment
            .earned
            .get(&(WORKSHOP_EMPLOYMENT, 3))
            .map_or(0, |e| e.delivered),
        0
    );
    let mut resumed_work = 0;
    for _ in 0..5 {
        let outcome = live.advance_month(input(&live)).unwrap();
        resumed_work += outcome
            .completions
            .iter()
            .filter(|p| p.operator == WORKSHOP)
            .count();
    }
    assert!(resumed_work > 0);
    assert_eq!(live.simulation().world.capacity_overrides[&(3, SMITH)], 0);
    assert!(
        !live
            .simulation()
            .world
            .capacity_overrides
            .contains_key(&(4, SMITH))
    );
    live.audit()
        .validate_boundary(&live.simulation().world, &live.simulation().state)
        .unwrap();
}

#[test]
fn invalid_inputs_and_repeated_stages_leave_everything_unchanged() {
    let original = fixture(Backend::Reference);
    let mut variants = Vec::new();
    let good = input(&original);
    let mut bad = good.clone();
    bad.boundary.site += 1;
    variants.push(bad);
    let mut bad = good.clone();
    bad.boundary.month += 1;
    variants.push(bad);
    let mut bad = good.clone();
    bad.boundary.month -= 1;
    variants.push(bad);
    let mut bad = good.clone();
    bad.capacities.pop();
    variants.push(bad);
    let mut bad = good.clone();
    bad.capacities.push(bad.capacities[0].clone());
    variants.push(bad);
    let mut bad = good.clone();
    bad.capacities[0].quantity = -1;
    variants.push(bad);
    let mut bad = good.clone();
    bad.capacities[0].quantity += 1;
    variants.push(bad);
    let mut bad = good.clone();
    bad.capacities[0].resource = u32::MAX;
    variants.push(bad);
    let mut bad = good.clone();
    bad.capacities[0].agent = u32::MAX;
    variants.push(bad);
    for bad in variants {
        let mut s = original.clone();
        assert!(s.open(bad).is_err());
        same(&s, &original);
    }
    let mut s = original.clone();
    assert!(s.execute(good.boundary).is_err());
    same(&s, &original);
    s.open(good.clone()).unwrap();
    let opened = s.clone();
    assert!(s.open(good.clone()).is_err());
    assert!(s.close(good.boundary).is_err());
    same(&s, &opened);
    s.reserve(good.boundary).unwrap();
    let reserved = s.clone();
    assert!(s.reserve(good.boundary).is_err());
    same(&s, &reserved);
    s.execute(good.boundary).unwrap();
    let executed = s.clone();
    assert!(s.execute(good.boundary).is_err());
    same(&s, &executed);
    s.respond(good.boundary).unwrap();
    s.close(good.boundary).unwrap();
    let closed = s.clone();
    assert!(s.close(good.boundary).is_err());
    assert!(s.open(good).is_err());
    same(&s, &closed);
}

#[test]
fn whole_month_failure_and_overflow_publish_nothing() {
    let mut run = skilled_workshop::run(true, Backend::Reference).unwrap();
    run.sim.effect_limit = 0;
    let mut s = Settlement::attach(Boundary { site: 7, month: 1 }, run.sim, run.audit).unwrap();
    let before = s.clone();
    let mut opening = input(&s);
    opening
        .capacities
        .iter_mut()
        .find(|c| c.agent == SMITH)
        .unwrap()
        .quantity = 0;
    assert!(s.advance_month(opening).is_err());
    same(&s, &before);
    let run = skilled_workshop::run(true, Backend::Reference).unwrap();
    let mut max = Settlement::attach(
        Boundary {
            site: 7,
            month: u32::MAX,
        },
        run.sim,
        run.audit,
    )
    .unwrap();
    let before = max.clone();
    assert!(max.open(input(&max)).is_err());
    same(&max, &before);
}

#[test]
fn attaching_requires_matching_history_and_open_boundary() {
    let run = skilled_workshop::run(true, Backend::Reference).unwrap();
    let mut modified = run.sim.clone();
    modified.state.balances.insert((SMITH, HOURS), 1);
    assert!(
        Settlement::attach(Boundary { site: 1, month: 1 }, modified, run.audit.clone()).is_err()
    );
    let mut direct = run;
    direct.audit.step(&mut direct.sim).unwrap();
    assert!(Settlement::attach(Boundary { site: 1, month: 1 }, direct.sim, direct.audit).is_err());
}

#[test]
fn reordered_inputs_and_existing_capacity_interruptions_are_preserved() {
    let mut run = skilled_workshop::run(true, Backend::Reference).unwrap();
    run.sim.world.capacity_overrides.insert((1, SMITH), 0);
    let mut a = Settlement::attach(Boundary { site: 2, month: 5 }, run.sim, run.audit).unwrap();
    let mut b = a.clone();
    let mut reversed = input(&b);
    reversed.capacities.reverse();
    assert_eq!(
        a.advance_month(input(&a)).unwrap(),
        b.advance_month(reversed).unwrap()
    );
    same(&a, &b);
    assert_eq!(a.simulation().world.capacity_overrides[&(1, SMITH)], 0);
}

#[test]
fn an_above_catalog_override_cannot_exceed_the_host_grant() {
    let mut run = skilled_workshop::run(true, Backend::Reference).unwrap();
    run.sim.world.capacity_overrides.insert((1, SMITH), 100);
    let mut s = Settlement::attach(Boundary { site: 2, month: 5 }, run.sim, run.audit).unwrap();
    let grant = s
        .capacity_ceilings()
        .into_iter()
        .find(|c| c.agent == SMITH)
        .unwrap()
        .quantity;
    s.open(input(&s)).unwrap();
    assert_eq!(s.simulation().world.capacity_overrides[&(1, SMITH)], grant);
    assert_eq!(s.simulation().state.balance(SMITH, HOURS), grant);
}

#[test]
fn a_demographic_driver_requires_explicit_host_rebinding_even_with_births_disabled() {
    for enabled in [true, false] {
        let mut run = skilled_workshop::run(true, Backend::Reference).unwrap();
        // Attachment rejects before attempting to reinterpret another roster.
        run.sim.world.demography =
            economics_compute_smoke::demography::scenario::scenario(enabled, false)
                .0
                .demography;
        let err =
            Settlement::attach(Boundary { site: 1, month: 1 }, run.sim, run.audit).unwrap_err();
        assert!(err.contains("fixed participant roster"));
    }
}

fn small(
    process_costs: bool,
) -> (
    economics_compute_smoke::simulation::Simulation,
    economics_compute_smoke::financial_reporting::Audit,
) {
    use economics_compute_smoke::{
        financial_reporting::Audit, model::*, scenario::*, simulation::Simulation,
    };
    let (mut world, state) = baseline();
    world.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    let mut audit = Audit::with_inventory(
        &world,
        &state,
        TOKEN,
        [(PLOT, 0)].into(),
        state
            .balances
            .iter()
            .map(|(k, q)| (*k, i128::from(*q)))
            .collect(),
    )
    .unwrap();
    if process_costs {
        audit = audit
            .with_process_policy(&world, Default::default())
            .unwrap();
    }
    (
        Simulation::new(world, state, Backend::Reference).unwrap(),
        audit,
    )
}

#[test]
fn later_attachment_keeps_active_work_and_costs_when_acquire_is_absent() {
    use economics_compute_smoke::model::Status;
    let (mut sim, mut audit) = small(true);
    while sim.state.month == 1 {
        audit.step(&mut sim).unwrap();
    }
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.status == Status::Active)
    );
    let mut s =
        Settlement::attach(Boundary { site: 3, month: 50 }, sim.clone(), audit.clone()).unwrap();
    let mut harvests = 0;
    for month in 2..=6 {
        let boundary = s.boundary();
        s.open(input(&s)).unwrap();
        assert_eq!(s.simulation().state.phase, Phase::Productive);
        let reserve = s.reserve(boundary).unwrap();
        assert_eq!(reserve.first_batch, reserve.next_batch);
        s.execute(boundary).unwrap();
        s.respond(boundary).unwrap();
        let outcome = s.close(boundary).unwrap();
        harvests += outcome
            .completions
            .iter()
            .filter(|p| p.definition == economics_compute_smoke::scenario::GROW)
            .count();
        while sim.state.month <= month {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(s.simulation().state, sim.state);
        assert_eq!(s.simulation().ledger, sim.ledger);
        assert_eq!(s.audit(), &audit);
    }
    assert_eq!(harvests, 1);
}

#[test]
fn accounting_failure_after_productive_execution_rolls_back_the_stage_and_whole_month() {
    let (sim, audit) = small(false);
    let mut s = Settlement::attach(Boundary { site: 1, month: 1 }, sim, audit).unwrap();
    let before_month = s.clone();
    let boundary = s.boundary();
    s.open(input(&s)).unwrap();
    s.reserve(boundary).unwrap();
    let before_execution = s.clone();
    // Physical execution is feasible; the missing cost policy rejects reporting.
    let mut physical = s.simulation().clone();
    physical.step().unwrap();
    assert_ne!(physical.state, s.simulation().state);
    let err = s.execute(boundary).unwrap_err();
    assert_eq!(err, "transaction needs an explicit accounting adapter");
    same(&s, &before_execution);
    let mut entire = before_month.clone();
    assert!(entire.advance_month(input(&entire)).is_err());
    same(&entire, &before_month);
}
