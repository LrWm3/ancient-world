//! Local unclaimed-cash review using a bounded part of completed administration.
use super::{deposit, HouseholdEconomy};
use crate::{civilization::History, credit::Account};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const VACANCY_REVIEW_MONTHS: u32 = 60;
const FOOD_RESERVE_MONTHS: f64 = 3.;
const ADMINISTRATION_REVIEW_SHARE: f64 = 0.25;
const REVIEW_WORKER_MONTHS: f64 = 0.025;
const MIN_RECLAIMED_CASH: f64 = 1.;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Reclamation {
    pub enabled: bool,
    pub processed_month: Option<u32>,
    pub receipts: Vec<Receipt>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub month: u32,
    pub estate: u32,
    pub site: u32,
    pub official: u32,
    pub controller: u32,
    pub cash: f64,
    pub protected_food_cash: f64,
    pub administration_used: f64,
}

fn review_capacity(work: f64) -> usize {
    (work * ADMINISTRATION_REVIEW_SHARE / REVIEW_WORKER_MONTHS).floor() as usize
}

pub(super) fn validate(e: &HouseholdEconomy, h: &History) -> Result<()> {
    ensure!(
        e.reclamation.processed_month.is_none_or(|m| m <= h.month),
        "invalid reclamation clock"
    );
    let mut sums = vec![0.; e.accounts.len()];
    let mut seen = BTreeSet::new();
    let mut cases = std::collections::BTreeMap::<(u32, u32), (usize, f64)>::new();
    for r in &e.reclamation.receipts {
        ensure!(
            (r.estate as usize) < sums.len()
                && (r.site as usize) < h.sites.len()
                && (r.official as usize) < h.people.len()
                && (r.controller as usize) < h.civilizations.len()
                && r.month <= e.reclamation.processed_month.unwrap_or(0)
                && r.cash.is_finite()
                && r.cash > 0.
                && r.protected_food_cash.is_finite()
                && r.protected_food_cash >= 0.
                && r.administration_used.is_finite()
                && r.administration_used >= 0.
                && seen.insert((r.month, r.estate)),
            "invalid reclamation receipt"
        );
        sums[r.estate as usize] += r.cash;
        let entry = cases
            .entry((r.month, r.site))
            .or_insert((0, r.administration_used));
        ensure!(
            entry.1 == r.administration_used,
            "inconsistent estate review capacity"
        );
        entry.0 += 1;
        ensure!(
            entry.0 <= review_capacity(entry.1),
            "estate reviews exceed delivered administration"
        );
    }
    for (a, sum) in e.accounts.iter().zip(sums) {
        ensure!(
            (a.reclaimed - sum).abs() <= super::BALANCE_TOLERANCE * (1. + sum),
            "reclamation cash does not reconcile"
        );
    }
    Ok(())
}

