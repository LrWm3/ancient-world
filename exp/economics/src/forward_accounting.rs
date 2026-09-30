//! Historical advance cost, not a mark to projected/spot prices.
use crate::{
    accounting::{self, Account, Line},
    forward::Contract,
    inventory_accounting::PrepaidSale,
    model::*,
};
use std::collections::BTreeMap;

pub(crate) fn released(c: &Contract, quantity: i32) -> Result<i128, String> {
    if c.goods.quantity <= 0 || quantity < 0 || quantity > c.goods.quantity {
        return Err("invalid forward cost quantity".into());
    }
    i128::from(c.advance.quantity)
        .checked_mul(i128::from(quantity))
        .map(|v| v / i128::from(c.goods.quantity))
        .ok_or_else(|| "forward cost overflow".into())
}
pub(crate) fn positions(
    state: &State,
    coin: ResourceId,
) -> Result<BTreeMap<(AgentId, Account), i128>, String> {
    let mut result = BTreeMap::new();
    for c in state.exchange.forwards.values() {
        if c.advance.resource != coin || c.advance.quantity <= 0 || c.goods.resource == coin {
            return Err("forward requires reporting-coin advance and noncoin delivery".into());
        }
        let settled = c
            .delivered
            .checked_add(c.written_off())
            .ok_or("forward quantity overflow")?;
        let value = i128::from(c.advance.quantity) - released(c, settled)?;
        accounting::add(
            &mut result,
            (c.creditor, Account::ForwardPrepayment(c.id)),
            value,
        )?;
        accounting::add(
            &mut result,
            (c.debtor, Account::DeferredRevenue(c.id)),
            -value,
        )?;
    }
    Ok(result)
}
/// Called only after exact settlement replay. Changes to delivery/relief counters
/// determine recognition, while the ledger retains physical performance provenance.
pub(crate) fn settle(
    before: &State,
    after: &State,
    batch: &Batch,
    coin: ResourceId,
    values: &BTreeMap<ResourceId, i128>,
) -> Result<(Vec<PrepaidSale>, Vec<Line>), String> {
    let mut sales = Vec::new();
    let mut lines = Vec::new();
    for tx in &batch.transactions {
        if let Some(crate::forward::Event::Accepted(c)) = &tx.forward {
            for (agent, sign) in [(c.creditor, -1), (c.debtor, 1)] {
                lines.push(Line {
                    agent,
                    account: Account::Cash,
                    debit: i128::from(c.advance.quantity) * sign,
                    flow: Some(accounting::Flow::Operating),
                });
            }
        }
    }
    for (id, old) in &before.exchange.forwards {
        let new = after
            .exchange
            .forwards
            .get(id)
            .ok_or("forward removed without accounting disposition")?;
        let delivered = new
            .delivered
            .checked_sub(old.delivered)
            .ok_or("forward delivery overflow")?;
        let waived = new
            .written_off()
            .checked_sub(old.written_off())
            .ok_or("forward relief overflow")?;
        if delivered < 0 || waived < 0 {
            return Err("forward performance cannot be reversed".into());
        }
        let settled = old
            .delivered
            .checked_add(old.written_off())
            .ok_or("forward quantity overflow")?;
        let after_delivery = settled
            .checked_add(delivered)
            .ok_or("forward quantity overflow")?;
        let guaranteed =
            batch
                .credit
                .iter()
                .flat_map(|b| &b.recovery)
                .try_fold(0_i32, |total, r| {
                    if let crate::recovery::Receipt::Guaranteed {
                        claim: crate::recovery::GuaranteedClaim::Forward(claim),
                        paid,
                        ..
                    } = r
                        && claim == id
                    {
                        total
                            .checked_add(*paid)
                            .ok_or("guaranteed forward overflow")
                    } else {
                        Ok(total)
                    }
                })?;
        if guaranteed > delivered || (guaranteed > 0 && guaranteed != delivered) {
            return Err(
                "guaranteed and ordinary delivery require distinct dated boundaries".into(),
            );
        }
        if guaranteed > 0 {
            let value = released(old, after_delivery)? - released(old, settled)?;
            let cost = crate::reporting_value::value(coin, values, old.goods.resource, guaranteed)?;
            lines.extend([
                Line {
                    agent: old.debtor,
                    account: Account::Sales,
                    debit: -value,
                    flow: None,
                },
                Line {
                    agent: old.debtor,
                    account: Account::CostOfSales,
                    debit: cost,
                    flow: None,
                },
            ]);
        } else if delivered > 0 {
            sales.push(PrepaidSale {
                seller: old.debtor,
                buyer: old.creditor,
                resource: old.goods.resource,
                quantity: delivered,
                value: released(old, after_delivery)? - released(old, settled)?,
            });
        }
        if waived > 0 {
            // Existing coordinator settles physical deliveries before relief;
            // extensions release no cost and never fabricate goods.
            let total = after_delivery
                .checked_add(waived)
                .ok_or("forward quantity overflow")?;
            let loss = released(old, total)? - released(old, after_delivery)?;
            for (agent, account, debit) in [
                (old.creditor, Account::CreditLoss, loss),
                (old.debtor, Account::DebtRelief, -loss),
            ] {
                if debit != 0 {
                    lines.push(Line {
                        agent,
                        account,
                        debit,
                        flow: None,
                    });
                }
            }
        }
    }
    Ok((sales, lines))
}
