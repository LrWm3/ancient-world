//! Shared consequence evaluator. Search proposes candidates; this module expands
//! dated work, forecasts through ordinary settlement and selects by the same score.
use crate::{
    compute::Backend,
    model::*,
    search::{
        CandidatePlan, OpportunitySearch, PlanStep, SearchBudget, SearchContext, SearchResult,
    },
    settlement::commit,
    simulation::Simulation,
};

const MAX_FORECAST_PARTICIPANTS: usize = 4;
const MAX_FORECAST_REQUIREMENTS: usize = 4;
const MAX_POSTED_OFFERS: usize = 4;
const MAX_SUBSTITUTE_PRODUCERS: usize = 4;
const SCORE_SCALE: u64 = 1000;

/// Survival, impairment, deprivation, terminal coverage, then work; lower wins.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    pub terminal_months: u64,
    pub impaired_months: u64,
    pub deprivation: u64,
    pub buffer_gap: u64,
    pub work: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Forecast {
    pub plan: CandidatePlan,
    pub score: Score,
    pub outcomes: Vec<MonthReport>,
    pub first_work: Vec<Receipt>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub search_name: String,
    pub search_budget: SearchBudget,
    pub search_budget_exhausted: bool,
    pub candidates_generated: usize,
    pub candidates_rejected: usize,
    pub month: u32,
    pub through: u32,
    pub alternatives: Vec<Forecast>,
    pub selected: usize,
}

pub fn validate(world: &World) -> Result<(), String> {
    if crate::search::substitute_producers(world).len() > MAX_SUBSTITUTE_PRODUCERS {
        return Err("at most four substitute producers are supported".into());
    }
    if world.decision_horizon == Some(0) {
        return Err("zero decision horizon".into());
    }
    if world.priority == Priority::ConsequenceAware
        && (world.participants.is_empty()
            || world.participants.len() > MAX_FORECAST_PARTICIPANTS
            || world
                .participants
                .iter()
                .any(|p| p.needs.len() > MAX_FORECAST_REQUIREMENTS)
            || world.offers.len()
                + world.access_offers.len()
                + world.bids.len()
                + world
                    .transaction_policy
                    .as_ref()
                    .map_or(0, |p| p.membership_offers.len())
                > MAX_POSTED_OFFERS)
    {
        return Err(
            "consequence forecast supports one to four participants, at most four requirements each and four offers".into(),
        );
    }
    Ok(())
}

pub(crate) fn choose(sim: &Simulation, batch: &mut Batch) -> Result<(), String> {
    let config = crate::search::configuration(&sim.world, &sim.state)?;
    match config.strategy {
        crate::search::SearchStrategy::NeedDirectedOpportunitySearch => choose_with_search(
            sim,
            batch,
            &crate::search::NeedDirectedOpportunitySearch,
            config.budget,
        ),
        crate::search::SearchStrategy::ExistingCommitmentsOnly => choose_with_search(
            sim,
            batch,
            &crate::search::ExistingCommitmentsOnly,
            config.budget,
        ),
    }
}

/// Injection point for experimental strategies without changing simulation execution.
/// Only the proposed batch is returned; the caller still commits through settlement.
pub fn choose_with_search(
    sim: &Simulation,
    batch: &mut Batch,
    search: &dyn OpportunitySearch,
    budget: SearchBudget,
) -> Result<(), String> {
    validate(&sim.world)?;
    let context = SearchContext::new(&sim.world, &sim.state);
    let result = search.search(&context, budget)?;
    evaluate_candidates(sim, batch, search.name(), budget, result)
}

