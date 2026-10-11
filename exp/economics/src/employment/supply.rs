//! Optional worker-side opportunity-cost protection for posted household jobs.
use super::{Boundary, Terms};
use crate::{compute::Backend, model::*, simulation::Simulation};
const MAX_HORIZON: u32 = 24;
const MAX_HOURS: i32 = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub horizon: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alternative {
    pub hours: i32,
    pub losses: Option<Vec<i128>>,
    pub failure: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub agreement: u32,
    pub month: u32,
    pub worker: AgentId,
    pub horizon: u32,
    pub maximum: i32,
    pub searched_maximum: i32,
    pub objectives: Vec<crate::agency::objectives::Objective>,
    /// Appended after the ordinary objective losses, one entry per frozen delivery.
    pub deliveries: Vec<super::performance::Delivery>,
    pub baseline: Vec<i128>,
    pub alternatives: Vec<Alternative>,
    /// Approved by the worker for this assessment; final delivery is in the receipt.
    pub selected: i32,
}
pub(super) fn validate(w: &World) -> Result<(), String> {
    if w.employment_supply.iter().any(|(a, p)| {
        !(1..=MAX_HORIZON).contains(&p.horizon) || !w.participants.iter().any(|x| x.agent == *a)
    }) {
        return Err("invalid worker supply policy".into());
    }
    Ok(())
}
pub(super) fn quantity(
    w: &World,
    s: &State,
    base: &Batch,
    prior: &Boundary,
    terms: &Terms,
    maximum: i32,
) -> Result<(i32, Option<Decision>), String> {
    let Some(policy) = w.employment_supply.get(&terms.worker) else {
        return Ok((maximum, None));
    };
    let (mut world, state) = crate::households::hiring::preview(w, s, base, prior)?;
    world.employment_supply.clear();
    let goals = crate::discovery::supply::objectives(&world, &state, terms.worker);
    let deliveries = super::performance::deliveries(&world, &state, terms.worker, policy.horizon);
    let project = |hours: i32| -> Result<Vec<i128>, String> {
        let (world, mut state) = crate::forecast::ForecastContext::new(&world, &state).into_parts();
        let held = state
            .balances
            .entry((terms.worker, terms.capacity.resource))
            .or_default();
        *held = held
            .checked_sub(hours)
            .filter(|n| *n >= 0)
            .ok_or("worker forecast overspending")?;
        // The opportunity-cost branch consumes hours, without inventing wages or
        // employer output. Actual work and payment still require normal settlement.
        let mut sim = Simulation::new(world, state, Backend::Reference)?;
        let mut delivery_losses = vec![None; deliveries.len()];
        for _ in 0..policy.horizon {
            let month = sim.state.month;
            sim.run_months(1)?;
            super::performance::observe(&deliveries, &sim.state, month, &mut delivery_losses);
        }
        let mut losses = crate::agency::objectives::measure(
            &sim.world,
            &sim.state,
            &sim.reports,
            terms.worker,
            &goals,
        )?;
        losses.extend(delivery_losses.into_iter().map(|v| v.unwrap_or(0)));
        Ok(losses)
    };
    let baseline = project(0)?;
    let mut decision = Decision {
        agreement: terms.id,
        month: s.month,
        worker: terms.worker,
        horizon: policy.horizon,
        maximum,
        searched_maximum: maximum.min(MAX_HOURS),
        objectives: goals.clone(),
        deliveries: deliveries.clone(),
        baseline,
        alternatives: vec![],
        selected: 0,
    };
    for hours in (1..=decision.searched_maximum).rev() {
        let result = project(hours);
        let acceptable = result
            .as_ref()
            .is_ok_and(|losses| losses.iter().zip(&decision.baseline).all(|(a, b)| a <= b));
        let (losses, failure) = match result {
            Ok(v) => (Some(v), None),
            Err(e) => (None, Some(e)),
        };
        decision.alternatives.push(Alternative {
            hours,
            losses,
            failure,
        });
        if acceptable {
            decision.selected = hours;
            break;
        }
    }
    Ok((decision.selected, Some(decision)))
}
