//! Canonical road materials weather; annual rebuilding competes for remaining craft work.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoadUpkeep {
    pub observed: u32,
    pub lost_kg: f64,
    pub work: f64,
    pub last_build: Option<u32>,
    pub matured: bool,
    pub impaired: bool,
}
impl RoadUpkeep {
    pub fn new(month: u32) -> Self {
        Self {
            observed: month,
            lost_kg: 0.,
            work: 0.,
            last_build: None,
            matured: false,
            impaired: false,
        }
    }
}
fn weathered_mass(stock: f64, flooded: bool) -> f32 {
    let requested = (stock * if flooded { 0.022 } else { 0.002 }).min(f32::MAX as f64);
    let mut mass = requested as f32;
    if mass as f64 > requested {
        mass = f32::from_bits(mass.to_bits().saturating_sub(1));
    }
    mass
}
impl crate::civilization::History {
    pub(crate) fn weather_roads(&mut self) {
        let Some(mut society) = self.society.take() else {
            return;
        };
        for r in &mut society.routes {
            let Some(care) = &mut r.upkeep else {
                continue;
            };
            if care.observed >= self.month {
                continue;
            }
            // Normal scheduling visits every month. Do not reconstruct missing historical
            // flooding from a single current observation when importing old state.
            care.observed = self.month;
            care.matured |= r.road_bricks >= 800.;
            let mass = weathered_mass(r.road_bricks, r.flood_months > 0);
            r.road_bricks -= mass as f64;
            care.lost_kg += mass as f64;
            let e = &mut self.sites[r.from as usize].economy;
            e.used[5] += mass;
            e.reserves[3] += mass;
            if let Some(catalog) = &self.economy_catalog {
                for (k, ratio) in catalog.composition(5).iter().enumerate() {
                    e.detritus[k] += mass * ratio;
                }
            }
        }
        self.society = Some(society);
        self.road_condition_events();
    }
    pub(crate) fn road_condition_events(&mut self) {
        let Some(mut society) = self.society.take() else {
            return;
        };
        for r in &mut society.routes {
            let Some(c) = &mut r.upkeep else {
                continue;
            };
            c.matured |= r.road_bricks >= 800.;
            let kind = if c.matured && !c.impaired && r.road_bricks < 500. {
                c.impaired = true;
                Some("road_deteriorated")
            } else if c.impaired && r.road_bricks >= 800. {
                c.impaired = false;
                Some("road_restored")
            } else {
                None
            };
            if let Some(kind) = kind {
                let cause = self
                    .events
                    .iter()
                    .rev()
                    .find(|e| e.subjects.contains(&("road".into(), r.id)))
                    .map(|e| e.id);
                self.event(kind,Some(r.from),Some(r.to),format!("Road {} retains {:.1} kg improved surface; {:.1} kg weathered; {:.2} worker-months spent building and repairing",r.id,r.road_bricks,c.lost_kg,c.work));
                let e = self.events.last_mut().unwrap();
                e.subjects.push(("road".into(), r.id));
                if let Some(cause) = cause {
                    e.causes.push(cause);
                }
            }
        }
        self.society = Some(society);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decay_is_bounded_and_flooding_accelerates_it() {
        let mut stock = 1000.;
        let mut lost = 0.;
        for _ in 0..12 {
            let m = weathered_mass(stock, false) as f64;
            stock -= m;
            lost += m;
        }
        assert!((stock - 1000. * 0.998f64.powi(12)).abs() < 1e-5);
        assert_eq!(stock + lost, 1000.);
        assert!(weathered_mass(1000., true) > weathered_mass(1000., false));
        assert_eq!(weathered_mass(0., true), 0.);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn road_weathering_transfers_material_and_changes_travel() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(16).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let r = &mut h.society.as_mut().unwrap().routes[0];
        // Declared fixture: an existing mature road, isolated from annual repairs.
        r.road_bricks = 1000.;
        r.open = true;
        r.flood_months = 0;
        let (from, to) = (r.from, r.to);
        let initial_cost = h.route_cost(from, to).unwrap();
        let used = h.sites[from as usize].economy.used[5];
        let waste = h.sites[from as usize].economy.reserves[3];
        for _ in 0..360 {
            h.month += 1;
            h.weather_roads();
        }
        let r = &h.society.as_ref().unwrap().routes[0];
        let care = r.upkeep.as_ref().unwrap();
        assert!((r.road_bricks + care.lost_kg - 1000.).abs() < 1e-9);
        assert!((r.road_bricks - 1000. * 0.998f64.powi(360)).abs() < 1e-4);
        assert!(care.impaired);
        assert!(h.route_cost(from, to).unwrap() > initial_cost);
        assert!(
            (h.sites[from as usize].economy.used[5] - used - care.lost_kg as f32).abs() < 0.002
        );
        assert!(
            (h.sites[from as usize].economy.reserves[3] - waste - care.lost_kg as f32).abs()
                < 0.002
        );
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "road_deteriorated")
                .count(),
            1
        );
        h.validate_event_links().unwrap();
        let snapshot = serde_json::to_vec(&h).unwrap();
        h.weather_roads();
        assert_eq!(snapshot, serde_json::to_vec(&h).unwrap());
        let mut resumed: crate::civilization::History = serde_json::from_slice(&snapshot).unwrap();
        for world in [&mut *h, &mut resumed] {
            world.month += 1;
            world.sites[from as usize].abandoned = true;
            let r = &mut world.society.as_mut().unwrap().routes[0];
            r.open = false;
            r.flood_months = 1;
            let before = r.road_bricks;
            world.weather_roads();
            let r = &world.society.as_ref().unwrap().routes[0];
            assert!((r.road_bricks - before * 0.978).abs() < 1e-5);
        }
        assert_eq!(
            serde_json::to_vec(&h).unwrap(),
            serde_json::to_vec(&resumed).unwrap()
        );

        // Audit only buildable work: cash shortage and absent bricks are distinct.
        for site in &mut h.sites {
            site.economy.finance[0] = 0.;
            site.stocks.stock[3] = 0.;
        }
        let controller = h.controller(from) as usize;
        let society = h.society.as_mut().unwrap();
        for council in &mut society.councils {
            council.treasury = 0.;
        }
        society.councils[controller].treasury = 40.;
        for route in &mut society.routes {
            route.open = false;
        }
        let route = &mut society.routes[0];
        route.open = true;
        route.flood_months = 0;
        route.road_bricks = 500.;
        let site = &mut h.sites[from as usize];
        site.abandoned = false;
        site.economy.goods[5] = 100.;
        site.economy.logistics[2] = 1.;
        let mut no_material = h.clone();
        no_material.sites[from as usize].economy.goods[5] = 0.;
        h.social_year();
        let receipt = &h.society.as_ref().unwrap().council_funding.roads;
        assert_eq!(receipt.requested, 200.);
        assert_eq!(receipt.paid, 40.);
        assert_eq!(receipt.shortfall, 160.);
        assert_eq!(receipt.underfunded, 1);
        assert_eq!(h.society.as_ref().unwrap().routes[0].road_bricks, 520.);
        no_material.social_year();
        let receipt = &no_material.society.as_ref().unwrap().council_funding.roads;
        assert_eq!(receipt.requested, 0.);
        assert_eq!(receipt.underfunded, 0);
    }
}
