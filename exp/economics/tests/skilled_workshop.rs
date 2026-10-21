//! Personal qualification composes with real hiring, capital upkeep and nine books.
use economics_compute_smoke::{
    acceptance::{
        SMITH, WORKSHOP, WORKSHOP_EMPLOYMENT, maintained_workshop, skilled_workshop as skilled,
    },
    compute::Backend,
    minting::{HOURS, REFINE},
    model::{Phase, Status},
    simulation::Simulation,
};
const MONTHS: u32 = 48;
const CPU_MONTHS: u32 = 12;
#[test]
fn skilled_worker_economy_recurs_with_real_wages_and_cpu_reconstruction() {
    let mut run = skilled::run(true, Backend::Reference).unwrap();
    let mut replay = run.sim.state.clone();
    let mut first_year = None;
    let mut completions = 0;
    let mut repairs = 0;
    while run.sim.state.month <= MONTHS {
        let before = run.sim.state.clone();
        run.audit.step(&mut run.sim).unwrap();
        let batch = run.sim.ledger.last().unwrap();
        if batch.month <= CPU_MONTHS {
            economics_compute_smoke::demography::replay(
                &mut run.sim.world.clone(),
                &mut replay,
                batch,
                Backend::Reference,
                run.sim.effect_limit,
            )
            .unwrap();
            assert_eq!(replay, run.sim.state);
        }
        if batch.phase == Phase::Productive {
            let mut used = 0;
            for tx in batch.all_transactions() {
                let Some(p) = &tx.process else { continue };
                if p.after.operator != WORKSHOP || p.after.status != Status::Completed {
                    continue;
                }
                let work = p.performer.as_ref().unwrap();
                assert_eq!(
                    (work.worker, work.agreement, work.month),
                    (SMITH, WORKSHOP_EMPLOYMENT, batch.month)
                );
                let hours = if p.after.definition == REFINE {
                    let qualified = completions >= skilled::QUALIFYING_COMPLETIONS;
                    assert_eq!(
                        tx.technique_use.as_ref().map(|t| t.technique),
                        qualified.then_some(skilled::TECHNIQUE)
                    );
                    completions += 1;
                    if qualified { 1 } else { 2 }
                } else {
                    assert_eq!(p.after.definition, maintained_workshop::REPAIR);
                    repairs += 1;
                    1
                };
                assert_eq!(work.capacity.quantity, hours);
                used += hours;
            }
            assert_eq!(
                before.balance(WORKSHOP, HOURS) - run.sim.state.balance(WORKSHOP, HOURS),
                used
            );
            assert_eq!(
                run.sim
                    .state
                    .employment
                    .earned
                    .get(&(WORKSHOP_EMPLOYMENT, batch.month))
                    .map_or(0, |e| e.delivered),
                used
            );
            assert_eq!(
                run.sim
                    .state
                    .practice
                    .get(&(SMITH, skilled::COMPETENCY))
                    .copied()
                    .unwrap_or(0),
                completions
            );
            assert!(
                !run.sim
                    .state
                    .practice
                    .contains_key(&(WORKSHOP, skilled::COMPETENCY))
            );
        }
        if run.sim.state.month == CPU_MONTHS + 1 && run.sim.state.phase == Phase::Open {
            first_year = Some((
                run.sim.world.clone(),
                run.sim.state.clone(),
                run.sim.ledger.clone(),
                run.audit.clone(),
            ));
        }
        if batch.phase == Phase::Close && batch.month.is_multiple_of(CPU_MONTHS) {
            println!(
                "boundary {:?}; practice={completions}, repairs={repairs}",
                run.evidence().unwrap()
            );
        }
    }
    let readiness = maintained_workshop::check(&run).unwrap();
    println!("{readiness:?}");
    assert!(readiness.ready(), "{readiness:?}");
    assert!(repairs >= 6 && completions > skilled::QUALIFYING_COMPLETIONS);
    let mut cpu = skilled::run(true, Backend::CubeCpu).unwrap();
    while cpu.sim.state.month <= CPU_MONTHS {
        cpu.audit.step(&mut cpu.sim).unwrap();
        let old = cpu.sim;
        let mut restored = Simulation::new(old.world, old.state, Backend::CubeCpu).unwrap();
        restored.ledger = old.ledger;
        restored.reports = old.reports;
        restored.effect_limit = old.effect_limit;
        cpu.sim = restored;
    }
    let (w, s, ledger, audit) = first_year.unwrap();
    assert_eq!(cpu.sim.world, w);
    assert_eq!(cpu.sim.state, s);
    assert_eq!(cpu.sim.ledger, ledger);
    assert_eq!(cpu.audit, audit);
}
