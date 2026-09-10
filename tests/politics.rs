use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
fn world() -> Generator {
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.found_civilizations(16).unwrap();
    g.enable_society().unwrap();
    g.enable_politics().unwrap();
    g
}
#[test]
#[ignore = "requires hardware GPU"]
fn genealogy_factions_and_claims_persist_for_a_century() {
    let mut g = world();
    let initial = g.civilizations.as_ref().unwrap().politics.as_ref().unwrap();
    assert!(initial
        .claims
        .iter()
        .any(|c| initial.claim_owners(c).len() > 1));
    g.advance_history(1200).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let p = h.politics.as_ref().unwrap();
    assert!(!p.marriages.is_empty());
    assert!(p.kin.iter().any(|k| k.parents[0].is_some()));
    assert!(
        p.kin.iter().any(|k| h.people[k.person as usize]
            .predecessor
            .is_some_and(|v| k.parents.contains(&Some(v)))),
        "inheritance should select a recorded child"
    );
    assert!(h.events.iter().any(|e| e.kind == "faction_shift"));
    // Conquest can unite rival owners; the overlapping site claims still persist.
    assert!(p.claims.iter().any(|c| c.sites.len() > 1));
    let cells = g.snapshot().unwrap();
    h.validate(&cells).unwrap();
    let mut bad = h.clone();
    let child = bad
        .politics
        .as_mut()
        .unwrap()
        .kin
        .iter_mut()
        .find(|k| k.parents[0].is_some())
        .unwrap();
    child.parents[0] = Some(child.person);
    assert!(bad.validate(&cells).is_err());
    let mut bad = h.clone();
    bad.politics.as_mut().unwrap().controllers[0] = 999;
    assert!(bad.validate(&cells).is_err());
    let file = std::env::temp_dir().join(format!("politics-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut b = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    g.advance_history(24).unwrap();
    for _ in 0..24 {
        b.advance_history(1).unwrap();
    }
    assert!(
        serde_json::to_vec(&g.civilizations).unwrap()
            == serde_json::to_vec(&b.civilizations).unwrap(),
        "political checkpoint diverged"
    );
}
#[test]
#[ignore = "requires hardware GPU"]
fn territorial_campaigns_transfer_control_not_resources_or_ancestry() {
    let mut g = world();
    let h = g.civilizations.as_ref().unwrap();
    let route = h
        .society
        .as_ref()
        .unwrap()
        .routes
        .iter()
        .find(|r| r.cost_km < 900. && h.controller(r.from) != h.controller(r.to))
        .unwrap()
        .clone();
    let original = h.sites[route.to as usize].civilization;
    let before = serde_json::to_vec(&g.civilizations).unwrap();
    assert!(g.declare_war(route.from, route.from).is_err());
    assert_eq!(before, serde_json::to_vec(&g.civilizations).unwrap());
    let id = g.declare_war(route.from, route.to).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert_eq!(id, 0);
    assert!(h.route_cost(route.from, route.to).is_none());
    assert!(h.society.as_ref().unwrap().raids[0].equipment > 0.);
    let file = std::env::temp_dir().join(format!("war-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut b = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    g.advance_history(18).unwrap();
    for _ in 0..18 {
        b.advance_history(1).unwrap();
    }
    assert!(
        serde_json::to_vec(&g.civilizations).unwrap()
            == serde_json::to_vec(&b.civilizations).unwrap(),
        "war checkpoint diverged"
    );
    let h = g.civilizations.as_ref().unwrap();
    assert_eq!(h.sites[route.to as usize].civilization, original);
    assert_eq!(h.politics.as_ref().unwrap().wars[0].outcome, "conquest");
    assert_eq!(h.controller(route.to), h.controller(route.from));
    assert!(h.food_residual().abs() < 0.001);
    assert!(h.population_residual().abs() < 0.001);
    assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
    assert!(h
        .events
        .iter()
        .any(|e| e.kind == "peace" && !e.causes.is_empty()));
    assert!(g.declare_war(route.from, route.to).is_err());
}

#[test]
#[ignore = "requires hardware GPU"]
fn insufficient_supplies_and_defeat_do_not_grant_territory() {
    let mut g = world();
    let h = g.civilizations.as_ref().unwrap();
    let route = h
        .society
        .as_ref()
        .unwrap()
        .routes
        .iter()
        .find(|r| r.cost_km < 900. && h.controller(r.from) != h.controller(r.to))
        .unwrap()
        .clone();
    g.set_route_open(route.id, false).unwrap();
    assert!(g.declare_war(route.from, route.to).is_err());
    g.set_route_open(route.id, true).unwrap();
    let saved = g.civilizations.clone();
    let site = &mut g.civilizations.as_mut().unwrap().sites[route.from as usize];
    let loss = site.economy.goods[3];
    site.economy.goods[3] = 0.;
    site.economy.used[3] += loss;
    site.economy.reserves[3] += loss;
    let before = serde_json::to_vec(&g.civilizations).unwrap();
    assert!(g.declare_war(route.from, route.to).is_err());
    assert_eq!(before, serde_json::to_vec(&g.civilizations).unwrap());
    g.civilizations = saved;
    let site = &mut g.civilizations.as_mut().unwrap().sites[route.from as usize];
    let loss = site.economy.goods[3] - 3.;
    site.economy.goods[3] = 3.;
    site.economy.used[3] += loss;
    site.economy.reserves[3] += loss;
    let owner = g.civilizations.as_ref().unwrap().controller(route.to);
    g.declare_war(route.from, route.to).unwrap();
    g.advance_history(18).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert_eq!(h.controller(route.to), owner);
    assert_eq!(h.politics.as_ref().unwrap().wars[0].outcome, "repulsed");
    assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
}

#[test]
#[ignore = "requires hardware GPU"]
fn competing_campaigns_withdraw_when_the_objective_changes_owner() {
    let mut g = world();
    let h = g.civilizations.as_ref().unwrap();
    let (target, sources) = (0..h.sites.len() as u32)
        .find_map(|t| {
            let sources: Vec<_> = h
                .society
                .as_ref()
                .unwrap()
                .routes
                .iter()
                .filter(|r| r.cost_km < 1500. && (r.from == t || r.to == t))
                .map(|r| if r.from == t { r.to } else { r.from })
                .collect();
            (sources.len() >= 2).then_some((t, sources))
        })
        .expect("fixture needs a shared objective");
    g.declare_war(sources[0], target).unwrap();
    g.declare_war(sources[1], target).unwrap();
    g.advance_history(24).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let p = h.politics.as_ref().unwrap();
    assert_eq!(
        p.wars.iter().filter(|w| w.outcome == "conquest").count(),
        1,
        "{:?}",
        p.wars
    );
    assert_eq!(
        p.wars.iter().filter(|w| w.outcome == "withdrawn").count(),
        1
    );
    assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
    assert!(h.population_residual().abs() < 0.001);
}

#[test]
#[ignore = "requires hardware GPU; small faction seed comparison"]
fn expanded_interests_seed_comparison() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    for seed in [17, 81, 256] {
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                seed,
                resolution: 64,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.found_civilizations(8).unwrap();
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.advance_history(360).unwrap();
        let h = g.civilizations.as_ref().unwrap();
        let p = h.politics.as_ref().unwrap();
        let mut memberships = [0; ancient_world::faction_interests::COUNT];
        for id in &p.household_factions {
            memberships[p.factions[*id as usize].interest as usize] += 1;
        }
        println!(
            "seed={seed} households_by_interest={memberships:?} shifts={} fragmentations={}",
            h.events
                .iter()
                .filter(|e| e.kind == "faction_shift")
                .count(),
            h.events
                .iter()
                .filter(|e| e.kind == "faction_fragmentation")
                .count()
        );
        assert_eq!(
            p.factions.len(),
            h.civilizations.len() * ancient_world::faction_interests::COUNT
        );
        h.validate(&g.snapshot().unwrap()).unwrap();
    }
}
