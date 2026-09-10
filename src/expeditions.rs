//! Finite, persistent research and rescue voyages to the enclosing continent.
use crate::{
    civilization::History,
    gpu::{Cell, Generator},
    grid,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
const FOOD_CNP: [f32; 3] = [0.45, 0.02, 0.003];
const WOOD_CNP: [f32; 3] = [0.5, 0.002, 0.0002];
const LIMIT: usize = 512;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rules {
    pub automatic: bool,
    pub hazard_scale: f32,
    pub reserve_months: u32,
    pub cooldown_months: u32,
}
impl Default for Rules {
    fn default() -> Self {
        Self {
            automatic: true,
            hazard_scale: 1.,
            reserve_months: 6,
            cooldown_months: 60,
        }
    }
}
impl Rules {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.hazard_scale.is_finite()
                && (0. ..=5.).contains(&self.hazard_scale)
                && (2..=24).contains(&self.reserve_months)
                && (12..=240).contains(&self.cooldown_months),
            "invalid expedition rules"
        );
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
    Outward,
    Camp,
    Stranded,
    Homeward,
    Returned,
    Lost,
    Rescued,
}
impl Phase {
    pub fn active(self) -> bool {
        matches!(
            self,
            Self::Outward | Self::Camp | Self::Stranded | Self::Homeward
        )
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Objective {
    Charts,
    Geology,
    Ecology,
    Rescue,
    PatronSearch,
    Inscriptions,
    OldLiterature,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Crew {
    /// None preserves the shared competence of older voyage records.
    #[serde(default)]
    pub expertise: Option<f32>,
    pub name: String,
    pub role: String,
    pub alive: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontierRoute {
    pub port: u32,
    pub cells: Vec<u32>,
    pub km: f32,
    pub travel_months: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Expedition {
    /// Frozen planned route, not an observed travel track. Absent in old archives.
    #[serde(default)]
    pub planned_cells: Option<Vec<u32>>,
    #[serde(default)]
    pub institution: Option<u32>,
    #[serde(default)]
    pub heritage: Option<crate::expedition_heritage::Charter>,
    pub id: u32,
    pub origin: u32,
    pub sponsor: u32,
    pub public_funding: bool,
    pub route: u32,
    pub objective: Objective,
    pub rescue: Option<u32>,
    pub crew: Vec<Crew>,
    pub phase: Phase,
    pub departed: u32,
    pub due: u32,
    pub ended: Option<u32>,
    pub food: f32,
    pub timber: f32,
    pub tools: f32,
    pub purse: f64,
    pub spent: f64,
    pub findings: f32,
    pub confirmed: bool,
    pub exposure: f32,
    pub skill: f32,
    pub cause: u64,
    pub field_months: u32,
    #[serde(default)]
    pub samples: [f64; 2],
}
impl Expedition {
    pub fn team_skill(&self, role: &str) -> f32 {
        team_skill(&self.crew, self.skill, role)
    }
    pub fn research_skill(&self) -> f32 {
        self.team_skill(match self.objective {
            Objective::Ecology => "naturalist",
            Objective::Geology => "engineer",
            Objective::Charts => "navigator",
            _ => "captain",
        })
    }
    pub fn survivors(&self) -> usize {
        self.crew.iter().filter(|c| c.alive).count()
    }
}
// Half general team capacity, half specialist capacity. Dead roster slots remain
// in the denominator: attrition cannot make the survivors magically more productive.
fn team_skill(crew: &[Crew], legacy: f32, role: &str) -> f32 {
    if crew.is_empty() || !crew.iter().any(|c| c.alive) {
        return 0.;
    }
    if crew.iter().all(|c| c.expertise.is_none()) {
        return legacy;
    }
    let contribution = |c: &Crew| {
        if c.alive {
            c.expertise.unwrap_or(legacy)
        } else {
            0.
        }
    };
    let general = crew.iter().map(contribution).sum::<f32>() / crew.len() as f32;
    let specialists: Vec<_> = crew.iter().filter(|c| c.role == role).collect();
    let specialist = if specialists.is_empty() {
        general * 0.5
    } else {
        specialists.iter().map(|c| contribution(c)).sum::<f32>() / specialists.len() as f32
    };
    (0.5 * general + 0.5 * specialist).clamp(0., 1.)
}
fn starting_expertise(base: f32, role: &str, knowledge: u32, variation: f32) -> f32 {
    let topic = match role {
        "captain" | "navigator" => 3,
        "naturalist" => 7,
        "engineer" => 4,
        "guard" => 11,
        _ => 6,
    };
    (base
        * if knowledge & (1 << topic) != 0 {
            1.
        } else {
            0.85
        }
        + (variation - 0.5) * 0.1)
        .clamp(0.05, 1.)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Expeditions {
    pub version: u32,
    pub started: u32,
    pub rules: Rules,
    pub surveyed_ports: u32,
    pub routes: Vec<FrontierRoute>,
    pub voyages: Vec<Expedition>,
    pub knowledge: Vec<f32>,
    pub next_launch: Vec<u32>,
    #[serde(default)]
    pub discoveries: Option<crate::discoveries::Discoveries>,
}
fn random(seed: u32, id: u32, month: u32, channel: u32) -> f32 {
    let mut x = seed
        ^ id.wrapping_mul(0x9e3779b9)
        ^ month.wrapping_mul(0x85ebca6b)
        ^ channel.wrapping_mul(0xc2b2ae35);
    x = (x ^ (x >> 16)).wrapping_mul(0x7feb352d);
    x = (x ^ (x >> 15)).wrapping_mul(0x846ca68b);
    ((x ^ (x >> 16)) >> 8) as f32 / 16777216.
}
fn spend_food(h: &mut History, e: &mut Expedition, kg: f32) {
    let used = e.food.min(kg);
    e.food -= used;
    let s = &mut h.sites[e.origin as usize];
    s.stocks.ledger[1] += used;
    for (k, f) in FOOD_CNP.into_iter().enumerate() {
        s.economy.external[k] -= used * f;
    }
}
fn lose_food(h: &mut History, e: &mut Expedition, kg: f32) {
    let lost = e.food.min(kg);
    e.food -= lost;
    let s = &mut h.sites[e.origin as usize];
    s.stocks.ledger[2] += lost;
    for (k, f) in FOOD_CNP.into_iter().enumerate() {
        s.economy.external[k] -= lost * f;
    }
}
fn spend_tools(h: &mut History, e: &mut Expedition, kg: f32) {
    let used = e.tools.min(kg);
    e.tools -= used;
    let s = &mut h.sites[e.origin as usize];
    s.economy.used[3] += used;
    s.economy.reserves[3] += used;
}
fn spend_wood(h: &mut History, e: &mut Expedition, kg: f32) {
    let used = e.timber.min(kg);
    e.timber -= used;
    let s = &mut h.sites[e.origin as usize];
    s.economy.used[0] += used;
    for (k, f) in WOOD_CNP.into_iter().enumerate() {
        s.economy.external[k] -= used * f;
    }
}
fn record(h: &mut History, e: &Expedition, kind: &str, detail: String) {
    h.event(
        kind,
        Some(e.origin),
        None,
        format!("Expedition {}: {detail}", e.id),
    );
    h.events.last_mut().unwrap().causes.push(e.cause);
    if kind == "expedition_return" {
        let cell = h.sites[e.origin as usize].cell;
        h.events
            .last_mut()
            .unwrap()
            .spatial
            .as_mut()
            .unwrap()
            .push(crate::spatial::EventAnchor {
                cell,
                role: crate::spatial::EventRole::Milestone,
            });
    }
}
fn record_at(h: &mut History, e: &Expedition, kind: &str, detail: String, cell: u32) {
    record(h, e, kind, detail);
    h.events
        .last_mut()
        .unwrap()
        .spatial
        .as_mut()
        .unwrap()
        .push(crate::spatial::EventAnchor {
            cell,
            role: crate::spatial::EventRole::Milestone,
        });
}
fn casualty(h: &mut History, e: &mut Expedition, reason: &str) {
    let alive = e.survivors();
    let slot = (random(h.seed, e.id, h.month, 71) * alive as f32) as usize;
    if let Some(c) = e.crew.iter_mut().filter(|c| c.alive).nth(slot) {
        c.alive = false;
        let name = c.name.clone();
        h.sites[e.origin as usize].stocks.people[1] += 1.;
        record(
            h,
            e,
            "expedition_casualty",
            format!("{name} died: {reason}"),
        );
    }
}
fn refund(h: &mut History, e: &mut Expedition) {
    if let Some(id) = e.institution {
        h.culture.as_mut().unwrap().institutions[id as usize].treasury += e.purse;
    } else if e.public_funding {
        h.society.as_mut().unwrap().councils[e.sponsor as usize].treasury += e.purse;
    } else {
        h.sites[e.origin as usize].economy.finance[0] += e.purse as f32;
    }
    e.purse = 0.;
}
impl Expeditions {
    pub fn validate(&self, h: &History, cells: &[Cell]) -> Result<()> {
        self.rules.validate()?;
        ensure!(
            self.version == 1
                && self.started <= h.month
                && h.shipping.is_some()
                && h.governance.is_some()
                && self.voyages.len() <= LIMIT
                && self.knowledge.len() == h.civilizations.len()
                && self.next_launch.len() == h.civilizations.len(),
            "invalid expedition baseline"
        );
        let shipping = h.shipping.as_ref().unwrap();
        ensure!(
            self.surveyed_ports as usize <= shipping.ports.len()
                && self.routes.len() <= shipping.ports.len()
                && self
                    .knowledge
                    .iter()
                    .all(|k| k.is_finite() && (0. ..=100.).contains(k)),
            "invalid expedition survey/knowledge"
        );
        let mut ports = std::collections::BTreeSet::new();
        for r in &self.routes {
            ensure!(
                (r.port as usize) < shipping.ports.len()
                    && ports.insert(r.port)
                    && r.cells.len() >= 2
                    && r.cells.first() == Some(&shipping.ports[r.port as usize].water_cell)
                    && r.cells.iter().all(|&i| (i as usize) < cells.len())
                    && r.km.is_finite()
                    && (0. ..=20000.).contains(&r.km)
                    && (1..=64).contains(&r.travel_months),
                "invalid frontier route"
            );
            ensure!(
                r.cells[..r.cells.len() - 1]
                    .iter()
                    .all(|&i| cells[i as usize].meta[0] == 1 && cells[i as usize].water[0] > 0.25)
                    && r.cells
                        .last()
                        .is_some_and(|&i| cells[i as usize].meta[0] == 3
                            && (h.living.is_some() || cells[i as usize].water[0] < 0.25))
                    && r.cells
                        .windows(2)
                        .all(
                            |w| [(-1, 0), (1, 0), (0, -1), (0, 1)]
                                .iter()
                                .any(|&(x, y)| grid::neighbor(w[0], h.terrain_resolution, x, y)
                                    == w[1])
                        ),
                "frontier route crosses forbidden land or has a broken seam"
            );
        }
        let mut active_names = std::collections::BTreeSet::new();
        for (id, e) in self.voyages.iter().enumerate() {
            ensure!(
                e.id as usize == id
                    && (e.origin as usize) < h.sites.len()
                    && (e.sponsor as usize) < h.civilizations.len()
                    && e.institution.is_none_or(|id| h
                        .culture
                        .as_ref()
                        .is_some_and(|c| (id as usize) < c.institutions.len()))
                    && (e.route as usize) < self.routes.len()
                    && shipping.ports[self.routes[e.route as usize].port as usize].site == e.origin
                    && e.planned_cells.as_ref().is_none_or(|path| !path.is_empty()
                        && path.iter().all(|&cell| (cell as usize) < cells.len()))
                    && e.departed >= self.started
                    && e.departed <= h.month
                    && e.cause < h.events.len() as u64
                    && h.events[e.cause as usize].kind == "expedition_departure"
                    && e.crew.iter().all(|c| c
                        .expertise
                        .is_none_or(|v| v.is_finite() && (0. ..=1.).contains(&v)))
                    && e.crew.len() >= 8
                    && e.crew.len() <= 16
                    && if e.phase.active() {
                        e.ended.is_none()
                    } else {
                        e.ended.is_some_and(|m| m >= e.departed && m <= h.month)
                    }
                    && (!e.confirmed || e.phase == Phase::Returned),
                "invalid expedition identity, clock or provenance"
            );
            ensure!(
                [e.food, e.timber, e.tools, e.findings, e.exposure, e.skill]
                    .iter()
                    .all(|v| v.is_finite() && *v >= 0.)
                    && e.purse.is_finite()
                    && e.purse >= 0.
                    && e.spent.is_finite()
                    && e.spent >= 0.
                    && e.skill <= 1.
                    && e.exposure <= 1.
                    && e.findings <= 24.,
                "invalid expedition inventory"
            );
            ensure!(
                e.rescue.is_some() == (e.objective == Objective::Rescue)
                    && e.rescue.is_none_or(|r| r < e.id
                        && self.voyages[r as usize].origin == e.origin
                        && self.voyages[r as usize].objective != Objective::Rescue),
                "invalid rescue target"
            );
            ensure!(
                e.crew.iter().all(|c| !c.name.is_empty()
                    && !c.role.is_empty()
                    && (!e.phase.active() || !c.alive || active_names.insert(c.name.clone()))),
                "invalid or duplicated active crew"
            );
            ensure!(
                e.samples.iter().all(|v| v.is_finite() && *v >= 0.)
                    && e.samples.iter().sum::<f64>() <= 24. + 1e-8
                    && (self.discoveries.is_some() || e.samples == [0.; 2]),
                "invalid specimen manifest"
            );
            ensure!(
                e.phase.active()
                    || (e.food == 0.
                        && e.timber == 0.
                        && e.tools == 0.
                        && e.purse == 0.
                        && e.samples == [0.; 2]),
                "completed expedition retains inventory"
            );
        }
        crate::expedition_heritage::validate(h, &self.voyages)?;
        if let Some(d) = &self.discoveries {
            d.validate(h, self, cells)?;
        }
        Ok(())
    }
    fn survey(&mut self, h: &History, cells: &[Cell], radius: f32) {
        let shipping = h.shipping.as_ref().unwrap();
        for i in self.surveyed_ports as usize..shipping.ports.len() {
            let p = &shipping.ports[i];
            if let Some((cells, km)) =
                crate::shipping::frontier_path(p.water_cell, h.terrain_resolution, radius, cells)
            {
                self.routes.push(FrontierRoute {
                    port: i as u32,
                    cells,
                    km,
                    travel_months: (km / 600. + p.access_km / 150.).ceil().max(1.) as u32,
                });
            }
        }
        self.surveyed_ports = shipping.ports.len() as u32;
    }
    fn launch(
        &mut self,
        h: &mut History,
        route: u32,
        objective: Objective,
        rescue: Option<u32>,
    ) -> Result<u32> {
        ensure!(
            self.voyages.len() < LIMIT,
            "expedition archive limit reached"
        );
        let r = self
            .routes
            .get(route as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown frontier route"))?;
        let p = &h.shipping.as_ref().unwrap().ports[r.port as usize];
        let origin = p.site;
        let sponsor = h.controller(origin);
        ensure!(p.capacity() >= 500., "harbor lacks expedition capacity");
        let rescue_target = rescue.and_then(|id| self.voyages.get(id as usize));
        ensure!(
            if objective == Objective::Rescue {
                rescue_target.is_some_and(|e| {
                    e.phase.active() && e.origin == origin && e.objective != Objective::Rescue
                }) && !self
                    .voyages
                    .iter()
                    .any(|e| e.phase.active() && e.rescue == rescue)
            } else {
                rescue.is_none()
                    && !self
                        .voyages
                        .iter()
                        .any(|e| e.sponsor == sponsor && e.phase.active())
                    && h.month >= self.next_launch[sponsor as usize]
            },
            "active voyage, cooldown or unavailable rescue target"
        );
        let s = &h.sites[origin as usize];
        let pop = s.stocks.stock[0];
        let seats = if rescue.is_some() { 16. } else { 8. };
        let food = seats * 18. * (2 * r.travel_months + 6 + self.rules.reserve_months) as f32;
        ensure!(
            !s.abandoned
                && s.economy.policy[3] >= 0.5
                && pop >= 80.
                && s.demography.ages[1] >= 16.
                && s.stocks.stock[1] >= food + pop * 18. * 12.
                && s.economy.goods[0] >= 100. + pop
                && s.economy.goods[3] >= 24. + pop * 0.5,
            "insufficient adults, food or expedition equipment above civilian reserves: population {:.0}/80, adults {:.0}/16, food {:.0}/{:.0} kg, wood {:.1}/{:.1} kg, tools {:.1}/{:.1} kg",
            pop, s.demography.ages[1], s.stocks.stock[1], food + pop*18.*12., s.economy.goods[0],100.+pop,s.economy.goods[3],24.+pop*0.5
        );
        let heritage = crate::expedition_heritage::charter(h, origin, objective)?;
        let institution = h.culture.as_ref().and_then(|c| {
            c.institutions
                .iter()
                .filter(|n| {
                    n.operational()
                        && n.site == origin
                        && (n.kind == crate::culture::InstitutionKind::Merchant
                            || (heritage.is_some()
                                && matches!(
                                    n.kind,
                                    crate::culture::InstitutionKind::Religious
                                        | crate::culture::InstitutionKind::Scholarly
                                )))
                        && n.treasury >= 1200.
                })
                .min_by_key(|n| {
                    let preferred = match objective {
                        Objective::PatronSearch => {
                            n.kind == crate::culture::InstitutionKind::Religious
                        }
                        Objective::Inscriptions | Objective::OldLiterature => {
                            n.kind == crate::culture::InstitutionKind::Scholarly
                        }
                        _ => n.kind == crate::culture::InstitutionKind::Merchant,
                    };
                    (!preferred, n.id)
                })
                .map(|n| n.id)
        });
        let public = institution.is_none()
            && h.society.as_ref().unwrap().councils[sponsor as usize].treasury >= 1200.;
        ensure!(
            institution.is_some() || public || s.economy.finance[0] >= 4600.,
            "no wealthy public or private sponsor"
        );
        if let Some(id) = institution {
            h.culture.as_mut().unwrap().institutions[id as usize].treasury -= 600.;
        } else if public {
            h.society.as_mut().unwrap().councils[sponsor as usize].treasury -= 600.;
        } else {
            h.sites[origin as usize].economy.finance[0] -= 600.;
        }
        let s = &mut h.sites[origin as usize];
        s.stocks.stock[0] -= 8.;
        s.demography.ages[1] -= 8.;
        s.stocks.people[3] += 8.;
        s.stocks.stock[1] -= food;
        s.economy.goods[0] -= 100.;
        s.economy.goods[3] -= 24.;
        let id = self.voyages.len() as u32;
        let skill = (0.45
            + random(h.seed, id, h.month, 0) * 0.35
            + self.knowledge[sponsor as usize] * 0.002
            + h.culture
                .as_ref()
                .and_then(|c| {
                    c.agents
                        .get(h.civilizations[sponsor as usize].leader as usize)
                })
                .map_or(0., |a| a.skills[3] * 0.05))
        .min(1.);
        h.event("expedition_departure",Some(origin),None,format!("Charter {id}: {objective:?}, eight adults, {food:0.0} kg food, 100 kg timber, 24 kg tools and 600 money escrow; {} sponsorship",if institution.is_some(){"institutional"}else if public{"government"}else{"merchant"}));
        if let Some(charter) = &heritage {
            let event = h.events.last_mut().unwrap();
            event.detail.push_str(&format!("; {}", charter.motive));
            event.subjects.push(("tradition".into(), charter.tradition));
            if let Some(patron) = charter.patron {
                event.subjects.push(("patron".into(), patron));
            }
        }
        let cause = h.events.last().unwrap().id;
        if let Some(target) = rescue_target {
            h.events.last_mut().unwrap().causes.push(target.cause);
        }
        let local_knowledge = h
            .culture
            .as_ref()
            .map_or(0, |c| c.available_knowledge(h, origin));
        let crew = [
            "captain",
            "navigator",
            "naturalist",
            "engineer",
            "guard",
            "guard",
            "porter",
            "porter",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, role)| Crew {
            expertise: Some(starting_expertise(
                skill,
                role,
                local_knowledge,
                random(h.seed, id, h.month, 80 + i as u32),
            )),
            name: h.civilizations[h.sites[origin as usize].civilization as usize]
                .naming(h.seed)
                .person_with(
                    "crew",
                    id * 16 + i as u32,
                    &crate::naming::PersonalContext::local(
                        &h.sites[origin as usize],
                        h.culture.as_ref(),
                    ),
                ),
            role: role.into(),
            alive: true,
        })
        .collect();
        self.voyages.push(Expedition {
            planned_cells: Some(self.routes[route as usize].cells.clone()),
            id,
            origin,
            sponsor,
            public_funding: public,
            institution,
            heritage,
            route,
            objective,
            rescue,
            crew,
            phase: Phase::Outward,
            departed: h.month,
            due: h.month + r.travel_months,
            ended: None,
            food,
            timber: 100.,
            tools: 24.,
            purse: 600.,
            spent: 0.,
            findings: 0.,
            confirmed: false,
            exposure: 0.,
            skill,
            cause,
            field_months: 0,
            samples: [0.; 2],
        });
        self.next_launch[sponsor as usize] = h.month + self.rules.cooldown_months;
        Ok(id)
    }
}
impl History {
    pub(crate) fn expedition_year(&mut self, cells: &[Cell], radius: f32) {
        let Some(mut x) = self.expeditions.take() else {
            return;
        };
        x.survey(self, cells, radius);
        if x.rules.automatic {
            for route in 0..x.routes.len() {
                let site = &self.sites[self.shipping.as_ref().unwrap().ports
                    [x.routes[route].port as usize]
                    .site as usize];
                let mut objective = if site.demography.health[0]
                    > if x.discoveries.is_some() { 0.01 } else { 0.15 }
                {
                    Objective::Ecology
                } else if if x.discoveries.is_some() {
                    site.economy.diagnostics[0] == 2.
                } else {
                    self.accessible_resources(site.id as usize)[0] < 1000.
                } {
                    Objective::Geology
                } else {
                    match (self.month / 12 + route as u32) % 6 {
                        0 => Objective::Charts,
                        1 => Objective::Geology,
                        2 => Objective::Ecology,
                        3 => Objective::PatronSearch,
                        4 => Objective::Inscriptions,
                        _ => Objective::OldLiterature,
                    }
                };
                if let Some(source) = x.discoveries.as_ref().and_then(|d| {
                    d.sources
                        .iter()
                        .find(|s| Some(&s.cell) == x.routes[route].cells.last())
                }) {
                    let kind = match objective {
                        Objective::Ecology => Some(0),
                        Objective::Geology => Some(1),
                        _ => None,
                    };
                    if let Some(k) = kind {
                        if source.remaining[k] <= 0. {
                            objective = if source.remaining[1 - k] > 0. {
                                if k == 0 {
                                    Objective::Geology
                                } else {
                                    Objective::Ecology
                                }
                            } else {
                                Objective::Charts
                            };
                        }
                    }
                }
                let _ = x.launch(self, route as u32, objective, None);
            }
        }
        self.expeditions = Some(x);
    }
    pub(crate) fn expedition_month(&mut self, cells: &[Cell]) {
        let Some(mut x) = self.expeditions.take() else {
            return;
        };
        for i in 0..x.voyages.len() {
            if !x.voyages[i].phase.active() {
                continue;
            }
            let mut e = x.voyages[i].clone();
            let r = &x.routes[e.route as usize];
            let cell = &cells[*r.cells.last().unwrap() as usize];
            let required = e.survivors() as f32 * 18.;
            if e.food + 0.001 < required {
                casualty(self, &mut e, "provisions exhausted");
            }
            spend_food(self, &mut e, required);
            let wage = e.purse.min(20.);
            e.purse -= wage;
            e.spent += wage;
            self.sites[e.origin as usize].economy.finance[0] += wage as f32;
            spend_tools(self, &mut e, 0.15);
            spend_wood(self, &mut e, 0.4);
            if e.survivors() == 0 {
                e.phase = Phase::Lost;
                if self.controller(e.origin) == e.sponsor {
                    let a =
                        &mut self.governance.as_mut().unwrap().administrations[e.origin as usize];
                    a.unrest = (a.unrest + 0.08).min(1.);
                    a.loyalty = (a.loyalty - 0.04).max(0.);
                }
                e.ended = Some(self.month);
                let food = e.food;
                lose_food(self, &mut e, food);
                let tools = e.tools;
                spend_tools(self, &mut e, tools);
                let wood = e.timber;
                spend_wood(self, &mut e, wood);
                refund(self, &mut e);
                if let Some(d) = &mut x.discoveries {
                    d.discard(self, &mut e);
                }
                record(
                    self,
                    &e,
                    "expedition_lost",
                    "No survivors; remaining stores written off; no findings confirmed".into(),
                );
            } else if e.phase == Phase::Outward && self.month >= e.due {
                if let Some(target) = e.rescue {
                    let t = &mut x.voyages[target as usize];
                    if t.phase == Phase::Stranded {
                        let survivors = t.survivors();
                        e.crew.extend(t.crew.iter().filter(|c| c.alive).cloned());
                        e.food += t.food;
                        e.timber += t.timber;
                        e.tools += t.tools;
                        e.findings = (e.findings + t.findings).min(24.);
                        e.exposure = e.exposure.max(t.exposure);
                        for k in 0..2 {
                            e.samples[k] += t.samples[k];
                            t.samples[k] = 0.;
                        }
                        t.food = 0.;
                        t.timber = 0.;
                        t.tools = 0.;
                        refund(self, t);
                        t.phase = Phase::Rescued;
                        t.ended = Some(self.month);
                        record_at(self,&e,"expedition_rescue",format!("Reached camp and recovered {survivors} survivors from expedition {target}"), *r.cells.last().unwrap());
                    } else {
                        record_at(
                            self,
                            &e,
                            "expedition_empty_camp",
                            format!("Expedition {target} was no longer waiting at camp"),
                            *r.cells.last().unwrap(),
                        );
                    }
                    e.phase = Phase::Homeward;
                    e.due = self.month + r.travel_months;
                } else {
                    e.phase = Phase::Camp;
                    e.due = self.month + 6;
                    record_at(
                        self,
                        &e,
                        "expedition_landfall",
                        format!(
                            "Temporary research camp at outer-continent cell {}",
                            r.cells.last().unwrap()
                        ),
                        *r.cells.last().unwrap(),
                    );
                }
            } else if matches!(e.phase, Phase::Camp | Phase::Stranded) {
                let danger = (0.03
                    + cell.life[0].clamp(0., 1.) * 0.08
                    + cell.geology[0].clamp(0., 1.) * 0.12
                    + (cell.climate[0] - 18.).abs().min(50.) * 0.002
                    + cell.terrain[0].clamp(0., 5000.) * 0.00001
                    + if self.living.is_some() {
                        crate::hazards::flood_depth(cell).clamp(0., 1.5) * 0.08
                    } else {
                        0.
                    })
                    * x.rules.hazard_scale
                    * (1.2 - (e.team_skill("guard") + e.team_skill("navigator")) * 0.25)
                    * (1. - x.knowledge[e.sponsor as usize] * 0.004)
                    * (if (self.month
                        + if grid::cell_direction(*r.cells.last().unwrap(), self.terrain_resolution)
                            [1]
                            < 0.
                        {
                            6
                        } else {
                            0
                        })
                        % 12
                        < 3
                    {
                        1.3
                    } else {
                        1.
                    });
                if random(self.seed, e.id, self.month, 1) < danger {
                    let event = random(self.seed, e.id, self.month, 2);
                    if event < 0.25 {
                        casualty(self, &mut e, "outer-continent field accident");
                    } else if event < 0.65 && e.phase == Phase::Camp {
                        let damage = (event - 0.25) / 0.4;
                        let tools = e.tools * (0.3 + 0.65 * damage);
                        let timber = e.timber * (0.4 + 0.5 * damage);
                        spend_tools(self, &mut e, tools);
                        spend_wood(self, &mut e, timber);
                        e.phase = Phase::Stranded;
                        e.due = self.month + 3;
                        record_at(self,&e,"expedition_stranded","Storm damage cut the camp's return access; attempting repairs and awaiting rescue".into(), *r.cells.last().unwrap());
                    } else {
                        let loss = e.food * 0.12;
                        lose_food(self, &mut e, loss);
                        spend_tools(self, &mut e, 2.);
                        e.exposure = (e.exposure + 0.15).min(1.);
                        record(
                            self,
                            &e,
                            "expedition_setback",
                            "Damaged stores and suspect biological exposure during regional work"
                                .into(),
                        );
                    }
                }
                if e.phase == Phase::Camp && e.survivors() > 0 {
                    e.field_months += 1;
                    for crew in e.crew.iter_mut().filter(|c| c.alive) {
                        if let Some(skill) = &mut crew.expertise {
                            *skill += 0.002 * (1. - *skill);
                        }
                    }
                    let destination = *r.cells.last().unwrap();
                    let already_surveyed = x.voyages.iter().any(|v| {
                        v.heritage
                            .as_ref()
                            .is_some_and(|c| c.find.as_ref().is_some_and(|f| f.cell == destination))
                    });
                    crate::expedition_heritage::survey(self, &mut e, destination, already_surveyed);
                    let suitability = match e.objective {
                        Objective::Geology => 0.5 + cell.geology[0].clamp(0., 1.),
                        Objective::Ecology => 0.5 + cell.life[0].clamp(0., 1.),
                        _ => 1.,
                    };
                    e.findings = (e.findings
                        + e.research_skill() * suitability * (e.tools / 12.).min(1.))
                    .min(24.);
                    if e.survivors() > 0 {
                        if let Some(d) = &mut x.discoveries {
                            d.collect(self, &mut e, *r.cells.last().unwrap(), cell);
                        }
                    }
                    if self.month >= e.due
                        || e.food < e.survivors() as f32 * 18. * (r.travel_months + 2) as f32
                        || e.tools < 4.
                    {
                        e.phase = Phase::Homeward;
                        e.due = self.month + r.travel_months;
                        record_at(
                            self,
                            &e,
                            "expedition_retreat",
                            "Research concluded or reserves triggered an early return".into(),
                            *r.cells.last().unwrap(),
                        );
                    }
                } else if self.month >= e.due
                    && e.timber >= 10.
                    && e.tools >= 3.
                    && random(self.seed, e.id, self.month, 3)
                        < 0.35 * (0.5 + 0.75 * e.team_skill("engineer"))
                {
                    spend_wood(self, &mut e, 10.);
                    spend_tools(self, &mut e, 3.);
                    e.phase = Phase::Homeward;
                    e.due = self.month + r.travel_months;
                    record_at(
                        self,
                        &e,
                        "expedition_repaired",
                        "Crew restored return access using reserved timber and tools".into(),
                        *r.cells.last().unwrap(),
                    );
                }
            } else if e.phase == Phase::Homeward
                && self.month >= e.due
                && self.shipping.as_ref().unwrap().ports[r.port as usize].flood_months > 0
            {
                // Remain offshore with finite provisions; capacity recovers independently
                // of the owner's lane policy. Log the interruption once per voyage.
                if !self.events.iter().any(|event| {
                    event.kind == "expedition_weather_delay" && event.causes.contains(&e.cause)
                }) {
                    record(
                        self,
                        &e,
                        "expedition_weather_delay",
                        format!(
                            "Expedition {} waiting offshore for flooded harbor access",
                            e.id
                        ),
                    );
                }
            } else if e.phase == Phase::Homeward && self.month >= e.due {
                let mut cargo_cleared = true;
                if e.exposure > 0. || e.samples[0] > 0. {
                    let home = &mut self.sites[e.origin as usize];
                    if home.economy.goods[3] >= 2. {
                        home.economy.goods[3] -= 2.;
                        home.economy.used[3] += 2.;
                        home.economy.reserves[3] += 2.;
                        record(
                            self,
                            &e,
                            "expedition_inspection",
                            "Suspect collections isolated; inspection consumed 2 kg tools".into(),
                        );
                    } else {
                        cargo_cleared = false;
                        home.demography.health[0] =
                            (home.demography.health[0] + e.exposure * 0.2).min(1.);
                        record(self,&e,"expedition_containment_failed","Insufficient inspection equipment; cargo discarded and any fictional exposure increased local disease burden".into());
                    }
                }
                if let Some(d) = &mut x.discoveries {
                    if cargo_cleared {
                        d.deliver(self, &mut e);
                    } else {
                        d.discard(self, &mut e);
                    }
                }
                let survivors = e.survivors() as f32;
                let s = &mut self.sites[e.origin as usize];
                s.stocks.stock[0] += survivors;
                s.demography.ages[1] += survivors;
                s.stocks.people[2] += survivors;
                s.stocks.stock[1] += e.food;
                s.economy.goods[0] += e.timber;
                s.economy.goods[3] += e.tools;
                s.abandoned = false;
                e.food = 0.;
                e.tools = 0.;
                e.timber = 0.;
                refund(self, &mut e);
                e.phase = Phase::Returned;
                e.ended = Some(self.month);
                e.confirmed = e.findings > 0.;
                if e.confirmed && self.controller(e.origin) == e.sponsor {
                    let a =
                        &mut self.governance.as_mut().unwrap().administrations[e.origin as usize];
                    a.loyalty = (a.loyalty + 0.02).min(1.);
                }
                x.knowledge[e.sponsor as usize] =
                    (x.knowledge[e.sponsor as usize] + e.findings).min(100.);
                record(self,&e,"expedition_return",format!("{} survivors returned; {:0.1} points of confirmed {:?} observations; sponsor knowledge {:0.1}",e.survivors(),e.findings,e.objective,x.knowledge[e.sponsor as usize]));
                crate::expedition_heritage::deliver(self, &mut e);
            }
            if e.phase.active() && self.month == e.departed + 2 * r.travel_months + 8 {
                record(
                    self,
                    &e,
                    "expedition_overdue",
                    "Expected return date missed; sponsors can now authorize an automatic search"
                        .into(),
                );
            }
            x.voyages[i] = e;
        }
        if let Some(d) = &mut x.discoveries {
            d.month(self);
        }
        if x.rules.automatic {
            let stranded: Vec<_> = x
                .voyages
                .iter()
                .filter(|e| {
                    e.phase.active()
                        && e.objective != Objective::Rescue
                        && self.month
                            >= e.departed + 2 * x.routes[e.route as usize].travel_months + 8
                })
                .map(|e| (e.route, e.id))
                .collect();
            for (route, id) in stranded {
                let _ = x.launch(self, route, Objective::Rescue, Some(id));
            }
        }
        self.expeditions = Some(x);
    }
}
impl Generator {
    pub fn enable_expeditions(&mut self) -> Result<()> {
        let cells = self.snapshot()?;
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(
            h.shipping.is_some() && h.governance.is_some() && h.expeditions.is_none(),
            "expeditions require shipping and governance without an existing expedition baseline"
        );
        let mut x = Expeditions {
            version: 1,
            started: h.month,
            rules: Rules::default(),
            surveyed_ports: 0,
            routes: vec![],
            voyages: vec![],
            knowledge: vec![0.; h.civilizations.len()],
            next_launch: vec![h.month; h.civilizations.len()],
            discoveries: None,
        };
        x.survey(&h, &cells, self.config.radius_km);
        h.expeditions = Some(x);
        h.event(
            "expedition_baseline",
            None,
            None,
            "Chartered outer-continent expeditions enabled; temporary camps and finite provisions"
                .into(),
        );
        h.validate(&cells)?;
        self.civilizations = Some(h);
        Ok(())
    }
    pub fn launch_expedition(
        &mut self,
        route: u32,
        objective: Objective,
        rescue: Option<u32>,
    ) -> Result<u32> {
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let mut x = h
            .expeditions
            .take()
            .ok_or_else(|| anyhow::anyhow!("enable expeditions first"))?;
        let id = x.launch(&mut h, route, objective, rescue)?;
        h.expeditions = Some(x);
        h.validate(&self.snapshot()?)?;
        self.civilizations = Some(h);
        Ok(id)
    }
    pub fn configure_expeditions(&mut self, rules: Rules) -> Result<()> {
        rules.validate()?;
        let detail = format!(
            "automatic={}, hazard multiplier={}, reserve months={}, cooldown months={}",
            rules.automatic, rules.hazard_scale, rules.reserve_months, rules.cooldown_months
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        h.expeditions
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable expeditions first"))?
            .rules = rules;
        h.event("expedition_rules", None, None, detail);
        Ok(())
    }
    pub fn recall_expedition(&mut self, id: u32) -> Result<()> {
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let x = h
            .expeditions
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable expeditions first"))?;
        let e = x
            .voyages
            .get_mut(id as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown expedition"))?;
        ensure!(
            matches!(e.phase, Phase::Camp | Phase::Outward),
            "only an outward voyage or mobile camp can be recalled"
        );
        let travel = x.routes[e.route as usize].travel_months;
        e.due = h.month
            + if e.phase == Phase::Outward {
                (h.month - e.departed).max(1).min(travel)
            } else {
                travel
            };
        e.phase = Phase::Homeward;
        let origin = e.origin;
        let cause = e.cause;
        h.event(
            "expedition_recall",
            Some(origin),
            None,
            format!("Expedition {id} recalled; return travel still required"),
        );
        h.events.last_mut().unwrap().causes.push(cause);
        Ok(())
    }
}

#[cfg(test)]
mod crew_tests {
    use super::*;
    #[test]
    fn specialist_loss_reduces_capacity_and_experience_survives_transfer() {
        let mut crew: Vec<_> = [
            "captain",
            "navigator",
            "naturalist",
            "engineer",
            "guard",
            "guard",
            "porter",
            "porter",
        ]
        .into_iter()
        .map(|role| Crew {
            name: role.into(),
            role: role.into(),
            alive: true,
            expertise: Some(0.8),
        })
        .collect();
        assert!((team_skill(&crew, 0.8, "naturalist") - 0.8).abs() < 1e-6);
        crew[2].alive = false;
        assert!((team_skill(&crew, 0.8, "naturalist") - 0.35).abs() < 1e-6);
        assert!(team_skill(&crew, 0.8, "engineer") > team_skill(&crew, 0.8, "naturalist"));
        let transferred: Vec<_> = crew.iter().filter(|c| c.alive).cloned().collect();
        assert_eq!(transferred.len(), 7);
        assert!(transferred.iter().all(|c| c.expertise == Some(0.8)));
        let restored: Vec<Crew> =
            serde_json::from_slice(&serde_json::to_vec(&crew).unwrap()).unwrap();
        assert_eq!(
            team_skill(&crew, 0.8, "naturalist"),
            team_skill(&restored, 0.8, "naturalist")
        );
        for c in &mut crew {
            c.alive = false;
        }
        assert_eq!(team_skill(&crew, 0.8, "naturalist"), 0.);
    }
    #[test]
    fn local_teaching_matters_and_legacy_rosters_remain_readable() {
        assert!(
            starting_expertise(0.7, "engineer", 1 << 4, 0.5)
                > starting_expertise(0.7, "engineer", 0, 0.5)
        );
        assert_eq!(
            starting_expertise(0.7, "engineer", 1 << 3, 0.5),
            starting_expertise(0.7, "engineer", 0, 0.5)
        );
        let old: Crew =
            serde_json::from_str(r#"{"name":"Aren","role":"captain","alive":true}"#).unwrap();
        assert!(old.expertise.is_none());
        assert_eq!(team_skill(&[old], 0.7, "captain"), 0.7);
    }
}
