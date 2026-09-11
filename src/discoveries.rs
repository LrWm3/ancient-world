//! Sparse, finite specimen collection and settlement research workshops.
//! The source inventory is a declared accessible frontier baseline, outside managed town plots.
use crate::{
    civilization::History,
    expeditions::{Expedition, Expeditions, Objective},
    gpu::{Cell, Generator},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
pub const CNP: [[f64; 3]; 2] = [[0.45, 0.02, 0.003], [0., 0., 0.08]];
pub const NAMES: [&str; 2] = ["faultroot resin", "phosphatic crust"];
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub cell: u32,
    pub initial: [f64; 2],
    pub remaining: [f64; 2],
    pub collected: [f64; 2],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workshop {
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
fn processing_limit(w: &Workshop, kind: usize, site: &crate::civilization::Site) -> f64 {
    if w.studied[kind] < 1.5 - 1e-8 && w.learned[kind].is_none() {
        return (1.5 - w.studied[kind]).min(0.25);
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
                - self.curated[k])
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
                    (w.delivered[k] - w.samples[k] - w.studied[k] - w.processed[k] - w.curated[k])
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
            event(h,e.origin,Some(e.cause),"specimen_source",format!("Accessible coastal baseline at cell {cell_id}: {:.1} kg resin, {:.1} kg phosphatic crust; no replenishment during frozen planetary history",initial[0],initial[1]));
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
        event(h,e.origin,Some(e.cause),"specimens_delivered",format!("Expedition {} delivered {:.2} kg resin and {:.2} kg phosphatic crust to the research workshop",e.id,e.samples[0],e.samples[1]));
        let cause = h.events.last().unwrap().id;
        for k in 0..2 {
            if e.samples[k] > 0. {
                self.workshops[i].causes[k] = Some(cause);
                self.workshops[i].samples[k] += e.samples[k];
                self.workshops[i].delivered[k] += e.samples[k];
                e.samples[k] = 0.;
            }
        }
    }
    pub(crate) fn month(&mut self, h: &mut History) {
        // Snapshot: transmitted methods cannot cross multiple contacts in one update.
        let teachers = self.workshops.clone();
        for w in &mut self.workshops {
            let site = w.site as usize;
            let s = &mut h.sites[site];
            let expired = w.remedy * 0.01;
            w.remedy -= expired;
            self.remedy_expired += expired;
            for k in 0..3 {
                s.economy.external[k] -= (expired * CNP[0][k]) as f32;
            }
            if s.abandoned || !w.enabled {
                continue;
            }
            // The GPU has reserved this labor from its normal craft budget for this month.
            let mut labor = (s.economy.external[3].min(s.economy.labor[3]).max(0.)) as f64;
            self.worker_months_reserved += labor;
            for (k, name) in NAMES.iter().enumerate() {
                if h.month.is_multiple_of(12)
                    && w.studied[k] < 1.5 - 1e-8
                    && w.learned[k].is_none()
                    && labor >= 0.25
                {
                    if let Some(teacher) = teachers
                        .iter()
                        .filter(|t| {
                            t.site != w.site
                                && !h.sites[t.site as usize].abandoned
                                && (t.studied[k] >= 1.5 - 1e-8 || t.learned[k].is_some())
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
                            }
                        }
                    }
                }
                let studying = w.studied[k] < 1.5 - 1e-8 && w.learned[k].is_none();
                let limit = processing_limit(w, k, &h.sites[site]);
                let e = &mut h.sites[site].economy;
                let kg = w.samples[k]
                    .min(limit)
                    .min(labor * 0.5)
                    .min(e.goods[3] as f64 / 0.1)
                    .min(e.goods[6] as f64 / 0.2);
                if kg <= 1e-8 {
                    continue;
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
                    if w.studied[k] >= 1.5 - 1e-8 {
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
impl History {
    pub(crate) fn prepare_discoveries(&mut self) {
        if let Some(d) = self
            .expeditions
            .as_ref()
            .and_then(|x| x.discoveries.as_ref())
        {
            for w in &d.workshops {
                let s = &mut self.sites[w.site as usize];
                if !s.abandoned
                    && w.enabled
                    && w.samples.iter().sum::<f64>() > 0.
                    && s.economy.goods[3] >= 0.025
                    && s.economy.goods[6] >= 0.05
                {
                    let wanted = (0..2)
                        .map(|k| w.samples[k].min(processing_limit(w, k, s)))
                        .sum::<f64>();
                    let possible = wanted
                        .min(s.economy.goods[3] as f64 / 0.1)
                        .min(s.economy.goods[6] as f64 / 0.2);
                    s.economy.external[3] +=
                        crate::labor::available(s, self.society.is_some(), self.living.is_some())
                            .min(2.)
                            .min((possible * 2.) as f32);
                }
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
        d.workshops[0].studied[1] = 1.5;
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
    }
}
