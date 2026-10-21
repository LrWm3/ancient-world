//! Fresh monthly labor consent composes with the financed nine-agent economy.
use economics_compute_smoke::{
    acceptance::{Run, SMITH, TOWN, WORKSHOP, continuation, voluntary_employment},
    compute::Backend,
    minting::{COIN, HOURS, WHEAT},
    model::Phase,
    simulation::Simulation,
};
const MONTHS: u32 = 48;
const STARTUP: u32 = 12;
#[test]
fn voluntary_economy_cpu_continuation_replay_and_catalog_order() {
    let (w, s) = voluntary_employment::scenario().unwrap();
    let (_, original) = voluntary_employment::with_startup_food(false).unwrap();
    for resource in [COIN, WHEAT] {
        let sum = |state: &economics_compute_smoke::model::State| -> i64 {
            state
                .balances
                .iter()
                .filter(|((_, r), _)| *r == resource)
                .map(|(_, q)| i64::from(*q))
                .sum()
        };
        assert_eq!(sum(&s), sum(&original));
    }
    assert!(s.employment.earned.is_empty());
    assert!(
        w.employment
            .iter()
            .all(|t| w.employment_offers.contains(&t.id))
    );
    let mut reference = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut replay = s.clone();
    let mut startup = None;
    while reference.sim.state.month <= MONTHS {
        reference.audit.step(&mut reference.sim).unwrap();
        let batch = reference.sim.ledger.last().unwrap();
        if batch.month <= STARTUP {
            let mut world = reference.sim.world.clone();
            economics_compute_smoke::demography::replay(
                &mut world,
                &mut replay,
                batch,
                Backend::Reference,
                reference.sim.effect_limit,
            )
            .unwrap();
            assert_eq!(replay, reference.sim.state);
        }
        if reference.sim.state.month == STARTUP + 1 && reference.sim.state.phase == Phase::Open {
            startup = Some((
                reference.sim.state.clone(),
                reference.sim.ledger.clone(),
                reference.audit.clone(),
            ));
        }
    }
    let readiness = continuation::check(&reference).unwrap();
    assert!(readiness.ready(), "{readiness:?}");
    let evidence = reference.evidence().unwrap();
    println!("{evidence:?}\n{readiness:?}");
    let decisions: Vec<_> = reference
        .sim
        .ledger
        .iter()
        .filter_map(|b| b.employment.as_ref())
        .flat_map(|b| &b.recruitment)
        .collect();
    assert!(decisions.iter().any(|d| d.selected == 0));
    for d in decisions.iter().filter(|d| d.selected > 0) {
        assert!(!d.hypothetical);
        assert!(d.governor.is_some());
        let chosen = d
            .alternatives
            .iter()
            .find(|a| a.hours == d.selected)
            .unwrap();
        assert_eq!(chosen.employer_accepts, Some(true));
        assert_eq!(chosen.worker_accepts, Some(true));
        let earned = &reference.sim.state.employment.earned[&(d.agreement, d.month)];
        assert_eq!(earned.claim.outstanding(), 0);
        let employer = reference
            .sim
            .world
            .employment
            .iter()
            .find(|t| t.id == d.agreement)
            .unwrap()
            .employer;
        let used: i32 = reference
            .sim
            .ledger
            .iter()
            .filter(|b| b.month == d.month && b.phase == Phase::Productive)
            .flat_map(|b| b.all_transactions())
            .flat_map(|t| &t.effects)
            .filter(|e| e.account == (employer, HOURS) && e.delta < 0)
            .map(|e| -e.delta)
            .sum();
        assert_eq!(used, d.selected);
        for p in d.baseline.iter().chain(
            d.alternatives
                .iter()
                .filter_map(|a| a.projection.as_ref().ok()),
        ) {
            assert!(p.conditional_labor.is_empty());
            assert!(p.peer_attempts.iter().all(|a| a.month > d.month));
            if d.peer_expectations.as_ref().unwrap().policy
                == economics_compute_smoke::agency::expectations::Policy::AcceptedOnly
            {
                assert!(p.peer_attempts.is_empty());
            }
        }
    }
    assert!(decisions.iter().any(|d| {
        d.peer_expectations
            .as_ref()
            .is_some_and(|p| !p.evidence.is_empty())
    }));
    let mut cpu = Run::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    while cpu.sim.state.month <= MONTHS {
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
    let mut reordered = w;
    reordered.agents.reverse();
    reordered.participants.reverse();
    reordered.resources.reverse();
    reordered.definitions.reverse();
    reordered.employment.reverse();
    for venue in &mut reordered.marketplaces {
        venue.markets.reverse();
    }
    let mut shuffled = Run::new(reordered, s, Backend::Reference).unwrap();
    shuffled.through(STARTUP).unwrap();
    let (state, ledger, audit) = startup.unwrap();
    assert_eq!(shuffled.sim.state, state);
    assert_eq!(shuffled.sim.ledger, ledger);
    assert_eq!(shuffled.audit, audit);
}

#[test]
fn withholding_labor_or_payroll_blocks_the_real_supply_chain() {
    for withhold_worker in [true, false] {
        let (mut w, s) = voluntary_employment::scenario().unwrap();
        if withhold_worker {
            w.recruitment.workers.remove(&SMITH);
        } else {
            for actor in [TOWN, WORKSHOP] {
                w.agency
                    .get_mut(&actor)
                    .unwrap()
                    .config
                    .hiring
                    .as_mut()
                    .unwrap()
                    .payroll
                    .quantity = 0;
            }
        }
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(STARTUP).unwrap();
        let e = run.evidence().unwrap();
        assert_eq!(e.workshop_batches, 0);
        assert_eq!(e.municipal_batches, 0);
        assert_eq!(e.completed_mints, 0);
        assert!(run.sim.state.employment.earned.is_empty());
        assert!(e.people_food_deficit > 0);
    }
}

#[test]
fn terminal_food_preferences_and_initial_distribution_have_distinct_effects() {
    use economics_compute_smoke::acceptance::WORKSHOP_EMPLOYMENT;
    let mut production = Vec::new();
    for cash_only in [true, false] {
        let (mut w, s) = voluntary_employment::scenario().unwrap();
        if cash_only {
            w.recruitment
                .workers
                .get_mut(&SMITH)
                .unwrap()
                .reserves
                .retain(|a| a.resource == COIN);
        }
        let mut run = Run::new(w, s, Backend::Reference).unwrap();
        run.through(STARTUP).unwrap();
        production.push(run.evidence().unwrap().workshop_batches);
        if cash_only {
            let d = run
                .sim
                .ledger
                .iter()
                .filter_map(|b| b.employment.as_ref())
                .flat_map(|b| &b.recruitment)
                .find(|d| d.month == 7 && d.agreement == WORKSHOP_EMPLOYMENT)
                .unwrap();
            let p = d.alternatives[0].projection.as_ref().unwrap();
            let b = d.baseline.as_ref().unwrap();
            assert_eq!(&p.worker[..3], &b.worker[..3]);
            assert!(p.worker[3] > b.worker[3]);
            assert!(p.worker_closing_stocks[&WHEAT] > b.worker_closing_stocks[&WHEAT]);
            assert!(p.worker_closing_stocks[&COIN] < b.worker_closing_stocks[&COIN]);
            assert_eq!(d.alternatives[0].employer_accepts, Some(true));
            assert_eq!(d.alternatives[0].worker_accepts, Some(false));
        }
    }
    assert!(production[1] > production[0], "{production:?}");
    let (w, s) = voluntary_employment::with_startup_food(false).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(STARTUP).unwrap();
    assert!(run.evidence().unwrap().smith_food_deficit > 0);
}

#[test]
fn a_temporary_labor_absence_creates_no_wage_and_work_resumes() {
    const ABSENCE: u32 = 15;
    let (mut w, s) = voluntary_employment::scenario().unwrap();
    w.capacity_overrides.insert((ABSENCE, SMITH), 0);
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(24).unwrap();
    assert!(
        run.sim
            .state
            .employment
            .earned
            .keys()
            .all(|(_, month)| *month != ABSENCE)
    );
    for employer in [WORKSHOP, TOWN] {
        let ids: Vec<_> = run
            .sim
            .world
            .employment
            .iter()
            .filter(|t| t.employer == employer)
            .map(|t| t.id)
            .collect();
        assert!(
            run.sim
                .state
                .employment
                .earned
                .keys()
                .any(|(id, month)| ids.contains(id) && *month > ABSENCE)
        );
    }
    let e = run.evidence().unwrap();
    println!("temporary absence: {e:?}");
    assert_eq!(e.wage_arrears, 0);
    assert_eq!(e.reconciled_agents, 9);
}
