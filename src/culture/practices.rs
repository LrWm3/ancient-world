use super::*;
impl Culture {
    pub(super) fn update_roles(&mut self, h: &History) {
        self.roles.resize(h.sites.len(), Default::default());
        for site in &h.sites {
            let activity = h.role_activity(site.id, Some(self));
            let state = &mut self.roles[site.id as usize];
            let years = h.month.saturating_sub(state.month).max(12) as f32 / 12.;
            for (i, (_, total)) in activity.into_iter().enumerate() {
                let annual = (total - state.cumulative[i]).max(0.) / years;
                state.scores[i] = if state.month == 0 {
                    annual
                } else {
                    state.scores[i] * 0.8 + annual * 0.2
                };
                state.cumulative[i] = total;
            }
            state.month = h.month;
        }
    }
}

impl Culture {
    /// Regional round trips completed within this month, using the actor's reserved
    /// work rather than creating a second demographic population in transit.
    pub(super) fn pilgrimage(&mut self, h: &mut History, site: u32, actor: u32, work: f32) -> bool {
        let Some(faith) = self.resident_tradition(h, site, actor) else {
            return false;
        };
        let heritage = crate::heritage_renown::destination(self, h, site, faith, work);
        let destination = heritage.map_or(self.traditions[faith as usize].sacred_site, |v| v.0);
        if site == destination || h.sites[destination as usize].abandoned {
            return false;
        }
        let km = h
            .society
            .as_ref()
            .and_then(|s| {
                s.routes.iter().find(|r| {
                    r.passable()
                        && ((r.from == site && r.to == destination)
                            || (r.to == site && r.from == destination))
                })
            })
            .map(|r| r.cost_km);
        let Some(km) = km else {
            return false;
        };
        // Forty km/day with established caravans, outbound and return. Longer
        // pilgrimages wait for greater travel infrastructure instead of teleporting.
        let travel_work = km * 2. / (40. * 30.);
        if travel_work > work || travel_work > 1. {
            return false;
        }
        let food = travel_work * 18.;
        let s = &h.sites[site as usize];
        if s.stocks.stock[1] < food + s.stocks.stock[0] * 18. * 3. || s.economy.goods[7] < 0.1 {
            return false;
        }
        self.labor_spent += travel_work as f64;
        crate::culture::work_requests::record_work(
            &mut self.work_plans,
            site,
            h.month,
            travel_work,
        );
        let source = &mut h.sites[site as usize];
        source.stocks.stock[1] -= food;
        source.stocks.ledger[1] += food;
        for (k, ratio) in [0.45, 0.02, 0.003].into_iter().enumerate() {
            source.economy.external[k] -= food * ratio;
        }
        source.economy.goods[7] -= 0.1;
        h.sites[destination as usize].economy.used[7] += 0.1;
        h.sites[destination as usize].economy.reserves[3] += 0.1;
        let cause = self.traditions[faith as usize]
            .patron
            .map(|id| self.patrons[id as usize].arrival_event);
        let departure = self.log(h,"pilgrimage_departed",site,Some(actor),Some(faith),None,cause,
            format!("A pious traveler set out for {}; {travel_work:.3} worker-months reserved, {food:.2} kg provisions consumed outside managed plots",h.sites[destination as usize].name));
        if let Some((_, artifact)) = heritage {
            let event = h.events.last_mut().unwrap();
            event.subjects.push(("artifact".into(), artifact));
            if let Some(r) = self.heritage_renown.iter().find(|r| r.artifact == artifact) {
                event.causes.push(r.event);
            }
            event
                .detail
                .push_str("; visiting a recovered heritage object");
        }
        let mut learned = None;
        let teacher = self.site_people(h, destination).first().copied();
        if let Some(teacher) = teacher {
            if let Some(topic) = self.agents[teacher as usize]
                .knowledge
                .difference(&self.agents[actor as usize].knowledge)
                .next()
                .copied()
            {
                // Observation during an already funded visit, not a second paid lesson.
                if let Some((completed, progress, previous)) =
                    self.agents[actor as usize].observe_topic(topic, h.month, 0.05)
                {
                    learned = Some((teacher, topic, completed, progress, previous));
                    self.agents[actor as usize].relations.insert(teacher, 0.5);
                }
            }
        }
        self.agents[actor as usize]
            .known_places
            .insert(h.sites[destination as usize].cell);
        self.agents[actor as usize].goal = "honor the founding charge".into();
        self.agents[actor as usize].actions += 1;
        let returned = self.log(h,"pilgrimage_returned",site,Some(actor),Some(faith),None,Some(departure),
            "The pilgrim returned within the month after a local visit, teaching contact and a 0.1 kg ceramic offering".into());
        if let Some((teacher, topic, completed, progress, previous)) = learned {
            self.agents[actor as usize].study_source(topic, returned, completed);
            h.events[returned as usize].detail.push_str(&format!(
                " Observed {}; study progress {:.0}%, practical access {}.",
                TOPICS[topic as usize],
                progress * 100.,
                if completed {
                    "acquired"
                } else {
                    "not yet acquired"
                }
            ));
            if let Some(previous) = previous {
                h.events[returned as usize].causes.push(previous);
            }
            h.events[returned as usize]
                .subjects
                .push(("person".into(), teacher));
            if let Some(&source) = self.agents[teacher as usize].knowledge_sources.get(&topic) {
                h.events[returned as usize].causes.push(source);
            }
        }
        self.account(
            h,
            faith,
            Some(actor),
            vec![departure, returned],
            "Visiting the remembered place renewed my understanding of our communal duties.".into(),
        );
        true
    }
}

