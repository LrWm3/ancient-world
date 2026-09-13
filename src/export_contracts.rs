//! Funded repeat procurement. Production remains GPU work; dispatch uses normal markets.
use crate::{
    civilization::History,
    economy::{Cargo, FOOD},
};
use serde::{Deserialize, Serialize};

const MIN_ADAPTIVE_QUOTE_PRICE: f32 = 0.0001;
const MIN_BASE_QUOTE_FRACTION: f32 = 0.4;
const WORK_OPPORTUNITY_FOOD_KG_PER_MONTH: f32 = 18.;
const WOOD_OUTPUT_KG_PER_WORKER_MONTH: f32 = 20.;
const MINERAL_OUTPUT_KG_PER_WORKER_MONTH: f32 = 5.;
const RAW_RESOURCE_OPPORTUNITY_PRICE_FRACTION: f32 = 0.25;
const MIN_OBSERVED_DELIVERY_KG: f32 = 1.;
const DELIVERY_ESTIMATE_RETENTION: f32 = 0.75;
const DELIVERY_ESTIMATE_NEW_WEIGHT: f32 = 0.25;
const MIN_OBSERVED_UNIT_PRICE: f32 = 0.01;
const COMPLETION_REMAINDER_KG: f32 = 0.001;
const DELIVERY_EVIDENCE_MAX_AGE_MONTHS: u32 = 24;
const QUARTERLY_ESCROW_CASH_FRACTION: f32 = 0.2;
const MIN_CONTRACT_DELIVERIES: u32 = 2;
const CONTRACT_MAX_EVIDENCE_AGE_MONTHS: u32 = 12;
const BUYER_FOOD_RESERVE_MONTHS: f32 = 6.;
const MIN_CONTRACT_ORDER_KG: f32 = 1.;
const OBSERVED_ORDER_HEADROOM: f32 = 1.25;
const MAX_ORDER_KG_PER_PERSON: f32 = 2.;
const CONTRACT_DURATION_MONTHS: u32 = 6;

