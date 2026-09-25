//! Private same-month settlement previews for contributed-labor comparisons.
use super::*;

/// Units are combined only within the same priority and fulfillment resource.
/// Lower-priority-number needs compare first; resource ID breaks equal ranks.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Deficit {
    pub priority: u32,
    pub resource: ResourceId,
    pub unmet: i64,
}

pub(super) fn project(
    world: &World,
    opening: &State,
    productive: &Batch,
    people: &BTreeSet<AgentId>,
) -> Result<Vec<Deficit>, String> {
    let mut state = opening.clone();
    crate::settlement::commit_core(
        world,
        &mut state,
        productive,
        Backend::Reference,
        crate::settlement::DEFAULT_EFFECT_LIMIT,
    )?;
    let (effects, remainders) = collect(world, opening, &state, productive)?;
    apply(world, &mut state, &effects, Backend::Reference)?;
    state.household_remainders = remainders;
    let mut sim = Simulation {
        world: world.clone(),
        state,
        ledger: vec![],
        reports: vec![],
        backend: Backend::Reference,
        effect_limit: crate::settlement::DEFAULT_EFFECT_LIMIT,
    };
    // Use actual pooling, arrears and consumption. Never re-enter Productive or
    // advance into another month (which would recursively allocate household work).
    while matches!(sim.state.phase, Phase::ClearArrears | Phase::Consumption) {
        sim.step()?;
    }
    if sim.state.phase != Phase::Close {
        return Err("household need preview did not reach the consumption boundary".into());
    }
    let mut deficits = BTreeMap::<(u32, ResourceId), i64>::new();
    for p in world
        .participants
        .iter()
        .filter(|p| people.contains(&p.agent))
    {
        for need in &p.needs {
            *deficits.entry((need.priority, need.resource)).or_default() +=
                i64::from((need.quantity - sim.state.balance(p.agent, need.resource)).max(0));
        }
    }
    Ok(deficits
        .into_iter()
        .map(|((priority, resource), unmet)| Deficit {
            priority,
            resource,
            unmet,
        })
        .collect())
}
