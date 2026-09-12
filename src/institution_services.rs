//! Dated room-capacity reservations for institutional services.
//!
//! Capacity is abstract usable room units multiplied by a fraction of one month,
//! not square metres or worker time. This ledger cannot grant personnel, funds or
//! materials. Callers must establish those before reserving a service and recheck
//! them before execution. Allocation order is explicit in the request sequence.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Service {
    PetitionHearing {
        faction: u32,
        speaker: u32,
        representative: u32,
    },
    Lesson {
        student: u32,
        teacher: u32,
        topic: u32,
    },
    HeritageStudy {
        artifact: u32,
        author: u32,
    },
}

impl Service {
    pub(crate) fn people(self) -> [Option<u32>; 2] {
        match self {
            Self::Lesson { teacher, .. } => [Some(teacher), None],
            Self::HeritageStudy { author, .. } => [Some(author), None],
            Self::PetitionHearing {
                speaker,
                representative,
                ..
            } => [Some(speaker), Some(representative)],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub service: Service,
    pub occupants: f64,
    pub duration: f64,
    pub requested: f64,
    /// Opening room grant, retained when later work matching reduces `granted`.
    #[serde(default)]
    pub space_granted: Option<f64>,
    pub granted: f64,
    pub used: f64,
    pub settled: bool,
}
impl Receipt {
    pub fn opening_grant(&self) -> f64 {
        self.space_granted.unwrap_or(self.granted)
    }
    pub fn released(&self) -> f64 {
        if self.settled {
            self.granted - self.used
        } else {
            0.
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub month: u32,
    pub site: u32,
    pub institution: u32,
    pub opening_space: f64,
    /// Largest usable working-group space; None retains old continuous-size plans.
    #[serde(default)]
    pub group_space: Option<f64>,
    pub receipts: Vec<Receipt>,
    pub closed: bool,
}
impl Plan {
    pub fn new(month: u32, site: u32, institution: u32, usable_space: f64) -> Self {
        Self {
            month,
            site,
            institution,
            opening_space: if usable_space.is_finite() {
                usable_space.max(0.)
            } else {
                0.
            },
            group_space: None,
            receipts: Vec::new(),
            closed: false,
        }
    }

    /// One working group must fit at once, as well as within monthly room time.
    /// Requests are indivisible. An unfunded request leaves capacity for others.
    pub fn reserve(&mut self, service: Service, occupants: f64, duration: f64) -> Option<usize> {
        if self.closed
            || self.receipts.iter().any(|r| r.settled)
            || !occupants.is_finite()
            || occupants <= 0.
            || !duration.is_finite()
            || duration <= 0.
            || duration > 1.
            || self.receipts.iter().any(|r| r.service == service)
        {
            return None;
        }
        let requested = occupants * duration;
        if !requested.is_finite() {
            return None;
        }
        let reserved: f64 = self.receipts.iter().map(Receipt::opening_grant).sum();
        let granted = if occupants <= self.group_space.unwrap_or(self.opening_space)
            && requested <= (self.opening_space - reserved).max(0.)
        {
            requested
        } else {
            0.
        };
        let id = self.receipts.len();
        self.receipts.push(Receipt {
            service,
            occupants,
            duration,
            requested,
            space_granted: Some(granted),
            granted,
            used: 0.,
            settled: false,
        });
        Some(id)
    }

    /// Consume a grant only at its captured boundary, after live eligibility checks.
    /// Lost space can invalidate later execution; newly built space cannot enlarge
    /// opening grants. Failure releases this grant for reporting, never reallocation.
    pub fn settle(
        &mut self,
        boundary: (u32, u32, u32),
        id: usize,
        live_space: f64,
        eligible: bool,
    ) -> bool {
        self.settle_with_group_space(boundary, id, live_space, live_space, eligible)
    }

    pub fn settle_with_group_space(
        &mut self,
        boundary: (u32, u32, u32),
        id: usize,
        live_space: f64,
        live_group_space: f64,
        eligible: bool,
    ) -> bool {
        if self.closed || boundary != (self.month, self.site, self.institution) {
            return false;
        }
        let used: f64 = self.receipts.iter().map(|r| r.used).sum();
        let Some(r) = self.receipts.get_mut(id).filter(|r| !r.settled) else {
            return false;
        };
        r.settled = true;
        if eligible
            && live_space.is_finite()
            && r.granted > 0.
            && live_group_space.is_finite()
            && r.occupants <= live_group_space
            && r.granted <= (live_space.min(self.opening_space) - used).max(0.)
        {
            r.used = r.granted;
            true
        } else {
            false
        }
    }

    /// Structural checks for persisted receipts. Historical opening space is not
    /// compared with today's building: damage and repairs legitimately change it.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.opening_space.is_finite()
                && self.opening_space >= 0.
                && self
                    .group_space
                    .is_none_or(|space| space.is_finite() && space >= 0.),
            "invalid service opening space"
        );
        for (i, r) in self.receipts.iter().enumerate() {
            anyhow::ensure!(
                [
                    r.occupants,
                    r.duration,
                    r.requested,
                    r.opening_grant(),
                    r.granted,
                    r.used
                ]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.)
                    && r.occupants > 0.
                    && r.duration > 0.
                    && r.duration <= 1.
                    && r.requested == r.occupants * r.duration
                    && (r.opening_grant() == 0. || r.opening_grant() == r.requested)
                    && r.granted <= r.opening_grant()
                    && (r.granted == 0. || r.granted == r.requested)
                    && (r.opening_grant() == 0.
                        || r.occupants <= self.group_space.unwrap_or(self.opening_space))
                    && (r.used == 0. || r.used == r.granted)
                    && (r.used == 0. || r.settled)
                    && (!self.closed || r.settled)
                    && !self.receipts[..i]
                        .iter()
                        .any(|other| other.service == r.service),
                "invalid institutional service receipt"
            );
        }
        anyhow::ensure!(
            self.receipts
                .iter()
                .map(Receipt::opening_grant)
                .sum::<f64>()
                <= self.opening_space + 1e-12,
            "institutional services exceed opening space"
        );
        Ok(())
    }

    pub fn close(&mut self, month: u32) {
        if month == self.month {
            for r in &mut self.receipts {
                r.settled = true;
            }
            self.closed = true;
        }
    }
}

/// Available room-months and the largest usable group's nominal capacity.
/// Wear reduces time availability; it does not shrink a two-person room to an
/// unusable 1.99-person group. Rooms below 25% condition provide neither.
/// Legacy meeting places have two abstract units without adding material stocks.
pub(crate) fn space(c: &crate::culture::Culture, institution: u32, site: u32) -> (f64, f64) {
    let Some(n) = c
        .institutions
        .get(institution as usize)
        .filter(|n| n.site == site && n.operational())
    else {
        return (0., 0.);
    };
    let Some(b) = n.capacity.as_ref().and_then(|c| c.building.as_ref()) else {
        return (0., 0.);
    };
    if !c
        .artifacts
        .get(b.artifact as usize)
        .is_some_and(|a| a.site == Some(site) && !a.lost && !a.destroyed)
    {
        return (0., 0.);
    }
    b.facility.as_ref().map_or_else(
        || {
            if b.construction_remaining == 0. {
                (2. * b.condition as f64, 2.)
            } else {
                (0., 0.)
            }
        },
        |f| {
            f.rooms
                .iter()
                .filter(|r| {
                    r.remaining_work == 0. && r.capacity > 0. && r.usable() >= 0.25 * r.capacity
                })
                .fold((0., 0_f64), |(time, group), r| {
                    (time + r.usable() as f64, group.max(r.capacity as f64))
                })
        },
    )
}

/// Actual opening-phase occupancy, attached to a committed relief mission.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpeningUse {
    pub building: u32,
    pub available: f64,
    pub group: f64,
    pub used: f64,
}
impl OpeningUse {
    pub(crate) fn valid(&self, objects: usize) -> bool {
        (self.building as usize) < objects
            && self.available.is_finite()
            && self.group.is_finite()
            && self.group >= 1.
            && self.used == 0.1
            && self.used <= self.available
    }
}
/// Opening dispatches have already consumed this month's room-time before Reserve.
pub(crate) fn remaining_space(
    c: &crate::culture::Culture,
    institution: u32,
    site: u32,
    month: u32,
) -> (f64, f64) {
    let (total, group) = space(c, institution, site);
    let used: f64 = c
        .religious_relief
        .missions
        .iter()
        .filter(|m| m.institution == institution && m.dispatched == month)
        .filter_map(|m| m.room.as_ref())
        .map(|r| r.used)
        .sum();
    ((total - used).max(0.), group)
}
pub(crate) fn opening_dispatch(
    c: &crate::culture::Culture,
    institution: u32,
    site: u32,
    month: u32,
) -> Option<OpeningUse> {
    let (available, group) = remaining_space(c, institution, site, month);
    let building = c
        .institutions
        .get(institution as usize)?
        .capacity
        .as_ref()?
        .building
        .as_ref()?
        .artifact;
    (available >= 0.1 && group >= 1.).then_some(OpeningUse {
        building,
        available,
        group,
        used: 0.1,
    })
}

