//! Sparse historical consequences of GPU surface-water exposure.
use crate::{civilization::History, gpu::Cell};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FloodImpact {
    pub flooded: bool,
    #[serde(default)]
    pub wet_streak: u32,
    #[serde(default)]
    pub dry_streak: u32,
    #[serde(default)]
    pub persistent: bool,
    /// Bricks embodied in raised granary foundations, kg (not loose stock).
    #[serde(default)]
    pub granary_bricks: f64,
    pub recovery_months: u32,
    pub flooded_months: u32,
    pub depth_m: f32,
    pub severity: f32,
    pub food_lost_kg: f64,
    pub crops_lost_kg: f64,
    pub cause: Option<u64>,
}

/// River corridors occupy 5% of a routing cell in this regional abstraction.
/// This estimates local exposure from area-averaged storage; it creates no water.
pub fn flood_depth(cell: &Cell) -> f32 {
    let corridor = cell.routing[0] != crate::gpu::NONE
        && cell.water[3] > 1.
        && cell.hydro[0] - cell.terrain[0] < 0.01;
    cell.water[0] / if corridor { 0.05 } else { 1. }
}

fn countdown(value: &mut u32, exposed: bool) -> Option<bool> {
    let was = *value > 0;
    *value = if exposed { 3 } else { value.saturating_sub(1) };
    (was != (*value > 0)).then_some(*value > 0)
}

