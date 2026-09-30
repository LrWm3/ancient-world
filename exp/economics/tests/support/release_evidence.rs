//! Release-only reporting derived from committed state and existing receipts.
use economics_compute_smoke::{financial_reporting::Audit, model::*, simulation::Simulation};
use std::collections::BTreeMap;

pub fn record(case: &str, sim: &Simulation, audit: &Audit) {
    let first = sim.ledger.first().map_or(sim.state.month, |b| b.month);
    let last = sim.ledger.last().map_or(sim.state.month, |b| b.month);
    let mut deficits = BTreeMap::<ResourceId, i64>::new();
    for report in &sim.reports {
        for (&resource, need) in &report.needs {
            *deficits.entry(resource).or_default() += i64::from(need.deficit);
        }
    }
    let mut idle = BTreeMap::<ResourceId, i64>::new();
    for resource in sim
        .world
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Capacity)
    {
        let expired: i64 = sim
            .ledger
            .iter()
            .filter(|b| b.phase == Phase::Open)
            .flat_map(|b| &b.transactions)
            .filter(|t| t.cause == "period expiration/regeneration")
            .flat_map(|t| &t.effects)
            .filter(|e| e.account.1 == resource.id && e.delta < 0)
            .map(|e| -i64::from(e.delta))
            .sum();
        let remaining: i64 = sim
            .state
            .balances
            .iter()
            .filter(|((_, r), _)| *r == resource.id)
            .map(|(_, q)| i64::from(*q))
            .sum();
        idle.insert(resource.id, expired + remaining);
    }
    let mut work = [0_i64; 3];
    for r in sim.ledger.iter().flat_map(|b| &b.receipts) {
        for (total, q) in work.iter_mut().zip([r.requested, r.allocated, r.completed]) {
            *total += i64::from(q);
        }
    }
    let mut volume = BTreeMap::<u32, i64>::new();
    for market in &sim.state.town_market.history {
        for (&id, result) in &market.markets {
            *volume.entry(id).or_default() += i64::from(result.volume);
        }
    }
    let mut issuance = 0;
    for agent in &sim.world.agents {
        let report = audit.book().statements(agent.id, first, last).unwrap();
        assert_eq!(report.assets, report.liabilities + report.equity);
        issuance += report.issuance_change;
    }
    let mut completed = BTreeMap::<DefinitionId, usize>::new();
    for p in sim
        .state
        .processes
        .values()
        .filter(|p| p.status == Status::Completed)
    {
        *completed.entry(p.definition).or_default() += 1;
    }
    println!(
        "V1_EVIDENCE {}",
        serde_json::json!({
            "case": case, "backend": format!("{:?}", sim.backend), "first_month": first, "last_month": last,
            "need_deficits_by_resource": deficits, "completed_processes_by_definition": completed,
            "requested_allocated_completed_receipt_units": work, "unused_capacity_by_resource": idle,
            "town_volume_by_market": volume,
            "land_paid": sim.state.obligations.values().map(|o| i64::from(o.paid)).sum::<i64>(),
            "land_unpaid": sim.state.obligations.values().map(|o| i64::from(o.outstanding())).sum::<i64>(),
            "issuance_reporting_ticks": issuance,
            "separate_statements": sim.world.agents.len(), "reconciled": true,
        })
    );
}