impl crate::culture::Culture {
    pub(crate) fn plan_service_space(
        &self,
        h: &crate::civilization::History,
        site: u32,
        actor: Option<u32>,
        lesson: Option<(u32, u32, u32)>,
        actions: &[(String, f32)],
        hearing: Option<&crate::civic_petitions::Hearing>,
    ) -> Vec<Plan> {
        let mut requests = Vec::new();
        let present = self.site_people(h, site);
        if actions.iter().any(|(a, _)| a == "heritage study") {
            if let Some(n) = self.institutions.iter().find(|n| {
                n.site == site
                    && n.operational()
                    && matches!(
                        n.kind,
                        crate::culture::InstitutionKind::Scholarly
                            | crate::culture::InstitutionKind::Religious
                    )
                    && present.contains(&n.leader)
            }) {
                if let Some(x) = &h.expeditions {
                    for find in x
                        .voyages
                        .iter()
                        .filter_map(|v| v.heritage.as_ref()?.find.as_ref())
                    {
                        if find.studies.len() >= 3
                            || find.studies.last().is_some_and(|s| h.month < s.month + 60)
                        {
                            continue;
                        }
                        if let Some(artifact) = find.artifact.filter(|&id| {
                            self.artifacts
                                .get(id as usize)
                                .is_some_and(|a| a.site == Some(site) && !a.lost && !a.destroyed)
                        }) {
                            requests.push((
                                n.id,
                                Service::HeritageStudy {
                                    artifact,
                                    author: n.leader,
                                },
                                1.,
                            ));
                        }
                    }
                }
            }
        }
        if actions.iter().any(|(a, _)| a == "study") {
            if let (Some(student), Some((topic, teacher, institution))) = (actor, lesson) {
                if self.readable_lesson(site, student).is_none() {
                    requests.push((
                        institution,
                        Service::Lesson {
                            student,
                            teacher,
                            topic,
                        },
                        2.,
                    ));
                }
            }
        }
        // Within an institution: heritage, then hearings, then lessons, as at execution.
        if let Some(hearing) = hearing {
            let position = requests
                .iter()
                .position(|(_, service, _)| matches!(service, Service::Lesson { .. }))
                .unwrap_or(requests.len());
            requests.insert(
                position,
                (
                    hearing.institution,
                    hearing.service(),
                    if hearing.speaker == hearing.representative {
                        1.
                    } else {
                        2.
                    },
                ),
            );
        }
        let mut plans: Vec<Plan> = Vec::new();
        for (institution, service, occupants) in requests {
            let index = plans
                .iter()
                .position(|p| p.institution == institution)
                .unwrap_or_else(|| {
                    let (time, group) = remaining_space(self, institution, site, h.month);
                    let mut plan = Plan::new(h.month, site, institution, time);
                    plan.group_space = Some(group);
                    plans.push(plan);
                    plans.len() - 1
                });
            plans[index].reserve(service, occupants, 0.1);
        }
        plans
    }

