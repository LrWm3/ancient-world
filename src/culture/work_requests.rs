//! Opening-boundary requests, not guarantees: execution rechecks live people,
//! routes and materials. Nothing is spent or named while forecasting work.
use super::*;
/// A site's bounded bundle: actor and eligible named targets are fixed at Reserve.
/// Material stocks remain live and must pass the action's execution checks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudyExpectation {
    pub lesson: LessonExpectation,
    pub object: Option<u32>,
    pub teacher: Option<u32>,
    pub institution: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstitutionWorkPlan {
    pub institution: u32,
    pub members: Vec<u32>,
    pub requested: f32,
    /// Do not reserve an indivisible task below this amount. Legacy plans default to zero.
    #[serde(default)]
    pub minimum: f32,
    pub commitment: Option<u32>,
    pub granted: f32,
    pub used: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkPlan {
    /// Opening lesson opportunity funnel; absent in legacy plans. No capacity is granted.
    #[serde(default)]
    pub lesson_opportunities: Option<[u32; 6]>,
    #[serde(default)]
    pub institutional_students: Option<bool>,
    /// Uncapped opening demand after room exclusions, before personal matching.
    /// None preserves the unfiltered requests of older plans.
    #[serde(default)]
    pub space_feasible_work: Option<f32>,
    /// None preserves archived plans created before room reservations.
    #[serde(default)]
    pub services: Option<Vec<crate::institution_services::Plan>>,
    #[serde(default)]
    pub hearing: Option<crate::civic_petitions::Hearing>,
    #[serde(default)]
    pub funding: Option<crate::institution_funding::Budget>,
    /// Captured ordering policy; old plans do not invent an allocation history.
    #[serde(default)]
    pub institution_priority: Option<crate::institution_capacity::Priority>,
    #[serde(default)]
    pub institution_work_policy: Option<crate::institution_capacity::WorkPolicy>,
    /// Separate institutional teams; None preserves older/aggregate bundled plans.
    #[serde(default)]
    pub upkeep: Option<Vec<InstitutionWorkPlan>>,
    #[serde(default)]
    pub elections: Option<Vec<InstitutionWorkPlan>>,
    #[serde(default)]
    pub administration: Option<Vec<InstitutionWorkPlan>>,
    #[serde(default)]
    pub study_expectation: Option<StudyExpectation>,
    #[serde(default)]
    pub completed: f32,
    #[serde(default)]
    pub commitment: Option<u32>,
    pub month: u32,
    pub site: u32,
    pub actor: Option<u32>,
    pub successor: Option<(u32, u32, u32)>,
    #[serde(default)]
    pub successor_expectation: Option<LessonExpectation>,
    #[serde(default)]
    pub participants: Option<Vec<u32>>,
    /// Explicit object dependencies; None keeps the legacy site-wide guard.
    #[serde(default)]
    pub object_dependencies: Option<Vec<u32>>,
    #[serde(default)]
    pub institution_lesson: Option<(u32, u32, u32)>,
    pub actions: Vec<(String, f32)>,
    pub identities: serde_json::Value,
    pub cancellation: Option<String>,
    #[serde(default)]
    pub granted: f32,
    #[serde(default)]
    pub cancelled_work: f32,
    #[serde(default)]
    pub changed_identities: Vec<String>,
}
impl InstitutionWorkPlan {
    // Reserve once, then optionally enlarge that same member's assignment in this window.
    // Never move it to a different person or extend a completed/stale commitment.
    fn reserve_up_to(
        &mut self,
        state: &mut crate::participation::Participation,
        month: u32,
        site: u32,
        ceiling: f32,
        available: &mut f32,
    ) {
        let wanted = (ceiling.min(self.requested) - self.granted)
            .max(0.)
            .min(*available);
        if wanted <= 0. {
            return;
        }
        let before = self.granted;
        if let Some(id) = self.commitment {
            let Some(c) = state.commitments.get(id as usize) else {
                return;
            };
            if state.month != Some(month)
                || c.month != month
                || c.site != site
                || c.settled
                || c.used > 0.
                || c.cancellation.is_some()
                || c.activity != crate::participation::Activity::Culture
                || c.people.len() != 1
            {
                return;
            }
            let person = c.people[0].0;
            if !self.members.contains(&person)
                || state
                    .residents
                    .get(&person)
                    .is_none_or(|r| r.presence != crate::participation::Presence::Resident(site))
            {
                return;
            }
            let extra = wanted.min(state.available(person));
            state.residents.get_mut(&person).unwrap().committed += extra;
            let c = &mut state.commitments[id as usize];
            c.people[0].1 += extra;
            c.granted += extra;
            self.granted = c.granted;
        } else {
            let member = self.members.iter().copied().max_by(|a, b| {
                state
                    .available(*a)
                    .total_cmp(&state.available(*b))
                    .then_with(|| b.cmp(a))
            });
            self.commitment = member.and_then(|id| {
                if wanted.min(state.available(id)) < self.minimum {
                    return None;
                }
                state.reserve(
                    month,
                    site,
                    crate::participation::Activity::Culture,
                    &[id],
                    wanted,
                )
            });
            self.granted = self
                .commitment
                .map_or(0., |id| state.commitments[id as usize].granted);
        }
        *available = (*available - (self.granted - before)).max(0.);
    }
}
impl WorkPlan {
    pub(crate) fn raw_work(&self) -> f32 {
        self.actions.iter().map(|(_, w)| *w).sum::<f32>().min(0.5)
    }
    pub(crate) fn feasible_work(&self) -> f32 {
        self.space_feasible_work
            .unwrap_or_else(|| self.raw_work())
            .min(0.5)
    }
    pub(crate) fn room_denied_work(&self) -> f32 {
        self.space_feasible_work.map_or(0., |feasible| {
            (self.actions.iter().map(|(_, w)| *w).sum::<f32>() - feasible).max(0.)
        })
    }
    pub(crate) fn service_people(&self) -> Vec<u32> {
        self.services.as_ref().map_or_else(
            || self.institution_lesson.map(|l| l.1).into_iter().collect(),
            |plans| {
                plans
                    .iter()
                    .flat_map(|p| &p.receipts)
                    .filter(|r| r.granted > 0.)
                    .flat_map(|r| r.service.people().into_iter().flatten())
                    .collect()
            },
        )
    }

    pub(crate) fn reserve_institution_work(
        &mut self,
        state: &mut crate::participation::Participation,
        month: u32,
        site: u32,
        available: &mut f32,
    ) -> f32 {
        if self.month != month || self.site != site {
            return 0.;
        }
        let essential = self.institution_work_policy
            == Some(crate::institution_capacity::WorkPolicy::EssentialFirst)
            && self.administration.is_some();
        let elections = self.elections.as_ref().map_or(0, Vec::len);
        let upkeep_end = elections + self.upkeep.as_ref().map_or(0, Vec::len);
        for (index, p) in self.institution_work_mut().enumerate() {
            let ceiling = if essential && (elections..upkeep_end).contains(&index) {
                0.025
            } else {
                p.requested
            };
            p.reserve_up_to(state, month, site, ceiling, available);
        }
        if essential {
            for p in self.upkeep.iter_mut().flatten() {
                p.reserve_up_to(state, month, site, p.requested, available);
            }
        }
        self.institution_work().map(|p| p.granted).sum()
    }

    pub(crate) fn institution_work(&self) -> impl Iterator<Item = &InstitutionWorkPlan> {
        self.elections
            .iter()
            .flatten()
            .chain(self.upkeep.iter().flatten())
            .chain(self.administration.iter().flatten())
    }
    pub(crate) fn institution_work_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut InstitutionWorkPlan> {
        self.elections
            .iter_mut()
            .flatten()
            .chain(self.upkeep.iter_mut().flatten())
            .chain(self.administration.iter_mut().flatten())
    }
}
impl Culture {
    pub(crate) fn readable_lesson(
        &self,
        site: u32,
        actor: u32,
    ) -> Option<(u32, Option<u32>, Option<u64>)> {
        self.artifacts.iter().find_map(|a| {
            a.topic
                .filter(|topic| {
                    !a.destroyed
                        && !a.lost
                        && a.site == Some(site)
                        && !self.agents[actor as usize].knowledge.contains(topic)
                })
                .map(|topic| (topic, Some(a.id), a.events.last().copied()))
        })
    }

    fn work_identities(
        &self,
        h: &History,
        site: u32,
        participants: Option<&[u32]>,
        objects: Option<&[u32]>,
    ) -> serde_json::Value {
        let people: Vec<_> = self
            .site_people(h, site)
            .into_iter()
            .filter(|p| participants.is_none_or(|ids| ids.contains(p)))
            .collect();
        serde_json::json!({
            "people": people.iter().map(|&p| (p, self.agents.get(p as usize).map(|a| (&a.knowledge, &a.studies, a.instruction_work)))).collect::<Vec<_>>(),
            "buildings": self.institutions.iter().filter_map(|n| {
                let b = n.capacity.as_ref()?.building.as_ref()?;
                objects.is_some_and(|ids| ids.contains(&b.artifact)).then_some((n.id, b.artifact))
            }).collect::<Vec<_>>(),
            "recoveries": self.local_recoveries.iter().filter(|r| r.site == site).collect::<Vec<_>>(),
            "faith": people.iter().map(|&p| self.resident_tradition(h, site, p)).collect::<Vec<_>>(),
            "objects": self.artifacts.iter().filter(|a| objects.map_or_else(|| a.site.is_some_and(|s| h.sites[s as usize].cell == h.sites[site as usize].cell), |ids| ids.contains(&a.id))).map(|a| (a.id, a.site, a.custodian, &a.owner, a.topic, a.lost, a.destroyed)).collect::<Vec<_>>(),
            "institutions": self.institutions.iter().filter(|n| n.site == site).map(|n| (n.id, n.leader, &n.members, &n.knowledge, n.active)).collect::<Vec<_>>(),
        })
    }
    fn work_actor(&self, h: &History, site: u32, people: &[u32]) -> Option<u32> {
        if people.is_empty() || people.iter().any(|&p| p as usize >= self.agents.len()) {
            return None;
        }
        let offset = (h.month / 3 + site) as usize % people.len();
        if self.institutional_students && (h.month / 3 + site).is_multiple_of(2) {
            // Start at the ordinary rotation, retaining alternating general-purpose turns.
            // This is opportunity selection, not a grant or a promise of instruction.
            if let Some(student) = people
                .iter()
                .cycle()
                .skip(offset)
                .take(people.len())
                .copied()
                .find(|&p| {
                    self.resident_tradition(h, site, p).is_some()
                        && self.institutional_lesson(h, site, p).is_some()
                })
            {
                return Some(student);
            }
        }
        Some(people[offset])
    }
    pub(super) fn plan_work(&self, h: &History, site: u32) -> WorkPlan {
        let people = self.site_people(h, site);
        let actor = self.work_actor(h, site, &people);
        let successor = actor.and_then(|a| self.succession_lesson(h, site, a));
        let institution_lesson = actor.and_then(|a| self.institutional_lesson(h, site, a));
        let actions: Vec<(String, f32)> = self
            .work_requests(h, site)
            .into_iter()
            .map(|(a, w)| (a.into(), w))
            .collect();
        let hearing = actions
            .iter()
            .any(|(a, _)| a == "petition hearing")
            .then(|| crate::civic_petitions::forecast(h, self, site))
            .flatten();
        let services = self.plan_service_space(
            h,
            site,
            actor,
            institution_lesson,
            &actions,
            hearing.as_ref(),
        );
        let participants = self.focused_work_identities.then(|| {
            let mut ids: Vec<_> = actor
                .into_iter()
                .chain(successor.map(|s| s.0))
                .chain(
                    services
                        .iter()
                        .flat_map(|p| &p.receipts)
                        .filter(|r| r.granted > 0.)
                        .flat_map(|r| r.service.people().into_iter().flatten()),
                )
                .collect();
            ids.sort_unstable();
            ids.dedup();
            ids
        });
        let successor_expectation = successor
            .filter(|_| actions.iter().any(|(a, _)| a == "teach successor"))
            .map(|(student, topic, _)| {
                self.agents[student as usize].lesson_expectation(
                    topic,
                    self.agents[actor.unwrap() as usize].instruction_support(),
                )
            });
        let study_expectation = actor
            .filter(|_| actions.iter().any(|(a, _)| a == "study"))
            .and_then(|actor| {
                if let Some((topic, object, _)) = self.readable_lesson(site, actor) {
                    Some(StudyExpectation {
                        lesson: self.agents[actor as usize].lesson_expectation(topic, 0.),
                        object,
                        teacher: None,
                        institution: None,
                    })
                } else {
                    institution_lesson.map(|(topic, teacher, institution)| StudyExpectation {
                        lesson: self.agents[actor as usize].lesson_expectation(
                            topic,
                            self.agents[teacher as usize].instruction_support(),
                        ),
                        object: None,
                        teacher: Some(teacher),
                        institution: Some(institution),
                    })
                }
            });
        // Only narrow guards for actions whose object dependencies are explicit.
        // Unknown/dynamic-target actions retain the full local object snapshot.
        let object_dependencies = self
            .focused_work_identities
            .then(|| {
                if actions.iter().any(|(a, _)| {
                    !matches!(
                        a.as_str(),
                        "study"
                            | "heritage study"
                            | "teach successor"
                            | "charity"
                            | "petition hearing"
                    )
                }) {
                    return None;
                }
                let mut ids: Vec<_> = study_expectation
                    .as_ref()
                    .and_then(|s| s.object)
                    .into_iter()
                    .collect();
                for plan in &services {
                    if let Some(b) = self.institutions[plan.institution as usize]
                        .capacity
                        .as_ref()
                        .and_then(|c| c.building.as_ref())
                    {
                        ids.push(b.artifact);
                    }
                    for receipt in &plan.receipts {
                        if let crate::institution_services::Service::HeritageStudy {
                            artifact,
                            ..
                        } = receipt.service
                        {
                            ids.push(artifact);
                        }
                    }
                }
                ids.sort_unstable();
                ids.dedup();
                Some(ids)
            })
            .flatten();
        let identities = self.work_identities(
            h,
            site,
            participants.as_deref(),
            object_dependencies.as_deref(),
        );
        let mut upkeep: Option<Vec<InstitutionWorkPlan>> = h.participation.as_ref().map(|_| {
            self.institutions
                .iter()
                .filter(|n| n.site == site && n.active)
                .filter_map(|n| {
                    let capacity = n.capacity.as_ref()?;
                    let members: Vec<_> = people
                        .iter()
                        .copied()
                        .filter(|p| n.members.contains(p))
                        .collect();
                    (h.month.is_multiple_of(3)
                        && !h.sites[site as usize].abandoned
                        && capacity.observed < h.month
                        && !members.is_empty())
                    .then(|| InstitutionWorkPlan {
                        institution: n.id,
                        members,
                        requested: self.upkeep_work_limit(n),
                        minimum: 0.,
                        commitment: None,
                        granted: 0.,
                        used: 0.,
                    })
                })
                .collect()
        });
        let mut elections: Option<Vec<InstitutionWorkPlan>> = h.participation.as_ref().map(|_| {
            self.institutions
                .iter()
                .filter(|n| n.site == site && n.active)
                .filter_map(|n| {
                    let m = n.capacity.as_ref()?.mandate.as_ref()?;
                    let members = self.institution_candidates(h, n.id);
                    (h.month.is_multiple_of(3)
                        && !h.sites[site as usize].abandoned
                        && m.holder.is_none()
                        && m.observed < h.month
                        && !members.is_empty())
                    .then_some(InstitutionWorkPlan {
                        institution: n.id,
                        members,
                        requested: 0.05,
                        minimum: 0.05,
                        commitment: None,
                        granted: 0.,
                        used: 0.,
                    })
                })
                .collect()
        });
        let mut administration =
            (self.named_administration && h.participation.is_some()).then(|| {
                self.institutions
                    .iter()
                    .filter(|n| n.site == site && n.active)
                    .filter_map(|n| {
                        let members: Vec<_> = people
                            .iter()
                            .copied()
                            .filter(|p| n.members.contains(p))
                            .collect();
                        (h.month.is_multiple_of(3)
                            && !h.sites[site as usize].abandoned
                            && h.sites[site as usize].economy.finance[0] > 0.
                            && !members.is_empty())
                        .then_some(InstitutionWorkPlan {
                            institution: n.id,
                            members,
                            requested: 0.05,
                            minimum: 0.05,
                            commitment: None,
                            granted: 0.,
                            used: 0.,
                        })
                    })
                    .collect::<Vec<_>>()
            });
        for plans in [&mut elections, &mut upkeep, &mut administration]
            .into_iter()
            .flatten()
        {
            self.institution_priority.order(plans, h.month, site);
        }
        let space_feasible_work = Some(crate::institution_services::feasible_work(
            &actions,
            &services,
            study_expectation
                .as_ref()
                .is_some_and(|s| s.institution.is_some()),
        ));
        WorkPlan {
            lesson_opportunities: Some(self.lesson_opportunities(h, site, actor)),
            institutional_students: Some(self.institutional_students),
            space_feasible_work,
            services: Some(services),
            hearing,
            funding: self.plan_institution_funding(h, site),
            institution_priority: h.participation.as_ref().map(|_| self.institution_priority),
            institution_work_policy: h
                .participation
                .as_ref()
                .map(|_| self.institution_work_policy),
            upkeep,
            elections,
            administration,
            study_expectation,
            successor_expectation,
            completed: 0.,
            commitment: None,
            successor,
            institution_lesson,
            participants,
            object_dependencies,
            month: h.month,
            site,
            actor,
            actions,
            identities,
            cancellation: None,
            granted: 0.,
            cancelled_work: 0.,
            changed_identities: vec![],
        }
    }
    pub(super) fn validate_work_plans(&mut self, h: &History) {
        for i in 0..self.work_plans.len() {
            let p = &self.work_plans[i];
            if p.cancellation.is_some() {
                continue;
            }
            let current = self.work_identities(
                h,
                p.site,
                p.participants.as_deref(),
                p.object_dependencies.as_deref(),
            );
            let changed: Vec<String> = ["people", "faith", "objects", "institutions", "recoveries"]
                .into_iter()
                .chain(p.object_dependencies.as_ref().map(|_| "buildings"))
                .filter(|key| p.identities[*key] != current[*key])
                .map(str::to_owned)
                .collect();
            let reason = if p.month != h.month {
                Some("stale month")
            } else if h.sites[p.site as usize].abandoned {
                Some("site abandoned")
            } else if p.commitment.is_some() && h.personal_grant_live(p.commitment) <= 0. {
                Some("participant unavailable")
            } else if !changed.is_empty() {
                Some("actor or named target changed")
            } else {
                None
            };
            if let Some(reason) = reason {
                self.work_plans[i].cancelled_work = self.labor_budget[p.site as usize];
                self.work_plans[i].changed_identities = changed;
                self.labor_budget[self.work_plans[i].site as usize] = 0.;
                self.work_plans[i].cancellation = Some(reason.into());
            }
        }
    }
    pub(crate) fn work_allowed(&self, site: u32, action: &str) -> bool {
        self.work_plans
            .get(site as usize)
            .is_none_or(|p| p.cancellation.is_none() && p.actions.iter().any(|(a, _)| a == action))
    }
    pub(super) fn work_requests(&self, h: &History, site: u32) -> Vec<(&'static str, f32)> {
        let s = &h.sites[site as usize];
        if s.abandoned || !h.month.is_multiple_of(3) {
            return vec![];
        }
        let people = self.site_people(h, site);
        // Read-only queries may occur immediately after a population change.
        if people.iter().any(|&p| p as usize >= self.agents.len()) {
            return vec![];
        }
        let mut requests = vec![];
        for n in self
            .institutions
            .iter()
            .filter(|n| n.site == site && n.active && n.members.iter().any(|p| people.contains(p)))
        {
            if let Some(c) = &n.capacity {
                if c.observed < h.month {
                    let large = c.building.as_ref().is_some_and(|b| {
                        b.facility.is_some()
                            || self.artifacts[b.artifact as usize].materials.iter().any(
                                |&(g, mass)| {
                                    g == 5 && mass >= crate::institution_capacity::HALL_BRICKS_KG
                                },
                            )
                    });
                    requests.push(("institution upkeep", if large { 0.125 } else { 0.025 }));
                }
                if c.mandate
                    .as_ref()
                    .is_some_and(|m| m.holder.is_none() && m.observed < h.month)
                    && !self.institution_candidates(h, n.id).is_empty()
                {
                    requests.push(("institution election", 0.05));
                }
            }
        }
        // Historical-document study requires an actual due find and writing stock.
        if h.month.is_multiple_of(12)
            && h.economy_catalog
                .as_ref()
                .and_then(|c| c.index("writing_material"))
                .is_some_and(|g| s.economy.goods[g] >= 0.05)
            && self.institutions.iter().any(|n| {
                n.site == site
                    && (!self.funded_heritage_study
                        || crate::expedition_heritage::can_fund_study(h, site, n.treasury))
                    && n.operational()
                    && matches!(
                        n.kind,
                        InstitutionKind::Scholarly | InstitutionKind::Religious
                    )
                    && h.people[n.leader as usize].died.is_none()
            })
        {
            if let Some(x) = &h.expeditions {
                for find in x
                    .voyages
                    .iter()
                    .filter_map(|v| v.heritage.as_ref()?.find.as_ref())
                {
                    if find.studies.len() < 3
                        && find.studies.last().is_none_or(|r| h.month >= r.month + 60)
                        && find.artifact.is_some_and(|id| {
                            let a = &self.artifacts[id as usize];
                            a.site == Some(site) && !a.lost && !a.destroyed
                        })
                    {
                        requests.push(("heritage study", 0.1));
                    }
                }
            }
        }
        let Some(actor) = self.work_actor(h, site, &people) else {
            return requests;
        };
        let Some(faith) = self.resident_tradition(h, site, actor) else {
            return requests;
        };
        let a = &self.agents[actor as usize];
        let traits = a.traits;
        if self.artifacts.iter().any(|o| {
            o.lost && !o.destroyed && o.site.is_some_and(|id| h.sites[id as usize].cell == s.cell)
        }) {
            requests.push(("local object recovery", 0.1));
        }
        if self.artifacts.iter().any(|o| {
            !o.lost
                && !o.destroyed
                && o.site == Some(site)
                && o.topic.is_some_and(|topic| !a.knowledge.contains(&topic))
        }) || self.institutional_lesson(h, site, actor).is_some()
        {
            requests.push(("study", 0.1));
        }
        if self.succession_lesson(h, site, actor).is_some() {
            requests.push(("teach successor", 0.1));
        }
        if traits[2] > 0.7 && unit(h.seed, actor, h.month, 990) < 0.12 {
            let dest = crate::heritage_renown::destination(self, h, site, faith, 0.5)
                .map_or(self.traditions[faith as usize].sacred_site, |v| v.0);
            if dest != site && !h.sites[dest as usize].abandoned {
                if let Some(route) = h.society.as_ref().and_then(|soc| {
                    soc.routes.iter().find(|r| {
                        r.passable()
                            && ((r.from == site && r.to == dest)
                                || (r.to == site && r.from == dest))
                    })
                }) {
                    let work = route.cost_km * 2. / 1200.;
                    if work <= 0.5
                        && s.stocks.stock[1] >= work * 18. + s.stocks.stock[0] * 54.
                        && s.economy.goods[7] >= 0.1
                    {
                        requests.push(("pilgrimage", work.max(0.1)));
                    }
                }
            }
        }
        if traits[3] > 0.6
            && h.expeditions
                .as_ref()
                .and_then(|x| x.discoveries.as_ref())
                .is_some_and(|d| {
                    d.workshops.iter().any(|w| {
                        w.site == site
                            && (0..2).any(|k| {
                                w.samples[k] >= 2.
                                    && !self.artifacts.iter().any(|o| {
                                        !o.destroyed
                                            && o.site == Some(site)
                                            && o.kind == "expedition specimen"
                                            && o.materials.iter().any(|&(g, _)| g == 30 + k as u32)
                                    })
                            })
                    })
                })
        {
            requests.push(("specimen curation", 0.1));
        }
        if traits[0] > 0.75
            && unit(h.seed, actor, h.month, 993) < 0.08
            && h.politics.is_some()
            && h.society.is_some()
            && s.economy.finance[0] >= 202.
            && h.civilizations[h.people[actor as usize].civilization as usize].leader != actor
            && a.relations.values().any(|&r| r > 0.)
        {
            requests.push(("office campaign", 0.1));
        }
        let named_admin = self.named_administration && h.participation.is_some();
        let offices = self
            .institutions
            .iter()
            .filter(|n| {
                n.site == site
                    && n.active
                    && (!named_admin || n.members.iter().any(|p| people.contains(p)))
            })
            .count();
        if offices > 0 && s.economy.finance[0] > 0. {
            requests.push((
                "institution administration",
                if named_admin {
                    offices as f32 * 0.05
                } else {
                    (offices as f32 * 0.05).max(0.1)
                },
            ));
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
        let members = people
            .iter()
            .filter(|&&p| {
                kind != InstitutionKind::Religious
                    || self.resident_tradition(h, site, p) == Some(faith)
            })
            .count();
        if members >= 2
            && s.economy.finance[0] > 500.
            && !self.institutions.iter().any(|n| {
                n.active
                    && n.site == site
                    && n.kind == kind
                    && (kind != InstitutionKind::Religious || n.tradition == Some(faith))
            })
            && h.economy_catalog.as_ref().is_some_and(|cat| {
                crate::facilities::choose(
                    cat,
                    &s.economy,
                    crate::facilities::demand(&kind, members),
                    (s.economy.finance[0] as f64 * 0.15).max(0.),
                )
                .is_some()
                    || (cat.materials.is_none()
                        && s.economy.goods[5] >= crate::institution_capacity::HALL_BRICKS_KG)
            })
        {
            requests.push(("institution founding", 0.2));
        }
        if h.month / 3 % 4 == site % 4
            && s.economy.goods[7] >= 1.
            && self
                .artifacts
                .iter()
                .filter(|o| o.site == Some(site) && !o.destroyed)
                .count()
                < 16
        {
            requests.push(("craft object or manuscript", 0.2));
        }
        if traits[1] > 0.6
            && s.economy.finance[0] > 0.
            && h.society.as_ref().is_some_and(|soc| {
                soc.routes
                    .iter()
                    .filter(|r| r.passable() && (r.from == site || r.to == site))
                    .map(|r| if r.from == site { r.to } else { r.from })
                    .any(|dest| {
                        if soc.relocation.witnessed_relief {
                            soc.relocation.appeals.iter().any(|a| {
                                a.host == site && a.origin == dest && h.month <= a.reported + 18
                            })
                        } else {
                            h.sites[dest as usize].stocks.stock[3] > 0.01
                        }
                    })
            })
        {
            requests.push(("charity", 0.1));
        }
        if traits[0] > 0.8
            && traits[4] < 0.3
            && unit(h.seed, actor, h.month, 600) < 0.05
            && self.artifacts.iter().any(|o| {
                !o.destroyed && !o.lost && o.site == Some(site) && o.custodian != Some(actor)
            })
        {
            requests.push(("ownership dispute", 0.1));
        }
        // The same read-only scorer supplies the dated hearing's represented parties.
        if crate::civic_petitions::forecast(h, self, site).is_some() {
            requests.push(("petition hearing", 0.1));
        }
        requests
    }
}

/// Execution sites report completed labor directly; remaining budgets have mixed legacy meanings.
pub(crate) fn record_work(plans: &mut [WorkPlan], site: u32, month: u32, work: f32) {
    if let Some(p) = plans.get_mut(site as usize).filter(|p| p.month == month) {
        p.completed += work;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn institution_operating_requests_require_work_and_commit_cash_once() {
        use crate::{institution_capacity::Capacity, institution_funding::Policy};
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
        let h = g.civilizations.as_mut().unwrap();
        h.month = 12;
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        for a in &mut c.agents {
            a.knowledge.clear();
            a.traits = [0.; 6];
        }
        let members: Vec<_> = c.site_people(h, 0).into_iter().take(2).collect();
        assert_eq!(members.len(), 2);
        c.institutions = (0..2)
            .map(|id| Institution {
                capacity: Some(Capacity::new(0)),
                id,
                name: format!("Budget fixture {id}"),
                kind: InstitutionKind::Scholarly,
                site: 0,
                tradition: None,
                members: members.clone(),
                leader: members[0],
                treasury: 0.,
                active: true,
                founded: 0,
                knowledge: Default::default(),
                property: vec![],
                dues: 0.,
                expenses: 0.,
            })
            .collect();
        c.institution_funding = Policy::Operating;
        h.sites[0].economy.finance[0] = 100.;
        let before = serde_json::to_value(&h).unwrap();
        let planned = c.plan_institution_funding(h, 0).unwrap();
        assert_eq!(before, serde_json::to_value(&h).unwrap());
        assert_eq!(planned.pool, 0.5);
        assert!(planned
            .requests
            .iter()
            .all(|r| r.ceiling == 0.25 && r.target == 2.));
        planned.validate(&c.institutions, 0).unwrap();
        let mut invalid = planned.clone();
        invalid.requests[0].paid = 1.;
        assert!(invalid.validate(&c.institutions, 0).is_err());
        c.work_plans = h.sites.iter().map(|s| c.plan_work(h, s.id)).collect();
        c.work_plans[0].actions = vec![("institution administration".into(), 0.1)];
        c.labor_budget = vec![0.; h.sites.len()];
        let mut unfunded = c.clone();
        unfunded.decisions(h);
        assert_eq!(h.sites[0].economy.finance[0], 100.);
        assert!(unfunded.institutions.iter().all(|n| n.treasury == 0.));
        // A late change of policy does not replace captured operating ceilings.
        c.institution_funding = Policy::Legacy;
        c.labor_budget[0] = 0.1;
        let mut resumed: Culture =
            serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
        let mut resumed_h = h.clone();
        let opening = c.clone();
        c.decisions(h);
        resumed.decisions(&mut resumed_h);
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&h).unwrap(),
            serde_json::to_value(&resumed_h).unwrap()
        );
        assert_eq!(h.sites[0].economy.finance[0], 99.5);
        assert!(c.institutions.iter().all(|n| n.treasury == 0.25));
        assert!((c.labor_spent - opening.labor_spent - 0.1).abs() < 1e-6);
        c.work_plans[0]
            .funding
            .as_ref()
            .unwrap()
            .validate(&c.institutions, 0)
            .unwrap();
        let money = h.sites[0].economy.finance[0] as f64
            + c.institutions.iter().map(|n| n.treasury).sum::<f64>();
        assert_eq!(money, 100.);
        let cash = h.sites[0].economy.finance[0];
        c.decisions(h);
        assert_eq!(cash, h.sites[0].economy.finance[0]);
        assert!((c.labor_spent - opening.labor_spent - 0.1).abs() < 1e-6);
        let mut scarce = opening.clone();
        let mut scarce_h = resumed_h.clone();
        scarce_h.sites[0].economy.finance[0] = 0.125;
        scarce.decisions(&mut scarce_h);
        assert_eq!(scarce_h.sites[0].economy.finance[0], 0.);
        assert_eq!(
            scarce.institutions.iter().map(|n| n.treasury).sum::<f64>(),
            0.125
        );
        let mut stale = opening.clone();
        scarce_h.month += 1;
        scarce_h.sites[0].economy.finance[0] = 100.;
        stale.decisions(&mut scarce_h);
        assert_eq!(scarce_h.sites[0].economy.finance[0], 100.);
        let mut filled = opening.clone();
        filled.institutions[0].treasury = 2.;
        assert_eq!(filled.collect_institution_funding(h, 0, 0), Some(0.));
        assert_eq!(filled.institutions[0].treasury, 2.);
        let mut inactive = opening.clone();
        inactive.institutions[0].active = false;
        let untouched = h.sites[0].economy.finance[0];
        assert_eq!(inactive.collect_institution_funding(h, 0, 0), None);
        assert_eq!(untouched, h.sites[0].economy.finance[0]);
        let mut moved = opening.clone();
        moved.institutions[0].site = 1;
        assert_eq!(moved.collect_institution_funding(h, 0, 0), None);
        // Old captured plans retain the old donation schedule; fractional credits balance.
        let mut old = serde_json::to_value(&opening.work_plans[0]).unwrap();
        old.as_object_mut().unwrap().remove("funding");
        let mut legacy = opening.clone();
        legacy.work_plans[0] = serde_json::from_value(old).unwrap();
        let start = h.sites[0].economy.finance[0];
        legacy.collect_institution_funding(h, 0, 0).unwrap();
        assert_eq!(
            start as f64,
            h.sites[0].economy.finance[0] as f64 + legacy.institutions[0].treasury
        );
        // Quote the actual old building, but never subsidize an inaccessible one.
        legacy.institution_funding = Policy::Operating;
        let mut building = legacy.artifacts[0].clone();
        building.id = legacy.artifacts.len() as u32;
        building.owner = Owner::Institution(0);
        building.site = Some(0);
        building.materials = vec![(5, 2000.)];
        building.lost = false;
        building.destroyed = false;
        let id = building.id;
        legacy.artifacts.push(building);
        legacy.institutions[0].capacity.as_mut().unwrap().building =
            Some(crate::institution_capacity::MeetingPlace::new(id));
        h.sites[0].economy.soil[3] = 0.;
        let quote = legacy.plan_institution_funding(h, 0).unwrap();
        assert!(
            (quote.requests[0].target - (2. + 20. * h.sites[0].economy.prices[5].max(0.01) as f64))
                .abs()
                < 1e-6
        );
        legacy.artifacts[id as usize].lost = true;
        assert_eq!(
            legacy.plan_institution_funding(h, 0).unwrap().requests[0].target,
            2.
        );
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn requests_follow_people_knowledge_and_materials() {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 32,
                ecology_resolution: 16,
                seed: 17,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 12;
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        c.artifacts.clear();
        c.institutions.clear();
        c.local_recoveries.clear();
        for a in &mut c.agents {
            a.knowledge.clear();
            a.traits = [0.; 6];
        }
        for site in &mut h.sites {
            site.economy.finance[0] = 0.;
            site.economy.goods[7] = 0.;
        }
        let people = c.site_people(h, 0);
        assert!(people.len() >= 2);
        assert!(c.work_requests(h, 0).is_empty());
        let actor = people[(h.month / 3) as usize % people.len()];
        c.agents[actor as usize].knowledge.insert(0);
        assert_eq!(c.work_requests(h, 0), vec![("teach successor", 0.1)]);
        h.culture = Some(c);
        h.begin_service_reservations();
        h.reserve_cultural_work();
        assert!((h.culture.as_ref().unwrap().labor_budget[0] - 0.1).abs() < 1e-6);
        let mut c = h.culture.take().unwrap();
        let predicted = c.work_plans[0]
            .successor_expectation
            .as_ref()
            .unwrap()
            .clone();
        assert!((predicted.expected_gain - 1. / 3.).abs() < 1e-6);
        assert_eq!(predicted.actual_gain, 0.);
        let mut unfunded = c.clone();
        let mut unfunded_history = h.clone();
        unfunded.labor_budget[0] = 0.;
        unfunded.decisions(&mut unfunded_history);
        assert_eq!(
            unfunded.work_plans[0]
                .successor_expectation
                .as_ref()
                .unwrap()
                .actual_gain,
            0.
        );
        assert!(!unfunded.agents[predicted.student as usize]
            .studies
            .contains_key(&predicted.topic));
        let before = c.labor_spent;
        c.decisions(h);
        let observed = c.work_plans[0].successor_expectation.as_ref().unwrap();
        assert!((observed.actual_gain - predicted.expected_gain).abs() < 1e-6);
        assert!(!observed.actual_acquisition);
        let restored: WorkPlan =
            serde_json::from_value(serde_json::to_value(&c.work_plans[0]).unwrap()).unwrap();
        assert_eq!(
            restored.successor_expectation.unwrap().actual_gain,
            observed.actual_gain
        );
        assert!((c.labor_spent - before - 0.1).abs() < 1e-6);
        assert!(h.events.iter().any(|e| e.kind == "practice_instruction"));
        assert!((c.work_plans[0].completed - 0.1).abs() < 1e-6);
        // Completion is explicit even when the legacy grant array is not decremented.
        let commitment = c.work_plans[0].commitment.unwrap();
        h.culture = Some(c.clone());
        h.settle_participation().unwrap();
        assert!(
            (h.participation.as_ref().unwrap().commitments[commitment as usize].used - 0.1).abs()
                < 1e-6
        );
        h.culture = None;

        for p in &mut c.work_plans {
            p.commitment = None;
        }
        for a in &mut c.agents {
            a.knowledge.clear();
        }
        c.labor_budget = vec![0.5; h.sites.len()];
        let before = c.labor_spent;
        c.decisions(h);
        assert_eq!(
            c.labor_spent, before,
            "unused grants are not completed work"
        );
        assert!(c.work_requests(h, 0).is_empty());
        h.sites[0].economy.goods[7] = 1.;
        assert_eq!(
            c.work_requests(h, 0),
            vec![("craft object or manuscript", 0.2)]
        );
        h.sites[0].economy.goods[7] = 0.;
        assert!(c.work_requests(h, 0).is_empty());

        // A requested craft cannot spend vanished materials or switch to newly learned teaching.
        h.sites[0].economy.goods[7] = 1.;
        c.work_plans.clear();
        let original = c.clone();
        c.work_plans = h.sites.iter().map(|s| c.plan_work(h, s.id)).collect();
        c.labor_budget = vec![0.2; h.sites.len()];
        h.sites[0].economy.goods[7] = 0.;
        let before = c.labor_spent;
        c.decisions(h);
        assert_eq!(c.labor_spent, before);
        assert!(c.artifacts.is_empty());
        assert!(!c.work_allowed(0, "teach successor"));

        // Named actor loss cancels the bundle rather than selecting the next resident.
        let mut death = original.clone();
        death.work_plans = h.sites.iter().map(|s| death.plan_work(h, s.id)).collect();
        death.labor_budget = vec![0.2; h.sites.len()];
        let old_death = h.people[actor as usize].died;
        h.people[actor as usize].died = Some(h.month);
        death.validate_work_plans(h);
        assert_eq!(death.labor_budget[0], 0.);
        assert!(death.work_plans[0].cancellation.is_some());
        h.people[actor as usize].died = old_death;
        let mut migration = original.clone();
        migration.work_plans = h
            .sites
            .iter()
            .map(|s| migration.plan_work(h, s.id))
            .collect();
        migration.labor_budget = vec![0.2; h.sites.len()];
        let household = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .position(|hh| hh.site == 0 && hh.head == actor)
            .unwrap();
        h.society.as_mut().unwrap().households[household].site = 1;
        migration.validate_work_plans(h);
        assert_eq!(migration.labor_budget[0], 0.);
        assert!(migration.work_plans[0].cancellation.is_some());
        h.society.as_mut().unwrap().households[household].site = 0;

        // Unrelated residents do not revoke a named actor's feasible craft plan.
        h.sites[0].economy.goods[7] = 1.;
        let mut focused = original.clone();
        focused.focused_work_identities = true;
        focused.work_plans = h.sites.iter().map(|s| focused.plan_work(h, s.id)).collect();
        focused.labor_budget = vec![0.2; h.sites.len()];
        let bystander = *people.iter().find(|&&p| p != actor).unwrap();
        let mut strict = focused.clone();
        strict.focused_work_identities = false;
        strict.work_plans = h.sites.iter().map(|s| strict.plan_work(h, s.id)).collect();
        focused.agents[bystander as usize].knowledge.insert(1);
        strict.agents[bystander as usize].knowledge.insert(1);
        focused.validate_work_plans(h);
        strict.validate_work_plans(h);
        assert!(focused.work_plans[0].cancellation.is_none());
        assert!(strict.work_plans[0].cancellation.is_some());
        assert_eq!(strict.work_plans[0].cancelled_work, 0.2);
        strict.validate_work_plans(h);
        assert_eq!(
            strict.work_plans[0].cancelled_work, 0.2,
            "a second guard must not erase the original loss"
        );
        let restored: Culture =
            serde_json::from_slice(&serde_json::to_vec(&focused).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(&restored.work_plans).unwrap(),
            serde_json::to_value(&focused.work_plans).unwrap()
        );

        let before_work = focused.labor_spent;
        focused.decisions(h);
        assert!((focused.labor_spent - before_work - 0.2).abs() < 1e-6);
        assert_eq!(focused.artifacts.len(), 1);
        assert_eq!(h.sites[0].economy.goods[7], 0.);

        // A real teaching recipient remains part of the guard.
        let mut teaching = original.clone();
        teaching.agents[actor as usize].knowledge.insert(0);
        teaching.work_plans = h
            .sites
            .iter()
            .map(|s| teaching.plan_work(h, s.id))
            .collect();
        teaching.labor_budget = vec![0.2; h.sites.len()];
        let student = teaching.work_plans[0].successor.unwrap().0;
        teaching.agents[student as usize].knowledge.insert(0);
        teaching.validate_work_plans(h);
        assert!(teaching.work_plans[0].cancellation.is_some());

        let mut stale = original.clone();
        stale.work_plans = h.sites.iter().map(|s| stale.plan_work(h, s.id)).collect();
        stale.labor_budget = vec![0.2; h.sites.len()];
        h.month += 3;
        stale.validate_work_plans(h);
        assert_eq!(stale.labor_budget[0], 0.);
        assert_eq!(
            stale.work_plans[0].cancellation.as_deref(),
            Some("stale month")
        );
    }
}

#[cfg(test)]
mod allocation_tests {
    use super::*;
    use crate::{institution_capacity::WorkPolicy, participation::Participation};
    #[test]
    fn essential_work_competes_with_repairs_without_extra_people_or_time() {
        for cap in [0., 0.024, 0.05, 0.075, 0.1, 0.175, 0.25] {
            for policy in [WorkPolicy::FullUpkeepFirst, WorkPolicy::EssentialFirst] {
                // Import an older empty plan: no allocation policy is fabricated.
                let mut plan: WorkPlan = serde_json::from_value(serde_json::json!({
                    "month":3,"site":0,"actor":null,"successor":null,
                    "actions":[],"identities":null,"cancellation":null
                }))
                .unwrap();
                assert!(plan.institution_work_policy.is_none());
                plan.institution_work_policy = Some(policy);
                let request = |work, minimum| InstitutionWorkPlan {
                    institution: 0,
                    members: vec![7],
                    requested: work,
                    minimum,
                    commitment: None,
                    granted: 0.,
                    used: 0.,
                };
                plan.upkeep = Some(vec![request(0.125, 0.)]);
                plan.administration = Some(vec![request(0.05, 0.05)]);
                let mut state = Participation {
                    month: Some(3),
                    ..Default::default()
                };
                state.residents.insert(
                    7,
                    serde_json::from_value(serde_json::json!({
                        "person":7,"household":null,"presence":{"Resident":0},
                        "capacity":0.2,"committed":0.,"completed":[0.,0.]
                    }))
                    .unwrap(),
                );
                let mut available = cap;
                let used = plan.reserve_institution_work(&mut state, 3, 0, &mut available);
                assert!(used <= cap + 1e-6 && used <= 0.175001);
                assert!((used + available - cap).abs() < 1e-6);
                assert!((state.residents[&7].committed - used).abs() < 1e-6);
                assert!(state.commitments.len() <= 2);
                if cap == 0.1 {
                    let a = plan.administration.as_ref().unwrap()[0].granted;
                    assert_eq!(
                        a,
                        if policy == WorkPolicy::EssentialFirst {
                            0.05
                        } else {
                            0.
                        }
                    );
                }
                // Exact decimal sums can fall just below an indivisible f32 minimum.
                // The larger allowance is the ample-capacity control.
                if cap > 0.175 {
                    assert!((used - 0.175).abs() < 1e-6);
                }
                let mut resumed: Participation =
                    serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
                for (index, c) in state.commitments.clone().iter().enumerate() {
                    state.settle(index as u32, c.granted).unwrap();
                    resumed.settle(index as u32, c.granted).unwrap();
                }
                assert_eq!(
                    serde_json::to_value(&state).unwrap(),
                    serde_json::to_value(&resumed).unwrap()
                );
                let before = serde_json::to_value(&state).unwrap();
                for p in plan
                    .institution_work_mut()
                    .filter(|p| p.commitment.is_some())
                {
                    let mut extra = 1.;
                    p.reserve_up_to(&mut state, 3, 0, p.requested, &mut extra);
                }
                assert_eq!(
                    before,
                    serde_json::to_value(&state).unwrap(),
                    "settled grants cannot grow"
                );
            }
        }
    }
}
