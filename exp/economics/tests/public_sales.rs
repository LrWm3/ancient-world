use economics_compute_smoke::{
    agency::objectives::{Metric, Objective, Scope},
    compute::Backend,
    discovery::scenario,
    minting::{self, orders::StockSales, *},
    model::*,
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    simulation::Simulation,
};

fn discovered() -> (World, State) {
    let (mut w, mut s) = scenario::scenario().unwrap();
    let c = w.discovery.as_mut().unwrap();
    c.horizon = 4;
    c.public_sales = true;
    c.finance = None;
    c.household = None;
    c.land = None;
    c.state.as_mut().unwrap().objectives = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Reserve {
            resource: WHEAT,
            target: 4,
        },
    }];
    for p in &mut w.participants {
        p.needs = if p.agent == WORKER {
            vec![Requirement {
                resource: NUTRITION,
                quantity: 1,
                priority: 0,
            }]
        } else {
            vec![]
        };
    }
    s.balances.insert((ISSUER, WHEAT), 10);
    s.balances.insert((ISSUER, COIN), 20);
    s.balances.insert((WORKER, COIN), 12);
    w.storage.capacities.insert(WORKER, 32);
    (w, s)
}
fn run(
    w: World,
    s: State,
    backend: Backend,
    months: u32,
) -> (
    Simulation,
    economics_compute_smoke::financial_reporting::Audit,
) {
    let mut audit = scenario::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.month <= months {
        audit.step(&mut sim).unwrap();
        let mut resumed = Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
        resumed.ledger = sim.ledger;
        resumed.reports = sim.reports;
        sim = resumed;
    }
    (sim, audit)
}
fn food_sales(sim: &Simulation) -> usize {
    sim.ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .flat_map(|b| &b.deals)
        .filter(|d| d.seller == ISSUER && d.market == WHEAT)
        .count()
}

#[test]
fn idle_public_agent_sells_surplus_to_discovered_buyer_with_cpu_books() {
    let (w, s) = discovered();
    let (reference, a) = run(w.clone(), s.clone(), Backend::Reference, 6);
    let (cpu, b) = run(w, s, Backend::CubeCpu, 6);
    assert_eq!(reference.world, cpu.world);
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.ledger, cpu.ledger);
    assert_eq!(a, b);
    assert_eq!(cpu.state.balance(ISSUER, WHEAT), 4);
    assert_eq!(food_sales(&cpu), 2);
    assert!(cpu.world.scheduled_starts.is_empty());
    assert!(
        cpu.reports
            .iter()
            .filter(|r| r.agent == WORKER && r.month > 1)
            .all(|r| r.deficit(NUTRITION) == 0)
    );
}

#[test]
fn absent_cash_permission_or_surplus_prevents_public_sale() {
    for case in ["cash", "trade", "eat", "reserve"] {
        let (mut w, mut s) = discovered();
        match case {
            "cash" => {
                s.balances.insert((WORKER, COIN), 0);
            }
            "trade" => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(PERSON_TYPE, Action::StockTrade));
            }
            "eat" => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(PERSON_TYPE, Action::Process(EAT)));
            }
            _ => {
                s.balances.insert((ISSUER, WHEAT), 4);
            }
        }
        let (sim, _) = run(w, s, Backend::Reference, 3);
        assert_eq!(food_sales(&sim), 0, "{case}");
    }
}

#[test]
fn public_sales_do_not_require_a_live_mint_target_or_mint_permission() {
    for case in ["expired", "forbidden", "disabled"] {
        let (mut w, mut s) = minting::order_scenario("normal").unwrap();
        w.minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap()
            .public_sale = Some(StockSales {
            reserve: 0,
            claim_months: 2,
        });
        match case {
            "expired" => {
                s.month = 3;
            }
            "forbidden" => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(STATE_TYPE, Action::Process(MINT)));
            }
            _ => {
                w.definitions
                    .iter_mut()
                    .find(|d| d.id == MINT)
                    .unwrap()
                    .enabled = false;
            }
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(1).unwrap();
        assert!(food_sales(&sim) > 0, "{case}");
    }
}

