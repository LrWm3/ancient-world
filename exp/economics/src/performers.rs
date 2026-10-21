//! Named personal work behind an organization's productive stage. Purchased hours
//! remain the employer's capacity; their provenance never transfers competency.
use crate::{employment, model::*};
use std::collections::BTreeMap;

/// One accepted employment delivery used for one single-month stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Work {
    pub month: u32,
    pub agreement: u32,
    pub worker: AgentId,
    pub technique: Option<u32>,
    pub capacity: Amount,
}
pub type Key = (u32, u64, usize); // month, process, stage
pub type Reservations = BTreeMap<u32, i32>; // current-month agreement -> hours

pub fn resource(w: &World, definition: DefinitionId, stage: usize) -> Option<ResourceId> {
    w.activities.performers.get(&(definition, stage)).copied()
}
fn quantity(services: &[Amount], resource: ResourceId) -> Option<i32> {
    services
        .iter()
        .filter(|a| a.resource == resource)
        .try_fold(0_i32, |sum, a| sum.checked_add(a.quantity))
        .filter(|n| *n > 0)
}
fn personal(w: &World, s: &State, t: &employment::Terms, month: u32) -> bool {
    w.participants.iter().any(|p| p.agent == t.worker && p.capacity.resource == t.capacity.resource)
        // Reselling someone else's hours is not personal practice. This first
        // adapter rejects relayed labor rather than guessing its ultimate source.
        && !w.employment.iter().any(|incoming| incoming.employer == t.worker
            && incoming.capacity.resource == t.capacity.resource
            && s.employment.earned.contains_key(&(incoming.id, month)))
}
pub fn candidates(w: &World, s: &State, operator: AgentId, resource: ResourceId) -> Vec<u32> {
    let mut terms: Vec<_> = w
        .employment
        .iter()
        .filter(|t| {
            t.employer == operator
                && t.capacity.resource == resource
                && !s.terminal.contains_key(&t.worker)
                && personal(w, s, t, s.month)
                && s.employment.earned.contains_key(&(t.id, s.month))
        })
        .collect();
    // Explicit initial allocation policy: employment priority, then stable ID.
    terms.sort_by_key(|t| (t.rank, t.id));
    terms.into_iter().map(|t| t.id).collect()
}
pub fn reserved(s: &State) -> Reservations {
    let mut result = Reservations::new();
    for work in s
        .employment
        .work
        .values()
        .filter(|work| work.month == s.month)
    {
        *result.entry(work.agreement).or_default() += work.capacity.quantity;
    }
    result
}
pub fn bind(
    w: &World,
    s: &State,
    id: u32,
    services: &[Amount],
    technique: Option<u32>,
    used: &Reservations,
) -> Option<Work> {
    let t = w.employment.iter().find(|t| t.id == id)?;
    let hours = quantity(services, t.capacity.resource)?;
    let earned = s.employment.earned.get(&(id, s.month))?;
    if s.terminal.contains_key(&t.worker)
        || !personal(w, s, t, s.month)
        || i64::from(hours) + i64::from(*used.get(&id).unwrap_or(&0)) > i64::from(earned.delivered)
    {
        return None;
    }
    Some(Work {
        month: s.month,
        agreement: id,
        worker: t.worker,
        technique,
        capacity: Amount::new(t.capacity.resource, hours),
    })
}
/// Forecast helpers must observe actual delivered provenance too. The resolver
/// subsequently chooses a concrete worker/technique and reserves their hours.
pub fn eligible(w: &World, s: &State, t: &crate::equipment::Technique, operator: AgentId) -> bool {
    match resource(w, t.definition, t.stage) {
        None => crate::equipment::eligible(t, s, operator),
        Some(r) => candidates(w, s, operator, r).into_iter().any(|id| {
            bind(w, s, id, &t.services, Some(t.id), &reserved(s))
                .is_some_and(|p| crate::equipment::eligible(t, s, p.worker))
        }),
    }
}

/// Validate provenance before technique eligibility and practice attribution.
/// This runs on staging, so a forged or overspent binding rolls back the batch.
pub fn apply(
    w: &World,
    opening: &State,
    staged: &mut State,
    tx: &Transaction,
) -> Result<(), String> {
    let Some(change) = &tx.process else {
        return Ok(());
    };
    let stage = change.before.as_ref().map_or(0, |p| p.stage);
    let expected = resource(w, change.after.definition, stage);
    if change.after.status == Status::Aborted || expected.is_none() {
        return if change.performer.is_none() {
            Ok(())
        } else {
            Err("unexpected productive performer".into())
        };
    }
    let work = change
        .performer
        .as_ref()
        .ok_or("missing named productive performer")?;
    let t = w
        .employment
        .iter()
        .find(|t| t.id == work.agreement)
        .ok_or("unknown performer employment")?;
    let services = match &tx.technique_use {
        Some(u) => {
            &w.techniques
                .iter()
                .find(|t| t.id == u.technique)
                .ok_or("unknown performer technique")?
                .services
        }
        None => &w.definition(change.after.definition).stages[stage].monthly_services,
    };
    if opening.phase != Phase::Productive
        || t.employer != change.after.operator
        || expected != Some(t.capacity.resource)
        || bind(
            w,
            opening,
            t.id,
            services,
            tx.technique_use.as_ref().map(|u| u.technique),
            &reserved(staged),
        )
        .as_ref()
            != Some(work)
        || crate::pools::input_account(w, change.after.definition, t.employer, t.capacity.resource)
            != (t.employer, t.capacity.resource)
        || tx
            .effects
            .iter()
            .filter(|e| e.account == (t.employer, t.capacity.resource))
            .map(|e| i64::from(e.delta))
            .sum::<i64>()
            != -i64::from(work.capacity.quantity)
    {
        return Err("invalid or overspent productive performer binding".into());
    }
    if staged
        .employment
        .work
        .insert((opening.month, change.after.id, stage), work.clone())
        .is_some()
    {
        return Err("duplicate productive performer binding".into());
    }
    Ok(())
}
pub fn subject(change: &ProcessChange) -> AgentId {
    change
        .performer
        .as_ref()
        .map_or(change.after.operator, |p| p.worker)
}

