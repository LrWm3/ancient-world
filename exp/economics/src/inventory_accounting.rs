//! Costed stock subledger. Boundary-average cost uses opening stock only;
//! purchases become available to subsequent boundaries, never to this one's sales.
use crate::{
    accounting::{Account, Flow, Line},
    model::*,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holding {
    pub quantity: i32,
    pub cost: i128,
}
/// Verified physical delivery with noncash consideration (prepayment release or barter).
#[derive(Clone, Debug)]
pub(crate) struct PrepaidSale {
    pub seller: AgentId,
    pub buyer: AgentId,
    pub resource: ResourceId,
    pub quantity: i32,
    pub value: i128,
}
/// Interpret only a replay-verified bilateral stock exchange. The payment
/// resource comes from its accepted venue terms, never effect order or a label.
pub(crate) fn barter(
    world: &World,
    transaction: &Transaction,
    payment: ResourceId,
    unit_value: i128,
) -> Result<[PrepaidSale; 2], String> {
    let effects = &transaction.effects;
    let paid = effects
        .iter()
        .find(|e| e.account.1 == payment && e.delta < 0)
        .ok_or("missing barter payment")?;
    let sold = effects
        .iter()
        .find(|e| e.account.1 != payment && e.delta < 0)
        .ok_or("missing barter goods")?;
    if effects.len() != 4
        || unit_value <= 0
        || paid.account.0 == sold.account.0
        || effects.iter().any(|e| {
            !world
                .resources
                .iter()
                .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Stock)
        })
        || !effects.iter().any(|e| {
            e.account == (sold.account.0, payment) && i64::from(e.delta) == -i64::from(paid.delta)
        })
        || !effects.iter().any(|e| {
            e.account == (paid.account.0, sold.account.1)
                && i64::from(e.delta) == -i64::from(sold.delta)
        })
    {
        return Err("invalid bilateral stock barter".into());
    }
    let value = unit_value
        .checked_mul(-i128::from(paid.delta))
        .ok_or("barter consideration overflow")?;
    Ok([
        PrepaidSale {
            seller: sold.account.0,
            buyer: paid.account.0,
            resource: sold.account.1,
            quantity: sold.delta.checked_neg().ok_or("barter quantity overflow")?,
            value,
        },
        PrepaidSale {
            seller: paid.account.0,
            buyer: sold.account.0,
            resource: payment,
            quantity: paid.delta.checked_neg().ok_or("barter quantity overflow")?,
            value,
        },
    ])
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory(pub BTreeMap<crate::model::Account, Holding>);
/// One opening-stock cost pool shared by every adapter in a committed boundary.
/// Accounting visits sales, process IDs, then dated dues in a stable order. That
/// order assigns rounding ticks only; it does not grant physical resources.
pub(crate) struct CostAllocation {
    opening: Inventory,
    used: BTreeMap<crate::model::Account, i128>,
}
impl CostAllocation {
    pub(crate) fn new(opening: &Inventory) -> Self {
        Self {
            opening: opening.clone(),
            used: BTreeMap::new(),
        }
    }
    pub(crate) fn take(
        &mut self,
        key: crate::model::Account,
        quantity: i128,
    ) -> Result<i128, String> {
        let h = self
            .opening
            .0
            .get(&key)
            .ok_or("disposal without opening inventory cost")?;
        let prior = self.used.get(&key).copied().unwrap_or(0);
        let total = prior
            .checked_add(quantity)
            .ok_or("inventory allocation overflow")?;
        if quantity <= 0 || h.quantity <= 0 || total > i128::from(h.quantity) || h.cost < 0 {
            return Err("disposal exceeds opening inventory".into());
        }
        let denominator = i128::from(h.quantity);
        // Exact floor(cost * q / quantity), without overflowing cost * q.
        // q <= quantity; the remainder product is bounded by i32::MAX squared.
        let basis = |q| (h.cost / denominator) * q + (h.cost % denominator) * q / denominator;
        let cost = basis(total) - basis(prior);
        self.used.insert(key, total);
        Ok(cost)
    }
}
impl Inventory {
    pub fn open(
        world: &World,
        state: &State,
        coin: ResourceId,
        costs: BTreeMap<crate::model::Account, i128>,
    ) -> Result<Self, String> {
        let mut book = Self::default();
        for (key, cost) in costs {
            let quantity = state.balance(key.0, key.1);
            if key.1 == coin
                || cost < 0
                || quantity <= 0
                || !world.agents.iter().any(|a| a.id == key.0)
                || !world
                    .resources
                    .iter()
                    .any(|r| r.id == key.1 && r.kind == ResourceKind::Stock)
            {
                return Err("invalid opening inventory cost".into());
            }
            book.0.insert(key, Holding { quantity, cost });
        }
        book.validate(world, state, coin)?;
        Ok(book)
    }
    pub fn validate(&self, world: &World, state: &State, coin: ResourceId) -> Result<(), String> {
        for (&key, &quantity) in &state.balances {
            if key.1 != coin
                && quantity != 0
                && world
                    .resources
                    .iter()
                    .any(|r| r.id == key.1 && r.kind == ResourceKind::Stock)
                && self.0.get(&key).is_none_or(|h| h.quantity != quantity)
            {
                return Err("unpriced or unreconciled inventory quantity".into());
            }
        }
        for (&key, h) in &self.0 {
            if state.balance(key.0, key.1) != h.quantity
                || h.quantity < 0
                || h.cost < 0
                || (h.quantity == 0 && h.cost != 0)
            {
                return Err("inventory quantity/cost does not reconcile".into());
            }
        }
        Ok(())
    }
    /// Carrying-cost transfers for validated household allocations/contributions.
    /// Every donor draws only on this sub-boundary's opening holding. Cumulative
    /// proportional allocation keeps integer remainders with the last recipient.
    pub(crate) fn pool(
        &self,
        world: &World,
        effects: &[Effect],
        coin: ResourceId,
    ) -> Result<(Self, Vec<Line>), String> {
        let stock: Vec<_> = effects
            .iter()
            .filter(|e| {
                world
                    .resources
                    .iter()
                    .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Stock)
            })
            .collect();
        if stock.len() % 2 != 0 {
            return Err("household stock must be paired transfers".into());
        }
        let mut next = self.clone();
        let mut spent = BTreeMap::<crate::model::Account, i32>::new();
        let mut lines = vec![];
        for pair in stock.chunks_exact(2) {
            let (from, to) = (pair[0], pair[1]);
            if from.delta >= 0
                || to.delta <= 0
                || i64::from(from.delta) != -i64::from(to.delta)
                || from.account.1 != to.account.1
                || from.account.0 == to.account.0
            {
                return Err("invalid household stock transfer".into());
            }
            let quantity = to.delta;
            let cost = if from.account.1 == coin {
                for effect in [from, to] {
                    lines.push(Line {
                        agent: effect.account.0,
                        account: Account::Cash,
                        debit: i128::from(effect.delta),
                        flow: Some(Flow::Operating),
                    });
                }
                i128::from(quantity)
            } else {
                let opening = self
                    .0
                    .get(&from.account)
                    .ok_or("unpriced household contribution")?;
                let used = spent.entry(from.account).or_default();
                let prior = *used;
                *used = used
                    .checked_add(quantity)
                    .ok_or("household quantity overflow")?;
                if *used > opening.quantity || opening.quantity <= 0 {
                    return Err("household contribution exceeds opening inventory".into());
                }
                let allocated = |q: i32| {
                    opening
                        .cost
                        .checked_mul(i128::from(q))
                        .ok_or_else(|| "household cost overflow".to_string())
                        .map(|v| v / i128::from(opening.quantity))
                };
                let cost = allocated(*used)? - allocated(prior)?;
                let donor = next.0.get_mut(&from.account).unwrap();
                donor.quantity -= quantity;
                donor.cost -= cost;
                let recipient = next.0.entry(to.account).or_insert(Holding {
                    quantity: 0,
                    cost: 0,
                });
                recipient.quantity = recipient
                    .quantity
                    .checked_add(quantity)
                    .ok_or("household quantity overflow")?;
                recipient.cost = recipient
                    .cost
                    .checked_add(cost)
                    .ok_or("household cost overflow")?;
                cost
            };
            for (agent, account, debit) in [
                (from.account.0, Account::TransferExpense, cost),
                (to.account.0, Account::TransferIncome, -cost),
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
        next.0.retain(|_, h| h.quantity != 0);
        Ok((next, lines))
    }
    /// Verified replenishment or nonrival service entitlements add quantity at
    /// zero incremental cost. Existing acquisition/production costs are preserved.
    pub(crate) fn add_uncosted(
        &self,
        effects: &[Effect],
        coin: ResourceId,
    ) -> Result<Self, String> {
        let mut next = self.clone();
        for effect in effects {
            if effect.delta <= 0 || effect.account.1 == coin {
                return Err("invalid nonmonetary pool replenishment".into());
            }
            let h = next.0.entry(effect.account).or_insert(Holding {
                quantity: 0,
                cost: 0,
            });
            h.quantity = h
                .quantity
                .checked_add(effect.delta)
                .ok_or("pool quantity overflow")?;
        }
        Ok(next)
    }
    /// Release basis only for the domain's explicit, validated expiration effects.
    pub(crate) fn expire(&self, effects: &[Effect]) -> Result<(Self, Vec<Line>), String> {
        let mut next = self.clone();
        let mut lines = vec![];
        for effect in effects {
            let old = next
                .0
                .remove(&effect.account)
                .ok_or("unpriced expired inventory")?;
            if effect.delta != -old.quantity {
                return Err("expiration must remove the complete holding".into());
            }
            if old.cost != 0 {
                lines.push(Line {
                    agent: effect.account.0,
                    account: Account::InventoryLoss,
                    debit: old.cost,
                    flow: None,
                });
            }
        }
        Ok((next, lines))
    }
    /// Return candidate costs and revenue/expense/cash legs. Audit separately emits
    /// the inventory balance-sheet movements and validates the complete entry.
    pub fn settle(
        &self,
        trades: &[&Transaction],
        coin: ResourceId,
    ) -> Result<(Self, Vec<Line>), String> {
        self.settle_with_prepaid(trades, coin, &[])
    }
    pub(crate) fn settle_with_prepaid(
        &self,
        trades: &[&Transaction],
        coin: ResourceId,
        prepaid: &[PrepaidSale],
    ) -> Result<(Self, Vec<Line>), String> {
        self.settle_allocated(trades, coin, prepaid, &mut CostAllocation::new(self))
    }
    pub(crate) fn settle_allocated(
        &self,
        trades: &[&Transaction],
        coin: ResourceId,
        prepaid: &[PrepaidSale],
        allocation: &mut CostAllocation,
    ) -> Result<(Self, Vec<Line>), String> {
        let mut next = self.clone();
        if self
            .0
            .iter()
            .any(|(key, h)| key.1 == coin || h.quantity <= 0 || h.cost < 0)
        {
            return Err("invalid opening inventory holding".into());
        }
        let mut outgoing = BTreeMap::new();
        let mut incoming = BTreeMap::new();
        let mut purchase_costs = BTreeMap::new();
        let mut lines = vec![];
        for t in trades {
            // Only a fully funded, one-good/one-coin spot exchange has this shape.
            let effects: Vec<_> = t.effects.iter().filter(|e| e.delta != 0).collect();
            if effects.len() != 4 {
                return Err("unsupported inventory exchange shape".into());
            }
            let sold = effects
                .iter()
                .find(|e| e.account.1 != coin && e.delta < 0)
                .ok_or("missing goods sale")?;
            let bought = effects
                .iter()
                .find(|e| e.account.1 == sold.account.1 && e.delta > 0)
                .ok_or("missing goods receipt")?;
            let quantity = sold
                .delta
                .checked_neg()
                .ok_or("inventory quantity overflow")?;
            let seller = sold.account.0;
            let buyer = bought.account.0;
            let received = effects
                .iter()
                .find(|e| e.account == (seller, coin) && e.delta > 0)
                .ok_or("missing sale proceeds")?;
            let paid = effects
                .iter()
                .find(|e| e.account == (buyer, coin) && e.delta < 0)
                .ok_or("missing purchase payment")?;
            if seller == buyer
                || bought.delta != quantity
                || i64::from(paid.delta) != -i64::from(received.delta)
            {
                return Err("unbalanced inventory exchange".into());
            }
            let price = i128::from(received.delta);
            crate::accounting::add(&mut outgoing, sold.account, i128::from(quantity))?;
            crate::accounting::add(&mut incoming, bought.account, i128::from(quantity))?;
            crate::accounting::add(&mut purchase_costs, bought.account, price)?;
            for (agent, account, debit, flow) in [
                (seller, Account::Cash, price, Some(Flow::Operating)),
                (seller, Account::Sales, -price, None),
                (buyer, Account::Cash, -price, Some(Flow::Operating)),
            ] {
                lines.push(Line {
                    agent,
                    account,
                    debit,
                    flow,
                });
            }
        }
        for sale in prepaid {
            if sale.quantity <= 0
                || sale.value < 0
                || sale.resource == coin
                || sale.seller == sale.buyer
            {
                return Err("invalid prepaid inventory delivery".into());
            }
            crate::accounting::add(
                &mut outgoing,
                (sale.seller, sale.resource),
                i128::from(sale.quantity),
            )?;
            crate::accounting::add(
                &mut incoming,
                (sale.buyer, sale.resource),
                i128::from(sale.quantity),
            )?;
            crate::accounting::add(&mut purchase_costs, (sale.buyer, sale.resource), sale.value)?;
            if sale.value != 0 {
                lines.push(Line {
                    agent: sale.seller,
                    account: Account::Sales,
                    debit: -sale.value,
                    flow: None,
                });
            }
        }
        for (key, quantity) in outgoing {
            // Aggregate sales before releasing basis; other adapters share this
            // cursor, so receipts cannot fund a second disposal in this boundary.
            let cost = allocation.take(key, quantity)?;
            let h = next.0.get_mut(&key).ok_or("missing cost holding")?;
            h.quantity -= i32::try_from(quantity).map_err(|_| "inventory quantity overflow")?;
            h.cost -= cost;
            if cost != 0 {
                lines.push(Line {
                    agent: key.0,
                    account: Account::CostOfSales,
                    debit: cost,
                    flow: None,
                });
            }
        }
        for (key, quantity) in incoming {
            let h = next.0.entry(key).or_insert(Holding {
                quantity: 0,
                cost: 0,
            });
            h.quantity = i32::try_from(
                i128::from(h.quantity)
                    .checked_add(quantity)
                    .ok_or("inventory quantity overflow")?,
            )
            .map_err(|_| "inventory quantity overflow")?;
            h.cost = h
                .cost
                .checked_add(purchase_costs[&key])
                .ok_or("inventory cost overflow")?;
        }
        next.0.retain(|_, h| h.quantity != 0);
        Ok((next, lines))
    }
}

#[cfg(test)]
mod pooling_tests {
    use super::*;
    use crate::scenario::{GRAIN, PERSON};
    fn transfers(items: &[(AgentId, AgentId, i32)]) -> Vec<Effect> {
        items
            .iter()
            .flat_map(|&(from, to, q)| {
                [
                    Effect {
                        account: (from, GRAIN),
                        delta: -q,
                    },
                    Effect {
                        account: (to, GRAIN),
                        delta: q,
                    },
                ]
            })
            .collect()
    }
    #[test]
    fn pooled_cost_rounding_retains_every_tick_without_spending_receipts() {
        let (world, _) = crate::scenario::baseline();
        let inventory = Inventory(BTreeMap::from([
            (
                (PERSON, GRAIN),
                Holding {
                    quantity: 3,
                    cost: 2,
                },
            ),
            (
                (1, GRAIN),
                Holding {
                    quantity: 1,
                    cost: 5,
                },
            ),
        ]));
        let (next, _) = inventory
            .pool(
                &world,
                &transfers(&[(PERSON, 1, 1), (PERSON, 2, 1), (PERSON, 3, 1), (1, 2, 1)]),
                crate::scenario::TOKEN,
            )
            .unwrap();
        assert!(!next.0.contains_key(&(PERSON, GRAIN)));
        assert_eq!(
            next.0[&(1, GRAIN)],
            Holding {
                quantity: 1,
                cost: 0
            }
        );
        assert_eq!(
            next.0[&(2, GRAIN)],
            Holding {
                quantity: 2,
                cost: 6
            }
        );
        assert_eq!(
            next.0[&(3, GRAIN)],
            Holding {
                quantity: 1,
                cost: 1
            }
        );
        assert_eq!(next.0.values().map(|h| h.cost).sum::<i128>(), 7);
        assert!(
            inventory
                .pool(
                    &world,
                    &transfers(&[(PERSON, 1, 1), (1, 2, 2)]),
                    crate::scenario::TOKEN
                )
                .is_err()
        );
        assert_eq!(inventory.0[&(1, GRAIN)].cost, 5);
    }
}
