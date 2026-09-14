//! Funded household footholds in ruins; title notices survive each occupation.
use super::{Journey, Passenger, TravelRoster};
use crate::{civilization::History, culture::Owner, gpu::Cell, participation::Presence};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

const REVIEW_MONTHS: u32 = 12;
const MAX_DISTANCE_KM: f32 = 600.;
const PROVISION_MONTHS: u32 = 12;
const ORIGIN_RESERVE_MONTHS: f32 = 12.;
const MIN_ORIGIN_RESIDENTS: f32 = 40.;
const MAX_PARTY_SHARE: f32 = 0.2;
const MAX_ANONYMOUS_PARTY: f32 = 8.;
const MIN_PARTY_RESIDENTS: f32 = 2.;
const ORIGIN_SUPPLY_RESERVE_FACTOR: f32 = 2.;
const DEFAULT_DISPOSITION: f32 = 0.5;
const TOOLS_KG_PER_PERSON: f32 = 0.5;
const CASH_PER_PERSON: f32 = 10.;
const SEED_KG_PER_CROP: f32 = 1.;
const RESTORATION_SHELTER_HEADROOM: f32 = 2.;
const REPAIR_WOOD_KG_PER_PERSON: f32 = 2.;
const REPAIR_BRICKS_KG_PER_PERSON: f32 = 3.;
const POTENTIAL_YIELD_FRACTION: f32 = 0.33;
const AMBITION_WEIGHT: f32 = 0.35;
const OPPORTUNITY_WEIGHT: f32 = 0.35;
const LOYALTY_WEIGHT: f32 = 0.25;
const CAUTION_WEIGHT: f32 = 0.3;
const DISTANCE_WEIGHT: f32 = 0.1;
const CLAIM_WINDOW_OCCUPIED_MONTHS: u32 = 120;
pub(super) const PROVISIONAL_SETTLER_SHARE: f32 = 0.05;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub enabled: bool,
    pub last_review: Option<u32>,
    pub last_claim_month: Option<u32>,
    pub occupations: Vec<Occupation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cargo {
    pub sponsor: u32,
    pub seeds: [f32; 6],
    pub wood: f32,
    pub bricks: f32,
}
impl Cargo {
    pub(crate) fn goods(&self, catalog: &crate::economy::EconomyCatalog) -> Vec<(usize, f32)> {
        let mut goods = vec![(0, self.wood), (5, self.bricks)];
        if let Some(a) = &catalog.agriculture {
            goods.extend(
                a.crops
                    .iter()
                    .zip(self.seeds)
                    .map(|(c, q)| (catalog.index(&c.good).unwrap(), q)),
            );
        }
        goods
    }
    pub(super) fn return_supplies(&self, site: &mut crate::civilization::Site) {
        site.economy.goods[0] += self.wood;
        site.economy.goods[5] += self.bricks;
        for (crop, seed) in site.economy.crops.iter_mut().zip(self.seeds) {
            crop[2] += seed;
        }
    }
    pub(super) fn lose_supplies(
        &self,
        site: &mut crate::civilization::Site,
        catalog: &crate::economy::EconomyCatalog,
    ) {
        site.economy.used[0] += self.wood;
        site.economy.used[5] += self.bricks;
        for (good, quantity) in self.goods(catalog) {
            for k in 0..3 {
                site.economy.detritus[k] += quantity * catalog.composition(good)[k];
            }
        }
    }
    pub(super) fn validate(&self, h: &History) -> Result<()> {
        ensure!(
            (self.sponsor as usize) < h.civilizations.len()
                && self
                    .seeds
                    .iter()
                    .chain([&self.wood, &self.bricks])
                    .all(|v| v.is_finite() && *v >= 0.)
                && (self.seeds.iter().all(|s| *s == 0.)
                    || h.economy_catalog
                        .as_ref()
                        .is_some_and(|c| c.agriculture.is_some())),
            "invalid resettlement cargo"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Subject {
    HouseholdInterest(u32),
    Artifact { id: u32, title: bool },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimStatus {
    Pending,
    Contested,
    Expired,
    Released,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claim {
    pub subject: Subject,
    pub claimant: Owner,
    pub status: ClaimStatus,
    pub event: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Occupation {
    pub site: u32,
    pub sponsor: u32,
    pub household: u32,
    pub arrived: u32,
    pub cause: u64,
    pub occupied_months: u32,
    pub claims: Vec<Claim>,
}
impl State {
    pub(super) fn validate(&self, h: &History) -> Result<()> {
        ensure!(
            self.last_review.is_none_or(|m| m <= h.month)
                && self.last_claim_month.is_none_or(|m| m <= h.month),
            "future resettlement boundary"
        );
        for o in &self.occupations {
            ensure!(
                (o.site as usize) < h.sites.len()
                    && (o.sponsor as usize) < h.civilizations.len()
                    && (o.household as usize) < h.society.as_ref().unwrap().households.len()
                    && o.arrived <= h.month
                    && o.occupied_months <= h.month - o.arrived
                    && h.events
                        .get(o.cause as usize)
                        .is_some_and(|e| e.kind == "resettlement_arrival"),
                "invalid occupation record"
            );
            for c in &o.claims {
                let owner = match c.claimant {
                    Owner::Person(id) => (id as usize) < h.people.len(),
                    Owner::Institution(id) => h
                        .culture
                        .as_ref()
                        .is_some_and(|v| (id as usize) < v.institutions.len()),
                    Owner::Community(id) => (id as usize) < h.sites.len(),
                };
                let subject = match c.subject {
                    Subject::HouseholdInterest(id) => {
                        (id as usize) < h.society.as_ref().unwrap().households.len()
                    }
                    Subject::Artifact { id, .. } => h
                        .culture
                        .as_ref()
                        .is_some_and(|v| (id as usize) < v.artifacts.len()),
                };
                ensure!(
                    owner
                        && subject
                        && c.event.is_none_or(|e| h
                            .events
                            .get(e as usize)
                            .is_some_and(|e| e.causes.contains(&o.cause))),
                    "invalid resettlement claim"
                );
            }
        }
        Ok(())
    }
}
impl History {
    /// Voluntary, finite request. All admission checks precede any debits.
    pub fn request_resettlement(
        &mut self,
        household: u32,
        to: u32,
        terrain: &[Cell],
    ) -> Result<()> {
        let society = self
            .society
            .as_ref()
            .context("resettlement requires society")?;
        ensure!(
            society.relocation.resettlement.enabled && self.version == 2 && self.politics.is_some(),
            "resettlement requires enabled policy, managed economy and political administration"
        );
        let home = society
            .households
            .get(household as usize)
            .context("unknown household")?;
        let from = home.site;
        let source = &self.sites[from as usize];
        let target = self.sites.get(to as usize).context("unknown destination")?;
        ensure!(
            source.economy.management[0] > 0.5 && target.economy.management[0] > 0.5,
            "resettlement requires managed crops"
        );
        ensure!(
            from != to
                && !source.abandoned
                && target.abandoned
                && target.stocks.stock[0] == 0.
                && source.island == target.island
                && self.household_available_for_relocation(household)
                && !self.civilizations.iter().any(|c| c.leader == home.head),
            "ineligible resettlement party or ruin"
        );
        ensure!(
            !society
                .relocation
                .journeys
                .iter()
                .any(|j| j.to == to && !j.returning),
            "destination already reserved"
        );
        ensure!(self.ruin_safe(to, terrain), "ruin currently unsafe");
        let route = society
            .routes
            .iter()
            .filter(|r| {
                ((r.from == from && r.to == to) || (r.to == from && r.from == to))
                    && r.passable()
                    && r.cost_km <= MAX_DISTANCE_KM
                    && r.cells
                        .iter()
                        .all(|&cell| terrain.get(cell as usize).is_some_and(|c| c.meta[0] == 2))
            })
            .min_by(|a, b| a.cost_km.total_cmp(&b.cost_km).then(a.id.cmp(&b.id)))
            .context("no known usable land route")?;
        ensure!(
            !self.resettlement_hostile(from, to),
            "hostile administration"
        );
        let traits = self
            .culture
            .as_ref()
            .and_then(|c| c.agents.get(home.head as usize))
            .map_or([DEFAULT_DISPOSITION; 6], |a| a.traits);
        ensure!(
            traits[0] * AMBITION_WEIGHT + OPPORTUNITY_WEIGHT
                > traits[4] * LOYALTY_WEIGHT
                    + traits[5] * CAUTION_WEIGHT
                    + DISTANCE_WEIGHT * route.cost_km / MAX_DISTANCE_KM,
            "household prefers to stay"
        );
        let roster = self.household_resident_roster(household);
        let mut cohorts = [0f32; 3];
        for &id in &roster {
            let age =
                crate::population_registry::age_band(self.month, self.people[id as usize].born)
                    .context("invalid passenger age")?;
            cohorts[age] += 1.;
        }
        let homes = society
            .households
            .iter()
            .filter(|h| h.site == from && self.household_available_for_relocation(h.id))
            .count();
        ensure!(homes >= 2, "origin needs remaining households");
        if !self.individual_demography_enabled() {
            let expected = (source.stocks.stock[0] / homes as f32).min(MAX_ANONYMOUS_PARTY);
            let reconciliation = self.population_reconciliation();
            let free: [f32; 3] = std::array::from_fn(|i| {
                (source.demography.ages[i] - reconciliation.sites[from as usize].known[i] as f32)
                    .max(0.)
            });
            let sum: f32 = free.iter().sum();
            let extra = (expected - roster.len() as f32).max(0.).min(sum);
            for i in 0..3 {
                cohorts[i] += extra * free[i] / sum.max(1e-10);
            }
        }
        let pop: f32 = cohorts.iter().sum();
        ensure!(
            pop >= MIN_PARTY_RESIDENTS
                && cohorts[1] >= 1.
                && pop <= source.stocks.stock[0] * MAX_PARTY_SHARE
                && source.stocks.stock[0] - pop >= MIN_ORIGIN_RESIDENTS
                && cohorts
                    .iter()
                    .zip(source.demography.ages)
                    .all(|(n, a)| *n <= a),
            "insufficient available settlers"
        );
        ensure!(
            target.economy.housing_capacity() >= pop,
            "ruin lacks surviving shelter"
        );
        let yearly_need = pop * super::PROVISION_KG_PER_RESIDENT_MONTH * PROVISION_MONTHS as f32;
        ensure!(
            target.stocks.habitat[0] * target.stocks.habitat[1] * POTENTIAL_YIELD_FRACTION
                >= yearly_need,
            "insufficient surveyed farmland"
        );
        let months = (route.cost_km / crate::society::LAND_TRAVEL_KM_PER_MONTH)
            .ceil()
            .max(1.) as u32;
        let food =
            pop * super::PROVISION_KG_PER_RESIDENT_MONTH * (months + PROVISION_MONTHS) as f32;
        let tools = pop * TOOLS_KG_PER_PERSON;
        let cash = pop * CASH_PER_PERSON;
        let cargo = Cargo {
            sponsor: self.controller(from),
            seeds: std::array::from_fn(|i| {
                if source.economy.management[0] > 0.5 {
                    source.economy.crops[i][2].min(SEED_KG_PER_CROP)
                } else {
                    0.
                }
            }),
            wood: (pop * RESTORATION_SHELTER_HEADROOM - target.economy.housing_capacity()).max(0.)
                * REPAIR_WOOD_KG_PER_PERSON,
            bricks: (pop * RESTORATION_SHELTER_HEADROOM - target.economy.housing_capacity())
                .max(0.)
                * REPAIR_BRICKS_KG_PER_PERSON,
        };
        ensure!(
            source.economy.management[0] < 0.5 || cargo.seeds.iter().any(|s| *s > 0.),
            "no crop seed available"
        );
        ensure!(
            source.stocks.stock[1]
                >= food
                    + (source.stocks.stock[0] - pop)
                        * super::PROVISION_KG_PER_RESIDENT_MONTH
                        * ORIGIN_RESERVE_MONTHS
                && source.economy.finance[0] >= cash * ORIGIN_SUPPLY_RESERVE_FACTOR
                && source.economy.goods[3] >= tools * ORIGIN_SUPPLY_RESERVE_FACTOR
                && source.economy.goods[0] >= cargo.wood * ORIGIN_SUPPLY_RESERVE_FACTOR
                && source.economy.goods[5] >= cargo.bricks * ORIGIN_SUPPLY_RESERVE_FACTOR,
            "unfunded restoration package or origin reserve"
        );
        let route_id = route.id;
        let mut path = route.cells.clone();
        if route.from != from {
            path.reverse();
        }
        let source = &mut self.sites[from as usize];
        source.stocks.stock[0] -= pop;
        source.stocks.people[3] += pop;
        source.stocks.stock[1] -= food;
        source.economy.finance[0] -= cash;
        source.economy.goods[3] -= tools;
        source.economy.goods[0] -= cargo.wood;
        source.economy.goods[5] -= cargo.bricks;
        for i in 0..3 {
            source.demography.ages[i] -= cohorts[i];
        }
        for i in 0..6 {
            source.economy.crops[i][2] -= cargo.seeds[i];
        }
        self.event("resettlement_departure", Some(from), Some(to), format!("Household {household} accepted a sponsored restoration of the ruins: {pop:.1} residents, {food:.1} kg food, {tools:.1} kg tools, {cash:.1} money, seed and repair supplies; {months} months travel, one year of arrival provisions"));
        let event = self.events.last_mut().unwrap();
        event.planned_path = Some(path);
        event.subjects.push(("household".into(), household));
        let cause = event.id;
        let infection = self.infection_departure(from, pop);
        let roster = TravelRoster {
            passengers: roster
                .into_iter()
                .map(|person| Passenger {
                    person,
                    band: crate::population_registry::age_band(
                        self.month,
                        self.people[person as usize].born,
                    )
                    .unwrap(),
                })
                .collect(),
            death_carry: [0.; 3],
        };
        let s = self.society.as_mut().unwrap();
        s.relocation
            .sites
            .resize_with(self.sites.len(), Default::default);
        s.relocation.sites[from as usize].last_departure = self.month;
        s.relocation.journeys.push(Journey {
            restoration: Some(cargo),
            infection,
            warning: None,
            roster: Some(roster),
            household,
            from,
            to,
            route: route_id,
            departed: self.month,
            arrives: self.month + months,
            cohorts,
            food,
            cash,
            tools,
            cause,
            blocked: false,
            returning: false,
            seek_help: false,
            report_population: self.sites[from as usize].stocks.stock[0],
            report_food_months: None,
        });
        Ok(())
    }
    fn resettlement_hostile(&self, from: u32, to: u32) -> bool {
        self.politics.as_ref().is_some_and(|p| {
            p.wars.iter().any(|w| {
                w.ended.is_none()
                    && ((w.attacker == self.controller(from) && w.defender == self.controller(to))
                        || (w.defender == self.controller(from)
                            && w.attacker == self.controller(to)))
            })
        })
    }
    fn ruin_safe(&self, to: u32, terrain: &[Cell]) -> bool {
        terrain
            .get(self.sites[to as usize].cell as usize)
            .is_some_and(|c| {
                c.meta[0] == 2
                    && crate::hazards::flood_depth(c) < crate::hazards::FLOOD_EXPOSURE_DEPTH_M
            })
    }
    pub(super) fn resettlement_arrival_safe(&self, j: &Journey) -> bool {
        let t = &self.sites[j.to as usize];
        // Open has already inspected the completed monthly environmental snapshot.
        let flooded = self
            .living
            .as_ref()
            .and_then(|l| l.floods.get(&j.to))
            .is_some_and(|f| f.flooded || f.persistent);
        !flooded
            && t.abandoned
            && t.stocks.stock[0] == 0.
            && t.economy.housing_capacity() >= j.population()
            && !self.resettlement_hostile(j.from, j.to)
            && j.food >= j.population() * super::PROVISION_KG_PER_RESIDENT_MONTH
    }
    pub(crate) fn resettlement_departures(&mut self, terrain: &[Cell]) {
        let Some(s) = &mut self.society else { return };
        if !s.relocation.resettlement.enabled
            || self.month % REVIEW_MONTHS != 0
            || s.relocation.resettlement.last_review == Some(self.month)
        {
            return;
        }
        s.relocation.resettlement.last_review = Some(self.month);
        // Stable source/household order, one successful foothold per source/year.
        for from in 0..self.sites.len() {
            if self.sites[from].abandoned
                || self
                    .society
                    .as_ref()
                    .unwrap()
                    .relocation
                    .sites
                    .get(from)
                    .is_some_and(|p| self.month < p.last_departure + REVIEW_MONTHS)
            {
                continue;
            }
            let homes: Vec<_> = self
                .society
                .as_ref()
                .unwrap()
                .households
                .iter()
                .filter(|h| h.site == from as u32)
                .map(|h| h.id)
                .collect();
            let mut targets: Vec<_> = self
                .sites
                .iter()
                .filter(|s| s.abandoned && s.island == self.sites[from].island)
                .map(|s| (s.id, s.economy.housing_capacity()))
                .collect();
            targets.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            'source: for (to, _) in targets {
                for &home in &homes {
                    if self.request_resettlement(home, to, terrain).is_ok() {
                        break 'source;
                    }
                }
            }
        }
    }
}

impl History {
    pub(super) fn begin_resettlement(&mut self, j: &Journey) {
        let site = j.to;
        let sponsor = j.restoration.as_ref().unwrap().sponsor;
        let mut claims = Vec::new();
        for h in &self.society.as_ref().unwrap().households {
            if h.site == site && h.share > 0. {
                claims.push(Claim {
                    subject: Subject::HouseholdInterest(h.id),
                    claimant: Owner::Person(h.head),
                    status: ClaimStatus::Pending,
                    event: None,
                });
            }
        }
        if let Some(c) = &self.culture {
            for a in &c.artifacts {
                if a.site != Some(site) || a.destroyed {
                    continue;
                }
                if a.owner != Owner::Community(site) {
                    claims.push(Claim {
                        subject: Subject::Artifact {
                            id: a.id,
                            title: true,
                        },
                        claimant: a.owner.clone(),
                        status: ClaimStatus::Pending,
                        event: None,
                    });
                }
                for owner in &a.claims {
                    claims.push(Claim {
                        subject: Subject::Artifact {
                            id: a.id,
                            title: false,
                        },
                        claimant: owner.clone(),
                        status: ClaimStatus::Pending,
                        event: None,
                    });
                }
            }
        }
        // Repeated attempts cannot erase an appearance recorded during an earlier occupation.
        for claim in &mut claims {
            if self
                .society
                .as_ref()
                .unwrap()
                .relocation
                .resettlement
                .occupations
                .iter()
                .any(|o| {
                    o.site == site
                        && o.claims.iter().any(|old| {
                            old.status == ClaimStatus::Contested
                                && same_subject(&old.subject, &claim.subject)
                                && old.claimant == claim.claimant
                        })
                })
            {
                claim.status = ClaimStatus::Contested;
            }
        }
        self.event("resettlement_arrival", Some(site), Some(j.from), format!("A new household began restoring the existing ruins under civilization {sponsor}; site identity, depleted land, buildings and objects retained. {} dormant claims have a ten-occupied-year notice period; verified appearances stop automatic expiry", claims.len()));
        let e = self.events.last_mut().unwrap();
        e.causes.push(j.cause);
        e.subjects.push(("household".into(), j.household));
        let cause = e.id;
        if let Some(p) = &mut self.politics {
            p.controllers[site as usize] = sponsor;
        }
        // Civilization field retains founding cultural provenance, just as conquest does.
        self.prepare_governance();
        self.society
            .as_mut()
            .unwrap()
            .relocation
            .resettlement
            .occupations
            .push(Occupation {
                site,
                sponsor,
                household: j.household,
                arrived: self.month,
                cause,
                occupied_months: 0,
                claims,
            });
    }
    fn claim_represented(&self, site: u32, claim: &Claim, person: u32) -> bool {
        if self.sites[site as usize].abandoned
            || self.person_presence(person).1 != Presence::Resident(site)
        {
            return false;
        }
        if let Subject::HouseholdInterest(id) = claim.subject {
            return self.person_presence(person).0 == Some(id);
        }
        match claim.claimant {
            Owner::Person(id) => id == person,
            Owner::Institution(id) => self
                .culture
                .as_ref()
                .and_then(|c| c.institutions.get(id as usize))
                .is_some_and(|n| n.active && n.members.contains(&person)),
            Owner::Community(id) => self
                .civilizations
                .iter()
                .any(|c| c.leader == person && c.id == self.controller(id)),
        }
    }
    /// Appearance needs an actual resident claimant or institutional representative.
    pub fn contest_resettlement_claim(
        &mut self,
        occupation: usize,
        claim: usize,
        representative: u32,
    ) -> Result<()> {
        let state = &self
            .society
            .as_ref()
            .context("no society")?
            .relocation
            .resettlement;
        let o = state
            .occupations
            .get(occupation)
            .context("unknown occupation")?;
        let c = o.claims.get(claim).context("unknown claim")?;
        ensure!(
            c.status == ClaimStatus::Pending && self.claim_represented(o.site, c, representative),
            "claim needs a present authorized representative before expiry"
        );
        let (site, cause) = (o.site, o.cause);
        self.event("resettlement_claim_contested", Some(site), None, "A represented claimant appeared; this title now requires agreement or adjudication and will not expire automatically".into());
        let e = self.events.last_mut().unwrap();
        e.causes.push(cause);
        e.subjects.push(("person".into(), representative));
        let event = e.id;
        let c = &mut self
            .society
            .as_mut()
            .unwrap()
            .relocation
            .resettlement
            .occupations[occupation]
            .claims[claim];
        c.status = ClaimStatus::Contested;
        c.event = Some(event);
        Ok(())
    }
    fn claim_has_representative(&self, site: u32, claim: &Claim) -> bool {
        let people = match claim.subject {
            Subject::HouseholdInterest(id) => self.household_resident_roster(id),
            _ => match claim.claimant {
                Owner::Person(id) => vec![id],
                Owner::Institution(id) => self
                    .culture
                    .as_ref()
                    .map_or_else(Vec::new, |c| c.institutions[id as usize].members.clone()),
                Owner::Community(id) => {
                    vec![self.civilizations[self.controller(id) as usize].leader]
                }
            },
        };
        people
            .into_iter()
            .any(|p| self.claim_represented(site, claim, p))
    }
    fn claim_still_exists(&self, site: u32, claim: &Claim) -> bool {
        match claim.subject {
            Subject::HouseholdInterest(id) => self
                .society
                .as_ref()
                .unwrap()
                .households
                .get(id as usize)
                .is_some_and(|h| h.site == site && h.share > 0.),
            Subject::Artifact { id, title } => self
                .culture
                .as_ref()
                .and_then(|c| c.artifacts.get(id as usize))
                .is_some_and(|a| {
                    !a.destroyed
                        && if title {
                            a.owner == claim.claimant
                        } else {
                            a.claims.contains(&claim.claimant)
                        }
                }),
        }
    }
    pub(crate) fn resettlement_claims_month(&mut self) {
        let Some(s) = &mut self.society else { return };
        if s.relocation.resettlement.last_claim_month == Some(self.month) {
            return;
        }
        s.relocation.resettlement.last_claim_month = Some(self.month);
        let mut occupations = std::mem::take(&mut s.relocation.resettlement.occupations);
        let latest: std::collections::BTreeMap<_, _> = occupations
            .iter()
            .enumerate()
            .map(|(i, o)| (o.site, i))
            .collect();
        for (index, o) in occupations.iter_mut().enumerate() {
            if latest.get(&o.site) != Some(&index)
                || o.arrived >= self.month
                || self.sites[o.site as usize].abandoned
            {
                continue;
            }
            o.occupied_months = o.occupied_months.saturating_add(1);
            for c in &mut o.claims {
                if !matches!(c.status, ClaimStatus::Pending | ClaimStatus::Contested) {
                    continue;
                }
                let status = if !self.claim_still_exists(o.site, c) {
                    Some(ClaimStatus::Released)
                } else if c.status == ClaimStatus::Contested {
                    None
                } else {
                    let appeared = self.claim_has_representative(o.site, c);
                    let petition = match c.subject {
                        Subject::Artifact { id, .. } => self.governance.as_ref().is_some_and(|g| {
                            g.artifact_petitions.iter().any(|p| {
                                p.artifact == id
                                    && p.claimant == c.claimant
                                    && p.opened >= o.arrived
                            })
                        }),
                        _ => false,
                    };
                    if appeared || petition {
                        Some(ClaimStatus::Contested)
                    } else if o.occupied_months >= CLAIM_WINDOW_OCCUPIED_MONTHS
                        && self.expire_resettlement_claim(o.site, o.household, c)
                    {
                        Some(ClaimStatus::Expired)
                    } else {
                        None
                    }
                };
                if let Some(status) = status {
                    self.event("resettlement_claim_status", Some(o.site), None, format!("Restoration title {:?}: {:?} after {} occupied months; appearances preserve disputes, expiry never deletes historical provenance",c.subject,status,o.occupied_months));
                    let e = self.events.last_mut().unwrap();
                    e.causes.push(o.cause);
                    if let Subject::Artifact { id, .. } = c.subject {
                        e.subjects.push(("artifact".into(), id));
                        self.culture.as_mut().unwrap().artifacts[id as usize]
                            .events
                            .push(e.id);
                    }
                    c.event = Some(e.id);
                    c.status = status;
                }
            }
        }
        self.society
            .as_mut()
            .unwrap()
            .relocation
            .resettlement
            .occupations = occupations;
    }
    fn expire_resettlement_claim(&mut self, site: u32, settler: u32, claim: &Claim) -> bool {
        match claim.subject {
            Subject::HouseholdInterest(id) => {
                // A household record without surviving residents cannot acquire an estate.
                if self.household_resident_roster(settler).is_empty() {
                    return false;
                }
                let society = self.society.as_mut().unwrap();
                // An absent successor group cannot acquire an estate by a timer alone.
                if !society
                    .households
                    .get(settler as usize)
                    .is_some_and(|h| h.site == site)
                    || society.relocation.away(settler)
                    || id == settler
                {
                    return false;
                }
                let share = society.households[id as usize].share;
                society.households[id as usize].share = 0.;
                society.households[settler as usize].share += share;
                // Historical household wallets are separate property, never confiscated here.
            }
            Subject::Artifact { id, title } => {
                let culture = self.culture.as_mut().unwrap();
                let a = &mut culture.artifacts[id as usize];
                // Moving an object elsewhere does not allow the restored site to seize it.
                if a.site != Some(site) {
                    return false;
                }
                if title {
                    a.owner = Owner::Community(site);
                    for n in &mut culture.institutions {
                        n.property.retain(|&v| v != id);
                    }
                } else {
                    a.claims.retain(|owner| *owner != claim.claimant);
                }
                // Lost objects stay lost. Physical recovery still requires its own work.
            }
        }
        true
    }
}
fn same_subject(a: &Subject, b: &Subject) -> bool {
    match (a, b) {
        (Subject::HouseholdInterest(a), Subject::HouseholdInterest(b)) => a == b,
        (Subject::Artifact { id: a, title: x }, Subject::Artifact { id: b, title: y }) => {
            a == b && x == y
        }
        _ => false,
    }
}

impl crate::gpu::Generator {
    pub fn request_resettlement(&mut self, household: u32, site: u32) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "finish the environmental epoch first"
        );
        let terrain = self.snapshot()?;
        self.civilizations
            .as_mut()
            .context("no history")?
            .request_resettlement(household, site, &terrain)
    }
    pub fn contest_resettlement_claim(
        &mut self,
        occupation: usize,
        claim: usize,
        person: u32,
    ) -> Result<()> {
        self.validate_living_boundary()?;
        self.civilizations
            .as_mut()
            .context("no history")?
            .contest_resettlement_claim(occupation, claim, person)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (crate::gpu::Generator, u32, u32, Vec<Cell>) {
        let mut g = crate::gpu::Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 64,
                ecology_resolution: 16,
                seed: 17,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(16).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.advance_history(24).unwrap();
        let terrain = g.snapshot().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .filter(|r| r.cost_km <= MAX_DISTANCE_KM && r.passable())
            .min_by(|a, b| a.cost_km.total_cmp(&b.cost_km))
            .unwrap()
            .clone();
        let (from, to) = (route.from, route.to);
        let household = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == from && !h.civilizations.iter().any(|c| c.leader == hh.head))
            .unwrap()
            .id;
        let heads: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|hh| hh.site == to)
            .map(|hh| hh.head)
            .collect();
        for id in heads {
            h.people[id as usize].died = Some(h.month);
        }
        for marriage in &mut h.politics.as_mut().unwrap().marriages {
            if marriage.ended.is_none()
                && marriage
                    .partners
                    .iter()
                    .any(|&id| h.people[id as usize].died.is_some())
            {
                marriage.ended = Some(h.month);
            }
        }
        for hh in &mut h.society.as_mut().unwrap().households {
            if hh.site == to {
                hh.vacant_since = Some(h.month);
            }
        }
        let s = &mut h.sites[to as usize];
        s.stocks.people[1] += s.stocks.stock[0];
        s.stocks.stock[0] = 0.;
        s.demography.ages = [0.; 4];
        s.abandoned = true;
        s.economy.housing[2] = 10.;
        s.economy.management[0] = 1.;
        // Explicit finite experimental endowments precede every balance comparison.
        let catalog = h.economy_catalog.as_ref().unwrap();
        let s = &mut h.sites[from as usize];
        let extra_food = 1_000_000. - s.stocks.stock[1];
        h.initial_food += extra_food as f64;
        s.stocks.stock[1] += extra_food;
        for k in 0..3 {
            s.economy.external[k] += extra_food * crate::economy::FOOD_CNP[k] as f32;
        }
        let extra_cash = 10000. - s.economy.finance[0];
        s.economy.finance[0] += extra_cash;
        s.economy.finance[1] += extra_cash;
        for good in [0, 3, 5] {
            let extra = 1000. - s.economy.goods[good];
            s.economy.goods[good] += extra;
            s.economy.initial[good] += extra;
            for k in 0..3 {
                s.economy.external[k] += extra * catalog.composition(good)[k];
            }
        }
        s.economy.management[0] = 1.;
        for (i, crop) in s.economy.crops.iter_mut().enumerate() {
            let extra = 10. - crop[2];
            crop[2] += extra;
            let good = catalog
                .index(&catalog.agriculture.as_ref().unwrap().crops[i].good)
                .unwrap();
            for k in 0..3 {
                s.economy.external[k] += extra * catalog.composition(good)[k];
            }
        }
        h.society.as_mut().unwrap().relocation.resettlement.enabled = true;
        (g, household, to, terrain)
    }
    fn compare_ledgers(h: &History, baseline: [f64; 6], food: f64, pop: f64) {
        for (a, b) in h.economy_residuals().iter().zip(baseline) {
            assert!((a - b).abs() < 1e-5, "economy {a} != {b}");
        }
        assert!((h.food_residual() - food).abs() < 1e-5);
        assert!((h.population_residual() - pop).abs() < 1e-5);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn restoration_moves_finite_supplies_preserves_ruins_and_rejects_competition() {
        let (mut g, household, to, terrain) = fixture();
        let h = g.civilizations.as_mut().unwrap();
        let before = h.clone();
        let mut automated = before.clone();
        automated
            .society
            .as_mut()
            .unwrap()
            .relocation
            .resettlement
            .enabled = false;
        automated.resettlement_departures(&terrain);
        assert!(automated
            .society
            .as_ref()
            .unwrap()
            .relocation
            .journeys
            .is_empty());
        automated
            .society
            .as_mut()
            .unwrap()
            .relocation
            .resettlement
            .enabled = true;
        automated.resettlement_departures(&terrain);
        assert!(automated
            .society
            .as_ref()
            .unwrap()
            .relocation
            .journeys
            .iter()
            .any(|j| j.to == to && j.restoration.is_some()));
        let reviewed = serde_json::to_value(&automated).unwrap();
        automated.resettlement_departures(&terrain);
        assert_eq!(reviewed, serde_json::to_value(&automated).unwrap());
        let ledgers = h.economy_residuals();
        let food = h.food_residual();
        let pop = h.population_residual();
        let site_id = h.sites[to as usize].id;
        let founded = h.sites[to as usize].founded;
        let housing = h.sites[to as usize].economy.housing;
        let soils = h.sites[to as usize].economy.soil;
        let from = h.society.as_ref().unwrap().households[household as usize].site;
        let mut unfunded = before.clone();
        unfunded.sites[from as usize].stocks.stock[1] = 0.;
        let saved = serde_json::to_value(&unfunded).unwrap();
        assert!(unfunded
            .request_resettlement(household, to, &terrain)
            .is_err());
        assert_eq!(saved, serde_json::to_value(&unfunded).unwrap());
        let mut wet = terrain.clone();
        wet[h.sites[to as usize].cell as usize].water[0] = 1.;
        assert!(h.request_resettlement(household, to, &wet).is_err());
        assert_eq!(
            serde_json::to_value(h.clone()).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
        h.request_resettlement(household, to, &terrain).unwrap();
        compare_ledgers(h, ledgers, food, pop);
        let competitor = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| {
                hh.site == from
                    && hh.id != household
                    && h.household_available_for_relocation(hh.id)
                    && !h.civilizations.iter().any(|c| c.leader == hh.head)
            })
            .unwrap()
            .id;
        let reserved = serde_json::to_value(h.clone()).unwrap();
        assert!(h
            .request_resettlement(competitor, to, &terrain)
            .unwrap_err()
            .to_string()
            .contains("reserved"));
        assert!(h.request_resettlement(household, to, &terrain).is_err());
        assert_eq!(reserved, serde_json::to_value(h.clone()).unwrap());
        let arrival = h.society.as_ref().unwrap().relocation.journeys[0].arrives;
        let mut resumed: History = serde_json::from_value(reserved).unwrap();
        for month in h.month + 1..=arrival {
            h.month = month;
            h.relocation_arrivals().unwrap();
            resumed.month = month;
            resumed.relocation_arrivals().unwrap();
        }
        assert_eq!(
            serde_json::to_value(h.clone()).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        // Open performs the normal reactivation after arrivals, without a new site.
        h.restore_returning_settlements();
        assert!(!h.sites[to as usize].abandoned);
        assert_eq!(h.sites.len(), before.sites.len());
        assert_eq!(h.sites[to as usize].id, site_id);
        assert_eq!(h.sites[to as usize].founded, founded);
        assert_eq!(h.sites[to as usize].economy.housing, housing);
        assert_eq!(h.sites[to as usize].economy.soil, soils);
        assert_eq!(
            h.society.as_ref().unwrap().households[household as usize].site,
            to
        );
        assert!(h.society.as_ref().unwrap().relocation.journeys.is_empty());
        assert_eq!(
            h.society
                .as_ref()
                .unwrap()
                .relocation
                .resettlement
                .occupations
                .len(),
            1
        );
        compare_ledgers(h, ledgers, food, pop);
        h.society.as_ref().unwrap().relocation.validate(h).unwrap();
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn ruined_destination_can_fail_and_restore_supplies_to_returning_household() {
        let (mut g, household, to, terrain) = fixture();
        let h = g.civilizations.as_mut().unwrap();
        let ledgers = h.economy_residuals();
        let food = h.food_residual();
        let pop = h.population_residual();
        h.request_resettlement(household, to, &terrain).unwrap();
        let j = h.society.as_ref().unwrap().relocation.journeys[0].clone();
        h.sites[to as usize].economy.housing[2] = 0.;
        for month in h.month + 1..=j.arrives + (j.arrives - j.departed) {
            h.month = month;
            h.relocation_arrivals().unwrap();
        }
        assert!(h.society.as_ref().unwrap().relocation.journeys.is_empty());
        assert!(h
            .society
            .as_ref()
            .unwrap()
            .relocation
            .resettlement
            .occupations
            .is_empty());
        assert_eq!(
            h.society.as_ref().unwrap().households[household as usize].site,
            j.from
        );
        assert!(h.sites[to as usize].abandoned);
        compare_ledgers(h, ledgers, food, pop);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn absent_claims_expire_but_appearances_and_provenance_survive() {
        let (mut g, household, to, terrain) = fixture();
        let h = g.civilizations.as_mut().unwrap();
        h.sync_culture();
        let from = h.society.as_ref().unwrap().households[household as usize].site;
        let resident = h.society.as_ref().unwrap().households[household as usize].head;
        let absent = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == from && hh.id != household)
            .unwrap()
            .head;
        let old = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == to)
            .unwrap()
            .head;
        let culture = h.culture.as_mut().unwrap();
        let start = culture.artifacts.len() as u32;
        for (i, owner) in [old, resident, absent].into_iter().enumerate() {
            culture.artifacts.push(crate::culture::Artifact {
                id: start + i as u32,
                name: "Old object".into(),
                kind: "keepsake".into(),
                creator: Some(old),
                owner: Owner::Person(owner),
                claims: vec![],
                site: Some(to),
                custodian: None,
                materials: vec![],
                topic: None,
                tradition: None,
                events: vec![],
                destroyed: false,
                lost: true,
            });
        }
        h.request_resettlement(household, to, &terrain).unwrap();
        let arrival = h.society.as_ref().unwrap().relocation.journeys[0].arrives;
        for m in h.month + 1..=arrival {
            h.month = m;
            h.relocation_arrivals().unwrap();
        }
        h.restore_returning_settlements();
        let ledgers = h.economy_residuals();
        let food = h.food_residual();
        let pop = h.population_residual();
        let claim = h
            .society
            .as_ref()
            .unwrap()
            .relocation
            .resettlement
            .occupations[0]
            .claims
            .iter()
            .position(|c| matches!(c.subject,Subject::Artifact{id,..} if id==start+2))
            .unwrap();
        let before = serde_json::to_value(h.clone()).unwrap();
        assert!(
            h.contest_resettlement_claim(0, claim, absent).is_err(),
            "remote claimant cannot assert arrival"
        );
        assert_eq!(before, serde_json::to_value(h.clone()).unwrap());
        // Claimants who actually arrive are protected automatically. The first object has no survivor.
        for m in arrival + 1..arrival + CLAIM_WINDOW_OCCUPIED_MONTHS {
            h.month = m;
            h.resettlement_claims_month();
        }
        assert_eq!(
            h.culture.as_ref().unwrap().artifacts[start as usize].owner,
            Owner::Person(old)
        );
        let count = h
            .society
            .as_ref()
            .unwrap()
            .relocation
            .resettlement
            .occupations[0]
            .occupied_months;
        h.resettlement_claims_month();
        assert_eq!(
            h.society
                .as_ref()
                .unwrap()
                .relocation
                .resettlement
                .occupations[0]
                .occupied_months,
            count
        );
        // A second abandonment pauses the occupied-month clock.
        h.sites[to as usize].abandoned = true;
        h.month += 1;
        h.resettlement_claims_month();
        assert_eq!(
            h.society
                .as_ref()
                .unwrap()
                .relocation
                .resettlement
                .occupations[0]
                .occupied_months,
            count
        );
        h.sites[to as usize].abandoned = false;
        let mut resumed: History =
            serde_json::from_str(&serde_json::to_string(h).unwrap()).unwrap();
        h.month += 1;
        h.resettlement_claims_month();
        resumed.month += 1;
        resumed.resettlement_claims_month();
        assert_eq!(
            serde_json::to_value(h.clone()).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let c = h.culture.as_ref().unwrap();
        assert_eq!(c.artifacts[start as usize].owner, Owner::Community(to));
        assert!(
            c.artifacts[start as usize].lost,
            "expiry is not physical recovery"
        );
        assert_eq!(c.artifacts[start as usize].creator, Some(old));
        assert_eq!(
            c.artifacts[(start + 1) as usize].owner,
            Owner::Person(resident)
        );
        let o = &h
            .society
            .as_ref()
            .unwrap()
            .relocation
            .resettlement
            .occupations[0];
        assert!(o.claims.iter().any(
            |c| matches!(c.subject,Subject::Artifact{id,..} if id==start+1)
                && c.status == ClaimStatus::Contested
        ));
        assert!(o
            .claims
            .iter()
            .any(|c| matches!(c.subject, Subject::HouseholdInterest(_))
                && c.status == ClaimStatus::Expired));
        assert!(
            (h.society
                .as_ref()
                .unwrap()
                .households
                .iter()
                .filter(|hh| hh.site == to)
                .map(|hh| hh.share)
                .sum::<f64>()
                - 1.)
                .abs()
                < 1e-6
        );
        compare_ledgers(h, ledgers, food, pop);
        h.society
            .as_ref()
            .unwrap()
            .relocation
            .resettlement
            .validate(h)
            .unwrap();
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn lost_restoration_party_accounts_for_seeds_and_repair_materials() {
        let (mut g, household, to, terrain) = fixture();
        let h = g.civilizations.as_mut().unwrap();
        let ledgers = h.economy_residuals();
        let food = h.food_residual();
        let pop = h.population_residual();
        h.request_resettlement(household, to, &terrain).unwrap();
        let route = h.society.as_ref().unwrap().relocation.journeys[0].route;
        h.society.as_mut().unwrap().routes[route as usize].open = false;
        for _ in 0..240 {
            h.month += 1;
            h.relocation_arrivals().unwrap();
        }
        assert!(h.society.as_ref().unwrap().relocation.journeys.is_empty());
        assert!(h
            .society
            .as_ref()
            .unwrap()
            .relocation
            .lost_households
            .contains(&household));
        assert!(h
            .society
            .as_ref()
            .unwrap()
            .relocation
            .resettlement
            .occupations
            .is_empty());
        compare_ledgers(h, ledgers, food, pop);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn individual_roster_restores_a_ruin_without_anonymous_new_residents() {
        let (mut g, household, to, terrain) = fixture();
        let h = g.civilizations.as_mut().unwrap();
        h.enable_individual_demography().unwrap();
        let roster = h.household_resident_roster(household);
        let pop = h.population_residual();
        h.request_resettlement(household, to, &terrain).unwrap();
        let j = h.society.as_ref().unwrap().relocation.journeys[0].clone();
        assert_eq!(j.population(), roster.len() as f32);
        for m in h.month + 1..=j.arrives {
            h.month = m;
            h.relocation_arrivals().unwrap();
        }
        h.restore_returning_settlements();
        assert_eq!(h.sites[to as usize].stocks.stock[0], roster.len() as f32);
        for id in roster {
            assert_eq!(h.person_presence(id).1, Presence::Resident(to));
        }
        assert!((h.population_residual() - pop).abs() < 1e-5);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn restored_site_continues_through_real_monthly_production_and_batching() {
        let (mut g, household, to, _) = fixture();
        g.request_resettlement(household, to).unwrap();
        let path =
            std::env::temp_dir().join(format!("ruin-restoration-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = crate::gpu::Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        g.advance_history(6).unwrap();
        for _ in 0..6 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
        assert!(!g.civilizations.as_ref().unwrap().sites[to as usize].abandoned);
    }
}
