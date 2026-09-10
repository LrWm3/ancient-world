//! Sparse town work orders; physical production and inventory transfers execute on GPU.
use crate::{
    civilization::History,
    economy::{EconomyCatalog, FOOD, GOODS},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ProductionSettings {
    pub enabled: bool,
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
            initial_housing_per_person: 1.1,
            specialized_workshops: false,
            export_contracts: false,
            supplier_profitability: false,
            contract_margin: 0.1,
            storage_kg_per_person: 100.,
            land_freight_kg_per_person: 20.,
        }
    }
}
pub const WORKSHOP_NAMES: [&str; 4] = [
    "General crafts",
    "Metalworking",
    "Kilns",
    "Textiles and leather",
];
pub fn empty_slots() -> [f32; GOODS] {
    [0.; GOODS]
}
/// Useful reserve levels, not a quota to own every possible catalog good.
pub fn reserve(id: &str) -> f32 {
    match id {
        "wood" => 4.,
        "tools" => 0.75,
        "bricks" => 3.,
        "pottery" => 2.,
        "cloth" => 0.8,
        "leather" => 0.2,
        "writing_material" => 0.1,
        "weapons" => 0.2,
        "armor" => 0.4,
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
    fn request(&mut self, good: usize, quantity: f32) {
        if quantity <= 0. || self.visiting[good] {
            return;
        }
        self.targets[good] += quantity;
        let from_stock = self.available[good].min(quantity);
        self.available[good] -= from_stock;
        let missing = quantity - from_stock;
        if missing < 0.001 {
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
        for s in &mut self.sites {
            let e = &mut s.economy;
            e.logistics = [0.; 4];
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
                let age_need = s.demography.ages[0] * 10.
                    + s.demography.ages[1] * 18.
                    + s.demography.ages[2] * 14.;
                let need = if age_need > 0. { age_need } else { pop * 18. };
                let months = s.stocks.stock[1].max(0.) / need;
                let shortage = if s.demography.ration_need[3] > 0. {
                    1. - s.demography.ration_eaten[3] / s.demography.ration_need[3]
                } else {
                    s.stocks.stock[3]
                };
                let tools = e.goods[3]
                    + if e.extraction[1] > 0.5 {
                        e.goods[41] + 0.6 * e.goods[43]
                    } else {
                        0.
                    };
                e.food_labor[1] = shortage.clamp(0., 1.).max((1. - months / 3.).clamp(0., 1.));
                e.food_labor[2] = months;
                e.food_labor[3] = (1. - tools / (pop * 0.5)).clamp(0., 1.);
            }
            let mut planner = Planner {
                catalog,
                available: e.goods,
                extractable: {
                    let mut supply = [0.; GOODS];
                    supply[e.extraction[0].max(1.) as usize] = e.reserves[1];
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
                        for (good, mass) in f.repair_order(e, n.treasury, 0.1) {
                            planner.request(good as usize, mass);
                        }
                        if f.remaining() == 0. && f.condition() >= 0.8 && target - f.planned() >= 2.
                        {
                            let mut quoted = *e;
                            quoted.goods.fill(1_000_000.);
                            if let Some(room) = crate::facilities::choose(
                                catalog,
                                &quoted,
                                target - f.planned(),
                                crate::facilities::expansion_budget(f, e, n.treasury),
                            ) {
                                for (good, mass) in room.materials() {
                                    planner.request(good as usize, mass * 2.);
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
                        || planner.available[39] >= 0.1
                        || planner.available[40] > 0.
                    {
                        41
                    } else {
                        43
                    }
                } else {
                    3
                };
                let covered: f32 = [(3, 1.), (41, 1.), (43, 0.6)]
                    .into_iter()
                    .filter(|(k, _)| *k != selected && (*k == 3 || e.extraction[1] > 0.5))
                    .map(|(k, efficiency)| planner.available[k] * efficiency)
                    .sum();
                replacement.request(
                    selected,
                    (pop * 0.5 - covered).max(0.) / if selected == 43 { 0.6 } else { 1. },
                );
            }
            if let Some(methods) = &catalog.materials {
                // Containers substitute by useful storage, not equal mass. Existing variants count.
                let held = planner.available[7]
                    + methods
                        .variants
                        .iter()
                        .filter(|v| v.role == "container")
                        .map(|v| planner.available[v.slot] * v.service)
                        .sum::<f32>();
                let selected = methods
                    .variants
                    .iter()
                    .filter(|v| v.role == "container")
                    .map(|v| (v.slot, v.service, v.wear))
                    .chain(std::iter::once((7, 1., 0.005)))
                    .min_by(|a, b| {
                        let cost = |x: &(usize, f32, f32)| {
                            e.prices[x.0].max(0.01) / x.1 * (1. + x.2 * 120.)
                        };
                        cost(a).total_cmp(&cost(b))
                    });
                if let Some((k, service, _)) = selected {
                    planner.request(
                        k,
                        planner.available[k] + (pop * 2. - held).max(0.) / service,
                    );
                }
                for v in &methods.variants {
                    let need = match v.role.as_str() {
                        "digging" | "breaking" => {
                            if e.reserves[1] + e.reserves[2] > 0. {
                                pop * 0.025
                            } else {
                                0.
                            }
                        }
                        "cutting" => {
                            if e.forest[0] > 0. {
                                pop * 0.025
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
                            || planner.available[39] >= 0.1
                            || planner.available[40] > 0.
                        {
                            41
                        } else {
                            43
                        }
                    } else {
                        k
                    };
                    let other: [(usize, f32); 3] = [(k, 1.), (41, 1.), (43, 0.6)];
                    let covered: f32 = other
                        .iter()
                        .filter(|(i, _)| *i != selected)
                        .map(|(i, eff)| planner.available[*i] * eff)
                        .sum();
                    planner.request(
                        selected,
                        (desired - covered).max(0.) / if selected == 43 { 0.6 } else { 1. },
                    );
                } else {
                    planner.request(k, pop * reserve(&g.id));
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
                let samples = w.samples.iter().sum::<f64>().min(12.) as f32;
                planner.request(6, samples * 0.2);
                planner.request(3, samples * 0.1);
            }
            // Feed reserves compete with food; preserve enough seed for establishment.
            if let Some(a) = &catalog.agriculture {
                for (j, herd) in a.herds.iter().enumerate() {
                    if let Some(k) = catalog.index(&herd.feed) {
                        planner.request(k, e.herds[j][0] * 0.08 * 6.);
                    }
                }
                for crop in &a.crops {
                    if let Some(k) = catalog.index(&crop.good) {
                        planner.targets[k] += pop * 0.04 + 2.;
                    }
                }
            }
            if e.fishery[3] > 0.5 && e.management[1] >= 1. {
                let traps = if e.fishery_traps[3] > 0.5 {
                    e.fishery_traps[0] * 0.997 / 20.
                } else {
                    0.
                };
                if e.fishery_traps[3] > 0.5 && (e.goods[16] < 4. || e.goods[3] < pop * 0.15 + 1.) {
                    // Request the feasible alternative, not two complete sets of gear.
                    let equipped = (e.fishery[0] / 40.)
                        .min(e.fishery[1])
                        .min(e.fishery[2] / 4.)
                        * 0.997;
                    let missing = (e.fishery_plan[0] - traps - equipped).max(0.) * 20.;
                    planner.request(0, missing.min(pop * 20. * 0.015));
                } else {
                    for (j, good) in [0, 3, 16].into_iter().enumerate() {
                        let cost = [40., 1., 4.][j];
                        let crew = (e.fishery_plan[0] - traps).max(0.);
                        let missing = (crew * cost - e.fishery[j] * 0.997).max(0.);
                        planner.request(good, missing.min(pop * cost * 0.015));
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
                let household = (e.labor.iter().sum::<f32>() * 0.025).max(1.);
                let demonstrated = e.workshop_plan[3] * 1.25;
                let desired =
                    ((industrial_work.min(demonstrated) - household).max(0.) / 4.).min(pop * 0.03);
                e.workshop_plan[0] = e.workshop_plan[0] * 0.9 + desired * 0.1;
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
                        *w = w.min(e.workshop_types[j][2] * 1.25);
                    }
                    let total: f32 = work.iter().sum();
                    let desired = ((total - household).max(0.) / 4.).min(pop * 0.03);
                    for (j, w) in work.iter().enumerate() {
                        e.workshop_types[j][1] =
                            e.workshop_types[j][1] * 0.9 + desired * w / total.max(0.001) * 0.1;
                    }
                    // Existing buildings retain their specialization; idle capacity cannot
                    // impersonate another industry's equipment.
                    let missing: f32 = e
                        .workshop_types
                        .iter()
                        .map(|t| (t[1] - t[0] * 0.998).max(0.))
                        .sum();
                    let units = (e.workshop[0] / 20.)
                        .min(e.workshop[1] / 30.)
                        .min(e.workshop[2] / 2.);
                    let assigned: f32 = e.workshop_types.iter().map(|t| t[0]).sum();
                    e.workshop_plan[0] = (units * 0.998
                        + (missing - (units - assigned).max(0.) * 0.998).max(0.))
                    .min(pop * 0.03);
                }
                for (j, good) in [0, 5, 3].into_iter().enumerate() {
                    let cost = [20., 30., 2.][j];
                    let missing = (e.workshop_plan[0] * cost - e.workshop[j]).max(0.);
                    planner.request(good, missing.min(pop * cost * 0.003));
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
                let desired = (stored * 1.25)
                    .min((pop * catalog.production.storage_kg_per_person * 2.).max(e.storage[2]));
                e.storage_plan[0] = (desired - e.storage[2]).max(0.);
                for (j, good) in [0, 5].into_iter().enumerate() {
                    let cost = [0.02, 0.03][j];
                    planner.request(
                        good,
                        (e.storage_plan[0] * cost - e.storage[j] * 0.999)
                            .max(0.)
                            .min(pop * cost * 10.),
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
                e.housing_plan[0] = ((pop + pending) * 1.1 - e.housing[2]).max(0.);
                for (j, good) in [0, 5].into_iter().enumerate() {
                    let cost = [2., 3.][j];
                    planner.request(
                        good,
                        (e.housing_plan[0] * cost - e.housing[j] * 0.999)
                            .max(0.)
                            .min(pop * cost * 0.02),
                    );
                }
            }
            if e.waterworks[3] > 0.5 {
                e.waterworks_plan[0] = pop * catalog.production.waterworks_target_fraction;
                for (j, good) in [0, 5].into_iter().enumerate() {
                    let cost = [2., 4.][j];
                    planner.request(
                        good,
                        (e.waterworks_plan[0] * cost - e.waterworks[j] * 0.999)
                            .max(0.)
                            .min(pop * cost * 0.02),
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
            for (j, condition) in [coverage < 0.1, coverage >= 0.5].into_iter().enumerate() {
                s.lifecycle.waterworks_months[j] = if condition {
                    s.lifecycle.waterworks_months[j].saturating_add(1).min(3)
                } else {
                    0
                };
            }
            let kind = match s.lifecycle.waterworks_operating {
                None if coverage >= 0.25 => Some("waterworks_established"),
                Some(true) if s.lifecycle.waterworks_months[0] >= 3 => Some("waterworks_disrupted"),
                Some(false) if s.lifecycle.waterworks_months[1] >= 3 => {
                    Some("waterworks_recovered")
                }
                _ => None,
            };
            if let Some(kind) = kind {
                s.lifecycle.waterworks_operating = Some(coverage >= 0.25);
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
                Some(v) if capacity > v * 1.1 && capacity - v > 2. => Some("housing_expanded"),
                Some(v) if capacity < v * 0.9 && v - capacity > 2. => Some("housing_deteriorated"),
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
                Some(v) if v < 100. && capacity >= 100. => Some("warehouse_established"),
                Some(v) if capacity > v * 1.25 && capacity - v >= 100. => {
                    Some("warehouse_expanded")
                }
                Some(v) if capacity < v * 0.75 && v - capacity >= 100. => {
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
