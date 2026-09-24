//! Explicit, mutually accepted disposition of overdue prepaid deliveries.
//! Configuration supplies consent, not a heuristic that declares a shortage impossible.
use crate::{credit, forward, model::*, recovery};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Extend { due: u32 },
    WriteOff { quantity: i32 },
}
/// A dated acceptance by both original parties within an authorized proceeding.
/// Exact outstanding quantity/date prevent silently applying stale negotiations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terms {
    pub id: u32,
    pub proceeding: u32,
    pub contract: AssetId,
    pub debtor: AgentId,
    pub creditor: AgentId,
    pub month: u32,
    pub expected_due: u32,
    pub expected_remaining: i32,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    InactiveProceeding,
    MissingContract,
    TermsMismatch,
    AlreadyApplied,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub terms: Terms,
    pub delivered: i32,
}

pub fn validate_terms(world: &World) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut dates = BTreeSet::new();
    for t in &world.recovery.delivery_relief {
        let p = world
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == t.proceeding)
            .ok_or("delivery relief requires an authorized proceeding")?;
        if !ids.insert(t.id)
            || !dates.insert((t.contract, t.month))
            || p.debtor != t.debtor
            || !world.agents.iter().any(|a| a.id == t.creditor)
            || t.creditor == t.debtor
            || t.creditor == p.estate
            || t.month < p.opening_month
            || t.expected_due >= t.month
            || t.expected_remaining <= 0
            || match t.action {
                Action::Extend { due } => due <= t.month,
                Action::WriteOff { quantity } => quantity <= 0 || quantity > t.expected_remaining,
            }
        {
            return Err("invalid accepted delivery relief terms".into());
        }
    }
    Ok(())
}

pub fn validate_history(world: &World, state: &State, c: &forward::Contract) -> Result<(), String> {
    if c.delivered < 0 {
        return Err("negative actual delivery".into());
    }
    let mut due = c.due;
    let mut waived = 0_i32;
    let mut delivered = 0;
    let mut previous_month = c.issued;
    for r in &c.relief {
        let t = &r.terms;
        let case = state
            .credit
            .recovery
            .proceedings
            .get(&t.proceeding)
            .ok_or("delivery relief without proceeding")?;
        if !world.recovery.delivery_relief.contains(t)
            || t.contract != c.id
            || t.debtor != c.debtor
            || t.creditor != c.creditor
            || t.month <= previous_month
            || t.month > state.month
            || t.month < case.opened
            || case.closed.is_some_and(|closed| closed < t.month)
            || t.expected_due != due
            || due >= t.month
            || r.delivered < delivered
            || r.delivered > c.delivered
            || i64::from(t.expected_remaining)
                != i64::from(c.goods.quantity) - i64::from(r.delivered) - i64::from(waived)
        {
            return Err("invalid applied delivery relief history".into());
        }
        match t.action {
            Action::Extend { due: next } => due = next,
            Action::WriteOff { quantity } => {
                waived = waived
                    .checked_add(quantity)
                    .ok_or("delivery write-off overflow")?;
            }
        }
        delivered = r.delivered;
        previous_month = t.month;
    }
    if i64::from(c.delivered) + i64::from(waived) > i64::from(c.goods.quantity) {
        return Err("delivery plus write-off exceeds original obligation".into());
    }
    Ok(())
}

/// Runs once at Due, after ordinary collection and before estate closure. No cash,
/// stock, storage, issuance or actual-delivery counters change through relief.
pub(crate) fn apply(world: &World, state: &State, out: &mut credit::Boundary) {
    let mut terms: Vec<_> = world
        .recovery
        .delivery_relief
        .iter()
        .filter(|t| t.month == state.month)
        .collect();
    terms.sort_by_key(|t| t.id);
    for t in terms {
        let mut c = state.exchange.forwards.get(&t.contract).cloned();
        let active = out
            .after
            .recovery
            .proceedings
            .get(&t.proceeding)
            .is_some_and(|p| p.stage == recovery::Stage::Active);
        let rejection = if !active {
            Some(Rejection::InactiveProceeding)
        } else if let Some(c) = &c {
            if c.relief.iter().any(|r| r.terms.id == t.id) {
                Some(Rejection::AlreadyApplied)
            } else if c.debtor != t.debtor
                || c.creditor != t.creditor
                || c.effective_due() != t.expected_due
                || c.claim().outstanding() != t.expected_remaining
            {
                Some(Rejection::TermsMismatch)
            } else {
                None
            }
        } else {
            Some(Rejection::MissingContract)
        };
        let applied = rejection.is_none();
        if applied {
            let c = c.as_mut().expect("checked contract");
            c.relief.push(Applied {
                terms: t.clone(),
                delivered: c.delivered,
            });
            out.forward_changes.insert(c.id, c.clone());
        }
        out.recovery.push(recovery::Receipt::DeliveryRelief {
            proceeding: t.proceeding,
            terms: t.id,
            contract: t.contract,
            creditor: t.creditor,
            applied,
            rejection,
            due: c.as_ref().map_or(t.expected_due, |c| c.effective_due()),
            written_off: if applied {
                match t.action {
                    Action::WriteOff { quantity } => quantity,
                    _ => 0,
                }
            } else {
                0
            },
            remaining: c.as_ref().map_or(0, |c| c.claim().outstanding()),
        });
    }
}
