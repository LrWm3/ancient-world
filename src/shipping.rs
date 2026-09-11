//! Sparse lake shipping: surveyed coastal access, finite harbor/fleet assets and reserved cargo.
use crate::{
    civilization::History,
    gpu::{Cell, Generator},
    grid,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, BinaryHeap};
pub const MATERIALS: [usize; 3] = [0, 3, 5];
pub const TARGET: [f32; 3] = [200., 10., 100.];
/// kg installed per worker-month: timber, tools/rigging, masonry.
const HARBOR_WORK_RATES: [f32; 3] = [100., 10., 100.];
fn harbor_work_needed(materials: [f32; 3]) -> f32 {
    materials
        .into_iter()
        .zip(HARBOR_WORK_RATES)
        .map(|(m, r)| m / r)
        .sum()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarborWork {
    pub observed: Option<u32>,
    pub worker_months: f64,
    pub impaired: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Port {
    /// Missing archives retain the old pooled capacity; new ports have staffed vessels.
    #[serde(default)]
    pub fleet: Option<crate::vessels::Fleet>,
    #[serde(default)]
    pub work: Option<HarborWork>,
    pub site: u32,
    pub access: Vec<u32>,
    pub water_cell: u32,
    pub access_km: f32,
    /// Timber (harbor and boats), tools, masonry kg; held outside private stockpiles.
    pub assets: [f32; 3],
    pub commissioned: Option<u32>,
    #[serde(default)]
    pub flood_months: u32,
}
impl Port {
    pub fn capacity(&self) -> f32 {
        if self.commissioned.is_none() || self.flood_months > 0 {
            return 0.;
        }
        self.fleet
            .as_ref()
            .map_or(1000., |f| f.capacity())
            .min(1000.)
            * self
                .assets
                .iter()
                .zip(TARGET)
                .map(|(a, t)| a / t)
                .fold(1., f32::min)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeaLane {
    pub ports: [u32; 2],
    pub cells: Vec<u32>,
    pub km: f32,
    pub open: bool,
    #[serde(default)]
    pub flood_months: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shipping {
    pub version: u32,
    pub started: u32,
    pub surveyed_sites: u32,
    pub ports: Vec<Port>,
    pub lanes: Vec<SeaLane>,
}
fn land(c: &Cell) -> bool {
    c.meta[0] == 2 && c.water[0] < 0.25
}
fn lake(c: &Cell) -> bool {
    c.meta[0] == 1 && c.water[0] > 0.25
}
/// Dijkstra uses integer meters and stable cell ordering. Access terminates at the first
/// reachable great-lake cell; sea searches cannot cross any land or exterior ocean.
pub(crate) fn path(
    start: u32,
    end: Option<u32>,
    n: u32,
    radius: f32,
    cells: &[Cell],
    frontier: bool,
) -> Option<(Vec<u32>, f32)> {
    let mut distances = vec![u64::MAX; cells.len()];
    let mut parents = vec![u32::MAX; cells.len()];
    let mut heap = BinaryHeap::new();
    distances[start as usize] = 0;
    heap.push(std::cmp::Reverse((0u64, start)));
    while let Some(std::cmp::Reverse((cost, i))) = heap.pop() {
        if cost != distances[i as usize] {
            continue;
        }
        if cost
            > if end.is_some() || frontier {
                20_000_000
            } else {
                2_000_000
            }
        {
            break;
        }
        if end.map_or(
            if frontier {
                cells[i as usize].meta[0] == 3 && cells[i as usize].water[0] < 0.25
            } else {
                lake(&cells[i as usize])
            },
            |end| i == end,
        ) {
            let mut route = vec![i];
            while *route.last().unwrap() != start {
                route.push(parents[*route.last().unwrap() as usize]);
            }
            route.reverse();
            return Some((route, cost as f32 / 1000.));
        }
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let j = grid::neighbor(i, n, dx, dy);
            let outer =
                frontier && cells[j as usize].meta[0] == 3 && cells[j as usize].water[0] < 0.25;
            if !outer
                && !lake(&cells[j as usize])
                && (frontier || end.is_some() || !land(&cells[j as usize]))
            {
                continue;
            }
            let friction = if frontier || end.is_some() || lake(&cells[j as usize]) {
                1.
            } else {
                1. + (cells[i as usize].terrain[0] - cells[j as usize].terrain[0]).abs() / 500.
                    + (cells[j as usize].water[3] / 1000.).min(3.)
            };
            let step =
                (crate::civilization::distance(i, j, n) * radius * 1000. * friction).max(1.) as u64;
            let next = cost + step;
            if next < distances[j as usize] {
                distances[j as usize] = next;
                parents[j as usize] = i;
                heap.push(std::cmp::Reverse((next, j)));
            }
        }
    }
    None
}
pub(crate) fn frontier_path(
    start: u32,
    n: u32,
    radius: f32,
    cells: &[Cell],
) -> Option<(Vec<u32>, f32)> {
    path(start, None, n, radius, cells, true)
}
impl Shipping {
    pub fn validate(&self, h: &History, cells: &[Cell]) -> Result<()> {
        ensure!(
            self.version == 1
                && (1..=1024).contains(&h.terrain_resolution)
                && cells.len() == 6 * h.terrain_resolution as usize * h.terrain_resolution as usize
                && h.society.is_some()
                && self.started <= h.month
                && self.surveyed_sites as usize <= h.sites.len()
                && self.ports.len() <= h.sites.len(),
            "invalid shipping baseline"
        );
        let mut islands = BTreeSet::new();
        let contiguous = |p: &[u32]| {
            p.windows(2).all(|w| {
                [(-1, 0), (1, 0), (0, -1), (0, 1)]
                    .iter()
                    .any(|&(x, y)| grid::neighbor(w[0], h.terrain_resolution, x, y) == w[1])
            })
        };
        for p in &self.ports {
            if let Some(f) = &p.fleet {
                ensure!(
                    f.vessels.len() <= 4
                        && f.vessels.iter().enumerate().all(|(id, v)| v.id == id as u32
                            && !v.name.is_empty()
                            && v.commissioned <= h.month
                            && v.funded_work.is_finite()
                            && (0. ..=0.25001).contains(&v.funded_work)
                            && v.wages_paid.is_finite()
                            && v.wages_paid >= 0.
                            && v.household.is_none_or(|id| h
                                .society
                                .as_ref()
                                .is_some_and(|s| (id as usize) < s.households.len()))),
                    "invalid vessel fleet"
                );
            }
            ensure!(
                (p.site as usize) < h.sites.len()
                    && islands.insert(h.sites[p.site as usize].island)
                    && p.access.first() == Some(&h.sites[p.site as usize].cell)
                    && !p.access.is_empty()
                    && p.access
                        .iter()
                        .all(|&c| cells
                            .get(c as usize)
                            .is_some_and(|c| if h.living.is_some() {
                                c.meta[0] == 2
                            } else {
                                land(c)
                            }))
                    && cells
                        .get(p.water_cell as usize)
                        .is_some_and(|c| if h.living.is_some() {
                            c.meta[0] == 1
                        } else {
                            lake(c)
                        })
                    && contiguous(&p.access)
                    && contiguous(&[*p.access.last().unwrap(), p.water_cell])
                    && p.access_km.is_finite()
                    && p.access_km > 0.
                    && p.assets
                        .iter()
                        .zip(TARGET)
                        .all(|(a, t)| a.is_finite() && (0. ..=t + 0.001).contains(a))
                    && p.work
                        .as_ref()
                        .is_none_or(|w| w.observed.is_none_or(|m| m <= h.month)
                            && w.worker_months.is_finite()
                            && w.worker_months >= 0.)
                    && p.commissioned
                        .is_none_or(|m| m >= self.started && m <= h.month),
                "invalid coastal port"
            );
        }
        let mut pairs = BTreeSet::new();
        for l in &self.lanes {
            ensure!(
                l.ports[0] < l.ports[1]
                    && (l.ports[1] as usize) < self.ports.len()
                    && pairs.insert(l.ports)
                    && l.cells.first() == Some(&self.ports[l.ports[0] as usize].water_cell)
                    && l.cells.last() == Some(&self.ports[l.ports[1] as usize].water_cell)
                    && l.km.is_finite()
                    && l.km > 0.
                    && l.km <= 20_000.
                    && l.cells
                        .iter()
                        .all(|&c| cells
                            .get(c as usize)
                            .is_some_and(|c| if h.living.is_some() {
                                c.meta[0] == 1
                            } else {
                                lake(c)
                            }))
                    && contiguous(&l.cells),
                "invalid sea lane"
            );
        }
        for c in &h.cargo {
            if let Some(lane) = c.sea_lane {
                ensure!(
                    (lane as usize) < self.lanes.len()
                        && (c.from as usize) < h.sites.len()
                        && (c.to as usize) < h.sites.len(),
                    "invalid maritime cargo lane"
                );
                let l = &self.lanes[lane as usize];
                let mut ports = l
                    .ports
                    .map(|p| h.sites[self.ports[p as usize].site as usize].island);
                let mut endpoints = [
                    h.sites[c.from as usize].island,
                    h.sites[c.to as usize].island,
                ];
                ports.sort();
                endpoints.sort();
                ensure!(ports == endpoints, "maritime cargo uses unrelated islands");
            }
        }
        Ok(())
    }
}
impl History {
    #[cfg(test)]
    pub(crate) fn shipping_year(&mut self, cells: &[Cell], radius: f32) {
        self.shipping_year_with_navigation(cells, radius, None)
            .expect("CPU shipping survey");
    }
    pub(crate) fn shipping_year_with_navigation(
        &mut self,
        cells: &[Cell],
        radius: f32,
        navigation: Option<&crate::navigation::Navigation>,
    ) -> Result<()> {
        let Some(mut shipping) = self.shipping.take() else {
            return Ok(());
        };
        // Existing histories can have exhausted founding settlements beside prosperous
        // daughter towns. Prefer real construction reserves when establishing a harbor.
        let mut candidates: Vec<_> = (shipping.surveyed_sites as usize..self.sites.len()).collect();
        let readiness = |i: usize| {
            let s = &self.sites[i];
            MATERIALS
                .into_iter()
                .enumerate()
                .map(|(k, good)| {
                    let reserve = s.stocks.stock[0] * if good == 3 { 0.5 } else { 1. };
                    (s.economy.goods[good] - reserve).max(0.) / TARGET[k]
                })
                .fold(f32::INFINITY, f32::min)
        };
        candidates.sort_by(|&a, &b| readiness(b).total_cmp(&readiness(a)).then(a.cmp(&b)));
        for i in candidates {
            if self.sites[i].abandoned
                || shipping
                    .ports
                    .iter()
                    .any(|p| self.sites[p.site as usize].island == self.sites[i].island)
            {
                continue;
            }
            if let Some((mut access, access_km)) = match navigation {
                Some(nav) => nav.route(
                    self.sites[i].cell,
                    None,
                    crate::navigation::RouteKind::Harbor,
                )?,
                None => path(
                    self.sites[i].cell,
                    None,
                    self.terrain_resolution,
                    radius,
                    cells,
                    false,
                ),
            } {
                let water_cell = access.pop().unwrap();
                let new = shipping.ports.len() as u32;
                for (j, p) in shipping.ports.iter().enumerate() {
                    if let Some((cells, km)) = match navigation {
                        Some(nav) => nav.route(
                            p.water_cell,
                            Some(water_cell),
                            crate::navigation::RouteKind::Sea,
                        )?,
                        None => path(
                            p.water_cell,
                            Some(water_cell),
                            self.terrain_resolution,
                            radius,
                            cells,
                            false,
                        ),
                    } {
                        shipping.lanes.push(SeaLane {
                            ports: [j as u32, new],
                            cells,
                            km,
                            open: true,
                            flood_months: 0,
                        });
                    }
                }
                shipping.ports.push(Port {
                    fleet: Some(Default::default()),
                    site: i as u32,
                    access,
                    water_cell,
                    access_km,
                    work: Some(HarborWork {
                        observed: None,
                        worker_months: 0.,
                        impaired: false,
                    }),
                    assets: [0.; 3],
                    commissioned: None,
                    flood_months: 0,
                });
                self.event(
                    "port_survey",
                    Some(i as u32),
                    None,
                    format!("Surveyed coastal access over {access_km:.0} travel km"),
                );
            }
        }
        shipping.surveyed_sites = self.sites.len() as u32;
        for p in &mut shipping.ports {
            if let Some(w) = &mut p.work {
                if w.observed == Some(self.month) {
                    continue;
                }
                w.observed = Some(self.month);
            }
            let s = &mut self.sites[p.site as usize];
            for (k, good) in MATERIALS.into_iter().enumerate() {
                let wear = p.assets[k] * 0.02;
                p.assets[k] -= wear;
                s.economy.used[good] += wear;
                if good == 0 {
                    for (j, f) in [0.5, 0.002, 0.0002].into_iter().enumerate() {
                        s.economy.external[j] -= wear * f;
                    }
                } else {
                    s.economy.reserves[3] += wear;
                }
            }
            if !s.abandoned && s.economy.policy[3] >= 0.5 {
                let mut requested = [0.; 3];
                for (k, good) in MATERIALS.into_iter().enumerate() {
                    let reserve = s.stocks.stock[0] * if good == 3 { 0.5 } else { 1. };
                    requested[k] = (TARGET[k] - p.assets[k])
                        .max(0.)
                        .min((s.economy.goods[good] - reserve).max(0.));
                }
                let work = harbor_work_needed(requested);
                let scale = if p.work.is_some() && work > 0. {
                    (s.economy.logistics[2].max(0.) / work).min(1.)
                } else {
                    1.
                };
                let mut used_work = 0.;
                for (k, good) in MATERIALS.into_iter().enumerate() {
                    let before = s.economy.goods[good];
                    let remaining = (before as f64 - (requested[k] * scale) as f64).max(0.);
                    let mut after = remaining as f32;
                    if (after as f64) < remaining {
                        after = f32::from_bits(after.to_bits() + 1);
                    }
                    let added = before - after;
                    s.economy.goods[good] = after;
                    p.assets[k] += added;
                    used_work += added / HARBOR_WORK_RATES[k];
                }
                if let Some(w) = &mut p.work {
                    s.economy.logistics[2] = (s.economy.logistics[2] - used_work).max(0.);
                    w.worker_months += used_work as f64;
                }
            }
            if p.commissioned.is_none() && p.assets.iter().zip(TARGET).all(|(a, t)| *a >= t * 0.999)
            {
                p.commissioned = Some(self.month);
                self.event("port_opened",Some(p.site),None,"Harbor and merchant boats commissioned from 200 kg timber, 10 kg tools and 100 kg masonry; 1000 kg shared transport capacity".into());
            }
        }
        self.shipping = Some(shipping);
        self.harbor_condition_events();
        Ok(())
    }
    fn harbor_condition_events(&mut self) {
        let Some(mut shipping) = self.shipping.take() else {
            return;
        };
        for (id, p) in shipping.ports.iter_mut().enumerate() {
            // Structural condition is distinct from a temporary flood closure.
            let capacity = 1000.
                * p.assets
                    .iter()
                    .zip(TARGET)
                    .map(|(a, t)| a / t)
                    .fold(1., f32::min);
            let Some(w) = &mut p.work else {
                continue;
            };
            let kind = if p.commissioned.is_some() && !w.impaired && capacity < 500. {
                w.impaired = true;
                Some("harbor_deteriorated")
            } else if w.impaired && capacity >= 800. {
                w.impaired = false;
                Some("harbor_restored")
            } else {
                None
            };
            if let Some(kind) = kind {
                let cause = self
                    .events
                    .iter()
                    .rev()
                    .find(|e| e.subjects.contains(&("port".into(), id as u32)))
                    .map(|e| e.id);
                self.event(kind,Some(p.site),None,format!("Harbor structural capacity {:.0} kg; {:.2} worker-months spent on construction and repair",capacity,w.worker_months));
                let e = self.events.last_mut().unwrap();
                e.subjects.push(("port".into(), id as u32));
                if let Some(cause) = cause {
                    e.causes.push(cause);
                }
            }
        }
        self.shipping = Some(shipping);
    }
    pub fn sea_capacity(&self, lane: u32) -> f32 {
        let Some(s) = &self.shipping else {
            return 0.;
        };
        let Some(l) = s.lanes.get(lane as usize) else {
            return 0.;
        };
        if !l.open || l.flood_months > 0 {
            return 0.;
        }
        l.ports
            .iter()
            .map(|&p| {
                let used: f32 = self
                    .cargo
                    .iter()
                    .filter(|c| {
                        c.sea_lane
                            .is_some_and(|id| s.lanes[id as usize].ports.contains(&p))
                    })
                    .map(|c| c.kg)
                    .sum();
                (s.ports[p as usize].capacity() - used).max(0.)
            })
            .fold(f32::INFINITY, f32::min)
    }
    /// One sea leg, with surveyed inland approaches at both ends. Cost is expressed
    /// in land travel-equivalent km: 150 km/month inland, 600 km/month afloat.
    pub fn sea_quotes(&self, roads: &[f32]) -> Vec<Option<(f32, u32)>> {
        let n = self.sites.len();
        let mut quotes = vec![None; n * n];
        let Some(s) = &self.shipping else {
            return quotes;
        };
        if roads.len() != n * n {
            return quotes;
        }
        let inland_capacity: Vec<_> = (0..n)
            .map(|site| self.land_freight_capacity(site as u32))
            .collect();
        let max_distance = self
            .economy_catalog
            .as_ref()
            .map_or(3000., |c| c.market.max_distance_km);
        let hostile = |a: u32, b: u32| {
            self.politics.as_ref().is_some_and(|p| {
                p.wars.iter().any(|w| {
                    w.ended.is_none()
                        && ((w.attacker == a && w.defender == b)
                            || (w.attacker == b && w.defender == a))
                })
            })
        };
        let wartime = self
            .politics
            .as_ref()
            .is_some_and(|p| p.wars.iter().any(|w| w.ended.is_none()));
        for a in 0..n {
            let onward: Vec<Vec<f32>> = if wartime {
                s.ports
                    .iter()
                    .map(|p| self.road_distances_from(p.site as usize, self.controller(a as u32)))
                    .collect()
            } else {
                vec![]
            };
            for b in 0..n {
                if self.sites[a].island == self.sites[b].island
                    || self.sites[a].abandoned
                    || self.sites[b].abandoned
                    || hostile(self.controller(a as u32), self.controller(b as u32))
                {
                    continue;
                }
                for (id, l) in s
                    .lanes
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| l.open && l.flood_months == 0)
                {
                    for endpoints in [l.ports, [l.ports[1], l.ports[0]]] {
                        let p = &s.ports[endpoints[0] as usize];
                        let q = &s.ports[endpoints[1] as usize];
                        if self.sites[a].island != self.sites[p.site as usize].island
                            || self.sites[b].island != self.sites[q.site as usize].island
                            || p.capacity() < 1.
                            || q.capacity() < 1.
                            || [a as u32, b as u32, p.site, q.site]
                                .iter()
                                .any(|&site| inland_capacity[site as usize] < 1.)
                            || self.sites[p.site as usize].abandoned
                            || self.sites[q.site as usize].abandoned
                            || [a as u32, b as u32, p.site, q.site]
                                .iter()
                                .any(|&x| self.sites[x as usize].economy.policy[3] < 0.5)
                            || [p.site, q.site].iter().any(|&x| {
                                hostile(self.controller(a as u32), self.controller(x))
                                    || hostile(self.controller(b as u32), self.controller(x))
                            })
                        {
                            continue;
                        }
                        let distance = roads[a * n + p.site as usize]
                            + p.access_km
                            + l.km / 4.
                            + q.access_km
                            + if wartime {
                                onward[endpoints[1] as usize][b]
                            } else {
                                roads[q.site as usize * n + b]
                            };
                        if distance.is_finite()
                            && distance < max_distance
                            && quotes[a * n + b].is_none_or(|(d, _)| distance < d)
                        {
                            quotes[a * n + b] = Some((distance, id as u32));
                        }
                    }
                }
            }
        }
        quotes
    }
}
impl Generator {
    pub fn enable_shipping(&mut self) -> Result<()> {
        let cells = self.snapshot()?;
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(
            h.society.is_some() && h.shipping.is_none(),
            "shipping requires social history without an existing shipping baseline"
        );
        h.shipping = Some(Shipping {
            version: 1,
            started: h.month,
            surveyed_sites: 0,
            ports: vec![],
            lanes: vec![],
        });
        h.shipping_year_with_navigation(
            &cells,
            self.config.radius_km,
            self.navigation_service()?.as_deref(),
        )?;
        h.event(
            "shipping_baseline",
            None,
            None,
            "Enabled surveyed great-lake shipping with finite harbor and fleet assets".into(),
        );
        h.validate(&cells)?;
        self.civilizations = Some(h);
        Ok(())
    }
    pub fn set_sea_lane_open(&mut self, lane: u32, open: bool) -> Result<()> {
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let l = h
            .shipping
            .as_mut()
            .and_then(|s| s.lanes.get_mut(lane as usize))
            .ok_or_else(|| anyhow::anyhow!("unknown sea lane"))?;
        l.open = open;
        h.event(
            "sea_lane_policy",
            None,
            None,
            format!("Sea lane {lane} {}", if open { "opened" } else { "closed" }),
        );
        self.civilizations = Some(h);
        Ok(())
    }
}

#[cfg(test)]
mod harbor_tests {
    use super::*;
    #[test]
    fn harbor_work_has_explicit_material_costs() {
        assert_eq!(harbor_work_needed(TARGET), 4.);
        assert!((harbor_work_needed(TARGET.map(|v| v * 0.02)) - 0.08).abs() < 1e-7);
        assert_eq!(harbor_work_needed([0.; 3]), 0.);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn harbor_work_limits_construction_and_preserves_condition_history() {
        use crate::{catalog::Catalog, config::Config, gpu::ContextGpu};
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
        g.enable_shipping().unwrap();
        let cells = g.snapshot().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let origin = h.shipping.as_ref().unwrap().ports[0].site as usize;
        // Declared construction fixture; no production or demographic changes.
        h.shipping.as_mut().unwrap().ports[0].assets = [0.; 3];
        h.shipping.as_mut().unwrap().ports[0].commissioned = None;
        h.sites[origin].economy.policy[3] = 1.;
        for good in MATERIALS {
            h.sites[origin].economy.goods[good] = 10000.;
        }
        h.sites[origin].economy.logistics[2] = 0.;
        h.month += 12;
        h.shipping_year(&cells, 6371.);
        assert_eq!(h.shipping.as_ref().unwrap().ports[0].assets, [0.; 3]);
        h.month += 12;
        h.sites[origin].economy.logistics[2] = 1.;
        h.shipping_year(&cells, 6371.);
        let p = &h.shipping.as_ref().unwrap().ports[0];
        assert_eq!(p.assets, [50., 2.5, 25.]);
        assert_eq!(p.capacity(), 0.);
        assert_eq!(p.work.as_ref().unwrap().worker_months, 1.);
        assert_eq!(h.sites[origin].economy.logistics[2], 0.);
        for (k, good) in MATERIALS.into_iter().enumerate() {
            assert_eq!(h.sites[origin].economy.goods[good] + p.assets[k], 10000.);
        }
        let snapshot = serde_json::to_vec(&h).unwrap();
        h.shipping_year(&cells, 6371.);
        assert_eq!(snapshot, serde_json::to_vec(&h).unwrap());
        h.month += 12;
        h.sites[origin].economy.logistics[2] = 4.;
        h.shipping_year(&cells, 6371.);
        assert!(h.shipping.as_ref().unwrap().ports[0].capacity() > 999.);
        h.sites[origin].abandoned = true;
        for _ in 0..35 {
            h.month += 12;
            h.shipping_year(&cells, 6371.);
        }
        let p = &h.shipping.as_ref().unwrap().ports[0];
        assert!(p.work.as_ref().unwrap().impaired);
        assert!((p.capacity() - 1000. * 0.98f32.powi(35)).abs() < 0.01);
        let mut resumed: History =
            serde_json::from_slice(&serde_json::to_vec(&h).unwrap()).unwrap();
        for world in [&mut *h, &mut resumed] {
            world.month += 12;
            world.sites[origin].abandoned = false;
            world.sites[origin].economy.logistics[2] = 4.;
            world.shipping_year(&cells, 6371.);
            assert!(
                !world.shipping.as_ref().unwrap().ports[0]
                    .work
                    .as_ref()
                    .unwrap()
                    .impaired
            );
            let e = world
                .events
                .iter()
                .find(|e| e.kind == "harbor_restored" && e.site == Some(origin as u32))
                .unwrap();
            assert!(!e.causes.is_empty());
        }
        assert_eq!(
            serde_json::to_vec(&h).unwrap(),
            serde_json::to_vec(&resumed).unwrap()
        );
        h.validate_event_links().unwrap();
        let e = h
            .events
            .iter_mut()
            .find(|e| e.kind == "harbor_restored")
            .unwrap();
        e.subjects.push(("port".into(), u32::MAX));
        assert!(h.validate_event_links().is_err());
    }
}
