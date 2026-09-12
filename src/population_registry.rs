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
/// Pass-local admission ledger. Construct before temporarily taking society/politics;
/// discard after the pass. Stocks remain authoritative and are checked on each claim.
pub(crate) struct ResidentSlots {
    known: Vec<[u32; 3]>,
}
fn whole_slots(stock: f64, known: u32) -> u32 {
    if !stock.is_finite() || stock < 0. {
        return 0;
    }
    (stock.floor() as u32).saturating_sub(known)
}
impl ResidentSlots {
    pub(crate) fn new(history: &History) -> Self {
        Self {
            known: history
                .population_reconciliation()
                .sites
                .into_iter()
                .map(|s| s.known)
                .collect(),
        }
    }
    pub(crate) fn available(&self, site: u32, band: usize, stock: f32) -> u32 {
        self.known
            .get(site as usize)
            .and_then(|s| s.get(band))
            .map_or(0, |&known| whole_slots(stock as f64, known))
    }
    pub(crate) fn reserve(&mut self, site: u32, band: usize, stock: f32, count: u32) -> bool {
        if !stock.is_finite()
            || stock < 0.
            || site as usize >= self.known.len()
            || band >= 3
            || count > self.available(site, band, stock)
        {
            return false;
        }
        self.known[site as usize][band] += count;
        true
    }
    /// Record legacy representative creation/death without changing demographic stocks.
    pub(crate) fn observe(&mut self, site: u32, band: usize, added: bool) {
        let known = &mut self.known[site as usize][band];
        *known = if added {
            known.saturating_add(1)
        } else {
            known.saturating_sub(1)
        };
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
    /// Current explicit membership, derived from stable identities rather than headcount
    /// estimates. Ownership membership is not evidence of biological kinship.
    pub fn household_resident_roster(&self, household: u32) -> Vec<u32> {
        self.people
            .iter()
            .filter(|p| {
                let (home, presence) = self.person_presence(p.id);
                home == Some(household)
                    && matches!(presence, Presence::Resident(_))
                    && age_band(self.month, p.born).is_some()
            })
            .map(|p| p.id)
            .collect()
    }
    /// Explicit observation baseline: name all currently unrepresented whole residents.
    /// Fractional remainders and existing overhang remain visible; no past families,
    /// births or population are invented. Call at a completed monthly boundary.
    pub fn identify_resident_baseline(&mut self) -> anyhow::Result<usize> {
        anyhow::ensure!(
            self.participation
                .as_ref()
                .is_none_or(|p| p.commitments.iter().all(|c| c.settled)),
            "resident baseline requires settled participation"
        );
        let society = self
            .society
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("resident baseline requires ownership accounts"))?;
        let politics = self
            .politics
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("resident baseline requires membership records"))?;
        let audit = self.population_reconciliation();
        let mut planned = Vec::new();
        for row in &audit.sites {
            if self.sites[row.site as usize].abandoned {
                continue;
            }
            let homes: Vec<_> = society
                .households
                .iter()
                .filter(|h| h.site == row.site && !society.relocation.away(h.id))
                .map(|h| h.id)
                .collect();
            let needed: u64 = row.unrepresented_slots.iter().map(|&n| u64::from(n)).sum();
            anyhow::ensure!(
                needed == 0 || !homes.is_empty(),
                "resident site has no available ownership account"
            );
            anyhow::ensure!(
                planned.len() as u64 + needed + politics.kin.len() as u64 <= 50000,
                "resident baseline exceeds membership capacity"
            );
            let mut sizes: Vec<_> = homes
                .iter()
                .map(|&id| self.household_resident_roster(id).len())
                .collect();
            for band in 0..3 {
                for _ in 0..row.unrepresented_slots[band] {
                    let h = (0..homes.len())
                        .min_by_key(|&i| (sizes[i], homes[i]))
                        .unwrap();
                    sizes[h] += 1;
                    planned.push((row.site, band, homes[h]));
                }
            }
        }
        anyhow::ensure!(
            (self.month as u64) <= i32::MAX as u64,
            "resident baseline date exceeds identity calendar"
        );
        let added = planned.len();
        for (site, band, household) in planned {
            let id = self.people.len() as u32;
            let (start, span) = [(0, 180), (180, 540), (720, 240)][band];
            let age = start
                + (crate::expeditions::random(self.seed, id, self.month, 219) * span as f32) as i32;
            let town = &self.sites[site as usize];
            self.people.push(crate::civilization::Person {
                id,
                name: self.civilizations[town.civilization as usize]
                    .naming(self.seed)
                    .person_with(
                        "person",
                        id,
                        &crate::naming::PersonalContext::local(town, self.culture.as_ref()),
                    ),
                civilization: town.civilization,
                born: self.month as i32 - age,
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
                    parents: [None; 2],
                });
        }
        if added > 0 {
            self.event("resident_observation_baseline", None, None,
                format!("Identified {added} previously unnamed whole residents; ages estimated and ownership membership assigned; no reconstructed ancestry or additional population"));
        }
        Ok(added)
    }

    /// A polity can retain its last ruler's historical ID during an interregnum.
    pub fn living_civilization_leader(&self, civilization: u32) -> Option<u32> {
        let id = self.civilizations.get(civilization as usize)?.leader;
        self.people
            .get(id as usize)
            .filter(|p| p.died.is_none())
            .map(|_| id)
    }
    pub(crate) fn resolve_council_vacancies(&mut self) {
        for civ in 0..self.civilizations.len() {
            if self.living_civilization_leader(civ as u32).is_some() {
                continue;
            }
            let candidate = self
                .society
                .as_ref()
                .into_iter()
                .flat_map(|s| &s.households)
                .filter(|f| f.vacant_since.is_none())
                .filter_map(|f| {
                    let p = &self.people[f.head as usize];
                    (p.civilization == civ as u32
                        && i64::from(self.month) - i64::from(p.born) >= 216
                        && matches!(self.person_presence(p.id).1, Presence::Resident(_)))
                    .then_some((p.born, p.id, f.site))
                })
                .min();
            if let Some((_, id, site)) = candidate {
                let old = self.civilizations[civ].leader;
                self.civilizations[civ].leader = id;
                self.event("interim_leadership", Some(site), None,
                    format!("{} became council caretaker after {}; the deceased ruler's estate was not transferred", self.people[id as usize].name, self.people[old as usize].name));
                self.events
                    .last_mut()
                    .unwrap()
                    .subjects
                    .extend([("person".into(), id), ("person".into(), old)]);
            }
        }
    }
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
                site.unrepresented_slots[i] = whole_slots(site.cohorts[i], site.known[i]);
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
    fn admission_is_local_whole_bounded_and_atomic() {
        let mut slots = ResidentSlots {
            known: vec![[1, 2, 0], [0; 3]],
        };
        assert_eq!(slots.available(0, 1, 3.9), 1);
        assert!(!slots.reserve(0, 1, 3.9, 2));
        assert_eq!(slots.available(0, 1, 3.9), 1);
        assert!(slots.reserve(0, 1, 3.9, 1));
        assert!(!slots.reserve(0, 1, 3.9, 1));
        assert_eq!(slots.available(0, 1, 2.), 0);
        assert_eq!(slots.available(1, 1, 3.), 3);
        assert_eq!(slots.available(0, 2, 3.), 3);
        for stock in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.] {
            assert_eq!(slots.available(1, 0, stock), 0);
            assert!(!slots.reserve(1, 0, stock, 0));
        }
        assert!(!slots.reserve(2, 0, 10., 0));
        assert!(!slots.reserve(0, 3, 10., 0));
        slots.observe(0, 1, false);
        assert_eq!(slots.available(0, 1, 3.), 1);
        assert!(slots.reserve(0, 1, 3., 1));
        assert_eq!(whole_slots(0.999, 0), 0);
    }
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
    #[serde(default)]
    pub individual: bool,
    #[serde(default)]
    pub birth_remainder: Vec<f64>,
    #[serde(default)]
    pub defense_remainder: Vec<f64>,
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
        if state.individual || state.month == Some(self.month) {
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
            enabled || !self.agriculture_refinement_enabled(),
            "disable agriculture refinement before named demography"
        );
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
            !self.individual
                || (h.society.is_some() && h.politics.is_some() && self.month.is_some()),
            "individual demography requires membership and a completed baseline"
        );
        anyhow::ensure!(
            self.birth_remainder.len() <= h.sites.len()
                && self.defense_remainder.len() <= h.sites.len()
                && self
                    .defense_remainder
                    .iter()
                    .all(|v| v.is_finite() && (0. ..1.).contains(v))
                && self
                    .birth_remainder
                    .iter()
                    .all(|v| v.is_finite() && (0. ..1.).contains(v)),
            "invalid individual birth carry"
        );
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
    fn whole_resident_baseline_preserves_stocks_and_continuation() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        let stocks: Vec<_> = h
            .sites
            .iter()
            .map(|s| serde_json::to_value((s.stocks, s.demography.ages)).unwrap())
            .collect();
        let old = h.people.len();
        let added = h.identify_resident_baseline().unwrap();
        assert!(added > 0);
        assert_eq!(h.people.len(), old + added);
        for row in h.population_reconciliation().sites {
            assert_eq!(row.unrepresented_slots, [0; 3]);
            assert_eq!(
                serde_json::to_value((
                    h.sites[row.site as usize].stocks,
                    h.sites[row.site as usize].demography.ages
                ))
                .unwrap(),
                stocks[row.site as usize]
            );
        }
        assert!(h
            .politics
            .as_ref()
            .unwrap()
            .kin
            .iter()
            .filter(|k| k.person as usize >= old)
            .all(|k| k.parents == [None; 2]));
        let once = serde_json::to_value(&*h).unwrap();
        assert_eq!(h.identify_resident_baseline().unwrap(), 0);
        assert_eq!(once, serde_json::to_value(&*h).unwrap());
        let baseline = h.clone();
        g.advance_history(12).unwrap();
        let completed = serde_json::to_value(&g.civilizations).unwrap();
        g.civilizations =
            Some(serde_json::from_value(serde_json::to_value(baseline).unwrap()).unwrap());
        for _ in 0..12 {
            g.advance_history(1).unwrap();
        }
        assert_eq!(completed, serde_json::to_value(&g.civilizations).unwrap());
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn succession_and_service_cannot_identify_the_same_last_slot() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 1;
        let ruler = h.civilizations[0].leader;
        // All other local adults are beyond service-recruitment age. They remain
        // real residents and must still consume adult identity slots.
        let local: Vec<_> = h
            .people
            .iter()
            .filter(|p| h.person_presence(p.id).1 == Presence::Resident(0))
            .map(|p| p.id)
            .collect();
        for id in local {
            h.people[id as usize].born = h.month as i32 - 660;
        }
        h.people[ruler as usize].died = Some(h.month);
        let known = h.population_reconciliation().sites[0].known;
        h.sites[0].demography.ages[1] = known[1] as f32 + 1.;
        h.sites[0].demography.ages[2] = known[2] as f32;
        let before = h.sites[0].demography.ages;
        h.social_month().unwrap();
        assert_ne!(h.civilizations[0].leader, ruler);
        assert_eq!(
            h.population_reconciliation().sites[0].unrepresented_slots[1],
            0
        );
        let people = h.people.len();
        // A fresh subsystem ledger sees the newly identified successor. The
        // ruler is ineligible for service; recruitment cannot name another adult.
        assert!(h.recruit_service_people(0, 1, 1).is_err());
        assert_eq!(h.people.len(), people);
        assert_eq!(h.sites[0].demography.ages, before);
        assert_eq!(h.population_reconciliation().sites[0].overhang[1], 0.);
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
        h.social_month().unwrap();
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
        away.social_month().unwrap();
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
        foreign.social_month().unwrap();
        let ruler = foreign.civilizations[0].leader;
        assert_ne!(ruler, member);
        assert_eq!(foreign.people[member as usize].civilization, 1);
        assert_eq!(foreign.people[ruler as usize].civilization, 0);
        // A relocated ruler's household inherits its own office, not its host's.
        let mut occupied_site = baseline.clone();
        occupied_site.society.as_mut().unwrap().households[account as usize].site = 1;
        occupied_site.people[child as usize].civilization = 1;
        let occupying_ruler = occupied_site.civilizations[1].leader;
        occupied_site.social_month().unwrap();
        assert_eq!(occupied_site.civilizations[0].leader, member);
        assert_eq!(occupied_site.civilizations[1].leader, occupying_ruler);
        assert_eq!(occupied_site.people[member as usize].civilization, 0);
        assert_eq!(occupied_site.people[child as usize].civilization, 1);
        assert_eq!(occupied_site.people.len(), count);
        // An existing person can precede the deceased head in identity creation order.
        let mut older = baseline.clone();
        older.society.as_mut().unwrap().households[account as usize].head = child;
        older.people[child as usize].died = Some(1);
        older.people[old as usize].died = None;
        older.social_month().unwrap();
        assert_eq!(
            older.society.as_ref().unwrap().households[account as usize].head,
            member
        );
        assert_eq!(older.people[member as usize].predecessor, Some(child));
        assert!(valid_succession_links(&older.people));
        // No duplicate successor when two ownership claims need heads in one month.
        let mut simultaneous = baseline.clone();
        simultaneous.people[accounts[1].1 as usize].died = Some(1);
        simultaneous.social_month().unwrap();
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
        elder.social_month().unwrap();
        let successor = elder.society.as_ref().unwrap().households[account as usize].head;
        assert_eq!(age_band(1, elder.people[successor as usize].born), Some(2));
        assert!(!elder
            .events
            .iter()
            .any(|e| e.kind == "succession_identity_overhang"));
        // With both slots exhausted, preserve a vacant estate instead of a new person.
        empty.sites[0].demography.ages[2] = 0.;
        let mut resumed: History =
            serde_json::from_slice(&serde_json::to_vec(&empty).unwrap()).unwrap();
        empty.social_month().unwrap();
        resumed.social_month().unwrap();
        assert_eq!(
            serde_json::to_value(&empty).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert_eq!(
            empty
                .events
                .iter()
                .filter(|e| e.kind == "household_vacant")
                .count(),
            1
        );
        assert_eq!(empty.people.len(), count);
        assert_eq!(
            empty.society.as_ref().unwrap().households[account as usize].head,
            old
        );
        assert_ne!(empty.living_civilization_leader(0), Some(old));
        assert!(empty.living_civilization_leader(0).is_some());
        assert!(empty.events.iter().any(|e| e.kind == "interim_leadership"));
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn vacant_estates_preserve_property_and_recover_without_inventing_residents() {
        let mut g = world();
        let cells = g.snapshot().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        // Missing vacancy fields in older archives retain represented accounts.
        let mut legacy = serde_json::to_value(&h.society.as_ref().unwrap().households[0]).unwrap();
        legacy.as_object_mut().unwrap().remove("vacant_since");
        let legacy: crate::society::Household = serde_json::from_value(legacy).unwrap();
        assert_eq!(legacy.vacant_since, None);
        h.month = 1;
        let accounts: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|a| a.site == 0)
            .map(|a| (a.id, a.head, a.share))
            .collect();
        let civ = h.sites[0].civilization;
        let last_ruler = h.civilizations[civ as usize].leader;
        let population = h.sites[0].stocks.stock[0];
        // Declared all-child population: no anonymous adult or elder slot exists.
        // Identity deaths are a fixture input; no second population debit is requested.
        h.sites[0].demography.ages[..3].copy_from_slice(&[population, 0., 0.]);
        for &(_, head, _) in &accounts {
            h.people[head as usize].died = Some(1);
        }
        let count = h.people.len();
        let stocks = serde_json::to_value(h.sites[0].stocks).unwrap();
        let money = h.economy_residuals();
        h.social_month().unwrap();
        assert_eq!(h.people.len(), count);
        assert_eq!(h.living_civilization_leader(civ), None);
        assert_eq!(h.civilizations[civ as usize].leader, last_ruler);
        assert_eq!(stocks, serde_json::to_value(h.sites[0].stocks).unwrap());
        assert_eq!(money, h.economy_residuals());
        for &(id, old, share) in &accounts {
            let account = &h.society.as_ref().unwrap().households[id as usize];
            assert_eq!(
                (account.head, account.share, account.vacant_since),
                (old, share, Some(1))
            );
            assert!(!h.household_available_for_relocation(id));
        }
        assert!(h.culture.as_ref().unwrap().site_people(h, 0).is_empty());
        h.validate(&cells).unwrap();
        let mut corrupt = h.clone();
        corrupt.society.as_mut().unwrap().households[accounts[0].0 as usize].vacant_since = Some(2);
        assert!(corrupt.validate(&cells).is_err());
        let mut corrupt = h.clone();
        corrupt.society.as_mut().unwrap().households[accounts[0].0 as usize].vacant_since = None;
        assert!(corrupt.validate(&cells).is_err());
        let mut legacy_mode = h.clone();
        legacy_mode.set_named_demography(false).unwrap();
        legacy_mode.social_month().unwrap();
        assert_eq!(legacy_mode.people.len(), count);
        assert!(legacy_mode
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|a| a.site == 0)
            .all(|a| a.vacant_since == Some(1)));
        // Repeated boundaries neither mint successors nor repeat the vacancy event.
        let events = h
            .events
            .iter()
            .filter(|e| e.kind == "household_vacant")
            .count();
        let mut resumed: History =
            serde_json::from_slice(&serde_json::to_vec(&*h).unwrap()).unwrap();
        h.month = 2;
        resumed.month = 2;
        h.social_month().unwrap();
        resumed.social_month().unwrap();
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert_eq!(h.people.len(), count);
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "household_vacant")
                .count(),
            events
        );
        // One newly available whole adult slot restores only one account. This is
        // declared cohort aging, not a population import or a resurrected head.
        h.sites[0].demography.ages[0] -= 1.;
        h.sites[0].demography.ages[1] = 1.;
        h.social_month().unwrap();
        assert_eq!(h.people.len(), count + 1);
        assert_eq!(h.sites[0].stocks.stock[0], population);
        assert_eq!(
            h.society
                .as_ref()
                .unwrap()
                .households
                .iter()
                .filter(|a| a.site == 0 && a.vacant_since.is_none())
                .count(),
            1
        );
        assert!(h.living_civilization_leader(civ).is_some());
        assert!(accounts
            .iter()
            .all(|&(_, id, _)| h.people[id as usize].died == Some(1)));
        h.validate(&cells).unwrap();
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn vacancy_checkpoint_matches_monthly_recovery() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 1;
        for f in &h.society.as_ref().unwrap().households {
            if f.site == 0 {
                h.people[f.head as usize].died = Some(1);
            }
        }
        let population = h.sites[0].stocks.stock[0];
        h.sites[0].demography.ages[..3].copy_from_slice(&[population, 0., 0.]);
        h.social_month().unwrap();
        assert!(h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .any(|a| a.vacant_since.is_some()));
        let file =
            std::env::temp_dir().join(format!("estate-vacancy-{}.world", std::process::id()));
        g.save(&file).unwrap();
        let mut resumed = crate::gpu::Generator::load(g.gpu.clone(), &file).unwrap();
        std::fs::remove_file(file).unwrap();
        g.advance_history(24).unwrap();
        for _ in 0..24 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
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
