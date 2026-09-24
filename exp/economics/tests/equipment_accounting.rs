use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    compute::Backend,
    financial_reporting::Audit,
    model::*,
    scenario::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;

fn fixture() -> (World, State) {
    let (mut w, mut s) = named("tool-beneficial").unwrap();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    s.month = 1;
    s.processes.clear();
    s.balances.insert((PERSON, GRAIN), 10);
    s.balances.insert((PERSON, SEED), 1);
    s.balances.insert((PERSON, FUEL), 6);
    s.conditions.clear();
    w.offers[0].price = Amount::new(TOKEN, 3);
    s.balances.insert((PERSON, TOKEN), 3);
    (w, s)
}
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
            let outputs: BTreeMap<_, _> = d
                .outputs
                .iter()
                .filter(|a| {
                    w.resources
                        .iter()
                        .any(|r| r.id == a.resource && r.kind == ResourceKind::Stock)
                })
                .map(|a| (a.resource, 1))
                .collect();
            (outputs.len() > 1).then_some((d.id, outputs))
        })
        .collect();
    Audit::with_processes(
        w,
        s,
        TOKEN,
        BTreeMap::from([(PLOT, 0), (TOOL, 2)]),
        costs,
        weights,
    )
    .unwrap()
}
#[test]
fn coin_purchase_and_wear_reconcile_on_cpu_and_resume() {
    let (w, s) = fixture();
    let mut a = audit(&w, &s);
    let mut b = a.clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    while cpu.state.month <= 6 {
        a.step(&mut cpu).unwrap();
        b.step(&mut reference).unwrap();
    }
    assert_eq!(a, b);
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.state.equipment[&TOOL].owner, PERSON);
    let seller = a.book().statements(STATE_AGENT, 1, 6).unwrap();
    assert_eq!(seller.income[&A::DisposalGain], 1);
    assert_eq!(seller.cash_flows[&Flow::Investing], 3);
    let buyer = a.book().statements(PERSON, 1, 6).unwrap();
    assert_eq!(buyer.cash_flows[&Flow::Investing], -3);
    // Cost 3 over six uses: rounding keeps the first fractional tick in the tool.
    assert_eq!(buyer.trial_balance[&A::Tangible(TOOL)], 3);
    let initial_equity: i128 = [PERSON, STATE_AGENT]
        .into_iter()
        .map(|id| a.book().statements(id, 1, 6).unwrap().opening_equity)
        .sum();
    let mut resumed = a.clone();
    let mut checkpoint = cpu.clone();
    while cpu.state.month <= 60 {
        a.step(&mut cpu).unwrap();
        resumed.step(&mut checkpoint).unwrap();
        let reports: Vec<_> = [PERSON, STATE_AGENT]
            .into_iter()
            .map(|id| a.book().statements(id, 1, cpu.state.month).unwrap())
            .collect();
        assert_eq!(
            reports
                .iter()
                .map(|r| r.assets - r.liabilities - r.net_income)
                .sum::<i128>(),
            initial_equity
        );
    }
    assert_eq!(a, resumed);
    assert_eq!(cpu.state.equipment[&TOOL].remaining_uses, 0);
    assert_eq!(
        a.book()
            .balances()
            .get(&(PERSON, A::Tangible(TOOL)))
            .copied()
            .unwrap_or(0),
        0
    );
}
#[test]
fn missing_basis_and_barter_fail_without_publishing() {
    let (w, s) = fixture();
    assert!(Audit::with_assets(&w, &s, TOKEN, BTreeMap::from([(PLOT, 0)])).is_err());
    let (mut w, s) = fixture();
    w.offers[0].price = Amount::new(GRAIN, 3);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    loop {
        assert!(sim.state.month < 20);
        let old = a.clone();
        let state = sim.state.clone();
        if let Err(e) = a.step(&mut sim) {
            assert!(e.contains("barter"), "{e}");
            assert_eq!(a, old);
            assert_eq!(sim.state, state);
            break;
        }
    }
}
