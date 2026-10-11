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
            while sim.state.phase != Phase::Acquire {
                a.step(&mut sim).unwrap();
            }
            let opening = sim.state.clone();
            let payroll =
                economics_compute_smoke::employment::projected_payroll(&w, &sim.state).unwrap();
            assert_eq!(
                payroll.get(&(HOME, TOKEN)).copied().unwrap_or(0),
                i128::from(hours * 2)
            );
            assert_eq!(sim.state, opening);
            assert!(sim.state.employment.earned.is_empty());
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
            relief: vec![],
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

#[test]
fn posted_labor_is_hired_only_for_incremental_feasible_work_and_keeps_wage_accounting() {
    use economics_compute_smoke::scenario::GRAIN;
    for control in 0..6 {
        let (mut w, s) = production(6);
        w.employment_offers.insert(1);
        w.employment[0].wage_per_unit.quantity = 1;
        match control {
            1 => w.activities.orders.clear(),
            2 => w.households[0].governance.charter.hiring_budget = Some(Amount::new(TOKEN, 1)),
            3 => {
                w.participants
                    .iter_mut()
                    .find(|p| p.agent == PERSON)
                    .unwrap()
                    .capacity
                    .quantity = 2
            }
            4 => w.households[0].governance.constitution.activities = Some(Default::default()),
            5 => w.employment[0].wage_per_unit.quantity = 2,
            _ => (),
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            assert!(
                !economics_compute_smoke::agreements::for_agent(&w, &s, HOME)
                    .unwrap()
                    .iter()
                    .any(|v| v.identity()
                        == economics_compute_smoke::agreements::Identity::Employment(1))
            );
            assert!(
                !households::dissolution::blockers(&w, &s, HOME)
                    .contains(&households::dissolution::Blocker::Employment)
            );
            let mut a = audit(&w, &s);
            let mut checkpoint = None;
            while sim.state.month <= 1 {
                if sim.state.phase == Phase::Acquire {
                    checkpoint = Some((sim.clone(), a.clone()));
                }
                a.step(&mut sim).unwrap();
            }
            let hours = if control == 0 { 2 } else { 0 };
            assert_eq!(sim.state.balance(WORKER, TOKEN), hours);
            assert_eq!(sim.state.balance(HOME, TOKEN), 6 - hours);
            assert_eq!(
                sim.state
                    .employment
                    .earned
                    .get(&(1, 1))
                    .map_or(0, |e| e.delivered),
                hours
            );
            assert_eq!(
                sim.state.balance(HOME, GRAIN),
                if [0, 3].contains(&control) { 2 } else { 0 }
            );
            assert_eq!(value(&a, HOME, A::PurchasedCapacity(LABOR)), 0);
            let (mut resumed, mut ra) = checkpoint.unwrap();
            through(&mut ra, &mut resumed, 1);
            assert_eq!(
                (&sim.state, &sim.ledger, &a),
                (&resumed.state, &resumed.ledger, &ra)
            );
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
fn common_hiring_offer_preparation_preserves_charter_law_and_useful_work_limits() {
    use economics_compute_smoke::offers::{self, Id, Request, Terms as OfferTerms};
    for control in 0..4 {
        let (mut w, s) = production(6);
        w.employment_offers.insert(1);
        w.employment[0].wage_per_unit.quantity = 1;
        match control {
            1 => w.activities.orders.clear(),
            2 => w.households[0].governance.charter.hiring_budget = Some(Amount::new(TOKEN, 1)),
            3 => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(HOUSEHOLD_TYPE, Action::CapacityTrade));
            }
            _ => (),
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            let request = Request::new(Id::Employment(1), HOME);
            assert!(
                offers::discover(&w, &s, HOME)
                    .iter()
                    .any(|o| matches!(&o.terms, OfferTerms::Employment(t) if t.id == 1))
            );
            assert!(
                !offers::discover(&w, &s, WORKER)
                    .iter()
                    .any(|o| o.id == Id::Employment(1))
            );
            while sim.state.phase != Phase::Acquire {
                a.step(&mut sim).unwrap();
            }
            let before = sim.clone();
            let proposal = offers::prepare(&sim, std::slice::from_ref(&request));
            assert_eq!(proposal.is_ok(), control == 0);
            assert_eq!(sim.state, before.state);
            assert_eq!(sim.ledger, before.ledger);
            if let Ok(batch) = proposal {
                // Three hours were offered, but only two complete useful work.
                assert_eq!(batch.employment.as_ref().unwrap().receipts[0].delivered, 2);
                let mut accepted = before.clone();
                offers::accept(&mut accepted, &[request]).unwrap();
                a.step(&mut sim).unwrap();
                assert_eq!(sim.state, accepted.state);
                assert_eq!(sim.ledger, accepted.ledger);
                assert!(offers::prepare(&sim, &[Request::new(Id::Employment(1), HOME)]).is_err());
            } else {
                assert!(offers::accept(&mut sim, &[request]).is_err());
                assert_eq!(sim.state, before.state);
                a.step(&mut sim).unwrap();
                assert!(sim.state.employment.earned.is_empty());
            }
            through(&mut a, &mut sim, 2);
            assert_eq!(
                sim.state.balance(WORKER, TOKEN),
                if control == 0 { 2 } else { 0 }
            );
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn common_forward_and_hiring_offers_keep_acquisition_receipts_out_of_same_window_payroll() {
    use economics_compute_smoke::{
        forward::direct::Terms as Forward,
        offers::{self, Id, Request},
        scenario::GRAIN,
    };
    let (mut w, mut s) = production(0);
    w.employment_offers.insert(1);
    w.employment[0].wage_per_unit.quantity = 1;
    w.employment[0].through = 2;
    s.balances.insert((92, TOKEN), 4);
    w.prepaid_deliveries.push(Forward {
        id: 70000,
        seller: HOME,
        buyer: 92,
        month: 1,
        due: 3,
        goods: Amount::new(GRAIN, 2),
        prepayment: Amount::new(TOKEN, 4),
    });
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut a = audit(&w, &s);
    while sim.state.phase != Phase::Acquire {
        a.step(&mut sim).unwrap();
    }
    let forward = Request::new(Id::PrepaidDelivery(70000), HOME);
    let hiring = Request::new(Id::Employment(1), HOME);
    assert!(offers::prepare(&sim, &[forward.clone(), hiring.clone()]).is_err());
    let prepared = offers::prepare(&sim, &[forward]).unwrap();
    a.step(&mut sim).unwrap();
    assert_eq!(sim.ledger.last(), Some(&prepared));
    assert!(sim.state.employment.earned.is_empty());
    while (sim.state.month, sim.state.phase) != (2, Phase::Acquire) {
        a.step(&mut sim).unwrap();
    }
    assert!(
        offers::discover(&sim.world, &sim.state, HOME)
            .iter()
            .any(|o| o.id == hiring.offer)
    );
    let prepared = offers::prepare(&sim, &[hiring]).unwrap();
    a.step(&mut sim).unwrap();
    assert_eq!(sim.ledger.last(), Some(&prepared));
    through(&mut a, &mut sim, 3);
    assert_eq!(sim.state.exchange.forwards[&70000].delivered, 2);
    assert_eq!(sim.state.balance(WORKER, TOKEN), 2);
}

#[test]
fn competing_labor_offers_do_not_duplicate_the_same_projected_work() {
    let (mut w, s) = production(6);
    w.employment[0].wage_per_unit.quantity = 1;
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
    w.employment_offers.extend([1, 2]);
    let run = |mut w: World, backend| {
        if matches!(backend, Backend::CubeCpu) {
            w.employment.reverse();
        }
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        while sim.state.phase != Phase::Acquire {
            a.step(&mut sim).unwrap();
        }
        let before = sim.state.clone();
        a.step(&mut sim).unwrap();
        assert_eq!(sim.state.employment.earned[&(1, 1)].delivered, 2);
        assert!(!sim.state.employment.earned.contains_key(&(2, 1)));
        let mut forged = sim.ledger.last().unwrap().clone();
        forged.employment.as_mut().unwrap().receipts[0].delivered = 3;
        let mut rejected = before.clone();
        assert!(commit(&w, &mut rejected, &forged, backend, DEFAULT_EFFECT_LIMIT).is_err());
        assert_eq!(rejected, before);
        through(&mut a, &mut sim, 2);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(w.clone(), Backend::Reference), run(w, Backend::CubeCpu));
}

#[test]
fn direct_prepayment_funds_next_month_useful_hiring_output_and_real_delivery() {
    use economics_compute_smoke::{forward::direct::Terms as Forward, scenario::GRAIN};
    let (mut w, mut s) = production(0);
    w.employment_offers.insert(1);
    w.employment[0].wage_per_unit.quantity = 1;
    w.employment[0].through = 2;
    s.balances.insert((92, TOKEN), 4);
    w.prepaid_deliveries.push(Forward {
        id: 70000,
        seller: HOME,
        buyer: 92,
        month: 1,
        due: 3,
        goods: Amount::new(GRAIN, 2),
        prepayment: Amount::new(TOKEN, 4),
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut a, &mut sim, 1);
        assert_eq!(sim.state.balance(HOME, TOKEN), 4);
        assert!(sim.state.employment.earned.is_empty());
        assert_eq!(value(&a, HOME, A::DeferredRevenue(70000)), -4);
        let checkpoint = (sim.clone(), a.clone());
        through(&mut a, &mut sim, 3);
        assert_eq!(sim.state.employment.earned[&(1, 2)].delivered, 2);
        assert_eq!(sim.state.employment.earned[&(1, 2)].claim.outstanding(), 0);
        assert_eq!(sim.state.exchange.forwards[&70000].delivered, 2);
        assert_eq!(sim.state.balance(92, GRAIN), 2);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
        assert_eq!(sim.state.balance(HOME, TOKEN), 2);
        assert_eq!(sim.state.balance(WORKER, TOKEN), 2);
        assert_eq!(value(&a, HOME, A::DeferredRevenue(70000)), 0);
        assert_eq!(value(&a, 92, A::ForwardPrepayment(70000)), 0);
        let (mut resumed, mut ra) = checkpoint;
        through(&mut ra, &mut resumed, 3);
        assert_eq!(
            (&sim.state, &sim.ledger, &a),
            (&resumed.state, &resumed.ledger, &ra)
        );
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
fn income_policy_prices_earned_payroll_into_a_bounded_next_book_hiring_decision() {
    use economics_compute_smoke::{
        household_governance::Policy,
        scenario::{GRAIN, NUTRITION},
    };
    let (mut w, mut s) = trading();
    w.employment_offers.insert(1);
    w.employment[0].through = 3;
    w.households[0]
        .governance
        .constitution
        .permitted_policies
        .insert(Policy::NeedsThenIncome);
    w.households[0].governance.charter.initial_policy = Policy::NeedsThenIncome;
    // Separate the worker's offered time from an independently funded food buyer.
    w.participants
        .iter_mut()
        .find(|p| p.agent == WORKER)
        .unwrap()
        .needs
        .clear();
    w.participants
        .iter_mut()
        .find(|p| p.agent == 92)
        .unwrap()
        .needs = vec![Requirement {
        resource: NUTRITION,
        quantity: 2,
        priority: 0,
    }];
    s.balances.insert((92, TOKEN), 12);
    s.balances.insert((HOME, GRAIN), 2);
    let c = w.town_market.as_mut().unwrap();
    for t in &mut c.traders {
        if t.trader.agent == WORKER {
            t.trader.agent = 92;
        }
        t.trader.limit = 3;
        t.trader.opening_quote = 3;
    }
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = Audit::with_opening(
            &w,
            &s,
            TOKEN,
            Opening {
                services: Some(Default::default()),
                processes: Some(Default::default()),
                inventory: [((HOME, GRAIN), 2)].into(),
                ..Default::default()
            },
        )
        .unwrap();
        through(&mut a, &mut sim, 3);
        assert_eq!(sim.state.employment.earned.len(), 3);
        assert!(
            sim.state
                .employment
                .earned
                .values()
                .all(|e| e.delivered == 2 && e.claim.outstanding() == 0)
        );
        assert_eq!(sim.state.balance(HOME, TOKEN), 7);
        assert_eq!(sim.state.balance(WORKER, TOKEN), 6);
        assert_eq!(sim.state.balance(92, TOKEN), 3);
        let checkpoint = (sim.clone(), a.clone());
        through(&mut a, &mut sim, 5);
        assert_eq!(sim.state.employment.earned.len(), 3);
        let (mut resumed, mut ra) = checkpoint;
        through(&mut ra, &mut resumed, 5);
        assert_eq!(
            (&sim.state, &sim.ledger, &a),
            (&resumed.state, &resumed.ledger, &ra)
        );
        let mut replay = s.clone();
        for b in &sim.ledger {
            commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        }
        assert_eq!(replay, sim.state);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    let mut expensive = w;
    expensive.employment[0].wage_per_unit.quantity = 2;
    expensive.households[0].governance.charter.hiring_budget = Some(Amount::new(TOKEN, 4));
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let mut sim = Simulation::new(expensive.clone(), s.clone(), backend).unwrap();
        sim.run_months(3).unwrap();
        assert!(sim.state.employment.earned.is_empty());
        assert_eq!(sim.state.balance(WORKER, TOKEN), 0);
    }
}

#[test]
fn hiring_preview_includes_collective_input_allocation_before_work_feasibility() {
    use economics_compute_smoke::scenario::{GRAIN, SEED};
    let (mut w, mut s) = production(6);
    w.employment_offers.insert(1);
    w.employment[0].wage_per_unit.quantity = 1;
    w.definitions
        .iter_mut()
        .find(|d| d.id == MAKE)
        .unwrap()
        .stages[0]
        .entry_inputs = vec![Amount::new(SEED, 1)];
    w.resources.push(Resource {
        id: SEED,
        name: "seed".into(),
        kind: ResourceKind::Stock,
    });
    s.balances.insert((HOME, SEED), 1);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = Audit::with_opening(
            &w,
            &s,
            TOKEN,
            Opening {
                services: Some(Default::default()),
                processes: Some(Default::default()),
                inventory: [((HOME, SEED), 1)].into(),
                ..Default::default()
            },
        )
        .unwrap();
        through(&mut a, &mut sim, 1);
        assert_eq!(sim.state.employment.earned[&(1, 1)].delivered, 2);
        assert_eq!(sim.state.balance(HOME, SEED), 0);
        assert_eq!(sim.state.balance(HOME, GRAIN), 2);
        assert_eq!(sim.state.balance(WORKER, TOKEN), 2);
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
fn bounded_discovery_composes_with_posted_hiring_without_discovered_land() {
    for case in [
        "useful",
        "no work",
        "no money",
        "no capacity",
        "expensive",
        "disabled",
    ] {
        let (mut w, mut s) = production(6);
        w.employment_offers.insert(1);
        w.employment[0].wage_per_unit.quantity = 1;
        let mut c = economics_compute_smoke::discovery::scenario::circulation()
            .unwrap()
            .0
            .discovery
            .unwrap();
        c.state = None;
        c.enabled = case != "disabled";
        w.discovery = Some(c);
        match case {
            "no work" => w.activities.orders.clear(),
            "no money" => {
                s.balances.insert((HOME, TOKEN), 0);
            }
            "no capacity" => {
                w.participants
                    .iter_mut()
                    .find(|p| p.agent == WORKER)
                    .unwrap()
                    .capacity
                    .quantity = 0
            }
            "expensive" => w.employment[0].wage_per_unit.quantity = 3,
            _ => {}
        }
        let run = |backend| {
            let mut a = audit(&w, &s);
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.month == 1 {
                a.step(&mut sim).unwrap();
                if matches!(backend, Backend::CubeCpu) {
                    let mut resumed =
                        Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
                    resumed.ledger = sim.ledger;
                    resumed.reports = sim.reports;
                    sim = resumed;
                }
            }
            let expected = if ["useful", "disabled"].contains(&case) {
                2
            } else {
                0
            };
            assert_eq!(sim.state.balance(WORKER, TOKEN), expected, "{case}");
            assert_eq!(
                sim.state
                    .employment
                    .earned
                    .values()
                    .map(|e| e.delivered)
                    .sum::<i32>(),
                expected
            );
            assert!(
                sim.state
                    .employment
                    .earned
                    .values()
                    .all(|e| e.claim.outstanding() == 0)
            );
            assert_eq!(
                sim.state
                    .balance(HOME, economics_compute_smoke::scenario::GRAIN),
                expected
            );
            (sim.world, sim.state, sim.ledger, sim.reports, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn discovered_citizenship_is_visible_to_work_but_hiring_uses_opening_permission() {
    use economics_compute_smoke::{membership::CITIZEN, scenario::GRAIN};
    for case in ["productive grant", "denied membership", "delayed trade"] {
        let (mut w, s) = production(6);
        w.employment_offers.insert(1);
        w.employment[0].wage_per_unit.quantity = 1;
        w.employment[0].through = 2;
        let mut c = economics_compute_smoke::discovery::scenario::circulation()
            .unwrap()
            .0
            .discovery
            .unwrap();
        c.state = None;
        w.discovery = Some(c);
        let law = w.transaction_policy.as_mut().unwrap();
        law.permissions
            .remove(&(PERSON_TYPE, Action::Process(MAKE)));
        law.membership_permissions
            .insert((CITIZEN, Action::Process(MAKE)));
        if case != "denied membership" {
            law.permissions.insert((PERSON_TYPE, Action::Membership));
        } else {
            law.permissions.remove(&(PERSON_TYPE, Action::Membership));
        }
        if case == "delayed trade" {
            law.permissions
                .remove(&(PERSON_TYPE, Action::CapacityTrade));
            law.membership_permissions
                .insert((CITIZEN, Action::CapacityTrade));
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut a, &mut sim, 1);
            let expected = if case == "productive grant" { 2 } else { 0 };
            assert_eq!(sim.state.balance(WORKER, TOKEN), expected, "{case}");
            assert_eq!(sim.state.balance(HOME, GRAIN), expected, "{case}");
            if case == "delayed trade" {
                assert!(
                    sim.ledger
                        .iter()
                        .filter_map(|b| b.employment.as_ref())
                        .flat_map(|b| &b.receipts)
                        .any(|r| r.reason == Reason::NotPermitted)
                );
                through(&mut a, &mut sim, 2);
                assert_eq!(sim.state.balance(WORKER, TOKEN), 2);
            }
            (sim.world, sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn optional_worker_policy_protects_food_without_inventing_wage_purchases() {
    use economics_compute_smoke::{
        activities::{Target, WorkOrder},
        employment::supply::Policy,
        scenario::{GRAIN, NUTRITION},
    };
    for (protected, capacity, expected) in [(false, 3, 2), (true, 3, 0), (true, 4, 2), (true, 0, 0)]
    {
        let (mut w, s) = production(6);
        w.employment_offers.insert(1);
        w.employment[0].wage_per_unit.quantity = 1;
        if protected {
            w.employment_supply.insert(WORKER, Policy { horizon: 1 });
        }
        let p = w
            .participants
            .iter_mut()
            .find(|p| p.agent == WORKER)
            .unwrap();
        p.capacity.quantity = capacity;
        p.needs.push(Requirement {
            resource: NUTRITION,
            quantity: 1,
            priority: 0,
        });
        w.activities.orders.push(WorkOrder {
            agent: WORKER,
            definition: MAKE,
            priority: 0,
            target: Target::Stock(Amount::new(GRAIN, 100)),
        });
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut a, &mut sim, 1);
            assert_eq!(
                sim.state.balance(WORKER, TOKEN),
                expected,
                "{protected} {capacity}"
            );
            let deficit = sim
                .reports
                .iter()
                .filter(|r| r.agent == WORKER)
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>();
            assert_eq!(deficit, if capacity == 0 || !protected { 1 } else { 0 });
            if protected && capacity == 3 {
                assert!(
                    sim.ledger
                        .iter()
                        .filter_map(|b| b.employment.as_ref())
                        .flat_map(|b| &b.receipts)
                        .any(|r| r.reason == Reason::WorkerProtection)
                );
            }
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn worker_comparisons_are_replay_validated_and_do_not_publish_on_tampering() {
    use economics_compute_smoke::employment::supply::Policy;
    let (mut w, s) = production(6);
    w.employment_offers.insert(1);
    w.employment[0].wage_per_unit.quantity = 1;
    w.employment_supply.insert(WORKER, Policy { horizon: 1 });
    let mut sim = Simulation::new(w.clone(), s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    let opening = sim.state.clone();
    sim.step().unwrap();
    let batch = sim.ledger.last().unwrap();
    let decisions = &batch.employment.as_ref().unwrap().supply;
    assert_eq!(decisions.len(), 1);
    assert_eq!(
        (
            decisions[0].maximum,
            decisions[0].searched_maximum,
            decisions[0].selected
        ),
        (2, 2, 2)
    );
    assert_eq!(
        decisions[0].alternatives[0].losses.as_ref(),
        Some(&decisions[0].baseline)
    );
    for case in 0..3 {
        let mut altered = batch.clone();
        let d = &mut altered.employment.as_mut().unwrap().supply[0];
        match case {
            0 => d.selected = 1,
            1 => d.horizon = 2,
            _ => d.alternatives.clear(),
        }
        let mut state = opening.clone();
        assert!(
            commit(
                &w,
                &mut state,
                &altered,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(state, opening);
    }
}

#[test]
fn worker_observer_respects_detail_and_both_party_filters_without_changing_execution() {
    use economics_compute_smoke::{
        employment::supply::Policy,
        telemetry::{Config, Observer, PlanningDetail},
    };
    let (mut w, s) = production(6);
    w.employment_offers.insert(1);
    w.employment[0].wage_per_unit.quantity = 1;
    w.employment_supply.insert(WORKER, Policy { horizon: 1 });
    let mut plain = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    plain.run_months(1).unwrap();
    for (detail, agent, count) in [
        (PlanningDetail::Off, WORKER, 0),
        (PlanningDetail::Selected, WORKER, 1),
        (PlanningDetail::Alternatives, HOME, 1),
        (PlanningDetail::Alternatives, PERSON, 0),
    ] {
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
        let mut observer = Observer::new(
            vec![],
            "worker",
            Config {
                planning: detail,
                agents: [agent].into(),
                ..Default::default()
            },
        )
        .unwrap();
        observer.run_months(&mut sim, 1).unwrap();
        assert_eq!((&sim.state, &sim.ledger), (&plain.state, &plain.ledger));
        let bytes = observer.finish().unwrap();
        let rows: Vec<serde_json::Value> = String::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .filter(|r: &serde_json::Value| r["kind"] == "worker_supply")
            .collect();
        assert_eq!(rows.len(), count);
        for r in rows {
            assert_eq!(r["worker_approved"], 2);
            assert_eq!(r["delivered"], 2);
            assert_eq!(
                r.get("alternatives").is_some(),
                detail == PlanningDetail::Alternatives
            );
        }
    }
}

#[test]
fn worker_supply_protects_accepted_forward_delivery_at_its_collection_deadline() {
    use economics_compute_smoke::{
        activities::{Target, WorkOrder},
        employment::supply::Policy,
        forward::direct::Terms as Forward,
        scenario::GRAIN,
    };
    for (funded, horizon, expected) in [(true, 2, 0), (false, 2, 2), (true, 1, 2)] {
        let (mut w, mut s) = production(6);
        w.employment_offers.insert(1);
        w.employment[0].wage_per_unit.quantity = 1;
        w.employment_supply.insert(WORKER, Policy { horizon });
        w.activities.orders.push(WorkOrder {
            agent: WORKER,
            definition: MAKE,
            priority: 0,
            target: Target::Stock(Amount::new(GRAIN, 4)),
        });
        s.balances.insert((92, TOKEN), if funded { 1 } else { 0 });
        w.prepaid_deliveries.push(Forward {
            id: 70001,
            seller: WORKER,
            buyer: 92,
            month: 1,
            due: 2,
            goods: Amount::new(GRAIN, 4),
            prepayment: Amount::new(TOKEN, 1),
        });
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut a, &mut sim, 2);
            assert_eq!(
                sim.state
                    .employment
                    .earned
                    .get(&(1, 1))
                    .map_or(0, |e| e.delivered),
                expected
            );
            if funded {
                assert_eq!(
                    sim.state.exchange.forwards[&70001].performed(),
                    if horizon == 2 { 4 } else { 0 }
                );
            }
            let d = &sim
                .ledger
                .iter()
                .find_map(|b| b.employment.as_ref().filter(|e| !e.supply.is_empty()))
                .unwrap()
                .supply[0];
            assert_eq!(d.deliveries.len(), usize::from(funded && horizon == 2));
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
