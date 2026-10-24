//! Checked single-site embedding boundary. The existing simulation and audit own
//! economic state; host receipts describe committed results, not new transfers.
use crate::{
    financial_reporting::Audit,
    model::{Account, AgentId, DefinitionId, Execution, Phase, ResourceId, ResourceKind, Status},
    simulation::Simulation,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boundary {
    pub site: u32,
    /// Host calendar month. Local agreement/process dates remain unchanged.
    pub month: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    Open,
    Reserve,
    Execute,
    Respond,
    Close,
}

/// Ceiling on this month's own endowment, before the engine's condition modifier.
/// This is not purchased labor and does not add to an organization's bought hours.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityLimit {
    pub agent: AgentId,
    pub resource: ResourceId,
    pub quantity: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opening {
    pub boundary: Boundary,
    /// Exactly one entry per participant, including explicit zero availability.
    /// The host must grant these units exclusively, not reuse them in other work.
    pub capacities: Vec<CapacityLimit>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageReceipt {
    pub boundary: Boundary,
    pub local_month: u32,
    pub stage: Stage,
    /// Half-open range of newly committed ledger batch IDs. Respond is empty.
    pub first_batch: u64,
    pub next_batch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockChange {
    pub agent: AgentId,
    pub resource: ResourceId,
    pub opening: i32,
    pub closing: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeedOutcome {
    pub agent: AgentId,
    pub resource: ResourceId,
    pub required: i32,
    pub fulfilled: i32,
    pub deficit: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Completion {
    pub process: u64,
    pub operator: AgentId,
    pub definition: DefinitionId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonthOutcome {
    pub boundary: Boundary,
    pub local_month: u32,
    pub stages: Vec<StageReceipt>,
    /// Only stock accounts that actually changed; capacities are never inventory.
    pub stocks: Vec<StockChange>,
    pub needs: Vec<NeedOutcome>,
    /// Actual productive completions, not requests or forecast completions.
    pub completions: Vec<Completion>,
}

/// Owns both the engine and its historical reporting book. Immutable accessors
/// prevent callers from changing balances between reservation and execution.
/// Clone is an in-memory checkpoint; the serializable DTOs are NOT a full save.
#[derive(Clone, Debug)]
pub struct Settlement {
    sim: Simulation,
    audit: Audit,
    boundary: Boundary,
    stage: Stage,
    opening_stocks: BTreeMap<Account, i32>,
    stages: Vec<StageReceipt>,
    first_ledger: usize,
    first_report: usize,
}

impl Settlement {
    /// Attach at an existing local month opening without resetting clocks, books,
    /// accepted obligations, controller memory or ongoing production.
    pub fn attach(boundary: Boundary, sim: Simulation, audit: Audit) -> Result<Self, String> {
        if boundary.month == 0 || sim.state.month == 0 || sim.state.phase != Phase::Open {
            return Err(
                "host attachment requires positive months and a local Open boundary".into(),
            );
        }
        // Disabling births still permits known children to mature at Open.
        // Host identity/labor rebinding for that transition is a separate adapter.
        if sim.world.demography.is_some() {
            return Err("host capacity binding requires a fixed participant roster".into());
        }
        crate::settlement::validate_world(&sim.world, &sim.state)?;
        audit.validate_boundary(&sim.world, &sim.state)?;
        Ok(Self {
            sim,
            audit,
            boundary,
            stage: Stage::Open,
            opening_stocks: BTreeMap::new(),
            stages: Vec::new(),
            first_ledger: 0,
            first_report: 0,
        })
    }

    pub fn simulation(&self) -> &Simulation {
        &self.sim
    }

    pub fn audit(&self) -> &Audit {
        &self.audit
    }

    pub fn boundary(&self) -> Boundary {
        self.boundary
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// Catalog ceilings for constructing an explicit host input. These values
    /// are not evidence that the host has reserved the corresponding resources.
    pub fn capacity_ceilings(&self) -> Vec<CapacityLimit> {
        let mut result: Vec<_> = self
            .sim
            .world
            .participants
            .iter()
            .map(|p| CapacityLimit {
                agent: p.agent,
                resource: p.capacity.resource,
                quantity: p.capacity.quantity,
            })
            .collect();
        result.sort_by_key(|c| c.agent);
        result
    }

    fn check(&self, boundary: Boundary, stage: Stage) -> Result<(), String> {
        if boundary != self.boundary || stage != self.stage {
            return Err("stale, wrong-site or out-of-order host boundary".into());
        }
        Ok(())
    }

    pub fn open(&mut self, input: Opening) -> Result<StageReceipt, String> {
        self.check(input.boundary, Stage::Open)?;
        // Check the entire month can close before publishing any part of it.
        self.boundary
            .month
            .checked_add(1)
            .ok_or("host month overflow")?;
        self.sim
            .state
            .month
            .checked_add(1)
            .ok_or("local month overflow")?;
        let expected: BTreeMap<_, _> = self
            .capacity_ceilings()
            .into_iter()
            .map(|c| (c.agent, c))
            .collect();
        let mut seen = BTreeSet::new();
        for c in &input.capacities {
            let ceiling = expected
                .get(&c.agent)
                .ok_or("unknown host capacity participant")?;
            if !seen.insert(c.agent)
                || c.resource != ceiling.resource
                || c.quantity < 0
                || c.quantity > ceiling.quantity
            {
                return Err("invalid or duplicate host capacity limit".into());
            }
        }
        if seen.len() != expected.len() {
            return Err("host opening must cover every capacity participant".into());
        }
        let mut next = self.clone();
        next.opening_stocks = next.stocks();
        next.first_ledger = next.sim.ledger.len();
        next.first_report = next.sim.reports.len();
        next.stages.clear();
        for c in input.capacities {
            let key = (next.sim.state.month, c.agent);
            // An external grant cannot undo an existing scenario interruption.
            if let Some(quantity) = next.sim.world.capacity_overrides.get_mut(&key) {
                *quantity = (*quantity).min(c.quantity);
            } else if c.quantity < expected[&c.agent].quantity {
                next.sim.world.capacity_overrides.insert(key, c.quantity);
            }
        }
        let receipt = next.commit_stage(Stage::Open)?;
        *self = next;
        Ok(receipt)
    }

    pub fn reserve(&mut self, boundary: Boundary) -> Result<StageReceipt, String> {
        self.advance(boundary, Stage::Reserve)
    }

    pub fn execute(&mut self, boundary: Boundary) -> Result<StageReceipt, String> {
        self.advance(boundary, Stage::Execute)
    }

    /// An explicit visibility boundary; no engine action is moved here. In
    /// particular ordinary employment still pays at the engine's Close phase.
    pub fn respond(&mut self, boundary: Boundary) -> Result<StageReceipt, String> {
        self.advance(boundary, Stage::Respond)
    }

    fn advance(&mut self, boundary: Boundary, stage: Stage) -> Result<StageReceipt, String> {
        self.check(boundary, stage)?;
        let mut next = self.clone();
        let receipt = next.commit_stage(stage)?;
        *self = next;
        Ok(receipt)
    }

    pub fn close(&mut self, boundary: Boundary) -> Result<MonthOutcome, String> {
        self.check(boundary, Stage::Close)?;
        let mut next = self.clone();
        let receipt = next.commit_stage(Stage::Close)?;
        next.audit
            .validate_boundary(&next.sim.world, &next.sim.state)?;
        let closing = next.stocks();
        let keys: BTreeSet<_> = next
            .opening_stocks
            .keys()
            .chain(closing.keys())
            .copied()
            .collect();
        let stocks = keys
            .into_iter()
            .filter_map(|(agent, resource)| {
                let opening = next
                    .opening_stocks
                    .get(&(agent, resource))
                    .copied()
                    .unwrap_or(0);
                let closing = closing.get(&(agent, resource)).copied().unwrap_or(0);
                (opening != closing).then_some(StockChange {
                    agent,
                    resource,
                    opening,
                    closing,
                })
            })
            .collect();
        let needs = next.sim.reports[next.first_report..]
            .iter()
            .flat_map(|r| {
                r.needs.iter().map(move |(resource, n)| NeedOutcome {
                    agent: r.agent,
                    resource: *resource,
                    required: n.desired,
                    fulfilled: n.fulfilled,
                    deficit: n.deficit,
                })
            })
            .collect();
        let completions = next.sim.ledger[next.first_ledger..]
            .iter()
            .filter(|b| b.phase == Phase::Productive)
            .flat_map(|b| b.all_transactions())
            .filter_map(|t| t.process.as_ref())
            .filter(|p| {
                p.after.status == Status::Completed
                    && next.sim.world.definition(p.after.definition).execution
                        == Execution::Productive
            })
            .map(|p| Completion {
                process: p.after.id,
                operator: p.after.operator,
                definition: p.after.definition,
            })
            .collect();
        let outcome = MonthOutcome {
            boundary,
            local_month: receipt.local_month,
            stages: next.stages.clone(),
            stocks,
            needs,
            completions,
        };
        next.boundary.month += 1; // Checked before Open; only Close advances it.
        *self = next;
        Ok(outcome)
    }

    /// Atomic convenience call. Staged hosts can instead interleave their own
    /// operations between the five checked methods above.
    pub fn advance_month(&mut self, input: Opening) -> Result<MonthOutcome, String> {
        let boundary = input.boundary;
        let mut next = self.clone();
        next.open(input)?;
        next.reserve(boundary)?;
        next.execute(boundary)?;
        next.respond(boundary)?;
        let outcome = next.close(boundary)?;
        *self = next;
        Ok(outcome)
    }

    fn stocks(&self) -> BTreeMap<Account, i32> {
        let stocks: BTreeSet<_> = self
            .sim
            .world
            .resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Stock)
            .map(|r| r.id)
            .collect();
        self.sim
            .state
            .balances
            .iter()
            .filter(|((_, r), q)| stocks.contains(r) && **q != 0)
            .map(|(key, q)| (*key, *q))
            .collect()
    }

    fn commit_stage(&mut self, stage: Stage) -> Result<StageReceipt, String> {
        let month = self.sim.state.month;
        let first_batch = self.sim.state.next_batch;
        let (phases, next_stage, end_phases): (&[_], _, &[_]) = match stage {
            Stage::Open => (
                &[Phase::Open, Phase::Due],
                Stage::Reserve,
                &[Phase::Acquire, Phase::Productive],
            ),
            Stage::Reserve => (&[Phase::Acquire], Stage::Execute, &[Phase::Productive]),
            Stage::Execute => (
                &[Phase::Productive, Phase::ClearArrears, Phase::Consumption],
                Stage::Respond,
                &[Phase::Close],
            ),
            Stage::Respond => (&[], Stage::Close, &[Phase::Close]),
            Stage::Close => (&[Phase::Close], Stage::Open, &[Phase::Open]),
        };
        // Phase ownership is fixed; a missing optional Acquire/Due is allowed.
        // No loop may follow Close into a second month.
        while self.sim.state.month == month && phases.contains(&self.sim.state.phase) {
            self.audit.step(&mut self.sim)?;
        }
        let expected_month = month + u32::from(stage == Stage::Close);
        if self.sim.state.month != expected_month || !end_phases.contains(&self.sim.state.phase) {
            return Err("unsupported engine phase transition at host boundary".into());
        }
        let receipt = StageReceipt {
            boundary: self.boundary,
            local_month: month,
            stage,
            first_batch,
            next_batch: self.sim.state.next_batch,
        };
        self.stages.push(receipt.clone());
        self.stage = next_stage;
        Ok(receipt)
    }
}
