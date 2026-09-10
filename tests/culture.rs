use ancient_world::{
    catalog::Catalog,
    config::Config,
    culture::{FoundingOptions, PatronCatalog},
    economy::EconomyCatalog,
    gpu::{ContextGpu, Generator},
};
#[test]
fn patron_catalog_and_inverse_service_are_bounded() {
    let c = PatronCatalog::bundled().unwrap();
    assert_eq!(c.patrons.iter().filter(|p| p.kind == "animal").count(), 8);
    assert_eq!(
        c.patrons.iter().filter(|p| p.kind == "intelligent").count(),
        4
    );
    let o = FoundingOptions::default();
    for v in [0.85, 1., 1.15] {
        assert!(o.duration(false, 1., v) < o.duration(false, 0., v));
        assert!(o.duration(true, 1., v) < o.duration(true, 0., v));
    }
    let mut bad = o;
    bad.variance[0] = f32::NAN;
    assert!(bad.validate().is_err());
}
#[test]
fn expanded_materials_reject_duplicate_ids_and_created_nutrients() {
    let c = EconomyCatalog::bundled().unwrap();
    assert_eq!(c.goods.len(), 64);
    assert_eq!(c.agriculture.as_ref().unwrap().crops.len(), 6);
    let mut bad = c.clone();
    bad.goods[9].id = bad.goods[8].id.clone();
    assert!(bad.validate().is_err());
    let mut bad = c;
    bad.recipes[5].output[14] = 100.;
    assert!(bad.validate().is_err());
}
#[test]
fn nitrogen_fixation_catalog_preserves_legacy_cost_and_rejects_free_energy() {
    let e = EconomyCatalog::bundled().unwrap();
    let a = e.agriculture.as_ref().unwrap();
    assert_eq!(a.fixation_cost_kg, 12.);
    assert_eq!(a.gpu(&e)[12][2], 12.);
    let mut old = serde_json::to_value(a).unwrap();
    old.as_object_mut().unwrap().remove("fixation_cost_kg");
    let old: ancient_world::agriculture::AgricultureCatalog = serde_json::from_value(old).unwrap();
    assert_eq!(old.fixation_cost_kg, 80.);
    for cost in [0., -1., f32::NAN, f32::INFINITY] {
        let mut bad = a.clone();
        bad.fixation_cost_kg = cost;
        assert!(bad.validate(&e).is_err());
    }
}
fn world() -> Generator {
    Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            seed: 17,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap()
}
#[test]
#[ignore = "requires a hardware GPU"]
fn patron_departures_and_cultural_continuation_match() {
    let mut g = world();
    g.run_epochs(1).unwrap();
    g.found_civilizations(5).unwrap();
    g.enable_society().unwrap();
    g.enable_politics().unwrap();
    g.advance_history(120).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let c = h.culture.as_ref().unwrap();
    assert_eq!(c.patrons.len(), 5);
    assert!(c.patrons.iter().all(|p| p.departed.is_some()));
    assert!(c.accounts.len() >= 10);
    assert!(c
        .accounts
        .iter()
        .all(|a| a.author.is_some() || a.institution.is_some()));
    assert!(
        c.agents
            .iter()
            .filter(|a| h.people[a.person as usize].born > 0)
            .all(|a| a.knowledge.is_empty()),
        "children cannot inherit knowledge without transmission"
    );
    assert!(c.agents.iter().any(|a| a.knowledge.len() > 1));
    let effort = c.patrons.iter().map(|p| p.effort).collect::<Vec<_>>();
    let mut legacy = serde_json::to_value(c).unwrap();
    legacy.as_object_mut().unwrap().remove("religious_relief");
    let legacy: ancient_world::culture::Culture = serde_json::from_value(legacy).unwrap();
    assert!(!legacy.religious_relief.enabled);
    assert!(legacy.religious_relief.missions.is_empty());
    g.civilizations
        .as_mut()
        .unwrap()
        .set_religious_relief(true)
        .unwrap();
    let file = std::env::temp_dir().join(format!("patrons-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    g.advance_history(36).unwrap();
    for _ in 0..36 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    assert_eq!(
        effort,
        g.cultural_history()
            .unwrap()
            .patrons
            .iter()
            .map(|p| p.effort)
            .collect::<Vec<_>>()
    );
}
#[test]
#[ignore = "requires a hardware GPU"]
fn aid_control_does_not_change_arrival_inventory_or_theology() {
    let mut a = world();
    a.found_civilizations(5).unwrap();
    let mut b = world();
    b.found_civilizations_with_options(
        5,
        FoundingOptions {
            aid_enabled: false,
            ..Default::default()
        },
    )
    .unwrap();
    let ac = a.cultural_history().unwrap();
    let bc = b.cultural_history().unwrap();
    assert_eq!(
        ac.traditions.iter().map(|t| t.themes).collect::<Vec<_>>(),
        bc.traditions.iter().map(|t| t.themes).collect::<Vec<_>>()
    );
    assert_eq!(
        a.civilizations.as_ref().unwrap().nutrition_initial,
        b.civilizations.as_ref().unwrap().nutrition_initial
    );
    b.advance_history(96).unwrap();
    assert!(b
        .cultural_history()
        .unwrap()
        .patrons
        .iter()
        .all(|p| p.effort == [0.; 4] && p.departed.is_some()));
}
#[test]
#[ignore = "requires a hardware GPU"]
fn corrupt_patron_and_artifact_references_are_rejected() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let cells = g.snapshot().unwrap();
    let mut h = g.civilizations.clone().unwrap();
    h.culture.as_mut().unwrap().patrons[0].provisions_kg += 1.;
    assert!(h.validate(&cells).is_err());
    let mut h = g.civilizations.clone().unwrap();
    h.culture.as_mut().unwrap().artifacts[0].site = Some(u32::MAX);
    assert!(h.validate(&cells).is_err());
    let mut h = g.civilizations.clone().unwrap();
    h.events[0].subjects.push(("patron".into(), u32::MAX));
    assert!(h.validate(&cells).is_err());
}

#[test]
#[ignore = "requires a hardware GPU"]
fn artifact_destruction_is_once_only_and_preserves_balances() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let before = g.civilizations.as_ref().unwrap().economy_residuals();
    let event_count = g.civilizations.as_ref().unwrap().events.len();
    g.destroy_artifact(0).unwrap();
    assert!(g.destroy_artifact(0).is_err());
    assert!(g.dedicate_artifact(0, 0).is_err());
    let h = g.civilizations.as_ref().unwrap();
    assert_eq!(h.events.len(), event_count + 1);
    let a = &h.culture.as_ref().unwrap().artifacts[0];
    assert!(a.destroyed && a.custodian.is_none());
    assert_eq!(a.events.len(), 2);
    for (before, after) in before.into_iter().zip(h.economy_residuals()) {
        assert!((before - after).abs() < 1e-6);
    }
    h.validate(&g.snapshot().unwrap()).unwrap();
}