#[test]
fn accepted_delivery_and_unpaid_process_inputs_protect_public_stock() {
    for claim in ["delivery", "process"] {
        let (mut w, mut s) = minting::order_scenario("normal").unwrap();
        w.minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap()
            .public_sale = Some(StockSales {
            reserve: 1,
            claim_months: 4,
        });
        s.balances.insert((ISSUER, WHEAT), 6);
        if claim == "delivery" {
            let terms = economics_compute_smoke::forward::direct::Terms {
                id: 900,
                seller: ISSUER,
                buyer: WORKER,
                month: 1,
                due: 3,
                goods: Amount::new(WHEAT, 3),
                prepayment: Amount::new(COIN, 1),
            };
            s.exchange.forwards.insert(900, terms.contract());
            w.prepaid_deliveries.push(terms);
        } else {
            w.definitions.push(ProcessDefinition {
                id: 900,
                name: "committed input".into(),
                enabled: true,
                execution: Execution::Productive,
                asset_kind: None,
                stages: vec![Stage {
                    name: "use wheat".into(),
                    months: 1,
                    entry_inputs: vec![Amount::new(WHEAT, 3)],
                    monthly_services: vec![],
                }],
                outputs: vec![Amount::new(METAL, 1)],
            });
            s.processes.insert(
                900,
                ProcessInstance {
                    id: 900,
                    definition: 900,
                    operator: ISSUER,
                    beneficiary: ISSUER,
                    goal: None,
                    asset: None,
                    right: None,
                    start: 1,
                    reserved_through: 1,
                    stage: 0,
                    elapsed: 0,
                    status: Status::Active,
                },
            );
        }
        // Inspect the Acquire plan before either claim is executed.
        s.phase = Phase::Acquire;
        let c = w.minting.as_ref().unwrap();
        let plan = minting::orders::generate(&w, &s, c, c.order_policy.as_ref().unwrap()).unwrap();
        assert!(plan.deals.iter().all(|d| d.seller != ISSUER), "{claim}");
    }
}

#[test]
fn public_sale_proceeds_cannot_fund_same_boundary_minting() {
    let (mut w, mut s) = minting::order_scenario("normal").unwrap();
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.public_sale = Some(StockSales {
        reserve: 0,
        claim_months: 2,
    });
    p.month = 1;
    for start in &mut w.scheduled_starts {
        start.month = 1;
    }
    s.balances.insert((ISSUER, COIN), 0);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert!(food_sales(&sim) > 0);
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.minting.as_ref())
            .flat_map(|b| &b.deals)
            .all(|d| d.buyer != ISSUER)
    );
    assert!(
        !sim.state
            .processes
            .values()
            .any(|p| p.definition == MINT && p.status == Status::Completed)
    );
}

#[test]
fn competing_buyers_share_one_surplus_and_new_wages_are_not_opening_cash() {
    use economics_compute_smoke::marketplace::Side;
    for funded_worker in [false, true] {
        let (mut w, mut s) = minting::order_scenario("normal").unwrap();
        let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
        p.public_sale = Some(StockSales {
            reserve: 3,
            claim_months: 2,
        });
        p.month = 1;
        let q = p
            .quotes
            .iter_mut()
            .find(|q| q.agent == WORKER && q.market == WHEAT)
            .unwrap();
        q.holding = 3;
        for start in &mut w.scheduled_starts {
            start.month = 1;
        }
        s.balances.insert((ISSUER, WHEAT), 6);
        s.balances.insert((ISSUER, COIN), 6);
        s.balances
            .insert((WORKER, COIN), if funded_worker { 3 } else { 0 });
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(1).unwrap();
        let deals: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.minting.as_ref())
            .flat_map(|b| &b.deals)
            .collect();
        assert_eq!(deals.iter().filter(|d| d.seller == ISSUER).count(), 1);
        assert!(
            deals
                .iter()
                .any(|d| d.seller == WORKER && d.market == HOURS)
        );
        assert!(!deals.iter().any(|d| d.buyer == WORKER && d.market == WHEAT));
        assert_eq!(sim.state.balance(ISSUER, WHEAT), 3);
        if !funded_worker {
            let plan = sim
                .ledger
                .iter()
                .filter_map(|b| b.minting.as_ref())
                .find_map(|b| b.plan.as_ref())
                .unwrap();
            assert!(
                !plan
                    .orders
                    .iter()
                    .any(|o| o.agent == WORKER && o.side == Side::Buy)
            );
        }
    }
}
