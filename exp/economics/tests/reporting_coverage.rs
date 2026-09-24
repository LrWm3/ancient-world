use economics_compute_smoke::{
    accounting::Account as A, compute::Backend, credit, credit_stress, financial_reporting::Audit,
    model::*, scenario::*, simulation::Simulation, town_market,
};
use std::collections::BTreeMap;
fn audit(w: &World, s: &State) -> Audit {
    let costs = s
        .balances
        .iter()
        .filter(|((_, r), q)| {
            *r != TOKEN
                && **q > 0
                && w.resources
                    .iter()
                    .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
        })
        .map(|(k, q)| (*k, i128::from(*q)))
        .collect();
    let weights = w
        .definitions
        .iter()
        .filter_map(|d| {
            let weights: BTreeMap<_, _> = d
                .outputs
                .iter()
                .filter(|a| {
                    w.resources
                        .iter()
                        .any(|r| r.id == a.resource && r.kind == ResourceKind::Stock)
                })
                .map(|a| (a.resource, 1))
                .collect();
            (weights.len() > 1).then_some((d.id, weights))
        })
        .collect();
    Audit::with_processes(
        w,
        s,
        TOKEN,
        w.assets.iter().map(|a| (a.id, 10_000)).collect(),
        costs,
        weights,
    )
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
#[test]
fn crop_cost_transfers_without_changing_debt_recovery_then_harvests_or_fails() {
    for maintain in [true, false] {
        let (w, s) = credit::crop_scenario(maintain).unwrap();
        let mut a = audit(&w, &s);
        let mut b = a.clone();
        let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
        let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut cpu, 2);
        let mut resumed = a.clone();
        let mut checkpoint = cpu.clone();
        through(&mut a, &mut cpu, 6);
        through(&mut resumed, &mut checkpoint, 6);
        through(&mut b, &mut reference, 6);
        assert_eq!(a, b);
        assert_eq!(a, resumed);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.state.credit.loans[&1].principal, 2160);
        let debtor = a.book().statements(PERSON, 1, 6).unwrap();
        let lender = a.book().statements(STATE_AGENT, 1, 6).unwrap();
        assert_eq!(debtor.expenses[&A::TransferExpense], 1);
        assert_eq!(lender.income[&A::TransferIncome], 1);
        if maintain {
            assert_eq!(cpu.state.balance(STATE_AGENT, GRAIN), 8);
            assert_eq!(lender.trial_balance.get(&A::Inventory(SEED)), Some(&1));
        } else {
            assert_eq!(lender.expenses[&A::ProductionLoss], 1);
        }
    }
}
#[test]
fn all_credit_stress_paths_have_reconciled_financial_statements() {
    for case in [
        credit_stress::Case::Normal,
        credit_stress::Case::Temporary,
        credit_stress::Case::Persistent,
    ] {
        let (w, s) = credit_stress::scenario(case).unwrap();
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut sim, credit_stress::OBSERVATION_MONTHS);
        a.finalize_through(credit_stress::OBSERVATION_MONTHS)
            .unwrap();
        for agent in [PERSON, STATE_AGENT] {
            let r = a
                .book()
                .finalized_statements(agent, 1, credit_stress::OBSERVATION_MONTHS)
                .unwrap();
            assert_eq!(r.assets - r.liabilities, r.equity);
        }
    }
}
#[test]
fn town_trades_recognize_revenue_cost_and_cpu_continuation() {
    let (w, s) = town_market::scenario();
    let mut a = audit(&w, &s);
    let mut b = a.clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut cpu, 1);
    through(&mut b, &mut reference, 1);
    assert_eq!(a, b);
    assert_eq!(cpu.state, reference.state);
    assert!(
        a.book()
            .entries()
            .iter()
            .flat_map(|e| &e.lines)
            .any(|l| l.account == A::Sales && l.debit < 0)
    );
    let mut resumed = a.clone();
    let mut checkpoint = cpu.clone();
    through(&mut a, &mut cpu, 3);
    through(&mut resumed, &mut checkpoint, 3);
    assert_eq!(a, resumed);
}
#[test]
fn explicit_stock_expiration_expenses_basis_once() {
    let (mut w, s) = credit::scenario("downpayment").unwrap();
    w.resources.push(Resource {
        id: GRAIN,
        name: "perishable".into(),
        kind: ResourceKind::Stock,
    });
    w.activities.perishable.insert(GRAIN);
    let mut s = s;
    s.balances.insert((PERSON, GRAIN), 3);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 2);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    assert_eq!(
        a.book().statements(PERSON, 1, 2).unwrap().expenses[&A::InventoryLoss],
        3
    );
}

