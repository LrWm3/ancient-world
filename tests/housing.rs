use ancient_world::{
    catalog::Catalog,
    config::Config,
    economy::EconomyCatalog,
    gpu::{ContextGpu, Generator},
};

#[test]
#[ignore = "requires hardware GPU"]
fn housing_is_finite_persistent_and_checkpointed() {
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
    g.enable_society().unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.recipes.clear();
    catalog.production.adaptive_labor = false;
    catalog.production.workshops = false;
    catalog.production.waterworks = false;
    catalog.production.persistent_storage = false;
    g.configure_economy(catalog.clone()).unwrap();
    // Declared fixture imports create pressure to build, without upstream production.
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.housing[2] = 10.;
        s.economy.policy[3] = 0.;
        for (good, mass) in [
            (0, 500.),
            (5, if s.id == 0 { 0. } else { 500. }),
            (7, 20000.),
        ] {
            s.economy.goods[good] += mass;
            s.economy.initial[good] += mass;
            for (k, ratio) in catalog.composition(good).iter().enumerate() {
                s.economy.external[k] += mass * ratio;
            }
        }
    }
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    for s in &h.sites {
        let e = &s.economy;
        if s.id == 0 {
            assert_eq!(e.housing[0], 0.); // Missing bricks block construction despite timber and work.
            assert_eq!(e.housing[1], 0.);
        } else {
            assert!(e.housing[0] > 0. && e.housing[1] > 0.);
        }
        assert_eq!(e.housing[2], 10.);
        assert!((e.housing[0] / 2. - e.housing[1] / 3.).abs() < 0.001);
        assert!(e.housing[3] <= e.labor[3] * 0.1 + 0.001);
    }
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    let path = std::env::temp_dir().join(format!("housing-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    g.advance_history(3).unwrap();
    for _ in 0..3 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap(),
    );
    let h = g.civilizations.as_ref().unwrap();
    let observation = h.social_indicators(0).unwrap();
    assert!(observation.housing[1] > 0.15 && observation.housing[2] > 0.15);
    assert_eq!(
        h.events
            .iter()
            .filter(|e| e.kind == "housing_pressure" && e.site == Some(0))
            .count(),
        1
    );
    // Destroy the resident cohort through an explicit fixture mortality exchange.
    let h = g.civilizations.as_mut().unwrap();
    for s in &mut h.sites {
        let pop = s.stocks.stock[0];
        s.stocks.people[1] += pop;
        s.demography.health[2] += pop;
        s.demography.ages[..3].fill(0.);
        s.stocks.stock[0] = 0.;
        s.abandoned = true;
    }
    let before: Vec<_> = h.sites.iter().map(|s| s.economy).collect();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    for (s, old) in h.sites.iter().zip(&before) {
        let e = &s.economy;
        assert_eq!(e.housing[2], old.housing[2]);
        assert_eq!(e.housing[3], old.housing[3]); // No residents or workers, no construction.
        if old.housing[0] > 0. {
            assert!(e.housing[0] < old.housing[0] && e.housing[0] > 0.);
            assert!(e.housing[1] < old.housing[1] && e.housing[1] > 0.);
            assert!(e.housing_capacity() > 10.);
        }
        for (j, good) in [0, 5].into_iter().enumerate() {
            let worn = old.housing[j] - e.housing[j];
            assert!((e.used[good] - old.used[good] - worn).abs() < 0.001);
        }
    }
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    let surviving = h.sites[1].economy;
    // Declared fixture arrivals establish a small new cohort in the existing site.
    let h = g.civilizations.as_mut().unwrap();
    h.initial_population += 20.;
    h.sites[1].stocks.stock[0] = 20.;
    h.sites[1].demography.ages[1] = 20.;
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let s = &h.sites[1];
    assert!(!s.abandoned);
    assert_eq!(s.economy.housing[2], surviving.housing[2]);
    assert!(s.economy.housing[3] >= surviving.housing[3]);
    assert!(
        s.economy.housing_capacity() - surviving.housing_capacity()
            <= s.economy.labor[3] * 0.1 / 0.2 + 0.01
    );
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
fn old_housing_state_is_explicitly_uninitialized() {
    let mut old = serde_json::to_value(ancient_world::economy::Economy::default()).unwrap();
    old.as_object_mut().unwrap().remove("housing");
    old.as_object_mut().unwrap().remove("housing_plan");
    let e: ancient_world::economy::Economy = serde_json::from_value(old).unwrap();
    assert_eq!(e.housing, [0.; 4]);
    assert_eq!(e.housing_plan, [0.; 4]);
    assert_eq!(
        std::mem::size_of::<ancient_world::economy::Economy>() % 16,
        0
    );
}
