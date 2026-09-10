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
    c.find = Some(Find {
        cell,
        description,
        observed: event.id,
        artifact: None,
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
