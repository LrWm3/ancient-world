use economics_compute_smoke::{
    acceptance::{self, Run, continuation as c},
    compute::Backend,
    model::{Phase, State, World},
    simulation::Simulation,
};

type Scenario = fn(c::Case) -> Result<(World, State), String>;

#[test]
fn spot_replenishment_resumes_after_a_temporary_ore_supply_interruption() {
    let run = |backend| {
        let (w, s) = c::spot_scenario(c::Case::Baseline).unwrap();
        let mut run = Run::new(w, s, backend).unwrap();
        for month in 1..=c::MONTHS {
            if month == 13 || month == 16 {
                run.sim
                    .world
                    .definitions
                    .iter_mut()
                    .find(|d| d.id == acceptance::MUNICIPAL_WORK)
                    .unwrap()
                    .enabled = month == 16;
            }
            run.through(month).unwrap();
        }
        run
    };
    let reference = run(Backend::Reference);
    let cpu = run(Backend::CubeCpu);
    assert_eq!(reference.sim.state, cpu.sim.state);
    assert_eq!(reference.sim.ledger, cpu.sim.ledger);
    assert_eq!(reference.audit, cpu.audit);
    let report = c::check(&cpu).unwrap();
    assert!(!report.ready(), "missed operating work must remain visible");
    for recovered in &report.periods[1..] {
        assert_eq!(recovered.workshop_batches, 12);
        assert_eq!(recovered.municipal_batches, 12);
        assert_eq!(recovered.ore_delivered, 24);
        assert!(recovered.workshop_income > 0 && recovered.municipal_income > 0);
    }
    let evidence = cpu.evidence().unwrap();
    assert_eq!(evidence.people_food_deficit, 0);
    assert_eq!(evidence.wage_arrears, 0);
    assert!(!report.failures.iter().any(|f| f == "wages entered arrears"));
    println!(
        "supply interruption: {:?}; {:?}; {:?}",
        cpu.evidence().unwrap(),
        report.failures,
        report.periods
    );
}

#[test]
fn recurring_economy_is_ready_on_cpu_with_phase_reconstruction_and_dated_replay() {
    verify_recurring(c::scenario);
}

#[test]
fn stock_replenishment_is_ready_on_cpu_with_phase_reconstruction_and_dated_replay() {
    verify_recurring(c::spot_scenario);
}

fn verify_recurring(scenario: Scenario) {
    let (w, s) = scenario(c::Case::Baseline).unwrap();
    if w.minting
        .as_ref()
        .unwrap()
        .order_policy
        .as_ref()
        .unwrap()
        .peer_stocks
        .is_some()
    {
        assert!(
            w.prepaid_deliveries
                .iter()
                .all(|t| t.goods.resource != economics_compute_smoke::minting::ORE)
        );
    }
    let mut reference = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    assert!(c::check(&reference).is_err());
    let mut replay_state = s.clone();
    let mut replay_audit = reference.audit.clone();
    while reference.sim.state.month <= c::MONTHS {
        reference.audit.step(&mut reference.sim).unwrap();
        let batch = reference.sim.ledger.last().unwrap();
        let mut context = reference.sim.world.clone();
        let before = replay_state.clone();
        economics_compute_smoke::demography::replay(
            &mut context,
            &mut replay_state,
            batch,
            Backend::Reference,
            reference.sim.effect_limit,
        )
        .unwrap();
        replay_audit
            .record(&context, &before, batch, &replay_state)
            .unwrap();
        assert_eq!(replay_state, reference.sim.state);
    }
    assert_eq!(replay_audit, reference.audit);
    let mut cpu = Run::new(w, s, Backend::CubeCpu).unwrap();
    while cpu.sim.state.month <= c::MONTHS {
        cpu.audit.step(&mut cpu.sim).unwrap();
        let mut restored = Simulation::new(
            cpu.sim.world.clone(),
            cpu.sim.state.clone(),
            Backend::CubeCpu,
        )
        .unwrap();
        restored.ledger = cpu.sim.ledger;
        restored.reports = cpu.sim.reports;
        restored.effect_limit = cpu.sim.effect_limit;
        cpu.sim = restored;
    }
    assert_eq!(cpu.sim.world, reference.sim.world);
    assert_eq!(cpu.sim.state, reference.sim.state);
    assert_eq!(cpu.sim.ledger, reference.sim.ledger);
    assert_eq!(cpu.sim.reports, reference.sim.reports);
    assert_eq!(cpu.audit, reference.audit);
    let report = c::check(&cpu).unwrap();
    assert!(report.ready(), "{:?}", report.failures);
    assert_eq!(report, c::check(&reference).unwrap());
    assert_eq!(report.periods.len(), 3);
    for p in report.periods {
        assert!(p.workshop_income > 0);
        assert!(p.municipal_income > 0);
        assert_eq!(p.ore_delivered, 24);
    }
    for (agent, first) in [
        (acceptance::WORKSHOP, acceptance::SMITH),
        (acceptance::TOWN, economics_compute_smoke::minting::SUPPLIER),
    ] {
        let controller = &cpu.sim.world.agency[&agent];
        let d = &controller.history[0];
        assert_eq!(d.authorized_by, Some(first));
        assert!(d.accepted.is_some());
        assert_eq!(d.effective_month, 2);
        assert_eq!(controller.history.len(), c::MONTHS as usize);
    }
    let history = &cpu.sim.world.agency[&acceptance::TOWN].history;
    assert_ne!(history[0].authorized_by, history[12].authorized_by);
    assert_eq!(history[0].authorized_by, history[24].authorized_by);
    assert_eq!(cpu.sim.state.phase, Phase::Open);
}

