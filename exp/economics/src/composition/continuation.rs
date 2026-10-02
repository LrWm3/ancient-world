//! Bounded review policies over existing Acquire/production settlement.
//! Checkpoint the controller alongside Simulation. No future grant is reusable.
use super::{Budget, Metrics, Scope, Strategy};
use crate::{compute::Backend, forecast::ForecastContext, model::*, simulation::Simulation};
use std::collections::BTreeMap;

pub mod persons;
pub mod posted;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    Monthly,
    RetainRepair,
    ScheduledReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Initial,
    Monthly,
    HorizonEnded,
    ObservationChanged,
    AdmissionRejected,
    Retained,
    Continued,
}

/// A forecasted opportunity schedule, not pre-authorized work or money.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub month: u32,
    expected: ForecastContext,
    pub starts: Vec<(AgentId, DefinitionId)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub month: u32,
    pub reason: Reason,
    pub observation_changed: bool,
    pub search: Option<Metrics>,
    /// One additional selected-plan projection, separate from the search budget.
    pub projection_months: u32,
    pub projection_steps: usize,
    pub replay_steps: usize,
    pub cheap_preview_steps: usize,
    pub starts: Vec<(AgentId, DefinitionId)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controller {
    scope: Scope,
    strategy: Strategy,
    budget: Budget,
    policy: Policy,
    review_at: Option<u32>,
    land_limit: Option<usize>,
    admission_rejected: bool,
    order_forecast: super::market::OrderForecast,
    counterparties: Option<super::market::Counterparties>,
    alternatives: Vec<Vec<crate::offers::Request>>,
    pub frames: BTreeMap<u32, Frame>,
    pub history: Vec<Receipt>,
}

fn starts(b: &Batch) -> Vec<(AgentId, DefinitionId)> {
    let mut result: Vec<_> = b
        .transactions
        .iter()
        .filter_map(|t| {
            let p = t.process.as_ref()?;
            p.before
                .is_none()
                .then_some((p.after.operator, p.after.definition))
        })
        .collect();
    result.sort();
    result
}

/// Re-evaluate the cheap policy and its household allocation at the actual
/// boundary. Reserve that fresh work in Acquire; Productive consumes it once.
fn cheap(sim: &Simulation) -> Result<Batch, String> {
    let mut preview = sim.clone();
    preview.backend = Backend::Reference;
    preview.ledger.clear();
    preview.reports.clear();
    preview.step()?;
    let mut acquire = preview.ledger.pop().ok_or("missing cheap acquisition")?;
    if preview.state.phase != Phase::Productive {
        return Err("continuation requires adjacent Acquire and Productive".into());
    }
    preview.step()?;
    acquire.production_plan = Some(Box::new(preview.ledger.pop().ok_or("missing cheap work")?));
    Ok(acquire)
}

impl Controller {
    pub fn new(scope: Scope, strategy: Strategy, budget: Budget, policy: Policy) -> Self {
        Self {
            scope,
            strategy,
            budget,
            policy,
            review_at: None,
            land_limit: None,
            admission_rejected: false,
            order_forecast: super::market::OrderForecast::CurrentBoundaryOnly,
            counterparties: None,
            alternatives: vec![],
            frames: BTreeMap::new(),
            history: vec![],
        }
    }

    pub fn step(&mut self, sim: &mut Simulation) -> Result<(), String> {
        self.step_comparing(sim, |a, b| a == b)
    }

