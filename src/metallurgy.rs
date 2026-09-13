//! Opt-in copper/tin industry and persistent, bounded on-site mineral residue.
use crate::{
    economy::{EconomyCatalog, Good, Recipe, GOODS},
    gpu::{Generator, Stage},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const AXE_IRON_FRACTION: f32 = 0.6;
const PICK_IRON_FRACTION: f32 = 0.7;
pub(crate) const HEMATITE_IRON_FRACTION: f32 = 0.7;
pub(crate) const MAGNETITE_IRON_FRACTION: f32 = 0.72;
pub(crate) const LIMONITE_IRON_FRACTION: f32 = 0.5;
const CHALCOPYRITE_COPPER_FRACTION: f32 = 0.34;
const MALACHITE_COPPER_FRACTION: f32 = 0.57;
const CASSITERITE_TIN_FRACTION: f32 = 0.78;
const BRONZE_COPPER_FRACTION: f32 = 0.9;
const BRONZE_TIN_FRACTION: f32 = 0.1;
const ORE_BASE_PRICE: f32 = 4.;
const METAL_BASE_PRICE: f32 = 20.;
const SMELTING_RECOVERY_FRACTION: f32 = 0.8;
const SMELTING_FUEL_KG: f32 = 0.5;
const PROCESSING_FUEL_KG: f32 = 0.1;
const ALLOY_WORKER_MONTHS: f32 = 0.08;
const BRONZE_TOOL_WORKER_MONTHS: f32 = 0.1;
const COPPER_TOOL_WORKER_MONTHS: f32 = 0.08;
const REMELTING_WORKER_MONTHS: f32 = 0.08;
const REMELTING_RECOVERY_FRACTION: f32 = 0.9;
const REMELTING_RESIDUE_KG: f32 = 0.1;
const RESIDUE_CAPACITY_KG_PER_M2: f32 = 0.02;
const SMELTING_CHAINS: [(usize, &str, usize, f32); 3] = [
    (35, "chalcopyrite", 38, 0.18),
    (36, "malachite", 38, 0.12),
    (37, "cassiterite", 39, 0.16),
];
pub const ALLOY_GOODS: [(usize, &str); 10] = [
    (35, "chalcopyrite_ore"),
    (36, "malachite_ore"),
    (37, "cassiterite_ore"),
    (38, "copper"),
    (39, "tin"),
    (40, "bronze"),
    (41, "bronze_tools"),
    (42, "bronze_scrap"),
    (43, "copper_tools"),
    (44, "copper_scrap"),
];
/// Modeled metal mass fractions; general historical metal remains the legacy iron-compatible pool.
pub fn metals(id: &str) -> [f32; 3] {
    match id {
        "metal_vessel" => [1., 0., 0.],
        "metal_axe" => [AXE_IRON_FRACTION, 0., 0.],
        "metal_pick" => [PICK_IRON_FRACTION, 0., 0.],
        "hematite_ore" => [HEMATITE_IRON_FRACTION, 0., 0.],
        "magnetite_ore" => [MAGNETITE_IRON_FRACTION, 0., 0.],
        "limonite_ore" => [LIMONITE_IRON_FRACTION, 0., 0.],
        "chalcopyrite_ore" => [0., CHALCOPYRITE_COPPER_FRACTION, 0.],
        "malachite_ore" => [0., MALACHITE_COPPER_FRACTION, 0.],
        "cassiterite_ore" => [0., 0., CASSITERITE_TIN_FRACTION],
        "copper" | "copper_tools" | "copper_scrap" => [0., 1., 0.],
        "tin" => [0., 0., 1.],
        "bronze" | "bronze_tools" | "bronze_scrap" => {
            [0., BRONZE_COPPER_FRACTION, BRONZE_TIN_FRACTION]
        }
        _ => [0.; 3],
    }
}
pub fn ore_slot(id: &str, alloys: bool) -> Option<u32> {
    match id {
        "hematite" => Some(32),
        "magnetite" => Some(33),
        "limonite" => Some(34),
        "chalcopyrite" if alloys => Some(35),
        "malachite" if alloys => Some(36),
        "cassiterite" if alloys => Some(37),
        _ => None,
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResidueDeposit {
    pub site: u32,
    pub cell: u32,
    pub kg: f32,
    pub produced_kg: f32,
    pub capacity_kg: f32,
    pub abandoned: bool,
    pub founding_event: Option<u64>,
}
impl crate::civilization::History {
    pub fn processing_deposits(&self) -> Vec<ResidueDeposit> {
        self.sites
            .iter()
            .filter(|s| s.economy.residue[1] > 0.)
            .map(|s| ResidueDeposit {
                site: s.id,
                cell: s.cell,
                kg: s.economy.residue[0],
                produced_kg: s.economy.residue[1],
                capacity_kg: s.economy.residue[2],
                abandoned: s.abandoned,
                founding_event: self
                    .resources
                    .as_ref()
                    .and_then(|r| r.residue_events.get(&s.id).copied()),
            })
            .collect()
    }
}
fn recipe(inputs: &[(usize, f32)], outputs: &[(usize, f32)], labor: f32, residue: f32) -> Recipe {
    let mut r = Recipe {
        input: [0.; GOODS],
        output: [0.; GOODS],
        work: [labor, 0., 1., residue],
    };
    for &(k, v) in inputs {
        r.input[k] = v;
    }
    for &(k, v) in outputs {
        r.output[k] = v;
    }
    r
}
impl EconomyCatalog {
    pub(crate) fn add_alloy_chains(&mut self, minerals: &crate::catalog::Catalog) -> Result<()> {
        ensure!(
            self.goods.len() > 44 && self.recipes.len() + 8 <= GOODS,
            "insufficient catalog slots for alloy processing"
        );
        for (k, id) in ALLOY_GOODS {
            ensure!(
                self.goods[k].id == format!("reserved_{k}"),
                "alloy slot {k} occupied"
            );
            self.goods[k] = Good {
                id: id.into(),
                name: id.replace('_', " "),
                base_price: if k < 38 {
                    ORE_BASE_PRICE
                } else {
                    METAL_BASE_PRICE
                },
                cnp: [0.; 3],
                food_energy: 0.,
                delay_spoilage: None,
            };
        }
        for (raw, id, metal, labor) in SMELTING_CHAINS {
            let m = minerals
                .minerals
                .iter()
                .find(|m| m.id == id)
                .ok_or_else(|| anyhow::anyhow!("missing {id}"))?;
            let recovered = m.yield_fraction * SMELTING_RECOVERY_FRACTION;
            self.recipes.push(recipe(
                &[(raw, 1.), (6, SMELTING_FUEL_KG)],
                &[(metal, recovered)],
                labor,
                1. - recovered,
            ));
        }
        self.recipes.push(recipe(
            &[
                (38, BRONZE_COPPER_FRACTION),
                (39, BRONZE_TIN_FRACTION),
                (6, PROCESSING_FUEL_KG),
            ],
            &[(40, 1.)],
            ALLOY_WORKER_MONTHS,
            0.,
        ));
        self.recipes.push(recipe(
            &[(40, 1.)],
            &[(41, 1.)],
            BRONZE_TOOL_WORKER_MONTHS,
            0.,
        ));
        self.recipes.push(recipe(
            &[(42, 1.), (6, PROCESSING_FUEL_KG)],
            &[(40, REMELTING_RECOVERY_FRACTION)],
            REMELTING_WORKER_MONTHS,
            REMELTING_RESIDUE_KG,
        ));
        self.recipes.push(recipe(
            &[(38, 1.)],
            &[(43, 1.)],
            COPPER_TOOL_WORKER_MONTHS,
            0.,
        ));
        self.recipes.push(recipe(
            &[(44, 1.), (6, PROCESSING_FUEL_KG)],
            &[(38, REMELTING_RECOVERY_FRACTION)],
            REMELTING_WORKER_MONTHS,
            REMELTING_RESIDUE_KG,
        ));
        // Existing smelting losses become physical only from this boundary onward.
        for r in &mut self.recipes {
            if r.input[32..35].iter().sum::<f32>() > 0. {
                r.work[3] = r.input[32..35].iter().sum::<f32>() - r.output[2];
            }
        }
        self.validate()
    }
}
impl Generator {
    pub fn enable_alloy_processing(&mut self) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "alloys require a completed boundary"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let r = h
            .resources
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("enable shared resources first"))?;
        ensure!(
            !r.mineral_catalog.is_empty(),
            "enable mineral processing first"
        );
        if r.alloy_processing {
            return Ok(());
        }
        let mut catalog = h.economy_catalog.clone().unwrap();
        for (slot, _) in ALLOY_GOODS {
            ensure!(
                h.sites.iter().all(|s| s.economy.goods[slot] == 0.
                    && s.economy.initial[slot] == 0.
                    && s.economy.made[slot] == 0.
                    && s.economy.used[slot] == 0.)
                    && h.cargo.iter().all(|c| c.good as usize != slot)
                    && catalog
                        .recipes
                        .iter()
                        .all(|r| r.input[slot] == 0. && r.output[slot] == 0.),
                "alloy slot {slot} already has material or recipe references"
            );
        }
        catalog.add_alloy_chains(&self.catalog)?;
        h.economy_catalog = Some(catalog);
        let r = h.resources.as_mut().unwrap();
        r.alloy_processing = true;
        for source in r.sources.values_mut() {
            source.ore_good = source.mineral.as_deref().and_then(|m| ore_slot(m, true));
        }
        for s in &mut h.sites {
            s.economy.extraction[0] = r.sources[&s.cell].ore_good.unwrap_or(1) as f32;
            s.economy.extraction[1] = 1.;
            s.economy.residue[2] = s.economy.claim[1] * RESIDUE_CAPACITY_KG_PER_M2;
        }
        h.event("alloy_processing",None,None,"Copper and tin sources identified; bronze tools and separate scrap chains established. Future mineral residues occupy finite on-site disposal land; old losses are not reconstructed".into());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        civilization::{History, Site, Stocks},
        economy::Economy,
    };
    #[test]
    fn copper_and_tin_keep_identity_in_paid_cargo() {
        let mut h:History=serde_json::from_value(serde_json::json!({"version":2,"seed":17,"terrain_resolution":16,"source_epoch":0,"month":0,"civilizations":[],"sites":[],"people":[],"events":[],"shipments":[],"candidates":[],"initial_food":7200.,"initial_population":30.})).unwrap();
        let mut c = EconomyCatalog::bundled().unwrap();
        c.add_alloy_chains(&crate::catalog::Catalog::bundled().unwrap())
            .unwrap();
        h.economy_catalog = Some(c);
        h.nutrition_initial = [3240., 144., 21.6];
        for id in 0..3 {
            let mut e = Economy {
                policy: [0., 0., 0., 1.],
                finance: [1000., 1000., 0., 0.],
                logistics: [10000., 0., 0., 2.],
                ..Default::default()
            };
            e.prices.fill(1.);
            if id == 0 {
                e.targets[38] = 9.;
                e.targets[39] = 1.;
            } else {
                let k = if id == 1 { 38 } else { 39 };
                let kg = if id == 1 { 9. } else { 1. };
                e.goods[k] = kg;
                e.initial[k] = kg;
            }
            h.sites.push(Site {
                id,
                civilization: 0,
                island: 0,
                cell: id,
                name: format!("Trade fixture {id}"),
                founded: 0,
                abandoned: false,
                lifecycle: Default::default(),
                stocks: Stocks {
                    stock: [10., 2400., 0., 0.],
                    ..bytemuck::Zeroable::zeroed()
                },
                economy: e,
                demography: Default::default(),
            });
        }
        h.market_month(1.);
        assert_eq!(h.cargo.len(), 2);
        assert!(h.cargo.iter().any(|c| c.good == 38 && c.kg == 9.));
        assert!(h.cargo.iter().any(|c| c.good == 39 && c.kg == 1.));
        assert_eq!(h.sites[0].economy.goods[38], 0.);
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
        let cash = h.sites[0].economy.finance[0];
        h.month = h.cargo.iter().map(|c| c.arrives).max().unwrap();
        h.market_month(1.);
        assert_eq!(h.sites[0].economy.goods[38], 9.);
        assert_eq!(h.sites[0].economy.goods[39], 1.);
        assert_eq!(h.sites[0].economy.finance[0], cash);
        assert!(h.cargo.is_empty());
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    }
}
