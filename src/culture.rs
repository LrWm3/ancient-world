//! Sparse historical agency. Claims are attributed interpretations, never physical facts.
use crate::{
    civilization::History,
    gpu::{Cell, Generator},
    grid,
};
use anyhow::{ensure, Result};
pub mod dynamics;
mod practices;
mod work_requests;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const TOPICS: [&str; 12] = [
    "crop calendars",
    "soil husbandry",
    "herding",
    "lake navigation",
    "metalworking",
    "weaving",
    "preservation",
    "medicine",
    "resin assay",
    "phosphorus assay",
    "writing",
    "weather signs",
];
pub const THEMES: [&str; 8] = [
    "hospitality",
    "stewardship",
    "courage",
    "restraint",
    "inquiry",
    "reciprocity",
    "remembrance",
    "independence",
];
fn unit(seed: u32, id: u32, time: u32, stream: u32) -> f32 {
    let mut x =
        seed ^ id.wrapping_mul(7919) ^ time.wrapping_mul(104729) ^ stream.wrapping_mul(0x9e3779b9);
    x = (x ^ (x >> 16)).wrapping_mul(0x7feb352d);
    x = (x ^ (x >> 15)).wrapping_mul(0x846ca68b);
    ((x ^ (x >> 16)) >> 8) as f32 / 16777216.
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoundingOptions {
    pub animal_months: u32,
    pub intelligent_months: u32,
    pub variance: [f32; 2],
    pub aid_enabled: bool,
}
impl Default for FoundingOptions {
    fn default() -> Self {
        Self {
            animal_months: 84,
            intelligent_months: 36,
            variance: [0.85, 1.15],
            aid_enabled: true,
        }
    }
}
impl FoundingOptions {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (1..=1200).contains(&self.animal_months)
                && (1..=1200).contains(&self.intelligent_months)
                && self
                    .variance
                    .iter()
                    .all(|v| v.is_finite() && (0.1..=3.).contains(v))
                && self.variance[0] <= self.variance[1],
            "invalid founding options"
        );
        Ok(())
    }
    pub fn duration(&self, intelligent: bool, strength: f32, variance: f32) -> u32 {
        ((if intelligent {
            self.intelligent_months
        } else {
            self.animal_months
        }) as f32
            * variance
            / (1. + strength))
            .round()
            .max(1.) as u32
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Archetype {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub appearance: String,
    pub communication: String,
    pub habitat: String,
    pub aid_strength: f32,
    pub teaching: u32,
    pub theme: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatronCatalog {
    pub version: u32,
    pub patrons: Vec<Archetype>,
}
impl PatronCatalog {
    pub fn bundled() -> Result<Self> {
        let c: Self = toml::from_str(include_str!("../assets/patrons.toml"))?;
        c.validate()?;
        Ok(c)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1 && !self.patrons.is_empty() && self.patrons.len() <= 64,
            "invalid patron catalog"
        );
        let mut ids = BTreeSet::new();
        for p in &self.patrons {
            ensure!(
                ids.insert(&p.id)
                    && !p.id.is_empty()
                    && matches!(p.kind.as_str(), "animal" | "intelligent")
                    && p.aid_strength.is_finite()
                    && (0. ..=1.).contains(&p.aid_strength)
                    && p.teaching < 12
                    && p.theme < 8
                    && matches!(
                        p.habitat.as_str(),
                        "lake" | "wetland" | "forest" | "volcanic" | "coast"
                    ),
                "invalid patron archetype"
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Patron {
    pub id: u32,
    pub archetype: u32,
    pub name: String,
    pub civilization: u32,
    pub site: u32,
    pub origin: u32,
    pub landing: u32,
    pub voyage: Vec<u32>,
    pub witnesses: Vec<u32>,
    pub arrival_event: u64,
    pub departure_month: u32,
    pub departed: Option<u32>,
    pub effort: [f32; 4],
    pub provisions_kg: f64,
    pub consumed_kg: f64,
    pub returned_kg: f64,
    pub imported_kg: f64,
    pub initial_people: f32,
    pub initial_food: f32,
    pub initial_tools: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tradition {
    pub id: u32,
    pub name: String,
    pub patron: Option<u32>,
    pub parent: Option<u32>,
    pub themes: [u32; 4],
    pub founded: u32,
    pub sacred_site: u32,
    pub dissent: f32,
    pub leader: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: u32,
    pub tradition: u32,
    pub author: Option<u32>,
    pub institution: Option<u32>,
    pub month: u32,
    pub facts: Vec<u64>,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub person: u32,
    pub traits: [f32; 6],
    pub skills: [f32; 4],
    pub occupation: String,
    pub goal: String,
    pub knowledge: BTreeSet<u32>,
    #[serde(default)]
    pub known_places: BTreeSet<u32>,
    #[serde(default)]
    pub knowledge_sources: BTreeMap<u32, u64>,
    #[serde(default)]
    pub last_campaign: Option<u64>,
    pub relations: BTreeMap<u32, f32>,
    pub actions: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstitutionKind {
    Religious,
    Merchant,
    Craft,
    Scholarly,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Institution {
    #[serde(default)]
    pub capacity: Option<crate::institution_capacity::Capacity>,
    pub id: u32,
    pub name: String,
    pub kind: InstitutionKind,
    pub site: u32,
    pub tradition: Option<u32>,
    pub members: Vec<u32>,
    pub leader: u32,
    pub treasury: f64,
    pub active: bool,
    pub founded: u32,
    pub knowledge: BTreeSet<u32>,
    pub property: Vec<u32>,
    pub dues: f64,
    pub expenses: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Owner {
    Person(u32),
    Institution(u32),
    Community(u32),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub id: u32,
    pub name: String,
    pub kind: String,
    pub creator: Option<u32>,
    pub owner: Owner,
    pub claims: Vec<Owner>,
    pub site: Option<u32>,
    pub custodian: Option<u32>,
    pub materials: Vec<(u32, f32)>,
    pub topic: Option<u32>,
    pub tradition: Option<u32>,
    pub events: Vec<u64>,
    pub destroyed: bool,
    #[serde(default)]
    pub lost: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Culture {
    #[serde(default)]
    pub religious_dynamics: dynamics::ReligiousDynamics,
    #[serde(default)]
    pub local_recoveries: Vec<crate::local_places::RecoveryRequest>,
    #[serde(default)]
    pub religious_relief: crate::religious_relief::ReligiousRelief,
    pub version: u32,
    pub started: u32,
    pub legacy_baseline: bool,
    pub options: FoundingOptions,
    pub catalog: PatronCatalog,
    pub patrons: Vec<Patron>,
    pub traditions: Vec<Tradition>,
    pub accounts: Vec<Account>,
    pub agents: Vec<Agent>,
    pub institutions: Vec<Institution>,
    pub artifacts: Vec<Artifact>,
    pub household_faith: Vec<u32>,
    pub site_faith: Vec<u32>,
    pub contact: BTreeMap<String, u32>,
    pub processed_voyages: BTreeSet<u32>,
    pub labor_spent: f64,
    pub labor_budget: Vec<f32>,
    #[serde(default)]
    pub roles: Vec<crate::agriculture::RoleState>,
}
impl Culture {
    fn empty(month: u32, legacy: bool, options: FoundingOptions) -> Result<Self> {
        Ok(Self {
            religious_dynamics: Default::default(),
            local_recoveries: vec![],
            religious_relief: Default::default(),
            version: 1,
            started: month,
            legacy_baseline: legacy,
            options,
            catalog: PatronCatalog::bundled()?,
            patrons: vec![],
            traditions: vec![],
            accounts: vec![],
            agents: vec![],
            institutions: vec![],
            artifacts: vec![],
            household_faith: vec![],
            site_faith: vec![],
            contact: BTreeMap::new(),
            processed_voyages: BTreeSet::new(),
            labor_spent: 0.,
            labor_budget: vec![],
            roles: vec![],
        })
    }
    pub fn validate(&self, h: &History, cells: &[Cell]) -> Result<()> {
        self.options.validate()?;
        self.catalog.validate()?;
        ensure!(
            self.version == 1
                && self.started <= h.month
                && self.site_faith.len() == h.sites.len()
                && self.agents.len() <= h.people.len(),
            "invalid cultural clock or dimensions"
        );
        ensure!(
            self.roles.len() <= h.sites.len()
                && self.roles.iter().all(|r| r.month <= h.month
                    && r.cumulative
                        .iter()
                        .chain(&r.scores)
                        .all(|v| v.is_finite() && *v >= 0.)),
            "invalid annual settlement roles"
        );
        for (i, p) in self.patrons.iter().enumerate() {
            ensure!(
                p.id as usize == i
                    && (p.archetype as usize) < self.catalog.patrons.len()
                    && (p.site as usize) < h.sites.len()
                    && (p.civilization as usize) < h.civilizations.len()
                    && cells.get(p.origin as usize).is_some_and(|c| c.meta[0] == 3)
                    && cells
                        .get(p.landing as usize)
                        .is_some_and(|c| c.meta[0] == 2)
                    && p.departure_month > 0
                    && p.arrival_event < h.events.len() as u64
                    && p.witnesses.iter().all(|&id| (id as usize) < h.people.len())
                    && p.effort.iter().all(|v| v.is_finite() && *v >= 0.)
                    && [p.provisions_kg, p.consumed_kg, p.returned_kg, p.imported_kg]
                        .iter()
                        .all(|v| v.is_finite() && *v >= 0.)
                    && (p.imported_kg - p.provisions_kg - p.consumed_kg - p.returned_kg).abs()
                        < 1e-6
                    && p.departed
                        .is_none_or(|m| m == p.departure_month && m <= h.month)
                    && (h.month < p.departure_month || p.departed.is_some()),
                "invalid patron or service ledger"
            );
        }
        self.validate_local_recoveries(h)?;
        self.religious_relief.validate(h, self.institutions.len())?;
        self.religious_dynamics.validate(h, self.traditions.len())?;
        for (i, t) in self.traditions.iter().enumerate() {
            ensure!(
                t.id as usize == i
                    && t.founded <= h.month
                    && t.parent.is_none_or(|id| id < t.id)
                    && t.patron.is_none_or(|id| (id as usize) < self.patrons.len())
                    && t.themes.iter().all(|&x| x < 8)
                    && (t.sacred_site as usize) < h.sites.len()
                    && (t.leader as usize) < h.people.len()
                    && t.dissent.is_finite(),
                "invalid tradition"
            );
        }
        ensure!(
            self.site_faith
                .iter()
                .chain(&self.household_faith)
                .all(|&t| (t as usize) < self.traditions.len()),
            "invalid affiliation"
        );
        for (i, a) in self.agents.iter().enumerate() {
            ensure!(
                a.person as usize == i
                    && a.traits
                        .iter()
                        .chain(&a.skills)
                        .all(|v| v.is_finite() && (0. ..=1.).contains(v))
                    && a.knowledge.iter().all(|&k| k < 12)
                    && a.knowledge_sources
                        .iter()
                        .all(|(k, event)| a.knowledge.contains(k)
                            && (*event as usize) < h.events.len())
                    && a.last_campaign.is_none_or(|event| h
                        .events
                        .get(event as usize)
                        .is_some_and(|e| e.kind == "office_campaign"))
                    && a.known_places.iter().all(|&k| (k as usize) < cells.len())
                    && a.relations
                        .iter()
                        .all(|(&id, &v)| (id as usize) < h.people.len()
                            && v.is_finite()
                            && (-1. ..=1.).contains(&v)),
                "invalid personal state"
            );
        }
        for (i, a) in self.accounts.iter().enumerate() {
            ensure!(
                a.id as usize == i
                    && (a.tradition as usize) < self.traditions.len()
                    && a.month <= h.month
                    && a.author.is_none_or(|id| (id as usize) < h.people.len())
                    && a.institution
                        .is_none_or(|id| (id as usize) < self.institutions.len())
                    && a.facts.iter().all(|&id| h
                        .events
                        .get(id as usize)
                        .is_some_and(|e| e.month <= a.month)),
                "invalid attributed account"
            );
        }
        for (i, n) in self.institutions.iter().enumerate() {
            ensure!(
                n.capacity.as_ref().is_none_or(|c| c.readiness.is_finite()
                    && (0. ..=1.).contains(&c.readiness)
                    && c.paid.is_finite()
                    && c.paid >= 0.
                    && c.work.is_finite()
                    && c.work >= 0.
                    && c.observed <= h.month),
                "invalid institution capacity"
            );
            ensure!(
                n.id as usize == i
                    && (n.site as usize) < h.sites.len()
                    && (n.leader as usize) < h.people.len()
                    && n.members.iter().all(|&id| (id as usize) < h.people.len())
                    && [n.treasury, n.dues, n.expenses]
                        .iter()
                        .all(|v| v.is_finite() && *v >= 0.)
                    && n.property
                        .iter()
                        .all(|&id| (id as usize) < self.artifacts.len()),
                "invalid institution"
            );
        }
        for n in &self.institutions {
            if let Some(m) = n.capacity.as_ref().and_then(|c| c.mandate.as_ref()) {
                ensure!(
                    m.since <= h.month
                        && m.observed <= h.month
                        && m.vacant_since.is_none_or(|t| t <= m.observed)
                        && (!m.contested || m.holder.is_none())
                        && m.holder.is_some() == m.vacant_since.is_none()
                        && m.holder
                            .is_none_or(|id| id == n.leader && (id as usize) < h.people.len())
                        && m.support.is_finite()
                        && (0. ..=1.).contains(&m.support)
                        && m.work.is_finite()
                        && m.work >= 0.
                        && m.events.windows(2).all(|w| w[0] < w[1])
                        && m.events.iter().all(|&id| h
                            .events
                            .get(id as usize)
                            .is_some_and(|e| e.month <= m.observed
                                && e.subjects.contains(&("institution".into(), n.id)))),
                    "invalid institutional mandate"
                );
            }
        }
        for n in &self.institutions {
            if let Some(b) = n.capacity.as_ref().and_then(|c| c.building.as_ref()) {
                ensure!(
                    b.condition.is_finite()
                        && (0. ..=1.).contains(&b.condition)
                        && b.construction_remaining.is_finite()
                        && b.construction_remaining >= 0.
                        && b.repaired_kg.is_finite()
                        && b.repaired_kg >= 0.
                        && b.repair_paid.is_finite()
                        && b.repair_paid >= 0.
                        && self
                            .artifacts
                            .get(b.artifact as usize)
                            .is_some_and(|a| a.kind == "institutional foundation"
                                && b.facility.as_ref().map_or_else(
                                    || a.materials.len() == 1
                                        && a.materials[0].0 == 5
                                        && a.materials[0].1.is_finite()
                                        && a.materials[0].1 > 0.
                                        && b.construction_remaining
                                            <= crate::institution_capacity::HALL_WORK_MONTHS,
                                    |f| f.valid(h.economy_catalog.as_ref().unwrap(), a)
                                        && b.construction_remaining == f.remaining()
                                )),
                    "invalid institutional meeting place"
                );
            }
        }
        for (i, a) in self.artifacts.iter().enumerate() {
            let valid_owner = |o: &Owner| match o {
                Owner::Person(id) => (*id as usize) < h.people.len(),
                Owner::Institution(id) => (*id as usize) < self.institutions.len(),
                Owner::Community(id) => (*id as usize) < h.sites.len(),
            };
            ensure!(
                a.id as usize == i
                    && valid_owner(&a.owner)
                    && a.claims.iter().all(valid_owner)
                    && a.site.is_none_or(|id| (id as usize) < h.sites.len())
                    && a.custodian.is_none_or(|id| (id as usize) < h.people.len())
                    && a.materials
                        .iter()
                        .all(|&(id, m)| (id as usize) < crate::economy::GOODS
                            && m.is_finite()
                            && m >= 0.)
                    && a.events.iter().all(|&id| id < h.events.len() as u64)
                    && a.events.windows(2).all(|v| v[0] < v[1]),
                "invalid artifact provenance"
            );
        }
        Ok(())
    }
    pub fn held_goods(&self) -> [f64; crate::economy::GOODS] {
        let mut held = [0.; crate::economy::GOODS];
        for a in &self.artifacts {
            if !a.destroyed {
                for &(g, q) in &a.materials {
                    held[g as usize] += q as f64;
                }
            }
        }
        held
    }
    fn account(&mut self, h: &History, t: u32, author: Option<u32>, facts: Vec<u64>, text: String) {
        let author = author
            .filter(|&id| h.people[id as usize].died.is_none())
            .or_else(|| {
                self.site_people(h, self.traditions[t as usize].sacred_site)
                    .first()
                    .copied()
            });
        let institution = self
            .institutions
            .iter()
            .find(|n| n.active && n.tradition == Some(t))
            .map(|n| n.id);
        if author.is_none() && institution.is_none() {
            return;
        }
        self.accounts.push(Account {
            id: self.accounts.len() as u32,
            tradition: t,
            author,
            institution,
            month: h.month,
            facts,
            text,
        });
    }
    fn sync(&mut self, h: &History) {
        while self.agents.len() < h.people.len() {
            let p = &h.people[self.agents.len()];
            let traits = std::array::from_fn(|k| unit(h.seed, p.id, 0, 200 + k as u32));
            let occupation = ["farmer", "navigator", "craftworker", "teacher"]
                [((traits[3] * 4.) as usize).min(3)]
            .into();
            self.agents.push(Agent {
                person: p.id,
                traits,
                skills: [0.1; 4],
                occupation,
                goal: "secure household livelihood".into(),
                // The declared baseline carries survival practices. Later-born people
                // acquire practices through teaching, institutions or readable objects.
                knowledge: if p.born <= self.started as i32 {
                    BTreeSet::from([0, 4, 6])
                } else {
                    BTreeSet::new()
                },
                relations: BTreeMap::new(),
                known_places: BTreeSet::new(),
                knowledge_sources: BTreeMap::new(),
                last_campaign: None,
                actions: 0,
            });
        }
        while self.site_faith.len() < h.sites.len() {
            let s = &h.sites[self.site_faith.len()];
            let faith = h
                .sites
                .iter()
                .take(s.id as usize)
                .find(|p| p.civilization == s.civilization)
                .map_or(0, |p| self.site_faith[p.id as usize]);
            self.site_faith.push(faith);
        }
        if let Some(s) = &h.society {
            while self.household_faith.len() < s.households.len() {
                let hh = &s.households[self.household_faith.len()];
                let t = hh
                    .parent
                    .and_then(|p| self.household_faith.get(p as usize))
                    .copied()
                    .unwrap_or(self.site_faith[hh.site as usize]);
                self.household_faith.push(t);
            }
        }
    }
    // Explicit optional references keep factual event links visible at each call site.
    #[allow(clippy::too_many_arguments)]
    fn log(
        &self,
        h: &mut History,
        kind: &str,
        site: u32,
        person: Option<u32>,
        tradition: Option<u32>,
        artifact: Option<u32>,
        cause: Option<u64>,
        text: String,
    ) -> u64 {
        h.event(kind, Some(site), None, text);
        let e = h.events.last_mut().unwrap();
        e.subjects = person
            .map(|id| ("person".into(), id))
            .into_iter()
            .chain(tradition.map(|id| ("tradition".into(), id)))
            .chain(artifact.map(|id| ("artifact".into(), id)))
            .collect();
        if let Some(patron) = tradition
            .and_then(|id| self.traditions.get(id as usize))
            .and_then(|t| t.patron)
        {
            e.subjects.push(("patron".into(), patron));
        }
        if let Some(Owner::Institution(id)) = artifact
            .and_then(|id| self.artifacts.get(id as usize))
            .map(|a| &a.owner)
        {
            e.subjects.push(("institution".into(), *id));
        }
        if let Some(c) = cause {
            if c < e.id {
                e.causes.push(c);
            }
        }
        e.id
    }
}
impl History {
    pub(crate) fn initialize_culture(
        &mut self,
        cells: &[Cell],
        options: FoundingOptions,
        catalog: PatronCatalog,
        surveyed_coasts: Option<BTreeMap<u32, Vec<u32>>>,
    ) -> Result<()> {
        options.validate()?;
        catalog.validate()?;
        let mut c = Culture::empty(self.month, false, options)?;
        c.catalog = catalog;
        let n = self.terrain_resolution;
        let island_coasts = if let Some(coasts) = surveyed_coasts {
            coasts
        } else {
            let mut island_coasts: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
            // Connectivity labels use the same first-cell ID as civilization founding.
            let mut visited = vec![false; cells.len()];
            for start in 0..cells.len() {
                if visited[start] || cells[start].meta[0] != 2 {
                    continue;
                }
                let mut queue = std::collections::VecDeque::from([start as u32]);
                visited[start] = true;
                let mut coast = vec![];
                while let Some(id) = queue.pop_front() {
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let j = grid::neighbor(id, n, dx, dy) as usize;
                        if cells[j].meta[0] == 1
                            && cells[j].water[0] > 0.25
                            && cells[id as usize].water[0] < 0.25
                        {
                            coast.push(id);
                        }
                        if cells[j].meta[0] == 2 && !visited[j] {
                            visited[j] = true;
                            queue.push_back(j as u32);
                        }
                    }
                }
                coast.sort_unstable();
                coast.dedup();
                island_coasts.insert(start as u32, coast);
            }
            island_coasts
        };
        let outer: Vec<u32> = cells
            .iter()
            .enumerate()
            .filter(|(_, v)| v.meta[0] == 3 && v.water[0] < 0.25)
            .map(|(i, _)| i as u32)
            .collect();
        ensure!(!outer.is_empty(), "no ancient-continent patron origin");
        for i in 0..self.civilizations.len() {
            let site = &self.sites[i];
            let coasts = island_coasts
                .get(&site.island)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    anyhow::anyhow!("no accessible island landing for founding group {i}")
                })?;
            let landing = *coasts
                .iter()
                .min_by(|&&a, &&b| {
                    crate::civilization::distance(a, site.cell, n)
                        .total_cmp(&crate::civilization::distance(b, site.cell, n))
                })
                .unwrap();
            let ai = (unit(self.seed, i as u32, 0, 100) * c.catalog.patrons.len() as f32) as usize;
            let a = &c.catalog.patrons[ai];
            let origin = *outer
                .iter()
                .min_by(|&&a_id, &&b_id| {
                    let score = |id: u32| {
                        let v = &cells[id as usize];
                        let suitability = match a.habitat.as_str() {
                            "volcanic" => v.geology[0].abs().min(1.),
                            "forest" => v.life[0],
                            "wetland" => (v.climate[1] / 2000.).clamp(0., 1.),
                            _ => 1. - (v.terrain[0] / 4000.).clamp(0., 1.),
                        };
                        crate::civilization::distance(id, landing, n) - suitability * 0.15
                    };
                    score(a_id).total_cmp(&score(b_id))
                })
                .unwrap();
            let leader = self.civilizations[i].leader;
            let variance = c.options.variance[0]
                + unit(self.seed, i as u32, 0, 101)
                    * (c.options.variance[1] - c.options.variance[0]);
            let duration = c
                .options
                .duration(a.kind == "intelligent", a.aid_strength, variance);
            let patron_name = format!(
                "{} the {}",
                self.civilizations[i].naming(self.seed).coin(
                    &format!("patron:{i}"),
                    &["guide", "journey"],
                    None
                ),
                a.name
            );
            let (population, food, tools) = (
                site.stocks.stock[0],
                site.stocks.stock[1],
                site.economy.goods[3],
            );
            let sid = site.id;
            self.event("patron_arrival",Some(sid),None,format!("{patron_name} guided {population:0.0} people from ancient region {origin} to island landing {landing}; voyage recorded as prologue. The guide's mandate is unknown."));
            let ev = self.events.last().unwrap().id;
            self.events.last_mut().unwrap().subjects = vec![
                ("patron".into(), i as u32),
                ("person".into(), leader),
                ("tradition".into(), i as u32),
            ];
            let mut themes =
                std::array::from_fn(|k| (unit(self.seed, i as u32, 0, 110 + k as u32) * 8.) as u32);
            if unit(self.seed, i as u32, 0, 115) < 0.35 {
                themes[3] = a.theme;
            }
            c.traditions.push(Tradition {
                id: i as u32,
                name: self.civilizations[i].naming(self.seed).coin(
                    &format!("tradition:{i}"),
                    &["memory"],
                    Some(crate::naming::Source {
                        kind: "patron".into(),
                        id: i as u32,
                        name: patron_name.clone(),
                    }),
                ),
                patron: Some(i as u32),
                parent: None,
                themes,
                founded: self.month,
                sacred_site: sid,
                dissent: 0.,
                leader,
            });
            c.site_faith.push(i as u32);
            let provisions = duration as f64 * 2.;
            c.patrons.push(Patron {
                id: i as u32,
                archetype: ai as u32,
                name: patron_name.clone(),
                civilization: i as u32,
                site: sid,
                origin,
                landing,
                voyage: vec![origin, landing, self.sites[i].cell],
                witnesses: vec![leader],
                arrival_event: ev,
                departure_month: duration,
                departed: None,
                effort: [0.; 4],
                provisions_kg: provisions,
                consumed_kg: 0.,
                returned_kg: 0.,
                imported_kg: provisions,
                initial_people: population,
                initial_food: food,
                initial_tools: tools,
            });
            c.account(self,i as u32,Some(leader),vec![ev],format!("The founding witnesses remember {patron_name} as a guardian; its mandate is interpreted through {} and {}.",THEMES[themes[0]as usize],THEMES[themes[1]as usize]));
            // A finite ceramic keepsake is explicitly imported, never duplicated in town stores.
            self.sites[i].economy.initial[7] += 1.;
            let ae=c.log(self,"founding_keepsake",sid,Some(leader),Some(i as u32),Some(c.artifacts.len()as u32),Some(ev),"A ceramic voyage token was entrusted to the founding community; one kg declared arrival import".into());
            c.artifacts.push(Artifact {
                id: c.artifacts.len() as u32,
                name: self.civilizations[i].naming(self.seed).coin(
                    &format!("artifact:{}", c.artifacts.len()),
                    &["gift", "memory"],
                    Some(crate::naming::Source {
                        kind: "patron".into(),
                        id: i as u32,
                        name: patron_name.clone(),
                    }),
                ),
                kind: "founding keepsake".into(),
                creator: None,
                owner: Owner::Community(sid),
                claims: vec![],
                site: Some(sid),
                custodian: Some(leader),
                materials: vec![(7, 1.)],
                topic: None,
                tradition: Some(i as u32),
                events: vec![ae],
                destroyed: false,
                lost: false,
            });
        }
        c.sync(self);
        self.culture = Some(c);
        Ok(())
    }
    pub(crate) fn patron_aid_month(&mut self) {
        let Some(mut c) = self.culture.take() else {
            return;
        };
        c.sync(self);
        for pi in 0..c.patrons.len() {
            if c.patrons[pi].departed.is_some() || self.month > c.patrons[pi].departure_month {
                continue;
            }
            let witness = c.site_people(self, c.patrons[pi].site).first().copied();
            let p = &mut c.patrons[pi];
            let eaten = p.provisions_kg.min(2.);
            p.provisions_kg -= eaten;
            p.consumed_kg += eaten;
            if !c.options.aid_enabled || self.sites[p.site as usize].abandoned {
                continue;
            }
            let a = &c.catalog.patrons[p.archetype as usize];
            let effort = 0.25 + a.aid_strength;
            for k in 0..4 {
                p.effort[k] += effort * 0.25;
            }
            // Teaching aids existing people, with no production or nutrient multiplier.
            let s = &mut self.sites[p.site as usize];
            let bricks = (effort * 0.25 * 4.)
                .min(s.economy.goods[4] / 2.)
                .min(s.economy.goods[6] / 0.2);
            s.economy.goods[4] -= bricks * 2.;
            s.economy.used[4] += bricks * 2.;
            s.economy.goods[6] -= bricks * 0.2;
            s.economy.used[6] += bricks * 0.2;
            s.economy.external[0] -= bricks * 0.2;
            s.economy.goods[5] += bricks * 2.;
            s.economy.made[5] += bricks * 2.;
            s.economy.reserves[3] += bricks * 0.2;
            let Some(person) = witness.map(|p| p as usize) else {
                continue;
            };
            c.agents[person].skills[3] = (c.agents[person].skills[3] + effort * 0.005).min(1.);
            if self.month % 6 == 0 {
                let site_cell = self.sites[p.site as usize].cell;
                let offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                let (dx, dy) = offsets[(self.month as usize / 6) % 4];
                let surveyed = grid::neighbor(site_cell, self.terrain_resolution, dx, dy);
                c.agents[person].known_places.insert(surveyed);
                c.agents[person].skills[1] = (c.agents[person].skills[1] + effort * 0.01).min(1.);
                let learned = c.agents[person].knowledge.insert(a.teaching);
                if learned {
                    let (site, cause, topic) = (p.site, p.arrival_event, a.teaching);
                    c.log(
                        self,
                        "patron_teaching",
                        site,
                        Some(person as u32),
                        Some(pi as u32),
                        None,
                        Some(cause),
                        format!(
                            "The patron demonstrated {}; the witness retained the practice",
                            TOPICS[topic as usize]
                        ),
                    );
                    c.agents[person]
                        .knowledge_sources
                        .insert(topic, self.events.last().unwrap().id);
                }
            }
        }
        self.culture = Some(c);
    }
    pub(crate) fn culture_month(&mut self) {
        let Some(mut c) = self.culture.take() else {
            return;
        };
        c.sync(self);
        for pi in 0..c.patrons.len() {
            let p = &mut c.patrons[pi];
            if p.departed.is_some() {
                continue;
            }
            if self.month >= p.departure_month {
                p.departed = Some(p.departure_month);
                p.returned_kg += p.provisions_kg;
                p.provisions_kg = 0.;
                let (site, name, cause) = (p.site, p.name.clone(), p.arrival_event);
                let ev=c.log(self,"patron_departure",site,None,Some(pi as u32),None,Some(cause),format!("{name} returned to the ancient continent; the community must continue without its guide"));
                c.account(
                    self,
                    pi as u32,
                    None,
                    vec![ev],
                    format!(
                        "The patron's return to the ancient continent is commemorated as a charge to practice {}.",
                        THEMES[c.traditions[pi].themes[0] as usize]
                    ),
                );
            }
        }
        c.commemorations(self);
        c.abandon_objects(self);
        // Object custody and ownership survive generations without resurrecting dead holders.
        for ai in 0..c.artifacts.len() {
            let a = &c.artifacts[ai];
            if a.destroyed || a.lost {
                continue;
            }
            let dead = a
                .custodian
                .filter(|&id| self.people[id as usize].died.is_some());
            if let Some(dead) = dead {
                let site = a.site.unwrap_or(0);
                let heir = self
                    .society
                    .as_ref()
                    .and_then(|s| {
                        s.households
                            .iter()
                            .find(|hh| {
                                hh.site == site && self.people[hh.head as usize].died.is_none()
                            })
                            .map(|hh| hh.head)
                    })
                    .unwrap_or(
                        self.civilizations[self.sites[site as usize].civilization as usize].leader,
                    );
                let ev = c.log(
                    self,
                    "artifact_inherited",
                    site,
                    Some(heir),
                    a.tradition,
                    Some(ai as u32),
                    a.events.last().copied(),
                    format!(
                        "Custody passed from {} to {}",
                        self.people[dead as usize].name, self.people[heir as usize].name
                    ),
                );
                let a = &mut c.artifacts[ai];
                a.custodian = Some(heir);
                if a.owner == Owner::Person(dead) {
                    a.owner = Owner::Person(heir);
                }
                a.events.push(ev);
            }
        }
        c.institutional_succession(self);
        if self.month % 3 == 0 {
            c.maintain_institutions(self);
            crate::expedition_heritage::study(self, &mut c);
            crate::civic_petitions::propose(self, &mut c);
            c.decisions(self);
        }
        if self.month % 12 == 0 {
            c.year(self);
        }
        self.culture = Some(c);
    }
    pub fn patron_protection(&self, site: u32) -> f32 {
        self.culture.as_ref().map_or(0., |c| {
            c.patrons
                .iter()
                .find(|p| {
                    p.site == site
                        && p.departed.is_none()
                        && self.month <= p.departure_month
                        && c.options.aid_enabled
                })
                .map_or(0., |p| {
                    (0.25 + c.catalog.patrons[p.archetype as usize].aid_strength) * 0.1
                })
        })
    }
}
impl Culture {
    pub(crate) fn site_people(&self, h: &History, site: u32) -> Vec<u32> {
        let mut people = vec![];
        if h.sites[site as usize].abandoned {
            return people;
        }
        if let Some(s) = &h.society {
            people.extend(
                s.households
                    .iter()
                    .filter(|hh| hh.site == site && !s.relocation.away(hh.id))
                    .map(|hh| hh.head),
            );
        }
        if h.society.is_none()
            && h.sites
                .iter()
                .find(|s| s.civilization == h.sites[site as usize].civilization)
                .is_some_and(|s| s.id == site)
        {
            people.push(h.civilizations[h.sites[site as usize].civilization as usize].leader);
        }
        people.sort_unstable();
        people.dedup();
        people.retain(|&p| {
            h.people[p as usize].died.is_none() && h.month as i32 - h.people[p as usize].born >= 180
        });
        people
    }
    /// Religious affiliation of a locally present adult representative, distinct
    /// from the town's majority faith and political administration.
    pub fn resident_tradition(&self, h: &History, site: u32, actor: u32) -> Option<u32> {
        if site as usize >= h.sites.len() || !self.site_people(h, site).contains(&actor) {
            return None;
        }
        if let Some(society) = &h.society {
            let household = society.households.iter().find(|hh| {
                hh.site == site && hh.head == actor && !society.relocation.away(hh.id)
            })?;
            self.household_faith.get(household.id as usize).copied()
        } else {
            self.site_faith.get(site as usize).copied()
        }
    }
    /// Living adult local representatives who can practice each topic; not a workforce count.
    pub fn knowledge_holders(&self, h: &History, site: u32) -> [u32; TOPICS.len()] {
        let mut counts = [0; TOPICS.len()];
        if site as usize >= h.sites.len() {
            return counts;
        }
        for person in self.site_people(h, site) {
            if let Some(agent) = self.agents.get(person as usize) {
                for &topic in &agent.knowledge {
                    if let Some(count) = counts.get_mut(topic as usize) {
                        *count += 1;
                    }
                }
            }
        }
        counts
    }
    /// Practical recipe access held by living, adult, locally present representatives.
    /// Institutional topic lists and unread documents are not trained workers.
    pub fn available_knowledge(&self, h: &History, site: u32) -> u32 {
        self.knowledge_holders(h, site)
            .iter()
            .enumerate()
            .fold(
                0,
                |mask, (topic, &count)| if count > 0 { mask | (1 << topic) } else { mask },
            )
    }
    fn succession_lesson(&self, h: &History, site: u32, actor: u32) -> Option<(u32, u32, u32)> {
        let people = self.site_people(h, site);
        if !people.contains(&actor) {
            return None;
        }
        let holders = self.knowledge_holders(h, site);
        let topic = self.agents[actor as usize]
            .knowledge
            .iter()
            .copied()
            .filter(|topic| {
                people
                    .iter()
                    .any(|&p| p != actor && !self.agents[p as usize].knowledge.contains(topic))
            })
            .min_by_key(|&topic| (holders[topic as usize], topic))?;
        let student = people
            .into_iter()
            .filter(|&p| p != actor && !self.agents[p as usize].knowledge.contains(&topic))
            .min_by(|&a, &b| {
                self.agents[a as usize]
                    .knowledge
                    .len()
                    .cmp(&self.agents[b as usize].knowledge.len())
                    .then_with(|| {
                        unit(h.seed, a, h.month, 992).total_cmp(&unit(h.seed, b, h.month, 992))
                    })
                    .then(a.cmp(&b))
            })?;
        Some((student, topic, holders[topic as usize]))
    }
    fn institutional_lesson(&self, h: &History, site: u32, actor: u32) -> Option<(u32, u32, u32)> {
        let present = self.site_people(h, site);
        self.institutions
            .iter()
            .filter(|n| n.operational() && n.site == site && n.members.contains(&actor))
            .find_map(|n| {
                present
                    .iter()
                    .copied()
                    .filter(|p| *p != actor && n.members.contains(p))
                    .find_map(|teacher| {
                        self.agents[teacher as usize]
                            .knowledge
                            .intersection(&n.knowledge)
                            .find(|topic| !self.agents[actor as usize].knowledge.contains(topic))
                            .map(|&topic| (topic, teacher, n.id))
                    })
            })
    }
    fn living_interpreter(&self, h: &History, tradition: u32) -> Option<(u32, u32)> {
        // Preserve diaspora traditions even when the original sacred site is ruined.
        for site in &h.sites {
            if site.abandoned {
                continue;
            }
            let people = self.site_people(h, site.id);
            if let Some(society) = &h.society {
                if let Some(hh) = society.households.iter().find(|hh| {
                    hh.site == site.id
                        && self.household_faith[hh.id as usize] == tradition
                        && people.contains(&hh.head)
                }) {
                    return Some((site.id, hh.head));
                }
            } else if self.site_faith[site.id as usize] == tradition {
                if let Some(&person) = people.first() {
                    return Some((site.id, person));
                }
            }
        }
        None
    }
    fn decisions(&mut self, h: &mut History) {
        let recovered_sites = self.process_local_recoveries(h);
        for si in 0..h.sites.len() {
            if h.sites[si].abandoned || recovered_sites.contains(&(si as u32)) {
                continue;
            }
            let site = si as u32;
            let people = self.site_people(h, site);
            if people.is_empty() {
                continue;
            }
            let actor = people[((h.month / 3 + site) as usize) % people.len()];
            let traits = self.agents[actor as usize].traits;
            let Some(faith) = self.resident_tradition(h, site, actor) else {
                continue;
            };
            let labor = self.labor_budget.get(si).copied().unwrap_or(0.);
            if labor < 0.1 {
                continue;
            }
            if traits[2] > 0.7
                && unit(h.seed, actor, h.month, 990) < 0.12
                && self.pilgrimage(h, site, actor, labor)
            {
                // Travel work is charged by the successful pilgrimage itself.
                continue;
            }
            if self.recover_object(h, site, actor) {
                self.labor_spent += 0.1;
                continue;
            }
            if traits[3] > 0.6 && self.curate_specimen(h, site, actor) {
                self.labor_spent += 0.1;
                continue;
            }
            let mut remaining_work = labor;
            // Reading and institutional instruction use this quarter's reserved work.
            // A book's physical survival matters: destroyed or remote objects cannot teach.
            let readable = self.artifacts.iter().find_map(|a| {
                a.topic
                    .filter(|topic| {
                        !a.destroyed
                            && !a.lost
                            && a.site == Some(site)
                            && !self.agents[actor as usize].knowledge.contains(topic)
                    })
                    .map(|topic| (topic, Some(a.id), a.events.last().copied()))
            });
            let institution_lesson = if readable.is_none() {
                self.institutional_lesson(h, site, actor)
            } else {
                None
            };
            let lesson = readable.or_else(|| {
                institution_lesson.map(|(topic, teacher, _)| {
                    (
                        topic,
                        None,
                        self.agents[teacher as usize]
                            .knowledge_sources
                            .get(&topic)
                            .copied(),
                    )
                })
            });
            if let Some((topic, object, cause)) = lesson.filter(|_| remaining_work >= 0.1) {
                remaining_work -= 0.1;
                self.agents[actor as usize].knowledge.insert(topic);
                let channel = institution_lesson.map_or_else(
                    || "an accessible document".to_string(),
                    |(_, teacher, institution)| {
                        format!(
                            "instruction from {} at {}",
                            h.people[teacher as usize].name,
                            self.institutions[institution as usize].name
                        )
                    },
                );
                self.log(
                    h,
                    "knowledge_studied",
                    site,
                    Some(actor),
                    None,
                    object,
                    cause,
                    format!(
                        "{} learned {} through {channel}, using reserved study work",
                        h.people[actor as usize].name, TOPICS[topic as usize]
                    ),
                );
                let event = h.events.last_mut().unwrap();
                if let Some((_, teacher, institution)) = institution_lesson {
                    event.subjects.push(("person".into(), teacher));
                    event.subjects.push(("institution".into(), institution));
                }
                self.agents[actor as usize]
                    .knowledge_sources
                    .insert(topic, event.id);
            }
            if let Some((student, topic, holders)) = self
                .succession_lesson(h, site, actor)
                .filter(|_| remaining_work >= 0.1)
            {
                remaining_work -= 0.1;
                self.agents[student as usize].knowledge.insert(topic);
                self.agents[actor as usize].relations.insert(student, 0.5);
                self.agents[student as usize].relations.insert(actor, 0.5);
                self.agents[actor as usize].goal = "teach a successor".into();
                self.log(
                        h,
                        "practice_taught",
                        site,
                        Some(actor),
                        None,
                        None,
                        None,
                        format!(
                            "{} taught {} to {}; {} local adult holder(s) before instruction, prioritizing scarce practical knowledge",
                            h.people[actor as usize].name,
                            TOPICS[topic as usize],
                            h.people[student as usize].name,
                            holders
                        ),
                    );
                let event = h.events.last_mut().unwrap();
                event.subjects.push(("person".into(), student));
                if let Some(&cause) = self.agents[actor as usize].knowledge_sources.get(&topic) {
                    event.causes.push(cause);
                }
                self.agents[student as usize]
                    .knowledge_sources
                    .insert(topic, event.id);
            }
            if remaining_work >= 0.1
                && traits[0] > 0.75
                && unit(h.seed, actor, h.month, 993) < 0.08
                && self.seek_office(h, site, actor)
            {
                self.labor_spent += (labor - remaining_work + 0.1) as f64;
                continue;
            }
            // Small donations are transfers, not extra community income.
            for ni in 0..self.institutions.len() {
                let inst = &mut self.institutions[ni];
                if inst.site != site
                    || !inst.active
                    || remaining_work < 0.05
                    || h.sites[si].economy.finance[0] <= 0.
                {
                    continue;
                }
                remaining_work -= 0.05;
                let donation = (h.sites[si].economy.finance[0] as f64 * 0.0005).min(2.);
                h.sites[si].economy.finance[0] -= donation as f32;
                inst.treasury += donation;
                inst.dues += donation;
                let fee = if inst.capacity.is_none() {
                    inst.treasury.min(0.5)
                } else {
                    0.
                };
                inst.treasury -= fee;
                inst.expenses += fee;
                h.sites[si].economy.finance[0] += fee as f32;
                inst.knowledge
                    .extend(self.agents[actor as usize].knowledge.iter().copied());
            }
            let kind = if traits[2] > 0.65 {
                InstitutionKind::Religious
            } else if traits[3] > 0.6 {
                InstitutionKind::Scholarly
            } else if traits[0] > 0.5 {
                InstitutionKind::Merchant
            } else {
                InstitutionKind::Craft
            };
            let members: Vec<_> = people
                .iter()
                .copied()
                .filter(|&p| {
                    kind != InstitutionKind::Religious
                        || self.resident_tradition(h, site, p) == Some(faith)
                })
                .collect();
            let room = h.economy_catalog.as_ref().and_then(|catalog| {
                crate::facilities::choose(
                    catalog,
                    &h.sites[si].economy,
                    crate::facilities::demand(&kind, members.len()),
                    (h.sites[si].economy.finance[0] as f64 * 0.15).max(0.),
                )
            });
            if remaining_work >= 0.2
                && members.len() >= 2
                && h.sites[si].economy.finance[0] > 500.
                && (room.is_some()
                    || (h
                        .economy_catalog
                        .as_ref()
                        .is_some_and(|c| c.materials.is_none())
                        && h.sites[si].economy.goods[5]
                            >= crate::institution_capacity::HALL_BRICKS_KG))
                && !self.institutions.iter().any(|n| {
                    n.site == site
                        && n.kind == kind
                        && n.active
                        && (kind != InstitutionKind::Religious || n.tradition == Some(faith))
                })
            {
                remaining_work -= 0.2;
                h.sites[si].economy.finance[0] -= 25.;
                let facility = room
                    .map(|room| crate::facilities::Facility::found(room, &mut h.sites[si].economy));
                if facility.is_none() {
                    h.sites[si].economy.goods[5] -= crate::institution_capacity::HALL_BRICKS_KG;
                }
                let materials = facility.as_ref().map_or(
                    vec![(5, crate::institution_capacity::HALL_BRICKS_KG)],
                    |f| f.embodied(),
                );

                let id = self.institutions.len() as u32;
                self.institutions.push(Institution {
                    capacity: Some(crate::institution_capacity::Capacity::new(h.month)),
                    id,
                    name: {
                        let purpose = match kind {
                            InstitutionKind::Religious => "sanctuary",
                            InstitutionKind::Merchant => "market",
                            InstitutionKind::Craft => "craft",
                            InstitutionKind::Scholarly => "learning",
                        };
                        let source = if kind == InstitutionKind::Religious {
                            crate::naming::Source {
                                kind: "tradition".into(),
                                id: faith,
                                name: self.traditions[faith as usize].name.clone(),
                            }
                        } else {
                            crate::naming::Source {
                                kind: "person".into(),
                                id: actor,
                                name: h.people[actor as usize].name.clone(),
                            }
                        };
                        let mut meanings = vec![purpose];
                        if kind == InstitutionKind::Craft {
                            // A material epithet requires actual local output, not a hidden deposit.
                            let e = &h.sites[si].economy;
                            if let Some((_, term)) = [(0usize, "timber"), (2, "metal"), (5, "clay")]
                                .into_iter()
                                .filter(|(g, _)| e.made[*g] > 1.)
                                .max_by(|(a, _), (b, _)| e.made[*a].total_cmp(&e.made[*b]))
                            {
                                meanings.insert(0, term);
                            }
                        }
                        h.civilizations[h.sites[si].civilization as usize]
                            .naming(h.seed)
                            .coin(&format!("institution:{id}"), &meanings, Some(source))
                    },
                    kind: kind.clone(),
                    site,
                    tradition: if kind == InstitutionKind::Religious {
                        Some(faith)
                    } else {
                        None
                    },
                    members,
                    leader: actor,
                    treasury: 25.,
                    active: true,
                    founded: h.month,
                    knowledge: self.agents[actor as usize].knowledge.clone(),
                    property: vec![],
                    dues: 25.,
                    expenses: 0.,
                });
                let artifact = self.artifacts.len() as u32;
                let material_summary = materials
                    .iter()
                    .map(|&(g, m)| {
                        format!(
                            "{m:.1} kg {}",
                            h.economy_catalog.as_ref().unwrap().goods[g as usize].name
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let event=self.log(h,"institution_founded",site,Some(actor),Some(faith),Some(artifact),None,format!("A {kind:?} institution formed with 25 money transferred and {material_summary} reserved in its meeting place; motive: shared practice"));
                h.events[event as usize]
                    .subjects
                    .push(("institution".into(), id));
                self.artifacts.push(Artifact {
                    id: artifact,
                    name: format!("{} meeting place", self.institutions[id as usize].name),
                    kind: "institutional foundation".into(),
                    creator: Some(actor),
                    owner: Owner::Institution(id),
                    claims: vec![],
                    site: Some(site),
                    custodian: None,
                    materials,
                    topic: None,
                    tradition: Some(faith),
                    events: vec![event],
                    destroyed: false,
                    lost: false,
                });
                self.institutions[id as usize].property.push(artifact);
                self.institutions[id as usize]
                    .capacity
                    .as_mut()
                    .unwrap()
                    .building = Some(facility.map_or_else(
                    || crate::institution_capacity::MeetingPlace::hall(artifact),
                    |f| crate::institution_capacity::MeetingPlace::facility(artifact, f),
                ));
            }
            if remaining_work >= 0.05 && traits[1] > 0.6 {
                if let Some(dest) = h.society.as_ref().and_then(|s| {
                    s.routes
                        .iter()
                        .filter(|r| r.passable() && (r.from == site || r.to == site))
                        .map(|r| if r.from == site { r.to } else { r.from })
                        .find(|&t| {
                            if s.relocation.witnessed_relief {
                                s.relocation.appeals.iter().any(|a| {
                                    a.host == site && a.origin == t && h.month <= a.reported + 18
                                })
                            } else {
                                h.sites[t as usize].stocks.stock[3] > 0.01
                            }
                        })
                }) {
                    remaining_work -= 0.05;
                    let gift = h.sites[si].economy.finance[0].min(3.);
                    h.sites[si].economy.finance[0] -= gift;
                    h.sites[dest as usize].economy.finance[0] += gift;
                    self.agents[actor as usize].goal = "aid a hungry neighbor".into();
                    self.log(
                        h,
                        "charitable_gift",
                        site,
                        Some(actor),
                        Some(faith),
                        None,
                        None,
                        format!(
                            "Sent {gift:0.1} money to {} for relief through an open route",
                            h.sites[dest as usize].name
                        ),
                    );
                }
            }
            let make = remaining_work >= 0.2
                && h.month / 3 % 4 == site % 4
                && h.sites[si].economy.goods[7] >= 1.
                && self
                    .artifacts
                    .iter()
                    .filter(|a| a.site == Some(site) && !a.destroyed)
                    .count()
                    < 16;
            if make {
                remaining_work -= 0.2;
                let manuscript = traits[3] > 0.6
                    && self.agents[actor as usize].knowledge.contains(&10)
                    && h.sites[si].economy.goods[21] >= 0.2;
                let topic = if manuscript {
                    self.agents[actor as usize]
                        .knowledge
                        .iter()
                        .copied()
                        .filter(|&t| t != 10)
                        .min_by_key(|&t| {
                            self.artifacts
                                .iter()
                                .filter(|a| {
                                    !a.destroyed
                                        && !a.lost
                                        && a.site == Some(site)
                                        && a.topic == Some(t)
                                })
                                .count()
                        })
                } else {
                    None
                };
                let kind = if manuscript {
                    "inscribed manuscript"
                } else if self.agents[actor as usize].skills[3] > 0.35
                    && unit(h.seed, actor, h.month, 994) < 0.15
                {
                    "crafted masterpiece"
                } else {
                    "dedicated craftsmanship"
                };
                let material = if manuscript { (21, 0.2) } else { (7, 1.) };
                h.sites[si].economy.goods[material.0] -= material.1;
                let id = self.artifacts.len() as u32;
                let ev = self.log(
                    h,
                    "artifact_created",
                    site,
                    Some(actor),
                    Some(faith),
                    Some(id),
                    topic.and_then(|t| {
                        self.agents[actor as usize]
                            .knowledge_sources
                            .get(&t)
                            .copied()
                    }),
                    format!(
                        "{} created {kind} using {:.2} kg of catalog material {}",
                        h.people[actor as usize].name, material.1, material.0
                    ),
                );
                self.agents[actor as usize].skills[3] =
                    (self.agents[actor as usize].skills[3] + 0.02).min(1.);
                self.agents[actor as usize].goal = "leave a useful crafted legacy".into();
                self.artifacts.push(Artifact {
                    id,
                    name: h.civilizations[h.sites[si].civilization as usize]
                        .naming(h.seed)
                        .coin(
                            &format!("artifact:{id}"),
                            &[if manuscript { "book" } else { "gift" }],
                            Some(crate::naming::Source {
                                kind: "person".into(),
                                id: actor,
                                name: h.people[actor as usize].name.clone(),
                            }),
                        ),
                    kind: kind.into(),
                    creator: Some(actor),
                    owner: Owner::Person(actor),
                    claims: vec![],
                    site: Some(site),
                    custodian: Some(actor),
                    materials: vec![(material.0 as u32, material.1)],
                    topic,
                    tradition: Some(faith),
                    events: vec![ev],
                    destroyed: false,
                    lost: false,
                });
            }
            if remaining_work >= 0.1
                && traits[0] > 0.8
                && traits[4] < 0.3
                && unit(h.seed, actor, h.month, 600) < 0.05
            {
                if let Some(id) = self.artifacts.iter().position(|a| {
                    !a.destroyed && !a.lost && a.site == Some(site) && a.custodian != Some(actor)
                }) {
                    let a = &self.artifacts[id];
                    let previous = a.owner.clone();
                    let ev=self.log(h,"artifact_theft",site,Some(actor),a.tradition,Some(id as u32),a.events.last().copied(),"An ambitious claimant took custody of a local object; the prior owner's claim persists".into());
                    let a = &mut self.artifacts[id];
                    if !a.claims.contains(&previous) {
                        a.claims.push(previous);
                    }
                    a.custodian = Some(actor);
                    a.events.push(ev);
                    self.agents[actor as usize].goal = "possess a prestigious object".into();
                    remaining_work -= 0.1;
                }
            }
            let used = (labor - remaining_work).max(0.);
            self.labor_spent += used as f64;
            if used <= 0. {
                continue;
            }
            self.agents[actor as usize].actions += 1;
            self.agents[actor as usize].skills[0] =
                (self.agents[actor as usize].skills[0] + 0.001).min(1.);
        }
    }
    fn commemorations(&mut self, h: &mut History) {
        // Departure anniversaries incur finite offering materials; no free festivals.
        for pi in 0..self.patrons.len() {
            if self.patrons[pi].departed.is_none()
                || h.month <= self.patrons[pi].departure_month
                || (h.month - self.patrons[pi].departure_month) % 12 != 0
            {
                continue;
            }
            let site = self.patrons[pi].site;
            let s = &mut h.sites[site as usize];
            if s.abandoned || s.economy.goods[7] < 0.1 {
                continue;
            }
            s.economy.goods[7] -= 0.1;
            s.economy.used[7] += 0.1;
            s.economy.reserves[3] += 0.1;
            if (h.month - self.patrons[pi].departure_month) % 120 == 0 {
                self.log(h,"founding_commemoration",site,None,Some(pi as u32),None,Some(self.patrons[pi].arrival_event),"The community renewed its departure remembrance with a finite ceramic offering".into());
            }
        }
    }
    fn year(&mut self, h: &mut History) {
        for i in 0..self.institutions.len() {
            let site = self.institutions[i].site;
            let people = self.site_people(h, site);
            let institution = &self.institutions[i];
            let recruits: Vec<_> = people
                .iter()
                .copied()
                .filter(|&p| {
                    institution.kind != InstitutionKind::Religious
                        || self.resident_tradition(h, site, p) == institution.tradition
                })
                .collect();
            let n = &mut self.institutions[i];
            n.members.retain(|&p| h.people[p as usize].died.is_none());
            for &person in &recruits {
                if n.members.len() < 24 && !n.members.contains(&person) {
                    n.members.push(person);
                }
            }
            if n.capacity.is_none() && h.people[n.leader as usize].died.is_some() {
                if let Some(&next) = n.members.first().or(people.first()) {
                    n.leader = next;
                }
            }
            if h.sites[site as usize].abandoned || people.is_empty() {
                n.active = false;
                h.sites[site as usize].economy.finance[0] += n.treasury as f32;
                n.expenses += n.treasury;
                n.treasury = 0.;
            }
        }
        for ti in 0..self.traditions.len() {
            let site = self.traditions[ti].sacred_site;
            let leader = self.traditions[ti].leader;
            if h.people[leader as usize].died.is_some() {
                self.traditions[ti].leader = self.site_people(h, site).first().copied().unwrap_or(
                    h.civilizations[h.sites[site as usize].civilization as usize].leader,
                );
            }
        }
        // Only open, physically represented routes establish contact.
        let mut routes = h
            .society
            .as_ref()
            .map(|s| {
                s.routes
                    .iter()
                    .filter(|r| r.passable())
                    .map(|r| (r.from, r.to))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        // Completed cargo journeys carry actual inter-island contact, including
        // contact within the same tradition. Reverse teaching uses the same route.
        let mut deliveries = BTreeMap::new();
        for e in h
            .events
            .iter()
            .rev()
            .take_while(|e| e.month > h.month.saturating_sub(12))
            .filter(|e| e.kind == "market_arrival")
        {
            if let (Some(from), Some(to)) = (e.other, e.site) {
                deliveries.entry((from, to)).or_insert(e.id);
            }
        }
        routes.extend(deliveries.keys().copied());
        let reverse = routes.iter().map(|&(a, b)| (b, a)).collect::<Vec<_>>();
        routes.extend(reverse);
        routes.sort_unstable();
        routes.dedup();
        let religious_routes = routes.clone();
        let mut counted = BTreeSet::new();
        for (from, to) in routes {
            if self.site_people(h, from).is_empty() || self.site_people(h, to).is_empty() {
                continue;
            }
            let a = self.site_faith[from as usize];
            let b = self.site_faith[to as usize];
            let key = format!("{}:{}", a.min(b), a.max(b));
            let first_contact = counted.insert(key.clone());
            let years = self.contact.entry(key).or_default();
            if first_contact {
                *years += 1;
            }
            if a != b && *years < 5 {
                continue;
            }
            let teachers = self.site_people(h, from);
            let students = self.site_people(h, to);
            if let (Some(&teacher), Some(&student)) = (teachers.first(), students.first()) {
                if let Some(topic) = self.agents[teacher as usize]
                    .knowledge
                    .difference(&self.agents[student as usize].knowledge)
                    .next()
                    .copied()
                {
                    self.agents[student as usize].knowledge.insert(topic);
                    for n in &mut self.institutions {
                        if n.site == from
                            && n.active
                            && n.kind == InstitutionKind::Scholarly
                            && n.members.len() < 24
                            && !n.members.contains(&student)
                        {
                            n.members.push(student);
                        }
                    }
                    self.log(
                        h,
                        "knowledge_contact",
                        to,
                        Some(student),
                        None,
                        None,
                        deliveries
                            .get(&(from, to))
                            .or_else(|| deliveries.get(&(to, from)))
                            .copied(),
                        format!(
                            "Learned {} through contact with {}",
                            TOPICS[topic as usize], h.sites[from as usize].name
                        ),
                    );
                    let event = h.events.last_mut().unwrap();
                    event.subjects.push(("person".into(), teacher));
                    if let Some(&cause) =
                        self.agents[teacher as usize].knowledge_sources.get(&topic)
                    {
                        event.causes.push(cause);
                    }
                    self.agents[student as usize]
                        .knowledge_sources
                        .insert(topic, event.id);
                }
            }
        }
        self.advance_religious_dynamics(h, &religious_routes);
        // Resident household counts determine plurality; ownership shares are wealth, not adherents.
        if let Some(s) = &h.society {
            for site in &h.sites {
                let mut votes = BTreeMap::<u32, f64>::new();
                for hh in s
                    .households
                    .iter()
                    .filter(|hh| hh.site == site.id && !s.relocation.away(hh.id))
                {
                    *votes
                        .entry(self.household_faith[hh.id as usize])
                        .or_default() += 1.;
                }
                if let Some((&faith, _)) = votes
                    .iter()
                    .max_by(|a, b| a.1.total_cmp(b.1).then_with(|| b.0.cmp(a.0)))
                {
                    self.site_faith[site.id as usize] = faith;
                }
            }
        }
        if h.month % 240 == 0 {
            for ti in 0..self.traditions.len() {
                let t = &self.traditions[ti];
                let sacred_site = t.sacred_site;
                let Some((site, leader)) = self.living_interpreter(h, ti as u32) else {
                    continue;
                };
                let cause = h
                    .events
                    .iter()
                    .rev()
                    .find(|e| {
                        (e.site == Some(site) || e.site == Some(sacred_site))
                            && matches!(
                                e.kind.as_str(),
                                "food_crisis" | "patron_departure" | "expedition_return"
                            )
                    })
                    .map(|e| e.id);
                if let Some(cause) = cause {
                    let ev = self.log(
                        h,
                        "religious_interpretation",
                        site,
                        Some(leader),
                        Some(ti as u32),
                        None,
                        Some(cause),
                        "A human interpreter related a remembered event to current communal duties"
                            .into(),
                    );
                    self.account(
                        h,
                        ti as u32,
                        Some(leader),
                        vec![cause, ev],
                        format!(
                            "{} teaches that the remembered experience calls for {}.",
                            h.people[leader as usize].name,
                            THEMES[self.traditions[ti].themes[0] as usize]
                        ),
                    );
                }
            }
        }
        self.update_roles(h);
        // Evidence is tied to returned voyages and an actual approach to the patron's origin.
        let evidence = h
            .expeditions
            .as_ref()
            .map(|x| {
                x.voyages
                    .iter()
                    .filter(|e| e.confirmed && !self.processed_voyages.contains(&e.id))
                    .map(|e| {
                        (
                            e.id,
                            e.origin,
                            *x.routes[e.route as usize].cells.last().unwrap(),
                            e.cause,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for (voyage, site, cell, cause) in evidence {
            self.processed_voyages.insert(voyage);
            for pi in 0..self.patrons.len() {
                if crate::civilization::distance(
                    cell,
                    self.patrons[pi].origin,
                    h.terrain_resolution,
                ) < 0.15
                {
                    let ev=self.log(h,"patron_origin_evidence",site,None,Some(pi as u32),None,Some(cause),format!("Voyage {voyage} returned observations from the homeland associated with {}; no reunion was witnessed",self.patrons[pi].name));
                    self.account(h,pi as u32,None,vec![ev],"Some interpreters see these observations as confirmation of the ancestral voyage; the mandate remains unknown.".into());
                }
            }
        }
    }
}
impl History {
    pub(crate) fn sync_culture(&mut self) {
        if let Some(mut c) = self.culture.take() {
            c.sync(self);
            self.culture = Some(c);
        }
    }
    pub(crate) fn reserve_cultural_work(&mut self) {
        self.sync_culture();
        let knowledge: Vec<u32> = self
            .culture
            .as_ref()
            .map(|c| {
                self.sites
                    .iter()
                    .map(|s| c.available_knowledge(self, s.id))
                    .collect()
            })
            .unwrap_or_default();
        let requests: Vec<f32> = self
            .sites
            .iter()
            .map(|s| {
                self.culture.as_ref().map_or(0., |c| {
                    c.work_requests(self, s.id)
                        .iter()
                        .map(|r| r.1)
                        .sum::<f32>()
                        .min(0.5)
                })
            })
            .collect();
        if let Some(c) = &mut self.culture {
            c.labor_budget = vec![0.; self.sites.len()];
            for (i, s) in self.sites.iter_mut().enumerate() {
                s.economy.management[3] = knowledge[i] as f32;
                if self.month % 3 == 0 && !s.abandoned {
                    let available =
                        crate::labor::available(s, self.society.is_some(), self.living.is_some());
                    let work = available.min(requests[i]);
                    c.labor_budget[i] = work;
                    s.economy.external[3] += work;
                }
            }
        }
    }
    pub(crate) fn release_cultural_work(&mut self) {
        if let Some(c) = &self.culture {
            for (i, s) in self.sites.iter_mut().enumerate() {
                s.economy.external[3] =
                    (s.economy.external[3] - c.labor_budget.get(i).copied().unwrap_or(0.)).max(0.);
            }
        }
    }
    /// Read-only requests at the current state; not a retrospective execution log.
    pub fn cultural_work_requests(&self) -> serde_json::Value {
        serde_json::json!(self.sites.iter().map(|s| {
            let requests = self.culture.as_ref().map(|c| c.work_requests(self, s.id)).unwrap_or_default();
            serde_json::json!({"site":s.id, "month":self.month,
                "requests":requests.iter().map(|(action,work)| serde_json::json!({"action":action,"worker_months":work})).collect::<Vec<_>>(),
                "requested_worker_months":requests.iter().map(|r|r.1).sum::<f32>().min(0.5)})
        }).collect::<Vec<_>>())
    }
    pub fn cultural_summary(&self) -> serde_json::Value {
        self.culture.as_ref().map_or(serde_json::Value::Null,|c|serde_json::json!({"religious_relief":c.religious_relief,"patrons":c.patrons.len(),"departed":c.patrons.iter().filter(|p|p.departed.is_some()).count(),"aid_effort":c.patrons.iter().map(|p|p.effort.iter().sum::<f32>()).sum::<f32>(),"traditions":c.traditions.len(),"accounts":c.accounts.len(),"institutions":c.institutions.iter().filter(|n|n.active).count(),"institution_capacity":c.institutions.iter().filter_map(|n|n.capacity.as_ref().map(|capacity|serde_json::json!({"id":n.id,"site":n.site,"kind":n.kind,"eligible_local_members":c.institution_candidates(self,n.id).len(),"active":n.active,"operational":n.operational(),"capacity":capacity}))).collect::<Vec<_>>(),"artifacts":c.artifacts.len(),"practical_knowledge":self.sites.iter().map(|s|serde_json::json!({"site":s.id,"topic_mask":c.available_knowledge(self,s.id)})).collect::<Vec<_>>(),"knowledge_links":c.agents.iter().map(|a|a.knowledge.len()).sum::<usize>(),"relationships":c.agents.iter().map(|a|a.relations.len()).sum::<usize>(),"actions":c.agents.iter().map(|a|a.actions as u64).sum::<u64>(),"labor":c.labor_spent,"pilgrimages":self.events.iter().filter(|e|e.kind=="pilgrimage_returned").count(),"office_campaigns":self.events.iter().filter(|e|e.kind=="office_campaign").count(),"specimens":c.artifacts.iter().filter(|a|a.kind=="expedition specimen").count(),"lost_objects":c.artifacts.iter().filter(|a|a.lost && !a.destroyed).count(),"knowledge_sources":c.agents.iter().map(|a|a.knowledge_sources.len()).sum::<usize>()}))
    }
}
impl Generator {
    pub fn found_civilizations_with_options(
        &mut self,
        count: u32,
        options: FoundingOptions,
    ) -> Result<()> {
        self.found_civilizations_with_catalog(count, options, PatronCatalog::bundled()?)
    }
    /// Validated editable patron content is archived with its founding records.
    pub fn found_civilizations_with_catalog(
        &mut self,
        count: u32,
        options: FoundingOptions,
        catalog: PatronCatalog,
    ) -> Result<()> {
        options.validate()?;
        catalog.validate()?;
        self.found_civilizations_base(count)?;
        let result = (|| {
            let cells = self.snapshot()?;
            let coasts = self
                .navigation_service()?
                .map(|n| n.island_coasts())
                .transpose()?;
            self.civilizations
                .as_mut()
                .unwrap()
                .initialize_culture(&cells, options, catalog, coasts)
        })();
        if result.is_err() {
            self.civilizations = None;
        }
        result
    }
    pub fn cultural_history(&self) -> Option<&Culture> {
        self.civilizations.as_ref()?.culture.as_ref()
    }
}

impl History {
    pub(crate) fn initialize_legacy_culture(&mut self) -> Result<()> {
        if self.culture.is_some() {
            return Ok(());
        }
        let mut c = Culture::empty(self.month, true, Default::default())?;
        for civilization in &self.civilizations {
            let site = self
                .sites
                .iter()
                .find(|s| s.civilization == civilization.id)
                .map_or(0, |s| s.id);
            c.traditions.push(Tradition {
                id: civilization.id,
                name: format!("{} inherited customs", civilization.name),
                patron: None,
                parent: None,
                themes: std::array::from_fn(|k| {
                    (unit(self.seed, civilization.id, 0, 110 + k as u32) * 8.) as u32
                }),
                founded: self.month,
                sacred_site: site,
                dissent: 0.,
                leader: civilization.leader,
            });
        }
        c.site_faith = self.sites.iter().map(|s| s.civilization).collect();
        c.sync(self);
        self.event("cultural_baseline",None,None,"Existing communities retain their history; founding patrons and witnessed arrivals are unknown. Cultural records begin at this boundary.".into());
        self.culture = Some(c);
        Ok(())
    }
}
impl Generator {
    /// Negotiate a local sale to an institution. Individual proceeds belong to the
    /// existing communal private-stock account; there is no second personal wallet.
    pub fn sell_artifact(&mut self, artifact: u32, buyer: u32, price: f64) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(price.is_finite() && price > 0., "invalid sale price");
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let c = h
            .culture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no cultural history"))?;
        let a = c
            .artifacts
            .get(artifact as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown artifact"))?;
        let n = c
            .institutions
            .get(buyer as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown buyer"))?;
        ensure!(
            !a.destroyed && !a.lost && a.site == Some(n.site) && n.active && n.treasury >= price,
            "sale requires local access and sufficient institutional funds"
        );
        ensure!(
            a.claims.is_empty() && a.owner != Owner::Institution(buyer),
            "object has disputed title or already belongs to buyer"
        );
        if let Owner::Person(owner) = a.owner {
            ensure!(
                a.custodian == Some(owner) && h.people[owner as usize].died.is_none(),
                "seller lacks custody"
            );
        }
        if let Owner::Institution(owner) = a.owner {
            ensure!(
                c.institutions[owner as usize].active,
                "seller institution is inactive"
            );
        }
        let mut c = h.culture.take().unwrap();
        let a = &c.artifacts[artifact as usize];
        let site = a.site.unwrap();
        let owner = a.owner.clone();
        let event = c.log(
            h,
            "artifact_sold",
            site,
            a.custodian,
            a.tradition,
            Some(artifact),
            a.events.last().copied(),
            format!(
                "{} bought the object for {price:.2}; ownership transferred by agreement",
                c.institutions[buyer as usize].name
            ),
        );
        h.events[event as usize]
            .subjects
            .push(("institution".into(), buyer));
        c.institutions[buyer as usize].treasury -= price;
        c.institutions[buyer as usize].expenses += price;
        match owner {
            Owner::Institution(id) => {
                c.institutions[id as usize].treasury += price;
                c.institutions[id as usize]
                    .property
                    .retain(|&id| id != artifact);
            }
            Owner::Community(id) => h.sites[id as usize].economy.finance[0] += price as f32,
            Owner::Person(_) => h.sites[site as usize].economy.finance[0] += price as f32,
        }
        c.institutions[buyer as usize].property.push(artifact);
        let a = &mut c.artifacts[artifact as usize];
        a.owner = Owner::Institution(buyer);
        a.custodian = None;
        a.events.push(event);
        h.culture = Some(c);
        Ok(())
    }
    /// Local ownership transfers keep physical custody separate from disputed title.
    pub fn dedicate_artifact(&mut self, artifact: u32, institution: u32) -> Result<()> {
        self.validate_living_boundary()?;
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let c = h
            .culture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no cultural history"))?;
        let a = c
            .artifacts
            .get(artifact as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown artifact"))?;
        let n = c
            .institutions
            .get(institution as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown institution"))?;
        ensure!(
            !a.destroyed && !a.lost && a.site == Some(n.site) && n.active,
            "dedication requires a living institution with access to the object"
        );
        let mut c = h.culture.take().unwrap();
        let a = &c.artifacts[artifact as usize];
        let site = a.site.unwrap();
        let ev = c.log(
            h,
            "artifact_dedicated",
            site,
            a.custodian,
            a.tradition,
            Some(artifact),
            a.events.last().copied(),
            format!(
                "Ownership dedicated to {}",
                c.institutions[institution as usize].name
            ),
        );
        c.artifacts[artifact as usize].owner = Owner::Institution(institution);
        c.artifacts[artifact as usize].events.push(ev);
        for n in &mut c.institutions {
            if n.id != institution {
                n.property.retain(|&id| id != artifact);
            }
        }
        if !c.institutions[institution as usize]
            .property
            .contains(&artifact)
        {
            c.institutions[institution as usize].property.push(artifact);
        }
        h.culture = Some(c);
        Ok(())
    }
    /// Physical destruction returns embodied matter to the local managed detritus/waste ledger.
    pub fn destroy_artifact(&mut self, artifact: u32) -> Result<()> {
        self.validate_living_boundary()?;
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let c = h
            .culture
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no cultural history"))?;
        let a = c
            .artifacts
            .get(artifact as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown artifact"))?;
        ensure!(
            !a.destroyed && a.site.is_some(),
            "artifact already destroyed or inaccessible"
        );
        let mut c = h.culture.take().unwrap();
        let a = &c.artifacts[artifact as usize];
        let site = a.site.unwrap();
        for &(good, mass) in &a.materials {
            let ratios = h
                .economy_catalog
                .as_ref()
                .unwrap()
                .composition(good as usize);
            let e = &mut h.sites[site as usize].economy;
            e.used[good as usize] += mass;
            e.reserves[3] += mass;
            for (k, ratio) in ratios.into_iter().enumerate() {
                e.detritus[k] += mass * ratio;
            }
        }
        let ev=c.log(h,"artifact_destroyed",site,None,a.tradition,Some(artifact),a.events.last().copied(),"The object was destroyed; embodied matter remains in local detritus and material waste".into());
        let a = &mut c.artifacts[artifact as usize];
        a.destroyed = true;
        a.custodian = None;
        a.events.push(ev);
        for n in &mut c.institutions {
            if let Some(b) = n.capacity.as_mut().and_then(|c| c.building.as_mut()) {
                if b.artifact == artifact {
                    b.condition = 0.;
                }
            }
        }
        h.culture = Some(c);
        Ok(())
    }
}
