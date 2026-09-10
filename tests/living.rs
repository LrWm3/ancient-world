use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};

#[test]
#[ignore = "requires a hardware GPU"]
fn live_history_conserves_and_resumes_on_matching_clocks() {
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
    g.advance_ecology().unwrap();
    g.found_civilizations(5).unwrap();
    g.enable_society().unwrap();
    g.advance_history(2).unwrap();
    let start = g.ecology.clock.month;
    let before = g.snapshot().unwrap();
    g.enable_living_history().unwrap();
    g.advance_history(0).unwrap();
    assert_eq!(g.ecology.clock.month, start);
    g.advance_history(12).unwrap();
    assert_eq!(g.ecology.clock.month, start + 12);
    assert_eq!(g.civilizations.as_ref().unwrap().month, 14);
    let after = g.snapshot().unwrap();
    assert!(before
        .iter()
        .zip(&after)
        .any(|(a, b)| a.climate != b.climate));
    assert!(before.iter().zip(&after).any(|(a, b)| a.life != b.life));
    assert!(before
        .iter()
        .zip(&after)
        .all(|(a, b)| a.terrain == b.terrain && a.routing == b.routing));
    let report = g.ecology.budget(&g.gpu, &g.config).unwrap();
    assert!(report.within_tolerance, "{report:?}");
    let path = std::env::temp_dir().join(format!("living-history-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    g.advance_history(3).unwrap();
    for _ in 0..3 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&g.snapshot().unwrap()),
        bytemuck::cast_slice::<_, u8>(&resumed.snapshot().unwrap())
    );
    assert!(
        resumed
            .ecology
            .budget(&resumed.gpu, &resumed.config)
            .unwrap()
            .within_tolerance
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&g.ecology.snapshot(&g.gpu, &g.config).unwrap()),
        bytemuck::cast_slice::<_, u8>(
            &resumed
                .ecology
                .snapshot(&resumed.gpu, &resumed.config)
                .unwrap()
        )
    );
    resumed
        .civilizations
        .as_mut()
        .unwrap()
        .living
        .as_mut()
        .unwrap()
        .incomplete = true;
    assert!(resumed.save(&path).is_err());
    assert!(resumed.advance_history(1).is_err());
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires a hardware GPU"]
fn live_drought_is_shared_by_farms_and_the_environment() {
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
    g.found_civilizations(5).unwrap();
    g.enable_society().unwrap();
    g.enable_living_history().unwrap();
    let mut catalog = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    catalog.weather.drought_probability = 1.;
    catalog.weather.drought_severity = 1.;
    g.configure_economy(catalog).unwrap();
    let rain: Vec<_> = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.economy.water[2])
        .collect();
    g.advance_history(1).unwrap();
    assert!(g
        .snapshot()
        .unwrap()
        .iter()
        .all(|c| c.budget[0] == 0. && c.climate[1] == 0.));
    for (s, old) in g.civilizations.as_ref().unwrap().sites.iter().zip(rain) {
        assert_eq!(s.economy.water[2], old);
        assert_eq!(s.demography.ages[3], 0.);
    }
    assert!(
        g.ecology
            .budget(&g.gpu, &g.config)
            .unwrap()
            .within_tolerance
    );
}
