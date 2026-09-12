//! Sparse local offices. Authority references existing administrations and treasuries.
pub mod service;
use crate::{
    civilization::History,
    gpu::{Generator, Stage},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Selection {
    Ruler,
    LocalCouncil,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tenure {
    pub person: u32,
    pub began: u32,
    pub due: u32,
    pub ended: Option<u32>,
    pub appointment: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Office {
    pub id: u32,
    pub site: u32,
    pub title: String,
    pub created: u32,
    /// Jurisdiction references the existing controller and council; it holds no money.
    pub controller: u32,
    pub selection: Selection,
    pub tenures: Vec<Tenure>,
    pub last_event: u64,
}
impl Office {
    pub fn holder(&self) -> Option<u32> {
        self.tenures
            .last()
            .filter(|t| t.ended.is_none())
            .map(|t| t.person)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Offices {
    #[serde(default)]
    pub service: Option<service::Service>,
    pub started: u32,
    pub last_decision: Option<u32>,
    pub seats: Vec<Office>,
}
impl Offices {
    pub fn validate(&self, h: &History) -> Result<()> {
        if let Some(service) = &self.service {
            service.validate(h)?;
        }
        ensure!(
            h.governance.is_some()
                && h.culture.is_some()
                && self.started <= h.month
                && self.last_decision.is_none_or(|m| m <= h.month)
                && self.seats.len() == h.sites.len(),
            "invalid office baseline"
        );
        for (i, o) in self.seats.iter().enumerate() {
            ensure!(
                o.id as usize == i
                    && o.site == o.id
                    && o.created >= self.started
                    && o.created <= h.month
                    && o.controller == h.controller(o.site)
                    && o.selection == h.office_selection(o.site)
                    && h.events
                        .get(o.last_event as usize)
                        .is_some_and(|e| e.site == Some(o.site)
                            && matches!(
                                e.kind.as_str(),
                                "office_created"
                                    | "office_appointed"
                                    | "office_vacated"
                                    | "office_jurisdiction"
                            )),
                "invalid office jurisdiction"
            );
            for (j, t) in o.tenures.iter().enumerate() {
                ensure!(
                    (t.person as usize) < h.people.len()
                        && t.began >= o.created
                        && t.began <= h.month
                        && t.due == t.began + 48
                        && t.ended.is_none_or(|m| m >= t.began && m <= h.month)
                        && (j + 1 == o.tenures.len() || t.ended.is_some())
                        && (j == 0 || o.tenures[j - 1].ended.is_some_and(|m| m <= t.began))
                        && h.events
                            .get(t.appointment as usize)
                            .is_some_and(|e| e.kind == "office_appointed"
                                && e.site == Some(o.site)
                                && e.subjects.contains(&("person".into(), t.person))),
                    "invalid office tenure"
                );
            }
        }
        Ok(())
    }
}
impl History {
    fn office_selection(&self, site: u32) -> Selection {
        if self
            .governance
            .as_ref()
            .and_then(|g| g.administrations.get(site as usize))
            .is_some_and(|a| a.autonomy >= 0.75)
        {
            Selection::LocalCouncil
        } else {
            Selection::Ruler
        }
    }
    fn office_candidates(&self, site: u32, controller: u32, rule: Selection) -> Vec<u32> {
        if self.sites[site as usize].abandoned {
            return vec![];
        }
        self.culture.as_ref().map_or_else(Vec::new, |c| {
            c.site_people(self, site)
                .into_iter()
                .filter(|&p| {
                    rule == Selection::LocalCouncil
                        || self.people[p as usize].civilization == controller
                })
                .collect()
        })
    }
    /// Vacant offices retain half of ordinary collection through unnamed staff.
    /// Knowledge improves coordination, never the underlying cash or material stocks.
    pub fn office_capacity(&self, site: u32) -> f32 {
        if site as usize >= self.sites.len() {
            return 0.;
        }
        let Some(offices) = &self.offices else {
            return 1.;
        };
        let Some(o) = offices.seats.get(site as usize) else {
            return 0.5;
        };
        let Some(p) = o.holder() else {
            return 0.5;
        };
        if o.controller != self.controller(site)
            || !self
                .office_candidates(site, o.controller, o.selection)
                .contains(&p)
        {
            return 0.5;
        }
        let breadth = self
            .culture
            .as_ref()
            .and_then(|c| c.agents.get(p as usize))
            .map_or(0., |a| a.knowledge.len().min(12) as f32 / 12.);
        let capacity = 0.75 + 0.25 * breadth;
        offices.service.as_ref().map_or(capacity, |s| {
            0.5 + (capacity - 0.5) * s.delivered(self.month, site, p, o.controller)
        })
    }
    pub(crate) fn sync_offices(&mut self) {
        let Some(mut offices) = self.offices.take() else {
            return;
        };
        while offices.seats.len() < self.sites.len() {
            let site = offices.seats.len() as u32;
            let event = self.events.len() as u64;
            self.event("office_created",Some(site),None,"Local stewardship established at the present cultural baseline; tax collection uses the existing council treasury".into());
            self.events
                .last_mut()
                .unwrap()
                .subjects
                .push(("office".into(), site));
            offices.seats.push(Office {
                id: site,
                site,
                title: format!("Steward of {}", self.sites[site as usize].name),
                created: self.month,
                controller: self.controller(site),
                selection: self.office_selection(site),
                tenures: vec![],
                last_event: event,
            });
        }
        for o in &mut offices.seats {
            let controller = self.controller(o.site);
            let selection = self.office_selection(o.site);
            let constitutional = controller != o.controller || selection != o.selection;
            let eligible = self.office_candidates(o.site, controller, selection);
            if let Some(t) = o.tenures.last_mut().filter(|t| t.ended.is_none()) {
                let reason = if constitutional {
                    Some("jurisdiction or selection rule changed")
                } else if self.month >= t.due {
                    Some("four-year term completed")
                } else if !eligible.contains(&t.person) {
                    Some("holder died, left, or ceased to be eligible")
                } else {
                    None
                };
                if let Some(reason) = reason {
                    t.ended = Some(self.month);
                    self.event(
                        "office_vacated",
                        Some(o.site),
                        None,
                        format!("{} became vacant: {reason}", o.title),
                    );
                    let e = self.events.last_mut().unwrap();
                    e.subjects
                        .extend([("office".into(), o.id), ("person".into(), t.person)]);
                    e.causes.push(o.last_event);
                    o.last_event = e.id;
                }
            }
            if constitutional {
                self.event("office_jurisdiction",Some(o.site),None,format!("{} now answers to administration {controller} under {selection:?} selection",o.title));
                let e = self.events.last_mut().unwrap();
                e.subjects.push(("office".into(), o.id));
                e.causes.push(o.last_event);
                if let Some(cause) = self
                    .governance
                    .as_ref()
                    .and_then(|g| g.administrations[o.site as usize].cause)
                {
                    if !e.causes.contains(&cause) {
                        e.causes.push(cause);
                    }
                }
                o.last_event = e.id;
                o.controller = controller;
                o.selection = selection;
            }
        }
        self.offices = Some(offices);
    }
    pub(crate) fn office_month(&mut self) {
        self.sync_offices();
        let Some(mut offices) = self.offices.take() else {
            return;
        };
        if self.month % 3 == 0 && offices.last_decision != Some(self.month) {
            offices.last_decision = Some(self.month);
            for o in &mut offices.seats {
                if o.holder().is_some() {
                    continue;
                }
                let score = |p: u32| {
                    let a = &self.culture.as_ref().unwrap().agents[p as usize];
                    let campaign = a.last_campaign.is_some_and(|id| {
                        self.events
                            .get(id as usize)
                            .is_some_and(|e| self.month.saturating_sub(e.month) < 12)
                    });
                    let social = if o.selection == Selection::LocalCouncil {
                        0.4 * a.traits[1] + 0.2 * a.traits[4]
                    } else {
                        0.4 * a.traits[4] + 0.2 * a.traits[5]
                    };
                    social
                        + 0.2 * a.knowledge.len().min(12) as f32 / 12.
                        + 0.1 * a.traits[0]
                        + 0.1 * f32::from(campaign)
                };
                let holder = self
                    .office_candidates(o.site, o.controller, o.selection)
                    .into_iter()
                    .max_by(|&a, &b| score(a).total_cmp(&score(b)).then(b.cmp(&a)));
                if let Some(person) = holder {
                    let utility = score(person);
                    let campaign = self.culture.as_ref().unwrap().agents[person as usize]
                        .last_campaign
                        .filter(|id| {
                            self.month.saturating_sub(self.events[*id as usize].month) < 12
                        });
                    self.event("office_appointed",Some(o.site),None,format!("{} selected as {} for four years by {:?}; assessed suitability {utility:.3}; existing administrative staff and treasury retained",self.people[person as usize].name,o.title,o.selection));
                    let e = self.events.last_mut().unwrap();
                    e.subjects
                        .extend([("office".into(), o.id), ("person".into(), person)]);
                    e.causes.push(o.last_event);
                    if let Some(id) = campaign {
                        if !e.causes.contains(&id) {
                            e.causes.push(id);
                        }
                    }
                    o.tenures.push(Tenure {
                        person,
                        began: self.month,
                        due: self.month + 48,
                        ended: None,
                        appointment: e.id,
                    });
                    o.last_event = e.id;
                }
            }
        }
        self.offices = Some(offices);
    }
}
impl Generator {
    pub fn enable_offices(&mut self) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "offices require a completed boundary"
        );
        let terrain = self.snapshot()?;
        let mut h = self
            .civilizations
            .clone()
            .ok_or_else(|| anyhow::anyhow!("found civilizations first"))?;
        ensure!(
            h.governance.is_some() && h.culture.is_some(),
            "enable governance and culture first"
        );
        if h.offices.is_some() {
            return Ok(());
        }
        h.sync_culture();
        h.offices = Some(Offices {
            service: None,
            started: h.month,
            last_decision: None,
            seats: vec![],
        });
        h.office_month();
        h.validate(&terrain)?;
        self.civilizations = Some(h);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> Generator {
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
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        g.civilizations.as_mut().unwrap().sync_culture();
        g
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn named_service_reserves_once_and_bounds_office_capacity() {
        let mut g = world();
        g.enable_offices().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.set_individual_participation(true).unwrap();
        h.set_office_service(true).unwrap();
        h.month += 1;
        h.begin_service_reservations();
        let holder = h.offices.as_ref().unwrap().seats[0].holder().unwrap();
        let opening = h.clone();
        let finance = h.sites[0].economy.finance;
        let external = h.sites[0].economy.external[3];
        assert_eq!(h.office_capacity(0), 0.5);
        h.reserve_office_service().unwrap();
        assert!(h.reserve_office_service().is_err());
        let p = &h.offices.as_ref().unwrap().service.as_ref().unwrap().plans[0];
        assert_eq!(p.holder, holder);
        assert!(p.work.granted > 0.);
        assert!((h.sites[0].economy.external[3] - external - p.work.granted as f32).abs() < 1e-6);
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        h.settle_office_service().unwrap();
        resumed.settle_office_service().unwrap();
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert!(h.office_capacity(0) > 0.5);
        assert_eq!(h.sites[0].economy.finance, finance);
        assert!((h.sites[0].economy.external[3] - external).abs() < 1e-6);
        assert!(h.settle_office_service().is_err());
        assert!(h.set_individual_participation(false).is_err());
        let mut unavailable = opening.clone();
        let available = unavailable
            .participation
            .as_ref()
            .unwrap()
            .available(holder);
        unavailable
            .participation
            .as_mut()
            .unwrap()
            .reserve(
                unavailable.month,
                0,
                crate::participation::Activity::Research,
                &[holder],
                available,
            )
            .unwrap();
        unavailable.reserve_office_service().unwrap();
        unavailable.settle_office_service().unwrap();
        assert_eq!(unavailable.office_capacity(0), 0.5);
        let mut partial = opening.clone();
        let available = partial.participation.as_ref().unwrap().available(holder);
        partial
            .participation
            .as_mut()
            .unwrap()
            .reserve(
                partial.month,
                0,
                crate::participation::Activity::Research,
                &[holder],
                available - 0.05,
            )
            .unwrap();
        partial.reserve_office_service().unwrap();
        partial.settle_office_service().unwrap();
        assert!(
            partial.office_capacity(0) > 0.5 && partial.office_capacity(0) < h.office_capacity(0)
        );
        let mut scarce = opening.clone();
        scarce.sites[0].economy.external[3] = 1e6;
        scarce.reserve_office_service().unwrap();
        scarce.settle_office_service().unwrap();
        assert_eq!(scarce.office_capacity(0), 0.5);
        let mut changed = opening;
        changed.reserve_office_service().unwrap();
        changed.offices.as_mut().unwrap().seats[0]
            .tenures
            .last_mut()
            .unwrap()
            .ended = Some(changed.month);
        changed.settle_office_service().unwrap();
        assert_eq!(changed.office_capacity(0), 0.5);
        let plan = &changed
            .offices
            .as_ref()
            .unwrap()
            .service
            .as_ref()
            .unwrap()
            .plans[0];
        assert_eq!(plan.work.used, 0.);
        assert_eq!(plan.work.granted, plan.work.released);
        plan.work.validate().unwrap();
        // Comparison is observational, including partial, absent and ended-tenure work.
        for mut case in [h.clone(), unavailable, partial, scarce, changed] {
            case.resolution = Some(crate::resolution::ResolutionState {
                compare: true,
                ..Default::default()
            });
            let mut silent = case.clone();
            silent.resolution.as_mut().unwrap().compare = false;
            case.settle_office_resolutions().unwrap();
            silent.settle_office_resolutions().unwrap();
            let receipts = &case.resolution.as_ref().unwrap().receipts;
            assert!(!receipts.is_empty());
            assert!(receipts
                .iter()
                .all(|r| r.metrics.len() == 3
                    && r.metrics.iter().all(|m| m.actual <= m.expected + 1e-6)));
            let before = serde_json::to_value(&case).unwrap();
            assert!(case.settle_office_resolutions().is_err());
            assert_eq!(before, serde_json::to_value(&case).unwrap());
            case.resolution = None;
            silent.resolution = None;
            assert_eq!(
                serde_json::to_value(&case).unwrap(),
                serde_json::to_value(silent).unwrap()
            );
            let service = case.offices.as_mut().unwrap().service.as_mut().unwrap();
            // Legacy archives have no captured opening expectation.
            for plan in &mut service.plans {
                plan.allowance = None;
            }
            case.resolution = Some(Default::default());
            case.settle_office_resolutions().unwrap();
            assert!(case.resolution.as_ref().unwrap().receipts.is_empty());
        }
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn selection_jurisdiction_and_tax_capacity_have_conserved_consequences() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        let people = h.culture.as_ref().unwrap().site_people(h, 0);
        assert!(people.len() >= 2);
        let (loyal, generous) = (people[0], people[1]);
        for a in &mut h.culture.as_mut().unwrap().agents {
            a.traits.fill(0.);
        }
        h.culture.as_mut().unwrap().agents[loyal as usize].traits[4] = 1.;
        h.culture.as_mut().unwrap().agents[loyal as usize].traits[5] = 1.;
        h.culture.as_mut().unwrap().agents[generous as usize].traits[1] = 1.;
        let legacy = serde_json::to_value(&*h).unwrap();
        let path = std::env::temp_dir().join(format!("office-choice-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut local = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        g.enable_offices().unwrap();
        local.set_autonomy(0, 0.75).unwrap();
        local.enable_offices().unwrap();
        assert_eq!(
            g.civilizations
                .as_ref()
                .unwrap()
                .offices
                .as_ref()
                .unwrap()
                .seats[0]
                .holder(),
            Some(loyal)
        );
        assert_eq!(
            local
                .civilizations
                .as_ref()
                .unwrap()
                .offices
                .as_ref()
                .unwrap()
                .seats[0]
                .holder(),
            Some(generous)
        );
        let mut staffed = g.civilizations.as_ref().unwrap().clone();
        for s in &mut staffed.sites {
            s.stocks.stock[3] = 0.;
            s.economy.finance[0] = 1000.;
        }
        staffed.society.as_mut().unwrap().routes.clear();
        let mut occupied = staffed.clone();
        occupied.politics.as_mut().unwrap().controllers[0] = 1;
        occupied.governance.as_mut().unwrap().administrations[0].controller = 1;
        occupied.sync_offices();
        assert_eq!(
            occupied.sites[0].civilization,
            staffed.sites[0].civilization
        );
        assert_eq!(
            occupied.culture.as_ref().unwrap().site_faith,
            staffed.culture.as_ref().unwrap().site_faith
        );
        assert_eq!(occupied.offices.as_ref().unwrap().seats[0].holder(), None);
        assert_eq!(occupied.office_capacity(0), 0.5);
        let full = staffed.office_capacity(0);
        assert!(full > 0.5);
        let cash = |h: &History| {
            h.sites
                .iter()
                .map(|s| s.economy.finance[0] as f64)
                .sum::<f64>()
                + h.society
                    .as_ref()
                    .unwrap()
                    .councils
                    .iter()
                    .map(|c| c.treasury)
                    .sum::<f64>()
        };
        let before = cash(&staffed);
        let rate = staffed.society.as_ref().unwrap().councils[0].tax_rate;
        let autonomy_factor =
            1. - 0.75 * staffed.governance.as_ref().unwrap().administrations[0].autonomy;
        assert_eq!(
            rate,
            occupied.society.as_ref().unwrap().councils[1].tax_rate
        );
        staffed.social_year();
        occupied.social_year();
        assert!(
            (staffed.sites[0].economy.finance[0] - (1000. - 1000. * rate * autonomy_factor * full))
                .abs()
                < 0.001
        );
        assert!(
            (occupied.sites[0].economy.finance[0] - (1000. - 1000. * rate * autonomy_factor * 0.5))
                .abs()
                < 0.001
        );
        assert!((cash(&staffed) - before).abs() < 0.001);
        assert!((cash(&occupied) - before).abs() < 0.001);
        occupied
            .offices
            .as_ref()
            .unwrap()
            .validate(&occupied)
            .unwrap();
        // A constitutional change closes the old tenure immediately, even before another month runs.
        g.set_autonomy(0, 0.75).unwrap();
        let h = g.civilizations.as_ref().unwrap();
        assert_eq!(h.offices.as_ref().unwrap().seats[0].holder(), None);
        assert!(h.events.iter().any(|e| e.kind == "office_jurisdiction"
            && e.causes
                .iter()
                .any(|&id| h.events[id as usize].kind == "autonomy_policy")));
        h.offices.as_ref().unwrap().validate(h).unwrap();
        let mut old = legacy;
        old.as_object_mut().unwrap().remove("offices");
        let old: History = serde_json::from_value(old).unwrap();
        assert!(old.offices.is_none());
        assert_eq!(old.office_capacity(0), 1.);
        old.validate(&g.snapshot().unwrap()).unwrap();
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn offices_keep_terms_and_match_batched_checkpoint_continuation() {
        let mut g = world();
        g.enable_offices().unwrap();
        g.advance_history(5).unwrap();
        let path = std::env::temp_dir().join(format!("office-resume-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        g.advance_history(55).unwrap();
        for _ in 0..55 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
        let h = g.civilizations.as_ref().unwrap();
        assert!(h
            .offices
            .as_ref()
            .unwrap()
            .seats
            .iter()
            .any(|o| o.tenures.len() >= 2));
        assert!(h
            .events
            .iter()
            .any(|e| e.kind == "office_vacated" && e.detail.contains("term completed")));
        assert!(h.economy_residuals().iter().all(|x| x.abs() < 0.001));
        h.validate(&g.snapshot().unwrap()).unwrap();
    }
}