impl History {
    pub(crate) fn environmental_month(&mut self, cells: &[Cell]) {
        self.environmental_month_with_inspections(cells, None);
    }
    pub(crate) fn environmental_month_with_inspections(
        &mut self,
        cells: &[Cell],
        inspections: Option<&[bool]>,
    ) {
        let expected = self.society.as_ref().map_or(0, |s| s.routes.len())
            + self
                .shipping
                .as_ref()
                .map_or(0, |s| s.ports.len() + s.lanes.len());
        assert!(
            inspections.is_none_or(|v| v.len() == expected),
            "route inspection count mismatch"
        );
        let mut inspected = inspections.into_iter().flatten().copied();
        let Some(mut living) = self.living.take() else {
            return;
        };
        let mut records = Vec::new();
        for s in &mut self.sites {
            let depth = flood_depth(&cells[s.cell as usize]);
            let impact = living.floods.entry(s.id).or_default();
            let flooded = depth >= 0.25;
            let was = impact.flooded;
            let recovering = impact.recovery_months > 0;
            // Old archives have no streak counters. Recover uninterrupted exposure
            // from their last flood event rather than granting another year's grace.
            if was && impact.wet_streak == 0 {
                impact.wet_streak = impact
                    .cause
                    .and_then(|id| self.events.get(id as usize))
                    .map_or(0, |e| self.month.saturating_sub(e.month).min(12));
            }
            impact.wet_streak = if flooded {
                (impact.wet_streak + 1).min(12)
            } else {
                0
            };
            impact.dry_streak = if flooded {
                0
            } else {
                (impact.dry_streak + 1).min(12)
            };
            if !impact.persistent && impact.wet_streak == 12 {
                impact.persistent = true;
                records.push(("persistent_inundation", s.id,
                    "A full year of inundation: expansion suspended; land remains waterlogged rather than undergoing temporary cleanup".into(), impact.cause));
            } else if impact.persistent && impact.dry_streak == 12 {
                impact.persistent = false;
                records.push((
                    "inundation_recovered",
                    s.id,
                    "Twelve dry months: persistent inundation ended; ordinary land use can resume"
                        .into(),
                    impact.cause,
                ));
            }
            impact.depth_m = depth;
            impact.flooded = flooded;
            if flooded {
                impact.severity = (depth / 1.5).clamp(0., 1.);
            }
            if impact.persistent {
                impact.recovery_months = 0;
            } else {
                countdown(&mut impact.recovery_months, flooded);
            }
            s.economy.soil[3] = if flooded {
                (depth / 1.5).clamp(0., 1.)
            } else if impact.persistent {
                impact.severity * (1. - impact.dry_streak as f32 / 12.)
            } else {
                impact.severity * impact.recovery_months as f32 / 3.
            };
            // Raised granaries protect stored food, not fields or transport routes.
            // Construction uses part of the labor already diverted by inundation.
            let worn = (impact.granary_bricks * 0.002) as f32;
            impact.granary_bricks -= worn as f64;
            s.economy.used[5] += worn;
            s.economy.reserves[3] += worn;
            if impact.persistent && flooded && !s.abandoned {
                let target = s.stocks.stock[0] * 20. * (depth + 0.25).min(1.5);
                let builders = if self.society.is_some() {
                    s.demography.ages[1] * 0.8
                } else {
                    s.stocks.stock[0] * 0.5
                };
                let bricks = s.economy.goods[5]
                    .min((target - impact.granary_bricks as f32).max(0.))
                    .min(builders * 0.2 * s.economy.soil[3] * 20.);
                if bricks > 0. {
                    let first = impact.granary_bricks < 0.01;
                    s.economy.goods[5] -= bricks;
                    impact.granary_bricks += bricks as f64;
                    if first || self.month.is_multiple_of(12) {
                        records.push(("flood_adaptation", s.id, format!("Invested {bricks:.1} kg bricks in raised granaries; {:.1} kg embodied in foundations", impact.granary_bricks), impact.cause));
                    }
                }
            }
            if flooded && !s.abandoned {
                impact.flooded_months += 1;
                let severity = (depth / 1.5).clamp(0., 1.);
                let height =
                    (impact.granary_bricks as f32 / (s.stocks.stock[0] * 20.).max(1.)).min(1.5);
                let food = s.stocks.stock[1] * ((depth - height) / 1.5).clamp(0., 1.) * 0.06;
                let crop = s.demography.crops[0] * severity * 0.35;
                s.stocks.stock[1] -= food;
                s.demography.crops[0] -= crop;
                s.stocks.ledger[2] += food + crop;
                // Spoiled food and dead crops become local detritus, preserving C/N/P.
                for (k, ratio) in [0.45, 0.02, 0.003].into_iter().enumerate() {
                    s.economy.detritus[k] += (food + crop) * ratio;
                }
                impact.food_lost_kg += food as f64;
                impact.crops_lost_kg += crop as f64;
                if let Some(catalog) = &self.economy_catalog {
                    for j in 0..6 {
                        let lost = s.economy.crops[j][1] * severity * 0.35;
                        s.economy.crops[j][1] -= lost;
                        for k in 0..3 {
                            s.economy.detritus[k] += lost * catalog.composition(8 + j)[k];
                        }
                        impact.crops_lost_kg += lost as f64;
                    }
                }
            }
            if flooded && !was {
                records.push(("flood", s.id, format!("Surface water reached {depth:.2} m; stores and standing crops exposed, labor diverted to cleanup"), None));
            } else if !flooded && was {
                records.push((
                    "flood_receded",
                    s.id,
                    if impact.persistent { "Water receded; persistent inundation requires twelve dry months before recovery" } else { "Water receded; cleanup continues for two more monthly steps" }.into(),
                    impact.cause,
                ));
            } else if !flooded && !impact.persistent && recovering && impact.recovery_months == 0 {
                records.push(("flood_recovery", s.id, format!("Cleanup complete; cumulative flood losses {:.1} kg stored food and {:.1} kg standing crops", impact.food_lost_kg, impact.crops_lost_kg), impact.cause));
            }
        }
        if let Some(society) = &mut self.society {
            for r in &mut society.routes {
                let wet = inspected.next().unwrap_or_else(|| {
                    r.cells
                        .iter()
                        .any(|&c| flood_depth(&cells[c as usize]) >= 0.25)
                });
                if let Some(closed) = countdown(&mut r.flood_months, wet) {
                    records.push((
                        if closed {
                            "road_flood_closed"
                        } else {
                            "road_flood_recovered"
                        },
                        if self.sites[r.from as usize].abandoned {
                            r.to
                        } else {
                            r.from
                        },
                        format!(
                            "Road {} {} after surface-water inspection; owner policy remains {}",
                            r.id,
                            if closed { "unavailable" } else { "recovered" },
                            if r.open { "open" } else { "closed" }
                        ),
                        None,
                    ));
                }
            }
        }
        if let Some(shipping) = &mut self.shipping {
            for (id, p) in shipping.ports.iter_mut().enumerate() {
                let wet = inspected.next().unwrap_or_else(|| {
                    p.access
                        .iter()
                        .any(|&c| flood_depth(&cells[c as usize]) >= 0.25)
                        || cells[p.water_cell as usize].water[0] <= 0.25
                });
                if let Some(closed) = countdown(&mut p.flood_months, wet) {
                    records.push((
                        if closed {
                            "port_weather_closed"
                        } else {
                            "port_weather_recovered"
                        },
                        p.site,
                        format!(
                            "Port {id}: {}",
                            if closed {
                                "access flooded or navigable water unavailable"
                            } else {
                                "access recovered; fleet capacity restored"
                            }
                        ),
                        None,
                    ));
                }
            }
            for (id, l) in shipping.lanes.iter_mut().enumerate() {
                let shallow = inspected
                    .next()
                    .unwrap_or_else(|| l.cells.iter().any(|&c| cells[c as usize].water[0] <= 0.25));
                if let Some(closed) = countdown(&mut l.flood_months, shallow) {
                    records.push((
                        if closed {
                            "sea_weather_closed"
                        } else {
                            "sea_weather_recovered"
                        },
                        l.ports
                            .iter()
                            .map(|&p| shipping.ports[p as usize].site)
                            .find(|&site| !self.sites[site as usize].abandoned)
                            .unwrap_or(shipping.ports[l.ports[0] as usize].site),
                        format!(
                            "Sea lane {id}: {}",
                            if closed {
                                "insufficient navigable depth"
                            } else {
                                "navigable depth recovered"
                            }
                        ),
                        None,
                    ));
                }
            }
        }
        for (kind, site, detail, cause) in records {
            // Physical exposure/countdowns still evolve at ruins, but there is no
            // resident workforce performing cleanup or reporting local recovery.
            if self.sites[site as usize].abandoned {
                continue;
            }
            let id = self.events.len() as u64;
            self.event(kind, Some(site), None, detail);
            if let Some(cause) = cause {
                self.events.last_mut().unwrap().causes.push(cause);
            }
            if kind == "flood" {
                living.floods.get_mut(&site).unwrap().cause = Some(id);
            }
        }
        self.living = Some(living);
    }

