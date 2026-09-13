//! Paid recovery of canonical bulk stocks using buyer-provided aggregate freight.
use crate::{
    civilization::History,
    economy::{Cargo, FOOD, GOODS},
};
use anyhow::{ensure, Result};

const MAX_RECOVERY_ONE_WAY_MONTHS: u32 = 6;
const MIN_RECOVERY_KG: f32 = 1.;
const MIN_RECOVERY_PRICE: f32 = 0.01;
const RECOVERY_FOOD_RESERVE_MONTHS: f32 = 6.;
const RECOVERY_ROAD_BASE_SHARE: f32 = 0.5;
const RECOVERY_ROAD_FULL_SURFACE_KG: f64 = 1000.;

impl History {
    pub(crate) fn recovery_route_open(&self, a: u32, b: u32) -> bool {
        self.society.as_ref().is_some_and(|s| {
            s.routes
                .iter()
                .any(|r| r.passable() && ((r.from == a && r.to == b) || (r.from == b && r.to == a)))
        })
    }

    /// After ordinary quarterly procurement: remaining cash and carrying capacity
    /// may obtain retained stock. Existing trading partners retain first access.
    pub(crate) fn recover_abandoned_stocks(&mut self) {
        if self.society.as_ref().is_none_or(|s| !s.stock_recovery) {
            return;
        }
        for buyer in 0..self.sites.len() {
            if self.sites[buyer].abandoned {
                continue;
            }
            for source in 0..self.sites.len() {
                if !self.sites[source].abandoned {
                    continue;
                }
                // Rotate goods to prevent a permanent material priority.
                for step in 0..GOODS {
                    let good = (step + self.month as usize) % GOODS;
                    let _ =
                        self.recover_abandoned_stock(buyer as u32, source as u32, good, f32::MAX);
                }
            }
        }
    }

