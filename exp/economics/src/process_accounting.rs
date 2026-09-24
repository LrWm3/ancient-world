//! Material-cost recognition for owner-operated processes; no imputed labor income.
use crate::{
    accounting::{self, Account, Line},
    inventory_accounting::{Holding, Inventory},
    model::*,
};
use std::collections::BTreeMap;

/// Financial output identity: resource IDs and durable kind IDs are distinct namespaces.
/// Stable order assigns rounding ticks to stocks by ID, then durable kinds by ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Output {
    Stock(ResourceId),
    Durable(u32),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Costs {
    /// Relative total cost shares by output identity, required for joint products.
    pub output_weights: BTreeMap<DefinitionId, BTreeMap<Output, u32>>,
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
    /// Validated title-following attachments transfer sunk cost, never debt value.
    pub(crate) fn transfer_attachments(
        &self,
        changes: &[ProcessChange],
    ) -> Result<(Self, Vec<Line>), String> {
        let mut next = self.clone();
        let mut lines = vec![];
        for change in changes {
            let before = change
                .before
                .as_ref()
                .ok_or("attachment transfer needs opening process")?;
            let after = &change.after;
            if before.operator != before.beneficiary
                || after.operator != after.beneficiary
                || before.status != Status::Active
                || after.status != Status::Active
                || before.id != after.id
            {
                return Err("unsupported attachment cost transfer".into());
            }
            if before.operator == after.operator {
                continue;
            }
            let (owner, cost) = next
                .work
                .get(&before.id)
                .copied()
                .unwrap_or((before.operator, 0));
            if owner != before.operator {
                return Err("attachment cost owner mismatch".into());
            }
            next.work.insert(after.id, (after.operator, cost));
            if cost != 0 {
                lines.push(Line {
                    agent: before.operator,
                    account: Account::TransferExpense,
                    debit: cost,
                    flow: None,
                });
                lines.push(Line {
                    agent: after.operator,
                    account: Account::TransferIncome,
                    debit: -cost,
                    flow: None,
                });
            }
        }
        Ok((next, lines))
    }
    pub fn settle(
        &self,
        world: &World,
        inventory: &Inventory,
        transactions: &[&Transaction],
        coin: ResourceId,
    ) -> Result<(Self, Inventory, Vec<Line>), String> {
        if transactions.iter().any(|t| {
            t.process.as_ref().is_some_and(|p| {
                matches!(
                    world.activities.outcomes.get(&p.after.definition),
                    Some(crate::activities::Outcome::Create(_))
                )
            })
        }) {
            return Err("durable output requires the equipment cost adapter".into());
        }
        self.settle_with_equipment(
            world,
            inventory,
            transactions,
            coin,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
        )
    }
    pub(crate) fn settle_with_equipment(
        &self,
        world: &World,
        inventory: &Inventory,
        transactions: &[&Transaction],
        coin: ResourceId,
        wear: &BTreeMap<u64, i128>,
        asset_values: &mut BTreeMap<AssetId, i128>,
    ) -> Result<(Self, Inventory, Vec<Line>), String> {
        self.settle_allocated(
            world,
            inventory,
            transactions,
            coin,
            wear,
            asset_values,
            &mut crate::inventory_accounting::CostAllocation::new(inventory),
        )
    }
    // The extra argument is the shared boundary cursor, not a second cost policy.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn settle_allocated(
        &self,
        world: &World,
        inventory: &Inventory,
        transactions: &[&Transaction],
        coin: ResourceId,
        wear: &BTreeMap<u64, i128>,
        asset_values: &mut BTreeMap<AssetId, i128>,
        allocation: &mut crate::inventory_accounting::CostAllocation,
    ) -> Result<(Self, Inventory, Vec<Line>), String> {
        let mut next = self.clone();
        let mut stocks = inventory.clone();
        let mut lines = vec![];
        let mut used = BTreeMap::new();
        let mut released = BTreeMap::new();
        let mut inputs = wear.clone();
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
                let shared_input = e.delta < 0
                    && world
                        .pool_inputs
                        .iter()
                        .any(|i| i.definition == p.definition && i.account == e.account);
                if e.account.1 == coin || (e.account.0 != p.operator && !shared_input) {
                    return Err(
                        "coin or unauthorized third-party process input/output unsupported".into(),
                    );
                }
                if e.delta < 0 {
                    let quantity = -i128::from(e.delta);
                    let cost = allocation.take(e.account, quantity)?;
                    accounting::add(&mut used, e.account, quantity)?;
                    accounting::add(&mut released, e.account, cost)?;
                    accounting::add(&mut inputs, p.id, cost)?;
                    if e.account.0 != p.operator && cost != 0 {
                        lines.extend([
                            Line {
                                agent: e.account.0,
                                account: Account::TransferExpense,
                                debit: cost,
                                flow: None,
                            },
                            Line {
                                agent: p.operator,
                                account: Account::TransferIncome,
                                debit: -cost,
                                flow: None,
                            },
                        ]);
                    }
                }
            }
        }
        for (key, q) in used {
            let cost = released[&key];
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
                    accounting::add(
                        &mut outputs,
                        Output::Stock(e.account.1),
                        i128::from(e.delta),
                    )?;
                }
            }
            if p.status == Status::Completed
                && let Some(crate::activities::Outcome::Create(kind)) =
                    world.activities.outcomes.get(&p.definition)
            {
                outputs.insert(Output::Durable(*kind), 1);
            }
            if p.status == Status::Completed
                && let Some(weights) = self.output_weights.get(&d.id)
                && (weights.len() != outputs.len()
                    || weights
                        .iter()
                        .any(|(output, weight)| *weight == 0 || !outputs.contains_key(output)))
            {
                return Err("output cost shares must cover actual joint products".into());
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
                        let weights = if let Some(weights) = self.output_weights.get(&d.id) {
                            weights.clone()
                        } else if outputs.len() == 1 {
                            BTreeMap::from([(*outputs.keys().next().ok_or("missing output")?, 1)])
                        } else {
                            return Err("joint outputs, including joint durable/stock, require explicit cost shares".into());
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
                            let value = share - allocated;
                            match resource {
                                Output::Stock(resource) => {
                                    let h = stocks.0.entry((p.beneficiary, resource)).or_insert(
                                        Holding {
                                            quantity: 0,
                                            cost: 0,
                                        },
                                    );
                                    h.quantity = i32::try_from(
                                        i128::from(h.quantity)
                                            .checked_add(quantity)
                                            .ok_or("output quantity overflow")?,
                                    )
                                    .map_err(|_| "output quantity overflow")?;
                                    h.cost =
                                        h.cost.checked_add(value).ok_or("output cost overflow")?;
                                }
                                Output::Durable(_) => {
                                    let id = crate::activities::produced_asset_id(p.id)?;
                                    if asset_values.insert(id, value).is_some() {
                                        return Err("produced asset already valued".into());
                                    }
                                }
                            }
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
