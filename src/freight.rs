//! Reservation footprints for commercial journeys. Goods stay solely in Cargo.
use crate::civilization::History;
use std::collections::BTreeMap;

#[derive(Clone, Default)]
pub(crate) struct FreightPath {
    pub stops: Vec<u32>,
    pub edges: Vec<[u32; 2]>,
}
impl FreightPath {
    fn road(stops: Vec<u32>) -> Self {
        let edges = stops
            .windows(2)
            .map(|w| [w[0].min(w[1]), w[0].max(w[1])])
            .collect();
        Self { stops, edges }
    }
    fn extend(&mut self, other: Self) {
        self.stops.extend(other.stops);
        self.edges.extend(other.edges);
    }
    fn normalize(&mut self) {
        self.stops.sort_unstable();
        self.stops.dedup();
        self.edges.sort_unstable();
        self.edges.dedup();
    }
}

type RoadTree = (Vec<f32>, Vec<u32>);
impl History {
    /// Respect the captured corridor, not an unreserved alternative path. A
    /// removed historical edge has no current flood observation; do not invent one.
    pub(crate) fn freight_path_flooded(&self, edges: &[[u32; 2]]) -> bool {
        let Some(society) = &self.society else {
            return false;
        };
        edges.iter().any(|edge| {
            let mut matching = society
                .routes
                .iter()
                .filter(|r| r.open && [r.from.min(r.to), r.from.max(r.to)] == *edge)
                .peekable();
            matching.peek().is_some() && matching.all(|r| r.flood_months > 0)
        })
    }

    /// Free kg in transit on an undirected corridor. Opposite directions share it.
    /// Legacy unmaintained roads and unplanned endpoint economies keep their old limit.
    pub fn road_freight_capacity(&self, edge: [u32; 2]) -> f32 {
        let edge = [edge[0].min(edge[1]), edge[0].max(edge[1])];
        let Some(society) = &self.society else {
            return 0.;
        };
        let Some(catalog) = &self.economy_catalog else {
            return 0.;
        };
        let gross = |site: u32| {
            self.sites.get(site as usize).map_or(0., |s| {
                if s.economy.logistics[3] <= 0.5 {
                    f32::INFINITY
                } else {
                    s.stocks.stock[0] * catalog.production.land_freight_kg_per_person
                }
            })
        };
        // The graph stores settlement predecessors, not parallel-road identities.
        // Treat parallel roads as one corridor with the best available surface.
        let capacity = society
            .routes
            .iter()
            .filter(|r| [r.from.min(r.to), r.from.max(r.to)] == edge && r.passable())
            .map(|r| {
                if r.upkeep.is_none() {
                    f32::INFINITY
                } else {
                    gross(edge[0]).min(gross(edge[1]))
                        * (0.5 + 0.5 * (r.road_bricks / 1000.).clamp(0., 1.) as f32)
                }
            })
            .fold(0_f32, f32::max);
        let used: f32 = self
            .cargo
            .iter()
            .filter(|c| c.freight_edges.contains(&edge))
            .map(|c| c.kg)
            .sum();
        let military: f32 = self
            .military
            .siege
            .supplies
            .iter()
            .filter(|s| [s.from.min(s.to), s.from.max(s.to)] == edge)
            .map(|s| s.food)
            .sum();
        (capacity - used - military).max(0.)
    }