    pub(crate) fn consume_service_space(
        &mut self,
        month: u32,
        site: u32,
        institution: u32,
        service: Service,
    ) -> bool {
        let (live, group) = remaining_space(self, institution, site, month);
        let Some(plans) = self
            .work_plans
            .get_mut(site as usize)
            .and_then(|p| p.services.as_mut())
        else {
            return true;
        };
        let Some(plan) = plans.iter_mut().find(|p| p.institution == institution) else {
            return false;
        };
        let Some(id) = plan.receipts.iter().position(|r| r.service == service) else {
            return false;
        };
        let group = if plan.group_space.is_some() {
            group
        } else {
            live
        };
        plan.settle_with_group_space((month, site, institution), id, live, group, true)
    }
}

/// Filter only room-dependent demand, before the site's 0.5 work ceiling.
/// Other cultural actions keep their existing eligibility rules.
pub(crate) fn feasible_work(
    actions: &[(String, f32)],
    plans: &[Plan],
    lesson_needs_room: bool,
) -> f32 {
    let mut studies = plans
        .iter()
        .flat_map(|p| &p.receipts)
        .filter(|r| r.granted > 0. && matches!(r.service, Service::HeritageStudy { .. }))
        .count();
    let lesson = plans
        .iter()
        .flat_map(|p| &p.receipts)
        .any(|r| r.granted > 0. && matches!(r.service, Service::Lesson { .. }));
    actions
        .iter()
        .filter_map(|(action, work)| match action.as_str() {
            "heritage study" => {
                if studies > 0 {
                    studies -= 1;
                    Some(*work)
                } else {
                    None
                }
            }
            "study" if lesson_needs_room && !lesson => None,
            "petition hearing"
                if !plans.iter().flat_map(|p| &p.receipts).any(|r| {
                    r.granted > 0. && matches!(r.service, Service::PetitionHearing { .. })
                }) =>
            {
                None
            }
            _ => Some(*work),
        })
        .sum::<f32>()
}

