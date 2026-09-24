//! Material-cost recognition for owner-operated processes; no imputed labor income.
use crate::{
    accounting::{self, Account, Line},
    inventory_accounting::{Holding, Inventory},
    model::*,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Costs {
    /// Relative total cost shares by output resource, required for joint products.
    pub output_weights: BTreeMap<DefinitionId, BTreeMap<ResourceId, u32>>,
    pub work: BTreeMap<u64, (AgentId, i128)>,
}
impl Costs {
    pub fn validate(&self, state: &State) -> Result<(), String> {
        for (id, (owner, cost)) in &self.work {
            let p = state.processes.get(id).ok_or("missing work in progress")?;
            if *cost < 0
                || p.status != Status::Active
                || p.operator != *owner
                || p.beneficiary != *owner
            {
                return Err("work-in-progress transfer/status needs an accounting adapter".into());
            }
        }
        Ok(())
    }
    pub fn settle(
        &self,
        world: &World,
        inventory: &Inventory,
        transactions: &[&Transaction],
        coin: ResourceId,
    ) -> Result<(Self, Inventory, Vec<Line>), String> {
        let mut next = self.clone();
        let mut stocks = inventory.clone();
        let mut lines = vec![];
        let mut used = BTreeMap::new();
        let mut inputs = BTreeMap::<u64, i128>::new();
        let mut ordered = transactions.to_vec();
        ordered.sort_by_key(|t| t.process.as_ref().map(|p| p.after.id));
        // Cumulative rounding assigns the boundary's released carrying cost by
        // stable process ID, never by the order of transaction iteration.
        for t in &ordered {
            let p = &t.process.as_ref().ok_or("missing process receipt")?.after;
            if p.operator != p.beneficiary {
                return Err("cross-agent production costs need a transfer adapter".into());
            }
            for e in &t.effects {
                let kind = world
                    .resources
                    .iter()
                    .find(|r| r.id == e.account.1)
                    .ok_or("unknown process resource")?
                    .kind;
                if kind != ResourceKind::Stock {
                    continue;
                }
                if e.account.1 == coin || e.account.0 != p.operator {
                    return Err("coin or third-party process input/output unsupported".into());
                }
                if e.delta < 0 {
                    let h = inventory
                        .0
                        .get(&e.account)
                        .ok_or("unpriced process input")?;
                    let previous = used.get(&e.account).copied().unwrap_or(0);
                    accounting::add(&mut used, e.account, -i128::from(e.delta))?;
                    let total = used[&e.account];
                    if total > i128::from(h.quantity) || h.quantity <= 0 {
                        return Err("process exceeds opening inventory".into());
                    }
                    let before = h
                        .cost
                        .checked_mul(previous)
                        .ok_or("input costing overflow")?
                        / i128::from(h.quantity);
                    let after = h.cost.checked_mul(total).ok_or("input costing overflow")?
                        / i128::from(h.quantity);
                    accounting::add(&mut inputs, p.id, after - before)?;
                }
            }
        }
        for (key, q) in used {
            let old = &inventory.0[&key];
            let cost =
                old.cost.checked_mul(q).ok_or("input costing overflow")? / i128::from(old.quantity);
            let h = stocks.0.get_mut(&key).ok_or("missing input holding")?;
            h.quantity -= i32::try_from(q).map_err(|_| "quantity overflow")?;
            h.cost -= cost;
        }
        for t in ordered {
            let p = &t.process.as_ref().ok_or("missing process receipt")?.after;
            let d = world.definition(p.definition);
            let old = next.work.remove(&p.id).map_or(0, |(_, c)| c);
            let cost = old
                .checked_add(inputs.get(&p.id).copied().unwrap_or(0))
                .ok_or("work-in-progress overflow")?;
            let mut outputs = BTreeMap::new();
            for e in &t.effects {
                if e.delta > 0
                    && world
                        .resources
                        .iter()
                        .any(|r| r.id == e.account.1 && r.kind == ResourceKind::Stock)
                {
                    accounting::add(&mut outputs, e.account.1, i128::from(e.delta))?;
                }
            }
            let expense = if d.execution == Execution::Consumption {
                if !outputs.is_empty() || p.status != Status::Completed {
                    return Err("consumption must complete into nonfinancial fulfillment".into());
                }
                Some(Account::ConsumptionExpense)
            } else {
                match p.status {
                    Status::Active => {
                        if !outputs.is_empty() {
                            return Err(
                                "intermediate outputs require a cost allocation policy".into()
                            );
                        }
                        next.work.insert(p.id, (p.operator, cost));
                        None
                    }
                    Status::Aborted => {
                        if !outputs.is_empty() {
                            return Err("aborted output unsupported".into());
                        }
                        Some(Account::ProductionLoss)
                    }
                    Status::Completed if outputs.is_empty() => Some(Account::ProductionExpense),
                    Status::Completed => {
                        let weights = if outputs.len() == 1 {
                            BTreeMap::from([(*outputs.keys().next().ok_or("missing output")?, 1)])
                        } else {
                            self.output_weights
                                .get(&d.id)
                                .cloned()
                                .ok_or("joint outputs require explicit cost shares")?
                        };
                        if weights.len() != outputs.len()
                            || weights
                                .iter()
                                .any(|(r, w)| *w == 0 || !outputs.contains_key(r))
                        {
                            return Err(
                                "output cost shares must cover actual joint products".into()
                            );
                        }
                        let total = weights
                            .values()
                            .try_fold(0_i128, |sum, w| sum.checked_add(i128::from(*w)))
                            .ok_or("weight overflow")?;
                        let mut cumulative = 0_i128;
                        let mut allocated = 0_i128;
                        for (resource, quantity) in outputs {
                            cumulative += i128::from(weights[&resource]);
                            let share = cost
                                .checked_mul(cumulative)
                                .ok_or("output costing overflow")?
                                / total;
                            let h = stocks
                                .0
                                .entry((p.beneficiary, resource))
                                .or_insert(Holding {
                                    quantity: 0,
                                    cost: 0,
                                });
                            h.quantity = i32::try_from(
                                i128::from(h.quantity)
                                    .checked_add(quantity)
                                    .ok_or("output quantity overflow")?,
                            )
                            .map_err(|_| "output quantity overflow")?;
                            h.cost = h
                                .cost
                                .checked_add(share - allocated)
                                .ok_or("output cost overflow")?;
                            allocated = share;
                        }
                        None
                    }
                }
            };
            if let Some(account) = expense
                && cost != 0
            {
                lines.push(Line {
                    agent: p.operator,
                    account,
                    debit: cost,
                    flow: None,
                });
            }
        }
        stocks.0.retain(|_, h| h.quantity != 0);
        Ok((next, stocks, lines))
    }
}
