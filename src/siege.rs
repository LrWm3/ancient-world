//! Small land sieges over canonical structures, existing armies and reserved freight.
use crate::{
    civilization::History,
    culture::{Artifact, Owner},
    society::{Raid, Society},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Defense {
    pub site: u32,
    pub artifact: u32,
    pub required: f32,
    pub completed: f32,
    pub integrity: f32,
    pub work: crate::labor::WorkReceipt,
    pub commitment: Option<u32>,
    pub individual: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Siege {
    pub army: u32,
    pub site: u32,
    pub started: u32,
    pub observed: u32,
    pub ended: Option<u32>,
    pub cause: u64,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Supply {
    pub army: u32,
    pub from: u32,
    pub to: u32,
    pub route: u32,
    pub food: f32,
    pub departed: u32,
    pub due: u32,
    pub duration: u32,
    pub returning: bool,
    pub finished: Option<u32>,
    pub cause: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub defenses: Vec<Defense>,
    pub sieges: Vec<Siege>,
    pub supplies: Vec<Supply>,
}
impl History {
    pub fn commission_defenses(&mut self, site: u32, bricks: f32) -> Result<()> {
        ensure!(
            (site as usize) < self.sites.len()
                && bricks.is_finite()
                && (100. ..=100_000.).contains(&bricks),
            "invalid defense project"
        );
        ensure!(
            self.culture.is_some() && !self.sites[site as usize].abandoned && !self.besieged(site),
            "construction requires an accessible occupied site"
        );
        ensure!(
            self.military.siege.defenses.iter().all(|d| d.site != site
                || self
                    .culture
                    .as_ref()
                    .is_some_and(|c| !c.artifacts[d.artifact as usize].destroyed)),
            "destroyed defenses require a new recovery project"
        );
        ensure!(
            self.sites[site as usize].economy.goods[5] >= bricks,
            "insufficient real bricks"
        );
        ensure!(
            self.military
                .siege
                .defenses
                .iter()
                .all(|d| d.site != site || d.work.settled || d.work.month == 0),
            "finish current construction grant"
        );
        self.event("defense_commissioned",Some(site),None,format!("{bricks:.1} kg bricks reserved in persistent defenses; construction still requires work"));
        let event = self.events.last().unwrap().id;
        self.sites[site as usize].economy.goods[5] -= bricks;
        if let Some(d) = self
            .military
            .siege
            .defenses
            .iter_mut()
            .find(|d| d.site == site)
        {
            let a = &mut self.culture.as_mut().unwrap().artifacts[d.artifact as usize];
            a.materials[0].1 += bricks;
            a.events.push(event);
            let old = d.required;
            d.required += bricks / 500.;
            d.integrity *= old / d.required;
        } else {
            let c = self.culture.as_mut().unwrap();
            let artifact = c.artifacts.len() as u32;
            c.artifacts.push(Artifact {
                id: artifact,
                name: format!("{} defenses", self.sites[site as usize].name),
                kind: "fortification".into(),
                creator: None,
                owner: Owner::Community(site),
                claims: vec![],
                site: Some(site),
                custodian: None,
                materials: vec![(5, bricks)],
                topic: None,
                tradition: None,
                events: vec![event],
                destroyed: false,
                lost: false,
            });
            self.military.siege.defenses.push(Defense {
                site,
                artifact,
                required: bricks / 500.,
                completed: 0.,
                integrity: 0.,
                work: Default::default(),
                commitment: None,
                individual: false,
            });
        }
        Ok(())
    }
    pub(crate) fn reserve_defense_work(&mut self) {
        let mut defs = std::mem::take(&mut self.military.siege.defenses);
        for d in &mut defs {
            if d.work.month == self.month {
                continue;
            }
            let wanted = (d.required - d.completed)
                .clamp(0., 0.5)
                .min(crate::labor::available(
                    &self.sites[d.site as usize],
                    self.society.is_some(),
                    self.living.is_some(),
                ));
            let live = !self.sites[d.site as usize].abandoned
                && !self.besieged(d.site)
                && self
                    .culture
                    .as_ref()
                    .is_some_and(|c| !c.artifacts[d.artifact as usize].destroyed);
            let people = self
                .culture
                .as_ref()
                .map_or(vec![], |c| c.site_people(self, d.site))
                .into_iter()
                .filter(|id| {
                    self.participation.as_ref().is_none_or(|p| {
                        p.residents.get(id).is_some_and(|r| {
                            r.presence == crate::participation::Presence::Resident(d.site)
                        }) && p.available(*id) > 1e-6
                    })
                })
                .collect::<Vec<_>>();
            d.individual = self.participation.is_some();
            d.commitment = if live {
                self.participation.as_mut().and_then(|p| {
                    p.reserve(
                        self.month,
                        d.site,
                        crate::participation::Activity::Construction,
                        &people,
                        wanted,
                    )
                })
            } else {
                None
            };
            let grant = if !live {
                0.
            } else if let Some(p) = &self.participation {
                d.commitment
                    .map_or(0., |id| p.commitments[id as usize].granted)
            } else {
                wanted
            };
            self.sites[d.site as usize].economy.external[3] += grant;
            d.work = crate::labor::WorkReceipt {
                month: self.month,
                requested: wanted as f64,
                granted: grant as f64,
                ..Default::default()
            };
        }
        self.military.siege.defenses = defs;
    }
    pub(crate) fn settle_defense_work(&mut self) -> Result<()> {
        let mut defs = std::mem::take(&mut self.military.siege.defenses);
        let result = (|| -> Result<()> {
            for d in &mut defs {
                if d.work.month != self.month || d.work.settled {
                    continue;
                }
                let accessible = !self.sites[d.site as usize].abandoned
                    && !self.besieged(d.site)
                    && self
                        .culture
                        .as_ref()
                        .is_some_and(|c| !c.artifacts[d.artifact as usize].destroyed);
                let used = if !accessible {
                    0.
                } else if d.individual {
                    self.personal_grant_live(d.commitment)
                        .min(d.work.granted as f32)
                } else {
                    d.work.granted as f32
                };
                if let Some(id) = d.commitment {
                    self.participation.as_mut().unwrap().settle(id, used)?;
                }
                d.completed += used;
                d.integrity = (d.integrity + used / d.required.max(0.01)).min(1.);
                self.sites[d.site as usize].economy.external[3] =
                    (self.sites[d.site as usize].economy.external[3] - d.work.granted as f32)
                        .max(0.);
                d.work.settle(used as f64);
            }
            Ok(())
        })();
        self.military.siege.defenses = defs;
        result
    }
    pub(crate) fn end_siege(&mut self, army: u32, reason: &str) {
        for s in &mut self.military.siege.sieges {
            if s.army == army && s.ended.is_none() {
                s.ended = Some(self.month);
                s.reason = reason.into();
            }
        }
    }
    pub fn besieged(&self, site: u32) -> bool {
        self.military
            .siege
            .sieges
            .iter()
            .any(|s| s.site == site && s.ended.is_none())
    }
    pub(crate) fn siege_month(&mut self, r: &mut Raid, society: &Society) -> bool {
        let existing = self
            .military
            .siege
            .sieges
            .iter()
            .position(|s| s.army == r.id && s.ended.is_none());
        if r.returning {
            if let Some(i) = existing {
                self.military.siege.sieges[i].ended = Some(self.month);
                self.military.siege.sieges[i].reason = "withdrawal".into();
            }
            return false;
        }
        if r.war.is_none() || !self.campaign_authorized(r) {
            self.end_siege(r.id, "political access lost");
            return false;
        }
        if r.arrives > self.month {
            return false;
        }
        let Some(di) = self.military.siege.defenses.iter().position(|d| {
            d.site == r.target
                && d.integrity > 0.02
                && self
                    .culture
                    .as_ref()
                    .is_some_and(|c| !c.artifacts[d.artifact as usize].destroyed)
        }) else {
            if let Some(i) = existing {
                self.military.siege.sieges[i].ended = Some(self.month);
                self.military.siege.sieges[i].reason = "defenses breached".into();
            }
            return false;
        };
        let i = if let Some(i) = existing {
            i
        } else {
            if self.besieged(r.target) {
                r.returning = true;
                r.arrives = self.month + r.return_duration(society);
                return true;
            }
            self.event(
                "siege_started",
                Some(r.target),
                Some(r.origin),
                format!(
                    "Army {} encircled the land approaches; sea access is not blockaded",
                    r.id
                ),
            );
            let cause = self.events.last().unwrap().id;
            self.events.last_mut().unwrap().causes.push(r.cause);
            self.military.siege.sieges.push(Siege {
                army: r.id,
                site: r.target,
                started: self.month,
                observed: self.month.saturating_sub(1),
                ended: None,
                cause,
                reason: String::new(),
            });
            self.military.siege.sieges.len() - 1
        };
        if self.military.siege.sieges[i].observed == self.month {
            return true;
        }
        self.military.siege.sieges[i].observed = self.month;
        let return_food = r.soldiers * 18. * (r.return_duration(society) + 1) as f32;
        if r.food < return_food || self.sites[r.target as usize].abandoned {
            self.military.siege.sieges[i].ended = Some(self.month);
            self.military.siege.sieges[i].reason = "withdrew to preserve return provisions".into();
            r.returning = true;
            r.arrives = self.month + r.return_duration(society);
            self.event(
                "siege_withdrawal",
                Some(r.target),
                Some(r.origin),
                format!("Army {} reserved remaining provisions for withdrawal", r.id),
            );
            self.events
                .last_mut()
                .unwrap()
                .causes
                .push(self.military.siege.sieges[i].cause);
            self.resolve_war(r, false);
            return true;
        }
        let d = &mut self.military.siege.defenses[di];
        d.integrity = (d.integrity - r.soldiers * 0.02 / d.required.max(1.)).max(0.);
        r.arrives = self.month + 1; // One supplied siege interval, no repeated battle this month.
        true
    }
    pub fn send_military_supply(&mut self, army: u32, requested: f32) -> Result<f32> {
        ensure!(
            requested.is_finite() && requested > 0.,
            "invalid resupply quantity"
        );
        let society = self
            .society
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no armies"))?;
        let r = society
            .raids
            .iter()
            .find(|r| r.id == army && !r.returning)
            .ok_or_else(|| anyhow::anyhow!("army unavailable"))?;
        ensure!(
            r.occupation_until.is_some()
                || self
                    .military
                    .siege
                    .sieges
                    .iter()
                    .any(|s| s.army == r.id && s.ended.is_none()),
            "resupply requires an established camp"
        );
        let route = society
            .routes
            .iter()
            .find(|p| {
                p.passable()
                    && ((p.from == r.origin && p.to == r.target)
                        || (p.to == r.origin && p.from == r.target))
            })
            .ok_or_else(|| anyhow::anyhow!("no open direct supply corridor"))?;
        let edge = [r.origin.min(r.target), r.origin.max(r.target)];
        let source = &self.sites[r.origin as usize];
        let available = (source.stocks.stock[1] - source.stocks.stock[0] * 18. * 2.).max(0.);
        let reserved: f32 = self
            .military
            .siege
            .supplies
            .iter()
            .filter(|s| s.from == r.origin)
            .map(|s| s.food)
            .sum();
        let food = requested
            .min(available)
            .min(self.road_freight_capacity(edge))
            .min(self.land_freight_capacity(r.origin))
            .min((source.stocks.stock[0] * 10. - reserved).max(0.));
        ensure!(
            food >= 1.,
            "no food or freight capacity above civilian reserve"
        );
        let (from, to, route_id, duration) =
            (r.origin, r.target, route.id, r.return_duration(society));
        let before = self.sites[from as usize].stocks.stock[1];
        self.sites[from as usize].stocks.stock[1] -= food;
        let food = before - self.sites[from as usize].stocks.stock[1];
        self.event(
            "military_supply_departure",
            Some(from),
            Some(to),
            format!("{food:.1} kg real provisions dispatched to army {army}"),
        );
        let cause = self.events.last().unwrap().id;
        self.military.siege.supplies.push(Supply {
            army,
            from,
            to,
            route: route_id,
            food,
            departed: self.month,
            due: self.month + duration,
            duration,
            returning: false,
            finished: None,
            cause,
        });
        Ok(food)
    }
    pub(crate) fn military_supply_arrivals(&mut self) {
        let mut shipments = std::mem::take(&mut self.military.siege.supplies);
        for s in &mut shipments {
            if s.finished.is_some() || s.due > self.month {
                continue;
            }
            let Some(soc) = &mut self.society else {
                continue;
            };
            if soc
                .routes
                .get(s.route as usize)
                .is_none_or(|r| !r.passable())
            {
                continue;
            }
            if s.returning {
                self.sites[s.from as usize].stocks.stock[1] += s.food;
            } else if let Some(r) = soc
                .raids
                .iter_mut()
                .find(|r| r.id == s.army && !r.returning && r.origin == s.from && r.target == s.to)
            {
                r.food += s.food;
            } else {
                s.returning = true;
                s.due = self.month + s.duration;
                continue;
            }
            self.event(
                "military_supply_arrival",
                Some(if s.returning { s.from } else { s.to }),
                Some(s.from),
                format!(
                    "{:.1} kg provisions {} for army {}",
                    s.food,
                    if s.returning { "returned" } else { "delivered" },
                    s.army
                ),
            );
            self.events.last_mut().unwrap().causes.push(s.cause);
            s.food = 0.;
            s.finished = Some(self.month);
        }
        self.military.siege.supplies = shipments;
    }
}
impl State {
    pub(crate) fn validate(&self, h: &History) -> Result<()> {
        let mut sites = std::collections::BTreeSet::new();
        for d in &self.defenses {
            ensure!(
                (d.site as usize) < h.sites.len()
                    && sites.insert(d.site)
                    && h.culture.as_ref().is_some_and(|c| c
                        .artifacts
                        .get(d.artifact as usize)
                        .is_some_and(|a| a.kind == "fortification" && a.site == Some(d.site)))
                    && d.required.is_finite()
                    && d.required > 0.
                    && d.completed.is_finite()
                    && d.completed >= 0.
                    && d.completed <= d.required + 1e-3
                    && (0. ..=1.).contains(&d.integrity),
                "invalid defense"
            );
            d.work.validate()?;
        }
        for s in &self.supplies {
            ensure!(
                [s.from, s.to].iter().all(|i| (*i as usize) < h.sites.len())
                    && s.from != s.to
                    && s.food.is_finite()
                    && s.food >= 0.
                    && s.duration > 0
                    && s.duration <= 10
                    && s.departed <= h.month
                    && s.due > s.departed
                    && s.finished.is_none_or(|m| m >= s.departed && m <= h.month)
                    && (s.cause as usize) < h.events.len()
                    && s.finished.is_none_or(|_| s.food == 0.)
                    && h.society.as_ref().is_some_and(|x| x
                        .routes
                        .get(s.route as usize)
                        .is_some_and(|r| [r.from.min(r.to), r.from.max(r.to)]
                            == [s.from.min(s.to), s.from.max(s.to)])),
                "invalid military freight"
            );
        }
        let mut active = std::collections::BTreeSet::new();
        for s in &self.sieges {
            ensure!(
                (s.site as usize) < h.sites.len()
                    && s.started <= s.observed
                    && s.observed <= h.month
                    && s.ended.is_none_or(|m| m >= s.started && m <= h.month)
                    && (s.cause as usize) < h.events.len()
                    && (s.ended.is_some()
                        || (active.insert(s.site)
                            && h.society
                                .as_ref()
                                .is_some_and(|x| x.raids.iter().any(|r| r.id == s.army
                                    && !r.returning
                                    && r.target == s.site)))),
                "invalid siege"
            );
        }
        Ok(())
    }
}
impl crate::gpu::Generator {
    pub fn commission_defenses(&mut self, site: u32, bricks: f32) -> Result<()> {
        self.validate_living_boundary()?;
        self.civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?
            .commission_defenses(site, bricks)
    }
    pub fn send_military_supply(&mut self, army: u32, food: f32) -> Result<f32> {
        self.validate_living_boundary()?;
        self.civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?
            .send_military_supply(army, food)
    }
}
#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires hardware GPU"]
    fn built_defenses_delay_war_and_real_resupply_competes_with_freight() {
        let mut g = crate::continuity_fixture::world();
        let h = g.civilizations.as_mut().unwrap();
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .find(|r| r.cost_km < 900. && h.controller(r.from) != h.controller(r.to))
            .unwrap()
            .clone();
        let target = route.to as usize;
        // Declared fixture import, not production by the construction operation.
        h.sites[target].economy.goods[5] += 5000.;
        h.sites[target].economy.initial[5] += 5000.;
        h.commission_defenses(route.to, 5000.).unwrap();
        assert_eq!(h.military.siege.defenses[0].integrity, 0.);
        assert!(h.commission_defenses(route.to, 100000.).is_err());
        g.advance_history(24).unwrap();
        let h = g.civilizations.as_ref().unwrap();
        assert!(
            h.military.siege.defenses[0].integrity > 0.9,
            "{:?}",
            h.military.siege.defenses[0]
        );
        assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
        // Fund the controlled military scenario from declared imports; ordinary
        // settlement economics must not accidentally decide whether this fixture starts.
        let h = g.civilizations.as_mut().unwrap();
        let source = &mut h.sites[route.from as usize];
        let before = source.stocks.stock[1];
        source.stocks.stock[1] += 100000.;
        let added = source.stocks.stock[1] - before;
        h.initial_food += added as f64;
        for (k, ratio) in crate::economy::FOOD_CNP.iter().enumerate() {
            h.nutrition_initial[k] += added as f64 * ratio;
        }
        source.economy.goods[3] += 100.;
        source.economy.initial[3] += 100.;
        for (k, ratio) in h
            .economy_catalog
            .as_ref()
            .unwrap()
            .composition(3)
            .iter()
            .enumerate()
        {
            h.nutrition_initial[k] += 100. * *ratio as f64;
        }
        g.declare_war(route.from, route.to).unwrap();
        let h = g.civilizations.as_ref().unwrap();
        let arrives = h.society.as_ref().unwrap().raids[0].arrives;
        g.advance_history(arrives - h.month).unwrap();
        let h = g.civilizations.as_mut().unwrap();
        assert!(h.besieged(route.to));
        assert_eq!(h.land_freight_capacity(route.to), 0.);
        assert!(h.politics.as_ref().unwrap().wars[0].ended.is_none());
        let id = h.society.as_ref().unwrap().raids[0].id;
        h.sites[route.from as usize].economy.logistics[3] = 1.;
        let original = h.sites[route.from as usize].stocks.stock[1];
        let capacity = h.land_freight_capacity(route.from);
        let sent = h.send_military_supply(id, 100.).unwrap();
        assert!((original - h.sites[route.from as usize].stocks.stock[1] - sent).abs() < 0.001);
        assert!(capacity.is_finite());
        assert!((capacity - h.land_freight_capacity(route.from) - sent).abs() < 0.01);
        assert!(h.food_residual().abs() < 0.001);
        let mut shortage = h.clone();
        let r = &mut shortage.society.as_mut().unwrap().raids[0];
        let consumed = r.food;
        r.food = 0.;
        shortage.sites[route.from as usize].stocks.ledger[1] += consumed;
        for (k, ratio) in crate::economy::FOOD_CNP.iter().enumerate() {
            shortage.sites[route.from as usize].economy.external[k] -= consumed * *ratio as f32;
        }
        shortage.month += 1;
        shortage.social_month().unwrap();
        assert!(!shortage.besieged(route.to));
        assert!(shortage.society.as_ref().unwrap().raids[0].returning);
        assert!(shortage.population_residual().abs() < 0.001);
        assert!(shortage.food_residual().abs() < 0.001);
        // Peace preserves traveling food; undelivered supply returns through its route.
        let payer = h.controller(route.from);
        let payee = h.controller(route.to);
        let cash = h.sites[route.from as usize].economy.finance[0].min(4.);
        assert!(cash > 0.);
        h.sites[route.from as usize].economy.finance[0] -= cash;
        h.society.as_mut().unwrap().councils[payer as usize].treasury += cash as f64;
        let offer = h.offer_peace(0, payer, cash as f64, 1).unwrap();
        h.accept_peace(offer, payee).unwrap();
        assert!(!h.besieged(route.to));
        assert!(h.society.as_ref().unwrap().raids[0].returning);
        let path = std::path::Path::new("output/siege-continuation.world");
        g.save(path).unwrap();
        let mut resumed = crate::gpu::Generator::load(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            path,
        )
        .unwrap();
        g.advance_history(18).unwrap();
        for _ in 0..18 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
        let h = g.civilizations.as_ref().unwrap();
        assert!(h
            .military
            .siege
            .supplies
            .iter()
            .all(|s| s.finished.is_some()));
        assert_eq!(
            h.politics.as_ref().unwrap().wars[0].outcome,
            "negotiated peace"
        );
        h.validate(&g.snapshot().unwrap()).unwrap();
    }
}