#[test]
fn historical_work_import_requires_complete_basis_and_preserves_it_when_policy_is_set() {
    use economics_compute_smoke::{financial_reporting::Opening, process_accounting::Costs};
    let (w, s) = credit::crop_scenario(true).unwrap();
    let mut original = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut original, &mut sim, 2);
    let work: BTreeMap<_, _> = sim
        .state
        .processes
        .values()
        .filter(|p| p.status == Status::Active)
        .map(|p| {
            (
                p.id,
                (
                    p.operator,
                    original.book().balances()[&(p.operator, A::WorkInProgress(p.id))],
                ),
            )
        })
        .collect();
    let inventory = sim
        .state
        .balances
        .iter()
        .filter(|((_, r), q)| {
            *r != TOKEN
                && **q > 0
                && sim
                    .world
                    .resources
                    .iter()
                    .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
        })
        .map(|((agent, r), _)| {
            (
                (*agent, *r),
                original.book().balances()[&(*agent, A::Inventory(*r))],
            )
        })
        .collect();
    let opening = Opening {
        assets: BTreeMap::from([(PLOT, 10_000)]),
        inventory,
        processes: Some(Costs {
            work,
            output_weights: BTreeMap::new(),
            ..Default::default()
        }),
        ..Opening::default()
    };
    let mut missing = opening.clone();
    missing.processes.as_mut().unwrap().work.clear();
    assert!(Audit::with_opening(&sim.world, &sim.state, TOKEN, missing).is_err());
    let mut negative = opening.clone();
    negative
        .processes
        .as_mut()
        .unwrap()
        .work
        .values_mut()
        .next()
        .unwrap()
        .1 = -1;
    assert!(Audit::with_opening(&sim.world, &sim.state, TOKEN, negative).is_err());
    let mut imported = Audit::with_opening(&sim.world, &sim.state, TOKEN, opening)
        .unwrap()
        .with_process_policy(
            &sim.world,
            BTreeMap::from([(GROW, BTreeMap::from([(GRAIN, 1), (SEED, 1)]))]),
        )
        .unwrap();
    let mut continued = sim.clone();
    through(&mut original, &mut sim, 6);
    through(&mut imported, &mut continued, 6);
    assert_eq!(sim.state, continued.state);
    for agent in [PERSON, STATE_AGENT] {
        let a = original.book().statements(agent, 3, 6).unwrap();
        let b = imported.book().statements(agent, 3, 6).unwrap();
        assert_eq!(
            (a.assets, a.liabilities, a.equity, a.net_income),
            (b.assets, b.liabilities, b.equity, b.net_income)
        );
    }
}

#[test]
fn audited_observer_keeps_simulation_and_reporting_atomic_on_rejection() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let (mut w, s) = credit::crop_scenario(true).unwrap();
    // A valid joint crop without configured cost shares must still fail accounting.
    w.credit.as_mut().unwrap().offers[0].loan.grace_months = 20;
    let mut a = audit(&w, &s)
        .with_process_policy(&w, BTreeMap::new())
        .unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut observer = Observer::new(Vec::new(), "audited", Config::default()).unwrap();
    loop {
        let old = a.clone();
        let before = sim.clone();
        if let Err(e) = observer.step_audited(&mut sim, &mut a) {
            assert!(e.contains("joint outputs"), "{e}");
            assert_eq!(old, a);
            assert_eq!(before.state, sim.state);
            assert_eq!(before.ledger, sim.ledger);
            break;
        }
        assert!(sim.state.month <= 6);
    }
    let log = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert!(log.contains("step_error"));
}
