//! Sparse genealogy, faction institutions and territorial warfare over GPU population stocks.
use crate::{
    civilization::{History, Person},
    gpu::{Cell, Generator},
    grid,
    society::Raid,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Kinship {
    pub person: u32,
    pub household: u32,
    pub parents: [Option<u32>; 2],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Marriage {
    pub partners: [u32; 2],
    pub started: u32,
    pub ended: Option<u32>,
    pub children: u32,
    pub last_birth: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Faction {
    #[serde(default = "full_cohesion")]
    pub cohesion: f32,
    #[serde(default)]
    pub organizer: Option<u32>,
    pub id: u32,
    pub civilization: u32,
    pub interest: u32,
    pub support: f32,
    pub dissent: f32,
}
fn full_cohesion() -> f32 {
    1.
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claim {
    pub cell: u32,
    pub sites: Vec<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct War {
    #[serde(default)]
    pub name: String,
    pub id: u32,
    pub attacker: u32,
    pub defender: u32,
    pub goal: u32,
    pub started: u32,
    pub ended: Option<u32>,
    pub outcome: String,
    pub cause: u64,
}
impl War {
    pub fn label(&self) -> String {
        if self.name.is_empty() {
            format!("War {}", self.id)
        } else {
            self.name.clone()
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Politics {
    /// Finite post-conquest occupation; absent in older archives.
    #[serde(default)]
    pub occupation_months: u32,
    pub version: u32,
    pub started: u32,
    pub kin: Vec<Kinship>,
    pub marriages: Vec<Marriage>,
    pub factions: Vec<Faction>,
    pub household_factions: Vec<u32>,
    pub governing: Vec<u32>,
    /// Political administration, distinct from settlement cultural affiliation.
    pub controllers: Vec<u32>,
    pub claims: Vec<Claim>,
    pub wars: Vec<War>,
    pub birth_observed: Vec<f32>,
    pub birth_credit: Vec<f32>,
}
impl Politics {
    pub fn claim_owners(&self, claim: &Claim) -> BTreeSet<u32> {
        claim
            .sites
            .iter()
            .map(|&s| self.controllers[s as usize])
            .collect()
    }
    pub fn validate(&self, h: &History, cells: &[Cell]) -> Result<()> {
        let social = h
            .society
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("politics requires households"))?;
        ensure!(
            matches!(self.version, 1 | 2)
                && self.started <= h.month
                && self.occupation_months <= 12,
            "invalid political version or clock"
        );
        ensure!(
            self.controllers.len() == h.sites.len()
                && self
                    .controllers
                    .iter()
                    .all(|&c| (c as usize) < h.civilizations.len()),
            "invalid territorial administration"
        );
        ensure!(
            self.birth_credit.len() == h.sites.len()
                && self.birth_observed.len() == h.sites.len()
                && self
                    .birth_credit
                    .iter()
                    .chain(&self.birth_observed)
                    .all(|x| x.is_finite() && *x >= 0.),
            "invalid named birth accounting"
        );
        // Monthly validation must not rescan every historical union for every child.
        // These temporary indexes preserve the exact chronology and ancestry checks.
        let kin_index: BTreeMap<_, _> = self.kin.iter().map(|k| (k.person, k)).collect();
        ensure!(
            self.kin.len() <= 50000 && kin_index.len() == self.kin.len(),
            "duplicate or excessive genealogy"
        );
        let pair = |a: u32, b: u32| (a.min(b), a.max(b));
        let mut unions: BTreeMap<_, Vec<&Marriage>> = BTreeMap::new();
        for m in &self.marriages {
            unions
                .entry(pair(m.partners[0], m.partners[1]))
                .or_default()
                .push(m);
        }
        let ancestors = |person: u32| {
            let mut result = BTreeSet::from([person]);
            let mut frontier = vec![person];
            for _ in 0..3 {
                let mut next = vec![];
                for id in frontier {
                    if let Some(k) = kin_index.get(&id) {
                        for &p in k.parents.iter().flatten() {
                            result.insert(p);
                            next.push(p);
                        }
                    }
                }
                frontier = next;
            }
            result
        };
        for k in &self.kin {
            ensure!(
                (k.person as usize) < h.people.len()
                    && (k.household as usize) < social.households.len(),
                "invalid family member"
            );
            let child = &h.people[k.person as usize];
            ensure!(
                k.parents[0].is_none() == k.parents[1].is_none()
                    && (k.parents[0].is_none() || k.parents[0] != k.parents[1]),
                "invalid parent pair"
            );
            if k.parents[0].is_some() {
                ensure!(
                    unions
                        .get(&pair(k.parents[0].unwrap(), k.parents[1].unwrap()))
                        .is_some_and(|unions| unions
                            .iter()
                            .any(|m| child.born >= m.started as i32
                                && m.ended.is_none_or(|v| child.born <= v as i32))),
                    "birth lacks a recorded parental union"
                );
            }
            for &parent in k.parents.iter().flatten() {
                ensure!(
                    parent < k.person && kin_index.contains_key(&parent),
                    "cyclic or missing ancestry"
                );
                let p = &h.people[parent as usize];
                ensure!(
                    child.born - p.born >= 192
                        && child.born - p.born <= 660
                        && p.died.is_none_or(|m| m as i32 >= child.born),
                    "impossible parent chronology"
                );
            }
        }
        let mut married = BTreeSet::new();
        for m in &self.marriages {
            ensure!(
                m.started >= self.started
                    && m.started <= h.month
                    && m.last_birth <= h.month
                    && m.ended.is_none_or(|v| v >= m.started && v <= h.month),
                "invalid marriage chronology"
            );
            ensure!(
                m.partners.iter().all(|&p| (p as usize) < h.people.len()
                    && kin_index.contains_key(&p)
                    && m.started as i32 - h.people[p as usize].born >= 216)
                    && ancestors(m.partners[0]).is_disjoint(&ancestors(m.partners[1])),
                "invalid marriage or close kin"
            );
            if m.ended.is_none() {
                ensure!(
                    m.partners
                        .iter()
                        .all(|&p| married.insert(p) && h.people[p as usize].died.is_none()),
                    "overlapping active marriages"
                );
            }
        }
        ensure!(
            self.factions.len()
                == h.civilizations.len()
                    * if self.version == 1 {
                        3
                    } else {
                        crate::faction_interests::COUNT
                    }
                && self
                    .factions
                    .iter()
                    .enumerate()
                    .all(|(i, f)| f.id == i as u32
                        && (f.civilization as usize) < h.civilizations.len()
                        && (f.interest as usize)
                            < if self.version == 1 {
                                3
                            } else {
                                crate::faction_interests::COUNT
                            }
                        && (0. ..=1.).contains(&f.support)
                        && (0. ..=1.).contains(&f.dissent)
                        && (0. ..=1.).contains(&f.cohesion)
                        && f.organizer.is_none_or(|id| (id as usize) < h.people.len()))
                && self
                    .factions
                    .iter()
                    .map(|f| (f.civilization, f.interest))
                    .collect::<BTreeSet<_>>()
                    .len()
                    == self.factions.len(),
            "invalid faction"
        );
        ensure!(
            self.household_factions.len() == social.households.len()
                && self
                    .household_factions
                    .iter()
                    .enumerate()
                    .all(|(i, &f)| (f as usize) < self.factions.len()
                        && self.factions[f as usize].civilization
                            == h.sites[social.households[i].site as usize].civilization),
            "invalid faction membership"
        );
        ensure!(
            self.governing.len() == h.civilizations.len()
                && self.governing.iter().enumerate().all(|(c, &f)| self
                    .factions
                    .get(f as usize)
                    .is_some_and(|f| f.civilization == c as u32)),
            "invalid governing faction"
        );
        let mut previous = None;
        for c in &self.claims {
            ensure!(
                previous.is_none_or(|p| p < c.cell)
                    && cells.get(c.cell as usize).is_some_and(
                        |x| x.meta[0] == 2 && (h.living.is_some() || x.water[0] < 0.25)
                    )
                    && !c.sites.is_empty()
                    && c.sites.windows(2).all(|s| s[0] < s[1])
                    && c.sites.iter().all(|&s| (s as usize) < h.sites.len()),
                "invalid or non-central claim"
            );
            previous = Some(c.cell);
        }
        for (i, w) in self.wars.iter().enumerate() {
            ensure!(
                w.id == i as u32
                    && w.attacker != w.defender
                    && (w.attacker as usize) < h.civilizations.len()
                    && (w.defender as usize) < h.civilizations.len()
                    && (w.goal as usize) < h.sites.len()
                    && w.started <= h.month
                    && w.ended.is_none_or(|m| m >= w.started && m <= h.month)
                    && (w.cause as usize) < h.events.len(),
                "invalid war record"
            );
        }
        Ok(())
    }
}
impl History {
    pub fn controller(&self, site: u32) -> u32 {
        self.politics
            .as_ref()
            .and_then(|p| p.controllers.get(site as usize))
            .copied()
            .unwrap_or(self.sites[site as usize].civilization)
    }
    pub(crate) fn prepare_politics(&mut self, cells: &[Cell]) {
        let Some(mut p) = self.politics.take() else {
            return;
        };
        if p.version == 1 {
            for civ in 0..self.civilizations.len() {
                for interest in 3..crate::faction_interests::COUNT {
                    p.factions.push(Faction {
                        id: p.factions.len() as u32,
                        civilization: civ as u32,
                        interest: interest as u32,
                        support: 0.,
                        dissent: 0.,
                        cohesion: if interest >= 6 { 0.3 } else { 1. },
                        organizer: None,
                    });
                }
            }
            p.version = 2;
            self.event("political_expansion",None,None,"Additional political interests became available; existing faction identities and household memberships retained".into());
        }
        let society = self.society.as_ref().unwrap();
        let rebuild = p.controllers.len() != self.sites.len() || p.claims.is_empty();
        while p.controllers.len() < self.sites.len() {
            let s = &self.sites[p.controllers.len()];
            let inherited = self
                .events
                .iter()
                .rev()
                .find(|e| {
                    e.kind == "migration"
                        && e.other == Some(s.id)
                        && e.month == self.month
                        && self.month > p.started
                })
                .and_then(|e| e.site)
                .and_then(|parent| p.controllers.get(parent as usize))
                .copied();
            p.controllers.push(inherited.unwrap_or(s.civilization));
            p.birth_observed.push(s.stocks.people[0]);
            p.birth_credit.push(0.);
        }
        for f in &society.households {
            if p.household_factions.len() <= f.id as usize {
                p.household_factions.push(
                    p.factions
                        .iter()
                        .find(|x| {
                            x.civilization == self.sites[f.site as usize].civilization
                                && x.interest == f.id % 3
                        })
                        .unwrap()
                        .id,
                );
            }
            if !p.kin.iter().any(|k| k.person == f.head) {
                p.kin.push(Kinship {
                    person: f.head,
                    household: f.id,
                    parents: [None; 2],
                });
            }
        }
        if rebuild {
            // Bounded regional claims: occupied cells, dry immediate hinterland and
            // surveyed road corridors. Unexplored island interiors remain unclaimed.
            let mut claims: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
            for s in &self.sites {
                claims.entry(s.cell).or_default().insert(s.id);
                for (x, y) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let cell = grid::neighbor(s.cell, self.terrain_resolution, x, y);
                    if cells[cell as usize].meta[0] == 2 && cells[cell as usize].water[0] < 0.25 {
                        claims.entry(cell).or_default().insert(s.id);
                    }
                }
            }
            for r in &society.routes {
                for &cell in &r.cells {
                    claims.entry(cell).or_default().extend([r.from, r.to]);
                }
            }
            p.claims = claims
                .into_iter()
                .map(|(cell, sites)| Claim {
                    cell,
                    sites: sites.into_iter().collect(),
                })
                .collect();
        }
        self.politics = Some(p);
    }
    pub(crate) fn genealogy_month(&mut self) {
        let mut child_slots = self
            .named_demography
            .as_ref()
            .map(|_| crate::population_registry::ResidentSlots::new(self));
        let Some(mut p) = self.politics.take() else {
            return;
        };
        let social = self.society.as_ref().unwrap();
        for s in &self.sites {
            let i = s.id as usize;
            p.birth_credit[i] += (s.stocks.people[0] - p.birth_observed[i]).max(0.);
            p.birth_observed[i] = s.stocks.people[0];
        }
        let heads: BTreeSet<_> = social.households.iter().map(|f| f.head).collect();
        for k in &p.kin {
            let on_service = self.person_on_service(k.person);
            let person = &mut self.people[k.person as usize];
            let site = social.households[k.household as usize].site as usize;
            if self.named_demography.is_none()
                && !social.relocation.away(k.household)
                && !on_service
                && !heads.contains(&k.person)
                && person.died.is_none()
                && self.month as i32 - person.born >= 840
                && self.sites[site].demography.health[2] >= 1.
            {
                person.died = Some(self.month);
                self.sites[site].demography.health[2] -= 1.;
            }
        }
        for m in &mut p.marriages {
            if m.ended.is_none()
                && m.partners
                    .iter()
                    .any(|&id| self.people[id as usize].died.is_some())
            {
                m.ended = Some(self.month);
            }
        }
        if self.month % 12 == 0 {
            let mut used: BTreeSet<u32> = p
                .marriages
                .iter()
                .filter(|m| m.ended.is_none())
                .flat_map(|m| m.partners)
                .collect();
            let eligible: Vec<_> = p
                .kin
                .iter()
                .filter(|k| {
                    let v = &self.people[k.person as usize];
                    let age = self.month as i32 - v.born;
                    !social.relocation.away(k.household)
                        && !self.person_on_service(k.person)
                        && v.died.is_none()
                        && !self.sites[social.households[k.household as usize].site as usize]
                            .abandoned
                        && (216..660).contains(&age)
                })
                .map(|k| (k.person, k.household))
                .collect();
            let lookup: BTreeMap<_, _> = p.kin.iter().map(|k| (k.person, k.parents)).collect();
            let ancestry: BTreeMap<_, _> = eligible
                .iter()
                .map(|&(id, _)| {
                    let mut result = BTreeSet::from([id]);
                    let mut frontier = vec![id];
                    for _ in 0..3 {
                        let mut next = vec![];
                        for person in frontier {
                            if let Some(parents) = lookup.get(&person) {
                                for &parent in parents.iter().flatten() {
                                    result.insert(parent);
                                    next.push(parent);
                                }
                            }
                        }
                        frontier = next;
                    }
                    (id, result)
                })
                .collect();
            for &(a, house) in &eligible {
                if used.contains(&a) {
                    continue;
                }
                let site = self.society.as_ref().unwrap().households[house as usize].site;
                if let Some(&(b, _)) = eligible.iter().find(|&&(b, h)| {
                    b != a
                        && !used.contains(&b)
                        && self.society.as_ref().unwrap().households[h as usize].site == site
                        && ancestry[&a].is_disjoint(&ancestry[&b])
                }) {
                    used.extend([a, b]);
                    p.marriages.push(Marriage {
                        partners: [a, b],
                        started: self.month,
                        ended: None,
                        children: 0,
                        last_birth: self.month,
                    });
                    self.event(
                        "marriage",
                        Some(site),
                        None,
                        format!(
                            "{} married {}",
                            self.people[a as usize].name, self.people[b as usize].name
                        ),
                    );
                }
            }
        }
        for index in 0..p.marriages.len() {
            let m = &p.marriages[index];
            if m.ended.is_some()
                || m.partners.iter().any(|id| self.person_on_service(*id))
                || m.partners.iter().any(|id| {
                    p.kin.iter().find(|k| k.person == *id).is_some_and(|k| {
                        let household =
                            &self.society.as_ref().unwrap().households[k.household as usize];
                        let other = p.kin.iter().find(|k| k.person == m.partners[0]).unwrap();
                        self.society.as_ref().unwrap().relocation.away(k.household)
                            || household.site
                                != self.society.as_ref().unwrap().households
                                    [other.household as usize]
                                    .site
                    })
                })
                || m.children >= 4
                || self.month - m.last_birth < 36
                || m.partners
                    .iter()
                    .any(|&id| self.month as i32 - self.people[id as usize].born >= 540)
            {
                continue;
            }
            let house = p
                .kin
                .iter()
                .find(|k| k.person == m.partners[0])
                .unwrap()
                .household;
            let f = &self.society.as_ref().unwrap().households[house as usize];
            let site = f.site as usize;
            if self.society.as_ref().unwrap().relocation.away(f.id)
                || self.sites[site].abandoned
                || p.birth_credit[site] < 1.
                || self.sites[site].demography.ages[0] < 1.
                || p.kin.len() >= 50000
            {
                continue;
            }
            if let Some(slots) = &mut child_slots {
                if !slots.reserve(site as u32, 0, self.sites[site].demography.ages[0], 1) {
                    continue;
                }
            }
            p.birth_credit[site] -= 1.;
            let id = self.people.len() as u32;
            let parents = m.partners;
            self.people.push(Person {
                id,
                name: self.civilizations[self.sites[site].civilization as usize]
                    .naming(self.seed)
                    .person_with(
                        "person",
                        id,
                        &crate::naming::PersonalContext::local(
                            &self.sites[site],
                            self.culture.as_ref(),
                        )
                        .with_person(&self.people[parents[0] as usize])
                        .with_person(&self.people[parents[1] as usize]),
                    ),
                civilization: self.sites[site].civilization,
                born: self.month as i32,
                died: None,
                predecessor: None,
            });
            p.kin.push(Kinship {
                person: id,
                household: house,
                parents: parents.map(Some),
            });
            p.marriages[index].children += 1;
            p.marriages[index].last_birth = self.month;
            self.event(
                "birth",
                Some(site as u32),
                None,
                format!(
                    "{} born to {} and {}",
                    self.people[id as usize].name,
                    self.people[parents[0] as usize].name,
                    self.people[parents[1] as usize].name
                ),
            );
        }
        self.politics = Some(p);
    }
    pub(crate) fn genealogical_heir(&self, old: u32, site: u32) -> Option<u32> {
        let p = self.politics.as_ref()?;
        let social = self.society.as_ref()?;
        p.kin
            .iter()
            .filter(|k| {
                k.parents.contains(&Some(old))
                    && self.person_presence(k.person).1
                        == crate::participation::Presence::Resident(site)
                    && social.households[k.household as usize].site == site
                    && !social.households.iter().any(|f| f.head == k.person)
            })
            .map(|k| &self.people[k.person as usize])
            .filter(|v| v.died.is_none() && self.month as i32 - v.born >= 216)
            .min_by_key(|v| (v.born, v.id))
            .map(|v| v.id)
    }
    pub(crate) fn politics_year(&mut self) {
        let Some(mut p) = self.politics.take() else {
            return;
        };
        for civ in 0..self.civilizations.len() {
            let ids: Vec<usize> = (0..crate::faction_interests::COUNT)
                .map(|k| {
                    p.factions
                        .iter()
                        .position(|f| f.civilization == civ as u32 && f.interest == k as u32)
                        .unwrap()
                })
                .collect();
            let mut votes = [0f32; crate::faction_interests::COUNT];
            let threat = p.wars.iter().any(|w| {
                w.ended.is_none() && (w.attacker == civ as u32 || w.defender == civ as u32)
            });
            let residents: Vec<_> = self
                .society
                .as_ref()
                .unwrap()
                .households
                .iter()
                .filter(|f| {
                    self.sites[f.site as usize].civilization == civ as u32
                        && !self.society.as_ref().unwrap().relocation.away(f.id)
                        && !self.sites[f.site as usize].abandoned
                        && self.people[f.head as usize].died.is_none()
                })
                .collect();
            let mut conditions = Vec::new();
            for f in &residents {
                let s = &self.sites[f.site as usize];
                let a = self
                    .culture
                    .as_ref()
                    .and_then(|c| c.agents.get(f.head as usize));
                let pressure = self
                    .social_indicators(s.id)
                    .map_or([s.stocks.stock[3], 0., 0., 0.], |c| c.pressure);
                let traits = a.map_or([0.5; 6], |a| a.traits);
                let workers = s.economy.labor.iter().sum::<f32>().max(1.);
                conditions.push([
                    pressure[0].clamp(0., 1.),
                    pressure[3],
                    pressure[2],
                    f32::from(threat),
                    (s.economy.labor[3] / workers).clamp(0., 1.),
                    (s.economy.finance[2] / s.economy.finance[1].max(1.)).clamp(0., 1.),
                    traits[2],
                    traits[3],
                ]);
            }
            let mut fragments = Vec::new();
            for (k, &id) in ids.iter().enumerate().skip(6) {
                let faction = &mut p.factions[id];
                let organizer = residents
                    .iter()
                    .zip(&conditions)
                    .max_by(|(a, x), (b, y)| {
                        crate::faction_interests::appeal(k, **x)
                            .total_cmp(&crate::faction_interests::appeal(k, **y))
                            .then_with(|| b.id.cmp(&a.id))
                    })
                    .map(|(f, _)| f.head);
                let pressure = conditions
                    .iter()
                    .map(|x| match k {
                        6 => (x[0] + x[1]) * 0.5,
                        7 => x[6] * (x[0] + x[2]).min(1.),
                        _ => x[3],
                    })
                    .sum::<f32>()
                    / conditions.len().max(1) as f32;
                let before = faction.cohesion;
                faction.cohesion = crate::faction_interests::cohesion(
                    k,
                    before,
                    pressure,
                    faction.organizer.is_some() && faction.organizer != organizer,
                );
                faction.organizer = organizer;
                if before >= 0.6 && faction.cohesion < 0.6 && faction.support > 0.05 {
                    fragments.push((k, faction.support, pressure));
                }
            }
            for (f, x) in residents.iter().zip(&conditions) {
                let previous =
                    p.factions[p.household_factions[f.id as usize] as usize].interest as usize;
                let score = |k: usize| {
                    crate::faction_interests::appeal(k, *x) * p.factions[ids[k]].cohesion
                        + crate::civic_petitions::credit(self, &p, f.site, k as u32)
                        + if k == previous { 0.25 } else { 0. }
                };
                let best = (0..crate::faction_interests::COUNT)
                    .max_by(|&a, &b| score(a).total_cmp(&score(b)).then_with(|| b.cmp(&a)))
                    .unwrap();
                if (f.id + self.month / 12) % 3 == 0 || p.factions[ids[previous]].cohesion < 0.25 {
                    p.household_factions[f.id as usize] = ids[best] as u32;
                }
            }
            drop(residents);
            for (k, support, pressure) in fragments {
                self.event("faction_fragmentation",self.sites.iter().find(|s|s.civilization==civ as u32).map(|s|s.id),None,format!("{}: {} lost cohesion after organizing pressure fell or leadership changed; prior support {:.0}%, pressure {:.2}",self.civilizations[civ].name,crate::faction_interests::NAMES[k],support*100.,pressure));
            }
            for f in &self.society.as_ref().unwrap().households {
                let s = &self.sites[f.site as usize];
                if self.society.as_ref().unwrap().relocation.away(f.id) {
                    continue;
                }
                if s.civilization != civ as u32 {
                    continue;
                }
                let interest =
                    p.factions[p.household_factions[f.id as usize] as usize].interest as usize;
                let threat = p.wars.iter().any(|w| {
                    w.ended.is_none() && (w.attacker == civ as u32 || w.defender == civ as u32)
                });
                let urgency = match interest {
                    0 => {
                        1. + self.social_indicators(s.id).map_or(s.stocks.stock[3], |c| {
                            0.5 * s.stocks.stock[3] + 0.5 * c.pressure[0]
                        }) * 4.
                    }
                    1 => 1. + s.economy.finance[2] / s.economy.finance[1].max(1.),
                    2 | 8 => 1. + if threat { 2. } else { 0. },
                    _ => 1.,
                };
                votes[interest] += f.share as f32
                    * s.stocks.stock[0]
                    * urgency.min(5.)
                    * p.factions[ids[interest]].cohesion;
            }
            let sum = votes.iter().sum::<f32>().max(1.);
            for (j, v) in votes.iter().enumerate() {
                let f = &mut p.factions[ids[j]];
                f.support = *v / sum;
                f.dissent = (1. - f.support)
                    * self
                        .sites
                        .iter()
                        .filter(|s| s.civilization == civ as u32)
                        .map(|s| s.stocks.stock[3])
                        .fold(0f32, f32::max);
            }
            let winner = (0..crate::faction_interests::COUNT)
                .max_by(|&a, &b| votes[a].total_cmp(&votes[b]).then_with(|| b.cmp(&a)))
                .unwrap();
            let faction = ids[winner] as u32;
            if p.governing[civ] != faction
                && p.factions[faction as usize].support
                    > p.factions[p.governing[civ] as usize].support + 0.05
            {
                p.governing[civ] = faction;
                if let Some(f) = self
                    .society
                    .as_ref()
                    .unwrap()
                    .households
                    .iter()
                    .filter(|f| {
                        p.household_factions[f.id as usize] == faction
                            && f.vacant_since.is_none()
                            && self.people[f.head as usize].died.is_none()
                            && !self.person_on_service(f.head)
                            && !self.society.as_ref().unwrap().relocation.away(f.id)
                            && !self.sites[f.site as usize].abandoned
                            && self.people[f.head as usize].civilization == civ as u32
                    })
                    .max_by(|a, b| {
                        let score = |head: u32| {
                            self.culture
                                .as_ref()
                                .and_then(|c| c.agents.get(head as usize))
                                .map_or(0., |a| a.traits[0] + a.traits[4] * 0.5 + a.skills[0])
                        };
                        score(a.head)
                            .total_cmp(&score(b.head))
                            .then_with(|| b.id.cmp(&a.id))
                    })
                {
                    let leader = f.head;
                    self.civilizations[civ].leader = leader;
                    self.event(
                        "faction_shift",
                        Some(f.site),
                        None,
                        format!(
                            "{} gained the council: {}",
                            self.civilizations[civ].name,
                            crate::faction_interests::NAMES[winner]
                        ),
                    );
                    let event = self.events.last_mut().unwrap();
                    event.subjects.push(("person".into(), leader));
                    if let Some(cause) = self
                        .culture
                        .as_ref()
                        .and_then(|c| c.agents.get(leader as usize))
                        .and_then(|a| a.last_campaign)
                    {
                        event.causes.push(cause);
                    }
                }
            }
            self.schedule_tax_policy(
                civ,
                crate::faction_interests::TAX
                    [p.factions[p.governing[civ] as usize].interest as usize],
            );
        }
        self.politics = Some(p);
        // Escalation needs a recent material grievance, a contested corridor and supplies.
        let pairs: Vec<_> = self
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .filter(|r| r.open && r.flood_months == 0)
            .flat_map(|r| [(r.from, r.to), (r.to, r.from)])
            .collect();
        for (a, b) in pairs {
            let grievance = self
                .events
                .iter()
                .rev()
                .find(|e| {
                    self.month.saturating_sub(e.month) <= 24
                        && ((e.kind == "raid_outcome"
                            && e.site == Some(a)
                            && self.controller(a) != self.controller(b)
                            && e.other.is_some_and(|origin| {
                                self.controller(origin) == self.controller(b)
                            }))
                            || (e.kind == "food_crisis"
                                && e.site == Some(a)
                                && self.sites[b as usize].stocks.stock[1]
                                    > self.sites[a as usize].stocks.stock[1] * 1.5))
                })
                .map(|e| e.id);
            if let Some(cause) = grievance {
                let _ = self.start_war(a, b, cause);
            }
        }
    }
    pub(crate) fn start_war(&mut self, origin: u32, target: u32, cause: u64) -> Result<u32> {
        ensure!(
            (origin as usize) < self.sites.len()
                && (target as usize) < self.sites.len()
                && (cause as usize) < self.events.len(),
            "invalid war target or cause"
        );
        let p = self
            .politics
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("enable dynasties and politics first"))?;
        let attacker = self.controller(origin);
        let defender = self.controller(target);
        ensure!(
            !self
                .governance
                .as_ref()
                .is_some_and(|g| g.protected(attacker, defender, self.month)),
            "non-aggression treaty prevents war"
        );
        ensure!(
            attacker != defender
                && !self.sites[origin as usize].abandoned
                && !self.sites[target as usize].abandoned,
            "war requires occupied foreign settlements"
        );
        ensure!(
            !p.wars
                .iter()
                .any(|w| ((w.attacker == attacker && w.defender == defender)
                    || (w.attacker == defender && w.defender == attacker))
                    && w.ended.is_none_or(|m| self.month.saturating_sub(m) < 120)),
            "active war or ten-year truce"
        );
        let distance = self
            .route_cost(origin, target)
            .ok_or_else(|| anyhow::anyhow!("no open land route"))?;
        ensure!(distance < 1500., "campaign exceeds supply range");
        ensure!(
            p.claims
                .iter()
                .any(|c| c.sites.contains(&origin) && c.sites.contains(&target)),
            "no contested territorial corridor"
        );
        ensure!(
            !self
                .society
                .as_ref()
                .unwrap()
                .raids
                .iter()
                .any(|r| r.origin == origin),
            "settlement already has an expedition"
        );
        let months = (distance / 150.).ceil().max(1.) as u32;
        let s = &self.sites[origin as usize];
        let reserve = s.stocks.stock[0] * 18. * 3.;
        let soldiers = (s.demography.ages[1] * 0.25)
            .min(s.economy.goods[3])
            .min((s.stocks.stock[1] - reserve).max(0.) / (18. * (months * 2 + 3) as f32));
        ensure!(
            soldiers >= 3.,
            "insufficient adult manpower, tools or campaign provisions"
        );
        let id = p.wars.len() as u32;
        let recruits = self.recruit_service_people(origin, soldiers.floor() as usize, 3)?;
        let soldiers = recruits.people.len() as f32;
        let food = soldiers * 18. * (months * 2 + 3) as f32;
        let raid_id = self.society.as_ref().unwrap().next_raid;
        self.assign_military_people(raid_id, origin, &recruits.people);
        let source = |site: u32| crate::naming::Source {
            kind: "site".into(),
            id: site,
            name: self.sites[site as usize].name.clone(),
        };
        let target_source = source(target);
        let origin_source = source(origin);
        let leader = self.civilizations[attacker as usize].leader;
        let leader_source = if self.people[leader as usize].died.is_none() {
            crate::naming::Source {
                kind: "person".into(),
                id: leader,
                name: self.people[leader as usize].name.clone(),
            }
        } else {
            source(origin)
        };
        let name = self.civilizations[attacker as usize].naming(self.seed).war(
            id,
            target_source,
            origin_source,
            leader_source,
        );
        let s = &mut self.sites[origin as usize];
        s.stocks.stock[0] -= soldiers;
        s.demography.ages[1] -= soldiers;
        s.stocks.people[3] += soldiers;
        s.stocks.stock[1] -= food;
        s.economy.goods[3] -= soldiers;
        self.event(
            "war_declared",
            Some(origin),
            Some(target),
            format!(
                "{name}: {} claims administration of {}; {:.0} adults mobilized",
                self.civilizations[attacker as usize].name,
                self.sites[target as usize].name,
                soldiers
            ),
        );
        self.record_recruitment(&recruits);
        let event = self.events.last_mut().unwrap();
        event.causes.push(cause);
        let declaration = event.id;
        self.politics.as_mut().unwrap().wars.push(War {
            name,
            id,
            attacker,
            defender,
            goal: target,
            started: self.month,
            ended: None,
            outcome: "campaigning".into(),
            cause: declaration,
        });
        let social = self.society.as_mut().unwrap();
        let raid_id = social.next_raid;
        social.next_raid += 1;
        social.raids.push(Raid {
            members: Some(recruits.people),
            loss_remainder: 0.,
            id: raid_id,
            origin,
            target,
            soldiers,
            food,
            arrives: self.month + months,
            cause: declaration,
            returning: false,
            war: Some(id),
            equipment: soldiers,
            occupation_until: None,
            travel_months: months,
        });
        Ok(id)
    }
    pub(crate) fn campaign_authorized(&self, raid: &Raid) -> bool {
        raid.war.is_none_or(|id| {
            self.politics.as_ref().is_some_and(|p| {
                let w = &p.wars[id as usize];
                w.ended.is_none()
                    && p.controllers[raid.target as usize] == w.defender
                    && p.controllers[raid.origin as usize] == w.attacker
            })
        })
    }
    pub(crate) fn resolve_war(&mut self, raid: &Raid, won: bool) {
        let Some(id) = raid.war else {
            return;
        };
        let Some(p) = self.politics.as_mut() else {
            return;
        };
        let w = &mut p.wars[id as usize];
        let valid = p.controllers[raid.target as usize] == w.defender
            && p.controllers[raid.origin as usize] == w.attacker;
        if won && valid {
            p.controllers[raid.target as usize] = w.attacker;
        }
        w.ended = Some(self.month);
        w.outcome = if !valid {
            "withdrawn"
        } else if won {
            "conquest"
        } else {
            "repulsed"
        }
        .into();
        let outcome = w.outcome.clone();
        let name = w.label();
        self.event(
            "peace",
            Some(raid.target),
            Some(raid.origin),
            format!(
                "{name} ended: {outcome}; ten-year truce. Resident identity and stocks retained."
            ),
        );
        self.events.last_mut().unwrap().causes.push(raid.cause);
    }
}
impl Generator {
    pub fn enable_politics(&mut self) -> Result<()> {
        let cells = self.snapshot()?;
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(
            h.society.is_some() && h.politics.is_none(),
            "politics requires social history without an existing political baseline"
        );
        let count = h.civilizations.len();
        h.politics = Some(Politics {
            occupation_months: 3,
            version: 1,
            started: h.month,
            kin: vec![],
            marriages: vec![],
            factions: (0..count * 3)
                .map(|id| Faction {
                    cohesion: 1.,
                    organizer: None,
                    id: id as u32,
                    civilization: id as u32 / 3,
                    interest: id as u32 % 3,
                    support: 1. / 3.,
                    dissent: 0.,
                })
                .collect(),
            household_factions: vec![],
            governing: (0..count).map(|c| c as u32 * 3).collect(),
            controllers: vec![],
            claims: vec![],
            wars: vec![],
            birth_observed: vec![],
            birth_credit: vec![],
        });
        h.prepare_politics(&cells);
        h.event("political_baseline",None,None,"Founding adults have unknown ancestry; family records, factions and surveyed claims established".into());
        h.record_territory();
        h.validate(&cells)?;
        self.civilizations = Some(h);
        Ok(())
    }
    pub fn declare_war(&mut self, origin: u32, target: u32) -> Result<u32> {
        let cells = self.snapshot()?;
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        h.event(
            "territorial_demand",
            Some(origin),
            Some(target),
            "Scenario: council demands neighboring territory".into(),
        );
        let cause = h.events.len() as u64 - 1;
        let id = h.start_war(origin, target, cause)?;
        h.validate(&cells)?;
        self.civilizations = Some(h);
        Ok(id)
    }
}

#[cfg(test)]
mod expanded_faction_tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn crisis_membership_policy_recovery_and_legacy_ids() {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 64,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(8).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        let cells = g.snapshot().unwrap();
        let mut h = g.civilizations.as_ref().unwrap().clone();
        let count = h.civilizations.len();
        // Recreate a version-one political catalog; upgrade must append, never renumber.
        {
            let p = h.politics.as_mut().unwrap();
            p.factions.truncate(count * 3);
            p.version = 1;
        }
        let before = h.politics.as_ref().unwrap().household_factions.clone();
        h.prepare_politics(&cells);
        assert_eq!(h.politics.as_ref().unwrap().household_factions, before);
        assert_eq!(h.politics.as_ref().unwrap().factions.len(), count * 9);
        h.validate(&cells).unwrap();
        // Freeze production and isolate political responses to declared pressure.
        for _ in 0..8 {
            h.month += 12;
            for s in &mut h
                .society
                .as_mut()
                .unwrap()
                .indicators
                .as_mut()
                .unwrap()
                .sites
            {
                s.pressure = [1., 0., 1., 1.];
            }
            h.activate_monthly_policies();
            h.politics_year();
        }
        let p = h.politics.as_ref().unwrap();
        assert!(p
            .household_factions
            .iter()
            .any(|id| p.factions[*id as usize].interest == 6));
        assert!(p
            .governing
            .iter()
            .any(|id| p.factions[*id as usize].interest == 6));
        for (c, id) in p.governing.iter().enumerate() {
            assert_eq!(
                h.society.as_ref().unwrap().councils[c]
                    .pending_tax
                    .as_ref()
                    .map_or(h.society.as_ref().unwrap().councils[c].tax_rate, |p| p.rate),
                crate::faction_interests::TAX[p.factions[*id as usize].interest as usize]
            );
        }
        let mut resumed: History =
            serde_json::from_slice(&serde_json::to_vec(&h).unwrap()).unwrap();
        for _ in 0..8 {
            for world in [&mut h, &mut resumed] {
                world.month += 12;
                for s in &mut world
                    .society
                    .as_mut()
                    .unwrap()
                    .indicators
                    .as_mut()
                    .unwrap()
                    .sites
                {
                    s.pressure = [0.; 4];
                }
                world.activate_monthly_policies();
                world.politics_year();
            }
        }
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert!(h.events.iter().any(|e| e.kind == "faction_fragmentation"));
        let p = h.politics.as_ref().unwrap();
        assert!(p
            .household_factions
            .iter()
            .all(|id| p.factions[*id as usize].interest != 6));
    }
}
