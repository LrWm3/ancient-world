//! Household-owned cloth: finite retail transfers and replacement through wear.
use super::deposit;
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

const CLOTH_GOOD: &str = "cloth";
const CLOTH_KG_PER_DIET_EQUIVALENT: f64 = 0.6;
const FOOD_KG_PER_DIET_EQUIVALENT_MONTH: f64 = 18.;
const PROTECTED_FOOD_MONTHS: f64 = 1.;
const MONTHLY_SURPLUS_SPENDING_SHARE: f64 = 0.1;
const CLOTH_MONTHLY_WEAR: f64 = 0.025;
const LEDGER_TOLERANCE: f64 = 1e-6;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Wardrobe {
    pub cloth_kg: f64,
    pub purchased_kg: f64,
    pub worn_kg: f64,
    pub spending: f64,
}
impl Wardrobe {
    pub(super) fn validate(&self) -> Result<()> {
        ensure!(
            [
                self.cloth_kg,
                self.purchased_kg,
                self.worn_kg,
                self.spending
            ]
            .iter()
            .all(|x| x.is_finite() && *x >= 0.),
            "invalid household clothing"
        );
        ensure!(
            (self.cloth_kg + self.worn_kg - self.purchased_kg).abs()
                <= LEDGER_TOLERANCE * (1. + self.purchased_kg),
            "household cloth ledger mismatch"
        );
        Ok(())
    }
}
fn demand(cash: f64, need: f64, food_price: f64, cloth_price: f64, held: f64) -> f64 {
    let target = need / FOOD_KG_PER_DIET_EQUIVALENT_MONTH * CLOTH_KG_PER_DIET_EQUIVALENT;
    (target - held).max(0.).min(
        (cash - need * food_price * PROTECTED_FOOD_MONTHS).max(0.) * MONTHLY_SURPLUS_SPENDING_SHARE
            / cloth_price,
    )
}
impl History {
    pub(crate) fn household_clothing_active(&self) -> bool {
        self.society
            .as_ref()
            .and_then(|s| s.household_economy.as_ref())
            .is_some_and(|e| {
                e.clothing_enabled || e.accounts.iter().any(|a| a.wardrobe.cloth_kg > 0.)
            })
    }
    /// Closing-boundary intentions for the coming month, not escrow or income.
    /// Only cloth receives this support; a food reserve remains unavailable to it.
    pub(crate) fn household_clothing_quote_budgets(&self) -> Option<(usize, Vec<f32>)> {
        let society = self.society.as_ref()?;
        let wallets = society.household_economy.as_ref()?;
        if !wallets.clothing_enabled {
            return None;
        }
        let good = self.economy_catalog.as_ref()?.index(CLOTH_GOOD)?;
        let mut budgets = vec![0.; self.sites.len()];
        for hh in &society.households {
            if hh.vacant_since.is_some() || society.relocation.away(hh.id) {
                continue;
            }
            let Some(a) = wallets.accounts.get(hh.id as usize) else {
                continue;
            };
            let s = &self.sites[hh.site as usize];
            if s.abandoned || a.food_site != Some(hh.site) {
                continue;
            }
            let price = f64::from(s.economy.prices[good].max(super::MIN_FOOD_PRICE));
            // Predict one wear step. Stock scarcity must not erase an unmet request.
            budgets[hh.site as usize] += (price
                * demand(
                    a.cash,
                    a.need,
                    f64::from(s.economy.prices[crate::economy::FOOD].max(super::MIN_FOOD_PRICE)),
                    price,
                    a.wardrobe.cloth_kg * (1. - CLOTH_MONTHLY_WEAR),
                )) as f32;
        }
        Some((good, budgets))
    }
    /// After food settlement; purchases and wear are visible to next month's planner.
    pub(crate) fn settle_household_clothing(&mut self) {
        if !self.household_clothing_active() {
            return;
        }
        let Some(catalog) = &self.economy_catalog else {
            return;
        };
        let Some(good) = catalog.index(CLOTH_GOOD) else {
            return;
        };
        let composition = catalog.composition(good);
        let Some(society) = &mut self.society else {
            return;
        };
        let Some(e) = &mut society.household_economy else {
            return;
        };
        if e.clothing_month == Some(self.month) {
            return;
        }
        e.clothing_month = Some(self.month);
        let mut requests = vec![0.; e.accounts.len()];
        let mut totals = vec![0.; self.sites.len()];
        for hh in &society.households {
            let Some(a) = e.accounts.get_mut(hh.id as usize) else {
                continue;
            };
            if society.relocation.away(hh.id) {
                continue;
            }
            let s = &mut self.sites[hh.site as usize];
            // Vacant households retain possessions; no new representative is invented.
            if hh.vacant_since.is_some() || s.abandoned || a.food_site != Some(hh.site) {
                continue;
            }
            let worn = a.wardrobe.cloth_kg * CLOTH_MONTHLY_WEAR;
            a.wardrobe.cloth_kg -= worn;
            a.wardrobe.worn_kg += worn;
            s.economy.used[good] += worn as f32;
            s.economy.reserves[3] += worn as f32;
            for (k, ratio) in composition.iter().enumerate() {
                s.economy.detritus[k] += (worn * f64::from(*ratio)) as f32;
            }
            if !e.clothing_enabled {
                continue;
            }
            let q = demand(
                a.cash,
                a.need,
                f64::from(s.economy.prices[crate::economy::FOOD].max(super::MIN_FOOD_PRICE)),
                f64::from(s.economy.prices[good].max(super::MIN_FOOD_PRICE)),
                a.wardrobe.cloth_kg,
            );
            requests[hh.id as usize] = q;
            totals[hh.site as usize] += q;
        }
        let factors: Vec<_> = self
            .sites
            .iter()
            .enumerate()
            .map(|(i, s)| {
                (f64::from(s.economy.goods[good]) / totals[i].max(LEDGER_TOLERANCE)).min(1.)
            })
            .collect();
        for hh in &society.households {
            let q = requests.get(hh.id as usize).copied().unwrap_or(0.) * factors[hh.site as usize];
            if q <= 0. {
                continue;
            }
            let a = &mut e.accounts[hh.id as usize];
            let s = &mut self.sites[hh.site as usize];
            let price = f64::from(s.economy.prices[good].max(super::MIN_FOOD_PRICE));
            let old_stock = s.economy.goods[good];
            let remaining = (f64::from(old_stock) - q).max(0.);
            let mut stock = remaining as f32;
            if f64::from(stock) < remaining {
                stock = f32::from_bits(stock.to_bits() + 1);
            }
            let taken = f64::from(old_stock) - f64::from(stock);
            if taken <= 0. || taken * price > a.cash {
                continue;
            }
            let paid = deposit(&mut s.economy.finance[0], taken * price);
            if paid <= 0. {
                continue;
            }
            s.economy.goods[good] = stock;
            a.cash -= paid;
            a.wardrobe.spending += paid;
            a.wardrobe.cloth_kg += taken;
            a.wardrobe.purchased_kg += taken;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clothing_demand_protects_food_and_stops_at_service_target() {
        assert_eq!(demand(18., 18., 1., 2., 0.), 0.);
        assert_eq!(demand(20., 18., 1., 2., 0.), 0.1);
        assert_eq!(demand(1000., 18., 1., 2., 0.), 0.6);
        assert_eq!(demand(1000., 18., 1., 2., 0.6), 0.);
        let w = Wardrobe {
            cloth_kg: 0.975,
            purchased_kg: 1.,
            worn_kg: 0.025,
            spending: 2.,
        };
        w.validate().unwrap();
        assert!(Wardrobe { cloth_kg: 2., ..w }.validate().is_err());
    }
    #[test]
    #[ignore = "requires GPU"]
    fn clothing_retail_conserves_cash_material_and_boundary_continuation() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                seed: 17,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let mut h = g.civilizations.as_ref().unwrap().clone();
        h.prepare_household_retail();
        h.month = 12;
        let good = h.economy_catalog.as_ref().unwrap().index("cloth").unwrap();
        let site = 0usize;
        let society = h.society.as_mut().unwrap();
        let ids: Vec<_> = society
            .households
            .iter()
            .filter(|f| f.site == site as u32)
            .map(|f| f.id as usize)
            .take(2)
            .collect();
        assert_eq!(ids.len(), 2);
        let wallets = society.household_economy.as_mut().unwrap();
        wallets.clothing_enabled = true;
        for a in &mut wallets.accounts {
            a.food_site = None;
        }
        for &id in &ids {
            let a = &mut wallets.accounts[id];
            a.wages += 200.;
            a.cash += 200.;
            a.need = 18.;
            a.food_site = Some(0);
        }
        h.sites[site].economy.goods[good] = 0.6;
        h.sites[site].economy.prices[good] = 2.;
        h.sites[site].economy.prices[crate::economy::FOOD] = 1.;
        let opening = serde_json::to_value(&h).unwrap();
        let (quoted_good, budgets) = h.household_clothing_quote_budgets().unwrap();
        assert_eq!(quoted_good, good);
        assert!((budgets[site] - 2.4).abs() < 1e-6);
        assert_eq!(opening, serde_json::to_value(&h).unwrap());
        let mut unavailable = h.clone();
        unavailable.sites[site].economy.goods[good] = 0.;
        assert_eq!(
            unavailable.household_clothing_quote_budgets().unwrap().1,
            budgets
        );
        unavailable.sites[site].abandoned = true;
        assert_eq!(
            unavailable.household_clothing_quote_budgets().unwrap().1[site],
            0.
        );
        // A funded, unfilled cloth request affects only its own local quote.
        // Non-quarterly month avoids dispatch decisions in this controlled case.
        let mut quotes = h.clone();
        quotes.month = 13;
        quotes
            .economy_catalog
            .as_mut()
            .unwrap()
            .market
            .adaptive_prices = true;
        quotes.sites[site].economy.finance[0] = 0.;
        quotes.sites[site].economy.goods[good] = 0.;
        quotes.sites[site].economy.targets[good] = 10.;
        quotes.sites[site].economy.logistics[3] = 1.;
        quotes.sites[site].economy.prices[good] =
            quotes.supplier_unit_cost(site, good).unwrap_or(9.);
        let mut no_quotes = quotes.clone();
        no_quotes
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .clothing_enabled = false;
        let observations = vec![[[0.; 2]; crate::economy::GOODS]; quotes.sites.len()];
        quotes.market_decisions(1., &observations);
        no_quotes.market_decisions(1., &observations);
        assert!(
            quotes.sites[site].economy.prices[good] > no_quotes.sites[site].economy.prices[good]
        );
        for k in 0..crate::economy::GOODS {
            if k != good {
                assert_eq!(
                    quotes.sites[site].economy.prices[k],
                    no_quotes.sites[site].economy.prices[k]
                );
            }
        }
        let before = h.economy_residuals();
        let mut disabled = h.clone();
        disabled
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .clothing_enabled = false;
        let unchanged = serde_json::to_value(&disabled).unwrap();
        assert!(disabled.household_clothing_quote_budgets().is_none());
        disabled.settle_household_clothing();
        assert_eq!(unchanged, serde_json::to_value(&disabled).unwrap());
        let mut poor = h.clone();
        for &id in &ids {
            poor.society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .accounts[id]
                .cash = 0.;
        }
        assert_eq!(poor.household_clothing_quote_budgets().unwrap().1[site], 0.);
        poor.settle_household_clothing();
        assert_eq!(
            poor.sites[site].economy.goods[good],
            h.sites[site].economy.goods[good]
        );
        assert!(poor
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .accounts
            .iter()
            .all(|a| a.wardrobe.purchased_kg == 0.));
        h.settle_household_clothing();
        let wallets = h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        for &id in &ids {
            let a = &wallets.accounts[id];
            assert!((a.wardrobe.cloth_kg - 0.3).abs() < 1e-6);
            assert!(a.cash >= 18.);
            a.wardrobe.validate().unwrap();
        }
        for (a, b) in before.into_iter().zip(h.economy_residuals()) {
            assert!((a - b).abs() < 1e-5, "{a} != {b}");
        }
        let snapshot = serde_json::to_value(&h).unwrap();
        h.settle_household_clothing();
        assert_eq!(snapshot, serde_json::to_value(&h).unwrap());
        let mut resumed: History = serde_json::from_value(snapshot).unwrap();
        for world in [&mut h, &mut resumed] {
            world.month += 1;
            world
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .clothing_enabled = false;
            world.settle_household_clothing();
        }
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let a = &h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .accounts[ids[0]];
        assert!(a.wardrobe.worn_kg > 0.);
        assert!((a.wardrobe.purchased_kg - 0.3).abs() < 1e-6);
        for (a, b) in before.into_iter().zip(h.economy_residuals()) {
            assert!((a - b).abs() < 1e-5, "{a} != {b}");
        }
    }
}
