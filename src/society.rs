//! Sparse households, institutions and route graphs; demographic and crop stocks stay on GPU.
use crate::{
    civilization::{History, Person},
    economy::FOOD_CNP,
    gpu::{Cell, Generator},
    grid,
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, BinaryHeap};
// Inverse multiplication can round above the food used to bound recruitment.
// Debit and carry the same bounded quantity; never repair an overdraw by minting food.
fn raid_muster(adults: f32, available_food: f32, months: u32) -> (f32, f32) {
    let men = (adults * 0.1).min(available_food / (18. * (months + 1) as f32));
    let food = (men * 18. * (months + 1) as f32).min(available_food);
    (men, food)
}
#[cfg(test)]
#[test]
fn ration_limited_muster_cannot_overdraw_by_rounding() {
    let available = f32::from_bits(0x434800c5);
    let (men, carried) = raid_muster(100., available, 1);
    assert!(
        men * 18. * 2. > available,
        "fixture must expose inverse-rounding overdraw"
    );
    assert_eq!(carried, available);
    assert_eq!(available - carried, 0.);
    let (men, carried) = raid_muster(40., 1000., 1);
    assert_eq!((men, carried), (4., 144.));
}
#[repr(C)]
#[derive(
    Clone, Copy, Debug, Default, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable,
)]
pub struct Demography {
    /// Children <15, working-age adults 15–59, elders; latest GPU regional weather multiplier.
    pub ages: [f32; 4],
    /// Standing grain kg, seed grain kg, harvest calendar month, annual seed viability.
    pub crops: [f32; 4],
    /// Disease burden, consecutive shortage months, unassigned mortality credit, last crop growth kg.
    pub health: [f32; 4],
    /// Additional ration priority, child/adult/elder, 0–3; zero means equal sufficiency.
    #[serde(default)]
    pub ration_priority: [f32; 4],
    /// Last production month's food need and allocation (kg calorie equivalent).
    #[serde(default)]
    pub ration_need: [f32; 4],
    #[serde(default)]
    pub ration_eaten: [f32; 4],
    #[serde(default)]
    /// Retail entitlement cap, enabled flag, last pre-consumption available food, reserved.
    pub household_food: [f32; 4],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Household {
    pub id: u32,
    pub site: u32,
    pub name: String,
    /// Beneficial ownership of the site's private stocks, not additional inventory.
    pub share: f64,
    /// Current head, or the last deceased head while the account is vacant.
    pub head: u32,
    #[serde(default)]
    pub vacant_since: Option<u32>,
    pub founded: u32,
    pub parent: Option<u32>,
    pub generation: u32,
}
/// A chosen rate is announced now and becomes active at a later monthly boundary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendingTaxPolicy {
    pub rate: f32,
    pub decided_month: u32,
    pub effective_month: u32,
    #[serde(default)]
    pub cause: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Council {
    #[serde(default)]
    pub distribution: Option<crate::household_economy::policy::DistributionPolicy>,
    #[serde(default)]
    pub pending_distribution: Option<crate::household_economy::policy::PendingDistribution>,
    #[serde(default)]
    pub distribution_review: Option<u32>,
    pub civilization: u32,
    pub treasury: f64,
    pub tax_rate: f32,
    #[serde(default)]
    pub pending_tax: Option<PendingTaxPolicy>,
    /// None means an inherited baseline whose activation date is unknown.
    #[serde(default)]
    pub tax_effective_since: Option<u32>,
    pub relief_paid: f64,
}
impl Council {
    fn plan_tax(&mut self, rate: f32, month: u32) -> bool {
        if self.pending_tax.as_ref().is_some_and(|p| p.rate == rate) {
            return false;
        }
        if self.tax_rate == rate {
            self.pending_tax = None;
            return false;
        }
        self.pending_tax = Some(PendingTaxPolicy {
            rate,
            decided_month: month,
            effective_month: month + 1,
            cause: None,
        });
        true
    }

    fn activate_tax(&mut self, month: u32) -> Option<PendingTaxPolicy> {
        if self
            .pending_tax
            .as_ref()
            .is_none_or(|p| p.effective_month > month)
        {
            return None;
        }
        let policy = self.pending_tax.take().unwrap();
        self.tax_rate = policy.rate;
        self.tax_effective_since = Some(month);
        Some(policy)
    }
}

impl History {
    pub(crate) fn schedule_tax_policy(&mut self, civilization: usize, rate: f32) {
        let previous = self.society.as_ref().unwrap().councils[civilization]
            .pending_tax
            .clone();
        if self.society.as_mut().unwrap().councils[civilization].plan_tax(rate, self.month) {
            self.event(
                "tax_policy_scheduled",
                None,
                None,
                format!(
                    "Civilization {civilization} adopted a {:.1}% council rate effective month {}",
                    rate * 100.,
                    self.month + 1
                ),
            );
            self.events
                .last_mut()
                .unwrap()
                .subjects
                .push(("civilization".into(), civilization as u32));
            self.society.as_mut().unwrap().councils[civilization]
                .pending_tax
                .as_mut()
                .unwrap()
                .cause = Some(self.events.last().unwrap().id);
            self.events
                .last_mut()
                .unwrap()
                .causes
                .extend(previous.and_then(|p| p.cause));
        } else if previous.is_some()
            && self.society.as_ref().unwrap().councils[civilization]
                .pending_tax
                .is_none()
        {
            self.event("tax_policy_cancelled", None, None,
                format!("Civilization {civilization} retained its active council rate; pending change withdrawn"));
            let event = self.events.last_mut().unwrap();
            event
                .subjects
                .push(("civilization".into(), civilization as u32));
            event.causes.extend(previous.and_then(|p| p.cause));
        }
    }

