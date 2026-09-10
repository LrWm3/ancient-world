use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
fn world() -> Generator {
    Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap()
}
#[test]
#[ignore = "requires hardware GPU"]
fn seasons_households_and_inheritance_conserve_and_resume() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    g.enable_society().unwrap();
    let initial = g.civilizations.as_ref().unwrap().sites[0].stocks.stock[0];
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    for s in &h.sites {
        let d = s.demography;
        assert!((d.ration_eaten[3] - s.stocks.ledger[1]).abs() < 0.01);
        assert!((d.ration_eaten[..3].iter().sum::<f32>() - d.ration_eaten[3]).abs() < 0.01);
    }
    // Crops now have separate calendars; dietary output can also come from stored seed or fish.
    assert!(h.sites[0].economy.crops.iter().map(|c| c[1]).sum::<f32>() > 0.);
    assert!(h.sites[0].demography.ages[1] < initial);
    g.advance_history(1199).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.events.iter().any(|e| e.kind == "inheritance"));
    assert!(h.events.iter().any(|e| e.kind == "succession"));
    assert!(h.events.iter().any(|e| e.kind == "harvest"));
    assert!(h.food_residual().abs() < 0.001);
    assert!(h.population_residual().abs() < 0.001);
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    assert!(g.set_ration_priority(0, [f32::NAN, 0., 0.]).is_err());
    assert!(g.set_ration_priority(u32::MAX, [0.; 3]).is_err());
    g.set_ration_priority(0, [3., 0., 3.]).unwrap();
    let file = std::env::temp_dir().join(format!("society-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut b = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    g.advance_history(12).unwrap();
    for _ in 0..12 {
        b.advance_history(1).unwrap();
    }
    assert!(
        serde_json::to_vec(&g.civilizations).unwrap()
            == serde_json::to_vec(&b.civilizations).unwrap(),
        "checkpoint continuation diverged"
    );
}
#[test]
#[ignore = "requires hardware GPU"]
fn terrain_routes_are_contiguous_and_closures_are_recorded() {
    let mut g = world();
    g.found_civilizations(16).unwrap();
    g.enable_society().unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let r = h.society.as_ref().unwrap().routes.first().unwrap().clone();
    assert!(r.cells.len() > 1);
    assert!(h.route_cost(r.from, r.to).is_some());
    g.set_route_open(r.id, false).unwrap();
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .route_cost(r.from, r.to)
        .is_none());
    g.advance_history(12).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    h.validate(&g.snapshot().unwrap()).unwrap();
    assert!(h.events.iter().any(|e| e.kind == "route_policy"));
    let mut invalid = h.clone();
    invalid.events.last_mut().unwrap().causes.push(u64::MAX);
    assert!(invalid.validate(&g.snapshot().unwrap()).is_err());
}

#[test]
fn route_passability_requires_both_political_and_physical_access() {
    let mut route = ancient_world::society::Route {
        id: 0,
        from: 0,
        to: 1,
        cells: vec![],
        cost_km: 10.,
        open: true,
        flood_months: 0,
        road_bricks: 0.,
        upkeep: None,
    };
    for open in [false, true] {
        for flood_months in [0, 1, 12] {
            route.open = open;
            route.flood_months = flood_months;
            assert_eq!(route.passable(), open && flood_months == 0);
        }
    }
}
