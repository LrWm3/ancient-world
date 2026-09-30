//! Dated land-dues recognition at a fixed, explicit value per native unit.
use crate::{
    accounting::{self, Account, Flow, Line},
    commitments,
    inventory_accounting::{Holding, Inventory},
    model::*,
};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Valuation(pub BTreeMap<u32, i128>);
impl Valuation {
    pub(crate) fn unit(
        &self,
        a: &commitments::Agreement,
        coin: ResourceId,
    ) -> Result<i128, String> {
        if a.payment.resource == coin {
            return Ok(1);
        }
        self.0
            .get(&a.id)
            .copied()
            .filter(|v| *v > 0)
            .ok_or("land dues require positive native-unit valuation".into())
    }
    pub fn positions(
        &self,
        world: &World,
        state: &State,
        coin: ResourceId,
    ) -> Result<BTreeMap<(AgentId, Account), i128>, String> {
        let mut p = BTreeMap::new();
        for (key, o) in &state.obligations {
            let a = commitments::active(world, state)
                .find(|a| a.id == key.0)
                .ok_or("missing dues agreement")?;
            let value = self
                .unit(a, coin)?
                .checked_mul(i128::from(o.outstanding()))
                .ok_or("dues valuation overflow")?;
            accounting::add(
                &mut p,
                (a.creditor, Account::DuesReceivable(key.0, key.1)),
                value,
            )?;
            accounting::add(
                &mut p,
                (a.debtor, Account::DuesPayable(key.0, key.1)),
                -value,
            )?;
        }
        Ok(p)
    }
    pub fn settle(
        &self,
        world: &World,
        before: &State,
        after: &State,
        inventory: &Inventory,
        coin: ResourceId,
        transactions: &[Transaction],
    ) -> Result<(Inventory, Vec<Line>), String> {
        self.settle_allocated(
            world,
            before,
            after,
            inventory,
            coin,
            transactions,
            &mut crate::inventory_accounting::CostAllocation::new(inventory),
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn settle_allocated(
        &self,
        world: &World,
        before: &State,
        after: &State,
        inventory: &Inventory,
        coin: ResourceId,
        transactions: &[Transaction],
        allocation: &mut crate::inventory_accounting::CostAllocation,
    ) -> Result<(Inventory, Vec<Line>), String> {
        let mut stocks = inventory.clone();
        let mut lines = vec![];
        let mut effects = BTreeMap::new();
        let mut used = BTreeMap::new();
        let mut released = BTreeMap::new();
        let mut received = BTreeMap::new();
        let mut received_cost = BTreeMap::new();
        let mut push = |agent, account, debit, flow| {
            if debit != 0 {
                lines.push(Line {
                    agent,
                    account,
                    debit,
                    flow,
                });
            }
        };
        if before
            .obligations
            .keys()
            .any(|key| !after.obligations.contains_key(key))
        {
            return Err("dues removal requires explicit relief accounting".into());
        }
        for (key, o) in &after.obligations {
            let a = commitments::active(world, after)
                .find(|a| a.id == key.0)
                .ok_or("missing dues agreement")?;
            let unit = self.unit(a, coin)?;
            let old = before.obligations.get(key);
            let newly_owed = o.owed - old.map_or(0, |o| o.owed);
            let paid = o.paid - old.map_or(0, |o| o.paid);
            let native = o.in_kind_paid - old.map_or(0, |o| o.in_kind_paid);
            if newly_owed < 0 || native < 0 || paid < native {
                return Err("dues relief requires explicit accounting".into());
            }
            let waived = o.written_off() - old.map_or(0, |o| o.written_off());
            if waived < 0 {
                return Err("land relief cannot be reversed".into());
            }
            let loss = unit
                .checked_mul(i128::from(waived))
                .ok_or("land relief value overflow")?;
            push(a.creditor, Account::CreditLoss, loss, None);
            push(a.debtor, Account::DebtRelief, -loss, None);
            let charge = unit
                .checked_mul(i128::from(newly_owed))
                .ok_or("dues valuation overflow")?;
            push(a.debtor, Account::DuesExpense, charge, None);
            push(a.creditor, Account::DuesIncome, -charge, None);
            let native_value = unit
                .checked_mul(i128::from(native))
                .ok_or("dues valuation overflow")?;
            if native > 0 {
                accounting::add(
                    &mut effects,
                    (a.debtor, a.payment.resource),
                    -i128::from(native),
                )?;
                accounting::add(
                    &mut effects,
                    (a.creditor, a.payment.resource),
                    i128::from(native),
                )?;
                if a.payment.resource == coin {
                    push(
                        a.debtor,
                        Account::Cash,
                        -native_value,
                        Some(Flow::Operating),
                    );
                    push(
                        a.creditor,
                        Account::Cash,
                        native_value,
                        Some(Flow::Operating),
                    );
                } else {
                    let key = (a.debtor, a.payment.resource);
                    let cost = allocation.take(key, i128::from(native))?;
                    accounting::add(&mut used, key, i128::from(native))?;
                    accounting::add(&mut released, key, cost)?;
                    let gain = native_value.checked_sub(cost).ok_or("dues gain overflow")?;
                    push(
                        a.debtor,
                        if gain >= 0 {
                            Account::DisposalGain
                        } else {
                            Account::DisposalLoss
                        },
                        -gain,
                        None,
                    );
                    accounting::add(
                        &mut received,
                        (a.creditor, a.payment.resource),
                        i128::from(native),
                    )?;
                    accounting::add(
                        &mut received_cost,
                        (a.creditor, a.payment.resource),
                        native_value,
                    )?;
                }
            }
            let alternate = paid - native;
            if alternate > 0 {
                let tender = world
                    .activities
                    .coin_payments
                    .get(&a.id)
                    .ok_or("missing accepted tender")?;
                if tender.resource != coin {
                    return Err("mixed dues tender denomination".into());
                }
                let cash = i128::from(alternate) * i128::from(tender.coins_per_unit);
                let value = unit
                    .checked_mul(i128::from(alternate))
                    .ok_or("dues valuation overflow")?;
                let gain = value.checked_sub(cash).ok_or("dues gain overflow")?;
                push(a.debtor, Account::Cash, -cash, Some(Flow::Operating));
                push(a.creditor, Account::Cash, cash, Some(Flow::Operating));
                push(
                    a.debtor,
                    if gain >= 0 {
                        Account::SettlementGain
                    } else {
                        Account::SettlementLoss
                    },
                    -gain,
                    None,
                );
                push(
                    a.creditor,
                    if gain >= 0 {
                        Account::SettlementLoss
                    } else {
                        Account::SettlementGain
                    },
                    gain,
                    None,
                );
                accounting::add(&mut effects, (a.debtor, coin), -cash)?;
                accounting::add(&mut effects, (a.creditor, coin), cash)?;
            }
        }
        let mut actual = BTreeMap::new();
        for t in transactions {
            for e in &t.effects {
                accounting::add(&mut actual, e.account, i128::from(e.delta))?;
            }
        }
        actual.retain(|_, v| *v != 0);
        effects.retain(|_, v| *v != 0);
        if actual != effects {
            return Err("dues receipts do not reconcile to actual transfers".into());
        }
        for (key, q) in used {
            let h = stocks.0.get_mut(&key).ok_or("missing inventory")?;
            h.quantity -= i32::try_from(q).map_err(|_| "dues quantity overflow")?;
            h.cost -= released[&key];
        }
        for (key, q) in received {
            let h = stocks.0.entry(key).or_insert(Holding {
                quantity: 0,
                cost: 0,
            });
            h.quantity =
                i32::try_from(i128::from(h.quantity) + q).map_err(|_| "dues quantity overflow")?;
            h.cost = h
                .cost
                .checked_add(received_cost[&key])
                .ok_or("dues cost overflow")?;
        }
        stocks.0.retain(|_, h| h.quantity != 0);
        Ok((stocks, lines))
    }
}

/// Estate money is owned restricted cash of the debtor, held by a custodian.
/// Project the verified physical payer into the dues adapter's debtor view, then
/// reclassify that adapter's cash debit to restricted cash. No income is added.
pub(crate) fn estate_payments(
    world: &World,
    boundary: Option<&crate::credit::Boundary>,
    transactions: &[Transaction],
    coin: ResourceId,
) -> Result<(Vec<Transaction>, Vec<Line>), String> {
    let mut expected = BTreeMap::new();
    let mut lines = vec![];
    let mut projected = vec![];
    if let Some(boundary) = boundary {
        for receipt in &boundary.recovery {
            let crate::recovery::Receipt::LandDistributed {
                proceeding,
                creditor,
                tender,
                ..
            } = receipt
            else {
                continue;
            };
            if tender.resource != coin || tender.quantity < 0 {
                return Err("unsupported estate dues tender".into());
            }
            if tender.quantity == 0 {
                continue;
            }
            let p = world
                .recovery
                .proceedings
                .iter()
                .find(|p| p.id == *proceeding)
                .ok_or("missing estate dues proceeding")?;
            if p.denomination != coin {
                return Err("mixed estate dues denomination".into());
            }
            let cash = i128::from(tender.quantity);
            accounting::add(&mut expected, (p.estate, *creditor), cash)?;
            // Receipt identity determines beneficial ownership even when several
            // estates share both a custodian and a creditor. This reporting-only
            // projection is checked against the aggregate physical legs below.
            projected.push(crate::credit::tx(
                format!("estate {} dues reporting projection", p.id),
                crate::finance::Transfer {
                    from: p.debtor,
                    to: *creditor,
                    amount: tender.clone(),
                }
                .effects()?,
            ));
            lines.extend([
                Line {
                    agent: p.debtor,
                    account: Account::Cash,
                    debit: cash,
                    flow: Some(Flow::Operating),
                },
                Line {
                    agent: p.debtor,
                    account: Account::RestrictedCash(p.id),
                    debit: -cash,
                    flow: Some(Flow::Operating),
                },
            ]);
        }
    }
    let mut actual = BTreeMap::new();
    for t in transactions {
        let Some((index, p)) = t.effects.iter().enumerate().find_map(|(i, e)| {
            (e.delta < 0)
                .then(|| {
                    world
                        .recovery
                        .proceedings
                        .iter()
                        .find(|p| e.account == (p.estate, coin))
                        .map(|p| (i, p))
                })
                .flatten()
        }) else {
            projected.push(t.clone());
            continue;
        };
        let paid = -i128::from(t.effects[index].delta);
        let received = t
            .effects
            .iter()
            .find(|e| e.account.1 == coin && i128::from(e.delta) == paid)
            .ok_or("missing estate dues recipient")?;
        if t.effects.len() != 2 || received.account.0 == p.estate {
            return Err("unsupported estate dues transfer".into());
        }
        accounting::add(&mut actual, (p.estate, received.account.0), paid)?;
    }
    if actual != expected {
        return Err("estate dues receipts do not reconcile to transfers".into());
    }
    Ok((projected, lines))
}

/// A guarantee substitutes funded recourse for the debtor's dues payable.
/// Project only verified guarantee transfers into the ordinary dues adapter;
/// reverse its fictitious debtor cash leg and classify the guarantor's outlay.
pub(crate) fn guarantee_payments(
    world: &World,
    boundary: Option<&crate::credit::Boundary>,
    transactions: &[Transaction],
    coin: ResourceId,
    values: &BTreeMap<ResourceId, i128>,
) -> Result<(Vec<Transaction>, Vec<Line>), String> {
    let mut transfers = transactions.to_vec();
    let mut used = std::collections::BTreeSet::new();
    let mut physical = std::collections::BTreeSet::new();
    let mut lines = vec![];
    if let Some(b) = boundary {
        for receipt in &b.recovery {
            let crate::recovery::Receipt::Guaranteed {
                guarantee,
                claim: crate::recovery::GuaranteedClaim::Land { .. },
                paid,
                tender,
                ..
            } = receipt
            else {
                continue;
            };
            if *paid == 0 {
                continue;
            }
            let g = world
                .recovery
                .guarantees
                .iter()
                .find(|g| g.id == *guarantee)
                .ok_or("missing land guarantee")?;
            let (debtor, creditor, denomination) =
                g.claim.parties(world).ok_or("missing guaranteed dues")?;
            let (index, _) = transactions
                .iter()
                .enumerate()
                .find(|(index, t)| {
                    !used.contains(index)
                        && t.effects
                            == vec![
                                Effect {
                                    account: (g.guarantor, tender.resource),
                                    delta: -tender.quantity,
                                },
                                Effect {
                                    account: (creditor, tender.resource),
                                    delta: tender.quantity,
                                },
                            ]
                })
                .ok_or("guaranteed dues receipt does not match actual transfer")?;
            used.insert(index);
            if tender.resource != coin {
                physical.insert(index);
                continue;
            }
            let value = crate::reporting_value::value(coin, values, denomination, *paid)?;
            let gain = value - i128::from(tender.quantity);
            if gain != 0 {
                let account = if gain > 0 {
                    Account::SettlementGain
                } else {
                    Account::SettlementLoss
                };
                // The debtor substitutes equal native recourse, while the guarantor
                // bears the difference between its cash outlay and native claim value.
                lines.push(Line {
                    agent: debtor,
                    account: account.clone(),
                    debit: gain,
                    flow: None,
                });
                lines.push(Line {
                    agent: g.guarantor,
                    account,
                    debit: -gain,
                    flow: None,
                });
            }
            transfers[index].effects[0].account.0 = debtor;
            lines.extend([
                Line {
                    agent: debtor,
                    account: Account::Cash,
                    debit: i128::from(tender.quantity),
                    flow: Some(Flow::Operating),
                },
                Line {
                    agent: g.guarantor,
                    account: Account::Cash,
                    debit: -i128::from(tender.quantity),
                    flow: Some(Flow::Investing),
                },
            ]);
        }
    }
    let transfers = transfers
        .into_iter()
        .enumerate()
        .filter_map(|(index, tx)| (!physical.contains(&index)).then_some(tx))
        .collect();
    Ok((transfers, lines))
}

/// Native guarantee delivery is financed by the guarantor, not debtor inventory.
/// Exclude that payment only from the ordinary inventory-payment adapter; the
/// authoritative after-state and financial positions retain actual performance.
pub(crate) fn without_physical_guarantees(
    world: &World,
    boundary: Option<&crate::credit::Boundary>,
    after: &State,
    coin: ResourceId,
) -> Result<State, String> {
    let mut adjusted = after.clone();
    if let Some(b) = boundary {
        for r in &b.recovery {
            if let crate::recovery::Receipt::Guaranteed {
                claim: claim @ crate::recovery::GuaranteedClaim::Land { agreement, due },
                paid,
                tender,
                ..
            } = r
                && tender.resource != coin
                && claim
                    .parties(world)
                    .ok_or("missing land guarantee terms")?
                    .2
                    != coin
            {
                let bill = adjusted
                    .obligations
                    .get_mut(&(*agreement, *due))
                    .ok_or("missing guaranteed bill")?;
                bill.paid = bill
                    .paid
                    .checked_sub(*paid)
                    .filter(|n| *n >= 0)
                    .ok_or("guaranteed dues exceed payment")?;
                bill.in_kind_paid = bill
                    .in_kind_paid
                    .checked_sub(*paid)
                    .filter(|n| *n >= 0)
                    .ok_or("guaranteed dues exceed native payment")?;
            }
        }
    }
    Ok(adjusted)
}