    /// Build once per market quarter, not once per good. Reuse trees per origin
    /// and transit administration; sea approaches retain the seller's permissions.
    pub(crate) fn trade_freight_stops(
        &self,
        seas: Option<&[Option<(f32, u32)>]>,
        network: bool,
    ) -> Vec<Option<FreightPath>> {
        let n = self.sites.len();
        let mut result = vec![None; n * n];
        let mut trees: BTreeMap<(u32, u32), RoadTree> = BTreeMap::new();
        let mut path = |start: u32, end: u32, administration: u32| -> Option<(f32, FreightPath)> {
            let (distances, parents) = trees
                .entry((start, administration))
                .or_insert_with(|| self.road_tree_from(start as usize, administration));
            let distance = distances[end as usize];
            if !distance.is_finite() {
                return None;
            }
            let mut stops = vec![end];
            let mut at = end;
            while at != start {
                at = *parents.get(at as usize)?;
                if at == u32::MAX || stops.len() >= n {
                    return None;
                }
                stops.push(at);
            }
            Some((distance, FreightPath::road(stops)))
        };
        for a in 0..n {
            for b in 0..n {
                let lane = seas.and_then(|s| s[a * n + b]).map(|(_, lane)| lane);
                let Some(ends) = self.freight_sites(a as u32, b as u32, lane) else {
                    continue;
                };
                let mut stops = if !network {
                    FreightPath {
                        stops: ends.to_vec(),
                        edges: vec![],
                    }
                } else if lane.is_some() {
                    let mut best: Option<(f32, FreightPath)> = None;
                    for (p, q) in [(ends[2], ends[3]), (ends[3], ends[2])] {
                        let admin = self.controller(a as u32);
                        if let (Some((d1, mut first)), Some((d2, second))) =
                            (path(a as u32, p, admin), path(q, b as u32, admin))
                        {
                            if best.as_ref().is_none_or(|(cost, _)| d1 + d2 < *cost) {
                                first.extend(second);
                                best = Some((d1 + d2, first));
                            }
                        }
                    }
                    let Some((_, stops)) = best else {
                        continue;
                    };
                    stops
                } else {
                    let Some((_, stops)) = path(a as u32, b as u32, self.controller(a as u32))
                    else {
                        continue;
                    };
                    stops
                };
                stops.normalize();
                result[a * n + b] = Some(stops);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
    };

    #[test]
    fn sea_approaches_keep_edges_without_inventing_a_land_crossing() {
        let mut path = FreightPath::road(vec![0, 1, 2]);
        path.extend(FreightPath::road(vec![4, 3]));
        path.extend(FreightPath::road(vec![2, 1]));
        path.normalize();
        assert_eq!(path.stops, vec![0, 1, 2, 3, 4]);
        assert_eq!(path.edges, vec![[0, 1], [1, 2], [3, 4]]);
    }

    #[test]
    #[ignore = "requires hardware GPU to initialize the history fixture"]
    fn junction_capacity_is_reserved_until_delivery_even_after_rerouting() {
        let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
        for seed in [17, 81, 256] {
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    seed,
                    resolution: 32,
                    ecology_resolution: 16,
                    ..Default::default()
                },
                Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.found_civilizations(5).unwrap();
            g.enable_society().unwrap();
            let h = g.civilizations.as_mut().unwrap();
            h.shipping = None;
            h.politics = None;
            h.cargo.clear();
            let catalog = h.economy_catalog.as_mut().unwrap();
            catalog.market.network_trade = true;
            catalog.market.max_distance_km = 100000.;
            catalog.production.land_freight_kg_per_person = 1.;
            for s in &mut h.sites {
                s.island = 0;
                s.stocks.stock[0] = if s.id == 1 { 3. } else { 10. };
                s.stocks.stock[1] = 10000.;
                s.economy.goods.fill(0.);
                s.economy.targets.fill(0.);
                s.economy.logistics = [1000., 0., 0., 1.];
                s.economy.finance[0] = 10000.;
                s.economy.policy[3] = 1.;
            }
            h.sites[0].economy.goods[3] = 100.;
            h.sites[2].economy.targets[3] = 20.;
            let roads = |junction| {
                [(0, junction), (junction, 2)]
                    .into_iter()
                    .enumerate()
                    .map(|(id, (from, to))| crate::society::Route {
                        id: id as u32,
                        from,
                        to,
                        cost_km: 600.,
                        cells: vec![],
                        open: true,
                        flood_months: 0,
                        road_bricks: 0.,
                        upkeep: None,
                    })
                    .collect()
            };
            h.society.as_mut().unwrap().routes = roads(1);
            let total = |h: &History| {
                (
                    h.sites
                        .iter()
                        .map(|s| s.economy.goods[3] as f64)
                        .sum::<f64>()
                        + h.cargo
                            .iter()
                            .filter(|c| c.good == 3)
                            .map(|c| c.kg as f64)
                            .sum::<f64>(),
                    h.sites
                        .iter()
                        .map(|s| s.economy.finance[0] as f64)
                        .sum::<f64>(),
                )
            };
            let before = total(h);
            // Identical town carriers; only maintained road surface differs.
            for (bricks, expected) in [(0., 1.5), (1000., 3.)] {
                let mut road = h.clone();
                for r in &mut road.society.as_mut().unwrap().routes {
                    r.upkeep = Some(crate::road_upkeep::RoadUpkeep::new(0));
                    r.road_bricks = bricks;
                }
                road.month = 3;
                road.market_month(6371.);
                assert_eq!(road.cargo.len(), 1);
                assert_eq!(road.cargo[0].kg, expected);
                assert_eq!(road.cargo[0].freight_edges, vec![[0, 1], [1, 2]]);
                assert_eq!(road.road_freight_capacity([1, 0]), 0.);
                assert_eq!(road.land_freight_capacity(1), 3. - expected);
                assert_eq!(total(&road).0, before.0);
                assert!((total(&road).1 - before.1).abs() < 0.01);
                // Both directions/goods reserve the same physical corridor.
                road.sites[0].economy.targets[4] = 20.;
                road.sites[2].economy.goods[4] = 100.;
                road.month = 6;
                road.market_month(6371.);
                assert_eq!(road.cargo.len(), 1);
                let arrival = road.cargo[0].arrives;
                let mut held = road.clone();
                held.society.as_mut().unwrap().routes[0].flood_months = 1;
                let mut alternatives: Vec<crate::society::Route> = roads(3);
                for r in &mut alternatives {
                    r.id += 2;
                }
                held.society.as_mut().unwrap().routes.extend(alternatives);
                assert!(held.freight_path_flooded(&held.cargo[0].freight_edges));
                assert!(!held.freight_path_flooded(&[[0, 3], [2, 3]]));
                assert!(
                    !held.freight_path_flooded(&[]),
                    "old cargo has no invented corridor"
                );
                let mut restored: History =
                    serde_json::from_value(serde_json::to_value(&held).unwrap()).unwrap();
                for state in [&mut held, &mut restored] {
                    state.month = arrival;
                    state.market_arrivals();
                    assert_eq!(state.cargo.len(), 1);
                    assert_eq!(state.cargo[0].weather_delay_months, 1);
                    assert_eq!(state.cargo[0].arrives, arrival + 1);
                    assert_eq!(state.cargo[0].freight_edges, vec![[0, 1], [1, 2]]);
                    assert_eq!(total(state), total(&road));
                    state.market_arrivals();
                    assert_eq!(
                        state.cargo[0].weather_delay_months, 1,
                        "same-month observation cannot charge a second delay"
                    );
                    state.society.as_mut().unwrap().routes[0].flood_months = 0;
                    state.month += 1;
                    state.market_arrivals();
                    assert!(state.cargo.is_empty());
                    assert_eq!(total(state), total(&road));
                }
                assert_eq!(
                    serde_json::to_value(held).unwrap(),
                    serde_json::to_value(restored).unwrap()
                );
                // Closing/reopening does not erase a claim; surface loss can overcommit
                // existing cargo but never grants negative or additional capacity.
                road.society.as_mut().unwrap().routes[0].flood_months = 1;
                assert_eq!(road.road_freight_capacity([0, 1]), 0.);
                road.society.as_mut().unwrap().routes[0].flood_months = 0;
                road.society.as_mut().unwrap().routes[0].road_bricks = 0.;
                assert_eq!(road.road_freight_capacity([0, 1]), 0.);
                let mut saved: History =
                    serde_json::from_slice(&serde_json::to_vec(&road).unwrap()).unwrap();
                for state in [&mut road, &mut saved] {
                    for site in &mut state.sites {
                        site.economy.policy[3] = 0.;
                    }
                    state.month = arrival;
                    state.market_month(6371.);
                    assert!(state.cargo.is_empty());
                    assert_eq!(state.road_freight_capacity([0, 1]), 1.5);
                }
                assert_eq!(
                    serde_json::to_value(&road).unwrap(),
                    serde_json::to_value(&saved).unwrap()
                );
            }
            let mut bypass = h.clone();
            bypass.society.as_mut().unwrap().routes = roads(3);
            h.month = 3;
            h.market_month(6371.);
            assert_eq!(h.cargo.len(), 1);
            assert_eq!(h.cargo[0].kg, 3.);
            assert_eq!(h.cargo[0].freight_stops, vec![0, 1, 2]);
            assert_eq!(h.cargo[0].freight_edges, vec![[0, 1], [1, 2]]);
            let mut old = serde_json::to_value(&h.cargo[0]).unwrap();
            old.as_object_mut().unwrap().remove("freight_edges");
            assert!(serde_json::from_value::<crate::economy::Cargo>(old)
                .unwrap()
                .freight_edges
                .is_empty());
            assert_eq!(h.land_freight_capacity(1), 0.);
            assert_eq!(h.land_freight_capacity(3), 10.);
            assert_eq!(total(h).0, before.0);
            assert!((total(h).1 - before.1).abs() < 0.01);
            bypass.month = 3;
            bypass.market_month(6371.);
            let bypass_kg: f32 = bypass.cargo.iter().map(|c| c.kg).sum();
            assert!(bypass_kg > 3.);
            // Road changes and serialization do not silently move active reservations.
            h.society.as_mut().unwrap().routes = roads(3);
            let saved = serde_json::to_vec(h).unwrap();
            let mut resumed: History = serde_json::from_slice(&saved).unwrap();
            assert_eq!(resumed.land_freight_capacity(1), 0.);
            let arrival = h.cargo[0].arrives;
            for state in [&mut *h, &mut resumed] {
                // Close buying for this fixture to inspect release without replacement cargo.
                state.sites[2].economy.policy[3] = 0.;
                state.month = 6;
                state.market_month(6371.);
                assert_eq!(state.land_freight_capacity(1), 0.);
                state.month = arrival;
                state.market_month(6371.);
                assert!(state.cargo.is_empty());
                assert_eq!(state.land_freight_capacity(1), 3.);
                assert_eq!(total(state).0, before.0);
            }
            assert!(serde_json::to_value(&*h).unwrap() == serde_json::to_value(resumed).unwrap());
            println!("seed {seed}: junction dispatch 3 kg; bypass {bypass_kg} kg; serialized reservations and delivery balanced");
        }
    }
}
