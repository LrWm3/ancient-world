//! Shared bounded organization decisions. Authority and economic execution stay
//! in their domain adapters; objectives never grant resources or legal powers.
use crate::{activities::WorkOrder, model::*, simulation::Simulation};
use std::collections::{BTreeMap, BTreeSet};

pub mod discovery;
pub mod discovery_sales;
pub mod expectations;
pub mod forwards;
pub mod integration;
pub mod markets;
pub mod objectives;
pub mod organization;
pub mod publication;
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
    /// Replace this organization's peer portfolio and signed issuer-input slots next Open.
    PeerStockQuotes(Vec<crate::minting::orders::Quote>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub name: String,
    pub commands: Vec<Command>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub horizon: u32,
    pub peer_expectations: expectations::Policy,
    pub publication: publication::Policy,
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
    pub forward_reviews: Vec<forwards::review::Attempt>,
    pub startup_forwards: Vec<forwards::execution::Agreement>,
    /// Static operating mandate sealed at the first live opening.
    mandate: Option<(Config, bool)>,
    organization: Option<organization::Agreement>,
}
impl Controller {
    pub fn operating_agreement(&self) -> Option<&organization::Agreement> {
        self.organization.as_ref()
    }

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
            startup_forwards: vec![],
            forward_reviews: vec![],
            mandate: None,
            organization: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ProjectedSale {
    pub month: u32,
    pub market: crate::marketplace::MarketId,
    pub buyer: AgentId,
    pub resource: ResourceId,
    pub quantity: i32,
    pub payment: ResourceId,
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Projection {
    /// Settled sales in this private branch, never receipts promised to live agents.
    pub sales: Vec<ProjectedSale>,
    pub closing_stocks: BTreeMap<ResourceId, i32>,
    pub peer_attempts: Vec<expectations::Attempt>,
    pub peer_ballots: Vec<expectations::elections::Attempt>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Alternative {
    /// None retains accepted policies and work; it does not cancel commitments.
    pub program: Option<u32>,
    pub losses: Option<Vec<i128>>,
    pub preferences: BTreeMap<AgentId, Vec<i128>>,
    pub failure: Option<String>,
    pub projection: Option<Projection>,
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
    pub peer_expectations: Option<expectations::Snapshot>,
    pub sale_opportunities: Vec<discovery_sales::Opportunity>,
    /// Current stocks/claims/history; need fulfillment is the preceding month.
    pub observed: Vec<i128>,
    pub reason: String,
    pub ballots: Vec<crate::governance::Ballot>,
}

pub fn governor(w: &World, s: &State, agent: AgentId) -> Option<AgentId> {
    if let Some(a) = w.agency.get(&agent).and_then(|c| c.organization.as_ref()) {
        return organization::current(w, s, a);
    }
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

/// Authorized operating choices from ordinary reviews or activated bilateral
/// startup agreements. Pending/rejected proposals are not completed choices.
pub fn accepted_programs(w: &World, agent: AgentId) -> impl Iterator<Item = &Program> {
    w.agency
        .get(&agent)
        .into_iter()
        .flat_map(|c| &c.history)
        .filter(|d| d.authorized_by.is_some())
        .filter_map(|d| d.accepted.as_ref())
        .chain(
            w.agency
                .values()
                .flat_map(|c| &c.startup_forwards)
                .filter(move |a| {
                    matches!(a.status, forwards::execution::Status::Activated(_))
                        && a.governors.contains_key(&agent)
                })
                .filter_map(move |a| a.programs.get(&agent)),
        )
}

pub fn validate(w: &World) -> Result<(), String> {
    for (&agent, controller) in &w.agency {
        let c = &controller.config;
        c.peer_expectations.validate()?;
        if let Some(a) = &controller.organization {
            organization::validate(w, agent, a)?;
        }
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
                && !w.households.iter().any(|h| h.agent == agent)
                && controller.organization.is_none())
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
            validate_program(w, agent, p)?;
        }
    }
    markets::validate_joint_authorities(w)
}

/// Validate a complete proposed program before publishing any commands.
pub fn validate_program(w: &World, agent: AgentId, p: &Program) -> Result<(), String> {
    if !w.agency.contains_key(&agent) {
        return Err("missing organization controller".into());
    }
    if p.name.trim().is_empty() || p.commands.is_empty() {
        return Err("empty organization program".into());
    }
    let mut kinds = BTreeSet::new();
    for command in &p.commands {
        let kind = match command {
            Command::StatePolicy(id) => {
                if !w
                    .state_governance
                    .as_ref()
                    .is_some_and(|g| g.state == agent && g.constitution.policies.contains_key(id))
                {
                    return Err("program is outside state constitution".into());
                }
                0
            }
            Command::HouseholdPolicy(id) => {
                if !w.households.iter().any(|h| {
                    h.agent == agent && h.governance.constitution.permitted_policies.contains(id)
                }) {
                    return Err("program is outside household constitution".into());
                }
                1
            }
            Command::WorkTargets(orders) => {
                crate::activities::validate_orders(w, orders)?;
                if orders.iter().any(|o| {
                    o.agent != agent
                        || !w.definitions.iter().any(|d| d.id == o.definition)
                        || w.agency[&agent]
                            .organization
                            .as_ref()
                            .is_some_and(|a| !a.processes.contains(&o.definition))
                }) {
                    return Err("program may direct only its own work".into());
                }
                2
            }
            Command::PeerStockQuotes(quotes) => {
                markets::validate_quotes(w, agent, quotes)?;
                4
            }
            Command::StartProcess(id) => {
                if w.agency[&agent]
                    .organization
                    .as_ref()
                    .is_some_and(|a| !a.processes.contains(id))
                    || !w
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
    Ok(())
}

pub fn validate_history(w: &World, s: &State) -> Result<(), String> {
    forwards::execution::validate_history(w, s)?;
    for (&agent, c) in &w.agency {
        if c.applied_through > s.month
            || ((!c.history.is_empty() || !c.startup_forwards.is_empty()) && c.mandate.is_none())
        {
            return Err("invalid organization decision boundary".into());
        }
        let catalog = if c.discover_programs {
            discovery::programs(w, agent)?
        } else {
            c.config.programs.clone()
        };
        let mut previous = 0;
        for d in &c.history {
            let authority = if let Some(a) = &c.organization {
                organization::at_open(w, s, a, d.month)
            } else if let Some(g) = w.state_governance.as_ref().filter(|g| g.state == agent) {
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
    validate_program(w, agent, p)?;
    let mut candidate = w.clone();
    issue_commands(&mut candidate, s, agent, p)?;
    *w = candidate;
    Ok(())
}

fn issue_commands(w: &mut World, s: &State, agent: AgentId, p: &Program) -> Result<(), String> {
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
            Command::PeerStockQuotes(quotes) => {
                markets::validate_quotes(w, agent, quotes)?;
                let venue = w.minting.as_ref().unwrap().venue;
                if !quotes.is_empty() && !crate::marketplace::eligible(w, s, venue, agent) {
                    return Err("organization stock trading is prohibited".into());
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
                schedule_start(w, agent, *definition, month)?;
            }
        }
    }
    Ok(())
}

/// Shared dated intent plumbing; authority belongs to the caller. No resources
/// are granted here, and every start still uses ordinary acquisition/execution.
pub(crate) fn schedule_start(
    w: &mut World,
    agent: AgentId,
    definition: DefinitionId,
    month: u32,
) -> Result<(), String> {
    if let Some(c) = w
        .minting
        .as_mut()
        .filter(|c| c.issuer == agent && c.definition == definition)
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
    if !w
        .scheduled_starts
        .iter()
        .any(|a| a.agent == agent && a.month == month && a.definition == definition)
    {
        w.scheduled_starts.push(ScheduledStart {
            month,
            agent,
            definition,
        });
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

fn apply_due(w: &mut World, s: &State) -> Result<(), String> {
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
            match command {
                Command::WorkTargets(orders) => {
                    w.activities.orders.retain(|o| o.agent != agent);
                    w.activities.orders.extend(orders);
                }
                Command::PeerStockQuotes(quotes) => markets::apply(w, agent, &quotes)?,
                _ => {}
            }
        }
        w.agency.get_mut(&agent).unwrap().applied_through = s.month;
    }
    Ok(())
}

fn forecast(
    w: &World,
    s: &State,
    agent: AgentId,
    c: &Config,
    choice: Option<u32>,
    peers: &expectations::Snapshot,
) -> Alternative {
    type Losses = (Vec<i128>, BTreeMap<AgentId, Vec<i128>>, Projection);
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
                    peer_expectations: None,
                    sale_opportunities: vec![],
                    observed: vec![],
                    reason: "hypothesis".into(),
                    ballots: vec![],
                });
        }
        let mut sim = Simulation::new(world, state, crate::compute::Backend::Reference)?;
        let end = s
            .month
            .checked_add(c.horizon)
            .ok_or("forecast month overflow")?;
        let mut starts = vec![];
        let mut peer_ballots = vec![];
        while sim.state.month < end {
            if sim.state.phase == Phase::Open {
                peer_ballots.extend(peers.apply_ballots(&mut sim.world, &sim.state)?);
                starts.extend(peers.apply_hypotheses(&mut sim.world, &sim.state)?);
            }
            sim.step()?;
        }
        let mut sales = vec![];
        if let Some(mint) = &sim.world.minting {
            let venue = crate::marketplace::venue(&sim.world, mint.venue)
                .ok_or("missing forecast venue")?;
            for batch in sim.ledger.iter().filter_map(|b| b.minting.as_ref()) {
                for d in batch.deals.iter().filter(|d| {
                    d.seller == agent
                        && batch
                            .receipts
                            .iter()
                            .any(|r| r.accepted && r.deals.contains(&d.id))
                }) {
                    let m = venue
                        .markets
                        .iter()
                        .find(|m| m.id == d.market)
                        .ok_or("missing forecast listing")?;
                    sales.push(ProjectedSale {
                        month: d.month,
                        market: d.market,
                        buyer: d.buyer,
                        resource: m.goods.resource,
                        quantity: m.goods.quantity,
                        payment: m.payment,
                        value: d.price,
                    });
                }
            }
        }
        let projection = Projection {
            sales,
            peer_ballots,
            closing_stocks: sim
                .world
                .resources
                .iter()
                .filter(|r| r.kind == ResourceKind::Stock)
                .map(|r| (r.id, sim.state.balance(agent, r.id)))
                .collect(),
            peer_attempts: starts
                .iter()
                .map(|a| expectations::Attempt {
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
        };
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
            projection,
        ))
    };
    match run() {
        Ok((losses, preferences, projection)) => Alternative {
            program: choice,
            losses: Some(losses),
            preferences,
            failure: None,
            projection: Some(projection),
        },
        Err(failure) => Alternative {
            program: choice,
            losses: None,
            preferences: BTreeMap::new(),
            failure: Some(failure),
            projection: None,
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
    apply_due(w, s)?;
    forwards::execution::activate_due(w, s)?;
    let opening = w.clone();
    let actors: Vec<_> = w
        .agency
        .iter()
        .filter(|(a, c)| {
            c.autonomous
                && !forwards::execution::pending(w, **a)
                && c.history.last().is_none_or(|d| d.month < s.month)
        })
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
            peer_expectations: None,
            sale_opportunities: if c.discover_programs {
                discovery_sales::opportunities(&opening, agent)?
            } else {
                vec![]
            },
            observed,
            reason: "waiting for review".into(),
            ballots: vec![],
        };
        let emergency = c
            .config
            .emergency
            .is_some_and(|(i, ceiling)| decision.observed[i] > ceiling);
        if due || emergency || election_due(&opening, s, agent) {
            let peers =
                expectations::Snapshot::observe(&opening, s, agent, c.config.peer_expectations)?;
            decision.alternatives = std::iter::once(None)
                .chain(config.programs.keys().copied().map(Some))
                .map(|choice| forecast(&opening, s, agent, &config, choice, &peers))
                .collect();
            decision.peer_expectations = Some(peers);
            let preference = authority.filter(|id| c.config.preferences.contains_key(id));
            let mut selected = best(&decision.alternatives, preference);
            let publication = if let Some(retain) = selected.filter(|a| a.program.is_none()) {
                publication::select(
                    &opening,
                    s,
                    agent,
                    &config,
                    preference,
                    retain,
                    &decision.alternatives,
                )?
            } else {
                None
            };
            if let Some(id) = publication {
                selected = decision.alternatives.iter().find(|a| a.program == Some(id));
            }
            let last = c.history.iter().rev().find(|d| d.chosen.is_some());
            let forward_month = opening
                .agency
                .values()
                .flat_map(|c| &c.startup_forwards)
                .filter(|a| {
                    [a.proposal.terms.buyer, a.proposal.terms.seller].contains(&agent)
                        && !matches!(a.status, forwards::execution::Status::AdmissionRejected(_))
                })
                .map(|a| a.month)
                .max();
            let held = last
                .map(|d| d.month)
                .into_iter()
                .chain(forward_month)
                .max()
                .is_some_and(|month| s.month - month < c.config.minimum_tenure);
            if !due && !emergency {
                decision.reason = "election observation; policy review not due".into();
            } else if authority.is_none() {
                decision.reason = "vacant office".into();
            } else if held && !emergency {
                decision.reason = "retained within minimum tenure".into();
            } else if let Some(selected) = selected {
                decision.chosen = selected.program;
                decision.reason = if publication.is_some() {
                    "publish stocked sale offers on equal projection"
                } else if selected.program.is_some() {
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
    forwards::review::open(w, s)?;
    Ok(())
}

#[cfg(test)]
mod atomic_program_tests {
    use super::*;
    #[test]
    fn later_command_failure_cannot_leave_an_earlier_scheduled_start() {
        use crate::{
            acceptance::{WORKSHOP, WORKSHOP_TYPE, continuation as c},
            minting::REFINE,
            opportunities::Action,
        };
        let (mut w, s) = c::spot_scenario(c::Case::Baseline).unwrap();
        let previous = w.agency.remove(&WORKSHOP).unwrap();
        let mut agreement = previous.operating_agreement().unwrap().clone();
        agreement.stock_markets.insert(
            crate::minting::ORE,
            markets::Options {
                side: crate::marketplace::Side::Buy,
                limits: [2].into(),
                holdings: [4].into(),
                max_lots: 2,
            },
        );
        organization::install_initial(
            &mut w,
            &s,
            agreement.clone(),
            &agreement.members,
            previous.config,
        )
        .unwrap();
        let old = w.agency[&WORKSHOP].config.programs[&1].clone();
        let mut p = Program {
            name: "compound".into(),
            commands: vec![
                Command::StartProcess(REFINE),
                Command::PeerStockQuotes(vec![crate::minting::orders::Quote {
                    agent: WORKSHOP,
                    market: crate::minting::ORE,
                    side: crate::marketplace::Side::Buy,
                    limit: 2,
                    holding: 4,
                    max_lots: Some(2),
                }]),
            ],
        };
        // First command schedules legal work; the later trade command lacks law.
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .remove(&(WORKSHOP_TYPE, Action::StockTrade));
        let before = w.clone();
        assert_eq!(
            issue(&mut w, &s, WORKSHOP, &p).unwrap_err(),
            "organization stock trading is prohibited"
        );
        assert_eq!(w, before);
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((WORKSHOP_TYPE, Action::StockTrade));
        let mut malformed = old;
        let Command::WorkTargets(orders) = &mut malformed.commands[0] else {
            panic!()
        };
        orders[0].target = crate::activities::Target::Stock(Amount::new(crate::minting::METAL, 0));
        assert!(validate_program(&w, WORKSHOP, &malformed).is_err());
        p.commands.push(Command::StartProcess(REFINE));
        let before = w.clone();
        assert!(issue(&mut w, &s, WORKSHOP, &p).is_err());
        assert_eq!(w, before);
    }
}
