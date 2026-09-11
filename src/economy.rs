//! Managed ecological plots, material inventories, recipes and conservative markets.
use crate::{civilization::History, gpu::Generator, grid};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
pub const GOODS: usize = 64;
pub const FOOD: usize = 63;
pub const IDS: [&str; 8] = [
    "wood", "ore", "metal", "tools", "clay", "bricks", "charcoal", "pottery",
];
pub const FOOD_CNP: [f64; 3] = [0.45, 0.02, 0.003];
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Good {
    pub id: String,
    pub name: String,
    pub base_price: f32,
    #[serde(default)]
    pub cnp: [f32; 3],
    #[serde(default)]
    pub food_energy: f32,
    /// Fraction lost per month held by a transport flood. Missing preserves legacy rates.
    #[serde(default)]
    pub delay_spoilage: Option<f32>,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Recipe {
    #[serde(with = "slots")]
    pub input: [f32; GOODS],
    #[serde(with = "slots")]
    pub output: [f32; GOODS],
    /// Worker-months/batch, required knowledge bit + 1, workshop family (0–3), inorganic residue kg/batch.
    pub work: [f32; 4],
}
/// Stochastic regional forcing during social time; not a global climate solver.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct HistoryWeather {
    pub drought_probability: f32,
    pub drought_severity: f32,
    pub regime_months: u32,
    pub storm_probability: f32,
    pub storm_multiplier: f32,
}
impl Default for HistoryWeather {
    fn default() -> Self {
        Self {
            drought_probability: 0.,
            drought_severity: 0.,
            regime_months: 48,
            storm_probability: 0.04,
            storm_multiplier: 3.,
        }
    }
}
/// Archived market rules. Missing settings retain the original direct-route market.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MarketSettings {
    /// Adjust quotes from previous prices, replacement costs and observed deliveries.
    pub adaptive_prices: bool,
    pub network_trade: bool,
    pub max_distance_km: f32,
    #[serde(with = "slots")]
    pub reserve_per_person: [f32; GOODS],
    pub food_reserve_months: f32,
}
impl Default for MarketSettings {
    fn default() -> Self {
        Self {
            adaptive_prices: false,
            network_trade: false,
            max_distance_km: 1500.,
            reserve_per_person: [3.; GOODS],
            food_reserve_months: 12.,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomyCatalog {
    #[serde(default)]
    pub materials: Option<crate::materials::MaterialCatalog>,
    #[serde(default)]
    pub production: crate::production::ProductionSettings,
    #[serde(default)]
    pub agriculture: Option<crate::agriculture::AgricultureCatalog>,
    pub version: u32,
    pub goods: Vec<Good>,
    pub recipes: Vec<Recipe>,
    #[serde(default)]
    pub market: MarketSettings,
    #[serde(default)]
    pub weather: HistoryWeather,
}
impl EconomyCatalog {
    pub fn delay_spoilage(&self, good: usize) -> f32 {
        self.goods
            .get(good)
            .and_then(|g| g.delay_spoilage)
            .unwrap_or(if good == FOOD { 0.2 } else { 0. })
    }

    pub fn index(&self, id: &str) -> Option<usize> {
        if id == "food" {
            Some(FOOD)
        } else {
            self.goods.iter().position(|g| g.id == id)
        }
    }
    pub fn composition(&self, index: usize) -> [f32; 3] {
        if self.version == 1 {
            return match index {
                0 => [0.5, 0.002, 0.0002],
                6 => [1., 0., 0.],
                _ => [0.; 3],
            };
        }
        self.goods.get(index).map_or([0.; 3], |g| g.cnp)
    }

    pub fn bundled() -> Result<Self> {
        let mut c: Self = toml::from_str(include_str!("../assets/economy.toml"))?;
        c.agriculture = Some(crate::agriculture::AgricultureCatalog::bundled());
        for r in &mut c.recipes {
            if r.output[16] + r.output[17] + r.output[18] > 0. {
                r.work[1] = 6.;
            }
            if r.output[21] > 0. {
                r.work[1] = 11.;
            }
            if r.output[22] + r.output[23] > 0. {
                r.work[1] = 5.;
            }
        }
        let materials = crate::materials::MaterialCatalog::bundled()?;
        materials.compile(&mut c)?;
        c.materials = Some(materials);
        c.validate()?;
        Ok(c)
    }
    pub fn validate(&self) -> Result<()> {
        if let Some(m) = &self.materials {
            m.validate(self)?;
        }
        ensure!(
            self.production.enabled
                || !(self.production.replacement_tool_jobs || self.production.toolmaking_expertise),
            "toolmaking policies require planned production"
        );
        ensure!(
            !self.production.food_security_labor
                || (self.production.enabled
                    && self.production.adaptive_labor
                    && !self.production.diagnostic_fixed_labor),
            "food-security staffing requires adaptive planning and excludes fixed staffing"
        );
        ensure!(
            !self.production.diagnostic_fixed_labor || self.production.enabled,
            "diagnostic fixed staffing requires production planning"
        );
        ensure!(
            (0. ..=1.).contains(&self.production.waterworks_target_fraction),
            "invalid waterworks target fraction"
        );
        ensure!(
            (0.25..=2.).contains(&self.production.initial_housing_per_person),
            "invalid initial housing allowance"
        );
        ensure!(
            (self.version == 1 || self.version == 2)
                && self.goods.len() <= GOODS
                && self.goods.len() >= 8
                && self.recipes.len() <= 64,
            "invalid economy catalog version/count"
        );
        ensure!(
            self.market.max_distance_km.is_finite()
                && (1. ..=10000.).contains(&self.market.max_distance_km)
                && self.market.reserve_per_person[..self.goods.len()]
                    .iter()
                    .all(|v| v.is_finite() && (0. ..=100.).contains(v))
                && (6. ..=36.).contains(&self.market.food_reserve_months),
            "invalid market reserves or range"
        );
        ensure!(
            (0. ..=1.).contains(&self.weather.drought_probability)
                && (0. ..=1.).contains(&self.weather.drought_severity)
                && (12..=120).contains(&self.weather.regime_months)
                && (0. ..=1.).contains(&self.weather.storm_probability)
                && (1. ..=8.).contains(&self.weather.storm_multiplier),
            "invalid historical weather settings"
        );
        for (g, id) in self.goods.iter().zip(IDS) {
            ensure!(
                g.id == id && g.base_price.is_finite() && g.base_price > 0. && g.base_price <= 1e6,
                "invalid good ID or price"
            );
        }
        if let Some(a) = &self.agriculture {
            a.validate(self)?;
        }
        ensure!(
            (0. ..=1.).contains(&self.production.contract_margin)
                && (1. ..=10000.).contains(&self.production.storage_kg_per_person)
                && (1. ..=1000.).contains(&self.production.land_freight_kg_per_person),
            "invalid production capacity"
        );
        let mut ids = std::collections::BTreeSet::new();
        for g in &self.goods {
            ensure!(
                g.delay_spoilage
                    .is_none_or(|rate| (0. ..=1.).contains(&rate)),
                "invalid delayed-cargo spoilage rate"
            );
            ensure!(
                ids.insert(&g.id)
                    && g.base_price.is_finite()
                    && g.base_price > 0.
                    && g.cnp
                        .iter()
                        .all(|v| v.is_finite() && (0. ..=1.).contains(v))
                    && g.cnp.iter().sum::<f32>() <= 1.
                    && g.food_energy.is_finite()
                    && g.food_energy >= 0.,
                "invalid material composition or ID"
            );
        }
        for r in &self.recipes {
            ensure!(
                r.input
                    .iter()
                    .chain(&r.output)
                    .chain(&r.work)
                    .all(|x| x.is_finite() && *x >= 0.)
                    && r.work[0] > 0.
                    && r.work[1] <= 32.
                    && r.work[1].fract() == 0.
                    && r.work[2] < 4.
                    && r.work[2].fract() == 0.,
                "invalid recipe units or labor"
            );
            ensure!(
                r.input.iter().sum::<f32>() >= r.output.iter().sum::<f32>()
                    && r.output.iter().sum::<f32>() > 0.,
                "recipe creates mass or has no output"
            );
            let iron_content = |amounts: &[f32; GOODS]| -> f32 {
                [
                    ("hematite_ore", 0.7),
                    ("magnetite_ore", 0.72),
                    ("limonite_ore", 0.5),
                ]
                .iter()
                .filter_map(|(id, f)| self.index(id).map(|i| amounts[i] * f))
                .sum()
            };
            let variant_content = |q: &[f32; GOODS], material: &str| -> f32 {
                self.materials.as_ref().map_or(0., |m| {
                    m.variants
                        .iter()
                        .map(|v| {
                            let total: f32 = v.inputs.iter().map(|(_, kg)| kg).sum();
                            let part: f32 = v
                                .inputs
                                .iter()
                                .filter(|(id, _)| id == material)
                                .map(|(_, kg)| kg)
                                .sum();
                            q[v.slot] * part / total
                        })
                        .sum()
                })
            };
            ensure!(
                variant_content(&r.output, "metal")
                    + r.output[2]
                    + r.output[3]
                    + r.output[22]
                    + r.output[23]
                    + r.output[29]
                    + iron_content(&r.output)
                    <= variant_content(&r.input, "metal")
                        + iron_content(&r.input)
                        + r.input[1] * 0.5
                        + r.input[2]
                        + r.input[3]
                        + r.input[22]
                        + r.input[23]
                        + r.input[29]
                    && variant_content(&r.output, "bricks") + r.output[5] + r.output[7]
                        <= variant_content(&r.input, "bricks")
                            + r.input[4]
                            + r.input[5]
                            + r.input[7],
                "recipe creates metal or ceramic feedstock"
            );
            for element in 1..3 {
                let sum = |q: &[f32; GOODS]| -> f32 {
                    self.goods
                        .iter()
                        .enumerate()
                        .map(|(i, g)| q[i] * crate::metallurgy::metals(&g.id)[element])
                        .sum()
                };
                ensure!(
                    sum(&r.output) <= sum(&r.input) + 1e-6,
                    "recipe creates copper or tin"
                );
            }
            let dry_input: f32 = self
                .goods
                .iter()
                .enumerate()
                .filter(|(_, g)| g.cnp == [0.; 3] && g.food_energy == 0.)
                .map(|(i, _)| r.input[i])
                .sum();
            let dry_output: f32 = self
                .goods
                .iter()
                .enumerate()
                .filter(|(_, g)| g.cnp == [0.; 3] && g.food_energy == 0.)
                .map(|(i, _)| r.output[i])
                .sum();
            ensure!(
                r.work[3] <= (dry_input - dry_output).max(0.) + 1e-6,
                "residue exceeds inorganic processing loss"
            );
            for k in 0..3 {
                let input: f32 = (0..GOODS)
                    .map(|g| r.input[g] * self.composition(g)[k])
                    .sum();
                let output: f32 = (0..GOODS)
                    .map(|g| r.output[g] * self.composition(g)[k])
                    .sum();
                ensure!(output <= input + 1e-6, "recipe creates C/N/P");
            }
            // Only wood/charcoal carry ecological carbon. Nutrients in burned wood return to compost.
            ensure!(
                r.output[0] == 0.
                    && r.output[1] == 0.
                    && r.output[4] == 0.
                    && r.output[6] <= r.input[0] * 0.5 + r.input[6],
                "recipe creates primary resources or carbon"
            );
        }
        Ok(())
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Economy {
    /// Named agricultural attendance: enabled, granted, used, requested.
    #[serde(default)]
    pub farm_workers: [f32; 4],
    /// Forestry/mining granted attendance and productive work.
    #[serde(default)]
    pub extraction_workers: [f32; 4],
    /// Last production inputs: tool multiplier, cultivated ha, effective tools kg, population.
    #[serde(default)]
    pub production_probe: [f32; 4],
    /// Remembered food pressure, target pressure, food reserve months, effective tool deficit.
    #[serde(default)]
    pub food_labor: [f32; 4],
    /// Toolmaking: expertise 0–1, priority enabled, learning enabled, cumulative worker-months.
    #[serde(default)]
    pub tool_craft: [f32; 4],
    /// Monthly priority budget, priority work, toolmaking work, completed tool kg.
    #[serde(default)]
    pub tool_work: [f32; 4],
    /// Subset of planned batches serving local working-tool demand.
    #[serde(default = "crate::production::empty_slots", with = "slots")]
    pub tool_orders: [f32; GOODS],
    /// On-site mineral residue kg: held, cumulative deposited, fixed capacity, blocked recipe attempts.
    #[serde(default)]
    pub residue: [f32; 4],
    /// Current source raw-good index; zero retains legacy ore. Not an inventory.
    #[serde(default)]
    pub extraction: [f32; 4],
    /// Pending managed C/N/P kg and water m3, committed to the fine river network.
    #[serde(default)]
    pub return_flow: [f32; 4],
    /// Enabled, released to wilderness, requested abandonment, cumulative released hectares.
    #[serde(default)]
    pub land_return: [f32; 4],
    #[serde(default = "crate::production::empty_slots", with = "slots")]
    pub targets: [f32; GOODS],
    #[serde(default = "crate::production::empty_slots", with = "slots")]
    pub orders: [f32; GOODS],
    /// Dry storage capacity, incoming dry cargo kg, unused craft worker-months, policy (0 legacy, 1 orders, 2 adaptive, 3 diagnostic fixed staffing, 4 food security, 5 food pressure without maintenance).
    #[serde(default)]
    pub logistics: [f32; 4],
    /// Warehouse timber/bricks kg, fixed baseline yard capacity kg, cumulative construction work.
    #[serde(default)]
    pub storage: [f32; 4],
    /// Target warehouse kg, cumulative worn material kg, newly built capacity kg, policy enabled.
    #[serde(default)]
    pub storage_plan: [f32; 4],
    /// Installed timber/bricks kg, recorded baseline shelter places, cumulative construction work.
    #[serde(default)]
    pub housing: [f32; 4],
    /// Target additional places, cumulative worn kg, monthly new places, policy enabled.
    #[serde(default)]
    pub housing_plan: [f32; 4],
    /// Installed timber/bricks kg, cumulative construction work, enabled policy.
    #[serde(default)]
    pub waterworks: [f32; 4],
    /// Target served residents, worn kg, monthly added service capacity, operating coverage.
    #[serde(default)]
    pub waterworks_plan: [f32; 4],
    /// Domestic water used m3, monthly shortfall fraction, cumulative operation work, served residents.
    #[serde(default)]
    pub water_service: [f32; 4],
    /// Installed wood, bricks, tools kg; workshop policy enabled.
    #[serde(default)]
    pub workshop: [f32; 4],
    /// Smoothed target units, cumulative worn kg, monthly built units, used capacity.
    #[serde(default)]
    pub workshop_plan: [f32; 4],
    /// Per industry: assigned units, target units, monthly work; first w enables specialization.
    #[serde(default)]
    pub workshop_types: [[f32; 4]; 4],
    /// Subset of installed units leased to private operators, never additional equipment.
    #[serde(default)]
    pub enterprise_lease: [f32; 4],
    /// Prepaid worker-month limits and actual GPU completed work by workshop family.
    #[serde(default)]
    pub enterprise_plan: [f32; 4],
    #[serde(default)]
    pub enterprise_used: [f32; 4],
    /// Enabled, receiving ecology cell + 1, receiving area m2, known-practice bitmask.
    #[serde(default)]
    pub management: [f32; 4],
    /// Land fraction, standing crop kg, seed kg, cumulative harvest kg.
    #[serde(default)]
    pub crops: [[f32; 4]; 6],
    /// Live kg, cumulative births kg, mortality kg, products kg.
    #[serde(default)]
    pub herds: [[f32; 4]; 3],
    /// Crop growth kg, feed kg, fish kg, food processing loss kg.
    #[serde(default)]
    pub agriculture: [f32; 4],
    #[serde(with = "slots")]
    pub goods: [f32; GOODS],
    #[serde(with = "slots")]
    pub made: [f32; GOODS],
    #[serde(with = "slots")]
    pub used: [f32; GOODS],
    #[serde(with = "slots")]
    pub initial: [f32; GOODS],
    #[serde(with = "slots")]
    pub prices: [f32; GOODS],
    /// Absolute kg C/N/P; w is living-history waterlogging/cleanup intensity (0–1).
    pub soil: [f32; 4],
    pub detritus: [f32; 4],
    pub forest: [f32; 4],
    /// Finite phosphorus kg, ore kg, clay kg; cumulative craft waste kg.
    pub reserves: [f32; 4],
    /// Signed external C/N/P kg; w reserves monthly research labor from GPU crafting.
    pub external: [f32; 4],
    pub baseline: [f32; 4],
    /// Water m3: stored, initial, rain/reclaimed input, evaporation/runoff/domestic outflow.
    pub water: [f32; 4],
    /// Cash, initial cash, cumulative sales, cumulative purchases.
    pub finance: [f32; 4],
    /// Monthly workers: farming, forestry, mining, crafting.
    pub labor: [f32; 4],
    /// Eco cell ID, reserved area m2, whole eco cell area m2, initialized flag.
    pub claim: [f32; 4],
    /// Crop constraint (0 none, 1 N, 2 P, 3 water), output kg, compost N, compost P.
    pub diagnostics: [f32; 4],
    /// Fallow fraction, manure return fraction, rainwater storage investment, market open.
    pub policy: [f32; 4],
    /// Installed timber, tools and fiber kg; adaptive policy flag.
    #[serde(default)]
    pub fishery: [f32; 4],
    /// Desired crew, actual fishing worker-months, construction worker-months, monthly catch kg.
    #[serde(default)]
    pub fishery_plan: [f32; 4],
    /// Max worker share, kg/worker-month, carbon-density half saturation, reserve months.
    #[serde(default)]
    pub fishery_config: [f32; 4],
    /// Cumulative worn kg, cumulative work, cumulative installed kg, latest stock response (0–1).
    #[serde(default)]
    pub fishery_stats: [f32; 4],
    /// Primitive trap timber kg, cumulative installed/worn kg, enabled flag.
    #[serde(default)]
    pub fishery_traps: [f32; 4],
    /// Smoothed alternative food/worker-month, expected fishing return, allocation factor, flag.
    #[serde(default)]
    pub fishery_choice: [f32; 4],
    /// Peak installed waterworks capacity, repair priority flag, monthly/cumulative repair work.
    #[serde(default)]
    pub waterworks_recovery: [f32; 4],
}
impl Default for Economy {
    fn default() -> Self {
        bytemuck::Zeroable::zeroed()
    }
}
impl Economy {
    pub fn waterworks_capacity(&self) -> f32 {
        (self.waterworks[0] / 2.).min(self.waterworks[1] / 4.)
    }
    pub fn housing_capacity(&self) -> f32 {
        self.housing[2] + (self.housing[0] / 2.).min(self.housing[1] / 3.)
    }
    pub fn crowding(&self, population: f32) -> f32 {
        if self.housing_plan[3] < 0.5 {
            return 0.;
        }
        ((population - self.housing_capacity()) / population.max(1.)).clamp(0., 1.)
    }
    pub fn housing_accepts(&self, population: f32) -> bool {
        self.housing_plan[3] < 0.5 || population <= self.housing_capacity()
    }
    pub fn container_capacity(&self, c: &EconomyCatalog) -> f32 {
        c.materials.as_ref().map_or(0., |m| {
            (self.goods[7]
                + m.variants
                    .iter()
                    .filter(|v| v.role == "container")
                    .map(|v| self.goods[v.slot] * v.service)
                    .sum::<f32>())
            .min(self.storage_capacity() * 0.2)
        })
    }
    pub fn storage_capacity(&self) -> f32 {
        self.storage[2] + (self.storage[0] / 0.02).min(self.storage[1] / 0.03)
    }
    pub fn new(
        g: &Generator,
        cell: u32,
        hectares: f32,
        founder: bool,
        catalog: &EconomyCatalog,
    ) -> Self {
        let n = g.config.resolution;
        let m = g.config.eco_resolution();
        let id = cell / (n * n) * m * m + (cell / n % n) / (n / m) * m + (cell % n) / (n / m);
        let mut e = Self {
            claim: [
                id as f32,
                hectares * 10000.,
                (grid::solid_angle(id, m) * (g.config.radius_km as f64 * 1000.).powi(2)) as f32,
                0.,
            ],
            policy: [0.15, 0.85, 0.25, 1.],
            finance: [
                if founder { 10000. } else { 0. },
                if founder { 10000. } else { 0. },
                0.,
                0.,
            ],
            ..Self::default()
        };
        if catalog.version >= 2 {
            e.management[0] = 1.;
            let a = catalog.agriculture.as_ref().expect("agricultural catalog");
            let fractions: [f32; 6] = std::array::from_fn(|i| a.crops[i].land_share);
            for (i, fraction) in fractions.into_iter().enumerate() {
                e.crops[i][0] = fraction;
                if founder {
                    e.crops[i][2] = 2.;
                }
            }
            if founder {
                for (i, mass) in a.herds.iter().map(|h| h.initial_kg).enumerate() {
                    e.herds[i][0] = mass;
                }
            }
            // All seed and breeding stock enter once as a declared founding inventory.
            for k in 0..3 {
                e.external[k] += e
                    .crops
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        c[2] * catalog.composition(
                            catalog.agriculture.as_ref().map_or(8 + i, |a| {
                                catalog.index(&a.crops[i].good).expect("validated crop")
                            }),
                        )[k]
                    })
                    .sum::<f32>()
                    + e.herds.iter().map(|a| a[0]).sum::<f32>() * [0.25, 0.04, 0.003][k];
            }
        }
        if founder {
            e.goods[3] = 60.;
            e.initial[3] = 60.;
        }
        for (i, good) in catalog.goods.iter().enumerate() {
            e.prices[i] = good.base_price;
        }
        e
    }
    pub fn valid(&self) -> bool {
        let bytes = bytemuck::bytes_of(self);
        let all = bytemuck::cast_slice::<u8, f32>(bytes);
        all.iter().all(|v| v.is_finite())
            && [0., 1., 32., 33., 34., 35., 36., 37.].contains(&self.extraction[0])
            && (0. ..=1.).contains(&self.waterworks_recovery[1])
            && (0. ..=1.).contains(&self.fishery_choice[3])
            && (0. ..=1.).contains(&self.fishery_choice[2])
            && (0. ..=1.).contains(&self.fishery_traps[3])
            && (0. ..=1.).contains(&self.fishery[3])
            && (0. ..=1.).contains(&self.fishery_stats[3])
            && (0. ..=0.25).contains(&self.fishery_config[0])
            && (0. ..=1.).contains(&self.tool_craft[0])
            && self
                .tool_craft
                .iter()
                .chain(&self.tool_work)
                .chain(&self.tool_orders)
                .all(|v| *v >= 0.)
            && self.residue.iter().all(|v| *v >= 0.)
            && (self.residue[0] - self.residue[1]).abs() <= 1e-5 * self.residue[1].max(1.)
            && self.residue[0] <= self.residue[2] + 0.001
            && (0. ..=1.).contains(&self.waterworks_plan[3])
            && (0. ..=1.).contains(&self.water_service[1])
            && self.workshop_types.iter().map(|t| t[0]).sum::<f32>()
                <= (self.workshop[0] / 20.)
                    .min(self.workshop[1] / 30.)
                    .min(self.workshop[2] / 2.)
                    + 0.001
            && self
                .goods
                .iter()
                .chain(&self.targets)
                .chain(&self.orders)
                .chain(&self.logistics)
                .chain(&self.storage)
                .chain(&self.storage_plan)
                .chain(&self.housing)
                .chain(&self.housing_plan)
                .chain(&self.waterworks)
                .chain(&self.waterworks_plan)
                .chain(&self.water_service)
                .chain(&self.waterworks_recovery)
                .chain(&self.return_flow)
                .chain(&self.land_return)
                .chain(&self.labor)
                .chain(&self.fishery)
                .chain(&self.fishery_plan)
                .chain(&self.fishery_config)
                .chain(&self.fishery_stats)
                .chain(&self.fishery_traps)
                .chain(&self.fishery_choice)
                .chain(&self.workshop)
                .chain(&self.workshop_plan)
                .chain(self.workshop_types.iter().flatten())
                .chain(&self.enterprise_lease)
                .chain(&self.enterprise_plan)
                .chain(&self.enterprise_used)
                .chain(&self.made)
                .chain(&self.used)
                .chain(&self.initial)
                .chain(&self.prices)
                .chain(&self.soil)
                .chain(&self.detritus)
                .chain(&self.forest)
                .chain(&self.reserves)
                .chain(&self.water)
                .chain(&self.finance)
                .chain(&self.baseline)
                .all(|v| *v >= 0.)
            && self
                .crops
                .iter()
                .flatten()
                .chain(self.herds.iter().flatten())
                .chain(&self.agriculture)
                .all(|v| *v >= 0.)
            && self.policy.iter().all(|v| (0. ..=1.).contains(v))
            && self.claim[1] > 0.
            && self.claim[2] >= self.claim[1]
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cargo {
    #[serde(default)]
    pub voyage_clock: Option<crate::vessels::VoyageClock>,
    /// Distinct inland service sites reserved at dispatch, including intermediate towns.
    /// Empty in older archives: retain the original endpoint/port footprint.
    #[serde(default)]
    pub freight_stops: Vec<u32>,
    pub from: u32,
    pub to: u32,
    pub good: u32,
    pub kg: f32,
    pub paid: f32,
    pub arrives: u32,
    #[serde(default)]
    pub sea_lane: Option<u32>,
    #[serde(default)]
    pub weather_delay_months: u32,
}
/// Partial price adjustment: inventory pressure plus separately observed costs/prices.
/// Rate limits bound adjustment speed, not a multiple of the catalog's base price.
fn adaptive_quote(
    previous: f32,
    stock: f32,
    target: f32,
    cost: Option<f32>,
    delivered: Option<f32>,
    purchasing_capacity: f32,
    reference: f32,
) -> f32 {
    let scarcity = if target > 0. {
        ((target - stock) / (target + stock).max(1.)).clamp(-1., 1.)
    } else if stock > 0. {
        -1.
    } else {
        0.
    };
    let shortage = (target - stock).max(0.);
    let funded = if shortage > 0. {
        (purchasing_capacity / (shortage * previous).max(0.0001)).clamp(0., 1.)
    } else {
        1.
    };
    let demand =
        scarcity.min(0.) + scarcity.max(0.) * funded - f32::from(shortage > 0.) * (1. - funded);
    // Scarcity shifts a supported quote; it is not a fresh inflation rate every month.
    // Catalog reference is a nominal anchor where no independent production cost exists.
    let anchor = cost
        .filter(|v| v.is_finite() && *v > 0.)
        .unwrap_or(reference)
        .max(0.0001);
    let desired = anchor * (0.8 * demand).exp();
    let evidence = delivered
        .filter(|v| v.is_finite() && *v > 0.)
        .map_or(desired.ln(), |v| 0.7 * desired.ln() + 0.3 * v.ln());
    let adjustment = (0.2 * (evidence - previous.max(0.0001).ln())).clamp(-0.15, 0.15);
    (previous * adjustment.exp()).clamp(0.0001, 1e6)
}
impl History {
    pub fn economy_residuals(&self) -> [f64; 6] {
        let mut baseline = self.nutrition_initial;
        let mut external = [0.; 3];
        let mut held = [0.; 3];
        let mut money = self
            .export_contracts
            .iter()
            .map(|c| c.escrow as f64)
            .sum::<f64>();
        let mut initial_money = 0.;
        let mut water = [0.; 4];
        let catalog = self.economy_catalog.as_ref().unwrap();
        let mut goods = [0.; GOODS];
        let mut goods_scale = [1.; GOODS];
        for s in &self.sites {
            let e = &s.economy;
            for k in 0..3 {
                baseline[k] += e.baseline[k] as f64;
                external[k] += e.external[k] as f64;
                held[k] += (e.soil[k] + e.detritus[k] + e.forest[k]) as f64
                    + s.stocks.stock[1] as f64 * FOOD_CNP[k];
            }
            for (j, good) in [0, 5].into_iter().enumerate() {
                goods[good] -= e.waterworks[j] as f64;
                for (k, held) in held.iter_mut().enumerate() {
                    *held += e.waterworks[j] as f64 * catalog.composition(good)[k] as f64;
                }
            }
            for (j, good) in [0, 5].into_iter().enumerate() {
                goods[good] -= e.housing[j] as f64;
                for (k, held) in held.iter_mut().enumerate() {
                    *held += e.housing[j] as f64 * catalog.composition(good)[k] as f64;
                }
            }
            for (j, good) in [0, 5].into_iter().enumerate() {
                goods[good] -= e.storage[j] as f64;
                for (k, held) in held.iter_mut().enumerate() {
                    *held += e.storage[j] as f64 * catalog.composition(good)[k] as f64;
                }
            }
            goods[0] -= e.fishery_traps[0] as f64;
            for (k, held) in held.iter_mut().enumerate() {
                *held += e.fishery_traps[0] as f64 * catalog.composition(0)[k] as f64;
            }
            for (j, good) in [0, 3, 16].into_iter().enumerate() {
                goods[good] -= e.fishery[j] as f64;
                for (k, held) in held.iter_mut().enumerate() {
                    *held += e.fishery[j] as f64 * catalog.composition(good)[k] as f64;
                }
            }
            for (j, good) in [0, 5, 3].into_iter().enumerate() {
                goods[good] -= e.workshop[j] as f64;
                for (k, held) in held.iter_mut().enumerate() {
                    *held += e.workshop[j] as f64 * catalog.composition(good)[k] as f64;
                }
            }
            for good in 0..GOODS {
                for (k, held) in held.iter_mut().enumerate() {
                    *held += e.goods[good] as f64 * catalog.composition(good)[k] as f64;
                }
            }
            for (k, held) in held.iter_mut().enumerate() {
                *held += e
                    .crops
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        (c[1] + c[2]) as f64
                            * catalog.composition(catalog.agriculture.as_ref().map_or(8 + i, |a| {
                                catalog.index(&a.crops[i].good).expect("validated crop")
                            }))[k] as f64
                    })
                    .sum::<f64>();
                *held += e.herds.iter().map(|a| a[0] as f64).sum::<f64>() * [0.25, 0.04, 0.003][k];
            }
            held[2] += e.reserves[0] as f64;
            money += e.finance[0] as f64;
            initial_money += e.finance[1] as f64;
            for (k, v) in water.iter_mut().enumerate() {
                *v += e.water[k] as f64;
            }
            for k in 0..GOODS {
                goods[k] +=
                    e.initial[k] as f64 + e.made[k] as f64 - e.used[k] as f64 - e.goods[k] as f64;
                goods_scale[k] += e.initial[k] as f64 + e.made[k] as f64;
            }
        }
        money += self
            .enterprises
            .as_ref()
            .map_or(0., |e| e.firms.iter().map(|f| f.cash).sum::<f64>());
        let restricted_tools: f64 = self.experimental_tool_reserves.values().sum();
        goods[3] -= restricted_tools;
        for (k, held) in held.iter_mut().enumerate() {
            *held += restricted_tools * catalog.composition(3)[k] as f64;
        }
        if let Some(c) = &self.culture {
            money += c.institutions.iter().map(|n| n.treasury).sum::<f64>();
            let owned = c.held_goods();
            for k in 0..GOODS {
                goods[k] -= owned[k];
            }
            for (good, owned) in owned.iter().enumerate() {
                for (k, held) in held.iter_mut().enumerate() {
                    *held += owned * catalog.composition(good)[k] as f64;
                }
            }
        }
        if let Some(x) = &self.expeditions {
            if let Some(d) = &x.discoveries {
                for (k, v) in d.held_cnp(x).into_iter().enumerate() {
                    held[k] += v;
                }
            }
            for e in &x.voyages {
                goods[0] -= e.timber as f64;
                goods[3] -= e.tools as f64;
                money += e.purse;
                for (k, f) in [0.5, 0.002, 0.0002].into_iter().enumerate() {
                    held[k] += e.timber as f64 * f + e.food as f64 * FOOD_CNP[k];
                }
            }
        }
        if let Some(shipping) = &self.shipping {
            for p in &shipping.ports {
                for (k, good) in crate::shipping::MATERIALS.into_iter().enumerate() {
                    goods[good] -= p.assets[k] as f64;
                }
                for (k, f) in [0.5, 0.002, 0.0002].into_iter().enumerate() {
                    held[k] += p.assets[0] as f64 * f;
                }
            }
        }
        if let Some(living) = &self.living {
            goods[5] -= living
                .floods
                .values()
                .map(|f| f.granary_bricks)
                .sum::<f64>();
        }
        if let Some(society) = &self.society {
            money += society
                .household_economy
                .as_ref()
                .map_or(0., |e| e.accounts.iter().map(|a| a.cash).sum::<f64>());
            money += society.councils.iter().map(|c| c.treasury).sum::<f64>();
            goods[5] -= society.routes.iter().map(|r| r.road_bricks).sum::<f64>();
            for (k, ratio) in catalog.composition(5).iter().enumerate() {
                held[k] +=
                    society.routes.iter().map(|r| r.road_bricks).sum::<f64>() * *ratio as f64;
            }
            for j in &society.relocation.journeys {
                money += j.cash as f64;
                goods[3] -= j.tools as f64;
                for k in 0..3 {
                    held[k] += j.food as f64 * FOOD_CNP[k]
                        + j.tools as f64 * catalog.composition(3)[k] as f64;
                }
            }
            for r in &society.raids {
                goods[3] -= r.equipment as f64;
                for k in 0..3 {
                    held[k] += r.food as f64 * FOOD_CNP[k];
                }
            }
        }
        for s in &self.sites {
            for k in 0..3 {
                held[k] +=
                    (s.demography.crops[0] as f64 + s.demography.crops[1] as f64) * FOOD_CNP[k];
            }
        }
        for s in &self.shipments {
            for k in 0..3 {
                held[k] += s.food_kg as f64 * FOOD_CNP[k];
            }
        }
        for c in &self.cargo {
            if c.good == FOOD as u32 {
                for k in 0..3 {
                    held[k] += c.kg as f64 * FOOD_CNP[k];
                }
            } else {
                goods[c.good as usize] -= c.kg as f64;
                for (k, held) in held.iter_mut().enumerate() {
                    *held += c.kg as f64 * catalog.composition(c.good as usize)[k] as f64;
                }
            }
        }
        [
            (baseline[0] + external[0] - held[0]) / baseline[0].max(1.),
            (baseline[1] + external[1] - held[1]) / baseline[1].max(1.),
            (baseline[2] + external[2] - held[2]) / baseline[2].max(1.),
            (water[1] + water[2] - water[3] - water[0]) / (water[1] + water[2]).max(1.),
            (initial_money - money) / initial_money.max(1.),
            (0..GOODS)
                .map(|k| (goods[k] / goods_scale[k]).abs())
                .fold(0., f64::max),
        ]
    }
    /// Cargo lost away from settlements leaves the managed domain. Payment was
    /// already transferred at dispatch: the buyer bears the loss, without minted refunds.
    pub(crate) fn lose_cargo(&mut self, from: u32, good: u32, kg: f32) {
        let site = &mut self.sites[from as usize];
        let ratios = if good == FOOD as u32 {
            site.stocks.ledger[2] += kg;
            FOOD_CNP.map(|v| v as f32)
        } else {
            site.economy.used[good as usize] += kg;
            site.economy.reserves[3] += kg;
            self.economy_catalog
                .as_ref()
                .unwrap()
                .composition(good as usize)
        };
        for (k, ratio) in ratios.into_iter().enumerate() {
            site.economy.external[k] -= kg * ratio;
        }
    }

    /// Origin, destination and sea-port approach services. Repeated sites reserve once.
    pub(crate) fn freight_sites(&self, from: u32, to: u32, lane: Option<u32>) -> Option<[u32; 4]> {
        let mut sites = [from, to, from, to];
        if let Some(lane) = lane {
            let shipping = self.shipping.as_ref()?;
            let lane = shipping.lanes.get(lane as usize)?;
            sites[2] = shipping.ports.get(lane.ports[0] as usize)?.site;
            sites[3] = shipping.ports.get(lane.ports[1] as usize)?.site;
        }
        sites
            .iter()
            .all(|&s| (s as usize) < self.sites.len())
            .then_some(sites)
    }

    /// Free inland service capacity, kg in transit, across archived stop footprints.
    /// Legacy towns retain unlimited capacity. Existing cargo survives capacity decline.
    pub fn land_freight_capacity(&self, site: u32) -> f32 {
        let Some(s) = self.sites.get(site as usize) else {
            return 0.;
        };
        if s.economy.logistics[3] <= 0.5 {
            return f32::INFINITY;
        }
        let Some(catalog) = &self.economy_catalog else {
            return 0.;
        };
        let used: f32 = self
            .cargo
            .iter()
            .filter(|c| {
                if c.freight_stops.is_empty() {
                    self.freight_sites(c.from, c.to, c.sea_lane)
                        .is_some_and(|sites| sites.contains(&site))
                } else {
                    c.freight_stops.contains(&site)
                }
            })
            .map(|c| c.kg)
            .sum();
        let relief: f32 = self
            .shipments
            .iter()
            .filter(|c| c.from == site || c.to == site)
            .map(|c| c.food_kg)
            .sum();
        (s.stocks.stock[0] * catalog.production.land_freight_kg_per_person - used - relief).max(0.)
    }

    #[cfg(test)]
    pub(crate) fn market_month(&mut self, radius: f32) {
        let observed = self.market_arrivals();
        self.market_decisions(radius, &observed);
    }

    pub(crate) fn market_arrivals(&mut self) -> Vec<[[f64; 2]; GOODS]> {
        self.advance_cargo_voyages();
        self.expire_export_contracts();
        let mut observed = vec![[[0_f64; 2]; GOODS]; self.sites.len()];
        let arrivals = std::mem::take(&mut self.cargo);
        for mut c in arrivals {
            if c.arrives <= self.month && self.flood_blocks_delivery(c.from, c.to, c.sea_lane) {
                if c.weather_delay_months == 0 {
                    self.event("cargo_weather_delay", Some(c.to), Some(c.from), format!("{:.1} kg cargo held by flooded transport access; goods remain in transit; purchase was paid at dispatch", c.kg));
                }
                c.weather_delay_months = c.weather_delay_months.saturating_add(1);
                let rate = self
                    .economy_catalog
                    .as_ref()
                    .map_or(if c.good == FOOD as u32 { 0.2 } else { 0. }, |catalog| {
                        catalog.delay_spoilage(c.good as usize)
                    });
                if rate > 0. && c.kg > 0. {
                    let remaining = (c.kg - c.kg * rate).max(0.);
                    let lost = c.kg - remaining;
                    c.kg = remaining;
                    self.lose_cargo(c.from, c.good, lost);
                    let name = self
                        .economy_catalog
                        .as_ref()
                        .and_then(|catalog| catalog.goods.get(c.good as usize))
                        .map_or("food", |good| good.name.as_str())
                        .to_owned();
                    self.event("cargo_spoilage", Some(c.to), Some(c.from),
                        format!("{lost:.1} kg {name} spoiled while transport was blocked ({:.0}% monthly loss); nutrients exported outside managed plots", rate * 100.));
                }
                if c.kg <= 0. {
                    self.event("cargo_spoilage_lost", Some(c.to), Some(c.from),
                        format!("All cargo spoiled after {} delay months; buyer bears {:.1} already-paid cost", c.weather_delay_months, c.paid));
                    continue;
                }
                if c.weather_delay_months >= 6 {
                    self.lose_cargo(c.from, c.good, c.kg);
                    self.event("cargo_weather_lost", Some(c.to), Some(c.from),
                        format!("Blocked journey terminated after {} delay months; {:.1} kg remaining cargo written off; buyer bears {:.1} already-paid cost", c.weather_delay_months, c.kg, c.paid));
                    continue;
                }
                c.arrives = self.month + 1;
                self.cargo.push(c);
                continue;
            }
            if c.arrives <= self.month {
                self.observe_export_delivery(&c);
                self.observe_lexical_trade(c.from, c.to, c.kg);
                self.trade_contact
                    .observe(self.month, c.from, c.to, c.kg as f64);
                if c.weather_delay_months > 0 {
                    self.event(
                        "cargo_weather_recovered",
                        Some(c.to),
                        Some(c.from),
                        format!(
                            "{:.1} kg cargo delivered after {} weather-delay months",
                            c.kg, c.weather_delay_months
                        ),
                    );
                }
                if let Some(lane) = c.sea_lane {
                    self.event(
                        "sea_arrival",
                        Some(c.to),
                        Some(c.from),
                        format!("Delivered {:.1} kg over sea lane {lane}", c.kg),
                    );
                }
                if c.good == FOOD as u32 {
                    self.sites[c.to as usize].stocks.stock[1] += c.kg;
                } else {
                    self.sites[c.to as usize].economy.goods[c.good as usize] += c.kg;
                }
                if c.kg > 0. && c.paid > 0. && c.weather_delay_months == 0 {
                    observed[c.to as usize][c.good as usize][0] += c.paid as f64;
                    observed[c.to as usize][c.good as usize][1] += c.kg as f64;
                }
                self.event(
                    "market_arrival",
                    Some(c.to),
                    Some(c.from),
                    format!(
                        "Received {:.1} kg {}",
                        c.kg,
                        if c.good == FOOD as u32 {
                            "food"
                        } else {
                            self.economy_catalog.as_ref().unwrap().goods[c.good as usize]
                                .id
                                .as_str()
                        }
                    ),
                );
            } else {
                self.cargo.push(c);
            }
        }
        observed
    }

    pub(crate) fn market_decisions(&mut self, radius: f32, observed: &[[[f64; 2]; GOODS]]) {
        let Some(catalog) = self.economy_catalog.clone() else {
            return;
        };
        // Cost quotes all read last month's price snapshot, avoiding settlement/good order feedback.
        let costs: Vec<Vec<_>> = if catalog.market.adaptive_prices {
            (0..self.sites.len())
                .map(|site| {
                    (0..catalog.goods.len())
                        .map(|good| self.supplier_unit_cost(site, good))
                        .collect()
                })
                .collect()
        } else {
            vec![]
        };
        let mut purchasing: Vec<f32> = self.sites.iter().map(|s| s.economy.finance[0]).collect();
        if let Some(society) = &self.society {
            if let Some(wallets) = &society.household_economy {
                for hh in &society.households {
                    if !society.relocation.away(hh.id) {
                        purchasing[hh.site as usize] += wallets
                            .accounts
                            .get(hh.id as usize)
                            .map_or(0., |a| a.cash as f32);
                    }
                }
            }
        }
        let quote_targets: Vec<Vec<f32>> = self
            .sites
            .iter()
            .map(|s| {
                catalog
                    .goods
                    .iter()
                    .enumerate()
                    .map(|(k, _)| {
                        if s.economy.logistics[3] > 0.5 {
                            if k == FOOD {
                                s.stocks.stock[0] * 18. * 6.
                            } else {
                                s.economy.targets[k]
                            }
                        } else {
                            s.stocks.stock[0] * if k == 3 { 0.5 } else { 2. }
                        }
                    })
                    .collect()
            })
            .collect();
        let quote_orders: Vec<Vec<f32>> = self
            .sites
            .iter()
            .enumerate()
            .map(|(i, s)| {
                quote_targets[i]
                    .iter()
                    .enumerate()
                    .map(|(k, target)| {
                        if catalog.goods[k].id.starts_with("reserved_") {
                            return 0.;
                        }
                        let stock = if k == FOOD {
                            s.stocks.stock[1]
                        } else {
                            s.economy.goods[k]
                        };
                        (target - stock).max(0.) * s.economy.prices[k].max(0.0001)
                    })
                    .collect()
            })
            .collect();
        // Prices are local offers, not equilibrium solutions or money transfers.
        for s in &mut self.sites {
            if s.abandoned || s.stocks.stock[0] <= 0. {
                continue;
            }
            let order_total: f32 = quote_orders[s.id as usize].iter().sum();
            for (k, good) in catalog.goods.iter().enumerate() {
                let target = quote_targets[s.id as usize][k];
                if catalog.market.adaptive_prices {
                    let stock = if k == FOOD {
                        s.stocks.stock[1]
                    } else {
                        s.economy.goods[k]
                    };
                    let trade = observed[s.id as usize][k];
                    s.economy.prices[k] = adaptive_quote(
                        s.economy.prices[k].max(0.0001),
                        stock,
                        target,
                        costs[s.id as usize][k],
                        (trade[1] > 0.).then(|| (trade[0] / trade[1]) as f32),
                        purchasing[s.id as usize] * quote_orders[s.id as usize][k]
                            / order_total.max(0.0001),
                        good.base_price,
                    );
                    continue;
                }
                s.economy.prices[k] = good.base_price
                    * (target
                        / ((if k == FOOD {
                            s.stocks.stock[1]
                        } else {
                            s.economy.goods[k]
                        }) + target * 0.25)
                            .max(1.))
                    .clamp(0.4, 4.);
            }
        }
        if self.month % 3 != 0 {
            return;
        }
        // Rebuild once per quarterly market: roads, closures and war can change accessibility.
        let network = if catalog.market.network_trade || self.shipping.is_some() {
            self.trade_distances()
        } else {
            None
        };
        let seas = network.as_ref().map(|roads| self.sea_quotes(roads));
        let site_count = self.sites.len();
        let freight = self.trade_freight_stops(seas.as_deref(), network.is_some());
        let sea = move |a: usize, b: usize| seas.as_ref().and_then(|q| q[a * site_count + b]);
        let route = |h: &History, a: usize, b: usize| -> Option<f32> {
            if h.sites[a].island != h.sites[b].island {
                return sea(a, b)
                    .filter(|(_, lane)| h.sea_capacity(*lane) >= 1.)
                    .map(|(d, _)| d);
            }
            if let Some(distances) = &network {
                let d = distances[a * h.sites.len() + b];
                d.is_finite().then_some(d)
            } else if h.society.is_some() {
                h.route_cost(a as u32, b as u32)
            } else {
                Some(
                    crate::civilization::distance(
                        h.sites[a].cell,
                        h.sites[b].cell,
                        h.terrain_resolution,
                    ) * radius,
                )
            }
        };
        let reachable: Vec<bool> = self
            .export_contracts
            .iter()
            .map(|c| {
                route(self, c.seller as usize, c.buyer as usize)
                    .is_some_and(|d| d < catalog.market.max_distance_km)
            })
            .collect();
        self.fund_export_contracts(&reachable);
        let mut land_remaining: Vec<f32> = (0..self.sites.len())
            .map(|i| self.land_freight_capacity(i as u32))
            .collect();
        for buyer in 0..self.sites.len() {
            if self.sites[buyer].abandoned || self.sites[buyer].economy.policy[3] < 0.5 {
                continue;
            }
            // Preserve archived ordering for legacy economies; planned towns buy food first.
            let mut goods: Vec<_> = catalog
                .goods
                .iter()
                .enumerate()
                .filter(|(_, g)| !g.id.starts_with("reserved_") && g.id != "food")
                .map(|(i, g)| (i, g.id.as_str()))
                .collect();
            if self.sites[buyer].economy.logistics[3] > 0.5 {
                goods.insert(0, (FOOD, "food"));
            } else {
                goods.push((FOOD, "food"));
            }
            for (k, good_id) in goods {
                let pop = self.sites[buyer].stocks.stock[0];
                let target = if k == FOOD {
                    pop * 18. * 6.
                } else if self.sites[buyer].economy.logistics[3] > 0.5 {
                    self.local_production_target(buyer, k)
                } else {
                    pop * if k == 3 { 0.5 } else { 2. }
                };
                let stock = if k == FOOD {
                    self.sites[buyer].stocks.stock[1]
                } else {
                    self.sites[buyer].economy.goods[k]
                };
                let transit: f32 = self
                    .cargo
                    .iter()
                    .filter(|c| c.to as usize == buyer && c.good as usize == k)
                    .map(|c| c.kg)
                    .sum();
                let need = (target
                    - stock
                    - if k != FOOD && self.sites[buyer].economy.logistics[3] > 0.5 {
                        0.
                    } else {
                        transit
                    })
                .max(0.);
                if need < 1. {
                    continue;
                }
                let contract = self.export_contracts.iter().position(|c| {
                    c.buyer as usize == buyer
                        && c.good as usize == k
                        && c.escrow > 0.
                        && c.remaining_kg >= 0.001
                });
                let seller = (0..self.sites.len())
                    .filter(|&j| {
                        j != buyer
                            && !self.sites[j].abandoned
                            && self.sites[j].economy.policy[3] >= 0.5
                            && route(self, j, buyer).is_some_and(|d| {
                                let bounded_distance =
                                    if catalog.market.network_trade || self.shipping.is_some() {
                                        d
                                    } else {
                                        crate::civilization::distance(
                                            self.sites[j].cell,
                                            self.sites[buyer].cell,
                                            self.terrain_resolution,
                                        ) * radius
                                    };
                                bounded_distance < catalog.market.max_distance_km
                            })
                    })
                    .filter(|&j| {
                        // A cheap supplier with committed carriers cannot block another offer.
                        freight[j * site_count + buyer]
                            .as_ref()
                            .is_some_and(|sites| {
                                sites
                                    .iter()
                                    .all(|&site| land_remaining[site as usize] >= 1.)
                            })
                    })
                    .filter(|&j| {
                        let s = &self.sites[j];
                        if k == FOOD {
                            s.stocks.stock[1]
                                > s.stocks.stock[0] * 18. * catalog.market.food_reserve_months
                        } else {
                            s.economy.goods[k]
                                > if s.economy.logistics[3] > 0.5 {
                                    self.local_production_target(j, k) * 0.75
                                } else {
                                    s.stocks.stock[0] * catalog.market.reserve_per_person[k]
                                }
                        }
                    })
                    .min_by(|&a, &b| {
                        let quote = |j: usize| {
                            let price = if k == FOOD && self.sites[j].economy.logistics[3] < 0.5 {
                                1.
                            } else {
                                self.sites[j].economy.prices[k]
                            };
                            // Buyers discount distant supply; payment remains the seller's quote.
                            price
                                * if catalog.market.network_trade {
                                    1. + route(self, j, buyer).unwrap_or(f32::INFINITY) / 1500.
                                } else {
                                    1.
                                }
                        };
                        let preferred = |j: usize| {
                            contract.is_some_and(|i| self.export_contracts[i].seller as usize == j)
                        };
                        preferred(b)
                            .cmp(&preferred(a))
                            .then_with(|| quote(a).total_cmp(&quote(b)))
                            .then(a.cmp(&b))
                    });
                if let Some(seller) = seller {
                    let distance = route(self, seller, buyer).expect("selected reachable seller");
                    let contract =
                        contract.filter(|i| self.export_contracts[*i].seller as usize == seller);
                    let escrow = contract.map_or(0., |i| self.export_contracts[i].escrow);
                    let price = if let Some(i) = contract {
                        self.export_contracts[i].unit_price
                    } else if k == FOOD && self.sites[seller].economy.logistics[3] < 0.5 {
                        1.
                    } else {
                        self.sites[seller].economy.prices[k]
                    };
                    let s = &self.sites[seller];
                    let surplus = if k == FOOD {
                        s.stocks.stock[1]
                            - s.stocks.stock[0] * 18. * catalog.market.food_reserve_months
                    } else {
                        s.economy.goods[k]
                            - if s.economy.logistics[3] > 0.5 {
                                self.local_production_target(seller, k) * 0.75
                            } else {
                                s.stocks.stock[0] * catalog.market.reserve_per_person[k]
                            }
                    };
                    let sea_lane = sea(seller, buyer).map(|(_, lane)| lane);
                    let freight_sites = freight[seller * site_count + buyer]
                        .as_ref()
                        .expect("selected valid freight services");
                    let capacity = freight_sites
                        .iter()
                        .map(|&site| land_remaining[site as usize])
                        .fold(f32::INFINITY, f32::min)
                        .min(sea_lane.map_or(f32::INFINITY, |lane| self.sea_capacity(lane)));
                    let e = &self.sites[buyer].economy;
                    let space = if e.logistics[3] > 0.5
                        && k != FOOD
                        && catalog.goods[k].food_energy <= 0.
                    {
                        let stored: f32 = e
                            .goods
                            .iter()
                            .enumerate()
                            .filter(|(k, _)| *k != FOOD && catalog.goods[*k].food_energy <= 0.)
                            .map(|(_, v)| v)
                            .sum();
                        let incoming: f32 = self
                            .cargo
                            .iter()
                            .filter(|c| {
                                c.to as usize == buyer
                                    && c.good as usize != FOOD
                                    && catalog.goods[c.good as usize].food_energy <= 0.
                            })
                            .map(|c| c.kg)
                            .sum();
                        (e.logistics[0] - stored - incoming).max(0.)
                    } else {
                        f32::INFINITY
                    };
                    let amount = need
                        .min(space)
                        .min(capacity)
                        .min(surplus)
                        .min(pop * 5.)
                        .min((self.sites[buyer].economy.finance[0] + escrow) / price);
                    if amount < 1. {
                        continue;
                    }
                    for (i, &site) in freight_sites.iter().enumerate() {
                        if !freight_sites[..i].contains(&site) {
                            land_remaining[site as usize] -= amount;
                        }
                    }
                    let cost = amount * price;
                    let reserved_payment = if let Some(i) = contract {
                        let c = &mut self.export_contracts[i];
                        let quantity = amount.min(c.remaining_kg);
                        let paid = (quantity * price).min(c.escrow);
                        c.escrow = (c.escrow - paid).max(0.);
                        c.remaining_kg = (c.remaining_kg - quantity).max(0.);
                        c.dispatched_kg += quantity;
                        if c.estimated_unit_cost > 0. {
                            c.quoted_surplus += quantity * (price - c.estimated_unit_cost).max(0.);
                        }
                        paid
                    } else {
                        0.
                    };
                    self.sites[buyer].economy.finance[0] =
                        (self.sites[buyer].economy.finance[0] - (cost - reserved_payment)).max(0.);
                    self.sites[buyer].economy.finance[3] += cost;
                    self.sites[seller].economy.finance[0] += cost;
                    self.sites[seller].economy.finance[2] += cost;
                    if k == FOOD {
                        self.sites[seller].stocks.stock[1] -= amount;
                    } else {
                        self.sites[seller].economy.goods[k] -= amount;
                    }
                    let arrives = self.month + (distance / 150.).ceil().max(1.) as u32;
                    self.cargo.push(Cargo {
                        voyage_clock: sea_lane.map(|_| crate::vessels::VoyageClock {
                            month: self.month,
                            remaining: (arrives - self.month) as f32,
                        }),
                        freight_stops: freight_sites.clone(),
                        sea_lane,
                        weather_delay_months: 0,
                        from: seller as u32,
                        to: buyer as u32,
                        good: k as u32,
                        kg: amount,
                        paid: cost,
                        arrives,
                    });
                    self.event(
                        "market_sale",
                        Some(seller as u32),
                        Some(buyer as u32),
                        format!(
                            "Sold {:.1} kg {} for {:.1}; arrival month {}",
                            amount, good_id, cost, arrives
                        ),
                    );
                }
            }
        }
    }
}

/// Checkpoint/catalog v1 arrays are padded without inventing stocks.
pub mod slots {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(v: &[f32; super::GOODS], s: S) -> Result<S::Ok, S::Error> {
        v.as_slice().serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[f32; super::GOODS], D::Error> {
        let v = Vec::<f32>::deserialize(d)?;
        if v.len() > super::GOODS {
            return Err(serde::de::Error::custom("too many material slots"));
        }
        let mut out = [0.; super::GOODS];
        out[..v.len()].copy_from_slice(&v);
        Ok(out)
    }
}

#[cfg(test)]
mod freight_tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn in_transit_land_freight_reserves_capacity_and_buyers_find_another_supplier() {
        let mut g = crate::gpu::Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        let h = g.civilizations.as_mut().unwrap();
        // Isolate market allocation from production and geography in this consumer fixture.
        h.society = None;
        h.shipping = None;
        h.cargo.clear();
        let catalog = h.economy_catalog.as_mut().unwrap();
        catalog.market.network_trade = false;
        catalog.market.max_distance_km = 100_000.;
        catalog.production.land_freight_kg_per_person = 1.;
        for s in &mut h.sites {
            s.island = 0;
            s.stocks.stock[0] = 10.;
            s.stocks.stock[1] = 10000.;
            s.economy.goods.fill(0.);
            s.economy.targets.fill(0.);
            s.economy.targets[3] = 20.;
            s.economy.logistics = [1000., 0., 0., 1.];
            s.economy.finance[0] = 10000.;
            s.economy.policy[3] = if s.id < 3 { 1. } else { 0. };
        }
        h.sites[1].economy.goods[3] = 200.;
        h.sites[2].economy.goods[3] = 40.;
        // Cheaper supplier's carriers are already away with goods removed at dispatch.
        h.sites[1].economy.goods[3] -= 10.;
        h.cargo.push(Cargo {
            voyage_clock: None,
            freight_stops: vec![],
            from: 1,
            to: 3,
            good: 3,
            kg: 10.,
            paid: 0.,
            arrives: 12,
            sea_lane: None,
            weather_delay_months: 0,
        });
        let totals = |h: &History| {
            (
                h.sites.iter().map(|s| s.economy.goods[3]).sum::<f32>()
                    + h.cargo
                        .iter()
                        .filter(|c| c.good == 3)
                        .map(|c| c.kg)
                        .sum::<f32>(),
                h.sites
                    .iter()
                    .map(|s| s.economy.finance[0] as f64)
                    .sum::<f64>(),
            )
        };
        let before = totals(h);
        assert_eq!(h.land_freight_capacity(1), 0.);
        assert_eq!(h.land_freight_capacity(0), 10.);
        assert_eq!(h.land_freight_capacity(u32::MAX), 0.);
        let mut free = h.clone();
        // Delivered alternative: same goods, but no continuing transport reservation.
        let c = free.cargo.remove(0);
        free.sites[c.to as usize].economy.goods[3] += c.kg;
        h.month = 3;
        free.month = 3;
        h.market_month(6371.);
        free.market_month(6371.);
        assert!(h.sites[1].economy.prices[3] < h.sites[2].economy.prices[3]);
        assert!(h
            .cargo
            .iter()
            .any(|c| c.from == 2 && c.to == 0 && c.kg == 10.));
        assert!(free.cargo.iter().any(|c| c.from == 1 && c.to == 0));
        assert_eq!(h.land_freight_capacity(0), 0.);
        // Keep the controlled journeys away through two market quarters.
        for c in &mut h.cargo {
            c.arrives = 12;
        }
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        for month in [6, 9] {
            h.month = month;
            h.market_month(6371.);
            resumed.month = month;
            resumed.market_month(6371.);
            assert_eq!(h.cargo.len(), 2);
            assert_eq!(h.land_freight_capacity(0), 0.);
        }
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        h.sites[0].stocks.stock[0] = 5.;
        assert_eq!(
            h.land_freight_capacity(0),
            0.,
            "capacity decline must not delete cargo"
        );
        for s in &mut h.sites {
            s.economy.policy[3] = 0.;
        }
        h.month = 12;
        h.market_month(6371.);
        assert!(h.cargo.is_empty());
        assert_eq!(h.land_freight_capacity(0), 5.);
        assert_eq!(totals(h).0, before.0);
        assert!((totals(h).1 - before.1).abs() < 0.01);
        h.sites[0].economy.logistics[3] = 0.;
        assert!(h.land_freight_capacity(0).is_infinite());
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn sea_trade_shares_freight_with_inland_approaches_and_deduplicates_ports() {
        let mut g = crate::gpu::Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.cargo.clear();
        let c = h.economy_catalog.as_mut().unwrap();
        c.market.network_trade = true;
        c.market.max_distance_km = 100000.;
        c.production.land_freight_kg_per_person = 1.;
        for s in &mut h.sites {
            s.island = u32::from(matches!(s.id, 2 | 3));
            s.stocks.stock[0] = if s.id == 1 { 3. } else { 10. };
            s.stocks.stock[1] = 10000.;
            s.economy.goods.fill(0.);
            s.economy.targets.fill(0.);
            s.economy.logistics = [1000., 0., 0., 1.];
            s.economy.finance[0] = 10000.;
            s.economy.policy[3] = 1.;
        }
        h.sites[0].economy.goods[3] = 100.;
        h.sites[4].economy.goods[3] = 40.;
        h.sites[1].economy.goods[4] = 3.;
        h.sites[2].economy.targets[3] = 20.;
        h.society.as_mut().unwrap().routes = [(0, 1), (4, 1), (2, 3)]
            .into_iter()
            .enumerate()
            .map(|(id, (from, to))| crate::society::Route {
                id: id as u32,
                from,
                to,
                cells: vec![],
                cost_km: 10.,
                open: true,
                flood_months: 0,
                road_bricks: 0.,
                upkeep: None,
            })
            .collect();
        h.shipping = Some(crate::shipping::Shipping {
            version: 1,
            started: 0,
            surveyed_sites: 5,
            ports: [1, 3]
                .into_iter()
                .map(|site| crate::shipping::Port {
                    fleet: None,
                    work: None,
                    site,
                    access: vec![],
                    water_cell: 0,
                    access_km: 1.,
                    assets: crate::shipping::TARGET,
                    commissioned: Some(0),
                    flood_months: 0,
                })
                .collect(),
            lanes: vec![crate::shipping::SeaLane {
                ports: [0, 1],
                cells: vec![],
                km: 600.,
                open: true,
                flood_months: 0,
            }],
        });
        // Geometry is prescribed to isolate reservation arbitration; shipping integration tests survey actual paths.
        let initial = h.clone();
        let mut blocked = h.clone();
        blocked.sites[1].economy.goods[4] -= 3.;
        blocked.cargo.push(Cargo {
            voyage_clock: None,
            freight_stops: vec![],
            from: 1,
            to: 4,
            good: 4,
            kg: 3.,
            paid: 0.,
            arrives: 12,
            sea_lane: None,
            weather_delay_months: 0,
        });
        assert_eq!(blocked.sea_capacity(0), 1000.);
        assert_eq!(blocked.land_freight_capacity(1), 0.);
        assert!(blocked
            .sea_quotes(&blocked.trade_distances().unwrap())
            .iter()
            .all(Option::is_none));
        let totals = |h: &History| {
            (
                h.sites
                    .iter()
                    .flat_map(|s| s.economy.goods.iter().map(|v| *v as f64))
                    .sum::<f64>()
                    + h.cargo.iter().map(|c| c.kg as f64).sum::<f64>(),
                h.sites
                    .iter()
                    .map(|s| s.economy.finance[0] as f64)
                    .sum::<f64>(),
            )
        };
        let before = totals(h);
        h.month = 3;
        blocked.month = 3;
        h.market_month(6371.);
        blocked.market_month(6371.);
        assert!(blocked.cargo.iter().all(|c| c.sea_lane.is_none()));
        let sea: Vec<_> = h.cargo.iter().filter(|c| c.sea_lane.is_some()).collect();
        assert_eq!(sea.len(), 1);
        assert_eq!((sea[0].from, sea[0].to, sea[0].kg), (0, 2, 3.));
        assert_eq!(h.land_freight_capacity(1), 0.);
        assert_eq!(h.land_freight_capacity(0), 7.);
        assert_eq!(h.land_freight_capacity(2), 7.);
        assert_eq!(h.land_freight_capacity(3), 7.);
        assert_eq!(totals(h).0, before.0);
        assert!((totals(h).1 - before.1).abs() < 0.01);
        for c in &mut h.cargo {
            // Prescribe both the delivery date and remaining travel, then pay
            // attention to every monthly interval rather than skipping quarters.
            c.arrives = 12;
            c.voyage_clock = Some(crate::vessels::VoyageClock {
                month: h.month,
                remaining: (12 - h.month) as f32,
            });
        }
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        for month in 4..12 {
            h.month = month;
            resumed.month = month;
            h.market_month(6371.);
            resumed.market_month(6371.);
            assert_eq!(h.cargo.len(), 1);
            assert_eq!(h.cargo[0].arrives, 12);
            assert_eq!(
                h.cargo[0].voyage_clock.as_ref().unwrap().remaining,
                (12 - month) as f32
            );
            assert_eq!(h.land_freight_capacity(1), 0.);
        }
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        for s in &mut h.sites {
            s.economy.policy[3] = 0.;
        }
        h.month = 12;
        h.market_month(6371.);
        assert!(h.cargo.is_empty());
        assert_eq!(h.land_freight_capacity(1), 3.);
        assert_eq!(totals(h).0, before.0);
        // Direct port-to-port orders share three kg once per site, not twice per role.
        let mut direct = initial;
        for s in &mut direct.sites {
            s.economy.goods.fill(0.);
            s.economy.targets.fill(0.);
            s.economy.policy[3] = f32::from(matches!(s.id, 1 | 3));
        }
        for k in [3, 4] {
            direct.sites[1].economy.goods[k] = 100.;
            direct.sites[3].economy.targets[k] = 2.;
        }
        direct.month = 3;
        direct.market_month(6371.);
        assert_eq!(direct.cargo.len(), 2);
        assert_eq!(direct.cargo.iter().map(|c| c.kg).sum::<f32>(), 3.);
        assert!(direct
            .cargo
            .iter()
            .all(|c| c.from == 1 && c.to == 3 && c.sea_lane == Some(0)));
        assert_eq!(direct.land_freight_capacity(3), 7.);
    }
}

#[cfg(test)]
mod adaptive_price_tests {
    use super::*;
    #[test]
    fn quotes_respond_to_costs_and_paid_evidence_without_catalog_price_ceiling() {
        assert_eq!(
            adaptive_quote(10., 100., 100., Some(10.), Some(10.), 1e12, 10.),
            10.
        );
        let ordinary = adaptive_quote(10., 100., 100., Some(10.), None, 1e12, 10.);
        assert!(adaptive_quote(10., 100., 100., Some(20.), None, 1e12, 10.) > ordinary);
        assert!(adaptive_quote(10., 100., 100., None, Some(5.), 1e12, 10.) < ordinary);
        assert!(adaptive_quote(10., 200., 100., None, None, 1e12, 10.) < ordinary);
        assert!(adaptive_quote(10., 0., 100., Some(20.), None, 0., 10.) < 10.);
        let mut scarce = 10.;
        for _ in 0..24 {
            scarce = adaptive_quote(scarce, 0., 100., None, None, 1e12, 10.);
        }
        assert!(scarce > 10. && scarce < 23.);
        let settled = scarce;
        for _ in 0..1200 {
            scarce = adaptive_quote(scarce, 0., 100., None, None, 1e12, 10.);
        }
        assert!((scarce - settled).abs() < 0.2);
        // Recover a checkpoint containing the previous experiment's inflated quote.
        let mut inherited = 10_000.;
        for _ in 0..600 {
            inherited = adaptive_quote(inherited, 0., 100., None, None, 1e12, 10.);
        }
        assert!((inherited - scarce).abs() < 0.001);
        for previous in [0.0001, 1., 100., 1e6] {
            let q = adaptive_quote(previous, 0., 1e9, Some(1e6), Some(1e6), 1e12, 10.);
            assert!(q <= previous * 0.15_f32.exp() + 0.001);
        }
        let mut legacy = serde_json::to_value(EconomyCatalog::bundled().unwrap()).unwrap();
        legacy["market"]
            .as_object_mut()
            .unwrap()
            .remove("adaptive_prices");
        let legacy: EconomyCatalog = serde_json::from_value(legacy).unwrap();
        assert!(!legacy.market.adaptive_prices);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn local_recipe_input_prices_change_the_actual_tool_quote() {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 64,
                ecology_resolution: 16,
                ecology_years_per_epoch: 1,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.economy_catalog.as_mut().unwrap().market.adaptive_prices = true;
        h.month = 1;
        h.sites[0].economy.prices[3] = h.supplier_unit_cost(0, 3).unwrap();
        let mut expensive = h.clone();
        expensive.sites[0].economy.prices[2] *= 4.;
        let before = expensive.supplier_unit_cost(0, 3).unwrap();
        assert!(before > h.supplier_unit_cost(0, 3).unwrap());
        h.market_month(6371.);
        expensive.market_month(6371.);
        assert!(expensive.sites[0].economy.prices[3] > h.sites[0].economy.prices[3]);
        assert_eq!(
            h.sites[0].economy.finance,
            expensive.sites[0].economy.finance
        );
    }
}
