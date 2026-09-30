//! Read-only repayment demand for already accepted loans, in native units.
use crate::{credit, model::*};
use std::collections::BTreeMap;

/// Dated demand under timely future installments and the current legal state.
/// This is not a funding prediction: no new loans, future guarantees, fixture
/// cashflows, collateral realization or hypothetical lifting of stays is assumed.
/// Each projected payment reduces only a private loan copy before next interest.
pub fn dues(
    world: &World,
    state: &State,
    debtor: AgentId,
    resource: ResourceId,
    months: u32,
) -> Result<BTreeMap<u32, i128>, String> {
    let end = state
        .month
        .checked_add(months)
        .ok_or("loan projection horizon overflow")?;
    let mut result = BTreeMap::new();
    if months == 0
        || !state
            .credit
            .loans
            .values()
            .any(|l| l.debtor == debtor && l.denomination == resource)
    {
        return Ok(result);
    }
    let mut boundary = state.clone();
    for original in state
        .credit
        .loans
        .values()
        .filter(|l| l.debtor == debtor && l.denomination == resource)
    {
        let mut loan = original.clone();
        for month in state.month..end {
            if matches!(
                loan.status,
                credit::Status::Repaid | credit::Status::Discharged | credit::Status::PendingSale
            ) {
                break;
            }
            if month <= loan.opened {
                continue;
            }
            boundary.month = month;
            let Some(request) = credit::current_claim(world, &boundary, &loan)? else {
                // A current stay is not forecast to disappear by itself.
                break;
            };
            if loan.status == credit::Status::Active && loan.last_accrued < month {
                credit::accrue(&mut loan, month)?;
            }
            let paid =
                request
                    .claim
                    .outstanding()
                    .saturating_sub(crate::recovery::current_recourse(
                        world,
                        &state.credit,
                        loan.id,
                        month,
                    ));
            if paid > 0 {
                *result.entry(month).or_default() += i128::from(paid);
                loan.apply_payment(paid);
            }
        }
    }
    Ok(result)
}
