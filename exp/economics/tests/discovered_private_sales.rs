use economics_compute_smoke::{
    agency::integration::GROWER, compute::Backend, discovery::scenario, financial_reporting::Audit,
    minting::*, model::*, opportunities::Action, simulation::Simulation,
};

fn fixture() -> (World, State) {
    let (mut w, mut s) = scenario::circulation().unwrap();
    let c = w.discovery.as_mut().unwrap();
    c.private_sales = true;
    c.public_sales = false;
    c.horizon = 1;
    c.state = None;
    for p in &mut w.participants {
        if p.agent != ISSUER {
            p.needs = vec![Requirement {
                resource: NUTRITION,
                quantity: 1,
                priority: 0,
            }];
        }
    }
    s.balances.insert((ISSUER, WHEAT), 0);
    s.balances.insert((SUPPLIER, WHEAT), 4);
    s.balances.insert((WORKER, WHEAT), 0);
    s.balances.insert((WORKER, COIN), 3);
    s.balances.insert((GROWER, WHEAT), 1);
    s.balances.insert((GROWER, COIN), 0);
    (w, s)
}
fn run(w: World, s: State, backend: Backend) -> (Simulation, Audit) {
    let mut audit = scenario::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.month == 1 {
        audit.step(&mut sim).unwrap();
        if matches!(backend, Backend::CubeCpu) {
            let mut next = Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
            next.ledger = sim.ledger;
            next.reports = sim.reports;
            sim = next;
        }
    }
    (sim, audit)
}
fn sales(sim: &Simulation) -> Vec<&Deal> {
    sim.ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .flat_map(|b| {
            b.deals.iter().filter(|d| {
                d.market == WHEAT
                    && b.receipts
                        .iter()
                        .any(|r| r.accepted && r.deals.contains(&d.id))
            })
        })
        .collect()
}
#[test]
fn private_surplus_feeds_a_person_without_public_supply_or_mint_targets() {
    for case in [
        "normal",
        "disabled",
        "no surplus",
        "no cash",
        "no permission",
        "issuer excluded",
    ] {
        let (mut w, mut s) = fixture();
        match case {
            "disabled" => w.discovery.as_mut().unwrap().private_sales = false,
            "no surplus" => {
                s.balances.insert((SUPPLIER, WHEAT), 1);
            }
            "no cash" => {
                s.balances.insert((WORKER, COIN), 0);
            }
            "no permission" => {
                w.transaction_policy.as_mut().unwrap().permissions.remove(&(
                    economics_compute_smoke::opportunities::PERSON_TYPE,
                    Action::StockTrade,
                ));
            }
            "issuer excluded" => {
                w.transaction_policy.as_mut().unwrap().permissions.remove(&(
                    economics_compute_smoke::opportunities::STATE_TYPE,
                    Action::StockTrade,
                ));
            }
            _ => {}
        }
        let (reference, book) = run(w.clone(), s.clone(), Backend::Reference);
        let (cpu, cpu_book) = run(w, s, Backend::CubeCpu);
        assert_eq!(cpu.world, reference.world);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
        assert_eq!(cpu_book, book);
        let expected = case == "normal" || case == "issuer excluded";
        let deals = sales(&cpu);
        assert_eq!(deals.len(), usize::from(expected), "{case}");
        if expected {
            assert_eq!(
                (deals[0].seller, deals[0].buyer, deals[0].price),
                (SUPPLIER, WORKER, 3)
            );
            assert_eq!(cpu.state.balance(WORKER, WHEAT), 2);
            assert_eq!(cpu.state.balance(WORKER, COIN), 0);
            assert_eq!(cpu.state.balance(SUPPLIER, WHEAT), 0);
            assert!(
                cpu.reports
                    .iter()
                    .filter(|r| [SUPPLIER, WORKER].contains(&r.agent))
                    .all(|r| r.deficit(NUTRITION) == 0)
            );
        }
    }
}

#[test]
fn private_bid_limits_preserve_accepted_coin_claims_at_unequal_prices() {
    use economics_compute_smoke::{forward::direct::Terms, marketplace::Side};
    for (ask, bid, expected) in [(4, 4, 0), (2, 2, 1)] {
        let (mut w, mut s) = fixture();
        w.discovery = None;
        w.storage.weights.insert(METAL, 0);
        s.balances.insert((WORKER, COIN), 6);
        let terms = Terms {
            id: 900,
            seller: WORKER,
            buyer: ISSUER,
            month: 1,
            due: 2,
            goods: Amount::new(COIN, 3),
            prepayment: Amount::new(METAL, 1),
        };
        s.exchange.forwards.insert(terms.id, terms.contract());
        w.prepaid_deliveries.push(terms);
        let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
        p.private_sales = Some(2);
        p.quotes = vec![
            orders::Quote {
                max_lots: Some(1),
                agent: SUPPLIER,
                market: WHEAT,
                side: Side::Sell,
                limit: ask,
                holding: 1,
            },
            orders::Quote {
                max_lots: Some(1),
                agent: WORKER,
                market: WHEAT,
                side: Side::Buy,
                limit: bid,
                holding: 3,
            },
        ];
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(1).unwrap();
        assert_eq!(sales(&sim).len(), expected);
        let budget = sim
            .ledger
            .iter()
            .filter_map(|b| b.minting.as_ref())
            .flat_map(|b| &b.plan.as_ref().unwrap().purchases)
            .find(|b| b.agent == WORKER)
            .unwrap();
        assert_eq!(budget.protected_cash, 3);
        assert_eq!(budget.submitted_lots, expected as i32);
        assert!(sim.state.balance(WORKER, COIN) >= 3);
    }
}

