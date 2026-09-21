//! Multiple open land offers, conditional alternatives, one settlement.
//! The generic allocator also supports other ask/bid pools; this adapter supplies
//! land eligibility, priority, bundle feasibility and the ordinary offer resolver.
use crate::{
    allocation::{self, Claim, Context, Policy, Receipt},
    compute::Backend,
    model::*,
    offers::{self, Id, Request},
    simulation::Simulation,
};
use std::collections::{BTreeMap, BTreeSet};

pub const DEFAULT_SEED: u64 = 7;
pub const SECOND_PERSON: AgentId = crate::scenario::PERSON + 1;
const TWO_PERSON_WOOD_CAPACITY: i32 = 24;
const TWO_PERSON_WOOD_REGENERATION: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application {
    pub agent: AgentId,
    /// Land plus optional citizenship and productive work, in prerequisite order.
    pub requests: Vec<Request>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub offers: Vec<u32>,
    pub awards: BTreeMap<u64, u64>,
    pub rejections: Vec<(AgentId, u32, String)>,
    pub context: Context,
    pub policy: Policy,
    pub applications: Vec<Application>,
    pub receipts: Vec<Receipt>,
}

/// Compatibility entry point for one posting.
pub fn prepare(
    sim: &Simulation,
    offer: u32,
    seed: u64,
    policy: Policy,
    applications: &[Application],
) -> Result<Batch, String> {
    prepare_many(sim, &[offer], seed, policy, applications)
}

fn land(application: &Application) -> Result<u32, String> {
    let ids: Vec<_> = application
        .requests
        .iter()
        .filter_map(|r| {
            if let Id::Land(id) = r.offer {
                Some(id)
            } else {
                None
            }
        })
        .collect();
    if ids.len() != 1
        || application
            .requests
            .iter()
            .any(|r| r.agent != application.agent)
    {
        return Err("application requires one land offer and only its own requests".into());
    }
    Ok(ids[0])
}

/// Repeated agent IDs represent acceptable alternatives, not cumulative requests.
/// All alternatives see the opening state. At most one is accepted per applicant.
pub fn prepare_many(
    sim: &Simulation,
    offers: &[u32],
    seed: u64,
    policy: Policy,
    applications: &[Application],
) -> Result<Batch, String> {
    let offers: Vec<_> = offers
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if sim.state.phase != Phase::Acquire
        || offers.is_empty()
        || offers
            .iter()
            .any(|id| !sim.world.open_access_offers.contains(id))
    {
        return Err("competition requires open offers at Acquire".into());
    }
    // The lowest stable posting ID identifies this scoped pool. A singleton keeps
    // its previous lottery; IDs and input iteration order do not act as timestamps.
    let context = Context {
        seed,
        pool: u64::from(offers[0]),
        round: u64::from(sim.state.month),
    };
    let mut keyed = BTreeMap::new();
    for application in applications {
        let id = land(application)?;
        if !offers.contains(&id)
            || keyed
                .insert((application.agent, id), application.clone())
                .is_some()
        {
            return Err("duplicate or out-of-round alternative".into());
        }
    }
    let applications: Vec<_> = keyed.values().cloned().collect();
    let mut rejections = Vec::new();
    let mut choices: BTreeMap<AgentId, allocation::Alternatives> = BTreeMap::new();
    for (&(agent, id), application) in &keyed {
        let has_plot = sim.world.rights.iter().any(|r| {
            crate::commitments::holder(&sim.world, &sim.state, r) == Some(agent)
                && r.from <= sim.state.month
                && r.through >= sim.state.month
                && sim
                    .world
                    .access_offers
                    .iter()
                    .find(|o| o.right == r.id)
                    .is_none_or(|o| sim.state.accepted_agreements.contains_key(&o.id))
        });
        let choice = choices
            .entry(agent)
            .or_insert_with(|| allocation::Alternatives {
                claim: Claim {
                    id: u64::from(agent),
                    priority: u32::from(has_plot),
                    requested: 1,
                    minimum: 1,
                },
                slots: vec![],
            });
        match offers::prepare(sim, &application.requests) {
            Ok(_) => choice.slots.push(u64::from(id)),
            Err(reason) => rejections.push((agent, id, reason)),
        }
    }
    // Matching is tentative. Recheck whole bundles cumulatively in policy order.
    // Each failed edge is removed once; retry may use another acceptable posting.
    let (mut batch, receipts, awards) = loop {
        let eligible: Vec<_> = choices
            .values()
            .filter(|c| !c.slots.is_empty())
            .cloned()
            .collect();
        let (mut receipts, awards) = allocation::assign(context, &policy, &eligible)?;
        for (&agent, c) in &choices {
            if c.slots.is_empty() {
                receipts.push(Receipt {
                    claim: c.claim.clone(),
                    offered: 0,
                    outcome: allocation::Outcome::Rejected(format!(
                        "no feasible alternative for agent {agent}"
                    )),
                });
            }
        }
        let mut accepted = Vec::new();
        let mut candidate = Batch::empty(&sim.state);
        let mut failed = false;
        for receipt in &receipts {
            let Some(&id) = awards.get(&receipt.claim.id) else {
                continue;
            };
            let agent = receipt.claim.id as AgentId;
            let id = id as u32;
            accepted.push(&keyed[&(agent, id)]);
            let mut requests: Vec<_> = accepted
                .iter()
                .flat_map(|a| {
                    a.requests
                        .iter()
                        .filter(|r| !matches!(r.offer, Id::Process(_)))
                        .cloned()
                })
                .collect();
            requests.extend(accepted.iter().flat_map(|a| {
                a.requests
                    .iter()
                    .filter(|r| matches!(r.offer, Id::Process(_)))
                    .cloned()
            }));
            match offers::prepare(sim, &requests) {
                Ok(prepared) => candidate = prepared,
                Err(reason) => {
                    choices
                        .get_mut(&agent)
                        .unwrap()
                        .slots
                        .retain(|slot| *slot != u64::from(id));
                    rejections.push((agent, id, reason));
                    failed = true;
                    break;
                }
            }
        }
        if !failed {
            break (candidate, receipts, awards);
        }
    };
    add_fallback_work(sim, &mut batch)?;
    batch.allocation = Some(Round {
        offers,
        context,
        policy,
        applications,
        receipts,
        awards,
        rejections,
    });
    Ok(batch)
}

