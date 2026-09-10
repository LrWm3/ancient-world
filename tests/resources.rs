use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
    grid,
};

#[test]
#[ignore = "requires hardware GPU"]
fn shared_sources_connect_mining_surveys_markets_and_checkpoints() {
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
    let mut c = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    c.production.enabled = false; // Controlled fixed extraction demand, no market-order confound.
    g.configure_economy(c).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.reserves[1] = 40.;
        s.economy.reserves[2] = 40.;
    }
    let legacy = serde_json::to_value(g.civilizations.as_ref().unwrap()).unwrap();
    let mut old = legacy.clone();
    old.as_object_mut().unwrap().remove("resources");
    let old: ancient_world::civilization::History = serde_json::from_value(old).unwrap();
    assert!(old.resources.is_none());
    g.enable_shared_resources().unwrap();
    let sources = g.civilizations.as_ref().unwrap().resources.clone().unwrap();
    assert_eq!(
        sources.sources.values().map(|s| s.initial[0]).sum::<f64>(),
        200.
    );
    let path = std::env::temp_dir().join(format!("resource-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut closed = Generator::load(g.gpu.clone(), &path).unwrap();
    let mut batched = Generator::load(g.gpu.clone(), &path).unwrap();
    for site in 0..5 {
        closed.set_mine_closed(site, true).unwrap();
    }
    assert!(closed.set_mine_closed(999, true).is_err());
    assert_eq!(
        serde_json::to_value(
            &closed
                .civilizations
                .as_ref()
                .unwrap()
                .resources
                .as_ref()
                .unwrap()
                .sources
        )
        .unwrap(),
        serde_json::to_value(&sources.sources).unwrap()
    );
    assert!(closed
        .civilizations
        .as_ref()
        .unwrap()
        .supplier_unit_cost(0, 1)
        .is_none());
    for _ in 0..12 {
        g.advance_history(1).unwrap();
    }
    batched.advance_history(12).unwrap();
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&batched.civilizations).unwrap()
    );
    closed.advance_history(12).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let r = h.resources.as_ref().unwrap();
    assert!(r.sources.values().map(|s| s.extracted[0]).sum::<f64>() > 0.);
    assert!(r.residual() < 1e-8);
    assert!(closed
        .civilizations
        .as_ref()
        .unwrap()
        .resources
        .as_ref()
        .unwrap()
        .sources
        .values()
        .all(|s| s.extracted == [0.; 2]));
    let ore_made = |g: &Generator| {
        g.civilizations
            .as_ref()
            .unwrap()
            .sites
            .iter()
            .map(|s| s.economy.made[1])
            .sum::<f32>()
    };
    assert!(ore_made(&g) > ore_made(&closed));
    let cell = h.sites[0].cell;
    let region = g
        .generate_region(grid::cell_direction(cell, 32), 10., 16)
        .unwrap();
    let source = region
        .resource_sources
        .iter()
        .find(|s| s.cell == cell)
        .unwrap();
    assert_eq!(source.remaining, r.sources[&cell].remaining);
    assert_eq!(region.history_month, Some(h.month));
    for site in 0..5 {
        closed.set_mine_closed(site, false).unwrap();
    }
    closed.advance_history(12).unwrap();
    assert!(ore_made(&closed) > 0.);
    assert!(closed
        .civilizations
        .as_ref()
        .unwrap()
        .economy_residuals()
        .iter()
        .all(|v| v.abs() < 0.001));
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires hardware GPU"]
fn regional_work_limits_control_gpu_extraction_and_retire_without_replenishment() {
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
    let mut catalog = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    catalog.production.enabled = false;
    g.configure_economy(catalog).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.reserves[1..3].fill(40.);
    }
    g.enable_shared_resources().unwrap();
    let cell = g.civilizations.as_ref().unwrap().sites[0].cell;
    let region = g
        .generate_region(grid::cell_direction(cell, 32), 10., 16)
        .unwrap();
    let before = g
        .civilizations
        .as_ref()
        .unwrap()
        .resources
        .as_ref()
        .unwrap()
        .sources[&cell]
        .remaining;
    assert!(g.activate_regional_mine(&region, 999, [1.; 2]).is_err());
    assert!(g
        .activate_regional_mine(&region, 0, [f32::NAN, 1.])
        .is_err());
    let mut altered: ancient_world::region::Region =
        serde_json::from_value(serde_json::to_value(&region).unwrap()).unwrap();
    altered
        .resource_sources
        .iter_mut()
        .find(|s| s.cell == cell)
        .unwrap()
        .remaining[0] += 1.;
    assert!(g.activate_regional_mine(&altered, 0, [1.; 2]).is_err());
    g.activate_regional_mine(&region, 0, [0.1, 0.2]).unwrap();
    assert!(g.activate_regional_mine(&region, 0, [1.; 2]).is_err());
    let path = std::env::temp_dir().join(format!("regional-mine-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    for _ in 0..12 {
        g.advance_history(1).unwrap();
    }
    resumed.advance_history(12).unwrap();
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    let r = h.resources.as_ref().unwrap();
    let used = r.regional_mines[&cell].extracted;
    assert!(used[0] > 0. && used[0] <= 1.200001);
    assert!(used[1] > 0. && used[1] <= 2.400001);
    for k in 0..2 {
        assert!((before[k] - r.sources[&cell].remaining[k] - used[k]).abs() < 1e-6);
    }
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    let updated = g
        .generate_region(grid::cell_direction(cell, 32), 10., 16)
        .unwrap();
    assert_eq!(updated.regional_mines[0].extracted, used);
    g.set_regional_mining_limit(0, [0.; 2]).unwrap();
    g.advance_history(1).unwrap();
    assert_eq!(
        g.civilizations
            .as_ref()
            .unwrap()
            .resources
            .as_ref()
            .unwrap()
            .regional_mines[&cell]
            .extracted,
        used
    );
    let remaining = g
        .civilizations
        .as_ref()
        .unwrap()
        .resources
        .as_ref()
        .unwrap()
        .sources[&cell]
        .remaining;
    g.retire_regional_mine(0).unwrap();
    let r = g
        .civilizations
        .as_ref()
        .unwrap()
        .resources
        .as_ref()
        .unwrap();
    assert!(r.regional_mines.is_empty());
    assert_eq!(r.retired_regional_mines[0].extracted, used);
    assert_eq!(r.retired_regional_mines[0].retired_month, Some(13));
    assert_eq!(
        g.civilizations
            .as_ref()
            .unwrap()
            .resources
            .as_ref()
            .unwrap()
            .sources[&cell]
            .remaining,
        remaining
    );
    assert!(g.activate_regional_mine(&region, 0, [1.; 2]).is_err()); // old boundary
    assert!(g.set_regional_mining_limit(0, [1.; 2]).is_err()); // retired control
    g.advance_history(1).unwrap();
    assert!(
        g.civilizations
            .as_ref()
            .unwrap()
            .resources
            .as_ref()
            .unwrap()
            .sources[&cell]
            .remaining[0]
            < remaining[0]
    );
    std::fs::remove_file(path).unwrap();
}
