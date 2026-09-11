//! Finite household journeys between existing communities on the same island.
use crate::{civilization::History, economy::FOOD_CNP};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

/// Transient decision inputs captured after consumption, before response actions.
/// Financial, food and housing capacity checks still use live reservations at commit.
#[derive(Clone, Debug)]
pub(crate) struct RelocationObservations {
    month: u32,
    social_month: Option<u32>,
    sites: Vec<RelocationSiteObservation>,
}
#[derive(Clone, Debug)]
struct RelocationSiteObservation {
    id: u32,
    shortage: f32,
    production: f32,
    crowded: bool,
    migration_pressure: f32,
    flooded: bool,
    persistent_flood: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RelocationState {
    /// Old archives remain disabled unless explicitly enabled.
    pub enabled: bool,
    pub witnessed_relief: bool,
    pub appeals: Vec<crate::relief::Appeal>,
    pub sites: Vec<Pressure>,
    pub journeys: Vec<Journey>,
    /// Extinct traveling households remain ownership/history records, not local actors.
    pub lost_households: std::collections::BTreeSet<u32>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Pressure {
    pub hungry: u32,
    pub production: [f32; 12],
    pub observed_months: u32,
    pub last_departure: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Journey {
    pub household: u32,
    pub from: u32,
    pub to: u32,
    pub route: u32,
    pub departed: u32,
    pub arrives: u32,
    pub cohorts: [f32; 3],
    pub food: f32,
    pub cash: f32,
    pub tools: f32,
    pub cause: u64,
    pub blocked: bool,
    pub returning: bool,
    #[serde(default)]
    pub seek_help: bool,
    #[serde(default)]
    pub report_population: f32,
    #[serde(default)]
    pub report_food_months: Option<f32>,
}
impl Journey {
    pub fn population(&self) -> f32 {
        self.cohorts.iter().sum()
    }
}
impl RelocationState {
    pub fn away(&self, household: u32) -> bool {
        self.lost_households.contains(&household)
            || self.journeys.iter().any(|j| j.household == household)
    }
    pub fn validate(&self, h: &History) -> Result<()> {
        let society = h.society.as_ref().unwrap();
        ensure!(
            self.sites.len() <= h.sites.len() && self.journeys.len() <= society.households.len(),
            "invalid relocation sizes"
        );
        ensure!(
            self.lost_households
                .iter()
                .all(|&id| (id as usize) < society.households.len()),
            "invalid lost household"
        );
        for a in &self.appeals {
            ensure!(
                (a.origin as usize) < h.sites.len()
                    && (a.host as usize) < h.sites.len()
                    && a.origin != a.host
                    && (a.household as usize) < society.households.len()
                    && a.reported <= a.received
                    && a.received <= h.month
                    && a.population.is_finite()
                    && a.population >= 0.
                    && society
                        .routes
                        .get(a.route as usize)
                        .is_some_and(|r| (r.from == a.origin && r.to == a.host)
                            || (r.to == a.origin && r.from == a.host))
                    && h.events
                        .get(a.cause as usize)
                        .is_some_and(|e| e.kind == "relief_appeal")
                    && a.response.is_none_or(|id| id > a.cause
                        && h.events
                            .get(id as usize)
                            .is_some_and(|e| e.causes.contains(&a.cause))),
                "invalid relief appeal"
            );
        }
        let mut households = self.lost_households.clone();
        for p in &self.sites {
            ensure!(
                p.observed_months <= 12
                    && p.last_departure <= h.month
                    && p.production.iter().all(|v| v.is_finite() && *v >= 0.),
                "invalid relocation pressure"
            );
        }
        for j in &self.journeys {
            ensure!(
                (j.from as usize) < h.sites.len()
                    && (j.to as usize) < h.sites.len()
                    && j.from != j.to
                    && h.sites[j.from as usize].island == h.sites[j.to as usize].island
                    && society
                        .households
                        .get(j.household as usize)
                        .is_some_and(|hh| hh.site == j.from)
                    && households.insert(j.household)
                    && society
                        .routes
                        .get(j.route as usize)
                        .is_some_and(|r| (r.from == j.from && r.to == j.to)
                            || (r.to == j.from && r.from == j.to))
                    && j.report_food_months
                        .is_none_or(|v| v.is_finite() && (0. ..=24.).contains(&v))
                    && j.report_population.is_finite()
                    && j.report_population >= 0.
                    && j.departed <= h.month
                    && j.arrives > j.departed
                    && j.cohorts
                        .iter()
                        .chain([&j.food, &j.cash, &j.tools])
                        .all(|v| v.is_finite() && *v >= 0.)
                    && h.events
                        .get(j.cause as usize)
                        .is_some_and(|e| e.kind == "household_departure"),
                "invalid household journey"
            );
        }
        Ok(())
    }
}
impl History {
    pub fn set_household_relocation(&mut self, enabled: bool) -> Result<()> {
        let society = self
            .society
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable society first"))?;
        society.relocation.enabled = enabled;
        Ok(())
    }
    pub fn set_witnessed_relief(&mut self, enabled: bool) -> Result<()> {
        let society = self
            .society
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable society first"))?;
        society.relocation.witnessed_relief = enabled;
        Ok(())
    }
    pub fn set_religious_relief(&mut self, enabled: bool) -> Result<()> {
        let c = self
            .culture
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("culture required"))?;
        c.religious_relief.enabled = enabled;
        self.event("religious_relief_policy",None,None,format!("Institution-funded witnessed relief enabled: {enabled}; existing shipments continue"));
        Ok(())
    }
    pub fn household_relocations(&self) -> Option<&RelocationState> {
        self.society.as_ref().map(|s| &s.relocation)
    }

    /// Complete/hold funded journeys before resident production. Travelers produce nothing.
    pub(crate) fn relocation_arrivals(&mut self) {
        let Some(society) = &mut self.society else {
            return;
        };
        let journeys = std::mem::take(&mut society.relocation.journeys);
        for mut j in journeys {
            let need = j
                .cohorts
                .iter()
                .zip([10., 18., 14.])
                .map(|(n, r)| n * r)
                .sum::<f32>();
            let eaten = j.food.min(need);
            j.food -= eaten;
            let origin = &mut self.sites[j.from as usize];
            origin.stocks.ledger[1] += eaten;
            for (k, ratio) in FOOD_CNP.iter().enumerate() {
                origin.economy.external[k] -= eaten * *ratio as f32;
            }
            if eaten + 0.001 < need {
                let loss = 0.08 * (1. - eaten / need.max(0.001));
                let dead = j.population() * loss;
                for age in &mut j.cohorts {
                    *age *= 1. - loss;
                }
                origin.stocks.people[1] += dead;
                origin.demography.health[2] += dead;
            }
            if j.population() < 0.01 {
                self.society
                    .as_mut()
                    .unwrap()
                    .relocation
                    .lost_households
                    .insert(j.household);
                let residual = j.population();
                self.sites[j.from as usize].stocks.people[1] += residual;
                let origin = &mut self.sites[j.from as usize];
                origin.economy.finance[0] += j.cash; // ownership of the unspent estate
                origin.economy.used[3] += j.tools;
                origin.economy.reserves[3] += j.tools; // tools lost to the regional mineral pool
                origin.stocks.ledger[2] += j.food;
                for (k, ratio) in FOOD_CNP.iter().enumerate() {
                    origin.economy.external[k] -= j.food * *ratio as f32;
                }
                self.relocation_event("household_journey_lost", &j, "No surviving traveler cohort; food and equipment losses recorded, unspent money retained by the origin estate".into());
                continue;
            }
            let r = &self.society.as_ref().unwrap().routes[j.route as usize];
            let accessible = r.passable();
            if self.month < j.arrives || !accessible {
                if self.month >= j.arrives && !j.blocked {
                    j.blocked = true;
                    self.relocation_event(
                        "household_journey_blocked",
                        &j,
                        "Journey held by route closure; finite provisions continue to be consumed"
                            .into(),
                    );
                }
                self.society.as_mut().unwrap().relocation.journeys.push(j);
                continue;
            }
            // A failed destination cannot absorb refugees: return along the reopened
            // route, paying the same travel time and continuing to consume supplies.
            if (self.sites[j.to as usize].abandoned
                || !self.sites[j.to as usize]
                    .economy
                    .housing_accepts(self.sites[j.to as usize].stocks.stock[0] + j.population()))
                && !j.returning
            {
                j.returning = true;
                j.arrives = self.month + (r.cost_km / 150.).ceil().max(1.) as u32;
                self.relocation_event(
                    "household_returning",
                    &j,
                    "Destination failed or lacks shelter capacity; household retracing its route home".into(),
                );
                self.society.as_mut().unwrap().relocation.journeys.push(j);
                continue;
            }
            self.finish_relocation(j);
        }
    }
    fn relocation_event(&mut self, kind: &str, j: &Journey, detail: String) {
        self.event(kind, Some(j.to), Some(j.from), detail);
        let e = self.events.last_mut().unwrap();
        e.subjects.push(("household".into(), j.household));
        e.causes.push(j.cause);
        if matches!(kind, "household_arrival" | "household_returned") {
            e.spatial
                .as_mut()
                .unwrap()
                .push(crate::spatial::EventAnchor {
                    cell: self.sites[j.to as usize].cell,
                    role: crate::spatial::EventRole::Milestone,
                });
        }
    }
    fn finish_relocation(&mut self, mut j: Journey) {
        if j.returning {
            j.to = j.from;
        }
        let report_destination = j.from;
        let pop = j.population();
        let target = &mut self.sites[j.to as usize];
        let old_pop = target.stocks.stock[0];
        target.stocks.stock[0] += pop;
        target.stocks.people[2] += pop;
        target.stocks.stock[1] += j.food;
        target.economy.finance[0] += j.cash;
        target.economy.goods[3] += j.tools;
        for k in 0..3 {
            target.demography.ages[k] += j.cohorts[k];
        }
        if j.to != j.from {
            let society = self.society.as_mut().unwrap();
            let old_share = society.households[j.household as usize].share;
            let remaining = society
                .households
                .iter()
                .filter(|hh| hh.site == j.from && hh.id != j.household)
                .map(|hh| hh.share)
                .sum::<f64>();
            let new_share = (pop / (old_pop + pop).max(1.)).clamp(0.001, 0.95) as f64;
            for hh in &mut society.households {
                if hh.id == j.household {
                    hh.site = j.to;
                    hh.share = new_share;
                } else if hh.site == j.from && remaining > 0. {
                    hh.share /= remaining;
                } else if hh.site == j.to {
                    hh.share *= 1. - new_share;
                }
            }
            if let Some(p) = &mut self.politics {
                let interest =
                    p.factions[p.household_factions[j.household as usize] as usize].interest;
                p.household_factions[j.household as usize] = p
                    .factions
                    .iter()
                    .find(|f| {
                        f.civilization == self.sites[j.to as usize].civilization
                            && f.interest == interest
                    })
                    .unwrap()
                    .id;
            }
            self.receive_appeal(&j);
            self.relocation_event("household_arrival", &j, format!("Household resettled with {pop:.2} residents, {:.1} kg food, {:.2} money and {:.2} kg tools; relinquished {:.2}% origin private-stock interest, acquired {:.2}% destination interest; ancestry and faith retained",j.food,j.cash,j.tools,old_share*100.,new_share*100.));
        } else {
            self.relocation_event(
                "household_returned",
                &j,
                format!("Household returned with {pop:.2} survivors and remaining belongings"),
            );
        }
        let cause = self.events.last().unwrap().id;
        self.remember_arrival(
            j.to,
            report_destination,
            j.departed,
            j.report_food_months,
            cause,
        );
    }

    /// Seasonal hardship memory and admissions after this month's food/demography.
    pub(crate) fn observe_relocation(&self) -> RelocationObservations {
        RelocationObservations {
            month: self.month,
            social_month: self
                .society
                .as_ref()
                .and_then(|s| s.indicators.as_ref())
                .map(|s| s.observed),
            sites: self
                .sites
                .iter()
                .map(|s| {
                    let social = self.social_indicators(s.id);
                    let flood = self.living.as_ref().and_then(|l| l.floods.get(&s.id));
                    RelocationSiteObservation {
                        id: s.id,
                        shortage: s.stocks.stock[3],
                        production: s.stocks.stock[2],
                        crowded: social.is_some_and(|c| c.housing[2] > 0.15),
                        migration_pressure: social.map_or(0., |c| c.migration_pressure()),
                        flooded: flood.is_some_and(|f| f.flooded),
                        persistent_flood: flood.is_some_and(|f| f.persistent),
                    }
                })
                .collect(),
        }
    }

    pub(crate) fn relocation_departures_observed(
        &mut self,
        observations: &RelocationObservations,
    ) -> Result<()> {
        ensure!(
            observations.month == self.month,
            "stale relocation observations"
        );
        ensure!(
            observations
                .social_month
                .is_none_or(|m| m <= observations.month),
            "future social observations"
        );
        ensure!(
            observations.sites.len() == self.sites.len()
                && observations
                    .sites
                    .iter()
                    .zip(&self.sites)
                    .all(|(o, s)| o.id == s.id),
            "relocation observation site mismatch"
        );
        self.relocation_departures_inner(observations);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn relocation_departures(&mut self) {
        let observations = self.observe_relocation();
        self.relocation_departures_observed(&observations).unwrap();
    }

    fn relocation_departures_inner(&mut self, observations: &RelocationObservations) {
        let Some(society) = &mut self.society else {
            return;
        };
        society
            .relocation
            .sites
            .resize_with(self.sites.len(), Default::default);
        for s in &self.sites {
            let p = &mut society.relocation.sites[s.id as usize];
            p.hungry = ((p.hungry << 1)
                | u32::from(observations.sites[s.id as usize].shortage > 0.05))
                & 0xffffff;
            p.production[self.month as usize % 12] = observations.sites[s.id as usize].production;
            p.observed_months = (p.observed_months + 1).min(12);
        }
        if !society.relocation.enabled || self.month % 3 != 0 {
            return;
        }
        for from in 0..self.sites.len() {
            let society = self.society.as_ref().unwrap();
            let pressure = &society.relocation.sites[from];
            let s = &self.sites[from];
            if s.abandoned
                || s.stocks.stock[0] < 2.
                || pressure.observed_months < 12
                || self.month < pressure.last_departure + 12
                || (pressure.hungry.count_ones() < 6
                    && !observations.sites[from].crowded
                    && !observations.sites[from].persistent_flood)
            {
                continue;
            }
            let homes = society
                .households
                .iter()
                .filter(|hh| hh.site == from as u32 && !society.relocation.away(hh.id))
                .collect::<Vec<_>>();
            // Keep a local ownership representative; this v1 moves one household
            // per origin per year rather than performing whole-town evacuation.
            if homes.len() < 2 {
                continue;
            }
            let Some(household) = homes
                .iter()
                .cycle()
                .skip((self.month as usize / 12 + from) % homes.len())
                .take(homes.len())
                .find(|hh| {
                    if self.civilizations.iter().any(|c| c.leader == hh.head) {
                        return false;
                    }
                    if !society.relocation.witnessed_relief {
                        return true;
                    }
                    let traits = self
                        .culture
                        .as_ref()
                        .and_then(|c| c.agents.get(hh.head as usize))
                        .map_or([0.5; 6], |a| a.traits);
                    // Persistent household ties and caution compete with ambition and hardship.
                    let urgency = pressure.hungry.count_ones() as f32 / 24.
                        + society
                            .household_economy
                            .as_ref()
                            .and_then(|e| e.accounts.get(hh.id as usize))
                            .map_or(0., |a| a.hunger as f32 * 0.2)
                        + observations.sites[from].migration_pressure * 0.2;
                    let tie = (hh.id.wrapping_mul(2654435761) ^ self.seed) % 100;
                    urgency + traits[0] * 0.35
                        > traits[4] * 0.25 + traits[5] * 0.3 + tie as f32 / 200.
                })
            else {
                if society.relocation.witnessed_relief && self.month % 12 == 0 {
                    let id = homes[0].id;
                    self.event("households_stay", Some(from as u32), None,
                        "Households remain despite hardship: local ties, caution or leadership obligations outweighed departure this year".into());
                    self.events
                        .last_mut()
                        .unwrap()
                        .subjects
                        .push(("household".into(), id));
                }
                continue;
            };
            let seek_help = society.relocation.witnessed_relief && household.id % 3 != 0;
            let reserve_months = if seek_help { 1 } else { 3 };
            let pop = (s.stocks.stock[0] / homes.len() as f32)
                .clamp(2., 8.)
                .min(s.stocks.stock[0] * 0.25);
            if pop < 1. {
                continue;
            }
            let mut best = None;
            for r in &society.routes {
                let to = if r.from as usize == from {
                    r.to
                } else if r.to as usize == from {
                    r.from
                } else {
                    continue;
                };
                let t = &self.sites[to as usize];
                let tp = &society.relocation.sites[to as usize];
                let pending = society
                    .relocation
                    .journeys
                    .iter()
                    .filter(|j| j.to == to)
                    .map(Journey::population)
                    .sum::<f32>();
                let demand_pop = t.stocks.stock[0] + pending + pop;
                let months = (r.cost_km / 150.).ceil().max(1.) as u32;
                let provisions = pop * 18. * (months + reserve_months) as f32;
                let resident_need = t.demography.ages[..3]
                    .iter()
                    .zip([10., 18., 14.])
                    .map(|(n, r)| n * r)
                    .sum::<f32>();
                let per_person_need = (resident_need / t.stocks.stock[0].max(1.)).max(10.);
                let fertile_capacity =
                    tp.production.iter().sum::<f32>() / (per_person_need * 12.) * 0.95;
                if !r.open
                    || r.flood_months > 0
                    || months > 10
                    || t.island != s.island
                    || t.abandoned
                    || tp.observed_months < 12
                    || tp.hungry.count_ones() > 1
                    || observations.sites[to as usize].shortage > 0.01
                    || !t.economy.housing_accepts(demand_pop)
                    || fertile_capacity < demand_pop
                    || t.stocks.habitat[1] * 2. < demand_pop
                    || t.stocks.stock[1] < demand_pop * 18. * 6.
                    || s.stocks.stock[1] < provisions + (s.stocks.stock[0] - pop) * 18.
                    || (observations.sites[to as usize].flooded
                        || observations.sites[to as usize].persistent_flood)
                    || self.politics.as_ref().is_some_and(|p| {
                        p.wars.iter().any(|w| {
                            w.ended.is_none()
                                && ((w.attacker == self.controller(from as u32)
                                    && w.defender == self.controller(to))
                                    || (w.defender == self.controller(from as u32)
                                        && w.attacker == self.controller(to)))
                        })
                    })
                {
                    continue;
                }
                let score = (fertile_capacity - demand_pop)
                    * (0.25 + self.relief_affinity(from as u32, to))
                    * self.destination_memory(from as u32, to)
                    / (1. + r.cost_km / 150.);
                if best.as_ref().is_none_or(|&(_, _, _, v)| score > v) {
                    best = Some((to, r.id, months, score));
                }
            }
            if let Some((to, route, months, _)) = best {
                let report_weight = self.destination_memory(from as u32, to);
                let id = household.id;
                let share = household.share as f32;
                let source = &mut self.sites[from];
                let fraction = pop / source.stocks.stock[0];
                let cohorts = std::array::from_fn(|k| source.demography.ages[k] * fraction);
                let people: f32 = cohorts.iter().sum();
                let food = people * 18. * (months + reserve_months) as f32;
                let cash = source.economy.finance[0] * share;
                let tools = (source.economy.goods[3] * share).min(people * 0.5);
                source.stocks.stock[0] -= people;
                source.stocks.people[3] += people;
                source.stocks.stock[1] -= food;
                source.economy.finance[0] -= cash;
                source.economy.goods[3] -= tools;
                for (k, amount) in cohorts.iter().enumerate() {
                    source.demography.ages[k] -= amount;
                }
                self.event("household_departure",Some(from as u32),Some(to),format!("Household {id} left repeated hardship with {people:.2} residents; {months}-month journey funded by {food:.1} kg existing food, {cash:.2} money and {tools:.2} kg tools; dated destination testimony preference {report_weight:.3}"));
                let road = &self.society.as_ref().unwrap().routes[route as usize];
                let mut path = road.cells.clone();
                if road.from != from as u32 {
                    path.reverse();
                }
                self.events.last_mut().unwrap().planned_path = Some(path);
                let cause = self.events.last().unwrap().id;
                self.events
                    .last_mut()
                    .unwrap()
                    .subjects
                    .push(("household".into(), id));
                if seek_help {
                    self.events
                        .last_mut()
                        .unwrap()
                        .detail
                        .push_str("; family intends to seek help for those remaining");
                }
                let report_population = self.sites[from].stocks.stock[0];
                let report_food_months = Some(
                    (self.sites[from].stocks.stock[1] / (report_population.max(1.) * 18.))
                        .clamp(0., 24.),
                );
                let society = self.society.as_mut().unwrap();
                society.relocation.sites[from].last_departure = self.month;
                society.relocation.journeys.push(Journey {
                    household: id,
                    from: from as u32,
                    to,
                    route,
                    departed: self.month,
                    arrives: self.month + months,
                    cohorts,
                    food,
                    cash,
                    tools,
                    cause,
                    blocked: false,
                    returning: false,
                    seek_help,
                    report_population,
                    report_food_months,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        gpu::{ContextGpu, Generator},
    };
    #[test]
    #[ignore = "requires hardware GPU"]
    fn relocation_conserves_and_reserves_capacity_and_preserves_identity() {
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
        g.found_civilizations(16).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.civilizations
            .as_mut()
            .unwrap()
            .society
            .as_mut()
            .unwrap()
            .relocation
            .witnessed_relief = false;
        g.advance_history(24).unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.sync_culture();
        h.month = 24;
        let society = h.society.as_mut().unwrap();
        let r = society.routes[0].clone();
        for route in &mut society.routes {
            route.open = route.id == r.id;
        }
        let (from, to) = (r.from as usize, r.to as usize);
        society.relocation.sites = vec![
            Pressure {
                production: [10000.; 12],
                observed_months: 12,
                ..Default::default()
            };
            h.sites.len()
        ];
        society.relocation.sites[from].hungry = 0xffffff;
        // Explicitly provision the admission fixture before taking budget baselines.
        for id in [from, to] {
            h.sites[id].stocks.stock[1] = 1_000_000.;
            h.sites[id].stocks.stock[3] = 0.;
            h.sites[id].stocks.habitat[1] = 100_000.;
        }
        h.society.as_mut().unwrap().routes[r.id as usize].flood_months = 0;
        let expected = h.economy_residuals();
        let food = h.food_residual();
        let pop = h.population_residual();
        // Decision evidence is immutable even if a later response edits live fields.
        let mut observed = h.clone();
        observed.society.as_mut().unwrap().relocation.enabled = false;
        observed.sites[from].stocks.stock[3] = 0.5;
        observed.sites[from].stocks.stock[2] = 123.;
        let snapshot = observed.observe_relocation();
        observed.sites[from].stocks.stock[3] = 0.;
        observed.sites[from].stocks.stock[2] = 999.;
        observed.relocation_departures_observed(&snapshot).unwrap();
        let pressure = &observed.society.as_ref().unwrap().relocation.sites[from];
        assert_eq!(pressure.hungry & 1, 1);
        assert_eq!(pressure.production[observed.month as usize % 12], 123.);
        observed.month += 1;
        let before = serde_json::to_value(&observed).unwrap();
        assert!(observed.relocation_departures_observed(&snapshot).is_err());
        assert_eq!(before, serde_json::to_value(&observed).unwrap());

        let mut no_capacity = h.clone();
        no_capacity.society.as_mut().unwrap().relocation.sites[to].production = [0.; 12];
        no_capacity.relocation_departures();
        assert!(no_capacity
            .household_relocations()
            .unwrap()
            .journeys
            .is_empty());
        let mut no_housing = h.clone();
        no_housing.sites[to].economy.housing = [0., 0., no_housing.sites[to].stocks.stock[0], 0.];
        no_housing.sites[to].economy.housing_plan[3] = 1.;
        no_housing.relocation_departures();
        assert!(no_housing
            .household_relocations()
            .unwrap()
            .journeys
            .is_empty());
        let mut crowded = h.clone();
        crowded.society.as_mut().unwrap().relocation.sites[from].hungry = 0;
        crowded
            .society
            .as_mut()
            .unwrap()
            .indicators
            .as_mut()
            .unwrap()
            .sites[from]
            .housing[2] = 0.3;
        crowded.relocation_departures();
        assert_eq!(crowded.household_relocations().unwrap().journeys.len(), 1);
        h.relocation_departures();
        assert_eq!(h.household_relocations().unwrap().journeys.len(), 1);
        let j = h.household_relocations().unwrap().journeys[0].clone();
        assert_eq!((j.from, j.to), (r.from, r.to));
        assert_eq!(
            h.events[j.cause as usize].planned_path.as_ref().unwrap(),
            &r.cells
        );
        let mut changed_route = h.clone();
        changed_route.society.as_mut().unwrap().routes[j.route as usize]
            .cells
            .reverse();
        assert_eq!(
            changed_route.events[j.cause as usize]
                .planned_path
                .as_ref()
                .unwrap(),
            &r.cells
        );

        let faith = h.culture.as_ref().unwrap().household_faith[j.household as usize];
        let head = h.society.as_ref().unwrap().households[j.household as usize].head;
        let ancestry = h.people[head as usize].civilization;
        assert!((h.food_residual() - food).abs() < 1e-6);
        assert!((h.population_residual() - pop).abs() < 1e-6);
        for (a, b) in expected.iter().zip(h.economy_residuals()) {
            assert!((a - b).abs() < 1e-6, "{a} {b}");
        }
        // Capacity lost en route causes an actual return, not an unrecorded arrival.
        let mut lost_shelter = h.clone();
        lost_shelter.sites[to].economy.housing = [0., 0., 1., 0.];
        lost_shelter.sites[to].economy.housing_plan[3] = 1.;
        lost_shelter.month = j.arrives;
        let residents = lost_shelter.sites[to].stocks.stock[0];
        lost_shelter.relocation_arrivals();
        assert_eq!(lost_shelter.sites[to].stocks.stock[0], residents);
        assert!(lost_shelter.household_relocations().unwrap().journeys[0].returning);
        assert!(lost_shelter
            .events
            .iter()
            .any(|e| e.kind == "household_returning"));
        // Housing also reserves places for already funded arrivals.
        let mut shelter_reserved = h.clone();
        shelter_reserved.month = 36;
        let target_pop = shelter_reserved.sites[to].stocks.stock[0];
        shelter_reserved.sites[to].economy.housing = [0., 0., target_pop + j.population(), 0.];
        shelter_reserved.sites[to].economy.housing_plan[3] = 1.;
        shelter_reserved.relocation_departures();
        assert_eq!(
            shelter_reserved
                .household_relocations()
                .unwrap()
                .journeys
                .len(),
            1
        );
        // A destination must reserve capacity for already funded arrivals.
        let mut reserved = h.clone();
        reserved.month = 36;
        let homes = reserved
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .filter(|hh| {
                hh.site == from as u32 && !reserved.household_relocations().unwrap().away(hh.id)
            })
            .count();
        let next_pop = (reserved.sites[from].stocks.stock[0] / homes as f32)
            .clamp(2., 8.)
            .min(reserved.sites[from].stocks.stock[0] * 0.25);
        let target = &reserved.sites[to];
        let diet = (target.demography.ages[..3]
            .iter()
            .zip([10., 18., 14.])
            .map(|(n, r)| n * r)
            .sum::<f32>()
            / target.stocks.stock[0].max(1.))
        .max(10.);
        let monthly = (target.stocks.stock[0] + next_pop + j.population() * 0.5) * diet / 0.95;
        reserved.sites[to].stocks.stock[2] = monthly;
        reserved.society.as_mut().unwrap().relocation.sites[to].production = [monthly; 12];
        reserved.society.as_mut().unwrap().relocation.sites[from].last_departure = 0;
        reserved.relocation_departures();
        assert_eq!(reserved.household_relocations().unwrap().journeys.len(), 1);
        reserved
            .society
            .as_mut()
            .unwrap()
            .relocation
            .journeys
            .clear();
        reserved.relocation_departures();
        assert_eq!(reserved.household_relocations().unwrap().journeys.len(), 1);
        // No request exists until an actual arrival carries it to a host.
        let mut appeal = h.clone();
        appeal.society.as_mut().unwrap().relocation.witnessed_relief = true;
        appeal.society.as_mut().unwrap().relocation.journeys[0].seek_help = true;
        assert!(appeal.household_relocations().unwrap().appeals.is_empty());
        for month in 25..=j.arrives {
            appeal.month = month;
            appeal.relocation_arrivals();
            if month < j.arrives {
                assert!(appeal.household_relocations().unwrap().appeals.is_empty());
            }
        }
        assert_eq!(appeal.household_relocations().unwrap().appeals.len(), 1);
        let mut report = j.clone();
        report.seek_help = true;
        appeal.receive_appeal(&report);
        assert_eq!(appeal.household_relocations().unwrap().appeals.len(), 1);
        appeal.month += 1;
        // Distance and stale testimony can each prevent help despite ample food.
        let mut stale = appeal.clone();
        stale.month += 24;
        stale.answer_appeals();
        assert_eq!(stale.events.last().unwrap().kind, "relief_appeal_declined");
        let mut distant = appeal.clone();
        distant.governance = None;
        distant.society.as_mut().unwrap().routes[r.id as usize].cost_km = 1500.;
        // Ensure unrelated controllers for the diplomatic-distance fixture.
        distant.sites[to].civilization =
            (distant.sites[from].civilization + 1) % distant.civilizations.len() as u32;
        distant.answer_appeals();
        assert_eq!(
            distant.events.last().unwrap().kind,
            "relief_appeal_declined"
        );
        appeal.society.as_mut().unwrap().routes[r.id as usize].cost_km = 150.;
        let mut poor = appeal.clone();
        poor.sites[to].stocks.stock[1] = 0.;
        poor.answer_appeals();
        assert_eq!(poor.events.last().unwrap().kind, "relief_appeal_declined");
        // Religious relief is a funded fallback, carried by the exact same witnessed appeal.
        let mut religious = appeal.clone();
        let origin_controller = religious.controller(from as u32);
        religious.politics.as_mut().unwrap().controllers[to] =
            (origin_controller + 1) % religious.civilizations.len() as u32;
        let host = &mut religious.sites[to];
        host.stocks.stock[1] = host.stocks.stock[0] * 18. * 8.;
        host.economy.prices[crate::economy::FOOD] = 1.;
        let donated = host.economy.finance[0].min(1000.);
        host.economy.finance[0] -= donated;
        let culture = religious.culture.as_mut().unwrap();
        culture.religious_relief.enabled = true;
        let tradition = culture.site_faith[to];
        culture.traditions[tradition as usize].themes[0] = 0; // hospitality admits outsiders
        culture.traditions[tradition as usize].themes[1] = 5; // reciprocal assistance
        let institution = culture.institutions.len() as u32;
        let leader = religious
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == to as u32)
            .unwrap()
            .head;
        culture.institutions.push(crate::culture::Institution {
            capacity: None,
            id: institution,
            name: "Witness shelter".into(),
            kind: crate::culture::InstitutionKind::Religious,
            site: to as u32,
            tradition: Some(tradition),
            members: vec![leader],
            leader,
            treasury: donated as f64,
            active: true,
            founded: religious.month,
            knowledge: Default::default(),
            property: vec![],
            dues: 0.,
            expenses: 0.,
        });
        let mut disabled = religious.clone();
        disabled.culture.as_mut().unwrap().religious_relief.enabled = false;
        disabled.answer_appeals();
        assert!(disabled.shipments.is_empty());
        for condition in ["poor", "closed", "inactive", "stale"] {
            let mut blocked = religious.clone();
            match condition {
                "poor" => {
                    blocked.culture.as_mut().unwrap().institutions[institution as usize].treasury =
                        0.
                }
                "closed" => blocked.society.as_mut().unwrap().routes[r.id as usize].open = false,
                "inactive" => {
                    blocked.culture.as_mut().unwrap().institutions[institution as usize].active =
                        false
                }
                _ => blocked.month += 24,
            }
            blocked.answer_appeals();
            assert!(
                blocked.shipments.is_empty(),
                "{condition} must block dispatch"
            );
        }
        let religious_faith = religious.culture.as_ref().unwrap().household_faith.clone();
        let before_economy = religious.economy_residuals();
        let religious_food = religious.food_residual();
        let affinity = religious.relief_affinity(from as u32, to as u32);
        religious.answer_appeals();
        assert_eq!(religious.shipments.len(), 1);
        let mission = &religious
            .culture
            .as_ref()
            .unwrap()
            .religious_relief
            .missions[0];
        assert!(mission.paid <= 250. && mission.promised_kg >= 18.);
        assert_eq!(
            religious.relief_affinity(from as u32, to as u32),
            affinity,
            "unreceived aid is not local evidence"
        );
        assert!((religious.food_residual() - religious_food).abs() < 1e-6);
        for (a, b) in before_economy.iter().zip(religious.economy_residuals()) {
            assert!((a - b).abs() < 1e-6);
        }
        religious.answer_appeals();
        assert_eq!(religious.shipments.len(), 1, "appeals cannot pay twice");
        let before = appeal.economy_residuals();
        let before_food = appeal.food_residual();
        let n = appeal.shipments.len();
        appeal.answer_appeals();
        assert_eq!(appeal.shipments.len(), n + 1);
        assert_eq!(appeal.events.last().unwrap().kind, "appeal_relief_sent");
        assert!((appeal.food_residual() - before_food).abs() < 1e-6);
        for (a, b) in before.iter().zip(appeal.economy_residuals()) {
            assert!((a - b).abs() < 1e-6);
        }
        appeal.answer_appeals();
        assert_eq!(appeal.shipments.len(), n + 1);
        appeal
            .household_relocations()
            .unwrap()
            .validate(&appeal)
            .unwrap();
        let mut choices = h.clone();
        choices
            .society
            .as_mut()
            .unwrap()
            .relocation
            .journeys
            .clear();
        choices
            .society
            .as_mut()
            .unwrap()
            .relocation
            .witnessed_relief = true;
        choices.society.as_mut().unwrap().relocation.sites[from].last_departure = 0;
        choices.society.as_mut().unwrap().relocation.sites[from].hungry = 0b111111;
        for hh in &choices.society.as_ref().unwrap().households {
            if hh.site == from as u32 {
                choices.culture.as_mut().unwrap().agents[hh.head as usize].traits =
                    [0., 0.5, 0.5, 0.5, 1., 1.];
            }
        }
        choices.relocation_departures();
        assert!(choices.household_relocations().unwrap().journeys.is_empty());
        assert_eq!(choices.events.last().unwrap().kind, "households_stay");
        choices.society.as_mut().unwrap().relocation.sites[from].hungry = 0xffffff;
        for hh in &choices.society.as_ref().unwrap().households {
            if hh.site == from as u32 {
                choices.culture.as_mut().unwrap().agents[hh.head as usize].traits =
                    [1., 0.5, 0.5, 0.5, 0., 0.];
            }
        }
        choices.relocation_departures();
        assert_eq!(choices.household_relocations().unwrap().journeys.len(), 1);
        let mut lost = h.clone();
        lost.society.as_mut().unwrap().routes[r.id as usize].open = false;
        for month in 25..=j.arrives + 180 {
            lost.month = month;
            lost.relocation_arrivals();
        }
        assert!(lost.household_relocations().unwrap().journeys.is_empty());
        assert!(lost
            .household_relocations()
            .unwrap()
            .lost_households
            .contains(&j.household));
        assert!(lost.household_relocations().unwrap().away(j.household));
        assert!((lost.population_residual() - pop).abs() < 1e-6);
        assert!((lost.food_residual() - food).abs() < 1e-6);
        for (a, b) in expected.iter().zip(lost.economy_residuals()) {
            assert!((a - b).abs() < 1e-6, "lost ledger {a} {b}");
        }
        let mut returned = h.clone();
        returned.sites[to].abandoned = true;
        for month in 25..=j.arrives * 2 - 24 {
            returned.month = month;
            returned.relocation_arrivals();
        }
        assert!(returned
            .household_relocations()
            .unwrap()
            .journeys
            .is_empty());
        assert_eq!(
            returned.society.as_ref().unwrap().households[j.household as usize].site,
            from as u32
        );
        assert!(returned
            .events
            .iter()
            .any(|e| e.kind == "household_returned"));
        assert!((returned.population_residual() - pop).abs() < 1e-6);
        assert!((returned.food_residual() - food).abs() < 1e-6);
        let mut blocked = h.clone();
        blocked.society.as_mut().unwrap().routes[r.id as usize].open = false;
        for month in 25..=j.arrives + 30 {
            blocked.month = month;
            blocked.relocation_arrivals();
        }
        assert!(blocked.household_relocations().unwrap().journeys[0].population() < j.population());
        assert!(blocked
            .events
            .iter()
            .any(|e| e.kind == "household_journey_blocked"));
        assert!((blocked.population_residual() - pop).abs() < 1e-6);
        assert!((blocked.food_residual() - food).abs() < 1e-6);
        for (a, b) in expected.iter().zip(blocked.economy_residuals()) {
            assert!((a - b).abs() < 1e-6, "blocked ledger {a} {b}");
        }
        blocked.society.as_mut().unwrap().routes[r.id as usize].open = true;
        blocked.month += 1;
        blocked.relocation_arrivals();
        assert!(blocked.household_relocations().unwrap().journeys.is_empty());
        // Policy affects new departures, never strands an already funded journey.
        h.set_household_relocation(false).unwrap();
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&h).unwrap()).unwrap();
        for month in 25..=j.arrives {
            h.month = month;
            resumed.month = month;
            h.relocation_arrivals();
            resumed.relocation_arrivals();
            h.culture_month();
            resumed.culture_month();
        }
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        assert!(h.household_relocations().unwrap().journeys.is_empty());
        assert_eq!(
            h.events[j.cause as usize].planned_path.as_ref().unwrap(),
            &r.cells
        );

        let arrival = h
            .events
            .iter()
            .rev()
            .find(|e| e.kind == "household_arrival")
            .unwrap();
        assert_eq!(
            arrival
                .spatial
                .as_ref()
                .unwrap()
                .iter()
                .find(|a| a.role == crate::spatial::EventRole::Milestone)
                .unwrap()
                .cell,
            h.sites[to].cell
        );
        assert!(h
            .events
            .iter()
            .filter(|e| e.kind == "household_journey_blocked" || e.kind == "household_journey_lost")
            .all(|e| e
                .spatial
                .as_ref()
                .unwrap()
                .iter()
                .all(|a| a.role != crate::spatial::EventRole::Milestone)));
        assert_eq!(
            h.society.as_ref().unwrap().households[j.household as usize].site,
            to as u32
        );
        assert_eq!(
            h.culture.as_ref().unwrap().household_faith[j.household as usize],
            faith
        );
        assert_eq!(h.people[head as usize].civilization, ancestry);
        assert!((h.food_residual() - food).abs() < 1e-6);
        assert!((h.population_residual() - pop).abs() < 1e-6);
        for (a, b) in expected.iter().zip(h.economy_residuals()) {
            assert!((a - b).abs() < 1e-6, "{a} {b}");
        }
        h.household_relocations().unwrap().validate(h).unwrap();
        // No provisions or no destination capacity means no departure, not free rescue.
        h.month = 48;
        h.set_household_relocation(true).unwrap();
        h.sites[from].stocks.stock[1] = 0.;
        h.relocation_departures();
        assert!(h.household_relocations().unwrap().journeys.is_empty());
        let mut lost = religious.clone();
        lost.society.as_mut().unwrap().routes[r.id as usize].open = false;
        for _ in 0..8 {
            lost.month += 1;
            lost.relief_arrivals();
        }
        assert_eq!(
            lost.culture.as_ref().unwrap().religious_relief.missions[0].delivered_kg,
            Some(0.)
        );
        assert_eq!(
            lost.culture
                .as_ref()
                .unwrap()
                .religious_relief
                .received_kg(from as u32, to as u32),
            0.
        );
        assert!(
            (lost.food_residual() - religious_food).abs() < 1e-6,
            "loss residual delta: {}",
            lost.food_residual() - religious_food
        );
        for (a, b) in before_economy.iter().zip(lost.economy_residuals()) {
            assert!((a - b).abs() < 1e-6);
        }
        // Serialize the history at the completed dispatch boundary, then use exactly
        // the arrival method called by Generator's monthly pipeline.
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&religious).unwrap()).unwrap();
        for _ in 0..2 {
            religious.month += 1;
            religious.relief_arrivals();
        }
        resumed.month += 1;
        resumed.relief_arrivals();
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&resumed).unwrap()).unwrap();
        resumed.month += 1;
        resumed.relief_arrivals();
        assert_eq!(
            serde_json::to_value(&religious).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let arrived = &religious;
        assert!((arrived.food_residual() - religious_food).abs() < 1e-6);
        for (a, b) in before_economy.iter().zip(arrived.economy_residuals()) {
            assert!((a - b).abs() < 1e-6);
        }
        arrived
            .culture
            .as_ref()
            .unwrap()
            .religious_relief
            .validate(
                arrived,
                arrived.culture.as_ref().unwrap().institutions.len(),
            )
            .unwrap();

        assert!(
            arrived.culture.as_ref().unwrap().religious_relief.missions[0]
                .delivered_kg
                .unwrap()
                > 0.
        );
        assert_eq!(
            arrived.culture.as_ref().unwrap().household_faith,
            religious_faith
        );
        let mut no_trust = arrived.clone();
        no_trust.culture.as_mut().unwrap().religious_relief.enabled = false;
        assert!(
            arrived.relief_affinity(from as u32, to as u32)
                > no_trust.relief_affinity(from as u32, to as u32)
        );
        assert!(
            arrived
                .culture
                .as_ref()
                .unwrap()
                .religious_relief
                .institution_trust(
                    from as u32,
                    institution,
                    arrived.sites[from].stocks.stock[0]
                )
                > 0.5
        );

        assert!(
            arrived
                .culture
                .as_ref()
                .unwrap()
                .religious_relief
                .received_kg(from as u32, to as u32)
                > 0.
        );
        // Reports become known only at receipt; repayment uses real reverse cargo.
        assert!(arrived.destination_memory(from as u32, to as u32) > 1.);
        let mut reverse = arrived.clone();
        let owed = reverse
            .culture
            .as_ref()
            .unwrap()
            .religious_relief
            .owed_kg(from as u32, to as u32);
        assert!(owed > 0.);
        let c = reverse.culture.as_mut().unwrap();
        let mut order = c.institutions[institution as usize].clone();
        order.id = c.institutions.len() as u32;
        order.site = from as u32;
        let reverse_id = order.id;
        // A non-hospitality, different-faith order can honor the local obligation.
        let witness = reverse.society.as_ref().unwrap().relocation.appeals[0].household;
        let local_tradition = (c.household_faith[witness as usize] + 1) % c.traditions.len() as u32;
        order.tradition = Some(local_tradition);
        c.traditions[local_tradition as usize].themes = [1, 2, 3, 4];
        order.treasury = 0.;
        c.institutions.push(order);
        let host = &mut reverse.sites[from];
        host.stocks.stock[1] = host.stocks.stock[0] * 18. * 20.;
        host.stocks.stock[3] = 0.;
        host.economy.prices[crate::economy::FOOD] = 1.;
        // Controlled recovered economy, measured from this declared fixture baseline.
        host.economy.finance[0] = 10000.;
        host.economy.finance[0] -= 5000.;
        reverse.culture.as_mut().unwrap().institutions[reverse_id as usize].treasury = 5000.;
        let baseline_food = reverse.food_residual();
        let baseline_money = reverse.economy_residuals();
        let faith = reverse.culture.as_ref().unwrap().household_faith.clone();
        let mut request = reverse.society.as_ref().unwrap().relocation.appeals[0].clone();
        request.origin = to as u32;
        request.host = from as u32;
        request.reported = reverse.month;
        request.population = reverse.sites[to].stocks.stock[0];
        let mut no_obligation = reverse.clone();
        no_obligation
            .culture
            .as_mut()
            .unwrap()
            .religious_relief
            .missions[0]
            .reciprocal = false;
        assert!(
            !no_obligation.sponsor_religious_relief(&request),
            "without the obligation an unrelated order declines"
        );
        assert!(reverse.sponsor_religious_relief(&request));
        assert_eq!(
            reverse
                .culture
                .as_ref()
                .unwrap()
                .religious_relief
                .owed_kg(from as u32, to as u32),
            owed
        );
        reverse.month += 1;
        reverse.relief_arrivals();
        let c = reverse.culture.as_ref().unwrap();
        assert!(c.religious_relief.owed_kg(from as u32, to as u32) < owed);
        assert_eq!(
            c.religious_relief.owed_kg(to as u32, from as u32),
            0.,
            "a gift creates no reverse debt"
        );
        assert!(c.religious_relief.missions.last().unwrap().repayment_kg > 0.);
        let before = serde_json::to_value(&reverse.culture).unwrap();
        reverse.relief_arrivals();
        assert_eq!(
            before,
            serde_json::to_value(&reverse.culture).unwrap(),
            "delivery cannot repay twice"
        );
        assert!((reverse.food_residual() - baseline_food).abs() < 1e-6);
        for (a, b) in baseline_money.iter().zip(reverse.economy_residuals()) {
            assert!((a - b).abs() < 1e-6);
        }
        assert_eq!(faith, reverse.culture.as_ref().unwrap().household_faith);
        // Repeated assistance transmits a practice through witnessed outcomes.
        let c = reverse.culture.as_mut().unwrap();
        c.traditions[local_tradition as usize].themes[0] = 0;
        assert!(reverse.sponsor_religious_relief(&request));
        reverse.month += 1;
        reverse.relief_arrivals();
        assert!(reverse
            .culture
            .as_ref()
            .unwrap()
            .religious_relief
            .memory
            .mutual_aid_sites
            .contains(&(to as u32)));
        assert!(reverse
            .events
            .iter()
            .any(|e| e.kind == "mutual_aid_learned" && !e.causes.is_empty()));
        assert_eq!(faith, reverse.culture.as_ref().unwrap().household_faith);
        reverse
            .culture
            .as_ref()
            .unwrap()
            .religious_relief
            .validate(
                &reverse,
                reverse.culture.as_ref().unwrap().institutions.len(),
            )
            .unwrap();
    }
}
