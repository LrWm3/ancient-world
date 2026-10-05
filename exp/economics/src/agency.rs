//! Shared bounded organization decisions. Authority and economic execution stay
//! in their domain adapters; objectives never grant resources or legal powers.
use crate::{activities::WorkOrder, model::*, simulation::Simulation};
use std::collections::{BTreeMap, BTreeSet};

pub mod discovery;
pub mod integration;
pub mod objectives;
pub mod scenario;
use objectives::{Objective, measure};

const MAX_HORIZON_MONTHS: u32 = 24;
const MAX_PROGRAMS: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    StatePolicy(crate::state_governance::PolicyId),
    HouseholdPolicy(crate::household_governance::Policy),
    /// Replace only this organization's discretionary work targets next Open.
    WorkTargets(Vec<WorkOrder>),
    /// One dated attempt. Ordinary acquisition, rights and resource checks apply.
    StartProcess(DefinitionId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub name: String,
    pub commands: Vec<Command>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub horizon: u32,
    pub review_every: u32,
    pub minimum_tenure: u32,
    /// Optional observed objective index and loss ceiling permitting early review.
    pub emergency: Option<(usize, i128)>,
    /// Lexicographic losses: unlike provisions and currencies are never summed.
    pub objectives: Vec<Objective>,
    /// Static person preferences, used for candidate platforms and voluntary votes.
    /// Absence means no autonomous ballot, not consent to another agent's choice.
    pub preferences: BTreeMap<AgentId, Vec<Objective>>,
    pub programs: BTreeMap<u32, Program>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controller {
    pub config: Config,
    pub autonomous: bool,
    /// Discover commands from constitutional options and technology, rather than a menu.
    pub discover_programs: bool,
    pub history: Vec<Decision>,
    pub applied_through: u32,
    /// Static operating mandate sealed at the first live opening.
    mandate: Option<(Config, bool)>,
}
impl Controller {
    pub fn discovering(mut config: Config) -> Self {
        config.programs.clear();
        Self {
            discover_programs: true,
            ..Self::new(config)
        }
    }
    pub fn new(config: Config) -> Self {
        Self {
            config,
            autonomous: true,
            discover_programs: false,
            history: vec![],
            applied_through: 0,
            mandate: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Alternative {
    /// None retains accepted policies and work; it does not cancel commitments.
    pub program: Option<u32>,
    pub losses: Option<Vec<i128>>,
    pub preferences: BTreeMap<AgentId, Vec<i128>>,
    pub failure: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Decision {
    pub month: u32,
    pub authorized_by: Option<AgentId>,
    pub effective_month: u32,
    pub chosen: Option<u32>,
    /// Accepted command snapshot; later catalog edits cannot rewrite work.
    #[serde(skip)]
    pub accepted: Option<Program>,
    /// Actual candidate catalog at this decision boundary.
    #[serde(skip)]
    pub catalog: BTreeMap<u32, Program>,
    pub alternatives: Vec<Alternative>,
    /// Current stocks/claims/history; need fulfillment is the preceding month.
    pub observed: Vec<i128>,
    pub reason: String,
    pub ballots: Vec<crate::governance::Ballot>,
}

pub fn governor(w: &World, s: &State, agent: AgentId) -> Option<AgentId> {
    if w.state_governance
        .as_ref()
        .is_some_and(|g| g.state == agent)
    {
        crate::state_governance::authority(w, s)?.governor
    } else {
        let h = w.households.iter().find(|h| h.agent == agent)?;
        crate::household_governance::authority(h, s).leader
    }
}

pub fn validate(w: &World) -> Result<(), String> {
    for (&agent, controller) in &w.agency {
        let c = &controller.config;
        if !w.agents.iter().any(|a| a.id == agent)
            || !(1..=MAX_HORIZON_MONTHS).contains(&c.horizon)
            || c.review_every == 0
            || c.minimum_tenure == 0
            || c.programs.len() > MAX_PROGRAMS
            || c.objectives.is_empty()
            || c.emergency
                .is_some_and(|(i, ceiling)| i >= c.objectives.len() || ceiling < 0)
            || controller
                .mandate
                .as_ref()
                .is_some_and(|m| m != &(c.clone(), controller.discover_programs))
            || (controller.discover_programs && !c.programs.is_empty())
            || (w.state_governance.as_ref().is_none_or(|g| g.state != agent)
                && !w.households.iter().any(|h| h.agent == agent))
        {
            return Err("invalid organization decision configuration".into());
        }
        objectives::validate(w, &c.objectives)?;
        for (&person, objectives) in &c.preferences {
            if !w.transaction_policy.as_ref().is_some_and(|p| {
                p.agent_types.get(&person) == Some(&crate::opportunities::PERSON_TYPE)
            }) || objectives.is_empty()
            {
                return Err("invalid supplied voter preference".into());
            }
            objectives::validate(w, objectives)?;
        }
        for p in c.programs.values() {
            if p.name.trim().is_empty() || p.commands.is_empty() {
                return Err("empty organization program".into());
            }
            let mut kinds = BTreeSet::new();
            for command in &p.commands {
                let kind = match command {
                    Command::StatePolicy(id) => {
                        if !w.state_governance.as_ref().is_some_and(|g| {
                            g.state == agent && g.constitution.policies.contains_key(id)
                        }) {
                            return Err("program is outside state constitution".into());
                        }
                        0
                    }
                    Command::HouseholdPolicy(id) => {
                        if !w.households.iter().any(|h| {
                            h.agent == agent
                                && h.governance.constitution.permitted_policies.contains(id)
                        }) {
                            return Err("program is outside household constitution".into());
                        }
                        1
                    }
                    Command::WorkTargets(orders) => {
                        if orders.iter().any(|o| {
                            o.agent != agent || !w.definitions.iter().any(|d| d.id == o.definition)
                        }) {
                            return Err("program may direct only its own work".into());
                        }
                        2
                    }
                    Command::StartProcess(id) => {
                        if !w
                            .definitions
                            .iter()
                            .any(|d| d.id == *id && d.execution == Execution::Productive)
                        {
                            return Err("unknown productive program".into());
                        }
                        3
                    }
                };
                if !kinds.insert(kind) {
                    return Err("duplicate program command kind".into());
                }
            }
        }
    }
    Ok(())
}

pub fn validate_history(w: &World, s: &State) -> Result<(), String> {
    for (&agent, c) in &w.agency {
        if c.applied_through > s.month || (!c.history.is_empty() && c.mandate.is_none()) {
            return Err("invalid organization decision boundary".into());
        }
        let catalog = if c.discover_programs {
            discovery::programs(w, agent)?
        } else {
            c.config.programs.clone()
        };
        let mut previous = 0;
        for d in &c.history {
            let authority =
                if let Some(g) = w.state_governance.as_ref().filter(|g| g.state == agent) {
                    crate::state_governance::governor_at_open(w, s, g, d.month)
                } else {
                    w.households
                        .iter()
                        .find(|h| h.agent == agent)
                        .and_then(|h| crate::household_governance::leader_at_open(h, s, d.month))
                };
            if d.month <= previous
                || d.month > s.month
                || d.month.checked_add(1) != Some(d.effective_month)
                || d.accepted.as_ref() != d.chosen.and_then(|id| d.catalog.get(&id))
                || d.catalog != catalog
                || (d.chosen.is_some()
                    && (d.authorized_by.is_none() || d.authorized_by != authority))
            {
                return Err("invalid accepted organization decision".into());
            }
            previous = d.month;
        }
    }
    Ok(())
}

fn issue(w: &mut World, s: &State, agent: AgentId, p: &Program) -> Result<(), String> {
    let authorized_by = governor(w, s, agent).ok_or("organization office is vacant")?;
    let month = s.month.checked_add(1).ok_or("decision month overflow")?;
    for command in &p.commands {
        match command {
            Command::StatePolicy(policy) => crate::state_governance::schedule(
                w,
                s,
                crate::state_governance::PolicyChange {
                    month,
                    authorized_by,
                    policy: *policy,
                },
            )?,
            Command::HouseholdPolicy(policy) => crate::household_governance::schedule(
                w,
                s,
                agent,
                crate::household_governance::PolicyChange {
                    month,
                    authorized_by,
                    policy: *policy,
                },
            )?,
            Command::WorkTargets(orders) => {
                if orders.iter().any(|o| {
                    !crate::opportunities::permits(
                        w,
                        s,
                        agent,
                        crate::opportunities::Action::Process(o.definition),
                    )
                }) {
                    return Err("organization work is prohibited".into());
                }
            }
            Command::StartProcess(definition) => {
                if !crate::opportunities::permits(
                    w,
                    s,
                    agent,
                    crate::opportunities::Action::Process(*definition),
                ) {
                    return Err("organization process is prohibited".into());
                }
                if !w
                    .scheduled_starts
                    .iter()
                    .any(|a| a.agent == agent && a.month == month && a.definition == *definition)
                {
                    w.scheduled_starts.push(ScheduledStart {
                        month,
                        agent,
                        definition: *definition,
                    });
                }
                if let Some(c) = w
                    .minting
                    .as_mut()
                    .filter(|c| c.issuer == agent && c.definition == *definition)
                {
                    let policy = c
                        .order_policy
                        .as_mut()
                        .ok_or("agentic minting requires generated market orders")?;
                    if policy.month == 0 {
                        policy.month = month;
                    } else if month < policy.month {
                        policy.additional_months.insert(policy.month);
                        policy.month = month;
                    } else if month > policy.month {
                        policy.additional_months.insert(month);
                    }
                }
            }
        }
    }
    Ok(())
}

/// Accepted dated work is public intent, unlike unpublished scenario fixtures.
pub(crate) fn accepted_start(w: &World, start: &ScheduledStart) -> bool {
    w.agency.get(&start.agent).is_some_and(|c| {
        c.history.iter().any(|d| {
            d.effective_month == start.month
                && d.accepted.as_ref().is_some_and(|p| {
                    p.commands
                        .contains(&Command::StartProcess(start.definition))
                })
        })
    })
}

fn apply_due(w: &mut World, s: &State) {
    let agents: Vec<_> = w.agency.keys().copied().collect();
    for agent in agents {
        let c = &w.agency[&agent];
        if c.applied_through >= s.month {
            continue;
        }
        let commands: Vec<_> = c
            .history
            .iter()
            .filter(|d| d.effective_month > c.applied_through && d.effective_month <= s.month)
            .filter_map(|d| d.accepted.as_ref())
            .flat_map(|p| p.commands.clone())
            .collect();
        for command in commands {
            if let Command::WorkTargets(orders) = command {
                w.activities.orders.retain(|o| o.agent != agent);
                w.activities.orders.extend(orders);
            }
        }
        w.agency.get_mut(&agent).unwrap().applied_through = s.month;
    }
}

fn forecast(w: &World, s: &State, agent: AgentId, c: &Config, choice: Option<u32>) -> Alternative {
    type Losses = (Vec<i128>, BTreeMap<AgentId, Vec<i128>>);
    let run = || -> Result<Losses, String> {
        let (mut world, state) = crate::forecast::ForecastContext::new(w, s).into_parts();
        if let Some(id) = choice {
            issue(&mut world, &state, agent, &c.programs[&id])?;
            world
                .agency
                .get_mut(&agent)
                .unwrap()
                .history
                .push(Decision {
                    month: s.month,
                    effective_month: s.month + 1,
                    authorized_by: governor(w, s, agent),
                    chosen: choice,
                    accepted: Some(c.programs[&id].clone()),
                    catalog: c.programs.clone(),
                    alternatives: vec![],
                    observed: vec![],
                    reason: "hypothesis".into(),
                    ballots: vec![],
                });
        }
        let mut sim = Simulation::new(world, state, crate::compute::Backend::Reference)?;
        sim.run_months(c.horizon)?;
        Ok((
            measure(
                &sim.world,
                &sim.state,
                &sim.reports,
                agent,
                &objectives::freeze(w, s, agent, &c.objectives),
            )?,
            c.preferences
                .iter()
                .map(|(&p, objectives)| {
                    Ok((
                        p,
                        measure(
                            &sim.world,
                            &sim.state,
                            &sim.reports,
                            agent,
                            &objectives::freeze(w, s, agent, objectives),
                        )?,
                    ))
                })
                .collect::<Result<_, String>>()?,
        ))
    };
    match run() {
        Ok((losses, preferences)) => Alternative {
            program: choice,
            losses: Some(losses),
            preferences,
            failure: None,
        },
        Err(failure) => Alternative {
            program: choice,
            losses: None,
            preferences: BTreeMap::new(),
            failure: Some(failure),
        },
    }
}

fn best(alternatives: &[Alternative], preferences: Option<AgentId>) -> Option<&Alternative> {
    alternatives
        .iter()
        .filter_map(|a| {
            let loss = if let Some(person) = preferences {
                a.preferences.get(&person)
            } else {
                a.losses.as_ref()
            }?;
            Some((loss, a.program, a))
        })
        .min_by_key(|(loss, program, _)| (*loss, *program))
        .map(|(_, _, a)| a)
}

fn election_context(
    w: &World,
    s: &State,
    agent: AgentId,
) -> Option<Vec<crate::governance::AcceptedBallot>> {
    if s.terminal.contains_key(&agent) {
        return None;
    }
    let (formed, term, leadership, ballots) =
        if let Some(g) = w.state_governance.as_ref().filter(|g| g.state == agent) {
            (
                g.formed,
                g.charter.term_months,
                g.constitution.leadership,
                &g.ballots,
            )
        } else {
            let h = w.households.iter().find(|h| h.agent == agent)?;
            if crate::households::dissolution::winding_at(h, s.month).is_some() {
                return None;
            }
            (
                h.formed,
                h.governance.charter.term_months,
                h.governance.constitution.leadership,
                &h.governance.ballots,
            )
        };
    (leadership == crate::governance::Leadership::Elected
        && s.month
            .checked_add(1)?
            .checked_sub(formed)?
            .is_multiple_of(term))
    .then(|| ballots.clone())
}

fn election_due(w: &World, s: &State, agent: AgentId) -> bool {
    election_context(w, s, agent).is_some()
}

fn vote(
    w: &mut World,
    s: &State,
    agent: AgentId,
    c: &Config,
    alternatives: &[Alternative],
) -> Result<Vec<crate::governance::Ballot>, String> {
    let Some(existing) = election_context(w, s, agent) else {
        return Ok(vec![]);
    };
    let members = objectives::members(w, s, agent);
    let state = w
        .state_governance
        .as_ref()
        .is_some_and(|g| g.state == agent);
    let eligible: Vec<_> = c
        .preferences
        .keys()
        .copied()
        .filter(|id| {
            members.contains(id)
                && !s.terminal.contains_key(id)
                && (!state
                    || s.memberships
                        .contains_key(&(*id, agent, crate::membership::CITIZEN)))
        })
        .collect();
    let mut ballots = vec![];
    for voter in &eligible {
        if existing
            .iter()
            .any(|b| b.ballot.voter == *voter && b.ballot.term_start == s.month + 1)
        {
            continue;
        }
        let candidate = eligible
            .iter()
            .filter_map(|candidate| {
                let platform = best(alternatives, Some(*candidate))?;
                Some((platform.preferences.get(voter)?, *candidate))
            })
            .min()
            .map(|(_, candidate)| candidate);
        let ballot = crate::governance::Ballot {
            term_start: s.month + 1,
            voter: *voter,
            candidate,
        };
        if state {
            crate::state_governance::cast(w, s, ballot.clone())?;
        } else {
            crate::household_governance::elections::cast(w, s, agent, ballot.clone())?;
        }
        ballots.push(ballot);
    }
    Ok(ballots)
}

/// One Open pass. Compare programs from a common snapshot; publish only the
/// receiving institution's authorized intent. Peers retain their own planners.
pub(crate) fn open(w: &mut World, s: &State) -> Result<(), String> {
    validate(w)?;
    validate_history(w, s)?;
    for c in w.agency.values_mut().filter(|c| c.autonomous) {
        c.mandate
            .get_or_insert_with(|| (c.config.clone(), c.discover_programs));
    }
    apply_due(w, s);
    let opening = w.clone();
    let actors: Vec<_> = w
        .agency
        .iter()
        .filter(|(_, c)| c.autonomous && c.history.last().is_none_or(|d| d.month < s.month))
        .map(|(&a, _)| a)
        .collect();
    for agent in actors {
        let c = &opening.agency[&agent];
        let mut config = c.config.clone();
        if c.discover_programs {
            config.programs = discovery::programs(&opening, agent)?;
        }
        let authority = governor(&opening, s, agent);
        let due = (s.month - 1).is_multiple_of(c.config.review_every);
        let observed = measure(&opening, s, &[], agent, &c.config.objectives)?;
        let mut decision = Decision {
            month: s.month,
            effective_month: s.month + 1,
            authorized_by: authority,
            chosen: None,
            accepted: None,
            catalog: config.programs.clone(),
            alternatives: vec![],
            observed,
            reason: "waiting for review".into(),
            ballots: vec![],
        };
        let emergency = c
            .config
            .emergency
            .is_some_and(|(i, ceiling)| decision.observed[i] > ceiling);
        if due || emergency || election_due(&opening, s, agent) {
            decision.alternatives = std::iter::once(None)
                .chain(config.programs.keys().copied().map(Some))
                .map(|choice| forecast(&opening, s, agent, &config, choice))
                .collect();
            let preference = authority.filter(|id| c.config.preferences.contains_key(id));
            let selected = best(&decision.alternatives, preference);
            let last = c.history.iter().rev().find(|d| d.chosen.is_some());
            let held = last.is_some_and(|last| s.month - last.month < c.config.minimum_tenure);
            if !due && !emergency {
                decision.reason = "election observation; policy review not due".into();
            } else if authority.is_none() {
                decision.reason = "vacant office".into();
            } else if held && !emergency {
                decision.reason = "retained within minimum tenure".into();
            } else if let Some(selected) = selected {
                decision.chosen = selected.program;
                decision.reason = if selected.program.is_some() {
                    "improves projected objectives"
                } else {
                    "retain on equal or better projection"
                }
                .into();
                if let Some(id) = selected.program {
                    issue(w, s, agent, &config.programs[&id])?;
                    decision.accepted = Some(config.programs[&id].clone());
                }
            } else {
                decision.reason = "no valid projection; preserve accepted commitments".into();
            }
            decision.ballots = vote(w, s, agent, &c.config, &decision.alternatives)?;
        }
        w.agency.get_mut(&agent).unwrap().history.push(decision);
    }
    Ok(())
}
