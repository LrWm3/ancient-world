//! Explicit accepted changes to dated claims. Relief is not actual payment.
pub use crate::delivery_relief::Rejection;
use crate::{credit, finance::ContractId, model::*, recovery};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Extend { due: u32 },
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
            || !matches!(t.contract, ContractId::Wages(_) | ContractId::Land(_))
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
                Action::Extend { due } => {
                    due <= t.month || !matches!(t.contract, ContractId::Wages(_))
                }
            }
        {
            return Err("invalid accepted claim relief terms".into());
        }
    }
    Ok(())
}
pub(crate) struct Adjustment {
    pub due: u32,
    pub written_off: i32,
}
/// Reconstruct outstanding entitlement without treating waivers as transfers.
pub(crate) fn validate_history(
    w: &World,
    s: &State,
    claim: &Claim,
    history: &[Applied],
) -> Result<Adjustment, String> {
    let mut waived = 0_i32;
    let mut due = claim.original_due;
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
            || t.expected_due != due
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
            Action::Extend { due: next } => due = next,
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
    Ok(Adjustment {
        due,
        written_off: waived,
    })
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
        let mut view = view(w, &current, t);
        let active = out
            .after
            .recovery
            .proceedings
            .get(&t.proceeding)
            .is_some_and(|c| c.stage == recovery::Stage::Active);
        let rejection = if !active {
            Some(Rejection::InactiveProceeding)
        } else if let Some((claim, history)) = &view {
            if history.iter().any(|r| r.terms.id == t.id) {
                Some(Rejection::AlreadyApplied)
            } else if claim.transfer.from != t.debtor
                || claim.transfer.to != t.creditor
                || claim.condition != crate::finance::Condition::OnOrAfterMonth(t.expected_due)
                || claim.outstanding() != t.expected_remaining
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
            let (claim, history) = view.as_mut().unwrap();
            history.push(Applied {
                terms: t.clone(),
                paid: claim.settled,
            });
            match t.action {
                Action::Extend { due } => {
                    claim.condition = crate::finance::Condition::OnOrAfterMonth(due);
                }
                Action::WriteOff { quantity } => {
                    claim.transfer.amount.quantity -= quantity;
                    written_off = Some(Amount::new(claim.transfer.amount.resource, quantity));
                }
            }
            match t.contract {
                ContractId::Wages(id) => {
                    let book = out.employment.get_or_insert_with(|| s.employment.clone());
                    let earned = book.earned.get_mut(&(id, t.original_due - 1)).unwrap();
                    earned.claim = claim.clone();
                    earned.relief = history.clone();
                }
                ContractId::Land(id) => {
                    if out.commitments.is_none() {
                        out.commitments = Some(crate::commitments::Settlement {
                            policy: w.payment_policy,
                            protected: crate::commitments::protected_stock(w, s)?,
                            obligations: crate::commitments::due_obligations(w, s)?,
                            transactions: vec![],
                        });
                    }
                    out.commitments
                        .as_mut()
                        .unwrap()
                        .obligations
                        .get_mut(&(id, t.original_due))
                        .unwrap()
                        .relief = history.clone();
                }
                _ => return Err("unsupported relief adapter".into()),
            }
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
fn view(w: &World, s: &State, t: &Terms) -> Option<(crate::finance::Obligation, Vec<Applied>)> {
    match t.contract {
        ContractId::Wages(id) => {
            let e = s.employment.earned.get(&(id, t.original_due - 1))?;
            Some((e.claim.clone(), e.relief.clone()))
        }
        ContractId::Land(id) => {
            let a = crate::commitments::active(w, s).find(|a| a.id == id)?;
            let o = s.obligations.get(&(id, t.original_due))?;
            Some((o.claim(a), o.relief.clone()))
        }
        _ => None,
    }
}
