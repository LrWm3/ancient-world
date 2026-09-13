//! Annual marginal cash taxation within a bounded share of delivered administration.
use super::*;
use crate::credit::Account;
use std::collections::BTreeSet;
const FOOD_RESERVE_MONTHS: f64 = 6.;
const SECOND_BAND_MONTHS: f64 = 12.;
const THIRD_BAND_MONTHS: f64 = 24.;
const RATES: [f64; 3] = [0.02, 0.05, 0.10];
const ADMIN_SHARE: f64 = 0.25;
const WORK_PER_ACCOUNT: f64 = 0.002;
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Policy {
    pub enabled: bool,
    pub processed_month: Option<u32>,
    pub receipts: Vec<Receipt>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub month: u32,
    pub household: u32,
    pub site: u32,
    pub council: u32,
    pub opening_cash: f64,
    pub monthly_food_cost: f64,
    pub assessed: f64,
    pub collection_fraction: f64,
    pub paid: f64,
}
fn assessment(cash: f64, food: f64) -> f64 {
    if food <= 0. {
        return 0.;
    }
    let excess = (cash - food * FOOD_RESERVE_MONTHS).max(0.);
    excess.min(food * (SECOND_BAND_MONTHS - FOOD_RESERVE_MONTHS)) * RATES[0]
        + (cash - food * SECOND_BAND_MONTHS)
            .max(0.)
            .min(food * (THIRD_BAND_MONTHS - SECOND_BAND_MONTHS))
            * RATES[1]
        + (cash - food * THIRD_BAND_MONTHS).max(0.) * RATES[2]
}
pub(super) fn validate(e: &HouseholdEconomy, h: &History) -> Result<()> {
    ensure!(
        e.wealth_tax.processed_month.is_none_or(|m| m <= h.month),
        "wealth tax clock"
    );
    let mut sums = vec![0.; e.accounts.len()];
    let mut seen = BTreeSet::new();
    for r in &e.wealth_tax.receipts {
        ensure!(
            (r.household as usize) < sums.len()
                && (r.site as usize) < h.sites.len()
                && (r.council as usize) < h.civilizations.len()
                && r.month <= h.month
                && r.month.is_multiple_of(12)
                && seen.insert((r.month, r.household))
                && [
                    r.opening_cash,
                    r.monthly_food_cost,
                    r.assessed,
                    r.collection_fraction,
                    r.paid
                ]
                .iter()
                .all(|x| x.is_finite() && *x >= 0.)
                && r.collection_fraction <= 1.
                && r.paid <= r.opening_cash
                && (r.assessed - assessment(r.opening_cash, r.monthly_food_cost)).abs()
                    < BALANCE_TOLERANCE
                && (r.paid - r.assessed * r.collection_fraction).abs() < BALANCE_TOLERANCE,
            "wealth tax receipt"
        );
        sums[r.household as usize] += r.paid;
    }
    for (a, paid) in e.accounts.iter().zip(sums) {
        ensure!(
            (a.wealth_tax_paid - paid).abs() <= BALANCE_TOLERANCE * (1. + paid),
            "wealth tax ledger"
        );
    }
    Ok(())
}
impl History {
    /// After food retail. Existing named office work supplies the collection budget;
    /// another quarter of that work is reserved for estate review, not reused here.
    pub(crate) fn collect_household_wealth_tax(&mut self) {
        if self.month == 0 || !self.month.is_multiple_of(12) {
            return;
        }
        let Some(soc) = &self.society else {
            return;
        };
        let Some(e) = &soc.household_economy else {
            return;
        };
        if !e.wealth_tax.enabled || e.wealth_tax.processed_month == Some(self.month) {
            return;
        }
        let mut plans = vec![];
        for s in &self.sites {
            if s.abandoned {
                continue;
            }
            let controller = self.controller(s.id);
            let work = self
                .offices
                .as_ref()
                .and_then(|o| o.service.as_ref())
                .map_or(0., |service| {
                    service
                        .plans
                        .iter()
                        .filter(|p| {
                            p.site == s.id
                                && p.controller == controller
                                && p.work.month == self.month
                                && p.work.settled
                                && self.people[p.holder as usize].died.is_none()
                                && self.offices.as_ref().unwrap().seats[s.id as usize].holder()
                                    == Some(p.holder)
                        })
                        .map(|p| p.work.used)
                        .fold(0., f64::max)
                });
            let candidates: Vec<_> = soc
                .households
                .iter()
                .filter(|hh| {
                    hh.site == s.id
                        && hh.vacant_since.is_none()
                        && !soc.relocation.away(hh.id)
                        && !self.credit.account_has_debt(Account::Household(hh.id))
                })
                .filter_map(|hh| {
                    e.accounts
                        .get(hh.id as usize)
                        .filter(|a| a.food_site == Some(s.id) && a.need > 0.)
                        .map(|a| (hh, a))
                })
                .collect();
            let fraction = (work * ADMIN_SHARE
                / (candidates.len() as f64 * WORK_PER_ACCOUNT).max(WORK_PER_ACCOUNT))
            .min(1.);
            for (hh, a) in candidates {
                let food =
                    a.need * f64::from(s.economy.prices[crate::economy::FOOD].max(MIN_FOOD_PRICE));
                let assessed = assessment(a.cash, food);
                if assessed * fraction > 0. {
                    plans.push(Receipt {
                        month: self.month,
                        household: hh.id,
                        site: s.id,
                        council: controller,
                        opening_cash: a.cash,
                        monthly_food_cost: food,
                        assessed,
                        collection_fraction: fraction,
                        paid: assessed * fraction,
                    });
                }
            }
        }
        let soc = self.society.as_mut().unwrap();
        let e = soc.household_economy.as_mut().unwrap();
        e.wealth_tax.processed_month = Some(self.month);
        for r in plans {
            let a = &mut e.accounts[r.household as usize];
            a.cash -= r.paid;
            a.wealth_tax_paid += r.paid;
            soc.councils[r.council as usize].treasury += r.paid;
            e.wealth_tax.receipts.push(r);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marginal_cash_tax_protects_food_and_has_no_bracket_cliff() {
        assert_eq!(assessment(600., 100.), 0.);
        assert_eq!(assessment(1200., 100.), 12.);
        assert_eq!(assessment(2400., 100.), 72.);
        assert_eq!(assessment(3400., 100.), 172.);
        assert_eq!(assessment(10000., 0.), 0.);
        assert!((assessment(2401., 100.) - assessment(2400., 100.) - 0.1).abs() < 1e-9);
    }
}
#[cfg(test)]
mod integration {
    use super::*;
    #[test]
    #[ignore = "requires GPU"]
    fn civic_tax_requires_completed_service_and_conserves_cash() {
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
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        g.civilizations.as_mut().unwrap().sync_culture();
        g.enable_offices().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.set_individual_participation(true).unwrap();
        h.set_office_service(true).unwrap();
        h.prepare_household_retail();
        h.month = 12;
        let hh = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|f| f.site == 0)
            .unwrap()
            .id as usize;
        let cash = super::super::withdraw(&mut h.sites[0].economy.finance[0], 1000.);
        h.sites[0].economy.prices[crate::economy::FOOD] = 1.;
        let e = h
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap();
        e.wealth_tax.enabled = true;
        let a = &mut e.accounts[hh];
        a.cash += cash;
        a.dividends += cash;
        a.need = 1.;
        a.food_site = Some(0);
        h.begin_service_reservations();
        h.reserve_office_service().unwrap();
        let mut unperformed = h.clone();
        unperformed.collect_household_wealth_tax();
        assert!(unperformed
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .wealth_tax
            .receipts
            .is_empty());
        h.settle_office_service().unwrap();
        let before = h.money_residual();
        h.collect_household_wealth_tax();
        assert!((h.money_residual() - before).abs() < 1e-9);
        let e = h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        assert!(!e.wealth_tax.receipts.is_empty());
        validate(e, h).unwrap();
        let state = serde_json::to_value(&h).unwrap();
        h.collect_household_wealth_tax();
        assert_eq!(state, serde_json::to_value(&h).unwrap());
        let mut resumed: History = serde_json::from_value(state).unwrap();
        resumed.collect_household_wealth_tax();
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
    }
}
