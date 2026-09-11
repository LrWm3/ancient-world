//! Modest, finite archaeological finds; interpretations never resolve the founding mystery.
use crate::{
    civilization::History,
    culture::{Account, Artifact, Owner},
    expeditions::{Expedition, Objective, Phase},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Charter {
    pub tradition: u32,
    pub patron: Option<u32>,
    pub motive: String,
    pub find: Option<Find>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Find {
    pub cell: u32,
    pub description: String,
    pub observed: u64,
    pub artifact: Option<u32>,
    #[serde(default)]
    pub studies: Vec<Study>,
}
pub fn charter(h: &History, origin: u32, objective: Objective) -> Result<Option<Charter>> {
    if !matches!(
        objective,
        Objective::PatronSearch | Objective::Inscriptions | Objective::OldLiterature
    ) {
        return Ok(None);
    }
    let c = h
        .culture
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("cultural history required"))?;
    let tradition = c.site_faith[origin as usize];
    let t = &c.traditions[tradition as usize];
    let patron = t.patron;
    let motive=match objective {
        Objective::PatronSearch=> {
            let p=patron.and_then(|id|c.patrons.get(id as usize)).ok_or_else(||anyhow::anyhow!("tradition has no recorded patron"))?;
            ensure!(p.departed.is_some(),"patron has not yet departed");
            format!("Seek traces associated with {} and commemorate the founding journey; no reunion or revelation promised",p.name)
        },
        Objective::Inscriptions=>"Survey weathered inscriptions and compare their marks with inherited founding accounts".into(),
        _=>"Seek fragments of old verse, household records and devotional writing; preserve uncertain readings".into(),
    };
    Ok(Some(Charter {
        tradition,
        patron,
        motive,
        find: None,
    }))
}
pub(crate) fn survey(h: &mut History, e: &mut Expedition, cell: u32, already: bool) {
    let survivors = e.survivors();
    let Some(c) = &mut e.heritage else { return };
    if already || c.find.is_some() || e.field_months != 1 || survivors == 0 {
        return;
    }
    // One small accessible ceramic fragment per qualifying endpoint, across all objectives.
    let hash = cell
        .wrapping_mul(747796405)
        .wrapping_add(h.seed.wrapping_mul(2891336453));
    if (hash ^ (hash >> 16)) % 5 >= 3 {
        return;
    }
    let description=match e.objective {
        Objective::PatronSearch=>"A worn ceramic charm bears an animal-like outline. Its resemblance to the patron is suggestive, not identification; no living patron was encountered.",
        Objective::Inscriptions=>"A broken ceramic plaque bears repeated marks and possible tally strokes. Dating, language and authorship remain uncertain.",
        _=>"A small fired-clay text fragment preserves repeated signs interpreted provisionally as a household list or refrain; most of the text is missing.",
    }.to_string();
    h.event(
        "heritage_fragment_observed",
        Some(e.origin),
        None,
        description.clone(),
    );
    let event = h.events.last_mut().unwrap();
    event.causes.push(e.cause);
    event
        .spatial
        .as_mut()
        .unwrap()
        .push(crate::spatial::EventAnchor {
            cell,
            role: crate::spatial::EventRole::Milestone,
        });
    c.find = Some(Find {
        cell,
        description,
        observed: event.id,
        artifact: None,
        studies: vec![],
    });
}
pub(crate) fn deliver(h: &mut History, e: &mut Expedition) {
    if e.phase != Phase::Returned || e.survivors() == 0 {
        return;
    }
    let Some(charter) = &mut e.heritage else {
        return;
    };
    let Some(find) = &mut charter.find else {
        return;
    };
    if find.artifact.is_some() {
        return;
    }
    let Some(good) = h
        .economy_catalog
        .as_ref()
        .and_then(|c| c.goods.iter().position(|g| g.id == "pottery"))
    else {
        return;
    };
    let Some(c) = &mut h.culture else { return };
    let id = c.artifacts.len() as u32;
    let mass = 0.125;
    // Explicit finite import from the external archaeological cache, once at port.
    h.sites[e.origin as usize].economy.initial[good] += mass;
    let composition = h.economy_catalog.as_ref().unwrap().composition(good);
    for (k, ratio) in composition.into_iter().enumerate() {
        h.sites[e.origin as usize].economy.external[k] += mass * ratio;
    }
    let owner = e
        .institution
        .map_or(Owner::Community(e.origin), Owner::Institution);
    let event_id = h.events.len() as u64;
    c.artifacts.push(Artifact {
        id,
        name: h.civilizations[h.sites[e.origin as usize].civilization as usize]
            .naming(h.seed)
            .coin(
                &format!("artifact:{id}"),
                &["clay", "memory"],
                Some(crate::naming::Source {
                    kind: "expedition".into(),
                    id: e.id,
                    name: format!("{} voyage {}", h.sites[e.origin as usize].name, e.id),
                }),
            ),
        kind: "ancient ceramic fragment".into(),
        creator: None,
        owner,
        claims: vec![],
        site: Some(e.origin),
        custodian: None,
        materials: vec![(good as u32, mass)],
        topic: None,
        tradition: Some(charter.tradition),
        events: vec![find.observed, event_id],
        destroyed: false,
        lost: false,
    });
    if let Some(n) = e.institution {
        c.institutions[n as usize].property.push(id);
    }
    find.artifact = Some(id);
    let author = c.traditions[charter.tradition as usize].leader;
    if h.people[author as usize].died.is_none() {
        c.accounts.push(Account{id:c.accounts.len() as u32,tradition:charter.tradition,author:Some(author),institution:e.institution,month:h.month,facts:vec![find.observed,event_id],text:format!("Our interpreter considers the fragment relevant to our remembered origins. This remains a disputed reading, not proof of our patron's identity or mission. {}",find.description)});
    }
    h.event("heritage_fragment_received",Some(e.origin),None,format!("Voyage {} delivered one 0.125 kg ancient ceramic fragment from cell {}; finite external material import, uncertain provenance, no new technology or supernatural effect",e.id,find.cell));
    let event = h.events.last_mut().unwrap();
    event.causes.extend([e.cause, find.observed]);
    event.subjects.extend([
        ("artifact".into(), id),
        ("tradition".into(), charter.tradition),
    ]);
}
pub(crate) fn validate(h: &History, voyages: &[Expedition]) -> Result<()> {
    let mut cells = std::collections::BTreeSet::new();
    for e in voyages {
        if let Some(c) = &e.heritage {
            let culture = h
                .culture
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("heritage without culture"))?;
            ensure!(
                (c.tradition as usize) < culture.traditions.len()
                    && c.patron
                        .is_none_or(|p| (p as usize) < culture.patrons.len()),
                "invalid heritage charter"
            );
            if let Some(f) = &c.find {
                ensure!(
                    f.studies.len() <= 3
                        && f.studies.windows(2).all(|w| w[1].month >= w[0].month + 60),
                    "invalid heritage study schedule"
                );
                for s in &f.studies {
                    ensure!(
                        s.month <= h.month
                            && (s.site as usize) < h.sites.len()
                            && (s.author as usize) < h.people.len()
                            && s.comparison
                                .is_none_or(|a| (a as usize) < culture.artifacts.len())
                            && h.events.get(s.event as usize).is_some_and(|e| e.kind
                                == "heritage_interpreted"
                                && e.month == s.month),
                        "invalid heritage study"
                    );
                }
                ensure!(
                    f.cell < 6 * h.terrain_resolution * h.terrain_resolution
                        && cells.insert(f.cell)
                        && h.events
                            .get(f.observed as usize)
                            .is_some_and(|v| v.kind == "heritage_fragment_observed")
                        && f.artifact.is_none_or(|a| e.phase == Phase::Returned
                            && culture
                                .artifacts
                                .get(a as usize)
                                .is_some_and(|a| a.kind == "ancient ceramic fragment")),
                    "invalid or duplicate heritage recovery"
                );
            }
        }
    }
    Ok(())
}

