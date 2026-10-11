use economics_compute_smoke::{
    acceptance::{Run, WORKSHOP, continuation as c, expectations as e},
    compute::Backend,
    telemetry::{Config, Observer},
};
#[test]
fn organization_observer_exports_frozen_evidence_and_private_outcomes_without_live_events() {
    let (w, s) = e::scenario(c::Case::Baseline).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(12).unwrap();
    let mut plain = run.sim.clone();
    let mut audit = run.audit.clone();
    let mut observer = Observer::new(
        vec![],
        "expectation",
        Config {
            settlement: true,
            agents: [WORKSHOP].into(),
            first_month: 13,
            last_month: Some(13),
            ..Default::default()
        },
    )
    .unwrap();
    observer.step_audited(&mut run.sim, &mut run.audit).unwrap();
    audit.step(&mut plain).unwrap();
    assert_eq!(run.sim.world, plain.world);
    assert_eq!(run.sim.state, plain.state);
    assert_eq!(run.sim.ledger, plain.ledger);
    assert_eq!(run.audit, audit);
    let text = String::from_utf8(observer.finish().unwrap()).unwrap();
    let rows: Vec<serde_json::Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let decisions: Vec<_> = rows
        .iter()
        .filter(|r| r["kind"] == "organization_decision")
        .collect();
    assert_eq!(decisions.len(), 1);
    let decision = run.sim.world.agency[&WORKSHOP].history.last().unwrap();
    assert_eq!(
        decisions[0]["decision"],
        serde_json::to_value(decision).unwrap()
    );
    assert!(
        !decision
            .peer_expectations
            .as_ref()
            .unwrap()
            .evidence
            .is_empty()
    );
    assert!(
        decision
            .alternatives
            .iter()
            .filter_map(|a| a.projection.as_ref())
            .any(|p| !p.peer_attempts.is_empty() && !p.peer_ballots.is_empty())
    );
    assert!(
        rows.iter()
            .filter(|r| r["kind"] == "transaction")
            .all(|r| r["month"] == 13)
    );
    assert!(run.sim.ledger.iter().all(|b| b.month <= 13));
}