pub fn accept(
    sim: &mut Simulation,
    offer: u32,
    seed: u64,
    policy: Policy,
    applications: &[Application],
) -> Result<(), String> {
    let batch = prepare(sim, offer, seed, policy, applications)?;
    crate::settlement::commit(
        &sim.world,
        &mut sim.state,
        &batch,
        sim.backend,
        sim.effect_limit,
    )?;
    sim.ledger.push(batch);
    Ok(())
}

pub(crate) fn validate_batch(
    world: &World,
    state: &State,
    batch: &Batch,
    effect_limit: usize,
) -> Result<(), String> {
    let round = batch
        .allocation
        .as_ref()
        .ok_or("missing allocation round")?;
    let mut sim = Simulation::new(world.clone(), state.clone(), Backend::Reference)?;
    sim.effect_limit = effect_limit;
    let expected = prepare_many(
        &sim,
        &round.offers,
        round.context.seed,
        round.policy,
        &round.applications,
    )?;
    if *batch != expected {
        return Err("allocation differs from opening applications and policy".into());
    }
    Ok(())
}

/// Retain the awarded bundle's reservations, then let everybody else replan with
/// the allocation now visible. Resolve all resulting work against one shared pool.
fn add_fallback_work(sim: &Simulation, batch: &mut Batch) -> Result<(), String> {
    let mut preview = sim.clone();
    let reserved = batch.production_plan.take();
    crate::settlement::commit(
        &preview.world,
        &mut preview.state,
        batch,
        Backend::Reference,
        preview.effect_limit,
    )?;
    let mut requests = Vec::new();
    if let Some(work) = reserved {
        for change in work.transactions.iter().filter_map(|t| t.process.as_ref()) {
            requests.push(crate::simulation::Request {
                agent: change.after.operator,
                definition: change.after.definition,
                existing: change.before.as_ref().map(|p| p.id),
                need: change.after.goal,
            });
        }
    } else {
        requests.extend(
            preview
                .state
                .processes
                .values()
                .filter(|p| p.status == Status::Active)
                .map(|p| crate::simulation::Request {
                    agent: p.operator,
                    definition: p.definition,
                    existing: Some(p.id),
                    need: p.goal,
                }),
        );
    }
    let mut agents: Vec<_> = sim.world.participants.iter().map(|p| p.agent).collect();
    agents.sort_unstable();
    for agent in agents {
        if batch.access_applicant == Some(agent)
            || batch.additional_access.iter().any(|(_, a)| *a == agent)
            || sim.state.terminal.contains_key(&agent)
        {
            continue;
        }
        let projection = local(&preview, agent);
        let mut proposed = Batch::empty(&projection.state);
        crate::planning::choose(&projection, &mut proposed)?;
        requests.extend(
            proposed
                .transactions
                .iter()
                .filter_map(|t| t.process.as_ref())
                .filter(|c| c.before.is_none())
                .map(|c| crate::simulation::Request {
                    agent,
                    definition: c.after.definition,
                    existing: None,
                    need: c.after.goal,
                }),
        );
    }
    let mut work = Batch::empty(&preview.state);
    preview.resolve_work(requests, &mut work)?;
    crate::settlement::commit(
        &preview.world,
        &mut preview.state,
        &work,
        Backend::Reference,
        preview.effect_limit,
    )?;
    batch.production_plan = Some(Box::new(work));
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub seed: u64,
    pub policy: Policy,
}

