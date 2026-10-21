//! Pure comparisons of explicitly proposed bilateral plans. No intent escapes
//! these branches; counterparties retain their separate governing objectives.
use super::*;
use crate::{
    agency::{self, Command},
    compute::Backend,
    simulation::Simulation,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comparison {
    pub baseline_labor: Vec<crate::employment::recruitment::Hypothesis>,
    pub proposed_labor: Vec<crate::employment::recruitment::Hypothesis>,
    pub governor: AgentId,
    pub horizon: u32,
    pub baseline: Vec<i128>,
    pub proposed: Vec<i128>,
    pub baseline_path: agency::path::Path,
    pub proposed_path: agency::path::Path,
    /// Each party sees only its own operator completions in each private branch.
    pub baseline_completions: Vec<agency::completions::Completion>,
    pub proposed_completions: Vec<agency::completions::Completion>,
    pub admitted: bool,
    pub delivered_on_time: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub proposal: Proposal,
    pub comparisons: BTreeMap<AgentId, Comparison>,
}
impl Assessment {
    pub fn mutually_beneficial(&self) -> bool {
        self.comparisons.len() == 2
            && self.comparisons.values().all(|c| {
                c.admitted
                    && c.delivered_on_time == self.proposal.terms.goods.quantity
                    && c.proposed <= c.baseline
            })
            && self.comparisons.values().any(|c| c.proposed < c.baseline)
    }
}

/// Uses full-information observations, existing frozen peer expectations and each
/// governor's own loss vector. No future autonomous controller is run recursively.
pub fn assess(w: &World, s: &State, proposal: &Proposal) -> Result<Assessment, String> {
    if s.phase != Phase::Open || !candidates(w, s)?.contains(proposal) {
        return Err("proposal is outside current signed candidates".into());
    }
    let mut comparisons = BTreeMap::new();
    for agent in [proposal.terms.buyer, proposal.terms.seller] {
        let controller = &w.agency[&agent];
        let governor = agency::governor(w, s, agent).ok_or("forward party has no governor")?;
        let horizon = controller
            .config
            .horizon
            .max(proposal.terms.due - s.month + 1);
        let objectives = agency::objectives::governing(w, s, agent).unwrap();
        let objectives = agency::objectives::freeze(w, s, agent, objectives);
        let peers = agency::expectations::Snapshot::observe(
            w,
            s,
            agent,
            controller.config.peer_expectations,
        )?;
        let observe_completions = objectives.iter().any(|o| {
            matches!(
                o.metric,
                agency::objectives::Metric::ProcessShortfall { .. }
            )
        });
        let (base, _, _, baseline_path) =
            project(w, s, horizon, &peers, None, observe_completions)?;
        let (proposed, admitted, delivered_on_time, proposed_path) =
            project(w, s, horizon, &peers, Some(proposal), observe_completions)?;
        comparisons.insert(
            agent,
            Comparison {
                baseline_labor: crate::employment::recruitment::hypotheses(
                    &base.world,
                    &base.ledger,
                ),
                proposed_labor: crate::employment::recruitment::hypotheses(
                    &proposed.world,
                    &proposed.ledger,
                ),
                governor,
                horizon,
                baseline: agency::objectives::measure_projection(
                    &base.world,
                    &base.state,
                    &base.reports,
                    agent,
                    &objectives,
                    &baseline_path,
                )?,
                proposed: agency::objectives::measure_projection(
                    &proposed.world,
                    &proposed.state,
                    &proposed.reports,
                    agent,
                    &objectives,
                    &proposed_path,
                )?,
                baseline_completions: agency::completions::observe(
                    &base.ledger,
                    agent,
                    s.month,
                    base.state.month - 1,
                ),
                proposed_completions: agency::completions::observe(
                    &proposed.ledger,
                    agent,
                    s.month,
                    proposed.state.month - 1,
                ),
                baseline_path,
                proposed_path,
                admitted,
                delivered_on_time,
            },
        );
    }
    Ok(Assessment {
        proposal: proposal.clone(),
        comparisons,
    })
}

fn project(
    w: &World,
    s: &State,
    horizon: u32,
    peers: &agency::expectations::Snapshot,
    proposal: Option<&Proposal>,
    observe_completions: bool,
) -> Result<(Simulation, bool, i32, agency::path::Path), String> {
    let (mut world, state) = crate::forecast::ForecastContext::new(w, s).into_parts();
    if let Some(p) = proposal {
        super::execution::record(&mut world, s, p)?;
    }
    let mut sim = Simulation::new(world, state, Backend::Reference)?;
    let end = s
        .month
        .checked_add(horizon)
        .ok_or("forward projection date overflow")?;
    let mut admitted = false;
    let mut on_time = 0;
    let mut path = agency::path::Path::new(s.month);
    while sim.state.month < end {
        let open = sim.state.phase == Phase::Open;
        if open {
            peers.apply_ballots(&mut sim.world, &sim.state)?;
            peers.apply_hypotheses(&mut sim.world, &sim.state)?;
        }
        sim.step()?;
        if sim.state.phase == Phase::Open {
            path.record(&sim.world, &sim.state)?;
        }
        if let Some(p) = proposal
            && let Some(c) = sim.state.exchange.forwards.get(&p.terms.id)
        {
            admitted = true;
            if sim.state.month <= p.terms.due {
                on_time = c.delivered;
            }
        }
    }
    if observe_completions {
        path.record_completions(&sim.ledger)?;
    }
    Ok((sim, admitted, on_time, path))
}

pub(super) fn activate(w: &mut World, p: &Proposal) -> Result<(), String> {
    for (agent, id) in [
        (p.terms.buyer, p.buyer_program),
        (p.terms.seller, p.seller_program),
    ] {
        let catalog = agency::discovery::programs(w, agent)?;
        let program = catalog
            .get(&id)
            .ok_or("missing proposed operating program")?;
        agency::validate_program(w, agent, program)?;
        for command in &program.commands {
            match command {
                Command::WorkTargets(orders) => {
                    w.activities.orders.retain(|o| o.agent != agent);
                    w.activities.orders.extend(orders.clone());
                }
                Command::PeerStockQuotes(quotes) => agency::markets::apply(w, agent, quotes)?,
                _ => return Err("unsupported forward operating program".into()),
            }
        }
    }
    Ok(())
}
