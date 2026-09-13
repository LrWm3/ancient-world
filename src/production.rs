//! Sparse town work orders; physical production and inventory transfers execute on GPU.
use crate::{
    civilization::History,
    economy::{EconomyCatalog, FOOD, GOODS},
};
use serde::{Deserialize, Serialize};

const FACILITY_REPAIR_FORECAST_WORKER_MONTHS: f32 = 0.1;
const DEFAULT_INITIAL_HOUSING_PER_PERSON: f32 = 1.1;
const DEFAULT_CONTRACT_MARGIN: f32 = 0.1;
const DEFAULT_STORAGE_KG_PER_PERSON: f32 = 100.;
const DEFAULT_LAND_FREIGHT_KG_PER_PERSON: f32 = 20.;
const WOOD_RESERVE_KG_PER_PERSON: f32 = 4.;
const TOOLS_RESERVE_KG_PER_PERSON: f32 = 0.75;
const BRICKS_RESERVE_KG_PER_PERSON: f32 = 3.;
const POTTERY_RESERVE_KG_PER_PERSON: f32 = 2.;
const CLOTH_RESERVE_KG_PER_PERSON: f32 = 0.8;
const LEATHER_RESERVE_KG_PER_PERSON: f32 = 0.2;
const WRITING_MATERIAL_RESERVE_KG_PER_PERSON: f32 = 0.1;
const WEAPONS_RESERVE_KG_PER_PERSON: f32 = 0.2;
const ARMOR_RESERVE_KG_PER_PERSON: f32 = 0.4;
const BASE_CONTAINER_MONTHLY_WEAR: f32 = 0.005;
const MIN_MATERIAL_PRICE: f32 = 0.01;
const MATERIAL_SERVICE_HORIZON_MONTHS: f32 = 120.;
const INITIAL_CONTAINER_SERVICE_PER_PERSON: f32 = 2.;
const MIN_PROCUREMENT_KG: f32 = 0.001;
const RECOVERY_PROCESSING_HORIZON_MONTHS: f32 = 3.;
const FOOD_SECURITY_RESERVE_MONTHS: f32 = 3.;
const MIN_FOREST_CARBON_FRACTION: f32 = 0.0001;
const FACILITY_ORDER_HEADROOM: f32 = 2.;
const MIN_TIN_ALLOY_STOCK_KG: f32 = 0.1;
const EXTRACTION_TOOLS_RESERVE_KG_PER_PERSON: f32 = 0.025;
const HERD_MONTHLY_FEED_FRACTION: f32 = 0.08;
const WRITING_KG_PER_PERSON: f32 = 0.04;
const FISHERY_MONTHLY_RETENTION: f32 = 0.997;
const FISHERY_MONTHLY_INVESTMENT_PER_PERSON: f32 = 0.015;
const WORKSHOP_DEMONSTRATED_HEADROOM: f32 = 1.25;
const MAX_WORKSHOP_UNITS_PER_PERSON: f32 = 0.03;
const WORKSHOP_PLAN_RETENTION: f32 = 0.9;
const WORKSHOP_PLAN_NEW_WEIGHT: f32 = 0.1;
const WORKSHOP_PLAN_SHARE_WEIGHT: f32 = 0.1;
const WORKSHOP_MONTHLY_INVESTMENT_PER_PERSON: f32 = 0.003;
const STORAGE_HEADROOM: f32 = 1.25;
const MAX_STORAGE_PER_PERSON_MULTIPLIER: f32 = 2.;
const STORAGE_MONTHLY_EXPANSION_KG_PER_PERSON: f32 = 10.;
const HOUSING_HEADROOM: f32 = 1.1;
const WATERWORKS_DISRUPTED_COVERAGE: f32 = 0.1;
const WATERWORKS_RECOVERED_COVERAGE: f32 = 0.5;
const WATERWORKS_ESTABLISHED_COVERAGE: f32 = 0.25;
const HOUSING_EXPANSION_NOTICE_RATIO: f32 = 1.1;
const HOUSING_DECLINE_NOTICE_RATIO: f32 = 0.9;
const HOUSING_NOTICE_MIN_RESIDENT_CHANGE: f32 = 2.;
const STORAGE_NOTICE_MIN_KG: f32 = 100.;
const STORAGE_EXPANSION_NOTICE_RATIO: f32 = 1.25;
const STORAGE_DECLINE_NOTICE_RATIO: f32 = 0.75;
const FEED_RESERVE_MONTHS: f32 = 6.;
const MIN_WRITING_STOCK_KG: f32 = 2.;
const MIN_WORKSHOP_PLAN_WORK: f32 = 0.001;
pub(crate) const WATERWORKS_NOTICE_MONTHS: u32 = 3;
const RESEARCH_PROCUREMENT_MAX_SAMPLES_KG: f64 = 12.;
const STORAGE_FORECAST_MONTHLY_RETENTION: f32 = 0.999;
const HOUSING_FORECAST_MONTHLY_RETENTION: f32 = 0.999;
const WATERWORKS_FORECAST_MONTHLY_RETENTION: f32 = 0.999;
const HOUSING_MONTHLY_INVESTMENT_PER_PERSON: f32 = 0.02;
const WATERWORKS_MONTHLY_INVESTMENT_PER_PERSON: f32 = 0.02;

crate::shared_shader_parameters! { SHADER_PARAMETERS {
    pub(crate) const ECONOMY_WATER_OPERATION_WORKER_MONTHS_PER_PERSON: f32 = 0.001;
    const MIN_WORK_TOOLS_KG_PER_PERSON: f32 = 0.5;
    pub(crate) const COPPER_TOOL_SERVICE_FACTOR: f32 = 0.6;
    pub(crate) const POTTERY_STORAGE_CAPACITY_FRACTION: f32 = 0.2;
    pub(crate) const WORKSHOP_WOOD_KG_PER_UNIT: f32 = 20.;
    pub(crate) const WORKSHOP_BRICKS_KG_PER_UNIT: f32 = 30.;
    pub(crate) const WORKSHOP_TOOLS_KG_PER_UNIT: f32 = 2.;
    pub(crate) const WORKSHOP_WORKER_MONTHS_PER_UNIT: f32 = 4.;
    pub(crate) const WORKSHOP_MONTHLY_WEAR: f32 = 0.002;
    pub(crate) const HOUSEHOLD_CRAFT_WORK_SHARE: f32 = 0.025;
    pub(crate) const HOUSING_WOOD_KG_PER_PERSON: f32 = 2.;
    pub(crate) const HOUSING_BRICKS_KG_PER_PERSON: f32 = 3.;
    pub(crate) const WATERWORKS_WOOD_KG_PER_PERSON: f32 = 2.;
    pub(crate) const WATERWORKS_BRICKS_KG_PER_PERSON: f32 = 4.;
    pub(crate) const WAREHOUSE_WOOD_KG_PER_KG: f32 = 0.02;
    pub(crate) const WAREHOUSE_BRICKS_KG_PER_KG: f32 = 0.03;
    pub(crate) const FISHERY_TRAP_WOOD_KG_PER_CREW: f32 = 20.;
    pub(crate) const FISHERY_BOAT_WOOD_KG_PER_CREW: f32 = 40.;
    pub(crate) const FISHERY_BOAT_TOOLS_KG_PER_CREW: f32 = 1.;
    pub(crate) const FISHERY_BOAT_CLOTH_KG_PER_CREW: f32 = 4.;
    pub(crate) const WORKSHOP_MONTHLY_RETENTION: f32 = 0.998;
    pub(crate) const FISHERY_TOOLS_RESERVE_KG_PER_PERSON: f32 = 0.15;
}}

