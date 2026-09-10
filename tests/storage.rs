use ancient_world::{
    catalog::Catalog,
    config::Config,
    economy::EconomyCatalog,
    gpu::{ContextGpu, Generator},
};

#[test]
#[ignore = "requires hardware GPU"]
fn warehouses_are_finite_persistent_and_checkpointed() {
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
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.recipes.clear();
    catalog.production.adaptive_labor = false;
    catalog.production.workshops = false;
    catalog.production.waterworks = false;
    g.configure_economy(catalog.clone()).unwrap();
    // Declared fixture imports create pressure to build, without upstream production.
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.storage[2] = 100.;
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
            assert_eq!(e.storage[0], 0.); // Missing bricks block construction despite timber and work.
            assert_eq!(e.storage[1], 0.);
        } else {
            assert!(e.storage[0] > 0. && e.storage[1] > 0.);
        }
        assert_eq!(e.storage[2], 100.);
        assert!((e.storage[0] / 0.02 - e.storage[1] / 0.03).abs() < 0.001);
        assert!(e.storage[3] + e.housing[3] <= e.labor[3] * 0.1 + 0.001);
    }
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    let path = std::env::temp_dir().join(format!("warehouses-{}.world", std::process::id()));
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
        assert_eq!(e.storage[2], old.storage[2]);
        assert_eq!(e.storage[3], old.storage[3]); // No workers, no construction.
        if old.storage[0] > 0. {
            assert!(e.storage[0] < old.storage[0] && e.storage[0] > 0.);
            assert!(e.storage[1] < old.storage[1] && e.storage[1] > 0.);
            assert!(e.storage_capacity() > 100.);
        }
        for (j, good) in [0, 5].into_iter().enumerate() {
            let worn = old.storage[j] - e.storage[j] + old.housing[j] - e.housing[j];
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
    assert_eq!(s.economy.storage[2], surviving.storage[2]);
    assert!(s.economy.storage[3] >= surviving.storage[3]);
    assert!(
        s.economy.storage_capacity() - surviving.storage_capacity()
            <= s.economy.labor[3] * 0.1 / 0.002 + 0.01
    );
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
fn old_storage_state_is_explicitly_uninitialized() {
    let mut old = serde_json::to_value(ancient_world::economy::Economy::default()).unwrap();
    old.as_object_mut().unwrap().remove("storage");
    old.as_object_mut().unwrap().remove("storage_plan");
    let e: ancient_world::economy::Economy = serde_json::from_value(old).unwrap();
    assert_eq!(e.storage, [0.; 4]);
    assert_eq!(e.storage_plan, [0.; 4]);
    assert_eq!(
        std::mem::size_of::<ancient_world::economy::Economy>() % 16,
        0
    );
}
