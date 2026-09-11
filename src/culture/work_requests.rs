//! Opening-boundary requests, not guarantees: execution rechecks live people,
//! routes and materials. Nothing is spent or named while forecasting work.
use super::*;
/// A site's bounded bundle: actor and eligible named targets are fixed at Reserve.
/// Material stocks remain live and must pass the action's execution checks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkPlan {
    #[serde(default)]
    pub completed: f32,
    #[serde(default)]
    pub commitment: Option<u32>,
    pub month: u32,
    pub site: u32,
    pub actor: Option<u32>,
    pub successor: Option<(u32, u32, u32)>,
    #[serde(default)]
    pub participants: Option<Vec<u32>>,
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
impl Culture {
    fn work_identities(
        &self,
        h: &History,
        site: u32,
        participants: Option<&[u32]>,
    ) -> serde_json::Value {
        let people: Vec<_> = self
            .site_people(h, site)
            .into_iter()
            .filter(|p| participants.is_none_or(|ids| ids.contains(p)))
            .collect();
        serde_json::json!({
            "people": people.iter().map(|&p| (p, self.agents.get(p as usize).map(|a| &a.knowledge))).collect::<Vec<_>>(),
            "recoveries": self.local_recoveries.iter().filter(|r| r.site == site).collect::<Vec<_>>(),
            "faith": people.iter().map(|&p| self.resident_tradition(h, site, p)).collect::<Vec<_>>(),
            "objects": self.artifacts.iter().filter(|a| a.site.is_some_and(|s| h.sites[s as usize].cell == h.sites[site as usize].cell)).map(|a| (a.id, a.site, a.custodian, &a.owner, a.topic, a.lost, a.destroyed)).collect::<Vec<_>>(),
            "institutions": self.institutions.iter().filter(|n| n.site == site).map(|n| (n.id, n.leader, &n.members, &n.knowledge, n.active)).collect::<Vec<_>>(),
        })
    }
    pub(super) fn plan_work(&self, h: &History, site: u32) -> WorkPlan {
        let people = self.site_people(h, site);
        let actor = people
            .get((h.month / 3 + site) as usize % people.len().max(1))
            .copied();
        let successor = actor.and_then(|a| self.succession_lesson(h, site, a));
        let institution_lesson = actor.and_then(|a| self.institutional_lesson(h, site, a));
        let participants = self.focused_work_identities.then(|| {
            let mut ids: Vec<_> = actor
                .into_iter()
                .chain(successor.map(|s| s.0))
                .chain(institution_lesson.map(|l| l.1))
                .collect();
            ids.sort_unstable();
            ids.dedup();
            ids
        });
        let identities = self.work_identities(h, site, participants.as_deref());
        WorkPlan {
            completed: 0.,
            commitment: None,
            successor,
            institution_lesson,
            participants,
            month: h.month,
            site,
            actor: people
                .get((h.month / 3 + site) as usize % people.len().max(1))
                .copied(),
            actions: self
                .work_requests(h, site)
                .into_iter()
                .map(|(a, w)| (a.into(), w))
                .collect(),
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
            let current = self.work_identities(h, p.site, p.participants.as_deref());
            let changed: Vec<String> = ["people", "faith", "objects", "institutions", "recoveries"]
                .into_iter()
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
                if c.mandate.as_ref().is_some_and(|m| m.holder.is_none()) {
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
        let Some(&actor) = people.get(((h.month / 3 + site) as usize) % people.len().max(1)) else {
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
            let dest = self.traditions[faith as usize].sacred_site;
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
        let offices = self
            .institutions
            .iter()
            .filter(|n| n.site == site && n.active)
            .count();
        if offices > 0 && s.economy.finance[0] > 0. {
            requests.push((
                "institution administration",
                (offices as f32 * 0.05).max(0.1),
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
        // Institutional petition eligibility is evaluated by its existing scoring
        // function at execution. Request a bounded hearing only with representation.
        if h.governance.as_ref().is_some_and(|g| {
            g.petitions_enabled
                && !g
                    .petitions
                    .iter()
                    .any(|p| p.site == site && (p.resolved.is_none() || h.month < p.opened + 60))
        }) && h.politics.is_some()
            && self
                .institutions
                .iter()
                .any(|n| n.site == site && n.operational() && n.members.contains(&actor))
        {
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
        let before = c.labor_spent;
        c.decisions(h);
        assert!((c.labor_spent - before - 0.1).abs() < 1e-6);
        assert!(h.events.iter().any(|e| e.kind == "practice_taught"));
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