pub const WORKSHOP_NAMES: [&str; 4] = [
    "General crafts",
    "Metalworking",
    "Kilns",
    "Textiles and leather",
];
// Production consumes installed capacity before household fallback. Preserve the
// utilized part of those assets even when household capacity could hypothetically
// have done the work. This is a repair target, never free capacity or a work grant.
fn demonstrated_workshop_units(installed: f32, completed_work: f32) -> f32 {
    installed
        .max(0.)
        .min(completed_work.max(0.) / WORKSHOP_WORKER_MONTHS_PER_UNIT)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ProductionSettings {
    pub enabled: bool,
    /// Bounded annual investment in a prospective food connection; false is an ablation.
    pub food_connection_investment: bool,
    /// Adapt staffing to feasible work; missing archive settings retain fixed staffing.
    pub adaptive_labor: bool,
    /// Diagnostic ablation: constant 62/8/10/20 percent work shares (fishery deducted).
    /// Disables shortage/order-driven reassignment, but retains finite workers and inputs.
    pub diagnostic_fixed_labor: bool,
    /// Experimental food-pressure response with bounded industrial maintenance capacity.
    pub food_security_labor: bool,
    /// Protect feasible industry within food-security staffing; false is a causal ablation.
    pub food_security_maintenance: bool,
    /// Opt-in: reserve at most 20% of existing recipe labor for local tool orders.
    pub replacement_tool_jobs: bool,
    /// Opt-in: completed toolmaking develops bounded local labor efficiency.
    pub toolmaking_expertise: bool,
    /// Require finite industrial workshop assets; absent archive setting is legacy.
    pub workshops: bool,
    pub persistent_storage: bool,
    pub persistent_housing: bool,
    pub waterworks: bool,
    /// Restore lost installed service before optional shelter headroom.
    pub waterworks_repair_priority: bool,
    /// Planned service capacity per resident; zero retains water demand and health exposure.
    pub waterworks_target_fraction: f32,
    pub initial_housing_per_person: f32,
    pub specialized_workshops: bool,
    pub export_contracts: bool,
    pub supplier_profitability: bool,
    /// Required markup over estimated replacement and opportunity cost.
    pub contract_margin: f32,
    /// Shared dry-goods yard/warehouse capacity; food has its own granaries.
    pub storage_kg_per_person: f32,
    /// Planned inland freight kg in transit per resident, including sea-port approaches.
    pub land_freight_kg_per_person: f32,
}
impl Default for ProductionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            food_connection_investment: true,
            adaptive_labor: false,
            diagnostic_fixed_labor: false,
            food_security_labor: false,
            food_security_maintenance: true,
            replacement_tool_jobs: false,
            toolmaking_expertise: false,
            workshops: false,
            persistent_storage: false,
            persistent_housing: false,
            waterworks: false,
            waterworks_repair_priority: false,
            waterworks_target_fraction: 1.,
            initial_housing_per_person: DEFAULT_INITIAL_HOUSING_PER_PERSON,
            specialized_workshops: false,
            export_contracts: false,
            supplier_profitability: false,
            contract_margin: DEFAULT_CONTRACT_MARGIN,
            storage_kg_per_person: DEFAULT_STORAGE_KG_PER_PERSON,
            land_freight_kg_per_person: DEFAULT_LAND_FREIGHT_KG_PER_PERSON,
        }
    }
}
pub fn empty_slots() -> [f32; GOODS] {
    [0.; GOODS]
}
/// Useful reserve levels, not a quota to own every possible catalog good.
pub fn reserve(id: &str) -> f32 {
    match id {
        "wood" => WOOD_RESERVE_KG_PER_PERSON,
        "tools" => TOOLS_RESERVE_KG_PER_PERSON,
        "bricks" => BRICKS_RESERVE_KG_PER_PERSON,
        "pottery" => POTTERY_RESERVE_KG_PER_PERSON,
        "cloth" => CLOTH_RESERVE_KG_PER_PERSON,
        "leather" => LEATHER_RESERVE_KG_PER_PERSON,
        "writing_material" => WRITING_MATERIAL_RESERVE_KG_PER_PERSON,
        "weapons" => WEAPONS_RESERVE_KG_PER_PERSON,
        "armor" => ARMOR_RESERVE_KG_PER_PERSON,
        _ => 0.,
    }
}
#[derive(Clone)]
struct Planner<'a> {
    catalog: &'a EconomyCatalog,
    available: [f32; GOODS],
    extractable: [f32; GOODS],
    targets: [f32; GOODS],
    orders: [f32; GOODS],
    visiting: [bool; GOODS],
    knowledge: u32,
}
impl Planner<'_> {
    /// One recipe stage backed by known stock/deliveries and unreserved local sources.
    /// This is a procurement forecast, not a promise about this month's workforce.
    fn supported_output(&self, good: usize) -> f32 {
        self.catalog
            .recipes
            .iter()
            .filter(|r| {
                r.output[good] > 0.
                    && (r.work[1] == 0. || self.knowledge & (1 << (r.work[1] as u32 - 1)) != 0)
            })
            .map(|r| {
                r.input
                    .iter()
                    .enumerate()
                    .filter(|(_, q)| **q > 0.)
                    .map(|(g, q)| {
                        (self.available[g] + (self.extractable[g] - self.targets[g]).max(0.)) / q
                    })
                    .fold(f32::INFINITY, f32::min)
                    * r.output[good]
            })
            .filter(|q| q.is_finite())
            .fold(0., f32::max)
    }
    fn harbor(&mut self, port: &crate::shipping::Port, population: f32) {
        for (k, good) in crate::shipping::MATERIALS.into_iter().enumerate() {
            let reserve_gap =
                (crate::shipping::material_reserve(good, population) - self.targets[good]).max(0.);
            self.request(good, reserve_gap + port.material_deficit()[k]);
        }
    }
    fn containers(&mut self, prices: &[f32; GOODS], population: f32) {
        let Some(materials) = &self.catalog.materials else {
            return;
        };
        let mut options: Vec<_> = materials
            .variants
            .iter()
            .filter(|v| v.role == "container")
            .map(|v| (v.slot, v.service, v.wear))
            .chain(std::iter::once((7, 1., BASE_CONTAINER_MONTHLY_WEAR)))
            .collect();
        options.sort_by(|a, b| {
            let cost = |x: &(usize, f32, f32)| {
                prices[x.0].max(MIN_MATERIAL_PRICE) / x.1
                    * (1. + x.2 * MATERIAL_SERVICE_HORIZON_MONTHS)
            };
            cost(a).total_cmp(&cost(b)).then(a.0.cmp(&b.0))
        });
        let mut needed = population.max(0.) * INITIAL_CONTAINER_SERVICE_PER_PERSON;
        // Keep only useful existing service; surplus remains available for other demands/trade.
        for &(g, service, _) in &options {
            let held = self.available[g].min(needed / service);
            self.request(g, held);
            needed = (needed - held * service).max(0.);
        }
        // Mix feasible substitutes instead of waiting exclusively for the cheapest missing input.
        for &(g, service, _) in &options {
            let make = self.supported_output(g).min(needed / service);
            self.request(g, make);
            needed = (needed - make * service).max(0.);
        }
        // Remaining demand still enters ordinary upstream production/import planning.
        if let Some(&(g, service, _)) = options.first() {
            self.request(g, needed / service);
        }
    }
    /// Replay only the requested recipe chain against finite projected inputs.
    /// No labor is promised and no physical inventory is written. Repeated passes
    /// permit upstream recipes later in the catalog without assuming free inputs.
    fn supported_chain(&self, good: usize, quantity: f32) -> f32 {
        let mut plan = self.clone();
        plan.orders.fill(0.);
        plan.request(good, quantity);
        let mut stock = self.available;
        for (g, held) in stock.iter_mut().enumerate() {
            *held += (self.extractable[g] - self.targets[g]).max(0.);
        }
        let opening = stock[good];
        for _ in 0..self.catalog.recipes.len() {
            let mut progressed = false;
            for (i, r) in self.catalog.recipes.iter().enumerate() {
                let batches = r
                    .input
                    .iter()
                    .enumerate()
                    .filter(|(_, q)| **q > 0.)
                    .map(|(g, q)| stock[g] / q)
                    .fold(plan.orders[i], f32::min);
                if batches < MIN_PROCUREMENT_KG {
                    continue;
                }
                plan.orders[i] = (plan.orders[i] - batches).max(0.);
                for (g, held) in stock.iter_mut().enumerate() {
                    *held = (*held - batches * r.input[g]).max(0.) + batches * r.output[g];
                }
                progressed = true;
            }
            if !progressed {
                break;
            }
        }
        (stock[good] - opening).max(0.).min(quantity)
    }
    /// Retain useful held substitutes before ordering the remaining tool service.
    /// Targets protect that stock from sale; counting coverage alone does not.
    fn tools(&mut self, selected: usize, service_needed: f32, alloys: bool) {
        self.tool_plan(selected, service_needed, alloys, true);
    }
    fn tool_plan(&mut self, selected: usize, service_needed: f32, alloys: bool, fallback: bool) {
        let mut missing = service_needed.max(0.);
        for (good, service) in [(3, 1.), (41, 1.), (43, COPPER_TOOL_SERVICE_FACTOR)] {
            if good != 3 && !alloys {
                continue;
            }
            let retained = self.available[good].min(missing / service);
            self.request(good, retained);
            missing = (missing - retained * service).max(0.);
        }
        // A town can work imported metal/ore even when its local deposit is a
        // different mineral. Prefer supported chains before speculative imports.
        if alloys {
            for good in [selected, 3, 41, 43] {
                let service = if good == 43 {
                    COPPER_TOOL_SERVICE_FACTOR
                } else {
                    1.
                };
                let make = self.supported_chain(good, missing / service);
                self.request(good, make);
                missing = (missing - make * service).max(0.);
            }
        }
        let service = if selected == 43 {
            COPPER_TOOL_SERVICE_FACTOR
        } else {
            1.
        };
        if fallback {
            self.request(selected, missing / service);
        }
    }
    fn construction_quote(&self, e: &crate::economy::Economy) -> crate::economy::Economy {
        let mut quote = *e;
        for g in 0..GOODS {
            quote.goods[g] = self.available[g] + (self.extractable[g] - self.targets[g]).max(0.);
        }
        // Include possible assembly of tiles from actual bricks; no speculative upstream cascade.
        if let Some(materials) = &self.catalog.materials {
            for v in materials.variants.iter().filter(|v| v.role == "roof") {
                quote.goods[v.slot] += self.supported_output(v.slot);
            }
        }
        quote
    }
    fn request(&mut self, good: usize, quantity: f32) {
        if quantity <= 0. || self.visiting[good] {
            return;
        }
        self.targets[good] += quantity;
        let from_stock = self.available[good].min(quantity);
        self.available[good] -= from_stock;
        let missing = quantity - from_stock;
        if missing < MIN_PROCUREMENT_KG {
            return;
        }
        // Prefer recipes with stocked inputs (including scrap), then lower labor cost.
        let recipe = self
            .catalog
            .recipes
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.output[good] > 0.
                    && (r.input[29] == 0.
                        || self.available[29] >= r.input[29] * missing / r.output[good])
                    && (r.input[44] == 0.
                        || self.available[44] >= r.input[44] * missing / r.output[good])
                    && (r.input[42] == 0.
                        || self.available[42] >= r.input[42] * missing / r.output[good])
                    && (r.work[1] == 0. || self.knowledge & (1 << (r.work[1] as u32 - 1)) != 0)
            })
            .min_by(|(ia, a), (ib, b)| {
                let cost = |r: &crate::economy::Recipe| {
                    (r.work[0]
                        + r.input
                            .iter()
                            .enumerate()
                            .map(|(k, v)| (v - self.available[k] - self.extractable[k]).max(0.))
                            .sum::<f32>())
                        / r.output[good]
                };
                cost(a).total_cmp(&cost(b)).then(ia.cmp(ib))
            })
            .map(|(i, r)| (i, *r));
        if let Some((i, r)) = recipe {
            self.visiting[good] = true;
            let batches = missing / r.output[good];
            self.orders[i] += batches;
            for (k, input) in r.input.iter().enumerate() {
                if *input > 0. {
                    self.request(k, input * batches);
                }
            }
            for (k, output) in r.output.iter().enumerate() {
                if k != good {
                    self.available[k] += output * batches;
                }
            }
            self.visiting[good] = false;
        }
    }
}
/// Demand for an offered stored metal input, bounded by an ordinary tool-service
/// deficit and a recipe chain supported by held/expected supplies. Travel, cash,
/// storage and source stock are checked by the recovery transaction separately.
pub(crate) fn recovery_tool_input_demand(
    catalog: &EconomyCatalog,
    economy: &crate::economy::Economy,
    population: f32,
    expected: [f32; GOODS],
    good: usize,
    offered: f32,
) -> f32 {
    if !matches!(good, 32..=40) || economy.extraction[1] <= 0.5 || economy.logistics[3] <= 0.5 {
        return 0.;
    }
    let mut available = economy.goods;
    for g in 0..GOODS {
        available[g] += expected[g];
    }
    let already = available[good];
    let mut planner = Planner {
        catalog,
        available,
        extractable: [0.; GOODS],
        targets: [0.; GOODS],
        orders: [0.; GOODS],
        visiting: [false; GOODS],
        knowledge: economy.management[3] as u32,
    };
    let desired = population.max(0.) * reserve("tools");
    planner.tool_plan(3, desired, true, false);
    let covered =
        planner.targets[3] + planner.targets[41] + COPPER_TOOL_SERVICE_FACTOR * planner.targets[43];
    // Use existing inputs first. An offer may meet remaining need, not merely
    // displace an already supported chain and leave old stock idle.
    let opening_orders = planner.orders;
    planner.available[good] += offered.max(0.);
    planner.tool_plan(3, (desired - covered).max(0.), true, false);
    let work: f32 = catalog
        .recipes
        .iter()
        .enumerate()
        .map(|(i, r)| (planner.orders[i] - opening_orders[i]).max(0.) * r.work[0])
        .sum();
    let work_budget = RECOVERY_PROCESSING_HORIZON_MONTHS
        * economy.workshop_types[1][2]
            .max(economy.enterprise_plan[1])
            .max(0.);
    let fraction = if work > 0. {
        (work_budget / work).min(1.)
    } else {
        0.
    };
    (planner.targets[good] - already)
        .max(0.)
        .min(offered.max(0.))
        * fraction
}

