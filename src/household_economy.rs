//! Finite household wallets and retail entitlements; no duplicate food or population stocks.
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HouseholdAccount {
    /// Relative earnings weights, not extra workers or skill-owned production.
    #[serde(default)]
    pub livelihood: Option<[f64; 4]>,
    #[serde(default)]
    pub sector_wages: [f64; 4],
    #[serde(default)]
    pub capital_invested: f64,
    #[serde(default)]
    pub capital_returned: f64,
    /// Wages paid by employers this month; already included in cumulative wages.
    #[serde(default)]
    pub employer_income: f64,
    pub cash: f64,
    pub wages: f64,
    #[serde(default)]
    pub relief: f64,
    pub dividends: f64,
    pub food_spending: f64,
    pub estate_returned: f64,
    pub need: f64,
    pub common_food: f64,
    pub purchased_food: f64,
    pub hunger: f64,
    /// Settlement whose last monthly food allocation this account describes.
    #[serde(default)]
    pub food_site: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HouseholdEconomy {
    /// Older archives retain equal payroll allocation.
    #[serde(default)]
    pub occupational_payroll: bool,
    pub started: u32,
    pub observed: u32,
    pub common_share: f32,
    pub payroll_share: f32,
    pub dividend_share: f32,
    /// Monthly fraction of treasury available for targeted food relief. Old archives retain zero.
    #[serde(default)]
    pub relief_share: f32,
    #[serde(default)]
    pub relief_target: f32,
    pub accounts: Vec<HouseholdAccount>,
    #[serde(default)]
    pub access_episodes: Vec<[u32; 3]>,
}
impl HouseholdEconomy {
    pub fn new(month: u32) -> Self {
        Self {
            occupational_payroll: true,
            started: month,
            observed: month,
            common_share: 0.5,
            payroll_share: 0.2,
            dividend_share: 0.01,
            relief_share: 0.05,
            relief_target: 0.75,
            accounts: vec![],
            access_episodes: vec![],
        }
    }
    pub fn validate(&self, h: &History) -> Result<()> {
        ensure!(
            self.started <= self.observed && self.observed <= h.month,
            "invalid household economy clock"
        );
        ensure!(
            [
                self.common_share,
                self.payroll_share,
                self.dividend_share,
                self.relief_share,
                self.relief_target
            ]
            .iter()
            .all(|v| v.is_finite() && (0. ..=1.).contains(v)),
            "invalid household distribution policy"
        );
        ensure!(
            self.accounts.len() <= h.society.as_ref().unwrap().households.len(),
            "unknown household account"
        );
        ensure!(
            self.access_episodes.len() <= h.sites.len()
                && self.access_episodes.iter().all(|v| v[2] <= 1),
            "invalid household access episodes"
        );
        for a in &self.accounts {
            ensure!(
                a.livelihood.is_none_or(|weights| weights
                    .iter()
                    .all(|v| v.is_finite() && (1. ..=2.).contains(v)))
                    && a.sector_wages.iter().all(|v| v.is_finite() && *v >= 0.)
                    && a.sector_wages.iter().sum::<f64>() <= a.wages + 1e-6,
                "invalid household livelihood or sector payroll"
            );
            ensure!(
                [
                    a.capital_invested,
                    a.capital_returned,
                    a.employer_income,
                    a.cash,
                    a.wages,
                    a.relief,
                    a.dividends,
                    a.food_spending,
                    a.estate_returned,
                    a.need,
                    a.common_food,
                    a.purchased_food,
                    a.hunger
                ]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.),
                "invalid household account"
            );
            ensure!(
                a.hunger <= 1.
                    && a.common_food + a.purchased_food <= a.need + 0.01
                    && a.food_site
                        .is_none_or(|site| (site as usize) < h.sites.len()),
                "invalid household food allocation"
            );
            ensure!(
                (a.cash - a.wages - a.dividends - a.relief
                    + a.food_spending
                    + a.estate_returned
                    + a.capital_invested
                    - a.capital_returned)
                    .abs()
                    < 1e-6
                        * (1.
                            + a.wages
                            + a.dividends
                            + a.relief
                            + a.capital_invested
                            + a.capital_returned),
                "household cash ledger does not reconcile"
            );
        }
        Ok(())
    }
}
pub(crate) struct RetailPlan {
    site: usize,
    ids: Vec<usize>,
    demand: Vec<f64>,
    need: f64,
    free: f64,
    price: f64,
}
// Return the actual f32 pool withdrawal: the wallet receives exactly what left.
pub(crate) fn withdraw(pool: &mut f32, requested: f64) -> f64 {
    let old = *pool;
    *pool = (old as f64 - requested.min(old as f64).max(0.)).max(0.) as f32;
    old as f64 - *pool as f64
}
// Credit no more than the buyer can pay, including f32 pool rounding.
pub(crate) fn deposit(pool: &mut f32, requested: f64) -> f64 {
    let old = *pool;
    let mut next = (old as f64 + requested.max(0.)) as f32;
    if next as f64 - old as f64 > requested {
        next = f32::from_bits(next.to_bits().saturating_sub(1)).max(old);
    }
    *pool = next;
    next as f64 - old as f64
}
// Allocate a finite payroll by the preceding production step’s allocated sector work, then by resident earnings weights.
// No workers, goods or money are added. The final recipient absorbs only rounding residue.
fn sector_payroll(weights: &[[f64; 4]], work: [f64; 4], payroll: f64) -> Vec<[f64; 4]> {
    let mut result = vec![[0.; 4]; weights.len()];
    let total = work.iter().sum::<f64>();
    if weights.is_empty() || total <= 0. || payroll <= 0. {
        return result;
    }
    let last_sector = work.iter().rposition(|w| *w > 0.).unwrap();
    let mut left = payroll;
    for sector in 0..4 {
        if work[sector] <= 0. {
            continue;
        }
        let budget = if sector == last_sector {
            left
        } else {
            (payroll * work[sector] / total).min(left)
        };
        left -= budget;
        let sum = weights.iter().map(|w| w[sector]).sum::<f64>();
        let mut remaining = budget;
        for (i, weight) in weights.iter().enumerate() {
            let paid = if i + 1 == weights.len() {
                remaining
            } else {
                (budget * weight[sector] / sum).min(remaining)
            };
            remaining -= paid;
            result[i][sector] = paid;
        }
    }
    result
}
fn apportion_relief(requests: &[f64], budget: f64) -> Vec<f64> {
    let demand = requests.iter().sum::<f64>();
    let budget = budget.min(demand).max(0.);
    let mut remaining = budget;
    requests
        .iter()
        .enumerate()
        .map(|(j, &request)| {
            let grant = if demand == 0. {
                0.
            } else if j + 1 == requests.len() {
                remaining.min(request)
            } else {
                (budget * request / demand).min(remaining).min(request)
            };
            remaining -= grant;
            grant
        })
        .collect()
}
impl History {
    pub fn household_account(&self, id: u32) -> Option<&HouseholdAccount> {
        self.society
            .as_ref()?
            .household_economy
            .as_ref()?
            .accounts
            .get(id as usize)
    }
    pub(crate) fn prepare_household_retail(&mut self) -> Vec<RetailPlan> {
        for s in &mut self.sites {
            s.demography.household_food = [0.; 4];
        }
        let controllers = (0..self.sites.len())
            .map(|i| self.controller(i as u32) as usize)
            .collect::<Vec<_>>();
        let Some(society) = self.society.as_mut() else {
            return vec![];
        };
        let Some(e) = society.household_economy.as_mut() else {
            return vec![];
        };
        e.accounts
            .resize(society.households.len(), HouseholdAccount::default());
        let mut residents = vec![vec![]; self.sites.len()];
        for hh in &society.households {
            let a = &mut e.accounts[hh.id as usize];
            a.need = 0.;
            a.common_food = 0.;
            a.purchased_food = 0.;
            a.hunger = 0.;
            a.food_site = None;
            a.sector_wages = [0., 0., 0., a.employer_income];
            if e.occupational_payroll
                && a.livelihood.is_none()
                && self.people[hh.head as usize].died.is_none()
            {
                let mut weights = [1.; 4];
                if let Some(agent) = self
                    .culture
                    .as_ref()
                    .and_then(|c| c.agents.get(hh.head as usize))
                {
                    match agent.occupation.as_str() {
                        "farmer" => weights[0] = 2.,
                        "craftworker" => weights[3] = 2.,
                        _ => {}
                    }
                }
                a.livelihood = Some(weights);
            }
            if society.relocation.lost_households.contains(&hh.id) {
                let returned =
                    deposit(&mut self.sites[hh.site as usize].economy.finance[0], a.cash);
                a.cash -= returned;
                a.estate_returned += returned;
            } else if !society.relocation.away(hh.id) {
                residents[hh.site as usize].push(hh.id as usize);
            }
        }
        let vessel_work: Vec<f32> = self.shipping.as_ref().map_or_else(
            || vec![0.; self.sites.len()],
            |shipping| {
                let mut work = vec![0.; self.sites.len()];
                for p in &shipping.ports {
                    work[p.site as usize] += p.fleet.as_ref().map_or(0., |f| f.work());
                }
                work
            },
        );
        let mut plans = vec![];
        for (i, ids) in residents.into_iter().enumerate() {
            let s = &mut self.sites[i];
            if ids.is_empty() || s.abandoned || s.stocks.stock[0] <= 0. {
                continue;
            }
            let need = s.demography.ages[..3]
                .iter()
                .zip([10., 18., 14.])
                .map(|(a, r)| (*a * r) as f64)
                .sum::<f64>();
            let price = s.economy.prices[crate::economy::FOOD].max(0.01) as f64;
            let mut municipal_work = s.economy.labor;
            municipal_work[3] = (municipal_work[3] - vessel_work[i]).max(0.);
            municipal_work[3] =
                (municipal_work[3] - s.economy.enterprise_plan.iter().sum::<f32>()).max(0.);
            let labor = municipal_work
                .iter()
                .sum::<f32>()
                .min(s.demography.ages[1])
                .max(0.) as f64;
            let payroll_request =
                (labor * 18. * price).min(s.economy.finance[0] as f64 * e.payroll_share as f64);
            let payroll = withdraw(&mut s.economy.finance[0], payroll_request);
            let dividend_request = s.economy.finance[0] as f64 * e.dividend_share as f64;
            let dividends = withdraw(&mut s.economy.finance[0], dividend_request);
            let shares = ids
                .iter()
                .map(|&id| society.households[id].share)
                .sum::<f64>();
            let work = municipal_work.map(|w| w.max(0.) as f64);
            let weights = ids
                .iter()
                .map(|&id| {
                    if e.occupational_payroll {
                        e.accounts[id].livelihood.unwrap_or([1.; 4])
                    } else {
                        [1.; 4]
                    }
                })
                .collect::<Vec<_>>();
            let paid_sectors = sector_payroll(&weights, work, payroll);
            let mut wage_left = payroll;
            let mut dividend_left = dividends;
            let free = need * e.common_share as f64;
            let mut demand = vec![];
            for (j, &id) in ids.iter().enumerate() {
                let wage = if j + 1 == ids.len() {
                    wage_left
                } else if e.occupational_payroll {
                    paid_sectors[j].iter().sum::<f64>().min(wage_left)
                } else {
                    payroll / ids.len() as f64
                };
                let dividend = if j + 1 == ids.len() {
                    dividend_left
                } else {
                    dividends * society.households[id].share / shares
                };
                wage_left -= wage;
                dividend_left -= dividend;
                let a = &mut e.accounts[id];
                if e.occupational_payroll {
                    for (k, paid) in paid_sectors[j].iter().enumerate() {
                        a.sector_wages[k] += paid;
                    }
                    // Slow adaptation follows available work, not household wealth.
                    let total_work = work.iter().sum::<f64>();
                    if total_work > 0. {
                        if let Some(weights) = &mut a.livelihood {
                            for k in 0..4 {
                                weights[k] = (weights[k] * 0.98
                                    + (1. + work[k] / total_work) * 0.02)
                                    .clamp(1., 2.);
                            }
                        }
                    }
                }
                a.wages += wage;
                a.dividends += dividend;
                a.cash += wage + dividend;
                a.need = need / ids.len() as f64;
                a.food_site = Some(i as u32);
                demand.push(((need - free) / ids.len() as f64).min(a.cash / price));
            }
            let cap = free + demand.iter().sum::<f64>();
            // Round down so GPU consumption cannot exceed funded entitlements.
            let mut cap32 = cap as f32;
            if cap32 as f64 > cap {
                cap32 = f32::from_bits(cap32.to_bits().saturating_sub(1));
            }
            s.demography.household_food = [cap32, 1., 0., 0.];
            plans.push(RetailPlan {
                site: i,
                ids,
                demand,
                need,
                free,
                price,
            });
        }
        // Gather all requests before spending any council budget: town ordering does not
        // confer first access to relief. Only the gap below the policy target is eligible.
        let mut requests = vec![vec![]; society.councils.len()];
        for p in &plans {
            for &id in &p.ids {
                let a = &e.accounts[id];
                let target = (e.relief_target - e.common_share).max(0.) as f64 * a.need * p.price;
                let request = (target - a.cash).max(0.);
                if request > 0. {
                    requests[controllers[p.site]].push((id, request));
                }
            }
        }
        for (council, requests) in society.councils.iter_mut().zip(requests) {
            let demand = requests.iter().map(|r| r.1).sum::<f64>();
            let budget = (council.treasury * e.relief_share as f64).min(demand);
            let grants =
                apportion_relief(&requests.iter().map(|r| r.1).collect::<Vec<_>>(), budget);
            let paid = grants.iter().sum::<f64>();
            for ((id, _), grant) in requests.iter().zip(grants) {
                e.accounts[*id].cash += grant;
                e.accounts[*id].relief += grant;
            }
            council.treasury -= paid;
            council.relief_paid += paid;
        }
        for p in &mut plans {
            for (j, &id) in p.ids.iter().enumerate() {
                p.demand[j] =
                    ((p.need - p.free) / p.ids.len() as f64).min(e.accounts[id].cash / p.price);
            }
            let cap = p.free + p.demand.iter().sum::<f64>();
            let mut cap32 = cap as f32;
            if cap32 as f64 > cap {
                cap32 = f32::from_bits(cap32.to_bits().saturating_sub(1));
            }
            self.sites[p.site].demography.household_food[0] = cap32;
        }
        plans
    }
    pub(crate) fn settle_household_retail(&mut self, plans: Vec<RetailPlan>) {
        let Some(e) = self
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
        else {
            return;
        };
        e.access_episodes.resize(self.sites.len(), [0; 3]);
        let mut notices = vec![];
        for p in plans {
            let s = &mut self.sites[p.site];
            let eaten = s.demography.ration_eaten[3] as f64;
            let blocked = (s.demography.household_food[2] as f64).min(p.need) - eaten;
            let excluded = blocked > p.need * 0.1 && p.need > 0.;
            let episode = &mut e.access_episodes[p.site];
            episode[0] = if excluded {
                episode[0].saturating_add(1)
            } else {
                0
            };
            episode[1] = if !excluded {
                episode[1].saturating_add(1)
            } else {
                0
            };
            if episode[0] >= 3 && episode[2] == 0 {
                episode[2] = 1;
                notices.push((p.site,"food_access_crisis",format!("Three months of purchasing-power shortages despite available food; {:.1} kg food equivalent left unconsumed this month for lack of funded entitlement",blocked)));
            } else if episode[1] >= 6 && episode[2] == 1 {
                episode[2] = 0;
                notices.push((p.site,"food_access_recovery","Six months without substantial purchasing-power exclusion; physical food shortages may still remain".into()));
            }
            let free = eaten.min(p.free);
            let paid = (eaten - free).max(0.);
            let demand = p.demand.iter().sum::<f64>();
            for (j, &id) in p.ids.iter().enumerate() {
                let a = &mut e.accounts[id];
                a.common_food = free / p.ids.len() as f64;
                a.purchased_food = if demand > 0. {
                    paid * p.demand[j] / demand
                } else {
                    0.
                };
                let payment = deposit(
                    &mut s.economy.finance[0],
                    (a.purchased_food * p.price).min(a.cash),
                );
                a.cash -= payment;
                a.food_spending += payment;
                a.hunger = if a.need > 0. {
                    (1. - (a.common_food + a.purchased_food) / a.need).clamp(0., 1.)
                } else {
                    0.
                };
            }
            debug_assert!(
                (p.need - s.demography.ration_need[3] as f64).abs() < 1e-5 * (1. + p.need)
            );
        }
        e.observed = self.month;
        for (site, kind, detail) in notices {
            let cause = self
                .events
                .iter()
                .rev()
                .find(|e| {
                    e.kind == "household_distribution_policy"
                        || (e.site == Some(site as u32) && e.kind == "food_access_crisis")
                })
                .map(|e| e.id);
            self.event(kind, Some(site as u32), None, detail);
            if let Some(cause) = cause {
                self.events.last_mut().unwrap().causes.push(cause);
            }
        }
    }
}