impl History {
    /// After retail settlement: no interference with this month's reserved food.
    /// Review is a task within delivered administration, not additional free work.
    pub(crate) fn reclaim_household_estates(&mut self) {
        let (Some(society), Some(politics), Some(offices)) =
            (&self.society, &self.politics, &self.offices)
        else {
            return;
        };
        let (Some(e), Some(service)) = (&society.household_economy, &offices.service) else {
            return;
        };
        if !e.reclamation.enabled || e.reclamation.processed_month == Some(self.month) {
            return;
        }
        let live: BTreeSet<_> = self
            .people
            .iter()
            .filter(|p| p.died.is_none())
            .map(|p| p.id)
            .collect();
        let presence = self.person_presences();
        let occupied: BTreeSet<_> = self
            .people
            .iter()
            .filter(|p| live.contains(&p.id))
            .filter_map(|p| presence[p.id as usize].0)
            .collect();
        // Any recorded relative blocks escheat, including through dead ancestors.
        // Conservative connected kin groups avoid inventing an exhaustive heir search.
        let mut protected = live.clone();
        loop {
            let before = protected.len();
            for kin in &politics.kin {
                for parent in kin.parents.iter().flatten() {
                    if protected.contains(&kin.person) || protected.contains(parent) {
                        protected.insert(kin.person);
                        protected.insert(*parent);
                    }
                }
            }
            for marriage in &politics.marriages {
                if marriage.partners.iter().any(|p| protected.contains(p)) {
                    protected.extend(marriage.partners);
                }
            }
            if before == protected.len() {
                break;
            }
        }
        let mut plans = vec![];
        let mut sites = BTreeSet::new();
        for plan in &service.plans {
            if !sites.insert(plan.site)
                || plan.work.month != self.month
                || !plan.work.settled
                || !live.contains(&plan.holder)
                || self.controller(plan.site) != plan.controller
                || self.sites[plan.site as usize].abandoned
                || offices
                    .seats
                    .get(plan.site as usize)
                    .and_then(|o| o.holder())
                    != Some(plan.holder)
            {
                continue;
            }
            let mut candidates: Vec<_> = society
                .households
                .iter()
                .filter(|f| {
                    f.site == plan.site
                        && f.vacant_since
                            .is_some_and(|m| self.month.saturating_sub(m) >= VACANCY_REVIEW_MONTHS)
                        && !occupied.contains(&f.id)
                        && !protected.contains(&f.head)
                        && !society.relocation.away(f.id)
                        && !self.credit.account_has_debt(Account::Household(f.id))
                })
                .collect();
            candidates.sort_by_key(|f| (f.vacant_since, f.id));
            let mut remaining = review_capacity(plan.work.used);
            for estate in candidates {
                if remaining == 0 {
                    break;
                }
                let Some(a) = e.accounts.get(estate.id as usize) else {
                    continue;
                };
                let reserve = a.need
                    * FOOD_RESERVE_MONTHS
                    * f64::from(
                        self.sites[estate.site as usize].economy.prices[crate::economy::FOOD]
                            .max(super::MIN_FOOD_PRICE),
                    );
                let cash = (a.cash - reserve).max(0.);
                if cash < MIN_RECLAIMED_CASH {
                    continue;
                }
                remaining -= 1;
                plans.push(Receipt {
                    month: self.month,
                    estate: estate.id,
                    site: estate.site,
                    official: plan.holder,
                    controller: plan.controller,
                    cash,
                    protected_food_cash: reserve.min(a.cash),
                    administration_used: plan.work.used,
                });
            }
        }
        let e = self
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap();
        e.reclamation.processed_month = Some(self.month);
        let mut committed = vec![];
        for mut r in plans {
            // Town cash is f32; debit only the amount actually representable there.
            let amount = deposit(&mut self.sites[r.site as usize].economy.finance[0], r.cash);
            if amount == 0. {
                continue;
            }
            let a = &mut e.accounts[r.estate as usize];
            a.cash -= amount;
            a.reclaimed += amount;
            r.cash = amount;
            e.reclamation.receipts.push(r.clone());
            committed.push(r);
        }
        for r in committed {
            self.event("unclaimed_estate_reclaimed", Some(r.site), None,
                format!("Local administration reclaimed {:.2} surplus cash from household {} after five years of vacancy and no recorded living kin; {:.2} retained for food, ownership and historical claims preserved", r.cash,r.estate,r.protected_food_cash));
            self.events.last_mut().unwrap().subjects.extend([
                ("household".into(), r.estate),
                ("person".into(), r.official),
            ]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn case_capacity_is_bounded_by_completed_work() {
        assert_eq!(review_capacity(0.), 0);
        assert_eq!(review_capacity(0.099), 0);
        assert_eq!(review_capacity(0.1), 1);
        assert_eq!(review_capacity(0.2), 2);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn reclamation_requires_delivered_office_work_and_unclaimed_cash() {
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
        h.month = 72;
        let official = h.offices.as_ref().unwrap().seats[0].holder().unwrap();
        let estate = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|f| f.site == 0 && f.head != official)
            .unwrap()
            .id;
        let root = h.society.as_ref().unwrap().households[estate as usize].head;
        let presence = h.person_presences();
        for p in &mut h.people {
            if presence[p.id as usize].0 == Some(estate) {
                p.died = Some(1);
            }
        }
        for k in &mut h.politics.as_mut().unwrap().kin {
            k.parents = [None; 2];
        }
        h.politics.as_mut().unwrap().marriages.clear();
        h.society.as_mut().unwrap().households[estate as usize].vacant_since = Some(1);
        let cash = super::super::withdraw(&mut h.sites[0].economy.finance[0], 100.);
        h.sites[0].economy.prices[crate::economy::FOOD] = 1.;
        let e = h
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap();
        e.reclamation.enabled = true;
        let a = &mut e.accounts[estate as usize];
        a.cash += cash;
        a.dividends += cash;
        a.need = 1.;
        a.common_food = 0.;
        a.purchased_food = 0.;
        h.begin_service_reservations();
        h.reserve_office_service().unwrap();
        let mut unperformed = h.clone();
        unperformed.reclaim_household_estates();
        assert!(unperformed
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .reclamation
            .receipts
            .is_empty());
        h.settle_office_service().unwrap();
        let mut kin = h.clone();
        kin.politics
            .as_mut()
            .unwrap()
            .kin
            .iter_mut()
            .find(|k| k.person == root)
            .unwrap()
            .parents = [Some(official), None];
        kin.reclaim_household_estates();
        assert!(kin
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .reclamation
            .receipts
            .is_empty());
        let before = h.money_residual();
        let share = h.society.as_ref().unwrap().households[estate as usize].share;
        h.reclaim_household_estates();
        let e = h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        assert_eq!(e.reclamation.receipts.len(), 1);
        assert!(e.reclamation.receipts[0].cash > 0.);
        assert!(e.accounts[estate as usize].cash >= FOOD_RESERVE_MONTHS);
        assert!((h.money_residual() - before).abs() < 1e-6);
        assert_eq!(
            h.society.as_ref().unwrap().households[estate as usize].share,
            share
        );
        e.validate(h).unwrap();
        let saved = serde_json::to_value(&*h).unwrap();
        h.reclaim_household_estates();
        assert_eq!(saved, serde_json::to_value(&*h).unwrap());
        let mut resumed: History = serde_json::from_value(saved).unwrap();
        resumed.month += 1;
        h.month += 1;
        resumed.reclaim_household_estates();
        h.reclaim_household_estates();
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        resumed
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .reclamation
            .receipts[0]
            .cash += 1.;
        assert!(resumed
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .validate(&resumed)
            .is_err());
    }
}
