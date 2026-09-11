//! Civilization beta: GPU habitat/production/demography, sparse social history on CPU.
use crate::{
    economy::{Cargo, Economy, EconomyCatalog, Recipe},
    gpu::{read_buffer, Generator, Stage},
    grid,
    society::{Demography, Society},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;
const LIMIT: usize = 256;
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Stocks {
    /// People, food kg, latest production kg/month, shortage fraction.
    pub stock: [f32; 4],
    /// Yield kg/ha/year, farm ha, reserved, stable site ID.
    pub habitat: [f32; 4],
    /// Cumulative production, consumption, spoilage kg; reserved.
    pub ledger: [f32; 4],
    /// Cumulative births, deaths, immigration, emigration (population equivalents).
    pub people: [f32; 4],
}
/// Persistent demographic scale, independent of political title and economic role.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementSize {
    Remnant,
    Hamlet,
    Village,
    #[default]
    Town,
}
impl SettlementSize {
    pub fn label(self) -> &'static str {
        match self {
            Self::Remnant => "remnant community",
            Self::Hamlet => "hamlet",
            Self::Village => "village",
            Self::Town => "town",
        }
    }
    fn from_population(pop: f32) -> Self {
        if pop < 5. {
            Self::Remnant
        } else if pop < 30. {
            Self::Hamlet
        } else if pop < 100. {
            Self::Village
        } else {
            Self::Town
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SettlementLifecycle {
    pub timeline: crate::history_timeline::Timeline,
    pub size: SettlementSize,
    pub pending_size: Option<SettlementSize>,
    pub pending_months: u32,
    pub depopulated_months: u32,
    /// Cumulative managed crop harvest observed at the previous monthly boundary.
    pub harvest_observed: Option<[f32; 6]>,
    /// Last reported warehouse capacity; baseline yards are recorded separately.
    pub storage_reported: Option<f32>,
    pub housing_reported: Option<f32>,
    pub waterworks_operating: Option<bool>,
    pub waterworks_months: [u32; 2],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    pub id: u32,
    pub civilization: u32,
    pub island: u32,
    pub cell: u32,
    pub name: String,
    pub founded: u32,
    pub abandoned: bool,
    #[serde(default)]
    pub lifecycle: SettlementLifecycle,
    pub stocks: Stocks,
    #[serde(default)]
    pub economy: Economy,
    #[serde(default)]
    pub demography: Demography,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Civilization {
    #[serde(default)]
    pub language: Option<crate::naming::Language>,
    pub id: u32,
    pub name: String,
    pub leader: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: u32,
    pub name: String,
    pub civilization: u32,
    pub born: i32,
    pub died: Option<u32>,
    pub predecessor: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    /// Saved intended route, not an observed travel track.
    #[serde(default)]
    pub planned_path: Option<Vec<u32>>,
    /// None means a legacy record; Some(empty) deliberately records no known anchor.
    #[serde(default)]
    pub spatial: Option<Vec<crate::spatial::EventAnchor>>,
    #[serde(default)]
    pub subjects: Vec<(String, u32)>,
    pub id: u64,
    pub month: u32,
    pub kind: String,
    pub site: Option<u32>,
    pub other: Option<u32>,
    pub detail: String,
    #[serde(default)]
    pub causes: Vec<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shipment {
    #[serde(default)]
    pub appeal_cause: Option<u64>,
    #[serde(default)]
    pub relief_route: Option<u32>,
    pub from: u32,
    pub to: u32,
    pub food_kg: f32,
    pub arrives: u32,
    #[serde(default)]
    pub weather_delay_months: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    /// Observed at the initial survey; used only for naming, never suitability.
    #[serde(default)]
    pub naming_landmark: Option<crate::naming::Landmark>,
    pub cell: u32,
    pub island: u32,
    pub yield_kg: f32,
    pub hectares: f32,
    pub score: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct History {
    #[serde(default)]
    pub service_allocation: crate::service_allocation::Allocation,
    #[serde(default)]
    pub resolution: Option<crate::resolution::ResolutionState>,
    #[serde(default)]
    pub named_demography: Option<crate::population_registry::NamedDemography>,
    #[serde(default)]
    pub domestic: Option<crate::domestic::Domestic>,
    #[serde(default)]
    pub military: crate::military::Military,
    /// Current named travel duties, independent of optional service-work accounting.
    #[serde(default)]
    pub person_duties: std::collections::BTreeMap<u32, crate::participation::TravelDuty>,
    #[serde(default)]
    pub participation: Option<crate::participation::Participation>,
    #[serde(default)]
    pub territorial_history: Vec<crate::territory::Snapshot>,
    #[serde(default)]
    pub enterprises: Option<crate::enterprises::Enterprises>,
    /// Counterfactual custody outside town production/trade; ordinary worlds keep this empty.
    #[serde(default)]
    pub experimental_tool_reserves: std::collections::BTreeMap<u32, f64>,
    #[serde(default)]
    pub resources: Option<crate::resources::Resources>,
    #[serde(default)]
    pub offices: Option<crate::offices::Offices>,
    #[serde(default)]
    pub farming_mode: Option<bool>,
    #[serde(default)]
    pub culture: Option<crate::culture::Culture>,
    #[serde(default)]
    pub living: Option<LivingHistory>,
    pub version: u32,
    pub seed: u32,
    pub terrain_resolution: u32,
    pub source_epoch: u32,
    pub month: u32,
    pub civilizations: Vec<Civilization>,
    pub sites: Vec<Site>,
    pub people: Vec<Person>,
    pub events: Vec<Event>,
    pub shipments: Vec<Shipment>,
    pub candidates: Vec<Candidate>,
    pub initial_food: f64,
    pub initial_population: f64,
    #[serde(default)]
    pub economy_catalog: Option<EconomyCatalog>,
    #[serde(default)]
    pub nutrition_initial: [f64; 3],
    #[serde(default)]
    pub cargo: Vec<Cargo>,
    #[serde(default)]
    pub export_contracts: Vec<crate::export_contracts::ExportContract>,
    #[serde(default)]
    pub society: Option<Society>,
    #[serde(default)]
    pub politics: Option<crate::politics::Politics>,
    #[serde(default)]
    pub governance: Option<crate::governance::Governance>,
    #[serde(default)]
    pub shipping: Option<crate::shipping::Shipping>,
    #[serde(default)]
    pub expeditions: Option<crate::expeditions::Expeditions>,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LivingHistory {
    #[serde(default)]
    pub environmental_returns: bool,
    pub history_start: u32,
    pub ecology_start: u64,
    pub incomplete: bool,
    #[serde(default)]
    pub floods: std::collections::BTreeMap<u32, crate::hazards::FloodImpact>,
}
fn random(mut x: u32) -> u32 {
    x = (x ^ (x >> 16)).wrapping_mul(0x7feb352d);
    x = (x ^ (x >> 15)).wrapping_mul(0x846ca68b);
    x ^ (x >> 16)
}
pub(crate) fn distance(a: u32, b: u32, n: u32) -> f32 {
    let a = grid::cell_direction(a, n);
    let b = grid::cell_direction(b, n);
    a.iter()
        .zip(b)
        .map(|(a, b)| a * b)
        .sum::<f32>()
        .clamp(-1., 1.)
        .acos()
}
impl History {
    pub(crate) fn validate_event_links(&self) -> Result<()> {
        ensure!(
            self.events
                .iter()
                .all(|e| e.causes.iter().all(|&id| id < e.id)
                    && e.subjects.iter().all(|(kind, id)| {
                        let limit = match kind.as_str() {
                            "civilization" => self.civilizations.len(),
                            "site" => self.sites.len(),
                            "person" => self.people.len(),
                            "road" => self.society.as_ref().map_or(0, |s| s.routes.len()),
                            "port" => self.shipping.as_ref().map_or(0, |s| s.ports.len()),
                            "faction" => self.politics.as_ref().map_or(0, |p| p.factions.len()),
                            "office" => self.offices.as_ref().map_or(0, |o| o.seats.len()),
                            "household" => self.society.as_ref().map_or(0, |s| s.households.len()),
                            "patron" => self.culture.as_ref().map_or(0, |c| c.patrons.len()),
                            "tradition" => self.culture.as_ref().map_or(0, |c| c.traditions.len()),
                            "institution" => {
                                self.culture.as_ref().map_or(0, |c| c.institutions.len())
                            }
                            "artifact" => self.culture.as_ref().map_or(0, |c| c.artifacts.len()),
                            _ => 0,
                        };
                        (*id as usize) < limit
                    })),
            "cyclic or invalid event causes"
        );
        self.validate_territory()?;
        for event in &self.events {
            if let Some(path) = &event.planned_path {
                let cells = 6 * self.terrain_resolution as u64 * self.terrain_resolution as u64;
                ensure!(
                    path.len() >= 2
                        && path.len() as u64 <= cells
                        && path.iter().all(|&c| (c as u64) < cells),
                    "invalid event planned path"
                );
            }
            if let Some(anchors) = &event.spatial {
                ensure!(
                    anchors.len() <= 3
                        && anchors.iter().all(|a| (a.cell as u64)
                            < 6 * self.terrain_resolution as u64 * self.terrain_resolution as u64),
                    "invalid event spatial cell"
                );
                ensure!(
                    anchors
                        .iter()
                        .enumerate()
                        .all(|(i, a)| anchors[..i].iter().all(|b| b.role != a.role)),
                    "duplicate event spatial role"
                );
            }
        }
        Ok(())
    }
    pub(crate) fn event(
        &mut self,
        kind: &str,
        site: Option<u32>,
        other: Option<u32>,
        detail: String,
    ) {
        let spatial = [
            (site, crate::spatial::EventRole::AssociatedSite),
            (other, crate::spatial::EventRole::AssociatedOtherSite),
        ]
        .into_iter()
        .filter_map(|(id, role)| {
            id.and_then(|id| self.sites.get(id as usize))
                .map(|s| crate::spatial::EventAnchor { cell: s.cell, role })
        })
        .collect();
        self.events.push(Event {
            planned_path: None,
            spatial: Some(spatial),
            id: self.events.len() as u64,
            month: self.month,
            kind: kind.into(),
            subjects: vec![],
            site,
            other,
            detail,
            causes: match kind {
                "harvest" => self
                    .events
                    .iter()
                    .rev()
                    .find(|e| {
                        e.site == site
                            && matches!(
                                e.kind.as_str(),
                                "regional_drought" | "weather_recovery" | "flood" | "flood_receded"
                            )
                    })
                    .map(|e| vec![e.id])
                    .unwrap_or_default(),
                "raid_departure" => self
                    .events
                    .iter()
                    .rev()
                    .find(|e| e.site == site && e.kind == "food_crisis")
                    .map(|e| vec![e.id])
                    .unwrap_or_default(),
                "food_crisis" => self
                    .events
                    .iter()
                    .rev()
                    .find(|e| {
                        e.site == site
                            && matches!(
                                e.kind.as_str(),
                                "harvest"
                                    | "food_spoilage"
                                    | "route_policy"
                                    | "regional_drought"
                                    | "flood"
                                    | "road_flood_closed"
                                    | "port_weather_closed"
                            )
                    })
                    .map(|e| vec![e.id])
                    .unwrap_or_default(),
                _ => vec![],
            },
        });
    }
    fn found(&mut self, c: &Candidate, civilization: u32, population: f32, food: f32) {
        let id = self.sites.len() as u32;
        // Named from the recorded survey, not hidden resources or invented past events.
        let source = self
            .sites
            .iter()
            .find(|s| s.civilization == civilization)
            .map(|s| crate::naming::Source {
                kind: "site".into(),
                id: s.id,
                name: s.name.clone(),
            });
        let land = c
            .naming_landmark
            .map(crate::naming::Landmark::meaning)
            .unwrap_or("island");
        let label = self.civilizations[civilization as usize]
            .naming(self.seed)
            .coin(&format!("site:{id}"), &[land, "home"], source);
        self.sites.push(Site {
            id,
            civilization,
            island: c.island,
            cell: c.cell,
            name: label.clone(),
            founded: self.month,
            abandoned: false,
            lifecycle: SettlementLifecycle {
                size: SettlementSize::from_population(population),
                ..Default::default()
            },
            economy: Economy::default(),
            demography: Demography::default(),
            stocks: Stocks {
                stock: [population, food, 0., 0.],
                habitat: [c.yield_kg, c.hectares, 0., id as f32],
                ledger: [0.; 4],
                people: [0.; 4],
            },
        });
        self.event(
            "founded",
            Some(id),
            None,
            format!("{label} founded with {:.0} settlers", population),
        );
    }
    pub fn food_residual(&self) -> f64 {
        let production = self
            .sites
            .iter()
            .map(|s| s.stocks.ledger[0] as f64)
            .sum::<f64>();
        let used = self
            .sites
            .iter()
            .map(|s| (s.stocks.ledger[1] as f64) + (s.stocks.ledger[2] as f64))
            .sum::<f64>();
        let stored = self
            .sites
            .iter()
            .map(|s| s.stocks.stock[1] as f64)
            .sum::<f64>()
            + self.shipments.iter().map(|s| s.food_kg as f64).sum::<f64>()
            + self
                .cargo
                .iter()
                .filter(|c| c.good == crate::economy::FOOD as u32)
                .map(|c| c.kg as f64)
                .sum::<f64>();
        let stored = stored
            + self
                .sites
                .iter()
                .map(|s| s.demography.crops[0] as f64 + s.demography.crops[1] as f64)
                .sum::<f64>()
            + self
                .society
                .as_ref()
                .map_or(0., |s| s.raids.iter().map(|r| r.food as f64).sum());
        let stored = stored
            + self
                .expeditions
                .as_ref()
                .map_or(0., |x| x.voyages.iter().map(|e| e.food as f64).sum::<f64>());
        let stored = stored
            + self.society.as_ref().map_or(0., |s| {
                s.relocation.journeys.iter().map(|j| j.food as f64).sum()
            });
        (self.initial_food + production - used - stored) / (self.initial_food + production).max(1.)
    }
    pub fn population_residual(&self) -> f64 {
        let born = self
            .sites
            .iter()
            .map(|s| s.stocks.people[0] as f64)
            .sum::<f64>();
        let died = self
            .sites
            .iter()
            .map(|s| s.stocks.people[1] as f64)
            .sum::<f64>();
        let living = self
            .sites
            .iter()
            .map(|s| s.stocks.stock[0] as f64)
            .sum::<f64>();
        let living = living
            + self
                .society
                .as_ref()
                .map_or(0., |s| s.raids.iter().map(|r| r.soldiers as f64).sum());
        let living = living
            + self.expeditions.as_ref().map_or(0., |x| {
                x.voyages
                    .iter()
                    .filter(|e| e.phase.active())
                    .map(|e| e.survivors() as f64)
                    .sum::<f64>()
            });
        let living = living
            + self.society.as_ref().map_or(0., |s| {
                s.relocation
                    .journeys
                    .iter()
                    .map(|j| j.population() as f64)
                    .sum()
            });
        (self.initial_population + born - died - living) / (self.initial_population + born).max(1.)
    }
    pub fn validate(&self, cells: &[crate::gpu::Cell]) -> Result<()> {
        self.validate_service_work()?;
        ensure!(
            self.civilizations
                .iter()
                .all(|c| c.language.as_ref().is_none_or(|l| l.valid())),
            "invalid naming language"
        );
        ensure!(
            self.sites
                .iter()
                .all(|s| s.lifecycle.timeline.valid(s.founded, self.month)),
            "invalid settlement timeline observations"
        );
        ensure!(
            self.experimental_tool_reserves
                .iter()
                .all(|(id, kg)| (*id as usize) < self.sites.len() && kg.is_finite() && *kg >= 0.),
            "invalid experimental tool custody"
        );
        if let Some(resources) = &self.resources {
            resources.validate(self, cells)?;
        }
        ensure!(
            self.export_contracts
                .iter()
                .map(|c| (c.buyer, c.good))
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == self.export_contracts.len(),
            "duplicate export customer/good"
        );
        ensure!(
            self.export_contracts.iter().all(|c| c.buyer != c.seller
                && (c.buyer as usize) < self.sites.len()
                && (c.seller as usize) < self.sites.len()
                && self
                    .economy_catalog
                    .as_ref()
                    .is_some_and(|e| (c.good as usize) < e.goods.len()
                        && c.good as usize != crate::economy::FOOD)
                && [
                    c.observed_kg,
                    c.unit_price,
                    c.remaining_kg,
                    c.escrow,
                    c.planned_kg,
                    c.estimated_unit_cost,
                    c.quoted_surplus,
                    c.dispatched_kg
                ]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.)
                && c.unit_price > 0.),
            "invalid export contract"
        );
        if let Some(shipping) = &self.shipping {
            shipping.validate(self, cells)?;
        } else {
            ensure!(
                self.cargo.iter().all(|c| c.sea_lane.is_none()),
                "maritime cargo without shipping state"
            );
        }
        if let Some(offices) = &self.offices {
            offices.validate(self)?;
        }
        if let Some(x) = &self.expeditions {
            x.validate(self, cells)?;
        }
        ensure!(
            (self.version == 1 || self.version == 2)
                && !self.sites.is_empty()
                && self.sites.len() <= LIMIT
                && self.events.len() <= 1_000_000
                && self.candidates.len() <= 2048
                && self.month <= 120000,
            "invalid civilization version, size or clock"
        );
        ensure!(
            self.civilizations.len() <= 16
                && self
                    .civilizations
                    .iter()
                    .enumerate()
                    .all(|(i, c)| c.id as usize == i
                        && self
                            .people
                            .get(c.leader as usize)
                            .is_some_and(|p| p.civilization == c.id
                                && (p.died.is_none()
                                    || self.society.as_ref().is_some_and(|s| {
                                        s.households
                                            .iter()
                                            .any(|f| f.head == c.leader && f.vacant_since.is_some())
                                    })))),
            "invalid civilization leadership"
        );
        ensure!(
            self.people
                .iter()
                .enumerate()
                .all(|(i, p)| p.id as usize == i
                    && (p.civilization as usize) < self.civilizations.len()
                    && p.born <= self.month as i32
                    && p.died.is_none_or(|d| d <= self.month && d as i32 >= p.born)
                    && p.predecessor
                        .is_none_or(|id| (id as usize) < self.people.len()))
                && crate::population_registry::valid_succession_links(&self.people),
            "invalid historical figure"
        );
        ensure!(
            self.sites
                .iter()
                .map(|s| s.cell)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == self.sites.len(),
            "overlapping settlements"
        );
        for (i, s) in self.sites.iter().enumerate() {
            ensure!(s.id as usize == i
                    && (s.civilization as usize) < self.civilizations.len()
                    && s.lifecycle
                        .storage_reported
                        .is_none_or(|v| v.is_finite() && v >= 0.)
                    && s.lifecycle
                        .housing_reported
                        .is_none_or(|v| v.is_finite() && v >= 0.)
                    && s.lifecycle.waterworks_months.iter().all(|n| *n <= 3)
                    && s.lifecycle.pending_months < 24
                    && s.lifecycle.depopulated_months <= 12
                    && s.lifecycle
                        .harvest_observed
                        .is_none_or(|v| v.iter().all(|x| x.is_finite() && *x >= 0.))
                    && cells.get(s.cell as usize).is_some_and(
                        |c| c.meta[0] == 2 && (self.living.is_some() || c.water[0] < 0.25)
                    )
                    && s.stocks
                        .stock
                        .iter()
                        .chain(&s.stocks.habitat)
                        .chain(&s.stocks.ledger)
                        .chain(&s.stocks.people)
                        .all(|x| x.is_finite() && *x >= 0.),
            "invalid or non-central settlement {} at month {}: stocks {:?}, habitat {:?}, ledger {:?}, people {:?}, lifecycle {:?}",
            s.id, self.month, s.stocks.stock, s.stocks.habitat, s.stocks.ledger, s.stocks.people, s.lifecycle
        );
        }
        ensure!(
            self.candidates.iter().all(|c| cells
                .get(c.cell as usize)
                .is_some_and(|v| v.meta[0] == 2 && v.water[0] < 0.25)
                && c.yield_kg.is_finite()
                && c.yield_kg > 0.
                && c.hectares.is_finite()
                && c.hectares > 0.),
            "invalid settlement candidate"
        );
        ensure!(
            self.shipments
                .iter()
                .all(|s| (s.from as usize) < self.sites.len()
                    && (s.to as usize) < self.sites.len()
                    && s.arrives > self.month
                    && s.weather_delay_months <= self.month
                    && s.food_kg.is_finite()
                    && s.food_kg > 0.
                    && s.appeal_cause.is_some() == s.relief_route.is_some()
                    && s.appeal_cause
                        .is_none_or(|id| self.events.get(id as usize).is_some_and(|e| matches!(
                            e.kind.as_str(),
                            "appeal_relief_sent" | "religious_relief_sent"
                        )))
                    && s.relief_route
                        .is_none_or(|id| self.society.as_ref().is_some_and(|soc| soc
                            .routes
                            .get(id as usize)
                            .is_some_and(|r| (r.from == s.from && r.to == s.to)
                                || (r.to == s.from && r.from == s.to))))),
            "invalid shipment"
        );
        ensure!(
            self.events.iter().enumerate().all(|(i, e)| e.id == i as u64
                && e.month <= self.month
                && e.site.is_none_or(|s| (s as usize) < self.sites.len())),
            "invalid history events"
        );
        if self.version == 2 {
            self.economy_catalog
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("missing economy catalog"))?
                .validate()?;
            ensure!(
                self.sites.iter().all(|s| s.economy.valid()
                    && s.economy.claim[3] == 1.
                    && s.stocks.habitat[2] == s.cell as f32),
                "invalid economic stocks or plot: {:?}",
                self.sites
                    .iter()
                    .find(|s| !s.economy.valid()
                        || s.economy.claim[3] != 1.
                        || s.stocks.habitat[2] != s.cell as f32)
                    .map(|s| (s.id, s.economy))
            );
            ensure!(
                self.cargo
                    .iter()
                    .all(|c| c.good < crate::economy::GOODS as u32
                        && (c.from as usize) < self.sites.len()
                        && (c.to as usize) < self.sites.len()
                        && c.from != c.to
                        && c.arrives > self.month
                        && c.weather_delay_months <= self.month
                        && (c.freight_stops.is_empty()
                            || (c.freight_stops.len() <= self.sites.len()
                                && c.freight_stops.windows(2).all(|w| w[0] < w[1])
                                && c.freight_stops
                                    .iter()
                                    .all(|&s| (s as usize) < self.sites.len())
                                && self.freight_sites(c.from, c.to, c.sea_lane).is_some_and(
                                    |ends| ends.iter().all(|s| c.freight_stops.contains(s))
                                )))
                        && c.kg.is_finite()
                        && c.kg > 0.
                        && c.paid.is_finite()
                        && c.paid > 0.),
                "invalid market cargo"
            );
            ensure!(
                self.economy_residuals()
                    .iter()
                    .all(|v| v.is_finite() && v.abs() < 0.001),
                "economy C/N/P, water, money or goods budget failed: {:?}",
                self.economy_residuals()
            );
        }
        if let Some(c) = &self.culture {
            c.validate(self, cells)?;
        }
        self.validate_hazards()?;
        if let Some(e) = &self.enterprises {
            e.validate(self)?;
        }
        if let Some(society) = &self.society {
            society.validate(self, cells)?;
        }
        if let Some(p) = &self.politics {
            p.validate(self, cells)?;
        }
        if let Some(g) = &self.governance {
            g.validate(self)?;
        }
        self.validate_event_links()?;
        ensure!(
            self.food_residual().abs() < 0.001 && self.population_residual().abs() < 0.001,
            "civilization conservation residual exceeds 0.1%"
        );
        Ok(())
    }
}
pub(crate) struct Engine {
    catalog_key: Vec<u8>,
    terrain_buffer: usize,
    input: wgpu::Buffer,
    output: wgpu::Buffer,
    uniform: wgpu::Buffer,
    prospects: wgpu::Buffer,
    group: wgpu::BindGroup,
    survey: wgpu::ComputePipeline,
    month: wgpu::ComputePipeline,
    claim: wgpu::ComputePipeline,
    fish: wgpu::ComputePipeline,
    adaptive_fish: wgpu::ComputePipeline,
    economic: wgpu::Buffer,
    ecological: wgpu::Buffer,
    demographic: wgpu::Buffer,
}
impl Engine {
    fn fish(&self, g: &Generator) {
        let mut encoder = g.gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_bind_group(0, &self.group, &[]);
            pass.set_pipeline(&self.adaptive_fish);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(&self.fish);
            pass.dispatch_workgroups(1, 1, 1);
        }
        g.gpu.queue.submit(Some(encoder.finish()));
    }
    fn new(g: &Generator) -> Result<Self> {
        let d = &g.gpu.device;
        let bytes = g.config.cells() as u64 * 16;
        let ecological_bytes = g.config.eco_cells() as u64 * crate::ecology::ECO_BYTES;
        ensure!(
            bytes <= d.limits().max_storage_buffer_binding_size as u64
                && g.config.estimated_bytes() + bytes + ecological_bytes < 4 * 1024 * 1024 * 1024,
            "civilization survey exceeds GPU memory budget"
        );
        let buffer = |size, usage| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Civilization beta"),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let input = buffer(
            LIMIT as u64 * 64,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );
        let output = buffer(
            LIMIT as u64 * 64,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let prospects = buffer(
            bytes,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        );
        let demographic = buffer(
            LIMIT as u64 * std::mem::size_of::<Demography>() as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );
        let economic = buffer(
            LIMIT as u64 * std::mem::size_of::<Economy>() as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );
        let ecological = buffer(
            ecological_bytes,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );
        let mut encoder = d.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(
            &g.ecology.buffers[g.ecology.current],
            0,
            &ecological,
            0,
            ecological_bytes,
        );
        g.gpu.queue.submit(Some(encoder.finish()));
        let catalog = g
            .civilizations
            .as_ref()
            .and_then(|h| h.economy_catalog.clone())
            .unwrap_or(EconomyCatalog::bundled()?);
        catalog.validate()?;
        let properties: Vec<[f32; 4]> = (0..crate::economy::GOODS)
            .map(|i| {
                let c = catalog.composition(i);
                [
                    c[0],
                    c[1],
                    c[2],
                    catalog.goods.get(i).map_or(0., |g| g.food_energy),
                ]
            })
            .collect();
        let mut recipe_bytes = bytemuck::cast_slice(&properties).to_vec();
        let agriculture = catalog
            .agriculture
            .as_ref()
            .map(|a| a.gpu(&catalog))
            .unwrap_or_else(|| {
                let mut table = vec![[0.; 4]; 21];
                // Archives predating managed agriculture used this fixation cost.
                table[12][2] = 80.;
                table
            });
        recipe_bytes.extend_from_slice(bytemuck::cast_slice(&agriculture));
        let mut methods = [[0f32; 4]; 6];
        if let Some(m) = &catalog.materials {
            for v in &m.variants {
                let role = match v.role.as_str() {
                    "container" => 1.,
                    "digging" => 2.,
                    "cutting" => 3.,
                    "breaking" => 4.,
                    _ => 5.,
                };
                let total: f32 = v.inputs.iter().map(|(_, m)| m).sum();
                let metal: f32 = v
                    .inputs
                    .iter()
                    .filter(|(g, _)| g == "metal")
                    .map(|(_, m)| m)
                    .sum();
                methods[v.slot - 45] = [role, v.service, v.wear, metal / total];
            }
        }
        recipe_bytes.extend_from_slice(bytemuck::cast_slice(&methods));
        if catalog.recipes.is_empty() {
            recipe_bytes.extend_from_slice(bytemuck::bytes_of(&Recipe {
                input: [0.; crate::economy::GOODS],
                output: [0.; crate::economy::GOODS],
                work: [1.; 4],
            }));
        } else {
            recipe_bytes.extend_from_slice(bytemuck::cast_slice(&catalog.recipes));
        }
        let recipes = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material properties and craft recipes"),
            contents: &recipe_bytes,
            usage: wgpu::BufferUsages::STORAGE,
        });
        let uniform = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                g.config.cells(),
                0u32,
                0u32,
                g.config.seed,
                1,
                catalog.recipes.len() as u32,
                (g.config.solar_scale * g.config.crop_yield_scale).to_bits(),
                0,
                catalog.weather.drought_probability.to_bits(),
                catalog.weather.drought_severity.to_bits(),
                catalog.weather.regime_months,
                g.config.resolution,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &(0..9)
                .map(|binding| wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: if binding == 3 {
                            wgpu::BufferBindingType::Uniform
                        } else {
                            wgpu::BufferBindingType::Storage {
                                read_only: binding < 2 || binding == 7,
                            }
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                })
                .collect::<Vec<_>>(),
        });
        let resources = [
            &g.buffers[g.current],
            &input,
            &output,
            &uniform,
            &prospects,
            &economic,
            &ecological,
            &recipes,
            &demographic,
        ];
        let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &resources
                .iter()
                .enumerate()
                .map(|(i, b)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: b.as_entire_binding(),
                })
                .collect::<Vec<_>>(),
        });
        let layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        d.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Civilization simulation"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}\n{}\n{}",
                    include_str!("../shaders/civilization.wgsl"),
                    include_str!("../shaders/economy.wgsl"),
                    include_str!("../shaders/society.wgsl")
                )
                .into(),
            ),
        });
        let pipeline = |name| {
            d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(name),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let survey = pipeline("survey");
        let month = pipeline("month");
        let claim = pipeline("claim_plots");
        let fish = pipeline("fish_plots");
        let adaptive_fish = pipeline("adaptive_fish_plots");
        if let Some(e) = pollster::block_on(d.pop_error_scope()) {
            anyhow::bail!("civilization shader: {e}");
        }
        Ok(Self {
            catalog_key: serde_json::to_vec(&catalog)?,
            terrain_buffer: g.current,
            input,
            output,
            uniform,
            prospects,
            group,
            survey,
            month,
            claim,
            fish,
            adaptive_fish,
            economic,
            ecological,
            demographic,
        })
    }
    fn dispatch(&self, g: &Generator, survey: bool, count: u32) {
        let mut encoder = g.gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(if survey { &self.survey } else { &self.month });
            pass.set_bind_group(0, &self.group, &[]);
            let groups = count.div_ceil(64);
            pass.dispatch_workgroups(groups.min(65535), groups.div_ceil(65535), 1);
        }
        g.gpu.queue.submit(Some(encoder.finish()));
    }
}
impl Generator {
    pub fn found_civilizations(&mut self, count: u32) -> Result<()> {
        self.found_civilizations_with_options(count, Default::default())
    }
    pub(crate) fn found_civilizations_base(&mut self, count: u32) -> Result<()> {
        ensure!(
            self.progress.stage == Stage::Boundary,
            "finish the current epoch before founding civilizations"
        );
        ensure!(
            self.civilizations.is_none(),
            "civilizations already founded"
        );
        ensure!((1..=16).contains(&count), "choose 1–16 civilizations");
        let engine = Engine::new(self)?;
        engine.dispatch(self, true, self.config.cells());
        let n = self.config.resolution;
        let terrain = self.snapshot()?;
        let mut candidates = if let Some(nav) = self.navigation_service()? {
            nav.candidates(
                &engine.prospects,
                self.config.settlement_plot_hectares as f64,
            )?
        } else {
            let bytes = read_buffer(
                &self.gpu,
                &engine.prospects,
                0,
                self.config.cells() as u64 * 16,
            )?;
            let scores = bytemuck::cast_slice::<u8, [f32; 4]>(&bytes);
            let n = self.config.resolution;
            // Sparse central-island connectivity and site selection use CPU graph traversal.
            let mut islands = vec![u32::MAX; scores.len()];
            for start in 0..scores.len() {
                if scores[start][3] != 2. || islands[start] != u32::MAX {
                    continue;
                }
                let mut queue = std::collections::VecDeque::from([start as u32]);
                islands[start] = start as u32;
                while let Some(i) = queue.pop_front() {
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let j = grid::neighbor(i, n, dx, dy) as usize;
                        if scores[j][3] == 2. && islands[j] == u32::MAX {
                            islands[j] = start as u32;
                            queue.push_back(j as u32);
                        }
                    }
                }
            }
            let mut candidates = Vec::new();
            for (i, p) in scores.iter().enumerate() {
                if p[0] > 450. {
                    let area = grid::solid_angle(i as u32, n)
                        * (self.config.radius_km as f64 * 1000.).powi(2);
                    let coast = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                        .into_iter()
                        .any(|(dx, dy)| {
                            terrain[grid::neighbor(i as u32, n, dx, dy) as usize].meta[0] == 1
                        });
                    use crate::naming::Landmark;
                    let landmark = if coast {
                        Landmark::Shore
                    } else if terrain[i].water[3] > 1. {
                        Landmark::Water
                    } else if terrain[i].terrain[0] > 1000. {
                        Landmark::Hill
                    } else {
                        Landmark::Field
                    };
                    candidates.push(Candidate {
                        naming_landmark: Some(landmark),
                        cell: i as u32,
                        island: islands[i],
                        yield_kg: p[0],
                        hectares: (area / 10000. * 0.01)
                            .min(self.config.settlement_plot_hectares as f64)
                            as f32,
                        score: p[1],
                    });
                }
            }
            candidates
        };
        candidates.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.cell.cmp(&b.cell)));
        let mut h = History {
            participation: Some(Default::default()),
            person_duties: Default::default(),
            service_allocation: Default::default(),
            domestic: Some(Default::default()),
            named_demography: Some(Default::default()),
            resolution: None,
            military: Default::default(),
            territorial_history: vec![],
            enterprises: Some(Default::default()),
            experimental_tool_reserves: Default::default(),
            resources: None,
            offices: None,
            culture: None,
            farming_mode: Some(true),
            version: 1,
            seed: self.config.seed,
            terrain_resolution: n,
            source_epoch: self.progress.epoch,
            month: 0,
            civilizations: vec![],
            sites: vec![],
            people: vec![],
            events: vec![],
            shipments: vec![],
            candidates: vec![],
            initial_food: 0.,
            initial_population: 0.,
            economy_catalog: None,
            nutrition_initial: [0.; 3],
            cargo: vec![],
            export_contracts: vec![],
            society: None,
            politics: None,
            governance: None,
            shipping: None,
            expeditions: None,
            living: None,
        };
        // Keep a bounded, spatially separated candidate set; geography remains authoritative.
        for c in candidates {
            if h.candidates
                .iter()
                .all(|other| distance(c.cell, other.cell, n) > 0.008)
            {
                h.candidates.push(c);
                if h.candidates.len() == 2048 {
                    break;
                }
            }
        }
        ensure!(
            h.candidates.len() >= count as usize,
            "not enough habitable central-island sites"
        );
        h.event(
            "humanity_banished",
            None,
            None,
            "What remains of humanity has been banished from the Ancient World.".into(),
        );
        let mut choices = h.candidates.clone();
        for id in 0..count {
            let at = choices
                .iter()
                .position(|c| h.sites.iter().all(|s| s.island != c.island))
                .unwrap_or(0);
            let c = choices.remove(at);
            let mut language = crate::naming::Language::new(h.seed, id);
            let leader_name = language.personal(id);
            let civname = language.coin(
                &format!("civilization:{id}"),
                &["league"],
                Some(crate::naming::Source {
                    kind: "person".into(),
                    id,
                    name: leader_name.clone(),
                }),
            );
            h.people.push(Person {
                id,
                name: leader_name,
                civilization: id,
                born: -360,
                died: None,
                predecessor: None,
            });
            h.civilizations.push(Civilization {
                language: Some(language),
                id,
                name: civname,
                leader: id,
            });
            h.found(&c, id, 120., 120. * 18. * 12.);
            h.initial_food += 120. * 18. * 12.;
            h.initial_population += 120.;
        }
        h.validate(&terrain)?;
        self.civilizations = Some(h);
        self.upgrade_economy()?;
        Ok(())
    }
    fn advance_history_snapshot(&mut self, months: u32) -> Result<()> {
        self.advance_history_with_terrain(months, None)
    }

    // A supplied view must contain current observations for every cell consumed
    // by this transaction. HistoryEnvironment refreshes those cells each month;
    // GPU surveys read current world buffers; CPU reference surveys require a full view.

    // Monthly schedule: stage outputs are passed explicitly; see docs/monthly-schedule.md.
    fn history_open_month(
        &mut self,
        h: &mut History,
        engine: &Engine,
        terrain: &[crate::gpu::Cell],
        navigation: Option<&crate::navigation::Navigation>,
    ) -> Result<Vec<[[f64; 2]; crate::economy::GOODS]>> {
        h.prepare_society_with_navigation(terrain, self.config.radius_km, navigation)?;
        h.prepare_politics(terrain);
        h.prepare_governance();
        ensure!(
            h.events.len() < 1_000_000,
            "civilization beta event limit reached"
        );
        h.month += 1;
        h.activate_monthly_policies();
        if let Some(nav) = navigation.as_ref().filter(|_| h.living.is_some()) {
            let inspections = nav.inspect_routes(h)?;
            h.environmental_month_with_inspections(terrain, Some(&inspections));
        } else {
            h.environmental_month(terrain);
        }
        let deliveries = if h.version == 2 {
            h.market_arrivals()
        } else {
            vec![]
        };
        h.relief_arrivals();
        h.relocation_arrivals();
        let relief_observations = h.observe_relief();
        h.answer_appeals_observed(&relief_observations)?;
        h.restore_returning_settlements();
        self.prepare_economy(h);
        if h.resources.is_some() && h.sites.iter().any(|s| s.economy.claim[3] < 0.5) {
            engine.upload(self, h);
            engine.claim(self);
            engine.read(self, h, false)?;
            h.register_resources(terrain, self.config.radius_km);
        }
        if h.economy_catalog
            .as_ref()
            .is_some_and(|c| c.materials.is_some())
        {
            for site in &mut h.sites {
                let t = &terrain[site.cell as usize];
                site.economy.extraction[2] = self
                    .catalog
                    .rocks
                    .get(t.ids[0] as usize)
                    .map_or(0.5, |r| r.hardness / 10.)
                    .clamp(0.1, 1.);
                site.economy.extraction[3] = h
                    .resources
                    .as_ref()
                    .and_then(|r| r.sources.get(&site.cell))
                    .map_or(0., |s| (s.extracted[0] / s.initial[0].max(1.)) as f32);
            }
        }
        h.patron_aid_month();
        // Snapshot actual cumulative crop harvest before this month's GPU work.
        for site in &mut h.sites {
            if site.lifecycle.harvest_observed.is_none() {
                site.lifecycle.harvest_observed =
                    Some(std::array::from_fn(|i| site.economy.crops[i][3]));
            }
            // Older archives retained frozen fractional cohorts in ruins.
            if site.abandoned && site.stocks.stock[0] > 0. && site.stocks.stock[0] < 1. {
                site.stocks.people[1] += site.stocks.stock[0];
                site.demography.health[2] += site.stocks.stock[0];
                site.stocks.stock[0] = 0.;
                site.demography.ages[..3].fill(0.);
            }
        }
        Ok(deliveries)
    }
    fn history_reserve_month(
        &self,
        h: &mut History,
        terrain: &[crate::gpu::Cell],
    ) -> Result<(Vec<[f32; 2]>, Vec<crate::household_economy::RetailPlan>)> {
        h.check_workshop_reservation_boundary()?;
        h.begin_service_reservations();
        h.prepare_committed_vessels();
        h.reserve_learning_services()?;
        h.prepare_fisheries(terrain, self.config.eco_resolution());
        let extraction_allowances = h.allocate_resources();
        h.plan_production();
        h.prepare_enterprises();
        h.prepare_vessels();
        let retail = h.prepare_household_retail();
        Ok((extraction_allowances, retail))
    }
    fn history_execute_month(
        &self,
        h: &mut History,
        engine: &Engine,
        extraction_allowances: &[[f32; 2]],
        retail: Vec<crate::household_economy::RetailPlan>,
    ) -> Result<f64> {
        let production_started = std::time::Instant::now();
        let deaths_before: Vec<_> = h.sites.iter().map(|s| s.stocks.people[1]).collect();
        let individual_observation = h.observe_individual_demography()?;
        engine.upload(self, h);
        engine.claim(self);
        engine.fish(self);
        engine.dispatch(self, false, h.sites.len() as u32);
        engine.read(self, h, true)?;
        h.settle_domestic_care();
        if let Some(observation) = individual_observation {
            h.settle_individual_demography(observation)?;
        }
        if !h.individual_demography_enabled() {
            h.assign_demographic_deaths(&deaths_before);
        }
        h.settle_resources(extraction_allowances)?;
        h.settle_enterprises();
        h.settle_workshop_resolutions()?;
        h.storage_events();
        h.housing_events();
        h.waterworks_events();
        h.settle_household_retail(retail);
        let elapsed = production_started.elapsed().as_secs_f64() * 1000.;
        Ok(elapsed)
    }
    fn history_respond_month(
        &self,
        h: &mut History,
        terrain: &[crate::gpu::Cell],
        navigation: Option<&crate::navigation::Navigation>,
        deliveries: &[[[f64; 2]; crate::economy::GOODS]],
    ) -> Result<()> {
        let relocation_observations = h.observe_relocation();
        if h.version == 2 {
            h.market_decisions(self.config.radius_km, deliveries);
        }
        h.release_vessel_work();
        h.release_cultural_work();
        h.expedition_month(terrain);
        h.relocation_departures_observed(&relocation_observations)?;
        h.settlement_lifecycle_month();
        h.sync_offices();
        h.social_month();
        h.genealogy_month();
        h.culture_month();
        h.office_month();
        let governance_observations = h.observe_governance();
        h.governance_month_observed(&governance_observations)?;
        if h.month % 12 == 0 {
            h.annual(self.config.radius_km);
            h.prepare_society_with_navigation(terrain, self.config.radius_km, navigation)?;
            h.prepare_politics(terrain);
            h.prepare_governance();
            h.politics_year();
            h.sync_offices();
            h.social_year();
            h.shipping_year_with_navigation(terrain, self.config.radius_km, navigation)?;
            h.expedition_year_with_navigation(terrain, self.config.radius_km, navigation)?;
            h.governance_year();
        }
        Ok(())
    }
    fn history_close_month(
        &mut self,
        h: &mut History,
        engine: &Engine,
        terrain: &[crate::gpu::Cell],
        navigation: Option<&crate::navigation::Navigation>,
        record: bool,
    ) -> Result<()> {
        h.prepare_society_with_navigation(terrain, self.config.radius_km, navigation)?;
        h.prepare_politics(terrain);
        h.prepare_governance();
        self.prepare_economy(h);
        engine.upload(self, h);
        engine.claim(self);
        engine.read(self, h, false)?;
        h.register_resources(terrain, self.config.radius_km);
        h.sync_culture();
        h.sync_domestic();
        h.sync_offices();
        h.settle_participation()?;
        h.settle_learning_resolutions()?;
        h.social_indicators_month();
        if record && h.living.is_none() {
            h.record_timeline();
        }
        h.validate(terrain)?;
        self.validate_economic_grid(h)?;
        Ok(())
    }
    /// Explicit setup used by economy migration and living-history activation.
    /// Does not advance either clock or append a timeline sample.
    fn initialize_history_boundary(&mut self) -> Result<()> {
        self.run_history_schedule(0, None)
    }

    fn advance_history_with_terrain(
        &mut self,
        months: u32,
        terrain: Option<&[crate::gpu::Cell]>,
    ) -> Result<()> {
        if months == 0 {
            return Ok(());
        }
        self.run_history_schedule(months, terrain)
    }

    fn run_history_schedule(
        &mut self,
        months: u32,
        terrain: Option<&[crate::gpu::Cell]>,
    ) -> Result<()> {
        let history_started = std::time::Instant::now();
        let mut production_ms = 0.;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "history requires a completed environmental epoch"
        );
        let h = self
            .civilizations
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(
            months <= 12000 && h.month.saturating_add(months) <= 120000,
            "history limit exceeded"
        );
        if h.version == 2 {
            h.economy_catalog
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("missing economy catalog"))?
                .validate()?;
            if h.sites.iter().all(|s| s.economy.claim[3] == 1.) {
                self.validate_economic_grid(h)?;
            }
        }
        let setup_started = std::time::Instant::now();
        let key = serde_json::to_vec(&h.economy_catalog)?;
        // Historical events are append-only in this transaction. Move their prefix
        // instead of cloning decades of strings every month; restore it on failure.
        let events = std::mem::take(&mut self.civilizations.as_mut().unwrap().events);
        let event_boundary = events.len();
        let mut h = self.civilizations.as_ref().unwrap().clone();
        h.events = events;
        let operation = (|| -> Result<(Engine, f64)> {
            let navigation = self.navigation_service()?;
            let engine = if let Some(e) = self
                .history_engine
                .take()
                .filter(|e| e.catalog_key == key && e.terrain_buffer == self.current)
            {
                let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
                encoder.copy_buffer_to_buffer(
                    &self.ecology.buffers[self.ecology.current],
                    0,
                    &e.ecological,
                    0,
                    self.config.eco_cells() as u64 * crate::ecology::ECO_BYTES,
                );
                self.gpu.queue.submit(Some(encoder.finish()));
                e
            } else {
                Engine::new(self)?
            };
            let setup_ms = setup_started.elapsed().as_secs_f64() * 1000.;
            let owned_terrain;
            let terrain = match terrain {
                Some(terrain) => terrain,
                None => {
                    owned_terrain = self.snapshot()?;
                    &owned_terrain
                }
            };
            if months == 0 {
                // Only the explicit initialization entry point submits a zero-length schedule.
                self.history_close_month(&mut h, &engine, terrain, navigation.as_deref(), false)?;
            }
            for _ in 0..months {
                let deliveries =
                    self.history_open_month(&mut h, &engine, terrain, navigation.as_deref())?;
                let (extraction, retail) = self.history_reserve_month(&mut h, terrain)?;
                production_ms +=
                    self.history_execute_month(&mut h, &engine, &extraction, retail)?;
                self.history_respond_month(&mut h, terrain, navigation.as_deref(), &deliveries)?;
                self.history_close_month(&mut h, &engine, terrain, navigation.as_deref(), true)?;
            }
            if h.version == 2 {
                let mut e = self.gpu.device.create_command_encoder(&Default::default());
                e.copy_buffer_to_buffer(
                    &engine.ecological,
                    0,
                    &self.ecology.buffers[self.ecology.current],
                    0,
                    self.config.eco_cells() as u64 * crate::ecology::ECO_BYTES,
                );
                self.gpu.queue.submit(Some(e.finish()));
            }
            Ok((engine, setup_ms))
        })();
        let (engine, setup_ms) = match operation {
            Ok(value) => value,
            Err(error) => {
                h.events.truncate(event_boundary);
                self.civilizations.as_mut().unwrap().events = h.events;
                return Err(error);
            }
        };
        self.civilizations = Some(h);
        self.history_engine = Some(engine);
        let elapsed = history_started.elapsed().as_secs_f64() * 1000.;
        for (name, value) in [
            ("history_setup_wall_ms", setup_ms),
            ("history_production_and_readback_wall_ms", production_ms),
            (
                "history_social_and_validation_wall_ms",
                (elapsed - setup_ms - production_ms).max(0.),
            ),
        ] {
            *self.progress.stage_ms.entry(name.into()).or_default() += value;
        }
        Ok(())
    }
}
impl History {
    fn annual(&mut self, radius: f32) {
        self.evolve_lexicons();
        let n = self.terrain_resolution;
        for i in 0..self.sites.len() {
            if self.sites[i].abandoned {
                continue;
            }
            let p = self.sites[i].stocks.stock;
            self.event(
                "year",
                Some(i as u32),
                None,
                format!(
                    "Population {:.0}; food {:.0} kg; shortage {:.0}%",
                    p[0],
                    p[1],
                    p[3] * 100.
                ),
            );
            if p[1] < p[0] * 18. * 3.
                && !self
                    .society
                    .as_ref()
                    .is_some_and(|s| s.relocation.witnessed_relief)
            {
                let donor = (0..self.sites.len())
                    .filter(|&j| {
                        j != i
                            && !self.sites[j].abandoned
                            && self.sites[j].island == self.sites[i].island
                            && if self.society.is_some() {
                                self.route_cost(j as u32, i as u32)
                                    .is_some_and(|c| c < 1500.)
                            } else {
                                distance(self.sites[j].cell, self.sites[i].cell, n) * radius < 1500.
                            }
                    })
                    .max_by(|&a, &b| {
                        self.sites[a].stocks.stock[1].total_cmp(&self.sites[b].stocks.stock[1])
                    });
                if let Some(j) = donor {
                    let surplus = (self.sites[j].stocks.stock[1]
                        - self.sites[j].stocks.stock[0] * 18. * 18.)
                        .max(0.);
                    let amount = surplus.min(p[0] * 18. * 6.);
                    if amount > 1. {
                        self.sites[j].stocks.stock[1] -= amount;
                        let travel = self.route_cost(j as u32, i as u32).unwrap_or_else(|| {
                            distance(self.sites[j].cell, self.sites[i].cell, n) * radius
                        });
                        let months = (travel / 150.).ceil().max(1.) as u32;
                        self.shipments.push(Shipment {
                            appeal_cause: None,
                            relief_route: None,
                            from: j as u32,
                            to: i as u32,
                            food_kg: amount,
                            weather_delay_months: 0,
                            arrives: self.month + months,
                        });
                        self.event(
                            "relief_sent",
                            Some(j as u32),
                            Some(i as u32),
                            format!("Sent {:.0} kg food; {} months travel", amount, months),
                        );
                    }
                }
            }
        }
        let old_len = self.sites.len();
        for i in 0..old_len {
            if self.sites.len() >= LIMIT {
                break;
            }
            let s = &self.sites[i];
            if s.abandoned
                || self
                    .living
                    .as_ref()
                    .and_then(|l| l.floods.get(&s.id))
                    .is_some_and(|f| f.persistent)
                || s.stocks.stock[0] < 160.
                || s.stocks.stock[1] < s.stocks.stock[0] * 18. * 12.
            {
                continue;
            }
            let candidate = self
                .candidates
                .iter()
                .filter(|c| {
                    c.island == s.island
                        && self.sites.iter().all(|s| s.cell != c.cell)
                        && distance(c.cell, s.cell, n) * radius < 600.
                })
                .max_by(|a, b| {
                    let attractiveness = |c: &Candidate| {
                        c.yield_kg / (1. + distance(c.cell, s.cell, n) * radius / 200.)
                    };
                    attractiveness(a).total_cmp(&attractiveness(b))
                })
                .cloned();
            if let Some(c) = candidate {
                // Productive homelands retain households longer; better farmland
                // encourages dispersal. Travel cost enters destination selection.
                let threshold =
                    160. + 120. * (s.stocks.habitat[0] / c.yield_kg.max(1.)).clamp(0.5, 2.);
                if s.stocks.stock[0] < threshold {
                    continue;
                }
                let civ = s.civilization;
                let settlers = (s.stocks.stock[0] * 0.2).clamp(40., 90.);
                let cohort_fraction = settlers / s.stocks.stock[0];
                let mut migrant_ages = s.demography.ages;
                for age in &mut migrant_ages[..3] {
                    *age *= cohort_fraction;
                }
                let food = settlers * 18. * 12.;
                self.sites[i].stocks.stock[0] -= settlers;
                self.sites[i].stocks.stock[1] -= food;
                self.sites[i].stocks.people[3] += settlers;
                self.found(&c, civ, settlers, food);
                self.sites.last_mut().unwrap().stocks.people[2] += settlers;
                if self.society.is_some() {
                    for (k, amount) in migrant_ages[..3].iter().enumerate() {
                        self.sites[i].demography.ages[k] -= amount;
                    }
                    self.sites.last_mut().unwrap().demography.ages = migrant_ages;
                }
                if self.version == 2 {
                    let cash = self.sites[i].economy.finance[0] * 0.2;
                    self.sites[i].economy.finance[0] -= cash;
                    self.sites.last_mut().unwrap().economy.finance[0] = cash;
                }
                self.event(
                    "migration",
                    Some(i as u32),
                    Some(self.sites.len() as u32 - 1),
                    format!("{settlers:.0} settlers founded a daughter village; local farmland and travel costs guided settlement"),
                );
            }
        }
        if self.society.is_some() {
            return;
        }
        for i in 0..self.civilizations.len() {
            let old = self.civilizations[i].leader as usize;
            let age = self.month as i32 - self.people[old].born;
            if age > 720 && random(self.seed ^ self.month ^ old as u32) % 12 == 0 {
                self.people[old].died = Some(self.month);
                let id = self.people.len() as u32;
                self.people.push(Person {
                    id,
                    name: self.civilizations[i].naming(self.seed).person_with(
                        "person",
                        id,
                        &crate::naming::PersonalContext::default().with_person(&self.people[old]),
                    ),
                    civilization: i as u32,
                    born: self.month as i32 - 360,
                    died: None,
                    predecessor: Some(old as u32),
                });
                self.civilizations[i].leader = id;
                self.event(
                    "succession",
                    None,
                    None,
                    format!(
                        "{} succeeded {} as leader of {}",
                        self.people[id as usize].name,
                        self.people[old].name,
                        self.civilizations[i].name
                    ),
                );
            }
        }
    }
}
impl Engine {
    fn upload(&self, g: &Generator, h: &History) {
        let weather = h
            .economy_catalog
            .as_ref()
            .map(|c| c.weather.clone())
            .unwrap_or_default();
        let stocks = h.sites.iter().map(|s| s.stocks).collect::<Vec<_>>();
        let economies = h.sites.iter().map(|s| s.economy).collect::<Vec<_>>();
        let demographic = h.sites.iter().map(|s| s.demography).collect::<Vec<_>>();
        g.gpu
            .queue
            .write_buffer(&self.demographic, 0, bytemuck::cast_slice(&demographic));
        g.gpu
            .queue
            .write_buffer(&self.input, 0, bytemuck::cast_slice(&stocks));
        g.gpu
            .queue
            .write_buffer(&self.economic, 0, bytemuck::cast_slice(&economies));
        g.gpu.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::cast_slice(&[
                g.config.cells(),
                h.sites.len() as u32,
                h.month,
                h.seed,
                h.version,
                h.economy_catalog.as_ref().map_or(0, |c| c.recipes.len()) as u32,
                (g.config.solar_scale * g.config.crop_yield_scale).to_bits(),
                u32::from(h.society.is_some())
                    | (u32::from(h.living.is_some()) << 1)
                    | (u32::from(h.individual_demography_enabled() || h.resolution.is_some()) << 2),
                weather.drought_probability.to_bits(),
                weather.drought_severity.to_bits(),
                weather.regime_months,
                g.config.resolution,
            ]),
        );
    }
    fn claim(&self, g: &Generator) {
        let mut e = g.gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = e.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.claim);
            pass.set_bind_group(0, &self.group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        g.gpu.queue.submit(Some(e.finish()));
    }
    fn read(&self, g: &Generator, h: &mut History, stocks: bool) -> Result<()> {
        if stocks {
            let bytes = read_buffer(&g.gpu, &self.output, 0, h.sites.len() as u64 * 64)?;
            for (s, v) in h
                .sites
                .iter_mut()
                .zip(bytemuck::cast_slice::<u8, Stocks>(&bytes))
            {
                s.stocks = *v;
            }
        }
        let bytes = read_buffer(
            &g.gpu,
            &self.economic,
            0,
            h.sites.len() as u64 * std::mem::size_of::<Economy>() as u64,
        )?;
        for (s, v) in h
            .sites
            .iter_mut()
            .zip(bytemuck::cast_slice::<u8, Economy>(&bytes))
        {
            s.economy = *v;
        }
        let bytes = read_buffer(
            &g.gpu,
            &self.demographic,
            0,
            h.sites.len() as u64 * std::mem::size_of::<Demography>() as u64,
        )?;
        for (s, d) in h
            .sites
            .iter_mut()
            .zip(bytemuck::cast_slice::<u8, Demography>(&bytes))
        {
            s.demography = *d;
        }
        Ok(())
    }
}
impl Generator {
    /// Explicit migration: reserves environmental plots and records the new inventory baseline.
    pub fn upgrade_economy(&mut self) -> Result<()> {
        if !self.ecology.clock.initialized {
            self.ecology.prepare(
                &self.gpu,
                &self.buffers[self.current],
                &self.catalog_buffer,
                &self.config,
                &self.catalog,
                false,
            );
        }
        let previous = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(previous.version == 1, "economy already enabled");
        let h = self.civilizations.as_mut().unwrap();
        h.version = 2;
        h.economy_catalog = Some(EconomyCatalog::bundled()?);
        let food = h
            .sites
            .iter()
            .map(|s| s.stocks.stock[1] as f64)
            .sum::<f64>()
            + h.shipments.iter().map(|s| s.food_kg as f64).sum::<f64>();
        h.nutrition_initial = crate::economy::FOOD_CNP.map(|v| v * food);
        h.event(
            "economy_baseline",
            None,
            None,
            "Managed plots reserved from ecology; tools and money declared as starting inventories"
                .into(),
        );
        if let Err(e) = self.initialize_history_boundary() {
            self.civilizations = Some(previous);
            return Err(e);
        }
        Ok(())
    }
    fn prepare_economy(&self, h: &mut History) {
        if h.version != 2 {
            return;
        }
        let returns = h.living.as_ref().is_some_and(|l| l.environmental_returns);
        let catalog = h.economy_catalog.as_ref().unwrap();
        let mut new_sites = vec![];
        for s in &mut h.sites {
            if s.economy.claim[1] == 0. {
                let carried_cash = s.economy.finance[0];
                // Sites present at upgrade are baseline residents; later founding imports no capital.
                let founder = s.stocks.people[2] == 0. && s.founded == 0;
                s.economy = Economy::new(
                    self,
                    s.cell,
                    s.stocks.habitat[1].min((s.stocks.stock[0] * 2.).max(1.)),
                    founder,
                    catalog,
                );
                if !founder {
                    new_sites.push(s.id);
                }
                s.economy.finance[0] += carried_cash;
                if h.farming_mode == Some(false) {
                    s.economy.management[0] = 0.;
                }
                s.stocks.habitat[1] = s.economy.claim[1] / 10000.;
                s.stocks.habitat[2] = s.cell as f32;
            }
        }
        for s in &mut h.sites {
            s.economy.land_return[0] = f32::from(returns);
            s.economy.land_return[2] = f32::from(s.abandoned);
        }
        for id in new_sites {
            if let Some(source) = h
                .sites
                .iter()
                .filter(|s| {
                    s.id != id
                        && s.civilization == h.sites[id as usize].civilization
                        && s.founded < h.sites[id as usize].founded
                        && !s.abandoned
                })
                .min_by(|a, b| {
                    distance(a.cell, h.sites[id as usize].cell, h.terrain_resolution).total_cmp(
                        &distance(b.cell, h.sites[id as usize].cell, h.terrain_resolution),
                    )
                })
                .map(|s| s.id as usize)
            {
                for crop in 0..6 {
                    let seeds = (h.sites[source].economy.crops[crop][2] * 0.2).min(2.);
                    h.sites[source].economy.crops[crop][2] -= seeds;
                    h.sites[id as usize].economy.crops[crop][2] += seeds;
                }
                for herd in 0..3 {
                    let animals = h.sites[source].economy.herds[herd][0] * 0.2;
                    h.sites[source].economy.herds[herd][0] -= animals;
                    h.sites[id as usize].economy.herds[herd][0] += animals;
                }
            }
        }
    }
    /// Boundary-applied farming and market policies; recorded for reproducible continuation.
    pub fn set_site_policy(&mut self, site: u32, policy: [f32; 4]) -> Result<()> {
        ensure!(
            policy
                .iter()
                .all(|p| p.is_finite() && (0. ..=1.).contains(p)),
            "policy values must be in 0..1"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no civilizations"))?;
        ensure!(h.version == 2, "upgrade the economy first");
        let s = h
            .sites
            .get_mut(site as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown site"))?;
        s.economy.policy = policy;
        h.event(
            "policy",
            Some(site),
            None,
            format!(
                "Fallow {:.2}, manure {:.2}, water storage {:.2}, market {:.0}",
                policy[0], policy[1], policy[2], policy[3]
            ),
        );
        Ok(())
    }
}

impl Generator {
    pub fn configure_economy(&mut self, catalog: EconomyCatalog) -> Result<()> {
        catalog.validate()?;
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(h.version == 2, "upgrade the economy first");
        if h.resources
            .as_ref()
            .is_some_and(|r| !r.mineral_catalog.is_empty())
        {
            for (slot, id) in [
                (32, "hematite_ore"),
                (33, "magnetite_ore"),
                (34, "limonite_ore"),
            ] {
                ensure!(
                    catalog
                        .goods
                        .get(slot)
                        .is_some_and(|g| g.id == id && g.cnp == [0.; 3] && g.food_energy == 0.),
                    "cannot relabel active mineral inventory"
                );
            }
        }
        if h.resources.as_ref().is_some_and(|r| r.alloy_processing) {
            for r in &catalog.recipes {
                let raw: f32 = r.input[32..38].iter().sum();
                if raw > 0. {
                    let product = r.output[2] + r.output[38..45].iter().sum::<f32>();
                    ensure!(
                        r.work[3] + 1e-6 >= (raw - product).max(0.),
                        "identified smelting must retain its inorganic residue"
                    );
                }
            }

            for (slot, id) in crate::metallurgy::ALLOY_GOODS {
                ensure!(
                    catalog
                        .goods
                        .get(slot)
                        .is_some_and(|g| g.id == id && g.cnp == [0.; 3] && g.food_energy == 0.),
                    "cannot relabel active alloy inventory"
                );
            }
        }
        h.economy_catalog = Some(catalog);
        h.event(
            "craft_catalog",
            None,
            None,
            "Updated archived craft recipes and price definitions".into(),
        );
        Ok(())
    }
    pub fn spoil_site_food(&mut self, site: u32, kg: f32) -> Result<()> {
        ensure!(kg.is_finite() && kg >= 0., "invalid food loss");
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no civilizations"))?;
        let s = h
            .sites
            .get_mut(site as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown site"))?;
        let loss = kg.min(s.stocks.stock[1]);
        s.stocks.stock[1] -= loss;
        s.stocks.ledger[2] += loss;
        if h.version == 2 {
            s.economy.external[0] -= loss * 0.45;
            for k in 1..3 {
                let nutrient = loss * crate::economy::FOOD_CNP[k] as f32;
                let retained = nutrient * s.economy.policy[1];
                s.economy.detritus[k] += retained;
                s.economy.external[k] -= nutrient - retained;
            }
        }
        h.event(
            "food_spoilage",
            Some(site),
            None,
            format!("Lost {loss:.1} kg food; returned retained nutrients to compost"),
        );
        Ok(())
    }
}
impl Generator {
    pub(crate) fn validate_economic_grid(&self, h: &History) -> Result<()> {
        if h.version != 2 {
            return Ok(());
        }
        let n = self.config.resolution;
        let m = self.config.eco_resolution();
        for s in &h.sites {
            let id =
                s.cell / (n * n) * m * m + (s.cell / n % n) / (n / m) * m + (s.cell % n) / (n / m);
            let area =
                (grid::solid_angle(id, m) * (self.config.radius_km as f64 * 1000.).powi(2)) as f32;
            ensure!(
                s.economy.claim[0] == id as f32
                    && (s.economy.claim[2] - area).abs() / area < 0.00001
                    && (s.economy.claim[1] - s.stocks.habitat[1] * 10000.).abs()
                        / s.economy.claim[1].max(1.)
                        < 0.00001,
                "managed plot does not match ecological grid"
            );
        }
        Ok(())
    }
}

impl Generator {
    /// Start a coupled monthly clock without resetting either simulation's history.
    pub fn enable_living_history(&mut self) -> Result<()> {
        ensure!(
            self.progress.stage == Stage::Boundary,
            "finish the environmental epoch first"
        );
        let h = self
            .civilizations
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(
            h.version == 2,
            "upgrade ecological farming before enabling living history"
        );
        if h.living.is_some() {
            return self.validate_living_boundary();
        }
        self.initialize_history_boundary()?;
        let terrain = self.snapshot()?;
        let h = self.civilizations.as_mut().unwrap();
        h.candidates
            .retain(|c| crate::hazards::flood_depth(&terrain[c.cell as usize]) < 0.25);
        h.living = Some(LivingHistory {
            environmental_returns: false,
            history_start: h.month,
            ecology_start: self.ecology.clock.month,
            incomplete: false,
            floods: Default::default(),
        });
        h.event("living_world", None, None, "Seasonal environment and society now advance together; geological terrain remains fixed".into());
        self.ecology.living = true;
        self.reconcile_managed_land();
        self.validate_living_boundary()
    }

    pub(crate) fn validate_living_boundary(&self) -> Result<()> {
        ensure!(
            self.civilizations.is_some()
                || self
                    .ecology
                    .clock
                    .events
                    .iter()
                    .all(|e| e.history_month.is_none() && e.history_event.is_none()),
            "ecological intervention references missing history"
        );
        if let Some(h) = &self.civilizations {
            for event in &self.ecology.clock.events {
                match (event.history_month, event.history_event) {
                    (None, None) => {}
                    (Some(month), Some(id)) => {
                        let record = h.events.get(id as usize);
                        ensure!(
                            h.living.as_ref().is_some_and(|l| {
                                month.checked_sub(l.history_start).map(u64::from)
                                    == event.month.checked_sub(l.ecology_start)
                                    && month >= l.history_start
                            }) && month <= h.month
                                && event.month <= self.ecology.clock.month
                                && record.is_some_and(|r| r.id == id
                                    && r.month == month
                                    && r.kind == "ecological_intervention"),
                            "invalid coupled ecological intervention reference"
                        );
                    }
                    _ => anyhow::bail!("incomplete coupled ecological intervention reference"),
                }
            }
            if let Some(l) = &h.living {
                ensure!(
                    h.version == 2 && h.economy_catalog.is_some() && self.ecology.clock.initialized,
                    "living history requires initialized ecological farming"
                );
                ensure!(
                    !l.incomplete && h.sites.iter().all(|s| s.economy.return_flow == [0.; 4]),
                    "coupled month did not complete; reload the last checkpoint"
                );
                ensure!(
                    h.month >= l.history_start
                        && self.ecology.clock.month.checked_sub(l.ecology_start)
                            == Some(u64::from(h.month - l.history_start)),
                    "environment and history clocks disagree"
                );
            }
        }
        Ok(())
    }

    fn reconcile_managed_land(&mut self) {
        let mut fractions = std::collections::BTreeMap::<u32, f32>::new();
        for s in &self.civilizations.as_ref().unwrap().sites {
            if s.economy.claim[3] == 1. {
                *fractions.entry(s.economy.claim[0] as u32).or_default() +=
                    if s.economy.land_return[1] > 0.5 {
                        0.
                    } else {
                        s.economy.claim[1] / s.economy.claim[2]
                    };
            }
        }
        for (cell, fraction) in fractions {
            self.gpu.queue.write_buffer(
                &self.ecology.buffers[self.ecology.current],
                u64::from(cell) * crate::ecology::ECO_BYTES + 24 * 16 + 12,
                bytemuck::bytes_of(&fraction.min(1.)),
            );
        }
        self.ecology.reconcile_plots(
            &self.gpu,
            &self.buffers[self.current],
            &self.catalog_buffer,
            &self.config,
            &self.catalog,
        );
    }

    pub fn advance_history(&mut self, months: u32) -> Result<()> {
        if self
            .civilizations
            .as_ref()
            .is_none_or(|h| h.living.is_none())
        {
            return self.advance_history_snapshot(months);
        }
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "history requires an environmental boundary"
        );
        let h = self.civilizations.as_ref().unwrap();
        ensure!(
            months <= 12000 && h.month.saturating_add(months) <= 120000,
            "history limit exceeded"
        );
        for _ in 0..months {
            self.civilizations
                .as_mut()
                .unwrap()
                .living
                .as_mut()
                .unwrap()
                .incomplete = true;
            let h = self.civilizations.as_ref().unwrap();
            let weather = &h.economy_catalog.as_ref().unwrap().weather;
            self.ecology.living_storms =
                [weather.storm_probability, weather.storm_multiplier, 0., 0.];
            self.ecology.living_weather = [
                weather.drought_probability.to_bits(),
                weather.drought_severity.to_bits(),
                weather.regime_months,
                h.month + 1,
            ];
            self.ecology.step(
                &self.gpu,
                &self.buffers[self.current],
                &self.catalog_buffer,
                &self.config,
                &self.catalog,
            )?;
            // Refresh every cell consumed this month, retaining static geography
            // between full survey refreshes. Drop the cache if the month fails.
            let mut environment = self.history_environment.take().unwrap_or_default();
            environment.refresh(self)?;
            let terrain = environment.for_month(self.civilizations.as_ref().unwrap().month + 1)?;
            self.civilizations
                .as_mut()
                .unwrap()
                .candidates
                .retain(|c| crate::hazards::flood_depth(&terrain[c.cell as usize]) < 0.25);
            self.advance_history_with_terrain(1, Some(terrain))?;
            self.commit_environmental_returns()?;
            self.reconcile_managed_land();
            self.civilizations.as_mut().unwrap().record_timeline();
            self.civilizations
                .as_mut()
                .unwrap()
                .living
                .as_mut()
                .unwrap()
                .incomplete = false;
            self.validate_living_boundary()?;
            self.history_environment = Some(environment);
        }
        Ok(())
    }
}

impl History {
    fn restore_returning_settlements(&mut self) {
        for i in 0..self.sites.len() {
            let s = &mut self.sites[i];
            if s.abandoned && s.stocks.stock[0] >= 1. {
                s.abandoned = false;
                s.lifecycle.depopulated_months = 0;
                s.lifecycle.pending_months = 0;
                s.lifecycle.pending_size = None;
                s.lifecycle.size = SettlementSize::from_population(s.stocks.stock[0]);
                self.event("settlement_reoccupied", Some(i as u32), None, "Returning residents reoccupied the site; existing people and property retained".into());
            }
        }
    }
    /// A full year below one person-equivalent resolves the last fractional cohort.
    /// Larger, even impoverished, households remain communities rather than disappearing.
    pub(crate) fn settlement_lifecycle_month(&mut self) {
        for i in 0..self.sites.len() {
            let s = &mut self.sites[i];
            if s.abandoned {
                continue;
            }
            let pop = s.stocks.stock[0];
            s.lifecycle.depopulated_months = if pop < 1. {
                s.lifecycle.depopulated_months.saturating_add(1)
            } else {
                0
            };
            if s.lifecycle.depopulated_months >= 12 {
                // Explicit demographic discretization, not a silent inventory deletion.
                s.stocks.people[1] += pop;
                s.demography.health[2] += pop;
                s.demography.ages[..3].fill(0.);
                s.stocks.stock[0] = 0.;
                s.stocks.stock[2] = 0.;
                s.abandoned = true;
                self.event("abandoned", Some(i as u32), None, format!("No viable resident cohort for 12 months; resolved {pop:.3} remaining person-equivalents in the mortality ledger. Buildings and stored goods remain as ruins"));
                continue;
            }
            let desired = SettlementSize::from_population(pop);
            // Ten percent separation from the current size's boundary plus two years
            // of persistence prevents a harvest, levy or temporary move changing status.
            let stable_pop = match s.lifecycle.size {
                SettlementSize::Town => pop < 90.,
                SettlementSize::Village => !(27. ..110.).contains(&pop),
                SettlementSize::Hamlet => !(4.5..33.).contains(&pop),
                SettlementSize::Remnant => pop >= 5.5,
            };
            if desired == s.lifecycle.size || !stable_pop {
                s.lifecycle.pending_size = None;
                s.lifecycle.pending_months = 0;
                continue;
            }
            if s.lifecycle.pending_size == Some(desired) {
                s.lifecycle.pending_months += 1;
            } else {
                s.lifecycle.pending_size = Some(desired);
                s.lifecycle.pending_months = 1;
            }
            if s.lifecycle.pending_months >= 24 {
                let old = s.lifecycle.size;
                s.lifecycle.size = desired;
                s.lifecycle.pending_size = None;
                s.lifecycle.pending_months = 0;
                self.event("settlement_reclassified", Some(i as u32), None, format!("{} became a {} after 24 months of sustained population change ({pop:.1} residents); political affiliation and property retained", old.label(), desired.label()));
            }
        }
    }
}

impl History {
    pub(crate) fn relief_arrivals(&mut self) {
        let arrivals = std::mem::take(&mut self.shipments);
        for mut shipment in arrivals {
            if shipment.arrives <= self.month
                && (self.flood_blocks_delivery(shipment.from, shipment.to, None)
                    || shipment.relief_route.is_some_and(|id| {
                        self.society.as_ref().is_none_or(|s| {
                            !s.routes[id as usize].open || s.routes[id as usize].flood_months > 0
                        })
                    }))
            {
                shipment.weather_delay_months = shipment.weather_delay_months.saturating_add(1);
                let lost = shipment.food_kg * 0.2;
                shipment.food_kg -= lost;
                self.lose_cargo(shipment.from, crate::economy::FOOD as u32, lost);
                if shipment.weather_delay_months >= 6 {
                    self.lose_cargo(shipment.from, crate::economy::FOOD as u32, shipment.food_kg);
                    self.event("relief_weather_lost", Some(shipment.to), Some(shipment.from),
                            "Relief journey terminated after six blocked months; remaining food written off".into());
                    self.events
                        .last_mut()
                        .unwrap()
                        .causes
                        .extend(shipment.appeal_cause);
                    let outcome = self.events.last().unwrap().id;
                    self.religious_relief_outcome(&shipment, false);
                    self.remember_relief(&shipment, false, outcome);
                    continue;
                }
                shipment.arrives = self.month + 1;
                self.shipments.push(shipment);
                continue;
            }
            if shipment.arrives <= self.month {
                if self.sites[shipment.to as usize].abandoned && shipment.appeal_cause.is_some() {
                    self.lose_cargo(shipment.from, crate::economy::FOOD as u32, shipment.food_kg);
                    self.event(
                        "appeal_relief_lost",
                        Some(shipment.to),
                        Some(shipment.from),
                        "No community remains to receive the relief; food written off".into(),
                    );
                    self.events
                        .last_mut()
                        .unwrap()
                        .causes
                        .extend(shipment.appeal_cause);
                    let outcome = self.events.last().unwrap().id;
                    self.religious_relief_outcome(&shipment, false);
                    self.remember_relief(&shipment, false, outcome);
                    continue;
                }
                self.sites[shipment.to as usize].stocks.stock[1] += shipment.food_kg;
                self.event(
                    "arrival",
                    Some(shipment.to),
                    Some(shipment.from),
                    format!("Received {:.0} kg food", shipment.food_kg),
                );
                self.events
                    .last_mut()
                    .unwrap()
                    .causes
                    .extend(shipment.appeal_cause);
                let outcome = self.events.last().unwrap().id;
                self.religious_relief_outcome(&shipment, true);
                self.remember_relief(&shipment, true, outcome);
            } else {
                self.shipments.push(shipment);
            }
        }
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::{catalog::Catalog, config::Config, gpu::ContextGpu};

    #[test]
    #[ignore = "requires hardware GPU"]
    fn living_history_reads_terrain_once_per_month() {
        use std::sync::atomic::Ordering::Relaxed;
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
        g.advance_ecology().unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.terrain_snapshot_count.store(0, Relaxed);
        g.advance_history(2).unwrap();
        assert_eq!(
            g.terrain_snapshot_count.load(Relaxed),
            1,
            "frozen history needs only one terrain snapshot per batch"
        );
        g.enable_living_history().unwrap();
        g.set_history_readback_mode(crate::history_environment::HistoryReadbackMode::Full);
        g.terrain_snapshot_count.store(0, Relaxed);
        g.advance_history(0).unwrap();
        assert_eq!(g.terrain_snapshot_count.load(Relaxed), 0);
        g.advance_history(3).unwrap();
        assert_eq!(
            g.terrain_snapshot_count.load(Relaxed),
            3,
            "each living month needs one fresh snapshot, shared by flood checks and history"
        );
        g.advance_history(1).unwrap();
        assert_eq!(
            g.terrain_snapshot_count.load(Relaxed),
            4,
            "a later call must not reuse stale terrain"
        );
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn decline_recovery_harvest_and_ruined_societies() {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
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
        h.sites[0].stocks.stock[0] = 25.;
        for _ in 0..23 {
            h.settlement_lifecycle_month();
        }
        assert_eq!(h.sites[0].lifecycle.size, SettlementSize::Town);
        h.settlement_lifecycle_month();
        assert_eq!(h.sites[0].lifecycle.size, SettlementSize::Hamlet);
        for pop in [31., 29., 32., 30.].into_iter().cycle().take(48) {
            h.sites[0].stocks.stock[0] = pop;
            h.settlement_lifecycle_month();
        }
        assert_eq!(h.sites[0].lifecycle.size, SettlementSize::Hamlet);
        h.sites[0].stocks.stock[0] = 120.;
        for _ in 0..24 {
            h.settlement_lifecycle_month();
        }
        assert_eq!(h.sites[0].lifecycle.size, SettlementSize::Town);
        h.sites[0].stocks.stock[0] = 0.75;
        for _ in 0..11 {
            h.settlement_lifecycle_month();
        }
        assert!(!h.sites[0].abandoned);
        h.sites[0].stocks.stock[0] = 2.;
        h.settlement_lifecycle_month();
        assert_eq!(h.sites[0].lifecycle.depopulated_months, 0);
        h.sites[0].stocks.stock[0] = 0.5;
        let deaths = h.sites[0].stocks.people[1];
        let food = h.sites[0].stocks.stock[1];
        for _ in 0..12 {
            h.settlement_lifecycle_month();
        }
        assert!(h.sites[0].abandoned);
        assert_eq!(h.sites[0].stocks.stock[0], 0.);
        assert_eq!(h.sites[0].stocks.people[1], deaths + 0.5);
        assert_eq!(h.sites[0].stocks.stock[1], food);
        h.sites[0].stocks.stock[0] = 10.;
        h.restore_returning_settlements();
        assert!(!h.sites[0].abandoned);
        assert_eq!(h.sites[0].stocks.stock[0], 10.);
        h.month = 12;
        h.sites[0].economy.management[0] = 1.;
        h.sites[0].lifecycle.harvest_observed = Some([0.; 6]);
        h.sites[0].economy.crops[1][3] = 15.;
        h.sites[0].stocks.stock[2] = 999.; // Food total is not the crop harvest.
        h.social_month();
        let harvests = h
            .events
            .iter()
            .filter(|e| e.kind == "harvest" && e.site == Some(0))
            .collect::<Vec<_>>();
        assert_eq!(harvests.len(), 1);
        assert!(harvests[0].detail.contains("15.0 kg"));
        assert!(!harvests[0].detail.contains("999"));
        h.social_month();
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "harvest" && e.site == Some(0))
                .count(),
            1
        );
        // A ruined sacred site does not erase support in a living daughter community.
        h.sites[0].abandoned = true;
        h.sync_culture();
        let culture = h.culture.as_mut().unwrap();
        culture.traditions[0].sacred_site = 0;
        let hh = &h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == 1)
            .unwrap();
        culture.household_faith[hh.id as usize] = 0;
        h.month = 240;
        h.event(
            "food_crisis",
            Some(0),
            None,
            "Remembered founding hardship".into(),
        );
        h.culture_month();
        assert!(h
            .events
            .iter()
            .any(|e| e.kind == "religious_interpretation" && e.site == Some(1)));
        let mut h = h.clone();
        for s in &mut h.sites {
            s.abandoned = true;
            s.demography.health[2] = 100.;
            s.stocks.stock[2] = 999.;
        }
        let start = h.events.len();
        for month in 1..=240 {
            h.month = 960 + month;
            h.social_month();
            h.genealogy_month();
            h.culture_month();
        }
        assert!(h.events[start..].iter().all(|e| !matches!(
            e.kind.as_str(),
            "harvest"
                | "marriage"
                | "inheritance"
                | "succession"
                | "faith_adopted"
                | "religious_schism"
                | "religious_syncretism"
                | "religious_interpretation"
        )));
        let value = serde_json::to_value(&h.sites[0]).unwrap();
        let restored: Site = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(
            restored.lifecycle.harvest_observed,
            h.sites[0].lifecycle.harvest_observed
        );
        let mut old = value;
        old.as_object_mut().unwrap().remove("lifecycle");
        assert!(serde_json::from_value::<Site>(old)
            .unwrap()
            .lifecycle
            .harvest_observed
            .is_none());
    }
}

#[cfg(test)]
mod material_integration_tests {
    use super::*;
    #[test]
    #[ignore = "requires a GPU"]
    fn container_substitution_orders_reach_actual_gpu_production() {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        let mut h = g.civilizations.as_ref().unwrap().clone();
        h.society = None;
        h.culture = None;
        h.month = 1;
        let catalog = h.economy_catalog.as_mut().unwrap();
        catalog.production.workshops = false;
        catalog.production.food_security_labor = false;
        let prices = std::array::from_fn(|g| catalog.goods[g].base_price);
        for s in &mut h.sites {
            s.stocks.stock[0] = 100.;
            s.economy = Economy::default();
            s.economy.prices = prices;
            s.economy.goods[2] = 200.;
            s.economy.finance[0] = 10000.;
            s.economy.claim = [1., 1000., 1000., 1.];
            s.economy.initial[2] = 200.;
        }
        h.plan_production();
        assert!(h.sites.iter().all(|s| s.economy.targets[46] > 0.));
        g.civilizations = Some(h.clone());
        let engine = Engine::new(&g).unwrap();
        engine.upload(&g, &h);
        engine.dispatch(&g, false, h.sites.len() as u32);
        engine.read(&g, &mut h, true).unwrap();
        for s in &h.sites {
            let e = &s.economy;
            eprintln!(
                "metal-only fixture: vessels {}, metal remaining {}, metal used {}, waste {}",
                e.made[46], e.goods[2], e.used[2], e.reserves[3]
            );
            assert!(e.made[46] > 0.);
            assert_eq!(e.made[45], 0.);
            assert!((e.goods[2] + e.used[2] - 200.).abs() < 0.001);
            assert!(e.used[2] >= e.made[46]);
            assert!(e.valid());
        }
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn tools_change_actual_extraction_without_changing_source_mass() {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        let engine = Engine::new(&g).unwrap();
        let mut h = g.civilizations.as_ref().unwrap().clone();
        h.society = None;
        h.month = 1;
        let base = h.sites[0].clone();
        for (j, s) in h.sites.iter_mut().enumerate() {
            s.stocks = base.stocks;
            s.stocks.stock[0] = 100.;
            s.economy = Economy::default();
            let e = &mut s.economy;
            e.claim = [1., 1000., 0., 1.];
            e.forest = [500., 2., 0.2, 0.];
            e.reserves = [0., 1000., 0., 0.];
            e.extraction = [1., 0., 0.8, 0.];
            e.logistics = [100000., 0., 0., 1.];
            e.targets[0] = 1000.;
            e.targets[1] = 1000.;
            if j == 1 {
                e.goods[49] = 100.;
            } // pick only: mining improves, forestry must not.
            if j == 2 {
                e.goods[48] = 100.;
            } // axe only: forestry improves, mining must not.
            if j == 3 {
                e.goods[49] = 100.;
                e.extraction[3] = 1.;
            } // depleted deposit harder despite same tool.
        }
        engine.upload(&g, &h);
        engine.dispatch(&g, false, h.sites.len() as u32);
        engine.read(&g, &mut h, true).unwrap();
        let e: Vec<_> = h.sites.iter().map(|s| s.economy).collect();
        assert!(e[1].made[1] > e[0].made[1]);
        assert_eq!(e[1].made[0], e[0].made[0]);
        assert!(e[2].made[0] > e[0].made[0]);
        assert_eq!(e[2].made[1], e[0].made[1]);
        assert!(e[3].made[1] < e[1].made[1]);
        for x in &e {
            assert!((x.reserves[1] + x.made[1] - 1000.).abs() < 0.001);
            assert!((x.forest[0] + x.made[0] * 0.5 - 500.).abs() < 0.001);
        }
        // Analytical hand-tool rate: 5/(1+0.8/0.2)=1 kg per worker-month.
        assert!((e[0].made[1] - e[0].labor[2]).abs() < 0.001);
    }
}