impl crate::gpu::Generator {
    /// Switch wage attribution without resetting livelihoods or funded wallets.
    pub fn set_occupational_payroll(&mut self, enabled: bool) -> Result<()> {
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "household policy requires a completed boundary"
        );
        self.validate_living_boundary()?;
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let economy = h
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
            .ok_or_else(|| anyhow::anyhow!("enable household economy first"))?;
        if economy.occupational_payroll != enabled {
            economy.occupational_payroll = enabled;
            h.event("household_wage_policy", None, None, format!("Industry-linked household payroll {} with existing wallets and livelihoods retained", if enabled { "enabled" } else { "disabled" }));
        }
        Ok(())
    }

    /// Explicit baseline for older worlds; policy changes apply before the next monthly production.
    pub fn configure_household_economy(
        &mut self,
        common_share: f32,
        payroll_share: f32,
        dividend_share: f32,
    ) -> Result<()> {
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "household policy requires a completed boundary"
        );
        self.validate_living_boundary()?;
        ensure!(
            [common_share, payroll_share, dividend_share]
                .iter()
                .all(|v| v.is_finite() && (0. ..=1.).contains(v)),
            "household policy fractions must be in 0–1"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let society = h
            .society
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable society first"))?;
        let e = society
            .household_economy
            .get_or_insert_with(|| HouseholdEconomy::new(h.month));
        e.common_share = common_share;
        e.payroll_share = payroll_share;
        e.dividend_share = dividend_share;
        h.event("household_distribution_policy",None,None,format!("Common food entitlement {:.0}% of need; monthly payroll capped at {:.0}% of settlement cash and dividends at {:.0}% of remaining cash; wallets begin with no invented capital",common_share*100.,payroll_share*100.,dividend_share*100.));
        Ok(())
    }
}