    fn step_comparing(
        &mut self,
        sim: &mut Simulation,
        same: impl Fn(&ForecastContext, &ForecastContext) -> bool,
    ) -> Result<(), String> {
        // This is an explicit experiment configuration, never a silent rewrite
        // of the world's existing default decision policy.
        if sim.world.priority != Priority::ContinuingFirst || sim.world.competition.is_some() {
            return Err(
                "continuation controller requires opt-in ContinuingFirst without competition"
                    .into(),
            );
        }
        if sim.state.phase != Phase::Acquire {
            return sim.step();
        }
        let actors = super::validate_search(sim, &self.scope, self.budget)?;
        if sim.world.town_market.is_some() && self.policy != Policy::Monthly {
            return Err("market continuation currently requires monthly review".into());
        }
        if let Scope::Household { .. } = &self.scope
            && sim
                .world
                .participants
                .iter()
                .any(|p| !actors.contains(&p.agent))
        {
            return Err(
                "household continuation requires every participant's private mandate".into(),
            );
        }
        let month = sim.state.month;
        let changed = self
            .frames
            .get(&month)
            .is_some_and(|f| !same(&f.expected, &ForecastContext::new(&sim.world, &sim.state)));
        let reason = match self.review_at {
            None => Reason::Initial,
            _ if self.admission_rejected => Reason::AdmissionRejected,
            _ if self.policy == Policy::Monthly => Reason::Monthly,
            Some(end) if month >= end => Reason::HorizonEnded,
            _ if self.policy == Policy::RetainRepair
                && (changed || !self.frames.contains_key(&month)) =>
            {
                Reason::ObservationChanged
            }
            _ if self.policy == Policy::RetainRepair => Reason::Retained,
            _ => Reason::Continued,
        };
        let search = !matches!(reason, Reason::Retained | Reason::Continued);
        let mut receipt = Receipt {
            month,
            reason,
            observation_changed: changed,
            search: None,
            projection_months: 0,
            projection_steps: 0,
            replay_steps: 0,
            cheap_preview_steps: 0,
            starts: vec![],
        };
        if search {
            let mut alternatives = vec![];
            let selection = super::choose_limited(
                sim,
                &self.scope,
                self.strategy,
                self.budget,
                super::calibration::Scoring::PrivateBuffers,
                super::SearchOptions {
                    land_limit: self.land_limit,
                    counterparties: self.counterparties.as_ref(),
                    required_orders: None,
                    persistent_orders: sim.world.town_market.is_some()
                        && self.order_forecast == super::market::OrderForecast::StandingPolicy,
                },
                &mut alternatives,
            )?;
            let end = month
                .checked_add(self.budget.months)
                .ok_or("review date overflow")?;
            let mut frames = BTreeMap::new();
            if self.policy != Policy::Monthly {
                // Capture only the selected continuation. Account for this extra
                // forecast and replay rather than hiding them in search timings.
                let (_, _, branch) =
                    super::forecast_package(sim, &selection.requests, self.budget.months)?;
                let mut state = sim.state.clone();
                for batch in &branch.ledger {
                    if state.phase == Phase::Acquire {
                        frames.insert(
                            state.month,
                            Frame {
                                month: state.month,
                                expected: ForecastContext::new(&branch.world, &state),
                                starts: vec![],
                            },
                        );
                    }
                    if batch.phase == Phase::Productive {
                        frames
                            .get_mut(&batch.month)
                            .ok_or("missing planned acquisition")?
                            .starts = starts(batch);
                    }
                    crate::settlement::commit(
                        &branch.world,
                        &mut state,
                        batch,
                        Backend::Reference,
                        sim.effect_limit,
                    )?;
                }
                if state != branch.state {
                    return Err("continuation projection replay differs".into());
                }
                receipt.projection_months = self.budget.months;
                receipt.projection_steps = branch.ledger.len();
                receipt.replay_steps = branch.ledger.len();
            }
            receipt.starts = selection
                .batch
                .production_plan
                .as_deref()
                .map(starts)
                .unwrap_or_default();
            selection.accept(sim)?;
            receipt.search = Some(selection.metrics);
            self.alternatives = alternatives;
            self.frames = frames;
            self.review_at = Some(end);
        } else {
            let batch = cheap(sim)?;
            receipt.starts = batch
                .production_plan
                .as_deref()
                .map(starts)
                .unwrap_or_default();
            // A stable retained plan must still describe today's freshly resolved
            // starts. No saved grants, old batch IDs or old resource balances apply.
            if reason == Reason::Retained && self.frames[&month].starts != receipt.starts {
                return Err("retained opportunity schedule differs from fresh continuation".into());
            }
            crate::settlement::commit(
                &sim.world,
                &mut sim.state,
                &batch,
                sim.backend,
                sim.effect_limit,
            )?;
            sim.ledger.push(batch);
            receipt.cheap_preview_steps = 2;
        }
        self.admission_rejected = false;
        self.history.push(receipt);
        Ok(())
    }
}
