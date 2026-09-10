//! Catalogs and regional allocations; biomass production stays in the GPU kernels.
use crate::{
    civilization::History,
    economy::EconomyCatalog,
    gpu::{Cell, Generator},
    grid,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
/// Annual activity with a five-year adjustment time. Infrastructure and inventories
/// remain physical switching costs; this record prevents instant role relabeling.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RoleState {
    pub month: u32,
    pub cumulative: [f32; 9],
    pub scores: [f32; 9],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Crop {
    /// Optional monthly seasonal model; absent archives retain their original growth.
    #[serde(default)]
    pub season: Option<CropSeason>,
    pub good: String,
    pub temperature: [f32; 2],
    pub rainfall_mm: f32,
    pub yield_scale: f32,
    pub water_m3_kg: f32,
    pub land_share: f32,
    pub harvest_offset: u32,
}
/// Regional crop-process hypotheses, not measured cultivar physiology.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CropSeason {
    pub harvest_index: f32,
    pub reproductive_stress: f32,
    pub frost_loss: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Herd {
    pub id: String,
    pub product: String,
    pub feed: String,
    pub initial_kg: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FisherySettings {
    pub adaptive: bool,
    pub primitive_gear: bool,
    pub opportunity_cost: bool,
    pub max_worker_share: f32,
    pub kg_per_worker_month: f32,
    pub half_saturation_kg_c_m2: f32,
    pub reserve_months: f32,
}
impl Default for FisherySettings {
    fn default() -> Self {
        Self {
            adaptive: false,
            primitive_gear: false,
            opportunity_cost: false,
            max_worker_share: 0.15,
            kg_per_worker_month: 80.,
            half_saturation_kg_c_m2: 0.0001,
            reserve_months: 6.,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgricultureCatalog {
    /// Disable harvest only; aquatic populations continue evolving.
    #[serde(default = "fisheries_enabled")]
    pub fisheries_enabled: bool,
    #[serde(default)]
    pub fishery: FisherySettings,
    pub version: u32,
    /// Crop-potential kg charged per kg atmospheric N fixed; historical catalogs retain 80.
    #[serde(default = "legacy_fixation_cost")]
    pub fixation_cost_kg: f32,
    pub crops: Vec<Crop>,
    pub herds: Vec<Herd>,
}
fn fisheries_enabled() -> bool {
    true
}
fn legacy_fixation_cost() -> f32 {
    80.
}
impl AgricultureCatalog {
    pub fn bundled() -> Self {
        toml::from_str(include_str!("../assets/agriculture.toml"))
            .expect("bundled agriculture TOML")
    }
    /// Explicit research preset: current founding/food budgets are NOT calibrated
    /// for its shorter productive season. Keep out of default worlds.
    pub fn seasonal_experiment() -> Self {
        toml::from_str(include_str!("../assets/agriculture-seasonal.toml"))
            .expect("seasonal experiment TOML")
    }
    pub fn validate(&self, e: &EconomyCatalog) -> Result<()> {
        ensure!(
            self.version == 1
                && self.crops.len() == 6
                && self.herds.len() == 3
                && self.fixation_cost_kg.is_finite()
                && (1. ..=1000.).contains(&self.fixation_cost_kg),
            "invalid agricultural catalog dimensions"
        );
        let f = &self.fishery;
        ensure!(
            (0.001..=0.25).contains(&f.max_worker_share)
                && (1. ..=200.).contains(&f.kg_per_worker_month)
                && (1e-8..=1.).contains(&f.half_saturation_kg_c_m2)
                && (1. ..=12.).contains(&f.reserve_months),
            "invalid fishery settings"
        );
        ensure!(
            e.index("wood") == Some(0)
                && e.index("tools") == Some(3)
                && e.index("fiber") == Some(16),
            "fishery equipment requires stable material slots"
        );
        ensure!(
            !f.adaptive
                || (e.composition(28).iter().all(|v| *v > 0.) && e.goods[28].food_energy > 0.),
            "adaptive fish must contain C/N/P and food energy"
        );
        let mut crop_ids = std::collections::BTreeSet::new();
        for c in &self.crops {
            if let Some(s) = &c.season {
                ensure!(
                    s.harvest_index.is_finite()
                        && (0.05..=1.).contains(&s.harvest_index)
                        && s.reproductive_stress.is_finite()
                        && (0. ..=1.).contains(&s.reproductive_stress)
                        && s.frost_loss.is_finite()
                        && (0. ..=1.).contains(&s.frost_loss),
                    "invalid crop seasonal traits"
                );
            }
            ensure!(
                crop_ids.insert(&c.good)
                    && e.index(&c.good).is_some_and(|i| i != crate::economy::FOOD)
                    && c.temperature.iter().all(|v| v.is_finite())
                    && c.temperature[0] < c.temperature[1]
                    && [c.rainfall_mm, c.yield_scale, c.water_m3_kg, c.land_share]
                        .iter()
                        .all(|v| v.is_finite() && *v > 0.)
                    && c.harvest_offset < 12,
                "invalid crop requirements"
            );
        }
        ensure!(
            (self.crops.iter().map(|c| c.land_share).sum::<f32>() - 1.).abs() < 0.001,
            "crop allocations must sum to one"
        );
        // Slaughter is a finite split of body material; edited products cannot create elements.
        let meat = e
            .index("meat")
            .ok_or_else(|| anyhow::anyhow!("missing meat material"))?;
        let hides = e
            .index("hides")
            .ok_or_else(|| anyhow::anyhow!("missing hide material"))?;
        ensure!(
            meat == 24 && hides == 19 && e.index("fish") == Some(28),
            "managed animal goods must retain their stable archive slots"
        );
        for (k, body) in [0.25, 0.04, 0.003].into_iter().enumerate() {
            ensure!(
                0.6 * e.composition(meat)[k] + 0.1 * e.composition(hides)[k] <= body + 1e-7,
                "slaughter products exceed embodied animal nutrients"
            );
        }
        let mut herd_ids = std::collections::BTreeSet::new();
        for a in &self.herds {
            ensure!(
                herd_ids.insert(&a.id)
                    && !a.id.is_empty()
                    && e.index(&a.product).is_some()
                    && e.index(&a.feed).is_some()
                    && a.initial_kg.is_finite()
                    && a.initial_kg >= 0.,
                "invalid herd"
            );
        }
        Ok(())
    }
    pub fn gpu(&self, e: &EconomyCatalog) -> Vec<[f32; 4]> {
        let mut out = vec![];
        for c in &self.crops {
            out.push([
                e.index(&c.good).unwrap() as f32,
                c.temperature[0],
                c.temperature[1],
                c.rainfall_mm,
            ]);
            out.push([
                c.yield_scale,
                c.water_m3_kg,
                c.land_share,
                c.harvest_offset as f32,
            ]);
        }
        for a in &self.herds {
            out.push([
                e.index(&a.product).unwrap() as f32,
                e.index(&a.feed).unwrap() as f32,
                if e.agriculture.is_some() {
                    self.fixation_cost_kg
                } else {
                    80.
                },
                if e.index("scrap_metal") == Some(29) {
                    1.
                } else {
                    0.
                },
            ]);
        }
        for c in &self.crops {
            out.push(c.season.as_ref().map_or([0.; 4], |s| {
                [1., s.harvest_index, s.reproductive_stress, s.frost_loss]
            }));
        }
        out
    }
}
impl History {
    pub(crate) fn prepare_fisheries(&mut self, cells: &[Cell], eco_n: u32) {
        let n = self.terrain_resolution;
        let enabled = self
            .economy_catalog
            .as_ref()
            .and_then(|c| c.agriculture.as_ref())
            .is_none_or(|a| a.fisheries_enabled);
        let policy = self
            .economy_catalog
            .as_ref()
            .and_then(|c| c.agriculture.as_ref())
            .map(|a| a.fishery.clone())
            .unwrap_or_default();
        for s in &mut self.sites {
            s.economy.fishery[3] = f32::from(policy.adaptive);
            s.economy.fishery_choice[3] = f32::from(policy.opportunity_cost);
            s.economy.fishery_traps[3] = f32::from(policy.primitive_gear);
            s.economy.fishery_config = [
                policy.max_worker_share,
                policy.kg_per_worker_month,
                policy.half_saturation_kg_c_m2,
                policy.reserve_months,
            ];
            // Clear stale receiving-water access, including explicit harvest closures.
            s.economy.management[1] = 0.;
            s.economy.management[2] = 0.;
            if !enabled {
                continue;
            }
            if s.economy.management[0] < 0.5 {
                continue;
            }
            let near = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .into_iter()
                .map(|(x, y)| grid::neighbor(s.cell, n, x, y))
                .find(|&id| cells[id as usize].meta[0] == 1 && cells[id as usize].water[0] > 0.25);
            if let Some(id) = near {
                let j = id / (n * n) * eco_n * eco_n
                    + (id / n % n) / (n / eco_n) * eco_n
                    + (id % n) / (n / eco_n);
                s.economy.management[1] = (j + 1) as f32;
                s.economy.management[2] = (grid::solid_angle(j, eco_n)
                    / grid::solid_angle(s.economy.claim[0] as u32, eco_n))
                    as f32
                    * s.economy.claim[2];
            }
        }
    }
    pub(crate) fn role_activity(
        &self,
        site: u32,
        culture: Option<&crate::culture::Culture>,
    ) -> Vec<(&'static str, f32)> {
        let Some(s) = self.sites.get(site as usize) else {
            return vec![];
        };
        let e = &s.economy;
        let mut values = vec![
            ("agricultural", e.agriculture[0] / 2000.),
            ("pastoral", e.herds.iter().map(|a| a[3] / 30.).sum()),
            ("fishing", e.agriculture[2] / 240.),
            ("mining", e.made[1] / 60.),
            (
                "manufacturing",
                (e.made[2] + e.made[3] + e.made[5] + e.made[7]) / 120.,
            ),
            ("trading", e.finance[2] / 5000.),
        ];
        if let Some(c) = culture {
            values.push((
                "religious",
                c.institutions
                    .iter()
                    .filter(|n| {
                        n.site == site && n.kind == crate::culture::InstitutionKind::Religious
                    })
                    .map(|n| n.expenses as f32 / 10.)
                    .sum(),
            ));
            values.push((
                "scholarly",
                c.institutions
                    .iter()
                    .filter(|n| {
                        n.site == site && n.kind == crate::culture::InstitutionKind::Scholarly
                    })
                    .map(|n| n.knowledge.len() as f32)
                    .sum(),
            ));
        }
        values.push((
            "expedition service",
            self.expeditions.as_ref().map_or(0., |x| {
                x.voyages.iter().filter(|v| v.origin == site).count() as f32 * 4.
            }),
        ));
        values
    }
    pub fn settlement_roles(&self, site: u32) -> Vec<(&'static str, f32)> {
        let mut values = self.role_activity(site, self.culture.as_ref());
        if let Some(state) = self
            .culture
            .as_ref()
            .and_then(|c| c.roles.get(site as usize))
        {
            if state.month > 0 {
                for (i, value) in values.iter_mut().enumerate() {
                    value.1 = state.scores[i];
                }
            }
        }
        values.sort_by(|a, b| b.1.total_cmp(&a.1));
        values.truncate(2);
        values
    }
}
impl Generator {
    pub fn set_diversified_farming(&mut self, enabled: bool) -> Result<()> {
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        h.farming_mode = Some(enabled);
        for s in &mut h.sites {
            s.economy.management[0] = if enabled { 1. } else { 0. };
        }
        h.event(
            "farming_scenario",
            None,
            None,
            format!("Diversified farming {enabled}; existing inventories retained"),
        );
        Ok(())
    }
}
