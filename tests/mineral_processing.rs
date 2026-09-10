use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
#[test]
#[ignore = "requires hardware GPU"]
fn identified_iron_ores_keep_identity_and_have_finite_processing_yields() {
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
    let mut terrain = g.snapshot().unwrap();
    let ids = [
        "hematite",
        "magnetite",
        "limonite",
        "bituminous_coal",
        "apatite",
    ];
    for (s, id) in g.civilizations.as_ref().unwrap().sites.iter().zip(ids) {
        terrain[s.cell as usize].meta[1] =
            g.catalog.minerals.iter().position(|m| m.id == id).unwrap() as u32;
    }
    g.restore_cells(&terrain, 0).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.reserves[1] = 40.;
        s.economy.goods[6] += 100.;
        s.economy.initial[6] += 100.;
        s.economy.external[0] += 100.;
    }
    g.enable_shared_resources().unwrap();
    g.enable_mineral_processing().unwrap();
    let mut c = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    let mut full_catalog = c.clone();
    full_catalog.production.enabled = true;
    full_catalog.production.adaptive_labor = true;
    c.production.enabled = false;
    c.recipes
        .retain(|r| r.input[32] + r.input[33] + r.input[34] > 0.);
    g.configure_economy(c.clone()).unwrap();
    let path =
        std::env::temp_dir().join(format!("mineral-processing-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    let mut planned = Generator::load(g.gpu.clone(), &path).unwrap();
    planned.configure_economy(full_catalog).unwrap();
    planned.advance_history(24).unwrap();
    assert!(planned
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .any(|s| s.economy.made[3] > 0.));
    assert!(planned
        .civilizations
        .as_ref()
        .unwrap()
        .resources
        .as_ref()
        .unwrap()
        .sources
        .values()
        .any(|s| s.extracted[0] > 0.));
    g.advance_history(12).unwrap();
    for _ in 0..12 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    let r = h.resources.as_ref().unwrap();
    for (i, s) in h.sites.iter().enumerate() {
        let source = &r.sources[&s.cell];
        assert_eq!(source.mineral.as_deref(), Some(ids[i]));
        assert_eq!(s.economy.made[1], 0.);
        if i < 3 {
            let good = 32 + i;
            assert!(s.economy.used[good] > 0.);
            assert!((s.economy.made[good] as f64 - source.extracted[0]).abs() < 0.001);
            assert!(
                (s.economy.made[2] - s.economy.used[good] * c.recipes[i].output[2]).abs() < 0.001
            );
        } else {
            assert_eq!(source.extracted[0], 0.);
            assert_eq!(source.remaining[0], 40.);
            assert_eq!(s.economy.made[2], 0.);
            assert!(h.supplier_unit_cost(i, 1).is_none());
        }
    }
    assert!(r.residual() < 1e-8);
    let residual = h.economy_residuals();
    assert!(residual.iter().all(|v| v.abs() < 0.001), "{residual:?}");
    let mut invalid = c.clone();
    let mut bad = invalid.recipes[0];
    bad.input.fill(0.);
    bad.output.fill(0.);
    bad.input[0] = 10.;
    bad.output[32] = 1.;
    invalid.recipes.push(bad);
    assert!(invalid.validate().is_err());
    std::fs::remove_file(path).unwrap();
}