impl History {
    pub(crate) fn plan_production(&mut self) {
        let Some(catalog) = self.economy_catalog.as_ref() else {
            return;
        };
        for c in &mut self.export_contracts {
            c.planned_kg = if catalog.production.enabled
                && catalog.production.export_contracts
                && c.escrow > 0.
                && self.month < c.expires
            {
                c.remaining_kg.min(c.escrow / c.unit_price)
            } else {
                0.
            };
        }
        let local_members: Vec<usize> = self
            .culture
            .as_ref()
            .map(|culture| {
                let present: Vec<_> = self
                    .sites
                    .iter()
                    .map(|s| culture.site_people(self, s.id))
                    .collect();
                culture
                    .institutions
                    .iter()
                    .map(|n| {
                        present[n.site as usize]
                            .iter()
                            .filter(|p| n.members.contains(p))
                            .count()
                    })
                    .collect()
            })
            .unwrap_or_default();
        let food_connections = if self.month % 12 == 0 {
            self.food_connection_investments()
        } else {
            vec![false; self.sites.len()]
        };
        for s in &mut self.sites {
            let e = &mut s.economy;
            e.logistics = [0.; 4];
            e.harbor_work = [0.; 4];
            e.tool_craft[1] =
                f32::from(catalog.production.enabled && catalog.production.replacement_tool_jobs);
            e.tool_craft[2] =
                f32::from(catalog.production.enabled && catalog.production.toolmaking_expertise);
            e.tool_orders = [0.; GOODS];
            e.waterworks[3] =
                f32::from(catalog.production.enabled && catalog.production.waterworks);
            e.waterworks_recovery[1] = f32::from(catalog.production.waterworks_repair_priority);
            e.housing_plan[3] =
                f32::from(catalog.production.enabled && catalog.production.persistent_housing);
            if e.housing_plan[3] > 0.5 && e.housing[2] == 0. {
                // Recorded founding shelter baseline, not newly imported building materials.
                e.housing[2] =
                    s.stocks.stock[0].max(1.) * catalog.production.initial_housing_per_person;
            }
            e.storage_plan[3] =
                f32::from(catalog.production.enabled && catalog.production.persistent_storage);
            if e.storage_plan[3] > 0.5 && e.storage[2] == 0. {
                // Baseline open yards, not imported building materials. Never resize later.
                e.storage[2] = s.stocks.stock[0].max(1.) * catalog.production.storage_kg_per_person;
            }
            if e.storage[2] > 0. {
                e.logistics[0] = e.storage_capacity() + e.container_capacity(catalog);
            }
            e.workshop[3] = f32::from(catalog.production.enabled && catalog.production.workshops);
            e.workshop_types[0][3] =
                f32::from(e.workshop[3] > 0.5 && catalog.production.specialized_workshops);
            if !catalog.production.enabled || s.abandoned {
                continue;
            }
            let pop = s.stocks.stock[0].max(1.);
            if catalog.production.food_security_labor {
                let age_need = s.demography.ages[0]
                    * crate::society::CHILD_RATION_KG_PER_MONTH as f32
                    + s.demography.ages[1] * crate::society::ADULT_RATION_KG_PER_MONTH as f32
                    + s.demography.ages[2] * crate::society::ELDER_RATION_KG_PER_MONTH as f32;
                let need = if age_need > 0. {
                    age_need
                } else {
                    pop * crate::society::ADULT_RATION_KG_PER_MONTH as f32
                };
                let months = s.stocks.stock[1].max(0.) / need;
                let shortage = if s.demography.ration_need[3] > 0. {
                    1. - s.demography.ration_eaten[3] / s.demography.ration_need[3]
                } else {
                    s.stocks.stock[3]
                };
                let tools = e.goods[3]
                    + if e.extraction[1] > 0.5 {
                        e.goods[41] + COPPER_TOOL_SERVICE_FACTOR * e.goods[43]
                    } else {
                        0.
                    };
                e.food_labor[1] = shortage
                    .clamp(0., 1.)
                    .max((1. - months / FOOD_SECURITY_RESERVE_MONTHS).clamp(0., 1.));
                e.food_labor[2] = months;
                e.food_labor[3] = (1. - tools / (pop * MIN_WORK_TOOLS_KG_PER_PERSON)).clamp(0., 1.);
            }
            let mut planner = Planner {
                catalog,
                available: e.goods,
                extractable: {
                    let mut supply = [0.; GOODS];
                    supply[e.extraction[0].max(1.) as usize] = e.reserves[1];
                    supply[0] =
                        e.forest[0] / catalog.composition(0)[0].max(MIN_FOREST_CARBON_FRACTION);
                    supply[4] = e.reserves[2];
                    supply
                },
                targets: [0.; GOODS],
                orders: [0.; GOODS],
                visiting: [false; GOODS],
                knowledge: e.management[3] as u32,
            };
            let mut incoming = [0.; GOODS];
            for cargo in self
                .cargo
                .iter()
                .filter(|c| c.to == s.id && c.good as usize != FOOD)
            {
                planner.available[cargo.good as usize] += cargo.kg;
                incoming[cargo.good as usize] += cargo.kg;
            }
            // Funded procurement is expected supply, not physical inventory.
            // Suppress duplicate local recipes; only actual cargo reduces market targets.
            for c in self
                .export_contracts
                .iter()
                .filter(|c| c.buyer == s.id && c.planned_kg > 0.)
            {
                planner.available[c.good as usize] += c.planned_kg;
            }
            // Institutional procurement feeds the ordinary production/trade path. Only funded,
            // unfinished demand is proposed; material still must be present when construction buys it.
            if let Some(culture) = &self.culture {
                for n in culture
                    .institutions
                    .iter()
                    .filter(|n| n.active && n.site == s.id)
                {
                    if let Some(f) = n
                        .capacity
                        .as_ref()
                        .and_then(|c| c.building.as_ref())
                        .and_then(|b| b.facility.as_ref())
                    {
                        let local = local_members[n.id as usize];
                        if local == 0 {
                            continue;
                        }
                        let target = crate::facilities::demand(&n.kind, local);
                        // Procurement precedes the due fee; repair_budget then protects the next.
                        for (good, mass) in f.repair_order(
                            e,
                            crate::facilities::repair_budget(n.treasury),
                            FACILITY_REPAIR_FORECAST_WORKER_MONTHS,
                        ) {
                            planner.request(good as usize, mass);
                        }
                        if f.remaining() == 0.
                            && f.condition() >= crate::facilities::MIN_EXPANSION_CONDITION
                            && target - f.planned() >= crate::facilities::MIN_ROOM_CAPACITY
                        {
                            let quoted = planner.construction_quote(e);
                            if let Some(room) = crate::facilities::choose(
                                catalog,
                                &quoted,
                                target - f.planned(),
                                crate::facilities::expansion_budget(f, e, n.treasury),
                            ) {
                                for (good, mass) in room.materials() {
                                    planner.request(good as usize, mass * FACILITY_ORDER_HEADROOM);
                                }
                            }
                        }
                    }
                }
            }
            // Same finite stock/expected deliveries and recipe knowledge as ordinary orders.
            // Only the working tool requirement is urgent; exports and reserve expansion are not.
            let mut replacement = planner.clone();
            if catalog.production.replacement_tool_jobs {
                let selected = if e.extraction[1] > 0.5 && matches!(e.extraction[0] as u32, 35..=37)
                {
                    if e.extraction[0] == 37.
                        || planner.available[39] >= MIN_TIN_ALLOY_STOCK_KG
                        || planner.available[40] > 0.
                    {
                        41
                    } else {
                        43
                    }
                } else {
                    3
                };
                replacement.tools(
                    selected,
                    pop * MIN_WORK_TOOLS_KG_PER_PERSON,
                    e.extraction[1] > 0.5,
                );
            }
            if let Some(methods) = &catalog.materials {
                planner.containers(&e.prices, pop);
                for v in &methods.variants {
                    let need = match v.role.as_str() {
                        "digging" | "breaking" => {
                            if e.reserves[1] + e.reserves[2] > 0. {
                                pop * EXTRACTION_TOOLS_RESERVE_KG_PER_PERSON
                            } else {
                                0.
                            }
                        }
                        "cutting" => {
                            if e.forest[0] > 0. {
                                pop * EXTRACTION_TOOLS_RESERVE_KG_PER_PERSON
                            } else {
                                0.
                            }
                        }
                        _ => 0.,
                    };
                    planner.request(v.slot, need);
                }
            }
            for (k, g) in catalog.goods.iter().enumerate() {
                if g.id == "pottery" && catalog.materials.is_some() {
                    continue;
                }
                if g.id == "tools" && e.extraction[1] > 0.5 {
                    let desired = pop * reserve("tools");
                    let selected = if matches!(e.extraction[0] as u32, 35..=37) {
                        if e.extraction[0] == 37.
                            || planner.available[39] >= MIN_TIN_ALLOY_STOCK_KG
                            || planner.available[40] > 0.
                        {
                            41
                        } else {
                            43
                        }
                    } else {
                        k
                    };
                    planner.tools(selected, desired, true);
                } else {
                    planner.request(k, pop * reserve(&g.id));
                }
            }
            // Surveyed harbors are finite construction customers. Forecast the next
            // annual installation after ordinary reserves; execution still requires
            // physical materials and leftover work at the annual boundary.
            if e.policy[3] >= 0.5 {
                if let Some(port) = self
                    .shipping
                    .as_ref()
                    .and_then(|shipping| shipping.ports.iter().find(|p| p.site == s.id))
                {
                    planner.harbor(port, s.stocks.stock[0]);
                    if self.month % 12 == 0
                        && port.flood_months == 0
                        && food_connections[s.id as usize]
                    {
                        e.harbor_work[0] = port.supported_work(&e.goods, s.stocks.stock[0]);
                    }
                }
            }
            for c in self
                .export_contracts
                .iter()
                .filter(|c| c.seller == s.id && c.planned_kg > 0.)
            {
                planner.request(c.good as usize, c.planned_kg);
            }
            // Research is a real customer even when ordinary craft chains are idle.
            if let Some(w) = self
                .expeditions
                .as_ref()
                .and_then(|x| x.discoveries.as_ref())
                .and_then(|d| d.workshops.iter().find(|w| w.site == s.id && w.enabled))
            {
                let samples = w
                    .samples
                    .iter()
                    .sum::<f64>()
                    .min(RESEARCH_PROCUREMENT_MAX_SAMPLES_KG) as f32;
                planner.request(
                    6,
                    samples * crate::discoveries::PROCESSING_FUEL_KG_PER_KG as f32,
                );
                planner.request(
                    3,
                    samples * crate::discoveries::PROCESSING_TOOLS_KG_PER_KG as f32,
                );
            }
            // Feed reserves compete with food; preserve enough seed for establishment.
            if let Some(a) = &catalog.agriculture {
                for (j, herd) in a.herds.iter().enumerate() {
                    if let Some(k) = catalog.index(&herd.feed) {
                        planner.request(
                            k,
                            e.herds[j][0] * HERD_MONTHLY_FEED_FRACTION * FEED_RESERVE_MONTHS,
                        );
                    }
                }
                for crop in &a.crops {
                    if let Some(k) = catalog.index(&crop.good) {
                        planner.targets[k] += pop * WRITING_KG_PER_PERSON + MIN_WRITING_STOCK_KG;
                    }
                }
            }
            if e.fishery[3] > 0.5 && e.management[1] >= 1. {
                let traps = if e.fishery_traps[3] > 0.5 {
                    e.fishery_traps[0] * FISHERY_MONTHLY_RETENTION / FISHERY_TRAP_WOOD_KG_PER_CREW
                } else {
                    0.
                };
                if e.fishery_traps[3] > 0.5
                    && (e.goods[16] < FISHERY_BOAT_CLOTH_KG_PER_CREW
                        || e.goods[3] < pop * FISHERY_TOOLS_RESERVE_KG_PER_PERSON + 1.)
                {
                    // Request the feasible alternative, not two complete sets of gear.
                    let equipped = (e.fishery[0] / FISHERY_BOAT_WOOD_KG_PER_CREW)
                        .min(e.fishery[1])
                        .min(e.fishery[2] / FISHERY_BOAT_CLOTH_KG_PER_CREW)
                        * FISHERY_MONTHLY_RETENTION;
                    let missing = (e.fishery_plan[0] - traps - equipped).max(0.)
                        * FISHERY_TRAP_WOOD_KG_PER_CREW;
                    planner.request(
                        0,
                        missing.min(
                            pop * FISHERY_TRAP_WOOD_KG_PER_CREW
                                * FISHERY_MONTHLY_INVESTMENT_PER_PERSON,
                        ),
                    );
                } else {
                    for (j, good) in [0, 3, 16].into_iter().enumerate() {
                        let cost = [
                            FISHERY_BOAT_WOOD_KG_PER_CREW,
                            FISHERY_BOAT_TOOLS_KG_PER_CREW,
                            FISHERY_BOAT_CLOTH_KG_PER_CREW,
                        ][j];
                        let crew = (e.fishery_plan[0] - traps).max(0.);
                        let missing =
                            (crew * cost - e.fishery[j] * FISHERY_MONTHLY_RETENTION).max(0.);
                        planner.request(
                            good,
                            missing.min(pop * cost * FISHERY_MONTHLY_INVESTMENT_PER_PERSON),
                        );
                    }
                }
            }
            if catalog.production.workshops {
                // Persistent orders justify capacity, bounded by the town workforce.
                // Food processing remains possible with household equipment.
                let industrial_work: f32 = catalog
                    .recipes
                    .iter()
                    .zip(planner.orders)
                    .filter(|(r, _)| {
                        !r.output.iter().enumerate().any(|(k, v)| {
                            *v > 0. && catalog.goods.get(k).is_some_and(|g| g.food_energy > 0.)
                        })
                    })
                    .map(|(r, batches)| r.work[0] * batches)
                    .sum();
                // A backlog is not proof that materials or customers can sustain
                // a larger industry. Expand from demonstrated use, with 25% headroom.
                let household = (e.labor.iter().sum::<f32>() * HOUSEHOLD_CRAFT_WORK_SHARE).max(1.);
                let demonstrated = e.workshop_plan[3] * WORKSHOP_DEMONSTRATED_HEADROOM;
                let desired = ((industrial_work.min(demonstrated) - household).max(0.)
                    / WORKSHOP_WORKER_MONTHS_PER_UNIT)
                    .min(pop * MAX_WORKSHOP_UNITS_PER_PERSON);
                e.workshop_plan[0] = e.workshop_plan[0] * WORKSHOP_PLAN_RETENTION
                    + desired * WORKSHOP_PLAN_NEW_WEIGHT;
                if catalog.production.specialized_workshops {
                    let mut work = [0f32; 4];
                    for (r, batches) in catalog.recipes.iter().zip(planner.orders) {
                        if !r
                            .output
                            .iter()
                            .enumerate()
                            .any(|(k, v)| *v > 0. && catalog.goods[k].food_energy > 0.)
                        {
                            work[r.work[2] as usize] += r.work[0] * batches;
                        }
                    }
                    for (j, w) in work.iter_mut().enumerate() {
                        *w = w.min(e.workshop_types[j][2] * WORKSHOP_DEMONSTRATED_HEADROOM);
                    }
                    let total: f32 = work.iter().sum();
                    let desired = ((total - household).max(0.) / WORKSHOP_WORKER_MONTHS_PER_UNIT)
                        .min(pop * MAX_WORKSHOP_UNITS_PER_PERSON);
                    for (j, w) in work.iter().enumerate() {
                        let utilized = demonstrated_workshop_units(
                            e.workshop_types[j][0],
                            e.workshop_types[j][2],
                        );
                        let target =
                            (desired * w / total.max(MIN_WORKSHOP_PLAN_WORK)).max(utilized);
                        e.workshop_types[j][1] = e.workshop_types[j][1] * WORKSHOP_PLAN_RETENTION
                            + target * WORKSHOP_PLAN_SHARE_WEIGHT;
                    }
                    // Existing buildings retain their specialization; idle capacity cannot
                    // impersonate another industry's equipment.
                    let missing: f32 = e
                        .workshop_types
                        .iter()
                        .map(|t| (t[1] - t[0] * WORKSHOP_MONTHLY_RETENTION).max(0.))
                        .sum();
                    let units = (e.workshop[0] / WORKSHOP_WOOD_KG_PER_UNIT)
                        .min(e.workshop[1] / WORKSHOP_BRICKS_KG_PER_UNIT)
                        .min(e.workshop[2] / WORKSHOP_TOOLS_KG_PER_UNIT);
                    let assigned: f32 = e.workshop_types.iter().map(|t| t[0]).sum();
                    e.workshop_plan[0] = (units * WORKSHOP_MONTHLY_RETENTION
                        + (missing - (units - assigned).max(0.) * WORKSHOP_MONTHLY_RETENTION)
                            .max(0.))
                    .min(pop * MAX_WORKSHOP_UNITS_PER_PERSON);
                }
                for (j, good) in [0, 5, 3].into_iter().enumerate() {
                    let cost = [
                        WORKSHOP_WOOD_KG_PER_UNIT,
                        WORKSHOP_BRICKS_KG_PER_UNIT,
                        WORKSHOP_TOOLS_KG_PER_UNIT,
                    ][j];
                    let missing = (e.workshop_plan[0] * cost - e.workshop[j]).max(0.);
                    planner.request(
                        good,
                        missing.min(pop * cost * WORKSHOP_MONTHLY_INVESTMENT_PER_PERSON),
                    );
                }
            }
            if e.storage_plan[3] > 0.5 {
                let stored: f32 = catalog
                    .goods
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| g.food_energy <= 0.)
                    .map(|(k, _)| e.goods[k] + incoming[k])
                    .sum();
                // Expand from stock actually handled, not speculative orders or population growth.
                let desired = (stored * STORAGE_HEADROOM).min(
                    (pop * catalog.production.storage_kg_per_person
                        * MAX_STORAGE_PER_PERSON_MULTIPLIER)
                        .max(e.storage[2]),
                );
                e.storage_plan[0] = (desired - e.storage[2]).max(0.);
                for (j, good) in [0, 5].into_iter().enumerate() {
                    let cost = [WAREHOUSE_WOOD_KG_PER_KG, WAREHOUSE_BRICKS_KG_PER_KG][j];
                    planner.request(
                        good,
                        (e.storage_plan[0] * cost
                            - e.storage[j] * STORAGE_FORECAST_MONTHLY_RETENTION)
                            .max(0.)
                            .min(pop * cost * STORAGE_MONTHLY_EXPANSION_KG_PER_PERSON),
                    );
                }
            }
            if e.housing_plan[3] > 0.5 {
                let pending: f32 = self.society.as_ref().map_or(0., |soc| {
                    soc.relocation
                        .journeys
                        .iter()
                        .filter(|j| j.to == s.id && !j.returning)
                        .map(|j| j.population())
                        .sum()
                });
                e.housing_plan[0] = ((pop + pending) * HOUSING_HEADROOM - e.housing[2]).max(0.);
                for (j, good) in [0, 5].into_iter().enumerate() {
                    let cost = [HOUSING_WOOD_KG_PER_PERSON, HOUSING_BRICKS_KG_PER_PERSON][j];
                    planner.request(
                        good,
                        (e.housing_plan[0] * cost
                            - e.housing[j] * HOUSING_FORECAST_MONTHLY_RETENTION)
                            .max(0.)
                            .min(pop * cost * HOUSING_MONTHLY_INVESTMENT_PER_PERSON),
                    );
                }
            }
            if e.waterworks[3] > 0.5 {
                e.waterworks_plan[0] = pop * catalog.production.waterworks_target_fraction;
                for (j, good) in [0, 5].into_iter().enumerate() {
                    let cost = [
                        WATERWORKS_WOOD_KG_PER_PERSON,
                        WATERWORKS_BRICKS_KG_PER_PERSON,
                    ][j];
                    planner.request(
                        good,
                        (e.waterworks_plan[0] * cost
                            - e.waterworks[j] * WATERWORKS_FORECAST_MONTHLY_RETENTION)
                            .max(0.)
                            .min(pop * cost * WATERWORKS_MONTHLY_INVESTMENT_PER_PERSON),
                    );
                }
            }
            e.targets = std::array::from_fn(|k| (planner.targets[k] - incoming[k]).max(0.));
            e.orders = planner.orders;
            e.tool_orders = std::array::from_fn(|k| replacement.orders[k].min(e.orders[k]));
            e.logistics = [
                if e.storage_plan[3] > 0.5 {
                    e.storage_capacity() + e.container_capacity(catalog)
                } else {
                    pop * catalog.production.storage_kg_per_person
                },
                incoming
                    .iter()
                    .enumerate()
                    .filter(|(k, _)| catalog.goods.get(*k).is_some_and(|g| g.food_energy <= 0.))
                    .map(|(_, v)| v)
                    .sum(),
                0.,
                if catalog.production.diagnostic_fixed_labor {
                    3.
                } else if catalog.production.food_security_labor {
                    if catalog.production.food_security_maintenance {
                        4.
                    } else {
                        5.
                    }
                } else if catalog.production.adaptive_labor {
                    2.
                } else {
                    1.
                },
            ];
        }
    }
}