/// Expand typed proposals at the current boundary. No proposal carries raw effects.
/// Unsupported bundle shapes are rejected instead of silently dropping steps.
fn acquisition(sim: &Simulation, base: &Batch, plan: &CandidatePlan) -> Result<Batch, String> {
    if plan.work.priority == Priority::ConsequenceAware {
        return Err("recursive forecast policy in candidate".into());
    }
    if plan.work.preferred_process.is_some_and(|id| {
        !sim.world
            .definitions
            .iter()
            .any(|d| d.id == id && d.enabled && d.execution == Execution::Productive)
    }) {
        return Err("unknown preferred process".into());
    }
    if !plan.steps.is_empty() && sim.state.phase != Phase::Acquire {
        return Err("acquisition proposal outside Acquire".into());
    }
    let mut batch = base.clone();
    let mut preview = sim.state.clone();
    for step in &plan.steps {
        match *step {
            PlanStep::AcceptMembership { offer, agent } => {
                if batch.accept_membership.is_some() {
                    return Err("multiple membership steps unsupported".into());
                }
                let m = crate::membership::acceptance(&sim.world, &preview, offer, agent)?;
                preview
                    .memberships
                    .insert((m.member, m.organization, m.role), m);
                batch.accept_membership = Some((offer, agent));
            }
            PlanStep::AcceptLand { offer, agent } => {
                if batch.accept_access.is_some() {
                    return Err("multiple land steps unsupported".into());
                }
                let a = crate::commitments::acceptance(&sim.world, &preview, offer)?;
                if a.debtor != agent {
                    return Err("land applicant differs from offer".into());
                }
                preview.accepted_agreements.insert(offer, a);
                batch.accept_access = Some(offer);
            }
            PlanStep::BuyEquipment { offer, agent } => {
                batch.transactions.push(crate::equipment::transaction(
                    &sim.world,
                    &sim.state,
                    crate::equipment::Trade {
                        offer,
                        buyer: agent,
                    },
                )?)
            }
            PlanStep::SellStock { bid, agent } => {
                batch.transactions.push(crate::currency::transaction(
                    &sim.world,
                    &sim.state,
                    crate::currency::StockTrade { bid, seller: agent },
                )?)
            }
        }
    }
    Ok(batch)
}

pub fn evaluate_candidates(
    sim: &Simulation,
    batch: &mut Batch,
    search_name: &str,
    budget: SearchBudget,
    result: SearchResult,
) -> Result<(), String> {
    validate(&sim.world)?;
    if result.candidates.len() > budget.max_candidates {
        return Err("search exceeded candidate budget".into());
    }
    if sim
        .world
        .participants
        .iter()
        .all(|p| sim.state.terminal.contains_key(&p.agent))
    {
        return if sim.state.phase == Phase::Acquire {
            Ok(())
        } else {
            sim.productive_with(batch, true)
        };
    }
    let mut horizon = sim.world.decision_horizon.unwrap_or(sim.world.horizon);
    if sim.world.transaction_policy.is_some() && !sim.world.access_offers.is_empty() {
        horizon = horizon.max(crate::commitments::MONTHS_PER_YEAR + 1);
    }
    let end = sim
        .state
        .month
        .checked_add(horizon)
        .ok_or("forecast horizon overflow")?;
    let candidates_generated = result.candidates.len();
    let mut alternatives = Vec::new();
    let mut first_batches = Vec::new();
    for plan in result.candidates {
        let Ok(acquisition) = acquisition(sim, batch, &plan) else {
            continue;
        };
        if sim.state.phase == Phase::Acquire {
            let mut preview = sim.state.clone();
            if commit(
                &sim.world,
                &mut preview,
                &acquisition,
                Backend::Reference,
                sim.effect_limit,
            )
            .is_err()
            {
                continue;
            }
        }
        let mut forecast_world = sim.world.clone();
        forecast_world.priority = plan.work.priority;
        forecast_world.capacity_overrides.clear();
        forecast_world
            .scheduled_starts
            .retain(|s| s.month == sim.state.month);
        let mut forecast = Simulation::new(forecast_world, sim.state.clone(), Backend::Reference)?;
        forecast.effect_limit = sim.effect_limit;
        let mut first = acquisition;
        let mut work;
        if sim.state.phase == Phase::Acquire {
            commit(
                &forecast.world,
                &mut forecast.state,
                &first,
                Backend::Reference,
                forecast.effect_limit,
            )?;
            forecast.ledger.push(first.clone());
            work = Batch::empty(&forecast.state);
        } else {
            work = first.clone();
        }
        forecast.productive_choice(&mut work, plan.work.defer_new, plan.work.preferred_process)?;
        commit(
            &forecast.world,
            &mut forecast.state,
            &work,
            Backend::Reference,
            forecast.effect_limit,
        )?;
        forecast.ledger.push(work.clone());
        if sim.state.phase == Phase::Acquire {
            first.production_plan = Some(Box::new(work.clone()));
        } else {
            first = work.clone();
        }
        while forecast.state.month < end {
            forecast.step()?;
        }
        if sim.world.transaction_policy.is_some()
            && plan.access_offer().is_some()
            && forecast
                .reports
                .iter()
                .any(|r| r.obligations.values().any(|o| o.paid < o.owed))
        {
            continue;
        }
        alternatives.push(Forecast {
            plan,
            score: score(&forecast),
            outcomes: forecast.reports,
            first_work: work.receipts.clone(),
        });
        first_batches.push(first);
    }
    let selected = alternatives
        .iter()
        .enumerate()
        .min_by_key(|(i, f)| (f.score.clone(), *i))
        .map(|(i, _)| i)
        .ok_or("no feasible search candidates")?;
    *batch = first_batches.swap_remove(selected);
    batch.decision = Some(Decision {
        search_name: search_name.into(),
        search_budget: budget,
        search_budget_exhausted: result.budget_exhausted,
        candidates_generated,
        candidates_rejected: candidates_generated - alternatives.len(),
        month: sim.state.month,
        through: end - 1,
        alternatives,
        selected,
    });
    Ok(())
}

