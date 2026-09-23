use economics_compute_smoke::{
    compute::Backend,
    model::Phase,
    scenario::PERSON,
    simulation::Simulation,
    telemetry::{Config, Observer},
    town_market,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::{self, Write},
};

fn simulation(backend: Backend) -> Simulation {
    let (world, state) = town_market::scenario();
    Simulation::new(world, state, backend).unwrap()
}
fn records(bytes: Vec<u8>) -> Vec<Value> {
    String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
#[test]
fn cpu_observation_preserves_results_and_reconciles_committed_metrics() {
    let mut observed = simulation(Backend::CubeCpu);
    let mut plain = observed.clone();
    let mut observer = Observer::new(vec![], "quoted\"run\nname", Config::default()).unwrap();
    observer.run_months(&mut observed, 2).unwrap();
    plain.run_months(2).unwrap();
    assert_eq!(observed.state, plain.state);
    assert_eq!(observed.ledger, plain.ledger);
    assert_eq!(observed.reports, plain.reports);
    let rows = records(observer.finish().unwrap());
    assert!(
        rows.iter()
            .all(|r| r["run"] == "quoted\"run\nname" && r["schema"] == 1)
    );
    assert_eq!(
        rows.iter().filter(|r| r["kind"] == "batch").count(),
        plain.ledger.len()
    );
    assert_eq!(
        rows.iter().filter(|r| r["kind"] == "transaction").count(),
        plain
            .ledger
            .iter()
            .map(|b| b.transactions.len())
            .sum::<usize>()
    );
    for row in rows.iter().filter(|r| r["kind"] == "market") {
        let round = plain
            .state
            .town_market
            .history
            .iter()
            .find(|r| r.month == row["month"].as_u64().unwrap() as u32)
            .unwrap();
        let result = &round.markets[&(row["market"].as_u64().unwrap() as u32)];
        assert_eq!(row["volume"], result.volume);
        assert_eq!(row["posted_price"], serde_json::json!(result.posted_price));
    }
    for row in rows.iter().filter(|r| r["kind"] == "account_flow") {
        let month = row["month"].as_u64().unwrap() as u32;
        let account = (
            row["agent"].as_u64().unwrap() as u32,
            row["resource"].as_u64().unwrap() as u32,
        );
        let effects = plain
            .ledger
            .iter()
            .filter(|b| b.month == month)
            .flat_map(|b| &b.transactions)
            .flat_map(|t| &t.effects)
            .filter(|e| e.account == account);
        let (credits, debits) = effects.fold((0i64, 0i64), |(c, d), e| {
            (c + i64::from(e.delta.max(0)), d - i64::from(e.delta.min(0)))
        });
        assert_eq!(row["credits"], credits);
        assert_eq!(row["debits"], debits);
    }
    for row in rows.iter().filter(|r| r["kind"] == "need") {
        let report = plain
            .reports
            .iter()
            .find(|r| {
                r.month == row["month"].as_u64().unwrap() as u32
                    && r.agent == row["agent"].as_u64().unwrap() as u32
            })
            .unwrap();
        assert_eq!(
            row["deficit"],
            report.deficit(row["resource"].as_u64().unwrap() as u32)
        );
    }
}
#[test]
fn filters_cadence_and_budget_are_explicit() {
    let mut sim = simulation(Backend::Reference);
    let start = sim.state.month;
    let config = Config {
        agents: BTreeSet::from([PERSON]),
        first_month: start,
        last_month: Some(start + 2),
        metric_every: 2,
        log_limit: 1,
        ..Config::default()
    };
    let mut observer = Observer::new(vec![], "filtered", config).unwrap();
    observer.run_months(&mut sim, 4).unwrap();
    let rows = records(observer.finish().unwrap());
    for r in rows.iter().filter(|r| r["agent"].is_number()) {
        assert_eq!(r["agent"], PERSON);
        assert!([start, start + 2].contains(&(r["month"].as_u64().unwrap() as u32)));
    }
    assert_eq!(rows.iter().filter(|r| r["kind"] == "batch").count(), 1);
    assert!(rows.last().unwrap()["omitted_logs"].as_u64().unwrap() > 0);
    assert!(rows.iter().any(|r| r["scope"] == "whole_market"));
}
#[test]
fn checkpoints_and_repeated_calls_do_not_reexport_history() {
    let mut sim = simulation(Backend::Reference);
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Acquire);
    let mut resumed =
        Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
    let mut a = Observer::new(vec![], "segment", Config::default()).unwrap();
    let mut b = Observer::new(vec![], "segment", Config::default()).unwrap();
    a.run_months(&mut sim, 2).unwrap();
    b.run_months(&mut resumed, 1).unwrap();
    b.run_months(&mut resumed, 1).unwrap();
    assert_eq!(sim.state, resumed.state);
    assert_eq!(a.finish().unwrap(), b.finish().unwrap());
}
#[test]
fn failed_step_has_no_fabricated_commit() {
    let mut sim = simulation(Backend::Reference);
    sim.step().unwrap(); // Admission has no resource effects; fail market settlement.
    sim.effect_limit = 0;
    let existing_batches = sim.ledger.len();
    let mut observer = Observer::new(
        vec![],
        "failure",
        Config {
            logs: false,
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    assert!(observer.step(&mut sim).is_err());
    let rows = records(observer.finish().unwrap());
    assert!(rows.iter().any(|r| r["kind"] == "step_error"));
    assert_eq!(
        rows.iter().filter(|r| r["kind"] == "batch").count(),
        sim.ledger.len() - existing_batches
    );
}
struct FailingWriter {
    writes: usize,
}
impl Write for FailingWriter {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        // Exercise failure while constructing the output stream.
        if self.writes >= 2 {
            return Err(io::Error::other("test failure"));
        }
        self.writes += 1;
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn output_failure_is_explicit_and_prevents_accidental_second_step() {
    // Permit start and attachment records; fail the first post-commit write.
    struct AfterHeader(u32);
    impl Write for AfterHeader {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            if self.0 >= 2 {
                return Err(io::Error::other("disk full"));
            }
            if b.contains(&b'\n') {
                self.0 += 1;
            }
            Ok(b.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut sim = simulation(Backend::Reference);
    let mut observer = Observer::new(AfterHeader(0), "failure", Config::default()).unwrap();
    assert!(
        observer
            .step(&mut sim)
            .unwrap_err()
            .contains("not rolled back")
    );
    let state = sim.state.clone();
    assert!(observer.step(&mut sim).is_err());
    assert_eq!(sim.state, state);
    assert!(observer.finish().is_err());
    assert!(Observer::new(FailingWriter { writes: 2 }, "failure", Config::default()).is_err());
}
#[test]
fn modes_and_invalid_configuration() {
    assert!(
        Observer::new(
            vec![],
            "bad",
            Config {
                metric_every: 0,
                ..Config::default()
            }
        )
        .is_err()
    );
    for (metrics, logs) in [(true, false), (false, true), (false, false)] {
        let mut observer = Observer::new(
            vec![],
            "modes",
            Config {
                metrics,
                logs,
                ..Config::default()
            },
        )
        .unwrap();
        observer
            .run_months(&mut simulation(Backend::Reference), 1)
            .unwrap();
        let rows = records(observer.finish().unwrap());
        assert_eq!(rows.iter().any(|r| r["kind"] == "need"), metrics);
        assert_eq!(rows.iter().any(|r| r["kind"] == "transaction"), logs);
    }
}

#[test]
fn private_planning_runs_are_not_exported_and_agent_logs_keep_counterparties() {
    let (world, state) = economics_compute_smoke::production_market::reciprocal_scenario(true);
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    let mut plain = sim.clone();
    let mut observer = Observer::new(vec![], "planning", Config::default()).unwrap();
    observer.run_months(&mut sim, 1).unwrap();
    plain.run_months(1).unwrap();
    assert_eq!(sim.state, plain.state);
    assert_eq!(sim.ledger, plain.ledger);
    let rows = records(observer.finish().unwrap());
    assert_eq!(
        rows.iter().filter(|r| r["kind"] == "batch").count(),
        sim.ledger.len()
    );
    assert!(!rows.iter().any(|r| r["kind"] == "decision"));

    let mut sim = simulation(Backend::Reference);
    let mut observer = Observer::new(
        vec![],
        "agent",
        Config {
            agents: BTreeSet::from([PERSON]),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 1).unwrap();
    let rows = records(observer.finish().unwrap());
    assert!(rows.iter().filter(|r| r["kind"] == "transaction").any(|r| {
        let effects = r["effects"].as_array().unwrap();
        effects.iter().any(|e| e["agent"] == PERSON) && effects.iter().any(|e| e["agent"] != PERSON)
    }));
}

#[test]
fn subsystem_observers_reconcile_forecasts_through_the_full_horizon() {
    use economics_compute_smoke::{
        negotiation::Outcome, production_market, telemetry::PlanningDetail,
    };
    let (world, state) = production_market::reciprocal_scenario(true);
    let months = world.production_market.as_ref().unwrap().horizon;
    let mut sim = Simulation::new(world, state, Backend::CubeCpu).unwrap();
    let mut plain = sim.clone();
    let mut observer = Observer::new(
        vec![],
        "subsystems",
        Config {
            planning: PlanningDetail::Alternatives,
            settlement: true,
            agents: BTreeSet::from([PERSON]),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, months).unwrap();
    plain.run_months(months).unwrap();
    assert_eq!(sim.state, plain.state);
    assert_eq!(sim.ledger, plain.ledger);
    let rows = records(observer.finish().unwrap());
    let plans: Vec<_> = rows.iter().filter(|r| r["kind"] == "plan").collect();
    assert_eq!(plans.len(), months as usize);
    let first = plans[0];
    let original = &sim.state.town_market.history[0]
        .planning
        .as_ref()
        .unwrap()
        .people[0];
    assert_eq!(first["selected"], original.selected);
    assert_eq!(
        first["forecast"]["closing_coins"],
        original.alternatives[original.selected].closing_coins
    );
    assert_eq!(
        rows.iter()
            .filter(|r| r["kind"] == "plan_alternative")
            .count(),
        sim.state
            .town_market
            .history
            .iter()
            .map(|r| r.planning.as_ref().unwrap().people[0].alternatives.len())
            .sum::<usize>()
    );
    let outcomes: Vec<_> = rows
        .iter()
        .filter(|r| r["kind"] == "plan_outcome")
        .collect();
    assert_eq!(outcomes.len(), 1); // Other rolling forecasts are still incomplete.
    let outcome = outcomes[0];
    assert_eq!(outcome["plan_batch"], first["batch"]);
    assert_eq!(outcome["through"], first["through"]);
    for (role, key) in [(true, "purchases"), (false, "sales")] {
        let mut expected = std::collections::BTreeMap::<u32, i64>::new();
        for a in sim
            .state
            .town_market
            .history
            .iter()
            .flat_map(|r| &r.attempts)
        {
            if matches!(a.round.outcome, Outcome::Traded { .. })
                && (if role {
                    a.session.buyer.agent
                } else {
                    a.session.seller.agent
                }) == PERSON
            {
                *expected.entry(a.session.market).or_default() +=
                    i64::from(a.session.goods.quantity);
            }
        }
        assert_eq!(outcome["actual"][key], serde_json::json!(expected));
    }
    assert_eq!(rows.last().unwrap()["pending_plan_outcomes"], months - 1);
    assert!(rows.iter().any(|r| r["kind"] == "work_receipt"));
    assert!(rows.iter().any(|r| r["kind"] == "market_order"));
    for r in rows.iter().filter(|r| r["kind"] == "market_attempt") {
        assert!(r["buyer"] == PERSON || r["seller"] == PERSON);
    }
}

#[test]
fn selected_only_and_settlement_share_filters_and_log_budget() {
    use economics_compute_smoke::{production_market, telemetry::PlanningDetail};
    let (world, state) = production_market::reciprocal_scenario(true);
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    let mut observer = Observer::new(
        vec![],
        "limited",
        Config {
            planning: PlanningDetail::Selected,
            settlement: true,
            metrics: false,
            logs: false,
            log_limit: 1,
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 1).unwrap();
    let rows = records(observer.finish().unwrap());
    assert_eq!(rows.iter().filter(|r| r["kind"] == "plan").count(), 1);
    assert!(!rows.iter().any(|r| r["kind"] == "plan_alternative"));
    assert!(rows.last().unwrap()["omitted_logs"].as_u64().unwrap() > 0);
}

#[test]
fn settlement_exports_recorded_funding_and_storage_failures() {
    use economics_compute_smoke::scenario::{GRAIN, TOKEN};
    for storage in [false, true] {
        let (mut world, mut state) = town_market::scenario();
        state.balances.insert((92, GRAIN), 2);
        if storage {
            world.storage.capacities.insert(PERSON, 0);
        } else {
            state.balances.insert((PERSON, TOKEN), 0);
        }
        let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
        let mut observer = Observer::new(
            vec![],
            "rejection",
            Config {
                settlement: true,
                ..Config::default()
            },
        )
        .unwrap();
        observer.step(&mut sim).unwrap();
        observer.step(&mut sim).unwrap();
        let rows = records(observer.finish().unwrap());
        let attempt = rows
            .iter()
            .find(|r| r["kind"] == "market_attempt" && r["buyer"] == PERSON)
            .unwrap();
        assert_eq!(
            attempt["outcome"],
            if storage {
                "InsufficientStorage"
            } else {
                "InsufficientPayment"
            }
        );
        let generation = rows
            .iter()
            .find(|r| r["kind"] == "order_generation" && r["agent"] == PERSON && r["side"] == "Buy")
            .unwrap();
        assert_eq!(generation["reason"], "Submitted"); // Funding/storage belong to matching, not this gate.
        assert_eq!(attempt["completed"], 0);
        assert!(attempt["price"].is_null());
        assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    }
}
