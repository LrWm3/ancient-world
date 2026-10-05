//! Reusable, unit-preserving loss observations for individual or collective goals.
use crate::model::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    Organization,
    Members,
    Agents(BTreeSet<AgentId>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Metric {
    Deaths,
    NeedDeficit(ResourceId),
    FailedProcesses,
    Reserve {
        resource: ResourceId,
        target: i32,
    },
    Debt(ResourceId),
    /// Same-denomination accepted claims lacking stock cover; forecasts are not assets.
    FundingGap {
        resource: ResourceId,
        months: u32,
    },
    MembershipShortfall(usize),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objective {
    pub scope: Scope,
    pub metric: Metric,
}

pub fn validate(w: &World, objectives: &[Objective]) -> Result<(), String> {
    for o in objectives {
        if let Scope::Agents(ids) = &o.scope
            && ids.iter().any(|id| !w.agents.iter().any(|a| a.id == *id))
        {
            return Err("objective references unknown agent".into());
        }
        let resource = match o.metric {
            Metric::NeedDeficit(r) => Some((r, ResourceKind::Fulfillment)),
            Metric::Reserve { resource, target } => {
                if target < 0 {
                    return Err("negative reserve target".into());
                }
                Some((resource, ResourceKind::Stock))
            }
            Metric::Debt(r) => Some((r, ResourceKind::Stock)),
            Metric::FundingGap { resource, months } => {
                if !(1..=crate::need_orders::MAX_RESERVE_MONTHS).contains(&months) {
                    return Err("invalid commitment coverage horizon".into());
                }
                Some((resource, ResourceKind::Stock))
            }
            _ => None,
        };
        if resource
            .is_some_and(|(id, kind)| !w.resources.iter().any(|r| r.id == id && r.kind == kind))
        {
            return Err("objective uses unknown resource or wrong units".into());
        }
    }
    Ok(())
}

pub fn members(w: &World, s: &State, organization: AgentId) -> BTreeSet<AgentId> {
    if let Some(h) = w.households.iter().find(|h| h.agent == organization) {
        crate::households::membership::roster_at(h, s.month)
            .into_iter()
            .collect()
    } else {
        s.memberships
            .values()
            .filter(|m| m.organization == organization && m.accepted_month <= s.month)
            .map(|m| m.member)
            .collect()
    }
}

/// Preserve opening beneficiaries: dying or leaving cannot improve a forecast by
/// removing the people whose needs and survival were being evaluated.
pub fn freeze(
    w: &World,
    s: &State,
    organization: AgentId,
    objectives: &[Objective],
) -> Vec<Objective> {
    objectives
        .iter()
        .map(|o| Objective {
            scope: if o.scope == Scope::Members
                && !matches!(o.metric, Metric::MembershipShortfall(_))
            {
                Scope::Agents(members(w, s, organization))
            } else {
                o.scope.clone()
            },
            metric: o.metric.clone(),
        })
        .collect()
}

pub fn measure(
    w: &World,
    s: &State,
    reports: &[MonthReport],
    organization: AgentId,
    objectives: &[Objective],
) -> Result<Vec<i128>, String> {
    objectives
        .iter()
        .map(|o| {
            let scope = match &o.scope {
                Scope::Organization => [organization].into(),
                Scope::Members => members(w, s, organization),
                Scope::Agents(ids) => ids.clone(),
            };
            Ok(match o.metric {
                Metric::Deaths => scope
                    .iter()
                    .filter(|id| s.terminal.contains_key(id))
                    .count() as i128,
                Metric::MembershipShortfall(minimum) => minimum.saturating_sub(
                    scope
                        .iter()
                        .filter(|id| !s.terminal.contains_key(id))
                        .count(),
                ) as i128,
                Metric::NeedDeficit(resource) if !reports.is_empty() => reports
                    .iter()
                    .filter(|r| scope.contains(&r.agent))
                    .map(|r| i128::from(r.deficit(resource)))
                    .sum(),
                Metric::NeedDeficit(resource) => {
                    w.participants
                        .iter()
                        .filter(|p| scope.contains(&p.agent) && !s.terminal.contains_key(&p.agent))
                        .flat_map(|p| {
                            p.needs
                                .iter()
                                .filter(move |n| n.resource == resource)
                                .map(move |n| {
                                    i128::from((n.quantity - s.balance(p.agent, resource)).max(0))
                                })
                        })
                        .sum::<i128>()
                        * i128::from(s.month > 1)
                }
                Metric::FailedProcesses => s
                    .processes
                    .values()
                    .filter(|p| scope.contains(&p.operator) && p.status == Status::Aborted)
                    .count() as i128,
                Metric::Reserve { resource, target } => scope
                    .iter()
                    .map(|id| i128::from((target - s.balance(*id, resource)).max(0)))
                    .sum(),
                Metric::Debt(resource) => s
                    .credit
                    .loans
                    .values()
                    .filter(|l| scope.contains(&l.debtor) && l.denomination == resource)
                    .try_fold(0_i128, |total, l| {
                        Ok::<_, String>(total + i128::from(l.debt()?))
                    })?,
                Metric::FundingGap { resource, months } => {
                    scope.iter().try_fold(0_i128, |total, id| {
                        let claims = crate::need_orders::claims(w, s, *id, months)?;
                        Ok::<_, String>(
                            total
                                + (claims.get(&resource).copied().unwrap_or(0)
                                    - i128::from(s.balance(*id, resource)))
                                .max(0),
                        )
                    })?
                }
            })
        })
        .collect()
}
