//! Persistent institutional readiness, supported by finite quarterly work and money.
use serde::{Deserialize, Serialize};
/// Distribution between institutional duties; it does not change execution timing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkPolicy {
    #[default]
    FullUpkeepFirst,
    EssentialFirst,
}
/// Priority within each institutional action class; elections still precede upkeep.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    #[default]
    Stable,
    Rotating,
}
impl Priority {
    pub(crate) fn order(
        self,
        plans: &mut [crate::culture::work_requests::InstitutionWorkPlan],
        month: u32,
        site: u32,
    ) {
        plans.sort_by_key(|p| p.institution);
        if self == Self::Rotating && !plans.is_empty() {
            let offset = (month / 3 + site) as usize % plans.len();
            plans.rotate_left(offset);
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capacity {
    #[serde(default)]
    pub mandate: Option<crate::institution_succession::Mandate>,
    pub readiness: f32,
    pub paid: f64,
    pub work: f64,
    pub observed: u32,
    pub impaired: bool,
    #[serde(default)]
    pub building: Option<MeetingPlace>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingPlace {
    #[serde(default)]
    pub facility: Option<crate::facilities::Facility>,
    pub artifact: u32,
    /// Unfinished construction labor; zero in legacy archives.
    #[serde(default)]
    pub construction_remaining: f32,
    pub condition: f32,
    pub repaired_kg: f64,
    pub repair_paid: f64,
    pub dilapidated: bool,
}
pub const HALL_BRICKS_KG: f32 = 2_000.;
pub const HALL_WORK_MONTHS: f32 = 4.;

impl MeetingPlace {
    pub fn facility(artifact: u32, f: crate::facilities::Facility) -> Self {
        Self {
            construction_remaining: f.remaining(),
            condition: f.condition(),
            facility: Some(f),
            ..Self::new(artifact)
        }
    }
    pub fn hall(artifact: u32) -> Self {
        Self {
            construction_remaining: HALL_WORK_MONTHS - 0.2,
            condition: 0.,
            ..Self::new(artifact)
        }
    }
    pub fn new(artifact: u32) -> Self {
        Self {
            artifact,
            facility: None,
            construction_remaining: 0.,
            condition: 1.,
            repaired_kg: 0.,
            repair_paid: 0.,
            dilapidated: false,
        }
    }
}
impl Capacity {
    pub fn new(month: u32) -> Self {
        Self {
            mandate: None,
            readiness: 0.5,
            paid: 0.,
            work: 0.,
            observed: month,
            impaired: false,
            building: None,
        }
    }
    fn advance(&mut self, support: f32, disruption: f32) {
        self.readiness =
            (self.readiness + 0.12 * support - 0.08 * (1. - support) - 0.12 * disruption)
                .clamp(0., 1.);
    }
}
impl crate::culture::Institution {
    pub fn operational(&self) -> bool {
        self.active
            && self.capacity.as_ref().is_none_or(|c| {
                c.mandate.as_ref().is_none_or(|m| m.holder.is_some())
                    && c.readiness >= 0.25
                    && c.building.as_ref().is_none_or(|b| {
                        (b.facility.is_some() || b.construction_remaining == 0.)
                            && b.condition >= 0.25
                    })
            })
    }
}
fn operating_space(
    kind: &crate::culture::InstitutionKind,
    members: usize,
    working_core: bool,
) -> f32 {
    crate::facilities::demand(
        kind,
        if working_core {
            members.min(2)
        } else {
            members
        },
    )
}

impl crate::civilization::History {
    /// Read-only boundary observations, not forecasts or additional service updates.
    /// Use the same present-adult membership rule as institutional upkeep.
    pub fn institution_state_report(&self) -> Vec<serde_json::Value> {
        let Some(culture) = &self.culture else {
            return vec![];
        };
        let present: Vec<_> = self
            .sites
            .iter()
            .map(|s| culture.site_people(self, s.id))
            .collect();
        culture.institutions.iter().map(|n| {
            let local = present[n.site as usize].iter().filter(|p| n.members.contains(p)).count();
            let facility = n.capacity.as_ref().and_then(|c| c.building.as_ref()).and_then(|b| b.facility.as_ref());
            serde_json::json!({
                "institution": n,
                "operational": n.operational(),
                "living_members": n.members.iter().filter(|id| self.people[**id as usize].died.is_none()).count(),
                "local_adult_members": local,
                "space_demand": crate::facilities::demand(&n.kind, local),
                "operating_space_demand": operating_space(&n.kind, local, culture.institution_working_core),
                "operating_space_coverage": facility.map(|f| (f.usable() / operating_space(&n.kind, local, culture.institution_working_core)).clamp(0., 1.)),
                "usable_space": facility.map(|f| f.usable()),
                "space_coverage": facility.map(|f| (f.usable() / crate::facilities::demand(&n.kind, local)).clamp(0., 1.)),
            })
        }).collect()
    }
}

impl crate::culture::Culture {
    pub(crate) fn upkeep_work_limit(&self, n: &crate::culture::Institution) -> f32 {
        let large = n
            .capacity
            .as_ref()
            .and_then(|c| c.building.as_ref())
            .is_some_and(|b| {
                b.facility.is_some()
                    || self.artifacts[b.artifact as usize]
                        .materials
                        .iter()
                        .any(|&(g, mass)| g == 5 && mass >= HALL_BRICKS_KG)
            });
        if large {
            0.125
        } else {
            0.025
        }
    }
    pub(crate) fn maintain_institutions(&mut self, h: &mut crate::civilization::History) {
        // Membership can outlive local residence. Only present adult representatives
        // provide staffing; empty institutions do not reserve the shared work allowance.
        let present: Vec<_> = h.sites.iter().map(|s| self.site_people(h, s.id)).collect();
        let staffing: Vec<_> = self
            .institutions
            .iter()
            .map(|n| {
                present[n.site as usize]
                    .iter()
                    .filter(|p| n.members.contains(p))
                    .count()
            })
            .collect();
        let mut counts = vec![0u32; h.sites.len()];
        for (n, &local) in self.institutions.iter().zip(&staffing) {
            if n.active && local > 0 && n.capacity.as_ref().is_some_and(|c| c.observed < h.month) {
                counts[n.site as usize] += 1;
            }
        }
        let available = self.labor_budget.clone();
        for (n, &living) in self.institutions.iter_mut().zip(&staffing) {
            if !n.active {
                continue;
            }
            let Some(c) = &mut n.capacity else { continue };
            if c.observed >= h.month {
                continue;
            }
            let i = n.site as usize;
            let large_building = c.building.as_ref().is_some_and(|b| {
                self.artifacts[b.artifact as usize]
                    .materials
                    .iter()
                    .any(|&(g, mass)| g == 5 && mass >= HALL_BRICKS_KG)
            });
            let work_limit =
                if large_building || c.building.as_ref().is_some_and(|b| b.facility.is_some()) {
                    0.125
                } else {
                    0.025
                };
            let assigned = self.work_plans.get(i).and_then(|p| p.upkeep.as_ref());
            let stale = self
                .work_plans
                .get(i)
                .is_some_and(|p| p.upkeep.is_some() && p.month != h.month);
            let work = if h.sites[i].abandoned || living == 0 || stale {
                0.
            } else if let Some(plans) = assigned {
                plans
                    .iter()
                    .find(|p| p.institution == n.id)
                    .map_or(0., |p| {
                        let eligible = p
                            .commitment
                            .and_then(|id| h.participation.as_ref()?.commitments.get(id as usize))
                            .is_some_and(|c| c.people.iter().all(|(id, _)| n.members.contains(id)));
                        if eligible {
                            h.personal_grant_live(p.commitment)
                                .min(p.granted)
                                .min(work_limit)
                        } else {
                            0.
                        }
                    })
            } else {
                (available.get(i).copied().unwrap_or(0.) / counts[i].max(1) as f32).min(work_limit)
            };
            let fee = if work > 0. { n.treasury.min(0.5) } else { 0. };
            let pool = &mut h.sites[i].economy.finance[0];
            let old = *pool;
            let mut next = (old as f64 + fee) as f32;
            if next as f64 - old as f64 > fee {
                next = f32::from_bits(next.to_bits().saturating_sub(1)).max(old);
            }
            let paid = next as f64 - old as f64;
            *pool = next;
            n.treasury -= paid;
            n.expenses += paid;
            c.paid += paid;
            if let Some(plans) = self.work_plans.get_mut(i).and_then(|p| p.upkeep.as_mut()) {
                if let Some(p) = plans.iter_mut().find(|p| p.institution == n.id) {
                    p.used = work;
                }
            } else if let Some(b) = self.labor_budget.get_mut(i) {
                *b = (*b - work).max(0.);
            }
            c.work += work as f64;
            self.labor_spent += work as f64;
            crate::culture::work_requests::record_work(
                &mut self.work_plans,
                i as u32,
                h.month,
                work,
            );
            let mut repair_work = 0.;
            let mut building_support = 1.;
            if let Some(b) = &mut c.building {
                if let Some(f) = &mut b.facility {
                    let a = &self.artifacts[b.artifact as usize];
                    let accessible = !a.destroyed
                        && !a.lost
                        && a.site == Some(n.site)
                        && a.owner == crate::culture::Owner::Institution(n.id);
                    if accessible {
                        let target = crate::facilities::demand(&n.kind, living);
                        let (used, mass, spent, completed, expanded) = f.advance(
                            &mut h.sites[i].economy,
                            h.economy_catalog.as_ref().unwrap(),
                            &mut n.treasury,
                            (work - 0.025).max(0.),
                            target,
                        );
                        repair_work = used;
                        b.repaired_kg += mass;
                        b.repair_paid += spent;
                        n.expenses += spent;
                        self.artifacts[b.artifact as usize].materials = f.embodied();
                        b.condition = f.condition();
                        b.construction_remaining = f.remaining();
                        if completed > 0 || expanded {
                            let kind = if expanded {
                                "meeting_place_expanded"
                            } else {
                                "meeting_place_completed"
                            };
                            let cause = self.artifacts[b.artifact as usize].events.last().copied();
                            h.event(kind,Some(n.site),None,format!("{}: {:.1} usable / {:.1} planned capacity; {:.2} worker-months construction remaining; {} rooms",n.name,f.usable(),f.planned(),f.remaining(),f.rooms.len()));
                            let event = h.events.last_mut().unwrap();
                            event.subjects.push(("institution".into(), n.id));
                            event.subjects.push(("artifact".into(), b.artifact));
                            if let Some(cause) = cause {
                                event.causes.push(cause);
                            }
                            self.artifacts[b.artifact as usize].events.push(event.id);
                        }
                    } else {
                        b.condition = 0.;
                    }
                    building_support = if accessible {
                        (f.usable()
                            / operating_space(&n.kind, living, self.institution_working_core))
                        .clamp(0., 1.)
                    } else {
                        0.
                    };
                } else {
                    let a = &self.artifacts[b.artifact as usize];
                    let accessible = !a.destroyed
                        && !a.lost
                        && a.site == Some(n.site)
                        && a.owner == crate::culture::Owner::Institution(n.id);
                    let embodied = a
                        .materials
                        .iter()
                        .filter(|&&(g, _)| g == 5)
                        .map(|&(_, m)| m)
                        .sum::<f32>();
                    let replacement_work = if large_building {
                        HALL_WORK_MONTHS
                    } else {
                        0.1
                    };
                    if accessible && b.construction_remaining > 0. {
                        let built = work.min(b.construction_remaining);
                        b.construction_remaining = (b.construction_remaining - built).max(0.);
                        repair_work = built;
                        if b.construction_remaining == 0. {
                            b.condition = 1.;
                            h.event("meeting_place_completed", Some(n.site), None, format!("{} completed its meeting room enclosure: {:.0} kg brick, {:.1} worker-months construction", n.name, embodied, HALL_WORK_MONTHS));
                            let event = h.events.last_mut().unwrap();
                            event.subjects.push(("institution".into(), n.id));
                            event.subjects.push(("artifact".into(), b.artifact));
                            if let Some(&cause) = a.events.last() {
                                event.causes.push(cause);
                            }
                            self.artifacts[b.artifact as usize].events.push(event.id);
                        }
                    } else if accessible {
                        b.condition = (b.condition
                            - 0.0025
                            - 0.08 * h.sites[i].economy.soil[3].clamp(0., 1.))
                        .max(0.);
                        let price = h.sites[i].economy.prices[5].max(0.01) as f64;
                        let improvement = (1. - b.condition)
                            .min(0.1)
                            .min(work / replacement_work)
                            .min(h.sites[i].economy.goods[5] / embodied)
                            .min(
                                (crate::facilities::repair_budget(n.treasury)
                                    / (embodied as f64 * price))
                                    as f32,
                            );
                        // Debit exactly the representable stock withdrawal. The same mass of
                        // old brick leaves the foundation as waste; its embodied mass stays fixed.
                        let old = h.sites[i].economy.goods[5];
                        let wanted = (improvement as f64 * embodied as f64)
                            .min(crate::facilities::repair_budget(n.treasury) / price);
                        let mut remaining = (old as f64 - wanted).max(0.) as f32;
                        if old as f64 - remaining as f64 > wanted {
                            remaining = f32::from_bits(remaining.to_bits() + 1).min(old);
                        }
                        let mass = old as f64 - remaining as f64;
                        let fee = mass * price;
                        let pool = &mut h.sites[i].economy.finance[0];
                        let old_cash = *pool;
                        let mut next = (old_cash as f64 + fee) as f32;
                        if next as f64 - old_cash as f64 > fee {
                            next = f32::from_bits(next.to_bits().saturating_sub(1)).max(old_cash);
                        }
                        let payment = next as f64 - old_cash as f64;
                        *pool = next;
                        n.treasury -= payment;
                        n.expenses += payment;
                        b.repair_paid += payment;
                        let e = &mut h.sites[i].economy;
                        e.goods[5] = remaining;
                        e.used[5] += mass as f32;
                        e.reserves[3] += mass as f32;
                        for (k, ratio) in h
                            .economy_catalog
                            .as_ref()
                            .unwrap()
                            .composition(5)
                            .iter()
                            .enumerate()
                        {
                            e.detritus[k] += mass as f32 * ratio;
                        }
                        b.repaired_kg += mass;
                        b.condition = (b.condition + mass as f32 / embodied).min(1.);
                        repair_work = mass as f32 / embodied * replacement_work;
                    } else {
                        b.condition = 0.;
                    }
                    building_support = b.condition;
                    let event =
                        if !b.dilapidated && b.construction_remaining == 0. && b.condition < 0.25 {
                            b.dilapidated = true;
                            Some("meeting_place_dilapidated")
                        } else if b.dilapidated && b.condition >= 0.7 {
                            b.dilapidated = false;
                            Some("meeting_place_repaired")
                        } else {
                            None
                        };
                    if let Some(kind) = event {
                        let cause = self.artifacts[b.artifact as usize].events.last().copied();
                        h.event(kind,Some(n.site),None,format!("{} meeting place condition {:.0}%; cumulative replacement bricks {:.3} kg",n.name,b.condition*100.,b.repaired_kg));
                        let event = h.events.last_mut().unwrap();
                        event.subjects.push(("institution".into(), n.id));
                        event.subjects.push(("artifact".into(), b.artifact));
                        if let Some(cause) = cause {
                            event.causes.push(cause);
                        }
                        self.artifacts[b.artifact as usize].events.push(event.id);
                    }
                }
            }
            let support = ((work - repair_work).max(0.) / 0.025)
                .min(building_support)
                .min((paid / 0.5) as f32)
                .min((living as f32 / 2.).min(1.));
            c.advance(support, h.sites[i].economy.soil[3].clamp(0., 1.));
            c.observed = h.month;
            let event = if !c.impaired && c.readiness < 0.25 {
                c.impaired = true;
                Some("institution_impaired")
            } else if c.impaired && c.readiness >= 0.6 {
                c.impaired = false;
                Some("institution_recovered")
            } else {
                None
            };
            if let Some(kind) = event {
                let cause = h
                    .events
                    .iter()
                    .rev()
                    .find(|e| e.subjects.contains(&("institution".into(), n.id)))
                    .map(|e| e.id);
                h.event(kind,Some(n.site),None,format!("{} operating readiness {:.0}%; quarterly support {:.0}%, disruption {:.0}%; {} locally present adult members",n.name,c.readiness*100.,support*100.,h.sites[i].economy.soil[3]*100.,living));
                let event = h.events.last_mut().unwrap();
                event.subjects.push(("institution".into(), n.id));
                if let Some(id) = cause {
                    event.causes.push(id);
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn funding_and_usable_space_jointly_limit_institution_recovery() {
        use crate::{
            catalog::Catalog,
            config::Config,
            culture::{Artifact, Institution, InstitutionKind, Owner},
            facilities::{Component, Facility, Room},
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let baseline = g.civilizations.as_ref().unwrap();
        let members: Vec<_> = baseline
            .culture
            .as_ref()
            .unwrap()
            .site_people(baseline, 0)
            .into_iter()
            .take(4)
            .collect();
        assert_eq!(members.len(), 4);
        let mut outcomes = vec![];
        for (working_core, capacity) in [(false, 2.), (false, 8.), (true, 2.), (true, 8.)] {
            for funding in [0., 32.] {
                let mut h = baseline.clone();
                let mut c = h.culture.take().unwrap();
                c.institution_working_core = working_core;
                // Declared finite fixture stocks. No production, travel or demography.
                let e = &mut h.sites[0].economy;
                e.goods.fill(0.);
                e.goods[0] = 1000.;
                e.prices.fill(1.);
                e.finance[0] = 1000. - funding as f32;
                e.soil[3] = 0.;
                let facility = Facility {
                    rooms: vec![Room {
                        method: "fixture timber room".into(),
                        capacity,
                        remaining_work: 0.,
                        components: vec![
                            Component {
                                good: 0,
                                kg: 10. * capacity,
                                condition: 1.,
                                wear: 0.006,
                                work: 0.06 * capacity,
                            },
                            Component {
                                good: 0,
                                kg: 4. * capacity,
                                condition: 1.,
                                wear: 0.008,
                                work: 0.02 * capacity,
                            },
                        ],
                    }],
                    ..Default::default()
                };
                let materials = facility.embodied();
                e.goods[0] -= materials[0].1;
                let artifact = c.artifacts.len() as u32;
                c.artifacts.push(Artifact {
                    id: artifact,
                    name: "Finite fixture room".into(),
                    kind: "institutional foundation".into(),
                    creator: None,
                    owner: Owner::Institution(0),
                    claims: vec![],
                    site: Some(0),
                    custodian: None,
                    materials,
                    topic: None,
                    tradition: None,
                    events: vec![],
                    destroyed: false,
                    lost: false,
                });
                c.institutions = vec![Institution {
                    capacity: Some(Capacity {
                        building: Some(MeetingPlace::facility(artifact, facility)),
                        ..Capacity::new(0)
                    }),
                    id: 0,
                    name: "Fixture merchant house".into(),
                    kind: InstitutionKind::Merchant,
                    site: 0,
                    tradition: None,
                    leader: members[0],
                    members: members.clone(),
                    treasury: funding,
                    active: true,
                    founded: 0,
                    knowledge: Default::default(),
                    property: vec![artifact],
                    dues: funding,
                    expenses: 0.,
                }];
                c.work_plans.clear(); // Isolate executed upkeep from reservation policy.
                c.labor_budget = vec![0.; h.sites.len()];
                let goods = h.sites[0].economy.goods[0];
                let waste = h.sites[0].economy.reserves[3];
                for quarter in 1..=12 {
                    h.month = quarter * 3;
                    c.labor_budget[0] = 0.125;
                    let mut resumed_h = h.clone();
                    let mut resumed_c: crate::culture::Culture =
                        serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
                    c.maintain_institutions(&mut h);
                    resumed_c.maintain_institutions(&mut resumed_h);
                    assert_eq!(
                        serde_json::to_value(&c).unwrap(),
                        serde_json::to_value(&resumed_c).unwrap()
                    );
                    assert_eq!(
                        serde_json::to_value(&h).unwrap(),
                        serde_json::to_value(&resumed_h).unwrap()
                    );
                    assert_eq!(
                        h.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury,
                        1000.
                    );
                }
                let n = &c.institutions[0];
                let state = n.capacity.as_ref().unwrap();
                let b = state.building.as_ref().unwrap();
                assert!((state.work - 1.5).abs() < 1e-6);
                assert!(b.facility.as_ref().unwrap().valid(
                    h.economy_catalog.as_ref().unwrap(),
                    &c.artifacts[artifact as usize]
                ));
                assert_eq!(
                    b.facility.as_ref().unwrap().rooms.len(),
                    1,
                    "funding must not buy extra space in this fixture"
                );
                assert!(
                    (goods as f64 - h.sites[0].economy.goods[0] as f64 - b.repaired_kg).abs()
                        < 1e-5
                );
                assert!(
                    (h.sites[0].economy.reserves[3] as f64 - waste as f64 - b.repaired_kg).abs()
                        < 1e-5
                );
                assert_eq!(
                    c.artifacts[artifact as usize].materials,
                    vec![(0, capacity * 14.)]
                );
                eprintln!("capacity={capacity}, funding={funding}, readiness={}, condition={}, fee={}, repairs={}, operational={}", state.readiness, b.condition, state.paid, b.repaired_kg, n.operational());
                if funding == 0. {
                    assert_eq!(state.paid, 0.);
                    assert_eq!(b.repaired_kg, 0.);
                } else {
                    assert_eq!(state.paid, 6.);
                    assert!(b.condition > 0.99);
                }
                outcomes.push(n.operational());
                h.culture = Some(c.clone());
                let observations = h.institution_state_report();
                assert_eq!(observations[0]["local_adult_members"], 4);
                assert_eq!(observations[0]["space_demand"], 8.);
                assert_eq!(
                    observations[0]["operating_space_demand"],
                    if working_core { 4. } else { 8. }
                );
                assert_eq!(observations[0]["operational"], n.operational());
                // Remote membership survives, but cannot inflate local staffing or demand.
                h.sites[0].abandoned = true;
                let absent = h.institution_state_report();
                assert_eq!(absent[0]["local_adult_members"], 0);
                assert_eq!(absent[0]["living_members"], 4);
                assert_eq!(
                    serde_json::to_value(h.culture.as_ref().unwrap()).unwrap(),
                    serde_json::to_value(&c).unwrap()
                );
            }
        }
        assert_eq!(
            outcomes,
            vec![false, false, false, true, false, true, false, true]
        );
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn upkeep_conserves_cash_and_work_and_recovers_after_neglect() {
        use crate::{
            catalog::Catalog,
            config::Config,
            culture::{Institution, InstitutionKind},
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let members = h
            .culture
            .as_ref()
            .unwrap()
            .site_people(h, 0)
            .into_iter()
            .take(2)
            .collect::<Vec<_>>();
        assert_eq!(members.len(), 2);
        h.sites[0].economy.finance[0] -= 25.;
        let mut c = h.culture.take().unwrap();
        c.institutions.push(Institution {
            capacity: Some(Capacity::new(0)),
            id: 0,
            name: "Fixture school".into(),
            kind: InstitutionKind::Scholarly,
            site: 0,
            tradition: None,
            leader: members[0],
            members,
            treasury: 25.,
            active: true,
            founded: 0,
            knowledge: Default::default(),
            property: vec![],
            dues: 25.,
            expenses: 0.,
        });
        c.labor_budget = vec![0.5; h.sites.len()];
        h.month = 3;
        let money = h.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury;
        c.maintain_institutions(h);
        assert_eq!(
            money,
            h.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury
        );
        assert!((c.labor_budget[0] - 0.475).abs() < 1e-6);
        assert!((c.labor_spent - 0.025).abs() < 1e-6);
        let before = serde_json::to_vec(&c.institutions).unwrap();
        c.maintain_institutions(h);
        assert_eq!(before, serde_json::to_vec(&c.institutions).unwrap());
        for _ in 0..8 {
            h.month += 3;
            c.labor_budget.fill(0.);
            c.maintain_institutions(h);
        }
        assert!(!c.institutions[0].operational());
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "institution_impaired")
                .count(),
            1
        );
        for _ in 0..8 {
            h.month += 3;
            c.labor_budget.fill(0.5);
            c.maintain_institutions(h);
        }
        assert!(c.institutions[0].operational());
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "institution_recovered")
                .count(),
            1
        );
        assert_eq!(
            money,
            h.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury
        );
        let mut old = serde_json::to_value(&c.institutions[0]).unwrap();
        old.as_object_mut().unwrap().remove("capacity");
        let old: Institution = serde_json::from_value(old).unwrap();
        assert!(old.capacity.is_none() && old.operational());
        // A real foundation holds two kg of bricks throughout replacement repairs.
        let artifact = c.artifacts.len() as u32;
        c.artifacts.push(crate::culture::Artifact {
            id: artifact,
            name: "Fixture meeting place".into(),
            kind: "institutional foundation".into(),
            creator: None,
            owner: crate::culture::Owner::Institution(0),
            claims: vec![],
            site: Some(0),
            custodian: None,
            materials: vec![(5, 2.)],
            topic: None,
            tradition: None,
            events: vec![],
            destroyed: false,
            lost: false,
        });
        c.institutions[0].property.push(artifact);
        c.institutions[0].capacity.as_mut().unwrap().building = Some(MeetingPlace::new(artifact));
        h.sites[0].economy.goods[5] = 10.;
        h.sites[0].economy.goods[5] -= 2.;
        // Surplus bricks must not turn the next fee into repair spending. Exercise the
        // actual quarterly consumer, including a serialized boundary and eventual depletion.
        {
            let mut run = h.clone();
            let mut culture = c.clone();
            culture.institutions[0].treasury = 1.;
            run.sites[0].economy.finance[0] = 1000.;
            run.sites[0].economy.prices[5] = 2.;
            run.sites[0].economy.soil[3] = 1.;
            let paid_before = culture.institutions[0].capacity.as_ref().unwrap().paid;
            let bricks = run.sites[0].economy.goods[5];
            for quarter in 1..=3 {
                run.month += 3;
                culture.labor_budget.fill(0.5);
                let mut resumed_run = run.clone();
                let mut resumed: crate::culture::Culture =
                    serde_json::from_slice(&serde_json::to_vec(&culture).unwrap()).unwrap();
                culture.maintain_institutions(&mut run);
                resumed.maintain_institutions(&mut resumed_run);
                assert_eq!(
                    serde_json::to_value(&culture).unwrap(),
                    serde_json::to_value(&resumed).unwrap()
                );
                assert_eq!(
                    serde_json::to_value(&run).unwrap(),
                    serde_json::to_value(&resumed_run).unwrap()
                );
                let n = &culture.institutions[0];
                assert_eq!(
                    n.capacity.as_ref().unwrap().paid - paid_before,
                    (quarter as f64 * 0.5).min(1.)
                );
                assert_eq!(n.treasury + run.sites[0].economy.finance[0] as f64, 1001.);
                assert_eq!(run.sites[0].economy.goods[5], bricks);
            }
        }
        // Neglect damages the actual place even when its organizational memory survives.
        h.sites[0].economy.soil[3] = 1.;
        for _ in 0..10 {
            h.month += 3;
            c.labor_budget.fill(0.);
            c.maintain_institutions(h);
        }
        let b = c.institutions[0]
            .capacity
            .as_ref()
            .unwrap()
            .building
            .as_ref()
            .unwrap();
        assert!(b.condition < 0.25 && b.repaired_kg == 0., "{b:?}");
        assert!(!c.institutions[0].operational());
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "meeting_place_dilapidated")
                .count(),
            1
        );
        h.sites[0].economy.soil[3] = 0.;
        // Restock from a declared fixture inventory; check subsequent transfer deltas.
        h.sites[0].economy.goods[5] = 10.;
        h.sites[0].economy.finance[0] -= 100.;
        c.institutions[0].treasury += 100.;
        let initial_goods = h.sites[0].economy.goods[5];
        let initial_used = h.sites[0].economy.used[5];
        let initial_waste = h.sites[0].economy.reserves[3];
        let initial_matter = h.sites[0].economy.detritus;
        let money = h.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury;
        for _ in 0..16 {
            h.month += 3;
            c.labor_budget.fill(0.5);
            c.maintain_institutions(h);
            assert!(c.labor_budget[0] >= 0.);
        }
        let b = c.institutions[0]
            .capacity
            .as_ref()
            .unwrap()
            .building
            .as_ref()
            .unwrap();
        assert!(b.condition > 0.9 && b.repaired_kg > 0.);
        assert!(c.institutions[0].operational());
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "meeting_place_repaired")
                .count(),
            1
        );
        assert_eq!(
            money,
            h.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury
        );
        let mass = initial_goods - h.sites[0].economy.goods[5];
        assert!((mass as f64 - b.repaired_kg).abs() < 1e-6);
        assert!((h.sites[0].economy.used[5] - initial_used - mass).abs() < 1e-5);
        assert!((h.sites[0].economy.reserves[3] - initial_waste - mass).abs() < 1e-5);
        for (k, ratio) in h
            .economy_catalog
            .as_ref()
            .unwrap()
            .composition(5)
            .iter()
            .enumerate()
        {
            assert!(
                (h.sites[0].economy.detritus[k] - initial_matter[k] - mass * ratio).abs() < 1e-5
            );
        }
        assert_eq!(c.artifacts[artifact as usize].materials, vec![(5, 2.)]);
        let snapshot = serde_json::to_vec(&c).unwrap();
        c.maintain_institutions(h);
        assert_eq!(snapshot, serde_json::to_vec(&c).unwrap());
        // An inaccessible foundation cannot be repaired from local stocks.
        c.artifacts[artifact as usize].destroyed = true;
        let goods = h.sites[0].economy.goods[5];
        h.month += 3;
        c.maintain_institutions(h);
        assert!(!c.institutions[0].operational());
        assert_eq!(goods, h.sites[0].economy.goods[5]);
        let mut archived = serde_json::to_value(Capacity::new(0)).unwrap();
        archived.as_object_mut().unwrap().remove("building");
        assert!(serde_json::from_value::<Capacity>(archived)
            .unwrap()
            .building
            .is_none());
        // New construction reserves a real material stock, then needs cumulative work.
        c.artifacts[artifact as usize].destroyed = false;
        c.artifacts[artifact as usize].materials = vec![(5, HALL_BRICKS_KG)];
        c.institutions[0].capacity.as_mut().unwrap().building = Some(MeetingPlace::hall(artifact));
        let work_before = c.labor_spent;
        let goods_before = h.sites[0].economy.goods[5];
        let waste_before = h.sites[0].economy.reserves[3];
        for _ in 0..30 {
            h.month += 3;
            c.labor_budget.fill(0.5);
            c.maintain_institutions(h);
            assert!(!c.institutions[0].operational());
        }
        assert!((c.labor_spent - work_before - 3.75).abs() < 1e-5);
        // Checkpoint mid-construction, then continue both copies.
        let mut resumed: crate::culture::Culture =
            serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
        let mut resumed_history = h.clone();
        h.month += 3;
        resumed_history.month = h.month;
        c.labor_budget.fill(0.5);
        resumed.labor_budget.fill(0.5);
        c.maintain_institutions(h);
        resumed.maintain_institutions(&mut resumed_history);
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        let b = c.institutions[0]
            .capacity
            .as_ref()
            .unwrap()
            .building
            .as_ref()
            .unwrap();
        assert_eq!(b.construction_remaining, 0.);
        assert_eq!(b.condition, 1.);
        assert_eq!(goods_before, h.sites[0].economy.goods[5]);
        assert_eq!(waste_before, h.sites[0].economy.reserves[3]);
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "meeting_place_completed")
                .count(),
            1
        );
        // A one-percent repair requires 20 kg, not 0.02 kg.
        c.institutions[0]
            .capacity
            .as_mut()
            .unwrap()
            .building
            .as_mut()
            .unwrap()
            .condition = 0.9925;
        c.institutions[0].treasury += 10000.; // explicit test funding
        h.sites[0].economy.goods[5] = 100.;
        h.month += 3;
        c.labor_budget.fill(0.5);
        c.maintain_institutions(h);
        let b = c.institutions[0]
            .capacity
            .as_ref()
            .unwrap()
            .building
            .as_ref()
            .unwrap();
        assert!((b.repaired_kg - 20.).abs() < 0.001, "{b:?}");
        assert!((h.sites[0].economy.goods[5] - 80.).abs() < 0.001);
        assert!((h.sites[0].economy.reserves[3] - waste_before - 20.).abs() < 0.001);
        h.culture = Some(c);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn local_staffing_controls_work_sharing_and_recovery() {
        use crate::{
            catalog::Catalog,
            config::Config,
            culture::{Institution, InstitutionKind},
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let mut c = h.culture.take().unwrap();
        let local: Vec<_> = c.site_people(h, 0).into_iter().take(2).collect();
        let remote: Vec<_> = c.site_people(h, 1).into_iter().take(2).collect();
        assert_eq!(local.len(), 2);
        assert_eq!(remote.len(), 2);
        for (id, members) in [local.clone(), remote].into_iter().enumerate() {
            h.sites[0].economy.finance[0] -= 25.;
            c.institutions.push(Institution {
                id: id as u32,
                name: format!("Attendance school {id}"),
                kind: InstitutionKind::Scholarly,
                site: 0,
                tradition: None,
                leader: members[0],
                members,
                treasury: 25.,
                active: true,
                founded: 0,
                knowledge: Default::default(),
                property: vec![],
                dues: 25.,
                expenses: 0.,
                capacity: Some(Capacity::new(0)),
            });
        }
        let cash = |c: &crate::culture::Culture, h: &crate::civilization::History| {
            h.sites[0].economy.finance[0] as f64
                + c.institutions.iter().map(|n| n.treasury).sum::<f64>()
        };
        let money = cash(&c, h);
        h.month = 3;
        c.labor_budget = vec![0.025; h.sites.len()];
        c.maintain_institutions(h);
        assert!((c.institutions[0].capacity.as_ref().unwrap().readiness - 0.62).abs() < 1e-6);
        assert_eq!(c.institutions[1].capacity.as_ref().unwrap().work, 0.);
        assert_eq!(c.institutions[1].treasury, 25.);
        assert!((c.labor_spent - 0.025).abs() < 1e-6);
        assert_eq!(cash(&c, h), money);

        // Attendance fixture: household representatives have completed relocation.
        // Only their location changes; this does not exercise journey accounting.
        for hh in &mut h.society.as_mut().unwrap().households {
            if local.contains(&hh.head) {
                hh.site = 1;
            }
        }
        let paid = c.institutions[0].expenses;
        let spent = c.labor_spent;
        for _ in 0..8 {
            h.month += 3;
            c.labor_budget.fill(0.025);
            c.maintain_institutions(h);
        }
        assert!(!c.institutions[0].operational());
        assert_eq!(c.institutions[0].members, local);
        assert_eq!(c.institutions[0].expenses, paid);
        assert_eq!(c.labor_spent, spent);
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "institution_impaired"
                    && e.subjects.contains(&("institution".into(), 0)))
                .count(),
            1
        );
        for hh in &mut h.society.as_mut().unwrap().households {
            if local.contains(&hh.head) {
                hh.site = 0;
            }
        }
        // A traveler remains associated with its home site but cannot staff it.
        let hh = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.head == local[0])
            .unwrap()
            .id;
        h.society
            .as_mut()
            .unwrap()
            .relocation
            .journeys
            .push(crate::relocation::Journey {
                warning: None,
                roster: None,
                household: hh,
                from: 0,
                to: 1,
                route: 0,
                departed: h.month,
                arrives: h.month + 3,
                cohorts: [0.; 3],
                food: 0.,
                cash: 0.,
                tools: 0.,
                cause: 0,
                blocked: false,
                returning: false,
                seek_help: false,
                report_population: 0.,
                report_food_months: None,
            });
        let prior = c.institutions[0].capacity.as_ref().unwrap().readiness;
        h.month += 3;
        c.labor_budget.fill(0.025);
        c.maintain_institutions(h);
        // One of two required local members supplies half staffing support.
        assert!(
            (c.institutions[0].capacity.as_ref().unwrap().readiness - prior - 0.02).abs() < 1e-6
        );
        h.society.as_mut().unwrap().relocation.journeys.clear();
        let mut resumed: crate::culture::Culture =
            serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
        let mut resumed_h = h.clone();
        for _ in 0..8 {
            h.month += 3;
            resumed_h.month += 3;
            c.labor_budget.fill(0.025);
            resumed.labor_budget.fill(0.025);
            c.maintain_institutions(h);
            resumed.maintain_institutions(&mut resumed_h);
        }
        assert!(c.institutions[0].operational());
        assert_eq!(cash(&c, h), money);
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(resumed_h).unwrap()
        );
        assert_eq!(
            h.events
                .iter()
                .filter(|e| e.kind == "institution_recovered"
                    && e.subjects.contains(&("institution".into(), 0)))
                .count(),
            1
        );
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn named_upkeep_uses_members_once_and_survives_unrelated_actor_absence() {
        use crate::{
            catalog::Catalog,
            config::Config,
            culture::{Institution, InstitutionKind},
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.set_individual_participation(true).unwrap();
        h.month = 3;
        h.begin_service_reservations();
        h.sync_culture();
        let local = h.culture.as_ref().unwrap().site_people(h, 0);
        assert!(local.len() >= 2);
        let member = local[0];
        h.sites[0].economy.finance[0] -= 25.;
        h.culture.as_mut().unwrap().institutions.push(Institution {
            capacity: Some(Capacity::new(0)),
            id: 0,
            name: "Staffed school".into(),
            kind: InstitutionKind::Scholarly,
            site: 0,
            tradition: None,
            leader: member,
            members: vec![member],
            treasury: 25.,
            active: true,
            founded: 0,
            knowledge: Default::default(),
            property: vec![],
            dues: 25.,
            expenses: 0.,
        });
        let opening = h.clone();
        for scenario in 0..4 {
            let blocked = scenario == 1;
            let mut case = opening.clone();
            if blocked {
                let p = case.participation.as_mut().unwrap();
                p.reserve(
                    case.month,
                    0,
                    crate::participation::Activity::Research,
                    &[member],
                    p.available(member),
                )
                .unwrap();
            }
            let plans = case.cultural_work_plans();
            case.reserve_cultural_plans(plans, &vec![0.5; case.sites.len()]);
            let plan = &case.culture.as_ref().unwrap().work_plans[0];
            let u = &plan.upkeep.as_ref().unwrap()[0];
            assert_eq!(u.granted > 0., !blocked);
            if let Some(id) = u.commitment {
                assert_eq!(
                    case.participation.as_ref().unwrap().commitments[id as usize].people[0].0,
                    member
                );
            }
            let mut restored: crate::civilization::History =
                serde_json::from_value(serde_json::to_value(&case).unwrap()).unwrap();
            for run in [&mut case, &mut restored] {
                let mut c = run.culture.take().unwrap();
                if scenario == 3 {
                    c.work_plans[0].month -= 3;
                }
                if scenario == 2 {
                    c.institutions[0].members.clear();
                }
                // Cancellation of the unrelated action bundle must not erase a member's upkeep.
                c.work_plans[0].cancellation = Some("unrelated actor absent".into());
                c.labor_budget[0] = 0.;
                let money = run.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury;
                c.maintain_institutions(run);
                assert_eq!(
                    c.institutions[0].capacity.as_ref().unwrap().work > 0.,
                    scenario == 0
                );
                assert_eq!(
                    money,
                    run.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury
                );
                let after = serde_json::to_value(&c).unwrap();
                c.maintain_institutions(run);
                assert_eq!(after, serde_json::to_value(&c).unwrap());
                run.culture = Some(c);
                run.settle_participation().unwrap();
                let p = &run.culture.as_ref().unwrap().work_plans[0];
                if let Some(id) = p.commitment {
                    assert_eq!(
                        run.participation.as_ref().unwrap().commitments[id as usize].used,
                        0.
                    );
                }
                if let Some(id) = p.upkeep.as_ref().unwrap()[0].commitment {
                    assert_eq!(
                        run.participation.as_ref().unwrap().commitments[id as usize].used > 0.,
                        scenario == 0
                    );
                }
            }
            assert_eq!(
                serde_json::to_value(&case).unwrap(),
                serde_json::to_value(&restored).unwrap()
            );
        }
        let mut legacy = serde_json::to_value(&opening).unwrap();
        legacy["culture"]
            .as_object_mut()
            .unwrap()
            .remove("institution_priority");
        let restored: crate::civilization::History = serde_json::from_value(legacy).unwrap();
        assert_eq!(
            restored.culture.as_ref().unwrap().institution_priority,
            Priority::Stable
        );
        // Matched twelve-quarter allocation trial: four equally staffed institutions.
        for cap in [0.025, 0.05] {
            for priority in [Priority::Stable, Priority::Rotating] {
                let mut run = opening.clone();
                run.culture.as_mut().unwrap().institutions[0].members = local[..2].to_vec();
                for id in 1..4 {
                    let mut n = run.culture.as_ref().unwrap().institutions[0].clone();
                    n.id = id;
                    n.name = format!("Allocation school {id}");
                    run.sites[0].economy.finance[0] -= 25.;
                    run.culture.as_mut().unwrap().institutions.push(n);
                }
                run.culture.as_mut().unwrap().institution_priority = priority;
                let money = run.sites[0].economy.finance[0] as f64
                    + run
                        .culture
                        .as_ref()
                        .unwrap()
                        .institutions
                        .iter()
                        .map(|n| n.treasury)
                        .sum::<f64>();
                for quarter in 1..=12 {
                    run.month = quarter * 3;
                    run.begin_service_reservations();
                    run.sync_culture();
                    let plans = run.cultural_work_plans();
                    assert_eq!(plans[0].institution_priority, Some(priority));
                    run.reserve_cultural_plans(plans, &vec![cap; run.sites.len()]);
                    let mut resumed: crate::civilization::History =
                        serde_json::from_value(serde_json::to_value(&run).unwrap()).unwrap();
                    for case in [&mut run, &mut resumed] {
                        case.release_cultural_work();
                        let mut c = case.culture.take().unwrap();
                        let before = c.labor_spent;
                        c.maintain_institutions(case);
                        c.work_receipt.settle(c.labor_spent - before);
                        case.culture = Some(c);
                        case.settle_participation().unwrap();
                        case.validate_service_work().unwrap();
                    }
                    assert_eq!(
                        serde_json::to_value(&run).unwrap(),
                        serde_json::to_value(&resumed).unwrap()
                    );
                }
                let c = run.culture.as_ref().unwrap();
                let work: Vec<_> = c
                    .institutions
                    .iter()
                    .map(|n| n.capacity.as_ref().unwrap().work)
                    .collect();
                let readiness: Vec<_> = c
                    .institutions
                    .iter()
                    .map(|n| n.capacity.as_ref().unwrap().readiness)
                    .collect();
                let operational = c.institutions.iter().filter(|n| n.operational()).count();
                assert!((work.iter().sum::<f64>() - cap as f64 * 12.).abs() < 1e-6);
                assert_eq!(
                    money,
                    run.sites[0].economy.finance[0] as f64
                        + c.institutions.iter().map(|n| n.treasury).sum::<f64>()
                );
                if priority == Priority::Rotating {
                    assert!(work.iter().all(|w| (w - cap as f64 * 3.).abs() < 1e-6));
                    assert_eq!(operational, if cap == 0.025 { 0 } else { 4 });
                } else {
                    assert_eq!(operational, if cap == 0.025 { 1 } else { 2 });
                }
                println!("institution priority={priority:?} cap={cap:.3} work={work:?} readiness={readiness:?} operational={operational}");
            }
        }
    }

    #[test]
    fn priorities_are_reproducible_and_independent_of_input_order() {
        use crate::culture::work_requests::InstitutionWorkPlan;
        let original: Vec<_> = (0..4)
            .map(|institution| InstitutionWorkPlan {
                institution,
                members: vec![0],
                requested: 0.025,
                minimum: 0.,
                commitment: None,
                granted: 0.,
                used: 0.,
            })
            .collect();
        for priority in [Priority::Stable, Priority::Rotating] {
            let restored: Priority =
                serde_json::from_value(serde_json::to_value(priority).unwrap()).unwrap();
            let mut first = [0; 4];
            for quarter in 1..=12 {
                let mut a = original.clone();
                let mut b = original.clone();
                b.reverse();
                priority.order(&mut a, quarter * 3, 0);
                restored.order(&mut b, quarter * 3, 0);
                assert_eq!(
                    serde_json::to_value(&a).unwrap(),
                    serde_json::to_value(&b).unwrap()
                );
                first[a[0].institution as usize] += 1;
            }
            assert_eq!(
                first,
                if priority == Priority::Stable {
                    [12, 0, 0, 0]
                } else {
                    [3, 3, 3, 3]
                }
            );
        }
    }

    #[test]
    fn neglect_funding_and_damage_have_persistent_effects() {
        let mut c = Capacity::new(0);
        for _ in 0..4 {
            c.advance(0., 0.);
        }
        assert!(c.readiness < 0.25);
        c.advance(1., 0.);
        assert!(c.readiness < 0.6);
        for _ in 0..8 {
            c.advance(1., 0.);
        }
        assert_eq!(c.readiness, 1.);
        for _ in 0..20 {
            c.advance(0., 1.);
        }
        assert_eq!(c.readiness, 0.);
    }
}
