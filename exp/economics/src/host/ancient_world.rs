//! Read-only bridge from Ancient World's settled named work to a private economy.
//! These are historical allocation comparisons, never permission to spend the
//! source history's people, stock or money again.
use super::{Boundary, CapacityLimit, MonthOutcome, Opening, Settlement, Stage};
use crate::model::{AgentId, ResourceId};
use ::ancient_world::{
    civilization::History,
    participation::{Activity, Presence},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonBinding {
    pub person: u32,
    /// Only this existing allocation pool contributes to the replay grant.
    pub activity: Activity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub agent: AgentId,
    /// None is permitted only for a zero-endowment experimental participant.
    pub source: Option<PersonBinding>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaborBindings {
    pub site: u32,
    /// The single experimental capacity resource represented by this labor pool.
    pub capacity_resource: ResourceId,
    /// Explicit units for the experimental capacity resource. Must be a power
    /// of two so scaling the host's binary f32 worker-months is exact in f64.
    pub ticks_per_worker_month: u32,
    pub people: Vec<Binding>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SourceShare {
    /// Index in this month's Participation.commitments, not a global ID.
    pub commitment: usize,
    pub reserved_worker_months: f64,
    /// Quantized share after enforcing this commitment's common grant ceiling.
    pub capacity_ticks: u64,
    /// Same proportional attribution used by the main participation settlement.
    /// It is diagnostic; it never substitutes for the original reservation.
    pub completed_worker_months: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LaborReceipt {
    pub agent: AgentId,
    pub source: Option<PersonBinding>,
    /// Opening participation membership; no household money/property is copied.
    pub household: Option<u32>,
    pub sources: Vec<SourceShare>,
    pub reserved_worker_months: f64,
    /// Whole ticks expressed in worker-months after shared and personal limits,
    /// before applying the experimental catalog ceiling.
    pub bounded_worker_months: f64,
    pub capacity_ticks: i32,
    /// Includes quantization, the catalog ceiling and any conservative bounding.
    /// It is never pooled across people or carried to another month.
    pub unused_worker_months: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReplayInput {
    pub opening: Opening,
    pub ticks_per_worker_month: u32,
    pub labor: Vec<LaborReceipt>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReplayOutcome {
    pub input: ReplayInput,
    pub economy: MonthOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceIdentity {
    seed: u32,
    source_epoch: u32,
    terrain_resolution: u32,
    site: (u32, u32, u32, u32),   // ID, cell, civilization, founding month
    people: Vec<(u32, i32, u32)>, // ID, birth month, civilization
}

/// Fixed bindings and available source identity for a sequence of replay months.
/// This detects accidental mixing, not authenticity or same-seed branch lineage.
#[derive(Clone, Debug)]
pub struct LaborReplay {
    bindings: LaborBindings,
    source: SourceIdentity,
    economy: Settlement,
}

impl LaborReplay {
    pub fn attach(
        bindings: LaborBindings,
        history: &History,
        economy: Settlement,
    ) -> Result<Self, String> {
        bindings.capture(history, &economy)?;
        let source = bindings.source_identity(history)?;
        Ok(Self {
            bindings,
            source,
            economy,
        })
    }

    pub fn economy(&self) -> &Settlement {
        &self.economy
    }

    /// The source History is read-only. Mapping, lineage or accounting failure
    /// publishes no engine work; successful prior replay months remain committed.
    pub fn advance(&mut self, history: &History) -> Result<ReplayOutcome, String> {
        if self.bindings.source_identity(history)? != self.source {
            return Err("labor replay source identity changed".into());
        }
        let input = self.bindings.capture(history, &self.economy)?;
        let economy = self.economy.advance_month(input.opening.clone())?;
        Ok(ReplayOutcome { input, economy })
    }
}

fn ticks(worker_months: f64, scale: u32) -> Result<u64, String> {
    let n = (worker_months * f64::from(scale)).floor();
    if !n.is_finite() || n < 0. || n >= u64::MAX as f64 {
        return Err("host labor quantity exceeds conversion range".into());
    }
    Ok(n as u64)
}

impl LaborBindings {
    fn source_identity(&self, history: &History) -> Result<SourceIdentity, String> {
        let site = history
            .sites
            .get(self.site as usize)
            .filter(|s| s.id == self.site)
            .ok_or("missing replay source site")?;
        let mut people = self
            .people
            .iter()
            .filter_map(|b| b.source.as_ref())
            .map(|b| {
                history
                    .people
                    .get(b.person as usize)
                    .filter(|p| p.id == b.person)
                    .map(|p| (p.id, p.born, p.civilization))
                    .ok_or("missing replay source person")
            })
            .collect::<Result<Vec<_>, _>>()?;
        people.sort_unstable();
        Ok(SourceIdentity {
            seed: history.seed,
            source_epoch: history.source_epoch,
            terrain_resolution: history.terrain_resolution,
            site: (site.id, site.cell, site.civilization, site.founded),
            people,
        })
    }
    /// Capture current dated, settled reservations. Main-engine availability and
    /// completed output are not new grants. All participants must be mapped;
    /// missing work for an existing mapped person produces an explicit zero.
    pub fn capture(&self, history: &History, economy: &Settlement) -> Result<ReplayInput, String> {
        let boundary = Boundary {
            site: self.site,
            month: history.month,
        };
        if economy.stage() != Stage::Open || boundary != economy.boundary() {
            return Err("labor replay requires the matching site/month at engine Open".into());
        }
        if !self.ticks_per_worker_month.is_power_of_two() {
            return Err("capacity ticks per worker-month must be a positive power of two".into());
        }
        if !history
            .sites
            .get(self.site as usize)
            .is_some_and(|s| s.id == self.site && s.founded <= history.month)
        {
            return Err("missing or future-founded labor replay site".into());
        }
        let participation = history
            .participation
            .as_ref()
            .ok_or("named participation is required")?;
        if participation.month != Some(history.month) {
            return Err("stale host participation month".into());
        }
        // Reuse the authoritative validator, including shared per-person totals.
        participation.validate(history).map_err(|e| e.to_string())?;
        if participation
            .commitments
            .iter()
            .any(|c| c.site == self.site && !c.settled)
        {
            return Err("historical labor replay requires settled site commitments".into());
        }
        let ceilings: BTreeMap<_, _> = economy
            .capacity_ceilings()
            .into_iter()
            .map(|c| (c.agent, c))
            .collect();
        if ceilings
            .values()
            .any(|c| c.resource != self.capacity_resource)
        {
            return Err(
                "labor bridge requires the declared capacity resource for every participant".into(),
            );
        }
        // Float host shares may add to slightly more than their f32 team grant.
        // Round each share down, then cap against the single whole-tick budget
        // in the host's validated person-ID order, including unmapped people.
        let mut source_ticks = Vec::with_capacity(participation.commitments.len());
        for commitment in &participation.commitments {
            let mut remaining = ticks(f64::from(commitment.granted), self.ticks_per_worker_month)?;
            let mut shares = BTreeMap::new();
            for (person, share) in &commitment.people {
                let quantity =
                    ticks(f64::from(*share), self.ticks_per_worker_month)?.min(remaining);
                shares.insert(*person, quantity);
                remaining -= quantity;
            }
            source_ticks.push(shares);
        }
        let mut agents = BTreeSet::new();
        let mut persons = BTreeSet::new();
        let mut capacities = Vec::new();
        let mut labor = Vec::new();
        for binding in &self.people {
            let ceiling = ceilings
                .get(&binding.agent)
                .ok_or("unknown experimental participant")?;
            if !agents.insert(binding.agent) {
                return Err("duplicate experimental participant binding".into());
            }
            let mut receipt = LaborReceipt {
                agent: binding.agent,
                source: binding.source.clone(),
                household: None,
                sources: Vec::new(),
                reserved_worker_months: 0.,
                bounded_worker_months: 0.,
                capacity_ticks: 0,
                unused_worker_months: 0.,
            };
            match &binding.source {
                None if ceiling.quantity != 0 => {
                    return Err("positive endowment requires a named host person".into());
                }
                None => {}
                Some(source) => {
                    if !persons.insert(source.person) {
                        return Err(
                            "one host person cannot endow multiple experimental participants"
                                .into(),
                        );
                    }
                    let person = history
                        .people
                        .get(source.person as usize)
                        .filter(|p| p.id == source.person)
                        .ok_or("missing host person identity")?;
                    let resident = participation
                        .residents
                        .get(&source.person)
                        .ok_or("missing host participation identity")?;
                    // Presence belongs to the original reservation boundary. A
                    // later Respond move/death does not rewrite historical work.
                    if resident.presence != Presence::Resident(self.site)
                        || i64::from(person.born) > i64::from(history.month)
                        || person.died.is_some_and(|m| m < history.month)
                    {
                        return Err(
                            "person was not eligible at the recorded reservation boundary".into(),
                        );
                    }
                    receipt.household = resident.household;
                    for (id, commitment) in participation
                        .commitments
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.site == self.site && c.activity == source.activity)
                    {
                        if let Some((_, share)) =
                            commitment.people.iter().find(|(p, _)| *p == source.person)
                        {
                            receipt.sources.push(SourceShare {
                                commitment: id,
                                reserved_worker_months: f64::from(*share),
                                capacity_ticks: source_ticks[id][&source.person],
                                completed_worker_months: f64::from(
                                    *share * commitment.used / commitment.granted,
                                ),
                            });
                        }
                    }
                    receipt.reserved_worker_months = receipt
                        .sources
                        .iter()
                        .map(|s| s.reserved_worker_months)
                        .sum();
                    if receipt.reserved_worker_months > 0.
                        && ::ancient_world::population_registry::age_band(
                            history.month,
                            person.born,
                        ) != Some(1)
                    {
                        return Err("positive host labor requires a working-age person".into());
                    }
                    // The host uses f32 with a validation tolerance. Never round
                    // a tolerated excess up into additional spendable work.
                    let shared_ticks = receipt.sources.iter().try_fold(0_u64, |sum, s| {
                        sum.checked_add(s.capacity_ticks)
                            .ok_or("host labor tick sum overflow")
                    })?;
                    let personal_ticks = ticks(
                        f64::from(resident.capacity.min(resident.committed)),
                        self.ticks_per_worker_month,
                    )?;
                    let bounded_ticks = shared_ticks.min(personal_ticks);
                    receipt.bounded_worker_months =
                        bounded_ticks as f64 / f64::from(self.ticks_per_worker_month);
                    receipt.capacity_ticks = bounded_ticks.min(ceiling.quantity as u64) as i32;
                    let represented =
                        f64::from(receipt.capacity_ticks) / f64::from(self.ticks_per_worker_month);
                    receipt.unused_worker_months = receipt.reserved_worker_months - represented;
                }
            }
            capacities.push(CapacityLimit {
                agent: binding.agent,
                resource: ceiling.resource,
                quantity: receipt.capacity_ticks,
            });
            labor.push(receipt);
        }
        if agents.len() != ceilings.len() {
            return Err("labor bindings must cover every experimental participant".into());
        }
        capacities.sort_by_key(|c| c.agent);
        labor.sort_by_key(|r| r.agent);
        Ok(ReplayInput {
            opening: Opening {
                boundary,
                capacities,
            },
            ticks_per_worker_month: self.ticks_per_worker_month,
            labor,
        })
    }
}
