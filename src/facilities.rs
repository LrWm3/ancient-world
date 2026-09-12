//! Incremental institutional rooms. Components remain in the owning artifact ledger.
use crate::{
    culture::{Artifact, InstitutionKind},
    economy::{Economy, EconomyCatalog},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Component {
    pub good: u32,
    pub kg: f32,
    pub condition: f32,
    pub wear: f32,
    pub work: f32,
}
impl Component {
    /// Catalog wear is also a bounded resilience proxy under local disruption.
    /// This is a game rule: roof loads, fire resistance and rot are not separately solved.
    pub fn wear_at(&self, disruption: f32) -> f32 {
        (self.wear * (1. + 10. * disruption.clamp(0., 1.))).min(1.)
    }
}
/// Repair inputs are measured after the currently due administration fee.
/// Keep one subsequent fee in the existing account; this is not a new cash stock.
pub(crate) fn repair_budget(treasury: f64) -> f64 {
    (treasury - 0.5).max(0.)
}
fn affordable_mass(cash: f64, price: f64) -> f32 {
    let bound = cash / price;
    let mut mass = bound as f32;
    if mass as f64 > bound {
        mass = f32::from_bits(mass.to_bits().saturating_sub(1));
    }
    mass
}
const INVESTMENT_QUARTERS: f64 = 80.;
const WORK_VALUE: f64 = 20.;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub method: String,
    pub capacity: f32,
    pub remaining_work: f32,
    pub components: Vec<Component>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Facility {
    pub rooms: Vec<Room>,
    pub invested: f64,
    pub construction_work: f64,
}
impl Room {
    pub fn materials(&self) -> Vec<(u32, f32)> {
        let mut map = std::collections::BTreeMap::new();
        for c in &self.components {
            *map.entry(c.good).or_insert(0.) += c.kg;
        }
        map.into_iter().collect()
    }
    pub fn cost(&self, e: &Economy) -> f64 {
        self.materials()
            .iter()
            .map(|&(g, m)| m as f64 * e.prices[g as usize].max(0.01) as f64)
            .sum()
    }
    pub fn usable(&self) -> f32 {
        if self.remaining_work > 0. {
            0.
        } else {
            self.capacity
                * self
                    .components
                    .iter()
                    .map(|c| c.condition)
                    .fold(1., f32::min)
        }
    }
}
pub fn demand(kind: &InstitutionKind, members: usize) -> f32 {
    let factor = match kind {
        InstitutionKind::Religious => 1.5,
        InstitutionKind::Merchant => 2.,
        InstitutionKind::Craft => 1.25,
        InstitutionKind::Scholarly => 1.,
    };
    (members as f32 * factor).clamp(2., 64.)
}
/// Keep ten cash for administration plus a year of component upkeep in cash.
/// Investment uses three quarters of the remainder; quotations and purchases share this limit.
pub fn expansion_budget(f: &Facility, e: &Economy, treasury: f64) -> f64 {
    let upkeep: f64 = f
        .rooms
        .iter()
        .flat_map(|r| &r.components)
        .map(|p| {
            p.kg as f64
                * p.wear_at(e.soil[3]) as f64
                * 4.
                * e.prices[p.good as usize].max(0.01) as f64
        })
        .sum();
    (treasury - 10. - upkeep).max(0.) * 0.75
}
/// Select one affordable extension, preserving a finite stock reserve. No planned goods count as stock.
pub fn choose(c: &EconomyCatalog, e: &Economy, target: f32, budget: f64) -> Option<Room> {
    let methods = c.materials.as_ref()?;
    let mut best: Option<(f64, Room)> = None;
    for wall in &methods.methods {
        for roof in &methods.roofs {
            let wg = c.index(&wall.wall)? as u32;
            let rg = c.index(&roof.good)? as u32;
            let mut per = std::collections::BTreeMap::<u32, f32>::new();
            *per.entry(wg).or_default() += wall.wall_kg;
            *per.entry(rg).or_default() += roof.kg;
            let unit_cost: f64 = per
                .iter()
                .map(|(&g, &m)| m as f64 * e.prices[g as usize].max(0.01) as f64)
                .sum();
            let mut units = target.min(16.).min((budget / unit_cost) as f32);
            for (&g, &kg) in &per {
                units = units.min(e.goods[g as usize] * 0.5 / kg);
            }
            units = units.floor();
            if units < 2. {
                continue;
            }
            let room = Room {
                method: format!("{} + {} roof", wall.id, roof.good),
                capacity: units,
                remaining_work: units * (wall.work + roof.work),
                components: vec![
                    Component {
                        good: wg,
                        kg: wall.wall_kg * units,
                        condition: 1.,
                        wear: wall.wear,
                        work: wall.work * units,
                    },
                    Component {
                        good: rg,
                        kg: roof.kg * units,
                        condition: 1.,
                        wear: roof.wear,
                        work: roof.work * units,
                    },
                ],
            };
            // Quote each component's own replacement cost and work. Roof wear must not
            // charge replacement of an intact wall. Current exposure is a planning scenario.
            let lifetime_per_unit: f64 = room
                .components
                .iter()
                .map(|p| {
                    let material = p.kg as f64 * e.prices[p.good as usize].max(0.01) as f64;
                    (material + WORK_VALUE * p.work as f64)
                        * (1. + INVESTMENT_QUARTERS * p.wear_at(e.soil[3]) as f64)
                        / units as f64
                })
                .sum();
            let score = units as f64 / lifetime_per_unit;
            if best.as_ref().is_none_or(|(s, _)| score > *s) {
                best = Some((score, room));
            }
        }
    }
    best.map(|(_, r)| r)
}
/// Exact float32 stock withdrawal, rounded toward retaining inventory.
fn take(e: &mut Economy, g: usize, wanted: f32) -> f32 {
    let old = e.goods[g];
    let mut left = (old as f64 - wanted as f64).max(0.) as f32;
    if old as f64 - left as f64 > wanted as f64 {
        left = f32::from_bits(left.to_bits() + 1).min(old);
    }
    e.goods[g] = left;
    old - left
}
/// Money remains in existing site/institution accounts; no construction capital is minted.
fn pay(e: &mut Economy, treasury: &mut f64, wanted: f64) -> f64 {
    let old = e.finance[0];
    let mut next = (old as f64 + wanted.min(*treasury)) as f32;
    if next as f64 - old as f64 > wanted.min(*treasury) {
        next = f32::from_bits(next.to_bits().saturating_sub(1)).max(old);
    }
    let paid = next as f64 - old as f64;
    e.finance[0] = next;
    *treasury -= paid;
    paid
}
impl Facility {
    pub fn usable(&self) -> f32 {
        self.rooms.iter().map(Room::usable).sum()
    }
    pub fn planned(&self) -> f32 {
        self.rooms.iter().map(|r| r.capacity).sum()
    }
    pub fn remaining(&self) -> f32 {
        self.rooms.iter().map(|r| r.remaining_work).sum()
    }
    pub fn condition(&self) -> f32 {
        let completed: f32 = self
            .rooms
            .iter()
            .filter(|r| r.remaining_work == 0.)
            .map(|r| r.capacity)
            .sum();
        if completed > 0. {
            self.usable() / completed
        } else {
            0.
        }
    }
    /// Founding materials are a declared communal gift, already withdrawn once.
    pub fn found(mut room: Room, e: &mut Economy) -> Self {
        let materials = room.materials();
        for (g, m) in materials {
            let actual = take(e, g as usize, m);
            for c in room.components.iter_mut().filter(|c| c.good == g) {
                c.kg *= actual / m;
            }
        }
        let built = room.remaining_work.min(0.2);
        room.remaining_work -= built;
        Self {
            rooms: vec![room],
            invested: 0.,
            construction_work: built as f64,
        }
    }
    /// Next quarter's bounded repairs, quoted against one shared cash/work allowance.
    /// Treasury is after the currently due fee. This requests supplies only;
    /// actual repairs still withdraw and pay at the boundary.
    pub fn repair_order(&self, e: &Economy, treasury: f64, work: f32) -> Vec<(u32, f32)> {
        let mut cash = repair_budget(treasury);
        let mut labor = work.max(0.);
        let mut goods = std::collections::BTreeMap::new();
        for r in &self.rooms {
            if r.remaining_work > 0. {
                labor = (labor - r.remaining_work).max(0.);
                continue;
            }
            for p in &r.components {
                let price = e.prices[p.good as usize].max(0.01) as f64;
                let damage = p.wear_at(e.soil[3]);
                let repair = (1. - p.condition + damage)
                    .clamp(0., 0.1)
                    .min(labor / p.work)
                    .min((cash / (p.kg as f64 * price)) as f32);
                let mass = repair * p.kg;
                *goods.entry(p.good).or_insert(0.) += mass;
                cash = (cash - mass as f64 * price).max(0.);
                labor = (labor - repair * p.work).max(0.);
            }
        }
        goods.into_iter().filter(|(_, mass)| *mass > 0.).collect()
    }
    pub fn embodied(&self) -> Vec<(u32, f32)> {
        let mut map = std::collections::BTreeMap::new();
        for r in &self.rooms {
            for c in &r.components {
                *map.entry(c.good).or_insert(0.) += c.kg;
            }
        }
        map.into_iter().collect()
    }
    pub fn valid(&self, c: &EconomyCatalog, a: &Artifact) -> bool {
        !self.rooms.is_empty()
            && self.rooms.len() <= 32
            && self.invested.is_finite()
            && self.invested >= 0.
            && self.construction_work.is_finite()
            && self.construction_work >= 0.
            && self.rooms.iter().all(|r| {
                r.capacity.is_finite()
                    && (2. ..=16.).contains(&r.capacity)
                    && r.remaining_work.is_finite()
                    && r.remaining_work >= 0.
                    && r.remaining_work <= r.components.iter().map(|p| p.work).sum::<f32>()
                    && r.components.len() == 2
                    && r.components.iter().all(|p| {
                        (p.good as usize) < c.goods.len()
                            && matches!(
                                c.goods[p.good as usize].id.as_str(),
                                "wood" | "bricks" | "metal" | "roof_tiles"
                            )
                            && p.kg.is_finite()
                            && p.kg > 0.
                            && (0. ..=1.).contains(&p.condition)
                            && (0. ..=1.).contains(&p.wear)
                            && p.work.is_finite()
                            && p.work > 0.
                    })
            })
            && self.embodied() == a.materials
    }
    /// Returns work spent, repair mass, actual expenditure, rooms completed, and expansion status.
    pub fn advance(
        &mut self,
        e: &mut Economy,
        c: &EconomyCatalog,
        treasury: &mut f64,
        work: f32,
        target: f32,
    ) -> (f32, f64, f64, usize, bool) {
        let mut available = work;
        let mut repaired = 0.;
        let mut spent = 0.;
        let mut completed = 0;
        for r in &mut self.rooms {
            if r.remaining_work > 0. {
                let built = available.min(r.remaining_work);
                r.remaining_work = (r.remaining_work - built).max(0.);
                available -= built;
                self.construction_work += built as f64;
                if r.remaining_work == 0. {
                    completed += 1;
                }
                continue;
            }
            for p in &mut r.components {
                p.condition = (p.condition - p.wear_at(e.soil[3])).max(0.);
                let g = p.good as usize;
                let price = e.prices[g].max(0.01) as f64;
                let improvement = (1. - p.condition)
                    .min(0.1)
                    .min(available / p.work)
                    .min(e.goods[g] / p.kg)
                    .min((repair_budget(*treasury) / (p.kg as f64 * price)) as f32);
                let mass = take(
                    e,
                    g,
                    (p.kg * improvement).min(affordable_mass(repair_budget(*treasury), price)),
                );
                let actual = mass / p.kg;
                let payment = pay(e, treasury, mass as f64 * price);
                spent += payment;
                repaired += mass as f64;
                p.condition = (p.condition + actual).min(1.);
                available = (available - actual * p.work).max(0.);
                e.used[g] += mass;
                e.reserves[3] += mass;
                for (x, ratio) in e.detritus.iter_mut().zip(c.composition(g)) {
                    *x += mass * ratio;
                }
            }
        }
        let mut expanded = false;
        // Protect operating reserves; finish and repair existing rooms before expansion.
        if available >= 0.025
            && self.remaining() == 0.
            && self.condition() >= 0.8
            && target - self.planned() >= 2.
            && self.rooms.len() < 32
        {
            if let Some(mut room) = choose(
                c,
                e,
                target - self.planned(),
                expansion_budget(self, e, *treasury),
            ) {
                let cost = room.cost(e);
                let paid = pay(e, treasury, cost);
                spent += paid;
                self.invested += paid;
                for (g, m) in room.materials() {
                    let actual = take(e, g as usize, m);
                    for p in room.components.iter_mut().filter(|p| p.good == g) {
                        p.kg *= actual / m;
                    }
                }
                self.rooms.push(room);
                expanded = true;
            }
        }
        (work - available, repaired, spent, completed, expanded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stocks() -> Economy {
        let mut e = Economy::default();
        e.goods.fill(10000.);
        e.prices.fill(2.);
        e.finance[0] = 1000.;
        e
    }
    #[test]
    fn durable_roofs_pay_back_under_sustained_exposure_but_not_in_shelter() {
        let c = EconomyCatalog::bundled().unwrap();
        let mut e = stocks();
        e.prices = std::array::from_fn(|g| c.goods[g].base_price);
        e.prices[0] = 4.5;
        e.goods[2] = 0.; // Compare stocked wood/ceramic alternatives, without a metal supplier.
        let dry = choose(&c, &e, 4., 10000.).unwrap();
        assert_eq!(dry.components[1].good, 0);
        e.soil[3] = 1.;
        let wet = choose(&c, &e, 4., 10000.).unwrap();
        assert_eq!(wet.components[0].good, dry.components[0].good);
        assert_eq!(wet.components[1].good, 50);
        let cost = [dry.cost(&e), wet.cost(&e)];
        assert!(cost[1] > cost[0], "durability has an upfront tradeoff");
        let mut outcome = Vec::new();
        for room in [dry, wet] {
            let mut town = e;
            let mut f = Facility::found(room, &mut town);
            let mut treasury = 10000.;
            let cash = treasury + town.finance[0] as f64;
            f.advance(&mut town, &c, &mut treasury, f.remaining(), 0.);
            let start_waste = town.reserves[3];
            let mut replaced = 0.;
            for _ in 0..80 {
                let before_goods: f64 = town.goods.iter().map(|v| *v as f64).sum();
                let (_, mass, _, _, _) = f.advance(&mut town, &c, &mut treasury, 0.1, 0.);
                let after_goods: f64 = town.goods.iter().map(|v| *v as f64).sum();
                assert!((before_goods - after_goods - mass).abs() < 0.001);
                assert_eq!(cash, treasury + town.finance[0] as f64);
                replaced += mass;
            }
            assert!((town.reserves[3] as f64 - start_waste as f64 - replaced).abs() < 0.01);
            assert!(f.condition() > 0.99);
            outcome.push(10000. - treasury);
        }
        eprintln!(
            "wood/tile upfront {:?}, 80-quarter upkeep {:?}",
            cost, outcome
        );
        assert!(cost[1] + outcome[1] < cost[0] + outcome[0]);
    }
    #[test]
    fn stocked_alternatives_respond_to_local_material_prices() {
        let c = EconomyCatalog::bundled().unwrap();
        for (cheap, roof, wall_name) in [
            (0, 0, "timber_room"),
            (2, 2, "metal_frame_room"),
            (5, 50, "masonry_room"),
        ] {
            let mut e = stocks();
            e.prices.fill(40.);
            e.prices[cheap] = 0.5;
            e.prices[roof] = 0.5;
            let room = choose(&c, &e, 8., 10000.).unwrap();
            assert!(room.method.starts_with(wall_name), "{}", room.method);
            assert_eq!(room.components[1].good as usize, roof);
        }
    }
    #[test]
    fn surplus_stock_cannot_consume_next_administration_fee() {
        let c = EconomyCatalog::bundled().unwrap();
        for opening in [0., 0.25, 0.5, 0.75, 5.] {
            let mut e = stocks();
            let mut f = Facility::found(choose(&c, &e, 4., 1000.).unwrap(), &mut e);
            for r in &mut f.rooms {
                r.remaining_work = 0.;
                for p in &mut r.components {
                    p.condition = 0.5;
                }
            }
            let embodied = f.embodied();
            let before: f64 = e.goods.iter().map(|&v| v as f64).sum();
            let money = e.finance[0] as f64 + opening;
            let mut cash = opening;
            let (_, mass, paid, _, expanded) = f.advance(&mut e, &c, &mut cash, 0.1, 4.);
            assert!(
                cash >= opening.min(0.5),
                "opening={opening}, remaining={cash}"
            );
            assert_eq!(money, e.finance[0] as f64 + cash);
            assert_eq!(paid, opening - cash);
            assert!(!expanded);
            assert_eq!(embodied, f.embodied());
            assert!((before - e.goods.iter().map(|&v| v as f64).sum::<f64>() - mass).abs() < 1e-5);
            if opening <= 0.5 {
                assert_eq!(mass, 0.);
            } else {
                assert!(mass > 0.);
            }
            // Existing reserves can fund the following fee without any new income.
            let fee = pay(&mut e, &mut cash, 0.5);
            assert_eq!(fee, opening.min(0.5));
            assert_eq!(money, e.finance[0] as f64 + cash);
        }
    }
    #[test]
    fn repair_orders_and_expansion_share_finite_operating_reserves() {
        let c = EconomyCatalog::bundled().unwrap();
        let mut e = stocks();
        let mut f = Facility::found(choose(&c, &e, 4., 1000.).unwrap(), &mut e);
        for r in &mut f.rooms {
            r.remaining_work = 0.;
            for p in &mut r.components {
                p.condition = 0.9;
            }
        }
        assert!(f.repair_order(&e, 0., 0.1).is_empty());
        assert!(f.repair_order(&e, 100., 0.).is_empty());
        let orders = f.repair_order(&e, 5., 0.1);
        let bill: f64 = orders
            .iter()
            .map(|&(g, m)| m as f64 * e.prices[g as usize] as f64)
            .sum();
        assert!(bill > 0. && bill <= 4.500001);
        // Receiving exactly the quoted goods permits the corresponding repair, never more.
        e.goods.fill(0.);
        for &(g, m) in &orders {
            e.goods[g as usize] = m;
        }
        let mut cash = 5.;
        let result = f.advance(&mut e, &c, &mut cash, 0.1, 2.);
        assert!((result.1 - orders.iter().map(|(_, m)| *m as f64).sum::<f64>()).abs() < 1e-5);
        assert!(cash >= 0.5 - 1e-5);
        assert_eq!(expansion_budget(&f, &e, 10.), 0.);
        let budget = expansion_budget(&f, &e, 1000.);
        assert!(budget > 247.5 && budget < 742.5);
        let before = budget;
        e.prices.iter_mut().for_each(|p| *p *= 10.);
        assert!(expansion_budget(&f, &e, 1000.) < before);
    }
    #[test]
    fn substitution_scale_and_component_repairs_have_finite_costs() {
        let c = EconomyCatalog::bundled().unwrap();
        let mut e = stocks();
        let small = choose(&c, &e, 16., 60.).unwrap();
        let large = choose(&c, &e, 16., 1000.).unwrap();
        assert!(small.capacity < large.capacity);
        for good in [0, 2, 5] {
            let mut local = stocks();
            local.goods.fill(0.);
            local.goods[good] = 10000.;
            if good == 5 {
                local.goods[50] = 10000.;
            }
            let room = choose(&c, &local, 4., 10000.).unwrap();
            assert!(room
                .materials()
                .iter()
                .all(|&(g, _)| g as usize == good || g == 50));
        }
        let before = e.goods;
        let mut f = Facility::found(small, &mut e);
        for (g, m) in f.embodied() {
            assert_eq!(before[g as usize] - e.goods[g as usize], m);
        }
        let goods = e.goods;
        let mut treasury = 1000.;
        let cash = e.finance[0] as f64 + treasury;
        // No construction work -> no usable room, no replacement mass.
        let result = f.advance(&mut e, &c, &mut treasury, 0., 2.);
        assert_eq!(result.1, 0.);
        assert_eq!(f.usable(), 0.);
        assert_eq!(e.goods, goods);
        let required = f.remaining();
        f.advance(&mut e, &c, &mut treasury, required, 2.);
        assert!(f.usable() > 0.);
        let mut resumed: Facility =
            serde_json::from_slice(&serde_json::to_vec(&f).unwrap()).unwrap();
        let mut e2 = e;
        let mut t2 = treasury;
        let before = e.goods;
        let waste = e.reserves[3];
        let result = f.advance(&mut e, &c, &mut treasury, 0.1, 16.);
        resumed.advance(&mut e2, &c, &mut t2, 0.1, 16.);
        assert_eq!(
            serde_json::to_value(&f).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert_eq!(e.goods, e2.goods);
        assert_eq!(treasury, t2);
        assert_eq!(cash, e.finance[0] as f64 + treasury);
        assert!(
            result.4,
            "a funded institution should reserve a real extension"
        );
        assert!(f.planned() > 2. && f.usable() > 0. && f.remaining() > 0.);
        let withdrawn: f32 = before.iter().zip(e.goods).map(|(a, b)| a - b).sum();
        assert!(withdrawn > result.1 as f32);
        assert!((e.reserves[3] - waste - result.1 as f32).abs() < 1e-4);
    }
}
