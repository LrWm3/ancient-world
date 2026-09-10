//! Canonical accessible extraction sources. Sparse arbitration; mining remains on GPU.
use crate::{
    civilization::History,
    gpu::{Cell, Generator},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

fn legacy_ore() -> Option<u32> {
    Some(1)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub cell: u32,
    #[serde(default)]
    pub mineral: Option<String>,
    #[serde(default = "legacy_ore")]
    pub ore_good: Option<u32>,
    /// Terrain prospect metadata, not a claim that generic economic ore has this composition.
    pub prospect: [f32; 4],
    /// Ore and clay kg. Historical baselines preserve preexisting accessible reserves.
    pub initial: [f64; 2],
    pub remaining: [f64; 2],
    pub extracted: [f64; 2],
    pub legacy_baseline: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Resources {
    #[serde(default)]
    pub regional_mines: BTreeMap<u32, crate::regional_mining::RegionalMine>,
    #[serde(default)]
    pub retired_regional_mines: Vec<crate::regional_mining::RegionalMine>,
    #[serde(default)]
    pub alloy_processing: bool,
    #[serde(default)]
    pub residue_events: BTreeMap<u32, u64>,
    pub started: u32,
    /// Archived terrain mineral identities; empty keeps generic legacy extraction.
    #[serde(default)]
    pub mineral_catalog: Vec<String>,
    pub sources: BTreeMap<u32, Source>,
    pub registered_sites: BTreeSet<u32>,
    pub closed_sites: BTreeSet<u32>,
}
impl Resources {
    pub fn residual(&self) -> f64 {
        self.sources
            .values()
            .flat_map(|s| {
                (0..2).map(move |k| {
                    (s.initial[k] - s.remaining[k] - s.extracted[k]).abs() / s.initial[k].max(1.)
                })
            })
            .fold(0., f64::max)
    }
    pub fn validate(&self, h: &History, cells: &[Cell]) -> Result<()> {
        ensure!(self.started <= h.month, "invalid resource clock");
        ensure!(
            self.residue_events.iter().all(|(site, id)| h
                .events
                .get(*id as usize)
                .is_some_and(|e| e.kind == "processing_residue" && e.site == Some(*site))
                && h.sites
                    .get(*site as usize)
                    .is_some_and(|s| s.economy.residue[1] > 0.)),
            "invalid residue provenance"
        );
        ensure!(
            h.sites
                .iter()
                .all(|s| s.economy.extraction[1] == f32::from(self.alloy_processing)),
            "alloy state disagrees with source registry"
        );
        for (&id, s) in &self.sources {
            ensure!(
                id == s.cell && (id as usize) < cells.len(),
                "invalid source identity"
            );
            ensure!(
                s.ore_good.is_none_or(|k| matches!(k, 1 | 32..=37)),
                "invalid source good"
            );
            if let Some(id) = s.mineral.as_deref() {
                let expected = crate::metallurgy::ore_slot(id, self.alloy_processing);
                ensure!(
                    s.ore_good == expected,
                    "mineral processing identity mismatch"
                );
                if let Some(k) = expected {
                    ensure!(
                        h.economy_catalog
                            .as_ref()
                            .and_then(|c| c.goods.get(k as usize))
                            .is_some_and(|g| g.id == format!("{id}_ore")),
                        "missing mineral good"
                    );
                }
            }
            ensure!(
                s.initial
                    .iter()
                    .chain(&s.remaining)
                    .chain(&s.extracted)
                    .all(|x| x.is_finite() && *x >= 0.)
                    && s.prospect.iter().all(|x| x.is_finite()),
                "invalid source inventory"
            );
        }
        ensure!(self.residual() < 1e-6, "resource conservation failure");
        ensure!(
            self.regional_mines
                .values()
                .all(|m| m.retired_month.is_none())
                && self.retired_regional_mines.iter().all(|m| m
                    .retired_month
                    .is_some_and(|t| t >= m.opened_month && t <= h.month)),
            "invalid regional retirement clock"
        );
        let mut regional_extracted = BTreeMap::<u32, [f64; 2]>::new();
        let mut regional_ids = BTreeSet::new();
        for (cell, m) in self
            .regional_mines
            .iter()
            .map(|(&cell, m)| (cell, m))
            .chain(self.retired_regional_mines.iter().map(|m| (m.cell, m)))
        {
            let total = regional_extracted.entry(cell).or_default();
            for (k, value) in total.iter_mut().enumerate() {
                *value += m.extracted[k];
            }
            ensure!(
                cell == m.cell
                    && regional_ids.insert(m.id)
                    && h.events
                        .get(m.id as usize)
                        .is_some_and(|e| e.kind == "regional_mine_activated"
                            && e.site == Some(m.site)
                            && e.month == m.opened_month)
                    && m.opened_month <= h.month
                    && h.sites.get(m.site as usize).is_some_and(|s| s.cell == cell)
                    && m.monthly_limit.iter().all(|x| x.is_finite() && *x >= 0.)
                    && self
                        .sources
                        .get(&cell)
                        .is_some_and(|s| (0..2).all(|k| m.extracted[k].is_finite()
                            && m.extracted[k] >= 0.
                            && m.extracted[k] <= s.extracted[k] + 1e-6)),
                "invalid regional source control"
            );
        }
        ensure!(
            regional_extracted.iter().all(|(cell, sum)| self
                .sources
                .get(cell)
                .is_some_and(|s| (0..2).all(|k| sum[k] <= s.extracted[k] + 1e-6))),
            "duplicated regional extraction history"
        );
        ensure!(
            h.sites
                .iter()
                .all(|s| self.registered_sites.contains(&s.id)),
            "unregistered resource claimant"
        );
        ensure!(
            self.closed_sites
                .iter()
                .all(|id| (*id as usize) < h.sites.len()),
            "invalid mine closure"
        );
        ensure!(
            self.registered_sites
                .iter()
                .all(|id| h.sites.get(*id as usize).is_some_and(|s| self
                    .sources
                    .get(&s.cell)
                    .is_some_and(|source| s.economy.extraction[0].max(1.)
                        == source.ore_good.unwrap_or(1) as f32)
                    && s.economy.reserves[1..3] == [0., 0.])),
            "uncommitted extraction allowances"
        );
        Ok(())
    }
}
impl History {
    /// Read-only accessible supply; it is not another spendable inventory.
    pub fn accessible_resources(&self, site: usize) -> [f32; 2] {
        let Some(s) = self.sites.get(site) else {
            return [0.; 2];
        };
        self.resources
            .as_ref()
            .map_or([s.economy.reserves[1], s.economy.reserves[2]], |r| {
                if r.closed_sites.contains(&s.id) {
                    return [0.; 2];
                }
                r.sources.get(&s.cell).map_or([0.; 2], |d| {
                    [
                        if d.ore_good.is_some() {
                            (d.remaining[0] as f32).min(r.regional_limit(s.cell, s.id)[0])
                        } else {
                            0.
                        },
                        (d.remaining[1] as f32).min(r.regional_limit(s.cell, s.id)[1]),
                    ]
                })
            })
    }

    /// Register new sites without allowing later claims to replenish an existing source.
    pub(crate) fn register_resources(&mut self, terrain: &[Cell], radius_km: f32) {
        let Some(r) = &mut self.resources else { return };
        for s in &mut self.sites {
            if s.economy.claim[3] < 0.5 || !r.registered_sites.insert(s.id) {
                continue;
            }
            r.sources.entry(s.cell).or_insert_with(|| {
                let t = &terrain[s.cell as usize];
                // A fixed 1 km² accessible prospect, bounded by the parent cell area.
                // It is a declared accessible baseline, not all mineral mass in the crust.
                let area = (crate::grid::solid_angle(s.cell, self.terrain_resolution)
                    * (radius_km as f64 * 1000.).powi(2))
                .clamp(0., 1_000_000.);
                let initial = [
                    area * 0.05 * t.geology[2].clamp(0., 1.) as f64,
                    area * 0.2 * t.terrain[2].clamp(0.01, 1.) as f64,
                ];
                Source {
                    cell: s.cell,
                    mineral: None,
                    ore_good: Some(1),
                    prospect: t.geology,
                    initial,
                    remaining: initial,
                    extracted: [0.; 2],
                    legacy_baseline: false,
                }
            });
            if !r.mineral_catalog.is_empty() && r.sources[&s.cell].ore_good == Some(1) {
                let source = r.sources.get_mut(&s.cell).unwrap();
                identify_source(
                    source,
                    &r.mineral_catalog,
                    &terrain[s.cell as usize],
                    r.alloy_processing,
                );
            }
            s.economy.extraction[0] = r.sources[&s.cell].ore_good.unwrap_or(1) as f32;
            if r.alloy_processing {
                s.economy.extraction[1] = 1.;
                s.economy.residue[2] = s.economy.claim[1] * 0.02;
            }
            s.economy.reserves[1..3].fill(0.);
        }
    }
    /// Reserve a fair share of the same source for each active claimant. No inventory copies survive the month.
    pub(crate) fn allocate_resources(&mut self) -> Vec<[f32; 2]> {
        let Some(r) = &self.resources else {
            return vec![];
        };
        let mut count = BTreeMap::<u32, usize>::new();
        for s in &self.sites {
            if !s.abandoned
                && s.stocks.stock[0] > 0.
                && !r.closed_sites.contains(&s.id)
                && r.regional_limit(s.cell, s.id).iter().any(|v| *v > 0.)
            {
                *count.entry(s.cell).or_default() += 1;
            }
        }
        let mut allowances = vec![[0.; 2]; self.sites.len()];
        for s in &mut self.sites {
            if s.abandoned
                || s.stocks.stock[0] <= 0.
                || r.closed_sites.contains(&s.id)
                || r.regional_limit(s.cell, s.id).iter().all(|v| *v <= 0.)
            {
                continue;
            }
            let source = &r.sources[&s.cell];
            s.economy.extraction[0] = source.ore_good.unwrap_or(1) as f32;
            for k in 0..2 {
                if k == 0 && source.ore_good.is_none() {
                    continue;
                }
                // Never lend centuries of reserves to a float32 monthly kernel.
                // Five kg per resident exceeds this month's possible extraction labor.
                let share = (source.remaining[k] / count[&s.cell] as f64)
                    .min(s.stocks.stock[0] as f64 * 5.)
                    .min(r.regional_limit(s.cell, s.id)[k] as f64);
                // Round downward so all parallel claims fit inside the source.
                let mut grant = share as f32;
                if grant as f64 > share {
                    grant = f32::from_bits(grant.to_bits() - 1);
                }
                allowances[s.id as usize][k] = grant;
                s.economy.reserves[k + 1] = grant;
            }
        }
        allowances
    }
    pub(crate) fn settle_resources(&mut self, allowances: &[[f32; 2]]) -> Result<()> {
        let Some(r) = &mut self.resources else {
            return Ok(());
        };
        for s in &mut self.sites {
            let source = r.sources.get_mut(&s.cell).expect("registered source");
            for k in 0..2 {
                let before = allowances[s.id as usize][k] as f64;
                let after = s.economy.reserves[k + 1] as f64;
                ensure!(
                    after.is_finite() && after >= 0. && after <= before,
                    "invalid GPU extraction withdrawal"
                );
                let used = before - after;
                source.remaining[k] -= used;
                source.extracted[k] += used;
                if let Some(m) = r.regional_mines.get_mut(&s.cell) {
                    ensure!(m.site == s.id || used == 0., "regional ownership violated");
                    m.extracted[k] += used;
                }
                s.economy.reserves[k + 1] = 0.;
            }
        }
        let newly_deposited: Vec<_> = self
            .sites
            .iter()
            .filter(|s| s.economy.residue[1] > 0. && !r.residue_events.contains_key(&s.id))
            .map(|s| (s.id, s.economy.residue[0]))
            .collect();
        for (site, mass) in newly_deposited {
            let id = self.events.len() as u64;
            self.event("processing_residue",Some(site),None,format!("Mineral processing established a persistent on-site residue deposit ({mass:.2} kg); finite disposal space limits further work"));
            self.resources
                .as_mut()
                .unwrap()
                .residue_events
                .insert(site, id);
        }
        Ok(())
    }
}
impl Generator {
    /// Explicit migration: move existing unmined town stocks into shared source custody.
    pub fn enable_shared_resources(&mut self) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "resources require a completed boundary"
        );
        let terrain = self.snapshot()?;
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(
            h.version == 2 && h.sites.iter().all(|s| s.economy.claim[3] > 0.5),
            "initialize economy before resource migration"
        );
        if h.resources.is_some() {
            return Ok(());
        }
        let mut r = Resources {
            started: h.month,
            ..Default::default()
        };
        for s in &mut h.sites {
            let source = r.sources.entry(s.cell).or_insert(Source {
                cell: s.cell,
                mineral: None,
                ore_good: Some(1),
                prospect: terrain[s.cell as usize].geology,
                initial: [0.; 2],
                remaining: [0.; 2],
                extracted: [0.; 2],
                legacy_baseline: true,
            });
            for k in 0..2 {
                let mass = s.economy.reserves[k + 1] as f64;
                source.initial[k] += mass;
                source.remaining[k] += mass;
                s.economy.reserves[k + 1] = 0.;
            }
            r.registered_sites.insert(s.id);
        }
        h.resources = Some(r);
        h.event("resource_baseline",None,None,"Existing accessible ore and clay transferred into shared planetary-cell sources; prior extraction is not reconstructed".into());
        Ok(())
    }
    /// A boundary intervention changes access, never the underlying inventory.
    pub fn set_mine_closed(&mut self, site: u32, closed: bool) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "mine policy requires a completed boundary"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        ensure!((site as usize) < h.sites.len(), "unknown mining site");
        let r = h
            .resources
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable shared resources first"))?;
        let changed = if closed {
            r.closed_sites.insert(site)
        } else {
            r.closed_sites.remove(&site)
        };
        if changed {
            h.event(
                "mine_access",
                Some(site),
                None,
                if closed {
                    "Extraction closed; source inventory retained"
                } else {
                    "Extraction reopened; finite remaining source available"
                }
                .into(),
            );
        }
        Ok(())
    }
}

