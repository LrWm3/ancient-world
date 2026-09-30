//! Acquisition-cost reporting for whole zero-interest receivables.
//! Claim principal stays authoritative in the loan book; this is a derived basis.
use crate::{
    accounting::{Account, Line},
    model::*,
    recovery::Receipt,
};
use std::collections::BTreeMap;

pub(crate) fn adjustment(
    world: &World,
    state: &State,
    id: u32,
    coin: ResourceId,
    exchange_values: &BTreeMap<ResourceId, i128>,
) -> Result<i128, String> {
    let Some(a) = state.credit.recovery.assignments.get(&id) else {
        return Ok(0);
    };
    if !world
        .recovery
        .receivable_price_floors
        .contains_key(&a.listing)
    {
        return Ok(0);
    }
    let loan = &state.credit.loans[&id];
    if loan.monthly_rate_bps != 0
        || loan.interest != 0
        || a.interest != 0
        || a.principal <= 0
        || loan.principal > a.principal
    {
        return Err("negotiated receivable basis requires a zero-interest claim".into());
    }
    // Floor remaining cost to reporting ticks. Full disposal releases all basis;
    // rounding never changes actual cash or the contractual principal.
    let remaining_cost = i128::from(a.price) * i128::from(loan.principal) / i128::from(a.principal);
    Ok(remaining_cost
        - crate::reporting_value::value(coin, exchange_values, loan.denomination, loan.principal)?)
}

pub(crate) fn settle(
    world: &World,
    before: &State,
    after: &State,
    batch: &Batch,
    coin: ResourceId,
    exchange_values: &BTreeMap<ResourceId, i128>,
) -> Result<Vec<Line>, String> {
    let mut lines = Vec::new();
    let mut add = |agent, account, debit| {
        if debit != 0 {
            lines.push(Line {
                agent,
                account,
                debit,
                flow: None,
            });
        }
    };
    for (&id, a) in &after.credit.recovery.assignments {
        if !world
            .recovery
            .receivable_price_floors
            .contains_key(&a.listing)
        {
            continue;
        }
        if !before.credit.recovery.assignments.contains_key(&id) {
            let seller = before.credit.loans[&id].creditor;
            let difference = crate::reporting_value::value(
                coin,
                exchange_values,
                before.credit.loans[&id].denomination,
                a.principal,
            )? - i128::from(a.price);
            add(
                seller,
                if difference > 0 {
                    Account::DisposalLoss
                } else {
                    Account::DisposalGain
                },
                difference,
            );
            continue;
        }
        let delta = adjustment(world, after, id, coin, exchange_values)?
            - adjustment(world, before, id, coin, exchange_values)?;
        if delta == 0 {
            continue;
        }
        let reduction = before.credit.loans[&id].principal - after.credit.loans[&id].principal;
        if reduction <= 0 {
            return Err("receivable basis changed without principal disposition".into());
        }
        let waived: i128 = batch
            .credit
            .iter()
            .flat_map(|c| &c.recovery)
            .filter_map(|r| {
                if let Receipt::WrittenOff {
                    loan, principal, ..
                } = r
                {
                    (*loan == id).then_some(i128::from(*principal))
                } else {
                    None
                }
            })
            .sum();
        if waived > i128::from(reduction) {
            return Err("receivable waiver exceeds disposition".into());
        }
        let waived_adjustment = delta * waived / i128::from(reduction);
        let creditor = after.credit.loans[&id].creditor;
        // Existing relief records the gross claim loss; release its corresponding
        // basis adjustment. Actual collections recognize the remaining difference.
        add(creditor, Account::CreditLoss, -waived_adjustment);
        let realized = delta - waived_adjustment;
        add(
            creditor,
            if realized > 0 {
                Account::SettlementGain
            } else {
                Account::SettlementLoss
            },
            -realized,
        );
    }
    Ok(lines)
}
