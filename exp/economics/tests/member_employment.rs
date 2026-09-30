use economics_compute_smoke::{
    accounting::Account as A,
    activities::{Target, WorkOrder},
    compute::Backend,
    employment::{ArrearsPolicy, Terms},
    financial_reporting::{Audit, Opening},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
    model::*,
    opportunities::{Action, PERSON_TYPE},
    scenario::{GRAIN, LABOR, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const WORKER: AgentId = 89;
const MEMBER: AgentId = 91;
const MAKE: DefinitionId = 901;
fn fixture() -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    s.balances.clear();
    s.balances.insert((PERSON, TOKEN), 10);
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = match p.agent {
            PERSON | WORKER => 5,
            MEMBER => 1,
            _ => 0,
        };
    }
    let law = w.transaction_policy.as_mut().unwrap();
    law.permissions.extend([
        (PERSON_TYPE, Action::CapacityTrade),
        (PERSON_TYPE, Action::Process(MAKE)),
    ]);
    w.employment.push(Terms {
        id: 1,
        employer: PERSON,
        worker: WORKER,
        from: 1,
        through: 3,
        capacity: Amount::new(LABOR, 5),
        wage_per_unit: Amount::new(TOKEN, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    w.definitions.push(ProcessDefinition {
        id: MAKE,
        name: "member output".into(),
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
        agent: MEMBER,
        definition: MAKE,
        priority: 0,
        target: Target::Stock(Amount::new(GRAIN, 100)),
    });
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| *r == GRAIN && **q > 0)
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            exchange_values: [(GRAIN, 1)].into(),
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
fn value(a: &Audit, who: AgentId, account: A) -> i128 {
    a.book()
        .balances()
        .get(&(who, account))
        .copied()
        .unwrap_or(0)
}
fn replay(w: &World, s: &State, sim: &Simulation) {
    let mut replay = s.clone();
    for b in &sim.ledger {
        commit(w, &mut replay, b, sim.backend, DEFAULT_EFFECT_LIMIT).unwrap();
    }
    assert_eq!(replay, sim.state);
}
#[test]
fn private_hiring_does_not_increase_own_labor_contribution_and_cost_follows_allocated_hours() {
    let (w, s) = fixture();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut a, &mut sim, 1);
        let labor = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .next()
            .unwrap();
        let c = labor
            .contributions
            .iter()
            .find(|c| c.member == PERSON)
            .unwrap();
        assert_eq!((c.available, c.reserved, c.directed), (10, 1, 1));
        assert_eq!(labor.recipient, Some(MEMBER));
        assert_eq!(sim.state.balance(WORKER, TOKEN), 10);
        assert_eq!(sim.state.balance(HOME, TOKEN), 0);
        assert_eq!(value(&a, PERSON, A::PurchasedCapacity(LABOR)), 9);
        assert_eq!(value(&a, PERSON, A::TransferExpense), 1);
        assert_eq!(value(&a, MEMBER, A::TransferIncome), -1);
        assert_eq!(
            value(&a, MEMBER, A::Inventory(GRAIN)) + value(&a, HOME, A::Inventory(GRAIN)),
            1
        );
        assert_eq!(sim.state.balance(MEMBER, GRAIN), 2);
        assert_eq!(sim.state.balance(HOME, GRAIN), 2);
        let (mut resumed, mut ra) = (sim.clone(), a.clone());
        a.step(&mut sim).unwrap();
        ra.step(&mut resumed).unwrap();
        assert_eq!(value(&a, PERSON, A::ServiceExpense), 9);
        assert_eq!(value(&a, PERSON, A::PurchasedCapacity(LABOR)), 0);
        assert_eq!(sim.state, resumed.state);
        assert_eq!(a, ra);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn purchased_hours_alone_create_no_member_labor_entitlement_and_internal_hires_reject() {
    let (mut w, s) = fixture();
    w.participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .capacity
        .quantity = 0;
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    let c = sim
        .ledger
        .iter()
        .filter_map(|b| b.household.as_ref())
        .flat_map(|h| &h.labor)
        .flat_map(|d| &d.contributions)
        .find(|c| c.member == PERSON)
        .unwrap();
    assert_eq!((c.available, c.reserved, c.directed), (5, 0, 0));
    assert_eq!(sim.state.balance(MEMBER, GRAIN), 0);
    w.employment[0].worker = MEMBER;
    assert!(
        Simulation::new(w, s, Backend::Reference)
            .unwrap_err()
            .contains("internal household")
    );
}

const WORKER_HOME: AgentId = HOME + 1;
fn worker_household(w: &mut World, s: &State) {
    let mut h = w.households[0].clone();
    h.id = 2;
    h.agent = WORKER_HOME;
    h.adults = vec![WORKER, 92];
    h.governance = economics_compute_smoke::household_governance::Governance::contributed(WORKER);
    h.support.clear();
    households::form(w, s, h).unwrap();
}
#[test]
fn cross_household_physical_payroll_preserves_storage_fractional_pooling_and_private_debt() {
    for (room, carry, paid) in [
        (0, 0, 1),
        (1, 0, 3),
        (2, 0, 4),
        (0, 1, 0),
        (1, 1, 2),
        (2, 1, 4),
    ] {
        let (mut w, mut s) = fixture();
        w.activities.orders.clear();
        w.employment[0].wage_per_unit = Amount::new(GRAIN, 2);
        w.employment[0].through = 1;
        s.balances.clear();
        s.balances.insert((PERSON, GRAIN), 10);
        for p in &w.participants {
            w.storage.capacities.insert(p.agent, 100);
        }
        worker_household(&mut w, &s);
        w.storage.capacities.insert(WORKER, 4);
        w.storage.capacities.insert(92, 2 * room);
        s.balances.insert((WORKER_HOME, GRAIN), 2);
        s.household_remainders
            .insert((WORKER_HOME, WORKER, GRAIN), carry);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut a, &mut sim, 1);
            assert_eq!(sim.state.employment.earned[&(1, 1)].delivered, 4);
            assert_eq!(
                sim.state.employment.earned[&(1, 1)].claim.outstanding(),
                8 - paid
            );
            let pooled = (paid + carry) / 2;
            assert_eq!(sim.state.balance(PERSON, GRAIN), 10 - paid);
            assert_eq!(sim.state.balance(HOME, GRAIN), 0);
            assert_eq!(sim.state.balance(WORKER, GRAIN), paid - pooled);
            assert_eq!(sim.state.balance(WORKER_HOME, GRAIN), 2 + pooled);
            assert_eq!(
                value(&a, PERSON, A::WagesPayable(1, 1)),
                -i128::from(8 - paid)
            );
            assert_eq!(value(&a, HOME, A::WagesPayable(1, 1)), 0);
            assert_eq!(
                value(&a, WORKER, A::WagesReceivable(1, 1)),
                i128::from(8 - paid)
            );
            replay(&w, &s, &sim);
            // Supplied room increase lets the existing, expired contract collect.
            sim.world.storage.capacities.insert(92, 20);
            let (mut resumed, mut ra) = (sim.clone(), a.clone());
            through(&mut a, &mut sim, 2);
            through(&mut ra, &mut resumed, 2);
            assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
            assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
            assert_eq!(
                sim.state.balance(WORKER, GRAIN) + sim.state.balance(WORKER_HOME, GRAIN),
                10
            );
            assert_eq!(sim.state, resumed.state);
            assert_eq!(a, ra);
            for agent in [PERSON, HOME, WORKER, WORKER_HOME] {
                let r = a.book().statements(agent, 1, 2).unwrap();
                assert_eq!(r.assets, r.liabilities + r.equity);
            }
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn optional_household_wage_assistance_keeps_the_debt_personal_and_protects_own_payroll() {
    for (enabled, own_payroll, expected) in [(false, false, 0), (true, false, 6), (true, true, 2)] {
        let (mut w, mut s) = fixture();
        w.activities.orders.clear();
        w.households[0].governance.charter.support_member_wages = enabled;
        s.balances.clear();
        s.balances.insert((HOME, TOKEN), 6);
        if own_payroll {
            use economics_compute_smoke::opportunities::HOUSEHOLD_TYPE;
            w.households[0].governance.charter.hiring_budget = Some(Amount::new(TOKEN, 4));
            w.transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .insert((HOUSEHOLD_TYPE, Action::CapacityTrade));
            w.participants
                .iter_mut()
                .find(|p| p.agent == 92)
                .unwrap()
                .capacity
                .quantity = 2;
            let mut t = w.employment[0].clone();
            t.id = 2;
            t.employer = HOME;
            t.worker = 92;
            t.capacity.quantity = 2;
            w.employment.push(t);
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            while sim.state.phase != Phase::Close {
                a.step(&mut sim).unwrap();
            }
            assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 10);
            let opening = sim.state.clone();
            let (mut resumed, mut ra) = (sim.clone(), a.clone());
            a.step(&mut sim).unwrap();
            ra.step(&mut resumed).unwrap();
            assert_eq!(sim.state.balance(WORKER, TOKEN), expected);
            assert_eq!(
                sim.state.employment.earned[&(1, 1)].claim.outstanding(),
                10 - expected
            );
            assert_eq!(value(&a, HOME, A::TransferExpense), i128::from(expected));
            assert_eq!(value(&a, PERSON, A::TransferIncome), -i128::from(expected));
            assert_eq!(
                value(&a, PERSON, A::WagesPayable(1, 1)),
                -i128::from(10 - expected)
            );
            assert_eq!(value(&a, HOME, A::WagesPayable(1, 1)), 0);
            if own_payroll {
                assert_eq!(sim.state.balance(92, TOKEN), 4);
                assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            }
            assert_eq!(sim.state, resumed.state);
            assert_eq!(a, ra);
            replay(&w, &s, &sim);
            if enabled {
                let mut bad = sim.ledger.last().unwrap().clone();
                bad.household.as_mut().unwrap().reservations[0].allocated += 1;
                let mut unchanged = opening.clone();
                assert!(commit(&w, &mut unchanged, &bad, backend, DEFAULT_EFFECT_LIMIT).is_err());
                assert_eq!(unchanged, opening);
            }
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
#[test]
fn scarce_member_payroll_uses_explicit_support_priority() {
    use economics_compute_smoke::household_governance::DebtSupportPolicy;
    for (policy, recipient) in [
        (DebtSupportPolicy::ReservationOrder, WORKER),
        (DebtSupportPolicy::ClaimPriority, 92),
    ] {
        let (mut w, mut s) = fixture();
        w.activities.orders.clear();
        w.households[0].governance.charter.support_member_wages = true;
        w.households[0].governance.charter.debt_support = policy;
        s.balances.clear();
        s.balances.insert((HOME, TOKEN), 5);
        w.employment[0].rank = 9;
        w.employment[0].capacity.quantity = 3;
        let mut t = w.employment[0].clone();
        t.id = 2;
        t.rank = 0;
        t.employer = MEMBER;
        t.worker = 92;
        w.employment.push(t);
        w.participants
            .iter_mut()
            .find(|p| p.agent == 92)
            .unwrap()
            .capacity
            .quantity = 3;
        let run = |w: World, backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut a, &mut sim, 1);
            assert_eq!(sim.state.balance(recipient, TOKEN), 5);
            assert_eq!(
                sim.state.balance(WORKER, TOKEN) + sim.state.balance(92, TOKEN),
                5
            );
            assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        let reference = run(w.clone(), Backend::Reference);
        w.participants.reverse();
        w.employment.reverse();
        if policy == DebtSupportPolicy::ClaimPriority {
            w.households[0].adults.reverse();
        }
        let cpu = run(w, Backend::CubeCpu);
        assert_eq!(reference.0, cpu.0);
        assert_eq!(reference.2, cpu.2);
        // Submission sequences record the actual roster order even when claim
        // priority selects the same recipient after the roster is reversed.
        if policy == DebtSupportPolicy::ReservationOrder {
            assert_eq!(reference.1, cpu.1);
        }
    }
}

fn arrears_market(support: bool, funding: bool, stocked: bool) -> (World, State) {
    use economics_compute_smoke::{
        employment::Earned, finance, marketplace::Side, negotiation::QuotePolicy,
    };
    let (mut w, mut s) = fixture();
    w.activities.orders.clear();
    let (market, opening) = households::market::scenario().unwrap();
    w.town_market = market.town_market;
    s.town_market = opening.town_market;
    s.month = 2;
    s.balances.clear();
    s.balances.insert((PERSON, TOKEN), 2);
    s.balances
        .insert((HOME, GRAIN), if stocked { 2 } else { 0 });
    s.balances.insert((92, TOKEN), 4);
    s.employment.earned.insert(
        (1, 1),
        Earned {
            relief: vec![],
            delivered: 3,
            claim: finance::Obligation {
                transfer: finance::Transfer {
                    from: PERSON,
                    to: WORKER,
                    amount: Amount::new(TOKEN, 6),
                },
                settled: 0,
                condition: finance::Condition::OnOrAfterMonth(2),
                failure: finance::FailureRule::CarryArrears,
            },
        },
    );
    w.households[0].governance.charter.support_member_wages = support;
    w.households[0].governance.charter.fund_earned_wages = funding;
    let c = w.town_market.as_mut().unwrap();
    c.traders.retain(|t| [HOME, 92].contains(&t.trader.agent));
    for t in &mut c.traders {
        t.side = if t.trader.agent == HOME {
            Side::Buy
        } else {
            Side::Sell
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
    m.goods = Amount::new(TOKEN, 4);
    m.payment = GRAIN;
    m.price_tick = 1;
    (w, s)
}
#[test]
fn collective_market_funds_only_enabled_member_wages_and_offsets_private_cash_once() {
    for (support, funding, stocked) in [
        (true, true, true),
        (false, true, true),
        (true, false, true),
        (true, true, false),
    ] {
        let (w, s) = arrears_market(support, funding, stocked);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            while sim.state.phase != Phase::Productive {
                a.step(&mut sim).unwrap();
            }
            let bought = support && funding && stocked;
            assert_eq!(sim.state.balance(HOME, TOKEN), if bought { 4 } else { 0 });
            assert!(!sim.state.employment.earned.contains_key(&(1, 2)));
            assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 6);
            let (mut resumed, mut ra) = (sim.clone(), a.clone());
            through(&mut a, &mut sim, 2);
            through(&mut ra, &mut resumed, 2);
            assert_eq!(
                sim.state.employment.earned[&(1, 1)].claim.outstanding(),
                if bought { 0 } else { 4 }
            );
            assert_eq!(sim.state.balance(WORKER, TOKEN), if bought { 6 } else { 2 });
            assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            assert_eq!(sim.state, resumed.state);
            assert_eq!(a, ra);
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

fn live_loop() -> (World, State) {
    use economics_compute_smoke::household_governance::{Governance, Policy};
    let (mut w, mut s) = arrears_market(true, true, true);
    s.month = 1;
    s.employment = Default::default();
    s.balances.clear();
    s.balances.insert((HOME, TOKEN), 4);
    s.balances.insert((92, TOKEN), 12);
    for p in &mut w.participants {
        p.capacity.quantity = if p.agent == WORKER { 2 } else { 0 };
        w.storage.capacities.insert(p.agent, 100);
    }
    w.households[0].governance = Governance::rotating(PERSON, 12);
    let c = &mut w.households[0].governance.charter;
    c.initial_policy = Policy::NeedsFirst;
    c.support_member_wages = true;
    c.fund_earned_wages = true;
    w.employment[0].capacity.quantity = 2;
    w.employment[0].through = 6;
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: MAKE,
        priority: 0,
        target: Target::Stock(Amount::new(GRAIN, 100)),
    });
    (w, s)
}
fn leave_at_three(w: &mut World, s: &State, leave: bool) {
    if leave && s.month == 3 && s.phase == Phase::Open && w.households[0].membership.is_empty() {
        households::membership::leave(w, s, HOME, PERSON).unwrap();
    }
}
#[test]
fn current_payroll_outlook_removes_funding_lag_but_cannot_create_counterparty_liquidity() {
    use economics_compute_smoke::employment::{PayrollOutlook, projected_payroll};
    for external_coins in [12, 20] {
        for outlook in [PayrollOutlook::EarnedOnly, PayrollOutlook::CurrentDelivery] {
            let (mut w, mut s) = live_loop();
            w.households[0].governance.charter.payroll_outlook = outlook;
            s.balances.insert((92, TOKEN), external_coins);
            let run = |mut w: World, backend| {
                if matches!(backend, Backend::CubeCpu) {
                    w.participants.reverse();
                    w.resources.reverse();
                }
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                let mut a = audit(&w, &s);
                let mut checkpoint = None;
                while sim.state.month <= 6 {
                    if sim.state.phase == Phase::Acquire {
                        let before = sim.state.clone();
                        let books = a.clone();
                        let estimate = projected_payroll(&w, &sim.state).unwrap();
                        assert_eq!(sim.state, before);
                        assert_eq!(a, books);
                        assert!(
                            !sim.state
                                .employment
                                .earned
                                .contains_key(&(1, sim.state.month))
                        );
                        assert!(estimate.get(&(PERSON, TOKEN)).copied().unwrap_or(0) <= 4);
                    }
                    a.step(&mut sim).unwrap();
                    if sim.state.month == 3 && sim.state.phase == Phase::Open {
                        checkpoint = Some((sim.clone(), a.clone()));
                    }
                }
                let anticipates = outlook == PayrollOutlook::CurrentDelivery;
                let working_months = if anticipates {
                    if external_coins == 20 { 6 } else { 5 }
                } else {
                    4
                };
                let paid = if anticipates { external_coins + 4 } else { 12 };
                assert_eq!(sim.state.employment.earned.len(), working_months);
                assert_eq!(sim.state.balance(WORKER, TOKEN), paid);
                assert_eq!(sim.state.balance(PERSON, GRAIN), working_months as i32 * 2);
                assert_eq!(
                    [HOME, PERSON, WORKER, 92]
                        .iter()
                        .map(|id| sim.state.balance(*id, GRAIN))
                        .sum::<i32>(),
                    working_months as i32 * 4
                );
                let owed = if anticipates && external_coins == 20 {
                    0
                } else {
                    4
                };
                assert_eq!(
                    sim.state
                        .employment
                        .earned
                        .values()
                        .map(|e| e.claim.outstanding())
                        .sum::<i32>(),
                    owed
                );
                assert_eq!(
                    [HOME, PERSON, WORKER, 92]
                        .iter()
                        .map(|id| sim.state.balance(*id, TOKEN))
                        .sum::<i32>(),
                    external_coins + 4
                );
                let month_two = sim
                    .state
                    .town_market
                    .history
                    .iter()
                    .find(|r| r.month == 2)
                    .unwrap();
                let receipt = month_two
                    .order_receipts
                    .iter()
                    .find(|r| r.agent == HOME)
                    .unwrap();
                assert_eq!(
                    receipt
                        .deficits_before
                        .as_ref()
                        .unwrap()
                        .get(&TOKEN)
                        .copied(),
                    Some(if anticipates { 4 } else { 0 })
                );
                for id in [PERSON, HOME, WORKER, 92] {
                    let r = a.book().statements(id, 1, 6).unwrap();
                    assert_eq!(r.assets, r.liabilities + r.equity);
                }
                for month in 1..=6 {
                    assert_eq!(value(&a, HOME, A::WagesPayable(1, month)), 0);
                }
                let (mut resumed, mut ra) = checkpoint.unwrap();
                through(&mut ra, &mut resumed, 6);
                assert_eq!(sim.state, resumed.state);
                assert_eq!(sim.ledger, resumed.ledger);
                assert_eq!(a, ra);
                replay(&w, &s, &sim);
                (sim.state, sim.ledger, a)
            };
            assert_eq!(run(w.clone(), Backend::Reference), run(w, Backend::CubeCpu));
        }
    }
}

#[test]
fn payroll_outlook_shares_finite_hours_and_respects_permissions_dates_and_arrears() {
    use economics_compute_smoke::employment::projected_payroll;
    let (mut w, mut s) = live_loop();
    // Two unrelated employers compete for one worker; contract ID breaks equal ranks.
    let mut competing = w.employment[0].clone();
    competing.id = 2;
    competing.employer = 92;
    w.employment.push(competing);
    s.phase = Phase::Acquire;
    s.balances.insert((WORKER, LABOR), 3);
    let before = s.clone();
    assert_eq!(
        projected_payroll(&w, &s).unwrap(),
        [((PERSON, TOKEN), 4), ((92, TOKEN), 2)].into()
    );
    w.employment.reverse();
    assert_eq!(
        projected_payroll(&w, &s).unwrap(),
        [((PERSON, TOKEN), 4), ((92, TOKEN), 2)].into()
    );
    assert_eq!(before, s);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(PERSON_TYPE, Action::CapacityTrade));
    assert!(projected_payroll(&w, &s).unwrap().is_empty());
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::CapacityTrade));
    s.month = 7;
    assert!(projected_payroll(&w, &s).unwrap().is_empty());
    let (w, mut s) = arrears_market(true, true, true);
    s.phase = Phase::Acquire;
    s.balances.insert((WORKER, LABOR), 5);
    assert!(projected_payroll(&w, &s).unwrap().is_empty());
    s.employment.earned.get_mut(&(1, 1)).unwrap().claim.settled = 6;
    assert_eq!(
        projected_payroll(&w, &s).unwrap(),
        [((PERSON, TOKEN), 10)].into()
    );
    s.phase = Phase::Close;
    assert!(projected_payroll(&w, &s).unwrap().is_empty());

    let (mut w, mut s) = live_loop();
    w.employment[0].employer = 92;
    w.employment[0].worker = PERSON;
    w.employment[0].capacity.quantity = 5;
    s.phase = Phase::Acquire;
    s.balances.insert((PERSON, LABOR), 5);
    w.capacity_overrides.insert((s.month, PERSON), 5);
    // One of five own hours belongs to the household, not external employment.
    assert_eq!(
        projected_payroll(&w, &s).unwrap(),
        [((92, TOKEN), 8)].into()
    );
}

#[test]
fn forecast_funding_offsets_private_cash_and_stops_with_membership_or_authorization() {
    use economics_compute_smoke::employment::PayrollOutlook;
    for (support, funding, private, leave, expected) in [
        (true, true, 0, false, 4),
        (true, true, 2, false, 2),
        (true, true, 4, false, 0),
        (false, true, 0, false, 0),
        (true, false, 0, false, 0),
        (true, true, 0, true, 0),
    ] {
        let (mut w, mut s) = live_loop();
        let c = &mut w.households[0].governance.charter;
        c.payroll_outlook = PayrollOutlook::CurrentDelivery;
        c.support_member_wages = support;
        c.fund_earned_wages = funding;
        s.month = 2; // Membership exit is allowed after the founding month.
        s.balances.insert((HOME, TOKEN), 0);
        s.balances.insert((HOME, GRAIN), 2);
        s.balances.insert((PERSON, TOKEN), private);
        if leave {
            households::membership::leave(&mut w, &s, HOME, PERSON).unwrap();
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut a, &mut sim, 2);
            let r = sim.state.town_market.history[0]
                .order_receipts
                .iter()
                .find(|r| r.agent == HOME)
                .unwrap();
            assert_eq!(
                r.deficits_before
                    .as_ref()
                    .unwrap()
                    .get(&TOKEN)
                    .copied()
                    .unwrap_or(0),
                expected
            );
            let bought = expected > 0;
            assert_eq!(sim.state.balance(92, TOKEN), if bought { 8 } else { 12 });
            assert_eq!(
                sim.state.balance(WORKER, TOKEN),
                if bought { 4 } else { private }
            );
            assert_eq!(
                sim.state.employment.earned[&(1, 2)].claim.outstanding(),
                if bought { 0 } else { 4 - private }
            );
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn continued_delivery_combines_old_claims_and_new_payroll_without_offsetting_cash_twice() {
    use economics_compute_smoke::employment::PayrollOutlook;
    let (mut w, s) = arrears_market(true, true, true);
    w.households[0].governance.charter.payroll_outlook = PayrollOutlook::CurrentDelivery;
    w.employment[0].on_arrears = ArrearsPolicy::Continue;
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        while sim.state.phase != Phase::Productive {
            a.step(&mut sim).unwrap();
        }
        let receipt = sim.state.town_market.history[0]
            .order_receipts
            .iter()
            .find(|r| r.agent == HOME)
            .unwrap();
        // Six old + ten projected - two private, not a private offset per claim.
        assert_eq!(receipt.deficits_before.as_ref().unwrap()[&TOKEN], 14);
        assert_eq!(receipt.deficits_after.as_ref().unwrap()[&TOKEN], 10);
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 6);
        assert_eq!(sim.state.employment.earned[&(1, 2)].claim.outstanding(), 10);
        assert_eq!(sim.state.balance(HOME, TOKEN), 4);
        through(&mut a, &mut sim, 2);
        assert_eq!(sim.state.balance(WORKER, TOKEN), 6);
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
        assert_eq!(sim.state.employment.earned[&(1, 2)].claim.outstanding(), 10);
        assert_eq!(value(&a, HOME, A::WagesPayable(1, 2)), 0);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn live_production_pooling_market_payroll_and_member_exit_preserve_claims_and_expose_funding_lag() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    for leave in [false, true] {
        let (w, s) = live_loop();
        let run = |w: World, backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            let mut observer = Observer::new(
                Vec::new(),
                "member-employment",
                Config {
                    settlement: true,
                    ..Default::default()
                },
            )
            .unwrap();
            let mut checkpoint = None;
            while sim.state.month <= 6 {
                leave_at_three(&mut sim.world, &sim.state, leave);
                observer.step_audited(&mut sim, &mut a).unwrap();
                if sim.state.month == 3 && sim.state.phase == Phase::Open {
                    checkpoint = Some((sim.clone(), a.clone()));
                }
            }
            for month in 1..=6 {
                let expected = if [1, 2].contains(&month) || (!leave && [4, 6].contains(&month)) {
                    2
                } else {
                    0
                };
                assert_eq!(
                    sim.state
                        .employment
                        .earned
                        .get(&(1, month))
                        .map_or(0, |e| e.delivered),
                    expected,
                    "month {month}, exit {leave}"
                );
            }
            assert_eq!(
                sim.state
                    .employment
                    .earned
                    .values()
                    .map(|e| e.claim.outstanding())
                    .sum::<i32>(),
                4
            );
            assert_eq!(sim.state.balance(WORKER, TOKEN), if leave { 4 } else { 12 });
            assert_eq!(sim.state.balance(HOME, GRAIN), 4);
            assert_eq!(sim.state.balance(PERSON, GRAIN), if leave { 4 } else { 8 });
            assert_eq!(
                [HOME, PERSON, WORKER, 92]
                    .iter()
                    .map(|id| sim.state.balance(*id, TOKEN))
                    .sum::<i32>(),
                16
            );
            assert_eq!(value(&a, HOME, A::WagesPayable(1, 2)), 0);
            if leave {
                assert_eq!(sim.state.employment.earned[&(1, 2)].claim.outstanding(), 4);
                assert_eq!(
                    economics_compute_smoke::household_governance::authority(
                        &sim.world.households[0],
                        &sim.state
                    )
                    .leader,
                    Some(MEMBER)
                );
                assert!(sim.ledger.iter().filter(|b| b.month >= 3).all(|b| {
                    b.household
                        .as_ref()
                        .unwrap()
                        .reservations
                        .iter()
                        .all(|r| r.request.member != PERSON)
                }));
            }
            for id in [PERSON, HOME, WORKER, 92] {
                let r = a.book().statements(id, 1, 6).unwrap();
                assert_eq!(r.assets, r.liabilities + r.equity);
            }
            let (mut resumed, mut ra) = checkpoint.unwrap();
            while resumed.state.month <= 6 {
                leave_at_three(&mut resumed.world, &resumed.state, leave);
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(sim.state, resumed.state);
            assert_eq!(sim.ledger, resumed.ledger);
            assert_eq!(a, ra);
            let mut replay = s.clone();
            let mut replay_world = w;
            for b in &sim.ledger {
                leave_at_three(&mut replay_world, &replay, leave);
                commit(&replay_world, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(replay, sim.state);
            let logs = String::from_utf8(observer.finish().unwrap()).unwrap();
            assert!(
                logs.lines()
                    .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
                    .any(|r| r["kind"] == "household_allocation"
                        && r["allocated"] == 4
                        && r["purpose"].as_str().unwrap().contains("WageSupport"))
            );
            (sim.state, sim.ledger, a)
        };
        let reference = run(w.clone(), Backend::Reference);
        let mut reordered = w;
        reordered.participants.reverse();
        reordered.resources.reverse();
        reordered.employment.reverse();
        assert_eq!(reference, run(reordered, Backend::CubeCpu));
    }
}

#[test]
fn native_wage_assistance_preserves_unspent_member_stock_when_worker_storage_blocks_payment() {
    for (space, paid) in [(2, 2), (10, 6)] {
        let (mut w, mut s) = fixture();
        w.activities.orders.clear();
        w.employment[0].wage_per_unit.resource = GRAIN;
        w.households[0].governance.charter.support_member_wages = true;
        w.storage.capacities.insert(WORKER, space);
        s.balances.clear();
        s.balances.insert((HOME, GRAIN), 6);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            while sim.state.phase != Phase::Close {
                a.step(&mut sim).unwrap();
            }
            let (mut resumed, mut ra) = (sim.clone(), a.clone());
            a.step(&mut sim).unwrap();
            ra.step(&mut resumed).unwrap();
            let r = sim.ledger.last().unwrap().household.as_ref().unwrap();
            assert_eq!(r.reservations[0].allocated, 6);
            assert_eq!(sim.state.balance(HOME, GRAIN), 0);
            assert_eq!(sim.state.balance(PERSON, GRAIN), 6 - paid);
            assert_eq!(sim.state.balance(WORKER, GRAIN), paid);
            assert_eq!(
                sim.state.employment.earned[&(1, 1)].claim.outstanding(),
                10 - paid
            );
            assert_eq!(value(&a, HOME, A::TransferExpense), 6);
            assert_eq!(value(&a, PERSON, A::Inventory(GRAIN)), i128::from(6 - paid));
            assert_eq!(value(&a, WORKER, A::Inventory(GRAIN)), i128::from(paid));
            assert_eq!(sim.state, resumed.state);
            assert_eq!(a, ra);
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
