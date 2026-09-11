//! Opt-in identity-owned demographic transitions; GPU still supplies ration and disease exposure.
use crate::{
    civilization::{History, Person},
    participation::Presence,
    population_registry::age_band,
};
use anyhow::{ensure, Result};

#[derive(Clone)]
pub(crate) struct Observation {
    month: u32,
    people: Vec<Vec<(u32, usize)>>,
    anonymous: Vec<[f64; 3]>,
    adults: Vec<f64>,
}
impl History {
    pub fn individual_demography_enabled(&self) -> bool {
        self.named_demography.as_ref().is_some_and(|s| s.individual)
    }
    /// Explicit conversion at a completed monthly boundary. The clone makes failed
    /// admission atomic; fractional unnamed stocks are preserved rather than rounded away.
    pub fn enable_individual_demography(&mut self) -> Result<()> {
        if self.individual_demography_enabled() {
            return Ok(());
        }
        let mut next = self.clone();
        next.set_named_demography(true)?;
        ensure!(
            next.household_relocations()
                .is_none_or(|r| r.journeys.is_empty())
                && next.person_duties.is_empty()
                && next.military.duties.is_empty(),
            "enable individual demographics before departure or after known travelers return"
        );
        next.identify_resident_baseline()?;
        ensure!(next.population_reconciliation().sites.iter().all(|s| s.overhang.iter().all(|v| *v < 1e-4)),
            "individual demography requires cohorts containing their known residents; existing overhang needs explicit conversion");
        let state = next.named_demography.as_mut().unwrap();
        state.individual = true;
        state.birth_remainder = vec![0.; next.sites.len()];
        state.month = Some(next.month);
        next.event("individual_demography_enabled", None, None,
            "Named residents now own birthdays, births and deaths; fractional unnamed residents remain explicit; population was not rounded or increased".into());
        *self = next;
        Ok(())
    }
    pub(crate) fn align_individual_arrival(&self, j: &mut crate::relocation::Journey) {
        if let Some(roster) = &j.roster {
            for entry in &roster.passengers {
                let person = &self.people[entry.person as usize];
                if person.died.is_none() {
                    // Opening arrivals enter the preceding completed age state;
                    // this month's birthday is committed once in execute.
                    let now = crate::population_registry::age_band(
                        self.month.saturating_sub(1),
                        person.born,
                    )
                    .unwrap();
                    if now != entry.band {
                        j.cohorts[entry.band] -= 1.;
                        j.cohorts[now] += 1.;
                    }
                }
            }
        }
    }
    pub(crate) fn individual_travel_losses(
        &mut self,
        journey: &mut crate::relocation::Journey,
        rate: f32,
    ) -> (f32, f32) {
        let mut named = [0u32; 3];
        let mut casualties = [0u32; 3];
        let mut ids = Vec::new();
        for entry in &journey.roster.as_ref().unwrap().passengers {
            if self.people[entry.person as usize].died.is_none() {
                named[entry.band] += 1;
                if crate::expeditions::random(self.seed, entry.person, self.month, 0x54524156)
                    < rate
                {
                    casualties[entry.band] += 1;
                    self.people[entry.person as usize].died = Some(self.month);
                    let account =
                        &mut self.society.as_mut().unwrap().households[journey.household as usize];
                    if account.head == entry.person {
                        account.vacant_since.get_or_insert(self.month);
                    }
                    ids.push(entry.person);
                }
            }
        }
        let mut anonymous_loss = 0.;
        for b in 0..3 {
            let lost = (journey.cohorts[b] - named[b] as f32).max(0.) * rate;
            anonymous_loss += lost;
            journey.cohorts[b] = (journey.cohorts[b] - lost - casualties[b] as f32).max(0.);
        }
        if !ids.is_empty() {
            self.event("individual_travel_deaths",Some(journey.from),Some(journey.to),format!("{} recorded passengers died from provision shortage; whole losses debited once from travel cohorts",ids.len()));
            let event = self.events.last_mut().unwrap();
            event.causes.push(journey.cause);
            event
                .subjects
                .extend(ids.iter().map(|&id| ("person".into(), id)));
        }
        (anonymous_loss + ids.len() as f32, anonymous_loss)
    }
    /// Defense casualties use real eligible residents just as deployed armies do.
    /// The caller debits the returned whole count once from stock and records the battle.
    pub(crate) fn individual_defender_losses(
        &mut self,
        site: u32,
        expected: f32,
        observed: &[u32],
    ) -> Vec<u32> {
        let state = self.named_demography.as_mut().unwrap();
        state.defense_remainder.resize(self.sites.len(), 0.);
        let mut eligible: Vec<_> = observed
            .iter()
            .copied()
            .filter(|&id| {
                self.people[id as usize].died.is_none()
                    && !self.person_duties.contains_key(&id)
                    && !self.military.duties.contains_key(&id)
            })
            .collect();
        eligible.sort_by_key(|&id| {
            (
                crate::expeditions::random(self.seed, id, self.month, 0x44454644).to_bits(),
                id,
            )
        });
        let target = f64::from(expected.max(0.)) + state.defense_remainder[site as usize];
        let count = (target.floor() as usize).min(eligible.len());
        state.defense_remainder[site as usize] = if count == eligible.len() {
            0.
        } else {
            target.fract()
        };
        eligible.truncate(count);
        for &id in &eligible {
            self.people[id as usize].died = Some(self.month);
        }
        if !eligible.is_empty() {
            self.event("resident_defense_deaths",Some(site),None,format!("{} resident defenders killed; identities and battle losses share one whole-person debit",eligible.len()));
            self.events
                .last_mut()
                .unwrap()
                .subjects
                .extend(eligible.iter().map(|&id| ("person".into(), id)));
        }
        eligible
    }
    pub(crate) fn observe_individual_demography(&self) -> Result<Option<Observation>> {
        if !self.individual_demography_enabled() {
            return Ok(None);
        }
        ensure!(
            self.named_demography.as_ref().unwrap().month != Some(self.month),
            "individual demographics already settled this month"
        );
        ensure!(
            self.month <= i32::MAX as u32,
            "individual demographic date exceeds identity calendar"
        );
        ensure!(
            self.society.is_some() && self.politics.is_some(),
            "individual demography requires membership"
        );
        for site in &self.sites {
            ensure!(
                site.demography.ages[1] <= 0.
                    || self
                        .society
                        .as_ref()
                        .unwrap()
                        .households
                        .iter()
                        .any(|h| h.site == site.id
                            && !self.society.as_ref().unwrap().relocation.away(h.id)),
                "individual birth capacity has no local membership account"
            );
        }
        let mut people = vec![Vec::new(); self.sites.len()];
        let mut known = vec![[0u32; 3]; self.sites.len()];
        for p in &self.people {
            if let Presence::Resident(site) = self.person_presence(p.id).1 {
                let band = age_band(self.month.saturating_sub(1), p.born)
                    .ok_or_else(|| anyhow::anyhow!("unsettled new resident birthday"))?;
                people[site as usize].push((p.id, band));
                known[site as usize][band] += 1;
            }
        }
        let mut anonymous = Vec::new();
        for (site, counts) in self.sites.iter().zip(&known) {
            ensure!(
                (0..3).all(|b| site.demography.ages[b].is_finite()
                    && site.demography.ages[b] + 1e-4 >= counts[b] as f32),
                "site {} month {} cohort {:?} cannot contain prior-age resident roster {:?}; recent events {:?}",
                site.id, self.month, &site.demography.ages[..3], counts,
                self.events.iter().rev().filter(|e| e.site == Some(site.id) || e.other == Some(site.id)).take(5).map(|e| e.kind.as_str()).collect::<Vec<_>>()
            );
            anonymous.push(std::array::from_fn(|b| {
                (site.demography.ages[b] as f64 - counts[b] as f64).max(0.)
            }));
        }
        let adults: Vec<_> = self
            .sites
            .iter()
            .map(|s| s.demography.ages[1] as f64)
            .collect();
        let possible_births: usize = adults
            .iter()
            .map(|n| (n * 0.004 + 1.).ceil() as usize)
            .sum();
        ensure!(
            self.politics
                .as_ref()
                .is_some_and(|p| p.kin.len() + possible_births <= 50000),
            "individual birth batch exceeds membership capacity"
        );
        Ok(Some(Observation {
            month: self.month,
            people,
            anonymous,
            adults,
        }))
    }
    pub(crate) fn settle_individual_demography(&mut self, observation: Observation) -> Result<()> {
        ensure!(
            observation.month == self.month
                && self.individual_demography_enabled()
                && self.named_demography.as_ref().unwrap().month != Some(self.month),
            "stale or repeated individual demographic settlement"
        );
        let mut state = self.named_demography.take().unwrap();
        state.birth_remainder.resize(self.sites.len(), 0.);
        for site in 0..self.sites.len() {
            let d = self.sites[site].demography;
            let hunger: [f64; 3] = std::array::from_fn(|b| {
                if d.ration_need[b] > 0. {
                    (1. - d.ration_eaten[b] / d.ration_need[b]).clamp(0., 1.) as f64
                } else {
                    0.
                }
            });
            let disease = d.health[0].clamp(0., 0.5) as f64;
            let rates: [f64; 3] = std::array::from_fn(|b| {
                [0.0005, 0.0006, 0.003][b] + hunger[b] * [0.06, 0.025, 0.05][b] + disease * 0.01
            });
            let mut counts = [0.; 3];
            let mut dead = Vec::new();
            for &(id, band) in &observation.people[site] {
                if (crate::expeditions::random(self.seed, id, self.month, 0x494e4444) as f64)
                    < rates[band]
                {
                    self.people[id as usize].died = Some(self.month);
                    dead.push(id);
                } else {
                    counts[age_band(self.month, self.people[id as usize].born).unwrap()] += 1.;
                }
            }
            let old = observation.anonymous[site];
            let loss: [f64; 3] = std::array::from_fn(|b| old[b] * rates[b]);
            let mature = old[0] / 180.;
            let retire = old[1] / 540.;
            let remaining = [
                old[0] - loss[0] - mature,
                old[1] - loss[1] + mature - retire,
                old[2] - loss[2] + retire,
            ];
            for b in 0..3 {
                counts[b] += remaining[b].max(0.);
            }
            let expected = observation.adults[site] * 0.004 * (1. - hunger[1]) * (1. - disease);
            let target = state.birth_remainder[site] + expected;
            let births = target.floor() as usize;
            state.birth_remainder[site] = target.fract();
            let homes: Vec<_> = self
                .society
                .as_ref()
                .unwrap()
                .households
                .iter()
                .filter(|h| {
                    h.site == site as u32 && !self.society.as_ref().unwrap().relocation.away(h.id)
                })
                .map(|h| h.id)
                .collect();
            // No population is born without somewhere to register it. Unsupported
            // settlement/account transitions surface before advancing another month.
            ensure!(
                births == 0 || !homes.is_empty(),
                "individual birth has no resident membership account"
            );
            let mut born = Vec::new();
            for index in 0..births {
                let id = self.people.len() as u32;
                let family = self
                    .politics
                    .as_ref()
                    .unwrap()
                    .marriages
                    .iter()
                    .enumerate()
                    .find(|(_, m)| {
                        m.ended.is_none()
                            && m.children < 4
                            && self.month.saturating_sub(m.last_birth) >= 36
                            && m.partners.iter().all(|&id| {
                                self.person_presence(id).1 == Presence::Resident(site as u32)
                                    && (216..540).contains(
                                        &(i64::from(self.month)
                                            - i64::from(self.people[id as usize].born)),
                                    )
                            })
                    })
                    .map(|(i, m)| (i, m.partners));
                let household = family
                    .and_then(|(_, parents)| self.person_presence(parents[0]).0)
                    .unwrap_or(homes[index % homes.len()]);
                let town = &self.sites[site];
                self.people.push(Person {
                    id,
                    name: self.civilizations[town.civilization as usize]
                        .naming(self.seed)
                        .person_with(
                            "person",
                            id,
                            &crate::naming::PersonalContext::local(town, self.culture.as_ref()),
                        ),
                    civilization: town.civilization,
                    born: self.month as i32,
                    died: None,
                    predecessor: None,
                });
                self.politics
                    .as_mut()
                    .unwrap()
                    .kin
                    .push(crate::politics::Kinship {
                        person: id,
                        household,
                        parents: family.map_or([None; 2], |(_, parents)| parents.map(Some)),
                    });
                if let Some((family, _)) = family {
                    let m = &mut self.politics.as_mut().unwrap().marriages[family];
                    m.children += 1;
                    m.last_birth = self.month;
                }
                born.push(id);
            }
            counts[0] += births as f64;
            let s = &mut self.sites[site];
            for (b, value) in counts.into_iter().enumerate() {
                s.demography.ages[b] = value as f32;
            }
            s.stocks.stock[0] = s.demography.ages[..3].iter().sum();
            s.stocks.people[0] += births as f32;
            s.stocks.people[1] += dead.len() as f32 + loss.iter().sum::<f64>() as f32;
            // Only anonymous losses remain available for later identification.
            s.demography.health[2] += loss.iter().sum::<f64>() as f32;
            state.assigned += dead.len() as u64;
            for (kind, ids) in [("individual_births", born), ("individual_deaths", dead)] {
                if !ids.is_empty() {
                    self.event(kind,Some(site as u32),None,format!("{} individual transitions committed once to identities and population; rates use completed ration and disease exposure",ids.len()));
                    self.events
                        .last_mut()
                        .unwrap()
                        .subjects
                        .extend(ids.into_iter().map(|id| ("person".into(), id)));
                }
            }
        }
        state.month = Some(self.month);
        self.named_demography = Some(state);
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
    fn world() -> Generator {
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
        g.enable_politics().unwrap();
        g
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn travel_birthdays_and_mortality_preserve_named_and_anonymous_counts() {
        use crate::relocation::{Journey, Passenger, TravelRoster};
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.enable_individual_demography().unwrap();
        let ids: Vec<_> = h
            .people
            .iter()
            .filter(|p| h.person_presence(p.id).1 == Presence::Resident(0))
            .take(2)
            .map(|p| p.id)
            .collect();
        h.month = 10;
        // One birthday completed while traveling; one occurs this arrival month.
        h.people[ids[0] as usize].born = 9 - 180;
        h.people[ids[1] as usize].born = 10 - 180;
        let mut j = Journey {
            roster: Some(TravelRoster {
                passengers: ids
                    .iter()
                    .map(|&person| Passenger { person, band: 0 })
                    .collect(),
                death_carry: [0.; 3],
            }),
            household: 0,
            from: 0,
            to: 1,
            route: 0,
            departed: 8,
            arrives: 10,
            cohorts: [2.5, 0.25, 0.25],
            food: 0.,
            cash: 0.,
            tools: 0.,
            cause: 0,
            blocked: false,
            returning: false,
            seek_help: false,
            report_population: 0.,
            report_food_months: None,
        };
        let mut arrived = j.clone();
        h.align_individual_arrival(&mut arrived);
        assert_eq!(arrived.cohorts, [1.5, 1.25, 0.25]);
        assert_eq!(arrived.population(), j.population());
        // Analytical total-loss fixture: two identities plus one anonymous person.
        let (total, anonymous) = h.individual_travel_losses(&mut j, 1.);
        assert_eq!((total, anonymous), (3., 1.));
        assert_eq!(j.cohorts, [0.; 3]);
        assert!(ids.iter().all(|&id| h.people[id as usize].died == Some(10)));
        // Settling an already empty manifest must not claim those deaths again.
        assert_eq!(h.individual_travel_losses(&mut j, 1.), (0., 0.));
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn resident_defense_uses_whole_identities_and_persisted_fractional_expectations() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.enable_individual_demography().unwrap();
        let ids: Vec<_> = h
            .people
            .iter()
            .filter(|p| {
                age_band(h.month, p.born) == Some(1)
                    && h.person_presence(p.id).1 == Presence::Resident(0)
            })
            .map(|p| p.id)
            .collect();
        assert!(h.individual_defender_losses(0, 0.75, &ids).is_empty());
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        let deaths = h.individual_defender_losses(0, 0.5, &ids);
        assert_eq!(deaths.len(), 1);
        assert_eq!(deaths, resumed.individual_defender_losses(0, 0.5, &ids));
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        assert_eq!(
            h.named_demography.as_ref().unwrap().defense_remainder[0],
            0.25
        );
        assert!(h.people[deaths[0] as usize].died.is_some());
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn individual_birthdays_population_and_checkpoint_share_one_authority() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        let before = h.population_residual();
        h.enable_individual_demography().unwrap();
        assert!((h.population_residual() - before).abs() < 1e-6);
        let known = h.people.len();
        h.enable_individual_demography().unwrap();
        assert_eq!(known, h.people.len());
        let mut birthday = Vec::new();
        for (band, age) in [(0, 179), (1, 719)] {
            let id = h
                .people
                .iter()
                .find(|p| {
                    age_band(h.month, p.born) == Some(band)
                        && h.person_presence(p.id).1 == Presence::Resident(0)
                        && crate::expeditions::random(h.seed, p.id, h.month + 1, 0x494e4444) > 0.2
                })
                .unwrap()
                .id;
            h.people[id as usize].born = h.month as i32 - age;
            birthday.push(id);
        }
        g.advance_history(1).unwrap();
        let h = g.civilizations.as_ref().unwrap();
        assert_eq!(
            age_band(h.month, h.people[birthday[0] as usize].born),
            Some(1)
        );
        assert_eq!(
            age_band(h.month, h.people[birthday[1] as usize].born),
            Some(2)
        );
        assert!(h
            .population_reconciliation()
            .sites
            .iter()
            .all(|s| s.overhang.iter().all(|v| *v < 1e-4)));
        assert!((h.population_residual() - before).abs() < 1e-5);
        let baseline = h.clone();
        g.advance_history(24).unwrap();
        let expected = serde_json::to_value(&g.civilizations).unwrap();
        g.civilizations =
            Some(serde_json::from_value(serde_json::to_value(baseline).unwrap()).unwrap());
        for _ in 0..24 {
            g.advance_history(1).unwrap();
        }
        assert_eq!(expected, serde_json::to_value(&g.civilizations).unwrap());
        let h = g.civilizations.as_ref().unwrap();
        assert!(h
            .population_reconciliation()
            .sites
            .iter()
            .all(|s| s.overhang.iter().all(|v| *v < 1e-4)));
        assert!(h.events.iter().any(|e| e.kind == "individual_births"));
        assert!(h.events.iter().any(|e| e.kind == "individual_deaths"));
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn individual_demography_refuses_incompatible_stock_without_mutation() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.sites[0].demography.ages[1] = 0.;
        let old = serde_json::to_value(&*h).unwrap();
        assert!(h.enable_individual_demography().is_err());
        assert_eq!(old, serde_json::to_value(&*h).unwrap());
    }
}
