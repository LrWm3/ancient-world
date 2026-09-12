//! Shared military identities over existing cohort manpower. Local defenders still
//! use aggregate casualties; older armies without rosters retain aggregate behavior.
use crate::{civilization::History, participation::Recruitment, society::Raid};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Military {
    #[serde(default)]
    pub siege: crate::siege::State,
    pub duties: BTreeMap<u32, Duty>,
    pub careers: BTreeMap<u32, Career>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Duty {
    pub army: u32,
    pub origin: u32,
    pub household: Option<u32>,
    pub started: u32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Career {
    pub campaigns: u32,
    pub months_served: u32,
}
impl Military {
    pub fn validate(&self, h: &History) -> Result<()> {
        self.siege.validate(h)?;
        let mut present = BTreeSet::new();
        if let Some(s) = &h.society {
            for raid in &s.raids {
                ensure!(
                    raid.loss_remainder.is_finite() && (0. ..1.).contains(&raid.loss_remainder),
                    "invalid military casualty remainder"
                );
                if let Some(members) = &raid.members {
                    ensure!(
                        raid.soldiers == members.len() as f32,
                        "army roster/manpower mismatch"
                    );
                    for &id in members {
                        ensure!(
                            present.insert(id)
                                && h.people.get(id as usize).is_some_and(|p| p.died.is_none())
                                && !h.person_duties.contains_key(&id),
                            "duplicate or dead military member"
                        );
                        ensure!(
                            self.duties.get(&id).is_some_and(|d| d.army == raid.id
                                && d.origin == raid.origin
                                && d.started <= h.month
                                && d.household.is_none_or(|hh| s
                                    .households
                                    .get(hh as usize)
                                    .is_some_and(|f| f.site == d.origin)
                                    && !s.relocation.away(hh))),
                            "invalid military duty"
                        );
                        ensure!(
                            self.careers.contains_key(&id),
                            "military member has no career"
                        );
                    }
                }
            }
        }
        ensure!(
            present.len() == self.duties.len() && self.duties.keys().all(|id| present.contains(id)),
            "orphaned military duty"
        );
        ensure!(
            self.careers
                .iter()
                .all(|(id, c)| (*id as usize) < h.people.len() && c.campaigns > 0),
            "invalid military career"
        );
        Ok(())
    }
}
impl History {
    pub(crate) fn assign_military_people(&mut self, army: u32, origin: u32, people: &[u32]) {
        for &person in people {
            let household = self.person_presence(person).0;
            self.military.duties.insert(
                person,
                Duty {
                    army,
                    origin,
                    household,
                    started: self.month,
                },
            );
            let career = self.military.careers.entry(person).or_default();
            career.campaigns = career.campaigns.saturating_add(1);
        }
    }
    pub(crate) fn record_recruitment(&mut self, recruits: &Recruitment) {
        let event = self.events.last_mut().unwrap();
        event
            .subjects
            .extend(recruits.people.iter().map(|&p| ("person".into(), p)));
        if recruits.identified > 0 {
            event.detail.push_str(&format!(
                "; {} previously unnamed cohort adults identified with estimated ages and unknown parents",
                recruits.identified));
        }
    }
    pub(crate) fn record_military_members(&mut self, raid: &Raid) {
        if let Some(people) = &raid.members {
            self.events
                .last_mut()
                .unwrap()
                .subjects
                .extend(people.iter().map(|&p| ("person".into(), p)));
        }
    }
    /// Bounded preparedness from actual service, not additional manpower or equipment.
    pub(crate) fn military_preparedness(&self, raid: &Raid) -> f32 {
        let Some(people) = &raid.members else {
            return 1.;
        };
        if people.is_empty() {
            return 1.;
        }
        1. + 0.15
            * people
                .iter()
                .map(|p| {
                    self.military
                        .careers
                        .get(p)
                        .map_or(0., |c| (c.months_served as f32 / 120.).min(1.))
                })
                .sum::<f32>()
            / people.len() as f32
    }
    pub(crate) fn advance_military_experience(&mut self, raid: &Raid) {
        for &id in raid.members.iter().flatten() {
            if self
                .military
                .duties
                .get(&id)
                .is_some_and(|d| d.started < self.month)
            {
                let career = self.military.careers.entry(id).or_default();
                career.months_served = career.months_served.saturating_add(1);
            }
        }
    }
    /// Expected fractional losses accumulate; only actual deaths debit population.
    pub(crate) fn military_losses(&mut self, raid: &mut Raid, expected: f32, reason: &str) -> f32 {
        let deaths = if let Some(members) = &mut raid.members {
            let expected = expected.max(0.).min(raid.soldiers) + raid.loss_remainder;
            let count = (expected.floor() as usize).min(members.len());
            raid.loss_remainder = if count == members.len() {
                0.
            } else {
                expected - count as f32
            };
            // Order-independent identity selection, with a subsystem-specific stream.
            members.sort_by_key(|&p| {
                crate::expeditions::random(self.seed, p, self.month, raid.id.wrapping_add(991))
                    .to_bits()
            });
            let lost: Vec<_> = members.drain(..count).collect();
            members.sort_unstable();
            for &id in &lost {
                self.people[id as usize].died = Some(self.month);
                self.military.duties.remove(&id);
            }
            if count > 0 {
                self.event(
                    "military_deaths",
                    Some(raid.origin),
                    Some(raid.target),
                    format!(
                        "Army {} lost {} named residents to {}",
                        raid.id, count, reason
                    ),
                );
                let event = self.events.last_mut().unwrap();
                event.causes.push(raid.cause);
                event
                    .subjects
                    .extend(lost.iter().map(|&p| ("person".into(), p)));
                raid.cause = event.id;
            }
            count as f32
        } else {
            expected.max(0.).min(raid.soldiers)
        };
        raid.soldiers -= deaths;
        self.sites[raid.origin as usize].stocks.people[1] += deaths;
        deaths
    }
    /// No surviving carriers: close the army and declare its stranded stores as losses.
    pub(crate) fn close_empty_army(&mut self, raid: &Raid) -> bool {
        if raid.soldiers > 0. {
            return false;
        }
        self.end_siege(raid.id, "army lost");
        let site = &mut self.sites[raid.origin as usize];
        site.stocks.ledger[1] += raid.food;
        for (k, v) in crate::economy::FOOD_CNP.iter().enumerate() {
            site.economy.external[k] -= raid.food * *v as f32;
        }
        site.economy.used[3] += raid.equipment;
        site.economy.reserves[3] += raid.equipment;
        if raid.war.is_some_and(|id| {
            self.politics
                .as_ref()
                .is_some_and(|p| p.wars[id as usize].ended.is_none())
        }) {
            self.resolve_war(raid, false);
        }
        self.event("army_lost", Some(raid.origin), Some(raid.target),
            format!("Army {} has no surviving carriers; {:.1} kg provisions spoiled and {:.1} kg equipment stranded",
                raid.id, raid.food, raid.equipment));
        self.events.last_mut().unwrap().causes.push(raid.cause);
        true
    }
    pub(crate) fn return_military_people(&mut self, raid: &Raid) {
        let mut elders = 0.;
        if let Some(people) = &raid.members {
            for &id in people {
                self.military.duties.remove(&id);
                if self.month as i32 - self.people[id as usize].born >= 720 {
                    elders += 1.;
                }
            }
        }
        let site = &mut self.sites[raid.origin as usize];
        site.stocks.stock[0] += raid.soldiers;
        site.demography.ages[1] += raid.soldiers - elders;
        site.demography.ages[2] += elders;
        site.stocks.people[2] += raid.soldiers;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
        participation::Presence,
    };

    #[test]
    #[ignore = "requires hardware GPU"]
    fn named_campaign_presence_losses_return_and_legacy_import() {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(16).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        let h = g.civilizations.as_ref().unwrap();
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .find(|r| r.cost_km < 900. && h.controller(r.from) != h.controller(r.to))
            .unwrap()
            .clone();
        let before = h.population_residual();
        g.declare_war(route.from, route.to).unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.military.validate(h).unwrap();
        assert!((h.population_residual() - before).abs() < 1e-5);
        let mut raid = h.society.as_ref().unwrap().raids[0].clone();
        let members = raid.members.clone().unwrap();
        crate::military_supply::verify_supply_comparison(h);
        assert!(members.len() >= 3);
        assert!(members
            .iter()
            .all(|&id| h.person_presence(id).1 == Presence::Military(raid.id)));
        // A second service request cannot reuse military travelers.
        if let Ok(next) = h.recruit_service_people(route.from, 8, 3) {
            assert!(next.people.iter().all(|id| !members.contains(id)));
        }
        h.open_participation();
        assert!(members
            .iter()
            .all(|id| h.participation.as_ref().unwrap().available(*id) == 0.));

        let mut corrupt = h.clone();
        corrupt.person_duties.insert(
            members[0],
            crate::participation::TravelDuty {
                voyage: 0,
                origin: route.from,
                household: h.person_presence(members[0]).0,
            },
        );
        assert!(corrupt.military.validate(&corrupt).is_err());
        let mut corrupt = h.clone();
        corrupt.society.as_mut().unwrap().raids[0].soldiers += 1.;
        assert!(corrupt.military.validate(&corrupt).is_err());

        // An old archive does not invent identities for people already at war.
        let mut old = serde_json::to_value(&*h).unwrap();
        old.as_object_mut().unwrap().remove("military");
        for r in old["society"]["raids"].as_array_mut().unwrap() {
            r.as_object_mut().unwrap().remove("members");
            r.as_object_mut().unwrap().remove("loss_remainder");
        }
        let mut legacy: History = serde_json::from_value(old).unwrap();
        legacy.military.validate(&legacy).unwrap();
        crate::military_supply::verify_supply_comparison(&legacy);
        let mut legacy_raid = legacy.society.as_ref().unwrap().raids[0].clone();
        assert_eq!(
            legacy.military_losses(&mut legacy_raid, 0.25, "legacy fixture"),
            0.25
        );

        // Smaller expected losses do not create fractional dead people.
        let local_death_credit = h.sites[route.from as usize].demography.health[2];
        assert_eq!(
            h.military_losses(&mut raid, 0.75, "controlled shortage"),
            0.
        );
        let mut continued: History =
            serde_json::from_slice(&serde_json::to_vec(&*h).unwrap()).unwrap();
        let mut resumed_raid: Raid =
            serde_json::from_slice(&serde_json::to_vec(&raid).unwrap()).unwrap();
        assert_eq!(h.military_losses(&mut raid, 0.5, "controlled combat"), 1.);
        assert_eq!(
            continued.military_losses(&mut resumed_raid, 0.5, "controlled combat"),
            1.
        );
        assert_eq!(
            serde_json::to_value(&raid).unwrap(),
            serde_json::to_value(&resumed_raid).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&continued).unwrap()
        );
        assert_eq!(raid.loss_remainder, 0.25);
        let lost: Vec<_> = members
            .iter()
            .filter(|id| !raid.members.as_ref().unwrap().contains(id))
            .copied()
            .collect();
        assert_eq!(lost.len(), 1);
        assert_eq!(h.people[lost[0] as usize].died, Some(h.month));
        assert!(!h.military.duties.contains_key(&lost[0]));
        assert_eq!(
            h.sites[route.from as usize].demography.health[2],
            local_death_credit
        );
        h.society.as_mut().unwrap().raids[0] = raid.clone();
        h.military.validate(h).unwrap();
        assert!((h.population_residual() - before).abs() < 1e-5);
        // Preparedness is tied to actual recorded service and is capped.
        let untrained = h.military_preparedness(&raid);
        h.month += 1;
        h.advance_military_experience(&raid);
        assert!(h.military_preparedness(&raid) > untrained);
        for c in h.military.careers.values_mut() {
            c.months_served = 1000;
        }
        assert!((h.military_preparedness(&raid) - 1.15).abs() < 1e-6);
        let mut lost_army = h.clone();
        let mut empty = raid.clone();
        let expected = empty.soldiers;
        lost_army.military_losses(&mut empty, expected, "controlled total loss");
        assert!(lost_army.close_empty_army(&empty));
        lost_army.society.as_mut().unwrap().raids.clear();
        assert!(lost_army.military.duties.is_empty());
        lost_army.military.validate(&lost_army).unwrap();
        assert!((lost_army.population_residual() - before).abs() < 1e-5);
        assert!((lost_army.food_residual() - h.food_residual()).abs() < 1e-5);
        assert!(lost_army
            .economy_residuals()
            .iter()
            .zip(h.economy_residuals())
            .all(|(a, b)| (a - b).abs() < 1e-5));
        let returning = raid.members.as_ref().unwrap()[0];
        h.people[returning as usize].born = h.month as i32 - 720;
        let elders = h.sites[route.from as usize].demography.ages[2];
        h.return_military_people(&raid);
        h.society.as_mut().unwrap().raids.clear();
        assert!(h.military.duties.is_empty());
        assert_eq!(
            h.person_presence(returning).1,
            Presence::Resident(route.from)
        );
        assert_eq!(h.sites[route.from as usize].demography.ages[2], elders + 1.);
        assert!((h.population_residual() - before).abs() < 1e-5);
        h.military.validate(h).unwrap();
    }
}