#[test]
fn legacy_goods_arrays_keep_quantities_when_padded() {
    let e = ancient_world::economy::Economy::default();
    let mut json = serde_json::to_value(e).unwrap();
    for field in ["goods", "made", "used", "initial", "prices"] {
        json[field] = serde_json::json!([1., 2., 3., 4., 5., 6., 7., 8.]);
    }
    for field in ["management", "crops", "herds", "agriculture"] {
        json.as_object_mut().unwrap().remove(field);
    }
    let e: ancient_world::economy::Economy = serde_json::from_value(json).unwrap();
    assert_eq!(&e.goods[..8], &[1., 2., 3., 4., 5., 6., 7., 8.]);
    assert!(e.goods[8..].iter().all(|&v| v == 0.));
    assert_eq!(e.management[0], 0.);
    assert_eq!(e.herds, [[0.; 4]; 3]);
}

#[test]
#[ignore = "requires a hardware GPU"]
fn political_control_does_not_convert_households() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    g.enable_society().unwrap();
    g.enable_politics().unwrap();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    let faith = h.culture.as_ref().unwrap().household_faith.clone();
    let site_faith = h.culture.as_ref().unwrap().site_faith.clone();
    let old = h.politics.as_ref().unwrap().controllers[0];
    h.politics.as_mut().unwrap().controllers[0] = (old + 1) % h.civilizations.len() as u32;
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert_ne!(h.politics.as_ref().unwrap().controllers[0], old);
    assert_eq!(h.culture.as_ref().unwrap().household_faith, faith);
    assert_eq!(h.culture.as_ref().unwrap().site_faith, site_faith);
}

#[test]
#[ignore = "requires a hardware GPU"]
fn failed_history_transaction_preserves_old_events_and_state() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    // Force end-of-transaction cultural validation to fail after new events were appended.
    let h = g.civilizations.as_mut().unwrap();
    h.culture.as_mut().unwrap().patrons[0].origin = h.sites[0].cell;
    let before = serde_json::to_vec(&g.civilizations).unwrap();
    assert!(g.advance_history(6).is_err());
    assert_eq!(before, serde_json::to_vec(&g.civilizations).unwrap());
}

