use economics_compute_smoke::{
    acceptance::{
        WORKSHOP, WORKSHOP_EMPLOYMENT,
        maintained_workshop::{self as m, Case},
    },
    compute::Backend,
    minting::{HOURS, ORE, REFINE},
    model::{Phase, Status},
    simulation::Simulation,
};
const MONTHS: u32 = 48;
const REPLAY_MONTHS: u32 = 12;

#[test]
fn maintained_economy_recurs_and_cpu_reconstructs_across_repairs() {
    let mut reference = m::run(Case::Discovered, Backend::Reference).unwrap();
    let mut replay = reference.sim.state.clone();
    let mut first_year = None;
    while reference.sim.state.month <= MONTHS {
        let before = reference.sim.state.clone();
        reference.audit.step(&mut reference.sim).unwrap();
        let batch = reference.sim.ledger.last().unwrap();
        if batch.month <= REPLAY_MONTHS {
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
        if batch.phase == Phase::Productive {
            let completed: Vec<_> = batch
                .all_transactions()
                .filter_map(|t| t.process.as_ref())
                .filter(|p| p.after.operator == WORKSHOP && p.after.status == Status::Completed)
                .collect();
            let repair = completed.iter().any(|p| p.after.definition == m::REPAIR);
            let refine = completed.iter().any(|p| p.after.definition == REFINE);
            assert!(
                !(repair && refine),
                "one asset cannot be used and repaired together"
            );
            if repair {
                assert_eq!(before.equipment[&m::MACHINE].remaining_uses, 0);
                assert_eq!(
                    reference.sim.state.equipment[&m::MACHINE].remaining_uses,
                    m::LIFETIME
                );
                assert_eq!(
                    before.balance(WORKSHOP, ORE) - reference.sim.state.balance(WORKSHOP, ORE),
                    1
                );
            }
            if repair || refine {
                assert_eq!(
                    before.balance(WORKSHOP, HOURS) - reference.sim.state.balance(WORKSHOP, HOURS),
                    1
                );
                assert_eq!(
                    reference.sim.state.employment.earned[&(WORKSHOP_EMPLOYMENT, batch.month)]
                        .delivered,
                    1
                );
            }
        }
        if batch.phase == Phase::Close && batch.month.is_multiple_of(REPLAY_MONTHS) {
            println!("reference boundary: {:?}", reference.evidence().unwrap());
        }
        if reference.sim.state.month == REPLAY_MONTHS + 1
            && reference.sim.state.phase == Phase::Open
        {
            first_year = Some((
                reference.sim.world.clone(),
                reference.sim.state.clone(),
                reference.sim.ledger.clone(),
                reference.audit.clone(),
            ));
        }
    }
    // The repair boundary exposes a tempting but unnecessary forward. It must
    // lose to the buyer's own independently funded operating alternative.
    assert!(
        reference.sim.world.agency[&WORKSHOP]
            .forward_reviews
            .iter()
            .any(|r| {
                r.assessment.as_ref().is_ok_and(|a| {
                    a.comparisons.values().all(|c| c.proposed <= c.baseline)
                        && a.comparisons.values().any(|c| c.proposed < c.baseline)
                        && !a.mutually_beneficial()
                        && a.comparisons.values().any(|c| {
                            c.self_funded
                                .as_ref()
                                .is_some_and(|o| o.losses < c.proposed)
                        })
                })
            })
    );
    let readiness = m::check(&reference).unwrap();
    println!("{:?}\n{readiness:?}", reference.evidence().unwrap());
    assert!(readiness.ready(), "{readiness:?}");
    assert!(
        reference.sim.world.agency[&WORKSHOP]
            .config
            .programs
            .is_empty()
    );
    let mut cpu = m::run(Case::Discovered, Backend::CubeCpu).unwrap();
    while cpu.sim.state.month <= REPLAY_MONTHS {
        cpu.audit.step(&mut cpu.sim).unwrap();
        let old = cpu.sim;
        let mut resumed = Simulation::new(old.world, old.state, Backend::CubeCpu).unwrap();
        resumed.ledger = old.ledger;
        resumed.reports = old.reports;
        resumed.effect_limit = old.effect_limit;
        cpu.sim = resumed;
    }
    let (w, s, ledger, audit) = first_year.unwrap();
    assert_eq!(cpu.sim.world, w);
    assert_eq!(cpu.sim.state, s);
    assert_eq!(cpu.sim.ledger, ledger);
    assert_eq!(cpu.audit, audit);
    assert!(
        s.processes
            .values()
            .any(|p| p.definition == m::REPAIR && p.status == Status::Completed)
    );
}

fn check_interruption(case: &str) {
    use economics_compute_smoke::{
        acceptance::{SMITH, WORKSHOP_TYPE},
        opportunities::Action,
    };
    let mut opening = m::run(Case::Discovered, Backend::Reference).unwrap();
    opening.through(7).unwrap();
    assert_eq!(opening.sim.state.equipment[&m::MACHINE].remaining_uses, 0);
    let mut w = opening.sim.world.clone();
    match case {
        "worker_dated" => {
            w.capacity_overrides.insert((8, SMITH), 0);
        }
        "law" => {
            w.transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(WORKSHOP_TYPE, Action::Process(m::REPAIR)));
        }
        "worker" => {
            w.participants
                .iter_mut()
                .find(|p| p.agent == SMITH)
                .unwrap()
                .capacity
                .quantity = 0;
        }
        _ => {
            w.definitions
                .iter_mut()
                .find(|d| d.id == m::REPAIR)
                .unwrap()
                .enabled = false;
        }
    }
    let mut sim = Simulation::new(w, opening.sim.state.clone(), Backend::Reference).unwrap();
    sim.ledger = opening.sim.ledger.clone();
    sim.reports = opening.sim.reports.clone();
    let mut audit = opening.audit.clone();
    while sim.state.month == 8 {
        audit.step(&mut sim).unwrap();
    }
    assert_eq!(sim.state.equipment[&m::MACHINE].remaining_uses, 0, "{case}");
    assert!(
        !sim.state
            .employment
            .earned
            .contains_key(&(WORKSHOP_EMPLOYMENT, 8)),
        "{case}"
    );
    match case {
        "worker_dated" => {}
        "law" => {
            sim.world
                .transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .insert((WORKSHOP_TYPE, Action::Process(m::REPAIR)));
        }
        "worker" => {
            sim.world
                .participants
                .iter_mut()
                .find(|p| p.agent == SMITH)
                .unwrap()
                .capacity
                .quantity = 2;
        }
        _ => {
            sim.world
                .definitions
                .iter_mut()
                .find(|d| d.id == m::REPAIR)
                .unwrap()
                .enabled = true;
        }
    }
    while sim.state.month <= 16 {
        audit.step(&mut sim).unwrap();
    }
    assert_eq!(
        sim.reports
            .iter()
            .map(|r| r.deficit(economics_compute_smoke::minting::NUTRITION))
            .sum::<i32>(),
        0,
        "{case}: recovery food deficit"
    );
    println!(
        "{case}: completed workshop work {:?}, condition {}, food {}, cash {}",
        sim.state
            .processes
            .values()
            .filter(|p| p.operator == WORKSHOP && p.status == Status::Completed)
            .map(|p| (p.definition, p.start))
            .collect::<Vec<_>>(),
        sim.state.equipment[&m::MACHINE].remaining_uses,
        sim.state
            .balance(SMITH, economics_compute_smoke::minting::WHEAT),
        sim.state
            .balance(SMITH, economics_compute_smoke::minting::COIN)
    );
    assert!(
        sim.state.processes.values().any(|p| p.operator == WORKSHOP
            && p.definition == m::REPAIR
            && p.start > 8
            && p.status == Status::Completed),
        "{case}: repair never resumed"
    );
    let refined = sim.state.processes.values().any(|p| {
        p.operator == WORKSHOP
            && p.definition == REFINE
            && p.start > 9
            && p.status == Status::Completed
    });
    assert!(refined, "{case}: production never resumed");
    for e in sim.state.employment.earned.values() {
        assert_eq!(e.claim.outstanding(), 0);
    }
}

#[test]
fn interruption_dated_worker_absence() {
    check_interruption("worker_dated");
}
#[test]
fn interruption_unannounced_worker_withdrawal() {
    check_interruption("worker");
}
#[test]
fn interruption_repair_authority() {
    check_interruption("law");
}
#[test]
fn interruption_repair_recipe() {
    check_interruption("disabled");
}