#[test]
fn missing_supply_replacement_access_and_margin_fail_acceptance_with_matching_cpu_evidence() {
    verify_controls(c::scenario);
}

#[test]
fn spot_controls_fail_with_matching_cpu_evidence() {
    verify_controls(c::spot_scenario);
}

fn verify_controls(scenario: Scenario) {
    for case in c::Case::ALL.into_iter().filter(|c| *c != c::Case::Baseline) {
        let (w, s) = scenario(case).unwrap();
        let mut reference = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
        reference.through(c::MONTHS).unwrap();
        let expected = c::check(&reference).unwrap();
        assert!(!expected.ready(), "{case:?}");
        if case == c::Case::LatePayroll {
            assert!(
                expected
                    .failures
                    .iter()
                    .any(|f| f == "wages entered arrears")
            );
            assert_eq!(
                reference.evidence().unwrap().wage_arrears,
                0,
                "late wages should recover, preserving the historical failure"
            );
            assert!(
                reference
                    .sim
                    .ledger
                    .iter()
                    .filter_map(|b| b.employment.as_ref())
                    .flat_map(|b| &b.receipts)
                    .all(|r| r.reason != economics_compute_smoke::employment::Reason::Arrears)
            );
        }

        let mut cpu = Run::new(w, s, Backend::CubeCpu).unwrap();
        cpu.through(c::MONTHS).unwrap();
        assert_eq!(cpu.sim.state, reference.sim.state, "{case:?}");
        assert_eq!(cpu.audit, reference.audit, "{case:?}");
        assert_eq!(c::check(&cpu).unwrap(), expected, "{case:?}");
    }
}

#[test]
fn verdict_requires_all_cycles_and_monthly_catalog_order_does_not_change_result() {
    verify_reordering(c::scenario);
    verify_reordering(c::spot_scenario);
}

fn verify_reordering(scenario: Scenario) {
    let (mut w, s) = scenario(c::Case::Baseline).unwrap();
    let mut base = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    base.through(14).unwrap();
    assert!(c::check(&base).is_err());
    base.through(c::MONTHS).unwrap();
    w.agents.reverse();
    w.participants.reverse();
    w.definitions.reverse();
    w.resources.reverse();
    w.employment.reverse();
    w.prepaid_deliveries.reverse();
    if let Some(p) = &mut w
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks
    {
        p.quotes.reverse();
    }
    w.marketplaces[0].markets.reverse();
    let mut reordered = Run::new(w, s, Backend::Reference).unwrap();
    for month in 1..=c::MONTHS {
        reordered.through(month).unwrap();
    }
    assert_eq!(reordered.sim.state, base.sim.state);
    assert_eq!(reordered.sim.ledger, base.sim.ledger);
    assert_eq!(reordered.audit, base.audit);
    assert_eq!(reordered.sim.world.agency, base.sim.world.agency);
    assert_eq!(c::check(&reordered).unwrap(), c::check(&base).unwrap());
}
