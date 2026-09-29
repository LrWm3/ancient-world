use economics_compute_smoke::{
    activities::{Target, WorkOrder},
    compute::Backend,
    financial_reporting::{Audit, Opening},
    household_governance::Policy,
    households::{
        LaborDecision,
        income::scenario::{self, COLLECT_FUEL, FUEL_MARKET},
        market::EXAMPLE_HOUSEHOLD as HOME,
    },
    model::*,
    opportunities::{Action, PERSON_TYPE},
    scenario::{FUEL, GRAIN, NUTRITION, PERSON, SEED, TOKEN, WARMTH},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| [GRAIN, SEED, FUEL].contains(r) && **q > 0)
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            processes: Some(scenario::costs()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn productive(w: World, s: State) -> Simulation {
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    sim
}
fn decision(sim: &Simulation) -> &LaborDecision {
    sim.ledger
        .iter()
        .rev()
        .filter_map(|b| b.household.as_ref())
        .flat_map(|h| &h.labor)
        .next()
        .unwrap()
}

#[test]
fn reciprocal_income_feeds_members_for_a_year_with_finite_coins_and_separate_books() {
    let (w, s) = scenario::scenario().unwrap();
    let total_coins = |s: &State| {
        s.balances
            .iter()
            .filter(|((_, r), _)| *r == TOKEN)
            .map(|(_, q)| *q)
            .sum::<i32>()
    };
    let run = |backend| {
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while sim.state.month <= 12 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(total_coins(&s), total_coins(&sim.state));
        assert_eq!(sim.state.balance(HOME, TOKEN), 60);
        assert_eq!(
            sim.state
                .town_market
                .history
                .iter()
                .map(|r| r.markets[&FUEL_MARKET].volume)
                .sum::<i32>(),
            11
        );
        assert_eq!(
            sim.reports
                .iter()
                .filter(|r| [PERSON, 91].contains(&r.agent))
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>(),
            0
        );
        for agent in &w.agents {
            let r = a.book().statements(agent.id, 1, 12).unwrap();
            assert_eq!(r.assets, r.liabilities + r.equity);
        }
        for d in sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
        {
            assert!(d.granted <= 2);
            assert!(
                d.contributions
                    .iter()
                    .all(|c| c.directed <= c.reserved && c.directed + c.returned == c.reserved)
            );
        }
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn forecast_does_not_create_cash_and_receipts_explain_incremental_income() {
    let (w, s) = scenario::scenario().unwrap();
    let mut sim = productive(w, s);
    let cash = sim.state.balance(HOME, TOKEN);
    sim.step().unwrap();
    let d = decision(&sim);
    assert_eq!(d.recipient, Some(PERSON));
    assert_eq!(d.baseline_income.as_ref().unwrap().net_coins, -40);
    assert_eq!(d.projected_income.as_ref().unwrap().net_coins, 0);
    assert_eq!(d.projected_income.as_ref().unwrap().sales[&FUEL_MARKET], 1);
    assert_eq!(d.projected_income.as_ref().unwrap().through, 2);
    assert_eq!(sim.state.balance(HOME, TOKEN), cash);
    assert_eq!(sim.state.balance(HOME, FUEL), 1);
}

#[test]
fn absent_demand_or_unfunded_or_uncrossed_buyers_do_not_justify_extra_work() {
    for mode in 0..3 {
        let (mut w, s) = scenario::scenario().unwrap();
        if mode == 0 {
            w.participants
                .iter_mut()
                .find(|p| p.agent == 89)
                .unwrap()
                .needs
                .retain(|n| n.resource != WARMTH);
        }
        if mode == 2 {
            let entry = w.town_market.as_mut().unwrap().additional[0]
                .traders
                .iter_mut()
                .find(|t| t.trader.agent == HOME)
                .unwrap();
            entry.trader.limit = 200;
            entry.trader.opening_quote = 200;
        }
        let mut sim = productive(w, s);
        if mode == 1 {
            sim.state.balances.insert((89, TOKEN), 0);
        }
        sim.step().unwrap();
        let d = decision(&sim);
        assert_eq!(d.granted, 0, "mode {mode}");
        assert!(d.contributions.iter().all(|c| c.returned == c.reserved));
        assert_eq!(sim.state.balance(HOME, FUEL), 0);
    }
}

#[test]
fn output_policy_and_income_policy_differ_on_unsaleable_surplus() {
    for policy in [Policy::NeedsFirst, Policy::NeedsThenIncome] {
        let (mut w, s) = scenario::scenario().unwrap();
        w.households[0].governance.charter.initial_policy = policy;
        w.participants
            .iter_mut()
            .find(|p| p.agent == 89)
            .unwrap()
            .needs
            .retain(|n| n.resource != WARMTH);
        let mut sim = productive(w, s);
        sim.step().unwrap();
        assert_eq!(
            decision(&sim).granted,
            if policy == Policy::NeedsFirst { 2 } else { 0 }
        );
    }
}

#[test]
fn current_food_needs_precede_a_higher_next_market_cash_return() {
    let (mut w, mut s) = scenario::scenario().unwrap();
    s.balances.insert((HOME, TOKEN), 0);
    let mut food = w.definition(COLLECT_FUEL).clone();
    food.id = 97;
    food.outputs = vec![Amount::new(GRAIN, 2)];
    w.definitions.push(food);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::Process(97)));
    w.households[0]
        .governance
        .constitution
        .activities
        .as_mut()
        .unwrap()
        .insert(97);
    w.activities.orders.push(WorkOrder {
        agent: 91,
        definition: 97,
        priority: 0,
        target: Target::Stock(Amount::new(GRAIN, 20)),
    });
    let mut sim = productive(w, s);
    sim.step().unwrap();
    let d = decision(&sim);
    assert_eq!(d.recipient, Some(91));
    assert_eq!(d.projected_needs.as_ref().unwrap()[0].unmet, 0);
    assert_eq!(d.projected_income.as_ref().unwrap().net_coins, 0);
}

#[test]
fn legal_mandate_inputs_and_real_contributions_still_bound_income_work() {
    for mode in 0..4 {
        let (mut w, s) = scenario::scenario().unwrap();
        match mode {
            0 => {
                w.households[0].governance.constitution.activities = Some(Default::default());
            }
            1 => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(PERSON_TYPE, Action::Process(COLLECT_FUEL)));
            }
            2 => {
                w.definitions
                    .iter_mut()
                    .find(|d| d.id == COLLECT_FUEL)
                    .unwrap()
                    .stages[0]
                    .entry_inputs
                    .push(Amount::new(SEED, 1));
            }
            _ => {
                w.capacity_overrides.insert((1, 91), 0);
            }
        }
        let mut sim = productive(w, s);
        sim.step().unwrap();
        assert_eq!(decision(&sim).granted, 0, "mode {mode}");
        assert_eq!(sim.state.balance(HOME, FUEL), 0);
    }
}

