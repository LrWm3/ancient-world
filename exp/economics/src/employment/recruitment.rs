//! Dated voluntary hiring at the existing Acquire boundary. Conditional forecast
//! labor is private, physically settled, and never evidence of live consent.
use super::{Boundary, Terms};
use crate::{
    agency::{
        self,
        objectives::{self, Objective},
    },
    compute::Backend,
    model::*,
    simulation::Simulation,
};
use std::collections::{BTreeMap, BTreeSet};

const MAX_HORIZON: u32 = 12;
const MAX_HOURS: i32 = 8;
const MAX_OFFERS: usize = 8;
const MAX_WORKER_RESERVES: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mandate {
    pub offers: BTreeSet<u32>,
    pub payroll: Amount,
    pub horizon: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worker {
    pub horizon: u32,
    /// Ordered terminal-stock preferences after essential needs.
    pub reserves: Vec<Amount>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Mode {
    #[default]
    Live,
    Conditional,
    Off,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub workers: BTreeMap<AgentId, Worker>,
    mode: Mode,
}
impl Config {
    pub(crate) fn freeze(&mut self) {
        if self.mode == Mode::Live {
            self.mode = Mode::Conditional;
        }
    }
    fn stop(&mut self) {
        self.mode = Mode::Off;
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection {
    pub employer: Vec<i128>,
    pub employer_duties: Vec<i128>,
    pub worker: Vec<i128>,
    pub worker_closing_stocks: BTreeMap<ResourceId, i32>,
    pub completed: usize,
    pub wages_paid: i32,
    pub used_hours: i64,
    pub conditional_labor: Vec<Hypothesis>,
    pub peer_attempts: Vec<agency::expectations::Attempt>,
    pub peer_ballots: Vec<agency::expectations::elections::Attempt>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alternative {
    pub hours: i32,
    pub projection: Result<Projection, String>,
    pub employer_accepts: Option<bool>,
    pub worker_accepts: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Duties {
    pub deliveries: Vec<super::performance::Delivery>,
    pub loans: Vec<super::performance::Loan>,
    pub wages: Vec<u32>,
}
/// Continuation assumptions are part of the receipt, separate from utility goals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Valuation {
    DatedJob,
    ConditionalFeasibility,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub valuation: Valuation,
    pub month: u32,
    pub agreement: u32,
    pub governor: Option<AgentId>,
    pub worker: AgentId,
    pub ceiling: i32,
    pub requested: i32,
    pub horizon: u32,
    pub affordable: i32,
    pub selected: i32,
    pub hypothetical: bool,
    pub employer_objectives: Vec<Objective>,
    pub worker_objectives: Vec<Objective>,
    pub baseline: Option<Projection>,
    pub peer_expectations: Option<agency::expectations::Snapshot>,
    pub protected_duties: BTreeMap<AgentId, Duties>,
    pub alternatives: Vec<Alternative>,
    pub reason: String,
}

pub fn authorized(w: &World, t: &Terms) -> bool {
    w.agency
        .get(&t.employer)
        .and_then(|c| c.config.hiring.as_ref())
        .is_some_and(|m| m.offers.contains(&t.id))
}
pub(super) fn validate(w: &World) -> Result<(), String> {
    if w.recruitment.mode != Mode::Live && w.agency.values().any(|c| c.autonomous) {
        return Err("conditional hiring requires frozen controllers".into());
    }
    for (&worker, p) in &w.recruitment.workers {
        if !(1..=MAX_HORIZON).contains(&p.horizon)
            || p.reserves.len() > MAX_WORKER_RESERVES
            || !w.participants.iter().any(|a| a.agent == worker)
            || p.reserves.iter().any(|a| {
                a.quantity < 0
                    || !w
                        .resources
                        .iter()
                        .any(|r| r.id == a.resource && r.kind == ResourceKind::Stock)
            })
            || p.reserves
                .iter()
                .map(|a| a.resource)
                .collect::<BTreeSet<_>>()
                .len()
                != p.reserves.len()
        {
            return Err("invalid voluntary worker policy".into());
        }
    }
    for (&employer, c) in &w.agency {
        let Some(m) = &c.config.hiring else { continue };
        if m.offers.is_empty()
            || m.offers.len() > MAX_OFFERS
            || m.payroll.quantity < 0
            || !(1..=MAX_HORIZON).contains(&m.horizon)
            || w.households.iter().any(|h| h.agent == employer)
            || m.offers.iter().any(|id| {
                !w.employment.iter().any(|t| {
                    t.id == *id
                        && t.employer == employer
                        && t.wage_per_unit.resource == m.payroll.resource
                        && t.capacity.quantity <= MAX_HOURS
                        && w.employment_offers.contains(id)
                })
            })
        {
            return Err("invalid organizational hiring mandate".into());
        }
    }
    Ok(())
}

/// Rank then agreement ID is the explicit allocation policy. Each request sees
/// prior reservations; declining a job leaves those hours for later requests.
pub(super) fn quantity(
    w: &World,
    s: &State,
    base: &Batch,
    prior: &Boundary,
    t: &Terms,
    maximum: i32,
    available: &BTreeMap<Account, i32>,
) -> Result<(i32, Decision), String> {
    let mandate = w.agency[&t.employer].config.hiring.as_ref().unwrap();
    let mut d = Decision {
        valuation: if w.recruitment.mode == Mode::Live {
            Valuation::DatedJob
        } else {
            Valuation::ConditionalFeasibility
        },
        month: s.month,
        agreement: t.id,
        governor: agency::governor(w, s, t.employer),
        worker: t.worker,
        ceiling: maximum,
        requested: 0,
        horizon: 0,
        affordable: 0,
        selected: 0,
        hypothetical: w.recruitment.mode != Mode::Live,
        employer_objectives: vec![],
        worker_objectives: vec![],
        baseline: None,
        peer_expectations: None,
        protected_duties: BTreeMap::new(),
        alternatives: vec![],
        reason: "no current authority or worker participation".into(),
    };
    let Some(worker) = w.recruitment.workers.get(&t.worker) else {
        return Ok((0, d));
    };
    if w.recruitment.mode == Mode::Off
        || d.governor.is_none()
        || (w.recruitment.mode == Mode::Live && !w.agency[&t.employer].autonomous)
    {
        return Ok((0, d));
    }
    let spent: i128 = prior
        .receipts
        .iter()
        .filter(|r| {
            w.employment.iter().any(|v| {
                v.id == r.agreement
                    && v.employer == t.employer
                    && v.wage_per_unit.resource == t.wage_per_unit.resource
            })
        })
        .map(|r| i128::from(r.earned))
        .sum();
    let debt = super::claims(s, t.employer)?
        .get(&t.wage_per_unit.resource)
        .copied()
        .unwrap_or(0);
    let liquid = available
        .get(&(t.employer, t.wage_per_unit.resource))
        .copied()
        .unwrap_or(0);
    let money = (i128::from(liquid) - debt - spent)
        .max(0)
        .min((i128::from(mandate.payroll.quantity) - spent).max(0)) as i32;
    d.affordable = maximum.min(money / t.wage_per_unit.quantity);
    if d.affordable == 0 {
        d.reason = "opening payroll budget exhausted".into();
        return Ok((0, d));
    }
    if base.production_plan.is_some() {
        d.reason = "dated production plan already reserved".into();
        return Ok((0, d));
    }
    let (mut world, state) = crate::households::hiring::preview(w, s, base, prior)?;
    // Evaluate this dated job without assuming any later unsigned jobs.
    // Broader opportunity forecasts may use explicitly conditional labor, but
    // that is not a substitute for accepting a useful current commitment.
    world.recruitment.stop();
    d.employer_objectives = objectives::freeze(
        w,
        s,
        t.employer,
        objectives::governing(w, s, t.employer).unwrap(),
    );
    d.worker_objectives = crate::discovery::supply::objectives(w, s, t.worker);
    let protected_objectives = d.worker_objectives.len();
    d.worker_objectives
        .extend(worker.reserves.iter().map(|reserve| Objective {
            scope: objectives::Scope::Organization,
            metric: objectives::Metric::Reserve {
                resource: reserve.resource,
                target: reserve.quantity,
            },
        }));
    let horizon = if d.hypothetical {
        1
    } else {
        mandate.horizon.max(worker.horizon)
    };
    d.horizon = horizon;
    for agent in [t.employer, t.worker] {
        let mut wages: Vec<_> = w
            .employment
            .iter()
            .filter(|v| v.employer == agent)
            .map(|v| v.id)
            .collect();
        wages.sort_unstable();
        d.protected_duties.insert(
            agent,
            Duties {
                deliveries: super::performance::deliveries(&world, &state, agent, horizon),
                loans: super::performance::loans(&world, &state, agent),
                wages,
            },
        );
    }
    // Apply the same dated public observations used by the employer's operating
    // planner. They predict peer activity, never consent to unsigned labor.
    let peers = agency::expectations::Snapshot::observe(
        w,
        s,
        t.employer,
        w.agency[&t.employer].config.peer_expectations,
    )?;
    d.peer_expectations = Some(peers.clone());
    let goals = Goals {
        employer: &d.employer_objectives,
        worker: &d.worker_objectives,
        peers: &peers,
    };
    let project = |hours| project(&world, &state, t, hours, horizon, &goals);
    let baseline = project(0)?;
    d.baseline = Some(baseline.clone());
    d.reason = "no mutually beneficial completed work".into();
    for hours in (1..=d.affordable).rev() {
        let result = project(hours);
        let feasible = result.as_ref().is_ok_and(|p| {
            p.completed > baseline.completed
                && p.used_hours - baseline.used_hours >= i64::from(hours)
                && p.wages_paid == hours * t.wage_per_unit.quantity
        });
        let employer_accepts = result.as_ref().is_ok_and(|p| {
            feasible
                && p.employer < baseline.employer
                && p.employer_duties
                    .iter()
                    .zip(&baseline.employer_duties)
                    .all(|(a, b)| a <= b)
        });
        let worker_accepts = result.as_ref().is_ok_and(|p| {
            p.worker <= baseline.worker
                && p.worker
                    .iter()
                    .zip(&baseline.worker)
                    .enumerate()
                    .all(|(i, (a, b))| {
                        (protected_objectives..d.worker_objectives.len()).contains(&i) || a <= b
                    })
        });
        if employer_accepts || (d.hypothetical && feasible) {
            d.requested = d.requested.max(hours);
        }
        let accepted = feasible && (d.hypothetical || (employer_accepts && worker_accepts));
        d.alternatives.push(Alternative {
            hours,
            projection: result,
            employer_accepts: (!d.hypothetical).then_some(employer_accepts),
            worker_accepts: (!d.hypothetical).then_some(worker_accepts),
        });
        if accepted {
            d.selected = hours;
            d.reason = if d.hypothetical {
                "conditional useful labor; consent not predicted"
            } else {
                "both parties accept dated work"
            }
            .into();
            break;
        }
    }
    if d.selected == 0 {
        d.reason = if d.requested > 0 {
            "worker declined dated employer demand"
        } else {
            "employer declined: no worthwhile feasible work"
        }
        .into();
    }
    Ok((d.selected, d))
}

struct Goals<'a> {
    employer: &'a [Objective],
    worker: &'a [Objective],
    peers: &'a agency::expectations::Snapshot,
}
fn project(
    w: &World,
    s: &State,
    t: &Terms,
    hours: i32,
    horizon: u32,
    goals: &Goals<'_>,
) -> Result<Projection, String> {
    let (world, mut state) = crate::forecast::ForecastContext::new(w, s).into_parts();
    if hours > 0 {
        let mut execution = crate::finance::Execution::opening(&world, &state);
        let effects = execution.exchange(
            &world,
            &[crate::finance::Transfer {
                from: t.worker,
                to: t.employer,
                amount: Amount::new(t.capacity.resource, hours),
            }],
        )?;
        crate::households::apply(&world, &mut state, &effects, Backend::Reference)?;
        state
            .employment
            .earned
            .insert((t.id, s.month), super::earning(t, s.month, hours)?);
    }
    let deliveries = super::performance::deliveries(&world, &state, t.worker, horizon);
    let loans = super::performance::loans(&world, &state, t.worker);
    let mut delivery_losses = vec![None; deliveries.len()];
    let employer_deliveries = super::performance::deliveries(&world, &state, t.employer, horizon);
    let employer_loans = super::performance::loans(&world, &state, t.employer);
    let mut employer_delivery_losses = vec![None; employer_deliveries.len()];
    let mut sim = Simulation::new(world, state, Backend::Reference)?;
    let mut path = agency::path::Path::new(s.month);
    let end = s
        .month
        .checked_add(horizon)
        .ok_or("hiring horizon overflow")?;
    let mut starts = vec![];
    let mut peer_ballots = vec![];
    while sim.state.month < end {
        let month = sim.state.month;
        if sim.state.phase == Phase::Open {
            peer_ballots.extend(goals.peers.apply_ballots(&mut sim.world, &sim.state)?);
            starts.extend(goals.peers.apply_hypotheses(&mut sim.world, &sim.state)?);
        }
        sim.step()?;
        if sim.state.phase == Phase::Open {
            path.record(&sim.world, &sim.state)?;
            super::performance::observe(&deliveries, &sim.state, month, &mut delivery_losses);
            super::performance::observe(
                &employer_deliveries,
                &sim.state,
                month,
                &mut employer_delivery_losses,
            );
        }
    }
    path.record_completions(&sim.ledger)?;
    let mut losses = objectives::measure_projection(
        &sim.world,
        &sim.state,
        &sim.reports,
        t.worker,
        goals.worker,
        &path,
    )?;
    losses.extend(delivery_losses.into_iter().map(|v| v.unwrap_or(0)));
    losses.extend(super::performance::arrears(&loans, &sim.ledger));
    losses.extend(wage_losses(w, &sim.ledger, t.worker));
    let wages_paid = sim
        .ledger
        .iter()
        .filter(|b| b.month == s.month && b.phase == Phase::Close)
        .filter_map(|b| b.employment.as_ref())
        .flat_map(|b| &b.receipts)
        .filter(|r| r.agreement == t.id && r.earned_month == s.month)
        .map(|r| r.paid)
        .sum();
    let used_hours = sim
        .ledger
        .iter()
        .filter(|b| b.month == s.month && b.phase == Phase::Productive)
        .flat_map(|b| b.all_transactions())
        .flat_map(|tx| &tx.effects)
        .filter(|e| e.account == (t.employer, t.capacity.resource) && e.delta < 0)
        .map(|e| -i64::from(e.delta))
        .sum();
    let mut employer_duties: Vec<_> = employer_delivery_losses
        .into_iter()
        .map(|v| v.unwrap_or(0))
        .collect();
    employer_duties.extend(super::performance::arrears(&employer_loans, &sim.ledger));
    employer_duties.extend(wage_losses(w, &sim.ledger, t.employer));
    Ok(Projection {
        employer_duties,
        employer: objectives::measure_projection(
            &sim.world,
            &sim.state,
            &sim.reports,
            t.employer,
            goals.employer,
            &path,
        )?,
        worker: losses,
        worker_closing_stocks: sim
            .world
            .resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Stock)
            .map(|r| (r.id, sim.state.balance(t.worker, r.id)))
            .collect(),
        completed: agency::completions::observe(&sim.ledger, t.employer, s.month, s.month)
            .into_iter()
            .filter(|c| {
                sim.world.definition(c.definition).execution == Execution::Productive
                    && sim.world.definition(c.definition).duration() == 1
            })
            .count(),
        conditional_labor: hypotheses(&sim.world, &sim.ledger),
        peer_attempts: starts
            .iter()
            .map(|a| agency::expectations::Attempt {
                agent: a.agent,
                definition: a.definition,
                month: a.month,
                completed: sim.state.processes.values().any(|p| {
                    p.operator == a.agent
                        && p.definition == a.definition
                        && p.start == a.month
                        && p.status == Status::Completed
                }),
            })
            .collect(),
        peer_ballots,
        wages_paid,
        used_hours,
    })
}

fn wage_losses(w: &World, ledger: &[Batch], employer: AgentId) -> Vec<i128> {
    let mut terms: Vec<_> = w
        .employment
        .iter()
        .filter(|t| t.employer == employer)
        .collect();
    terms.sort_by_key(|t| t.id);
    terms
        .into_iter()
        .map(|t| {
            ledger
                .iter()
                .filter_map(|b| b.employment.as_ref())
                .flat_map(|b| &b.receipts)
                .filter(|r| {
                    r.agreement == t.id
                        && matches!(
                            r.reason,
                            super::Reason::Collection | super::Reason::CollectionStayed
                        )
                })
                .map(|r| i128::from((r.requested - r.paid).max(0)))
                .max()
                .unwrap_or(0)
        })
        .collect()
}

/// Explicitly conditional deliveries observed in a private branch, not signatures.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Hypothesis {
    pub month: u32,
    pub agreement: u32,
    pub employer: AgentId,
    pub worker: AgentId,
    pub hours: i32,
}
pub(crate) fn hypotheses(w: &World, ledger: &[Batch]) -> Vec<Hypothesis> {
    ledger
        .iter()
        .filter_map(|b| b.employment.as_ref())
        .flat_map(|b| &b.recruitment)
        .filter(|d| d.hypothetical && d.selected > 0)
        .map(|d| Hypothesis {
            month: d.month,
            agreement: d.agreement,
            worker: d.worker,
            hours: d.selected,
            employer: w
                .employment
                .iter()
                .find(|t| t.id == d.agreement)
                .unwrap()
                .employer,
        })
        .collect()
}
