use economics_compute_smoke::{
    acceptance::{self, Run, WORKSHOP, continuation as c, governed as g, sales as e},
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
fn missing_satisfied_or_non_crossing_buyer_does_not_create_sale_income() {
    for control in ["absent", "satisfied", "noncrossing"] {
        let (mut w, s) =
            e::with_startup(c::Case::Baseline, e::Startup::SupplyChainInventory).unwrap();
        if control == "absent" {
            w.agency.remove(&ISSUER);
        } else if control == "noncrossing" {
            w.minting
                .as_mut()
                .unwrap()
                .order_policy
                .as_mut()
                .unwrap()
                .input_limits
                .insert(economics_compute_smoke::minting::METAL, 1);
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

#[test]
fn sale_discovery_keeps_supply_access_margin_and_payroll_failure_gates() {
    for case in c::Case::ALL
        .into_iter()
        .filter(|case| *case != c::Case::Baseline)
    {
        let (w, s) = e::with_startup(case, e::Startup::SupplyChainInventory).unwrap();
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