    /// Reserve existing stock as cargo pending collection and return. No stock
    /// appears at the buyer before arrival. The source estate receives payment.
    /// Uses aggregate freight, not a newly simulated individual salvage crew.
    pub fn recover_abandoned_stock(
        &mut self,
        buyer: u32,
        source: u32,
        good: usize,
        requested: f32,
    ) -> Result<f32> {
        ensure!(
            requested.is_finite() && requested >= MIN_RECOVERY_KG,
            "invalid recovery quantity"
        );
        ensure!(
            buyer != source
                && (buyer as usize) < self.sites.len()
                && (source as usize) < self.sites.len(),
            "invalid recovery sites"
        );
        let catalog = self
            .economy_catalog
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("recovery needs an economy"))?;
        ensure!(
            good < GOODS
                && good < catalog.goods.len()
                && !catalog.goods[good].id.starts_with("reserved_"),
            "invalid recovery good"
        );
        let home = &self.sites[buyer as usize];
        let ruin = &self.sites[source as usize];
        ensure!(
            !home.abandoned
                && home.stocks.stock[0] > 0.
                && home.economy.policy[3] >= 0.5
                && ruin.abandoned
                && ruin.stocks.stock[0] == 0.,
            "recovery requires an inhabited buyer and an empty abandoned source"
        );
        ensure!(
            home.island == ruin.island && ruin.economy.policy[3] >= 0.5,
            "recovery requires the same inner continent and source trade permission"
        );
        let buyer_controller = self.controller(buyer);
        let source_controller = self.controller(source);
        ensure!(
            self.politics
                .as_ref()
                .is_none_or(|p| !p.wars.iter().any(|w| {
                    w.ended.is_none()
                        && ((w.attacker == buyer_controller && w.defender == source_controller)
                            || (w.defender == buyer_controller && w.attacker == source_controller))
                })),
            "recovery cannot trade across an active war"
        );
        ensure!(
            !self.besieged(buyer) && !self.besieged(source),
            "recovery cannot bypass a siege"
        );
        let society = self
            .society
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("recovery needs transport services"))?;
        let route = society
            .routes
            .iter()
            .find(|r| {
                r.passable()
                    && ((r.from == buyer && r.to == source) || (r.from == source && r.to == buyer))
            })
            .ok_or_else(|| anyhow::anyhow!("no passable direct recovery route"))?;
        ensure!(
            route.cost_km.is_finite() && route.cost_km >= 0.,
            "invalid recovery route distance"
        );
        let months = (route.cost_km / crate::society::LAND_TRAVEL_KM_PER_MONTH)
            .ceil()
            .max(1.) as u32;
        ensure!(
            months <= MAX_RECOVERY_ONE_WAY_MONTHS,
            "recovery exceeds round-trip range"
        );
        let edge = [buyer.min(source), buyer.max(source)];
        let road_used: f32 = self
            .cargo
            .iter()
            .filter(|c| c.freight_edges.contains(&edge))
            .map(|c| c.kg)
            .sum();
        // An empty endpoint cannot supply carriers. The buyer supplies them for
        // both legs; the existing road surface still limits throughput.
        let gross = home.stocks.stock[0] * catalog.production.land_freight_kg_per_person;
        let surface = if route.upkeep.is_some() {
            RECOVERY_ROAD_BASE_SHARE
                + (1. - RECOVERY_ROAD_BASE_SHARE)
                    * (route.road_bricks / RECOVERY_ROAD_FULL_SURFACE_KG).clamp(0., 1.) as f32
        } else {
            1.
        };
        let capacity = self
            .land_freight_capacity(buyer)
            .min((gross * surface - road_used).max(0.));
        let stock = if good == FOOD {
            ruin.stocks.stock[1]
        } else {
            ruin.economy.goods[good]
        };
        let incoming: f32 = self
            .cargo
            .iter()
            .filter(|c| c.to == buyer && c.good as usize == good)
            .map(|c| c.kg)
            .sum();
        let held = if good == FOOD {
            home.stocks.stock[1]
        } else {
            home.economy.goods[good]
        };
        let target = if good == FOOD {
            home.stocks.stock[0]
                * crate::economy::CIVILIAN_RESERVE_KG_PER_PERSON_MONTH
                * RECOVERY_FOOD_RESERVE_MONTHS
        } else {
            home.economy.targets[good]
        };
        let mut space = (target - held - incoming).max(0.);
        if good != FOOD && catalog.goods[good].food_energy <= 0. {
            let dry: f32 = home
                .economy
                .goods
                .iter()
                .enumerate()
                .filter(|(k, _)| *k != FOOD && catalog.goods[*k].food_energy <= 0.)
                .map(|(_, v)| *v)
                .sum();
            let pending: f32 = self
                .cargo
                .iter()
                .filter(|c| {
                    c.to == buyer
                        && c.good as usize != FOOD
                        && catalog.goods[c.good as usize].food_energy <= 0.
                })
                .map(|c| c.kg)
                .sum();
            space = space.min((home.economy.logistics[0] - dry - pending).max(0.));
        }
        let price = ruin.economy.prices[good].max(MIN_RECOVERY_PRICE);
        let kg = requested
            .min(stock)
            .min(space)
            .min(capacity)
            .min(home.economy.finance[0] / price);
        ensure!(
            kg.is_finite() && kg >= MIN_RECOVERY_KG,
            "insufficient recovery stock, demand, cash, storage or freight"
        );
        let paid = kg * price;
        let arrives = self
            .month
            .checked_add(months * 2)
            .ok_or_else(|| anyhow::anyhow!("recovery date overflow"))?;
        let mut path = route.cells.clone();
        if route.from != buyer {
            path.reverse();
        }
        let outward = path.clone();
        path.extend(outward.into_iter().rev().skip(1));
        self.sites[buyer as usize].economy.finance[0] -= paid;
        self.sites[buyer as usize].economy.finance[3] += paid;
        self.sites[source as usize].economy.finance[0] += paid;
        self.sites[source as usize].economy.finance[2] += paid;
        if good == FOOD {
            self.sites[source as usize].stocks.stock[1] -= kg;
        } else {
            self.sites[source as usize].economy.goods[good] -= kg;
        }
        self.cargo.push(Cargo {
            recovery: true,
            export_payment: None,
            infection: None,
            voyage_clock: None,
            freight_edges: vec![edge],
            freight_stops: edge.to_vec(),
            sea_lane: None,
            weather_delay_months: 0,
            from: source,
            to: buyer,
            good: good as u32,
            kg,
            paid,
            arrives,
        });
        self.event("stock_recovery_dispatched", Some(buyer), Some(source), format!("Reserved {kg:.2} kg of good {good} from the abandoned estate for {paid:.2}; buyer-provided aggregate freight returns in month {arrives}. Estate wallets, objects and ownership claims remain separate."));
        self.events.last_mut().unwrap().planned_path = Some(path);
        Ok(kg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU for founding fixture"]
    fn recovery_reserves_stock_capacity_and_preserves_estates() {
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
        g.enable_politics().unwrap();
        let politics = g.civilizations.as_ref().unwrap().politics.clone().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.politics = None;
        h.month = 3;
        h.cargo.clear();
        h.sites[1].civilization = h.sites[0].civilization;
        h.sites[1].island = h.sites[0].island;
        h.sites[1].abandoned = true;
        h.sites[1].stocks.stock[0] = 0.;
        h.sites[1].economy.goods[2] = 100.;
        h.sites[1].economy.prices[2] = 4.;
        h.sites[0].stocks.stock[0] = 100.;
        h.sites[0].economy.finance[0] = 1000.;
        h.sites[0].economy.goods.fill(0.);
        h.sites[0].economy.targets.fill(1000.);
        h.sites[0].economy.logistics = [10000., 0., 0., 1.];
        h.society.as_mut().unwrap().routes = vec![crate::society::Route {
            upkeep: None,
            id: 0,
            from: 0,
            to: 1,
            cells: vec![h.sites[0].cell, h.sites[1].cell],
            cost_km: 150.,
            open: true,
            flood_months: 0,
            road_bricks: 0.,
        }];
        let before = h.clone();
        let cash = h
            .sites
            .iter()
            .map(|s| s.economy.finance[0] as f64)
            .sum::<f64>();
        let capacity = h.land_freight_capacity(0);
        assert_eq!(h.recover_abandoned_stock(0, 1, 2, 60.).unwrap(), 60.);
        assert_eq!(h.sites[1].economy.goods[2], 40.);
        assert_eq!(h.sites[0].economy.goods[2], 0.);
        assert_eq!(h.cargo[0].kg, 60.);
        assert_eq!(h.cargo[0].arrives, 5);
        assert_eq!(h.land_freight_capacity(0), capacity - 60.);
        assert_eq!(
            h.sites
                .iter()
                .map(|s| s.economy.finance[0] as f64)
                .sum::<f64>(),
            cash
        );
        assert_eq!(
            serde_json::to_value(&h.society.as_ref().unwrap().household_economy).unwrap(),
            serde_json::to_value(&before.society.as_ref().unwrap().household_economy).unwrap()
        );
        assert_eq!(h.recover_abandoned_stock(0, 1, 2, 100.).unwrap(), 40.);
        let unchanged = serde_json::to_value(&h).unwrap();
        assert!(h.recover_abandoned_stock(0, 1, 2, 1.).is_err());
        assert_eq!(serde_json::to_value(&h).unwrap(), unchanged);
        let mut foreign = before.clone();
        foreign.sites[1].civilization = 1;
        foreign.politics = Some(politics);
        foreign.politics.as_mut().unwrap().controllers[0] = 0;
        foreign.politics.as_mut().unwrap().controllers[1] = 1;
        let peace = foreign.clone();
        assert_eq!(foreign.recover_abandoned_stock(0, 1, 2, 10.).unwrap(), 10.);
        for reverse in [false, true] {
            let mut hostile = peace.clone();
            hostile
                .politics
                .as_mut()
                .unwrap()
                .wars
                .push(crate::politics::War {
                    name: "Test war".into(),
                    id: 0,
                    attacker: u32::from(reverse),
                    defender: u32::from(!reverse),
                    goal: 1,
                    started: 1,
                    ended: None,
                    outcome: String::new(),
                    cause: 0,
                });
            let snapshot = serde_json::to_value(&hostile).unwrap();
            assert!(hostile.recover_abandoned_stock(0, 1, 2, 10.).is_err());
            assert_eq!(snapshot, serde_json::to_value(&hostile).unwrap());
            hostile
                .politics
                .as_mut()
                .unwrap()
                .wars
                .last_mut()
                .unwrap()
                .ended = Some(2);
            assert_eq!(hostile.recover_abandoned_stock(0, 1, 2, 10.).unwrap(), 10.);
        }
        for change in 0..6 {
            let mut blocked = before.clone();
            match change {
                0 => blocked.society.as_mut().unwrap().routes[0].open = false,
                1 => blocked.sites[0].economy.finance[0] = 0.,
                2 => blocked.sites[1].abandoned = false,
                3 => blocked.sites[1].economy.policy[3] = 0.,
                4 => blocked.sites[1].island = u32::MAX,
                _ => blocked.sites[0].economy.targets[2] = 0.,
            }
            let snapshot = serde_json::to_value(&blocked).unwrap();
            assert!(blocked.recover_abandoned_stock(0, 1, 2, 10.).is_err());
            assert_eq!(snapshot, serde_json::to_value(&blocked).unwrap());
        }
        let mut automatic = before.clone();
        automatic.recover_abandoned_stocks();
        assert!(automatic.cargo.is_empty());
        automatic.society.as_mut().unwrap().stock_recovery = true;
        automatic
            .economy_catalog
            .as_mut()
            .unwrap()
            .production
            .land_freight_kg_per_person = 0.1;
        automatic.recover_abandoned_stocks();
        assert_eq!(automatic.cargo.iter().map(|c| c.kg).sum::<f32>(), 10.);
        automatic.recover_abandoned_stocks();
        assert_eq!(automatic.cargo.iter().map(|c| c.kg).sum::<f32>(), 10.);
        let mut old = serde_json::to_value(&automatic).unwrap();
        old["society"]
            .as_object_mut()
            .unwrap()
            .remove("stock_recovery");
        for c in old["cargo"].as_array_mut().unwrap() {
            c.as_object_mut().unwrap().remove("recovery");
        }
        let imported: History = serde_json::from_value(old).unwrap();
        assert!(!imported.society.as_ref().unwrap().stock_recovery);
        assert!(imported.cargo.iter().all(|c| !c.recovery));
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
        for month in 4..=6 {
            for state in [&mut *h, &mut resumed] {
                state.month = month;
                // A closed road at the due boundary delays delivery.
                state.society.as_mut().unwrap().routes[0].open = month != 5;
                state.market_arrivals();
            }
            assert_eq!(
                serde_json::to_value(&h).unwrap(),
                serde_json::to_value(&resumed).unwrap()
            );
            if month < 6 {
                assert_eq!(h.sites[0].economy.goods[2], 0.);
            }
        }
        assert!(h.cargo.is_empty());
        assert_eq!(h.sites[0].economy.goods[2], 100.);
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "stock_recovery_arrival")
                .count(),
            2
        );
        assert_eq!(
            serde_json::to_value(&h.trade_contact).unwrap(),
            serde_json::to_value(&before.trade_contact).unwrap()
        );
    }
}
