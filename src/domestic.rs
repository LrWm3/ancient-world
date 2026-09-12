//! Observed domestic groups of known people, separate from ownership accounts.
//! This is not a resident census. No population, goods, money or parentage is created.
use crate::{civilization::History, labor::WorkReceipt, participation::Presence};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod assistance;
mod resolution;

fn help_enabled() -> bool {
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Anchor {
    Person(u32),
    Union(u32),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: u32,
    pub anchor: Anchor,
    pub home: u32,
    pub formed: u32,
    pub ended: Option<u32>,
    pub members: Vec<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MembershipChange {
    pub month: u32,
    pub person: u32,
    pub from: Option<u32>,
    pub to: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeChange {
    pub month: u32,
    pub unit: u32,
    pub from: u32,
    pub to: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CareRow {
    pub unit: u32,
    pub site: u32,
    pub need: f64,
    pub granted: f64,
    pub used: f64,
    pub carers: Vec<(u32, f32)>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CarePlan {
    /// Opening comparison inputs; older archives have no reconstructed forecast.
    #[serde(default)]
    pub projections: Vec<resolution::CareProjection>,
    pub receipt: WorkReceipt,
    pub reserved: bool,
    pub rows: Vec<CareRow>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Domestic {
    #[serde(default = "help_enabled")]
    pub neighbor_help: bool,
    #[serde(default = "help_enabled")]
    pub kin_help: bool,
    pub enabled: bool,
    pub baseline: Option<u32>,
    pub observed: Option<u32>,
    pub units: Vec<Unit>,
    pub membership: BTreeMap<u32, u32>,
    pub changes: Vec<MembershipChange>,
    pub moves: Vec<HomeChange>,
    pub care: Option<CarePlan>,
    pub care_completed: f64,
}
impl Default for Domestic {
    fn default() -> Self {
        Self {
            neighbor_help: true,
            kin_help: true,
            enabled: true,
            baseline: None,
            observed: None,
            units: vec![],
            membership: BTreeMap::new(),
            changes: vec![],
            moves: vec![],
            care: None,
            care_completed: 0.,
        }
    }
}
fn need(age: i32, disease: f32) -> f64 {
    let baseline = if (0..60).contains(&age) {
        0.12
    } else if (60..180).contains(&age) {
        0.04
    } else if age >= 720 {
        // Gradual old-age support, capped at 0.08 worker-months by age 90.
        ((age - 720) as f64 / 360.).min(1.) * 0.08
    } else {
        0.
    };
    // Settlement exposure is a proxy, not an individual diagnosis. Illness
    // increases dependent care while the existing capacity rule limits carers.
    baseline * (1. + disease.clamp(0., 0.5) as f64)
}

fn capacity(h: &History, person: u32, site: u32) -> f32 {
    if h.person_presence(person).1 == Presence::Resident(site)
        && (180..720).contains(&(h.month as i32 - h.people[person as usize].born))
    {
        0.8 * (1. - 0.5 * h.sites[site as usize].demography.health[0].clamp(0., 0.5))
    } else {
        0.
    }
}
impl History {
    /// Enable observations at the current boundary; never invent earlier family biographies.
    pub fn set_domestic_households(&mut self, enabled: bool) -> Result<()> {
        ensure!(
            self.participation
                .as_ref()
                .is_none_or(|p| p.commitments.iter().all(|c| c.settled))
                && self
                    .domestic
                    .as_ref()
                    .and_then(|d| d.care.as_ref())
                    .is_none_or(|c| c.receipt.settled),
            "change domestic participation at a completed work boundary"
        );
        if enabled {
            self.domestic.get_or_insert_with(Default::default).enabled = true;
            self.sync_domestic();
        } else if let Some(d) = &mut self.domestic {
            d.enabled = false;
        }
        Ok(())
    }
    pub fn domestic_report(&self) -> serde_json::Value {
        serde_json::json!(self.domestic)
    }
    pub(crate) fn sync_domestic(&mut self) {
        let Some(mut d) = self.domestic.take() else {
            return;
        };
        if !d.enabled {
            self.domestic = Some(d);
            return;
        }
        let homes: BTreeMap<_, _> = self
            .people
            .iter()
            .filter(|p| p.died.is_none())
            .filter_map(|p| {
                let (account, presence) = self.person_presence(p.id);
                let home = account
                    .and_then(|id| {
                        self.society
                            .as_ref()?
                            .households
                            .get(id as usize)
                            .map(|f| f.site)
                    })
                    .or(match presence {
                        Presence::Resident(s) => Some(s),
                        _ => None,
                    });
                home.map(|s| (p.id, s))
            })
            .collect();
        let mut roots = BTreeMap::new();
        if let Some(p) = &self.politics {
            // Latest supported union wins; bereavement does not by itself erase a household.
            for (index, m) in p.marriages.iter().enumerate().rev() {
                let [a, b] = m.partners;
                let together = homes
                    .get(&a)
                    .zip(homes.get(&b))
                    .is_some_and(|(a, b)| a == b);
                let widowed = self.people[a as usize].died.is_some()
                    || self.people[b as usize].died.is_some();
                if (m.ended.is_none() && together) || widowed {
                    for id in [a, b] {
                        if homes.contains_key(&id) {
                            roots.entry(id).or_insert(Anchor::Union(index as u32));
                        }
                    }
                }
            }
        }
        for &id in homes.keys() {
            if self.month as i32 - self.people[id as usize].born >= 216 {
                roots.entry(id).or_insert(Anchor::Person(id));
            }
        }
        let parents: BTreeMap<_, _> = self
            .politics
            .as_ref()
            .map(|p| p.kin.iter().map(|k| (k.person, k.parents)).collect())
            .unwrap_or_default();
        let mut young: Vec<_> = homes
            .keys()
            .copied()
            .filter(|id| !roots.contains_key(id))
            .collect();
        young.sort_by_key(|id| (self.people[*id as usize].born, *id));
        for id in young {
            let root = parents
                .get(&id)
                .into_iter()
                .flatten()
                .flatten()
                .find_map(|parent| {
                    (homes.get(parent) == homes.get(&id))
                        .then(|| roots.get(parent).copied())
                        .flatten()
                })
                .or_else(|| {
                    // Orphans retain their previous domestic affiliation; no random guardian.
                    parents
                        .get(&id)
                        .filter(|p| {
                            p.iter()
                                .flatten()
                                .any(|id| self.people[*id as usize].died.is_some())
                        })
                        .and_then(|_| d.membership.get(&id))
                        .map(|u| d.units[*u as usize].anchor)
                })
                .unwrap_or(Anchor::Person(id));
            roots.insert(id, root);
        }
        let mut groups = BTreeMap::<(Anchor, u32), Vec<u32>>::new();
        for (&person, &site) in &homes {
            groups
                .entry((roots[&person], site))
                .or_default()
                .push(person);
        }
        let mut membership = BTreeMap::new();
        let mut assigned = BTreeSet::new();
        let old_units = d.units.clone();
        for (&(anchor, home), members) in &groups {
            let found = old_units
                .iter()
                .find(|u| u.ended.is_none() && u.anchor == anchor && u.home == home)
                .or_else(|| {
                    old_units.iter().find(|u| {
                        u.ended.is_none()
                            && u.anchor == anchor
                            && !assigned.contains(&u.id)
                            && !groups.contains_key(&(anchor, u.home))
                            && u.members.iter().any(|p| members.contains(p))
                    })
                });
            let id = if let Some(u) = found {
                u.id
            } else {
                let id = d.units.len() as u32;
                d.units.push(Unit {
                    id,
                    anchor,
                    home,
                    formed: self.month,
                    ended: None,
                    members: vec![],
                });
                id
            };
            let unit = &mut d.units[id as usize];
            if unit.home != home {
                d.moves.push(HomeChange {
                    month: self.month,
                    unit: id,
                    from: unit.home,
                    to: home,
                });
                unit.home = home;
            }
            unit.members = members.clone();
            assigned.insert(id);
            for &person in members {
                membership.insert(person, id);
            }
        }
        for u in &mut d.units {
            if !assigned.contains(&u.id) {
                u.members.clear();
                u.ended.get_or_insert(self.month);
            }
        }
        for person in d
            .membership
            .keys()
            .chain(membership.keys())
            .copied()
            .collect::<BTreeSet<_>>()
        {
            let from = d.membership.get(&person).copied();
            let to = membership.get(&person).copied();
            if from != to {
                d.changes.push(MembershipChange {
                    month: self.month,
                    person,
                    from,
                    to,
                });
            }
        }
        d.membership = membership;
        d.baseline.get_or_insert(self.month);
        d.observed = Some(self.month);
        self.domestic = Some(d);
    }
    pub(crate) fn domestic_care_for(&self, person: u32) -> f32 {
        self.domestic
            .as_ref()
            .filter(|d| d.enabled)
            .and_then(|d| d.care.as_ref())
            .filter(|c| c.receipt.month == self.month)
            .map_or(0., |c| {
                c.rows
                    .iter()
                    .flat_map(|r| &r.carers)
                    .filter(|(p, _)| *p == person)
                    .map(|(_, w)| w)
                    .sum()
            })
    }
    /// A proposed departure must leave enough present caregivers, not just an ownership head.
    pub(crate) fn domestic_departure_allowed(&self, person: u32, leaving: &[u32]) -> bool {
        let Some(d) = self.domestic.as_ref().filter(|d| d.enabled) else {
            return true;
        };
        let Some(unit) = d.membership.get(&person).map(|u| &d.units[*u as usize]) else {
            return true;
        };
        if d.care
            .as_ref()
            .is_some_and(|p| p.receipt.month == self.month && !p.receipt.settled)
            && self.domestic_care_for(person) > 0.
        {
            return false;
        }
        let demand: f64 = unit
            .members
            .iter()
            .filter(|p| self.person_presence(**p).1 == Presence::Resident(unit.home))
            .map(|p| {
                need(
                    self.month as i32 - self.people[*p as usize].born,
                    self.sites[unit.home as usize].demography.health[0],
                )
            })
            .sum();
        let remaining: f64 = unit
            .members
            .iter()
            .filter(|p| **p != person && !leaving.contains(p))
            .map(|p| capacity(self, *p, unit.home) as f64)
            .sum();
        remaining + 1e-6 >= demand
    }
    pub(crate) fn reserve_domestic_care(&mut self) {
        if self
            .domestic
            .as_ref()
            .and_then(|d| d.care.as_ref())
            .is_none_or(|c| c.receipt.month != self.month)
        {
            self.sync_domestic();
        }
        let Some(mut d) = self.domestic.take() else {
            return;
        };
        if !d.enabled {
            self.domestic = Some(d);
            return;
        }
        if d.care
            .as_ref()
            .is_none_or(|c| c.receipt.month != self.month)
        {
            let mut rows = vec![];
            for unit in d.units.iter().filter(|u| u.ended.is_none()) {
                let demand: f64 = unit
                    .members
                    .iter()
                    .filter(|p| self.person_presence(**p).1 == Presence::Resident(unit.home))
                    .map(|p| {
                        need(
                            self.month as i32 - self.people[*p as usize].born,
                            self.sites[unit.home as usize].demography.health[0],
                        )
                    })
                    .sum();
                if demand == 0. {
                    continue;
                }
                let carers: Vec<_> = unit
                    .members
                    .iter()
                    .filter_map(|p| {
                        let cap = capacity(self, *p, unit.home);
                        (cap > 0.).then_some((*p, cap))
                    })
                    .collect();
                let grant = demand.min(carers.iter().map(|(_, c)| *c as f64).sum());
                rows.push(CareRow {
                    unit: unit.id,
                    site: unit.home,
                    need: demand,
                    granted: grant,
                    used: 0.,
                    carers,
                });
            }
            let mut projections = vec![];
            for s in &self.sites {
                if self.resolution.is_some() {
                    projections.push(resolution::CareProjection {
                        site: s.id,
                        demand: rows.iter().filter(|r| r.site == s.id).map(|r| r.need).sum(),
                        pooled_capacity: d
                            .units
                            .iter()
                            .filter(|u| u.ended.is_none() && u.home == s.id)
                            .flat_map(|u| &u.members)
                            .map(|p| capacity(self, *p, s.id) as f64)
                            .sum(),
                        labor: crate::labor::available(
                            s,
                            self.society.is_some(),
                            self.living.is_some(),
                        ) as f64,
                    });
                }
                let requested: f64 = rows
                    .iter()
                    .filter(|r| r.site == s.id)
                    .map(|r| r.granted)
                    .sum();
                let scale =
                    (crate::labor::available(s, self.society.is_some(), self.living.is_some())
                        as f64
                        / requested.max(1e-12))
                    .min(1.);
                for row in rows.iter_mut().filter(|r| r.site == s.id) {
                    row.granted *= scale;
                    let total: f64 = row.carers.iter().map(|(_, c)| *c as f64).sum();
                    for (_, c) in &mut row.carers {
                        *c = (*c as f64 * row.granted / total.max(1e-12)) as f32;
                    }
                    row.granted = row.carers.iter().map(|(_, c)| *c as f64).sum();
                }
            }
            if d.neighbor_help {
                self.match_neighbor_care(&d, &mut rows);
            }
            d.care = Some(CarePlan {
                projections,
                receipt: WorkReceipt {
                    month: self.month,
                    requested: rows.iter().map(|r| r.need).sum(),
                    granted: rows.iter().map(|r| r.granted).sum(),
                    ..Default::default()
                },
                reserved: false,
                rows,
            });
        }
        let plan = d.care.as_mut().unwrap();
        if !plan.reserved && !plan.receipt.settled {
            for r in &plan.rows {
                self.sites[r.site as usize].economy.external[3] += r.granted as f32;
            }
            plan.reserved = true;
        }
        self.domestic = Some(d);
    }
    pub(crate) fn settle_domestic_care(&mut self) {
        let Some(d) = self.domestic.as_mut() else {
            return;
        };
        let Some(plan) = d
            .care
            .as_mut()
            .filter(|p| p.receipt.month == self.month && !p.receipt.settled)
        else {
            return;
        };
        for site in &mut self.sites {
            let grant: f64 = plan
                .rows
                .iter()
                .filter(|r| r.site == site.id)
                .map(|r| r.granted)
                .sum();
            let used = grant.min(site.economy.labor[3].max(0.) as f64);
            for r in plan.rows.iter_mut().filter(|r| r.site == site.id) {
                r.used = r.granted * used / grant.max(1e-12);
            }
            site.economy.external[3] = (site.economy.external[3] - grant as f32).max(0.);
            site.economy.labor[3] = (site.economy.labor[3] - used as f32).max(0.);
        }
        let used = plan.rows.iter().map(|r| r.used).sum();
        plan.receipt.settle(used);
        d.care_completed += used;
    }
}
impl Domestic {
    pub fn validate(&self, h: &History) -> Result<()> {
        ensure!(
            self.baseline.is_none_or(|m| m <= h.month)
                && self.observed.is_none_or(|m| m <= h.month)
                && self.care_completed.is_finite()
                && self.care_completed >= 0.,
            "invalid domestic clock or work"
        );
        let mut members = BTreeMap::new();
        for (i, u) in self.units.iter().enumerate() {
            ensure!(
                u.id == i as u32
                    && (u.home as usize) < h.sites.len()
                    && u.formed <= h.month
                    && u.ended
                        .is_none_or(|m| m >= u.formed && m <= h.month && u.members.is_empty()),
                "invalid domestic unit"
            );
            ensure!(
                match u.anchor {
                    Anchor::Person(p) => (p as usize) < h.people.len(),
                    Anchor::Union(m) => h
                        .politics
                        .as_ref()
                        .is_some_and(|p| (m as usize) < p.marriages.len()),
                },
                "invalid domestic anchor"
            );
            for &p in &u.members {
                ensure!(
                    (p as usize) < h.people.len() && members.insert(p, u.id).is_none(),
                    "duplicate domestic member"
                );
            }
        }
        ensure!(members == self.membership, "domestic membership mismatch");
        for c in &self.changes {
            ensure!(
                c.month <= h.month
                    && (c.person as usize) < h.people.len()
                    && c.from
                        .into_iter()
                        .chain(c.to)
                        .all(|u| (u as usize) < self.units.len())
                    && c.from != c.to,
                "invalid domestic membership history"
            );
        }
        for m in &self.moves {
            ensure!(
                m.month <= h.month
                    && (m.unit as usize) < self.units.len()
                    && [m.from, m.to].iter().all(|s| (*s as usize) < h.sites.len())
                    && m.from != m.to,
                "invalid domestic move"
            );
        }
        if let Some(c) = &self.care {
            c.receipt.validate()?;
            let mut projected_sites = BTreeSet::new();
            for p in &c.projections {
                ensure!(
                    (p.site as usize) < h.sites.len()
                        && projected_sites.insert(p.site)
                        && [p.demand, p.pooled_capacity, p.labor]
                            .iter()
                            .all(|v| v.is_finite() && *v >= 0.),
                    "invalid care projection"
                );
                ensure!(
                    (p.demand
                        - c.rows
                            .iter()
                            .filter(|r| r.site == p.site)
                            .map(|r| r.need)
                            .sum::<f64>())
                    .abs()
                        <= 1e-4,
                    "care projection demand mismatch"
                );
            }
            ensure!(
                c.projections.is_empty()
                    || c.rows.iter().all(|r| projected_sites.contains(&r.site)),
                "incomplete care projections"
            );
            ensure!(c.receipt.month <= h.month, "future domestic care");
            let mut carers: BTreeMap<u32, (u32, f64)> = BTreeMap::new();
            for r in &c.rows {
                ensure!(
                    (r.unit as usize) < self.units.len()
                        && (r.site as usize) < h.sites.len()
                        && [r.need, r.granted, r.used]
                            .iter()
                            .all(|v| v.is_finite() && *v >= 0.)
                        && r.used <= r.granted + 1e-5
                        && r.granted <= r.need + 1e-5
                        && (r.granted - r.carers.iter().map(|(_, w)| *w as f64).sum::<f64>()).abs()
                            < 1e-5,
                    "invalid care allocation"
                );
                let mut row_carers = BTreeSet::new();
                for &(p, w) in &r.carers {
                    ensure!(
                        (p as usize) < h.people.len()
                            && row_carers.insert(p)
                            && w.is_finite()
                            && (0. ..=0.80001).contains(&w),
                        "invalid or duplicate caregiver"
                    );
                    let entry = carers.entry(p).or_insert((r.site, 0.));
                    entry.1 += w as f64;
                    ensure!(
                        entry.0 == r.site && entry.1 <= 0.80001,
                        "caregiver exceeds shared capacity or works across towns"
                    );
                }
            }
            ensure!(
                (c.receipt.requested - c.rows.iter().map(|r| r.need).sum::<f64>()).abs() < 1e-5
                    && (c.receipt.granted - c.rows.iter().map(|r| r.granted).sum::<f64>()).abs()
                        < 1e-5
                    && (c.receipt.used - c.rows.iter().map(|r| r.used).sum::<f64>()).abs() < 1e-5,
                "care receipt mismatch"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependent_care_changes_with_age_and_exposure_with_bounded_demand() {
        assert_eq!(need(-1, 0.5), 0.);
        assert_eq!(need(180, 0.5), 0.);
        assert_eq!(need(719, 0.5), 0.);
        assert_eq!(need(720, 0.), 0.);
        assert!((need(900, 0.) - 0.04).abs() < 1e-12);
        assert!((need(1080, 0.5) - 0.12).abs() < 1e-12);
        assert_eq!(need(1440, 1.), need(1080, 0.5));
        assert!((need(24, 0.5) - 0.18).abs() < 1e-12);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn family_care_competes_for_time_and_protects_dependents() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
            politics::{Kinship, Marriage},
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
        let h = g.civilizations.as_mut().unwrap();
        h.month = 12;
        let heads: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|a| a.site == 0)
            .map(|a| (a.head, a.id))
            .collect();
        assert!(heads.len() >= 2);
        let (a, account) = heads[0];
        let b = heads[1].0;
        h.people[a as usize].born = -300;
        h.people[b as usize].born = -300;
        h.sites[0].demography.health[0] = 0.;
        let child = h.people.len() as u32;
        let mut p = h.people[a as usize].clone();
        p.id = child;
        p.name = "Known child fixture".into();
        p.born = -12;
        h.people.push(p);
        let politics = h.politics.as_mut().unwrap();
        politics.kin.push(Kinship {
            person: child,
            household: account,
            parents: [Some(a), Some(b)],
        });
        politics.marriages.push(Marriage {
            partners: [a, b],
            started: 0,
            ended: None,
            children: 1,
            last_birth: 0,
        });
        let economy = serde_json::to_value(h.sites[0].economy).unwrap();
        let stocks = serde_json::to_value(h.sites[0].stocks).unwrap();
        h.sync_domestic();
        let d = h.domestic.as_ref().unwrap();
        let unit = d.membership[&a];
        assert_eq!(d.membership[&b], unit);
        assert_eq!(d.membership[&child], unit);
        let observed = serde_json::to_value(d).unwrap();
        h.sync_domestic();
        assert_eq!(
            observed,
            serde_json::to_value(h.domestic.as_ref().unwrap()).unwrap()
        );
        assert_eq!(economy, serde_json::to_value(h.sites[0].economy).unwrap());
        assert_eq!(stocks, serde_json::to_value(h.sites[0].stocks).unwrap());
        // Caring for one's own child need not prohibit helping an isolated neighbor.
        let mut sharing = h.clone();
        let neighbor = sharing.people.len() as u32;
        let mut neighbor_person = sharing.people[child as usize].clone();
        neighbor_person.id = neighbor;
        sharing.people.push(neighbor_person);
        sharing.politics.as_mut().unwrap().kin.push(Kinship {
            person: neighbor,
            household: account,
            parents: [None, None],
        });
        for agent in &mut sharing.culture.as_mut().unwrap().agents {
            agent.relations.clear();
            agent.traits[1] = 0.;
        }
        sharing.culture.as_mut().unwrap().agents[a as usize].traits[1] = 1.;
        sharing.culture.as_mut().unwrap().agents[a as usize]
            .relations
            .insert(neighbor, 1.);
        sharing.sync_domestic();
        let mut family_only = sharing.clone();
        family_only.domestic.as_mut().unwrap().neighbor_help = false;
        family_only.open_participation();
        assert!((family_only.domestic_care_for(a) - 0.06).abs() < 1e-6);
        let mut scarce_family = sharing.clone();
        let free = crate::labor::available(&scarce_family.sites[0], true, false);
        scarce_family.sites[0].economy.external[3] += free - 0.06;
        scarce_family.open_participation();
        let scarce_rows = &scarce_family
            .domestic
            .as_ref()
            .unwrap()
            .care
            .as_ref()
            .unwrap()
            .rows;
        assert!(scarce_rows.iter().find(|r| r.unit == unit).unwrap().granted < 0.07);
        assert!(scarce_rows
            .iter()
            .filter(|r| r.unit != unit)
            .all(|r| r.granted < 1e-6));
        let mut resumed_sharing: History =
            serde_json::from_value(serde_json::to_value(&sharing).unwrap()).unwrap();
        sharing.open_participation();
        resumed_sharing.open_participation();
        assert!((sharing.domestic_care_for(a) - 0.16).abs() < 1e-6);
        assert!(
            (sharing.participation.as_ref().unwrap().residents[&a].capacity - 0.64).abs() < 1e-6
        );
        let plan = sharing.domestic.as_ref().unwrap().care.as_ref().unwrap();
        assert!((plan.rows.iter().find(|r| r.unit == unit).unwrap().granted - 0.12).abs() < 1e-6);
        assert!((plan.receipt.granted - 0.22).abs() < 1e-6);
        sharing
            .domestic
            .as_ref()
            .unwrap()
            .validate(&sharing)
            .unwrap();
        let mut overbooked = sharing.domestic.clone().unwrap();
        let overbooked_plan = overbooked.care.as_mut().unwrap();
        overbooked_plan.projections.clear();
        for row in &mut overbooked_plan.rows {
            for (person, work) in &mut row.carers {
                if *person == a {
                    *work = 0.5;
                }
            }
            row.granted = row.carers.iter().map(|(_, work)| *work as f64).sum();
            row.need = row.granted;
        }
        overbooked_plan.receipt.requested = overbooked_plan.rows.iter().map(|r| r.need).sum();
        overbooked_plan.receipt.granted = overbooked_plan.receipt.requested;
        assert!(overbooked
            .validate(&sharing)
            .unwrap_err()
            .to_string()
            .contains("shared capacity"));
        for world in [&mut sharing, &mut resumed_sharing] {
            world.sites[0].economy.labor[3] = 0.22;
            world.settle_domestic_care();
            world.domestic.as_ref().unwrap().validate(world).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&sharing).unwrap(),
            serde_json::to_value(&resumed_sharing).unwrap()
        );
        assert!((sharing.domestic.as_ref().unwrap().care_completed - 0.22).abs() < 1e-6);
        let mut exposed = h.clone();
        exposed.sites[0].demography.health[0] = 0.5;
        exposed.open_participation();
        let care = exposed.domestic.as_ref().unwrap().care.as_ref().unwrap();
        assert!((care.receipt.requested - 0.18).abs() < 1e-6);
        assert!((exposed.domestic_care_for(a) - 0.09).abs() < 1e-6);
        assert!(
            (exposed.participation.as_ref().unwrap().residents[&a].capacity - 0.51).abs() < 1e-6
        );
        let mut elder = h.clone();
        elder.people[a as usize].born = elder.month as i32 - 1080;
        elder.open_participation();
        assert!(
            (elder
                .domestic
                .as_ref()
                .unwrap()
                .care
                .as_ref()
                .unwrap()
                .receipt
                .requested
                - 0.20)
                .abs()
                < 1e-6
        );
        assert!(!elder.domestic_departure_allowed(b, &[a]));
        assert!(h.domestic_departure_allowed(a, &[]));
        assert!(!h.domestic_departure_allowed(b, &[a]));
        let mut disabled = h.clone();
        disabled.set_domestic_households(false).unwrap();
        disabled.open_participation();
        h.resolution = Some(crate::resolution::ResolutionState {
            compare: true,
            ..Default::default()
        });
        // Same residents and labor, but no family connection to the child.
        // The pooled counterfactual sees spare adults; actual care must not invent a guardian.
        let mut isolated = h.clone();
        isolated
            .politics
            .as_mut()
            .unwrap()
            .kin
            .iter_mut()
            .find(|k| k.person == child)
            .unwrap()
            .parents = [None, None];
        isolated.open_participation();
        isolated.settle_domestic_care();
        isolated.settle_care_resolutions().unwrap();
        let comparison = &isolated.resolution.as_ref().unwrap().receipts[0];
        assert!((comparison.metrics[1].expected - 0.12).abs() < 1e-6);
        assert_eq!(comparison.metrics[1].actual, 0.);
        // Adult children retain a local care connection after forming a separate unit.
        let mut separated = isolated.clone();
        separated.domestic.as_mut().unwrap().care = None;
        separated.participation.as_mut().unwrap().month = None;
        separated.resolution.as_mut().unwrap().receipts.clear();
        separated.people[a as usize].born = separated.month as i32 - 1080;
        for marriage in &mut separated.politics.as_mut().unwrap().marriages {
            marriage.ended = Some(separated.month);
        }
        separated
            .politics
            .as_mut()
            .unwrap()
            .kin
            .iter_mut()
            .find(|k| k.person == b)
            .unwrap()
            .parents = [Some(a), None];
        // This fixture predates the monthly culture sync for newly named household heads.
        let agents = &mut separated.culture.as_mut().unwrap().agents;
        while agents.len() <= b as usize {
            let mut helper = agents[a as usize].clone();
            helper.person = agents.len() as u32;
            agents.push(helper);
        }
        for agent in &mut separated.culture.as_mut().unwrap().agents {
            agent.relations.clear();
        }
        separated.culture.as_mut().unwrap().agents[b as usize].traits[1] = 1.;
        let mut without_kin = separated.clone();
        without_kin.domestic.as_mut().unwrap().kin_help = false;
        without_kin.open_participation();
        assert_eq!(without_kin.domestic_care_for(b), 0.);
        let mut estranged = separated.clone();
        estranged.culture.as_mut().unwrap().agents[b as usize]
            .relations
            .insert(a, -1.);
        estranged.open_participation();
        assert_eq!(estranged.domestic_care_for(b), 0.);
        separated.open_participation();
        let d = separated.domestic.as_ref().unwrap();
        assert_ne!(d.membership[&a], d.membership[&b]);
        assert!((separated.domestic_care_for(b) - 0.075).abs() < 1e-6);
        assert!(
            (separated.participation.as_ref().unwrap().residents[&b].capacity - 0.725).abs() < 1e-6
        );
        assert!(!separated.domestic_departure_allowed(b, &[]));
        separated.sites[0].economy.labor[3] = 0.075;
        separated.settle_domestic_care();
        separated.settle_care_resolutions().unwrap();
        separated
            .domestic
            .as_ref()
            .unwrap()
            .validate(&separated)
            .unwrap();
        // A known, generous adult can help the isolated child, using real time.
        let mut connected = isolated.clone();
        connected.domestic.as_mut().unwrap().care = None;
        connected.participation.as_mut().unwrap().month = None;
        connected.resolution.as_mut().unwrap().receipts.clear();
        connected.culture.as_mut().unwrap().agents[a as usize].traits[1] = 1.;
        connected.culture.as_mut().unwrap().agents[a as usize]
            .relations
            .insert(child, 1.);
        let mut no_help = connected.clone();
        no_help.domestic.as_mut().unwrap().neighbor_help = false;
        no_help.open_participation();
        assert_eq!(no_help.domestic_care_for(a), 0.);
        let mut absent = connected.clone();
        absent.people[a as usize].died = Some(absent.month);
        absent.open_participation();
        assert_eq!(absent.domestic_care_for(a), 0.);
        let mut scarce = connected.clone();
        let spare = crate::labor::available(&scarce.sites[0], true, false);
        scarce.sites[0].economy.external[3] += spare;
        scarce.open_participation();
        assert!(scarce.domestic_care_for(a) < 1e-5);
        let mut resumed_help: History =
            serde_json::from_value(serde_json::to_value(&connected).unwrap()).unwrap();
        connected.open_participation();
        resumed_help.open_participation();
        assert!((connected.domestic_care_for(a) - 0.1).abs() < 1e-6);
        assert!(
            (connected.participation.as_ref().unwrap().residents[&a].capacity - 0.7).abs() < 1e-6
        );
        assert!(!connected.domestic_departure_allowed(a, &[]));
        connected.sites[0].economy.labor[3] = 0.1;
        resumed_help.sites[0].economy.labor[3] = 0.1;
        connected.settle_domestic_care();
        resumed_help.settle_domestic_care();
        connected.settle_care_resolutions().unwrap();
        resumed_help.settle_care_resolutions().unwrap();
        assert_eq!(
            serde_json::to_value(&connected).unwrap(),
            serde_json::to_value(&resumed_help).unwrap()
        );
        assert!((connected.domestic.as_ref().unwrap().care_completed - 0.1).abs() < 1e-6);
        connected
            .domestic
            .as_ref()
            .unwrap()
            .validate(&connected)
            .unwrap();

        let available = crate::labor::available(&h.sites[0], true, false);
        h.open_participation();
        let grant = h
            .domestic
            .as_ref()
            .unwrap()
            .care
            .as_ref()
            .unwrap()
            .receipt
            .granted;
        assert!((grant - 0.12).abs() < 1e-6);
        assert!((h.domestic_care_for(a) - 0.06).abs() < 1e-6);
        assert!((h.participation.as_ref().unwrap().residents[&a].capacity - 0.74).abs() < 1e-6);
        assert!(
            (disabled.participation.as_ref().unwrap().residents[&a].capacity - 0.8).abs() < 1e-6
        );
        assert!(
            (available - crate::labor::available(&h.sites[0], true, false) - 0.12).abs() < 1e-4
        );
        assert!(!h.domestic_departure_allowed(a, &[]));
        let reservation = h.sites[0].economy.external[3];
        h.open_participation();
        assert_eq!(reservation, h.sites[0].economy.external[3]);
        // Simulated completed production supplies only half the promised service labor.
        h.sites[0].economy.labor[3] = 0.06;
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        h.settle_domestic_care();
        resumed.settle_domestic_care();
        let physical = serde_json::to_value(&*h).unwrap();
        h.settle_care_resolutions().unwrap();
        resumed.settle_care_resolutions().unwrap();
        let r = &h.resolution.as_ref().unwrap().receipts[0];
        assert_eq!(r.boundary.system, crate::resolution::System::DomesticCare);
        assert!((r.metrics[2].actual - 0.06).abs() < 1e-6);
        let mut measured = serde_json::to_value(&*h).unwrap();
        measured["resolution"] = physical["resolution"].clone();
        assert_eq!(measured, physical);
        let before = serde_json::to_value(&*h).unwrap();
        assert!(h.settle_care_resolutions().is_err());
        assert_eq!(before, serde_json::to_value(&*h).unwrap());
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        assert!((h.domestic.as_ref().unwrap().care_completed - 0.06).abs() < 1e-6);
        assert_eq!(h.sites[0].economy.labor[3], 0.);
        h.settle_domestic_care();
        assert!((h.domestic.as_ref().unwrap().care_completed - 0.06).abs() < 1e-6);
        h.domestic.as_ref().unwrap().validate(h).unwrap();
        let mut corrupt = h.domestic.clone().unwrap();
        corrupt.units[unit as usize].members.push(a);
        assert!(corrupt.validate(h).is_err());
        let mut bereaved = h.clone();
        bereaved.people[b as usize].died = Some(12);
        bereaved.month = 13;
        bereaved.sites[0].economy.external[3] = 0.;
        bereaved.open_participation();
        assert!((bereaved.domestic_care_for(a) - 0.12).abs() < 1e-6);
        assert!(
            (bereaved.participation.as_ref().unwrap().residents[&a].capacity - 0.68).abs() < 1e-6
        );
        assert!(!bereaved.domestic_departure_allowed(a, &[]));
        // Maturing children become separate domestic units; widowhood retains union identity.
        h.month = 228;
        h.sync_domestic();
        assert_ne!(h.domestic.as_ref().unwrap().membership[&child], unit);
        h.people[b as usize].died = Some(228);
        h.politics
            .as_mut()
            .unwrap()
            .marriages
            .last_mut()
            .unwrap()
            .ended = Some(228);
        h.sync_domestic();
        assert_eq!(h.domestic.as_ref().unwrap().membership[&a], unit);
        assert!(!h.domestic.as_ref().unwrap().membership.contains_key(&b));
        h.domestic.as_ref().unwrap().validate(h).unwrap();
        let mut legacy = serde_json::to_value(&*h).unwrap();
        legacy.as_object_mut().unwrap().remove("domestic");
        let mut legacy: History = serde_json::from_value(legacy).unwrap();
        assert!(legacy.domestic.is_none());
        legacy.set_domestic_households(true).unwrap();
        assert_eq!(legacy.domestic.unwrap().baseline, Some(228));
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn care_monthly_batch_and_checkpoint_agree() {
        use crate::{
            catalog::Catalog,
            config::Config,
            gpu::{ContextGpu, Generator},
            politics::{Kinship, Marriage},
        };
        let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
        let mut g = Generator::new(
            gpu.clone(),
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
        let h = g.civilizations.as_mut().unwrap();
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
        let child = h.people.len() as u32;
        let mut p = h.people[a as usize].clone();
        p.id = child;
        p.name = "Care continuation fixture".into();
        p.born = 0;
        h.people.push(p);
        h.politics.as_mut().unwrap().kin.push(Kinship {
            person: child,
            household: heads[0].1,
            parents: [None, None],
        });
        h.politics.as_mut().unwrap().marriages.push(Marriage {
            partners: [a, b],
            started: 0,
            ended: None,
            children: 1,
            last_birth: 0,
        });
        h.culture.as_mut().unwrap().agents[a as usize].traits[1] = 1.;
        h.culture.as_mut().unwrap().agents[a as usize]
            .relations
            .insert(child, 1.);
        h.sync_domestic();
        h.resolution = Some(crate::resolution::ResolutionState {
            compare: true,
            ..Default::default()
        });
        g.advance_history(3).unwrap();
        let path = std::path::PathBuf::from(format!(
            "output/domestic-checkpoint-{}.world",
            std::process::id()
        ));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(gpu, &path).unwrap();
        std::fs::remove_file(&path).unwrap();
        g.advance_history(12).unwrap();
        for _ in 0..12 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
        let h = g.civilizations.as_ref().unwrap();
        assert!(h.domestic.as_ref().unwrap().care_completed > 0.);
        h.validate_service_work().unwrap();
    }
}
