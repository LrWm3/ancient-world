//! Explicit accepted changes to dated claims. Relief is not actual payment.
pub use crate::delivery_relief::Rejection;
use crate::{credit, finance::ContractId, model::*, recovery};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    WriteOff { quantity: i32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terms {
    pub id: u32,
    pub proceeding: u32,
    pub contract: ContractId,
    /// Original due date identifies the dated claim even after restructuring.
    pub original_due: u32,
    pub debtor: AgentId,
    pub creditor: AgentId,
    pub month: u32,
    pub expected_due: u32,
    pub expected_remaining: i32,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub terms: Terms,
    /// Actual units paid when the accepted relief was applied.
    pub paid: i32,
}
pub(crate) struct Claim {
    pub contract: ContractId,
    pub original_due: u32,
    pub debtor: AgentId,
    pub creditor: AgentId,
    pub quantity: i32,
    pub paid: i32,
}
pub fn validate_terms(w: &World) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut dates = BTreeSet::new();
    for t in &w.recovery.claim_relief {
        let p = w
            .recovery
            .proceedings
            .iter()
            .find(|p| p.id == t.proceeding)
            .ok_or("claim relief requires an authorized proceeding")?;
        if !ids.insert(t.id)
            || !dates.insert((t.contract, t.original_due, t.month))
            || !matches!(t.contract, ContractId::Wages(_))
            || t.original_due < 2
            || t.expected_due < t.original_due
            || t.expected_due >= t.month
            || t.month < p.opening_month
            || t.debtor != p.debtor
            || t.creditor == t.debtor
            || t.creditor == p.estate
            || !w.agents.iter().any(|a| a.id == t.creditor)
            || t.expected_remaining <= 0
            || match t.action {
                Action::WriteOff { quantity } => quantity <= 0 || quantity > t.expected_remaining,
            }
        {
            return Err("invalid accepted claim relief terms".into());
        }
    }
    Ok(())
}
/// Reconstruct outstanding entitlement without treating waivers as transfers.
pub(crate) fn validate_history(
    w: &World,
    s: &State,
    claim: &Claim,
    history: &[Applied],
) -> Result<i32, String> {
    let mut waived = 0_i32;
    let mut previous_month = 0;
    let mut paid = 0;
    for r in history {
        let t = &r.terms;
        let case = s
            .credit
            .recovery
            .proceedings
            .get(&t.proceeding)
            .ok_or("claim relief without proceeding")?;
        if !w.recovery.claim_relief.contains(t)
            || t.contract != claim.contract
            || t.original_due != claim.original_due
            || t.expected_due != claim.original_due
            || t.debtor != claim.debtor
            || t.creditor != claim.creditor
            || t.month <= previous_month
            || t.month > s.month
            || t.month < case.opened
            || case.closed.is_some_and(|m| m < t.month)
            || r.paid < paid
            || r.paid > claim.paid
            || i64::from(t.expected_remaining)
                != i64::from(claim.quantity) - i64::from(r.paid) - i64::from(waived)
        {
            return Err("invalid applied claim relief history".into());
        }
        match t.action {
            Action::WriteOff { quantity } => {
                waived = waived
                    .checked_add(quantity)
                    .ok_or("claim relief overflow")?;
            }
        }
        previous_month = t.month;
        paid = r.paid;
    }
    if i64::from(waived) + i64::from(claim.paid) > i64::from(claim.quantity) {
        return Err("payment plus relief exceeds original claim".into());
    }
    Ok(waived)
}

/// Due, after ordinary collections and before estate allocation. Both parties'
/// accepted terms must still match the current dated claim exactly.
pub(crate) fn apply(w: &World, s: &State, out: &mut credit::Boundary) -> Result<(), String> {
    let mut terms: Vec<_> = w
        .recovery
        .claim_relief
        .iter()
        .filter(|t| t.month == s.month)
        .collect();
    terms.sort_by_key(|t| t.id);
    for t in terms {
        let current = crate::recovery_claims::current(s, out);
        let ContractId::Wages(id) = t.contract else {
            return Err("unsupported relief adapter".into());
        };
        let key = (id, t.original_due - 1);
        let mut earned = current.employment.earned.get(&key).cloned();
        let active = out
            .after
            .recovery
            .proceedings
            .get(&t.proceeding)
            .is_some_and(|c| c.stage == recovery::Stage::Active);
        let rejection = if !active {
            Some(Rejection::InactiveProceeding)
        } else if let Some(e) = &earned {
            if e.relief.iter().any(|r| r.terms.id == t.id) {
                Some(Rejection::AlreadyApplied)
            } else if e.claim.transfer.from != t.debtor
                || e.claim.transfer.to != t.creditor
                || e.claim.condition != crate::finance::Condition::OnOrAfterMonth(t.expected_due)
                || e.claim.outstanding() != t.expected_remaining
            {
                Some(Rejection::TermsMismatch)
            } else {
                None
            }
        } else {
            Some(Rejection::MissingContract)
        };
        let mut written_off = None;
        if rejection.is_none() {
            let e = earned.as_mut().unwrap();
            e.relief.push(Applied {
                terms: t.clone(),
                paid: e.claim.settled,
            });
            match t.action {
                Action::WriteOff { quantity } => {
                    e.claim.transfer.amount.quantity -= quantity;
                    written_off = Some(Amount::new(e.claim.transfer.amount.resource, quantity));
                }
            }
            out.employment
                .get_or_insert_with(|| s.employment.clone())
                .earned
                .insert(key, e.clone());
        }
        out.recovery.push(recovery::Receipt::ClaimRelief {
            proceeding: t.proceeding,
            terms: t.id,
            contract: t.contract,
            creditor: t.creditor,
            rejection,
            written_off,
        });
    }
    Ok(())
}
