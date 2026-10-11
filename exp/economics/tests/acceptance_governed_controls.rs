use economics_compute_smoke::{
    acceptance::{
        self, MUNICIPAL_WORK, Run, TOWN_TYPE, WORKSHOP, continuation as c, governed as g,
    },
    compute::Backend,
    model::*,
    opportunities::Action,
};

fn compare(w: World, s: State) -> Run {
    let mut reference = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    reference.through(c::MONTHS).unwrap();
    let mut cpu = Run::new(w, s, Backend::CubeCpu).unwrap();
    cpu.through(c::MONTHS).unwrap();
    assert_eq!(reference.sim.world, cpu.sim.world);
    assert_eq!(reference.sim.state, cpu.sim.state);
    assert_eq!(reference.sim.ledger, cpu.sim.ledger);
    assert_eq!(reference.audit, cpu.audit);
    assert_eq!(g::check(&reference).unwrap(), g::check(&cpu).unwrap());
    cpu
}

#[test]
fn governed_economy_retains_supply_access_margin_and_historical_payroll_controls() {
    for case in c::Case::ALL.into_iter().filter(|c| *c != c::Case::Baseline) {
        let (w, s) = g::scenario(case).unwrap();
        let run = compare(w, s);
        let report = g::check(&run).unwrap();
        println!("{case:?}: {:?}; {:?}", run.evidence().unwrap(), report);
        assert!(!report.ready(), "{case:?}");
        if case == c::Case::LatePayroll {
            assert!(report.failures.iter().any(|f| f == "wages entered arrears"));
            assert_eq!(run.evidence().unwrap().wage_arrears, 0);
        }
    }
}

#[test]
fn missing_controller_trade_law_and_cash_first_priority_do_not_pass_readiness() {
    for control in ["controller", "trade law", "cash first"] {
        let (mut w, s) = g::scenario(c::Case::Baseline).unwrap();
        match control {
            "controller" => {
                w.agency.remove(&WORKSHOP);
            }
            "trade law" => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(TOWN_TYPE, Action::StockTrade));
            }
            "cash first" => {
                w.agency
                    .get_mut(&WORKSHOP)
                    .unwrap()
                    .config
                    .objectives
                    .rotate_right(1);
            }
            _ => unreachable!(),
        }
        let run = compare(w, s);
        let report = g::check(&run).unwrap();
        assert!(!report.ready(), "{control}");
        if control == "controller" {
            assert!(
                report
                    .failures
                    .iter()
                    .any(|f| f.contains("no operating controller"))
            );
        }
        println!(
            "{control}: {:?}; {:?}",
            run.evidence().unwrap(),
            report.failures
        );
    }
}

#[test]
fn selected_programs_recover_after_temporary_supply_loss_without_new_fixture_orders() {
    let execute = |backend| {
        let (w, s) = g::scenario(c::Case::Baseline).unwrap();
        let mut run = Run::new(w, s, backend).unwrap();
        for month in 1..=c::MONTHS {
            if month == 13 || month == 16 {
                run.sim
                    .world
                    .definitions
                    .iter_mut()
                    .find(|d| d.id == MUNICIPAL_WORK)
                    .unwrap()
                    .enabled = month == 16;
            }
            run.through(month).unwrap();
        }
        run
    };
    let reference = execute(Backend::Reference);
    let cpu = execute(Backend::CubeCpu);
    assert_eq!(reference.sim.world, cpu.sim.world);
    assert_eq!(reference.sim.state, cpu.sim.state);
    assert_eq!(reference.sim.ledger, cpu.sim.ledger);
    assert_eq!(reference.audit, cpu.audit);
    let report = g::check(&cpu).unwrap();
    assert!(!report.ready());
    for period in &report.periods[1..] {
        assert_eq!(period.workshop_batches, 12);
        assert_eq!(period.municipal_batches, 12);
        assert_eq!(period.ore_delivered, 24);
        assert!(period.workshop_income > 0 && period.municipal_income > 0);
    }
    let evidence = cpu.evidence().unwrap();
    assert_eq!(evidence.people_food_deficit, 0);
    assert_eq!(evidence.wage_arrears, 0);
    assert!(!report.failures.iter().any(|f| f == "wages entered arrears"));
    assert!(
        acceptance::accepted_deals(&cpu.sim)
            .any(|d| d.buyer == WORKSHOP && d.market == economics_compute_smoke::minting::ORE)
    );
    println!("temporary supply: {evidence:?}; {report:?}");
}