/// Validate references and work backing as well as the independent room ledger.
/// Deceased authors and destroyed objects remain legitimate historical references.
pub(crate) fn validate_work_plan(
    p: &crate::culture::work_requests::WorkPlan,
    c: &crate::culture::Culture,
    people: usize,
) -> anyhow::Result<()> {
    let Some(plans) = &p.services else {
        return Ok(());
    };
    anyhow::ensure!(
        plans.len() <= c.institutions.len(),
        "unbounded service plans"
    );
    let mut seen = Vec::new();
    let mut granted = 0.;
    let mut used = 0.;
    for (i, plan) in plans.iter().enumerate() {
        plan.validate()?;
        anyhow::ensure!(
            plan.month == p.month
                && plan.site == p.site
                && c.institutions
                    .get(plan.institution as usize)
                    .is_some_and(|n| n.site == p.site)
                && !plans[..i]
                    .iter()
                    .any(|other| other.institution == plan.institution)
                && plan.receipts.len() <= c.artifacts.len().saturating_add(1),
            "invalid institutional service boundary"
        );
        for r in &plan.receipts {
            anyhow::ensure!(
                !seen.iter().any(|other| match (*other, r.service) {
                    (
                        Service::HeritageStudy { artifact: a, .. },
                        Service::HeritageStudy { artifact: b, .. },
                    ) => a == b,
                    _ => *other == r.service,
                }),
                "duplicate institutional service"
            );
            seen.push(r.service);
            let valid = match r.service {
                Service::PetitionHearing {
                    faction,
                    speaker,
                    representative,
                } => {
                    (speaker as usize) < people
                        && (representative as usize) < people
                        && p.hearing.as_ref().is_some_and(|q| {
                            q.site == p.site
                                && q.institution == plan.institution
                                && q.faction == faction
                                && q.speaker == speaker
                                && q.representative == representative
                        })
                        && p.actions.iter().any(|(a, _)| a == "petition hearing")
                        && r.occupants == if speaker == representative { 1. } else { 2. }
                }
                Service::Lesson {
                    student,
                    teacher,
                    topic,
                } => {
                    (student as usize) < people
                        && (teacher as usize) < people
                        && student != teacher
                        && (topic as usize) < crate::culture::TOPICS.len()
                        && p.actor == Some(student)
                        && p.institution_lesson == Some((topic, teacher, plan.institution))
                        && p.actions.iter().any(|(a, _)| a == "study")
                        && r.occupants == 2.
                }
                Service::HeritageStudy { artifact, author } => {
                    (artifact as usize) < c.artifacts.len()
                        && (author as usize) < people
                        && p.actions.iter().any(|(a, _)| a == "heritage study")
                        && r.occupants == 1.
                }
            };
            anyhow::ensure!(
                valid && r.duration == 0.1,
                "invalid institutional service source or duration"
            );
            if r.granted > 0. {
                granted += 0.1;
            }
            if r.used > 0. {
                used += 0.1;
            }
        }
    }
    let duties: f32 = p.institution_work().map(|u| u.granted).sum();
    let duty_used: f32 = p.institution_work().map(|u| u.used).sum();
    anyhow::ensure!(
        granted + duties as f64 <= p.granted as f64 + 1e-5
            && used + duty_used as f64 <= p.completed as f64 + 1e-5,
        "institutional services lack cultural work backing"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn lesson(student: u32) -> Service {
        Service::Lesson {
            student,
            teacher: 9,
            topic: 2,
        }
    }
    #[test]
    fn groups_compete_for_finite_room_time_and_physical_space() {
        let mut small = Plan::new(12, 1, 3, 2.);
        let a = small.reserve(lesson(1), 2., 0.75).unwrap();
        let b = small.reserve(lesson(2), 2., 0.5).unwrap();
        assert_eq!(small.receipts[a].granted, 1.5);
        assert_eq!(small.receipts[b].granted, 0.);
        // A short meeting still cannot fit an oversized group.
        let c = small.reserve(lesson(3), 3., 0.01).unwrap();
        assert_eq!(small.receipts[c].granted, 0.);
        let d = small
            .reserve(
                Service::HeritageStudy {
                    artifact: 5,
                    author: 4,
                },
                1.,
                0.5,
            )
            .unwrap();
        assert_eq!(small.receipts[d].granted, 0.5);
        assert!(small.settle((12, 1, 3), a, 2., true));
        assert!(small.settle((12, 1, 3), d, 2., true));
        assert_eq!(small.receipts.iter().map(|r| r.used).sum::<f64>(), 2.);
        let mut large = Plan::new(12, 1, 3, 4.);
        large.reserve(lesson(1), 2., 0.75);
        large.reserve(lesson(2), 2., 0.5);
        assert_eq!(large.receipts.iter().map(|r| r.granted).sum::<f64>(), 2.5);
    }
    #[test]
    fn absence_damage_stale_and_duplicate_execution_do_not_create_capacity() {
        let mut p = Plan::new(12, 1, 3, 2.);
        let a = p.reserve(lesson(1), 2., 0.5).unwrap();
        let b = p.reserve(lesson(2), 2., 0.5).unwrap();
        assert!(p.reserve(lesson(1), 2., 0.5).is_none());
        assert!(!p.settle((13, 1, 3), a, 2., true));
        assert!(!p.settle((12, 2, 3), a, 2., true));
        assert!(!p.settle((12, 1, 4), a, 2., true));
        assert!(!p.receipts[a].settled);
        assert!(!p.settle((12, 1, 3), a, 2., false));
        assert!(!p.settle((12, 1, 3), a, 2., true));
        assert_eq!(p.receipts[a].released(), 1.);
        assert!(p.reserve(lesson(3), 1., 0.5).is_none());
        assert!(!p.settle((12, 1, 3), b, 1., true));
        assert_eq!(p.receipts.iter().map(|r| r.used).sum::<f64>(), 0.);
    }
    #[test]
    fn opening_space_survives_work_denial_and_checkpointing() {
        let mut plan = Plan::new(12, 0, 0, 0.2);
        plan.group_space = Some(2.);
        let id = plan.reserve(lesson(1), 2., 0.1).unwrap();
        plan.receipts[id].granted = 0.; // Personal matching cannot fill the request.
        let late = plan.reserve(lesson(2), 2., 0.1).unwrap();
        assert_eq!(
            plan.receipts[late].granted, 0.,
            "no new opening entitlement after matching"
        );
        plan.close(12);
        plan.validate().unwrap();
        let restored: Plan = serde_json::from_value(serde_json::to_value(&plan).unwrap()).unwrap();
        assert_eq!(restored.receipts[id].opening_grant(), 0.2);
        assert_eq!(restored.receipts[id].granted, 0.);
        let mut legacy = serde_json::to_value(&restored.receipts[id]).unwrap();
        legacy.as_object_mut().unwrap().remove("space_granted");
        let legacy: Receipt = serde_json::from_value(legacy).unwrap();
        assert_eq!(
            legacy.opening_grant(),
            0.,
            "old receipts have no invented opening grant"
        );
        let mut corrupt = plan.clone();
        corrupt.receipts[id].space_granted = Some(-1.);
        assert!(corrupt.validate().is_err());
    }

    #[test]
    fn hearings_and_lessons_share_room_time_before_work_allocation() {
        let hearing = Service::PetitionHearing {
            faction: 0,
            speaker: 4,
            representative: 5,
        };
        let actions = vec![("petition hearing".into(), 0.1), ("study".into(), 0.1)];
        let mut p = Plan::new(12, 0, 0, 0.3);
        p.group_space = Some(2.);
        let first = p.reserve(hearing, 2., 0.1).unwrap();
        let second = p.reserve(lesson(1), 2., 0.1).unwrap();
        assert_eq!(p.receipts[first].granted, 0.2);
        assert_eq!(p.receipts[second].granted, 0.);
        assert_eq!(feasible_work(&actions, &[p.clone()], true), 0.1);
        assert!(p.settle_with_group_space((12, 0, 0), first, 0.3, 2., true));
        assert!(!p.settle_with_group_space((12, 0, 0), first, 0.3, 2., true));
        assert!(!p.settle_with_group_space((12, 0, 0), second, 0.3, 2., true));
        p.validate().unwrap();
        let mut ample = Plan::new(12, 0, 0, 0.4);
        ample.group_space = Some(2.);
        ample.reserve(hearing, 2., 0.1);
        ample.reserve(lesson(1), 2., 0.1);
        assert_eq!(feasible_work(&actions, &[ample], true), 0.2);
    }

    #[test]
    fn room_denials_filter_demand_before_the_work_ceiling() {
        let mut p = Plan::new(12, 0, 0, 0.5);
        p.group_space = Some(2.);
        let mut actions = Vec::new();
        for artifact in 0..4 {
            p.reserve(
                Service::HeritageStudy {
                    artifact,
                    author: 7,
                },
                1.,
                0.1,
            );
            actions.push(("heritage study".into(), 0.1));
        }
        p.reserve(lesson(1), 2., 0.1);
        actions.push(("study".into(), 0.1));
        assert_eq!(feasible_work(&actions, &[p.clone()], true), 0.4);
        assert_eq!(feasible_work(&actions, &[p.clone()], false), 0.5);
        assert_eq!(feasible_work(&actions, &[], true), 0.);
        assert_eq!(feasible_work(&actions, &[], false), 0.1);
        // Other genuine requests can fill the freed allowance; subtracting from
        // an already capped 0.5 total would incorrectly remove this opportunity.
        actions.push(("charity".into(), 0.2));
        assert_eq!(feasible_work(&actions, &[p], true), 0.6);
    }

    #[test]
    fn corrupted_receipts_are_rejected_after_deserialization() {
        let mut original = Plan::new(12, 1, 3, 2.);
        original.reserve(lesson(1), 2., 0.5);
        original.reserve(lesson(2), 2., 0.5);
        original.validate().unwrap();
        let corruptions: Vec<fn(&mut Plan)> = vec![
            |p| p.opening_space = -1.,
            |p| p.group_space = Some(-1.),
            |p| p.group_space = Some(1.),
            |p| p.opening_space = 1.,
            |p| {
                for r in &mut p.receipts {
                    r.duration = 1.;
                    r.requested = 2.;
                    r.granted = 2.;
                }
            },
            |p| p.receipts[0].duration = 2.,
            |p| p.receipts[0].requested = 0.5,
            |p| p.receipts[0].granted = 0.5,
            |p| p.receipts[0].used = 0.5,
            |p| p.receipts[0].used = 1.,
            |p| p.closed = true,
            |p| p.receipts[1].service = p.receipts[0].service,
        ];
        for corrupt in corruptions {
            let mut p = original.clone();
            corrupt(&mut p);
            let restored: Plan = serde_json::from_value(serde_json::to_value(p).unwrap()).unwrap();
            assert!(restored.validate().is_err());
        }
        original.receipts[0].granted = 0.; // Unfunded conditional request is valid.
        original.close(12);
        original.validate().unwrap();
        original.opening_space = f64::INFINITY;
        assert!(original.validate().is_err());
    }

    #[test]
    fn continuation_and_close_preserve_receipts() {
        let mut p = Plan::new(12, 1, 3, 2.);
        p.reserve(lesson(1), 2., 0.5);
        p.reserve(lesson(2), 2., 0.5);
        assert!(p.settle((12, 1, 3), 0, 2., true));
        let mut restored: Plan = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        for plan in [&mut p, &mut restored] {
            assert!(plan.settle((12, 1, 3), 1, 2., true));
            plan.close(12);
            assert!(plan.reserve(lesson(4), 1., 0.1).is_none());
        }
        assert_eq!(p, restored);
        let mut invalid = Plan::new(0, 0, 0, f64::NAN);
        assert_eq!(invalid.opening_space, 0.);
        assert!(invalid.reserve(lesson(1), f64::INFINITY, 1.).is_none());
        assert!(invalid.reserve(lesson(1), 1., -1.).is_none());
    }
}
