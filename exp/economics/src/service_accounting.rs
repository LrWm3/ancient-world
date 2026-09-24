//! Opt-in historical cost of bought, current-period capacity. Physical availability
//! comes from settlement; this subledger never creates hours or pays imputed wages.
use crate::{
    accounting::{self, Account, Line},
    model::*,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Costs {
    /// Remaining purchase cost mixed across the account's actual available capacity.
    pub balances: BTreeMap<crate::model::Account, i128>,
}
type Settlement = (Costs, Vec<Line>, BTreeMap<u64, i128>);

impl Costs {
    pub(crate) fn positions(
        &self,
        world: &World,
        state: &State,
    ) -> Result<BTreeMap<(AgentId, Account), i128>, String> {
        let mut positions = BTreeMap::new();
        for (&key, &value) in &self.balances {
            if value < 0
                || !world
                    .resources
                    .iter()
                    .any(|r| r.id == key.1 && r.kind == ResourceKind::Capacity)
                || !world.agents.iter().any(|a| a.id == key.0)
                || (value > 0 && state.balance(key.0, key.1) <= 0)
            {
                return Err("invalid purchased capacity cost or availability".into());
            }
            if value != 0 {
                positions.insert((key.0, Account::PurchasedCapacity(key.1)), value);
            }
        }
        Ok(positions)
    }
    pub(crate) fn settle(
        &self,
        world: &World,
        before: &State,
        batch: &Batch,
        services: &[&Transaction],
        coin: ResourceId,
    ) -> Result<Settlement, String> {
        self.positions(world, before)?;
        let mut next = self.clone();
        let mut lines = vec![];
        let mut work = BTreeMap::new();
        if batch.phase == Phase::Open {
            // The old period expires even if regeneration restores the same quantity.
            for (&(agent, _), &value) in &next.balances {
                if value != 0 {
                    lines.push(Line {
                        agent,
                        account: Account::ServiceExpense,
                        debit: value,
                        flow: None,
                    });
                }
            }
            next.balances.clear();
            return Ok((next, lines, work));
        }
        let mut used = BTreeMap::new();
        let mut released = BTreeMap::new();
        let mut transactions: Vec<_> = batch.transactions.iter().collect();
        transactions.sort_by_key(|t| t.process.as_ref().map(|p| p.after.id));
        for t in transactions {
            for e in t.effects.iter().filter(|e| {
                e.delta < 0
                    && world
                        .resources
                        .iter()
                        .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Capacity)
            }) {
                let basis = self.balances.get(&e.account).copied().unwrap_or(0);
                if basis == 0 {
                    continue;
                }
                let opening = i128::from(before.balance(e.account.0, e.account.1));
                accounting::add(&mut used, e.account, -i128::from(e.delta))?;
                let quantity = used[&e.account];
                if quantity > opening {
                    return Err("paid capacity use exceeds opening availability".into());
                }
                let cumulative =
                    (basis / opening) * quantity + (basis % opening) * quantity / opening;
                let cost = cumulative - released.get(&e.account).copied().unwrap_or(0);
                released.insert(e.account, cumulative);
                *next
                    .balances
                    .get_mut(&e.account)
                    .ok_or("missing service basis")? -= cost;
                if let Some(p) = &t.process {
                    if p.after.operator != e.account.0 {
                        return Err("third-party paid capacity use needs a transfer policy".into());
                    }
                    accounting::add(&mut work, p.after.id, cost)?;
                } else if services.contains(&t) {
                    if cost != 0 {
                        lines.push(Line {
                            agent: e.account.0,
                            account: Account::ServiceExpense,
                            debit: cost,
                            flow: None,
                        });
                    }
                } else {
                    return Err("paid capacity movement lacks a cost adapter".into());
                }
            }
        }
        // Incoming hours cannot supply the opening uses costed above.
        for t in services {
            let mut sale = crate::issuance_accounting::services(t, coin)?;
            let bought = t
                .effects
                .iter()
                .find(|e| {
                    e.delta > 0
                        && world
                            .resources
                            .iter()
                            .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Capacity)
                })
                .ok_or("missing bought capacity")?;
            let expense = sale
                .iter()
                .find(|l| l.agent == bought.account.0 && l.account == Account::ServiceExpense)
                .ok_or("missing service cost")?;
            accounting::add(&mut next.balances, bought.account, expense.debit)?;
            sale.retain(|l| !(l.agent == bought.account.0 && l.account == Account::ServiceExpense));
            lines.extend(sale);
        }
        next.balances.retain(|_, value| *value != 0);
        Ok((next, lines, work))
    }
}