pub mod identities;
pub mod payments;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportContract {
    #[serde(default)]
    pub payment_timing: payments::Timing,
    /// Missing only for archives predating persistent source identities.
    #[serde(default)]
    pub id: Option<u64>,
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
                MIN_ADAPTIVE_QUOTE_PRICE
            } else {
                catalog.goods[k].base_price * MIN_BASE_QUOTE_FRACTION
            })
        };
        let labor = WORK_OPPORTUNITY_FOOD_KG_PER_MONTH * price(FOOD);
        let raw = match item.id.as_str() {
            "wood" if e.forest[0] > 0. && e.forest[1] > 0. && e.forest[2] > 0. => Some(
                labor / WOOD_OUTPUT_KG_PER_WORKER_MONTH
                    + item.base_price * RAW_RESOURCE_OPPORTUNITY_PRICE_FRACTION,
            ),
            _ if good == e.extraction[0].max(1.) as usize
                && self.accessible_resources(site)[0] > 0. =>
            {
                Some(
                    labor / MINERAL_OUTPUT_KG_PER_WORKER_MONTH
                        + item.base_price * RAW_RESOURCE_OPPORTUNITY_PRICE_FRACTION,
                )
            }
            "clay" if self.accessible_resources(site)[1] > 0. => Some(
                labor / MINERAL_OUTPUT_KG_PER_WORKER_MONTH
                    + item.base_price * RAW_RESOURCE_OPPORTUNITY_PRICE_FRACTION,
            ),
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
                    (crate::production::WORKSHOP_WOOD_KG_PER_UNIT * price(0)
                        + crate::production::WORKSHOP_BRICKS_KG_PER_UNIT * price(5)
                        + crate::production::WORKSHOP_TOOLS_KG_PER_UNIT * price(3))
                        * crate::production::WORKSHOP_MONTHLY_WEAR
                        / crate::production::WORKSHOP_WORKER_MONTHS_PER_UNIT
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
            || cargo.kg < MIN_OBSERVED_DELIVERY_KG
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
                c.observed_kg = c.observed_kg * DELIVERY_ESTIMATE_RETENTION
                    + cargo.kg * DELIVERY_ESTIMATE_NEW_WEIGHT;
                if c.escrow == 0. {
                    c.unit_price = (cargo.paid / cargo.kg).max(MIN_OBSERVED_UNIT_PRICE);
                }
            }
        } else {
            let id = self.create_export_identity(cargo.to, cargo.from, cargo.good, false);
            self.export_contracts.push(ExportContract {
                id: Some(id),
                payment_timing: payments::Timing::Dispatch,
                buyer: cargo.to,
                seller: cargo.from,
                good: cargo.good,
                deliveries: 1,
                last_delivery: self.month,
                observed_kg: cargo.kg,
                unit_price: (cargo.paid / cargo.kg).max(MIN_OBSERVED_UNIT_PRICE),
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
                    || c.remaining_kg < COMPLETION_REMAINDER_KG
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
        let retained = |c: &ExportContract| {
            c.escrow > 0.
                || c.planned_kg > 0.
                || self.month.saturating_sub(c.last_delivery) <= DELIVERY_EVIDENCE_MAX_AGE_MONTHS
        };
        for contract in &self.export_contracts {
            if !retained(contract) {
                if let Some(identity) = contract
                    .id
                    .and_then(|id| self.export_identities.get_mut(id as usize))
                {
                    identity.retired_month = Some(self.month);
                }
            }
        }
        self.export_contracts.retain(retained);
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
            .map(|s| s.economy.finance[0] * QUARTERLY_ESCROW_CASH_FRACTION)
            .collect();
        for (i, &reachable) in reachable.iter().enumerate() {
            let c = &self.export_contracts[i];
            let (buyer, seller, k) = (c.buyer as usize, c.seller as usize, c.good as usize);
            if !reachable
                || c.deliveries < MIN_CONTRACT_DELIVERIES
                || c.escrow > 0.
                || self.month.saturating_sub(c.last_delivery) > CONTRACT_MAX_EVIDENCE_AGE_MONTHS
                || [buyer, seller]
                    .iter()
                    .any(|s| self.sites[*s].abandoned || self.sites[*s].economy.policy[3] < 0.5)
                || self.sites[buyer].stocks.stock[1]
                    < self.sites[buyer].stocks.stock[0]
                        * crate::economy::CIVILIAN_RESERVE_KG_PER_PERSON_MONTH
                        * BUYER_FOOD_RESERVE_MONTHS
            {
                continue;
            }
            let need = (self.local_production_target(buyer, k)
                - self.sites[buyer].economy.goods[k])
                .max(0.);
            if need < MIN_CONTRACT_ORDER_KG {
                continue;
            }
            let mut price = c.unit_price;
            let mut estimate = 0.;
            let catalog = self.economy_catalog.as_ref().unwrap();
            if catalog.production.supplier_profitability {
                let requested = need
                    .min(c.observed_kg * OBSERVED_ORDER_HEADROOM)
                    .min(self.sites[buyer].stocks.stock[0] * MAX_ORDER_KG_PER_PERSON);
                let cost = self.supplier_cost_for_quantity(seller, k, requested);
                let ceiling = self.sites[buyer].economy.prices[k]
                    .max(catalog.goods[k].base_price * MIN_BASE_QUOTE_FRACTION);
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
                .min(c.observed_kg * OBSERVED_ORDER_HEADROOM)
                .min(self.sites[buyer].stocks.stock[0] * MAX_ORDER_KG_PER_PERSON)
                .min(budgets[buyer] / price);
            if amount < MIN_CONTRACT_ORDER_KG {
                continue;
            }
            let cost = amount * price;
            self.sites[buyer].economy.finance[0] -= cost;
            budgets[buyer] -= cost;
            let c = &mut self.export_contracts[i];
            c.payment_timing = self.export_payment_timing;
            c.unit_price = price;
            c.estimated_unit_cost = estimate;
            c.remaining_kg = amount;
            c.escrow = cost;
            c.expires = self.month + CONTRACT_DURATION_MONTHS;
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
            credit: Default::default(),
            contagion: None,
            trade_contact: Default::default(),
            resolution: None,
            person_duties: Default::default(),
            service_allocation: Default::default(),
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
            export_identities: vec![],
            export_payments: vec![],
            export_payment_timing: Default::default(),
            society: None,
            politics: None,
            governance: None,
            shipping: None,
            expeditions: None,
        }
    }
    fn evidence(h: &mut History) {
        let delivery = Cargo {
            export_payment: None,
            infection: None,
            voyage_clock: None,
            freight_edges: vec![],
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
    fn automatic_commercial_credit_requires_gap_and_preserves_lender_reserve() {
        let mut h = fixture();
        h.export_payment_timing = payments::Timing::Delivery;
        evidence(&mut h);
        evidence(&mut h);
        h.fund_export_contracts(&[true]);
        h.market_month(6371.);
        h.credit.commercial_policy.enabled = true;
        let recipe = &mut h.economy_catalog.as_mut().unwrap().recipes[0];
        recipe.input.fill(0.);
        recipe.output.fill(0.);
        recipe.input[1] = 10.;
        recipe.output[2] = 1.;
        h.sites[0].economy.orders.fill(0.);
        h.sites[0].economy.orders[0] = 1.;
        h.sites[0].economy.prices[1] = 2.;
        assert_eq!(h.commercial_input_costs()[0], 20.);
        let mut rich = h.clone();
        assert_eq!(rich.commercial_credit_month().unwrap(), 0);
        assert!(rich.credit.loans.is_empty());
        h.sites[0].economy.finance[0] = 0.;
        h.sites[0].economy.finance[1] = 0.;
        let mut reserved = h.clone();
        reserved.credit.commercial_policy.operating_cash_floor = 10000.;
        assert_eq!(reserved.commercial_credit_month().unwrap(), 0);
        let mut distant = h.clone();
        distant.export_payments[0].expected_month = h.month + 121;
        distant.cargo[0].arrives = h.month + 121;
        assert_eq!(distant.commercial_credit_month().unwrap(), 0);
        let mut delayed = h.clone();
        delayed.cargo[0].arrives += 1;
        assert_eq!(delayed.commercial_credit_month().unwrap(), 0);
        let mut disabled = h.clone();
        disabled.credit.commercial_policy.enabled = false;
        assert_eq!(disabled.commercial_credit_month().unwrap(), 0);
        assert!(h.commercial_credit_month().unwrap() > 0);
        let principal: f64 = h.credit.loans.iter().map(|l| l.original_principal).sum();
        assert!(principal > 0. && principal <= 20.);
        assert!(h.sites[1].economy.finance[0] >= 100.);
        h.validate_credit().unwrap();
        assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
        let once = serde_json::to_value(&h).unwrap();
        assert_eq!(h.commercial_credit_month().unwrap(), 0);
        assert_eq!(once, serde_json::to_value(&h).unwrap());
        let mut resumed: History = serde_json::from_value(once.clone()).unwrap();
        assert_eq!(resumed.commercial_credit_month().unwrap(), 0);
        assert_eq!(once, serde_json::to_value(&resumed).unwrap());
    }

    #[test]
    fn delivery_proceeds_back_bounded_credit_and_repay_after_arrival() {
        use crate::credit::{
            underwriting::{Offer, Policy, Request},
            Account, Status, Terms, SHARED_CURRENCY,
        };
        let mut h = fixture();
        h.export_payment_timing = payments::Timing::Delivery;
        evidence(&mut h);
        evidence(&mut h);
        h.fund_export_contracts(&[true]);
        h.market_month(6371.);
        let payment_month = h.export_payments[0].expected_month;
        let mut observations = h.export_credit_evidence(0.).unwrap();
        observations[0].operating_costs = 20.;
        let source_budget = (observations[0].expected_receipts - 20.) * 0.5;
        assert!(source_budget > 0.);
        for id in 0..2 {
            let offer = Offer {
                lender: Account::Town(1),
                month: h.month,
                cash: h.credit_account_cash(Account::Town(1)).unwrap(),
                operating_reserve: 100.,
                offered_principal: 1000.,
                minimum_annual_rate: 0.,
            };
            let request = Request {
                id,
                month: h.month,
                principal: 1000.,
                terms: Terms {
                    lender: Account::Town(1),
                    borrower: Account::Town(0),
                    currency: SHARED_CURRENCY,
                    source: observations[0].source,
                    annual_simple_rate: 0.,
                    maturity_month: payment_month + 1,
                    grace_months: 3,
                },
            };
            h.fund_credit_requests(
                Policy::default(),
                vec![offer],
                observations.clone(),
                vec![request],
            )
            .unwrap();
        }
        assert!(!h.credit.loans.is_empty());
        let borrowed: f64 = h.credit.loans.iter().map(|l| l.original_principal).sum();
        assert!(borrowed > 0. && borrowed <= source_budget);
        assert!((borrowed - source_budget).abs() < 0.001);
        h.validate_credit().unwrap();
        assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
        h.month = payment_month;
        h.market_arrivals();
        h.settle_export_payments().unwrap();
        h.month += 1;
        h.credit.servicing_policy.available_cash_share = 1.;
        h.service_credit_month().unwrap();
        assert!(h.credit.loans.iter().all(|l| l.status == Status::Repaid));
        h.validate_credit().unwrap();
        h.validate_export_payments().unwrap();
        assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
    }

    #[test]
    fn delayed_export_workout_uses_funded_proceeds_and_later_collects() {
        use crate::credit::{restructuring::Decision, Account, Status, Terms, SHARED_CURRENCY};
        let mut h = fixture();
        h.export_payment_timing = payments::Timing::Delivery;
        evidence(&mut h);
        evidence(&mut h);
        h.fund_export_contracts(&[true]);
        h.market_month(6371.);
        let observed = h.export_credit_evidence(0.).unwrap().remove(0);
        let due = h.export_payments[0].expected_month;
        let principal = 1.;
        h.commit_credit_loan(
            Terms {
                lender: Account::Town(1),
                borrower: Account::Town(0),
                currency: SHARED_CURRENCY,
                source: observed.source,
                annual_simple_rate: 0.,
                maturity_month: due + 1,
                grace_months: 3,
            },
            principal,
        )
        .unwrap()
        .unwrap();
        h.month = due + 1;
        h.cargo[0].arrives = h.month + 2;
        h.credit.servicing_policy.available_cash_share = 0.;
        h.service_credit_month().unwrap();
        let initial = serde_json::to_value(&h).unwrap();
        let forecast = h.export_restructuring_evidence(0, 0.).unwrap().unwrap();
        assert_eq!(forecast.evidence.source, observed.source);
        assert_eq!(forecast.expected_payment_month, h.month + 2);
        assert_eq!(initial, serde_json::to_value(&h).unwrap());
        assert!(h.export_credit_evidence(0.).unwrap().is_empty());
        h.cargo[0].kg *= 0.5;
        let half = h.export_restructuring_evidence(0, 0.).unwrap().unwrap();
        assert!(
            (half.evidence.expected_receipts - forecast.evidence.expected_receipts * 0.5).abs()
                < 1e-8
        );
        h.cargo[0].kg *= 2.;
        // Preserve cash while making the exporter temporarily unable to pay;
        // this controlled transfer isolates consent from harvest and pricing.
        let cash = h.sites[0].economy.finance[0];
        h.transfer_credit_cash(
            Account::Town(0),
            Account::Town(1),
            SHARED_CURRENCY,
            f64::from(cash),
            0.,
        )
        .unwrap();
        h.renegotiate_export_credit_month().unwrap();
        assert!(h.credit.restructurings.is_empty()); // Disabled policy is inert.
        h.credit.commercial_policy.enabled = true;
        h.credit.commercial_policy.operating_cash_floor = 0.;
        let mut cautious: History =
            serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
        cautious.credit.commercial_policy.operating_cash_floor = f64::MAX;
        cautious.renegotiate_export_credit_month().unwrap();
        assert_eq!(
            cautious.credit.restructurings[0].decision,
            Decision::NoConsent
        );
        assert!(!cautious.credit.loans[0].restructured);
        let mut lost: History = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
        lost.cargo[0].kg = 0.;
        assert!(lost.export_restructuring_evidence(0, 0.).unwrap().is_none());
        lost = serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
        lost.cargo[0].voyage_clock = Some(crate::vessels::VoyageClock {
            month: lost.month,
            remaining: 1.,
        });
        assert!(lost.export_restructuring_evidence(0, 0.).unwrap().is_none());
        h.renegotiate_export_credit_month().unwrap();
        assert_eq!(h.credit.restructurings[0].decision, Decision::Accepted);
        assert_eq!(h.credit.loans[0].status, Status::Performing);
        let after = serde_json::to_value(&h).unwrap();
        h.renegotiate_export_credit_month().unwrap();
        assert_eq!(after, serde_json::to_value(&h).unwrap());
        h.month = half.expected_payment_month;
        h.market_arrivals();
        h.settle_export_payments().unwrap();
        assert!(h.export_restructuring_evidence(0, 0.).unwrap().is_none());
        h.month += 1;
        h.credit.servicing_policy.available_cash_share = 1.;
        h.service_credit_month().unwrap();
        assert_eq!(h.credit.loans[0].status, Status::Repaid);
        h.validate_credit().unwrap();
        h.validate_export_payments().unwrap();
    }

    #[test]
    fn commercial_evidence_uses_actual_pending_payee_and_immutable_due_date() {
        use crate::credit::{Account, RepaymentSource};
        let mut h = fixture();
        evidence(&mut h);
        evidence(&mut h);
        h.fund_export_contracts(&[true]);
        h.market_month(6371.);
        // Cash paid at dispatch is not another future receipt.
        assert!(h.export_credit_evidence(0.1).unwrap().is_empty());
        let mut h = fixture();
        h.export_payment_timing = payments::Timing::Delivery;
        evidence(&mut h);
        evidence(&mut h);
        h.fund_export_contracts(&[true]);
        h.market_month(6371.);
        let payment = h.export_payments[0].clone();
        let initial = serde_json::to_value(&h).unwrap();
        let observations = h.export_credit_evidence(0.1).unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].beneficiary, Account::Town(0));
        assert_eq!(
            observations[0].source,
            RepaymentSource::Export {
                contract: payment.contract,
                payment_month: payment.expected_month,
            }
        );
        assert!((observations[0].expected_receipts - payment.funded * 0.9).abs() < 1e-8);
        assert_eq!(initial, serde_json::to_value(&h).unwrap());
        h.cargo[0].kg *= 0.5;
        assert!(
            (h.export_credit_evidence(0.1).unwrap()[0].expected_receipts - payment.funded * 0.45)
                .abs()
                < 1e-8
        );
        h.cargo[0].arrives += 1;
        assert!(h.export_credit_evidence(0.1).unwrap().is_empty());
        h.cargo[0].arrives = payment.expected_month;
        h.month = payment.expected_month;
        assert!(h.export_credit_evidence(0.1).unwrap().is_empty());
        h.market_arrivals();
        h.settle_export_payments().unwrap();
        assert!(h.export_credit_evidence(0.1).unwrap().is_empty());
        assert!(h.export_credit_evidence(f64::NAN).is_err());
    }

    #[test]
    fn delivery_escrow_survives_expiry_and_pays_only_delivered_quantity() {
        for delivered_share in [0., 0.5, 1.] {
            let mut h = fixture();
            h.export_payment_timing = payments::Timing::Delivery;
            evidence(&mut h);
            evidence(&mut h);
            h.fund_export_contracts(&[true]);
            h.market_month(6371.);
            assert_eq!(h.sites[0].economy.finance[0], 1000.);
            assert_eq!(h.export_payments.len(), 1);
            h.validate_export_payments().unwrap();
            let mut invalid = h.clone();
            invalid.cargo[0].paid += 1.;
            assert!(invalid.validate_export_payments().is_err());
            let mut invalid = h.clone();
            invalid.cargo[0].good = 0;
            assert!(invalid.validate_export_payments().is_err());
            let mut invalid = h.clone();
            invalid.cargo.clear();
            assert!(invalid.validate_export_payments().is_err());
            assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
            let funded = h.export_payments[0].funded;
            // Expiry refunds unspent order cash, never the traveling invoice.
            h.month = 30;
            h.expire_export_contracts();
            assert!(h.export_contracts.is_empty());
            assert_eq!(h.export_payments[0].escrow, funded);
            let mut cargo = h.cargo.remove(0);
            let lost = cargo.kg * (1. - delivered_share);
            h.lose_cargo(cargo.from, cargo.good, lost);
            cargo.kg -= lost;
            h.cargo.push(cargo);
            h.market_arrivals();
            h.settle_export_payments().unwrap();
            let payment = &h.export_payments[0];
            assert!((payment.seller_paid - funded * f64::from(delivered_share)).abs() < 0.001);
            assert!((payment.refunded - funded * f64::from(1. - delivered_share)).abs() < 0.001);
            assert!(
                h.economy_residuals().iter().all(|v| v.abs() < 0.001),
                "{:?}",
                h.economy_residuals()
            );
            h.validate_export_payments().unwrap();
            let mut resumed: History =
                serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
            let cash = h
                .sites
                .iter()
                .map(|s| s.economy.finance[0])
                .collect::<Vec<_>>();
            h.settle_export_payments().unwrap();
            resumed.settle_export_payments().unwrap();
            assert_eq!(
                cash,
                h.sites
                    .iter()
                    .map(|s| s.economy.finance[0])
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                serde_json::to_value(&h).unwrap(),
                serde_json::to_value(&resumed).unwrap()
            );
        }
    }

    #[test]
    fn export_source_ids_survive_expiry_and_legacy_assignment() {
        let mut h = fixture();
        evidence(&mut h);
        assert_eq!(h.export_contracts[0].id, Some(0));
        h.month = 30;
        h.expire_export_contracts();
        assert!(h.export_contracts.is_empty());
        assert_eq!(h.export_identities.len(), 1);
        assert_eq!(h.export_identities[0].retired_month, Some(30));
        evidence(&mut h);
        assert_eq!(h.export_contracts[0].id, Some(1));
        h.validate_export_identities().unwrap();
        let mut archive = serde_json::to_value(&h).unwrap();
        archive.as_object_mut().unwrap().remove("export_identities");
        archive["export_contracts"][0]
            .as_object_mut()
            .unwrap()
            .remove("id");
        let mut legacy: History = serde_json::from_value(archive).unwrap();
        let before = legacy.export_contracts[0].escrow;
        legacy.ensure_export_contract_identities().unwrap();
        assert!(legacy.export_identities[0].legacy_baseline);
        assert_eq!(legacy.export_identities[0].assigned_month, 30);
        assert_eq!(legacy.export_contracts[0].escrow, before);
        let once = serde_json::to_value(&legacy).unwrap();
        legacy.ensure_export_contract_identities().unwrap();
        assert_eq!(once, serde_json::to_value(&legacy).unwrap());
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
        resumed.month = 60;
        resumed.expire_export_contracts();
        evidence(&mut resumed);
        assert_eq!(resumed.export_contracts[0].id, Some(2));
        resumed.export_contracts[0].id = Some(0);
        // Identical counterparties cannot resurrect a retired source identity.
        assert!(resumed.validate_export_identities().is_err());
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
