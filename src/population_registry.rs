//! Read-only reconciliation between sparse identities and authoritative population stocks.
//! Overhang is evidence of incompatible representations, never permission to delete people.
use crate::{civilization::History, participation::Presence};
use serde::{Deserialize, Serialize};

pub fn age_band(month: u32, born: i32) -> Option<usize> {
    match i64::from(month) - i64::from(born) {
        ..=-1 => None,
        0..=179 => Some(0),
        180..=719 => Some(1),
        _ => Some(2),
    }
}
/// IDs are creation order, not succession order. Reusing residents permits a
/// predecessor with a larger ID; references must instead form an acyclic graph.
pub(crate) fn valid_succession_links(people: &[crate::civilization::Person]) -> bool {
    let mut state = vec![0u8; people.len()];
    let mut path = Vec::new();
    for start in 0..people.len() {
        if state[start] != 0 {
            continue;
        }
        path.clear();
        let mut next = Some(start as u32);
        while let Some(id) = next {
            let i = id as usize;
            if i >= people.len() || state[i] == 1 {
                return false;
            }
            if state[i] == 2 {
                break;
            }
            state[i] = 1;
            path.push(i);
            next = people[i].predecessor;
        }
        for &i in &path {
            state[i] = 2;
        }
    }
    true
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteReconciliation {
    pub site: u32,
    pub cohorts: [f64; 3],
    pub known: [u32; 3],
    /// Whole identities that can still be assigned without exceeding this age stock.
    pub unrepresented_slots: [u32; 3],
    /// Named people beyond the fractional stock; not an extra population inventory.
    pub overhang: [f64; 3],
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PopulationReconciliation {
    pub month: u32,
    pub sites: Vec<SiteReconciliation>,
    pub expedition: u32,
    pub military: u32,
    pub relocating: u32,
    pub unresolved: Vec<u32>,
    pub dead: u32,
    pub future_births: Vec<u32>,
}
impl History {
    /// Stable local succession choices, observed before the society is temporarily taken
    /// out of History. Headship is an ownership role, not a newly invented family tie.
    pub(crate) fn resident_successors(&self) -> std::collections::BTreeMap<u32, Vec<u32>> {
        let Some(society) = &self.society else {
            return Default::default();
        };
        let occupied: std::collections::BTreeSet<_> = society
            .households
            .iter()
            .map(|f| f.head)
            .chain(self.civilizations.iter().map(|c| c.leader))
            .collect();
        let parents: std::collections::BTreeMap<_, _> = self
            .politics
            .as_ref()
            .into_iter()
            .flat_map(|p| &p.kin)
            .map(|k| (k.person, k.parents))
            .collect();
        let mut residents = vec![Vec::new(); self.sites.len()];
        for person in &self.people {
            if occupied.contains(&person.id) || i64::from(self.month) - i64::from(person.born) < 216
            {
                continue;
            }
            let (home, presence) = self.person_presence(person.id);
            if let Presence::Resident(site) = presence {
                residents[site as usize].push((person.id, home));
            }
        }
        society
            .households
            .iter()
            .map(|account| {
                let mut candidates: Vec<_> = residents[account.site as usize]
                    .iter()
                    .map(|&(id, home)| {
                        let child = parents
                            .get(&id)
                            .is_some_and(|p| p.contains(&Some(account.head)));
                        let rank = if child {
                            0
                        } else if home == Some(account.id) {
                            1
                        } else {
                            2
                        };
                        (rank, self.people[id as usize].born, id)
                    })
                    .collect();
                candidates.sort_unstable();
                (
                    account.id,
                    candidates.into_iter().map(|(_, _, id)| id).collect(),
                )
            })
            .collect()
    }
    /// Complete mutually exclusive classification of every existing identity at this boundary.
    /// This does not allocate names for unrepresented residents or change demographic stocks.
    pub fn population_reconciliation(&self) -> PopulationReconciliation {
        let mut result = PopulationReconciliation {
            month: self.month,
            sites: self
                .sites
                .iter()
                .map(|s| SiteReconciliation {
                    site: s.id,
                    cohorts: std::array::from_fn(|i| s.demography.ages[i] as f64),
                    known: [0; 3],
                    unrepresented_slots: [0; 3],
                    overhang: [0.; 3],
                })
                .collect(),
            ..Default::default()
        };
        for person in &self.people {
            match self.person_presence(person.id).1 {
                Presence::Dead => result.dead += 1,
                Presence::Expedition(_) => result.expedition += 1,
                Presence::Military(_) => result.military += 1,
                Presence::Traveling(_) => result.relocating += 1,
                Presence::Unknown => result.unresolved.push(person.id),
                Presence::Resident(site) => {
                    if let Some(band) = age_band(self.month, person.born) {
                        result.sites[site as usize].known[band] += 1;
                    } else {
                        result.future_births.push(person.id);
                    }
                }
            }
        }
        for site in &mut result.sites {
            for i in 0..3 {
                site.unrepresented_slots[i] =
                    (site.cohorts[i].floor().max(0.) as u32).saturating_sub(site.known[i]);
                site.overhang[i] = (site.known[i] as f64 - site.cohorts[i]).max(0.);
            }
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn succession_order_is_not_identity_creation_order() {
        let mut people: Vec<_> = (0..3)
            .map(|id| crate::civilization::Person {
                id,
                name: format!("Person {id}"),
                civilization: 0,
                born: -300,
                died: None,
                predecessor: None,
            })
            .collect();
        people[0].predecessor = Some(2);
        people[2].predecessor = Some(1);
        assert!(valid_succession_links(&people));
        people[1].predecessor = Some(0);
        assert!(!valid_succession_links(&people));
        people[1].predecessor = Some(3);
        assert!(!valid_succession_links(&people));
        people[1].predecessor = Some(1);
        assert!(!valid_succession_links(&people));
    }
    #[test]
    fn cohort_age_boundaries_do_not_overflow() {
        assert_eq!(age_band(0, 1), None);
        assert_eq!(age_band(0, 0), Some(0));
        assert_eq!(age_band(179, 0), Some(0));
        assert_eq!(age_band(180, 0), Some(1));
        assert_eq!(age_band(719, 0), Some(1));
        assert_eq!(age_band(720, 0), Some(2));
        assert_eq!(age_band(u32::MAX, i32::MIN), Some(2));
    }
}

/// Credits assign identities to deaths already debited by the GPU. No extra mortality.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NamedDemography {
    pub month: Option<u32>,
    pub remainder: Vec<f64>,
    pub assigned: u64,
}
fn mortality_weights(h: &History, site: usize) -> [f64; 3] {
    let d = &h.sites[site].demography;
    std::array::from_fn(|i| {
        let hunger = if d.ration_need[i] > 0. {
            (1. - d.ration_eaten[i] / d.ration_need[i]).clamp(0., 1.)
        } else {
            0.
        };
        [0.0005, 0.0006, 0.003][i]
            + hunger as f64 * [0.06, 0.025, 0.05][i]
            + d.health[0].clamp(0., 0.5) as f64 * 0.01
    })
}
impl History {
    /// Apply only this dispatch's demographic losses, never accumulated archive-era deaths.
    pub(crate) fn assign_demographic_deaths(&mut self, before: &[f32]) {
        let Some(mut state) = self.named_demography.take() else {
            return;
        };
        if state.month == Some(self.month) {
            self.named_demography = Some(state);
            return;
        }
        state.month = Some(self.month);
        state.remainder.resize(self.sites.len(), 0.);
        let report = self.population_reconciliation();
        let mut candidates = vec![Vec::new(); self.sites.len()];
        for person in &self.people {
            if let Presence::Resident(site) = self.person_presence(person.id).1 {
                if let Some(age) = age_band(self.month, person.born) {
                    let weight = mortality_weights(self, site as usize)[age];
                    let draw =
                        crate::expeditions::random(self.seed, person.id, self.month, 0x4d4f5254)
                            .max(1e-7) as f64;
                    candidates[site as usize].push((person.id, -draw.ln() / weight));
                }
            }
        }
        for (i, row) in report.sites.iter().enumerate() {
            let fresh = (self.sites[i].stocks.people[1] - before[i]).max(0.) as f64;
            let weights = mortality_weights(self, i);
            let known: f64 = (0..3).map(|a| row.known[a] as f64 * weights[a]).sum();
            let total: f64 = (0..3).map(|a| row.cohorts[a] * weights[a]).sum();
            let coverage = if total > 0. {
                (known / total).clamp(0., 1.)
            } else {
                0.
            };
            let target = state.remainder[i] + fresh * coverage;
            // Fractional carry only: never save a backlog of hypothetical named deaths.
            state.remainder[i] = target.fract();
            let count = (target.floor() as usize)
                .min(candidates[i].len())
                .min(self.sites[i].demography.health[2].max(0.).floor() as usize);
            candidates[i].sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
            let ids: Vec<_> = candidates[i]
                .iter()
                .take(count)
                .map(|(id, _)| *id)
                .collect();
            if ids.is_empty() {
                continue;
            }
            for &id in &ids {
                self.people[id as usize].died = Some(self.month);
            }
            self.sites[i].demography.health[2] -= count as f32;
            state.assigned += count as u64;
            self.event("demographic_deaths",Some(i as u32),None,format!("{} known residents died among this month's recorded cohort losses ({fresh:.3}); identity coverage {:.3}",count,coverage));
            self.events
                .last_mut()
                .unwrap()
                .subjects
                .extend(ids.into_iter().map(|id| ("person".into(), id)));
        }
        self.named_demography = Some(state);
    }
}
impl History {
    /// Toggle the named-identity adapter at a completed work boundary. Cohorts stay authoritative.
    pub fn set_named_demography(&mut self, enabled: bool) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.participation
                .as_ref()
                .is_none_or(|p| p.commitments.iter().all(|c| c.settled)),
            "change named demography at a completed work boundary"
        );
        if enabled {
            self.named_demography.get_or_insert_with(Default::default);
        } else {
            self.named_demography = None;
        }
        Ok(())
    }
}
impl NamedDemography {
    pub fn validate(&self, h: &History) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.month.is_none_or(|m| m <= h.month)
                && self.remainder.len() <= h.sites.len()
                && self
                    .remainder
                    .iter()
                    .all(|v| v.is_finite() && (0. ..1.).contains(v)),
            "invalid named-demography clock or fractional credit"
        );
        Ok(())
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
        politics::{Kinship, Marriage},
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
    fn credits_assign_existing_deaths_once_without_killing_travelers() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 1;
        let report = h.population_reconciliation();
        for row in &report.sites {
            h.sites[row.site as usize].demography.ages[..3]
                .copy_from_slice(&row.known.map(|v| v as f32));
        }
        // All known residents under seventy are eligible; no retroactive legacy credit use.
        for p in &mut h.people {
            p.born = -300;
        }
        let report = h.population_reconciliation();
        for row in &report.sites {
            h.sites[row.site as usize].demography.ages[..3]
                .copy_from_slice(&row.known.map(|v| v as f32));
        }
        let before: Vec<_> = h.sites.iter().map(|s| s.stocks.people[1]).collect();
        for s in &mut h.sites {
            s.demography.health[2] = 100.;
        }
        h.sites[0].stocks.people[1] += 0.5;
        h.assign_demographic_deaths(&before);
        assert_eq!(h.named_demography.as_ref().unwrap().assigned, 0);
        assert_eq!(h.named_demography.as_ref().unwrap().remainder[0], 0.5);
        h.month += 1;
        let before: Vec<_> = h.sites.iter().map(|s| s.stocks.people[1]).collect();
        h.sites[0].stocks.people[1] += 0.5;
        let stocks: Vec<_> = h
            .sites
            .iter()
            .map(|s| serde_json::to_value(s.stocks).unwrap())
            .collect();
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        h.assign_demographic_deaths(&before);
        resumed.assign_demographic_deaths(&before);
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        assert_eq!(h.named_demography.as_ref().unwrap().assigned, 1);
        assert_eq!(h.sites[0].demography.health[2], 99.);
        assert!(h.sites[1..].iter().all(|s| s.demography.health[2] == 100.));
        assert_eq!(
            stocks,
            h.sites
                .iter()
                .map(|s| serde_json::to_value(s.stocks).unwrap())
                .collect::<Vec<_>>()
        );
        h.assign_demographic_deaths(&before);
        assert_eq!(h.named_demography.as_ref().unwrap().assigned, 1);
        let dead = h
            .people
            .iter()
            .find(|p| p.died == Some(h.month))
            .unwrap()
            .id;
        let account = h.person_presence(dead).0.unwrap();
        assert!(
            !h.household_available_for_relocation(account),
            "a death before succession must not strand an ownerless account in transit"
        );
        // Every identity has exactly one reported presence. An unresolved person is not a death.
        let r = h.population_reconciliation();
        assert_eq!(
            r.sites.iter().flat_map(|s| s.known).sum::<u32>()
                + r.dead
                + r.expedition
                + r.military
                + r.relocating
                + r.unresolved.len() as u32
                + r.future_births.len() as u32,
            h.people.len() as u32
        );
        // Moving all surviving residents of site 0 onto service excludes them from local losses.
        let ids: Vec<_> = h
            .people
            .iter()
            .filter(|p| h.person_presence(p.id).1 == Presence::Resident(0))
            .map(|p| p.id)
            .collect();
        for id in &ids {
            h.person_duties.insert(
                *id,
                crate::participation::TravelDuty {
                    voyage: 0,
                    origin: 0,
                    household: h.person_presence(*id).0,
                },
            );
        }
        h.month += 1;
        let before: Vec<_> = h.sites.iter().map(|s| s.stocks.people[1]).collect();
        h.sites[0].stocks.people[1] += 100.;
        h.assign_demographic_deaths(&before);
        assert!(ids.iter().all(|id| h.people[*id as usize].died.is_none()));
        assert_eq!(h.named_demography.as_ref().unwrap().assigned, 1);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn succession_reuses_present_people_and_reports_exhausted_slots() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 1;
        let accounts: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|a| a.site == 0)
            .map(|a| (a.id, a.head))
            .collect();
        let (account, old) = accounts[0];
        // An unrelated member and a younger recorded child are both real adults.
        let add = |h: &mut History, born, parents| {
            let id = h.people.len() as u32;
            let mut person = h.people[old as usize].clone();
            person.id = id;
            person.born = born;
            person.died = None;
            h.people.push(person);
            h.politics.as_mut().unwrap().kin.push(Kinship {
                person: id,
                household: account,
                parents,
            });
            id
        };
        let member = add(h, -400, [None; 2]);
        let child = add(h, -250, [Some(old), None]);
        h.people[old as usize].died = Some(1);
        let baseline = h.clone();
        let count = h.people.len();
        let stocks = serde_json::to_value(h.sites[0].stocks).unwrap();
        h.social_month();
        assert_eq!(
            h.society.as_ref().unwrap().households[account as usize].head,
            child
        );
        assert_eq!(h.people.len(), count);
        assert_eq!(stocks, serde_json::to_value(h.sites[0].stocks).unwrap());
        assert_eq!(
            h.politics
                .as_ref()
                .unwrap()
                .kin
                .iter()
                .find(|k| k.person == child)
                .unwrap()
                .parents,
            [Some(old), None]
        );
        // A child at sea is not eligible; the existing unrelated member succeeds.
        let mut away = baseline.clone();
        away.person_duties.insert(
            child,
            crate::participation::TravelDuty {
                voyage: 0,
                origin: 0,
                household: Some(account),
            },
        );
        away.social_month();
        assert_eq!(
            away.society.as_ref().unwrap().households[account as usize].head,
            member
        );
        assert_eq!(away.people.len(), count);
        // A ruler's successor must satisfy the existing civilization membership
        // invariant; appointment must not rewrite a migrant's cultural identity.
        let mut foreign = baseline.clone();
        foreign.people[member as usize].civilization = 1;
        foreign.person_duties.insert(
            child,
            crate::participation::TravelDuty {
                voyage: 0,
                origin: 0,
                household: Some(account),
            },
        );
        foreign.social_month();
        let ruler = foreign.civilizations[0].leader;
        assert_ne!(ruler, member);
        assert_eq!(foreign.people[member as usize].civilization, 1);
        assert_eq!(foreign.people[ruler as usize].civilization, 0);
        // An existing person can precede the deceased head in identity creation order.
        let mut older = baseline.clone();
        older.society.as_mut().unwrap().households[account as usize].head = child;
        older.people[child as usize].died = Some(1);
        older.people[old as usize].died = None;
        older.social_month();
        assert_eq!(
            older.society.as_ref().unwrap().households[account as usize].head,
            member
        );
        assert_eq!(older.people[member as usize].predecessor, Some(child));
        assert!(valid_succession_links(&older.people));
        // No duplicate successor when two ownership claims need heads in one month.
        let mut simultaneous = baseline.clone();
        simultaneous.people[accounts[1].1 as usize].died = Some(1);
        simultaneous.social_month();
        assert_eq!(simultaneous.people.len(), count);
        let heads: Vec<_> = simultaneous
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .map(|a| a.head)
            .collect();
        let unique: std::collections::BTreeSet<_> = heads.iter().collect();
        assert_eq!(unique.len(), heads.len());
        // No known candidates: use an anonymous elder rather than overidentify adults.
        let mut empty = baseline;
        empty.people[member as usize].died = Some(1);
        empty.people[child as usize].died = Some(1);
        empty.sites[0].demography.ages[1] = 0.;
        empty.sites[0].demography.ages[2] = 1.;
        let mut elder = empty.clone();
        elder.social_month();
        let successor = elder.society.as_ref().unwrap().households[account as usize].head;
        assert_eq!(age_band(1, elder.people[successor as usize].born), Some(2));
        assert!(!elder
            .events
            .iter()
            .any(|e| e.kind == "succession_identity_overhang"));
        // The legacy living-head requirement is explicit when even that slot is gone.
        empty.sites[0].demography.ages[2] = 0.;
        let mut resumed: History =
            serde_json::from_slice(&serde_json::to_vec(&empty).unwrap()).unwrap();
        empty.social_month();
        resumed.social_month();
        assert_eq!(
            serde_json::to_value(&empty).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert_eq!(
            empty
                .events
                .iter()
                .filter(|e| e.kind == "succession_identity_overhang")
                .count(),
            1
        );
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn birth_credits_cannot_overidentify_the_child_cohort() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 36;
        let heads: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|a| a.site == 0)
            .map(|a| (a.head, a.id))
            .collect();
        let a = heads[0].0;
        let b = heads[1].0;
        h.people[a as usize].born = -300;
        h.people[b as usize].born = -300;
        let id = h.people.len() as u32;
        let mut child = h.people[a as usize].clone();
        child.id = id;
        child.born = 0;
        h.people.push(child);
        let p = h.politics.as_mut().unwrap();
        p.kin.push(Kinship {
            person: id,
            household: heads[0].1,
            parents: [Some(a), Some(b)],
        });
        p.marriages.push(Marriage {
            partners: [a, b],
            started: 0,
            ended: None,
            children: 1,
            last_birth: 0,
        });
        p.birth_credit[0] = 500.;
        h.sites[0].demography.ages[0] = 1.;
        let count = h.people.len();
        h.genealogy_month();
        assert_eq!(h.people.len(), count);
        assert_eq!(
            h.population_reconciliation().sites[0].unrepresented_slots[0],
            0
        );
        h.sites[0].demography.ages[0] = 2.;
        let stocks = serde_json::to_value(h.sites[0].stocks).unwrap();
        h.genealogy_month();
        assert_eq!(h.people.len(), count + 1);
        assert_eq!(h.population_reconciliation().sites[0].known[0], 2);
        assert_eq!(stocks, serde_json::to_value(h.sites[0].stocks).unwrap());
        let mut legacy = serde_json::to_value(&*h).unwrap();
        legacy.as_object_mut().unwrap().remove("named_demography");
        let legacy: History = serde_json::from_value(legacy).unwrap();
        assert!(legacy.named_demography.is_none());
    }
}