impl Culture {
    pub(super) fn curate_specimen(&mut self, h: &mut History, site: u32, actor: u32) -> bool {
        let Some(catalog) = &h.economy_catalog else {
            return false;
        };
        if catalog.index("faultroot_specimen") != Some(30)
            || catalog.index("phosphatic_specimen") != Some(31)
        {
            return false;
        }
        // Exactly the same material identity as the discovery ledger is required.
        for k in 0..2 {
            for element in 0..3 {
                if (catalog.composition(30 + k)[element] as f64
                    - crate::discoveries::CNP[k][element])
                    .abs()
                    > 1e-7
                {
                    return false;
                }
            }
        }
        let Some(d) = h.expeditions.as_mut().and_then(|x| x.discoveries.as_mut()) else {
            return false;
        };
        let Some(w) = d.workshops.iter_mut().find(|w| w.site == site) else {
            return false;
        };
        let kind = (0..2).find(|&k| {
            w.samples[k] >= 2.
                && !self.artifacts.iter().any(|a| {
                    !a.destroyed
                        && a.site == Some(site)
                        && a.kind == "expedition specimen"
                        && a.materials.iter().any(|&(id, _)| id == 30 + k as u32)
                })
        });
        let Some(k) = kind else {
            return false;
        };
        let mass = 0.125;
        w.samples[k] -= mass;
        w.curated[k] += mass;
        d.curated[k] += mass;
        let cause = w.causes[k];
        let topic = if crate::discoveries::study_complete(w.studied[k]) {
            Some(8 + k as u32)
        } else {
            None
        };
        // This is a material identity conversion, not an elemental import: C/N/P
        // move out of discovery holdings into artifact holdings unchanged.
        h.sites[site as usize].economy.initial[30 + k] += mass as f32;
        let institution = self
            .institutions
            .iter()
            .find(|n| n.site == site && n.active && n.kind == InstitutionKind::Scholarly)
            .map(|n| n.id);
        let id = self.artifacts.len() as u32;
        let event = self.log(h,"specimen_curated",site,Some(actor),None,Some(id),cause,
            format!("A curator preserved {mass} kg of {} from delivered workshop stocks as a unique physical object",crate::discoveries::NAMES[k]));
        self.artifacts.push(Artifact {
            id,
            name: format!(
                "{} sample held by {}",
                crate::discoveries::NAMES[k],
                h.people[actor as usize].name
            ),
            kind: "expedition specimen".into(),
            creator: Some(actor),
            owner: institution.map_or(Owner::Person(actor), Owner::Institution),
            claims: vec![],
            site: Some(site),
            custodian: Some(actor),
            materials: vec![(30 + k as u32, mass as f32)],
            topic,
            tradition: None,
            events: vec![event],
            destroyed: false,
            lost: false,
        });
        if let Some(institution) = institution {
            self.institutions[institution as usize].property.push(id);
            h.events[event as usize]
                .subjects
                .push(("institution".into(), institution));
        }
        if let Some(topic) = topic {
            self.agents[actor as usize].knowledge.insert(topic);
            self.agents[actor as usize]
                .knowledge_sources
                .insert(topic, event);
        }
        self.agents[actor as usize].actions += 1;
        self.agents[actor as usize].goal = "preserve evidence from the ancient world".into();
        true
    }
}

impl Culture {
    pub(super) fn abandon_objects(&mut self, h: &mut History) {
        for id in 0..self.artifacts.len() {
            let a = &self.artifacts[id];
            if a.destroyed || a.lost || !a.site.is_some_and(|s| h.sites[s as usize].abandoned) {
                continue;
            }
            let site = a.site.unwrap();
            let event = self.log(h,"artifact_lost",site,None,a.tradition,Some(id as u32),a.events.last().copied(),
                "The settlement was abandoned; the object remains in its ruins, but access and custody were lost".into());
            let a = &mut self.artifacts[id];
            a.lost = true;
            a.custodian = None;
            a.events.push(event);
        }
    }
    pub(super) fn recover_object(&mut self, h: &mut History, site: u32, actor: u32) -> bool {
        if h.sites[site as usize].abandoned {
            return false;
        }
        let Some(id) = self.artifacts.iter().position(|a| {
            a.lost
                && !a.destroyed
                && a.site
                    .is_some_and(|s| h.sites[s as usize].cell == h.sites[site as usize].cell)
        }) else {
            return false;
        };
        self.recover_specific(h, site, actor, id)
    }
    pub(crate) fn recover_specific(
        &mut self,
        h: &mut History,
        site: u32,
        actor: u32,
        id: usize,
    ) -> bool {
        if h.sites[site as usize].abandoned
            || !self.artifacts.get(id).is_some_and(|a| {
                a.lost
                    && !a.destroyed
                    && a.site
                        .is_some_and(|s| h.sites[s as usize].cell == h.sites[site as usize].cell)
            })
        {
            return false;
        }
        let a = &self.artifacts[id];
        let event = self.log(h,"artifact_recovered",site,Some(actor),a.tradition,Some(id as u32),a.events.last().copied(),
            "A resident used reserved work to recover an object from local ruins; prior ownership claims remain".into());
        let a = &mut self.artifacts[id];
        a.site = Some(site);
        a.custodian = Some(actor);
        a.lost = false;
        a.events.push(event);
        self.agents[actor as usize].actions += 1;
        self.agents[actor as usize].goal = "recover a communal legacy".into();
        true
    }
}