impl History {
    pub(crate) fn waterworks_events(&mut self) {
        for i in 0..self.sites.len() {
            let s = &mut self.sites[i];
            if s.economy.waterworks[3] < 0.5 {
                continue;
            }
            let coverage = s.economy.waterworks_plan[3];
            for (j, condition) in [
                coverage < WATERWORKS_DISRUPTED_COVERAGE,
                coverage >= WATERWORKS_RECOVERED_COVERAGE,
            ]
            .into_iter()
            .enumerate()
            {
                s.lifecycle.waterworks_months[j] = if condition {
                    s.lifecycle.waterworks_months[j]
                        .saturating_add(1)
                        .min(WATERWORKS_NOTICE_MONTHS)
                } else {
                    0
                };
            }
            let kind = match s.lifecycle.waterworks_operating {
                None if coverage >= WATERWORKS_ESTABLISHED_COVERAGE => {
                    Some("waterworks_established")
                }
                Some(true) if s.lifecycle.waterworks_months[0] >= WATERWORKS_NOTICE_MONTHS => {
                    Some("waterworks_disrupted")
                }
                Some(false) if s.lifecycle.waterworks_months[1] >= WATERWORKS_NOTICE_MONTHS => {
                    Some("waterworks_recovered")
                }
                _ => None,
            };
            if let Some(kind) = kind {
                s.lifecycle.waterworks_operating =
                    Some(coverage >= WATERWORKS_ESTABLISHED_COVERAGE);
                let detail = format!("Operating sanitation coverage {:.0}%; domestic water shortfall {:.0}%; installed service capacity {:.1} residents; historical peak {:.1}; priority repair work {:.2} worker-months this month",coverage*100.,s.economy.water_service[1]*100.,s.economy.waterworks_capacity(),s.economy.waterworks_recovery[0],s.economy.waterworks_recovery[2]);
                let cause = self
                    .events
                    .iter()
                    .rev()
                    .find(|e| e.site == Some(i as u32) && e.kind.starts_with("waterworks_"))
                    .map(|e| e.id);
                self.event(kind, Some(i as u32), None, detail);
                self.events.last_mut().unwrap().causes.extend(cause);
            }
        }
    }
    pub(crate) fn housing_events(&mut self) {
        for i in 0..self.sites.len() {
            let s = &mut self.sites[i];
            if s.economy.housing_plan[3] < 0.5 {
                continue;
            }
            let capacity = s.economy.housing_capacity();
            let kind = match s.lifecycle.housing_reported {
                None => Some("housing_baseline"),
                Some(v)
                    if capacity > v * HOUSING_EXPANSION_NOTICE_RATIO
                        && capacity - v > HOUSING_NOTICE_MIN_RESIDENT_CHANGE =>
                {
                    Some("housing_expanded")
                }
                Some(v)
                    if capacity < v * HOUSING_DECLINE_NOTICE_RATIO
                        && v - capacity > HOUSING_NOTICE_MIN_RESIDENT_CHANGE =>
                {
                    Some("housing_deteriorated")
                }
                _ => None,
            };
            if let Some(kind) = kind {
                s.lifecycle.housing_reported = Some(capacity);
                let detail = format!("Shelter capacity {:.1} places, including {:.1} recorded baseline places; {:.1} residents; installed timber/bricks {:.2}/{:.2} kg",
                    capacity,s.economy.housing[2],s.stocks.stock[0],s.economy.housing[0],s.economy.housing[1]);
                let cause = self
                    .events
                    .iter()
                    .rev()
                    .find(|e| {
                        e.site == Some(i as u32)
                            && matches!(
                                e.kind.as_str(),
                                "housing_baseline" | "housing_expanded" | "housing_deteriorated"
                            )
                    })
                    .map(|e| e.id);
                self.event(kind, Some(i as u32), None, detail);
                self.events.last_mut().unwrap().causes.extend(cause);
            }
        }
    }
    pub(crate) fn storage_events(&mut self) {
        for i in 0..self.sites.len() {
            let s = &mut self.sites[i];
            if s.economy.storage_plan[3] < 0.5 {
                continue;
            }
            let capacity = s.economy.storage_capacity() - s.economy.storage[2];
            let previous = s.lifecycle.storage_reported;
            let kind = match previous {
                None => Some("storage_baseline"),
                Some(v) if v < STORAGE_NOTICE_MIN_KG && capacity >= STORAGE_NOTICE_MIN_KG => {
                    Some("warehouse_established")
                }
                Some(v)
                    if capacity > v * STORAGE_EXPANSION_NOTICE_RATIO
                        && capacity - v >= STORAGE_NOTICE_MIN_KG =>
                {
                    Some("warehouse_expanded")
                }
                Some(v)
                    if capacity < v * STORAGE_DECLINE_NOTICE_RATIO
                        && v - capacity >= STORAGE_NOTICE_MIN_KG =>
                {
                    Some("warehouse_deteriorated")
                }
                _ => None,
            };
            if let Some(kind) = kind {
                s.lifecycle.storage_reported = Some(capacity);
                let detail = format!("Recorded yard allowance {:.0} kg; material-backed warehouse capacity {:.0} kg; timber {:.2} kg, bricks {:.2} kg, cumulative wear {:.2} kg",
                    s.economy.storage[2],capacity,s.economy.storage[0],s.economy.storage[1],s.economy.storage_plan[1]);
                let cause = self
                    .events
                    .iter()
                    .rev()
                    .find(|e| {
                        e.site == Some(i as u32)
                            && (e.kind.starts_with("warehouse_") || e.kind == "storage_baseline")
                    })
                    .map(|e| e.id);
                self.event(kind, Some(i as u32), None, detail);
                self.events.last_mut().unwrap().causes.extend(cause);
            }
        }
    }