/// Local forecast adapters currently discard other workers and their work history.
pub fn require_local_compatible(w: &World) -> Result<(), String> {
    if !w.activities.performers.is_empty() {
        return Err("named productive work requires full-world forecasts; person-local adapter is unsupported".into());
    }
    Ok(())
}

fn completion_month(w: &World, p: &ProcessInstance, stage: usize) -> Result<u32, String> {
    w.definition(p.definition).stages[..stage]
        .iter()
        .try_fold(p.start, |month, s| month.checked_add(s.months))
        .ok_or_else(|| "performed work date overflow".into())
}

pub fn validate(w: &World, s: &State) -> Result<(), String> {
    if w.competition.is_some()
        || w.production_market.is_some()
        || w.pool_market.is_some()
        || w.work_choice.is_some()
        || w.market
            .as_ref()
            .is_some_and(|m| m.cash.is_some() || m.plots.is_some())
    {
        require_local_compatible(w)?;
    }
    for (&(definition, stage), &r) in &w.activities.performers {
        let d = w
            .definitions
            .iter()
            .find(|d| d.id == definition)
            .ok_or("unknown performer process")?;
        let st = d.stages.get(stage).ok_or("unknown performer stage")?;
        if d.execution != Execution::Productive
            || st.months != 1
            || quantity(&st.monthly_services, r).is_none()
            || !w
                .resources
                .iter()
                .any(|v| v.id == r && v.kind == ResourceKind::Capacity)
            || st.entry_inputs.iter().any(|a| a.resource == r)
            || d.outputs.iter().any(|a| a.resource == r)
            || w.techniques
                .iter()
                .filter(|t| t.definition == definition && t.stage == stage)
                .any(|t| quantity(&t.services, r).is_none())
            || w.pool_inputs
                .iter()
                .any(|p| p.definition == definition && p.account.1 == r)
        {
            return Err(
                "named work requires a single-month stage with positive personal labor".into(),
            );
        }
    }
    for p in s.processes.values() {
        for &(definition, stage) in w.activities.performers.keys() {
            if definition == p.definition && (p.status == Status::Completed || p.stage > stage) {
                let month = completion_month(w, p, stage)?;
                if !s.employment.work.contains_key(&(month, p.id, stage)) {
                    return Err("completed named work lacks participation history".into());
                }
            }
        }
    }
    let mut totals = BTreeMap::<(u32, u32), i64>::new();
    for (&(month, process, stage), work) in &s.employment.work {
        let p = s
            .processes
            .get(&process)
            .ok_or("unknown performed process")?;
        let t = w
            .employment
            .iter()
            .find(|t| t.id == work.agreement)
            .ok_or("unknown performed employment")?;
        let earned = s
            .employment
            .earned
            .get(&(t.id, month))
            .ok_or("performed work without earned delivery")?;
        if (month == s.month
            && matches!(
                s.phase,
                Phase::Open | Phase::Acquire | Phase::Due | Phase::Productive
            ))
            || work.month != month
            || month > s.month
            || work.worker != t.worker
            || t.employer != p.operator
            || work.capacity.quantity <= 0
            || work.capacity.resource != t.capacity.resource
            || resource(w, p.definition, stage) != Some(t.capacity.resource)
            || month != completion_month(w, p, stage)?
            || (p.stage <= stage && p.status != Status::Completed)
        {
            return Err("invalid performed work history".into());
        }
        let services = if let Some(id) = work.technique {
            let technique = w
                .techniques
                .iter()
                .find(|v| v.id == id && v.definition == p.definition && v.stage == stage)
                .ok_or("unknown performed technique")?;
            &technique.services
        } else {
            &w.definition(p.definition).stages[stage].monthly_services
        };
        if quantity(services, t.capacity.resource) != Some(work.capacity.quantity)
            || !personal(w, s, t, month)
        {
            return Err("performed work differs from technique or personal source".into());
        }
        let total = totals.entry((t.id, month)).or_default();
        *total += i64::from(work.capacity.quantity);
        if *total > i64::from(earned.delivered) {
            return Err("performed hours exceed delivery".into());
        }
    }
    Ok(())
}
