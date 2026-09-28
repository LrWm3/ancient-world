//! Recognition of founding household agreements. Admission is historical;
//! subsequent law changes do not silently annul existing organizations.
use crate::{
    household_governance::{Contribution, Leadership, Policy},
    households::Agreement,
    model::*,
};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub leadership: BTreeSet<Leadership>,
    pub policies: BTreeSet<Policy>,
    pub min_adults: usize,
    pub max_adults: usize,
    pub max_labor_percent: u32,
    pub min_term_months: u32,
    pub max_term_months: u32,
    /// A legal ceiling on the constitutional mandate. None leaves it unbounded.
    pub activities: Option<BTreeSet<DefinitionId>>,
}
impl Default for Rules {
    fn default() -> Self {
        Self {
            leadership: [
                Leadership::FixedFounder,
                Leadership::Rotating,
                Leadership::Elected,
            ]
            .into(),
            policies: [
                Policy::NetOutput,
                Policy::PreserveCommittedWork,
                Policy::NeedsFirst,
            ]
            .into(),
            min_adults: 1,
            max_adults: crate::households::FOUNDING_ADULT_LIMIT,
            max_labor_percent: crate::household_governance::PERCENT as u32,
            min_term_months: 1,
            max_term_months: u32::MAX,
            activities: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Admission {
    pub month: u32,
    pub authority: Option<AgentId>,
    pub founders: Vec<(AgentId, super::Decision)>,
    pub rules: Option<Rules>,
}

pub(super) fn validate_rules(w: &World, rules: &Rules) -> Result<(), String> {
    if rules.min_term_months == 0
        || rules.min_term_months > rules.max_term_months
        || rules.min_adults == 0
        || rules.min_adults > rules.max_adults
        || rules.max_adults > crate::households::FOUNDING_ADULT_LIMIT
        || rules.max_labor_percent > crate::household_governance::PERCENT as u32
        || rules.activities.as_ref().is_some_and(|ids| {
            ids.iter().any(|id| {
                !w.definitions
                    .iter()
                    .any(|d| d.id == *id && d.execution == Execution::Productive)
            })
        })
    {
        return Err("invalid household founding law".into());
    }
    Ok(())
}
fn check_terms(a: &Agreement, rules: &Rules) -> Result<(), String> {
    let g = &a.governance;
    if !(rules.min_term_months..=rules.max_term_months).contains(&g.charter.term_months)
        || !rules.leadership.contains(&g.constitution.leadership)
        || !g.constitution.permitted_policies.is_subset(&rules.policies)
        || !(rules.min_adults..=rules.max_adults).contains(&a.adults.len())
        || match g.charter.contribution {
            Contribution::Percent(p) => p > rules.max_labor_percent,
            Contribution::SpareLabor => {
                rules.max_labor_percent < crate::household_governance::PERCENT as u32
            }
        }
        || rules.activities.as_ref().is_some_and(|allowed| {
            g.constitution
                .activities
                .as_ref()
                .is_none_or(|ids| !ids.is_subset(allowed))
        })
    {
        return Err("household founding terms exceed recognized legal limits".into());
    }
    Ok(())
}

pub fn admit(w: &World, s: &State, a: &Agreement) -> Result<Admission, String> {
    let rules = w
        .transaction_policy
        .as_ref()
        .and_then(|p| p.agreement_limits.household.clone());
    if let Some(rules) = &rules {
        validate_rules(w, rules)?;
        check_terms(a, rules)?;
    }
    let mut founders: Vec<_> = a
        .adults
        .iter()
        .map(|&id| {
            (
                id,
                super::evaluate_agreement(w, s, id, super::AgreementForm::Household),
            )
        })
        .collect();
    founders.sort_by_key(|(id, _)| *id);
    if founders.iter().any(|(_, d)| !d.allowed) {
        return Err(
            "household founding denied by agreement recognition or founder permissions".into(),
        );
    }
    Ok(Admission {
        month: s.month,
        authority: w.transaction_policy.as_ref().map(|p| p.authority),
        founders,
        rules,
    })
}

pub fn validate_admission(w: &World, a: &Agreement) -> Result<(), String> {
    let receipt = a
        .admission
        .as_ref()
        .ok_or("missing household founding admission")?;
    let expected: BTreeSet<_> = a.adults.iter().copied().collect();
    let actual: BTreeSet<_> = receipt.founders.iter().map(|(id, _)| *id).collect();
    if receipt.month != a.formed
        || expected != actual
        || actual.len() != receipt.founders.len()
        || receipt
            .authority
            .is_some_and(|id| !w.agents.iter().any(|a| a.id == id))
        || receipt
            .founders
            .iter()
            .any(|(_, d)| !d.allowed || d.authority != receipt.authority)
    {
        return Err("invalid household founding admission".into());
    }
    if let Some(rules) = &receipt.rules {
        validate_rules(w, rules)?;
        check_terms(a, rules)?;
    }
    Ok(())
}