#[test]
fn passive_person_stock_owners_preserve_claims_without_consumption_model() {
    use economics_compute_smoke::forward::direct::Terms;
    for claim in [false, true] {
        let (mut w, mut s) = fixture();
        w.participants.retain(|p| p.agent != SUPPLIER);
        w.storage.weights.insert(METAL, 0);
        w.discovery.as_mut().unwrap().horizon = 2;
        if claim {
            let terms = Terms {
                id: 900,
                seller: SUPPLIER,
                buyer: ISSUER,
                month: 1,
                due: 2,
                goods: Amount::new(WHEAT, 2),
                prepayment: Amount::new(METAL, 1),
            };
            s.exchange.forwards.insert(terms.id, terms.contract());
            w.prepaid_deliveries.push(terms);
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(1).unwrap();
        assert_eq!(sales(&sim).len(), usize::from(!claim));
        assert_eq!(
            sim.state.balance(SUPPLIER, WHEAT),
            if claim { 4 } else { 1 }
        );
    }
}

#[test]
fn failed_mint_input_package_preserves_independent_private_food_sale() {
    let (mut w, mut s) = fixture();
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .month = 1;
    w.scheduled_starts.push(ScheduledStart {
        month: 1,
        agent: ISSUER,
        definition: MINT,
    });
    s.balances.insert((SUPPLIER, METAL), 0);
    let (reference, book) = run(w.clone(), s.clone(), Backend::Reference);
    let (cpu, cpu_book) = run(w, s, Backend::CubeCpu);
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu_book, book);
    assert_eq!(sales(&cpu).len(), 1);
    let market = cpu.ledger.iter().find_map(|b| b.minting.as_ref()).unwrap();
    assert!(
        market
            .plan
            .as_ref()
            .unwrap()
            .reason
            .starts_with("input package unmatched")
    );
    assert!(market.deals.iter().all(|d| d.market == WHEAT));
    assert!(!cpu.state.processes.values().any(|p| p.definition == MINT));
}

const HOME: AgentId = 800;
fn household_fixture() -> (World, State) {
    use economics_compute_smoke::{
        household_governance as h, households,
        opportunities::{HOUSEHOLD_TYPE, PERSON_TYPE},
    };
    let (mut w, mut s) = fixture();
    let law = w.transaction_policy.as_mut().unwrap();
    law.permissions
        .insert((PERSON_TYPE, Action::FoundHousehold));
    law.permissions.insert((HOUSEHOLD_TYPE, Action::StockTrade));
    w.marketplaces
        .iter_mut()
        .find(|v| v.agent == VENUE)
        .unwrap()
        .allowed_types
        .insert(HOUSEHOLD_TYPE);
    let mut governance = h::Governance::contributed(SUPPLIER);
    governance.charter.initial_policy = h::Policy::NeedsFirst;
    households::form(
        &mut w,
        &s,
        households::Agreement {
            id: 1,
            agent: HOME,
            adults: vec![SUPPLIER, GROWER],
            formed: 1,
            governance,
            dwelling_process: None,
            admission: None,
            membership: vec![],
            asset_sales: vec![],
            equipment_retirements: vec![],
            support: vec![],
        },
    )
    .unwrap();
    s.balances.insert((SUPPLIER, WHEAT), 0);
    s.balances.insert((GROWER, WHEAT), 0);
    s.balances.insert((HOME, WHEAT), 5);
    (w, s)
}

#[test]
fn collective_surplus_sales_preserve_member_food_and_household_claims() {
    use economics_compute_smoke::{forward::direct::Terms, opportunities::HOUSEHOLD_TYPE};
    for case in [
        "normal",
        "no surplus",
        "claim",
        "member claim",
        "no permission",
    ] {
        let (mut w, mut s) = household_fixture();
        w.discovery.as_mut().unwrap().horizon = 2;
        s.balances.insert((HOME, WHEAT), 7);
        match case {
            "no surplus" => {
                s.balances.insert((HOME, WHEAT), 2);
            }
            "no permission" => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(HOUSEHOLD_TYPE, Action::StockTrade));
            }
            "claim" | "member claim" => {
                let terms = Terms {
                    id: 900,
                    seller: if case == "claim" { HOME } else { SUPPLIER },
                    buyer: ISSUER,
                    month: 1,
                    due: 2,
                    goods: Amount::new(WHEAT, 1),
                    prepayment: Amount::new(COIN, 1),
                };
                s.exchange.forwards.insert(terms.id, terms.contract());
                w.prepaid_deliveries.push(terms);
                w.discovery.as_mut().unwrap().horizon = 2;
            }
            _ => {}
        }
        let (reference, book) = run(w.clone(), s.clone(), Backend::Reference);
        let (cpu, cpu_book) = run(w, s, Backend::CubeCpu);
        assert_eq!(cpu.world, reference.world);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu_book, book);
        let trades = sales(&cpu);
        assert_eq!(trades.len(), usize::from(case == "normal"), "{case}");
        if case == "normal" {
            assert_eq!((trades[0].seller, trades[0].buyer), (HOME, WORKER));
            assert_eq!(cpu.state.balance(HOME, COIN), 3);
            let d = cpu
                .world
                .discovery
                .as_ref()
                .unwrap()
                .supply
                .iter()
                .find(|d| d.agent == HOME && d.resource == WHEAT)
                .unwrap();
            assert_eq!((d.protected, d.selected_lots), (4, 1));
        }
        assert!(
            cpu.reports
                .iter()
                .filter(|r| [SUPPLIER, GROWER].contains(&r.agent))
                .all(|r| r.deficit(NUTRITION) == 0),
            "{case}"
        );
    }
}
