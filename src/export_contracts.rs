//! Funded repeat procurement. Production remains GPU work; dispatch uses normal markets.
use crate::{
    civilization::History,
    economy::{Cargo, FOOD},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportContract {
    pub buyer: u32,
    pub seller: u32,
    pub good: u32,
    pub deliveries: u32,
    pub last_delivery: u32,
    pub observed_kg: f32,
    pub unit_price: f32,
    pub remaining_kg: f32,
    pub escrow: f32,
    pub expires: u32,
    /// Snapshot of the export addition to this month's production target.
    pub planned_kg: f32,
    pub dispatched_kg: f32,
    #[serde(default)]
    pub estimated_unit_cost: f32,
    #[serde(default)]
    pub declined: u32,
    /// Estimated contribution at accepted prices, not a cash account or realized profit.
    #[serde(default)]
    pub quoted_surplus: f32,
}
impl History {
    /// Replacement-cost quote. Labor is a food-valued opportunity cost, not paid wages.
    /// Returns None without a modeled extraction/recipe cost; a stored item's own
    /// asking price is not independent evidence of its production cost.
    pub fn supplier_unit_cost(&self, site: usize, good: usize) -> Option<f32> {
        self.supplier_cost_for_quantity(site, good, 1.)
    }
    fn supplier_cost_for_quantity(&self, site: usize, good: usize, quantity: f32) -> Option<f32> {
        let catalog = self.economy_catalog.as_ref()?;
        let e = &self.sites.get(site)?.economy;
        let item = catalog.goods.get(good)?;
        let price = |k: usize| {
            e.prices[k].max(if catalog.market.adaptive_prices {
                0.0001
            } else {
                catalog.goods[k].base_price * 0.4
            })
        };
        let labor = 18. * price(FOOD);
        let raw = match item.id.as_str() {
            "wood" if e.forest[0] > 0. && e.forest[1] > 0. && e.forest[2] > 0. => {
                Some(labor / 20. + item.base_price * 0.25)
            }
            _ if good == e.extraction[0].max(1.) as usize
                && self.accessible_resources(site)[0] > 0. =>
            {
                Some(labor / 5. + item.base_price * 0.25)
            }
            "clay" if self.accessible_resources(site)[1] > 0. => {
                Some(labor / 5. + item.base_price * 0.25)
            }
            _ => None,
        };
        let recipe = catalog
            .recipes
            .iter()
            .filter(|r| {
                r.output[good] > 0.
                    && (r.work[1] == 0.
                        || e.management[3] as u32 & (1 << (r.work[1] as u32 - 1)) != 0)
                    && (r.input[44] == 0. || e.goods[44] >= r.input[44] * quantity / r.output[good])
                    && (r.input[42] == 0. || e.goods[42] >= r.input[42] * quantity / r.output[good])
                    && (r.input[29] == 0. || e.goods[29] >= r.input[29] * quantity / r.output[good])
            })
            .map(|r| {
                // Conservative allocation: no speculative revenue from unsold byproducts.
                let inputs: f32 = r
                    .input
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| **v > 0.)
                    .map(|(k, v)| v * price(k))
                    .sum();
                let upkeep = if catalog.production.workshops {
                    (20. * price(0) + 30. * price(5) + 2. * price(3)) * 0.002 / 4.
                } else {
                    0.
                };
                (inputs + r.work[0] * (labor + upkeep)) / r.output[good]
            })
            .min_by(f32::total_cmp);
        raw.into_iter().chain(recipe).min_by(f32::total_cmp)
    }
    pub(crate) fn observe_export_delivery(&mut self, cargo: &Cargo) {
        let Some(catalog) = &self.economy_catalog else {
            return;
        };
        if !catalog.production.enabled
            || !catalog.production.export_contracts
            || cargo.good as usize == FOOD
            || catalog.goods[cargo.good as usize].food_energy > 0.
            || cargo.kg < 1.
        {
            return;
        }
        if let Some(c) = self
            .export_contracts
            .iter_mut()
            .find(|c| c.buyer == cargo.to && c.good == cargo.good)
        {
            if c.seller == cargo.from {
                c.deliveries = c.deliveries.saturating_add(1);
                c.last_delivery = self.month;
                c.observed_kg = c.observed_kg * 0.75 + cargo.kg * 0.25;
                if c.escrow == 0. {
                    c.unit_price = (cargo.paid / cargo.kg).max(0.01);
                }
            }
        } else {
            self.export_contracts.push(ExportContract {
                buyer: cargo.to,
                seller: cargo.from,
                good: cargo.good,
                deliveries: 1,
                last_delivery: self.month,
                observed_kg: cargo.kg,
                unit_price: (cargo.paid / cargo.kg).max(0.01),
                remaining_kg: 0.,
                escrow: 0.,
                expires: 0,
                planned_kg: 0.,
                dispatched_kg: 0.,
                estimated_unit_cost: 0.,
                declined: 0,
                quoted_surplus: 0.,
            });
        }
    }
    pub(crate) fn local_production_target(&self, site: usize, good: usize) -> f32 {
        let exports: f32 = self
            .export_contracts
            .iter()
            .filter(|c| c.seller as usize == site && c.good as usize == good)
            .map(|c| c.planned_kg)
            .sum();
        (self.sites[site].economy.targets[good] - exports).max(0.)
    }
    pub(crate) fn expire_export_contracts(&mut self) {
        let enabled = self
            .economy_catalog
            .as_ref()
            .is_some_and(|c| c.production.enabled && c.production.export_contracts);
        for i in 0..self.export_contracts.len() {
            let c = &self.export_contracts[i];
            if c.escrow > 0.
                && (!enabled
                    || self.month >= c.expires
                    || c.remaining_kg < 0.001
                    || [c.buyer, c.seller].iter().any(|s| {
                        self.sites[*s as usize].abandoned
                            || self.sites[*s as usize].economy.policy[3] < 0.5
                    }))
            {
                let c = &mut self.export_contracts[i];
                let (buyer, seller, refund) = (c.buyer, c.seller, c.escrow);
                self.sites[buyer as usize].economy.finance[0] += refund;
                c.escrow = 0.;
                c.remaining_kg = 0.;
                self.event(
                    "export_refund",
                    Some(buyer),
                    Some(seller),
                    format!("Returned {refund:.2} unspent procurement escrow"),
                );
            }
        }
        // Evidence expires too; a new successful supplier may then establish a relationship.
        self.export_contracts.retain(|c| {
            c.escrow > 0. || c.planned_kg > 0. || self.month.saturating_sub(c.last_delivery) <= 24
        });
    }
    pub(crate) fn fund_export_contracts(&mut self, reachable: &[bool]) {
        if !self
            .economy_catalog
            .as_ref()
            .is_some_and(|c| c.production.enabled && c.production.export_contracts)
        {
            return;
        }
        // Across all goods, reserve at most 20% of currently liquid town cash per quarter.
        let mut budgets: Vec<f32> = self
            .sites
            .iter()
            .map(|s| s.economy.finance[0] * 0.2)
            .collect();
        for (i, &reachable) in reachable.iter().enumerate() {
            let c = &self.export_contracts[i];
            let (buyer, seller, k) = (c.buyer as usize, c.seller as usize, c.good as usize);
            if !reachable
                || c.deliveries < 2
                || c.escrow > 0.
                || self.month.saturating_sub(c.last_delivery) > 12
                || [buyer, seller]
                    .iter()
                    .any(|s| self.sites[*s].abandoned || self.sites[*s].economy.policy[3] < 0.5)
                || self.sites[buyer].stocks.stock[1] < self.sites[buyer].stocks.stock[0] * 18. * 6.
            {
                continue;
            }
            let need = (self.local_production_target(buyer, k)
                - self.sites[buyer].economy.goods[k])
                .max(0.);
            if need < 1. {
                continue;
            }
            let mut price = c.unit_price;
            let mut estimate = 0.;
            let catalog = self.economy_catalog.as_ref().unwrap();
            if catalog.production.supplier_profitability {
                let requested = need
                    .min(c.observed_kg * 1.25)
                    .min(self.sites[buyer].stocks.stock[0] * 2.);
                let cost = self.supplier_cost_for_quantity(seller, k, requested);
                let ceiling =
                    self.sites[buyer].economy.prices[k].max(catalog.goods[k].base_price * 0.4);
                let proposal =
                    cost.map(|v| price.max(v * (1. + catalog.production.contract_margin)));
                if proposal.is_none_or(|v| v > ceiling || !v.is_finite()) {
                    self.export_contracts[i].declined =
                        self.export_contracts[i].declined.saturating_add(1);
                    continue;
                }
                price = proposal.unwrap();
                estimate = cost.unwrap();
            }
            let c = &self.export_contracts[i];
            let amount = need
                .min(c.observed_kg * 1.25)
                .min(self.sites[buyer].stocks.stock[0] * 2.)
                .min(budgets[buyer] / price);
            if amount < 1. {
                continue;
            }
            let cost = amount * price;
            self.sites[buyer].economy.finance[0] -= cost;
            budgets[buyer] -= cost;
            let c = &mut self.export_contracts[i];
            c.unit_price = price;
            c.estimated_unit_cost = estimate;
            c.remaining_kg = amount;
            c.escrow = cost;
            c.expires = self.month + 6;
            self.event(
                "export_contract",
                Some(buyer as u32),
                Some(seller as u32),
                format!(
                    "Funded {amount:.1} kg {} with {cost:.2} escrow; expires in six months",
                    self.economy_catalog.as_ref().unwrap().goods[k].name
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        civilization::{Site, Stocks},
        economy::{Economy, EconomyCatalog},
        society::Demography,
    };
    fn fixture() -> History {
        let mut catalog = EconomyCatalog::bundled().unwrap();
        catalog.market.network_trade = false;
        catalog.production.supplier_profitability = false;
        History {
            resolution: None,
            person_duties: Default::default(),
            domestic: None,
            named_demography: None,
            military: Default::default(),
            participation: None,
            territorial_history: vec![],
            enterprises: None,
            experimental_tool_reserves: Default::default(),
            resources: None,
            offices: None,
            farming_mode: None,
            culture: None,
            living: None,
            version: 2,
            seed: 17,
            terrain_resolution: 64,
            source_epoch: 0,
            month: 3,
            civilizations: vec![],
            sites: (0..2)
                .map(|id| {
                    let mut economy = Economy {
                        policy: [0., 0., 0., 1.],
                        finance: [1000., 1000., 0., 0.],
                        logistics: [10000., 0., 0., 2.],
                        ..Default::default()
                    };
                    economy.targets[3] = 75.;
                    if id == 0 {
                        economy.goods[3] = 200.;
                        economy.initial[3] = 200.;
                    }
                    Site {
                        id,
                        civilization: 0,
                        island: 0,
                        cell: id,
                        name: format!("Fixture {id}"),
                        founded: 0,
                        abandoned: false,
                        lifecycle: Default::default(),
                        stocks: Stocks {
                            stock: [100., 24000., 0., 0.],
                            ..bytemuck::Zeroable::zeroed()
                        },
                        economy,
                        demography: Demography::default(),
                    }
                })
                .collect(),
            people: vec![],
            events: vec![],
            shipments: vec![],
            candidates: vec![],
            initial_food: 48000.,
            initial_population: 200.,
            economy_catalog: Some(catalog),
            nutrition_initial: [21600., 960., 144.],
            cargo: vec![],
            export_contracts: vec![],
            society: None,
            politics: None,
            governance: None,
            shipping: None,
            expeditions: None,
        }
    }
    fn evidence(h: &mut History) {
        let delivery = Cargo {
            voyage_clock: None,
            freight_stops: vec![],
            from: 0,
            to: 1,
            good: 3,
            kg: 10.,
            paid: 40.,
            arrives: 0,
            sea_lane: None,
            weather_delay_months: 0,
        };
        h.observe_export_delivery(&delivery);
    }
    #[test]
    fn repeated_deliveries_required_and_escrow_refunds_without_minting_money() {
        let mut h = fixture();
        evidence(&mut h);
        h.fund_export_contracts(&[true]);
        assert_eq!(h.export_contracts[0].escrow, 0.);
        evidence(&mut h);
        h.fund_export_contracts(&[false]);
        assert_eq!(h.export_contracts[0].escrow, 0.);
        h.fund_export_contracts(&[true]);
        let c = &h.export_contracts[0];
        assert!(c.escrow > 0. && c.escrow <= 200.);
        assert!((h.sites[1].economy.finance[0] + c.escrow - 1000.).abs() < 0.001);
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
        let money = h.sites[1].economy.finance[0];
        h.fund_export_contracts(&[true]);
        assert_eq!(money, h.sites[1].economy.finance[0]);
        h.month = 9;
        h.expire_export_contracts();
        assert_eq!(h.export_contracts[0].escrow, 0.);
        assert_eq!(h.sites[1].economy.finance[0], 1000.);
        h.expire_export_contracts();
        assert_eq!(h.sites[1].economy.finance[0], 1000.);
    }
    #[test]
    fn contract_dispatch_pays_from_escrow_once_and_uses_normal_cargo() {
        let mut h = fixture();
        evidence(&mut h);
        evidence(&mut h);
        h.fund_export_contracts(&[true]);
        let escrow = h.export_contracts[0].escrow;
        h.market_month(6371.);
        assert!(h.export_contracts[0].escrow < escrow);
        assert!(h.export_contracts[0].dispatched_kg > 0.);
        assert!(h
            .cargo
            .iter()
            .any(|c| c.from == 0 && c.to == 1 && c.good == 3));
        assert!(
            h.economy_residuals().iter().all(|v| v.abs() < 0.001),
            "{:?}",
            h.economy_residuals()
        );
        let paid = h.sites[0].economy.finance[0];
        h.month += 1;
        h.market_month(6371.);
        assert_eq!(h.sites[0].economy.finance[0], paid);
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    }
    #[test]
    fn funded_exports_do_not_become_local_import_targets_and_disable_refunds() {
        let mut h = fixture();
        evidence(&mut h);
        evidence(&mut h);
        h.fund_export_contracts(&[true]);
        h.sites[0].economy.goods[3] = 0.;
        h.sites[0].economy.initial[3] = 0.;
        let mut local = h.clone();
        local.export_contracts.clear();
        local.plan_production();
        h.plan_production();
        let tools = h
            .economy_catalog
            .as_ref()
            .unwrap()
            .recipes
            .iter()
            .position(|r| r.output[3] > 0.)
            .unwrap();
        assert!(h.sites[0].economy.orders[tools] > local.sites[0].economy.orders[tools]);
        assert!(h.sites[1].economy.orders[tools] < local.sites[1].economy.orders[tools]);
        let c = &h.export_contracts[0];
        assert!(c.planned_kg > 0.);
        assert!((h.local_production_target(0, 3) - 75.).abs() < 0.001);
        h.economy_catalog
            .as_mut()
            .unwrap()
            .production
            .export_contracts = false;
        h.plan_production();
        h.expire_export_contracts();
        assert_eq!(h.export_contracts[0].planned_kg, 0.);
        assert_eq!(h.sites[1].economy.finance[0], 1000.);
    }
    #[test]
    fn suppliers_reject_underpriced_work_then_negotiate_a_funded_margin() {
        let mut h = fixture();
        evidence(&mut h);
        evidence(&mut h);
        h.economy_catalog
            .as_mut()
            .unwrap()
            .production
            .supplier_profitability = true;
        h.sites[0].economy.prices[2] = 100.;
        h.sites[1].economy.prices[3] = 1.;
        h.fund_export_contracts(&[true]);
        assert_eq!(h.export_contracts[0].escrow, 0.);
        assert_eq!(h.export_contracts[0].declined, 1);
        assert_eq!(h.sites[1].economy.finance[0], 1000.);
        h.sites[1].economy.prices[3] = 200.;
        h.fund_export_contracts(&[true]);
        let c = &h.export_contracts[0];
        assert!(c.escrow > 0. && c.escrow <= 200.);
        assert!(c.unit_price >= c.estimated_unit_cost * 1.0999);
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
        h.month = 9;
        h.expire_export_contracts();
        assert_eq!(h.sites[1].economy.finance[0], 1000.);
    }
    #[test]
    fn cost_estimates_respect_knowledge_scrap_and_input_prices() {
        let mut h = fixture();
        assert!(h.supplier_unit_cost(0, 18).is_none());
        h.sites[0].economy.management[3] = (1u32 << 5) as f32;
        assert!(h.supplier_unit_cost(0, 18).is_some());
        let virgin = h.supplier_unit_cost(0, 2).unwrap();
        h.sites[0].economy.goods[29] = 2.;
        assert!(h.supplier_unit_cost(0, 2).unwrap() < virgin);
        assert_eq!(h.supplier_cost_for_quantity(0, 2, 100.).unwrap(), virgin);
        let cheap = h.supplier_unit_cost(0, 3).unwrap();
        h.sites[0].economy.prices[2] = 100.;
        assert!(h.supplier_unit_cost(0, 3).unwrap() > cheap);
    }
    #[test]
    fn old_contracts_and_catalogs_do_not_acquire_profit_rules_or_money() {
        let mut h = fixture();
        evidence(&mut h);
        let mut data = serde_json::to_value(&h).unwrap();
        for key in ["estimated_unit_cost", "quoted_surplus", "declined"] {
            data["export_contracts"][0]
                .as_object_mut()
                .unwrap()
                .remove(key);
        }
        data["economy_catalog"]["production"]
            .as_object_mut()
            .unwrap()
            .remove("supplier_profitability");
        data["economy_catalog"]["production"]
            .as_object_mut()
            .unwrap()
            .remove("contract_margin");
        let old: History = serde_json::from_value(data).unwrap();
        assert!(
            !old.economy_catalog
                .as_ref()
                .unwrap()
                .production
                .supplier_profitability
        );
        assert_eq!(old.export_contracts[0].estimated_unit_cost, 0.);
        assert_eq!(old.sites[1].economy.finance[0], 1000.);
        let mut bad = old.economy_catalog.unwrap();
        bad.production.contract_margin = f32::NAN;
        assert!(bad.validate().is_err());
    }
}
