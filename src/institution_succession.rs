//! Local institutional mandates. Membership is an affiliation; authority requires presence.
use crate::{
    civilization::History,
    culture::{Culture, InstitutionKind},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mandate {
    pub holder: Option<u32>,
    pub since: u32,
    pub vacant_since: Option<u32>,
    pub observed: u32,
    pub contested: bool,
    pub support: f32,
    pub work: f64,
    pub events: Vec<u64>,
}

// Scores are decision proxies, not measurements of historical electoral behavior.
fn competence(kind: &InstitutionKind, a: &crate::culture::Agent) -> f32 {
    match kind {
        InstitutionKind::Religious => a.traits[2],
        InstitutionKind::Scholarly => (a.traits[3] + a.knowledge.len() as f32 / 12.) * 0.5,
        InstitutionKind::Craft => a.skills[3],
        InstitutionKind::Merchant => (a.skills[1] + a.traits[4]) * 0.5,
    }
    .clamp(0., 1.)
}

/// One vote per present adult representative, stable ID tie breaking.
fn ballot(
    agents: &[crate::culture::Agent],
    kind: &InstitutionKind,
    electorate: &[u32],
) -> Vec<(u32, usize)> {
    let mut votes: Vec<_> = electorate.iter().map(|&id| (id, 0)).collect();
    for &voter in electorate {
        let best = electorate
            .iter()
            .copied()
            .max_by(|&a, &b| {
                let score = |id| {
                    let candidate = &agents[id as usize];
                    let affinity = agents[voter as usize]
                        .relations
                        .get(&id)
                        .copied()
                        .unwrap_or(0.)
                        .clamp(-1., 1.);
                    0.6 * competence(kind, candidate)
                        + 0.3 * affinity
                        + if id == voter {
                            0.1 * candidate.traits[0]
                        } else {
                            0.
                        }
                };
                score(a).total_cmp(&score(b)).then_with(|| b.cmp(&a))
            })
            .unwrap();
        votes.iter_mut().find(|(id, _)| *id == best).unwrap().1 += 1;
    }
    votes.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    votes
}

impl Culture {
    /// Present adult members eligible for this institution's local mandate.
    pub fn institution_candidates(&self, h: &History, id: u32) -> Vec<u32> {
        let Some(n) = self.institutions.get(id as usize) else {
            return vec![];
        };
        let mut people: Vec<_> = self
            .site_people(h, n.site)
            .into_iter()
            .filter(|p| n.members.contains(p) && (*p as usize) < self.agents.len())
            .filter(|&p| {
                n.kind != InstitutionKind::Religious
                    || self.resident_tradition(h, n.site, p) == n.tradition
            })
            .collect();
        people.sort_unstable();
        people.dedup();
        people
    }

    pub(crate) fn institutional_succession(&mut self, h: &mut History) {
        for i in 0..self.institutions.len() {
            let n = &self.institutions[i];
            if !n.active || n.capacity.is_none() {
                continue;
            }
            let site = n.site;
            let electorate = self.institution_candidates(h, n.id);
            let initial_holder = electorate.contains(&n.leader).then_some(n.leader);
            let capacity = self.institutions[i].capacity.as_mut().unwrap();
            if capacity.mandate.is_none() {
                // Observation baseline, not a fabricated historical election.
                capacity.mandate = Some(Mandate {
                    holder: initial_holder,
                    since: h.month,
                    vacant_since: initial_holder.is_none().then_some(h.month),
                    observed: h.month,
                    contested: false,
                    support: 0.,
                    work: 0.,
                    events: vec![],
                });
                continue;
            }
            let m = capacity.mandate.as_mut().unwrap();
            if m.observed >= h.month {
                continue;
            }
            m.observed = h.month;
            if let Some(holder) = m.holder {
                if electorate.contains(&holder) {
                    continue;
                }
                m.holder = None;
                m.vacant_since = Some(h.month);
                m.contested = false;
                m.support = 0.;
                self.succession_event(h, i, "institution_vacant", Some(holder),
                    "The institutional mandate became vacant: its holder is no longer an eligible local member".into());
                continue;
            }
            if h.month % 3 != 0 || electorate.is_empty() || h.sites[site as usize].abandoned {
                continue;
            }
            let votes = ballot(&self.agents, &self.institutions[i].kind, &electorate);
            let (winner, count) = votes[0];
            let share = count as f32 / electorate.len() as f32;
            let budget = self.labor_budget.get_mut(site as usize);
            let Some(budget) = budget.filter(|b| **b >= 0.05) else {
                continue;
            };
            let before = *budget;
            *budget -= 0.05;
            let work = (before - *budget) as f64;
            self.labor_spent += work;
            let m = self.institutions[i]
                .capacity
                .as_mut()
                .unwrap()
                .mandate
                .as_mut()
                .unwrap();
            m.support = share;
            m.work += work;
            // A divided first ballot requires another paid quarterly deliberation.
            if count * 2 <= electorate.len() && !m.contested {
                m.contested = true;
                self.succession_event(h, i, "institution_succession_contested", Some(winner),
                    format!("No majority: leading candidate received {count}/{} local representative votes; {work:.4} worker-months spent; a runoff awaits another quarter", electorate.len()));
                continue;
            }
            let n = &mut self.institutions[i];
            n.leader = winner;
            let m = n.capacity.as_mut().unwrap().mandate.as_mut().unwrap();
            m.holder = Some(winner);
            m.since = h.month;
            m.vacant_since = None;
            m.contested = false;
            self.succession_event(h, i, "institution_successor_selected", Some(winner),
                format!("Local members selected {} with {count}/{} votes; {work:.4} worker-months spent convening succession", h.people[winner as usize].name, electorate.len()));
        }
    }

    fn succession_event(
        &mut self,
        h: &mut History,
        i: usize,
        kind: &str,
        person: Option<u32>,
        text: String,
    ) {
        let n = &mut self.institutions[i];
        let m = n.capacity.as_mut().unwrap().mandate.as_mut().unwrap();
        let cause = m.events.last().copied().or_else(|| {
            h.events
                .iter()
                .rev()
                .find(|e| e.subjects.contains(&("institution".into(), n.id)))
                .map(|e| e.id)
        });
        h.event(kind, Some(n.site), None, text);
        let e = h.events.last_mut().unwrap();
        e.subjects.push(("institution".into(), n.id));
        if let Some(p) = person {
            e.subjects.push(("person".into(), p));
        }
        if let Some(cause) = cause {
            e.causes.push(cause);
        }
        m.events.push(e.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn votes_follow_competence_affinity_and_stable_ids() {
        let mut agents: Vec<_> = (0..2)
            .map(|person| crate::culture::Agent {
                person,
                traits: [0.5; 6],
                skills: [0.1; 4],
                occupation: "teacher".into(),
                goal: String::new(),
                knowledge: Default::default(),
                known_places: Default::default(),
                knowledge_sources: Default::default(),
                last_campaign: None,
                relations: Default::default(),
                actions: 0,
            })
            .collect();
        let kind = InstitutionKind::Scholarly;
        assert!(ballot(&agents, &kind, &[]).is_empty());
        assert_eq!(ballot(&agents, &kind, &[1, 0]), vec![(0, 1), (1, 1)]);
        agents[1].traits[3] = 1.;
        agents[1].knowledge.extend(0..12);
        assert_eq!(ballot(&agents, &kind, &[0, 1]), vec![(1, 2), (0, 0)]);
        agents[0].traits[3] = 1.;
        agents[0].knowledge.extend(0..12);
        agents[0].relations.insert(1, -1.);
        agents[1].relations.insert(0, 1.);
        assert_eq!(ballot(&agents, &kind, &[1, 0]), vec![(0, 2), (1, 0)]);
        // Craft qualification must use the actual practiced craftsmanship slot.
        agents[0].skills[3] = 0.9;
        assert_eq!(competence(&InstitutionKind::Craft, &agents[0]), 0.9);
    }

    #[test]
    fn older_capacity_has_no_fabricated_mandate() {
        let c: crate::institution_capacity::Capacity = serde_json::from_str(
            r#"{"readiness":0.5,"paid":0,"work":0,"observed":0,"impaired":false}"#,
        )
        .unwrap();
        assert!(c.mandate.is_none());
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn vacancies_ballots_work_and_serialized_continuation() {
        use crate::{
            catalog::Catalog,
            config::Config,
            culture::Institution,
            gpu::{ContextGpu, Generator},
            institution_capacity::Capacity,
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
        h.sync_culture();
        let mut c = h.culture.take().unwrap();
        let members: Vec<_> = c.site_people(h, 0).into_iter().take(3).collect();
        assert_eq!(members.len(), 3);
        for &p in &members {
            let a = &mut c.agents[p as usize];
            a.traits = [0.5; 6];
            a.skills = [0.1; 4];
            a.relations.clear();
            a.knowledge.clear();
        }
        h.sites[0].economy.finance[0] -= 25.;
        c.institutions.push(Institution {
            id: 0,
            name: "Succession school".into(),
            kind: InstitutionKind::Scholarly,
            site: 0,
            tradition: None,
            members: members.clone(),
            leader: members[0],
            treasury: 25.,
            active: true,
            founded: 0,
            knowledge: Default::default(),
            property: vec![],
            dues: 25.,
            expenses: 0.,
            capacity: Some(Capacity::new(0)),
        });
        c.labor_budget = vec![1.; h.sites.len()];
        c.institutional_succession(h);
        assert!(c.institutions[0].operational());
        let money = h.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury;
        // Declared attendance intervention: founder has finished moving to another town.
        for hh in &mut h.society.as_mut().unwrap().households {
            if hh.head == members[0] {
                hh.site = 1;
            }
        }
        h.month = 1;
        c.institutional_succession(h);
        assert!(!c.institutions[0].operational());
        assert_eq!(h.events.last().unwrap().kind, "institution_vacant");
        assert!(c.institutions[0].members.contains(&members[0]));
        h.month = 3;
        c.labor_budget[0] = 0.;
        c.institutional_succession(h);
        assert_eq!(h.events.last().unwrap().kind, "institution_vacant");
        h.month = 6;
        c.labor_budget[0] = 0.2;
        let start = c.labor_budget[0];
        c.institutional_succession(h);
        assert_eq!(
            h.events.last().unwrap().kind,
            "institution_succession_contested"
        );
        assert!(!c.institutions[0].operational());
        assert!((start - c.labor_budget[0] - 0.05).abs() < 1e-7);
        let unchanged = serde_json::to_value(&c).unwrap();
        c.institutional_succession(h);
        assert_eq!(unchanged, serde_json::to_value(&c).unwrap());
        let mut resumed: Culture =
            serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
        let mut resumed_h = h.clone();
        h.month = 9;
        resumed_h.month = 9;
        c.institutional_succession(h);
        resumed.institutional_succession(&mut resumed_h);
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&*h).unwrap(),
            serde_json::to_value(&resumed_h).unwrap()
        );
        assert!(c.institutions[0].operational());
        assert_eq!(c.institutions[0].leader, members[1].min(members[2]));
        let m = c.institutions[0]
            .capacity
            .as_ref()
            .unwrap()
            .mandate
            .as_ref()
            .unwrap();
        assert!((m.work - 0.1).abs() < 1e-7);
        assert!((c.labor_spent - m.work).abs() < 1e-9);
        assert_eq!(
            money,
            h.sites[0].economy.finance[0] as f64 + c.institutions[0].treasury
        );
        assert_eq!(m.events.len(), 3);
        assert_eq!(h.events[m.events[2] as usize].causes, vec![m.events[1]]);
        // Competence and relationships alter the same electorate's choice.
        let voters = vec![members[1], members[2]];
        c.agents[members[2] as usize].traits[3] = 1.;
        c.agents[members[2] as usize].knowledge.extend(0..12);
        assert_eq!(
            ballot(&c.agents, &InstitutionKind::Scholarly, &voters)[0],
            (members[2], 2)
        );
        c.agents[members[1] as usize]
            .relations
            .insert(members[2], -1.);
        c.agents[members[1] as usize].traits[3] = 1.;
        c.agents[members[1] as usize].knowledge.extend(0..12);
        c.agents[members[2] as usize]
            .relations
            .insert(members[1], 1.);
        assert_eq!(
            ballot(&c.agents, &InstitutionKind::Scholarly, &voters)[0],
            (members[1], 2)
        );
    }
}