    pub(crate) fn activate_monthly_policies(&mut self) {
        self.activate_distribution();
        let changes: Vec<_> = self.society.as_mut().map_or_else(Vec::new, |s| {
            s.councils
                .iter_mut()
                .filter_map(|c| {
                    c.activate_tax(self.month)
                        .map(|rate| (c.civilization, rate))
                })
                .collect()
        });
        for (id, policy) in changes {
            self.event(
                "tax_policy_effective",
                None,
                None,
                format!(
                    "Civilization {id} council rate is now {:.1}%",
                    policy.rate * 100.
                ),
            );
            self.events
                .last_mut()
                .unwrap()
                .subjects
                .push(("civilization".into(), id));
            self.events.last_mut().unwrap().causes.extend(policy.cause);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Route {
    #[serde(default)]
    pub upkeep: Option<crate::road_upkeep::RoadUpkeep>,
    pub id: u32,
    pub from: u32,
    pub to: u32,
    pub cells: Vec<u32>,
    pub cost_km: f32,
    pub open: bool,
    #[serde(default)]
    pub flood_months: u32,
    pub road_bricks: f64,
}
impl Route {
    /// Political access and temporary physical access must both permit travel.
    pub fn passable(&self) -> bool {
        self.open && self.flood_months == 0
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Raid {
    /// Living shared person IDs. None preserves legacy aggregate armies on import.
    #[serde(default)]
    pub members: Option<Vec<u32>>,
    /// Fractional expected losses carried until a whole person is lost.
    #[serde(default)]
    pub loss_remainder: f32,
    /// Outbound itinerary duration; retained for the return leg, including multi-hop routes.
    #[serde(default)]
    pub travel_months: u32,
    /// Stationed at the target, still outside either settlement population.
    #[serde(default)]
    pub occupation_until: Option<u32>,
    pub id: u32,
    pub origin: u32,
    pub target: u32,
    pub soldiers: f32,
    pub food: f32,
    pub arrives: u32,
    pub cause: u64,
    pub returning: bool,
    #[serde(default)]
    pub war: Option<u32>,
    #[serde(default)]
    pub equipment: f32,
}
/// Observational totals at actual council payment boundaries. Currency is abstract.
/// Imported histories start a new diagnostic baseline; these never drive decisions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CouncilFunding {
    /// Latest annual collection boundary; observations never supply spendable revenue.
    #[serde(default)]
    pub taxes: Vec<TaxReceipt>,
    pub administration: FundingTotals,
    pub roads: FundingTotals,
    pub emergency_town_support: FundingTotals,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaxReceipt {
    pub month: u32,
    pub site: u32,
    pub council: u32,
    pub opening_cash: f32,
    pub rate: f32,
    pub autonomy: f32,
    pub office_capacity: f32,
    pub paid: f32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FundingTotals {
    pub requested: f64,
    pub paid: f64,
    pub shortfall: f64,
    pub requests: u64,
    /// Count requests below 90% funding, matching the administrative crisis threshold.
    pub underfunded: u64,
}
impl FundingTotals {
    pub(crate) fn record(&mut self, requested: f64, paid: f64) {
        self.requested += requested;
        self.paid += paid;
        self.shortfall += (requested - paid).max(0.);
        if requested > 0. {
            self.requests += 1;
            self.underfunded += u64::from(paid / requested < 0.9);
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Society {
    #[serde(default)]
    pub council_funding: CouncilFunding,
    #[serde(default)]
    pub household_economy: Option<crate::household_economy::HouseholdEconomy>,
    #[serde(default)]
    pub indicators: Option<crate::social_state::SocialState>,
    pub version: u32,
    pub started: u32,
    pub households: Vec<Household>,
    pub councils: Vec<Council>,
    pub routes: Vec<Route>,
    pub routed_sites: u32,
    pub raids: Vec<Raid>,
    pub next_raid: u32,
    #[serde(default)]
    pub relocation: crate::relocation::RelocationState,
}
fn traversable(c: &Cell) -> bool {
    c.meta[0] == 2 && c.water[0] < 0.25
}
impl Society {
    pub fn validate(&self, h: &History, cells: &[Cell]) -> Result<()> {
        self.relocation.validate(h)?;
        let mut previous_tax_site = None;
        for t in &self.council_funding.taxes {
            ensure!(
                t.month <= h.month
                    && (t.site as usize) < h.sites.len()
                    && (t.council as usize) < self.councils.len()
                    && previous_tax_site.is_none_or(|p| p < t.site),
                "invalid tax observation boundary"
            );
            ensure!(
                [
                    t.opening_cash,
                    t.rate,
                    t.autonomy,
                    t.office_capacity,
                    t.paid
                ]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.)
                    && t.rate <= 0.25
                    && t.autonomy <= 1.
                    && t.office_capacity <= 1.
                    && t.paid
                        == t.opening_cash * t.rate * (1. - t.autonomy * 0.75) * t.office_capacity,
                "invalid tax collection observation"
            );
            previous_tax_site = Some(t.site);
        }
        if let Some(e) = &self.household_economy {
            e.validate(h)?;
        }
        if let Some(state) = &self.indicators {
            state.validate(h)?;
        }
        ensure!(
            self.version == 1
                && self.started <= h.month
                && self.routed_sites as usize <= h.sites.len(),
            "invalid social clock"
        );
        ensure!(
            self.councils.len() == h.civilizations.len()
                && self
                    .councils
                    .iter()
                    .enumerate()
                    .all(|(i, c)| c.civilization == i as u32
                        && c.distribution.is_none_or(|p| p.valid())
                        && c.distribution_review.is_none_or(|m| m <= h.month)
                        && c.pending_distribution
                            .as_ref()
                            .is_none_or(|p| p.policy.valid()
                                && p.decided <= h.month
                                && p.effective == p.decided.saturating_add(1)
                                && p.effective > h.month
                                && h.events.iter().any(|e| e.id == p.cause))
                        && c.treasury.is_finite()
                        && c.treasury >= 0.
                        && c.relief_paid.is_finite()
                        && c.relief_paid >= 0.
                        && (0. ..=0.25).contains(&c.tax_rate)
                        && c.tax_effective_since.is_none_or(|m| m <= h.month)
                        && c.pending_tax
                            .as_ref()
                            .is_none_or(|p| (0. ..=0.25).contains(&p.rate)
                                && p.decided_month <= h.month
                                && p.effective_month == p.decided_month.saturating_add(1)
                                && p.effective_month > h.month)),
            "invalid council"
        );
        ensure!(
            self.households.len() <= 8192
                && self
                    .households
                    .iter()
                    .enumerate()
                    .all(|(i, f)| f.id == i as u32
                        && (f.site as usize) < h.sites.len()
                        && h.people
                            .get(f.head as usize)
                            .is_some_and(|p| match f.vacant_since {
                                Some(month) =>
                                    month <= h.month
                                        && month >= f.founded
                                        && p.died.is_some_and(|d| d <= month),
                                None => p.died.is_none(),
                            })
                        && f.share.is_finite()
                        && f.share > 0.
                        && f.founded <= h.month
                        && f.parent.is_none_or(|p| p < f.id)),
            "invalid household or lineage"
        );
        let heads = self
            .households
            .iter()
            .map(|f| f.head)
            .collect::<BTreeSet<_>>();
        ensure!(
            heads.len() == self.households.len(),
            "person heads multiple households"
        );
        for s in &h.sites {
            let fraction: f64 = self
                .households
                .iter()
                .filter(|f| f.site == s.id)
                .map(|f| f.share)
                .sum();
            ensure!(
                (fraction - 1.).abs() < 0.00001,
                "household ownership does not sum to one"
            );
            ensure!(
                s.demography
                    .ages
                    .iter()
                    .chain(&s.demography.crops)
                    .chain(&s.demography.health)
                    .chain(&s.demography.ration_need)
                    .chain(&s.demography.ration_eaten)
                    .chain(&s.demography.household_food)
                    .all(|v| v.is_finite() && *v >= 0.)
                    && s.demography
                        .ration_priority
                        .iter()
                        .all(|v| v.is_finite() && (0. ..=3.).contains(v))
                    && (0..4)
                        .all(|k| s.demography.ration_eaten[k] <= s.demography.ration_need[k] + 0.01)
                    && s.demography.household_food[1] <= 1.
                    && s.demography.crops[2] < 12.
                    && s.demography.crops[2].fract() == 0.
                    && s.demography.crops[3] <= 1.
                    && s.demography.health[0] <= 1.
                    && ((s.demography.ages[..3].iter().sum::<f32>() - s.stocks.stock[0])
                        / s.stocks.stock[0].max(1.))
                    .abs()
                        < 0.0001,
                "age cohorts do not match residents at site {} month {}: {:?}, population {}",
                s.id,
                h.month,
                s.demography.ages,
                s.stocks.stock[0]
            );
        }
        for (i, r) in self.routes.iter().enumerate() {
            ensure!(
                r.id == i as u32
                    && (r.from as usize) < h.sites.len()
                    && (r.to as usize) < h.sites.len()
                    && r.from != r.to
                    && r.cost_km.is_finite()
                    && r.cost_km > 0.
                    && r.road_bricks.is_finite()
                    && r.road_bricks >= 0.
                    && r.upkeep.as_ref().is_none_or(|c| c.observed <= h.month
                        && c.last_build.is_none_or(|m| m <= h.month)
                        && c.lost_kg.is_finite()
                        && c.lost_kg >= 0.
                        && c.work.is_finite()
                        && c.work >= 0.),
                "invalid route"
            );
            ensure!(
                r.cells.first() == Some(&h.sites[r.from as usize].cell)
                    && r.cells.last() == Some(&h.sites[r.to as usize].cell)
                    && r.cells
                        .iter()
                        .all(|&c| cells
                            .get(c as usize)
                            .is_some_and(|c| if h.living.is_some() {
                                c.meta[0] == 2
                            } else {
                                traversable(c)
                            })),
                "route leaves dry central land"
            );
            ensure!(
                r.cells
                    .windows(2)
                    .all(|p| [(-1, 0), (1, 0), (0, -1), (0, 1)]
                        .iter()
                        .any(|&(x, y)| grid::neighbor(p[0], h.terrain_resolution, x, y) == p[1])),
                "route jumps across terrain"
            );
        }
        ensure!(
            self.raids
                .iter()
                .map(|r| r.id)
                .collect::<BTreeSet<_>>()
                .len()
                == self.raids.len()
                && self.raids.iter().all(|r| r.id < self.next_raid
                    && r.origin != r.target
                    && (r.origin as usize) < h.sites.len()
                    && (r.target as usize) < h.sites.len()
                    && r.war.is_none_or(|id| h
                        .politics
                        .as_ref()
                        .is_some_and(|p| (id as usize) < p.wars.len()))
                    && r.equipment.is_finite()
                    && r.equipment >= 0.
                    && r.soldiers.is_finite()
                    && r.soldiers >= 0.
                    && r.food.is_finite()
                    && r.food >= 0.
                    && r.travel_months <= 10
                    && r.occupation_until
                        .is_none_or(|m| m == r.arrives && !r.returning && r.war.is_some())
                    && r.arrives > h.month
                    && (r.cause as usize) < h.events.len()),
            "invalid expedition"
        );
        Ok(())
    }
}
impl History {
    pub fn route_cost(&self, a: u32, b: u32) -> Option<f32> {
        let s = self.society.as_ref()?;
        if let Some(p) = &self.politics {
            let ca = self.controller(a);
            let cb = self.controller(b);
            if p.wars.iter().any(|w| {
                w.ended.is_none()
                    && ((w.attacker == ca && w.defender == cb)
                        || (w.attacker == cb && w.defender == ca))
            }) {
                return None;
            }
        }
        s.routes
            .iter()
            .filter(|r| r.passable() && ((r.from == a && r.to == b) || (r.from == b && r.to == a)))
            .map(|r| r.cost_km / (1. + (r.road_bricks / 1000.).min(1.) as f32))
            .min_by(f32::total_cmp)
    }
    /// All-pairs commercial distances over surveyed roads. Military routes remain direct.
    /// A merchant cannot enter an administration at war with its origin or cross a hostile edge.
    pub fn trade_distances(&self) -> Option<Vec<f32>> {
        self.society.as_ref()?;
        let n = self.sites.len();
        let mut result = vec![f32::INFINITY; n * n];
        for origin in 0..n {
            result[origin * n..(origin + 1) * n]
                .copy_from_slice(&self.road_distances_from(origin, self.controller(origin as u32)));
        }
        Some(result)
    }
    /// Cargo retains its origin administration's transit restrictions after a sea leg.
    pub(crate) fn road_distances_from(&self, origin: usize, administration: u32) -> Vec<f32> {
        self.road_tree_from(origin, administration).0
    }
    /// Stable predecessor tree shared by commercial distances and freight reservations.
    pub(crate) fn road_tree_from(
        &self,
        origin: usize,
        administration: u32,
    ) -> (Vec<f32>, Vec<u32>) {
        let n = self.sites.len();
        let mut parents = vec![u32::MAX; n];
        let empty = vec![f32::INFINITY; n];
        let Some(society) = &self.society else {
            return (empty, parents);
        };
        if self.sites[origin].abandoned || self.sites[origin].economy.policy[3] < 0.5 {
            return (empty, parents);
        }
        let hostile = |a: u32, b: u32| {
            self.politics.as_ref().is_some_and(|p| {
                p.wars.iter().any(|w| {
                    w.ended.is_none()
                        && ((w.attacker == a && w.defender == b)
                            || (w.attacker == b && w.defender == a))
                })
            })
        };
        let mut distances = vec![f32::INFINITY; n];
        let mut visited = vec![false; n];
        distances[origin] = 0.;
        for _ in 0..n {
            let Some(u) = (0..n)
                .filter(|&i| !visited[i] && distances[i].is_finite())
                .min_by(|&a, &b| distances[a].total_cmp(&distances[b]).then(a.cmp(&b)))
            else {
                break;
            };
            visited[u] = true;
            for r in society
                .routes
                .iter()
                .filter(|r| r.passable() && (r.from as usize == u || r.to as usize == u))
            {
                let v = if r.from as usize == u {
                    r.to as usize
                } else {
                    r.from as usize
                };
                if self.sites[v].abandoned
                    || self.sites[v].economy.policy[3] < 0.5
                    || hostile(administration, self.controller(v as u32))
                    || hostile(self.controller(u as u32), self.controller(v as u32))
                {
                    continue;
                }
                let cost = r.cost_km / (1. + (r.road_bricks / 1000.).min(1.) as f32);
                if distances[u] + cost < distances[v] {
                    distances[v] = distances[u] + cost;
                    parents[v] = u as u32;
                }
            }
        }
        (distances, parents)
    }
    pub(crate) fn prepare_society_with_navigation(
        &mut self,
        cells: &[Cell],
        radius: f32,
        navigation: Option<&crate::navigation::Navigation>,
    ) -> Result<()> {
        let Some(mut society) = self.society.take() else {
            return Ok(());
        };
        for site in &mut self.sites {
            if !society.households.iter().any(|f| f.site == site.id) {
                if site.demography.ages[..3].iter().sum::<f32>() == 0. {
                    site.demography.ages = [
                        site.stocks.stock[0] * 0.3,
                        site.stocks.stock[0] * 0.6,
                        site.stocks.stock[0] * 0.1,
                        0.,
                    ];
                }
                let seed = site.stocks.stock[1].min(site.stocks.stock[0]);
                site.stocks.stock[1] -= seed;
                site.demography.crops = [0., seed, 8., 1.];
                let dir = grid::cell_direction(site.cell, self.terrain_resolution);
                if dir[1] < 0. {
                    site.demography.crops[2] = 2.;
                }
                let count = ((site.stocks.stock[0] / 8.).ceil() as usize).clamp(1, 24);
                for family in 0..count {
                    let id = society.households.len() as u32;
                    let leader = self.civilizations[site.civilization as usize].leader;
                    let head =
                        if family == 0 && !society.households.iter().any(|f| f.head == leader) {
                            leader
                        } else {
                            let person = self.people.len() as u32;
                            self.people.push(Person {
                                id: person,
                                name: self.civilizations[site.civilization as usize]
                                    .naming(self.seed)
                                    .person_with(
                                        "person",
                                        person,
                                        &crate::naming::PersonalContext::local(
                                            site,
                                            self.culture.as_ref(),
                                        ),
                                    ),
                                civilization: site.civilization,
                                born: self.month as i32 - 360 - (family as i32 % 20) * 12,
                                died: None,
                                predecessor: None,
                            });
                            person
                        };
                    society.households.push(Household {
                        id,
                        site: site.id,
                        name: self.civilizations[site.civilization as usize]
                            .naming(self.seed)
                            .coin(
                                &format!("household:{id}"),
                                &["house"],
                                Some(crate::naming::Source {
                                    kind: "person".into(),
                                    id: head,
                                    name: self.people[head as usize].name.clone(),
                                }),
                            ),
                        share: 1. / count as f64,
                        head,
                        vacant_since: None,
                        founded: self.month,
                        parent: None,
                        generation: 0,
                    });
                }
            }
        }
        // Sparse paths are computed only when sites appear, never once per simulation month.
        let n = self.terrain_resolution;
        for target in society.routed_sites as usize..self.sites.len() {
            let mut neighbors = (0..target)
                .filter(|&j| self.sites[j].island == self.sites[target].island)
                .collect::<Vec<_>>();
            neighbors.sort_by(|&a, &b| {
                crate::civilization::distance(self.sites[a].cell, self.sites[target].cell, n)
                    .total_cmp(&crate::civilization::distance(
                        self.sites[b].cell,
                        self.sites[target].cell,
                        n,
                    ))
            });
            for source in neighbors.into_iter().take(3) {
                if let Some((path, cost)) = match navigation {
                    Some(nav) => nav.route(
                        self.sites[source].cell,
                        Some(self.sites[target].cell),
                        crate::navigation::RouteKind::Road,
                    )?,
                    None => terrain_path(
                        self.sites[source].cell,
                        self.sites[target].cell,
                        n,
                        radius,
                        cells,
                    ),
                } {
                    society.routes.push(Route {
                        id: society.routes.len() as u32,
                        from: source as u32,
                        to: target as u32,
                        cells: path,
                        cost_km: cost,
                        open: true,
                        flood_months: 0,
                        road_bricks: 0.,
                        upkeep: Some(crate::road_upkeep::RoadUpkeep::new(self.month)),
                    });
                }
            }
        }
        society.routed_sites = self.sites.len() as u32;
        self.society = Some(society);
        Ok(())
    }
    pub(crate) fn social_month(&mut self) -> Result<()> {
        let mut supply_comparison = crate::military_supply::SupplyComparison::capture(self)?;
        self.weather_roads();
        let heirs: std::collections::BTreeMap<_, _> = if self.named_demography.is_none() {
            self.society
                .as_ref()
                .map(|s| {
                    s.households
                        .iter()
                        .map(|f| (f.head, self.genealogical_heir(f.head, f.site)))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Default::default()
        };
        let successors = if self.named_demography.is_some()
            || self
                .society
                .as_ref()
                .is_some_and(|s| s.households.iter().any(|f| f.vacant_since.is_some()))
        {
            self.resident_successors()
        } else {
            Default::default()
        };
        let mut resident_defenders = vec![Vec::new(); self.sites.len()];
        if self.individual_demography_enabled() {
            for person in &self.people {
                if crate::population_registry::age_band(self.month, person.born) == Some(1) {
                    if let crate::participation::Presence::Resident(site) =
                        self.person_presence(person.id).1
                    {
                        resident_defenders[site as usize].push(person.id);
                    }
                }
            }
        }
        let mut slots = crate::population_registry::ResidentSlots::new(self);
        let mut occupied: BTreeSet<u32> = self
            .society
            .as_ref()
            .map(|s| s.households.iter().map(|f| f.head).collect())
            .unwrap_or_default();
        let Some(mut society) = self.society.take() else {
            return Ok(());
        };
        for i in 0..self.sites.len() {
            if self.sites[i].abandoned {
                continue;
            }
            let d = self.sites[i].demography;
            // GPU weather factor is stored in the spare cohort component. Events describe
            // forcing transitions; they do not inject hunger, deaths or political outcomes.
            let regime = self
                .economy_catalog
                .as_ref()
                .map_or(48, |c| c.weather.regime_months);
            if !self.sites[i].abandoned
                && ((self.month - 1) % regime == 0 || self.sites[i].founded + 1 == self.month)
            {
                let kind = if d.ages[3] < 0.999 {
                    "regional_drought"
                } else {
                    "weather_recovery"
                };
                let previous = self.events.iter().rev().find(|e| {
                    e.site == Some(i as u32)
                        && matches!(e.kind.as_str(), "regional_drought" | "weather_recovery")
                });
                if previous.map_or(kind == "regional_drought", |e| e.kind != kind) {
                    let cause = previous.map(|e| e.id);
                    self.event(
                        kind,
                        Some(i as u32),
                        None,
                        format!(
                            "Regional rainfall and crop potential at {:.0}% of ordinary weather",
                            d.ages[3] * 100.
                        ),
                    );
                    if let Some(cause) = cause {
                        self.events.last_mut().unwrap().causes.push(cause);
                    }
                }
            }
            if d.health[1] == 3. {
                self.event(
                    "food_crisis",
                    Some(i as u32),
                    None,
                    "Three consecutive hungry months; weakened health and reduced births".into(),
                );
            }
            if self.sites[i].economy.management[0] > 0.5 {
                let current = std::array::from_fn(|k| self.sites[i].economy.crops[k][3]);
                let previous = self.sites[i]
                    .lifecycle
                    .harvest_observed
                    .replace(current)
                    .unwrap_or(current);
                let harvested: Vec<String> = (0..6)
                    .filter_map(|k| {
                        let kg = current[k] - previous[k];
                        (kg > 0.01).then(|| {
                            let name = self
                                .economy_catalog
                                .as_ref()
                                .and_then(|c| c.goods.get(8 + k))
                                .map_or("crop", |g| g.name.as_str());
                            format!("{kg:.1} kg {name}")
                        })
                    })
                    .collect();
                if !harvested.is_empty() {
                    self.event(
                        "harvest",
                        Some(i as u32),
                        None,
                        format!(
                            "Crop harvest: {} (actual crop mass, before processing)",
                            harvested.join(", ")
                        ),
                    );
                }
            } else if self.sites[i].stocks.stock[2] > 0. && self.month % 12 == d.crops[2] as u32 {
                self.event(
                    "harvest",
                    Some(i as u32),
                    None,
                    format!(
                        "Harvested {:.1} kg grain; seed reserve {:.1} kg",
                        self.sites[i].stocks.stock[2], d.crops[1]
                    ),
                );
            }
        }

        let raids = std::mem::take(&mut society.raids);
        for mut raid in raids {
            let before_supply = raid.soldiers;
            self.advance_military_experience(&raid);
            let consume = raid.food.min(raid.soldiers * 18.);
            raid.food -= consume;
            let origin = &mut self.sites[raid.origin as usize];
            origin.stocks.ledger[1] += consume;
            for (k, v) in FOOD_CNP.iter().enumerate() {
                origin.economy.external[k] -= consume * *v as f32;
            }
            if consume + 0.001 < raid.soldiers * 18. {
                let expected = raid.soldiers * 0.1;
                self.military_losses(&mut raid, expected, "insufficient provisions");
            }
            supply_comparison.observe(&raid, raid.members.is_some(), before_supply, consume);
            if self.close_empty_army(&raid) {
                continue;
            }
            if self.occupation_month(&mut raid, &society) {
                society.raids.push(raid);
                continue;
            }
            if raid.arrives <= self.month && raid.returning {
                self.return_military_people(&raid);
                let site = &mut self.sites[raid.origin as usize];
                site.stocks.stock[1] += raid.food;
                site.economy.goods[3] += raid.equipment;
                self.event(
                    "raid_return",
                    Some(raid.origin),
                    Some(raid.target),
                    format!(
                        "Raid {} returned with {:.0} survivors and {:.0} kg food",
                        raid.id, raid.soldiers, raid.food
                    ),
                );
                self.events.last_mut().unwrap().causes.push(raid.cause);
                self.record_military_members(&raid);
            } else if raid.arrives <= self.month {
                if !self.campaign_authorized(&raid) {
                    raid.returning = true;
                    raid.arrives = self.month + raid.return_duration(&society);
                    self.resolve_war(&raid, false);
                    raid.cause = self.events.last().unwrap().id;
                    society.raids.push(raid);
                    continue;
                }
                let protection = self.patron_protection(raid.target);
                let defenders = self.sites[raid.target as usize].demography.ages[1] * 0.15;
                let won = raid.soldiers
                    * self.military_preparedness(&raid)
                    * (1. + (raid.equipment / raid.soldiers.max(1.)).min(1.) * 0.5)
                    > defenders * (1.1 + protection);
                let equipment_loss = raid.equipment * 0.15;
                raid.equipment -= equipment_loss;
                let expected = (defenders * 0.1).min(raid.soldiers * 0.2);
                let casualties = self.military_losses(&mut raid, expected, "combat");
                let expected_defense = (raid.soldiers * 0.05 * (1. - protection))
                    .min(self.sites[raid.target as usize].demography.ages[1]);
                let defender_losses = if self.individual_demography_enabled() {
                    let ids = self.individual_defender_losses(
                        raid.target,
                        expected_defense,
                        &resident_defenders[raid.target as usize],
                    );
                    for _ in &ids {
                        slots.observe(raid.target, 1, false);
                    }
                    if !ids.is_empty() {
                        self.events.last_mut().unwrap().causes.push(raid.cause);
                    }
                    ids.len() as f32
                } else {
                    expected_defense
                };
                let target = &mut self.sites[raid.target as usize];
                target.demography.ages[1] -= defender_losses;
                target.stocks.stock[0] -= defender_losses;
                target.stocks.people[1] += defender_losses;
                let loot = if raid.war.is_none() || won {
                    target.stocks.stock[1].min(raid.soldiers * 36.)
                } else {
                    0.
                };
                target.stocks.stock[1] -= loot;
                self.sites[raid.origin as usize].economy.used[3] += equipment_loss;
                self.sites[raid.origin as usize].economy.reserves[3] += equipment_loss;
                raid.food += loot;
                raid.returning = true;
                raid.arrives = self.month + raid.return_duration(&society);
                self.event(
                    "raid_outcome",
                    Some(raid.target),
                    Some(raid.origin),
                    format!(
                        "Raid {}: {:0.0} attackers and {:0.0} defenders lost; {:0.0} kg food taken",
                        raid.id, casualties, defender_losses, loot
                    ),
                );
                if let Some(e) = self.events.last_mut() {
                    e.causes.push(raid.cause);
                    raid.cause = e.id;
                }
                if self.close_empty_army(&raid) {
                    continue;
                }
                self.resolve_war(&raid, won);
                if won {
                    self.begin_occupation(&mut raid);
                }
                society.raids.push(raid);
            } else {
                society.raids.push(raid);
            }
        }
        // A named head's death uses accumulated cohort deaths, never an additional population decrement.
        for f in &mut society.households {
            if society.relocation.away(f.id) || self.person_on_service(f.head) {
                continue;
            }
            let site = &mut self.sites[f.site as usize];
            if site.abandoned && self.people[f.head as usize].died.is_none() {
                continue;
            }
            let old = f.head as usize;
            if self.people[old].died.is_some()
                || (self.named_demography.is_none()
                    && self.month as i32 - self.people[old].born >= 840
                    && site.demography.health[2] >= 1.)
            {
                // A recorded travel death already removed this person from the
                // travelling population; succession must not spend a local death twice.
                if self.people[old].died.is_none() {
                    site.demography.health[2] -= 1.;
                    self.people[old].died = Some(self.month);
                    if let Some(band) =
                        crate::population_registry::age_band(self.month, self.people[old].born)
                    {
                        slots.observe(f.site, band, false);
                    }
                }
                // Political identity survives occupation or household relocation.
                // The host site's administrator does not own this ruler's office.
                let polity = self.people[old].civilization;
                let ruler = self.civilizations[polity as usize].leader == old as u32;
                let successor_civilization = if ruler { polity } else { site.civilization };
                let resident_mode = self.named_demography.is_some() || f.vacant_since.is_some();
                let existing = if resident_mode {
                    successors.get(&f.id).and_then(|ids| {
                        ids.iter().copied().find(|id| {
                            !occupied.contains(id)
                                && (!ruler
                                    || self.people[*id as usize].civilization
                                        == successor_civilization)
                                && self.people[*id as usize].died.is_none()
                                && !self.person_duties.contains_key(id)
                                && !self.military.duties.contains_key(id)
                        })
                    })
                } else {
                    heirs.get(&(old as u32)).copied().flatten()
                }
                .filter(|id| {
                    !occupied.contains(id)
                        && (!ruler
                            || self.people[*id as usize].civilization == successor_civilization)
                        && self.people[*id as usize].died.is_none()
                        && !self.person_duties.contains_key(id)
                        && !self.military.duties.contains_key(id)
                });
                let id = existing.unwrap_or(self.people.len() as u32);
                // Identification may use a whole anonymous adult or elder slot.
                // Without one, retain the estate instead of fabricating a person.
                let band = [1, 2]
                    .into_iter()
                    .find(|&b| slots.available(f.site, b, site.demography.ages[b]) > 0);
                if existing.is_none() && band.is_none() && resident_mode {
                    if f.vacant_since.is_none() {
                        f.vacant_since = Some(self.month);
                        self.event("household_vacant", Some(f.site), None,
                            format!("{} has no eligible resident representative or unrepresented adult/elder slot; ownership and the estate wallet are retained", f.name));
                        self.events
                            .last_mut()
                            .unwrap()
                            .subjects
                            .extend([("person".into(), old as u32), ("household".into(), f.id)]);
                    }
                    continue;
                }
                let vacancy = f.vacant_since.take();
                let identified_age = if resident_mode && band == Some(2) {
                    720
                } else {
                    300
                };
                if existing.is_none() {
                    if resident_mode {
                        let band = band.expect("resident slot checked before admission");
                        assert!(slots.reserve(f.site, band, site.demography.ages[band], 1));
                    }
                    self.people.push(Person {
                        id,
                        name: self.civilizations[successor_civilization as usize]
                            .naming(self.seed)
                            .person_with(
                                "person",
                                id,
                                &crate::naming::PersonalContext::local(site, self.culture.as_ref())
                                    .with_person(&self.people[old]),
                            ),
                        civilization: successor_civilization,
                        born: self.month as i32 - identified_age,
                        died: None,
                        predecessor: Some(old as u32),
                    });
                } else {
                    self.people[id as usize].predecessor = Some(old as u32);
                }
                if existing.is_none() {
                    let band = crate::population_registry::age_band(
                        self.month,
                        self.people[id as usize].born,
                    )
                    .unwrap();
                    if !resident_mode {
                        slots.observe(f.site, band, true);
                    }
                }
                // Keep the membership lookup consistent with the new ownership role;
                // recorded parents and unions are unchanged, and nobody changes site.
                if let Some(politics) = &mut self.politics {
                    if let Some(kin) = politics.kin.iter_mut().find(|k| k.person == id) {
                        kin.household = f.id;
                    } else {
                        politics.kin.push(crate::politics::Kinship {
                            person: id,
                            household: f.id,
                            parents: [None; 2],
                        });
                    }
                }
                occupied.insert(id);
                f.head = id;
                f.generation += 1;
                if ruler {
                    self.civilizations[successor_civilization as usize].leader = id;
                }
                self.event(
                    "inheritance",
                    Some(f.site),
                    None,
                    format!(
                        "{} succeeded {}; retained {:0.2}% ownership of {}",
                        self.people[id as usize].name,
                        self.people[old].name,
                        f.share * 100.,
                        f.name
                    ),
                );
                if let Some(since) = vacancy {
                    self.event("household_represented", Some(f.site), None,
                        format!("{} again has a representative after {} months of vacancy; existing estate claims and wallet remain with the account", f.name, self.month.saturating_sub(since)));
                    self.events
                        .last_mut()
                        .unwrap()
                        .subjects
                        .extend([("person".into(), id), ("household".into(), f.id)]);
                }
                if ruler {
                    self.event(
                        "succession",
                        Some(f.site),
                        None,
                        format!(
                            "{} assumed the household's governing office",
                            self.people[id as usize].name
                        ),
                    );
                }
            }
        }
        self.society = Some(society);
        self.resolve_council_vacancies();
        supply_comparison.settle(self)?;
        Ok(())
    }
    pub(crate) fn social_year(&mut self) {
        let office_capacity: Vec<_> = (0..self.sites.len())
            .map(|s| self.office_capacity(s as u32))
            .collect();
        let Some(mut society) = self.society.take() else {
            return;
        };
        let controllers: Vec<_> = (0..self.sites.len())
            .map(|s| self.controller(s as u32))
            .collect();
        society.council_funding.taxes.clear();
        for s in &mut self.sites {
            if s.abandoned {
                continue;
            }
            let council = &mut society.councils[controllers[s.id as usize] as usize];
            let autonomy = self
                .governance
                .as_ref()
                .and_then(|g| g.administrations.get(s.id as usize))
                .map_or(0., |a| a.autonomy);
            let tax = s.economy.finance[0]
                * council.tax_rate
                * (1. - autonomy * 0.75)
                * office_capacity[s.id as usize];
            society.council_funding.taxes.push(TaxReceipt {
                month: self.month,
                site: s.id,
                council: council.civilization,
                opening_cash: s.economy.finance[0],
                rate: council.tax_rate,
                autonomy,
                office_capacity: office_capacity[s.id as usize],
                paid: tax,
            });
            s.economy.finance[0] -= tax;
            council.treasury += tax as f64;
            if s.stocks.stock[3] > 0.05 {
                let relief = council.treasury.min(s.stocks.stock[0] as f64 * 10.) as f32;
                council.treasury = (council.treasury - relief as f64).max(0.);
                s.economy.finance[0] += relief;
                council.relief_paid += relief as f64;
                society
                    .council_funding
                    .emergency_town_support
                    .record(s.stocks.stock[0] as f64 * 10., relief as f64);
            }
        }
        let planned = self
            .economy_catalog
            .as_ref()
            .is_some_and(|c| c.production.enabled);
        for route in &mut society.routes {
            if !route.passable()
                || ((planned || route.upkeep.is_some())
                    && self.sites[route.from as usize].abandoned)
            {
                continue;
            }
            if let Some(care) = &mut route.upkeep {
                if care.last_build == Some(self.month) {
                    continue;
                }
                care.last_build = Some(self.month);
            }
            let site = &mut self.sites[route.from as usize];
            let council = &mut society.councils[controllers[site.id as usize] as usize];
            let bricks = site.economy.goods[5]
                .min(if planned || route.upkeep.is_some() {
                    (1000. - route.road_bricks).max(0.) as f32
                } else {
                    f32::INFINITY
                })
                .min(100.)
                .min(if route.upkeep.is_some() {
                    site.economy.logistics[2].max(0.) * 100.
                } else {
                    f32::INFINITY
                });
            // Feasible materials and work before the cash cap, not hypothetical road demand.
            let requested_cash = bricks as f64 * 2.;
            let bricks = bricks.min((council.treasury / 2.) as f32);
            // Credit only material actually withdrawn from the f32 inventory.
            let before = site.economy.goods[5];
            let remaining = (before as f64 - bricks as f64).max(0.);
            let mut after = remaining as f32;
            if (after as f64) < remaining {
                after = f32::from_bits(after.to_bits() + 1);
            }
            site.economy.goods[5] = after;
            let bricks = before - after;
            route.road_bricks += bricks as f64;
            if let Some(care) = &mut route.upkeep {
                let before = site.economy.logistics[2];
                site.economy.logistics[2] = (before - bricks / 100.).max(0.);
                care.work += (before - site.economy.logistics[2]) as f64;
            }
            council.treasury = (council.treasury - bricks as f64 * 2.).max(0.);
            site.economy.finance[0] += bricks * 2.;
            society
                .council_funding
                .roads
                .record(requested_cash, bricks as f64 * 2.);
        }
        self.society = Some(society);
        self.road_condition_events();
        if self.politics.is_some() {
            return;
        }
        // Hunger provides a motive; only reachable, politically distinct neighbors can be raided.
        for i in 0..self.sites.len() {
            let recent_crisis = self.events.iter().rev().any(|e| {
                e.kind == "food_crisis"
                    && e.site == Some(i as u32)
                    && self.month.saturating_sub(e.month) <= 24
            });
            if !recent_crisis || self.sites[i].demography.ages[1] < 30. {
                continue;
            }
            if self
                .society
                .as_ref()
                .unwrap()
                .raids
                .iter()
                .any(|r| r.origin == i as u32)
            {
                continue;
            }
            let target = (0..self.sites.len()).find(|&j| {
                j != i
                    && self.sites[j].civilization != self.sites[i].civilization
                    && self.sites[j].stocks.stock[1] > self.sites[i].stocks.stock[1] * 2.
                    && self
                        .route_cost(i as u32, j as u32)
                        .is_some_and(|c| c < 900.)
            });
            if let Some(j) = target {
                let months = (self.route_cost(i as u32, j as u32).unwrap() / 150.)
                    .ceil()
                    .max(1.) as u32;
                let (men, _) = raid_muster(
                    self.sites[i].demography.ages[1],
                    self.sites[i].stocks.stock[1],
                    months,
                );
                if men < 3. {
                    continue;
                }
                let Ok(recruits) = self.recruit_service_people(i as u32, men.floor() as usize, 3)
                else {
                    continue;
                };
                let men = recruits.people.len() as f32;
                let food = (men * 18. * (months + 1) as f32).min(self.sites[i].stocks.stock[1]);
                let id = self.society.as_ref().unwrap().next_raid;
                self.assign_military_people(id, i as u32, &recruits.people);
                let site = &mut self.sites[i];
                site.stocks.stock[1] -= food;
                site.stocks.stock[0] -= men;
                site.demography.ages[1] -= men;
                site.stocks.people[3] += men;
                self.event("raid_departure",Some(i as u32),Some(j as u32),format!("Food crisis prompted a raid: {:0.0} residents mustered with {:0.0} kg provisions",men,food));
                self.record_recruitment(&recruits);
                let cause = self.events.len() as u64 - 1;
                let society = self.society.as_mut().unwrap();
                let id = society.next_raid;
                society.next_raid += 1;
                society.raids.push(Raid {
                    members: Some(recruits.people),
                    loss_remainder: 0.,
                    id,
                    origin: i as u32,
                    target: j as u32,
                    soldiers: men,
                    food,
                    arrives: self.month + months,
                    cause,
                    returning: false,
                    war: None,
                    travel_months: months,
                    equipment: 0.,
                    occupation_until: None,
                });
            }
        }
    }
}
/// Dijkstra over actual dry central cells; slope and river crossings increase travel cost.
pub(crate) fn terrain_path(
    start: u32,
    end: u32,
    n: u32,
    radius: f32,
    cells: &[Cell],
) -> Option<(Vec<u32>, f32)> {
    let mut distance = vec![u64::MAX; cells.len()];
    let mut parent = vec![u32::MAX; cells.len()];
    let mut heap = BinaryHeap::new();
    distance[start as usize] = 0;
    heap.push(std::cmp::Reverse((0u64, start)));
    while let Some(std::cmp::Reverse((cost, i))) = heap.pop() {
        if cost != distance[i as usize] {
            continue;
        }
        if i == end {
            break;
        }
        if cost > 3_000_000 {
            continue;
        }
        for (x, y) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let j = grid::neighbor(i, n, x, y);
            if !traversable(&cells[j as usize]) {
                continue;
            }
            let km = crate::civilization::distance(i, j, n) * radius;
            let slope = (cells[i as usize].terrain[0] - cells[j as usize].terrain[0]).abs() / 500.;
            let river = (cells[j as usize].water[3] / 1000.).min(3.);
            let next = cost + (km * 1000. * (1. + slope + river)).max(1.) as u64;
            if next < distance[j as usize] {
                distance[j as usize] = next;
                parent[j as usize] = i;
                heap.push(std::cmp::Reverse((next, j)));
            }
        }
    }
    if distance[end as usize] == u64::MAX {
        return None;
    }
    let mut path = vec![end];
    while *path.last().unwrap() != start {
        path.push(parent[*path.last().unwrap() as usize]);
    }
    path.reverse();
    Some((path, distance[end as usize] as f32 / 1000.))
}
impl Generator {
    /// Apply a bounded ration policy at a completed history boundary.
    pub fn set_ration_priority(&mut self, site: u32, priority: [f32; 3]) -> Result<()> {
        ensure!(
            self.progress.stage == crate::gpu::Stage::Boundary,
            "ration policy requires a completed boundary"
        );
        self.validate_living_boundary()?;
        ensure!(
            priority
                .iter()
                .all(|v| v.is_finite() && (0. ..=3.).contains(v)),
            "ration priorities must be in 0–3"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(h.society.is_some(), "enable society first");
        let s = h
            .sites
            .get_mut(site as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown settlement"))?;
        if s.demography.ration_priority[..3] == priority {
            return Ok(());
        }
        s.demography.ration_priority = [priority[0], priority[1], priority[2], 0.];
        h.event("ration_policy", Some(site), None, format!("Additional child/adult/elder ration priority: {:.1}/{:.1}/{:.1}; half of available rations remains need-proportional", priority[0], priority[1], priority[2]));
        Ok(())
    }
}
impl Generator {
    pub fn enable_society(&mut self) -> Result<()> {
        let cells = self.snapshot()?;
        let h = self
            .civilizations
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        let mut h = h.clone();
        ensure!(
            h.version == 2 && h.society.is_none(),
            "society requires an economy without an existing social baseline"
        );
        h.society = Some(Society {
            council_funding: Default::default(),
            household_economy: Some(crate::household_economy::HouseholdEconomy::new(h.month)),
            indicators: Some(crate::social_state::SocialState {
                started: h.month,
                observed: h.month,
                sites: vec![],
            }),
            version: 1,
            started: h.month,
            households: vec![],
            councils: h
                .civilizations
                .iter()
                .map(|c| Council {
                    civilization: c.id,
                    treasury: 0.,
                    tax_rate: 0.03,
                    pending_tax: None,
                    distribution: None,
                    pending_distribution: None,
                    distribution_review: None,
                    tax_effective_since: None,
                    relief_paid: 0.,
                })
                .collect(),
            routes: vec![],
            routed_sites: 0,
            raids: vec![],
            next_raid: 0,
            relocation: crate::relocation::RelocationState {
                witnessed_relief: true,
                enabled: true,
                ..Default::default()
            },
        });
        h.prepare_society_with_navigation(
            &cells,
            self.config.radius_km,
            self.navigation_service()?.as_deref(),
        )?;
        h.event(
            "social_baseline",
            None,
            None,
            "Age cohorts, household ownership and councils established; terrain routes surveyed"
                .into(),
        );
        if h.month == 0 {
            h.event("household_distribution_policy", None, None,
                "Founding provisions are held in common: one year of full food entitlement, then four years of gradual transition to household purchasing; all meals draw on existing town stocks".into());
        }
        h.social_indicators_month();
        h.validate(&cells)?;
        self.civilizations = Some(h);
        Ok(())
    }
    pub fn set_route_open(&mut self, route: u32, open: bool) -> Result<()> {
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no civilizations"))?;
        let society = h
            .society
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable social history first"))?;
        let r = society
            .routes
            .get_mut(route as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown route"))?;
        r.open = open;
        let a = r.from;
        let b = r.to;
        h.event(
            "route_policy",
            Some(a),
            Some(b),
            format!("Route {route} {}", if open { "opened" } else { "closed" }),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{catalog::Catalog, config::Config, gpu::ContextGpu};

    #[test]
    #[ignore = "requires hardware GPU"]
    fn planned_road_limit_survives_unprepared_and_abandoned_sites() {
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
        let h = g.civilizations.as_mut().unwrap();
        let society = h.society.as_mut().unwrap();
        society.routes.truncate(1);
        let route = &mut society.routes[0];
        route.open = true;
        route.road_bricks = 995.;
        let origin = route.from as usize;
        for c in &mut society.councils {
            c.treasury = 10000.;
            c.tax_rate = 0.;
        }
        h.sites[origin].economy.goods[5] = 100.;
        h.sites[origin].economy.logistics = [0.; 4];
        h.social_year();
        assert_eq!(h.society.as_ref().unwrap().routes[0].road_bricks, 995.);
        assert_eq!(h.sites[origin].economy.goods[5], 100.);
        h.month += 12;
        h.sites[origin].economy.logistics[2] = 0.05;
        h.social_year();
        assert_eq!(h.society.as_ref().unwrap().routes[0].road_bricks, 1000.);
        assert_eq!(h.sites[origin].economy.goods[5], 95.);
        h.society.as_mut().unwrap().routes[0].road_bricks = 995.;
        h.sites[origin].abandoned = true;
        h.social_year();
        assert_eq!(h.society.as_ref().unwrap().routes[0].road_bricks, 995.);
        assert_eq!(h.sites[origin].economy.goods[5], 95.);
        // Controlled damage followed by four finite annual works seasons.
        h.sites[origin].abandoned = false;
        h.sites[origin].economy.goods[5] = 400.;
        h.society.as_mut().unwrap().routes[0].road_bricks = 495.;
        h.road_condition_events();
        assert!(
            h.society.as_ref().unwrap().routes[0]
                .upkeep
                .as_ref()
                .unwrap()
                .impaired
        );
        for _ in 0..4 {
            h.month += 12;
            h.sites[origin].economy.logistics[2] = 1.;
            h.social_year();
        }
        assert_eq!(h.sites[origin].economy.goods[5], 0.);
        let r = &h.society.as_ref().unwrap().routes[0];
        assert_eq!(r.road_bricks, 895.);
        assert!(!r.upkeep.as_ref().unwrap().impaired);
        assert!((r.upkeep.as_ref().unwrap().work - 4.05).abs() < 1e-6);
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "road_restored")
                .count(),
            1
        );
        let e = h.events.iter().find(|e| e.kind == "road_restored").unwrap();
        assert_eq!(e.causes.len(), 1);
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn provisioned_raids_conserve_and_resume_in_transit() {
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
        let cells = g.snapshot().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .find(|r| {
                r.cost_km < 900.
                    && h.sites[r.from as usize].civilization != h.sites[r.to as usize].civilization
            })
            .expect("fixture needs neighboring polities")
            .clone();
        // A recovered shortage motivates the raid. Move existing food to establish
        // unequal reserves without injecting resources into the fixture.
        let transfer = h.sites[route.from as usize].stocks.stock[1] * 0.8;
        h.sites[route.from as usize].stocks.stock[1] -= transfer;
        h.sites[route.to as usize].stocks.stock[1] += transfer;
        h.event(
            "food_crisis",
            Some(route.from),
            None,
            "Controlled recent shortage".into(),
        );
        h.social_year();
        assert!(!h.society.as_ref().unwrap().raids.is_empty());
        assert!(h
            .society
            .as_ref()
            .unwrap()
            .raids
            .iter()
            .all(|r| r.food > 0. && r.soldiers > 0. && !r.returning));
        assert!(h
            .events
            .iter()
            .filter(|e| e.kind == "raid_departure")
            .all(|e| !e.causes.is_empty()));
        h.validate(&cells).unwrap();
        let file = std::env::temp_dir().join(format!("raid-{}.world", std::process::id()));
        g.save(&file).unwrap();
        let mut b = Generator::load(g.gpu.clone(), &file).unwrap();
        std::fs::remove_file(file).unwrap();
        g.advance_history(12).unwrap();
        let mut saw_return_leg = false;
        for _ in 0..12 {
            b.advance_history(1).unwrap();
            saw_return_leg |= b
                .civilizations
                .as_ref()
                .unwrap()
                .society
                .as_ref()
                .unwrap()
                .raids
                .iter()
                .any(|r| r.returning);
        }
        assert!(saw_return_leg, "return travel must take time");
        let h = g.civilizations.as_ref().unwrap();
        assert!(h.events.iter().any(|e| e.kind == "raid_outcome"));
        assert!(h.events.iter().any(|e| e.kind == "raid_return"));
        assert!(h.population_residual().abs() < 0.001);
        assert!(h.food_residual().abs() < 0.001);
        assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
        assert!(
            serde_json::to_vec(&g.civilizations).unwrap()
                == serde_json::to_vec(&b.civilizations).unwrap(),
            "in-transit checkpoint diverged"
        );
    }
}

#[cfg(test)]
mod policy_timing_tests {
    use super::*;
    fn council() -> Council {
        Council {
            civilization: 0,
            treasury: 100.,
            tax_rate: 0.03,
            pending_tax: None,
            distribution: None,
            pending_distribution: None,
            distribution_review: None,
            tax_effective_since: None,
            relief_paid: 0.,
        }
    }
    #[test]
    fn policy_waits_for_boundary_and_resumes_without_reapplying() {
        let mut c = council();
        assert!(c.plan_tax(0.1, 12));
        assert_eq!(c.tax_rate, 0.03);
        assert!(c.activate_tax(12).is_none());
        let mut resumed: Council =
            serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
        assert_eq!(c.activate_tax(13), resumed.activate_tax(13));
        assert_eq!(c.tax_rate, 0.1);
        assert_eq!(c.tax_effective_since, Some(13));
        assert!(c.activate_tax(13).is_none());
        assert!(c.activate_tax(14).is_none());
        assert_eq!(
            c.treasury, 100.,
            "activating policy does not itself collect tax"
        );
    }
    #[test]
    fn revision_replaces_pending_rate_and_old_archives_keep_baseline() {
        let mut c = council();
        assert!(c.plan_tax(0.1, 12));
        assert!(!c.plan_tax(0.1, 12));
        assert!(c.plan_tax(0.05, 12));
        assert_eq!(c.pending_tax.as_ref().unwrap().effective_month, 13);
        assert!(!c.plan_tax(0.03, 12));
        assert!(c.pending_tax.is_none());
        let old: Council = serde_json::from_str(
            r#"{"civilization":0,"treasury":100.0,"tax_rate":0.07,"relief_paid":0.0}"#,
        )
        .unwrap();
        assert_eq!(old.tax_rate, 0.07);
        assert!(old.pending_tax.is_none());
        assert!(old.tax_effective_since.is_none());
    }
}
