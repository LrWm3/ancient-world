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
    #[serde(default)]
    pub category: FindCategory,
    pub cell: u32,
    pub description: String,
    pub observed: u64,
    pub artifact: Option<u32>,
    #[serde(default)]
    pub studies: Vec<Study>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum FindCategory {
    #[default]
    Ceramic,
    Weaving,
    TradeWeight,
    Tool,
}
impl FindCategory {
    fn material(self) -> &'static str {
        match self {
            Self::Ceramic => "pottery",
            Self::Weaving => "cloth",
            Self::TradeWeight => "metal",
            Self::Tool => "tools",
        }
    }
    fn kind(self) -> &'static str {
        match self {
            Self::Ceramic => "ancient ceramic fragment",
            Self::Weaving => "ancient textile remnant",
            Self::TradeWeight => "ancient trade weight",
            Self::Tool => "ancient tool fragment",
        }
    }
    fn description(self) -> &'static str {
        match self {
        Self::Ceramic => "A worn ceramic fragment preserves uncertain marks.",
        Self::Weaving => "A mineral-encrusted strip of weaving preserves an unfamiliar pattern; its makers and original use are unknown.",
        Self::TradeWeight => "A small corroded weight bears repeated notches; it may record an old trading measure, but its unit is uncertain.",
        Self::Tool => "A worn tool fragment retains traces of repair, evidence of ordinary craft and reuse rather than lost miraculous technology.",
    }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudyFunding {
    pub institution: u32,
    pub good: String,
    pub kg: f32,
    pub paid: f64,
}
struct Purchase {
    good: usize,
    remaining: f32,
    kg: f32,
    cash: f32,
    paid: f64,
}
fn writing_batch(stock: f32) -> Option<(f32, f32)> {
    if !stock.is_finite() || stock < 0.05 {
        return None;
    }
    let mut remaining = stock - 0.05;
    if stock - remaining > 0.05 {
        remaining = f32::from_bits(remaining.to_bits() + 1).min(stock);
    }
    let kg = stock - remaining;
    (kg > 0.).then_some((remaining, kg))
}

