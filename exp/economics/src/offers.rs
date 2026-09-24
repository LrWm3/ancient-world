//! One discovery / proposal / feasibility / acceptance interface over domain resolvers.
//! A proposal is read-only; only ordinary settlement can publish it.
use crate::{agreements::ProductionTerms, compute::Backend, model::*, simulation::Simulation};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Id {
    Membership(u32),
    Land(u32),
    Process(DefinitionId),
    FinancedPurchase(u32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Terms {
    Membership(crate::membership::Offer),
    Land(crate::commitments::Agreement),
    Production(ProductionTerms),
    Collection {
        production: ProductionTerms,
        supply: crate::pool_market::Supply,
    },
    /// The first adapter retains the configured buyer, date and downpayment.
    FinancedPurchase {
        offer: crate::credit::Offer,
        application: crate::credit::Application,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub id: Id,
    pub terms: Terms,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub offer: Id,
    pub agent: AgentId,
    pub continuing: Option<u64>,
    pub need: Option<ResourceId>,
}
impl Request {
    pub fn new(offer: Id, agent: AgentId) -> Self {
        Self {
            offer,
            agent,
            continuing: None,
            need: None,
        }
    }
}

pub fn discover(world: &World, state: &State, agent: AgentId) -> Vec<Offer> {
    let mut offers: Vec<_> = crate::opportunities::discover(world, state, agent)
        .into_iter()
        .map(|o| match o {
            crate::opportunities::Opportunity::Membership(m) => Offer {
                id: Id::Membership(m.id),
                terms: Terms::Membership(m.clone()),
            },
            crate::opportunities::Opportunity::StateAccess(a) => Offer {
                id: Id::Land(a.id),
                terms: Terms::Land(a.clone()),
            },
            crate::opportunities::Opportunity::Environment(d) => Offer {
                id: Id::Process(d.id),
                terms: if let Some(supply) =
                    crate::pool_market::supply(world, state).filter(|s| s.definition == d.id)
                {
                    Terms::Collection {
                        production: ProductionTerms::from_definition(d),
                        supply,
                    }
                } else {
                    Terms::Production(ProductionTerms::from_definition(d))
                },
            },
        })
        .collect();
    if let Some(c) = &world.credit
        && c.application.buyer == agent
        && state.month <= c.application.month
    {
        offers.extend(
            crate::credit::discover(world, state, agent)
                .into_iter()
                .filter(|o| o.id == c.application.offer)
                .map(|o| Offer {
                    id: Id::FinancedPurchase(o.id),
                    terms: Terms::FinancedPurchase {
                        offer: o.clone(),
                        application: c.application.clone(),
                    },
                }),
        );
    }
    offers
}

/// Scripted buyer policy; acceptance still passes through the common dispatcher.
pub(crate) fn scripted_credit_request(world: &World, state: &State) -> Option<Request> {
    let c = world.credit.as_ref()?;
    (state.phase == Phase::Acquire
        && c.application.month == state.month
        && !state.credit.loans.contains_key(&c.application.offer))
    .then(|| {
        Request::new(
            Id::FinancedPurchase(c.application.offer),
            c.application.buyer,
        )
    })
}

/// Shared dispatcher used by search acquisitions and normal process allocation.
/// Callers retain ordering policy; work shares one opening budget and reservation map.
pub(crate) fn resolve(
    sim: &Simulation,
    requests: &[Request],
    batch: &mut Batch,
) -> Result<(), String> {
    if requests
        .iter()
        .any(|r| matches!(r.offer, Id::FinancedPurchase(_)))
    {
        if requests.len() != 1 || *batch != Batch::empty(&sim.state) {
            return Err("financed purchase requires a standalone acquisition request".into());
        }
        let request = &requests[0];
        if scripted_credit_request(&sim.world, &sim.state).as_ref() != Some(request) {
            return Err("financed purchase differs from configured application or is stale".into());
        }
        if sim.world.negotiation.is_some() || sim.world.market.is_some() {
            *batch = crate::acquisition::evaluate(&sim.world, &sim.state)?;
            return Ok(());
        }
        // Credit owns funding, collateral checks and the exact rejection receipts.
        let credit = crate::credit::evaluate(&sim.world, &sim.state)?;
        let mut staged = batch.clone();
        staged.transactions = credit
            .as_ref()
            .ok_or("missing credit boundary")?
            .transactions
            .clone();
        staged.production_plan = credit.as_ref().and_then(|c| c.production_plan.clone());
        staged.credit = credit;
        *batch = staged;
        return Ok(());
    }
    let mut staged_batch = batch.clone();
    let mut preview = sim.state.clone();
    let mut work = Vec::new();
    for request in requests {
        if !sim.world.agents.iter().any(|a| a.id == request.agent) {
            return Err("unknown offer applicant".into());
        }
        match request.offer {
            Id::FinancedPurchase(_) => unreachable!("handled above"),
            Id::Membership(offer) => {
                if request.continuing.is_some() || !work.is_empty() {
                    return Err("duplicate or misordered membership acceptance".into());
                }
                let m = crate::membership::acceptance(&sim.world, &preview, offer, request.agent)?;
                preview
                    .memberships
                    .insert((m.member, m.organization, m.role), m);
                if staged_batch.accept_membership.is_none() {
                    staged_batch.accept_membership = Some((offer, request.agent));
                } else {
                    staged_batch
                        .additional_memberships
                        .push((offer, request.agent));
                }
            }
            Id::Land(offer) => {
                if request.continuing.is_some() || !work.is_empty() {
                    return Err("duplicate or misordered land acceptance".into());
                }
                let a =
                    crate::commitments::acceptance_for(&sim.world, &preview, offer, request.agent)?;
                if a.debtor != request.agent {
                    return Err("land applicant differs from offer".into());
                }
                preview.accepted_agreements.insert(offer, a);
                if staged_batch.accept_access.is_none() {
                    staged_batch.accept_access = Some(offer);
                    if sim.world.open_access_offers.contains(&offer) {
                        staged_batch.access_applicant = Some(request.agent);
                    }
                } else {
                    staged_batch.additional_access.push((offer, request.agent));
                }
            }
            Id::Process(definition) => {
                let d = sim
                    .world
                    .definitions
                    .iter()
                    .find(|d| d.id == definition)
                    .ok_or("unknown process offer")?;
                let expected = match d.execution {
                    Execution::Productive => Phase::Productive,
                    Execution::Consumption => Phase::Consumption,
                };
                if sim.state.phase != expected {
                    return Err("process acceptance outside execution boundary".into());
                }
                if let Some(id) = request.continuing {
                    let p = sim
                        .state
                        .processes
                        .get(&id)
                        .ok_or("unknown continuing agreement")?;
                    if p.operator != request.agent
                        || p.definition != definition
                        || p.status != Status::Active
                    {
                        return Err("continuing agreement does not belong to request".into());
                    }
                }
                work.push(crate::simulation::Request {
                    agent: request.agent,
                    definition,
                    existing: request.continuing,
                    need: request.need,
                });
            }
        }
    }
    if !work.is_empty() || (sim.world.pool_market.is_some() && sim.state.phase == Phase::Productive)
    {
        sim.resolve_work(work, &mut staged_batch)?;
    }
    *batch = staged_batch;
    Ok(())
}

/// Preview an explicit bundle. Acquire can accept prerequisites plus reserve dated
/// productive work; its first execution still occurs at Productive. All existing
/// processes precede requested new work. Automatic planning uses its own ordering.
pub fn prepare(sim: &Simulation, requests: &[Request]) -> Result<Batch, String> {
    if requests
        .iter()
        .any(|r| matches!(r.offer, Id::FinancedPurchase(_)))
    {
        let mut batch = Batch::empty(&sim.state);
        resolve(sim, requests, &mut batch)?;
        let boundary = batch
            .credit
            .as_ref()
            .ok_or("missing financed purchase receipt")?;
        if !boundary
            .events
            .iter()
            .any(|e| matches!(e, crate::credit::Event::Purchased { .. }))
        {
            return Err(format!("financed purchase rejected: {:?}", boundary.events));
        }
        let mut checked = sim.state.clone();
        crate::settlement::commit(
            &sim.world,
            &mut checked,
            &batch,
            Backend::Reference,
            sim.effect_limit,
        )?;
        return Ok(batch);
    }
    if requests.iter().any(|r| r.continuing.is_some()) {
        return Err("explicit acceptance requests must be new offers".into());
    }
    let split = requests
        .iter()
        .position(|r| matches!(r.offer, Id::Process(_)))
        .unwrap_or(requests.len());
    if requests[split..]
        .iter()
        .any(|r| !matches!(r.offer, Id::Process(_)))
    {
        return Err("prerequisites must precede process offers".into());
    }
    let mut batch = Batch::empty(&sim.state);
    let mut preview = sim.clone();
    resolve(&preview, &requests[..split], &mut batch)?;
    if requests.len() > split {
        if preview.state.phase == Phase::Acquire {
            crate::settlement::commit(
                &preview.world,
                &mut preview.state,
                &batch,
                Backend::Reference,
                preview.effect_limit,
            )?;
        }
        let mut work = Batch::empty(&preview.state);
        let mut work_requests: Vec<_> = preview
            .state
            .processes
            .values()
            .filter(|p| p.status == Status::Active && preview.state.phase == Phase::Productive)
            .map(|p| Request {
                offer: Id::Process(p.definition),
                agent: p.operator,
                continuing: Some(p.id),
                need: p.goal,
            })
            .collect();
        work_requests.extend_from_slice(&requests[split..]);
        resolve(&preview, &work_requests, &mut work)?;
        let created = work
            .transactions
            .iter()
            .filter(|t| t.process.as_ref().is_some_and(|c| c.before.is_none()))
            .count();
        if created != requests.len() - split {
            return Err(
                "offer bundle cannot meet opening resource, right or capacity requirements".into(),
            );
        }
        crate::settlement::commit(
            &preview.world,
            &mut preview.state,
            &work,
            Backend::Reference,
            preview.effect_limit,
        )?;
        if sim.state.phase == Phase::Acquire {
            batch.production_plan = Some(Box::new(work));
        } else {
            batch = work;
        }
    }
    let mut checked = sim.state.clone();
    crate::settlement::commit(
        &sim.world,
        &mut checked,
        &batch,
        Backend::Reference,
        sim.effect_limit,
    )?;
    Ok(batch)
}

pub fn feasible(sim: &Simulation, requests: &[Request]) -> Result<(), String> {
    prepare(sim, requests).map(|_| ())
}

pub fn accept(sim: &mut Simulation, requests: &[Request]) -> Result<(), String> {
    let batch = prepare(sim, requests)?;
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
