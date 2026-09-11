//! Individual participation pilot. Population remains authoritative in settlement cohorts.
//! Named residents consume the existing service allowance; they never add workers to it.
use crate::civilization::History;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Culture,
    Research,
    Workshop,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Presence {
    Expedition(u32),
    Military(u32),
    Resident(u32),
    Traveling(u32),
    Dead,
    Unknown,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelDuty {
    pub voyage: u32,
    pub origin: u32,
    pub household: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resident {
    pub person: u32,
    /// Existing ownership household, not a reconstructed domestic family.
    pub household: Option<u32>,
    pub presence: Presence,
    #[serde(default)]
    pub care: f32,
    pub capacity: f32,
    pub committed: f32,
    pub completed: [f64; 2],
    #[serde(default)]
    pub workshop_completed: f64,
    /// Completed worker-months by the existing four recipe families. Older untyped
    /// experience remains in workshop_completed; no historical trade is invented.
    #[serde(default)]
    pub workshop_practice: [f64; 4],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commitment {
    pub month: u32,
    pub site: u32,
    pub activity: Activity,
    /// Each participant's share of the same grant, never additional site labor.
    pub people: Vec<(u32, f32)>,
    pub granted: f32,
    pub used: f32,
    pub settled: bool,
    pub cancellation: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Participation {
    pub month: Option<u32>,
    pub residents: BTreeMap<u32, Resident>,
    pub commitments: Vec<Commitment>,
}
impl Participation {
    pub fn available(&self, person: u32) -> f32 {
        self.residents
            .get(&person)
            .map_or(0., |p| (p.capacity - p.committed).max(0.))
    }
    /// Equal shares for a bounded team. All participants must be present; no silent replacement.
    pub fn reserve(
        &mut self,
        month: u32,
        site: u32,
        activity: Activity,
        ids: &[u32],
        wanted: f32,
    ) -> Option<u32> {
        if self.month != Some(month) || !wanted.is_finite() || wanted <= 0. {
            return None;
        }
        let mut ids = ids.to_vec();
        ids.sort_unstable();
        ids.dedup();
        if ids.is_empty()
            || ids.iter().any(|id| {
                self.residents
                    .get(id)
                    .is_none_or(|p| p.presence != Presence::Resident(site))
            })
        {
            return None;
        }
        let share = ids
            .iter()
            .map(|&id| self.available(id))
            .fold(wanted / ids.len() as f32, f32::min);
        if share <= 1e-6 {
            return None;
        }
        for id in &ids {
            self.residents.get_mut(id).unwrap().committed += share;
        }
        let id = self.commitments.len() as u32;
        self.commitments.push(Commitment {
            month,
            site,
            activity,
            granted: share * ids.len() as f32,
            people: ids.into_iter().map(|p| (p, share)).collect(),
            used: 0.,
            settled: false,
            cancellation: None,
        });
        Some(id)
    }
    pub fn settle(&mut self, id: u32, used: f32) -> Result<()> {
        self.settle_work(id, used, None)
    }
    pub(crate) fn settle_workshop(&mut self, id: u32, used: f32, family: u32) -> Result<()> {
        ensure!(
            family < 4
                && self
                    .commitments
                    .get(id as usize)
                    .is_some_and(|c| c.activity == Activity::Workshop),
            "invalid workshop learning assignment"
        );
        self.settle_work(id, used, Some(family as usize))
    }
    fn settle_work(&mut self, id: u32, used: f32, family: Option<usize>) -> Result<()> {
        let c = self
            .commitments
            .get_mut(id as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown personal commitment"))?;
        ensure!(
            !c.settled && used.is_finite() && used >= 0. && used <= c.granted + 1e-5,
            "invalid personal work settlement"
        );
        c.used = used.min(c.granted);
        c.settled = true;
        let category = match c.activity {
            Activity::Culture => 0,
            Activity::Research => 1,
            Activity::Workshop => 2,
        };
        for &(person, share) in &c.people {
            let resident = self.residents.get_mut(&person).unwrap();
            let contribution = (share * c.used / c.granted) as f64;
            if category == 2 {
                resident.workshop_completed += contribution;
                if let Some(family) = family {
                    resident.workshop_practice[family] += contribution;
                }
            } else {
                resident.completed[category] += contribution;
            }
        }
        // Reservations, including unproductive time, remain unavailable until the next month.
        Ok(())
    }
    pub fn validate(&self, h: &History) -> Result<()> {
        ensure!(
            self.month.is_none_or(|m| m <= h.month),
            "future participation month"
        );
        for (&id, p) in &self.residents {
            ensure!(
                id == p.person
                    && (id as usize) < h.people.len()
                    && p.household.is_none_or(|id| h
                        .society
                        .as_ref()
                        .is_some_and(|s| (id as usize) < s.households.len()))
                    && [p.capacity, p.committed, p.care]
                        .iter()
                        .all(|v| v.is_finite() && *v >= 0.)
                    && p.capacity + p.care <= 0.80001
                    && p.committed <= p.capacity + 1e-5
                    && p.completed.iter().all(|v| v.is_finite() && *v >= 0.)
                    && p.workshop_completed.is_finite()
                    && p.workshop_completed >= 0.
                    && p.workshop_practice
                        .iter()
                        .all(|v| v.is_finite() && *v >= 0.)
                    && p.workshop_practice.iter().sum::<f64>() <= p.workshop_completed + 1e-6,
                "invalid resident participation"
            );
            let committed: f32 = self
                .commitments
                .iter()
                .flat_map(|c| &c.people)
                .filter(|(who, _)| *who == id)
                .map(|(_, v)| v)
                .sum();
            ensure!(
                (committed - p.committed).abs() < 1e-4,
                "personal commitment ledger mismatch"
            );
        }
        for c in &self.commitments {
            ensure!(
                Some(c.month) == self.month
                    && (c.site as usize) < h.sites.len()
                    && c.granted.is_finite()
                    && c.granted > 0.
                    && c.used.is_finite()
                    && c.used >= 0.
                    && c.used <= c.granted + 1e-5
                    && (c.people.iter().map(|(_, v)| v).sum::<f32>() - c.granted).abs() < 1e-4
                    && c.people.windows(2).all(|p| p[0].0 < p[1].0)
                    && c.people
                        .iter()
                        .all(|(id, v)| self.residents.contains_key(id) && v.is_finite() && *v > 0.),
                "invalid personal commitment"
            );
        }
        Ok(())
    }
}
impl History {
    /// Sparse known membership only: Unknown is preferable to inventing a residence.
    pub fn person_presence(&self, person: u32) -> (Option<u32>, Presence) {
        let Some(p) = self.people.get(person as usize) else {
            return (None, Presence::Unknown);
        };
        let household = self
            .society
            .as_ref()
            .and_then(|s| {
                s.households
                    .iter()
                    .find(|hh| hh.head == person)
                    .map(|hh| hh.id)
            })
            .or_else(|| {
                self.politics.as_ref().and_then(|p| {
                    p.kin
                        .iter()
                        .find(|k| k.person == person)
                        .map(|k| k.household)
                })
            });
        if p.died.is_some() {
            return (household, Presence::Dead);
        }
        if let Some(duty) = self.person_duties.get(&person) {
            return (duty.household, Presence::Expedition(duty.voyage));
        }
        if let Some(duty) = self.military.duties.get(&person) {
            return (duty.household, Presence::Military(duty.army));
        }
        if let Some((society, hh)) = self.society.as_ref().zip(household) {
            if let Some(journey) = society
                .relocation
                .journeys
                .iter()
                .find(|j| j.household == hh)
            {
                if journey
                    .roster
                    .as_ref()
                    .is_none_or(|r| r.passengers.iter().any(|p| p.person == person))
                {
                    return (household, Presence::Traveling(hh));
                }
            } else if society.relocation.lost_households.contains(&hh) {
                return (household, Presence::Unknown);
            }
            if let Some(hh) = society.households.get(hh as usize) {
                return (
                    household,
                    if self.sites[hh.site as usize].abandoned {
                        Presence::Unknown
                    } else {
                        Presence::Resident(hh.site)
                    },
                );
            }
        }
        let site = self
            .sites
            .iter()
            .find(|s| !s.abandoned && self.civilizations[s.civilization as usize].leader == person);
        (
            household,
            site.map_or(Presence::Unknown, |s| Presence::Resident(s.id)),
        )
    }
    pub(crate) fn open_participation(&mut self) {
        self.reserve_domestic_care();
        let Some(mut state) = self.participation.take() else {
            return;
        };
        if state.month == Some(self.month) {
            self.participation = Some(state);
            return;
        }
        state.month = Some(self.month);
        state.commitments.clear();
        for p in &self.people {
            let (household, presence) = self.person_presence(p.id);
            let capacity = match presence {
                Presence::Resident(site) if (180..720).contains(&(self.month as i32 - p.born)) => {
                    0.8 * (1. - 0.5 * self.sites[site as usize].demography.health[0].clamp(0., 0.5))
                }
                _ => 0.,
            };
            let care = self.domestic_care_for(p.id).min(capacity);
            let capacity = (capacity - care).max(0.);
            let entry = state.residents.entry(p.id).or_insert(Resident {
                person: p.id,
                care,
                household,
                presence,
                capacity,
                committed: 0.,
                completed: [0.; 2],
                workshop_completed: 0.,
                workshop_practice: [0.; 4],
            });
            entry.care = care;
            entry.household = household;
            entry.presence = presence;
            entry.capacity = capacity;
            entry.committed = 0.;
        }
        self.participation = Some(state);
    }
    pub(crate) fn personal_grant_live(&self, id: Option<u32>) -> f32 {
        let Some(state) = &self.participation else {
            return f32::MAX;
        };
        let Some(c) = id.and_then(|id| state.commitments.get(id as usize)) else {
            return 0.;
        };
        if c.month != self.month
            || c.settled
            || c.people
                .iter()
                .any(|(id, _)| self.person_presence(*id).1 != Presence::Resident(c.site))
        {
            0.
        } else {
            c.granted
        }
    }
    /// Change modes only at a completed personal-work boundary. This does not convert population.
    pub fn set_individual_participation(&mut self, enabled: bool) -> Result<()> {
        ensure!(
            self.participation
                .as_ref()
                .is_none_or(|p| p.commitments.iter().all(|c| c.settled)),
            "personal work remains unsettled"
        );
        if enabled && self.participation.is_none() {
            self.participation = Some(Default::default());
        }
        if !enabled {
            ensure!(
                !self
                    .resolution
                    .as_ref()
                    .is_some_and(|r| r.workshop_individual),
                "disable workshop refinement before personal participation"
            );
            self.participation = None;
            if let Some(c) = &mut self.culture {
                for p in &mut c.work_plans {
                    p.commitment = None;
                }
            }
            if let Some(d) = self
                .expeditions
                .as_mut()
                .and_then(|x| x.discoveries.as_mut())
            {
                for p in d.workshops.iter_mut().filter_map(|w| w.work_plan.as_mut()) {
                    p.commitment = None;
                }
            }
        }
        Ok(())
    }
    pub fn participation_report(&self) -> serde_json::Value {
        serde_json::json!({"mode":if self.participation.is_some(){"named participation; aggregate demography"}else{"legacy"},"state":self.participation,"domestic":self.domestic})
    }
}

impl History {
    pub(crate) fn settle_participation(&mut self) -> Result<()> {
        let Some(state) = &mut self.participation else {
            return Ok(());
        };
        if let Some(c) = &self.culture {
            for p in &c.work_plans {
                if let Some(id) = p.commitment.filter(|_| p.month == self.month) {
                    if !state.commitments[id as usize].settled {
                        let used = p.completed;
                        state.commitments[id as usize].cancellation = p.cancellation.clone();
                        state.settle(id, used)?;
                    }
                }
            }
        }
        if let Some(d) = self
            .expeditions
            .as_ref()
            .and_then(|x| x.discoveries.as_ref())
        {
            for p in d.workshops.iter().filter_map(|w| w.work_plan.as_ref()) {
                if let Some(id) = p.commitment.filter(|_| p.receipt.month == self.month) {
                    if !state.commitments[id as usize].settled {
                        state.commitments[id as usize].cancellation = p.cancellation.clone();
                        state.settle(id, p.receipt.used as f32)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// A bounded identification step over existing cohort residents, not a population import.
pub(crate) struct Recruitment {
    pub people: Vec<u32>,
    pub identified: usize,
    pub first_identified: u32,
}
impl History {
    pub(crate) fn recruit_service_people(
        &mut self,
        origin: u32,
        maximum: usize,
        minimum: usize,
    ) -> Result<Recruitment> {
        ensure!(
            minimum > 0 && maximum >= minimum && (origin as usize) < self.sites.len(),
            "invalid service recruitment request"
        );
        self.sync_domestic();
        let maximum = maximum.min(
            self.sites[origin as usize].demography.ages[1]
                .floor()
                .max(0.) as usize,
        );
        // Choose real available adults before spending anything. Sparse historical people
        // are a subset of the cohort population; identification never adds population.
        let mut candidates: Vec<_> = self
            .people
            .iter()
            .filter(|person| {
                (180..660).contains(&(self.month as i32 - person.born))
                    && self.person_presence(person.id).1
                        == crate::participation::Presence::Resident(origin)
                    && !self.civilizations.iter().any(|c| c.leader == person.id)
                    && self.participation.as_ref().is_none_or(|p| {
                        p.month != Some(self.month)
                            || p.residents
                                .get(&person.id)
                                .is_none_or(|r| r.committed <= 1e-6)
                    })
            })
            .map(|p| p.id)
            .collect();
        candidates.sort_by_key(|&person| {
            crate::expeditions::random(self.seed, person, self.month, 211).to_bits()
        });
        let mut chosen = Vec::new();
        for person in candidates {
            if chosen.len() == maximum {
                break;
            }
            if self.domestic_departure_allowed(person, &chosen) {
                chosen.push(person);
            }
        }
        let mut candidates = chosen;
        let mut slots = crate::population_registry::ResidentSlots::new(self);
        let unnamed_adults =
            slots.available(origin, 1, self.sites[origin as usize].demography.ages[1]) as usize;
        let identify = (maximum - candidates.len()).min(unnamed_adults).min(
            self.politics
                .as_ref()
                .map_or(0, |p| 50000usize.saturating_sub(p.kin.len())),
        );
        ensure!(
            candidates.len() + identify >= minimum,
            "insufficient uncommitted resident adults for service"
        );
        let homes: Vec<_> = self
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|hh| {
                hh.site == origin && !self.society.as_ref().unwrap().relocation.away(hh.id)
            })
            .map(|hh| hh.id)
            .collect();
        ensure!(
            identify == 0 || !homes.is_empty(),
            "no resident ownership account for unnamed crew"
        );
        ensure!(
            slots.reserve(
                origin,
                1,
                self.sites[origin as usize].demography.ages[1],
                identify as u32
            ),
            "resident admission changed during service recruitment"
        );
        let first_identified = self.people.len() as u32;
        for slot in 0..identify {
            let person = self.people.len() as u32;
            let civilization = self.sites[origin as usize].civilization;
            let name = self.civilizations[civilization as usize]
                .naming(self.seed)
                .person_with(
                    "person",
                    person,
                    &crate::naming::PersonalContext::local(
                        &self.sites[origin as usize],
                        self.culture.as_ref(),
                    ),
                );
            self.people.push(crate::civilization::Person {
                id: person,
                name,
                civilization,
                born: self.month as i32
                    - 300
                    - (crate::expeditions::random(self.seed, person, self.month, 212) * 180.)
                        as i32,
                died: None,
                predecessor: None,
            });
            self.politics
                .as_mut()
                .unwrap()
                .kin
                .push(crate::politics::Kinship {
                    person,
                    household: homes[slot % homes.len()],
                    parents: [None; 2],
                });
            candidates.push(person);
        }

        Ok(Recruitment {
            people: candidates,
            identified: identify,
            first_identified,
        })
    }
    pub(crate) fn person_on_service(&self, person: u32) -> bool {
        self.person_duties.contains_key(&person) || self.military.duties.contains_key(&person)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> Participation {
        Participation {
            month: Some(12),
            residents: (0..3)
                .map(|id| {
                    (
                        id,
                        Resident {
                            person: id,
                            household: None,
                            presence: Presence::Resident(0),
                            care: 0.,
                            capacity: 0.8,
                            committed: 0.,
                            completed: [0.; 2],
                            workshop_completed: 0.,
                            workshop_practice: [0.; 4],
                        },
                    )
                })
                .collect(),
            commitments: vec![],
        }
    }
    #[test]
    fn shared_participant_cannot_supply_two_full_jobs() {
        let mut p = state();
        let a = p.reserve(12, 0, Activity::Research, &[0], 0.7).unwrap();
        let b = p.reserve(12, 0, Activity::Culture, &[0], 0.5).unwrap();
        assert!((p.commitments[b as usize].granted - 0.1).abs() < 1e-6);
        p.settle(a, 0.4).unwrap();
        assert!(
            p.reserve(12, 0, Activity::Culture, &[0], 0.1).is_none(),
            "late unused time cannot be spent twice"
        );
        assert!(p.settle(a, 0.4).is_err());
        assert!((p.residents[&0].completed[1] - 0.4).abs() < 1e-6);
    }
    #[test]
    fn team_work_is_partitioned_and_deduplicated() {
        let mut p = state();
        let id = p.reserve(12, 0, Activity::Culture, &[1, 0, 1], 1.).unwrap();
        assert_eq!(p.commitments[id as usize].people, vec![(0, 0.5), (1, 0.5)]);
        p.settle(id, 0.6).unwrap();
        assert!((p.residents.values().map(|p| p.completed[0]).sum::<f64>() - 0.6).abs() < 1e-6);
        assert!(p.reserve(11, 0, Activity::Culture, &[2], 0.1).is_none());
        assert!(p.reserve(12, 1, Activity::Culture, &[2], 0.1).is_none());
        assert!(p
            .reserve(12, 0, Activity::Culture, &[2], f32::NAN)
            .is_none());
    }
    #[test]
    fn archived_commitments_preserve_limits_and_experience() {
        let mut p = state();
        p.reserve(12, 0, Activity::Research, &[0, 1], 1.).unwrap();
        let mut q: Participation =
            serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        let a = p.reserve(12, 0, Activity::Culture, &[0], 0.5);
        let b = q.reserve(12, 0, Activity::Culture, &[0], 0.5);
        assert_eq!(a, b);
        assert_eq!(
            serde_json::to_value(p).unwrap(),
            serde_json::to_value(q).unwrap()
        );
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn personal_absence_and_shared_work_change_actual_research() {
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
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        g.enable_shipping().unwrap();
        g.enable_expeditions().unwrap();
        g.enable_discoveries().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 12;
        h.open_participation();
        let ids: Vec<_> = h
            .participation
            .as_ref()
            .unwrap()
            .residents
            .values()
            .filter(|p| p.presence == Presence::Resident(0) && p.capacity > 0.)
            .map(|p| p.person)
            .collect();
        assert!(ids.len() >= 2);
        let person = ids[0];
        let id = h
            .participation
            .as_mut()
            .unwrap()
            .reserve(h.month, 0, Activity::Research, &[person], 0.6)
            .unwrap();
        assert!(h.personal_grant_live(Some(id)) > 0.);
        h.people[person as usize].died = Some(12);
        assert_eq!(h.personal_grant_live(Some(id)), 0.);
        h.people[person as usize].died = None;
        let household = h.person_presence(person).0.unwrap();
        h.society.as_mut().unwrap().households[household as usize].site = 1;
        assert_eq!(h.personal_grant_live(Some(id)), 0.);
        h.society.as_mut().unwrap().households[household as usize].site = 0;
        // A real workshop with finite samples and materials, identical except available people.
        let mut d = h
            .expeditions
            .as_ref()
            .unwrap()
            .discoveries
            .as_ref()
            .unwrap()
            .clone();
        let sample = crate::discoveries::Workshop {
            work_plan: None,
            site: 0,
            enabled: true,
            samples: [1., 0.],
            studied: [0.; 2],
            processed: [0.; 2],
            curated: [0.; 2],
            learned: [None; 2],
            remedy: 0.,
            delivered: [1., 0.],
            causes: [None; 2],
            batches: [0; 2],
        };
        d.workshops = vec![sample];
        h.expeditions.as_mut().unwrap().discoveries = Some(d);
        h.sites[0].economy.goods[3] = 10.;
        h.sites[0].economy.goods[6] = 10.;
        h.month = 13;
        h.begin_service_reservations();
        let mut blocked = h.clone();
        for person in &ids {
            blocked.participation.as_mut().unwrap().reserve(
                13,
                0,
                Activity::Culture,
                &[*person],
                0.8,
            );
        }
        h.prepare_discoveries();
        blocked.prepare_discoveries();
        let run = |h: &mut History| {
            h.sites[0].economy.labor[3] = 10.;
            let mut d = h.expeditions.as_mut().unwrap().discoveries.take().unwrap();
            d.month(h);
            let used = d.workshops[0].work_plan.as_ref().unwrap().receipt.used;
            h.expeditions.as_mut().unwrap().discoveries = Some(d);
            used
        };
        assert!(run(h) > 0.);
        assert_eq!(run(&mut blocked), 0.);
        assert!(
            h.expeditions
                .as_ref()
                .unwrap()
                .discoveries
                .as_ref()
                .unwrap()
                .workshops[0]
                .samples[0]
                < 1.
        );
        assert_eq!(
            blocked
                .expeditions
                .as_ref()
                .unwrap()
                .discoveries
                .as_ref()
                .unwrap()
                .workshops[0]
                .samples[0],
            1.
        );
        h.settle_participation().unwrap();
        h.participation.as_ref().unwrap().validate(h).unwrap();
        assert!(h
            .participation
            .as_ref()
            .unwrap()
            .residents
            .values()
            .any(|p| p.completed[1] > 0.));
        // Participation must include known adult family members, not just owners.
        let member = h.people.len() as u32;
        let mut person_record = h.people[person as usize].clone();
        person_record.id = member;
        person_record.name = "Participation fixture member".into();
        person_record.born = h.month as i32 - 240;
        h.people.push(person_record);
        h.politics
            .as_mut()
            .unwrap()
            .kin
            .push(crate::politics::Kinship {
                person: member,
                household,
                parents: [None; 2],
            });
        h.sync_culture();
        assert!(!h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .any(|hh| hh.head == member));
        assert!(h
            .culture
            .as_ref()
            .unwrap()
            .site_people(h, 0)
            .contains(&member));
        h.people[member as usize].born = h.month as i32 - 720;
        assert!(!h
            .culture
            .as_ref()
            .unwrap()
            .site_people(h, 0)
            .contains(&member));
    }
}
