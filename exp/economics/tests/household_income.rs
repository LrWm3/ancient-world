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