#[test]
fn income_receipts_and_transfers_replay_atomically() {
    let (w, s) = scenario::scenario().unwrap();
    let mut sim = productive(w.clone(), s);
    let opening = sim.state.clone();
    sim.step().unwrap();
    let b = sim.ledger.last().unwrap();
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let mut state = opening.clone();
        commit(&w, &mut state, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        assert_eq!(state, sim.state);
        for mode in 0..2 {
            let mut bad = b.clone();
            let h = bad.household.as_mut().unwrap();
            if mode == 0 {
                h.labor[0].projected_income.as_mut().unwrap().net_coins += 1;
            } else {
                h.before[0].delta += 1;
            }
            let mut state = opening.clone();
            assert!(commit(&w, &mut state, &bad, backend, DEFAULT_EFFECT_LIMIT).is_err());
            assert_eq!(state, opening);
        }
    }
}

#[test]
fn reordered_cpu_checkpoint_continuation_preserves_income_decisions() {
    let (mut w, s) = scenario::scenario().unwrap();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    reference.run_months(3).unwrap();
    w.participants.reverse();
    w.definitions.reverse();
    w.town_market.as_mut().unwrap().traders.reverse();
    w.town_market.as_mut().unwrap().additional[0]
        .traders
        .reverse();
    let mut cpu = Simulation::new(w.clone(), s, Backend::CubeCpu).unwrap();
    cpu.run_months(1).unwrap();
    let mut resumed = Simulation::new(w, cpu.state.clone(), Backend::CubeCpu).unwrap();
    cpu.run_months(2).unwrap();
    resumed.run_months(2).unwrap();
    assert_eq!(resumed.state, cpu.state);
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.ledger, cpu.ledger);
    assert_eq!(reference.reports, cpu.reports);
}

