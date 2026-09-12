use ancient_world::{
    catalog::Catalog,
    config::Config,
    expeditions::{Objective, Phase, Rules},
    gpu::{ContextGpu, Generator},
};
fn world() -> Generator {
    world_with_domestic(true)
}
fn world_with_domestic(domestic: bool) -> Generator {
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
    g.civilizations
        .as_mut()
        .unwrap()
        .set_domestic_households(domestic)
        .unwrap();
    // Isolate transport, escrow and rescue accounting from new crop balance.
    g.set_diversified_farming(false).unwrap();
    g.enable_society().unwrap();
    g.enable_politics().unwrap();
    g.enable_governance().unwrap();
    g.enable_shipping().unwrap();
    g.enable_expeditions().unwrap();
    g.configure_expeditions(Rules {
        automatic: false,
        hazard_scale: 0.,
        ..Default::default()
    })
    .unwrap();
    g.advance_history(240).unwrap();
    // These are voyage accounting fixtures, not a claim that 20 years of the
    // current town balance always produces expedition wealth. Declare outfitting
    // imports explicitly; launch still checks civilian reserves and real crews.
    let terrain = g.snapshot().unwrap();
    let h = g.civilizations.as_mut().unwrap();
    let tools_cnp = h.economy_catalog.as_ref().unwrap().composition(3);
    for site in &mut h.sites {
        site.economy.goods[3] += 200.;
        site.economy.initial[3] += 200.;
        for (k, ratio) in tools_cnp.into_iter().enumerate() {
            site.economy.external[k] += 200. * ratio;
        }
        site.economy.finance[0] += 5000.;
        site.economy.finance[1] += 5000.;
    }
    h.validate(&terrain).unwrap();
    g
}
fn launch(g: &mut Generator) -> u32 {
    let routes = g
        .civilizations
        .as_ref()
        .unwrap()
        .expeditions
        .as_ref()
        .unwrap()
        .routes
        .len();
    (0..routes)
        .find_map(
            |r| match g.launch_expedition(r as u32, Objective::Ecology, None) {
                Ok(id) => Some(id),
                Err(e) => {
                    eprintln!("route {r}: {e}");
                    None
                }
            },
        )
        .expect("funded fixture must support a voyage")
}
#[test]
#[ignore = "requires hardware GPU"]
fn voyages_conserve_and_deliver_knowledge_after_exact_checkpoint_continuation() {
    let mut g = world();
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .expeditions
        .as_ref()
        .unwrap()
        .voyages
        .is_empty());
    let people_before = g.civilizations.as_ref().unwrap().people.len();
    let population_before: f32 = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.stocks.stock[0])
        .sum();
    let id = launch(&mut g);
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.people.len() >= people_before && h.people.len() <= people_before + 8);
    let population_after: f32 = h.sites.iter().map(|s| s.stocks.stock[0]).sum();
    assert!((population_before - population_after - 8.).abs() < 0.01);
    let x = h.expeditions.as_ref().unwrap();
    let e = &x.voyages[id as usize];
    assert_eq!(
        e.planned_cells.as_ref().unwrap(),
        &x.routes[e.route as usize].cells
    );
    let spatial_before = serde_json::to_value(g.spatial_features().unwrap()).unwrap();
    let history_before = serde_json::to_value(h).unwrap();
    let exported = g.spatial_features().unwrap().geojson().unwrap();
    assert!(exported["features"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["properties"]["role"] == "planned_route"));
    assert_eq!(
        history_before,
        serde_json::to_value(g.civilizations.as_ref().unwrap()).unwrap()
    );
    let initial_expertise: Vec<_> = e.crew.iter().map(|c| c.expertise.unwrap()).collect();
    let original_people: Vec<_> = e.crew.iter().map(|c| c.person.unwrap()).collect();
    let port = &h.shipping.as_ref().unwrap().ports[x.routes[e.route as usize].port as usize];
    assert!(port.harbor_capacity() >= 500.);
    assert!(
        port.capacity() < 500.,
        "fixture exercises launch without prepaid merchant crews"
    );
    for &person in &original_people {
        assert_eq!(
            h.person_presence(person).1,
            ancient_world::participation::Presence::Expedition(id)
        );
    }
    let mut disabled = h.clone();
    disabled.set_individual_participation(false).unwrap();
    assert_eq!(
        disabled.person_presence(original_people[0]).1,
        ancient_world::participation::Presence::Expedition(id)
    );
    let mut broken = h.clone();
    broken.person_duties.remove(&original_people[0]);
    assert!(broken.validate(&g.snapshot().unwrap()).is_err());
    let duration = x.routes[e.route as usize].travel_months * 2 + 10;
    assert_eq!(e.survivors(), 8);
    assert!(!e.confirmed);
    assert!(x.knowledge.iter().all(|v| *v == 0.));
    let terrain = g.snapshot().unwrap();
    h.validate(&terrain).unwrap();
    let before = serde_json::to_vec(&g.civilizations).unwrap();
    assert!(g
        .launch_expedition(e.route, Objective::Charts, None)
        .is_err());
    assert_eq!(before, serde_json::to_vec(&g.civilizations).unwrap());
    let file = std::env::temp_dir().join(format!("expedition-resume-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut b = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    assert_eq!(g.config.spatial_world_id, b.config.spatial_world_id);
    assert_eq!(
        spatial_before,
        serde_json::to_value(b.spatial_features().unwrap()).unwrap()
    );
    // Changing the current route graph must not rewrite a saved expedition plan.
    let bx = b
        .civilizations
        .as_mut()
        .unwrap()
        .expeditions
        .as_mut()
        .unwrap();
    let route = bx.voyages[id as usize].route as usize;
    let saved = bx.routes[route].cells.clone();
    bx.routes[route].cells.reverse();
    let altered = b.spatial_features().unwrap();
    let planned = altered
        .features
        .iter()
        .find(|f| f.entity.kind == "expedition" && f.role == "planned_route")
        .unwrap();
    if let ancient_world::spatial::Geometry::Path(cells) = &planned.geometry {
        assert_eq!(cells.iter().map(|c| c.cell).collect::<Vec<_>>(), saved);
    } else {
        panic!("expected path");
    }
    b.civilizations
        .as_mut()
        .unwrap()
        .expeditions
        .as_mut()
        .unwrap()
        .routes[route]
        .cells = saved;

    let recorded_before = serde_json::to_value(b.spatial_events(0..=u32::MAX).unwrap()).unwrap();
    let origin = b
        .civilizations
        .as_ref()
        .unwrap()
        .expeditions
        .as_ref()
        .unwrap()
        .voyages[id as usize]
        .origin as usize;
    let original_cell = b.civilizations.as_ref().unwrap().sites[origin].cell;
    b.civilizations.as_mut().unwrap().sites[origin].cell = (original_cell + 1) % (6 * 64 * 64);
    assert_eq!(
        recorded_before,
        serde_json::to_value(b.spatial_events(0..=u32::MAX).unwrap()).unwrap()
    );
    b.civilizations.as_mut().unwrap().sites[origin].cell = original_cell;
    assert_eq!(
        serde_json::to_value(g.spatial_events(0..=u32::MAX).unwrap()).unwrap(),
        recorded_before
    );

    let cause = b
        .civilizations
        .as_ref()
        .unwrap()
        .expeditions
        .as_ref()
        .unwrap()
        .voyages[id as usize]
        .cause;
    let frozen = b.civilizations.as_mut().unwrap().events[cause as usize]
        .spatial
        .take();
    let legacy_features = b.spatial_events(0..=u32::MAX).unwrap();
    let legacy_features: Vec<_> = legacy_features
        .features
        .iter()
        .filter(|f| f.entity.id == cause)
        .collect();
    assert!(!legacy_features.is_empty());
    assert!(legacy_features.iter().all(|f| matches!(
        f.precision,
        ancient_world::spatial::Precision::LegacySiteAssociation
    )));
    b.civilizations.as_mut().unwrap().events[cause as usize].spatial = Some(vec![]);
    assert!(!b
        .spatial_events(0..=u32::MAX)
        .unwrap()
        .features
        .iter()
        .any(|f| f.entity.id == cause));
    b.civilizations.as_mut().unwrap().events[cause as usize].spatial = frozen;
    assert!(b
        .spatial_events(std::ops::RangeInclusive::new(1, 0))
        .is_err());

    g.advance_history(duration).unwrap();
    for _ in 0..duration {
        b.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&b.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    let x = h.expeditions.as_ref().unwrap();
    let e = &x.voyages[id as usize];
    assert_eq!(e.phase, Phase::Returned);
    assert_eq!(
        e.crew.iter().map(|c| c.person.unwrap()).collect::<Vec<_>>(),
        original_people
    );
    for crew in &e.crew {
        let person = crew.person.unwrap();
        assert_eq!(
            h.person_presence(person).1,
            ancient_world::participation::Presence::Resident(e.origin)
        );
        if let Some(agent) = h
            .culture
            .as_ref()
            .and_then(|c| c.agents.get(person as usize))
        {
            if e.field_months > 0 {
                match crew.role.as_str() {
                    "engineer" => assert!(agent.skills[3] >= crew.expertise.unwrap()),
                    "navigator" => assert!(agent.skills[1] >= crew.expertise.unwrap()),
                    _ => {}
                }
            }
        }
    }
    assert!(h.person_duties.is_empty());
    let landfall = h
        .events
        .iter()
        .find(|v| v.kind == "expedition_landfall" && v.causes.contains(&e.cause))
        .unwrap();
    let milestone = landfall
        .spatial
        .as_ref()
        .unwrap()
        .iter()
        .find(|a| a.role == ancient_world::spatial::EventRole::Milestone)
        .unwrap();
    assert_eq!(
        milestone.cell,
        *e.planned_cells.as_ref().unwrap().last().unwrap()
    );
    let event_features = g.spatial_events(landfall.month..=landfall.month).unwrap();
    assert!(event_features
        .features
        .iter()
        .all(|f| f.history_month == landfall.month));
    assert!(event_features
        .features
        .iter()
        .any(|f| f.entity.id == landfall.id && f.role == "milestone"));
    let mut bad_event_history = h.clone();
    bad_event_history.events[landfall.id as usize]
        .spatial
        .as_mut()
        .unwrap()[0]
        .cell = 6 * 64 * 64;
    assert!(bad_event_history.validate(&terrain).is_err());
    let mut legacy = serde_json::to_value(landfall).unwrap();
    legacy.as_object_mut().unwrap().remove("spatial");
    let legacy: ancient_world::civilization::Event = serde_json::from_value(legacy).unwrap();
    assert!(legacy.spatial.is_none());

    assert!(e
        .crew
        .iter()
        .zip(&initial_expertise)
        .any(|(c, before)| c.expertise.unwrap() > *before));
    assert!(e.confirmed && x.knowledge[e.sponsor as usize] > 0.);
    h.validate(&terrain).unwrap();
    assert!(h
        .sites
        .iter()
        .all(|s| terrain[s.cell as usize].meta[0] == 2));
    let mut bad = h.clone();
    bad.expeditions.as_mut().unwrap().routes[0].cells[1] = h.sites[0].cell;
    assert!(bad.validate(&terrain).is_err());
    let mut bad = h.clone();
    bad.expeditions.as_mut().unwrap().voyages[0].purse = f64::NAN;
    assert!(bad.validate(&terrain).is_err());
    let mut bad = h.clone();
    bad.expeditions.as_mut().unwrap().voyages[0].crew[0].expertise = Some(1.01);
    assert!(bad.validate(&terrain).is_err());
}
#[test]
#[ignore = "requires hardware GPU"]
fn rescue_transfers_real_survivors_and_stores_and_recall_takes_time() {
    let mut g = world();
    let id = launch(&mut g);
    let x = g
        .civilizations
        .as_ref()
        .unwrap()
        .expeditions
        .as_ref()
        .unwrap();
    let route = x.voyages[id as usize].route;
    let travel = x.routes[route as usize].travel_months;
    g.advance_history(travel + 2).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    let e = &mut h.expeditions.as_mut().unwrap().voyages[id as usize];
    assert_eq!(e.phase, Phase::Camp);
    e.phase = Phase::Stranded;
    e.due = h.month + 1000;
    let origin = e.origin as usize;
    let rescued_expertise: Vec<_> = e
        .crew
        .iter()
        .filter(|c| c.alive)
        .map(|c| (c.name.clone(), c.expertise))
        .collect();
    // This fixture tests survivor and store transfers, not whether a sponsor can afford two
    // consecutive voyages. Fund the rescue with existing neighboring tool stocks.
    let mut needed =
        (25. + h.sites[origin].stocks.stock[0] * 0.5 - h.sites[origin].economy.goods[3]).max(0.);
    for donor in 0..h.sites.len() {
        if donor == origin {
            continue;
        }
        let transfer = needed.min(h.sites[donor].economy.goods[3]);
        h.sites[donor].economy.goods[3] -= transfer;
        h.sites[origin].economy.goods[3] += transfer;
        needed -= transfer;
    }
    assert!(needed < 0.001, "fixture needs existing rescue equipment");
    // Explicitly fund the second voyage too: unrelated institutional succession and
    // wage changes can leave the original sponsor unable to finance a rescue.
    // Transfer existing town cash into the council; do not mint fixture money.
    let controller = h.controller(origin as u32) as usize;
    for donor in 0..h.sites.len() {
        let treasury = h.society.as_ref().unwrap().councils[controller].treasury;
        let needed = (1201. - treasury).max(0.);
        let pool = &mut h.sites[donor].economy.finance[0];
        let before = *pool;
        *pool = (*pool as f64 - needed.min(*pool as f64)).max(0.) as f32;
        h.society.as_mut().unwrap().councils[controller].treasury += before as f64 - *pool as f64;
    }
    assert!(
        h.society.as_ref().unwrap().councils[controller].treasury >= 1200.,
        "fixture needs existing rescue capital"
    );
    let rescue = g
        .launch_expedition(route, Objective::Rescue, Some(id))
        .unwrap();
    g.advance_history(travel).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let x = h.expeditions.as_ref().unwrap();
    assert_eq!(x.voyages[id as usize].phase, Phase::Rescued);
    assert_eq!(x.voyages[rescue as usize].survivors(), 16);
    let identities: std::collections::BTreeSet<_> = x.voyages[rescue as usize]
        .crew
        .iter()
        .map(|c| c.person.unwrap())
        .collect();
    assert_eq!(identities.len(), 16);
    assert_eq!(h.person_duties.len(), 16);
    assert!(h.person_duties.values().all(|d| d.voyage == rescue));
    for (name, expertise) in rescued_expertise {
        assert_eq!(
            x.voyages[rescue as usize]
                .crew
                .iter()
                .find(|c| c.name == name)
                .unwrap()
                .expertise,
            expertise
        );
    }
    assert_eq!(x.voyages[rescue as usize].phase, Phase::Homeward);
    h.validate(&g.snapshot().unwrap()).unwrap();
    g.advance_history(travel + 1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert_eq!(
        h.expeditions.as_ref().unwrap().voyages[rescue as usize].phase,
        Phase::Returned
    );
    h.validate(&g.snapshot().unwrap()).unwrap();
    g.advance_history(60).unwrap();
    // Recall tests travel timing, not recovery of the local food surplus after
    // two voyages. Provision this third launch from existing neighboring food.
    let h = g.civilizations.as_mut().unwrap();
    let reserve = h.expeditions.as_ref().unwrap().rules.reserve_months;
    let required = 8. * 18. * (2 * travel + 6 + reserve) as f32
        + h.sites[origin].stocks.stock[0] * 18. * 12.
        + 100.;
    for donor in 0..h.sites.len() {
        if donor == origin {
            continue;
        }
        let needed = (required - h.sites[origin].stocks.stock[1]).max(0.);
        let before = h.sites[donor].stocks.stock[1];
        h.sites[donor].stocks.stock[1] -= needed.min(before);
        let removed = before - h.sites[donor].stocks.stock[1];
        h.sites[origin].stocks.stock[1] += removed;
    }
    assert!(h.sites[origin].stocks.stock[1] >= required);
    let controller = h.controller(origin as u32) as usize;
    for donor in 0..h.sites.len() {
        let treasury = h.society.as_ref().unwrap().councils[controller].treasury;
        let needed = (1201. - treasury).max(0.);
        let pool = &mut h.sites[donor].economy.finance[0];
        let before = *pool;
        *pool = (*pool as f64 - needed.min(*pool as f64)).max(0.) as f32;
        h.society.as_mut().unwrap().councils[controller].treasury += before as f64 - *pool as f64;
    }
    assert!(h.society.as_ref().unwrap().councils[controller].treasury >= 1200.);
    let next = g
        .launch_expedition(route, Objective::Ecology, None)
        .unwrap();
    g.advance_history(2).unwrap();
    g.recall_expedition(next).unwrap();
    let e = &g
        .civilizations
        .as_ref()
        .unwrap()
        .expeditions
        .as_ref()
        .unwrap()
        .voyages[next as usize];
    assert_eq!(e.phase, Phase::Homeward);
    assert!(e.due > g.civilizations.as_ref().unwrap().month);
    g.advance_history(3).unwrap();
    assert_eq!(
        g.civilizations
            .as_ref()
            .unwrap()
            .expeditions
            .as_ref()
            .unwrap()
            .voyages[next as usize]
            .phase,
        Phase::Returned
    );
}
#[test]
#[ignore = "requires hardware GPU"]
fn starvation_writes_off_losses_without_creating_resources() {
    let mut g = world();
    let id = launch(&mut g);
    let h = g.civilizations.as_mut().unwrap();
    let e = &mut h.expeditions.as_mut().unwrap().voyages[id as usize];
    h.sites[e.origin as usize].stocks.stock[1] += e.food;
    e.food = 0.;
    e.due = h.month + 100; // Move food home, preserving the fixture's inventory.
    g.advance_history(9).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let e = &h.expeditions.as_ref().unwrap().voyages[id as usize];
    assert_eq!(e.phase, Phase::Lost);
    assert_eq!(e.survivors(), 0);
    assert!(!e.confirmed);
    h.validate(&g.snapshot().unwrap()).unwrap();
}

#[test]
#[ignore = "requires hardware GPU"]
fn merchant_institution_pays_existing_escrow_and_receives_its_refund() {
    use ancient_world::culture::{Institution, InstitutionKind};
    let mut g = world();
    // Probe only to identify a port satisfying all the existing physical launch requirements.
    let baseline = g.civilizations.clone();
    let voyage = launch(&mut g);
    let e = &g
        .civilizations
        .as_ref()
        .unwrap()
        .expeditions
        .as_ref()
        .unwrap()
        .voyages[voyage as usize];
    let (origin, route) = (e.origin, e.route);
    g.civilizations = baseline;
    let h = g.civilizations.as_mut().unwrap();
    let leader = h.civilizations[h.sites[origin as usize].civilization as usize].leader;
    let c = h.culture.as_mut().unwrap();
    let institution = c.institutions.len() as u32;
    // Declared fixture starting capital, held once in the institution rather than town stocks.
    h.sites[origin as usize].economy.finance[1] += 1300.;
    c.institutions.push(Institution {
        capacity: None,
        id: institution,
        name: "Fixture expedition house".into(),
        kind: InstitutionKind::Merchant,
        site: origin,
        tradition: None,
        members: vec![leader],
        leader,
        treasury: 1300.,
        active: true,
        founded: h.month,
        knowledge: Default::default(),
        property: vec![],
        dues: 1300.,
        expenses: 0.,
    });
    let before = h.economy_residuals();
    let voyage = g.launch_expedition(route, Objective::Charts, None).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let e = &h.expeditions.as_ref().unwrap().voyages[voyage as usize];
    assert_eq!(e.institution, Some(institution));
    assert_eq!(e.purse, 600.);
    assert_eq!(
        h.culture.as_ref().unwrap().institutions[institution as usize].treasury,
        700.
    );
    for (a, b) in before.into_iter().zip(h.economy_residuals()) {
        assert!((a - b).abs() < 1e-6);
    }
    g.recall_expedition(voyage).unwrap();
    g.advance_history(2).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let e = &h.expeditions.as_ref().unwrap().voyages[voyage as usize];
    assert_eq!(e.phase, Phase::Returned);
    assert_eq!(e.purse, 0.);
    assert!(h.culture.as_ref().unwrap().institutions[institution as usize].treasury > 700.);
    h.validate(&g.snapshot().unwrap()).unwrap();
}

#[test]
#[ignore = "requires hardware GPU"]
fn heritage_voyages_preserve_minor_finds_and_checkpoint_continuity() {
    // This fixture requires a funded route to a specific find. Isolate that
    // precondition from caregiving competition for tool-production labor.
    let mut g = world_with_domestic(false);
    let eligible: Vec<_> = {
        let h = g.civilizations.as_ref().unwrap();
        h.expeditions
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                let cell = *r.cells.last().unwrap();
                let hash = cell
                    .wrapping_mul(747796405)
                    .wrapping_add(h.seed.wrapping_mul(2891336453));
                ((hash ^ (hash >> 16)) % 5 < 3).then_some(i as u32)
            })
            .collect()
    };
    let id = eligible
        .into_iter()
        .find_map(
            |r| match g.launch_expedition(r, Objective::PatronSearch, None) {
                Ok(id) => Some(id),
                Err(error) => {
                    eprintln!("patron route {r}: {error:#}");
                    None
                }
            },
        )
        .expect("eligible funded patron voyage");
    let h = g.civilizations.as_ref().unwrap();
    let e = &h.expeditions.as_ref().unwrap().voyages[id as usize];
    assert!(e.heritage.as_ref().unwrap().patron.is_some());
    for objective in [Objective::Inscriptions, Objective::OldLiterature] {
        assert!(
            ancient_world::expedition_heritage::charter(h, e.origin, objective)
                .unwrap()
                .is_some()
        );
    }
    let mut old = serde_json::to_value(e).unwrap();
    old.as_object_mut().unwrap().remove("heritage");
    let old: ancient_world::expeditions::Expedition = serde_json::from_value(old).unwrap();
    assert!(old.heritage.is_none());
    let duration = h.expeditions.as_ref().unwrap().routes[e.route as usize].travel_months * 2 + 10;
    let file = std::env::temp_dir().join(format!("heritage-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    g.advance_history(duration).unwrap();
    for _ in 0..duration {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    let e = &h.expeditions.as_ref().unwrap().voyages[id as usize];
    assert_eq!(e.phase, Phase::Returned);
    let find = e
        .heritage
        .as_ref()
        .unwrap()
        .find
        .as_ref()
        .expect("fragment from qualifying endpoint");
    let artifact = &h.culture.as_ref().unwrap().artifacts[find.artifact.unwrap() as usize];
    assert_eq!(artifact.kind, "ancient ceramic fragment");
    assert_eq!(artifact.materials.iter().map(|v| v.1).sum::<f32>(), 0.125);
    assert!(
        artifact.topic.is_none(),
        "minor artifact does not grant a technology"
    );
    assert!(h
        .culture
        .as_ref()
        .unwrap()
        .accounts
        .iter()
        .any(|a| a.facts.contains(&find.observed)));
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 1e-3));
    h.validate(&g.snapshot().unwrap()).unwrap();
    let mut duplicate = h.clone();
    let x = duplicate.expeditions.as_mut().unwrap();
    let mut copy = x.voyages[id as usize].clone();
    copy.id = x.voyages.len() as u32;
    x.voyages.push(copy);
    assert!(
        duplicate.validate(&g.snapshot().unwrap()).is_err(),
        "duplicate recovery must fail validation"
    );
}

#[test]
#[ignore = "requires hardware GPU"]
fn named_casualty_and_crew_remittances_reach_people_and_households() {
    let mut g = world();
    // Occupy the known workers for this month. Recruitment may record genuinely
    // unnamed cohort adults, but may not duplicate the already committed people.
    let h = g.civilizations.as_mut().unwrap();
    let p = h.participation.as_mut().unwrap();
    let available: Vec<_> = p
        .residents
        .values()
        .filter_map(|r| {
            if let ancient_world::participation::Presence::Resident(site) = r.presence {
                Some((r.person, site))
            } else {
                None
            }
        })
        .collect();
    for (person, site) in available {
        if let Some(id) = p.reserve(
            h.month,
            site,
            ancient_world::participation::Activity::Culture,
            &[person],
            p.available(person),
        ) {
            p.settle(id, 0.).unwrap();
        }
    }
    let id = launch(&mut g);
    let h = g.civilizations.as_mut().unwrap();
    let e = &mut h.expeditions.as_mut().unwrap().voyages[id as usize];
    let origin = e.origin as usize;
    assert!(e.crew.iter().any(|c| c.identified_from_cohort));
    // Conserved intervention: return the provisions to shore, stranding a hungry crew.
    h.sites[origin].stocks.stock[1] += e.food;
    e.food = 0.;
    e.phase = Phase::Stranded;
    e.due = h.month + 100;
    let roster: Vec<_> = e.crew.iter().map(|c| c.person.unwrap()).collect();
    let deaths_before = h.sites[origin].stocks.people[1];
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let e = &h.expeditions.as_ref().unwrap().voyages[id as usize];
    assert_eq!(e.survivors(), 7);
    let dead = e.crew.iter().find(|c| !c.alive).unwrap().person.unwrap();
    assert_eq!(h.people[dead as usize].died, Some(h.month));
    assert!(!h.person_duties.contains_key(&dead));
    assert_eq!(
        h.person_presence(dead).1,
        ancient_world::participation::Presence::Dead
    );
    assert!(h.events.iter().any(
        |ev| ev.kind == "expedition_casualty" && ev.subjects.contains(&("person".into(), dead))
    ));
    assert!(h.sites[origin].stocks.people[1] >= deaths_before + 1.);
    for person in roster.into_iter().filter(|p| *p != dead) {
        assert_eq!(
            h.person_presence(person).1,
            ancient_world::participation::Presence::Expedition(id)
        );
        let household = h.person_duties[&person].household.unwrap();
        if let Some(account) = h.household_account(household) {
            assert!(account.wages >= 20. / 7.);
        }
    }
    h.validate(&g.snapshot().unwrap()).unwrap();
}
