use economics_compute_smoke::{
    compute::Backend,
    discovery::{scenario, supply::Decision},
    minting::*,
    model::*,
    opportunities::{Action, PERSON_TYPE},
    simulation::Simulation,
};

fn worker(held_food: i32, hours: i32) -> (World, State) {
    let (mut w, mut s) = scenario::scenario().unwrap();
    let p = w
        .participants
        .iter_mut()
        .find(|p| p.agent == WORKER)
        .unwrap();
    p.needs = vec![Requirement {
        resource: NUTRITION,
        quantity: 1,
        priority: 0,
    }];
    p.capacity.quantity = hours;
    s.balances.insert((WORKER, WHEAT), held_food);
    w.storage.capacities.insert(WORKER, 32);
    (w, s)
}
fn isolated() -> (World, State) {
    let (mut w, s) = worker(0, 2);
    let c = w.discovery.as_mut().unwrap();
    c.horizon = 1;
    c.state = None;
    c.household = None;
    c.finance = None;
    (w, s)
}
fn food_recipe(w: &mut World, id: u32, stock: bool) {
    w.definitions.push(ProcessDefinition {
        id,
        name: format!("test food source {id}"),
        enabled: true,
        execution: Execution::Productive,
        asset_kind: None,
        stages: vec![Stage {
            name: "prepare".into(),
            months: 1,
            entry_inputs: if stock {
                vec![Amount::new(METAL, 2)]
            } else {
                vec![]
            },
            monthly_services: if stock {
                vec![]
            } else {
                vec![Amount::new(HOURS, 2)]
            },
        }],
        outputs: vec![Amount::new(WHEAT, 1)],
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::Process(id)));
}
fn opening(w: World, s: State) -> Simulation {
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    sim
}
fn choice(sim: &Simulation, resource: ResourceId) -> &Decision {
    sim.world
        .discovery
        .as_ref()
        .unwrap()
        .supply
        .iter()
        .find(|d| d.agent == WORKER && d.resource == resource)
        .unwrap()
}

#[test]
fn scarce_labor_is_retained_but_surplus_labor_is_offered() {
    let (mut w, s) = isolated();
    food_recipe(&mut w, 900, false);
    let scarce = opening(w.clone(), s.clone());
    assert_eq!(choice(&scarce, HOURS).selected_lots, 0);
    assert!(choice(&scarce, HOURS).alternatives.iter().any(|a| {
        a.losses
            .as_ref()
            .is_some_and(|v| v > &choice(&scarce, HOURS).baseline)
    }));
    w.participants
        .iter_mut()
        .find(|p| p.agent == WORKER)
        .unwrap()
        .capacity
        .quantity = 4;
    let surplus = opening(w, s);
    assert_eq!(choice(&surplus, HOURS).selected_lots, 1);
}

#[test]
fn stock_supply_retains_inputs_needed_to_feed_the_seller() {
    let (mut w, mut s) = isolated();
    food_recipe(&mut w, 900, true);
    s.balances.insert((WORKER, METAL), 4);
    let sim = opening(w, s);
    assert_eq!(choice(&sim, METAL).available, 4);
    assert_eq!(choice(&sim, METAL).selected_lots, 1);
}

#[test]
fn two_substitutable_inputs_are_not_both_sold_as_independent_surpluses() {
    let (mut w, mut s) = isolated();
    food_recipe(&mut w, 900, true);
    food_recipe(&mut w, 901, false);
    s.balances.insert((WORKER, METAL), 2);
    let sim = opening(w, s);
    assert_eq!(choice(&sim, METAL).selected_lots, 1);
    assert_eq!(choice(&sim, HOURS).selected_lots, 0);
}

#[test]
fn opening_capacity_shock_cannot_be_hidden_by_the_forecast() {
    let (mut w, s) = worker(8, 4);
    w.capacity_overrides.insert((1, WORKER), 1);
    let sim = opening(w, s);
    assert_eq!(choice(&sim, HOURS).available, 1);
    assert_eq!(choice(&sim, HOURS).selected_lots, 0);
}

#[test]
fn passive_person_can_offer_stock_without_a_consumption_model() {
    let (mut w, mut s) = isolated();
    w.participants.retain(|p| p.agent != WORKER);
    s.balances.insert((WORKER, METAL), 4);
    let sim = opening(w, s);
    assert_eq!(choice(&sim, METAL).selected_lots, 2);
    assert_eq!(choice(&sim, HOURS).selected_lots, 0);
}