impl crate::gpu::Generator {
    pub fn configure_household_relief(
        &mut self,
        treasury_share: f32,
        food_target: f32,
    ) -> Result<()> {
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "relief policy requires a completed boundary"
        );
        self.validate_living_boundary()?;
        ensure!(
            [treasury_share, food_target]
                .iter()
                .all(|v| v.is_finite() && (0. ..=1.).contains(v)),
            "relief fractions must be in 0–1"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let e = h
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
            .ok_or_else(|| anyhow::anyhow!("enable household economy first"))?;
        e.relief_share = treasury_share;
        e.relief_target = food_target;
        h.event("household_distribution_policy",None,None,format!("Targeted food relief may spend {:.0}% of monthly council treasury to support {:.0}% dietary entitlement; towns share the budget in proportion to unmet purchasing power",treasury_share*100.,food_target*100.));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relief_is_need_weighted_and_old_archives_keep_zero_policy() {
        let requests = [10., 100., 0., 1.];
        for budget in [0., 5., 55., 1000.] {
            let grants = apportion_relief(&requests, budget);
            let reversed =
                apportion_relief(&requests.into_iter().rev().collect::<Vec<_>>(), budget);
            assert!((grants.iter().sum::<f64>() - budget.min(111.)).abs() < 1e-10);
            for ((&g, &r), &b) in grants.iter().zip(&requests).zip(reversed.iter().rev()) {
                assert!(g >= 0. && g <= r);
                assert!((g - b).abs() < 1e-10);
            }
        }
        let mut old = serde_json::to_value(HouseholdEconomy::new(0)).unwrap();
        old.as_object_mut().unwrap().remove("relief_share");
        old.as_object_mut().unwrap().remove("relief_target");
        let e: HouseholdEconomy = serde_json::from_value(old).unwrap();
        assert_eq!(e.relief_share, 0.);
        assert_eq!(e.relief_target, 0.);
        let a:HouseholdAccount=serde_json::from_value(serde_json::json!({"cash":0.,"wages":0.,"dividends":0.,"food_spending":0.,"estate_returned":0.,"need":0.,"common_food":0.,"purchased_food":0.,"hunger":0.})).unwrap();
        assert_eq!(a.relief, 0.);
    }
    #[test]
    fn sector_income_follows_work_and_preserves_payroll() {
        let weights = [[2., 1., 1., 1.], [1., 1., 1., 2.]];
        let farm = sector_payroll(&weights, [10., 0., 0., 0.], 90.);
        assert_eq!(farm, vec![[60., 0., 0., 0.], [30., 0., 0., 0.]]);
        let craft = sector_payroll(&weights, [0., 0., 0., 10.], 90.);
        assert_eq!(craft, vec![[0., 0., 0., 30.], [0., 0., 0., 60.]]);
        for budget in [0., 0.001, 100., 1e10] {
            let result = sector_payroll(&weights, [3., 2., 5., 7.], budget);
            assert!((result.iter().flatten().sum::<f64>() - budget).abs() <= 1e-6);
            let reverse = sector_payroll(&[weights[1], weights[0]], [3., 2., 5., 7.], budget);
            for k in 0..4 {
                assert!((result[0][k] - reverse[1][k]).abs() <= 1e-6);
            }
        }
        assert_eq!(sector_payroll(&weights, [0.; 4], 90.), vec![[0.; 4]; 2]);
        let mut old = serde_json::to_value(HouseholdEconomy::new(0)).unwrap();
        old.as_object_mut().unwrap().remove("occupational_payroll");
        assert!(
            !serde_json::from_value::<HouseholdEconomy>(old)
                .unwrap()
                .occupational_payroll
        );
    }
    #[test]
    fn pool_transfers_reconcile_at_float_boundaries() {
        for mut pool in [0., 0.001, 10., 10000., 1e10] {
            for request in [0., 0.00001, 0.1, 1., 9.9, 1234.] {
                let old = pool as f64;
                let withdrawn = withdraw(&mut pool, request);
                assert_eq!(old - pool as f64, withdrawn);
                let old = pool as f64;
                let credited = deposit(&mut pool, request);
                assert_eq!(pool as f64 - old, credited);
                assert!(credited >= 0. && credited <= request);
            }
        }
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn industry_income_changes_food_access_and_survives_serialization() {
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
        let h = g.civilizations.as_mut().unwrap();
        let ids = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|hh| hh.site == 0)
            .map(|hh| hh.id as usize)
            .collect::<Vec<_>>();
        assert!(ids.len() >= 2);
        let count = h.society.as_ref().unwrap().households.len();
        let e = h
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap();
        e.accounts.resize(count, HouseholdAccount::default());
        e.dividend_share = 0.;
        e.relief_share = 0.;
        e.common_share = 0.;
        for &id in &ids {
            e.accounts[id].livelihood = Some([1.; 4]);
        }
        e.accounts[ids[0]].livelihood = Some([2., 1., 1., 1.]);
        e.accounts[ids[1]].livelihood = Some([1., 1., 1., 2.]);
        h.sites[0].economy.labor = [10., 0., 0., 0.];
        h.sites[0].economy.finance[0] = 100.;
        let baseline = h.clone();
        let before = h.economy_residuals()[4];
        let plans = h.prepare_household_retail();
        let p = plans.iter().find(|p| p.site == 0).unwrap();
        assert!(p.demand[0] > p.demand[1]);
        assert!((h.economy_residuals()[4] - before).abs() < 1e-6);
        let farm = h.household_account(ids[0] as u32).unwrap().wages;
        let mut craft = baseline.clone();
        craft.sites[0].economy.labor = [0., 0., 0., 10.];
        craft.prepare_household_retail();
        assert!(craft.household_account(ids[0] as u32).unwrap().wages < farm);
        assert!(
            craft.household_account(ids[1] as u32).unwrap().wages
                > craft.household_account(ids[0] as u32).unwrap().wages
        );
        let mut restored: History =
            serde_json::from_slice(&serde_json::to_vec(h).unwrap()).unwrap();
        h.prepare_household_retail();
        restored.prepare_household_retail();
        assert_eq!(
            serde_json::to_value(&h.society.as_ref().unwrap().household_economy).unwrap(),
            serde_json::to_value(&restored.society.as_ref().unwrap().household_economy).unwrap()
        );
        h.society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap()
            .validate(h)
            .unwrap();
        let mut legacy = baseline;
        legacy
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .occupational_payroll = false;
        legacy.prepare_household_retail();
        assert!(
            (legacy.household_account(ids[0] as u32).unwrap().wages
                - legacy.household_account(ids[1] as u32).unwrap().wages)
                .abs()
                < 1e-8
        );
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn wallets_purchases_and_common_rations_reconcile() {
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
        assert!(g.configure_household_economy(f32::NAN, 0.2, 0.01).is_err());
        let h = g.civilizations.as_mut().unwrap();
        let ids = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|hh| hh.site == 0)
            .map(|hh| hh.id as usize)
            .collect::<Vec<_>>();
        assert!(ids.len() > 1);
        h.society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .occupational_payroll = false;
        // Unequal ownership affects dividends, not the equal-size household food approximation.
        for (j, &id) in ids.iter().enumerate() {
            h.society.as_mut().unwrap().households[id].share = if j == 0 {
                0.8
            } else {
                0.2 / (ids.len() - 1) as f64
            };
        }
        let before = h.economy_residuals()[4];
        let plans = h.prepare_household_retail();
        let e = h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        assert_eq!(e.accounts[ids[0]].need, e.accounts[ids[1]].need);
        assert!((e.accounts[ids[0]].wages - e.accounts[ids[1]].wages).abs() < 1e-6);
        assert!(e.accounts[ids[0]].dividends > e.accounts[ids[1]].dividends);
        assert!((h.economy_residuals()[4] - before).abs() < 1e-7);
        for p in &plans {
            h.sites[p.site].demography.ration_need[3] = p.need as f32;
            h.sites[p.site].demography.ration_eaten[3] =
                h.sites[p.site].demography.household_food[0];
        }
        h.settle_household_retail(plans);
        let e = h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        e.validate(h).unwrap();
        assert!((h.economy_residuals()[4] - before).abs() < 1e-6);
        let total = ids
            .iter()
            .map(|&id| e.accounts[id].common_food + e.accounts[id].purchased_food)
            .sum::<f64>();
        assert!((total - h.sites[0].demography.ration_eaten[3] as f64).abs() < 0.001);
        assert!(e.accounts[ids[0]].food_spending > 0.);
        let mut corrupt = e.clone();
        corrupt.accounts[0].cash += 100.;
        assert!(corrupt.validate(h).is_err());
        // A cashless town still receives the common ration; no wages or purchases are invented.
        h.society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .payroll_share = 0.;
        h.society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .dividend_share = 0.;
        for &id in &ids {
            let a = &mut h
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .accounts[id];
            a.estate_returned += a.cash;
            h.sites[0].economy.finance[0] += a.cash as f32;
            a.cash = 0.;
        }
        let plans = h.prepare_household_retail();
        let p = plans.iter().find(|p| p.site == 0).unwrap();
        assert_eq!(p.demand.iter().sum::<f64>(), 0.);
        assert!((h.sites[0].demography.household_food[0] as f64 - p.need * 0.5).abs() < 0.01);
        for month in 0..11 {
            h.month += 1;
            if month == 5 {
                h.society
                    .as_mut()
                    .unwrap()
                    .household_economy
                    .as_mut()
                    .unwrap()
                    .common_share = 1.;
            }
            let plans = h.prepare_household_retail();
            for p in &plans {
                h.sites[p.site].demography.ration_need[3] = p.need as f32;
                h.sites[p.site].demography.household_food[2] = p.need as f32;
                h.sites[p.site].demography.ration_eaten[3] =
                    h.sites[p.site].demography.household_food[0];
            }
            h.settle_household_retail(plans);
        }
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "food_access_crisis" && e.site == Some(0))
                .count(),
            1
        );
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "food_access_recovery" && e.site == Some(0))
                .count(),
            1
        );
        // Relief moves actual council cash into underfunded wallets, not site funds.
        let council = h.controller(0) as usize;
        let funding = withdraw(&mut h.sites[0].economy.finance[0], 1000.);
        h.society.as_mut().unwrap().councils[council].treasury += funding;
        let treasury = h.society.as_ref().unwrap().councils[council].treasury;
        let e = h
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap();
        e.common_share = 0.5;
        e.relief_share = 0.1;
        e.relief_target = 0.75;
        let before_relief = e.accounts.iter().map(|a| a.relief).sum::<f64>();
        let before_money = h.economy_residuals()[4];
        h.prepare_household_retail();
        let e = h
            .society
            .as_ref()
            .unwrap()
            .household_economy
            .as_ref()
            .unwrap();
        let paid = e.accounts.iter().map(|a| a.relief).sum::<f64>() - before_relief;
        assert!(paid > 0. && paid <= treasury * 0.100001);
        assert!(
            (h.society.as_ref().unwrap().councils[council].treasury + paid - treasury).abs() < 1e-6
        );
        assert!((h.economy_residuals()[4] - before_money).abs() < 1e-7);
        e.validate(h).unwrap();
        let mut old = serde_json::to_value(h.society.as_ref().unwrap()).unwrap();
        old.as_object_mut().unwrap().remove("household_economy");
        let old: crate::society::Society = serde_json::from_value(old).unwrap();
        assert!(old.household_economy.is_none());
    }
}
