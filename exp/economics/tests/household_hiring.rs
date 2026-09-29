use economics_compute_smoke::{
    accounting::Account as A,
    compute::Backend,
    employment::{ArrearsPolicy, Reason, Terms},
    financial_reporting::{Audit, Opening},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
    model::*,
    opportunities::{Action, HOUSEHOLD_TYPE, PERSON_TYPE},
    scenario::{LABOR, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const WORKER: AgentId = 89;
fn fixture(cash: i32, budget: i32) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    s.balances.clear();
    s.balances.insert((HOME, TOKEN), cash);
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == WORKER { 3 } else { 0 };
    }
    w.households[0].governance.charter.hiring_budget = Some(Amount::new(TOKEN, budget));
    let law = w.transaction_policy.as_mut().unwrap();
    law.permissions.extend([
        (HOUSEHOLD_TYPE, Action::CapacityTrade),
        (PERSON_TYPE, Action::CapacityTrade),
    ]);
    w.employment.push(Terms {
        id: 1,
        employer: HOME,
        worker: WORKER,
        from: 1,
        through: 1,
        capacity: Amount::new(LABOR, 3),
        wage_per_unit: Amount::new(TOKEN, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            services: Some(Default::default()),
            processes: Some(Default::default()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim)
            .unwrap_or_else(|e| panic!("{e} at {} {:?}", sim.state.month, sim.state.phase));
    }
}
fn value(a: &Audit, agent: AgentId, account: A) -> i128 {
    a.book()
        .balances()
        .get(&(agent, account))
        .copied()
        .unwrap_or(0)
}
#[test]
fn household_hiring_is_bounded_by_charter_cash_and_whole_hours() {
    for (cash, budget, hours) in [(5, 6, 2), (6, 3, 1), (0, 6, 0), (6, 0, 0), (6, 6, 3)] {
        let (w, s) = fixture(cash, budget);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut a, &mut sim, 1);
            let r = sim
                .ledger
                .iter()
                .filter_map(|b| b.employment.as_ref())
                .flat_map(|b| &b.receipts)
                .find(|r| r.requested == 3)
                .unwrap();
            assert_eq!(r.delivered, hours);
            if hours < 3 {
                assert_eq!(r.reason, Reason::HiringBudget);
            }
            assert_eq!(sim.state.balance(WORKER, TOKEN), hours * 2);
            assert_eq!(sim.state.balance(HOME, TOKEN), cash - hours * 2);
            assert_eq!(
                value(&a, HOME, A::PurchasedCapacity(LABOR)),
                i128::from(hours * 2)
            );
            through(&mut a, &mut sim, 2);
            assert_eq!(sim.state.balance(HOME, LABOR), 0);
            assert_eq!(value(&a, HOME, A::ServiceExpense), i128::from(hours * 2));
            let mut replay = s.clone();
            for b in &sim.ledger {
                commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(replay, sim.state);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
#[test]
fn competing_hires_share_one_budget_and_permissions_still_apply() {
    let (mut w, s) = fixture(6, 6);
    let mut second = w.employment[0].clone();
    second.id = 2;
    second.worker = 92;
    w.participants
        .iter_mut()
        .find(|p| p.agent == 92)
        .unwrap()
        .capacity
        .quantity = 3;
    w.employment.push(second);
    w.employment.reverse();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.employment.earned[&(1, 1)].delivered, 3);
    assert!(!sim.state.employment.earned.contains_key(&(2, 1)));
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(HOUSEHOLD_TYPE, Action::CapacityTrade));
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert!(sim.state.employment.earned.is_empty());
}
#[test]
fn hiring_requires_explicit_matching_charter_and_outside_worker() {
    let (w, s) = fixture(6, 6);
    for case in 0..3 {
        let mut w = w.clone();
        match case {
            0 => w.households[0].governance.charter.hiring_budget = None,
            1 => {
                w.households[0]
                    .governance
                    .charter
                    .hiring_budget
                    .as_mut()
                    .unwrap()
                    .quantity = -1
            }
            _ => w.employment[0].worker = PERSON,
        }
        assert!(Simulation::new(w, s.clone(), Backend::Reference).is_err());
    }
}

const MAKE: DefinitionId = 901;
fn production(cash: i32) -> (World, State) {
    use economics_compute_smoke::{
        activities::{Target, WorkOrder},
        scenario::GRAIN,
    };
    let (mut w, s) = fixture(cash, 6);
    w.definitions.push(ProcessDefinition {
        id: MAKE,
        name: "household directed output".into(),
        enabled: true,
        execution: Execution::Productive,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 1,
            entry_inputs: vec![],
            monthly_services: vec![Amount::new(LABOR, 2)],
        }],
        outputs: vec![Amount::new(GRAIN, 4)],
    });
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: MAKE,
        priority: 0,
        target: Target::Stock(Amount::new(GRAIN, 100)),
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::Process(MAKE)));
    (w, s)
}
#[test]
fn hired_hours_and_basis_follow_member_work_output_pooling_and_expiration() {
    use economics_compute_smoke::scenario::GRAIN;
    let (w, s) = production(6);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut a, &mut sim, 1);
        assert_eq!(sim.state.balance(HOME, GRAIN), 2);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
        assert_eq!(value(&a, HOME, A::Inventory(GRAIN)), 2);
        assert_eq!(value(&a, PERSON, A::Inventory(GRAIN)), 2);
        assert_eq!(value(&a, HOME, A::PurchasedCapacity(LABOR)), 2);
        assert_eq!(value(&a, HOME, A::TransferExpense), 4);
        assert_eq!(value(&a, PERSON, A::TransferIncome), -4);
        let d = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|b| &b.labor)
            .find(|d| d.recipient == Some(PERSON))
            .unwrap();
        assert_eq!(d.purchased[0].available, 3);
        assert_eq!(d.purchased[0].directed, 2);
        assert_eq!(d.purchased[0].unused, 1);
        let saved = (sim.clone(), a.clone());
        through(&mut a, &mut sim, 2);
        assert_eq!(value(&a, HOME, A::ServiceExpense), 2);
        assert_eq!(value(&a, HOME, A::PurchasedCapacity(LABOR)), 0);
        let (mut resumed, mut ra) = saved;
        through(&mut ra, &mut resumed, 2);
        assert_eq!(sim.state, resumed.state);
        assert_eq!(a, ra);
        let mut replay = s.clone();
        for b in &sim.ledger {
            commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        }
        assert_eq!(replay, sim.state);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn paid_hours_do_not_override_constitution_or_member_permissions() {
    use economics_compute_smoke::scenario::GRAIN;
    for legal in [false, true] {
        let (mut w, s) = production(6);
        if legal {
            w.households[0].governance.constitution.activities = Some(Default::default());
        } else {
            w.transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(PERSON_TYPE, Action::Process(MAKE)));
        }
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
        let mut a = audit(&w, &s);
        through(&mut a, &mut sim, 2);
        assert_eq!(sim.state.balance(HOME, GRAIN), 0);
        assert_eq!(value(&a, HOME, A::ServiceExpense), 6);
        assert_eq!(sim.state.balance(WORKER, TOKEN), 6);
    }
}

