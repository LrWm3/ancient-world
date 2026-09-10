//! Bounded material-specific objects compiled into ordinary conserved goods/recipes.
use crate::economy::{EconomyCatalog, Good, Recipe, GOODS};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialCatalog {
    pub variants: Vec<Variant>,
    pub methods: Vec<Method>,
    pub roofs: Vec<Roof>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    pub slot: usize,
    pub role: String,
    pub inputs: Vec<(String, f32)>,
    pub work: f32,
    pub industry: u32,
    pub service: f32,
    pub wear: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Method {
    pub id: String,
    pub wall: String,
    pub wall_kg: f32,
    pub work: f32,
    pub wear: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Roof {
    pub good: String,
    pub kg: f32,
    pub work: f32,
    pub wear: f32,
}
impl MaterialCatalog {
    pub fn bundled() -> Result<Self> {
        Ok(toml::from_str(include_str!("../assets/materials.toml"))?)
    }
    pub fn validate(&self, c: &EconomyCatalog) -> Result<()> {
        ensure!(
            self.variants.len() <= 6
                && !self.methods.is_empty()
                && self.methods.len() <= 16
                && !self.roofs.is_empty()
                && self.roofs.len() <= 16,
            "material table bounds"
        );
        let mut slots = std::collections::BTreeSet::new();
        for v in &self.variants {
            ensure!(
                (45..=50).contains(&v.slot)
                    && slots.insert(v.slot)
                    && !v.inputs.is_empty()
                    && v.inputs.len() <= 4,
                "invalid material variant slots"
            );
            ensure!(
                v.work.is_finite()
                    && v.work > 0.
                    && v.industry < 4
                    && v.service.is_finite()
                    && v.service > 0.
                    && (0. ..=1.).contains(&v.wear),
                "invalid material method units"
            );
            ensure!(
                matches!(
                    v.role.as_str(),
                    "container" | "digging" | "cutting" | "breaking" | "roof"
                ),
                "unknown object role"
            );
            for (id, mass) in &v.inputs {
                ensure!(
                    c.index(id).is_some() && mass.is_finite() && *mass > 0.,
                    "unknown or invalid component"
                );
                ensure!(
                    matches!(id.as_str(), "wood" | "metal" | "bricks" | "pottery"),
                    "unsupported material form"
                );
                if matches!(v.role.as_str(), "cutting" | "breaking") {
                    ensure!(
                        !matches!(id.as_str(), "bricks" | "pottery"),
                        "brittle ceramics cannot be impact/cutting tools"
                    );
                }
            }
            if matches!(v.role.as_str(), "cutting" | "breaking") {
                ensure!(
                    v.inputs.iter().any(|(g, _)| g == "metal"),
                    "cutting/breaking tool requires a metal working head"
                );
            }
        }
        for m in &self.methods {
            ensure!(
                matches!(m.wall.as_str(), "wood" | "bricks" | "metal")
                    && c.index(&m.wall).is_some()
                    && m.wall_kg.is_finite()
                    && m.wall_kg > 0.
                    && m.work.is_finite()
                    && m.work > 0.
                    && (0. ..=1.).contains(&m.wear),
                "invalid structural method"
            );
        }
        for r in &self.roofs {
            ensure!(
                matches!(r.good.as_str(), "wood" | "roof_tiles" | "metal")
                    && c.index(&r.good).is_some()
                    && r.kg.is_finite()
                    && r.kg > 0.
                    && r.work.is_finite()
                    && r.work > 0.
                    && (0. ..=1.).contains(&r.wear),
                "invalid roof method"
            );
        }
        Ok(())
    }
    pub fn compile(&self, c: &mut EconomyCatalog) -> Result<()> {
        ensure!(
            c.recipes.len() + self.variants.len() <= GOODS,
            "material recipes exceed GPU table"
        );
        for v in &self.variants {
            ensure!(
                v.slot < c.goods.len() && c.goods[v.slot].id == format!("reserved_{}", v.slot),
                "material slot occupied"
            );
            let mut input = [0.; GOODS];
            let mut chem = [0.; 3];
            let mut mass = 0.;
            let mut price = 0.;
            for (id, kg) in &v.inputs {
                let k = c
                    .index(id)
                    .ok_or_else(|| anyhow::anyhow!("unknown material {id}"))?;
                input[k] += kg;
                mass += kg;
                price += kg * c.goods[k].base_price;
                for (x, ratio) in chem.iter_mut().zip(c.composition(k)) {
                    *x += kg * ratio;
                }
            }
            ensure!(mass.is_finite() && mass > 0., "invalid object mass");
            c.goods[v.slot] = Good {
                id: v.id.clone(),
                name: v.id.replace('_', " "),
                base_price: price / mass * 1.5,
                cnp: chem.map(|x| x / mass),
                food_energy: 0.,
                delay_spoilage: None,
            };
            let mut output = [0.; GOODS];
            output[v.slot] = mass;
            c.recipes.push(Recipe {
                input,
                output,
                work: [v.work, 0., v.industry as f32, 0.],
            });
        }
        self.validate(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU; small multi-seed history smoke"]
    fn material_history_seed_smoke() {
        for seed in [17, 81, 256] {
            let mut g = crate::gpu::Generator::new(
                pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
                crate::config::Config {
                    seed,
                    resolution: 32,
                    ecology_resolution: 16,
                    ..Default::default()
                },
                crate::catalog::Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.run_epochs(1).unwrap();
            g.found_civilizations(5).unwrap();
            g.enable_society().unwrap();
            g.advance_history(120).unwrap();
            if seed == 17 {
                let path = std::env::temp_dir().join(format!(
                    "ancient-material-check-{}.world",
                    std::process::id()
                ));
                g.save(&path).unwrap();
                let mut resumed = crate::gpu::Generator::load(g.gpu.clone(), &path).unwrap();
                std::fs::remove_file(path).unwrap();
                g.advance_history(12).unwrap();
                for _ in 0..12 {
                    resumed.advance_history(1).unwrap();
                }
                assert_eq!(
                    serde_json::to_value(&g.civilizations).unwrap(),
                    serde_json::to_value(&resumed.civilizations).unwrap()
                );
            }
            let h = g.civilizations.as_ref().unwrap();
            let culture = h.culture.as_ref().unwrap();
            let facilities: Vec<_> = culture
                .institutions
                .iter()
                .filter_map(|n| {
                    n.capacity
                        .as_ref()
                        .and_then(|c| c.building.as_ref())
                        .and_then(|b| b.facility.as_ref())
                })
                .collect();
            let made: Vec<f32> = (45..=50)
                .map(|k| h.sites.iter().map(|s| s.economy.made[k]).sum())
                .collect();
            eprintln!("seed {seed}: population {:.0}, institutions {}, facilities {}, usable {:.1}, expansion {}, produced {:?}, residuals {:?}",h.sites.iter().map(|s|s.stocks.stock[0]).sum::<f32>(),culture.institutions.len(),facilities.len(),facilities.iter().map(|f|f.usable()).sum::<f32>(),h.events.iter().filter(|e|e.kind=="meeting_place_expanded").count(),made,h.economy_residuals());
            assert!(h.sites.iter().all(|s| s.economy.valid()));
            assert!(h.economy_residuals().iter().all(|x| x.abs() < 1e-4));
        }
    }
    #[test]
    fn compatible_objects_preserve_composition_and_reject_brittle_heads() {
        let c = EconomyCatalog::bundled().unwrap();
        let m = c.materials.as_ref().unwrap();
        for (actual, expected) in c.goods[48].cnp.into_iter().zip([0.2, 0.0008, 0.00008]) {
            assert!((actual - expected).abs() < 1e-8);
        }
        for v in &m.variants {
            let r = c.recipes.iter().find(|r| r.output[v.slot] > 0.).unwrap();
            assert_eq!(r.input.iter().sum::<f32>(), r.output.iter().sum::<f32>());
            for k in 0..3 {
                let input: f32 = r
                    .input
                    .iter()
                    .enumerate()
                    .map(|(g, q)| q * c.composition(g)[k])
                    .sum();
                let output: f32 = r
                    .output
                    .iter()
                    .enumerate()
                    .map(|(g, q)| q * c.composition(g)[k])
                    .sum();
                assert!((input - output).abs() < 1e-6);
            }
        }
        let mut invalid = m.clone();
        invalid.variants[3].inputs = vec![("pottery".into(), 1.)];
        assert!(invalid.validate(&c).is_err());
        invalid = m.clone();
        invalid.variants[0].slot = 64;
        assert!(invalid.validate(&c).is_err());
        let mut legacy = serde_json::to_value(c).unwrap();
        legacy.as_object_mut().unwrap().remove("materials");
        let old: EconomyCatalog = serde_json::from_value(legacy).unwrap();
        assert!(old.materials.is_none());
    }
}
