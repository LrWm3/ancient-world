use economics_compute_smoke::{
    activities::{Target, WorkOrder},
    agency,
    compute::Backend,
    employment::recruitment::{Mandate, Worker},
    equipment::{PracticeRule, Technique},
    minting::*,
    model::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const SKILL: u32 = 700;
const TECHNIQUE: u32 = 700;
fn fixture(voluntary: bool) -> (World, State) {
    let (mut w, mut s) = agency::scenario::public_work().unwrap();
    w.employment[0].from = 1;
    w.employment[0].through = 12;
    if voluntary {
        w.employment_offers.insert(1);
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
    } else {
        w.agency.clear();
    }
    w.activities.orders = vec![WorkOrder {
        agent: ISSUER,
        definition: GATHER,
        priority: 0,
        target: Target::Stock(Amount::new(FIREWOOD, 100)),
    }];
    w.activities.performers.insert((GATHER, 0), HOURS);
    w.practice_rules.push(PracticeRule {
        definition: GATHER,
        stage: 0,
        competency: SKILL,
        points: 1,
    });
    w.techniques.push(Technique {
        id: TECHNIQUE,
        definition: GATHER,
        stage: 0,
        equipment_kind: None,
        competency: Some((SKILL, 2)),
        output_multiplier: 1,
        wear: 0,
        services: vec![Amount::new(HOURS, 1)],
    });
    s.phase = Phase::Acquire;
    s.balances.insert((WORKER, HOURS), 2);
    s.balances.insert((ISSUER, HOURS), 0);
    s.balances.insert((ISSUER, COIN), 100);
    w.storage.capacities.insert(ISSUER, 200);
    (w, s)
}
fn productive(w: World, s: State) -> Simulation {
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    sim
}
fn work(sim: &Simulation) -> &Transaction {
    sim.ledger
        .last()
        .unwrap()
        .transactions
        .iter()
        .find(|t| {
            t.process.as_ref().is_some_and(|p| {
                p.after.definition == GATHER && p.after.status == Status::Completed
            })
        })
        .unwrap()
}
#[test]
fn personal_learning_changes_later_work_and_replays_on_cpu() {
    let (w, s) = fixture(false);
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    reference.run_months(4).unwrap();
    let jobs: Vec<_> = reference
        .ledger
        .iter()
        .flat_map(|b| &b.transactions)
        .filter(|t| {
            t.process.as_ref().is_some_and(|p| {
                p.after.definition == GATHER && p.after.status == Status::Completed
            })
        })
        .collect();
    assert_eq!(jobs.len(), 4);
    for (index, tx) in jobs.iter().enumerate() {
        let p = tx.process.as_ref().unwrap();
        let performed = p.performer.as_ref().unwrap();
        assert_eq!(
            (p.after.operator, p.after.beneficiary, performed.worker),
            (ISSUER, ISSUER, WORKER)
        );
        assert_eq!(performed.capacity.quantity, if index < 2 { 2 } else { 1 });
        assert_eq!(tx.technique_use.is_some(), index >= 2);
    }
    assert_eq!(reference.state.practice.get(&(WORKER, SKILL)), Some(&4));
    assert!(!reference.state.practice.contains_key(&(ISSUER, SKILL)));
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    while cpu.state.month < reference.state.month {
        cpu.step().unwrap();
        let mut resumed =
            Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu).unwrap();
        resumed.ledger = cpu.ledger;
        resumed.reports = cpu.reports;
        cpu = resumed;
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    let mut replay = s;
    for batch in &cpu.ledger {
        commit(
            &w,
            &mut replay,
            batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
    }
    assert_eq!(replay, cpu.state);
}
#[test]
fn voluntary_hiring_buys_and_pays_only_the_skilled_hours_used() {
    for qualified in [false, true] {
        let (w, mut s) = fixture(true);
        if qualified {
            s.practice.insert((WORKER, SKILL), 2);
        }
        let mut sim = productive(w, s);
        let hours = if qualified { 1 } else { 2 };
        assert_eq!(sim.state.employment.earned[&(1, 1)].delivered, hours);
        sim.step().unwrap();
        assert_eq!(
            work(&sim)
                .process
                .as_ref()
                .unwrap()
                .performer
                .as_ref()
                .unwrap()
                .capacity
                .quantity,
            hours
        );
        sim.run_months(1).unwrap();
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.settled, hours);
    }
}
#[test]
fn absent_nonconsenting_or_unqualified_workers_do_not_lend_their_skill() {
    for case in 0..5 {
        let (mut w, mut s) = fixture(true);
        // Organizational skill never qualifies its employee.
        s.practice.insert((ISSUER, SKILL), 100);
        match case {
            0 => {
                s.balances.insert((WORKER, HOURS), 0);
                s.practice.insert((WORKER, SKILL), 2);
            }
            1 => {
                w.recruitment.workers.clear();
                s.practice.insert((WORKER, SKILL), 2);
            }
            2 => {
                s.balances.insert((ISSUER, COIN), 0);
            }
            3 => {
                w.definitions
                    .iter_mut()
                    .find(|d| d.id == GATHER)
                    .unwrap()
                    .enabled = false;
            }
            _ => {}
        }
        let mut sim = productive(w, s);
        sim.step().unwrap();
        if case < 4 {
            assert!(sim.state.employment.work.is_empty());
            assert!(
                !sim.state.practice.contains_key(&(WORKER, SKILL))
                    || sim.state.practice[&(WORKER, SKILL)] == 2
            );
        } else {
            assert!(work(&sim).technique_use.is_none());
            assert_eq!(sim.state.practice[&(WORKER, SKILL)], 1);
        }
    }
}
#[test]
fn missing_forged_and_overspent_participation_is_rejected_atomically() {
    let (w, s) = fixture(false);
    let mut sim = productive(w, s);
    let opening = sim.state.clone();
    sim.step().unwrap();
    let batch = sim.ledger.last().unwrap().clone();
    for case in 0..5 {
        let mut forged = batch.clone();
        let p = forged
            .transactions
            .iter_mut()
            .find_map(|t| t.process.as_mut())
            .unwrap();
        match case {
            0 => p.performer = None,
            1 => p.performer.as_mut().unwrap().worker = ISSUER,
            2 => p.performer.as_mut().unwrap().capacity.quantity = 1,
            3 => p.performer.as_mut().unwrap().month = 2,
            _ => p.performer.as_mut().unwrap().agreement = 999,
        }
        let mut untouched = opening.clone();
        assert!(
            commit(
                &sim.world,
                &mut untouched,
                &forged,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(untouched, opening);
    }
    let mut forged = batch.clone();
    let mut second = work(&sim).clone();
    second.process.as_mut().unwrap().after.id += 1;
    forged.transactions.push(second);
    // Give anonymous spare capacity so the failure is personal provenance,
    // not just the employer's gross resource budget.
    let mut opening = opening;
    opening.balances.insert((ISSUER, HOURS), 4);
    let before = opening.clone();
    assert!(
        commit(
            &sim.world,
            &mut opening,
            &forged,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(opening, before);
}
#[test]
fn same_batch_learning_cannot_unlock_a_second_job_and_hours_cannot_be_reused() {
    let (mut w, mut s) = fixture(false);
    let mut other = w.definition(GATHER).clone();
    other.id = 701;
    w.definitions.push(other);
    w.activities.performers.insert((701, 0), HOURS);
    let mut technique = w.techniques.last().unwrap().clone();
    technique.id = 701;
    technique.definition = 701;
    w.techniques.push(technique);
    w.activities.orders.push(WorkOrder {
        agent: ISSUER,
        definition: 701,
        priority: 1,
        target: Target::Stock(Amount::new(FIREWOOD, 100)),
    });
    w.transaction_policy.as_mut().unwrap().permissions.insert((
        economics_compute_smoke::opportunities::STATE_TYPE,
        economics_compute_smoke::opportunities::Action::Process(701),
    ));
    s.practice.insert((WORKER, SKILL), 1);
    w.employment[0].capacity.quantity = 3;
    s.balances.insert((WORKER, HOURS), 3);
    let mut sim = productive(w, s);
    sim.step().unwrap();
    assert_eq!(sim.state.employment.work.len(), 1);
    assert_eq!(sim.state.practice[&(WORKER, SKILL)], 2);
    assert!(
        sim.ledger
            .last()
            .unwrap()
            .receipts
            .iter()
            .any(|r| r.definition == Some(701) && r.reason == Reason::InsufficientCapacity)
    );
}
#[test]
fn paid_but_failed_work_does_not_create_practice() {
    let (mut w, s) = fixture(false);
    w.definitions
        .iter_mut()
        .find(|d| d.id == GATHER)
        .unwrap()
        .stages[0]
        .entry_inputs
        .push(Amount::new(METAL, 1));
    let mut sim = productive(w, s);
    sim.state.balances.insert((ISSUER, METAL), 0);
    sim.step().unwrap();
    assert!(sim.state.employment.work.is_empty());
    assert!(sim.state.practice.is_empty());
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.settled, 2);
}
#[test]
fn unsupported_multi_month_and_zero_labor_rules_fail_closed() {
    for case in 0..2 {
        let (mut w, s) = fixture(false);
        if case == 0 {
            w.definitions
                .iter_mut()
                .find(|d| d.id == GATHER)
                .unwrap()
                .stages[0]
                .months = 2;
        } else {
            w.techniques.last_mut().unwrap().services.clear();
        }
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}

#[test]
fn replacement_cannot_inherit_personal_qualification_or_erase_provenance() {
    let (mut w, s) = fixture(false);
    w.employment[0].through = 2;
    let mut replacement = w.employment[0].clone();
    replacement.id = 2;
    replacement.worker = SUPPLIER;
    replacement.from = 3;
    replacement.through = 12;
    w.employment.push(replacement);
    w.participants
        .iter_mut()
        .find(|p| p.agent == SUPPLIER)
        .unwrap()
        .capacity
        .quantity = 2;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(4).unwrap();
    assert_eq!(sim.state.practice[&(WORKER, SKILL)], 2);
    assert_eq!(sim.state.practice[&(SUPPLIER, SKILL)], 2);
    assert_eq!(
        sim.state
            .employment
            .work
            .values()
            .filter(|p| p.worker == WORKER)
            .count(),
        2
    );
    assert_eq!(
        sim.state
            .employment
            .work
            .values()
            .filter(|p| p.worker == SUPPLIER)
            .count(),
        2
    );
    assert!(
        sim.ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter(|t| t
                .process
                .as_ref()
                .is_some_and(|p| p.after.definition == GATHER))
            .all(|t| t.technique_use.is_none())
    );
    let mut missing = sim.state.clone();
    missing.employment.work.clear();
    assert!(Simulation::new(sim.world.clone(), missing, Backend::Reference).is_err());
    let mut altered = sim.state.clone();
    altered
        .employment
        .work
        .values_mut()
        .next()
        .unwrap()
        .capacity
        .quantity = 1;
    assert!(Simulation::new(sim.world.clone(), altered, Backend::Reference).is_err());
    let mut altered = sim.state.clone();
    altered
        .employment
        .work
        .values_mut()
        .next()
        .unwrap()
        .technique = Some(u32::MAX);
    assert!(Simulation::new(sim.world.clone(), altered, Backend::Reference).is_err());
    let mut cpu = Simulation::new(sim.world.clone(), fixture(false).1, Backend::CubeCpu).unwrap();
    cpu.run_months(4).unwrap();
    assert_eq!(cpu.state, sim.state);
    assert_eq!(cpu.ledger, sim.ledger);
}

#[test]
fn resold_hours_are_not_personal_work() {
    let (mut w, mut s) = fixture(false);
    let mut incoming = w.employment[0].clone();
    incoming.id = 0;
    incoming.employer = WORKER;
    incoming.worker = SUPPLIER;
    incoming.rank = 0;
    w.employment.push(incoming);
    s.balances.insert((SUPPLIER, HOURS), 2);
    s.balances.insert((WORKER, COIN), 10);
    let mut sim = productive(w, s);
    assert!(sim.state.employment.earned.contains_key(&(0, 1)));
    sim.step().unwrap();
    assert!(sim.state.employment.work.is_empty());
    assert!(sim.state.practice.is_empty());
}

#[test]
fn local_forecast_adapters_reject_named_work_before_mutating() {
    use economics_compute_smoke::{
        allocation::Policy as Allocation,
        competition,
        composition::{
            self, Budget, Scope, Strategy,
            continuation::{Controller, Policy},
        },
        work_choice,
    };
    let (w, s) = fixture(false);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    assert!(!sim.state.employment.work.is_empty());
    let before = sim.clone();
    let budget = Budget {
        expansions: 2,
        forecasts: 2,
        months: 1,
    };
    let scope = Scope::Person(WORKER);
    let mut controller = Controller::new(scope.clone(), Strategy::Beam, budget, Policy::Monthly);
    assert!(
        controller
            .step(&mut sim)
            .unwrap_err()
            .contains("person-local adapter")
    );
    assert_eq!(sim.state, before.state);
    assert_eq!(sim.world, before.world);
    assert_eq!(sim.ledger, before.ledger);
    assert!(controller.history.is_empty());
    assert!(
        competition::prepare_many(&sim, &[], 7, Allocation::StablePriority, &[])
            .unwrap_err()
            .contains("person-local adapter")
    );
    assert!(
        composition::choose(&sim, &scope, Strategy::Beam, budget)
            .unwrap_err()
            .contains("person-local adapter")
    );
    let mut w = sim.world.clone();
    w.work_choice = Some(work_choice::Config {
        agent: WORKER,
        horizon: 2,
        values: Default::default(),
    });
    assert!(
        economics_compute_smoke::performers::validate(&w, &sim.state)
            .unwrap_err()
            .contains("person-local adapter")
    );
    assert!(Simulation::new(w, sim.state.clone(), Backend::Reference).is_err());
    let mut w = sim.world.clone();
    w.competition = Some(competition::Config {
        seed: 7,
        policy: Allocation::StablePriority,
    });
    assert!(
        economics_compute_smoke::performers::validate(&w, &sim.state)
            .unwrap_err()
            .contains("person-local adapter")
    );
    assert!(Simulation::new(w, sim.state, Backend::Reference).is_err());
}

#[test]
fn named_single_month_stages_keep_distinct_history_and_check_dates() {
    let (mut w, s) = fixture(false);
    let d = w.definitions.iter_mut().find(|d| d.id == GATHER).unwrap();
    d.stages.push(d.stages[0].clone());
    w.activities.performers.insert((GATHER, 1), HOURS);
    let mut sim = productive(w, s);
    sim.step().unwrap();
    let process = sim
        .state
        .processes
        .values()
        .find(|p| p.definition == GATHER)
        .unwrap()
        .id;
    assert_eq!(sim.state.processes[&process].stage, 1);
    assert!(sim.state.employment.work.contains_key(&(1, process, 0)));
    while sim.state.month < 2 || sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    sim.step().unwrap();
    assert_eq!(sim.state.processes[&process].status, Status::Completed);
    assert!(sim.state.employment.work.contains_key(&(2, process, 1)));
    assert_eq!(sim.state.practice[&(WORKER, SKILL)], 1);
    let mut bad = sim.state.clone();
    bad.processes.get_mut(&process).unwrap().start = u32::MAX;
    // Isolate the second-stage date: stage zero would reject its missing row first.
    let mut w = sim.world.clone();
    w.activities.performers.remove(&(GATHER, 0));
    assert!(
        economics_compute_smoke::performers::validate(&w, &bad)
            .unwrap_err()
            .contains("date overflow")
    );
}