/// Independent projections read the same opening balances. Other people's plans
/// are not known; settlement remains responsible for shared-resource contention.
fn local(sim: &Simulation, agent: AgentId) -> Simulation {
    let mut local = sim.clone();
    local.world.competition = None;
    local.world.participants.retain(|p| p.agent == agent);
    local.world.condition_rules.retain(|r| r.subject == agent);
    local
        .state
        .conditions
        .retain(|(owner, _), _| *owner == agent);
    local.state.terminal.retain(|owner, _| *owner == agent);
    local.state.processes.retain(|_, p| p.operator == agent);
    local.state.pending_production = None;
    local
}

pub(crate) fn choose(sim: &Simulation, batch: &mut Batch) -> Result<(), String> {
    let config = sim
        .world
        .competition
        .as_ref()
        .ok_or("missing competition policy")?;
    let offers: Vec<_> = sim
        .world
        .open_access_offers
        .iter()
        .copied()
        .filter(|id| !sim.state.accepted_agreements.contains_key(id))
        .collect();
    if offers.is_empty() {
        return crate::planning::choose(sim, batch);
    }
    let mut applications = Vec::new();
    for participant in &sim.world.participants {
        if sim.state.terminal.contains_key(&participant.agent) {
            continue;
        }
        for &offer in &offers {
            let mut projection = local(sim, participant.agent);
            // Independently evaluate each acceptable alternative at the same opening.
            projection.world.access_offers.retain(|a| {
                a.id == offer || projection.state.accepted_agreements.contains_key(&a.id)
            });
            projection
                .world
                .open_access_offers
                .retain(|id| projection.world.access_offers.iter().any(|a| a.id == *id));
            let mut proposed = Batch::empty(&projection.state);
            crate::planning::choose(&projection, &mut proposed)?;
            if proposed.accept_access != Some(offer) {
                continue;
            }
            let mut requests = Vec::new();
            if let Some((id, agent)) = proposed.accept_membership {
                requests.push(Request::new(Id::Membership(id), agent));
            }
            requests.push(Request::new(Id::Land(offer), participant.agent));
            if let Some(work) = &proposed.production_plan {
                requests.extend(
                    work.transactions
                        .iter()
                        .filter_map(|t| t.process.as_ref())
                        .filter(|c| c.before.is_none() && c.after.operator == participant.agent)
                        .map(|c| Request {
                            offer: Id::Process(c.after.definition),
                            agent: participant.agent,
                            continuing: None,
                            need: c.after.goal,
                        }),
                );
            }
            applications.push(Application {
                agent: participant.agent,
                requests,
            });
        }
    }
    *batch = prepare_many(sim, &offers, config.seed, config.policy, &applications)?;
    Ok(())
}

/// Two independent people; only plot count changes between the two controls.
pub fn scenario(plots: u32, seed: u64) -> Result<(World, State), String> {
    use crate::scenario::*;
    if !(1..=2).contains(&plots) {
        return Err("pilot supports one or two plots".into());
    }
    let (mut world, mut state) = crate::membership::scenario()?;
    world.competition = Some(Config {
        seed,
        policy: Policy::PriorityLottery,
    });
    world.agents.push(Agent {
        id: SECOND_PERSON,
        name: "second person".into(),
    });
    let mut person = world.participants[0].clone();
    person.agent = SECOND_PERSON;
    world.participants.push(person);
    let rules = world.condition_rules.clone();
    for mut rule in rules {
        rule.subject = SECOND_PERSON;
        world.condition_rules.push(rule);
    }
    let stocks: Vec<_> = state
        .balances
        .iter()
        .filter(|((a, _), _)| *a == PERSON)
        .map(|((_, r), q)| (*r, *q))
        .collect();
    for (resource, quantity) in stocks {
        state.balances.insert((SECOND_PERSON, resource), quantity);
    }
    if let Some(capacity) = world.storage.capacities.get(&PERSON).copied() {
        world.storage.capacities.insert(SECOND_PERSON, capacity);
    }
    let policy = world.transaction_policy.as_mut().unwrap();
    let person_type = policy.agent_types[&PERSON];
    policy.agent_types.insert(SECOND_PERSON, person_type);
    // Placeholder holder is the issuer; it grants nobody permission before binding.
    world.access_offers[0].debtor = STATE_AGENT;
    world.rights[0].holder = STATE_AGENT;
    world.rights[0].output_owner = STATE_AGENT;
    for offset in 1..plots {
        let mut asset = world.assets[0].clone();
        asset.id += offset;
        world.assets.push(asset);
        let mut right = world.rights[0].clone();
        right.id += offset;
        right.asset += offset;
        world.rights.push(right);
        let mut offer = world.access_offers[0].clone();
        offer.id += offset;
        offer.right += offset;
        world.access_offers.push(offer);
    }
    world.open_access_offers = world.access_offers.iter().map(|a| a.id).collect();
    for pool in &mut world.pools {
        pool.capacity = TWO_PERSON_WOOD_CAPACITY;
        pool.monthly_regeneration = TWO_PERSON_WOOD_REGENERATION;
        state
            .balances
            .insert(pool.account, TWO_PERSON_WOOD_CAPACITY);
    }
    Ok((world, state))
}