#[test]
#[ignore = "requires a hardware GPU"]
fn artifact_sale_transfers_existing_money_and_rejects_disputed_title() {
    use ancient_world::culture::{Institution, InstitutionKind, Owner};
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    let leader = h.civilizations[0].leader;
    h.sites[0].economy.finance[0] -= 25.;
    h.culture.as_mut().unwrap().institutions.push(Institution {
        capacity: None,
        id: 0,
        name: "Fixture library".into(),
        kind: InstitutionKind::Scholarly,
        site: 0,
        tradition: None,
        members: vec![leader],
        leader,
        treasury: 25.,
        active: true,
        founded: 0,
        knowledge: Default::default(),
        property: vec![],
        dues: 25.,
        expenses: 0.,
    });
    let before = h.economy_residuals();
    let original = serde_json::to_vec(&g.civilizations).unwrap();
    assert!(g.sell_artifact(0, 0, 26.).is_err());
    assert!(g.sell_artifact(0, 0, f64::NAN).is_err());
    assert_eq!(original, serde_json::to_vec(&g.civilizations).unwrap());
    g.civilizations
        .as_mut()
        .unwrap()
        .culture
        .as_mut()
        .unwrap()
        .artifacts[0]
        .claims
        .push(Owner::Person(leader));
    assert!(g.sell_artifact(0, 0, 10.).is_err());
    g.civilizations
        .as_mut()
        .unwrap()
        .culture
        .as_mut()
        .unwrap()
        .artifacts[0]
        .claims
        .clear();
    g.sell_artifact(0, 0, 10.).unwrap();
    assert!(g.sell_artifact(0, 0, 10.).is_err());
    let h = g.civilizations.as_ref().unwrap();
    let c = h.culture.as_ref().unwrap();
    assert_eq!(c.institutions[0].treasury, 15.);
    assert_eq!(c.institutions[0].property, vec![0]);
    assert_eq!(c.artifacts[0].owner, Owner::Institution(0));
    assert_eq!(c.artifacts[0].site, Some(0));
    assert!(c.artifacts[0].custodian.is_none());
    for (a, b) in before.into_iter().zip(h.economy_residuals()) {
        assert!((a - b).abs() < 1e-6);
    }
    h.validate(&g.snapshot().unwrap()).unwrap();
}

#[test]
#[ignore = "requires a hardware GPU"]
fn supported_schism_keeps_patron_ancestry_and_records_its_human_author() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    g.enable_society().unwrap();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    // Controlled cultural fixture: a mature shared tradition with sustained dissent.
    h.month = 251;
    let c = h.culture.as_mut().unwrap();
    c.site_faith.fill(0);
    c.household_faith.fill(0);
    c.traditions[0].dissent = 0.8;
    for a in &mut c.agents {
        a.traits[0] = 1.;
        a.traits[2] = 1.;
    }
    let count = c.traditions.len();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let c = h.culture.as_ref().unwrap();
    assert_eq!(c.traditions.len(), count + 1);
    let child = c.traditions.last().unwrap();
    assert_eq!(child.parent, Some(0));
    assert_eq!(child.patron, c.traditions[0].patron);
    assert_ne!(child.themes, c.traditions[0].themes);
    let account = c.accounts.iter().find(|a| a.tradition == child.id).unwrap();
    assert_eq!(account.author, Some(child.leader));
    assert!(account
        .facts
        .iter()
        .any(|&id| h.events[id as usize].kind == "religious_schism"));
    h.validate(&g.snapshot().unwrap()).unwrap();
}

#[test]
#[ignore = "requires a hardware GPU"]
fn custodian_death_does_not_transfer_another_persons_title() {
    use ancient_world::culture::Owner;
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    h.people[1].died = Some(0);
    let mut successor = h.people[1].clone();
    successor.id = h.people.len() as u32;
    successor.died = None;
    successor.predecessor = Some(1);
    h.civilizations[1].leader = successor.id;
    h.people.push(successor);
    let a = &mut h.culture.as_mut().unwrap().artifacts[0];
    a.owner = Owner::Person(2);
    a.custodian = Some(1);
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let a = &h.culture.as_ref().unwrap().artifacts[0];
    assert_eq!(a.owner, Owner::Person(2));
    assert_ne!(a.custodian, Some(1));
    h.validate(&g.snapshot().unwrap()).unwrap();
}