#[test]
fn covered_recurring_worker_need_allows_paid_mint_work_with_cpu_books() {
    let (mut w, s) = worker(14, 2);
    // Isolate paid supply from the separate short-horizon forward seller, which
    // can voluntarily sell this food buffer before the full run has elapsed.
    // Loan discovery stays enabled; no supplied forward valuation authorizes a
    // proposed wheat delivery in this paired control.
    w.discovery
        .as_mut()
        .unwrap()
        .finance
        .as_mut()
        .unwrap()
        .unit_values
        .clear();
    // Competing people cannot fill a two-hour labor lot in this control.
    // This isolates paid worker supply, not household agricultural viability.
    for p in &mut w.participants {
        if p.agent != WORKER {
            p.capacity.quantity = 1;
        }
    }
    assert!(
        w.minting
            .as_ref()
            .unwrap()
            .order_policy
            .as_ref()
            .unwrap()
            .quotes
            .is_empty()
    );
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut a = scenario::audit(&reference.world, &reference.state).unwrap();
    let mut b = scenario::audit(&cpu.world, &cpu.state).unwrap();
    while reference.state.month <= 14 {
        a.step(&mut reference).unwrap();
        b.step(&mut cpu).unwrap();
        let mut resumed =
            Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu).unwrap();
        resumed.ledger = cpu.ledger;
        resumed.reports = cpu.reports;
        cpu = resumed;
    }
    assert_eq!(reference.world, cpu.world);
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.ledger, cpu.ledger);
    assert_eq!(a, b);
    assert!(
        cpu.reports
            .iter()
            .filter(|r| r.agent == WORKER)
            .all(|r| r.deficit(NUTRITION) == 0)
    );
    // The endowment change disables viable collective farming; do not mistake
    // this paid-supply control for a sustainable economy for the other people.
    assert!(
        cpu.reports
            .iter()
            .any(|r| r.agent != WORKER && r.deficit(NUTRITION) > 0)
    );
    assert!(
        cpu.ledger
            .iter()
            .filter_map(|b| b.minting.as_ref())
            .flat_map(|b| &b.deals)
            .any(|d| d.seller == WORKER && d.market == HOURS && d.price > 0)
    );
    assert!(cpu.ledger.iter().flat_map(|b| &b.transactions).any(|t| {
        t.effects
            .iter()
            .any(|e| e.account == (WORKER, HOURS) && e.delta < 0)
            && t.effects
                .iter()
                .any(|e| e.account == (WORKER, COIN) && e.delta > 0)
    }));
    assert!(
        cpu.state
            .processes
            .values()
            .any(|p| p.definition == MINT && p.status == Status::Completed)
    );
}

#[test]
fn short_horizon_forward_sales_can_deplete_a_longer_food_buffer() {
    let (mut w, s) = worker(14, 2);
    for p in &mut w.participants {
        if p.agent != WORKER {
            p.capacity.quantity = 1;
        }
    }
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut audit = scenario::audit(&sim.world, &sim.state).unwrap();
    while sim.state.month <= 14 {
        audit.step(&mut sim).unwrap();
    }
    let sales: Vec<_> = sim
        .state
        .exchange
        .forwards
        .values()
        .filter(|f| f.debtor == WORKER && f.goods.resource == WHEAT)
        .collect();
    assert_eq!(sales.len(), 2);
    assert!(sales.iter().all(|f| f.delivered == f.goods.quantity));
    assert_eq!(sales.iter().map(|f| f.delivered).sum::<i32>(), 2);
    let worker: Vec<_> = sim.reports.iter().filter(|r| r.agent == WORKER).collect();
    assert_eq!(
        worker.iter().map(|r| r.fulfilled(NUTRITION)).sum::<i32>(),
        12
    );
    assert_eq!(
        worker
            .iter()
            .filter(|r| r.deficit(NUTRITION) > 0)
            .map(|r| r.month)
            .collect::<Vec<_>>(),
        vec![13, 14]
    );
    assert!(sim.state.balance(WORKER, COIN) >= 3);
    assert_eq!(sim.state.balance(WORKER, WHEAT), 0);
    // The projections cover fulfillment but not all later subsistence: successful
    // delivery and money in hand do not promise future food supply.
    assert!(
        sim.world
            .prepaid_deliveries
            .iter()
            .all(|f| f.month + sim.world.discovery.as_ref().unwrap().horizon < 13)
    );
}

#[test]
fn income_and_supply_do_not_guarantee_future_food_is_for_sale() {
    let (w, s) = worker(1, 2);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(14).unwrap();
    assert!(
        sim.world
            .discovery
            .as_ref()
            .unwrap()
            .supply
            .iter()
            .any(|d| d.agent == WORKER && d.selected_lots > 0)
    );
    assert!(
        sim.reports
            .iter()
            .any(|r| r.agent == WORKER && r.deficit(NUTRITION) > 0)
    );
}