fn identify_source(s: &mut Source, minerals: &[String], terrain: &Cell, alloys: bool) {
    s.mineral = minerals.get(terrain.meta[1] as usize).cloned();
    s.ore_good = s
        .mineral
        .as_deref()
        .and_then(|m| crate::metallurgy::ore_slot(m, alloys));
}

impl Generator {
    /// Explicitly identify remaining source material. Previously mined generic goods retain identity.
    pub fn enable_mineral_processing(&mut self) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "processing requires a completed boundary"
        );
        let terrain = self.snapshot()?;
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let r = h
            .resources
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("enable shared resources first"))?;
        if !r.mineral_catalog.is_empty() {
            return Ok(());
        }
        let mut catalog = h.economy_catalog.clone().unwrap();
        ensure!(
            catalog.recipes.len() + 3 <= 64 && catalog.goods.len() > 34,
            "processing requires three recipe slots and reserved goods 32–34"
        );
        for (slot, id, work) in [
            (32, "hematite", 0.12),
            (33, "magnetite", 0.14),
            (34, "limonite", 0.18),
        ] {
            ensure!(
                catalog.goods[slot].id == format!("reserved_{slot}")
                    && h.sites.iter().all(|s| s.economy.goods[slot] == 0.
                        && s.economy.initial[slot] == 0.
                        && s.economy.made[slot] == 0.
                        && s.economy.used[slot] == 0.)
                    && h.cargo.iter().all(|c| c.good as usize != slot)
                    && catalog
                        .recipes
                        .iter()
                        .all(|r| r.input[slot] == 0. && r.output[slot] == 0.),
                "processing slot {slot} is already in use"
            );
            let mineral = self
                .catalog
                .minerals
                .iter()
                .find(|m| m.id == id)
                .ok_or_else(|| anyhow::anyhow!("missing {id} definition"))?;
            catalog.goods[slot] = crate::economy::Good {
                id: format!("{id}_ore"),
                name: format!("{} ore", mineral.name),
                base_price: 4.,
                cnp: [0.; 3],
                food_energy: 0.,
                delay_spoilage: None,
            };
            let mut recipe = crate::economy::Recipe {
                input: [0.; 64],
                output: [0.; 64],
                work: [work, 0., 1., 0.],
            };
            recipe.input[slot] = 1.;
            recipe.input[6] = 0.5;
            recipe.output[2] = mineral.yield_fraction * 0.8;
            catalog.recipes.push(recipe);
        }
        catalog.validate()?;
        h.economy_catalog = Some(catalog);
        let r = h.resources.as_mut().unwrap();
        r.mineral_catalog = self.catalog.minerals.iter().map(|m| m.id.clone()).collect();
        for source in r.sources.values_mut() {
            identify_source(
                source,
                &r.mineral_catalog,
                &terrain[source.cell as usize],
                r.alloy_processing,
            );
        }
        for s in &mut h.sites {
            s.economy.extraction[0] = r.sources[&s.cell].ore_good.unwrap_or(1) as f32;
        }
        h.event("mineral_processing",None,None,"Remaining sources identified from terrain. Hematite, magnetite and limonite now retain separate ore inventories and processing yields; unsupported minerals cannot supply generic metal. Existing manufactured goods retain their identity".into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilization::{Site, Stocks};
    fn fixture() -> History {
        let mut h: History = serde_json::from_value(serde_json::json!({
            "version":2,"seed":17,"terrain_resolution":16,"source_epoch":0,"month":0,
            "civilizations":[],"sites":[],"people":[],"events":[],"shipments":[],
            "candidates":[],"initial_food":0.,"initial_population":0.
        }))
        .unwrap();
        for id in 0..2 {
            let mut stocks = Stocks {
                stock: [0.; 4],
                habitat: [0.; 4],
                people: [0.; 4],
                ledger: [0.; 4],
            };
            stocks.stock[0] = 10.;
            h.sites.push(Site {
                id,
                civilization: 0,
                island: 0,
                cell: 0,
                name: format!("Town {id}"),
                founded: 0,
                abandoned: false,
                lifecycle: Default::default(),
                stocks,
                economy: Default::default(),
                demography: Default::default(),
            });
        }
        let source = Source {
            cell: 0,
            mineral: None,
            ore_good: Some(1),
            prospect: [0.; 4],
            initial: [9., 9.],
            remaining: [9., 9.],
            extracted: [0.; 2],
            legacy_baseline: true,
        };
        h.resources = Some(Resources {
            sources: BTreeMap::from([(0, source)]),
            registered_sites: BTreeSet::from([0, 1]),
            ..Default::default()
        });
        h
    }
    #[test]
    fn competing_claims_return_unused_allowances_and_do_not_replenish() {
        let mut h = fixture();
        let quotas = h.allocate_resources();
        assert_eq!(quotas, vec![[4.5; 2]; 2]);
        h.sites[0].economy.reserves[1] = 0.; // First town spends its ore allowance.
        h.settle_resources(&quotas).unwrap();
        assert_eq!(
            h.resources.as_ref().unwrap().sources[&0].remaining,
            [4.5, 9.]
        );
        h.resources.as_mut().unwrap().closed_sites.insert(0);
        let quotas = h.allocate_resources();
        assert_eq!(quotas[0], [0.; 2]);
        assert_eq!(quotas[1], [4.5, 9.]);
        h.sites[1].economy.reserves[1] = 0.;
        h.settle_resources(&quotas).unwrap();
        assert_eq!(h.resources.as_ref().unwrap().sources[&0].remaining[0], 0.);
        assert_eq!(h.resources.as_ref().unwrap().residual(), 0.);
        // A new claim in the same cell cannot reseed its exhausted ore.
        let mut third = h.sites[0].clone();
        third.id = 2;
        third.economy.claim[3] = 1.;
        third.economy.reserves[1] = 100.;
        h.sites.push(third);
        h.register_resources(&[Cell::default()], 6371.);
        assert_eq!(h.resources.as_ref().unwrap().sources[&0].remaining[0], 0.);
        assert_eq!(h.sites[2].economy.reserves[1], 0.);
    }
    #[test]
    fn regional_control_conserves_and_releases_shared_source() {
        let mut h = fixture();
        h.event(
            "regional_mine_activated",
            Some(0),
            None,
            "Controlled fixture".into(),
        );
        h.resources.as_mut().unwrap().regional_mines.insert(
            0,
            crate::regional_mining::RegionalMine {
                id: 0,
                cell: 0,
                site: 0,
                opened_month: 0,
                retired_month: None,
                monthly_limit: [1., 2.],
                extracted: [0.; 2],
            },
        );
        assert_eq!(h.accessible_resources(1), [0.; 2]);
        assert_eq!(h.accessible_resources(0), [1., 2.]);
        let quotas = h.allocate_resources();
        assert_eq!(quotas, vec![[1., 2.], [0.; 2]]);
        h.sites[0].economy.reserves[1] = 0.5;
        h.sites[0].economy.reserves[2] = 1.;
        h.settle_resources(&quotas).unwrap();
        let r = h.resources.as_ref().unwrap();
        assert_eq!(r.sources[&0].remaining, [8.5, 8.]);
        assert_eq!(r.regional_mines[&0].extracted, [0.5, 1.]);
        r.validate(&h, &[Cell::default()]).unwrap();
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
        for case in [&mut h, &mut resumed] {
            case.resources
                .as_mut()
                .unwrap()
                .regional_mines
                .get_mut(&0)
                .unwrap()
                .monthly_limit = [0.; 2];
            let q = case.allocate_resources();
            assert_eq!(q, vec![[0.; 2]; 2]);
            case.settle_resources(&q).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let r = h.resources.as_mut().unwrap();
        let mut retired = r.regional_mines.remove(&0).unwrap();
        retired.retired_month = Some(h.month);
        r.retired_regional_mines.push(retired);
        assert_eq!(r.sources[&0].remaining, [8.5, 8.]);
        assert_eq!(r.residual(), 0.);
        h.resources
            .as_ref()
            .unwrap()
            .validate(&h, &[Cell::default()])
            .unwrap();
        let mut corrupt = h.resources.clone().unwrap();
        corrupt
            .retired_regional_mines
            .push(corrupt.retired_regional_mines[0].clone());
        assert!(corrupt.validate(&h, &[Cell::default()]).is_err());
        assert_eq!(h.allocate_resources(), vec![[4.25, 4.]; 2]);
        let mut old = serde_json::to_value(&h.resources).unwrap();
        old.as_object_mut().unwrap().remove("regional_mines");
        let old: Resources = serde_json::from_value(old).unwrap();
        assert!(old.regional_mines.is_empty());
    }
}
