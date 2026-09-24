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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory(pub BTreeMap<crate::model::Account, Holding>);
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
    /// Return candidate costs and revenue/expense/cash legs. Audit separately emits
    /// the inventory balance-sheet movements and validates the complete entry.
    pub fn settle(
        &self,
        trades: &[&Transaction],
        coin: ResourceId,
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
        for (key, quantity) in outgoing {
            let opening = self
                .0
                .get(&key)
                .ok_or("sale without opening inventory cost")?;
            if quantity > i128::from(opening.quantity) || opening.quantity <= 0 {
                return Err("sale exceeds opening inventory".into());
            }
            // Pool all same-boundary sales before rounding, making split lots and
            // transaction ordering irrelevant. Full depletion releases every tick.
            let cost = opening
                .cost
                .checked_mul(quantity)
                .ok_or("inventory costing overflow")?
                / i128::from(opening.quantity);
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
