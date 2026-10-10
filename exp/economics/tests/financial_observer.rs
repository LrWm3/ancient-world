use economics_compute_smoke::{
    compute::Backend,
    discovery::scenario,
    minting::{ISSUER, SUPPLIER},
    simulation::Simulation,
    telemetry::{Config, Observer, PlanningDetail},
};
use serde_json::Value;

fn rows(bytes: Vec<u8>) -> Vec<Value> {
    String::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}

#[test]
fn financial_observation_is_read_only_new_only_and_party_filterable() {
    let (w, s) = scenario::scenario().unwrap();
    let mut plain = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut observed = plain.clone();
    let config = Config {
        planning: PlanningDetail::Alternatives,
        metrics: false,
        logs: false,
        agents: [ISSUER].into(),
        ..Config::default()
    };
    let mut observer = Observer::new(vec![], "finance", config.clone()).unwrap();
    observer.run_months(&mut observed, 3).unwrap();
    plain.run_months(3).unwrap();
    assert_eq!(observed.world, plain.world);
    assert_eq!(observed.state, plain.state);
    assert_eq!(observed.ledger, plain.ledger);
    let output = rows(observer.finish().unwrap());
    let assessments: Vec<_> = output
        .iter()
        .filter(|r| r["kind"] == "financial_assessment")
        .collect();
    assert_eq!(assessments.len(), plain.world.discovery.as_ref().unwrap().financial.iter()
        .filter(|a| a.requester == ISSUER || a.attempts.iter().any(|p| p.counterparty == ISSUER)).count());
    assert!(assessments.iter().any(|a| {
        a["attempts"].as_array().unwrap().iter().any(|p| {
            p["outcome"]["kind"] == "published"
                && p["comparisons"].is_array()
                && p["performance"]["outstanding"] == 0
        })
    }));
    for row in &assessments {
        let duration = row["duration"].as_u64().unwrap();
        assert!(duration > 0 && row["horizon"].as_u64().unwrap() >= duration + 2);
    }
    // Attaching to a continuation exports no previously observed assessment.
    let financial_from = observed.world.discovery.as_ref().unwrap().financial.len();
    let mut resumed = Observer::new(vec![], "resume", config).unwrap();
    resumed.run_months(&mut observed, 1).unwrap();
    let continuation = rows(resumed.finish().unwrap());
    let emitted: Vec<_> = continuation
        .iter()
        .filter(|r| r["kind"] == "financial_assessment")
        .collect();
    let expected = observed.world.discovery.as_ref().unwrap().financial[financial_from..]
        .iter()
        .filter(|a| a.requester == ISSUER || a.attempts.iter().any(|p| p.counterparty == ISSUER))
        .count();
    assert_eq!(emitted.len(), expected);
    assert!(emitted.iter().all(|r| r["month"].as_u64().unwrap() >= 4));
    // A counterparty filter sees its assessments even though the issuer requested them.
    let (w, s) = scenario::scenario().unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut observer = Observer::new(
        vec![],
        "lender",
        Config {
            planning: PlanningDetail::Selected,
            agents: [SUPPLIER].into(),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 3).unwrap();
    let output = rows(observer.finish().unwrap());
    let assessments: Vec<_> = output
        .iter()
        .filter(|r| r["kind"] == "financial_assessment")
        .collect();
    assert!(!assessments.is_empty());
    assert!(assessments.iter().all(|a| {
        a["requester"] == SUPPLIER
            || a["attempts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["counterparty"] == SUPPLIER)
    }));
    assert!(assessments.iter().all(|a| {
        a["attempts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p.get("comparisons").is_none())
    }));
}

#[test]
fn disabled_or_failed_open_does_not_export_financial_assessments() {
    for fail in [false, true] {
        let (w, s) = scenario::scenario().unwrap();
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        let mut observer = Observer::new(
            vec![],
            "negative",
            Config {
                planning: if fail {
                    PlanningDetail::Alternatives
                } else {
                    PlanningDetail::Off
                },
                ..Config::default()
            },
        )
        .unwrap();
        if fail {
            sim.effect_limit = 0;
            assert!(observer.step(&mut sim).is_err());
        } else {
            observer.run_months(&mut sim, 3).unwrap();
        }
        assert!(
            !rows(observer.finish().unwrap())
                .iter()
                .any(|r| r["kind"] == "financial_assessment")
        );
    }
}
