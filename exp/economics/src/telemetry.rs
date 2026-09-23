//! External observer: only the public, real simulation is stepped here.
//! No hooks are installed in policies, settlement, or private forecast branches.
use crate::{model::AgentId, simulation::Simulation, town_market};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
};

mod observers;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlanningDetail {
    #[default]
    Off,
    Selected,
    Alternatives,
}

const SCHEMA_VERSION: u32 = 1;
const DEFAULT_LOG_LIMIT: usize = 100_000;

#[derive(Clone, Debug)]
pub struct Config {
    pub planning: PlanningDetail,
    pub settlement: bool,
    pub metrics: bool,
    pub logs: bool,
    /// Empty means all agents. Market totals remain whole-market observations.
    pub agents: BTreeSet<AgentId>,
    pub first_month: u32,
    pub last_month: Option<u32>,
    /// Sample metrics relative to first_month; logs are not sampled.
    pub metric_every: u32,
    pub log_limit: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            planning: PlanningDetail::Off,
            settlement: false,
            metrics: true,
            logs: true,
            agents: BTreeSet::new(),
            first_month: 0,
            last_month: None,
            metric_every: 1,
            log_limit: DEFAULT_LOG_LIMIT,
        }
    }
}

/// One output stream per run/continuation segment. Attach before the first
/// observed step; existing history is intentionally not exported.
pub struct Observer<W: Write> {
    writer: W,
    config: Config,
    run: String,
    written_logs: usize,
    omitted_logs: usize,
    failed: bool,
    attached: bool,
    pending: Vec<observers::Pending>,
    flows: BTreeMap<(u32, AgentId, u32), (i64, i64)>,
}
impl<W: Write> Observer<W> {
    pub fn new(writer: W, run: impl Into<String>, config: Config) -> Result<Self, String> {
        if config.metric_every == 0
            || config
                .last_month
                .is_some_and(|last| last < config.first_month)
        {
            return Err("invalid telemetry month range or metric cadence".into());
        }
        let mut observer = Self {
            writer,
            config,
            run: run.into(),
            written_logs: 0,
            omitted_logs: 0,
            failed: false,
            attached: false,
            pending: vec![],
            flows: BTreeMap::new(),
        };
        observer.write(json!({"kind":"start", "config": {
            "planning":format!("{:?}",observer.config.planning), "settlement":observer.config.settlement,
            "metrics":observer.config.metrics, "logs":observer.config.logs,
            "agents":observer.config.agents, "first_month":observer.config.first_month,
            "last_month":observer.config.last_month, "metric_every":observer.config.metric_every,
            "log_limit":observer.config.log_limit
        }}))?;
        Ok(observer)
    }
    fn write(&mut self, mut record: Value) -> Result<(), String> {
        record["schema"] = json!(SCHEMA_VERSION);
        record["run"] = json!(self.run);
        let result = serde_json::to_writer(&mut self.writer, &record)
            .map_err(|e| e.to_string())
            .and_then(|()| self.writer.write_all(b"\n").map_err(|e| e.to_string()));
        if let Err(error) = result {
            self.failed = true;
            return Err(format!(
                "telemetry output failed: {error}; simulation work already committed is not rolled back"
            ));
        }
        Ok(())
    }
    fn month(&self, month: u32) -> bool {
        month >= self.config.first_month && self.config.last_month.is_none_or(|last| month <= last)
    }
    fn agent(&self, agent: AgentId) -> bool {
        self.config.agents.is_empty() || self.config.agents.contains(&agent)
    }
    fn metric(&self, month: u32) -> bool {
        self.config.metrics
            && self.month(month)
            && (month - self.config.first_month).is_multiple_of(self.config.metric_every)
    }
    fn log(&mut self, record: Value) -> Result<(), String> {
        if self.written_logs < self.config.log_limit {
            self.write(record)?;
            self.written_logs += 1;
        } else {
            self.omitted_logs += 1;
        }
        Ok(())
    }
    /// Inspect all newly appended committed records, including when a public
    /// step commits more than one batch. Never inspect pending production plans.
    pub fn step(&mut self, sim: &mut Simulation) -> Result<(), String> {
        if self.failed {
            return Err(
                "telemetry output previously failed; replace observer before advancing".into(),
            );
        }
        if !self.attached {
            self.write(json!({"kind":"attached", "month":sim.state.month,
                "phase":format!("{:?}", sim.state.phase), "next_batch":sim.state.next_batch,
                "backend":format!("{:?}", sim.backend),
                "resources":sim.world.resources.iter().map(|r| json!({"id":r.id,"name":r.name,
                    "kind":format!("{:?}",r.kind)})).collect::<Vec<_>>(),
                "agents":sim.world.agents.iter().map(|a| json!({"id":a.id,"name":a.name})).collect::<Vec<_>>()
            }))?;
            self.attached = true;
        }
        let batches = sim.ledger.len();
        let reports = sim.reports.len();
        let month = sim.state.month;
        let phase = format!("{:?}", sim.state.phase);
        let result = sim.step();
        for batch in &sim.ledger[batches..] {
            if self.month(batch.month) {
                for record in observers::batch(&self.config, &sim.world, batch, &mut self.pending) {
                    self.log(record)?;
                }
            }
            observers::outcomes(batch, &mut self.pending);
            if self.metric(batch.month) {
                for effect in batch.transactions.iter().flat_map(|t| &t.effects) {
                    if self.agent(effect.account.0) {
                        let flow = self
                            .flows
                            .entry((batch.month, effect.account.0, effect.account.1))
                            .or_default();
                        if effect.delta < 0 {
                            flow.1 -= i64::from(effect.delta);
                        } else {
                            flow.0 += i64::from(effect.delta);
                        }
                    }
                }
            }
            if self.config.logs && self.month(batch.month) {
                self.log(
                    json!({"kind":"batch", "month":batch.month, "phase":format!("{:?}",batch.phase),
                    "batch":batch.id, "transactions":batch.transactions.len()}),
                )?;
                for (index, transaction) in batch.transactions.iter().enumerate() {
                    let relevant = self.config.agents.is_empty()
                        || transaction.effects.iter().any(|e| self.agent(e.account.0))
                        || transaction
                            .process
                            .as_ref()
                            .is_some_and(|p| self.agent(p.after.operator));
                    if !relevant {
                        continue;
                    }
                    // Retain both sides of a selected transaction for reconciliation.
                    let effects: Vec<_> = transaction.effects.iter().map(|e|
                        json!({"agent":e.account.0, "resource":e.account.1, "delta":e.delta})).collect();
                    self.log(json!({"kind":"transaction", "month":batch.month,
                    "phase":format!("{:?}",batch.phase), "batch":batch.id, "index":index,
                    "cause":transaction.cause, "effects":effects,
                    "process":transaction.process.as_ref().map(|p| json!({
                        "id":p.after.id,"operator":p.after.operator,
                        "definition":p.after.definition,"status":format!("{:?}",p.after.status)
                    }))}))?;
                }
            }
            if self.metric(batch.month)
                && let Some(town_market::Boundary::Market(round)) = &batch.town_market
            {
                for (market, result) in &round.markets {
                    self.write(
                        json!({"kind":"market", "month":batch.month,"batch":batch.id,
                        "market":market,"scope":"whole_market", "volume":result.volume,
                        "posted_price":result.posted_price,"unfilled_buy":result.unfilled_buy,
                        "unfilled_sell":result.unfilled_sell}),
                    )?;
                }
            }
        }
        observers::reports(&sim.reports[reports..], &mut self.pending);
        if sim.state.month > month {
            for record in observers::close(month, &mut self.pending) {
                if self.month(month) {
                    self.log(record)?;
                }
            }
        }
        for report in &sim.reports[reports..] {
            if !self.metric(report.month) || !self.agent(report.agent) {
                continue;
            }
            for (resource, need) in &report.needs {
                self.write(
                    json!({"kind":"need", "month":report.month,"agent":report.agent,
                    "resource":resource,"desired":need.desired,"fulfilled":need.fulfilled,
                    "deficit":need.deficit}),
                )?;
            }
            self.write(
                json!({"kind":"agent_status","month":report.month,"agent":report.agent,
                "terminal":report.terminal.is_some()}),
            )?;
        }
        if sim.state.month > month && self.metric(month) {
            for ((month, agent, resource), (credits, debits)) in std::mem::take(&mut self.flows) {
                self.write(json!({"kind":"account_flow","month":month,"agent":agent,
                    "resource":resource,"credits":credits,"debits":debits}))?;
            }
            let mut processes = BTreeMap::<(AgentId, String), usize>::new();
            for process in sim.state.processes.values() {
                if self.agent(process.operator) {
                    *processes
                        .entry((process.operator, format!("{:?}", process.status)))
                        .or_default() += 1;
                }
            }
            for ((agent, status), count) in processes {
                self.write(json!({"kind":"process_count","month":month,"agent":agent,
                    "status":status,"count":count}))?;
            }
            for (&(agent, resource), &quantity) in &sim.state.balances {
                if self.agent(agent) {
                    self.write(json!({"kind":"balance","month":month,"agent":agent,
                        "resource":resource,"quantity":quantity}))?;
                }
            }
        }
        if let Err(error) = &result
            && (self.config.logs || self.config.settlement)
            && self.month(month)
        {
            self.log(
                json!({"kind":"step_error","month":month,"phase":phase,"message":error,
                "committed_batches":sim.ledger.len()-batches}),
            )?;
        }
        result
    }
    pub fn run_months(&mut self, sim: &mut Simulation, months: u32) -> Result<(), String> {
        let end = sim
            .state
            .month
            .checked_add(months)
            .ok_or("month overflow")?;
        while sim.state.month < end {
            self.step(sim)?;
        }
        Ok(())
    }
    /// Explicit finish makes truncated logs and flush failures visible. No Drop I/O.
    pub fn finish(mut self) -> Result<W, String> {
        if self.failed {
            return Err("telemetry output previously failed".into());
        }
        self.write(json!({"kind":"finish","written_logs":self.written_logs,
            "omitted_logs":self.omitted_logs,"pending_plan_outcomes":self.pending.len()}))?;
        self.writer
            .flush()
            .map_err(|e| format!("telemetry flush failed: {e}; simulation is not rolled back"))?;
        Ok(self.writer)
    }
}