    /// Cargo remains held (and paid for) while its physical approach is impassable.
    pub(crate) fn flood_blocks_delivery(&self, from: u32, to: u32, lane: Option<u32>) -> bool {
        if self.living.is_none() {
            return false;
        }
        if let Some(lane) = lane {
            if let Some(s) = &self.shipping {
                let l = &s.lanes[lane as usize];
                if l.flood_months > 0
                    || l.ports
                        .iter()
                        .any(|&p| s.ports[p as usize].flood_months > 0)
                {
                    return true;
                }
                for &p in &l.ports {
                    let site = s.ports[p as usize].site;
                    if self.sites[site as usize].island == self.sites[from as usize].island
                        && self.flood_blocks_delivery(from, site, None)
                    {
                        return true;
                    }
                    if self.sites[site as usize].island == self.sites[to as usize].island
                        && self.flood_blocks_delivery(site, to, None)
                    {
                        return true;
                    }
                }
            }
        }
        let Some(s) = &self.society else {
            return false;
        };
        if self.sites[from as usize].island != self.sites[to as usize].island {
            return false;
        }
        let reachable = |ignore_floods: bool| {
            let mut seen = vec![false; self.sites.len()];
            let mut queue = vec![from];
            seen[from as usize] = true;
            while let Some(i) = queue.pop() {
                if i == to {
                    return true;
                }
                for r in &s.routes {
                    if !r.open || (!ignore_floods && r.flood_months > 0) {
                        continue;
                    }
                    let next = if r.from == i {
                        r.to
                    } else if r.to == i {
                        r.from
                    } else {
                        continue;
                    };
                    if !seen[next as usize] {
                        seen[next as usize] = true;
                        queue.push(next);
                    }
                }
            }
            false
        };
        reachable(true) && !reachable(false)
    }
}