impl Culture {
    pub(super) fn seek_office(&mut self, h: &mut History, site: u32, actor: u32) -> bool {
        let civilization = h.people[actor as usize].civilization;
        if h.civilizations[civilization as usize].leader == actor
            || h.politics.is_none()
            || h.sites[site as usize].economy.finance[0] < 202.
            || !self.agents[actor as usize]
                .relations
                .values()
                .any(|&v| v > 0.)
        {
            return false;
        }
        let Some(society) = &mut h.society else {
            return false;
        };
        h.sites[site as usize].economy.finance[0] -= 2.;
        society.councils[civilization as usize].treasury += 2.;
        self.agents[actor as usize].skills[0] =
            (self.agents[actor as usize].skills[0] + 0.02).min(1.);
        self.agents[actor as usize].goal = "seek office through faction support".into();
        self.agents[actor as usize].actions += 1;
        let event = self.log(h,"office_campaign",site,Some(actor),None,None,None,
            "An ambitious resident with personal supporters spent reserved work and two money on a civic campaign; office remains subject to existing faction and succession rules".into());
        self.agents[actor as usize].last_campaign = Some(event);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world() -> Generator {
        let mut g = Generator::new(
            pollster::block_on(crate::gpu::ContextGpu::headless()).unwrap(),
            crate::config::Config {
                resolution: 64,
                ecology_resolution: 64,
                seed: 17,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        g.found_civilizations(16).unwrap();
        g.enable_society().unwrap();
        g
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn contact_completion_cannot_relay_across_two_routes_in_one_year() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 12;
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .find(|r| r.passable() && !r.cells.is_empty())
            .unwrap()
            .clone();
        // Controlled three-town contact graph, independent of transport capacity.
        let mut ab = route.clone();
        ab.id = 0;
        ab.from = 0;
        ab.to = 1;
        let mut bc = route;
        bc.id = 1;
        bc.from = 1;
        bc.to = 2;
        h.society.as_mut().unwrap().routes = vec![ab, bc];
        c.site_faith.fill(0);
        c.household_faith.fill(0);
        for a in &mut c.agents {
            a.knowledge.clear();
            a.studies.clear();
            a.knowledge_sources.clear();
            a.last_learning_exposure = None;
        }
        let teacher = c.site_people(h, 0)[0] as usize;
        let student = c.site_people(h, 1)[0] as usize;
        let distant = c.site_people(h, 2)[0] as usize;
        c.agents[teacher].knowledge.insert(4);
        c.agents[student].studies.insert(
            4,
            Study {
                progress: 0.99,
                source: None,
            },
        );
        c.year(h);
        assert!(c.agents[student].knowledge.contains(&4));
        assert!(!c.agents[distant].studies.contains_key(&4));
        assert!(!c.agents[distant].knowledge.contains(&4));
        h.month = 24;
        c.year(h);
        assert!(c.agents[distant].studies[&4].progress > 0.);
    }

    #[test]
    #[ignore = "requires a hardware GPU"]
    fn flood_closure_blocks_new_contact_but_not_completed_delivery_evidence() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 12;
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .find(|r| r.passable())
            .unwrap()
            .clone();
        for r in &mut h.society.as_mut().unwrap().routes {
            r.open = false;
        }
        h.society.as_mut().unwrap().routes[route.id as usize].open = true;
        h.society.as_mut().unwrap().routes[route.id as usize].flood_months = 2;
        for a in &mut c.agents {
            a.knowledge.clear();
            a.knowledge_sources.clear();
        }
        c.site_faith[route.to as usize] = c.site_faith[route.from as usize];
        for hh in &h.society.as_ref().unwrap().households {
            if hh.site == route.to {
                c.household_faith[hh.id as usize] = c.site_faith[route.from as usize];
            }
        }
        let teacher = c.site_people(h, route.from)[0];
        let student = c.site_people(h, route.to)[0];
        c.agents[teacher as usize].knowledge.insert(4);
        let mut closed_h = h.clone();
        let mut closed = c.clone();
        closed.year(&mut closed_h);
        assert!(!closed.agents[student as usize].knowledge.contains(&4));
        assert!(closed.contact.is_empty());
        let mut open_h = h.clone();
        open_h.society.as_mut().unwrap().routes[route.id as usize].flood_months = 0;
        let mut open = c.clone();
        open.year(&mut open_h);
        assert!(!open.agents[student as usize].knowledge.contains(&4));
        assert!(open.agents[student as usize].studies[&4].progress > 0.);
        assert!(open_h
            .events
            .iter()
            .any(|e| e.kind == "knowledge_contact_progress" && e.site == Some(route.to)));
        let prior = open.agents[student as usize].studies[&4].progress;
        open.year(&mut open_h);
        assert_eq!(open.agents[student as usize].studies[&4].progress, prior);
        // A witnessed arrival before the closure remains real historical contact.
        let mut delivered_h = h.clone();
        delivered_h.event(
            "market_arrival",
            Some(route.to),
            Some(route.from),
            "Declared completed-delivery contact fixture".into(),
        );
        let arrival = delivered_h.events.last().unwrap().id;
        let mut delivered = c.clone();
        delivered.year(&mut delivered_h);
        assert!(!delivered.agents[student as usize].knowledge.contains(&4));
        assert!(delivered.agents[student as usize].studies[&4].progress > 0.);
        assert!(delivered_h
            .events
            .iter()
            .any(|e| e.kind == "knowledge_contact_progress"
                && e.site == Some(route.to)
                && e.causes.contains(&arrival)));
        // Evidence outside the one-year contact window cannot bypass current closure.
        let mut stale_h = h.clone();
        stale_h.month = 0;
        stale_h.event(
            "market_arrival",
            Some(route.to),
            Some(route.from),
            "Old delivery fixture".into(),
        );
        stale_h.month = 12;
        let mut stale = c.clone();
        stale.year(&mut stale_h);
        assert!(!stale.agents[student as usize].knowledge.contains(&4));
        // Small personal relief transfers must obey the same closure and conserve cash.
        let mut gift_h = h.clone();
        gift_h.society.as_mut().unwrap().relocation.witnessed_relief = false;
        gift_h.sites[route.to as usize].stocks.stock[3] = 0.2;
        let mut gifts = c.clone();
        for a in &mut gifts.agents {
            a.knowledge.clear();
            a.knowledge_sources.clear();
            a.traits = [0.; 6];
            a.traits[1] = 1.;
        }
        gifts.labor_budget = vec![0.; h.sites.len()];
        gifts.labor_budget[route.from as usize] = 0.1;
        let receiving = gift_h.sites[route.to as usize].economy.finance[0];
        let money = gift_h
            .sites
            .iter()
            .map(|s| s.economy.finance[0] as f64)
            .sum::<f64>();
        gifts.decisions(&mut gift_h);
        assert_eq!(
            gift_h.sites[route.to as usize].economy.finance[0],
            receiving
        );
        assert!(!gift_h.events.iter().any(|e| e.kind == "charitable_gift"));
        gift_h.society.as_mut().unwrap().routes[route.id as usize].flood_months = 0;
        gifts.decisions(&mut gift_h);
        assert_eq!(
            gift_h.sites[route.to as usize].economy.finance[0],
            receiving + 3.
        );
        assert!(
            (gift_h
                .sites
                .iter()
                .map(|s| s.economy.finance[0] as f64)
                .sum::<f64>()
                - money)
                .abs()
                < 0.001
        );
        // Restoring passability permits the formerly isolated population to learn.
        closed_h.society.as_mut().unwrap().routes[route.id as usize].flood_months = 0;
        closed_h.month = 24;
        closed.year(&mut closed_h);
        assert!(!closed.agents[student as usize].knowledge.contains(&4));
        assert!(closed.agents[student as usize].studies[&4].progress > 0.);
    }
    #[test]
    #[ignore = "requires a hardware GPU"]
    fn scarce_practices_receive_successors_before_common_topics() {
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
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        let people = c.site_people(h, 0);
        assert!(people.len() >= 3);
        let actor = people[(h.month / 3) as usize % people.len()];
        for agent in &mut c.agents {
            agent.knowledge.clear();
            agent.knowledge_sources.clear();
            agent.traits = [0.; 6];
        }
        c.artifacts.clear();
        c.institutions.clear();
        c.agents[actor as usize].knowledge.extend([0, 4]);
        c.agents[actor as usize]
            .knowledge_sources
            .insert(4, h.events[0].id);
        let helper = *people.iter().find(|&&p| p != actor).unwrap();
        c.agents[helper as usize].knowledge.insert(0);
        assert_eq!(c.knowledge_holders(h, 0)[0], 2);
        assert_eq!(c.knowledge_holders(h, 0)[4], 1);
        assert_eq!(c.knowledge_holders(h, u32::MAX), [0; TOPICS.len()]);
        let (student, topic, holders) = c.succession_lesson(h, 0, actor).unwrap();
        assert_eq!(
            (topic, holders),
            (4, 1),
            "rare metalworking precedes common topic zero"
        );
        let untaught = c.clone();
        let cash = h.sites[0].economy.finance[0];
        let goods = h.sites[0].economy.goods;
        c.labor_budget = vec![0.; h.sites.len()];
        c.labor_budget[0] = 0.09;
        c.decisions(h);
        assert!(!c.agents[student as usize].knowledge.contains(&4));
        let spent = c.labor_spent;
        c.labor_budget[0] = 0.1;
        c.decisions(h);
        assert!(!c.agents[student as usize].knowledge.contains(&4));
        assert!(c.agents[student as usize].studies[&4].progress > 0.);
        assert_eq!(c.succession_lesson(h, 0, actor).unwrap().0, student);
        // Separate fixture grants represent successive paid lessons.
        c.decisions(h);
        c.decisions(h);
        assert!(c.agents[student as usize].knowledge.contains(&4));
        assert!(!c.agents[student as usize].knowledge.contains(&0));
        assert!((c.labor_spent - spent - 0.3).abs() < 1e-7);
        assert_eq!(h.sites[0].economy.finance[0], cash);
        assert_eq!(h.sites[0].economy.goods, goods);
        let event = h
            .events
            .iter()
            .rev()
            .find(|e| e.kind == "practice_taught")
            .unwrap();
        assert!(event.detail.contains("1 local adult holder(s)"));
        assert!(event.causes.contains(&h.events[0].id));
        assert!(event.subjects.contains(&("person".into(), student)));
        assert_eq!(c.agents[student as usize].knowledge_sources[&4], event.id);
        // The teacher's death removes the skill only from the untaught comparison.
        h.people[actor as usize].died = Some(h.month);
        assert_eq!(untaught.available_knowledge(h, 0) & (1 << 4), 0);
        assert_ne!(c.available_knowledge(h, 0) & (1 << 4), 0);
        assert_eq!(c.succession_lesson(h, 0, actor), None);
        h.culture = Some(c);
        let mut resumed: History =
            serde_json::from_value(serde_json::to_value(&*h).unwrap()).unwrap();
        h.reserve_cultural_work();
        resumed.reserve_cultural_work();
        assert_ne!((h.sites[0].economy.management[3] as u32) & (1 << 4), 0);
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        h.culture = Some(untaught);
        h.reserve_cultural_work();
        assert_eq!((h.sites[0].economy.management[3] as u32) & (1 << 4), 0);
    }

    #[test]
    #[ignore = "requires a hardware GPU"]
    fn institutional_learning_needs_a_present_source_and_preserves_provenance() {
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
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        let site = 0;
        let people = c.site_people(h, site);
        assert!(people.len() >= 2);
        let actor = people[((h.month / 3 + site) as usize) % people.len()];
        let teacher = *people.iter().find(|&&p| p != actor).unwrap();
        for &p in &people {
            c.agents[p as usize].knowledge.clear();
            c.agents[p as usize].knowledge_sources.clear();
            c.agents[p as usize].traits = [0.; 6];
        }
        let institution = c.institutions.len() as u32;
        c.institutions.push(Institution {
            capacity: None,
            id: institution,
            name: "Fixture craft school".into(),
            kind: InstitutionKind::Craft,
            site,
            tradition: None,
            members: vec![actor, teacher],
            leader: teacher,
            treasury: 0.,
            active: true,
            founded: h.month,
            knowledge: [4].into_iter().collect(),
            property: vec![],
            dues: 0.,
            expenses: 0.,
        });
        assert_eq!(
            c.institutional_lesson(h, site, actor),
            None,
            "historical topic list cannot teach"
        );
        assert_eq!(c.available_knowledge(h, site), 0);
        c.agents[teacher as usize].knowledge.insert(4);
        h.event(
            "knowledge_taught",
            Some(site),
            None,
            "Declared teacher's earlier instruction".into(),
        );
        let source = h.events.last().unwrap().id;
        c.agents[teacher as usize]
            .knowledge_sources
            .insert(4, source);
        assert_eq!(
            c.institutional_lesson(h, site, actor),
            Some((4, teacher, institution))
        );
        assert_eq!(c.available_knowledge(h, site), 1 << 4);
        h.people[teacher as usize].died = Some(h.month);
        assert_eq!(c.institutional_lesson(h, site, actor), None);
        assert_eq!(
            c.available_knowledge(h, site),
            0,
            "dead household heads cannot enable production"
        );
        h.people[teacher as usize].died = None;
        let household = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .position(|hh| hh.head == teacher && hh.site == site)
            .unwrap();
        h.society.as_mut().unwrap().households[household].site = 1;
        assert_eq!(
            c.institutional_lesson(h, site, actor),
            None,
            "remote membership is not local instruction"
        );
        assert_eq!(c.available_knowledge(h, site), 0);
        h.society.as_mut().unwrap().households[household].site = site;
        c.institutions[institution as usize].active = false;
        assert_eq!(c.institutional_lesson(h, site, actor), None);
        assert_eq!(
            c.available_knowledge(h, site),
            1 << 4,
            "institution closure does not erase personal skills"
        );
        c.institutions[institution as usize].active = true;
        // Declare a finite existing-stock room for the service-space fixture.
        let room = c.artifacts.len() as u32;
        let materials = vec![(0, 20.)];
        h.sites[site as usize].economy.goods[0] += 20.; // Declared fixture input.
        h.sites[site as usize].economy.goods[0] -= 20.;
        c.artifacts.push(Artifact {
            id: room,
            name: "Fixture school room".into(),
            kind: "institutional foundation".into(),
            creator: None,
            owner: Owner::Institution(institution),
            claims: vec![],
            site: Some(site),
            custodian: None,
            materials,
            topic: None,
            tradition: None,
            events: vec![],
            destroyed: false,
            lost: false,
        });
        c.institutions[institution as usize].capacity =
            Some(crate::institution_capacity::Capacity {
                building: Some(crate::institution_capacity::MeetingPlace {
                    condition: 0.99, // Minor wear reduces service time, not the two-person group.
                    ..crate::institution_capacity::MeetingPlace::new(room)
                }),
                ..crate::institution_capacity::Capacity::new(h.month)
            });
        // An unfillable duty must not fund a lesson already denied physical space.
        let mut blocked_history = h.clone();
        let mut blocked = c.clone();
        blocked.artifacts[room as usize].lost = true;
        let mut plans: Vec<_> = blocked_history
            .sites
            .iter()
            .map(|s| blocked.plan_work(&blocked_history, s.id))
            .collect();
        let plan = &mut plans[site as usize];
        plan.actions = vec![("study".into(), 0.1), ("institution upkeep".into(), 0.125)];
        plan.elections = None;
        plan.administration = None;
        plan.upkeep = Some(vec![super::super::work_requests::InstitutionWorkPlan {
            institution,
            members: vec![],
            requested: 0.125,
            minimum: 0.,
            commitment: None,
            granted: 0.,
            used: 0.,
        }]);
        plan.space_feasible_work = Some(crate::institution_services::feasible_work(
            &plan.actions,
            plan.services.as_ref().unwrap(),
            true,
        ));
        assert_eq!(plan.feasible_work(), 0.125);
        blocked_history.culture = Some(blocked);
        blocked_history.open_participation();
        assert!(blocked_history.participation.is_some());
        blocked_history.reserve_cultural_plans(plans, &vec![0.5; h.sites.len()]);
        let plan = &blocked_history.culture.as_ref().unwrap().work_plans[site as usize];
        assert_eq!(plan.granted, 0.);
        assert!(plan.commitment.is_none());
        // Reservation captures a specific source and does not itself teach.
        let mut predicted = c.clone();
        predicted.work_plans = h
            .sites
            .iter()
            .map(|s| predicted.plan_work(h, s.id))
            .collect();
        predicted.labor_budget = vec![0.; h.sites.len()];
        let expectation = predicted.work_plans[0]
            .study_expectation
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(expectation.teacher, Some(teacher));
        assert_eq!(expectation.institution, Some(institution));
        assert_eq!(expectation.object, None);
        assert!(expectation.lesson.expected_gain > 0.);
        // A late manuscript must neither revoke this lesson nor replace its teacher.
        let mut scoped_history = h.clone();
        scoped_history.sites[site as usize].economy.finance[0] = 0.;
        let mut scoped = c.clone();
        let unrelated = scoped.artifacts.len() as u32;
        let mut book = scoped.artifacts[room as usize].clone();
        book.id = unrelated;
        book.name = "Declared fixture manuscript".into();
        book.kind = "manuscript".into();
        book.materials = vec![(0, 1.)]; // Declared fixture inventory.
        book.topic = None;
        scoped.artifacts.push(book);
        scoped.work_plans = scoped_history
            .sites
            .iter()
            .map(|s| scoped.plan_work(&scoped_history, s.id))
            .collect();
        assert_eq!(
            scoped.work_plans[site as usize].object_dependencies,
            Some(vec![room])
        );
        scoped.labor_budget = vec![0.; h.sites.len()];
        scoped.labor_budget[site as usize] = 0.1;
        let mut baseline = scoped.clone();
        baseline.decisions(&mut scoped_history.clone());
        let mut legacy = scoped.clone();
        legacy.focused_work_identities = false;
        legacy.work_plans = scoped_history
            .sites
            .iter()
            .map(|s| legacy.plan_work(&scoped_history, s.id))
            .collect();
        assert!(legacy.work_plans[site as usize]
            .object_dependencies
            .is_none());
        scoped.artifacts[unrelated as usize].topic = Some(4);
        legacy.artifacts[unrelated as usize].topic = Some(4);
        legacy.validate_work_plans(&scoped_history);
        assert!(legacy.work_plans[site as usize].cancellation.is_some());
        let mut restored: Culture =
            serde_json::from_value(serde_json::to_value(&scoped).unwrap()).unwrap();
        scoped.decisions(&mut scoped_history.clone());
        restored.decisions(&mut scoped_history.clone());
        assert!(scoped.work_plans[site as usize].cancellation.is_none());
        let outcome = scoped.work_plans[site as usize]
            .study_expectation
            .as_ref()
            .unwrap();
        assert!(outcome.lesson.actual_gain > 0.);
        assert_eq!(outcome.teacher, Some(teacher));
        assert_eq!(outcome.object, None);
        assert_eq!(
            outcome.lesson.actual_gain,
            baseline.work_plans[site as usize]
                .study_expectation
                .as_ref()
                .unwrap()
                .lesson
                .actual_gain
        );
        assert_eq!(
            serde_json::to_value(&scoped).unwrap(),
            serde_json::to_value(&restored).unwrap()
        );
        // A different building cannot inherit the old room grant.
        let mut replaced = c.clone();
        replaced
            .artifacts
            .push(scoped.artifacts[unrelated as usize].clone());
        replaced.artifacts[unrelated as usize].topic = None;
        replaced.work_plans = scoped_history
            .sites
            .iter()
            .map(|s| replaced.plan_work(&scoped_history, s.id))
            .collect();
        assert_eq!(
            replaced.work_plans[site as usize].object_dependencies,
            Some(vec![room])
        );
        replaced.institutions[institution as usize]
            .capacity
            .as_mut()
            .unwrap()
            .building
            .as_mut()
            .unwrap()
            .artifact = unrelated;
        replaced.labor_budget = vec![0.1; h.sites.len()];
        replaced.validate_work_plans(&scoped_history);
        assert!(replaced.work_plans[site as usize].cancellation.is_some());
        let mut damaged = predicted.clone();
        let mut damaged_history = h.clone();
        damaged.artifacts[room as usize].destroyed = true;
        damaged.labor_budget[0] = 0.1;
        let spent = damaged.labor_spent;
        damaged.decisions(&mut damaged_history);
        assert_eq!(
            damaged.work_plans[0]
                .study_expectation
                .as_ref()
                .unwrap()
                .lesson
                .actual_gain,
            0.
        );
        assert_eq!(damaged.labor_spent, spent);
        // Production cancellation leaves its grant unused until cultural settlement.
        for plan in damaged.work_plans[0].services.as_mut().unwrap() {
            plan.close(h.month);
        }
        let receipt = &damaged.work_plans[0].services.as_ref().unwrap()[0].receipts[0];
        assert!(receipt.settled);
        assert_eq!(receipt.used, 0.);
        assert!(receipt.released() > 0.);
        let mut absent = predicted.clone();
        let mut absent_history = h.clone();
        absent_history.people[teacher as usize].died = Some(h.month);
        absent.labor_budget[0] = 0.1;
        absent.decisions(&mut absent_history);
        assert_eq!(
            absent.work_plans[0]
                .study_expectation
                .as_ref()
                .unwrap()
                .lesson
                .actual_gain,
            0.
        );
        let mut predicted_history = h.clone();
        predicted.decisions(&mut predicted_history);
        assert_eq!(
            predicted.work_plans[0]
                .study_expectation
                .as_ref()
                .unwrap()
                .lesson
                .actual_gain,
            0.
        );
        predicted.labor_budget[0] = 0.1;
        predicted.decisions(&mut predicted_history);
        assert!(
            (predicted.work_plans[0]
                .study_expectation
                .as_ref()
                .unwrap()
                .lesson
                .actual_gain
                - expectation.lesson.expected_gain)
                .abs()
                < 1e-6
        );
        c.labor_budget = vec![0.; h.sites.len()];
        c.decisions(h);
        assert!(
            !c.agents[actor as usize].knowledge.contains(&4),
            "no reserved study work"
        );
        c.labor_budget[site as usize] = 0.1;
        c.decisions(h);
        assert!(!c.agents[actor as usize].knowledge.contains(&4));
        let partial_source = c.agents[actor as usize].studies[&4].source.unwrap();
        h.people[teacher as usize].died = Some(h.month);
        assert_eq!(
            c.available_knowledge(h, site),
            0,
            "partial study cannot replace the last teacher"
        );
        let progress = c.agents[actor as usize].studies[&4].progress;
        c.decisions(h);
        assert_eq!(
            c.agents[actor as usize].studies[&4].progress, progress,
            "absent source cannot advance study"
        );
        h.people[teacher as usize].died = None;
        let mut resumed: Culture =
            serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
        let mut resumed_history = h.clone();
        c.decisions(h);
        c.decisions(h);
        resumed.decisions(&mut resumed_history);
        resumed.decisions(&mut resumed_history);
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&h.events).unwrap(),
            serde_json::to_value(&resumed_history.events).unwrap()
        );
        assert!(h.events.iter().any(|e| e.causes.contains(&partial_source)));
        assert!(c.agents[actor as usize].knowledge.contains(&4));
        let learned = c.agents[actor as usize].knowledge_sources[&4];
        assert!(h.events[learned as usize].causes.contains(&source));
        assert!(h.events[learned as usize]
            .subjects
            .contains(&("person".into(), teacher)));
        assert!(h.events[learned as usize]
            .subjects
            .contains(&("institution".into(), institution)));
        // The original teacher's death no longer removes the student's practical knowledge.
        h.people[teacher as usize].died = Some(h.month);
        // An accessible text can reintroduce a technique when no teacher survives.
        let mut books = c.clone();
        books.agents[actor as usize].knowledge.clear();
        books.agents[actor as usize].knowledge_sources.clear();
        let book = books
            .artifacts
            .iter()
            .position(|a| a.site == Some(site))
            .unwrap();
        books.artifacts[book].topic = Some(4); // Declared manuscript-content fixture.
        books.artifacts[book].destroyed = true;
        books.decisions(h);
        assert!(!books.agents[actor as usize].knowledge.contains(&4));
        books.artifacts[book].destroyed = false;
        books.artifacts[book].lost = false;
        books.decisions(h);
        assert!(!books.agents[actor as usize].knowledge.contains(&4));
        books.decisions(h);
        books.decisions(h);
        assert!(books.agents[actor as usize].knowledge.contains(&4));
        let reading = books.agents[actor as usize].knowledge_sources[&4];
        assert!(h.events[reading as usize]
            .subjects
            .contains(&("artifact".into(), books.artifacts[book].id)));
        h.culture = Some(c);
        h.reserve_cultural_work();
        assert_eq!(h.sites[site as usize].economy.management[3] as u32, 1 << 4);
        let c = h.culture.as_ref().unwrap();
        let restored: Culture = serde_json::from_value(serde_json::to_value(c).unwrap()).unwrap();
        assert_eq!(
            restored.available_knowledge(h, site),
            c.available_knowledge(h, site)
        );
        assert_eq!(
            restored.agents[actor as usize].knowledge_sources[&4],
            learned
        );
    }
    #[test]
    #[ignore = "requires a hardware GPU"]
    fn office_campaign_requires_support_and_preserves_money() {
        let mut g = world();
        g.enable_politics().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        let (site, actor) = h
            .sites
            .iter()
            .find_map(|site| {
                c.site_people(h, site.id)
                    .into_iter()
                    .find(|&p| {
                        h.civilizations[h.people[p as usize].civilization as usize].leader != p
                    })
                    .map(|p| (site.id, p))
            })
            .unwrap();
        c.agents[actor as usize].relations.clear();
        assert!(!c.seek_office(h, site, actor));
        let civ = h.people[actor as usize].civilization as usize;
        c.agents[actor as usize]
            .relations
            .insert(h.civilizations[civ].leader, 0.5);
        let private = h.sites[site as usize].economy.finance[0];
        let treasury = h.society.as_ref().unwrap().councils[civ].treasury;
        assert!(private >= 202.);
        assert!(c.seek_office(h, site, actor));
        assert_eq!(h.sites[site as usize].economy.finance[0], private - 2.);
        assert_eq!(
            h.society.as_ref().unwrap().councils[civ].treasury,
            treasury + 2.
        );
        let event = c.agents[actor as usize].last_campaign.unwrap();
        assert_eq!(h.events[event as usize].kind, "office_campaign");
        assert_ne!(h.civilizations[civ].leader, actor);
        // A paid campaign can now win an explicit local post when candidates otherwise tie.
        for a in &mut c.agents {
            a.traits.fill(0.);
            a.knowledge = [0, 1].into_iter().collect();
            a.knowledge_sources.retain(|topic, _| *topic < 2);
        }
        h.culture = Some(c);
        h.sync_culture();
        g.enable_governance().unwrap();
        g.enable_offices().unwrap();
        let h = g.civilizations.as_ref().unwrap();
        let seat = &h.offices.as_ref().unwrap().seats[site as usize];
        assert_eq!(seat.holder(), Some(actor));
        assert!(h.events[seat.tenures.last().unwrap().appointment as usize]
            .causes
            .contains(&event));
    }
    #[test]
    #[ignore = "requires a hardware GPU"]
    fn annual_roles_measure_new_activity_and_retain_inertia() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        let mut c = h.culture.take().unwrap();
        h.month = 12;
        h.sites[0].economy.agriculture[0] = 2000.;
        c.update_roles(h);
        assert_eq!(c.roles[0].scores[0], 1.);
        h.month = 24;
        c.update_roles(h);
        assert!((c.roles[0].scores[0] - 0.8).abs() < 1e-6);
        h.month = 36;
        h.sites[0].economy.agriculture[0] = 4000.;
        c.update_roles(h);
        assert!((c.roles[0].scores[0] - 0.84).abs() < 1e-6);
    }
    #[test]
    #[ignore = "requires a hardware GPU"]
    fn minority_congregations_need_adherents_and_finite_founding_resources() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 3;
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        let site = 0;
        let people = c.site_people(h, site);
        assert!(people.len() >= 3);
        let actor = people[((h.month / 3 + site) as usize) % people.len()];
        let companion = *people.iter().find(|&&p| p != actor).unwrap();
        let majority = c.site_faith[site as usize];
        let minority = (majority + 1) % c.traditions.len() as u32;
        for &p in &people {
            c.agents[p as usize].traits = [0.; 6];
        }
        c.agents[actor as usize].traits[2] = 0.68; // Religious founding; below pilgrimage threshold.
        for hh in &h.society.as_ref().unwrap().households {
            if hh.site == site {
                c.household_faith[hh.id as usize] = majority;
            }
        }
        // Declared construction stock; both branches start with the same inventory.
        h.sites[site as usize].economy.goods[0] += 1000.;
        h.sites[site as usize].economy.initial[0] += 1000.;
        h.sites[site as usize].economy.goods[5] += 2. * crate::institution_capacity::HALL_BRICKS_KG;
        h.sites[site as usize].economy.initial[5] +=
            2. * crate::institution_capacity::HALL_BRICKS_KG;
        assert!(h.sites[site as usize].economy.finance[0] > 550.);
        c.labor_budget = vec![0.; h.sites.len()];
        c.labor_budget[site as usize] = 0.5;
        c.decisions(h);
        let majority_order = c
            .institutions
            .iter()
            .find(|n| {
                n.site == site
                    && n.kind == InstitutionKind::Religious
                    && n.tradition == Some(majority)
            })
            .unwrap()
            .id;
        // A single minority representative cannot form a congregation alone.
        let actor_household = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.head == actor)
            .unwrap()
            .id;
        c.household_faith[actor_household as usize] = minority;
        let mut alone = c.clone();
        let mut alone_h = h.clone();
        alone.decisions(&mut alone_h);
        assert!(!alone
            .institutions
            .iter()
            .any(|n| n.tradition == Some(minority)));
        let companion_household = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.head == companion)
            .unwrap()
            .id;
        c.household_faith[companion_household as usize] = minority;
        let mut poor = c.clone();
        let mut poor_h = h.clone();
        for good in [0, 2, 5, 50] {
            poor_h.sites[site as usize].economy.goods[good] = 0.;
        }
        poor.decisions(&mut poor_h);
        assert!(!poor
            .institutions
            .iter()
            .any(|n| n.tradition == Some(minority)));
        let faiths = c.household_faith.clone();
        let stocks = h.sites[site as usize].economy.goods;
        let cash = h.sites[site as usize].economy.finance[0] as f64
            + c.institutions.iter().map(|n| n.treasury).sum::<f64>();
        c.decisions(h);
        let order = c
            .institutions
            .iter()
            .find(|n| n.tradition == Some(minority))
            .unwrap();
        assert_eq!(
            order.operational(),
            order
                .capacity
                .as_ref()
                .unwrap()
                .building
                .as_ref()
                .unwrap()
                .construction_remaining
                == 0.
        );
        assert!(order
            .capacity
            .as_ref()
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .facility
            .is_some());
        assert_eq!(order.members.len(), 2);
        assert!(order.members.contains(&actor) && order.members.contains(&companion));
        for &(good, mass) in &c.artifacts[order.property[0] as usize].materials {
            assert!(
                (stocks[good as usize]
                    - h.sites[site as usize].economy.goods[good as usize]
                    - mass)
                    .abs()
                    < 0.001
            );
        }
        assert!(
            (cash
                - h.sites[site as usize].economy.finance[0] as f64
                - c.institutions.iter().map(|n| n.treasury).sum::<f64>())
            .abs()
                < 0.001
        );
        assert_eq!(c.household_faith, faiths);
        assert_eq!(c.site_faith[site as usize], majority);
        assert!(c.institutions[majority_order as usize].active);
        let id = order.id;
        let foundation = order.property[0];
        assert_eq!(c.artifacts[foundation as usize].tradition, Some(minority));
        c.decisions(h);
        assert_eq!(
            c.institutions
                .iter()
                .filter(|n| n.tradition == Some(minority))
                .count(),
            1
        );
        let mut resumed: Culture =
            serde_json::from_value(serde_json::to_value(&c).unwrap()).unwrap();
        let mut resumed_h = h.clone();
        h.month = 12;
        resumed_h.month = 12;
        c.year(h);
        resumed.year(&mut resumed_h);
        assert_eq!(c.institutions[id as usize].members.len(), 2);
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::to_value(resumed).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(resumed_h).unwrap()
        );
    }

    #[test]
    #[ignore = "requires a hardware GPU"]
    fn pilgrimage_and_recovery_use_real_routes_work_and_stocks() {
        let mut g = world();
        let cells = g.snapshot().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        let route = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .filter(|r| r.open)
            .min_by(|a, b| a.cost_km.total_cmp(&b.cost_km))
            .unwrap()
            .clone();
        assert!(
            route.cost_km <= 600.,
            "fixture needs a within-month caravan route: {}",
            route.cost_km
        );
        let site = route.from;
        let destination = route.to;
        h.sites[site as usize].economy.goods[7] += 1.;
        h.sites[site as usize].economy.initial[7] += 1.;
        let before = h.economy_residuals();
        let food = h.food_residual();
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        let actor = c.site_people(h, site)[0];
        let majority = c.site_faith[site as usize];
        let faith = (majority + 1) % c.traditions.len() as u32;
        let household = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == site && hh.head == actor)
            .unwrap()
            .id;
        c.household_faith[household as usize] = faith;
        c.traditions[majority as usize].sacred_site = site;
        c.traditions[faith as usize].sacred_site = destination;
        assert_eq!(c.resident_tradition(h, site, actor), Some(faith));
        let teacher = c.site_people(h, destination)[0] as usize;
        c.agents[teacher].knowledge = BTreeSet::from([11]);
        c.agents[actor as usize].knowledge.remove(&11);
        c.agents[actor as usize].studies.remove(&11);
        c.agents[actor as usize].last_learning_exposure = None;
        let affiliations = c.household_faith.clone();
        assert!(!c.pilgrimage(h, site, actor, 0.));
        let food_before = h.sites[site as usize].stocks.stock[1];
        let offerings_before = h.sites[site as usize].economy.goods[7];
        h.society.as_mut().unwrap().routes[route.id as usize].flood_months = 2;
        assert!(!c.pilgrimage(h, site, actor, 1.));
        assert_eq!(h.sites[site as usize].stocks.stock[1], food_before);
        assert_eq!(h.sites[site as usize].economy.goods[7], offerings_before);
        h.society.as_mut().unwrap().routes[route.id as usize].flood_months = 0;
        assert!(c.pilgrimage(h, site, actor, 1.));
        assert!(!c.agents[actor as usize].knowledge.contains(&11));
        assert!(c.agents[actor as usize].studies[&11].progress > 0.);
        assert!(c.agents[actor as usize].studies[&11].source.is_some());

        assert_eq!(c.site_faith[site as usize], majority);
        assert_eq!(c.household_faith, affiliations);
        assert!(h.events.iter().any(|e| e.kind == "pilgrimage_returned"
            && e.subjects.contains(&("tradition".into(), faith))));
        assert_eq!(c.accounts.last().unwrap().tradition, faith);
        assert!(c.agents[actor as usize]
            .known_places
            .contains(&h.sites[destination as usize].cell));
        let artifact = c
            .artifacts
            .iter()
            .position(|a| a.site == Some(site))
            .unwrap();
        h.sites[site as usize].abandoned = true;
        c.abandon_objects(h);
        assert!(c.artifacts[artifact].lost);
        h.sites[site as usize].abandoned = false;
        let count = c.artifacts.len();
        assert!(c.recover_object(h, site, actor));
        assert!(!c.artifacts[artifact].lost);
        assert_eq!(c.artifacts.len(), count);
        h.culture = Some(c);
        assert!((food - h.food_residual()).abs() < 1e-6);
        for (a, b) in before.into_iter().zip(h.economy_residuals()) {
            assert!((a - b).abs() < 1e-6);
        }
        h.validate(&cells).unwrap();
    }
    #[test]
    #[ignore = "requires a hardware GPU"]
    fn curation_and_destruction_do_not_duplicate_specimen_matter() {
        let mut g = world();
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        g.enable_shipping().unwrap();
        g.enable_expeditions().unwrap();
        g.enable_discoveries().unwrap();
        let cells = g.snapshot().unwrap();
        let source = cells
            .iter()
            .position(|c| c.meta[0] == 3 && c.water[0] < 0.25)
            .unwrap() as u32;
        let h = g.civilizations.as_mut().unwrap();
        h.event(
            "specimens_delivered",
            Some(0),
            None,
            "Declared ten-kg specimen fixture".into(),
        );
        let event = h.events.last().unwrap().id;
        let d = h
            .expeditions
            .as_mut()
            .unwrap()
            .discoveries
            .as_mut()
            .unwrap();
        d.sources.push(crate::discoveries::Source {
            cell: source,
            initial: [10., 0.],
            remaining: [0., 0.],
            collected: [10., 0.],
        });
        d.collected = [10., 0.];
        d.workshops.push(crate::discoveries::Workshop {
            botanicals: Default::default(),
            work_plan: None,
            site: 0,
            enabled: true,
            processed: [0.; 2],
            curated: [0.; 2],
            samples: [10., 0.],
            studied: [0.; 2],
            learned: [None; 2],
            remedy: 0.,
            delivered: [10., 0.],
            causes: [Some(event), None],
            batches: [0; 2],
        });
        for (k, ratio) in crate::discoveries::CNP[0].into_iter().enumerate() {
            h.sites[0].economy.external[k] += (10. * ratio) as f32;
        }
        let before = h.economy_residuals();
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        assert!(c.curate_specimen(h, 0, 0));
        let id = c.artifacts.last().unwrap().id;
        assert!(!c.curate_specimen(h, 0, 0));
        h.culture = Some(c);
        h.validate(&cells).unwrap();
        for (a, b) in before.into_iter().zip(h.economy_residuals()) {
            assert!((a - b).abs() < 1e-6);
        }
        g.destroy_artifact(id).unwrap();
        let h = g.civilizations.as_ref().unwrap();
        h.validate(&cells).unwrap();
        for (a, b) in before.into_iter().zip(h.economy_residuals()) {
            assert!((a - b).abs() < 1e-6);
        }
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn regional_recovery_is_a_paid_canonical_action_and_resumes_exactly() {
        let mut g = world();
        let h = g.civilizations.as_mut().unwrap();
        let mut c = h.culture.take().unwrap();
        c.sync(h);
        let site = 0;
        let actor = c.site_people(h, site)[0];
        let artifact = c
            .artifacts
            .iter()
            .find(|a| a.site == Some(site))
            .unwrap()
            .id;
        h.sites[0].abandoned = true;
        c.abandon_objects(h);
        h.sites[0].abandoned = false;
        let original = c.artifacts[artifact as usize].clone();
        h.culture = Some(c);
        let cell = h.sites[0].cell;
        let region = g
            .generate_region(
                crate::grid::cell_direction(cell, g.config.resolution),
                10.,
                16,
            )
            .unwrap();
        assert!(region.historical_sites.iter().any(|s| s.id == site));
        assert!(region
            .historical_artifacts
            .iter()
            .any(|a| a.id == artifact && a.lost));
        g.request_local_recovery(&region, site, actor, artifact)
            .unwrap();
        let before = serde_json::to_vec(&g.civilizations).unwrap();
        assert!(g
            .request_local_recovery(&region, site, actor, artifact)
            .is_err());
        assert_eq!(before, serde_json::to_vec(&g.civilizations).unwrap());
        let mut unpaid = g.civilizations.as_ref().unwrap().clone();
        let mut c = unpaid.culture.take().unwrap();
        c.labor_budget.fill(0.);
        let work = c.labor_spent;
        assert!(c.process_local_recoveries(&mut unpaid).is_empty());
        assert_eq!(c.local_recoveries.len(), 1);
        assert!(c.artifacts[artifact as usize].lost);
        assert_eq!(c.labor_spent, work);
        // Death cancels the request rather than inventing a replacement worker.
        unpaid.people[actor as usize].died = Some(unpaid.month);
        assert!(c.process_local_recoveries(&mut unpaid).is_empty());
        assert!(c.local_recoveries.is_empty());
        assert!(c.artifacts[artifact as usize].lost);
        assert_eq!(
            unpaid.events.last().unwrap().kind,
            "local_recovery_cancelled"
        );
        let path =
            std::env::temp_dir().join(format!("local-recovery-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        g.advance_history(3).unwrap();
        for _ in 0..3 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_vec(&g.civilizations).unwrap(),
            serde_json::to_vec(&resumed.civilizations).unwrap()
        );
        let h = g.civilizations.as_ref().unwrap();
        let c = h.culture.as_ref().unwrap();
        let a = &c.artifacts[artifact as usize];
        assert!(!a.lost);
        assert_eq!(a.owner, original.owner);
        assert_eq!(a.claims, original.claims);
        assert_eq!(a.materials, original.materials);
        assert_eq!(a.custodian, Some(actor));
        assert!(c.local_recoveries.is_empty());
        assert!(c.labor_spent > work);
        let event = &h.events[*a.events.last().unwrap() as usize];
        assert_eq!(event.kind, "artifact_recovered");
        assert!(event
            .causes
            .iter()
            .any(|&id| h.events[id as usize].kind == "local_recovery_requested"));
        assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
        let again = g
            .generate_region(
                crate::grid::cell_direction(cell, g.config.resolution),
                10.,
                16,
            )
            .unwrap();
        assert!(
            !again
                .historical_artifacts
                .iter()
                .find(|a| a.id == artifact)
                .unwrap()
                .lost
        );
        assert!(g
            .request_local_recovery(&region, site, actor, artifact)
            .is_err());
    }
}
