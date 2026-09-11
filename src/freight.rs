//! Reservation footprints for commercial journeys. Goods stay solely in Cargo.
use crate::civilization::History;
use std::collections::BTreeMap;

type RoadTree = (Vec<f32>, Vec<u32>);
impl History {
    /// Build once per market quarter, not once per good. Reuse trees per origin
    /// and transit administration; sea approaches retain the seller's permissions.
    pub(crate) fn trade_freight_stops(
        &self,
        seas: Option<&[Option<(f32, u32)>]>,
        network: bool,
    ) -> Vec<Option<Vec<u32>>> {
        let n = self.sites.len();
        let mut result = vec![None; n * n];
        let mut trees: BTreeMap<(u32, u32), RoadTree> = BTreeMap::new();
        let mut path = |start: u32, end: u32, administration: u32| -> Option<(f32, Vec<u32>)> {
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
            Some((distance, stops))
        };
        for a in 0..n {
            for b in 0..n {
                let lane = seas.and_then(|s| s[a * n + b]).map(|(_, lane)| lane);
                let Some(ends) = self.freight_sites(a as u32, b as u32, lane) else {
                    continue;
                };
                let mut stops = if !network {
                    ends.to_vec()
                } else if lane.is_some() {
                    let mut best: Option<(f32, Vec<u32>)> = None;
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
                stops.sort_unstable();
                stops.dedup();
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
            let mut bypass = h.clone();
            bypass.society.as_mut().unwrap().routes = roads(3);
            h.month = 3;
            h.market_month(6371.);
            assert_eq!(h.cargo.len(), 1);
            assert_eq!(h.cargo[0].kg, 3.);
            assert_eq!(h.cargo[0].freight_stops, vec![0, 1, 2]);
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
