//! Physical loan transfers use explicit fixed claim values and the common stock
//! cost pool. Financing creates neither sales nor cash-flow entries.
use crate::{
    accounting::{Account, Line},
    credit::{Boundary, Event},
    finance::Transfer,
    inventory_accounting::{CostAllocation, Holding, Inventory},
    model::*,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn settle(
    world: &World,
    boundary: Option<&Boundary>,
    before: &State,
    coin: ResourceId,
    values: &BTreeMap<ResourceId, i128>,
    inventory: &Inventory,
    allocation: &mut CostAllocation,
) -> Result<(Inventory, Vec<Line>, Vec<Transaction>), String> {
    let Some(b) = boundary else {
        return Ok((inventory.clone(), vec![], vec![]));
    };
    let mut transfers = vec![];
    let loan = |id: &u32| {
        b.after
            .loans
            .get(id)
            .ok_or("missing physical reporting loan")
    };
    for event in &b.events {
        match event {
            Event::Advanced {
                creditor,
                debtor,
                amount,
                ..
            } if amount.resource != coin => {
                transfers.push(Transfer {
                    from: *creditor,
                    to: *debtor,
                    amount: amount.clone(),
                });
            }
            Event::Paid {
                loan: id,
                principal,
                interest,
            } if loan(id)?.denomination != coin => {
                let l = loan(id)?;
                let quantity = principal
                    .checked_add(*interest)
                    .ok_or("physical repayment overflow")?;
                if quantity > 0 {
                    transfers.push(Transfer {
                        from: l.debtor,
                        to: l.creditor,
                        amount: Amount::new(l.denomination, quantity),
                    });
                }
            }
            _ => (),
        }
    }
    let mut delivery_values = BTreeMap::new();
    let mut forward_progress = BTreeMap::new();
    for receipt in &b.recovery {
        if let crate::recovery::Receipt::Guaranteed {
            guarantee,
            claim,
            paid,
            ..
        } = receipt
        {
            let (_, creditor, denomination) =
                claim.parties(world).ok_or("missing guaranteed terms")?;
            if *paid > 0 && denomination != coin {
                let g = world
                    .recovery
                    .guarantees
                    .iter()
                    .find(|g| g.id == *guarantee)
                    .ok_or("missing physical guarantee")?;
                if let crate::recovery::GuaranteedClaim::Forward(id) = claim {
                    let c = before
                        .exchange
                        .forwards
                        .get(id)
                        .ok_or("missing guaranteed forward basis")?;
                    let settled = forward_progress
                        .entry(*id)
                        .or_insert(c.delivered + c.written_off());
                    let next = settled
                        .checked_add(*paid)
                        .ok_or("guaranteed basis overflow")?;
                    let value = crate::forward_accounting::released(c, next)?
                        - crate::forward_accounting::released(c, *settled)?;
                    delivery_values.insert(transfers.len(), value);
                    *settled = next;
                }
                transfers.push(Transfer {
                    from: g.guarantor,
                    to: creditor,
                    amount: Amount::new(denomination, *paid),
                });
            }
        }
    }
    let mut next = inventory.clone();
    let mut lines = vec![];
    let mut used = BTreeSet::new();
    let mut recognized = vec![];
    for (transfer_index, transfer) in transfers.into_iter().enumerate() {
        let effects = transfer.effects()?;
        let (index, transaction) = b
            .transactions
            .iter()
            .enumerate()
            .find(|(index, tx)| !used.contains(index) && tx.effects == effects)
            .ok_or("physical loan receipt does not match actual transfer")?;
        used.insert(index);
        recognized.push(transaction.clone());
        let value = crate::reporting_value::value(
            coin,
            values,
            transfer.amount.resource,
            transfer.amount.quantity,
        )?;
        let source = (transfer.from, transfer.amount.resource);
        let cost = allocation.take(source, i128::from(transfer.amount.quantity))?;
        let h = next
            .0
            .get_mut(&source)
            .ok_or("missing physical loan inventory")?;
        h.quantity -= transfer.amount.quantity;
        h.cost -= cost;
        let h = next
            .0
            .entry((transfer.to, transfer.amount.resource))
            .or_insert(Holding {
                quantity: 0,
                cost: 0,
            });
        h.quantity = h
            .quantity
            .checked_add(transfer.amount.quantity)
            .ok_or("physical loan inventory overflow")?;
        h.cost = h
            .cost
            .checked_add(
                delivery_values
                    .get(&transfer_index)
                    .copied()
                    .unwrap_or(value),
            )
            .ok_or("physical loan basis overflow")?;
        if cost != value {
            lines.push(Line {
                agent: transfer.from,
                account: if cost > value {
                    Account::SettlementLoss
                } else {
                    Account::SettlementGain
                },
                debit: cost - value,
                flow: None,
            });
        }
    }
    Ok((next, lines, recognized))
}
