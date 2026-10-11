use economics_compute_smoke::{
    acceptance::{self, Run, WORKSHOP, continuation as c, expectations as e, governed as g},
    agency::objectives::{Metric, Objective, Scope},
    compute::Backend,
    minting::{COIN, ISSUER},
    model::*,
};
fn compare(w: World, s: State) -> Run {
    let mut reference = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    reference.through(c::MONTHS).unwrap();
    let mut cpu = Run::new(w, s, Backend::CubeCpu).unwrap();
    cpu.through(c::MONTHS).unwrap();
    assert_eq!(cpu.sim.world, reference.sim.world);
    assert_eq!(cpu.sim.state, reference.sim.state);
    assert_eq!(cpu.sim.ledger, reference.sim.ledger);
    assert_eq!(cpu.audit, reference.audit);
    assert_eq!(g::check(&cpu).unwrap(), g::check(&reference).unwrap());
    cpu
}
#[test]
fn expected_demand_cannot_make_broken_supply_access_margin_or_late_payroll_ready() {
    for case in c::Case::ALL
        .into_iter()
        .filter(|case| *case != c::Case::Baseline)
    {
        let (w, s) = e::scenario(case).unwrap();
        let run = compare(w, s);
        let report = g::check(&run).unwrap();
        println!(
            "{case:?}: {:?}; {:?}",
            run.evidence().unwrap(),
            report.failures
        );
        assert!(!report.ready(), "{case:?}");
        if case == c::Case::LatePayroll {
            assert!(report.failures.iter().any(|f| f == "wages entered arrears"));
        }
    }
}
#[test]
fn missing_buyer_or_satisfied_buyer_goals_do_not_create_speculative_sales() {
    for control in ["absent", "satisfied"] {
        let (mut w, s) = e::scenario(c::Case::Baseline).unwrap();
        if control == "absent" {
            w.agency.remove(&ISSUER);
        } else {
            let config = &mut w.agency.get_mut(&ISSUER).unwrap().config;
            let objectives = vec![Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: COIN,
                    target: 0,
                },
            }];
            config.objectives = objectives.clone();
            for preference in config.preferences.values_mut() {
                *preference = objectives.clone();
            }
        }
        let run = compare(w, s);
        assert!(!g::check(&run).unwrap().ready());
        assert!(
            run.sim.world.agency[&WORKSHOP]
                .history
                .iter()
                .flat_map(|d| &d.alternatives)
                .filter_map(|a| a.projection.as_ref())
                .all(|p| p.peer_attempts.is_empty())
        );
        assert!(!acceptance::accepted_deals(&run.sim).any(|d| d.month > 12 && d.buyer == ISSUER));
        println!("{control}: {:?}", run.evidence().unwrap());
    }
}