#[test]
fn lost_counterparty_permission_breaks_the_forecast_without_creating_income() {
    let (w, s) = scenario::scenario().unwrap();
    let mut sim = productive(w, s);
    sim.step().unwrap();
    assert_eq!(
        decision(&sim).projected_income.as_ref().unwrap().sales[&FUEL_MARKET],
        1
    );
    while sim.state.month < 2 {
        sim.step().unwrap();
    }
    // Isolate a venue rule change for the buyer by revoking all person trading;
    // the household remains permitted but its counterparties cannot transact.
    sim.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(PERSON_TYPE, Action::StockTrade));
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(
        sim.state.town_market.history.last().unwrap().markets[&FUEL_MARKET].volume,
        0
    );
    assert_eq!(sim.state.balance(HOME, TOKEN), 60);
    sim.step().unwrap();
    assert_eq!(decision(&sim).granted, 0);
}

#[test]
fn dated_governor_instruction_switches_the_income_objective() {
    use economics_compute_smoke::household_governance::{PolicyChange, schedule};
    let (mut w, s) = scenario::scenario().unwrap();
    w.households[0].governance.charter.initial_policy = Policy::NeedsFirst;
    w.participants
        .iter_mut()
        .find(|p| p.agent == 89)
        .unwrap()
        .needs
        .retain(|n| n.resource != WARMTH);
    schedule(
        &mut w,
        &s,
        HOME,
        PolicyChange {
            month: 2,
            authorized_by: PERSON,
            policy: Policy::NeedsThenIncome,
        },
    )
    .unwrap();
    let mut sim = productive(w, s);
    sim.step().unwrap();
    assert_eq!(decision(&sim).granted, 2);
    while sim.state.month < 2 || sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    sim.step().unwrap();
    assert_eq!(decision(&sim).policy, Policy::NeedsThenIncome);
    assert_eq!(decision(&sim).granted, 0);
}

#[test]
fn income_objective_requires_explicit_constitution_and_supported_market() {
    for mode in 0..3 {
        let (mut w, s) = scenario::scenario().unwrap();
        match mode {
            0 => {
                w.town_market = None;
            }
            1 => {
                w.households[0]
                    .governance
                    .constitution
                    .permitted_policies
                    .remove(&Policy::NeedsThenIncome);
            }
            _ => {
                w.households[0].governance.charter.contribution =
                    economics_compute_smoke::household_governance::Contribution::SpareLabor;
            }
        }
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}

#[test]
fn observer_exposes_forecast_assumptions_without_affecting_decisions() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let (w, s) = scenario::scenario().unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut plain = sim.clone();
    let mut observer = Observer::new(
        vec![],
        "income",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 1).unwrap();
    plain.run_months(1).unwrap();
    assert_eq!(sim.state, plain.state);
    assert_eq!(sim.ledger, plain.ledger);
    let text = String::from_utf8(observer.finish().unwrap()).unwrap();
    let rows: Vec<serde_json::Value> = text
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let row = rows
        .iter()
        .find(|r| r["kind"] == "household_labor")
        .unwrap();
    assert_eq!(row["policy"], "NeedsThenIncome");
    assert_eq!(row["projected_income"]["observed_through"], 1);
    assert_eq!(row["projected_income"]["through"], 2);
    assert_eq!(row["projected_income"]["net_coins"], 0);
}

#[test]
fn finite_private_work_target_eventually_stops_collective_income() {
    let (w, s) = scenario::scenario().unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(27).unwrap();
    assert_eq!(sim.state.balance(PERSON, FUEL), 24);
    assert_eq!(sim.state.balance(HOME, TOKEN), 20);
    assert_eq!(
        sim.state.town_market.history[24].markets[&FUEL_MARKET].volume,
        1
    );
    assert_eq!(
        sim.state.town_market.history[25].markets[&FUEL_MARKET].volume,
        0
    );
    assert_eq!(decision(&sim).granted, 0);
    assert_eq!(
        sim.reports
            .iter()
            .filter(|r| r.month == 27 && [PERSON, 91].contains(&r.agent))
            .map(|r| r.deficit(NUTRITION))
            .sum::<i32>(),
        2
    );
}

