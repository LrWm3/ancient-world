use economics_compute_smoke::{
    acceptance::{self, Run, TOWN, WORKSHOP, continuation as c, sales},
    compute::Backend,
};
#[test]
fn cash_only_cold_start_without_posted_quotes_remains_unready() {
    let (w, s) = sales::scenario(c::Case::Baseline).unwrap();
    for id in [WORKSHOP, TOWN] {
        assert_eq!(w.agency[&id].config.objectives.len(), 1);
    }
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(c::MONTHS).unwrap();
    println!(
        "evidence={:?}\nreadiness={:?}",
        run.evidence().unwrap(),
        acceptance::governed::check(&run).unwrap()
    );
    for id in [WORKSHOP, TOWN] {
        for d in &run.sim.world.agency[&id].history {
            if d.month == 1 || d.month == 13 || d.month == 25 || d.month == 37 {
                println!(
                    "agent={id} month={} chosen={:?} reason={} alternatives={:?}",
                    d.month,
                    d.accepted,
                    d.reason,
                    d.alternatives
                        .iter()
                        .map(|a| (&a.program, &a.losses, &a.failure))
                        .collect::<Vec<_>>()
                );
            }
        }
    }
    assert!(!acceptance::governed::check(&run).unwrap().ready());
    assert_eq!(run.evidence().unwrap().completed_mints, 0);
    assert_eq!(run.evidence().unwrap().workshop_batches, 0);
}
