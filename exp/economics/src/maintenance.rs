//! Generic requirement consequences, independent of agent type and decision policy.
use crate::model::*;
use std::collections::BTreeSet;

pub const FULL_CAPACITY_PERMILLE: u32 = 1000;

/// One monthly requirement drives one accumulating condition in this first version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionRule {
    pub subject: AgentId,
    pub provision: ResourceId,
    pub name: String,
    /// Deprivation points per unmet provision unit.
    pub shortfall_cost: u32,
    /// Points recovered in a fully supplied, positive-demand month (not per surplus unit).
    pub recovery: u32,
    pub impaired_at: u32,
    pub retained_capacity_permille: u32,
    pub affected_capacity: ResourceId,
    pub terminal_at: u32,
    pub terminal_state: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Condition {
    pub deprivation: u32,
    /// Consecutive closes with positive deprivation, including incomplete recovery.
    pub adverse_months: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provision {
    pub month: u32,
    pub subject: AgentId,
    pub resource: ResourceId,
    pub quantity: i32,
    pub source_process: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionChange {
    pub subject: AgentId,
    pub resource: ResourceId,
    pub required: i32,
    pub supplied: i32,
    pub before: Condition,
    pub after: Condition,
    pub capacity_permille: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalTransition {
    pub month: u32,
    pub subject: AgentId,
    pub reason: ResourceId,
    pub state: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MaintenanceSettlement {
    pub provisions: Vec<Provision>,
    pub changes: Vec<ConditionChange>,
    pub transitions: Vec<TerminalTransition>,
}

/// Pure dynamics, also usable later by a forecaster. No decision policy is implied.
pub fn advance(
    rule: &ConditionRule,
    before: &Condition,
    required: i32,
    supplied: i32,
) -> Result<Condition, String> {
    if required < 0 || supplied < 0 {
        return Err("negative requirement/provision".into());
    }
    let deficit = required.saturating_sub(supplied).max(0) as u32;
    let deprivation = if deficit > 0 {
        before
            .deprivation
            .checked_add(
                deficit
                    .checked_mul(rule.shortfall_cost)
                    .ok_or("deprivation overflow")?,
            )
            .ok_or("deprivation overflow")?
    } else if required > 0 {
        before.deprivation.saturating_sub(rule.recovery)
    } else {
        // Inactive demand neither causes damage nor magically repairs old damage.
        before.deprivation
    };
    Ok(Condition {
        deprivation,
        adverse_months: if deprivation == 0 {
            0
        } else {
            before
                .adverse_months
                .checked_add(1)
                .ok_or("condition duration overflow")?
        },
    })
}

pub fn capacity(
    world: &World,
    state: &State,
    subject: AgentId,
    resource: ResourceId,
    base: i32,
) -> i32 {
    if state.terminal.contains_key(&subject) {
        return 0;
    }
    let factor = world
        .condition_rules
        .iter()
        .filter(|r| r.subject == subject && r.affected_capacity == resource)
        .map(|r| {
            if state
                .conditions
                .get(&(subject, r.provision))
                .is_some_and(|c| c.deprivation >= r.impaired_at)
            {
                r.retained_capacity_permille
            } else {
                FULL_CAPACITY_PERMILLE
            }
        })
        .min()
        .unwrap_or(FULL_CAPACITY_PERMILLE);
    // Most restrictive modifier wins; integer capacity rounds down.
    (i64::from(base) * i64::from(factor) / i64::from(FULL_CAPACITY_PERMILLE)) as i32
}

pub fn evaluate(world: &World, state: &State) -> Result<MaintenanceSettlement, String> {
    if state.phase != Phase::Close {
        return Err("conditions require settled consumption".into());
    }
    let mut result = MaintenanceSettlement::default();
    // Receipts come from completed processes, never reservations or stored fuel/food.
    for p in state
        .processes
        .values()
        .filter(|p| p.status == Status::Completed && p.reserved_through == state.month)
    {
        let d = world.definition(p.definition);
        if d.execution == Execution::Consumption {
            for output in &d.outputs {
                result.provisions.push(Provision {
                    month: state.month,
                    subject: p.beneficiary,
                    resource: output.resource,
                    quantity: output.quantity,
                    source_process: p.id,
                });
            }
        }
    }
    let mut rules: Vec<_> = world.condition_rules.iter().collect();
    rules.sort_by_key(|r| (r.subject, r.provision));
    let mut terminal = BTreeSet::new();
    for rule in rules {
        if state.terminal.contains_key(&rule.subject) {
            continue;
        }
        let need = world
            .participants
            .iter()
            .find(|p| p.agent == rule.subject)
            .and_then(|p| p.needs.iter().find(|n| n.resource == rule.provision))
            .ok_or("condition rule has no requirement")?;
        let supplied = state.balance(rule.subject, rule.provision);
        let delivered: i64 = result
            .provisions
            .iter()
            .filter(|p| p.subject == rule.subject && p.resource == rule.provision)
            .map(|p| i64::from(p.quantity))
            .sum();
        if delivered != i64::from(supplied) {
            return Err("provision disagrees with completed processes".into());
        }
        let before = state
            .conditions
            .get(&(rule.subject, rule.provision))
            .cloned()
            .unwrap_or_default();
        let after = advance(rule, &before, need.quantity, supplied)?;
        let factor = if after.deprivation >= rule.impaired_at {
            rule.retained_capacity_permille
        } else {
            FULL_CAPACITY_PERMILLE
        };
        if after.deprivation >= rule.terminal_at && terminal.insert(rule.subject) {
            result.transitions.push(TerminalTransition {
                month: state.month,
                subject: rule.subject,
                reason: rule.provision,
                state: rule.terminal_state.clone(),
            });
        }
        result.changes.push(ConditionChange {
            subject: rule.subject,
            resource: rule.provision,
            required: need.quantity,
            supplied,
            before,
            after,
            capacity_permille: factor,
        });
    }
    Ok(result)
}

pub fn validate(world: &World, state: &State) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for r in &world.condition_rules {
        if !seen.insert((r.subject, r.provision))
            || r.shortfall_cost == 0
            || r.impaired_at == 0
            || r.terminal_at <= r.impaired_at
            || r.retained_capacity_permille > FULL_CAPACITY_PERMILLE
            || r.name.is_empty()
            || r.terminal_state.is_empty()
            || !world.participants.iter().any(|p| {
                p.agent == r.subject
                    && p.capacity.resource == r.affected_capacity
                    && p.needs.iter().any(|n| n.resource == r.provision)
            })
        {
            return Err("invalid or duplicate condition rule".into());
        }
    }
    for (key, c) in &state.conditions {
        if !seen.contains(key) || (c.deprivation == 0) != (c.adverse_months == 0) {
            return Err("invalid condition state".into());
        }
        let rule = world
            .condition_rules
            .iter()
            .find(|r| (r.subject, r.provision) == *key)
            .unwrap();
        if c.deprivation >= rule.terminal_at && !state.terminal.contains_key(&key.0) {
            return Err("terminal condition without lifecycle transition".into());
        }
    }
    for (&subject, transition) in &state.terminal {
        if subject != transition.subject
            || transition.month == 0
            || transition.month >= state.month
            || !world.condition_rules.iter().any(|r| {
                r.subject == subject
                    && r.provision == transition.reason
                    && r.terminal_state == transition.state
                    && state
                        .conditions
                        .get(&(subject, r.provision))
                        .is_some_and(|c| c.deprivation >= r.terminal_at)
            })
        {
            return Err("invalid terminal transition".into());
        }
    }
    Ok(())
}