    pub fn production_summary(&self) -> serde_json::Value {
        let totals: Vec<f64> = (0..GOODS)
            .map(|k| self.sites.iter().map(|s| s.economy.goods[k] as f64).sum())
            .collect();
        let pop: f64 = self.sites.iter().map(|s| s.stocks.stock[0] as f64).sum();
        serde_json::json!({"resource_sources":self.resources,"goods_kg":totals,"goods_kg_per_person":totals.iter().sum::<f64>()/pop.max(1.),
            "dry_capacity_kg":self.sites.iter().map(|s|s.economy.logistics[0] as f64).sum::<f64>(),
            "waterworks_assets": self.sites.iter().map(|s| serde_json::json!({"site":s.id,"capacity":s.economy.waterworks_capacity(),"materials":&s.economy.waterworks[..2],"construction_work":s.economy.waterworks[2],"coverage":s.economy.waterworks_plan[3],"worn":s.economy.waterworks_plan[1],"domestic_water":s.economy.water_service[0],"water_shortage":s.economy.water_service[1],"operating_work":s.economy.water_service[2],"served_residents":s.economy.water_service[3],"target_residents":s.economy.waterworks_plan[0],"recovery":s.economy.waterworks_recovery})).collect::<Vec<_>>(),
            "housing_assets": self.sites.iter().map(|s| serde_json::json!({"site":s.id,"capacity":s.economy.housing_capacity(),"baseline":s.economy.housing[2],"crowding":s.economy.crowding(s.stocks.stock[0]),"materials":&s.economy.housing[..2],"work":s.economy.housing[3],"worn":s.economy.housing_plan[1]})).collect::<Vec<_>>(),
            "storage_assets": self.sites.iter().map(|s| serde_json::json!({"site":s.id,"yard_kg":s.economy.storage[2],"warehouse_kg":s.economy.storage_capacity()-s.economy.storage[2],"materials_kg":&s.economy.storage[..2],"worn_kg":s.economy.storage_plan[1],"construction_work":s.economy.storage[3]})).collect::<Vec<_>>(),
            "incoming_dry_kg":self.sites.iter().map(|s|s.economy.logistics[1] as f64).sum::<f64>(),
            "funded_export_contracts":self.export_contracts.iter().filter(|c| c.escrow>0.).count(),
            "toolmaking": self.sites.iter().map(|s| serde_json::json!({"site":s.id,"expertise":s.economy.tool_craft[0],"completed_tool_work":s.economy.tool_craft[3],"monthly_work":s.economy.tool_work})).collect::<Vec<_>>(),
            "supplier_declines":self.export_contracts.iter().map(|c|c.declined as u64).sum::<u64>(),
            "quoted_export_surplus":self.export_contracts.iter().map(|c|c.quoted_surplus as f64).sum::<f64>(),
            "export_escrow":self.export_contracts.iter().map(|c|c.escrow as f64).sum::<f64>(),
            "contract_dispatched_kg":self.export_contracts.iter().map(|c|c.dispatched_kg as f64).sum::<f64>(),
            "workshop_materials_kg":(0..3).map(|k|self.sites.iter().map(|s|s.economy.workshop[k] as f64).sum::<f64>()).collect::<Vec<_>>(),
            "workshop_types": WORKSHOP_NAMES.iter().enumerate().map(|(j,name)| serde_json::json!({"name":name,"units":self.sites.iter().map(|s|s.economy.workshop_types[j][0] as f64).sum::<f64>(),"work":self.sites.iter().map(|s|s.economy.workshop_types[j][2] as f64).sum::<f64>()})).collect::<Vec<_>>(),
            "workshop_used_worker_months":self.sites.iter().map(|s|s.economy.workshop_plan[3] as f64).sum::<f64>(),
            "labor_worker_months":(0..4).map(|k|self.sites.iter().map(|s|s.economy.labor[k] as f64).sum::<f64>()).collect::<Vec<_>>(),
            "unused_craft_worker_months":self.sites.iter().map(|s|s.economy.logistics[2] as f64).sum::<f64>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utilized_workshop_repair_does_not_require_exhausting_household_capacity() {
        // A quarter worker-month completed in installed equipment warrants
        // maintaining 1/16 unit, even below the one-worker household fallback.
        assert_eq!(demonstrated_workshop_units(0.1, 0.25), 0.0625);
        assert_eq!(demonstrated_workshop_units(0.1, 0.), 0.);
        // Household overflow cannot justify pretending more equipment was used.
        assert_eq!(demonstrated_workshop_units(0.1, 4.), 0.1);
        assert_eq!(demonstrated_workshop_units(0., 4.), 0.);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn workshop_planning_repairs_used_assets_without_granting_resources() {
        let mut g = crate::gpu::Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                seed: 17,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let e = &mut h.sites[0].economy;
        e.workshop = [2., 3., 0.2, 1.]; // Declared fixture equipment, not a planner import.
        e.workshop_types = [[0.; 4]; 4];
        e.workshop_types[2] = [0.1, 0.1, 0.25, 0.];
        e.labor = [1000., 0., 0., 0.]; // Ample hypothetical household fallback.
        e.goods[5] = 0.; // Real outstanding brick demand.
        let goods = e.goods;
        let assets = e.workshop;
        let mut idle = h.clone();
        idle.sites[0].economy.workshop_types[2][2] = 0.;
        h.plan_production();
        idle.plan_production();
        let target = h.sites[0].economy.workshop_types[2][1];
        assert!((target - 0.09625).abs() < 1e-6, "{target}");
        assert!((idle.sites[0].economy.workshop_types[2][1] - 0.09).abs() < 1e-6);
        assert_eq!(h.sites[0].economy.goods, goods);
        assert_eq!(h.sites[0].economy.workshop, assets);
        assert_eq!(h.sites[0].economy.workshop_types[2][0], 0.1);
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        h.plan_production();
        resumed.plan_production();
        assert_eq!(
            h.sites[0].economy.workshop_types,
            resumed.sites[0].economy.workshop_types
        );
    }
    fn planner(c: &EconomyCatalog) -> Planner<'_> {
        Planner {
            catalog: c,
            available: [0.; GOODS],
            extractable: [0.; GOODS],
            targets: [0.; GOODS],
            orders: [0.; GOODS],
            visiting: [false; GOODS],
            knowledge: u32::MAX,
        }
    }
    #[test]
    fn harbor_orders_missing_materials_and_protects_construction_reserves() {
        let c = EconomyCatalog::bundled().unwrap();
        let port = crate::shipping::Port {
            fleet: None,
            work: None,
            site: 0,
            access: vec![],
            water_cell: 0,
            access_km: 0.,
            assets: [100., 0., 50.],
            commissioned: None,
            flood_months: 0,
        };
        let mut empty = planner(&c);
        empty.harbor(&port, 20.);
        assert_eq!(
            empty.targets[3], 20.,
            "ten working tools plus ten installed tools"
        );
        assert!(empty.orders.iter().sum::<f32>() > 0.);
        // Incoming cargo uses the same available array and suppresses duplicate recipes.
        let mut supplied = planner(&c);
        for (k, good) in crate::shipping::MATERIALS.into_iter().enumerate() {
            supplied.available[good] =
                crate::shipping::material_reserve(good, 20.) + port.material_deficit()[k];
        }
        supplied.harbor(&port, 20.);
        assert_eq!(supplied.orders, [0.; GOODS]);
        assert_eq!(
            port.assets,
            [100., 0., 50.],
            "planning cannot install materials"
        );
        let mut existing_reserve = planner(&c);
        existing_reserve.available[3] = 100.;
        existing_reserve.request(3, 15.);
        existing_reserve.harbor(&port, 20.);
        assert_eq!(
            existing_reserve.targets[3], 25.,
            "do not add the working reserve twice"
        );
    }
    #[test]
    fn containers_mix_supported_materials_without_overfilling_service() {
        let c = EconomyCatalog::bundled().unwrap();
        let prices = std::array::from_fn(|g| c.goods[g].base_price);
        let mut p = planner(&c);
        p.available[0] = 2.;
        p.available[2] = 100.;
        p.containers(&prices, 100.);
        assert!((p.targets[45] - 2.).abs() < 1e-5);
        assert!((p.targets[46] - 38.8).abs() < 1e-5);
        assert!((p.targets[45] * 3. + p.targets[46] * 5. - 200.).abs() < 1e-4);
        assert_eq!(p.targets[7], 0.);
        assert!(p.available[2] > 61.);
        let mut delivered = planner(&c);
        delivered.available[46] = 100.; // Also represents expected cargo in the caller.
        delivered.containers(&prices, 100.);
        assert_eq!(delivered.targets[46], 40.);
        assert_eq!(delivered.available[46], 60.);
        assert_eq!(delivered.orders, [0.; GOODS]);
        let mut forest = planner(&c);
        forest.extractable[0] = 1000.;
        forest.available[2] = 100.;
        forest.containers(&prices, 100.);
        assert_eq!(forest.targets[46], 0.);
        assert!((forest.targets[45] - 200. / 3.).abs() < 1e-4);
        let mut no_supply = planner(&c);
        no_supply.containers(&prices, 100.);
        assert_eq!(no_supply.targets[46], 0.);
        assert!(
            no_supply.targets[0] > 0.,
            "uncovered demand must still request upstream supplies"
        );
    }
    #[test]
    fn construction_quotes_follow_local_stock_and_tile_assembly() {
        let c = EconomyCatalog::bundled().unwrap();
        let e = crate::economy::Economy {
            prices: std::array::from_fn(|g| c.goods[g].base_price),
            ..Default::default()
        };
        let empty = planner(&c);
        assert!(crate::facilities::choose(&c, &empty.construction_quote(&e), 8., 10000.).is_none());
        let mut kiln = planner(&c);
        kiln.available[5] = 500.;
        let room = crate::facilities::choose(&c, &kiln.construction_quote(&e), 8., 10000.).unwrap();
        assert_eq!(room.components[0].good, 5);
        assert_eq!(room.components[1].good, 50);
        assert_eq!(
            e.goods[50], 0.,
            "a quote must not manufacture physical tiles"
        );
        for (g, kg) in room.materials() {
            kiln.request(g as usize, kg * 2.);
        }
        assert!(kiln.targets[50] > 0.);
        assert!(kiln.orders.iter().sum::<f32>() > 0.);
    }
    #[test]
    fn held_tool_substitutes_remain_targeted_without_duplicate_production() {
        let c = EconomyCatalog::bundled().unwrap();
        let mut p = planner(&c);
        p.available[3] = 2.;
        p.available[41] = 3.;
        p.available[43] = 20.;
        p.tools(3, 10., true);
        assert_eq!(p.targets[3], 2.);
        assert_eq!(p.targets[41], 3.);
        assert!((p.targets[43] * COPPER_TOOL_SERVICE_FACTOR - 5.).abs() < 1e-5);
        assert_eq!(p.orders, [0.; GOODS]);
        assert!(p.available[43] > 0., "excess tools remain tradable");
        let retained_service =
            p.targets[3] + p.targets[41] + p.targets[43] * COPPER_TOOL_SERVICE_FACTOR;
        assert!((retained_service - 10.).abs() < 1e-5);
        // A second independent use cannot claim the already reserved substitutes.
        let before = p.available[43];
        p.tools(3, 1., true);
        assert!((before - p.available[43] - 1. / COPPER_TOOL_SERVICE_FACTOR).abs() < 1e-5);
        let mut disabled = planner(&c);
        disabled.available[43] = 20.;
        disabled.tools(3, 10., false);
        assert_eq!(disabled.targets[43], 0.);
        assert_eq!(disabled.targets[3], 10.);
        assert!(disabled.orders.iter().sum::<f32>() > 0.);
        let mut short = planner(&c);
        short.available[43] = 2.;
        short.tools(3, 10., true);
        assert_eq!(short.targets[43], 2.);
        assert!((short.targets[3] - (10. - 2. * COPPER_TOOL_SERVICE_FACTOR)).abs() < 1e-5);
    }
    #[test]
    fn orders_follow_inputs_and_existing_deliveries_prevent_duplicate_work() {
        let c = EconomyCatalog::bundled().unwrap();
        let mut p = planner(&c);
        p.request(3, 10.);
        assert!(p.targets[2] >= 10. && p.targets[0] > 0. && p.targets[1] > 0.);
        assert!(p.orders.iter().sum::<f32>() > 0.);
        let mut stocked = planner(&c);
        stocked.available[3] = 10.;
        stocked.request(3, 10.);
        assert_eq!(stocked.orders, [0.; GOODS]);
        assert_eq!(stocked.targets[0], 0.);
    }
    #[test]
    fn closed_knowledge_gate_does_not_order_unused_upstream_goods() {
        let c = EconomyCatalog::bundled().unwrap();
        let mut p = planner(&c);
        p.knowledge = 0;
        p.request(18, 10.);
        assert_eq!(p.targets[13], 0.);
        assert_eq!(p.orders, [0.; GOODS]);
    }
    #[test]
    fn legacy_catalog_and_state_import_without_inventories() {
        let c = EconomyCatalog::bundled().unwrap();
        let mut prior = serde_json::to_value(&c).unwrap();
        prior["production"]
            .as_object_mut()
            .unwrap()
            .remove("diagnostic_fixed_labor");
        prior["production"]
            .as_object_mut()
            .unwrap()
            .remove("food_security_labor");
        prior["production"]
            .as_object_mut()
            .unwrap()
            .remove("food_security_maintenance");
        prior["production"]
            .as_object_mut()
            .unwrap()
            .remove("adaptive_labor");
        prior["production"]
            .as_object_mut()
            .unwrap()
            .remove("workshops");
        prior["production"]
            .as_object_mut()
            .unwrap()
            .remove("export_contracts");
        prior["production"]
            .as_object_mut()
            .unwrap()
            .remove("specialized_workshops");
        prior["production"]
            .as_object_mut()
            .unwrap()
            .remove("persistent_storage");
        let prior: EconomyCatalog = serde_json::from_value(prior).unwrap();
        assert!(!prior.production.persistent_storage);
        assert!(!prior.production.diagnostic_fixed_labor);
        assert!(!prior.production.food_security_labor);
        assert!(prior.production.food_security_maintenance);
        assert!(!prior.production.replacement_tool_jobs);
        assert!(!prior.production.toolmaking_expertise);
        let mut old_housing = serde_json::to_value(&prior).unwrap();
        old_housing["production"]
            .as_object_mut()
            .unwrap()
            .remove("persistent_housing");
        let old_housing: EconomyCatalog = serde_json::from_value(old_housing).unwrap();
        assert!(!old_housing.production.persistent_housing);
        assert!(!prior.production.specialized_workshops);
        assert!(!prior.production.export_contracts);
        assert!(!prior.production.workshops);
        assert!(prior.production.enabled && !prior.production.adaptive_labor);
        let mut json = serde_json::to_value(c).unwrap();
        json.as_object_mut().unwrap().remove("production");
        let old: EconomyCatalog = serde_json::from_value(json).unwrap();
        assert!(!old.production.enabled);
        let mut invalid = old.clone();
        invalid.production.diagnostic_fixed_labor = true;
        assert!(invalid.validate().is_err());
        let mut food = EconomyCatalog::bundled().unwrap();
        food.production.food_security_labor = true;
        assert!(food.validate().is_ok());
        food.production.diagnostic_fixed_labor = true;
        assert!(food.validate().is_err());
        food.production.diagnostic_fixed_labor = false;
        food.production.adaptive_labor = false;
        assert!(food.validate().is_err());
        let e = crate::economy::Economy::default();
        let mut json = serde_json::to_value(e).unwrap();
        for field in [
            "food_labor",
            "tool_craft",
            "tool_work",
            "tool_orders",
            "storage",
            "storage_plan",
            "targets",
            "orders",
            "logistics",
            "workshop",
            "workshop_plan",
            "workshop_types",
        ] {
            json.as_object_mut().unwrap().remove(field);
        }
        let old: crate::economy::Economy = serde_json::from_value(json).unwrap();
        assert_eq!(old.goods, [0.; GOODS]);
        assert_eq!(old.logistics, [0.; 4]);
    }
}

#[cfg(test)]
mod mineral_tests {
    use super::*;
    #[test]
    fn recovered_ore_demand_requires_a_supported_tool_chain() {
        let mut c = EconomyCatalog::bundled().unwrap();
        c.add_alloy_chains(&crate::catalog::Catalog::bundled().unwrap())
            .unwrap();
        let mut e = crate::economy::Economy::default();
        e.extraction[1] = 1.;
        e.logistics[3] = 1.;
        e.management[3] = 4095.;
        e.goods[3] = 70.;
        e.goods[6] = 100.;
        assert_eq!(
            recovery_tool_input_demand(&c, &e, 100., [0.; GOODS], 36, 100.),
            0.
        );
        e.workshop_types[1][2] = 2.;
        let demand = recovery_tool_input_demand(&c, &e, 100., [0.; GOODS], 36, 100.);
        // Malachite -> 0.456 copper per kg -> copper tools at 0.6 service/kg.
        assert!((demand - 5. / (0.456 * 0.6)).abs() < 0.001, "{demand}");
        let mut expected = [0.; GOODS];
        expected[36] = demand;
        assert!(recovery_tool_input_demand(&c, &e, 100., expected, 36, 100.) < 0.001);
        assert_eq!(e.goods[36], 0.); // proposal never credits inventory
        e.goods[38] = 20.;
        assert_eq!(
            recovery_tool_input_demand(&c, &e, 100., [0.; GOODS], 36, 100.),
            0.
        );
        e.goods[38] = 0.;
        e.goods[6] = 0.;
        assert_eq!(
            recovery_tool_input_demand(&c, &e, 100., [0.; GOODS], 36, 100.),
            0.
        );
        e.goods[6] = 100.;
        e.goods[3] = 75.;
        assert_eq!(
            recovery_tool_input_demand(&c, &e, 100., [0.; GOODS], 36, 100.),
            0.
        );
        e.goods[3] = 70.;
        e.extraction[1] = 0.;
        assert_eq!(
            recovery_tool_input_demand(&c, &e, 100., [0.; GOODS], 36, 100.),
            0.
        );
    }

    #[test]
    fn imported_copper_ore_can_change_a_non_copper_towns_tool_plan() {
        let mut c = EconomyCatalog::bundled().unwrap();
        c.add_alloy_chains(&crate::catalog::Catalog::bundled().unwrap())
            .unwrap();
        let mut p = Planner {
            catalog: &c,
            available: [0.; GOODS],
            extractable: [0.; GOODS],
            targets: [0.; GOODS],
            orders: [0.; GOODS],
            visiting: [false; GOODS],
            knowledge: u32::MAX,
        };
        p.available[36] = 10.;
        p.available[6] = 100.;
        p.tools(3, 10., true);
        assert!((p.targets[43] - 4.56).abs() < 0.001);
        assert!((p.targets[3] - (10. - 4.56 * 0.6)).abs() < 0.001);
        assert!((p.targets[36] - 10.).abs() < 0.001);
        assert!(p.available[36] < 0.001);
    }

    #[test]
    fn locally_extractable_ore_changes_recipe_without_becoming_inventory() {
        let mut c = EconomyCatalog::bundled().unwrap();
        let mut recipe = crate::economy::Recipe {
            input: [0.; GOODS],
            output: [0.; GOODS],
            work: [0.12, 0., 1., 0.],
        };
        recipe.input[32] = 1.;
        recipe.input[6] = 0.5;
        recipe.output[2] = 0.56;
        c.recipes.push(recipe);
        let mut p = Planner {
            catalog: &c,
            available: [0.; GOODS],
            extractable: [0.; GOODS],
            targets: [0.; GOODS],
            orders: [0.; GOODS],
            visiting: [false; GOODS],
            knowledge: u32::MAX,
        };
        p.extractable[32] = 10.;
        p.request(2, 5.6);
        assert!((p.orders[c.recipes.len() - 1] - 10.).abs() < 1e-5);
        assert!((p.targets[32] - 10.).abs() < 1e-5);
        assert_eq!(p.available[32], 0.);
        assert_eq!(p.targets[1], 0.);
    }
}
