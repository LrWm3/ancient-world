use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
#[test]
#[ignore = "requires hardware GPU"]
fn abandoned_land_returns_once_and_continues_after_checkpoint() {
    for seed in [17, 81, 256] {
        check_seed(seed);
    }
}
fn check_seed(seed: u32) {
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            seed,
            resolution: 32,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.advance_ecology().unwrap();
    g.found_civilizations(5).unwrap();
    g.enable_living_history().unwrap();
    g.enable_environmental_returns().unwrap();
    g.advance_history(12).unwrap();
    assert!(
        g.ecology
            .budget(&g.gpu, &g.config)
            .unwrap()
            .within_tolerance
    );
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        let pop = s.stocks.stock[0];
        s.stocks.people[1] += pop;
        s.demography.health[2] += pop;
        s.stocks.stock[0] = 0.;
        s.demography.ages[..3].fill(0.);
        s.abandoned = true;
    }
    g.advance_history(1).unwrap();
    let released: Vec<_> = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| {
            assert_eq!(s.economy.land_return[1], 1.);
            assert_eq!(s.economy.return_flow, [0.; 4]);
            assert_eq!(s.economy.soil[..3], [0.; 3]);
            s.economy.land_return[3]
        })
        .collect();
    let path = std::env::temp_dir().join(format!(
        "environmental-returns-{}.world",
        std::process::id()
    ));
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
        bytemuck::cast_slice::<_, u8>(&g.ecology.snapshot(&g.gpu, &g.config).unwrap()),
        bytemuck::cast_slice::<_, u8>(
            &resumed
                .ecology
                .snapshot(&resumed.gpu, &resumed.config)
                .unwrap()
        )
    );
    for (s, area) in g.civilizations.as_ref().unwrap().sites.iter().zip(released) {
        assert_eq!(s.economy.land_return[3], area);
    }
    let residual = g.civilizations.as_ref().unwrap().economy_residuals();
    assert!(residual.iter().all(|v| v.abs() < 0.001), "{residual:?}");
    let budget = g.ecology.budget(&g.gpu, &g.config).unwrap();
    assert!(budget.within_tolerance, "{budget:?}");
    // Restore a test population through the declared birth ledger to exercise land reclamation.
    let s = &mut g.civilizations.as_mut().unwrap().sites[0];
    s.stocks.stock[0] = 120.;
    s.stocks.people[0] += 120.;
    s.demography.ages[1] = 120.;
    s.abandoned = false;
    g.advance_history(1).unwrap();
    let s = &g.civilizations.as_ref().unwrap().sites[0];
    assert_eq!(s.economy.land_return[1], 0.);
    assert!(s.economy.soil[0] > 0.);
    let residual = g.civilizations.as_ref().unwrap().economy_residuals();
    assert!(residual.iter().all(|v| v.abs() < 0.001), "{residual:?}");
    assert!(
        g.ecology
            .budget(&g.gpu, &g.config)
            .unwrap()
            .within_tolerance
    );
    std::fs::remove_file(path).unwrap();
}
