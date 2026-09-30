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
/// Accepted loan disposition; a partial secured reduction retains its lien.
/// This is provenance, not another balance; debt remains in the ordinary loan book.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoanWriteOff {
    pub terms: Terms,
    pub principal: i32,
    pub interest: i32,
    /// Post-disposition snapshot for validating later collections under the stay.
    pub remaining_principal: i32,
    pub remaining_interest: i32,
    pub interest_remainder: i64,
    /// A secured partial reduction retains this asset's lien until ordinary sale/settlement.
    pub retained_collateral: Option<AssetId>,
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
            || !matches!(
                t.contract,
                ContractId::Wages(_) | ContractId::Land(_) | ContractId::Loan(_)
            )
            || (matches!(t.contract, ContractId::Loan(_))
                && (t.expected_due != t.original_due
                    || !matches!(t.action, Action::WriteOff { .. })))
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
                Action::Extend { due } => due <= t.month,
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
                ContractId::Loan(id) => {
                    let loan = out.after.loans.get_mut(&id).ok_or("missing relief loan")?;
                    let Action::WriteOff { quantity } = t.action else {
                        return Err("loan relief requires a write-off".into());
                    };
                    // Same interest-first ordering as collection, but no payment
                    // or interest income: the reporting adapter records a loss.
                    let interest = quantity.min(loan.interest);
                    let principal = quantity - interest;
                    loan.apply_payment(quantity);
                    if loan.debt()? == 0 {
                        loan.status = credit::Status::Discharged;
                    }
                    let disposition = LoanWriteOff {
                        terms: t.clone(),
                        principal,
                        interest,
                        remaining_principal: loan.principal,
                        remaining_interest: loan.interest,
                        interest_remainder: loan.interest_remainder,
                        retained_collateral: loan.collateral.as_ref().map(|c| c.asset),
                    };
                    if let Some(asset) = disposition.retained_collateral {
                        recovery::refresh_lien_proceeds(w, s, out, t.proceeding, asset)?;
                    }
                    out.recovery.push(recovery::Receipt::WrittenOff {
                        proceeding: t.proceeding,
                        loan: id,
                        principal,
                        interest,
                    });
                    out.after
                        .recovery
                        .loan_writeoffs
                        .entry(id)
                        .or_default()
                        .push(disposition);
                }
                ContractId::Wages(id) => {
                    let book = out.employment.get_or_insert_with(|| s.employment.clone());
                    let earned = book.earned.get_mut(&(id, t.original_due - 1)).unwrap();
                    earned.claim = claim.clone();
                    earned.relief = history.clone();
                }
                ContractId::Land(id) => {
                    if out.commitments.is_none() {
                        out.commitments = Some(crate::commitments::Settlement {
                            collections: vec![],
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
        ContractId::Loan(id) => {
            let l = s.credit.loans.get(&id)?;
            if l.opened.checked_add(1)? != t.original_due
                || l.collateral.as_ref().is_some_and(|c| {
                    (!c.pledged && !s.credit.recovery.proceedings.get(&t.proceeding)
                        .is_some_and(|case| case.sold.contains(&c.asset) && case.secured.contains_key(&id)))
                        || c.settlement != credit::CollateralSettlement::AuthorizedLiquidation
                        || !matches!(t.action, Action::WriteOff { quantity } if quantity < t.expected_remaining)
                }) {
                return None;
            }
            let history = s
                .credit
                .recovery
                .loan_writeoffs
                .get(&id)
                .map(|records| {
                    records
                        .iter()
                        .map(|r| Applied {
                            terms: r.terms.clone(),
                            paid: 0,
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some((
                crate::finance::Obligation {
                    transfer: crate::finance::Transfer {
                        from: l.debtor,
                        to: l.creditor,
                        amount: Amount::new(l.denomination, l.debt().ok()?),
                    },
                    settled: 0,
                    condition: crate::finance::Condition::OnOrAfterMonth(t.original_due),
                    failure: crate::finance::FailureRule::CarryArrears,
                },
                history,
            ))
        }
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

/// Loan write-offs retain their exact accepted terms through checkpoint and
/// closure. They never change the original quantity or masquerade as repayment.
pub(crate) fn validate_loans(w: &World, s: &State) -> Result<(), String> {
    for (&id, records) in &s.credit.recovery.loan_writeoffs {
        let l = s.credit.loans.get(&id).ok_or("write-off without loan")?;
        if records.is_empty() {
            return Err("empty loan write-off history".into());
        }
        let mut previous_month = 0;
        let mut principal = 0_i64;
        let mut previous = None;
        for r in records {
            let t = &r.terms;
            let case = s
                .credit
                .recovery
                .proceedings
                .get(&t.proceeding)
                .ok_or("loan write-off without proceeding")?;
            if !w.recovery.claim_relief.contains(t)
                || t.contract != ContractId::Loan(id)
                || t.debtor != l.debtor
                || t.creditor != creditor_at_due(w, s, l, t.month)?
                || Some(t.original_due) != l.opened.checked_add(1)
                || t.month > s.month
                || t.month <= previous_month
                || t.month < case.opened
                || case.closed.is_some_and(|m| m < t.month)
                || r.retained_collateral != l.collateral.as_ref().map(|c| c.asset)
                || l.collateral.as_ref().is_some_and(|c| {
                    c.settlement != credit::CollateralSettlement::AuthorizedLiquidation
                        || (r.remaining_principal == 0 && r.remaining_interest == 0)
                })
                || r.principal < 0
                || r.interest < 0
                || r.remaining_principal < 0
                || r.remaining_interest < 0
                || !(0..credit::RATE_SCALE).contains(&r.interest_remainder)
                || (r.remaining_interest > 0 && r.principal != 0)
                || (r.remaining_principal == 0
                    && r.remaining_interest == 0
                    && r.interest_remainder != 0)
                || t.action
                    != (Action::WriteOff {
                        quantity: r
                            .principal
                            .checked_add(r.interest)
                            .ok_or("relief overflow")?,
                    })
                || i64::from(r.principal)
                    + i64::from(r.interest)
                    + i64::from(r.remaining_principal)
                    + i64::from(r.remaining_interest)
                    != i64::from(t.expected_remaining)
            {
                return Err("invalid accepted loan write-off history".into());
            }
            if let Some(prior) = previous {
                let (p, i) = remaining_bound(w, s, l, prior, t.month);
                if i128::from(r.principal) + i128::from(r.remaining_principal) > p
                    || i128::from(r.interest) + i128::from(r.remaining_interest) > i
                    || ((r.remaining_principal > 0 || r.remaining_interest > 0)
                        && r.interest_remainder != prior.interest_remainder)
                {
                    return Err(
                        "loan relief history exceeds remaining debt and intervening advances"
                            .into(),
                    );
                }
            }
            previous = Some(r);
            principal = principal
                .checked_add(i64::from(r.principal))
                .ok_or("loan relief overflow")?;
            previous_month = t.month;
        }
        let last = records.last().unwrap();
        let (p, i) = remaining_bound(w, s, l, last, s.month);
        let fully_disposed =
            p == 0 && i == 0 && last.remaining_principal == 0 && last.remaining_interest == 0;
        if principal + i64::from(l.principal) > i64::from(l.original_principal)
            || i128::from(l.principal) > p
            || i128::from(l.interest) > i
            || (l.debt()? > 0 && l.interest_remainder != last.interest_remainder)
            || (fully_disposed && l.status != credit::Status::Discharged)
            || (!fully_disposed
                && l.status == credit::Status::Discharged
                && !deficiency_discharged(w, s, l))
        {
            return Err(
                "loan disposition does not reconcile to remaining debt and later advances".into(),
            );
        }
    }
    // Legacy deficient closure can discharge only its own custody denomination.
    // Other-denomination discharge needs this explicit loan-specific acceptance.
    for l in s
        .credit
        .loans
        .values()
        .filter(|l| l.status == credit::Status::Discharged)
    {
        if !s.credit.recovery.loan_writeoffs.contains_key(&l.id) && !deficiency_discharged(w, s, l)
        {
            return Err("discharged loan without accepted disposition".into());
        }
    }
    Ok(())
}

fn deficiency_discharged(w: &World, s: &State, l: &credit::Loan) -> bool {
    w.recovery.proceedings.iter().any(|p| {
        p.debtor == l.debtor
            && p.denomination == l.denomination
            && p.discharge_deficiency
            && s.credit
                .recovery
                .proceedings
                .get(&p.id)
                .is_some_and(|c| c.stage == recovery::Stage::Closed)
    })
}

/// Upper bounds allow intervening actual collections. New recourse is dated;
/// ordinary loans cannot create fresh principal after acceptance. The authorized
/// proceeding freezes accrual; relief does not end that stay.
fn remaining_bound(
    w: &World,
    s: &State,
    l: &credit::Loan,
    r: &LoanWriteOff,
    month: u32,
) -> (i128, i128) {
    let advances = w
        .recovery
        .guarantees
        .iter()
        .find(|g| g.recourse == l.id)
        .map(|g| {
            s.credit
                .recovery
                .guarantee_advances
                .iter()
                .filter(|((id, m), _)| *id == g.id && *m > r.terms.month && *m <= month)
                .map(|(_, q)| i128::from(*q))
                .sum::<i128>()
        })
        .unwrap_or(0);
    let principal = i128::from(r.remaining_principal) + advances;
    (principal, i128::from(r.remaining_interest))
}

/// Disposition runs at Due; assignment runs later at Acquire. Same-month relief
/// therefore belongs to the seller, while later consent must name the new holder.
fn creditor_at_due(w: &World, s: &State, l: &credit::Loan, month: u32) -> Result<AgentId, String> {
    let Some(a) = s
        .credit
        .recovery
        .assignments
        .get(&l.id)
        .filter(|a| month <= a.month)
    else {
        return Ok(l.creditor);
    };
    let listing = w
        .recovery
        .receivable_listings
        .iter()
        .find(|listing| listing.id == a.listing && listing.loan == l.id)
        .ok_or("loan relief assignment has no listing")?;
    w.recovery
        .proceedings
        .iter()
        .find(|p| p.id == listing.proceeding)
        .map(|p| p.debtor)
        .ok_or_else(|| "loan relief assignment has no seller".into())
}