/// Re-examine an accessible object; the resulting reading is an attributed account.
pub(crate) fn study(h: &mut History, c: &mut crate::culture::Culture) {
    if !h.month.is_multiple_of(12) {
        return;
    }
    let Some(mut x) = h.expeditions.take() else {
        return;
    };
    for voyage in &mut x.voyages {
        let Some(charter) = &mut voyage.heritage else {
            continue;
        };
        let Some(find) = &mut charter.find else {
            continue;
        };
        if find.studies.len() >= 3 || find.studies.last().is_some_and(|s| h.month < s.month + 60) {
            continue;
        }
        let Some(id) = find.artifact else { continue };
        let a = &c.artifacts[id as usize];
        let Some(site) = a.site.filter(|_| !a.lost && !a.destroyed) else {
            continue;
        };
        if !c.work_allowed(site, "heritage study")
            || h.sites[site as usize].abandoned
            || c.labor_budget.get(site as usize).copied().unwrap_or(0.) < 0.1
        {
            continue;
        }
        let Some(n) = c.institutions.iter().find(|n| {
            n.site == site
                && n.operational()
                && matches!(
                    n.kind,
                    crate::culture::InstitutionKind::Scholarly
                        | crate::culture::InstitutionKind::Religious
                )
                && h.people[n.leader as usize].died.is_none()
        }) else {
            continue;
        };
        let (institution, author) = (n.id, n.leader);
        let Some(good) = h
            .economy_catalog
            .as_ref()
            .and_then(|c| c.goods.iter().position(|g| g.id == "writing_material"))
        else {
            continue;
        };
        let e = &mut h.sites[site as usize].economy;
        if e.goods[good] < 0.05 {
            continue;
        }
        e.goods[good] -= 0.05;
        e.used[good] += 0.05;
        e.reserves[3] += 0.05;
        for (k, r) in h
            .economy_catalog
            .as_ref()
            .unwrap()
            .composition(good)
            .into_iter()
            .enumerate()
        {
            e.detritus[k] += 0.05 * r;
        }
        c.labor_budget[site as usize] -= 0.1;
        c.labor_spent += 0.1;
        let comparison = c
            .artifacts
            .iter()
            .find(|other| {
                other.id != id
                    && other.site == Some(site)
                    && !other.lost
                    && !other.destroyed
                    && other.kind == "ancient ceramic fragment"
            })
            .map(|a| a.id);
        let reading = if let Some(other) = comparison {
            format!("Compared the marks with object {other}; possible shared practices remain an interpretation, not established common authorship.")
        } else if find.studies.is_empty() {
            "Recorded the surviving marks and wear; the fragment's date and original use remain uncertain.".into()
        } else {
            "Reconsidered the earlier reading against local founding accounts; resemblance does not establish the patron's identity.".into()
        };
        h.event(
            "heritage_interpreted",
            Some(site),
            None,
            format!("{}: {reading}", h.people[author as usize].name),
        );
        let ev = h.events.last_mut().unwrap();
        ev.causes
            .push(find.studies.last().map_or(find.observed, |s| s.event));
        ev.subjects.extend([
            ("artifact".into(), id),
            ("institution".into(), institution),
            ("person".into(), author),
        ]);
        if let Some(other) = comparison {
            ev.subjects.push(("artifact".into(), other));
        }
        let event = ev.id;
        c.accounts.push(Account {
            id: c.accounts.len() as u32,
            tradition: charter.tradition,
            author: Some(author),
            institution: Some(institution),
            month: h.month,
            facts: vec![find.observed, event],
            text: reading,
        });
        c.artifacts[id as usize].events.push(event);
        find.studies.push(Study {
            month: h.month,
            event,
            site,
            author,
            comparison,
        });
    }
    h.expeditions = Some(x);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Study {
    pub month: u32,
    pub event: u64,
    pub site: u32,
    pub author: u32,
    pub comparison: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn heritage_study_requires_access_work_and_preserves_prior_readings() {
        use crate::{
            catalog::Catalog,
            config::Config,
            culture::{Institution, InstitutionKind},
            gpu::{ContextGpu, Generator},
        };
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
                ecology_resolution: 64,
                seed: 7,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        g.enable_shipping().unwrap();
        g.enable_expeditions().unwrap();
        let h = g.civilizations.as_mut().unwrap();
        h.month = 12;
        let mut c = h.culture.take().unwrap();
        let mut a = c.artifacts[0].clone();
        a.kind = "ancient ceramic fragment".into();
        let site = a.site.unwrap();
        let id = a.id;
        a.lost = false;
        a.destroyed = false;
        c.artifacts[id as usize] = a;
        let author = c.traditions[0].leader;
        c.institutions.push(Institution {
            id: c.institutions.len() as u32,
            name: "Study circle".into(),
            kind: InstitutionKind::Scholarly,
            site,
            tradition: Some(0),
            members: vec![author],
            leader: author,
            treasury: 10.,
            active: true,
            founded: 0,
            knowledge: Default::default(),
            property: vec![id],
            dues: 0.,
            expenses: 0.,
            capacity: None,
        });
        h.expeditions.as_mut().unwrap().voyages.push(Expedition {
            planned_cells: None,
            institution: None,
            heritage: Some(Charter {
                tradition: 0,
                patron: None,
                motive: "Test".into(),
                find: Some(Find {
                    cell: 0,
                    description: "marks".into(),
                    observed: 0,
                    artifact: Some(id),
                    studies: vec![],
                }),
            }),
            id: 0,
            origin: site,
            sponsor: 0,
            public_funding: true,
            route: 0,
            objective: Objective::Inscriptions,
            rescue: None,
            crew: vec![],
            phase: Phase::Returned,
            departed: 0,
            due: 0,
            ended: Some(0),
            food: 0.,
            timber: 0.,
            tools: 0.,
            purse: 0.,
            spent: 0.,
            findings: 0.,
            confirmed: false,
            exposure: 0.,
            skill: 0.,
            cause: 0,
            field_months: 1,
            samples: [0.; 2],
        });
        let good = h
            .economy_catalog
            .as_ref()
            .unwrap()
            .goods
            .iter()
            .position(|g| g.id == "writing_material")
            .unwrap();
        h.sites[site as usize].economy.goods[good] = 1.;
        c.labor_budget = vec![0.; h.sites.len()];
        let count = |h: &History| {
            h.expeditions.as_ref().unwrap().voyages[0]
                .heritage
                .as_ref()
                .unwrap()
                .find
                .as_ref()
                .unwrap()
                .studies
                .len()
        };
        study(h, &mut c);
        assert_eq!(count(h), 0);
        c.labor_budget[site as usize] = 0.5;
        c.artifacts[id as usize].lost = true;
        study(h, &mut c);
        assert_eq!(count(h), 0);
        c.artifacts[id as usize].lost = false;
        let materials = c.artifacts[id as usize].materials.clone();
        let accounts = c.accounts.len();
        study(h, &mut c);
        assert_eq!(count(h), 1);
        assert_eq!(c.accounts.len(), accounts + 1);
        assert_eq!(materials, c.artifacts[id as usize].materials);
        assert!((h.sites[site as usize].economy.goods[good] - 0.95).abs() < 1e-6);
        study(h, &mut c);
        assert_eq!(count(h), 1);
        h.month += 60;
        study(h, &mut c);
        assert_eq!(count(h), 2);
        assert_eq!(c.accounts.len(), accounts + 2);
        assert_eq!(c.artifacts[id as usize].materials, materials);
    }
}
