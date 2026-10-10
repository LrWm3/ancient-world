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
    while sim.state.month <= 10 {
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

fn mint_loan(held_metal: i32) -> (World, State) {
    let (mut w, mut s) = scenario::scenario().unwrap();
    let c = w.discovery.as_mut().unwrap();
    c.land = None;
    c.household = None;
    c.finance.as_mut().unwrap().unit_values.clear();
    // Isolate lot financing with an interest-free project and a term whose
    // first installment leaves the next-month procurement budget intact.
    c.finance.as_mut().unwrap().monthly_rate_bps = 0;
    c.finance.as_mut().unwrap().loan_months = 8;
    for p in &mut w.participants {
        p.needs.clear();
        if p.agent != WORKER {
            p.capacity.quantity = 0;
        }
    }
    s.balances.insert((ISSUER, COIN), 0);
    s.balances.insert((ISSUER, WHEAT), 0);
    s.balances.insert((ISSUER, METAL), held_metal);
    // A single candidate lender makes the funding boundary observable.
    s.balances.insert((WORKER, COIN), 0);
    (w, s)
}

#[test]
fn partial_lot_financing_matches_real_input_orders_and_repays() {
    for (held, principal) in [(0, 6), (1, 6), (2, 4)] {
        let (w, s) = mint_loan(held);
        let reference = run(w.clone(), s.clone(), Backend::Reference);
        let cpu = run(w, s, Backend::CubeCpu);
        assert_eq!(cpu.world, reference.world);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        let loan = cpu.state.credit.loans.values().next().unwrap_or_else(|| {
            panic!(
                "held={held}: {:?}",
                cpu.world.discovery.as_ref().unwrap().receipts
            )
        });
        assert_eq!(loan.original_principal, principal);
        assert_eq!(loan.status, economics_compute_smoke::credit::Status::Repaid);
        assert!(
            cpu.state
                .processes
                .values()
                .any(|p| p.definition == MINT && p.status == Status::Completed)
        );
        assert_eq!(
            cpu.state.balance(ISSUER, METAL),
            if held == 1 { 1 } else { 0 }
        );
    }
}

#[test]
fn rounded_input_funding_requires_real_lender_money_and_supply() {
    for case in ["cash", "metal", "storage", "early installment"] {
        let (mut w, mut s) = mint_loan(1);
        match case {
            "cash" => {
                s.balances.insert((SUPPLIER, COIN), 5);
            }
            "metal" => {
                s.balances.insert((SUPPLIER, METAL), 0);
            }
            "storage" => {
                w.storage.capacities.insert(ISSUER, 1);
            }
            "early installment" => {
                w.discovery
                    .as_mut()
                    .unwrap()
                    .finance
                    .as_mut()
                    .unwrap()
                    .loan_months = 4;
            }
            _ => unreachable!(),
        }
        let sim = run(w, s, Backend::Reference);
        assert!(sim.state.credit.loans.is_empty(), "{case}");
        assert!(
            !sim.state
                .processes
                .values()
                .any(|p| p.definition == MINT && p.status == Status::Completed)
        );
    }
}

#[test]
fn mint_underwriting_cannot_treat_grain_as_procurement_coins() {
    let (mut w, s) = mint_loan(1);
    w.discovery
        .as_mut()
        .unwrap()
        .finance
        .as_mut()
        .unwrap()
        .denomination = WHEAT;
    let err = match Simulation::new(w, s, Backend::Reference) {
        Ok(_) => panic!("mixed-unit underwriting was admitted"),
        Err(err) => err,
    };
    assert!(err.contains("procurement currency"), "{err}");
}
