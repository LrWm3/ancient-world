use economics_compute_smoke::{
    compute::Backend,
    minting::{
        self,
        provisioning::{self, Choice, ProvisionGoal},
        *,
    },
    model::*,
    simulation::Simulation,
};
fn sim(case: &str, goal: ProvisionGoal, backend: Backend) -> Simulation {
    let (w, s) = minting::provision_policy_scenario(case, goal).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn deficit(s: &Simulation) -> i32 {
    s.reports.iter().map(|r| r.deficit(NUTRITION)).sum()
}
fn choice(s: &Simulation, month: u32, agent: AgentId) -> &provisioning::Decision {
    s.ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .find(|b| b.month == month)
        .unwrap()
        .plan
        .as_ref()
        .unwrap()
        .provision
        .iter()
        .find(|d| d.agent == agent)
        .unwrap()
}
#[test]
fn identical_resources_show_scarcity_gain_and_adequate_supply_tradeoff() {
    for (case, full, incremental) in [
        ("adequate", 0, 1),
        ("tight", 10, 8),
        ("empty", 12, 12),
        ("endowed", 0, 0),
        ("recovery", 10, 6),
    ] {
        let mut a = sim(case, ProvisionGoal::FullBuffer, Backend::CubeCpu);
        let mut b = sim(case, ProvisionGoal::Incremental, Backend::CubeCpu);
        assert_eq!(a.state, b.state);
        assert_eq!(a.world.definitions, b.world.definitions);
        assert_eq!(a.world.scheduled_starts, b.world.scheduled_starts);
        a.run_months(6).unwrap();
        b.run_months(6).unwrap();
        assert_eq!(deficit(&a), full, "{case}");
        assert_eq!(deficit(&b), incremental, "{case}");
        if case == "tight" {
            assert_eq!(choice(&a, 1, SUPPLIER).choice, Choice::NoFoodAccess);
            assert_eq!(choice(&b, 1, SUPPLIER).choice, Choice::SeekIncome);
            assert_eq!(choice(&b, 1, SUPPLIER).purchase_target, 1);
            assert_eq!(choice(&b, 1, SUPPLIER).cash_gap, 3);
        }
    }
}
#[test]
fn actual_finite_release_reactivates_work_only_after_it_is_visible() {
    let mut s = sim("recovery", ProvisionGoal::Incremental, Backend::CubeCpu);
    s.run_months(6).unwrap();
    for a in [SUPPLIER, WORKER] {
        assert_eq!(choice(&s, 3, a).choice, Choice::NoFoodAccess);
        assert_eq!(choice(&s, 4, a).choice, Choice::SeekIncome);
    }
    // Check actual process definition rather than relying on transaction labels.
    let id = s
        .world
        .definitions
        .iter()
        .find(|d| d.name == "release stored grain")
        .unwrap()
        .id;
    let completed: Vec<_> = s
        .ledger
        .iter()
        .filter(|b| {
            b.transactions.iter().any(|t| {
                t.process.as_ref().is_some_and(|p| {
                    p.after.definition == id && p.after.status == Status::Completed
                })
            })
        })
        .collect();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].month, 3);
    let reserve = s
        .world
        .resources
        .iter()
        .find(|r| r.name == "sealed grain reserve")
        .unwrap()
        .id;
    assert_eq!(s.state.balance(ISSUER, reserve), 0);
    assert_eq!(
        s.state.balance(ISSUER, WHEAT)
            + s.reports
                .iter()
                .map(|r| r.fulfilled(NUTRITION))
                .sum::<i32>(),
        8
    );
}
#[test]
fn partially_funded_food_is_not_a_complete_buffer_or_leisure() {
    let mut s = sim("tight", ProvisionGoal::Incremental, Backend::Reference);
    s.state.balances.insert((SUPPLIER, WHEAT), 1);
    s.state.balances.insert((ISSUER, WHEAT), 2);
    s.state.balances.insert((SUPPLIER, COIN), 3);
    let c = s.world.minting.as_ref().unwrap();
    let p = c.order_policy.as_ref().unwrap();
    let v = p.provisioning.as_ref().unwrap();
    let d = provisioning::decision(&s.world, &s.state, c, p, v, SUPPLIER);
    assert_eq!(d.choice, Choice::AwaitOpportunity);
    assert_eq!(d.cash_gap, 0);
    assert_eq!(d.food_required, 3);
    assert_eq!(d.purchase_target, 1);
    assert!(!provisioning::activity_allowed(
        &s.world, &s.state, SUPPLIER, REST
    ));
    s.state.balances.insert((SUPPLIER, WHEAT), 0);
    assert_eq!(
        provisioning::decision(&s.world, &s.state, c, p, v, SUPPLIER).choice,
        Choice::AwaitFood
    );
    s.state.balances.insert((SUPPLIER, WHEAT), 3);
    assert_eq!(
        provisioning::decision(&s.world, &s.state, c, p, v, SUPPLIER).choice,
        Choice::Covered
    );
}
#[test]
fn both_policies_match_reference_under_reordering_and_phase_restart() {
    for case in ["tight", "recovery"] {
        for goal in [ProvisionGoal::FullBuffer, ProvisionGoal::Incremental] {
            let mut reference = sim(case, goal, Backend::Reference);
            reference.run_months(6).unwrap();
            let mut cpu = sim(case, goal, Backend::CubeCpu);
            cpu.world.participants.reverse();
            cpu.world.definitions.reverse();
            cpu.world.scheduled_starts.reverse();
            cpu.world.activities.orders.reverse();
            while cpu.state.month <= 6 {
                cpu.step().unwrap();
                cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                    .unwrap()
                    .state;
            }
            assert_eq!(reference.state, cpu.state);
            assert_eq!(reference.ledger, cpu.ledger);
            assert_eq!(reference.reports, cpu.reports);
        }
    }
}
#[test]
fn telemetry_records_goal_target_and_reassessment_without_mutation() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let mut s = sim("recovery", ProvisionGoal::Incremental, Backend::CubeCpu);
    let mut o = Observer::new(
        Vec::new(),
        "partial",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    o.run_months(&mut s, 6).unwrap();
    let bytes = o.finish().unwrap();
    let rows: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(rows.iter().any(|r| r["kind"] == "physical_minting_orders"
        && r["provision"][0]["goal"] == "Incremental"
        && r["provision"][0]["purchase_target"] == 1));
    let mut plain = sim("recovery", ProvisionGoal::Incremental, Backend::CubeCpu);
    plain.run_months(6).unwrap();
    assert_eq!(plain.state, s.state);
    assert_eq!(plain.ledger, s.ledger);
}