fn purchase(h: &History, site: u32, treasury: f64) -> Option<Purchase> {
    let good = h.economy_catalog.as_ref()?.index("writing_material")?;
    let e = &h.sites.get(site as usize)?.economy;
    let (remaining, kg) = writing_batch(e.goods[good])?;
    let cost = kg as f64 * e.prices[good] as f64;
    if !cost.is_finite() || cost <= 0. || cost > treasury {
        return None;
    }
    let mut cash = e.finance[0];
    let paid = crate::household_economy::deposit(&mut cash, cost);
    (paid > 0. && paid <= treasury).then_some(Purchase {
        good,
        remaining,
        kg,
        cash,
        paid,
    })
}
pub(crate) fn can_fund_study(h: &History, site: u32, treasury: f64) -> bool {
    purchase(h, site, treasury).is_some()
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
    // One small accessible object per qualifying endpoint, across all objectives.
    let hash = cell
        .wrapping_mul(747796405)
        .wrapping_add(h.seed.wrapping_mul(2891336453));
    if (hash ^ (hash >> 16)) % 5 >= 3 {
        return;
    }
    let category = if e.objective == Objective::PatronSearch {
        FindCategory::Ceramic
    } else {
        match (hash >> 8) % 4 {
            0 => FindCategory::Ceramic,
            1 => FindCategory::Weaving,
            2 => FindCategory::TradeWeight,
            _ => FindCategory::Tool,
        }
    };
    let mut description=match e.objective {
        Objective::PatronSearch=>"A worn ceramic charm bears an animal-like outline. Its resemblance to the patron is suggestive, not identification; no living patron was encountered.",
        Objective::Inscriptions=>"A broken ceramic plaque bears repeated marks and possible tally strokes. Dating, language and authorship remain uncertain.",
        _=>"A small fired-clay text fragment preserves repeated signs interpreted provisionally as a household list or refrain; most of the text is missing.",
    }.to_string();
    if !matches!(category, FindCategory::Ceramic) {
        description = category.description().into();
    }
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
        category,
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
    let Some(good) = h.economy_catalog.as_ref().and_then(|c| {
        c.goods
            .iter()
            .position(|g| g.id == find.category.material())
    }) else {
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
                &["memory"],
                Some(crate::naming::Source {
                    kind: "expedition".into(),
                    id: e.id,
                    name: format!("{} voyage {}", h.sites[e.origin as usize].name, e.id),
                }),
            ),
        kind: find.category.kind().into(),
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
    h.event("heritage_fragment_received",Some(e.origin),None,format!("Voyage {} delivered one 0.125 kg {} from cell {}; preserved for material study and competing interpretations",e.id,find.category.kind(),find.cell));
    let event = h.events.last_mut().unwrap();
    event.causes.extend([e.cause, find.observed]);
    event.subjects.extend([
        ("artifact".into(), id),
        ("tradition".into(), charter.tradition),
    ]);
    let received = event.id;
    event.subjects.push(("civilization".into(), e.sponsor));
    if let Some(institution) = e.institution {
        event.subjects.push(("institution".into(), institution));
    }
    let people: Vec<_> = e
        .crew
        .iter()
        .filter(|p| p.alive)
        .filter_map(|p| p.person)
        .collect();
    event
        .subjects
        .extend(people.iter().map(|&p| ("person".into(), p)));
    h.culture
        .as_mut()
        .unwrap()
        .heritage_renown
        .push(crate::heritage_renown::Recognition {
            artifact: id,
            expedition: e.id,
            event: received,
            month: h.month,
            origin: e.origin,
            civilization: e.sponsor,
            tradition: charter.tradition,
            institution: e.institution,
            people,
            survival: e.crew.iter().filter(|p| p.alive).count() as f32 / e.crew.len().max(1) as f32,
            witnesses: vec![(e.origin, h.month)],
        });
    // Locally witnessed public patronage benefits the government that actually paid.
    // Private/institutional recoveries cannot be claimed automatically by a ruler.
    if e.public_funding && e.institution.is_none() && h.controller(e.origin) == e.sponsor {
        let survival =
            e.crew.iter().filter(|p| p.alive).count() as f32 / e.crew.len().max(1) as f32;
        if let Some(g) = &mut h.governance {
            let a = &mut g.administrations[e.origin as usize];
            a.loyalty = (a.loyalty + 0.01 * survival).min(1.);
        }
    }
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
                    if let Some(funding) = &s.funding {
                        ensure!(
                            funding.good == "writing_material"
                                && funding.kg.is_finite()
                                && funding.kg > 0.
                                && funding.kg <= 0.05
                                && funding.paid.is_finite()
                                && funding.paid > 0.
                                && h.events.get(s.event as usize).is_some_and(|e| e
                                    .subjects
                                    .contains(&("institution".into(), funding.institution)))
                                && culture
                                    .institutions
                                    .get(funding.institution as usize)
                                    .is_some_and(|n| n.expenses + 1e-8 >= funding.paid),
                            "invalid heritage study funding"
                        );
                    }
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
                                .is_some_and(|a| a.kind == f.category.kind())),
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
        let captured = c
            .work_plans
            .get(site as usize)
            .and_then(|p| p.services.as_ref());
        let selected = captured.and_then(|plans| {
            plans.iter().find_map(|p| {
                p.receipts.iter().find_map(|r| match r.service {
                    crate::institution_services::Service::HeritageStudy { artifact, author }
                        if artifact == id =>
                    {
                        Some((p.institution, author))
                    }
                    _ => None,
                })
            })
        });
        if captured.is_some() && selected.is_none() {
            continue;
        }
        let present = c.site_people(h, site);
        let Some(n) = c.institutions.iter().find(|n| {
            selected.is_none_or(|(institution, author)| {
                n.id == institution && n.leader == author && present.contains(&author)
            }) && n.site == site
                && (!c.funded_heritage_study || can_fund_study(h, site, n.treasury))
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
        if h.sites[site as usize].economy.goods[good] < 0.05 {
            continue;
        }
        let Some((remaining, kg)) = writing_batch(h.sites[site as usize].economy.goods[good])
        else {
            continue;
        };
        let funded = if c.funded_heritage_study {
            let Some(quote) = purchase(h, site, n.treasury) else {
                continue;
            };
            Some(quote)
        } else {
            None
        };
        if !c.consume_service_space(
            h.month,
            site,
            institution,
            crate::institution_services::Service::HeritageStudy {
                artifact: id,
                author,
            },
        ) {
            continue;
        }
        let e = &mut h.sites[site as usize].economy;

        let funding = funded.map(|q| {
            debug_assert_eq!(q.good, good);
            debug_assert_eq!(q.remaining, remaining);
            e.finance[0] = q.cash;
            let n = &mut c.institutions[institution as usize];
            n.treasury -= q.paid;
            n.expenses += q.paid;
            StudyFunding {
                institution,
                good: "writing_material".into(),
                kg: q.kg,
                paid: q.paid,
            }
        });
        e.goods[good] = remaining;
        e.used[good] += kg;
        e.reserves[3] += kg;
        for (k, r) in h
            .economy_catalog
            .as_ref()
            .unwrap()
            .composition(good)
            .into_iter()
            .enumerate()
        {
            e.detritus[k] += kg * r;
        }
        c.labor_budget[site as usize] -= 0.1;
        c.labor_spent += 0.1;
        crate::culture::work_requests::record_work(&mut c.work_plans, site, h.month, 0.1);
        let comparison = c
            .artifacts
            .iter()
            .find(|other| {
                other.id != id
                    && other.site == Some(site)
                    && !other.lost
                    && !other.destroyed
                    && other.kind.starts_with("ancient ")
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
            funding,
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
    #[serde(default)]
    pub funding: Option<StudyFunding>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn writing_batches_never_overdraw_or_charge_unrepresentable_stock() {
        for stock in [0., 0.049, 0.05, 1., 123., 10000., 1_000_000., f32::INFINITY] {
            if let Some((remaining, kg)) = writing_batch(stock) {
                assert!(remaining >= 0. && kg > 0. && kg <= 0.05);
                assert_eq!(stock - remaining, kg);
            } else {
                assert!(!(0.05..1_000_000.).contains(&stock));
            }
        }
        assert!(writing_batch(1_000_000.).is_none());
        assert!(writing_batch(f32::NAN).is_none());
    }
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
        h.sync_culture();
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
                    category: FindCategory::Ceramic,
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
            botanicals: [0.; 3],
            botanical_sources: [None; 3],
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
        // Financed study has one payer and cannot buy missing work, access or stock.
        for blocked in 0..5 {
            let mut run = h.clone();
            let mut culture = c.clone();
            culture.funded_heritage_study = true;
            culture.work_plans.clear();
            culture.labor_budget[site as usize] = if blocked == 3 { 0. } else { 0.1 };
            culture.institutions.last_mut().unwrap().treasury = if blocked == 1 { 0. } else { 10. };
            run.sites[site as usize].economy.prices[good] = 2.;
            if blocked == 2 {
                run.sites[site as usize].economy.goods[good] = 0.;
            }
            if blocked == 4 {
                culture.artifacts[id as usize].lost = true;
            }
            let money = run.sites[site as usize].economy.finance[0] as f64
                + culture.institutions.last().unwrap().treasury;
            let stock = run.sites[site as usize].economy.goods[good];
            let mut restored_h: History =
                serde_json::from_value(serde_json::to_value(&run).unwrap()).unwrap();
            let mut restored_c =
                serde_json::from_value(serde_json::to_value(&culture).unwrap()).unwrap();
            study(&mut run, &mut culture);
            study(&mut restored_h, &mut restored_c);
            assert_eq!(
                serde_json::to_value(&run).unwrap(),
                serde_json::to_value(&restored_h).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&culture).unwrap(),
                serde_json::to_value(&restored_c).unwrap()
            );
            assert_eq!(count(&run), usize::from(blocked == 0));
            assert_eq!(
                money,
                run.sites[site as usize].economy.finance[0] as f64
                    + culture.institutions.last().unwrap().treasury
            );
            if blocked == 0 {
                let receipt = &run.expeditions.as_ref().unwrap().voyages[0]
                    .heritage
                    .as_ref()
                    .unwrap()
                    .find
                    .as_ref()
                    .unwrap()
                    .studies[0];
                let funding = receipt.funding.as_ref().unwrap();
                assert!(funding.paid > 0. && funding.paid <= 0.101);
                assert_eq!(
                    funding.kg,
                    stock - run.sites[site as usize].economy.goods[good]
                );
                assert_eq!(culture.institutions.last().unwrap().expenses, funding.paid);
                let mut old = serde_json::to_value(receipt).unwrap();
                old.as_object_mut().unwrap().remove("funding");
                assert!(serde_json::from_value::<Study>(old)
                    .unwrap()
                    .funding
                    .is_none());
            } else {
                assert_eq!(stock, run.sites[site as usize].economy.goods[good]);
                assert_eq!(culture.institutions.last().unwrap().expenses, 0.);
            }
            let before = serde_json::to_value((&run, &culture)).unwrap();
            study(&mut run, &mut culture);
            assert_eq!(before, serde_json::to_value((&run, &culture)).unwrap());
        }
        // New plans capture the same find, author and physical room at Reserve.
        let mut planned_history = h.clone();
        let mut planned_culture = c.clone();
        let mut room = planned_culture.artifacts[id as usize].clone();
        room.id = planned_culture.artifacts.len() as u32;
        room.kind = "fixture meeting place".into();
        room.materials = vec![(0, 20.)]; // Declared finite room input, separate from find.
        let room_id = room.id;
        planned_culture.artifacts.push(room);
        planned_culture.institutions.last_mut().unwrap().capacity =
            Some(crate::institution_capacity::Capacity {
                building: Some(crate::institution_capacity::MeetingPlace::new(room_id)),
                ..crate::institution_capacity::Capacity::new(h.month)
            });
        // Four finds and one lesson share the ordinary 0.5-work ceiling. A
        // damaged but operational two-person room has only 0.5 room-months;
        // the heritage-first execution order then leaves too little for a lesson.
        for condition in [0.25, 0.99] {
            let mut run = planned_history.clone();
            let mut culture = planned_culture.clone();
            culture.funded_heritage_study = true;
            run.sites[site as usize].economy.prices[good] = 2.;
            let people = culture.site_people(&run, site);
            let student = people[((run.month / 3 + site) as usize) % people.len()];
            let teacher = *people.iter().find(|&&p| p != student).unwrap();
            for agent in &mut culture.agents {
                agent.knowledge.clear();
                agent.knowledge_sources.clear();
                agent.studies.clear();
                agent.traits = [0.; 6];
            }
            for a in &mut culture.artifacts {
                a.topic = None;
            }
            for n in &mut culture.institutions {
                n.active = false;
            }
            let n = culture.institutions.last_mut().unwrap();
            n.active = true;
            n.leader = teacher;
            n.members = people.clone();
            n.knowledge = [4].into_iter().collect();
            n.capacity
                .as_mut()
                .unwrap()
                .building
                .as_mut()
                .unwrap()
                .condition = condition;
            culture.agents[teacher as usize].knowledge.insert(4);
            for _ in 1..4 {
                let mut fragment = culture.artifacts[id as usize].clone();
                fragment.id = culture.artifacts.len() as u32;
                let mut voyage = run.expeditions.as_ref().unwrap().voyages[0].clone();
                voyage.id = run.expeditions.as_ref().unwrap().voyages.len() as u32;
                voyage
                    .heritage
                    .as_mut()
                    .unwrap()
                    .find
                    .as_mut()
                    .unwrap()
                    .artifact = Some(fragment.id);
                culture.artifacts.push(fragment); // Declared additional finite fixture finds.
                run.expeditions.as_mut().unwrap().voyages.push(voyage);
            }
            run.culture = Some(culture);
            run.open_participation();
            let mut plans = run.cultural_work_plans();
            // Isolate these real study requests from optional administrative or
            // personal actions that could legitimately use the remaining time.
            let p = &mut plans[site as usize];
            p.actions
                .retain(|(a, _)| a == "study" || a == "heritage study");
            p.space_feasible_work = Some(crate::institution_services::feasible_work(
                &p.actions,
                p.services.as_ref().unwrap(),
                true,
            ));
            assert_eq!(p.raw_work(), 0.5);
            assert_eq!(p.feasible_work(), if condition == 0.25 { 0.4 } else { 0.5 });
            run.reserve_cultural_plans(plans, &vec![0.5; run.sites.len()]);
            let mut culture = run.culture.take().unwrap();
            let before = run.sites[site as usize].economy.goods[good];
            let grant = culture.work_plans[site as usize].granted;
            assert_eq!(grant, if condition == 0.25 { 0.4 } else { 0.5 });
            // A quote is not escrow: losing funding after reservation consumes no room/work.
            let mut poor_run = run.clone();
            let mut poor_culture = culture.clone();
            let payer = poor_culture.institutions.last_mut().unwrap();
            let moved = crate::household_economy::deposit(
                &mut poor_run.sites[site as usize].economy.finance[0],
                payer.treasury,
            );
            payer.treasury -= moved;
            payer.expenses += moved;
            let unused_work = poor_culture.labor_budget.clone();
            study(&mut poor_run, &mut poor_culture);
            assert_eq!(count(&poor_run), 0);
            assert_eq!(poor_run.sites[site as usize].economy.goods[good], before);
            assert_eq!(poor_culture.labor_budget, unused_work);
            assert!(poor_culture.work_plans[site as usize]
                .services
                .as_ref()
                .unwrap()
                .iter()
                .flat_map(|p| &p.receipts)
                .all(|r| r.used == 0.));
            let mut resumed_run = run.clone();
            let mut resumed_culture =
                serde_json::from_value(serde_json::to_value(&culture).unwrap()).unwrap();
            study(&mut run, &mut culture);
            culture.decisions(&mut run);
            study(&mut resumed_run, &mut resumed_culture);
            resumed_culture.decisions(&mut resumed_run);
            assert_eq!(
                serde_json::to_value(&culture).unwrap(),
                serde_json::to_value(&resumed_culture).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&run.events).unwrap(),
                serde_json::to_value(&resumed_run.events).unwrap()
            );
            assert_eq!(
                run.sites[site as usize].economy.goods,
                resumed_run.sites[site as usize].economy.goods
            );
            let reads: usize = run
                .expeditions
                .as_ref()
                .unwrap()
                .voyages
                .iter()
                .map(|v| {
                    v.heritage
                        .as_ref()
                        .unwrap()
                        .find
                        .as_ref()
                        .unwrap()
                        .studies
                        .len()
                })
                .sum();
            assert_eq!(reads, 4);
            assert!((before - run.sites[site as usize].economy.goods[good] - 0.2).abs() < 1e-6);
            let gain = culture.work_plans[site as usize]
                .study_expectation
                .as_ref()
                .unwrap()
                .lesson
                .actual_gain;
            assert_eq!(gain > 0., condition > 0.25);
            let services = culture.work_plans[site as usize].services.as_ref().unwrap();
            let room = services.iter().find(|p| p.receipts.len() == 5).unwrap();
            assert_eq!(room.group_space, Some(2.));
            let used: f64 = room.receipts.iter().map(|r| r.used).sum();
            assert!((used - if condition == 0.25 { 0.4 } else { 0.6 }).abs() < 1e-6);
            assert!(used <= room.opening_space);
            crate::institution_services::validate_work_plan(
                &culture.work_plans[site as usize],
                &culture,
                run.people.len(),
            )
            .unwrap();
        }
        planned_history.culture = Some(planned_culture);
        planned_history.open_participation();
        let plans = planned_history.cultural_work_plans();
        planned_history.reserve_cultural_plans(plans, &vec![0.5; h.sites.len()]);
        let planned_culture = planned_history.culture.take().unwrap();
        assert!(planned_culture.work_plans[site as usize].services.as_ref().unwrap().iter()
            .flat_map(|p| &p.receipts).any(|r| matches!(r.service,
                crate::institution_services::Service::HeritageStudy { artifact, .. } if artifact == id)));
        for failed in [false, true] {
            let mut run = planned_history.clone();
            let mut culture = planned_culture.clone();
            // Explicit bounded work isolates room availability from production.
            culture.labor_budget[site as usize] = 0.1;
            if failed {
                culture.artifacts[room_id as usize].destroyed = true;
            }
            let before = run.sites[site as usize].economy.goods[good];
            study(&mut run, &mut culture);
            crate::institution_services::validate_work_plan(
                &culture.work_plans[site as usize],
                &culture,
                run.people.len(),
            )
            .unwrap();
            for corruption in 0..6 {
                let mut invalid = culture.work_plans[site as usize].clone();
                let plan = &mut invalid.services.as_mut().unwrap()[0];
                match corruption {
                    0 => plan.site = u32::MAX,
                    1 => plan.month += 1,
                    2 => plan.institution = u32::MAX,
                    3 => {
                        plan.receipts[0].service =
                            crate::institution_services::Service::HeritageStudy {
                                artifact: u32::MAX,
                                author,
                            }
                    }
                    4 => {
                        plan.receipts[0].service =
                            crate::institution_services::Service::HeritageStudy {
                                artifact: id,
                                author: u32::MAX,
                            }
                    }
                    _ => invalid.granted = 0.,
                }
                let restored =
                    serde_json::from_value(serde_json::to_value(invalid).unwrap()).unwrap();
                assert!(crate::institution_services::validate_work_plan(
                    &restored,
                    &culture,
                    run.people.len()
                )
                .is_err());
            }
            assert_eq!(count(&run), usize::from(!failed));
            assert!(
                (before
                    - run.sites[site as usize].economy.goods[good]
                    - if failed { 0. } else { 0.05 })
                .abs()
                    < 1e-6
            );
            let after = run.sites[site as usize].economy.goods[good];
            study(&mut run, &mut culture);
            assert_eq!(run.sites[site as usize].economy.goods[good], after);
        }
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

        // Deliver a new finite find through the production path. Repeating delivery
        // must not duplicate either material or reputation.
        let mut voyage = h.expeditions.as_ref().unwrap().voyages[0].clone();
        voyage
            .heritage
            .as_mut()
            .unwrap()
            .find
            .as_mut()
            .unwrap()
            .artifact = None;
        voyage.crew.push(crate::expeditions::Crew {
            person: Some(author),
            identified_from_cohort: false,
            expertise: None,
            name: "Witness".into(),
            role: "interpreter".into(),
            alive: true,
        });
        h.culture = Some(c);
        let loyalty = h.governance.as_ref().unwrap().administrations[site as usize].loyalty;
        let mut private = h.clone();
        let mut private_voyage = voyage.clone();
        private_voyage.public_funding = false;
        deliver(&mut private, &mut private_voyage);
        assert_eq!(
            private.governance.as_ref().unwrap().administrations[site as usize].loyalty,
            loyalty
        );
        deliver(h, &mut voyage);
        assert_eq!(
            h.governance.as_ref().unwrap().administrations[site as usize].loyalty,
            (loyalty + 0.01).min(1.)
        );
        let artifact = voyage
            .heritage
            .as_ref()
            .unwrap()
            .find
            .as_ref()
            .unwrap()
            .artifact
            .unwrap();
        let before = h.economy_residuals();
        deliver(h, &mut voyage);
        assert_eq!(before, h.economy_residuals());
        let mut c = h.culture.take().unwrap();
        assert_eq!(c.heritage_renown.len(), 1);
        crate::heritage_renown::validate(&c, h).unwrap();
        assert!(
            crate::heritage_renown::score(&c, site, h.month, |r| r.people.contains(&author)) > 0.
        );
        // Controlled route chain, independent of planet geography.
        let middle = (site + 1) % h.sites.len() as u32;
        let far = (site + 2) % h.sites.len() as u32;
        h.society.as_mut().unwrap().routes = vec![(site, middle), (middle, far)]
            .into_iter()
            .enumerate()
            .map(|(i, (from, to))| crate::society::Route {
                id: i as u32,
                from,
                to,
                cells: vec![],
                cost_km: 100.,
                open: true,
                flood_months: 0,
                road_bricks: 0.,
                upkeep: None,
            })
            .collect();
        h.month += 1;
        crate::heritage_renown::spread(h, &mut c);
        assert_eq!(
            crate::heritage_renown::score(&c, middle, h.month, |_| true),
            0.,
            "route without traffic carries no news"
        );
        h.trade_contact.observe(h.month, site, middle, 100.);
        h.trade_contact.observe(h.month, middle, far, 100.);
        crate::heritage_renown::spread(h, &mut c);
        assert!(crate::heritage_renown::score(&c, middle, h.month, |_| true) > 0.);
        assert_eq!(
            crate::heritage_renown::score(&c, far, h.month, |_| true),
            0.
        );
        crate::heritage_renown::spread(h, &mut c);
        assert_eq!(
            crate::heritage_renown::score(&c, far, h.month, |_| true),
            0.,
            "same-month repeat cannot send a second hop"
        );
        assert_eq!(
            crate::heritage_renown::destination(&c, h, middle, 0, 1.),
            Some((site, artifact))
        );
        // Nearby space alone does not expose private or foreign institutional objects.
        let host = c.institutions.last().unwrap().id;
        let unrelated = h
            .society
            .as_ref()
            .unwrap()
            .households
            .iter()
            .find(|hh| hh.site == middle)
            .unwrap()
            .head;
        let original_owner = c.artifacts[artifact as usize].owner.clone();
        for (owner, custodian, accessible) in [
            (Owner::Institution(host), None, true),
            (Owner::Institution(u32::MAX), None, false),
            (Owner::Person(author), None, true),
            (Owner::Person(unrelated), None, false),
            (Owner::Community(site), Some(author), true),
            (Owner::Community(site), Some(unrelated), false),
            (Owner::Community(middle), None, false),
        ] {
            c.artifacts[artifact as usize].owner = owner;
            c.artifacts[artifact as usize].custodian = custodian;
            assert_eq!(
                crate::heritage_renown::destination(&c, h, middle, 0, 1.).is_some(),
                accessible
            );
        }
        c.artifacts[artifact as usize].owner = original_owner;
        c.artifacts[artifact as usize].custodian = None;
        let leader = c.institutions[host as usize].leader;
        c.institutions[host as usize].leader = unrelated;
        assert_eq!(
            crate::heritage_renown::destination(&c, h, middle, 0, 1.),
            None
        );
        c.institutions[host as usize].leader = leader;
        c.artifacts[artifact as usize].lost = true;
        assert_eq!(
            crate::heritage_renown::destination(&c, h, middle, 0, 1.),
            None
        );
        assert!(
            crate::heritage_renown::score(&c, middle, h.month, |r| r.people.contains(&author)) > 0.,
            "loss of custody does not erase the witnessed achievement"
        );
        c.artifacts[artifact as usize].lost = false;
        h.society.as_mut().unwrap().routes[1].open = false;
        h.month += 1;
        crate::heritage_renown::spread(h, &mut c);
        assert_eq!(
            crate::heritage_renown::score(&c, far, h.month, |_| true),
            0.
        );
        let mut resumed: crate::culture::Culture =
            serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
        h.society.as_mut().unwrap().routes[1].open = true;
        h.month += 1;
        h.trade_contact.observe(h.month, middle, far, 100.);
        crate::heritage_renown::spread(h, &mut c);
        crate::heritage_renown::spread(h, &mut resumed);
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::to_value(&resumed).unwrap()
        );
        assert!(crate::heritage_renown::score(&c, far, h.month, |_| true) > 0.);
    }
}
