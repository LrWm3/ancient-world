use economics_compute_smoke::{
    acceptance::{Run, WORKSHOP, continuation as c, sales as e},
    compute::Backend,
    telemetry::{Config, Observer},
};
#[test]
fn sale_observer_connects_signed_opportunities_to_private_settled_sales() {
    let (w, s) = e::with_startup(c::Case::Baseline, e::Startup::SupplyChainInventory).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(4).unwrap();
    let mut plain = run.sim.clone();
    let mut audit = run.audit.clone();
    let mut observer = Observer::new(
        vec![],
        "expectation",
        Config {
            settlement: true,
            agents: [WORKSHOP].into(),
            first_month: 5,
            last_month: Some(5),
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
    use economics_compute_smoke::minting::{COIN, ISSUER, METAL};
    assert!(
        decision
            .alternatives
            .iter()
            .filter_map(|a| a.projection.as_ref())
            .flat_map(|p| &p.sales)
            .any(|s| (
                s.buyer, s.market, s.resource, s.payment, s.quantity, s.value
            ) == (ISSUER, METAL, METAL, COIN, 2, 4))
    );
    assert_eq!(decision.sale_opportunities.len(), 1);
    assert_eq!(decision.sale_opportunities[0].target, 2);
    assert!(
        decision
            .alternatives
            .iter()
            .filter_map(|a| a.projection.as_ref())
            .any(|p| !p.sales.is_empty())
    );
    assert!(
        decision
            .alternatives
            .iter()
            .filter_map(|a| a.projection.as_ref())
            .flat_map(|p| &p.sales)
            .all(|s| s.month >= 5 && s.month < 17 && s.quantity > 0 && s.value > 0)
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
            .all(|r| r["month"] == 5)
    );
    assert!(run.sim.ledger.iter().all(|b| b.month <= 5));
}
