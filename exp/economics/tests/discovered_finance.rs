use economics_compute_smoke::{
    agency::objectives::{Metric, Objective, Scope},
    compute::Backend,
    discovery::scenario,
    minting::*,
    model::*,
    opportunities::{Action, PERSON_TYPE},
    simulation::Simulation,
};

fn surplus(quantity: i32) -> (World, State) {
    let (mut w, mut s) = scenario::scenario().unwrap();
    for p in &mut w.participants {
        p.needs.clear();
    }
    let c = w.discovery.as_mut().unwrap();
    c.land = None;
    c.household = None;
    c.state.as_mut().unwrap().objectives = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Reserve {
            resource: WHEAT,
            target: 1,
        },
    }];
    s.balances.insert((ISSUER, WHEAT), 0);
    s.balances.insert((ISSUER, COIN), 1);
    s.balances.insert((WORKER, WHEAT), quantity);
    (w, s)
}
fn run(w: World, s: State, backend: Backend) -> Simulation {
    let mut audit = scenario::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.month <= 6 {
        audit.step(&mut sim).unwrap();
    }
    sim
}

#[test]
fn exact_one_unit_surplus_can_be_discovered_and_delivered() {
    for quantity in [1, 2] {
        let (w, s) = surplus(quantity);
        let sim = run(w, s, Backend::CubeCpu);
        assert_eq!(sim.state.exchange.forwards.len(), 1);
        let f = sim.state.exchange.forwards.values().next().unwrap();
        assert_eq!((f.debtor, f.creditor, f.delivered), (WORKER, ISSUER, 1));
        assert_eq!(sim.state.balance(WORKER, WHEAT), quantity - 1);
        assert_eq!(sim.state.balance(ISSUER, WHEAT), 1);
    }
}

#[test]
fn absent_needed_committed_or_forbidden_stock_is_not_a_forward_surplus() {
    for case in ["absent", "needed", "committed", "forbidden"] {
        let (mut w, mut s) = surplus(if case == "absent" { 0 } else { 1 });
        if case == "needed" {
            w.participants
                .iter_mut()
                .find(|p| p.agent == WORKER)
                .unwrap()
                .needs = vec![Requirement {
                resource: NUTRITION,
                quantity: 1,
                priority: 0,
            }];
        }
        if case == "forbidden" {
            w.transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(PERSON_TYPE, Action::StockTrade));
        }
        if case == "committed" {
            let t = economics_compute_smoke::forward::direct::Terms {
                id: 900,
                seller: WORKER,
                buyer: SUPPLIER,
                month: 1,
                due: 3,
                goods: Amount::new(WHEAT, 1),
                prepayment: Amount::new(COIN, 1),
            };
            s.exchange.forwards.insert(t.id, t.contract());
            w.prepaid_deliveries.push(t);
        }
        // Existing accepted opening claims use the ordinary simulation; fresh
        // positive cases above additionally reconcile complete financial books.
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(6).unwrap();
        assert!(
            sim.state
                .exchange
                .forwards
                .values()
                .all(|f| f.creditor != ISSUER || f.debtor != WORKER),
            "{case}"
        );
        if case == "committed" {
            assert_eq!(sim.state.exchange.forwards[&900].delivered, 1);
        }
    }
}
