use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Catalog {
    pub version: u32,
    #[serde(default)]
    pub geological_provinces: bool,
    #[serde(default)]
    pub producer_competition: bool,
    pub rocks: Vec<Rock>,
    pub minerals: Vec<Mineral>,
    pub soils: Vec<Soil>,
    pub plants: Vec<Plant>,
    pub biomes: Vec<Biome>,
    #[serde(default)]
    pub guilds: Vec<Guild>,
    #[serde(default)]
    pub microbes: Vec<Microbe>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rock {
    pub id: String,
    pub name: String,
    pub formation: u32,
    pub hardness: f32,
    pub permeability: f32,
    pub weathering: f32,
    #[serde(default = "rock_chemistry")]
    pub chemistry: [f32; 4], // P fraction, reactive fraction, nutrient release/year, trace fraction
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mineral {
    #[serde(default)]
    pub deposit_setting: DepositSetting,
    #[serde(default = "province_scale")]
    pub province_scale_km: f32,
    pub id: String,
    pub name: String,
    pub hosts: Vec<String>,
    pub formation: u32,
    pub abundance: f32,
    pub yield_fraction: f32,
    pub depth_m: f32,
    #[serde(default)]
    pub phosphorus_fraction: f32,
}
/// Game-scale formation environments, not resolved ore-body processes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u32)]
pub enum DepositSetting {
    #[default]
    Legacy = 0,
    Magmatic = 1,
    Hydrothermal = 2,
    Sedimentary = 3,
    Weathered = 4,
    Metamorphic = 5,
    Evaporite = 6,
}
impl DepositSetting {
    pub fn label(self) -> &'static str {
        match self {
            Self::Legacy => "legacy host potential",
            Self::Magmatic => "magmatic province",
            Self::Hydrothermal => "hydrothermal belt",
            Self::Sedimentary => "sedimentary province",
            Self::Weathered => "weathering profile",
            Self::Metamorphic => "metamorphic belt",
            Self::Evaporite => "evaporitic basin potential",
        }
    }
}
fn province_scale() -> f32 {
    350.
}
fn stable_salt(id: &str) -> u32 {
    id.bytes()
        .fold(2166136261u32, |h, b| (h ^ b as u32).wrapping_mul(16777619))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Soil {
    pub id: String,
    pub name: String,
    pub texture: String,
    pub fertility: f32,
    pub drainage: f32,
    pub parent_formation: u32,
    #[serde(default = "soil_chemistry")]
    pub chemistry: [f32; 4], // retention, sorption/year, leaching/year, decomposition/year
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plant {
    pub id: String,
    pub name: String,
    pub temp_min: f32,
    pub temp_max: f32,
    pub rain_min: f32,
    pub rain_max: f32,
    pub soil_min: f32,
    pub growth: f32,
    pub height_m: f32,
    pub outer: bool,
    pub substrate: u32,
    #[serde(default)]
    pub ecology: PlantEcology,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Biome {
    pub id: String,
    pub name: String,
    pub temp_min: f32,
    pub temp_max: f32,
    pub rain_min: f32,
    pub rain_max: f32,
    pub elevation_min: f32,
    pub elevation_max: f32,
    pub vegetation: f32,
    #[serde(default)]
    pub ecological_habitat: u32,
    pub color: [f32; 3],
}
fn body_mass() -> f32 {
    10.
}
fn rock_chemistry() -> [f32; 4] {
    [0.001, 0.05, 0.001, 0.02]
}
fn soil_chemistry() -> [f32; 4] {
    [0.7, 0.1, 0.05, 0.5]
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PlantEcology {
    pub layer: u32,
    pub nitrogen: f32,
    pub phosphorus: f32,
    pub maintenance: f32,
    pub symbiosis: f32,
    pub fixation: f32,
    pub shade: f32,
    pub turnover: f32,
}
impl Default for PlantEcology {
    fn default() -> Self {
        Self {
            layer: 0,
            nitrogen: 0.025,
            phosphorus: 0.002,
            maintenance: 0.08,
            symbiosis: 0.,
            fixation: 0.02,
            shade: 0.2,
            turnover: 0.15,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DietItem {
    /// Fraction of the guild assimilation efficiency usable for this food.
    #[serde(default = "unit_efficiency")]
    pub efficiency: f32,
    pub prey: u32,
    pub weight: f32,
}
fn unit_efficiency() -> f32 {
    1.
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guild {
    /// Accessible weighted food carbon at half maximum intake, kg C/m².
    #[serde(default)]
    pub food_half_saturation: f32,
    /// Animal prey become harder to capture below this density (kg C/m²).
    #[serde(default)]
    pub animal_prey_refuge: f32,
    pub id: String,
    pub name: String,
    pub prey: u32,
    #[serde(default)]
    pub diet: Vec<DietItem>,
    pub aquatic: bool,
    pub nitrogen: f32,
    pub phosphorus: f32,
    pub assimilation: f32,
    pub maintenance: f32,
    /// Maximum kg food C per kg consumer C per year, not an efficiency.
    pub feeding: f32,
    pub migration: f32,
    #[serde(default = "body_mass")]
    pub body_mass_kg: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Microbe {
    pub id: String,
    pub name: String,
    pub process: u32,
    pub efficiency: f32,
    pub rate: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Entry {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub c: [f32; 4],
    pub d: [f32; 4],
    pub ids: [u32; 4],
}
impl Catalog {
    pub fn bundled() -> Result<Self> {
        Self::parse(include_str!("../assets/catalog.toml"))
    }
    pub fn parse(s: &str) -> Result<Self> {
        let c: Self = toml::from_str(s).context("parsing catalog")?;
        c.validate()?;
        Ok(c)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (1..=2).contains(&self.version),
            "unsupported catalog version"
        );
        ensure!(
            (24..=32).contains(&self.rocks.len())
                && (24..=128).contains(&self.minerals.len())
                && (8..=32).contains(&self.soils.len())
                && (48..=256).contains(&self.plants.len())
                && (16..=64).contains(&self.biomes.len()),
            "catalog counts outside supported bounds"
        );
        let mut ids = HashSet::new();
        for id in self
            .rocks
            .iter()
            .map(|x| &x.id)
            .chain(self.minerals.iter().map(|x| &x.id))
            .chain(self.soils.iter().map(|x| &x.id))
            .chain(self.plants.iter().map(|x| &x.id))
            .chain(self.biomes.iter().map(|x| &x.id))
        {
            ensure!(
                !id.is_empty() && ids.insert(id),
                "empty or duplicate catalog ID: {id}"
            );
        }
        let unit = |x: f32| x.is_finite() && (0.0..=1.0).contains(&x);
        for r in &self.rocks {
            ensure!(
                r.formation < 3
                    && r.hardness.is_finite()
                    && r.hardness > 0.
                    && unit(r.permeability)
                    && unit(r.weathering),
                "invalid rock {}",
                r.id
            );
        }
        for m in &self.minerals {
            ensure!(
                !m.hosts.is_empty()
                    && m.hosts
                        .iter()
                        .all(|h| self.rocks.iter().any(|r| &r.id == h))
                    && unit(m.abundance)
                    && unit(m.yield_fraction)
                    && m.depth_m.is_finite()
                    && m.depth_m >= 0.
                    && m.formation < 3
                    && m.province_scale_km.is_finite()
                    && (10. ..=5000.).contains(&m.province_scale_km),
                "invalid mineral {}",
                m.id
            );
        }
        for s in &self.soils {
            ensure!(
                unit(s.fertility) && unit(s.drainage) && s.parent_formation < 3,
                "invalid soil {}",
                s.id
            );
        }
        for p in &self.plants {
            ensure!(
                [p.temp_min, p.temp_max, p.rain_min, p.rain_max, p.height_m]
                    .iter()
                    .all(|x| x.is_finite())
                    && p.temp_min < p.temp_max
                    && p.rain_min >= 0.
                    && p.rain_min < p.rain_max
                    && unit(p.soil_min)
                    && unit(p.growth)
                    && p.height_m > 0.
                    && p.substrate < 4,
                "invalid plant {}",
                p.id
            );
        }
        ensure!(
            self.plants.iter().filter(|p| p.outer).count() >= 12,
            "at least 12 original outer species required"
        );
        for b in &self.biomes {
            ensure!(
                [
                    b.temp_min,
                    b.temp_max,
                    b.rain_min,
                    b.rain_max,
                    b.elevation_min,
                    b.elevation_max
                ]
                .iter()
                .all(|x| x.is_finite())
                    && b.temp_min < b.temp_max
                    && b.rain_min >= 0.
                    && b.rain_min < b.rain_max
                    && b.elevation_min < b.elevation_max
                    && b.ecological_habitat <= 8
                    && unit(b.vegetation)
                    && b.color.iter().all(|x| unit(*x)),
                "invalid biome {}",
                b.id
            );
        }
        for r in &self.rocks {
            ensure!(
                r.chemistry.iter().all(|v| unit(*v)),
                "invalid rock chemistry"
            );
        }
        for m in &self.minerals {
            ensure!(unit(m.phosphorus_fraction), "invalid mineral chemistry");
        }
        for soil in &self.soils {
            ensure!(
                soil.chemistry.iter().all(|v| unit(*v)),
                "invalid soil chemistry"
            );
        }
        for plant in &self.plants {
            let e = &plant.ecology;
            ensure!(
                e.layer <= 5
                    && [
                        e.nitrogen,
                        e.phosphorus,
                        e.maintenance,
                        e.symbiosis,
                        e.fixation,
                        e.shade,
                        e.turnover
                    ]
                    .iter()
                    .all(|v| unit(*v))
                    && e.nitrogen > 0.
                    && e.phosphorus > 0.,
                "invalid producer traits"
            );
        }
        if self.version == 2 {
            ensure!(
                self.guilds.len() == 12 && self.microbes.len() == 8,
                "v2 requires twelve guilds and eight microbial groups"
            );
        }
        for (index, g) in self.guilds.iter().enumerate() {
            let mut prey = std::collections::HashSet::new();
            ensure!(
                g.diet.len() <= 4
                    && g.diet.iter().all(|d| {
                        (d.prey < 17 || [18, 22, 23].contains(&d.prey))
                            && d.prey != index as u32 + 5
                            && d.efficiency.is_finite()
                            && (0. ..=1.).contains(&d.efficiency)
                            && d.weight.is_finite()
                            && d.weight > 0.
                            && d.weight <= 1.
                            && prey.insert(d.prey)
                    }),
                "invalid guild diet"
            );
            ensure!(
                ids.insert(&g.id)
                    && g.food_half_saturation.is_finite()
                    && g.food_half_saturation >= 0.
                    && g.animal_prey_refuge.is_finite()
                    && g.animal_prey_refuge >= 0.
                    && (g.animal_prey_refuge == 0. || g.food_half_saturation > 0.)
                    && g.prey < 26
                    && g.body_mass_kg.is_finite()
                    && g.body_mass_kg > 0.
                    && g.feeding.is_finite()
                    && (0. ..=12.).contains(&g.feeding)
                    && [
                        g.nitrogen,
                        g.phosphorus,
                        g.assimilation,
                        g.maintenance,
                        g.migration
                    ]
                    .iter()
                    .all(|v| unit(*v))
                    && g.nitrogen > 0.
                    && g.phosphorus > 0.,
                "invalid guild"
            );
        }
        ensure!(
            self.guilds
                .iter()
                .enumerate()
                .all(|(i, g)| g.id == format!("guild_{i}")),
            "guild slots require stable guild_0 through guild_11 IDs"
        );
        let mut processes = HashSet::new();
        for m in &self.microbes {
            ensure!(
                ids.insert(&m.id)
                    && processes.insert(m.process)
                    && m.process < 8
                    && unit(m.efficiency)
                    && unit(m.rate),
                "invalid microbe"
            );
        }
        Ok(())
    }
    pub fn entries(&self) -> Vec<Entry> {
        let mut out = Vec::new();
        let z = [0.; 4];
        for r in &self.rocks {
            out.push(Entry {
                a: [r.hardness, r.permeability, r.weathering, 0.],
                b: r.chemistry,
                c: z,
                d: z,
                ids: [r.formation, stable_salt(&r.id), 0, 0],
            });
        }
        for m in &self.minerals {
            let mask = m.hosts.iter().fold(0, |v, h| {
                v | (1u32 << self.rocks.iter().position(|r| &r.id == h).unwrap())
            });
            out.push(Entry {
                a: [
                    m.abundance,
                    m.yield_fraction,
                    m.depth_m,
                    m.phosphorus_fraction,
                ],
                b: [m.province_scale_km, 0., 0., 0.],
                c: z,
                d: z,
                ids: [
                    mask,
                    m.formation,
                    m.deposit_setting as u32,
                    stable_salt(&m.id),
                ],
            });
        }
        for s in &self.soils {
            out.push(Entry {
                a: [s.fertility, s.drainage, 0., 0.],
                b: s.chemistry,
                c: z,
                d: z,
                ids: [s.parent_formation, 0, 0, 0],
            });
        }
        for p in &self.plants {
            out.push(Entry {
                a: [p.temp_min, p.temp_max, p.rain_min, p.rain_max],
                b: [p.soil_min, p.growth, p.height_m, 0.],
                c: [
                    p.ecology.layer as f32,
                    p.ecology.nitrogen,
                    p.ecology.phosphorus,
                    p.ecology.maintenance,
                ],
                d: [
                    p.ecology.symbiosis,
                    p.ecology.fixation,
                    p.ecology.shade,
                    p.ecology.turnover,
                ],
                ids: [p.outer as u32, p.substrate, 0, 0],
            });
        }
        for b in &self.biomes {
            out.push(Entry {
                a: [b.temp_min, b.temp_max, b.rain_min, b.rain_max],
                b: [b.elevation_min, b.elevation_max, b.vegetation, 0.],
                c: [b.color[0], b.color[1], b.color[2], 1.],
                d: z,
                ids: [b.ecological_habitat, 0, 0, 0],
            });
        }
        for g in &self.guilds {
            let mut prey = [0.; 4];
            let mut weights = [0.; 4];
            let mut efficiencies = [65535u32; 4];
            if g.diet.is_empty() {
                prey[0] = g.prey as f32;
                weights[0] = 1.;
            } else {
                for (k, d) in g.diet.iter().enumerate() {
                    prey[k] = d.prey as f32;
                    weights[k] = d.weight;
                    efficiencies[k] = (d.efficiency * 65535.).round() as u32;
                }
            }
            out.push(Entry {
                a: [g.nitrogen, g.phosphorus, g.assimilation, g.maintenance],
                b: [
                    g.feeding,
                    g.migration,
                    g.body_mass_kg,
                    g.food_half_saturation,
                ],
                c: prey,
                d: weights,
                ids: [
                    // All prey IDs already live in c; reuse this word without growing tables.
                    g.animal_prey_refuge.to_bits(),
                    g.aquatic as u32,
                    efficiencies[0] | (efficiencies[1] << 16),
                    efficiencies[2] | (efficiencies[3] << 16),
                ],
            });
        }
        for m in &self.microbes {
            out.push(Entry {
                a: [m.efficiency, m.rate, 0., 0.],
                b: z,
                c: z,
                d: z,
                ids: [m.process, 0, 0, 0],
            });
        }
        out
    }
    pub fn counts(&self) -> [u32; 4] {
        [
            self.rocks.len() as u32,
            self.minerals.len() as u32,
            self.soils.len() as u32,
            self.plants.len() as u32,
        ]
    }
}