impl History {
    pub(crate) fn validate_hazards(&self) -> anyhow::Result<()> {
        if let Some(living) = &self.living {
            for (&site, f) in &living.floods {
                anyhow::ensure!(
                    (site as usize) < self.sites.len()
                        && f.recovery_months <= 3
                        && f.wet_streak <= 12
                        && f.dry_streak <= 12
                        && (!f.persistent || f.recovery_months == 0)
                        && f.granary_bricks.is_finite()
                        && f.granary_bricks >= 0.
                        && f.flooded_months <= self.month
                        && f.depth_m.is_finite()
                        && f.depth_m >= 0.
                        && (0. ..=1.).contains(&f.severity)
                        && f.food_lost_kg.is_finite()
                        && f.food_lost_kg >= 0.
                        && f.crops_lost_kg.is_finite()
                        && f.crops_lost_kg >= 0.
                        && f.cause.is_none_or(|id| self
                            .events
                            .get(id as usize)
                            .is_some_and(|e| e.kind == "flood" && e.site == Some(site))),
                    "invalid flood history"
                );
            }
        }
        anyhow::ensure!(
            self.society
                .as_ref()
                .is_none_or(|s| s.routes.iter().all(|r| r.flood_months <= 3))
                && self.shipping.as_ref().is_none_or(|s| s
                    .ports
                    .iter()
                    .all(|p| p.flood_months <= 3)
                    && s.lanes.iter().all(|l| l.flood_months <= 3)),
            "invalid transport recovery timer"
        );
        Ok(())
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
    #[ignore = "requires a hardware GPU"]
    fn flood_damage_conserves_food_and_nutrients_and_recovery_preserves_policy() {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
                ecology_resolution: 16,
                seed: 7,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        g.found_civilizations(16).unwrap();
        g.enable_society().unwrap();
        g.enable_shipping().unwrap();
        g.enable_living_history().unwrap();
        let mut h = g.civilizations.clone().unwrap();
        let mut cells = g.snapshot().unwrap();
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .first()
            .expect("inland road")
            .clone();
        h.society.as_mut().unwrap().routes[0].open = false;
        // Transfer an existing food inventory into standing crops; no free biomass.
        let s = &mut h.sites[route.from as usize];
        s.stocks.stock[1] -= 100.;
        s.demography.crops[0] += 100.;
        cells[s.cell as usize].water[0] = 1.5;
        let port = h
            .shipping
            .as_ref()
            .unwrap()
            .ports
            .first()
            .expect("coastal access")
            .clone();
        cells[port.access[0] as usize].water[0] = 1.5;
        h.month += 1;
        h.environmental_month(&cells);
        let f = &h.living.as_ref().unwrap().floods[&route.from];
        assert!(f.food_lost_kg > 0. && f.crops_lost_kg > 0.);
        assert_eq!(h.society.as_ref().unwrap().routes[0].flood_months, 3);
        assert_eq!(h.shipping.as_ref().unwrap().ports[0].flood_months, 3);
        assert_eq!(h.shipping.as_ref().unwrap().ports[0].capacity(), 0.);
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
        assert!(h.food_residual().abs() < 0.001);
        h.candidates
            .retain(|c| cells[c.cell as usize].water[0] < 0.25);
        h.validate(&cells).unwrap();
        let mut delivery = h.clone();
        delivery.society.as_mut().unwrap().routes[0].open = true;
        for site in &mut delivery.sites {
            site.economy.policy[3] = 0.;
        } // suppress new market orders
        delivery.sites[route.from as usize].stocks.stock[1] -= 10.;
        delivery.cargo.push(crate::economy::Cargo {
            freight_stops: vec![],
            from: route.from,
            to: route.to,
            good: crate::economy::FOOD as u32,
            kg: 10.,
            paid: 0.,
            arrives: delivery.month,
            sea_lane: None,
            weather_delay_months: 0,
        });
        let recipient_food = delivery.sites[route.to as usize].stocks.stock[1];
        delivery.market_month(g.config.radius_km);
        assert_eq!(delivery.cargo.len(), 1);
        assert_eq!(
            delivery.sites[route.to as usize].stocks.stock[1],
            recipient_food
        );
        let dry_delivery = g.snapshot().unwrap();
        for _ in 0..3 {
            delivery.month += 1;
            delivery.environmental_month(&dry_delivery);
            delivery.market_month(g.config.radius_km);
        }
        assert!(delivery.cargo.is_empty());
        assert_eq!(
            delivery.sites[route.to as usize].stocks.stock[1],
            recipient_food + 10. * 0.8_f32.powi(3)
        );
        delivery.market_month(g.config.radius_km);
        assert_eq!(
            delivery.sites[route.to as usize].stocks.stock[1],
            recipient_food + 10. * 0.8_f32.powi(3)
        );
        assert!(delivery.food_residual().abs() < 0.001);
        // A blocked contract ends after six months, including durable goods.
        let mut lost = h.clone();
        lost.society.as_mut().unwrap().routes[0].open = true;
        for site in &mut lost.sites {
            site.economy.policy[3] = 0.;
        }
        for good in [0u32, 6, crate::economy::FOOD as u32] {
            let site = &mut lost.sites[route.from as usize];
            if good == crate::economy::FOOD as u32 {
                site.stocks.stock[1] -= 10.;
            } else {
                // Declare a small fixture inventory directly in transit.
                site.economy.initial[good as usize] += 10.;
                let ratios = if good == 0 {
                    [0.5, 0.002, 0.0002]
                } else {
                    [1., 0., 0.]
                };
                for (k, ratio) in ratios.into_iter().enumerate() {
                    site.economy.baseline[k] += 10. * ratio;
                }
            }
            lost.cargo.push(crate::economy::Cargo {
                freight_stops: vec![],
                from: route.from,
                to: route.to,
                good,
                kg: 10.,
                paid: 0.,
                arrives: lost.month,
                sea_lane: None,
                weather_delay_months: 0,
            });
        }
        for _ in 0..6 {
            lost.market_month(g.config.radius_km);
            lost.month += 1;
        }
        assert!(lost.cargo.is_empty());
        assert_eq!(
            lost.events
                .iter()
                .filter(|e| e.kind == "cargo_weather_lost")
                .count(),
            3
        );
        assert!(lost.food_residual().abs() < 0.001);
        assert!(lost.economy_residuals().iter().all(|v| v.abs() < 0.001));

        // Declared material cargo starts in transit; a blocked fresh food shipment
        // loses more mass than preserved food, while tools retain their mass.
        let mut perishables = h.clone();
        perishables.society.as_mut().unwrap().routes[0].open = true;
        for site in &mut perishables.sites {
            site.economy.policy[3] = 0.;
        }
        let catalog = perishables.economy_catalog.as_ref().unwrap().clone();
        let goods: Vec<_> = ["fish", "preserved_food", "tools"]
            .map(|id| catalog.index(id).unwrap())
            .into();
        for &good in &goods {
            let source = &mut perishables.sites[route.from as usize];
            source.economy.initial[good] += 10.;
            for (k, ratio) in catalog.composition(good).iter().enumerate() {
                source.economy.baseline[k] += 10. * ratio;
            }
            perishables.cargo.push(crate::economy::Cargo {
                freight_stops: vec![],
                from: route.from,
                to: route.to,
                good: good as u32,
                kg: 10.,
                paid: 0.,
                arrives: perishables.month,
                sea_lane: None,
                weather_delay_months: 0,
            });
        }
        let mut total_loss = perishables.clone();
        total_loss.economy_catalog.as_mut().unwrap().goods[goods[0]].delay_spoilage = Some(1.);
        total_loss.economy_catalog.as_mut().unwrap().goods[goods[1]].delay_spoilage = Some(0.);
        total_loss.market_month(g.config.radius_km);
        assert!(!total_loss.cargo.iter().any(|c| c.good == goods[0] as u32));
        assert_eq!(
            total_loss
                .cargo
                .iter()
                .find(|c| c.good == goods[1] as u32)
                .unwrap()
                .kg,
            10.
        );
        assert!(total_loss
            .events
            .iter()
            .any(|e| e.kind == "cargo_spoilage_lost"));
        assert!(total_loss
            .economy_residuals()
            .iter()
            .all(|v| v.abs() < 0.001));
        let before = perishables.sites[route.to as usize].economy.goods;
        perishables.market_month(g.config.radius_km);
        let once = serde_json::to_value(&perishables.cargo).unwrap();
        perishables.market_month(g.config.radius_km);
        assert_eq!(once, serde_json::to_value(&perishables.cargo).unwrap());
        let mut resumed: crate::civilization::History =
            serde_json::from_value(serde_json::to_value(&perishables).unwrap()).unwrap();
        for world in [&mut perishables, &mut resumed] {
            world.month += 1;
            world.market_month(g.config.radius_km);
            assert_eq!(world.cargo.len(), 3);
            for &good in &goods {
                let cargo = world.cargo.iter().find(|c| c.good == good as u32).unwrap();
                let expected = 10. * (1. - catalog.delay_spoilage(good)).powi(2);
                assert!((cargo.kg - expected).abs() < 1e-5);
                assert_eq!(
                    world.sites[route.to as usize].economy.goods[good],
                    before[good]
                );
            }
            assert!(world.economy_residuals().iter().all(|v| v.abs() < 0.001));
            for r in &mut world.society.as_mut().unwrap().routes {
                r.flood_months = 0;
            }
            world.month += 1;
            world.market_month(g.config.radius_km);
            assert!(world.cargo.is_empty());
            for &good in &goods {
                let expected = 10. * (1. - catalog.delay_spoilage(good)).powi(2);
                assert!(
                    (world.sites[route.to as usize].economy.goods[good] - before[good] - expected)
                        .abs()
                        < 0.001
                );
            }
            assert!(world.economy_residuals().iter().all(|v| v.abs() < 0.001));
        }
        assert_eq!(
            serde_json::to_value(perishables).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );

        let mut chronic = h.clone();
        let mut unprotected = h.clone();
        // The earlier transport fixture can represent tens of metres of local
        // river exposure. Foundations only mitigate a shallow, persistent flood.
        let mut adaptation_cells = cells.clone();
        let cell = &mut adaptation_cells[chronic.sites[route.from as usize].cell as usize];
        cell.water[0] *= 0.75 / flood_depth(cell);
        // A declared brick grant funds adaptation; a paired settlement has none.
        chronic.sites[route.from as usize].economy.goods[5] += 5000.;
        chronic.sites[route.from as usize].economy.initial[5] += 5000.;
        for _ in 0..11 {
            chronic.month += 1;
            chronic.environmental_month(&adaptation_cells);
            unprotected.month += 1;
            unprotected.environmental_month(&adaptation_cells);
        }
        let impact = &chronic.living.as_ref().unwrap().floods[&route.from];
        assert!(impact.persistent);
        assert_eq!(impact.recovery_months, 0);
        assert!(impact.granary_bricks > 0.);
        assert!(
            chronic.sites[route.from as usize].stocks.stock[1]
                > unprotected.sites[route.from as usize].stocks.stock[1]
        );
        assert_eq!(
            chronic.sites[route.from as usize].demography.crops[0],
            unprotected.sites[route.from as usize].demography.crops[0]
        );
        assert!(chronic.economy_residuals().iter().all(|v| v.abs() < 0.001));
        assert_eq!(
            chronic
                .events
                .iter()
                .filter(|e| e.kind == "persistent_inundation" && e.site == Some(route.from))
                .count(),
            1
        );
        let mut continued: History =
            serde_json::from_value(serde_json::to_value(&chronic).unwrap()).unwrap();
        for _ in 0..12 {
            chronic.month += 1;
            continued.month += 1;
            chronic.environmental_month(&dry_delivery);
            continued.environmental_month(&dry_delivery);
        }
        assert!(!chronic.living.as_ref().unwrap().floods[&route.from].persistent);
        assert_eq!(
            serde_json::to_value(&chronic).unwrap(),
            serde_json::to_value(&continued).unwrap()
        );
        assert!(chronic.economy_residuals().iter().all(|v| v.abs() < 0.001));
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
        let dry = g.snapshot().unwrap();
        for _ in 0..3 {
            h.month += 1;
            h.environmental_month(&dry);
            resumed.month += 1;
            resumed.environmental_month(&dry);
        }
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert_eq!(h.society.as_ref().unwrap().routes[0].flood_months, 0);
        assert!(!h.society.as_ref().unwrap().routes[0].open);
        assert_eq!(h.shipping.as_ref().unwrap().ports[0].flood_months, 0);
        assert!(h
            .events
            .iter()
            .any(|e| e.kind == "flood_recovery" && !e.causes.is_empty()));
        h.validate(&dry).unwrap();
        let mut ruins = h.clone();
        for site in &mut ruins.sites {
            site.abandoned = true;
        }
        let events = ruins.events.len();
        ruins.month += 1;
        ruins.environmental_month(&cells);
        assert!(ruins
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .any(|r| r.flood_months > 0));
        for _ in 0..12 {
            ruins.month += 1;
            ruins.environmental_month(&dry);
        }
        assert_eq!(
            ruins.events.len(),
            events,
            "ruins must not report resident cleanup or inspection"
        );
        assert_eq!(ruins.society.as_ref().unwrap().routes[0].flood_months, 0);
    }
}
