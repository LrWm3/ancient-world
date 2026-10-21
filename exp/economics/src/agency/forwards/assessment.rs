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
pub struct OutsideOption {
    pub month: u32,
    pub losses: Vec<i128>,
    pub conditional_labor: Vec<crate::employment::recruitment::Hypothesis>,
    pub completions: Vec<agency::completions::Completion>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comparison {
    pub baseline_labor: Vec<crate::employment::recruitment::Hypothesis>,
    pub proposed_labor: Vec<crate::employment::recruitment::Hypothesis>,
    pub governor: AgentId,
    pub horizon: u32,
    pub baseline: Vec<i128>,
    /// The party's own signed operating program, with spot funding and unchanged peers.
    /// None also covers proposals rejected before outside options were evaluated.
    /// Acceptance is conditional on the same private labor hypotheses as other branches.
    pub self_funded: Option<OutsideOption>,
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
                    && c.self_funded
                        .as_ref()
                        .is_none_or(|o| c.proposed <= o.losses)
            })
            && self.comparisons.values().any(|c| {
                c.proposed < c.baseline
                    && c.self_funded.as_ref().is_none_or(|o| c.proposed < o.losses)
            })
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
            project(w, s, horizon, &peers, Plan::Retain, observe_completions)?;
        let (proposed, admitted, delivered_on_time, proposed_path) = project(
            w,
            s,
            horizon,
            &peers,
            Plan::Forward(proposal),
            observe_completions,
        )?;
        let baseline = agency::objectives::measure_projection(
            &base.world,
            &base.state,
            &base.reports,
            agent,
            &objectives,
            &baseline_path,
        )?;
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
                baseline,
                self_funded: None,
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
    let mut assessment = Assessment {
        proposal: proposal.clone(),
        comparisons,
    };
    // An already rejected proposal needs no additional outside-option rollouts.
    if assessment.mutually_beneficial() {
        for (&agent, comparison) in &mut assessment.comparisons {
            let controller = &w.agency[&agent];
            let horizon = comparison.horizon;
            let baseline = &comparison.baseline;
            let objectives = agency::objectives::freeze(
                w,
                s,
                agent,
                agency::objectives::governing(w, s, agent).unwrap(),
            );
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
            let program = if agent == proposal.terms.buyer {
                proposal.buyer_program
            } else {
                proposal.seller_program
            };
            let mut self_funded: Option<OutsideOption> = None;
            // Ordinary next-Open work and the forward's delayed activation are both
            // real alternatives. No counterparty program or future consent is imposed.
            for month in [proposal.terms.month, proposal.terms.month + 1] {
                let (spot, _, _, path) = project(
                    w,
                    s,
                    horizon,
                    &peers,
                    Plan::SelfFunded {
                        agent,
                        program,
                        month,
                    },
                    observe_completions,
                )?;
                let losses = agency::objectives::measure_projection(
                    &spot.world,
                    &spot.state,
                    &spot.reports,
                    agent,
                    &objectives,
                    &path,
                )?;
                if &losses <= baseline && self_funded.as_ref().is_none_or(|o| losses < o.losses) {
                    self_funded = Some(OutsideOption {
                        month,
                        losses,
                        conditional_labor: crate::employment::recruitment::hypotheses(
                            &spot.world,
                            &spot.ledger,
                        ),
                        completions: agency::completions::observe(
                            &spot.ledger,
                            agent,
                            s.month,
                            spot.state.month - 1,
                        ),
                    });
                }
            }
            comparison.self_funded = self_funded;
        }
    }
    Ok(assessment)
}

enum Plan<'a> {
    Retain,
    Forward(&'a Proposal),
    SelfFunded {
        agent: AgentId,
        program: u32,
        month: u32,
    },
}

fn project(
    w: &World,
    s: &State,
    horizon: u32,
    peers: &agency::expectations::Snapshot,
    plan: Plan<'_>,
    observe_completions: bool,
) -> Result<(Simulation, bool, i32, agency::path::Path), String> {
    let (mut world, state) = crate::forecast::ForecastContext::new(w, s).into_parts();
    let proposal = match plan {
        Plan::Forward(p) => {
            super::execution::record(&mut world, s, p)?;
            Some(p)
        }
        Plan::SelfFunded { agent, program, .. } => {
            let catalog = agency::discovery::programs(&world, agent)?;
            agency::issue(&mut world, s, agent, &catalog[&program])?;
            None
        }
        Plan::Retain => None,
    };
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
        if open
            && let Plan::SelfFunded {
                agent,
                program,
                month,
            } = plan
            && sim.state.month == month
        {
            activate_program(&mut sim.world, agent, program)?;
        }
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
        activate_program(w, agent, id)?;
    }
    Ok(())
}

fn activate_program(w: &mut World, agent: AgentId, id: u32) -> Result<(), String> {
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
    Ok(())
}
