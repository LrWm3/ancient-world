use economics_compute_smoke::{
    compute::Backend,
    minting::{self, *},
    model::*,
    simulation::Simulation,
};
fn sim(case: &str, backend: Backend) -> Simulation {
    let (w, s) = minting::repeated_scenario(case).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn completions(s: &Simulation) -> Vec<u32> {
    s.ledger
        .iter()
        .filter(|b| {
            b.transactions.iter().any(|t| {
                t.process.as_ref().is_some_and(|p| {
                    p.after.definition == MINT && p.after.status == Status::Completed
                })
            })
        })
        .map(|b| b.month)
        .collect()
}
#[test]
fn repeated_cpu_cycles_conserve_ore_and_distribute_new_coins() {
    let mut s = sim("normal", Backend::CubeCpu);
    s.run_months(6).unwrap();
    assert_eq!(completions(&s), vec![2, 4, 6]);
    assert_eq!(s.state.balance(ISSUER, COIN), 18);
    assert_eq!(s.state.balance(SUPPLIER, COIN), 9);
    assert_eq!(s.state.balance(WORKER, COIN), 15);
    assert_eq!(s.state.balance(SUPPLIER, ORE), 0);
    assert_eq!(s.state.balance(SUPPLIER, METAL), 0);
    assert_eq!(s.state.balance(WORKER, FIREWOOD), 3);
    // Only initial wheat sales; later procurement uses completed prior issuance.
    let sales: Vec<_> = s
        .ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .flat_map(|b| &b.deals)
        .filter(|d| d.market == WHEAT)
        .collect();
    assert_eq!(sales.len(), 2);
    assert!(sales.iter().all(|d| d.month == 1));
    let before = s
        .state
        .balances
        .iter()
        .filter(|((_, r), _)| *r == COIN)
        .map(|(_, v)| *v)
        .sum::<i32>();
    s.run_months(2).unwrap();
    assert_eq!(completions(&s), vec![2, 4, 6]);
    assert_eq!(
        before,
        s.state
            .balances
            .iter()
            .filter(|((_, r), _)| *r == COIN)
            .map(|(_, v)| *v)
            .sum::<i32>()
    );
}
#[test]
fn exhaustion_labor_interruption_and_insufficient_yield_have_distinct_outcomes() {
    for (case, months, treasury, firewood) in [
        ("ore", vec![2], 10, 5),
        ("labor", vec![2, 6], 14, 3),
        ("low_yield", vec![2], 4, 5),
    ] {
        let mut s = sim(case, Backend::CubeCpu);
        s.run_months(6).unwrap();
        assert_eq!(completions(&s), months, "{case}");
        assert_eq!(s.state.balance(ISSUER, COIN), treasury, "{case}");
        assert_eq!(s.state.balance(WORKER, FIREWOOD), firewood, "{case}");
        let fourth = s
            .ledger
            .iter()
            .filter_map(|b| b.minting.as_ref())
            .find(|b| b.month == 4)
            .unwrap();
        assert!(fourth.transactions.is_empty());
        let reason = &fourth.plan.as_ref().unwrap().reason;
        assert!(reason.contains(if case == "low_yield" {
            "insufficient opening funds"
        } else {
            "input package unmatched"
        }));
        if case == "labor" {
            assert!(reason.contains(&format!("market {HOURS}")));
        }
        if case == "ore" {
            assert!(reason.contains(&format!("market {METAL}")));
        }
    }
}
#[test]
fn new_metal_is_not_available_to_an_earlier_acquire_boundary() {
    let mut s = sim("normal", Backend::CubeCpu);
    s.world
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .month = 1;
    s.world
        .scheduled_starts
        .iter_mut()
        .find(|p| p.month == 2)
        .unwrap()
        .month = 1;
    s.state.balances.insert((ISSUER, COIN), 6);
    s.run_months(1).unwrap();
    assert!(completions(&s).is_empty());
    assert_eq!(s.state.balance(SUPPLIER, METAL), 2);
    assert_eq!(s.state.balance(ISSUER, COIN), 6);
    assert!(
        s.ledger
            .iter()
            .filter_map(|b| b.minting.as_ref())
            .all(|b| b.deals.is_empty())
    );
}
#[test]
fn reference_cpu_batched_and_phase_restart_match_for_all_controls() {
    for case in ["normal", "ore", "labor", "low_yield"] {
        let mut reference = sim(case, Backend::Reference);
        reference.run_months(6).unwrap();
        let mut cpu = sim(case, Backend::CubeCpu);
        cpu.world.scheduled_starts.reverse();
        cpu.world.activities.orders.reverse();
        cpu.world.participants.reverse();
        cpu.world.definitions.reverse();
        while cpu.state.month <= 6 {
            cpu.step().unwrap();
            cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                .unwrap()
                .state;
        }
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        let mut monthly = sim(case, Backend::Reference);
        for _ in 0..6 {
            monthly.run_months(1).unwrap();
        }
        assert_eq!(reference.state, monthly.state);
        assert_eq!(reference.ledger, monthly.ledger);
    }
}
#[test]
fn targets_must_match_unique_dated_production_requests() {
    for case in ["earlier", "duplicate", "missing"] {
        let (mut w, s) = minting::repeated_scenario("normal").unwrap();
        match case {
            "earlier" => {
                w.minting
                    .as_mut()
                    .unwrap()
                    .order_policy
                    .as_mut()
                    .unwrap()
                    .additional_months
                    .insert(1);
            }
            "duplicate" => w.scheduled_starts.push(w.scheduled_starts[0].clone()),
            _ => {
                w.scheduled_starts.pop();
            }
        }
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}
#[test]
fn telemetry_preserves_target_dates_across_a_missed_cycle() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let mut s = sim("labor", Backend::CubeCpu);
    let mut o = Observer::new(
        Vec::new(),
        "cycles",
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
    let targets: Vec<_> = rows
        .iter()
        .filter(|r| r["kind"] == "physical_minting_orders")
        .map(|r| r["target_month"].as_u64().unwrap())
        .collect();
    assert_eq!(targets, vec![2, 2, 4, 4, 6, 6]);
    let mut plain = sim("labor", Backend::CubeCpu);
    plain.run_months(6).unwrap();
    assert_eq!(s.state, plain.state);
    assert_eq!(s.ledger, plain.ledger);
}
