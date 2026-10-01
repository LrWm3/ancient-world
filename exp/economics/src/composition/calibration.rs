//! Opt-in score ablation and comparable forecast/outcome observations.
use crate::{
    model::*,
    planning::{SCORE_SCALE, Score},
    simulation::Simulation,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scoring {
    PrivateBuffers,
    /// Same household consumption projection, with collective stock hidden.
    HouseholdPrivateBuffers,
    /// Use the existing household consumption projection, including common stock,
    /// claims and member allocation. No new permission to spend pooled property.
    HouseholdBuffers,
}

pub fn score(sim: &Simulation, scoring: Scoring) -> Result<Score, String> {
    let mut score = crate::planning::score(sim);
    score.broken_commitments = sim
        .state
        .processes
        .values()
        .filter(|p| p.status == Status::Aborted)
        .count() as u64;
    if scoring == Scoring::PrivateBuffers || sim.world.households.is_empty() {
        return Ok(score);
    }
    // Experimental search already requires one isolated household. Keep the
    // ablation narrow rather than implicitly consolidate unrelated organizations.
    if sim.world.households.len() != 1 {
        return Err("household buffer ablation requires one household".into());
    }
    let h = &sim.world.households[0];
    let members = crate::households::membership::current(h);
    if sim
        .world
        .participants
        .iter()
        .any(|p| !members.contains(&p.agent))
    {
        return Err("household buffer ablation requires isolated members".into());
    }
    let mut stocks = if scoring == Scoring::HouseholdPrivateBuffers {
        BTreeMap::new()
    } else {
        crate::substitution::stocks(&sim.state, h.agent)
    };
    // The forecast ends at next Open before reset. Last month's fulfillment
    // must not count as food or warmth stored for this coverage window.
    let mut opening = sim.state.clone();
    for ((_, r), q) in &mut opening.balances {
        if sim
            .world
            .resources
            .iter()
            .any(|a| a.id == *r && a.kind == ResourceKind::Fulfillment)
        {
            *q = 0;
        }
    }
    let deficits = crate::households::market::consume(
        &sim.world,
        &opening,
        h,
        sim.world.horizon,
        &mut stocks,
        false,
    )?;
    let mut targets = BTreeMap::<ResourceId, (u64, u64)>::new();
    for p in sim
        .world
        .participants
        .iter()
        .filter(|p| !sim.state.terminal.contains_key(&p.agent))
    {
        for n in p.needs.iter().filter(|n| n.quantity > 0) {
            let (quantity, people) = targets.entry(n.resource).or_default();
            *quantity += n.quantity as u64 * u64::from(sim.world.horizon);
            *people += 1;
        }
    }
    // Equal-demand members retain the original scale. Unequal demands are
    // weighted by quantity within each provision, not by individual headcount.
    score.buffer_gap = targets
        .iter()
        .map(|(r, (target, people))| {
            deficits.get(r).copied().unwrap_or(0).max(0) as u64 * SCORE_SCALE * people / target
        })
        .sum();
    Ok(score)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub deficits: BTreeMap<ResourceId, i64>,
    /// Completion dates, rather than only an ending process count.
    pub completions: Vec<(u32, AgentId, DefinitionId)>,
    pub pooled: BTreeMap<Account, i32>,
    pub arrears: BTreeMap<ResourceId, i64>,
    pub score: Score,
}
pub fn observe(sim: &Simulation, scoring: Scoring) -> Result<Outcome, String> {
    let mut deficits = BTreeMap::new();
    for r in &sim.reports {
        for (resource, n) in &r.needs {
            *deficits.entry(*resource).or_default() += i64::from(n.deficit);
        }
    }
    let completions = sim
        .ledger
        .iter()
        .filter(|b| b.phase == Phase::Productive)
        .flat_map(|b| {
            b.transactions.iter().filter_map(move |t| {
                let p = &t.process.as_ref()?.after;
                (p.status == Status::Completed).then_some((b.month, p.operator, p.definition))
            })
        })
        .collect();
    let pooled = sim
        .state
        .balances
        .iter()
        .filter(|((a, _), _)| sim.world.households.iter().any(|h| h.agent == *a))
        .map(|(a, q)| (*a, *q))
        .collect();
    let mut arrears = BTreeMap::new();
    for o in sim.state.obligations.values() {
        *arrears
            .entry(
                sim.world
                    .agreements
                    .iter()
                    .chain(sim.state.accepted_agreements.values())
                    .find(|a| a.id == o.agreement)
                    .ok_or("unknown observation agreement")?
                    .payment
                    .resource,
            )
            .or_default() += i64::from(o.outstanding());
    }
    Ok(Outcome {
        deficits,
        completions,
        pooled,
        arrears,
        score: score(sim, scoring)?,
    })
}
