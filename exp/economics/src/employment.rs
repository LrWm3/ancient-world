//! Accepted capacity-for-wage agreements. Acquire delivers current hours; Close
//! collects earned claims. Cash shortage never undoes delivered work.
use crate::{
    agreements, finance,
    model::*,
    opportunities::{self, Action},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrearsPolicy {
    Continue,
    SuspendDelivery,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terms {
    pub id: u32,
    pub employer: AgentId,
    pub worker: AgentId,
    pub from: u32,
    pub through: u32,
    pub capacity: Amount,
    /// Integer denomination units per delivered capacity unit.
    pub wage_per_unit: Amount,
    pub on_arrears: ArrearsPolicy,
    /// Lower rank first, then oldest earning month and stable agreement ID.
    pub rank: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Earned {
    pub delivered: i32,
    pub claim: finance::Obligation,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Book {
    pub earned: BTreeMap<(u32, u32), Earned>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    Delivered,
    Collection,
    Arrears,
    Inactive,
    NotPermitted,
    Unavailable,
    HiringBudget,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub agreement: u32,
    pub earned_month: u32,
    pub requested: i32,
    pub delivered: i32,
    pub earned: i32,
    pub paid: i32,
    pub reason: Reason,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boundary {
    pub after: Book,
    pub receipts: Vec<Receipt>,
    pub transactions: Vec<Transaction>,
}
fn transaction(effects: Vec<Effect>) -> Transaction {
    Transaction {
        cause: "employment settlement".into(),
        effects,
        process: None,
        forward: None,
        delivery: None,
        royalty: None,
        technique_use: None,
        trade: None,
        stock_trade: None,
    }
}
pub fn validate(w: &World, s: &State) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let kind = |id| w.resources.iter().find(|r| r.id == id).map(|r| r.kind);
    for t in &w.employment {
        if w.households.iter().any(|h| {
            h.agent == t.worker || crate::households::membership::ever_member(h, t.employer)
        }) {
            return Err(
                "household workers and member employers need a paid-capacity adapter".into(),
            );
        }
        if let Some(h) = w.households.iter().find(|h| h.agent == t.employer) {
            if h.governance
                .charter
                .hiring_budget
                .as_ref()
                .is_none_or(|b| b.resource != t.wage_per_unit.resource)
                || crate::households::membership::ever_member(h, t.worker)
            {
                return Err(
                    "household employment requires a matching budget and outside worker".into(),
                );
            }
        }
        if !ids.insert(t.id)
            || t.worker == t.employer
            || t.from == 0
            || t.through < t.from
            || t.capacity.quantity <= 0
            || t.wage_per_unit.quantity <= 0
            || kind(t.capacity.resource) != Some(ResourceKind::Capacity)
            || kind(t.wage_per_unit.resource) != Some(ResourceKind::Stock)
            || ![t.worker, t.employer]
                .iter()
                .all(|id| w.agents.iter().any(|a| a.id == *id))
            || t.capacity
                .quantity
                .checked_mul(t.wage_per_unit.quantity)
                .is_none()
        {
            return Err("invalid employment terms".into());
        }
    }
    for (&(id, month), e) in &s.employment.earned {
        let t = w
            .employment
            .iter()
            .find(|t| t.id == id)
            .ok_or("unknown wage agreement")?;
        if month < t.from
            || month > t.through
            || month > s.month
            || e.delivered <= 0
            || e.delivered > t.capacity.quantity
            || e.claim
                != (finance::Obligation {
                    transfer: finance::Transfer {
                        from: t.employer,
                        to: t.worker,
                        amount: Amount::new(
                            t.wage_per_unit.resource,
                            e.delivered
                                .checked_mul(t.wage_per_unit.quantity)
                                .ok_or("wage overflow")?,
                        ),
                    },
                    settled: e.claim.settled,
                    condition: finance::Condition::OnOrAfterMonth(
                        month.checked_add(1).ok_or("wage due date overflow")?,
                    ),
                    failure: finance::FailureRule::CarryArrears,
                })
            || e.claim.settled < 0
            || e.claim.settled > e.claim.transfer.amount.quantity
        {
            return Err("invalid earned wage claim".into());
        }
    }
    Ok(())
}
pub(crate) fn outstanding(
    state: &State,
    employer: AgentId,
    resource: ResourceId,
) -> Result<i128, String> {
    state
        .employment
        .earned
        .values()
        .filter(|e| {
            e.claim.transfer.from == employer && e.claim.transfer.amount.resource == resource
        })
        .try_fold(0_i128, |sum, e| {
            sum.checked_add(i128::from(e.claim.outstanding()))
                .ok_or_else(|| "wage claim overflow".into())
        })
}

/// Existing boundary commitments reserve first; incoming goods/cash/hours are
/// not spendable again in this boundary. Employment delivery is divisible.
pub(crate) fn evaluate(w: &World, s: &State, base: &Batch) -> Result<Option<Boundary>, String> {
    if w.employment.is_empty() || !matches!(s.phase, Phase::Acquire | Phase::Close) {
        return Ok(None);
    }
    validate(w, s)?;
    let mut resources = crate::acquisition::Resources::opening(w, s);
    resources.reserve(w, &base.transactions)?;
    // A dated productive grant takes precedence over a later employment request.
    if let Some(plan) = &base.production_plan {
        for effect in plan.transactions.iter().flat_map(|t| &t.effects) {
            if effect.delta < 0
                && w.resources
                    .iter()
                    .any(|r| r.id == effect.account.1 && r.kind == ResourceKind::Capacity)
            {
                let available = resources.available.entry(effect.account).or_default();
                *available = available.saturating_add(effect.delta).max(0);
            }
        }
    }
    if s.phase == Phase::Acquire {
        for p in &w.participants {
            let reserved = crate::households::labor_reserve(w, s, p.agent, p.capacity.resource);
            let available = resources
                .available
                .entry((p.agent, p.capacity.resource))
                .or_default();
            *available = available.saturating_sub(reserved).max(0);
        }
    }
    let mut pooling =
        crate::households::income_reservations::Reservations::new(w, s, resources.storage.clone());
    let mut execution = finance::Execution::from_parts(resources.available, resources.storage);
    let mut b = Boundary {
        after: s.employment.clone(),
        receipts: vec![],
        transactions: vec![],
    };
    if s.phase == Phase::Acquire {
        let mut terms: Vec<_> = w.employment.iter().collect();
        terms.sort_by_key(|t| (t.rank, t.id));
        let mut hiring = BTreeMap::<AgentId, i32>::new();
        for t in terms {
            if s.month < t.from || s.month > t.through {
                continue;
            }
            if b.after.earned.contains_key(&(t.id, s.month)) {
                return Err("duplicate wage earning boundary".into());
            }
            let mut reason = if [t.worker, t.employer]
                .iter()
                .any(|a| s.terminal.contains_key(a) || !crate::households::market::active(w, s, *a))
            {
                Reason::Inactive
            } else if ![t.worker, t.employer]
                .iter()
                .all(|a| opportunities::permits(w, s, *a, Action::CapacityTrade))
            {
                Reason::NotPermitted
            } else if t.on_arrears == ArrearsPolicy::SuspendDelivery
                && s.employment.earned.iter().any(|((id, month), e)| {
                    *id == t.id && *month < s.month && e.claim.outstanding() > 0
                })
            {
                Reason::Arrears
            } else {
                Reason::Delivered
            };
            let mut delivered = if reason == Reason::Delivered {
                execution
                    .available
                    .get(&(t.worker, t.capacity.resource))
                    .copied()
                    .unwrap_or(0)
                    .max(0)
                    .min(t.capacity.quantity)
            } else {
                0
            };
            if let Some(h) = w.households.iter().find(|h| h.agent == t.employer) {
                let budget = h.governance.charter.hiring_budget.as_ref().unwrap();
                let spent = hiring.entry(t.employer).or_default();
                let arrears = outstanding(s, t.employer, budget.resource)?;
                let liquid = execution
                    .available
                    .get(&(t.employer, budget.resource))
                    .copied()
                    .unwrap_or(0);
                let allowance = (budget.quantity - *spent).max(0).min(
                    (i128::from(liquid) - arrears - i128::from(*spent))
                        .max(0)
                        .min(i128::from(i32::MAX)) as i32,
                );
                let bounded = delivered.min(allowance / t.wage_per_unit.quantity);
                if bounded < delivered {
                    reason = Reason::HiringBudget;
                }
                delivered = bounded;
                *spent = spent
                    .checked_add(delivered * t.wage_per_unit.quantity)
                    .ok_or("hiring budget overflow")?;
            }
            let wage = delivered
                .checked_mul(t.wage_per_unit.quantity)
                .ok_or("wage overflow")?;
            if delivered > 0 {
                let effects = execution.exchange(
                    w,
                    &[finance::Transfer {
                        from: t.worker,
                        to: t.employer,
                        amount: Amount::new(t.capacity.resource, delivered),
                    }],
                )?;
                b.transactions.push(transaction(effects));
                b.after.earned.insert(
                    (t.id, s.month),
                    Earned {
                        delivered,
                        claim: finance::Obligation {
                            transfer: finance::Transfer {
                                from: t.employer,
                                to: t.worker,
                                amount: Amount::new(t.wage_per_unit.resource, wage),
                            },
                            settled: 0,
                            condition: finance::Condition::OnOrAfterMonth(
                                s.month.checked_add(1).ok_or("wage due date overflow")?,
                            ),
                            failure: finance::FailureRule::CarryArrears,
                        },
                    },
                );
            }
            b.receipts.push(Receipt {
                agreement: t.id,
                earned_month: s.month,
                requested: t.capacity.quantity,
                delivered,
                earned: wage,
                paid: 0,
                reason: if reason == Reason::Delivered && delivered == 0 {
                    Reason::Unavailable
                } else {
                    reason
                },
            });
        }
    } else {
        let mut keys: Vec<_> = b.after.earned.keys().copied().collect();
        keys.sort_by_key(|(id, month)| {
            (
                w.employment.iter().find(|t| t.id == *id).unwrap().rank,
                *month,
                *id,
            )
        });
        for key in keys {
            let e = b.after.earned.get_mut(&key).unwrap();
            if e.claim.outstanding() == 0 {
                continue;
            }
            let maximum = pooling.payment_limit(w, &execution, &e.claim)?;
            let p = execution.pay_bounded(
                w,
                s.month.checked_add(1).ok_or("wage due date overflow")?,
                true,
                &e.claim,
                maximum,
            )?;
            pooling = pooling
                .preview(w, &p.effects)?
                .ok_or("wage pooling reservation changed")?;
            e.claim.settled = e
                .claim
                .settled
                .checked_add(p.paid)
                .ok_or("wage payment overflow")?;
            if !p.effects.is_empty() {
                b.transactions.push(transaction(p.effects));
            }
            b.receipts.push(Receipt {
                agreement: key.0,
                earned_month: key.1,
                requested: p.requested,
                delivered: 0,
                earned: 0,
                paid: p.paid,
                reason: Reason::Collection,
            });
        }
    }
    Ok(Some(b))
}

pub fn contract(t: &Terms, book: &Book) -> agreements::Agreement {
    let grant = agreements::Grant::Employment(t.id);
    agreements::Agreement {
        identity: agreements::Identity::Employment(t.id),
        grantor: agreements::Counterparty::Agent(t.worker),
        holder: t.employer,
        accepted_month: t.from,
        through: Some(t.through),
        grants: vec![grant.clone()],
        payments: vec![],
        production: None,
        obligations: book
            .earned
            .iter()
            .filter(|((id, _), _)| *id == t.id)
            .map(|(_, e)| agreements::Obligation {
                claim: e.claim.clone(),
                on_unpaid: match t.on_arrears {
                    ArrearsPolicy::SuspendDelivery => {
                        agreements::Consequence::SuspendNewUse(grant.clone())
                    }
                    ArrearsPolicy::Continue => agreements::Consequence::CarryArrears,
                },
            })
            .collect(),
    }
}
