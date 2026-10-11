//! Optional worker-side opportunity-cost protection for posted household jobs.
use super::{Boundary, Terms};
use crate::{compute::Backend, model::*, simulation::Simulation};
const MAX_HORIZON: u32 = 24;
const MAX_HOURS: i32 = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub horizon: u32,
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
) -> Result<i32, String> {
    let Some(policy) = w.employment_supply.get(&terms.worker) else {
        return Ok(maximum);
    };
    let (mut world, state) = crate::households::hiring::preview(w, s, base, prior)?;
    world.employment_supply.clear();
    let goals = crate::discovery::supply::objectives(&world, &state, terms.worker);
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
        sim.run_months(policy.horizon)?;
        crate::agency::objectives::measure(
            &sim.world,
            &sim.state,
            &sim.reports,
            terms.worker,
            &goals,
        )
    };
    let baseline = project(0)?;
    for hours in (1..=maximum.min(MAX_HOURS)).rev() {
        if project(hours).is_ok_and(|losses| losses.iter().zip(&baseline).all(|(a, b)| a <= b)) {
            return Ok(hours);
        }
    }
    Ok(0)
}