#[test]
fn earned_physical_payroll_is_protected_from_member_input_allocations() {
    use economics_compute_smoke::scenario::GRAIN;
    let (mut w, mut s) = production(0);
    w.households[0].governance.charter.hiring_budget = Some(Amount::new(GRAIN, 6));
    w.employment[0].wage_per_unit.resource = GRAIN;
    w.definitions
        .iter_mut()
        .find(|d| d.id == MAKE)
        .unwrap()
        .stages[0]
        .entry_inputs = vec![Amount::new(GRAIN, 2)];
    s.balances.insert((HOME, GRAIN), 6);
    let run = |backend| {
        let mut a = Audit::with_opening(
            &w,
            &s,
            TOKEN,
            Opening {
                services: Some(Default::default()),
                processes: Some(Default::default()),
                inventory: [((HOME, GRAIN), 6)].into(),
                exchange_values: [(GRAIN, 1)].into(),
                ..Default::default()
            },
        )
        .unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        through(&mut a, &mut sim, 1);
        assert_eq!(sim.state.balance(WORKER, GRAIN), 6);
        assert_eq!(sim.state.balance(HOME, GRAIN), 0);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
        assert!(
            sim.ledger
                .iter()
                .filter_map(|b| b.household.as_ref())
                .flat_map(|h| &h.reservations)
                .any(|r| r.request.purpose == households::Purpose::Input(MAKE)
                    && r.request.quantity == 2
                    && r.allocated == 0)
        );
        through(&mut a, &mut sim, 2);
        assert_eq!(value(&a, HOME, A::ServiceExpense), 6);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

fn arrears(cash: i32) -> (World, State) {
    use economics_compute_smoke::{
        employment::Earned,
        finance::{Condition, FailureRule, Obligation, Transfer},
    };
    let (mut w, mut s) = fixture(cash, 6);
    s.month = 2;
    w.employment[0].through = 2;
    w.employment[0].on_arrears = ArrearsPolicy::Continue;
    s.employment.earned.insert(
        (1, 1),
        Earned {
            delivered: 3,
            claim: Obligation {
                transfer: Transfer {
                    from: HOME,
                    to: WORKER,
                    amount: Amount::new(TOKEN, 6),
                },
                settled: 0,
                condition: Condition::OnOrAfterMonth(2),
                failure: FailureRule::CarryArrears,
            },
        },
    );
    (w, s)
}
#[test]
fn old_earned_claims_reduce_new_household_hiring_even_when_work_on_credit_is_allowed() {
    let (w, s) = arrears(8);
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut a = audit(&w, &s);
    through(&mut a, &mut sim, 2);
    assert_eq!(sim.state.employment.earned[&(1, 2)].delivered, 1);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
    assert_eq!(sim.state.employment.earned[&(1, 2)].claim.outstanding(), 0);
    assert_eq!(sim.state.balance(WORKER, TOKEN), 8);
    assert_eq!(sim.state.balance(HOME, TOKEN), 0);
}

fn arrears_market(enabled: bool, funded: bool) -> (World, State) {
    use economics_compute_smoke::{negotiation::QuotePolicy, scenario::GRAIN};
    let (mut w, mut s) = arrears(0);
    let (mw, ms) = households::market::scenario().unwrap();
    w.town_market = mw.town_market;
    s.town_market = ms.town_market;
    w.households[0].governance.charter.fund_earned_wages = enabled;
    s.balances.insert((HOME, GRAIN), if funded { 2 } else { 0 });
    s.balances.insert((92, TOKEN), 6);
    let c = w.town_market.as_mut().unwrap();
    c.traders.retain(|t| [HOME, 92].contains(&t.trader.agent));
    for t in &mut c.traders {
        t.trader.limit = 2;
        t.trader.opening_quote = 2;
        t.trader.policy = QuotePolicy::Fixed;
    }
    let market = &mut w
        .marketplaces
        .iter_mut()
        .find(|m| m.agent == c.venue)
        .unwrap()
        .markets[0];
    market.goods = Amount::new(TOKEN, 6);
    market.payment = GRAIN;
    market.price_tick = 1;
    (w, s)
}
#[test]
fn collective_bids_clear_real_wage_arrears_without_funding_same_boundary_hires() {
    use economics_compute_smoke::scenario::GRAIN;
    for enabled in [false, true] {
        for funded in [false, true] {
            let (w, s) = arrears_market(enabled, funded);
            let run = |backend| {
                let mut a = Audit::with_opening(
                    &w,
                    &s,
                    TOKEN,
                    Opening {
                        services: Some(Default::default()),
                        processes: Some(Default::default()),
                        inventory: if funded {
                            [((HOME, GRAIN), 2)].into()
                        } else {
                            Default::default()
                        },
                        exchange_values: [(GRAIN, 3)].into(),
                        ..Default::default()
                    },
                )
                .unwrap();
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while sim.state.phase != Phase::Productive {
                    a.step(&mut sim).unwrap();
                }
                assert!(!sim.state.employment.earned.contains_key(&(1, 2)));
                let traded = enabled && funded;
                assert_eq!(sim.state.balance(HOME, TOKEN), if traded { 6 } else { 0 });
                assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 6);
                let checkpoint = (sim.clone(), a.clone());
                through(&mut a, &mut sim, 2);
                assert_eq!(sim.state.balance(WORKER, TOKEN), if traded { 6 } else { 0 });
                assert_eq!(
                    sim.state.employment.earned[&(1, 1)].claim.outstanding(),
                    if traded { 0 } else { 6 }
                );
                let (mut resumed, mut ra) = checkpoint;
                through(&mut ra, &mut resumed, 2);
                assert_eq!(sim.state, resumed.state);
                assert_eq!(a, ra);
                let mut replay = s.clone();
                for b in &sim.ledger {
                    commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
                }
                assert_eq!(replay, sim.state);
                (sim.state, sim.ledger, a)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

fn trading() -> (World, State) {
    use economics_compute_smoke::{
        marketplace::Side,
        negotiation::QuotePolicy,
        scenario::{GRAIN, NUTRITION},
    };
    let (mut w, s) = production(4);
    let (mw, ms) = households::market::scenario().unwrap();
    w.town_market = mw.town_market;
    let mut s = s;
    s.town_market = ms.town_market;
    let p = w
        .participants
        .iter_mut()
        .find(|p| p.agent == WORKER)
        .unwrap();
    p.capacity.quantity = 2;
    p.needs.push(Requirement {
        resource: NUTRITION,
        quantity: 2,
        priority: 0,
    });
    for p in &w.participants {
        w.storage.capacities.insert(p.agent, 100);
    }
    w.households[0].governance.charter.hiring_budget = Some(Amount::new(TOKEN, 2));
    w.employment[0].capacity.quantity = 2;
    w.employment[0].wage_per_unit.quantity = 1;
    w.employment[0].through = 8;
    let c = w.town_market.as_mut().unwrap();
    c.traders
        .retain(|t| [HOME, WORKER].contains(&t.trader.agent));
    for t in &mut c.traders {
        t.side = if t.trader.agent == HOME {
            Side::Sell
        } else {
            Side::Buy
        };
        t.trader.limit = 2;
        t.trader.opening_quote = 2;
        t.trader.policy = QuotePolicy::Fixed;
    }
    let m = &mut w
        .marketplaces
        .iter_mut()
        .find(|m| m.agent == c.venue)
        .unwrap()
        .markets[0];
    m.goods = Amount::new(GRAIN, 2);
    m.price_tick = 1;
    (w, s)
}
fn market_window(w: &mut World, month: u32, shock: bool) {
    w.town_market.as_mut().unwrap().match_limit = if shock && month == 3 { Some(0) } else { None };
}
#[test]
fn repeated_household_hiring_output_sales_and_payroll_respond_to_a_revenue_interruption() {
    use economics_compute_smoke::{
        scenario::NUTRITION,
        telemetry::{Config, Observer},
    };
    for shock in [false, true] {
        let (w, s) = trading();
        let run = |mut w: World, backend| {
            let initial = w.clone();
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            let mut observer = Observer::new(
                Vec::new(),
                "household-hiring",
                Config {
                    settlement: true,
                    ..Default::default()
                },
            )
            .unwrap();
            let mut checkpoint = None;
            while sim.state.month <= 8 {
                let month = sim.state.month;
                market_window(&mut sim.world, month, shock);
                observer.step_audited(&mut sim, &mut a).unwrap();
                if sim.state.month == 4 && sim.state.phase == Phase::Open {
                    checkpoint = Some((sim.clone(), a.clone()));
                }
            }
            for month in 1..=8 {
                let expected = if shock && [4, 6, 7, 8].contains(&month) {
                    0
                } else {
                    2
                };
                assert_eq!(
                    sim.state
                        .employment
                        .earned
                        .get(&(1, month))
                        .map_or(0, |e| e.delivered),
                    expected,
                    "month {month}, interruption {shock}"
                );
            }
            assert!(
                sim.state
                    .employment
                    .earned
                    .values()
                    .all(|e| e.claim.outstanding() == 0)
            );
            assert_eq!(
                sim.state.balance(HOME, TOKEN) + sim.state.balance(WORKER, TOKEN),
                4
            );
            assert!(
                sim.reports
                    .iter()
                    .filter(|r| r.agent == WORKER && r.month >= 5)
                    .all(|r| r.deficit(NUTRITION) == 0)
            );
            // Released worker hours can supply their own food. The interruption
            // changes later demand; reopening does not guarantee the old cash loop.
            assert_eq!(
                sim.ledger.iter().any(|b| b.phase == Phase::Productive
                    && b.receipts.iter().any(|r| r.agent == WORKER
                        && r.definition == Some(MAKE)
                        && r.completed > 0)),
                shock
            );
            assert_eq!(sim.state.balance(HOME, TOKEN), if shock { 0 } else { 2 });
            for agent in [HOME, PERSON, WORKER] {
                let r = a.book().statements(agent, 1, 8).unwrap();
                assert_eq!(r.assets, r.liabilities + r.equity);
            }
            let (mut resumed, mut ra) = checkpoint.unwrap();
            while resumed.state.month <= 8 {
                let month = resumed.state.month;
                market_window(&mut resumed.world, month, shock);
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(sim.state, resumed.state);
            assert_eq!(sim.ledger, resumed.ledger);
            assert_eq!(a, ra);
            let mut replay = s.clone();
            w = initial;
            for b in &sim.ledger {
                market_window(&mut w, b.month, shock);
                commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(replay, sim.state);
            let logs = String::from_utf8(observer.finish().unwrap()).unwrap();
            assert!(
                logs.lines()
                    .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
                    .any(|r| r["kind"] == "household_labor" && r["purchased"][0]["directed"] == 2)
            );
            (sim.state, sim.ledger, a)
        };
        let reference = run(w.clone(), Backend::Reference);
        let mut reversed = w;
        reversed.participants.reverse();
        reversed.resources.reverse();
        reversed.employment.reverse();
        assert_eq!(reference, run(reversed, Backend::CubeCpu));
    }
}

#[test]
fn purchased_labor_receipts_and_transfers_are_verified_before_publication() {
    let (w, s) = production(6);
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while sim.state.phase != Phase::Productive {
            sim.step().unwrap();
        }
        let before = sim.state.clone();
        sim.step().unwrap();
        let good = sim.ledger.last().unwrap();
        for case in 0..2 {
            let mut bad = good.clone();
            let h = bad.household.as_mut().unwrap();
            if case == 0 {
                h.labor[0].purchased[0].directed += 1;
            } else {
                h.before
                    .iter_mut()
                    .find(|e| e.account == (HOME, LABOR))
                    .unwrap()
                    .delta += 1;
            }
            let mut state = before.clone();
            assert!(commit(&w, &mut state, &bad, backend, DEFAULT_EFFECT_LIMIT).is_err());
            assert_eq!(state, before);
        }
        let mut replay = before;
        commit(&w, &mut replay, good, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        assert_eq!(replay, sim.state);
        let completed = replay.clone();
        assert!(commit(&w, &mut replay, good, backend, DEFAULT_EFFECT_LIMIT).is_err());
        assert_eq!(replay, completed);
    }
}
