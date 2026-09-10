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
            // Favor useful capacity, then lifetime cost including upkeep and construction labor.
            let score = units as f64
                / (unit_cost * (1. + 20. * (wall.wear + roof.wear) as f64)
                    + 20. * (wall.work + roof.work) as f64);
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
                p.condition = (p.condition - p.wear - 0.08 * e.soil[3].clamp(0., 1.)).max(0.);
                let g = p.good as usize;
                let price = e.prices[g].max(0.01) as f64;
                let improvement = (1. - p.condition)
                    .min(0.1)
                    .min(available / p.work)
                    .min(e.goods[g] / p.kg)
                    .min((*treasury / (p.kg as f64 * price)) as f32);
                let mass = take(e, g, p.kg * improvement);
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
        // Keep 75% of treasury for operation and other obligations; finish/repair first.
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
                (*treasury - 10.).max(0.) * 0.25,
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
