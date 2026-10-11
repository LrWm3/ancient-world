//! Founding input-stock preferences, using existing exposure objectives.
//! A buffer is not a promise of feasible production or a resource reservation.
use super::{
    REFINE, WORKSHOP,
    continuation::Case,
    financing::{self, Preferences, Stocks},
};
use crate::{
    agency::{
        Config,
        objectives::{Metric, Objective, Scope},
    },
    minting::COIN,
    model::*,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    Off,
    InputsFirst,
    CashFirst,
}

/// Changes only the initial workshop objectives. The town retains the supplied
/// preference. Recipe inputs supply targets; no inventory, finance, or powers
/// are granted. Both organization fallback and applicable personal goals change.
pub fn scenario(
    case: Case,
    enabled: bool,
    stocks: Stocks,
    preferences: Preferences,
    policy: Policy,
) -> Result<(World, State), String> {
    let (mut w, s) = financing::with_preferences(case, enabled, stocks, preferences)?;
    if policy != Policy::Off {
        let goals = recipe_inputs(w.definition(REFINE))?;
        apply(
            &mut w.agency.get_mut(&WORKSHOP).unwrap().config,
            &goals,
            policy,
        );
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}

// One complete recipe's entry inputs, not one input charge per elapsed month.
// Resource IDs provide explicit, stable lexicographic ordering for multiple inputs.
fn recipe_inputs(definition: &ProcessDefinition) -> Result<Vec<Objective>, String> {
    let mut amounts = BTreeMap::<ResourceId, i32>::new();
    for input in definition
        .stages
        .iter()
        .flat_map(|stage| &stage.entry_inputs)
    {
        let total = amounts.entry(input.resource).or_default();
        *total = total
            .checked_add(input.quantity)
            .ok_or("input buffer target overflow")?;
    }
    Ok(amounts
        .into_iter()
        .map(|(resource, target)| Objective {
            scope: Scope::Organization,
            metric: Metric::ReserveExposure { resource, target },
        })
        .collect())
}

fn apply(config: &mut Config, buffer: &[Objective], policy: Policy) {
    if policy == Policy::Off {
        return;
    }
    for goals in std::iter::once(&mut config.objectives).chain(config.preferences.values_mut()) {
        // Personal preferences without an organizational cash goal stay unchanged.
        let Some(index) = goals.iter().position(|g| {
            g.scope == Scope::Organization
                && matches!(
                    g.metric,
                    Metric::Reserve { resource: COIN, .. }
                        | Metric::ReserveExposure { resource: COIN, .. }
                )
        }) else {
            continue;
        };
        let index = index + usize::from(policy == Policy::CashFirst);
        goals.splice(index..index, buffer.iter().cloned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn targets_sum_stage_entry_inputs_once_and_priorities_are_explicit() {
        let (w, _) = financing::scenario(Case::Baseline).unwrap();
        let mut recipe = w.definition(REFINE).clone();
        recipe.stages[0].months = 3;
        recipe.stages.push(recipe.stages[0].clone());
        let buffer = recipe_inputs(&recipe).unwrap();
        assert_eq!(
            buffer,
            vec![Objective {
                scope: Scope::Organization,
                metric: Metric::ReserveExposure {
                    resource: crate::minting::ORE,
                    target: 4
                }
            }]
        );
        let original = w.agency[&WORKSHOP].config.clone();
        for policy in [Policy::Off, Policy::InputsFirst, Policy::CashFirst] {
            let mut config = original.clone();
            apply(&mut config, &buffer, policy);
            let mut expected = original.objectives.clone();
            if policy != Policy::Off {
                expected.insert(usize::from(policy == Policy::CashFirst), buffer[0].clone());
            }
            assert_eq!(config.objectives, expected);
        }
    }
    #[test]
    fn personal_goals_preserve_scope_and_unrelated_priorities() {
        let (w, _) = financing::scenario(Case::Baseline).unwrap();
        let buffer = recipe_inputs(w.definition(REFINE)).unwrap();
        let mut c = w.agency[&WORKSHOP].config.clone();
        let cash = c.objectives[0].clone();
        let need = Objective {
            scope: Scope::Members,
            metric: Metric::Deaths,
        };
        c.preferences.insert(7, vec![need.clone(), cash.clone()]);
        let mut personal_cash = cash.clone();
        personal_cash.scope = Scope::Agents([8].into());
        c.preferences.insert(8, vec![personal_cash.clone()]);
        for policy in [Policy::Off, Policy::InputsFirst, Policy::CashFirst] {
            let mut variant = c.clone();
            apply(&mut variant, &buffer, policy);
            let mut expected = vec![need.clone(), cash.clone()];
            if policy != Policy::Off {
                expected.insert(
                    1 + usize::from(policy == Policy::CashFirst),
                    buffer[0].clone(),
                );
            }
            assert_eq!(variant.preferences[&7], expected);
            assert_eq!(variant.preferences[&8], [personal_cash.clone()]);
            assert_eq!(
                variant.objectives.len(),
                c.objectives.len() + usize::from(policy != Policy::Off)
            );
        }
    }
}
