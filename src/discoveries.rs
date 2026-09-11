//! Sparse, finite specimen collection and settlement research workshops.
//! The source inventory is a declared accessible frontier baseline, outside managed town plots.
use crate::{
    civilization::History,
    expeditions::{Expedition, Expeditions, Objective},
    gpu::{Cell, Generator},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
pub mod returns;
pub const CNP: [[f64; 3]; 2] = [[0.45, 0.02, 0.003], [0., 0., 0.08]];
pub const NAMES: [&str; 2] = ["faultroot resin", "phosphatic crust"];
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub cell: u32,
    pub initial: [f64; 2],
    pub remaining: [f64; 2],
    pub collected: [f64; 2],
}
/// Requested outcomes before allocation; execution alone updates actual transfers.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResearchOutcomes {
    /// Resin studied, crust studied, remedy made, phosphorus released (kg).
    pub expected: [f64; 4],
    pub actual: [f64; 4],
    pub methods_expected: [bool; 2],
    pub methods_actual: [bool; 2],
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResearchPlan {
    #[serde(default)]
    pub outcomes: Option<ResearchOutcomes>,
    #[serde(default)]
    pub botanical_kg: [f64; 3],
    #[serde(default)]
    pub commitment: Option<u32>,
    pub receipt: crate::labor::WorkReceipt,
    pub teachers: [Option<u32>; 2],
    pub processing_kg: [f64; 2],
    pub cancellation: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workshop {
    #[serde(default)]
    pub botanicals: returns::Botanicals,
    #[serde(default)]
    pub work_plan: Option<ResearchPlan>,
    pub site: u32,
    pub enabled: bool,
    pub processed: [f64; 2],
    /// Cumulative transfer into catalogued unique specimens, no longer workshop stocks.
    #[serde(default)]
    pub curated: [f64; 2],
    pub samples: [f64; 2],
    pub studied: [f64; 2],
    #[serde(default)]
    pub learned: [Option<u64>; 2],
    pub remedy: f64,
    pub delivered: [f64; 2],
    pub causes: [Option<u64>; 2],
    pub batches: [u32; 2],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Discoveries {
    pub version: u32,
    pub started: u32,
    pub sources: Vec<Source>,
    pub workshops: Vec<Workshop>,
    pub collected: [f64; 2],
    pub discarded: [f64; 2],
    pub studied: [f64; 2],
    pub processed: [f64; 2],
    /// Cumulative transfer into catalogued unique specimens, no longer workshop stocks.
    #[serde(default)]
    pub curated: [f64; 2],
    pub remedy_made: f64,
    pub remedy_used: f64,
    pub remedy_expired: f64,
    pub phosphorus_applied: f64,
    pub worker_months: f64,
    pub worker_months_reserved: f64,
}
// Research work passes through f32 labor grants with a minimum useful allocation.
// A sub-milligram remainder must not strand a completed 1.5 kg study forever.
// This is an eligibility tolerance only: never round up consumed mass or ledgers.
const STUDY_KG: f64 = 1.5;
const STUDY_TOLERANCE_KG: f64 = 1e-6;
pub(crate) fn study_complete(kg: f64) -> bool {
    kg >= STUDY_KG - STUDY_TOLERANCE_KG
}
fn method_known(w: &Workshop, kind: usize) -> bool {
    study_complete(w.studied[kind]) || w.learned[kind].is_some()
}
fn processing_limit(w: &Workshop, kind: usize, site: &crate::civilization::Site) -> f64 {
    if !method_known(w, kind) {
        return (STUDY_KG - w.studied[kind]).min(0.25);
    }
    if kind == 0 {
        // Keep a small emergency reserve in healthy towns, two months of doses during illness.
        let target = site.stocks.stock[0] as f64
            * if site.demography.health[0] > 0.01 {
                0.01
            } else {
                0.0005
            };
        return ((target - w.remedy).max(0.) * 2.).min(1.);
    }
    1.
}
fn exchange(h: &mut History, site: u32, kind: usize, kg: f64) {
    for k in 0..3 {
        h.sites[site as usize].economy.external[k] += (CNP[kind][k] * kg) as f32;
    }
}
fn event(h: &mut History, site: u32, cause: Option<u64>, kind: &str, text: String) {
    h.event(kind, Some(site), None, text);
    if let Some(cause) = cause {
        h.events.last_mut().unwrap().causes.push(cause);
    }
}
impl Discoveries {
    pub fn new(started: u32) -> Self {
        Self {
            version: 2,
            started,
            sources: vec![],
            workshops: vec![],
            collected: [0.; 2],
            discarded: [0.; 2],
            studied: [0.; 2],
            processed: [0.; 2],
            curated: [0.; 2],
            remedy_made: 0.,
            remedy_used: 0.,
            remedy_expired: 0.,
            phosphorus_applied: 0.,
            worker_months: 0.,
            worker_months_reserved: 0.,
        }
    }
    pub fn residuals(&self, x: &Expeditions) -> [f64; 3] {
        let mut out = [0.; 3];
        for (k, r) in out[..2].iter_mut().enumerate() {
            let aboard: f64 = x.voyages.iter().map(|e| e.samples[k]).sum();
            let stores: f64 = self.workshops.iter().map(|w| w.samples[k]).sum();
            *r = (self.collected[k]
                - aboard
                - stores
                - self.discarded[k]
                - self.studied[k]
                - self.processed[k]
                - self.curated[k]
                - if k == 0 {
                    self.workshops
                        .iter()
                        .map(|w| w.botanicals.received.iter().sum::<f64>())
                        .sum()
                } else {
                    0.
                })
                / self.collected[k].max(1.);
        }
        out[2] = (self.remedy_made
            - self.remedy_used
            - self.remedy_expired
            - self.workshops.iter().map(|w| w.remedy).sum::<f64>())
            / self.remedy_made.max(1.);
        out
    }
    pub fn held_cnp(&self, x: &Expeditions) -> [f64; 3] {
        let mut held = [0.; 3];
        for (kind, composition) in CNP.iter().enumerate() {
            let kg = x.voyages.iter().map(|e| e.samples[kind]).sum::<f64>()
                + self.workshops.iter().map(|w| w.samples[kind]).sum::<f64>();
            for (v, fraction) in held.iter_mut().zip(composition) {
                *v += kg * fraction;
            }
        }
        for w in &self.workshops {
            for (v, fraction) in held.iter_mut().zip(CNP[0]) {
                *v += w.botanicals.stock.iter().sum::<f64>() * fraction;
            }
        }
        let remedy = self.workshops.iter().map(|w| w.remedy).sum::<f64>();
        for (v, fraction) in held.iter_mut().zip(CNP[0]) {
            *v += remedy * fraction;
        }
        held
    }
    pub fn validate(&self, h: &History, x: &Expeditions, cells: &[Cell]) -> Result<()> {
        ensure!(
            self.version == 2 && self.started >= x.started && self.started <= h.month,
            "invalid specimen baseline"
        );
        let mut sites = std::collections::BTreeSet::new();
        let mut cells_seen = std::collections::BTreeSet::new();
        for source in &self.sources {
            ensure!(
                (source.cell as usize) < cells.len()
                    && cells[source.cell as usize].meta[0] == 3
                    && (h.living.is_some() || cells[source.cell as usize].water[0] < 0.25)
                    && cells_seen.insert(source.cell),
                "invalid or duplicated specimen source"
            );
            for k in 0..2 {
                ensure!(
                    [source.initial[k], source.remaining[k], source.collected[k]]
                        .into_iter()
                        .all(|v| v.is_finite() && v >= 0.)
                        && (source.initial[k] - source.remaining[k] - source.collected[k]).abs()
                            < 1e-6 * source.initial[k].max(1.),
                    "specimen source creates material"
                );
            }
        }
        for w in &self.workshops {
            w.botanicals.validate(h)?;
            if let Some(plan) = &w.work_plan {
                if let Some(outcomes) = &plan.outcomes {
                    ensure!(
                        outcomes
                            .expected
                            .iter()
                            .chain(&outcomes.actual)
                            .all(|v| v.is_finite() && *v >= 0.),
                        "invalid research outcomes"
                    );
                }
                ensure!(
                    plan.botanical_kg
                        .iter()
                        .all(|v| v.is_finite() && *v >= 0. && *v <= 0.25 + 1e-8),
                    "invalid botanical work request"
                );
            }
            ensure!(
                (w.site as usize) < h.sites.len()
                    && sites.insert(w.site)
                    && w.samples
                        .iter()
                        .chain(&w.studied)
                        .chain(&w.delivered)
                        .chain(&w.processed)
                        .chain(&w.curated)
                        .chain([&w.remedy])
                        .all(|v| v.is_finite() && *v >= 0.)
                    && w.studied.iter().all(|v| *v <= 1.5 + 1e-8),
                "invalid research workshop"
            );
            for learned in w.learned.into_iter().flatten() {
                ensure!(
                    h.events.get(learned as usize).is_some_and(|e| e.kind
                        == "specimen_method_transmitted"
                        && e.month <= h.month),
                    "invalid transmitted research"
                );
            }
            for k in 0..2 {
                ensure!(
                    (w.delivered[k]
                        - w.samples[k]
                        - w.studied[k]
                        - w.processed[k]
                        - w.curated[k]
                        - if k == 0 {
                            w.botanicals.received.iter().sum::<f64>()
                        } else {
                            0.
                        })
                    .abs()
                        < 1e-6 * w.delivered[k].max(1.),
                    "workshop specimen ledger mismatch"
                );
                ensure!(
                    (w.delivered[k] == 0. || w.causes[k].is_some())
                        && w.causes[k].is_none_or(|c| (c as usize) < h.events.len()
                            && h.events[c as usize].kind == "specimens_delivered"
                            && h.events[c as usize].site == Some(w.site)),
                    "invalid specimen provenance"
                );
            }
        }
        ensure!(
            self.collected
                .iter()
                .chain(&self.discarded)
                .chain(&self.studied)
                .chain(&self.processed)
                .chain(&self.curated)
                .chain([
                    &self.remedy_made,
                    &self.remedy_used,
                    &self.remedy_expired,
                    &self.phosphorus_applied,
                    &self.worker_months,
                    &self.worker_months_reserved
                ])
                .all(|v| v.is_finite() && *v >= 0.),
            "invalid specimen ledger"
        );
        ensure!(
            self.worker_months
                <= self.worker_months_reserved + 1e-6 * self.worker_months_reserved.max(1.),
            "workshop uses unreserved labor"
        );
        for k in 0..2 {
            ensure!(
                (self.studied[k] - self.workshops.iter().map(|w| w.studied[k]).sum::<f64>()).abs()
                    < 1e-6 * self.studied[k].max(1.)
                    && (self.processed[k]
                        - self.workshops.iter().map(|w| w.processed[k]).sum::<f64>())
                    .abs()
                        < 1e-6 * self.processed[k].max(1.)
                    && (self.curated[k] - self.workshops.iter().map(|w| w.curated[k]).sum::<f64>())
                        .abs()
                        < 1e-6 * self.curated[k].max(1.),
                "research ledger mismatch"
            );
            ensure!(
                (self.collected[k] - self.sources.iter().map(|s| s.collected[k]).sum::<f64>())
                    .abs()
                    < 1e-6 * self.collected[k].max(1.),
                "collection source ledger mismatch"
            );
        }
        ensure!(
            (self.remedy_made - self.processed[0] * 0.5).abs() < 1e-6 * self.remedy_made.max(1.)
                && (self.phosphorus_applied - self.processed[1] * CNP[1][2]).abs()
                    < 1e-6 * self.phosphorus_applied.max(1.),
            "specimen processing yield mismatch"
        );
        ensure!(
            self.residuals(x)
                .iter()
                .all(|v| v.is_finite() && v.abs() < 1e-6),
            "specimen conservation residual exceeds tolerance"
        );
        Ok(())
    }
    pub(crate) fn collect(
        &mut self,
        h: &mut History,
        e: &mut Expedition,
        cell_id: u32,
        cell: &Cell,
    ) {
        if h.living.is_some() && crate::hazards::flood_depth(cell) >= 0.25 {
            return;
        }
        let kind = match e.objective {
            Objective::Ecology => 0,
            Objective::Geology => 1,
            _ => return,
        };
        let index = if let Some(i) = self.sources.iter().position(|s| s.cell == cell_id) {
            i
        } else {
            let activity = cell.geology[0].clamp(0., 1.) as f64;
            let life = cell.life[0].clamp(0., 1.) as f64;
            let bio = if (-5. ..=40.).contains(&cell.climate[0]) && life > 0.15 {
                240. * life * (0.25 + 0.75 * activity)
            } else {
                0.
            };
            let initial = [bio, 400. * activity];
            self.sources.push(Source {
                cell: cell_id,
                initial,
                remaining: initial,
                collected: [0.; 2],
            });
            event(h,e.origin,Some(e.cause),"specimen_source",format!("Accessible coastal baseline at cell {cell_id}: {:.1} kg organic collection material, {:.1} kg phosphatic crust; no replenishment during frozen planetary history",initial[0],initial[1]));
            self.sources.len() - 1
        };
        let capacity = (12. - e.samples.iter().sum::<f64>()).max(0.);
        let kg = (2. * e.research_skill() as f64 * (e.tools as f64 / 12.).min(1.))
            .min(capacity)
            .min(self.sources[index].remaining[kind]);
        if kg <= 0. {
            return;
        }
        let src = &mut self.sources[index];
        src.remaining[kind] -= kg;
        src.collected[kind] += kg;
        self.collected[kind] += kg;
        e.samples[kind] += kg;
        if kind == 0 {
            // Half the accessible organic collection is retained as typed botanical material.
            let profile = (cell_id.wrapping_add(h.seed) % 3) as usize;
            e.botanicals[profile] += kg * 0.5;
            e.botanical_sources[profile] = Some(cell_id);
        }
        exchange(h, e.origin, kind, kg);
        if src.remaining[kind] <= 1e-8 {
            event(
                h,
                e.origin,
                Some(e.cause),
                "specimen_source_depleted",
                format!("Accessible {} exhausted at cell {cell_id}", NAMES[kind]),
            );
        }
    }
    pub(crate) fn discard(&mut self, h: &mut History, e: &mut Expedition) {
        e.botanicals = [0.; 3];
        e.botanical_sources = [None; 3];
        for kind in 0..2 {
            self.discarded[kind] += e.samples[kind];
            exchange(h, e.origin, kind, -e.samples[kind]);
            e.samples[kind] = 0.;
        }
    }
    pub(crate) fn deliver(&mut self, h: &mut History, e: &mut Expedition) {
        if e.samples.iter().sum::<f64>() <= 0. {
            return;
        }
        let i = if let Some(i) = self.workshops.iter().position(|w| w.site == e.origin) {
            i
        } else {
            self.workshops.push(Workshop {
                botanicals: Default::default(),
                work_plan: None,
                site: e.origin,
                enabled: true,
                processed: [0.; 2],
                curated: [0.; 2],
                samples: [0.; 2],
                studied: [0.; 2],
                learned: [None; 2],
                remedy: 0.,
                delivered: [0.; 2],
                causes: [None; 2],
                batches: [0; 2],
            });
            self.workshops.len() - 1
        };
        event(h,e.origin,Some(e.cause),"specimens_delivered",format!("Expedition {} delivered {:.2} kg organic specimens (including typed botanical material) and {:.2} kg phosphatic crust to the research workshop",e.id,e.samples[0],e.samples[1]));
        let cause = h.events.last().unwrap().id;
        for k in 0..3 {
            let kg = e.botanicals[k];
            self.workshops[i].botanicals.stock[k] += kg;
            self.workshops[i].botanicals.received[k] += kg;
            if kg > 0. {
                self.workshops[i].botanicals.causes[k] = Some(cause);
                self.workshops[i].botanicals.sources[k] = e.botanical_sources[k];
                event(
                    h,
                    e.origin,
                    Some(cause),
                    "botanical_collection_received",
                    format!(
                        "Received {kg:.2} kg {}; held for finite trials and processing",
                        returns::NAMES[k]
                    ),
                );
            }
        }
        let botanical_mass = e.botanicals.iter().sum::<f64>();
        e.botanicals = [0.; 3];
        e.botanical_sources = [None; 3];

        for k in 0..2 {
            if e.samples[k] > 0. {
                self.workshops[i].causes[k] = Some(cause);
                self.workshops[i].samples[k] += e.samples[k];
                self.workshops[i].delivered[k] += e.samples[k];
                e.samples[k] = 0.;
            }
        }
        self.workshops[i].samples[0] = (self.workshops[i].samples[0] - botanical_mass).max(0.);
    }
    #[cfg(test)]
    pub(crate) fn month(&mut self, h: &mut History) {
        self.month_in_environment(h, &[]);
    }
    pub(crate) fn month_in_environment(&mut self, h: &mut History, cells: &[Cell]) {
        // Snapshot: transmitted methods cannot cross multiple contacts in one update.
        let teachers = self.workshops.clone();
        for w in &mut self.workshops {
            let site = w.site as usize;
            let personal_limit = w
                .work_plan
                .as_ref()
                .filter(|p| p.commitment.is_some())
                .map_or(f32::MAX, |p| h.personal_grant_live(p.commitment));
            let s = &mut h.sites[site];
            let expired = w.remedy * 0.01;
            w.remedy -= expired;
            self.remedy_expired += expired;
            for k in 0..3 {
                s.economy.external[k] -= (expired * CNP[0][k]) as f32;
            }
            if s.abandoned || !w.enabled {
                if let Some(p) = &mut w.work_plan {
                    p.receipt.settle(0.);
                    p.cancellation = Some("workshop closed or site abandoned".into());
                }
                continue;
            }
            // The GPU has reserved this labor from its normal craft budget for this month.
            let mut labor = (s.economy.external[3].min(s.economy.labor[3]).max(0.)) as f64;
            if let Some(p) = &w.work_plan {
                labor = if p.receipt.month == h.month && !p.receipt.settled {
                    labor.min(p.receipt.granted).min(personal_limit as f64)
                } else {
                    0.
                };
            }
            let before_work = self.worker_months;
            self.worker_months_reserved += labor;
            for (k, name) in NAMES.iter().enumerate() {
                if w.work_plan.as_ref().is_none_or(|p| p.teachers[k].is_some())
                    && h.month.is_multiple_of(12)
                    && !method_known(w, k)
                    && labor >= 0.25
                {
                    if let Some(teacher) = teachers
                        .iter()
                        .filter(|t| {
                            w.work_plan
                                .as_ref()
                                .is_none_or(|p| p.teachers[k] == Some(t.site))
                                && t.site != w.site
                                && !h.sites[t.site as usize].abandoned
                                && method_known(t, k)
                                && h.route_cost(t.site, w.site).is_some()
                        })
                        .min_by_key(|t| t.site)
                    {
                        if let Some(good) = h
                            .economy_catalog
                            .as_ref()
                            .and_then(|c| c.goods.iter().position(|g| g.id == "writing_material"))
                        {
                            let ratios = h.economy_catalog.as_ref().unwrap().composition(good);
                            let e = &mut h.sites[site].economy;
                            if e.goods[good] >= 0.05 {
                                e.goods[good] -= 0.05;
                                e.used[good] += 0.05;
                                e.reserves[3] += 0.05;
                                for (j, r) in ratios.into_iter().enumerate() {
                                    e.detritus[j] += 0.05 * r;
                                }
                                labor -= 0.25;
                                self.worker_months += 0.25;
                                event(h,w.site,teacher.learned[k].or(teacher.causes[k]),
                                    "specimen_method_transmitted",format!("Researchers copied the {} method from {} through an open route; specimens still must be acquired locally",name,h.sites[teacher.site as usize].name));
                                w.learned[k] = h.events.last().map(|e| e.id);
                                if let Some(outcomes) =
                                    w.work_plan.as_mut().and_then(|p| p.outcomes.as_mut())
                                {
                                    outcomes.methods_actual[k] = true;
                                }
                            }
                        }
                    }
                }
                let studying = !method_known(w, k);
                let limit = processing_limit(w, k, &h.sites[site]);
                let e = &mut h.sites[site].economy;
                let kg = w.samples[k]
                    .min(
                        w.work_plan
                            .as_ref()
                            .map_or(f64::INFINITY, |p| p.processing_kg[k]),
                    )
                    .min(limit)
                    .min(labor * 0.5)
                    .min(e.goods[3] as f64 / 0.1)
                    .min(e.goods[6] as f64 / 0.2);
                if kg <= 1e-8 {
                    continue;
                }
                if let Some(outcomes) = w.work_plan.as_mut().and_then(|p| p.outcomes.as_mut()) {
                    if studying {
                        outcomes.actual[k] += kg;
                        outcomes.methods_actual[k] |= study_complete(w.studied[k] + kg);
                    } else {
                        outcomes.actual[2 + k] += kg * if k == 0 { 0.5 } else { CNP[1][2] };
                    }
                }
                w.samples[k] -= kg;
                labor -= kg * 2.;
                self.worker_months += kg * 2.;
                let tools = (kg * 0.1) as f32;
                let fuel = (kg * 0.2) as f32;
                e.goods[3] -= tools;
                e.used[3] += tools;
                e.reserves[3] += tools;
                e.goods[6] -= fuel;
                e.used[6] += fuel;
                e.external[0] -= fuel;
                if studying {
                    w.studied[k] += kg;
                    self.studied[k] += kg;
                    exchange(h, w.site, k, -kg);
                    if study_complete(w.studied[k]) {
                        event(h,w.site,w.causes[k],"specimen_application_discovered",format!("Destructive study of {} established {}; production still requires fresh material, fuel, tools and labor",name,if k==0{"a fictional remedy recipe"}else{"phosphorus extraction"}));
                    }
                } else {
                    self.processed[k] += kg;
                    w.processed[k] += kg;
                    w.batches[k] += 1;
                    if k == 0 {
                        w.remedy += kg * 0.5;
                        self.remedy_made += kg * 0.5;
                        exchange(h, w.site, 0, -kg * 0.5);
                    } else {
                        h.sites[site].economy.soil[2] += (kg * CNP[1][2]) as f32;
                        self.phosphorus_applied += kg * CNP[1][2];
                    }
                    if w.batches[k] == 1 || w.batches[k].is_multiple_of(12) {
                        event(
                            h,
                            w.site,
                            w.causes[k],
                            "specimen_production",
                            format!(
                                "{} batch {} processed {:.2} kg; {}",
                                name,
                                w.batches[k],
                                kg,
                                if k == 0 {
                                    "half retained as remedy; processing losses accounted"
                                } else {
                                    "phosphorus transferred into managed farm soil"
                                }
                            ),
                        );
                    }
                }
            }
            self.worker_months += returns::process(h, w, cells, &mut labor);
            if let Some(p) = &mut w.work_plan {
                if !p.receipt.settled {
                    if p.receipt.month != h.month {
                        p.cancellation = Some("stale month".into());
                    }
                    p.receipt.settle(self.worker_months - before_work);
                    if p.receipt.released > 1e-5 && p.cancellation.is_none() {
                        p.cancellation = Some("planned teacher, supplies or execution labor unavailable; remainder expired".into());
                    }
                }
            }
            let s = &mut h.sites[site];
            if s.demography.health[0] > 0.01 && s.stocks.stock[0] > 0. {
                let demand = s.stocks.stock[0] as f64 * 0.005;
                let used = w.remedy.min(demand);
                w.remedy -= used;
                self.remedy_used += used;
                s.demography.health[0] =
                    (s.demography.health[0] - (0.02 * used / demand) as f32).max(0.);
                for k in 0..3 {
                    s.economy.external[k] -= (used * CNP[0][k]) as f32;
                }
                if used > 0. {
                    event(
                        h,
                        w.site,
                        w.causes[0],
                        "specimen_treatment",
                        format!(
                            "Consumed {used:.2} kg fictional remedy; reduced local disease burden"
                        ),
                    );
                }
            }
        }
    }
}
// Read-only feasible work forecast; shared tools, fuel and writing stock are
// deducted locally so multiple activities cannot each claim the same supplies.
fn plan_work(h: &History, workshop: &Workshop, teachers: &[Workshop]) -> ResearchPlan {
    let mut plan = ResearchPlan {
        outcomes: Some(ResearchOutcomes::default()),
        ..Default::default()
    };
    plan.receipt.month = h.month;
    let site = &h.sites[workshop.site as usize];
    if site.abandoned || !workshop.enabled {
        return plan;
    }
    let budget =
        crate::labor::available(site, h.society.is_some(), h.living.is_some()).min(2.) as f64;
    let mut w = workshop.clone();
    let mut tools = site.economy.goods[3] as f64;
    let mut fuel = site.economy.goods[6] as f64;
    let mut writing = h
        .economy_catalog
        .as_ref()
        .and_then(|c| c.index("writing_material"))
        .map_or(0., |g| site.economy.goods[g] as f64);
    let mut work = 0.;
    for k in 0..2 {
        if budget - work >= 0.25
            && h.month.is_multiple_of(12)
            && !method_known(&w, k)
            && writing >= 0.05
            && teachers.iter().any(|t| {
                t.site != w.site
                    && !h.sites[t.site as usize].abandoned
                    && method_known(t, k)
                    && h.route_cost(t.site, w.site).is_some()
            })
        {
            plan.teachers[k] = teachers
                .iter()
                .filter(|t| {
                    t.site != w.site
                        && !h.sites[t.site as usize].abandoned
                        && method_known(t, k)
                        && h.route_cost(t.site, w.site).is_some()
                })
                .map(|t| t.site)
                .min();
            work += 0.25;
            writing -= 0.05;
            w.learned[k] = Some(0); // Forecast only; no historical discovery is committed.
            plan.outcomes.as_mut().unwrap().methods_expected[k] = true;
        }
        let kg = w.samples[k]
            .min(processing_limit(&w, k, site))
            .min(tools / 0.1)
            .min(fuel / 0.2)
            .min((budget - work).max(0.) * 0.5);
        plan.processing_kg[k] = kg;
        let outcomes = plan.outcomes.as_mut().unwrap();
        if !method_known(&w, k) {
            outcomes.expected[k] = kg;
            outcomes.methods_expected[k] |= study_complete(w.studied[k] + kg);
        } else {
            outcomes.expected[2 + k] = kg * if k == 0 { 0.5 } else { CNP[1][2] };
        }
        work += kg * 2.;
        tools -= kg * 0.1;
        fuel -= kg * 0.2;
    }
    for k in 0..3 {
        let kg = returns::limit(&w.botanicals, k)
            .min(tools / 0.1)
            .min(fuel / 0.2)
            .min((budget - work).max(0.) / 2.);
        plan.botanical_kg[k] = kg;
        work += 2. * kg;
        tools -= 0.1 * kg;
        fuel -= 0.2 * kg;
    }
    plan.receipt.requested = work.min(2.);
    plan.receipt.granted = plan.receipt.requested;
    plan
}
#[cfg(test)]
fn requested_work(h: &History, workshop: &Workshop, teachers: &[Workshop]) -> f32 {
    plan_work(h, workshop, teachers).receipt.requested as f32
}

impl History {
    #[cfg(test)]
    pub(crate) fn prepare_discoveries(&mut self) {
        self.open_participation();
        let requests = self.discovery_work_plans();
        self.reserve_discovery_plans(requests, &[]);
    }
    pub(crate) fn discovery_work_plans(&self) -> Vec<(u32, ResearchPlan)> {
        self.expeditions
            .as_ref()
            .and_then(|x| x.discoveries.as_ref())
            .map(|d| {
                d.workshops
                    .iter()
                    .map(|w| (w.site, plan_work(self, w, &d.workshops)))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub(crate) fn reserve_discovery_plans(
        &mut self,
        requests: Vec<(u32, ResearchPlan)>,
        caps: &[f32],
    ) {
        let mut remaining = caps.to_vec();
        for (site, mut plan) in requests {
            let mut grant = crate::labor::available(
                &self.sites[site as usize],
                self.society.is_some(),
                self.living.is_some(),
            )
            .min(plan.receipt.requested as f32)
            .min(remaining.get(site as usize).copied().unwrap_or(f32::MAX));
            if let Some(state) = &mut self.participation {
                let mut candidates: Vec<_> = state
                    .residents
                    .values()
                    .filter(|p| {
                        p.presence == crate::participation::Presence::Resident(site)
                            && state.available(p.person) > 0.
                    })
                    .map(|p| {
                        let interest = self
                            .culture
                            .as_ref()
                            .and_then(|c| c.agents.get(p.person as usize))
                            .map_or(0.5, |a| a.traits[3]);
                        (p.person, interest as f64 + (p.completed[1] / 12.).min(1.))
                    })
                    .collect();
                candidates.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                let ids: Vec<_> = candidates.into_iter().take(4).map(|p| p.0).collect();
                plan.commitment = state.reserve(
                    self.month,
                    site,
                    crate::participation::Activity::Research,
                    &ids,
                    grant,
                );
                grant = plan
                    .commitment
                    .map_or(0., |id| state.commitments[id as usize].granted);
            }
            self.sites[site as usize].economy.external[3] += grant;
            plan.receipt.granted = grant as f64;
            if let Some(left) = remaining.get_mut(site as usize) {
                *left = (*left - grant).max(0.);
            }
            if let Some(w) = self
                .expeditions
                .as_mut()
                .and_then(|x| x.discoveries.as_mut())
                .and_then(|d| d.workshops.iter_mut().find(|w| w.site == site))
            {
                w.work_plan = Some(plan);
            }
        }
    }
}
impl Generator {
    pub fn set_specimen_workshop_open(&mut self, site: u32, open: bool) -> Result<()> {
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let d = h
            .expeditions
            .as_mut()
            .and_then(|x| x.discoveries.as_mut())
            .ok_or_else(|| anyhow::anyhow!("enable discoveries first"))?;
        let w = d
            .workshops
            .iter_mut()
            .find(|w| w.site == site)
            .ok_or_else(|| anyhow::anyhow!("settlement has no specimen workshop"))?;
        w.enabled = open;
        h.event(
            "specimen_workshop_policy",
            Some(site),
            None,
            format!(
                "Research and processing {}",
                if open {
                    "opened"
                } else {
                    "paused; existing remedies remain available for care"
                }
            ),
        );
        Ok(())
    }
    pub fn enable_discoveries(&mut self) -> Result<()> {
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let x = h
            .expeditions
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable expeditions first"))?;
        ensure!(x.discoveries.is_none(), "specimen baseline already enabled");
        x.discoveries = Some(Discoveries::new(h.month));
        h.event("specimen_baseline",None,None,"Finite frontier specimens and research workshops enabled; earlier voyages receive no retroactive cargo".into());
        h.validate(&self.snapshot()?)?;
        self.civilizations = Some(h);
        Ok(())
    }
}

#[cfg(test)]
mod exchange_tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn methods_need_contact_and_supplies_without_creating_specimens() {
        use crate::{catalog::Catalog, config::Config, gpu::ContextGpu, society::Route};
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
                ecology_resolution: 64,
                seed: 7,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 12;
        h.society.as_mut().unwrap().routes = vec![Route {
            id: 0,
            from: 0,
            to: 1,
            cells: vec![],
            cost_km: 10.,
            open: false,
            flood_months: 0,
            road_bricks: 0.,
            upkeep: None,
        }];
        let workshop = |site| Workshop {
            botanicals: Default::default(),
            work_plan: None,
            site,
            enabled: true,
            processed: [0.; 2],
            curated: [0.; 2],
            samples: [0., 4.],
            studied: [0.; 2],
            learned: [None; 2],
            remedy: 0.,
            delivered: [0., 4.],
            causes: [None; 2],
            batches: [0; 2],
        };
        let mut d = Discoveries::new(h.month);
        d.workshops = vec![workshop(0), workshop(1)];
        // A teacher with f32 labor rounding residue still knows the method.
        d.workshops[0].studied[1] = 1.4999999552965164;
        d.workshops[0].samples[1] = 2.5;
        let good = h
            .economy_catalog
            .as_ref()
            .unwrap()
            .goods
            .iter()
            .position(|g| g.id == "writing_material")
            .unwrap();
        for s in &mut h.sites {
            s.economy.external[3] = 0.;
        }
        let fund = |h: &mut History| {
            let e = &mut h.sites[1].economy;
            e.external[3] = 0.5;
            e.labor[3] = 1.;
            e.goods[3] = 1.;
            e.goods[6] = 1.;
        };
        fund(h);
        h.sites[1].economy.goods[good] = 1.;
        d.month(h);
        assert!(d.workshops[1].learned[1].is_none());
        h.society.as_mut().unwrap().routes[0].open = true;
        fund(h);
        h.sites[1].economy.goods[good] = 0.;
        d.month(h);
        assert!(d.workshops[1].learned[1].is_none());
        fund(h);
        h.sites[1].economy.goods[good] = 1.;
        let studied = d.workshops[1].studied[1];
        let samples = d.workshops[1].samples[1];
        d.month(h);
        assert!(d.workshops[1].learned[1].is_some());
        assert_eq!(d.workshops[1].studied[1], studied);
        assert!((samples - d.workshops[1].samples[1] - d.workshops[1].processed[1]).abs() < 1e-8);
        assert!((h.sites[1].economy.goods[good] - 0.95).abs() < 1e-6);
        assert!(d.worker_months <= d.worker_months_reserved + 1e-8);
        let mut learner = d.workshops[1].clone();
        learner.samples = [0.; 2];
        learner.studied = [0.; 2];
        learner.learned = [None; 2];
        h.society.as_mut().unwrap().routes[0].open = false;
        assert_eq!(requested_work(h, &learner, &d.workshops), 0.);
        h.society.as_mut().unwrap().routes[0].open = true;
        h.sites[1].economy.goods[good] = 0.;
        assert_eq!(requested_work(h, &learner, &d.workshops), 0.);
        h.sites[1].economy.goods[good] = 1.;
        assert_eq!(requested_work(h, &learner, &d.workshops), 0.25);
        // Learning alone no longer requires local samples, tools or fuel.
        h.sites[1].economy.goods[3] = 0.;
        h.sites[1].economy.goods[6] = 0.;
        assert_eq!(requested_work(h, &learner, &d.workshops), 0.25);
        d.workshops[1] = learner.clone();
        h.sites[1].economy.external[3] = 0.25;
        let before = d.worker_months;
        d.month(h);
        assert!(d.workshops[1].learned[1].is_some());
        assert!((d.worker_months - before - 0.25).abs() < 1e-8);
        learner.samples = [0., 4.];
        assert_eq!(requested_work(h, &learner, &[]), 0.);
        h.sites[1].economy.goods[3] = 0.001;
        h.sites[1].economy.goods[6] = 1.;
        assert!((requested_work(h, &learner, &[]) - 0.02).abs() < 1e-6);
        // Both specimen kinds compete for one tool stock, not two independent claims.
        learner.samples = [4.; 2];
        let plan = plan_work(h, &learner, &[]);
        assert!((plan.processing_kg.iter().sum::<f64>() - 0.01).abs() < 1e-8);
        assert_eq!(plan.processing_kg[1], 0.);
        assert!((plan.outcomes.as_ref().unwrap().expected[0] - 0.01).abs() < 1e-8);
        assert_eq!(plan.outcomes.as_ref().unwrap().actual, [0.; 4]);
        learner.work_plan = Some(plan);
        d.workshops[1] = learner.clone();
        h.sites[1].economy.goods[3] = 0.; // sold after Reserve
        h.sites[1].economy.external[3] = 0.02;
        let before = d.worker_months;
        d.month(h);
        assert_eq!(d.worker_months, before);
        let receipt = &d.workshops[1].work_plan.as_ref().unwrap().receipt;
        receipt.validate().unwrap();
        assert_eq!(receipt.used, 0.);
        assert_eq!(receipt.released, receipt.granted);
        assert_eq!(
            d.workshops[1]
                .work_plan
                .as_ref()
                .unwrap()
                .outcomes
                .as_ref()
                .unwrap()
                .actual,
            [0.; 4]
        );
        // New fuel/tools cannot revive an already settled grant in the same month.
        h.sites[1].economy.goods[3] = 1.;
        d.month(h);
        assert_eq!(d.worker_months, before);
        // A teacher disappearing does not silently substitute another source.
        learner.samples = [0.; 2];
        learner.work_plan = None;
        h.sites[1].economy.goods[good] = 1.;
        let plan = plan_work(h, &learner, &d.workshops);
        assert_eq!(plan.teachers[1], Some(0));
        learner.work_plan = Some(plan);
        d.workshops[1] = learner;
        h.sites[1].economy.external[3] = 0.25;
        h.society.as_mut().unwrap().routes[0].open = false;
        d.month(h);
        assert!(d.workshops[1].learned[1].is_none());
        assert_eq!(d.workshops[1].work_plan.as_ref().unwrap().receipt.used, 0.);
        let outcomes = d.workshops[1]
            .work_plan
            .as_ref()
            .unwrap()
            .outcomes
            .as_ref()
            .unwrap();
        assert!(outcomes.methods_expected[1]);
        assert!(!outcomes.methods_actual[1]);

        // Known methods produce physical outputs, not another destructive study.
        let mut producing = d.clone();
        let mut world = h.clone();
        let w = &mut producing.workshops[1];
        w.studied = [1.5; 2];
        w.samples = [1.; 2];
        w.remedy = 0.;
        w.work_plan = None;
        world.sites[1].economy.goods[3] = 10.;
        world.sites[1].economy.goods[6] = 10.;
        world.sites[1].economy.labor[3] = 2.;
        let plan = plan_work(&world, w, &[]);
        let predicted = plan.outcomes.as_ref().unwrap().clone();
        assert!(predicted.expected[2] > 0. && predicted.expected[3] > 0.);
        assert_eq!(predicted.expected[..2], [0.; 2]);
        assert!((predicted.expected[2] - plan.processing_kg[0] * 0.5).abs() < 1e-9);
        assert!((predicted.expected[3] - plan.processing_kg[1] * 0.08).abs() < 1e-9);
        world.sites[1].economy.external[3] = plan.receipt.granted as f32;
        w.work_plan = Some(plan);
        producing.month(&mut world);
        let actual = producing.workshops[1]
            .work_plan
            .as_ref()
            .unwrap()
            .outcomes
            .as_ref()
            .unwrap();
        for (a, b) in actual.actual.iter().zip(predicted.expected) {
            assert!((a - b).abs() < 1e-7);
        }
        let restored: ResearchPlan = serde_json::from_value(
            serde_json::to_value(producing.workshops[1].work_plan.as_ref().unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(restored.outcomes.unwrap().actual, actual.actual);
    }
}

#[cfg(test)]
mod precision_tests {
    #[test]
    fn study_completion_tolerates_labor_rounding() {
        assert!(super::study_complete(1.4999999552965164));
        assert!(super::study_complete(1.5));
        assert!(!super::study_complete(1.5 - 2e-6));
        assert!(!super::study_complete(1.49));
    }
}