#[test]
fn voluntary_surplus_closes_the_loop_for_ten_years_on_cpu_with_separate_books() {
    let (w, s) = scenario::coordinated().unwrap();
    let run = |backend| {
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while sim.state.month <= 120 {
            a.step(&mut sim).unwrap();
            assert!(sim.state.balance(PERSON, FUEL) <= 3);
        }
        assert_eq!(sim.state.balance(HOME, TOKEN), 60);
        assert!(
            sim.reports
                .iter()
                .filter(|r| [PERSON, 91].contains(&r.agent))
                .all(|r| r.deficit(NUTRITION) == 0)
        );
        assert_eq!(
            sim.state
                .town_market
                .history
                .iter()
                .map(|r| r.markets[&FUEL_MARKET].volume)
                .sum::<i32>(),
            119
        );
        assert_eq!(
            s.balances
                .iter()
                .filter(|((_, r), _)| *r == TOKEN)
                .map(|(_, q)| q)
                .sum::<i32>(),
            sim.state
                .balances
                .iter()
                .filter(|((_, r), _)| *r == TOKEN)
                .map(|(_, q)| q)
                .sum::<i32>()
        );
        let support: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.support)
            .filter(|r| r.accepted > 0)
            .collect();
        assert!(!support.is_empty());
        assert!(support.iter().all(|r| r.accepted == 1 && r.protected >= 2));
        let decisions: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .collect();
        assert!(decisions.iter().any(|d| d.granted == 0));
        assert!(decisions.iter().any(|d| d.granted == 2));
        for agent in &w.agents {
            let r = a.book().statements(agent.id, 1, 120).unwrap();
            assert_eq!(r.assets, r.liabilities + r.equity);
        }
        (sim.state, sim.ledger, sim.reports, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn support_is_voluntary_prospective_bounded_and_atomic() {
    use economics_compute_smoke::households::support::authorize;
    let (mut w, s) = scenario::coordinated().unwrap();
    let mandate = w.households[0].support.pop().unwrap();
    let original = w.clone();
    assert!(authorize(&mut w, &s, HOME, 91, mandate.clone()).is_err());
    assert_eq!(w, original);
    authorize(&mut w, &s, HOME, PERSON, mandate.clone()).unwrap();
    let original = w.clone();
    assert!(authorize(&mut w, &s, HOME, PERSON, mandate).is_err());
    assert_eq!(w, original);
    let mut sim = productive(w, s);
    sim.state.balances.insert((PERSON, FUEL), 24);
    let opening = sim.state.clone();
    sim.step().unwrap();
    let b = sim.ledger.last().unwrap();
    let r = &b.household.as_ref().unwrap().support[0];
    assert_eq!((r.protected, r.offered, r.accepted), (2, 1, 1));
    assert_eq!(sim.state.balance(PERSON, FUEL), 23);
    assert_eq!(sim.state.balance(HOME, FUEL), 1);
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let mut replay = opening.clone();
        commit(&sim.world, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        assert_eq!(replay, sim.state);
        let mut bad = b.clone();
        bad.household.as_mut().unwrap().support[0].accepted += 1;
        let mut rejected = opening.clone();
        assert!(
            commit(
                &sim.world,
                &mut rejected,
                &bad,
                backend,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(rejected, opening);
    }
}

#[test]
fn unavailable_demand_private_needs_and_expired_consent_prevent_surplus_capture() {
    for mode in 0..5 {
        let (mut w, mut s) = scenario::coordinated().unwrap();
        s.balances.insert((PERSON, FUEL), 3);
        match mode {
            0 => w.households[0].support[0].from = 2,
            1 => w
                .participants
                .iter_mut()
                .find(|p| p.agent == 89)
                .unwrap()
                .needs
                .retain(|n| n.resource != WARMTH),
            2 => w
                .participants
                .iter_mut()
                .find(|p| p.agent == PERSON)
                .unwrap()
                .needs
                .push(Requirement {
                    resource: WARMTH,
                    quantity: 3,
                    priority: 1,
                }),
            3 => w.households[0].governance.charter.initial_policy = Policy::NeedsFirst,
            _ => w.households[0].support[0].private_reserve = 3,
        }
        let mut sim = productive(w, s);
        sim.step().unwrap();
        assert!(
            sim.ledger
                .last()
                .unwrap()
                .household
                .as_ref()
                .unwrap()
                .support
                .iter()
                .all(|r| r.accepted == 0),
            "mode {mode}"
        );
    }
    let (mut w, s) = scenario::coordinated().unwrap();
    w.households[0].support[0].through = 1;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(30).unwrap();
    assert!(
        sim.ledger
            .iter()
            .filter(|b| b.month > 1)
            .filter_map(|b| b.household.as_ref())
            .all(|h| h.support.is_empty())
    );
    assert!(
        sim.reports
            .iter()
            .any(|r| r.month == 30 && r.agent == PERSON && r.deficit(NUTRITION) > 0)
    );
}

#[test]
fn coordinated_income_recovers_after_a_temporary_market_demand_loss() {
    let (w, s) = scenario::coordinated().unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(10).unwrap();
    sim.world.town_market.as_mut().unwrap().additional[0].match_limit = Some(0);
    sim.run_months(3).unwrap();
    assert!(
        sim.ledger
            .iter()
            .filter(|b| (11..=13).contains(&b.month))
            .filter_map(|b| b.household.as_ref())
            .all(|h| h.support.iter().all(|r| r.accepted == 0)
                && h.labor.iter().all(|d| d.granted == 0))
    );
    sim.world.town_market.as_mut().unwrap().additional[0].match_limit = None;
    sim.run_months(12).unwrap();
    assert!(
        sim.reports
            .iter()
            .filter(|r| r.month >= 15 && [PERSON, 91].contains(&r.agent))
            .all(|r| r.deficit(NUTRITION) == 0)
    );
    assert!(sim.state.balance(HOME, TOKEN) >= 40);
}

#[test]
fn coordinated_checkpoint_and_reordered_inputs_preserve_support_and_work() {
    let (mut w, s) = scenario::coordinated().unwrap();
    let mut plain = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    plain.run_months(36).unwrap();
    w.participants.reverse();
    w.definitions.reverse();
    w.town_market.as_mut().unwrap().traders.reverse();
    let mut cpu = Simulation::new(w.clone(), s, Backend::CubeCpu).unwrap();
    cpu.run_months(24).unwrap();
    let mut resumed = Simulation::new(w, cpu.state.clone(), Backend::CubeCpu).unwrap();
    resumed.run_months(12).unwrap();
    cpu.run_months(12).unwrap();
    assert_eq!(resumed.state, cpu.state);
    assert_eq!(
        resumed.ledger,
        cpu.ledger
            .iter()
            .filter(|b| b.month > 24)
            .cloned()
            .collect::<Vec<_>>()
    );
    assert_eq!(cpu.state, plain.state);
    assert_eq!(cpu.ledger, plain.ledger);
}

#[test]
fn member_withdrawal_preserves_history_and_stops_new_transfers_next_month() {
    use economics_compute_smoke::households::support::revoke;
    let (w, mut s) = scenario::coordinated().unwrap();
    s.balances.insert((PERSON, FUEL), 3);
    let mut sim = productive(w, s);
    let before = sim.state.clone();
    sim.step().unwrap();
    let batch = sim.ledger.last().unwrap().clone();
    let after = sim.state.clone();
    let original = sim.world.clone();
    assert!(revoke(&mut sim.world, &sim.state, HOME, 91, FUEL, 1).is_err());
    assert_eq!(sim.world, original);
    revoke(&mut sim.world, &sim.state, HOME, PERSON, FUEL, 1).unwrap();
    let mut replay = before;
    commit(
        &sim.world,
        &mut replay,
        &batch,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    assert_eq!(replay, after);
    while sim.state.month <= 4 {
        sim.step().unwrap();
    }
    assert!(
        sim.ledger
            .iter()
            .filter(|b| b.month >= 2)
            .filter_map(|b| b.household.as_ref())
            .all(|h| h.support.is_empty())
    );
}

#[test]
fn committed_inputs_are_not_voluntary_surplus() {
    let (mut w, mut s) = scenario::coordinated().unwrap();
    w.definitions.push(ProcessDefinition {
        id: 101,
        name: "committed fuel use".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 2,
            entry_inputs: vec![Amount::new(FUEL, 3)],
            monthly_services: vec![],
        }],
        outputs: vec![Amount::new(FUEL, 1)],
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::Process(101)));
    s.balances.insert((PERSON, FUEL), 3);
    s.processes.insert(
        1,
        ProcessInstance {
            id: 1,
            definition: 101,
            operator: PERSON,
            beneficiary: PERSON,
            goal: None,
            asset: None,
            right: None,
            start: 1,
            reserved_through: 2,
            stage: 0,
            elapsed: 0,
            status: Status::Active,
        },
    );
    let mut sim = productive(w, s);
    sim.step().unwrap();
    let r = &sim
        .ledger
        .last()
        .unwrap()
        .household
        .as_ref()
        .unwrap()
        .support[0];
    assert_eq!((r.protected, r.offered, r.accepted), (3, 0, 0));
    assert_eq!(sim.state.processes[&1].status, Status::Active);
    assert_eq!(sim.state.processes[&1].elapsed, 1);
}

#[test]
fn support_observer_explains_transfers_without_changing_execution() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let (w, mut s) = scenario::coordinated().unwrap();
    s.balances.insert((PERSON, FUEL), 3);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut plain = sim.clone();
    let mut observer = Observer::new(
        vec![],
        "support",
        Config {
            settlement: true,
            ..Default::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 1).unwrap();
    plain.run_months(1).unwrap();
    assert_eq!(sim.state, plain.state);
    assert_eq!(sim.ledger, plain.ledger);
    let rows = String::from_utf8(observer.finish().unwrap()).unwrap();
    let r: serde_json::Value = rows
        .lines()
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .find(|r| r["kind"] == "household_support")
        .unwrap();
    assert_eq!(r["receipt"]["accepted"], 1);
    assert_eq!(r["receipt"]["protected"], 2);
}

#[test]
fn withdrawn_mandates_allow_new_prospective_terms_including_cancelled_future_offers() {
    use economics_compute_smoke::households::support::{authorize, revoke};
    for start in [1, 10] {
        let (mut w, s) = scenario::coordinated().unwrap();
        w.households[0].support[0].from = start;
        let mut replacement = w.households[0].support[0].clone();
        revoke(&mut w, &s, HOME, PERSON, FUEL, start).unwrap();
        replacement.from = 2;
        replacement.private_reserve = 4;
        authorize(&mut w, &s, HOME, PERSON, replacement).unwrap();
        assert_eq!(w.households[0].support.len(), 2);
    }
}

#[test]
fn collective_target_caps_competing_member_offers_in_policy_order() {
    use economics_compute_smoke::{household_governance::TieBreak, households::support::authorize};
    let (mut w, mut s) = scenario::coordinated().unwrap();
    w.households[0].governance.charter.tie_break = TieBreak::MemberId;
    let mut other = w.households[0].support[0].clone();
    other.member = 91;
    authorize(&mut w, &s, HOME, 91, other).unwrap();
    w.households[0].support.reverse();
    s.balances.insert((PERSON, FUEL), 3);
    s.balances.insert((91, FUEL), 3);
    let mut sim = productive(w, s);
    sim.step().unwrap();
    let r = &sim
        .ledger
        .last()
        .unwrap()
        .household
        .as_ref()
        .unwrap()
        .support;
    assert_eq!(r.iter().map(|r| r.accepted).sum::<i32>(), 1);
    assert_eq!(r[0].mandate.member, PERSON);
    assert_eq!(r[0].accepted, 1);
    assert_eq!(r[1].accepted, 0);
    assert_eq!(sim.state.balance(91, FUEL), 3);
}

#[test]
fn support_cannot_exceed_collective_storage() {
    let (mut w, mut s) = scenario::coordinated().unwrap();
    w.resources.push(Resource {
        id: 100,
        name: "stored material".into(),
        kind: ResourceKind::Stock,
    });
    w.storage.weights.insert(100, 1);
    s.balances.insert((PERSON, FUEL), 3);
    s.balances.insert((HOME, 100), 64);
    let mut sim = productive(w, s);
    sim.step().unwrap();
    let r = &sim
        .ledger
        .last()
        .unwrap()
        .household
        .as_ref()
        .unwrap()
        .support[0];
    assert_eq!(r.offered, 1);
    assert_eq!(r.accepted, 0);
    assert_eq!(r.reason, "collective storage unavailable");
    assert_eq!(sim.state.balance(PERSON, FUEL), 3);
}
