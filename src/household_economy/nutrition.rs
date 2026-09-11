//! Pure allocation views over food already consumed by the GPU; no extra food debit.
use super::RetailPlan;
use crate::{civilization::History, participation::Presence, population_registry::age_band};
use std::collections::BTreeMap;

/// Known members anchor age-weighted need. Unrepresented cohort stock is shared
/// explicitly across accounts. Sparse/overhanging identities cannot enlarge need.
pub(super) fn food_needs(ages: [f64; 3], members: &[[f64; 3]]) -> Vec<f64> {
    if members.is_empty() {
        return vec![];
    }
    let known: [f64; 3] = std::array::from_fn(|b| members.iter().map(|n| n[b]).sum());
    members
        .iter()
        .map(|m| {
            (0..3)
                .map(|b| {
                    let scale = if known[b] > 0. {
                        (ages[b] / known[b]).min(1.)
                    } else {
                        0.
                    };
                    (m[b] * scale + (ages[b] - known[b]).max(0.) / members.len() as f64)
                        * [10., 18., 14.][b]
                })
                .sum()
        })
        .collect()
}
impl RetailPlan {
    pub(super) fn food_allocation(&self, eaten: f64) -> Vec<(usize, f64, f64)> {
        let free = eaten.min(self.free);
        let paid = (eaten - free).max(0.);
        let demand = self.demand.iter().sum::<f64>();
        let total_need = self.needs.iter().sum::<f64>();
        self.ids
            .iter()
            .enumerate()
            .map(|(j, &id)| {
                (
                    id,
                    free * self.needs[j] / total_need.max(1e-12),
                    if demand > 0. {
                        paid * self.demand[j] / demand
                    } else {
                        0.
                    },
                )
            })
            .collect()
    }
    fn household_hunger(&self, eaten: f64) -> Vec<(usize, f64)> {
        self.food_allocation(eaten)
            .into_iter()
            .enumerate()
            .map(|(j, (id, common, paid))| {
                (
                    id,
                    if self.needs[j] > 0. {
                        (1. - (common + paid) / self.needs[j]).clamp(0., 1.)
                    } else {
                        0.
                    },
                )
            })
            .collect()
    }
}
impl History {
    pub(super) fn household_food_members(&self) -> BTreeMap<usize, [f64; 3]> {
        let mut members = BTreeMap::new();
        if let Some(pool) = &self.participation {
            for r in pool.residents.values() {
                let (Some(hh), Presence::Resident(_)) = (r.household, r.presence) else {
                    continue;
                };
                let Some(person) = self.people.get(r.person as usize) else {
                    continue;
                };
                if person.died.is_some() {
                    continue;
                }
                if let Some(band) = age_band(self.month.saturating_sub(1), person.born) {
                    members.entry(hh as usize).or_insert([0.; 3])[band] += 1.;
                }
            }
        }
        members
    }
    /// Current-month realized entitlement, observed before demographic commit and
    /// later used unchanged by retail settlement. Aggregate mortality is untouched.
    pub(crate) fn household_mortality(&self, plans: &[RetailPlan]) -> BTreeMap<u32, f64> {
        if !self
            .society
            .as_ref()
            .and_then(|s| s.household_economy.as_ref())
            .is_some_and(|e| e.individual_nutrition)
        {
            return BTreeMap::new();
        }
        let mut hunger = BTreeMap::new();
        for plan in plans {
            let eaten = self.sites[plan.site].demography.ration_eaten[3] as f64;
            for (hh, value) in plan.household_hunger(eaten) {
                hunger.insert(hh, (plan.site, value));
            }
        }
        let mut rates = BTreeMap::new();
        if let Some(pool) = &self.participation {
            for r in pool.residents.values() {
                let (Some(hh), Presence::Resident(site)) = (r.household, r.presence) else {
                    continue;
                };
                let Some(&(observed_site, shortage)) = hunger.get(&(hh as usize)) else {
                    continue;
                };
                if observed_site != site as usize {
                    continue;
                }
                let Some(p) = self.people.get(r.person as usize) else {
                    continue;
                };
                if p.died.is_some() {
                    continue;
                }
                if let Some(band) = age_band(self.month.saturating_sub(1), p.born) {
                    let disease =
                        self.sites[site as usize].demography.health[0].clamp(0., 0.5) as f64;
                    rates.insert(
                        r.person,
                        [0.0005, 0.0006, 0.003][band]
                            + shortage * [0.06, 0.025, 0.05][band]
                            + disease * 0.01,
                    );
                }
            }
        }
        rates
    }
    pub(crate) fn household_work_nutrition(&self, household: Option<u32>, site: u32) -> f32 {
        let Some(e) = self
            .society
            .as_ref()
            .and_then(|s| s.household_economy.as_ref())
        else {
            return 1.;
        };
        if !e.individual_nutrition || e.observed.saturating_add(1) != self.month {
            return 1.;
        }
        let Some(a) = household.and_then(|id| e.accounts.get(id as usize)) else {
            return 1.;
        };
        if a.food_site != Some(site) || a.need <= 0. {
            return 1.;
        }
        // Toy short-term weakness, separate from the common disease exposure.
        1. - 0.35 * a.hunger.clamp(0., 1.) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn membership_weights_need_without_creating_food_demand() {
        assert_eq!(
            food_needs([2., 2., 1.], &[[2., 1., 0.], [0., 1., 1.]]),
            vec![38., 32.]
        );
        assert_eq!(food_needs([2., 2., 1.], &[[0.; 3]; 2]), vec![35., 35.]);
        // Sparse identity overhang is bounded to the actual cohort, not extra mouths.
        assert_eq!(
            food_needs([0., 1., 0.], &[[0., 2., 0.], [0., 2., 0.]]),
            vec![9., 9.]
        );
        let p = RetailPlan {
            site: 0,
            ids: vec![0, 1],
            needs: vec![38., 32.],
            need: 70.,
            free: 35.,
            demand: vec![19., 0.],
            price: 1.,
        };
        let food = p.food_allocation(54.);
        assert_eq!(food, vec![(0, 19., 19.), (1, 16., 0.)]);
        assert_eq!(p.household_hunger(54.), vec![(0, 0.), (1, 0.5)]);
        assert_eq!(
            p.food_allocation(10.)
                .iter()
                .map(|(_, a, b)| a + b)
                .sum::<f64>(),
            10.
        );
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn scarcity_seed_256_never_reverses_workshop_production() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                seed: 256,
                resolution: 32,
                ecology_resolution: 32,
                crop_yield_scale: 0.33,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        g.enable_offices().unwrap();
        g.enable_shipping().unwrap();
        g.civilizations
            .as_mut()
            .unwrap()
            .set_demographic_resolution(crate::resolution::Mode::Individual, true)
            .unwrap();
        for month in 1..=48 {
            g.advance_history(1)
                .unwrap_or_else(|e| panic!("month {month}: {e:#}"));
            let h = g.civilizations.as_ref().unwrap();
            assert!(h.population_residual().abs() < 1e-6);
            for site in &h.sites {
                assert!(site
                    .economy
                    .enterprise_used
                    .iter()
                    .all(|v| v.is_finite() && *v >= 0.));
                assert!(site
                    .economy
                    .made
                    .iter()
                    .chain(&site.economy.used)
                    .all(|v| v.is_finite() && *v >= 0.));
            }
        }
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn funded_family_food_changes_exposure_and_next_month_capacity() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
        };
        let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
        for seed in [17, 81, 256] {
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    resolution: 32,
                    ecology_resolution: 16,
                    seed,
                    ..Default::default()
                },
                Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.found_civilizations(5).unwrap();
            g.enable_society().unwrap();
            g.enable_politics().unwrap();
            let h = g.civilizations.as_mut().unwrap();
            h.enable_individual_demography().unwrap();
            h.month += 1;
            h.begin_service_reservations();
            let accounts_len = h.society.as_ref().unwrap().households.len();
            let e = h
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap();
            e.payroll_share = 0.;
            e.dividend_share = 0.;
            e.relief_share = 0.;
            e.founding_access = None; // Isolate unequal retail access after the founding phase.
            e.accounts.resize(accounts_len, Default::default());
            let members = h.household_food_members();
            let accounts: Vec<_> = h
                .society
                .as_ref()
                .unwrap()
                .households
                .iter()
                .filter(|hh| {
                    hh.site == 0 && members.get(&(hh.id as usize)).is_some_and(|m| m[1] > 0.)
                })
                .map(|hh| hh.id)
                .take(2)
                .collect();
            assert_eq!(accounts.len(), 2);
            let paid = super::super::withdraw(&mut h.sites[0].economy.finance[0], 500.);
            let a = &mut h
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .accounts[accounts[0] as usize];
            a.cash += paid;
            a.wages += paid;
            let plans = h.prepare_household_retail();
            for p in &plans {
                let d = &mut h.sites[p.site].demography;
                d.ration_need[3] = p.need as f32;
                d.ration_eaten[3] = d.household_food[0];
            }
            let exposure = h.household_mortality(&plans);
            let mut control = h.clone();
            control
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .individual_nutrition = false;
            assert!(control.household_mortality(&plans).is_empty());
            assert_eq!(
                serde_json::to_value(
                    &h.society
                        .as_ref()
                        .unwrap()
                        .household_economy
                        .as_ref()
                        .unwrap()
                        .accounts
                )
                .unwrap(),
                serde_json::to_value(
                    &control
                        .society
                        .as_ref()
                        .unwrap()
                        .household_economy
                        .as_ref()
                        .unwrap()
                        .accounts
                )
                .unwrap()
            );
            h.settle_household_retail(plans);
            let e = h
                .society
                .as_ref()
                .unwrap()
                .household_economy
                .as_ref()
                .unwrap();
            assert!(
                e.accounts[accounts[0] as usize].hunger < e.accounts[accounts[1] as usize].hunger
            );
            let adults: Vec<_> = accounts
                .iter()
                .map(|hh| {
                    h.participation
                        .as_ref()
                        .unwrap()
                        .residents
                        .values()
                        .find(|r| {
                            r.household == Some(*hh)
                                && age_band(h.month - 1, h.people[r.person as usize].born)
                                    == Some(1)
                        })
                        .unwrap()
                        .person
                })
                .collect();
            assert!(exposure[&adults[0]] < exposure[&adults[1]]);
            h.month += 1;
            assert!(
                h.household_work_nutrition(Some(accounts[0]), 0)
                    > h.household_work_nutrition(Some(accounts[1]), 0)
            );
            let factors = adults
                .iter()
                .map(|id| {
                    let r = &h.participation.as_ref().unwrap().residents[id];
                    h.household_work_nutrition(r.household, 0)
                })
                .collect::<Vec<_>>();
            h.open_participation();
            for (j, id) in adults.iter().enumerate() {
                let r = &h.participation.as_ref().unwrap().residents[id];
                let expected =
                    0.8 * (1. - 0.5 * h.sites[0].demography.health[0].clamp(0., 0.5)) * factors[j];
                assert!((r.capacity + r.care - expected).abs() < 1e-6);
            }
            assert_eq!(h.household_work_nutrition(Some(accounts[1]), 1), 1.);
            control.month = h.month;
            control
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .accounts = h
                .society
                .as_ref()
                .unwrap()
                .household_economy
                .as_ref()
                .unwrap()
                .accounts
                .clone();
            control
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .observed = h.month - 1;
            assert_eq!(control.household_work_nutrition(Some(accounts[1]), 0), 1.);
            let mut old = serde_json::to_value(
                h.society
                    .as_ref()
                    .unwrap()
                    .household_economy
                    .as_ref()
                    .unwrap(),
            )
            .unwrap();
            old.as_object_mut().unwrap().remove("individual_nutrition");
            assert!(
                serde_json::from_value::<super::super::HouseholdEconomy>(old)
                    .unwrap()
                    .individual_nutrition
            );
            let restored: History =
                serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
            assert_eq!(
                h.household_work_nutrition(Some(accounts[1]), 0),
                restored.household_work_nutrition(Some(accounts[1]), 0)
            );
        }
    }
}