#[test]
fn active_non_need_work_keeps_its_service_capacity() {
    let (mut w, mut s) = isolated();
    w.participants
        .iter_mut()
        .find(|p| p.agent == WORKER)
        .unwrap()
        .needs
        .clear();
    food_recipe(&mut w, 900, false);
    w.definitions
        .iter_mut()
        .find(|d| d.id == 900)
        .unwrap()
        .stages[0]
        .months = 2;
    s.month = 2;
    s.processes.insert(
        1,
        ProcessInstance {
            id: 1,
            definition: 900,
            operator: WORKER,
            beneficiary: WORKER,
            goal: None,
            asset: None,
            right: None,
            start: 1,
            reserved_through: 2,
            stage: 0,
            elapsed: 1,
            status: Status::Active,
        },
    );
    let sim = opening(w, s);
    assert_eq!(choice(&sim, HOURS).selected_lots, 0);
    assert!(choice(&sim, HOURS).alternatives.iter().any(|a| {
        a.losses
            .as_ref()
            .is_some_and(|losses| losses.last() > choice(&sim, HOURS).baseline.last())
    }));
}

#[test]
fn eligible_worker_offer_is_not_a_guaranteed_fill() {
    let (w, s) = worker(14, 2);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(6).unwrap();
    assert!(
        sim.world
            .discovery
            .as_ref()
            .unwrap()
            .supply
            .iter()
            .any(|d| d.agent == WORKER && d.selected_lots > 0)
    );
    let sales: Vec<_> = sim
        .ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .flat_map(|b| &b.deals)
        .filter(|d| d.market == HOURS)
        .collect();
    assert!(!sales.is_empty());
    assert!(sales.iter().all(|d| d.seller == SUPPLIER));
}

#[test]
fn longer_forward_assessment_protects_food_without_changing_market_horizon() {
    let (mut w, s) = worker(14, 2);
    for p in &mut w.participants {
        if p.agent != WORKER {
            p.capacity.quantity = 1;
        }
    }
    w.discovery
        .as_mut()
        .unwrap()
        .finance
        .as_mut()
        .unwrap()
        .forward_horizon = Some(14);
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut cpu_audit = scenario::audit(&cpu.world, &cpu.state).unwrap();
    let mut audit = scenario::audit(&sim.world, &sim.state).unwrap();
    while sim.state.month <= 14 {
        audit.step(&mut sim).unwrap();
        cpu_audit.step(&mut cpu).unwrap();
        let mut resumed =
            Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu).unwrap();
        resumed.ledger = cpu.ledger;
        resumed.reports = cpu.reports;
        cpu = resumed;
    }
    assert_eq!(sim.world, cpu.world);
    assert_eq!(sim.state, cpu.state);
    assert_eq!(sim.ledger, cpu.ledger);
    assert_eq!(audit, cpu_audit);
    assert_eq!(sim.world.discovery.as_ref().unwrap().horizon, 4);
    assert!(
        sim.state
            .exchange
            .forwards
            .values()
            .all(|f| f.debtor != WORKER)
    );
    assert!(
        sim.reports
            .iter()
            .filter(|r| r.agent == WORKER)
            .all(|r| r.deficit(NUTRITION) == 0)
    );
    assert!(
        sim.world
            .discovery
            .as_ref()
            .unwrap()
            .receipts
            .iter()
            .any(|r| !r.accepted && r.description.contains("14 months, 0 projected suppliers"))
    );
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.definition == MINT && p.status == Status::Completed)
    );
}

#[test]
fn forward_assessment_horizon_is_bounded() {
    for horizon in [0, 25, u32::MAX] {
        let (mut w, s) = scenario::scenario().unwrap();
        w.discovery
            .as_mut()
            .unwrap()
            .finance
            .as_mut()
            .unwrap()
            .forward_horizon = Some(horizon);
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}

#[test]
fn longer_forward_assessment_still_allows_unneeded_stock_sales() {
    use economics_compute_smoke::agency::objectives::{Metric, Objective, Scope};
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
    s.balances.insert((WORKER, WHEAT), 2);
    w.discovery
        .as_mut()
        .unwrap()
        .finance
        .as_mut()
        .unwrap()
        .forward_horizon = Some(8);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut audit = scenario::audit(&sim.world, &sim.state).unwrap();
    while sim.state.month <= 8 {
        audit.step(&mut sim).unwrap();
    }
    assert!(
        sim.state
            .exchange
            .forwards
            .values()
            .any(|f| f.delivered == f.goods.quantity)
    );
}