fn score(sim: &Simulation) -> Score {
    let mut score = Score::default();
    for row in &sim.reports {
        if row.terminal.is_some() {
            score.terminal_months += 1;
        }
        let mut impaired = false;
        for rule in &sim.world.condition_rules {
            if rule.subject != row.agent {
                continue;
            }
            let points = row
                .conditions
                .get(&rule.provision)
                .map(|c| c.deprivation)
                .unwrap_or(0);
            impaired |= points >= rule.impaired_at;
            score.deprivation += u64::from(points) * SCORE_SCALE / u64::from(rule.terminal_at);
        }
        score.impaired_months += u64::from(impaired);
    }
    // Compare capped end-of-horizon coverage in period-equivalents, not raw kg vs fuel.
    // This tie-break is deliberately secondary to all condition consequences.
    for participant in &sim.world.participants {
        let mut stocks = crate::substitution::stocks(&sim.state, participant.agent);
        let mut needs: Vec<_> = participant.needs.iter().collect();
        needs.sort_by_key(|n| (n.priority, n.resource));
        let mut deficits = std::collections::BTreeMap::<ResourceId, i128>::new();
        // Fulfillment expires monthly. A two-provision lot cannot feed two
        // separate months when only one provision is required each month.
        for _ in 0..sim.world.horizon {
            for need in &needs {
                if need.quantity == 0 {
                    continue;
                }
                let (_, unmet) = crate::substitution::allocate(
                    &sim.world,
                    need.resource,
                    i128::from(need.quantity),
                    &mut stocks,
                    &Default::default(),
                );
                *deficits.entry(need.resource).or_default() += unmet;
            }
        }
        for need in needs {
            if need.quantity == 0 {
                continue;
            }
            let target = i128::from(need.quantity) * i128::from(sim.world.horizon);
            score.buffer_gap +=
                (deficits[&need.resource] * i128::from(SCORE_SCALE) / target) as u64;
        }
    }
    score.work = sim
        .ledger
        .iter()
        .filter(|b| b.phase == Phase::Productive)
        .flat_map(|b| &b.receipts)
        .map(|r| r.completed as u64)
        .sum();
    score
}
