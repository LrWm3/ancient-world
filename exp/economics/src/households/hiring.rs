//! Bounded monthly acceptance of posted labor terms. Uses the same member work
//! allocator and objectives as execution; forecasts cannot recursively hire.
use super::*;
use crate::{employment, household_governance::Policy};

pub(crate) fn quantity(
    world: &World,
    opening: &State,
    base: &Batch,
    prior: &employment::Boundary,
    terms: &employment::Terms,
    maximum: i32,
) -> Result<i32, String> {
    let h = world
        .households
        .iter()
        .find(|h| h.agent == terms.employer)
        .unwrap();
    let mut w = world.clone();
    // A projection observes existing commitments, not hypothetical future hires.
    // Keeping offer identities and terms preserves validation of earned claims.
    for household in &mut w.households {
        if let Some(budget) = &mut household.governance.charter.hiring_budget {
            budget.quantity = 0;
        }
    }
    let mut state = opening.clone();
    for tx in base.transactions.iter().chain(&prior.transactions) {
        apply(&w, &mut state, &tx.effects, Backend::Reference)?;
        crate::exchange::record(&mut state, tx);
    }
    if let Some(c) = &base.credit {
        crate::credit::record(&mut state, c);
    }
    crate::town_market::record(&mut state, &base.town_market);
    state.employment = prior.after.clone();
    let (pooled, remainders) = collect(&w, opening, &state, base)?;
    apply(&w, &mut state, &pooled, Backend::Reference)?;
    state.household_remainders = remainders;
    state.phase = Phase::Productive;
    // Acquire is complete before any productive work. Only already accepted
    // acquisitions enter this hypothesis; no assumed fills or added resources.
    let project = |state: &State| -> Result<LaborDecision, String> {
        // Include normal input allocation and consented support before comparing
        // labor, exactly as the Productive boundary will. No second allocator.
        prepare(&w, state)?
            .1
            .labor
            .into_iter()
            .find(|d| d.household == h.agent)
            .ok_or("missing household hiring projection".into())
    };
    let baseline = project(&state)?;
    let trial = |hours: i32| -> Result<LaborDecision, String> {
        let mut candidate = state.clone();
        apply(
            &w,
            &mut candidate,
            &transfer(terms.worker, terms.employer, terms.capacity.resource, hours),
            Backend::Reference,
        )?;
        candidate.employment.earned.insert(
            (terms.id, state.month),
            employment::earning(terms, state.month, hours)?,
        );
        project(&candidate)
    };
    let bought = |d: &LaborDecision| {
        d.purchased
            .iter()
            .find(|p| p.resource == terms.capacity.resource)
            .map_or(0, |p| p.directed)
    };
    let candidate = trial(maximum)?;
    let useful = (bought(&candidate) - bought(&baseline)).max(0).min(maximum);
    if useful == 0 {
        return Ok(0);
    }
    // Indivisible processes may need several hours, but unused offer capacity
    // is never hired. Recheck the trimmed quantity against the identical baseline.
    let candidate = if useful == maximum {
        candidate
    } else {
        trial(useful)?
    };
    if bought(&candidate) - bought(&baseline) != useful {
        return Ok(0);
    }
    let wage = i64::from(useful) * i64::from(terms.wage_per_unit.quantity);
    let worthwhile = candidate.projected_needs < baseline.projected_needs
        || (candidate.projected_needs == baseline.projected_needs
            && match h.governance.policy(state.month) {
                Policy::NeedsThenCommitments { .. } => {
                    let comparison = funding::compare(
                        candidate.projected_funding.as_ref().unwrap(),
                        baseline.projected_funding.as_ref().unwrap(),
                    );
                    comparison.is_lt()
                        || (comparison.is_eq()
                            && candidate.projected_value - baseline.projected_value
                                > wage * INPUT_BENEFIT)
                }
                Policy::NeedsThenIncome => income::improves(
                    h,
                    baseline.projected_income.as_ref().unwrap(),
                    candidate.projected_income.as_ref().unwrap(),
                ),
                // Same documented net-output proxy as allocation, with the cash wage
                // valued at par. This is a policy score, not a promise of market revenue.
                _ => candidate.projected_value - baseline.projected_value > wage * INPUT_BENEFIT,
            });
    Ok(if worthwhile { useful } else { 0 })
}
