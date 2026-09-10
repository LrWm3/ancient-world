use ancient_world::{
    catalog::Catalog,
    config::Config,
    economy::EconomyCatalog,
    gpu::{ContextGpu, Generator},
};

#[test]
fn legacy_waterworks_are_absent() {
    let mut old = serde_json::to_value(EconomyCatalog::bundled().unwrap()).unwrap();
    old["production"]
        .as_object_mut()
        .unwrap()
        .remove("waterworks");
    old["production"]
        .as_object_mut()
        .unwrap()
        .remove("waterworks_repair_priority");
    let old: EconomyCatalog = serde_json::from_value(old).unwrap();
    assert!(!old.production.waterworks);
    assert!(!old.production.waterworks_repair_priority);
    let mut settings = serde_json::to_value(&old.production).unwrap();
    settings
        .as_object_mut()
        .unwrap()
        .remove("waterworks_target_fraction");
    let settings: ancient_world::production::ProductionSettings =
        serde_json::from_value(settings).unwrap();
    assert_eq!(settings.waterworks_target_fraction, 1.);
    let mut old = serde_json::to_value(ancient_world::economy::Economy::default()).unwrap();
    for name in [
        "waterworks",
        "waterworks_plan",
        "water_service",
        "waterworks_recovery",
    ] {
        old.as_object_mut().unwrap().remove(name);
    }
    let old: ancient_world::economy::Economy = serde_json::from_value(old).unwrap();
    assert_eq!(old.waterworks_capacity(), 0.);
    assert_eq!(old.water_service, [0.; 4]);
    assert_eq!(old.waterworks_recovery, [0.; 4]);
}

#[test]
#[ignore = "requires hardware GPU"]
fn waterworks_conserve_and_reduce_exposure_only_with_service() {
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
    let mut c = EconomyCatalog::bundled().unwrap();
    c.recipes.clear();
    c.production.waterworks_target_fraction = 1.; // Full-service fixture, independent of calibration.
    c.production.adaptive_labor = false;
    c.production.workshops = false;
    c.production.persistent_storage = false;
    g.configure_economy(c.clone()).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.housing[2] = 1.; // Same controlled crowding in both comparisons.
        s.economy.policy[3] = 0.;
    }
    let path = std::env::temp_dir().join(format!("waterworks-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut unbuilt = Generator::load(g.gpu.clone(), &path).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        let places = s.stocks.stock[0];
        for (j, good) in [0, 5].into_iter().enumerate() {
            let mass = places * [2., 4.][j];
            s.economy.waterworks[j] = mass;
            s.economy.initial[good] += mass;
            for (k, ratio) in c.composition(good).iter().enumerate() {
                s.economy.external[k] += mass * ratio;
            }
        }
    }
    g.advance_history(1).unwrap();
    unbuilt.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    for (s, control) in h
        .sites
        .iter()
        .zip(&unbuilt.civilizations.as_ref().unwrap().sites)
    {
        assert!(s.economy.waterworks_plan[3] > 0.9);
        assert_eq!(control.economy.waterworks_plan[3], 0.);
        assert!(s.demography.health[0] < control.demography.health[0]);
        assert!(s.economy.water_service[0] > 0.);
        assert!(
            s.economy.water_service[2] + s.economy.waterworks[2] + s.economy.housing[3]
                <= s.economy.labor[3] * 0.1 + 0.001
        );
    }
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    g.advance_history(3).unwrap();
    for _ in 0..3 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    // Intact waterworks cannot supply an empty reservoir during zero rainfall.
    let mut terrain = resumed.snapshot().unwrap();
    for cell in &mut terrain {
        cell.hydro[2] = 0.;
    }
    resumed
        .restore_cells(&terrain, resumed.progress.epoch)
        .unwrap();
    for s in &mut resumed.civilizations.as_mut().unwrap().sites {
        s.economy.water[3] += s.economy.water[0];
        s.economy.water[0] = 0.;
    }
    resumed.advance_history(1).unwrap();
    for s in &resumed.civilizations.as_ref().unwrap().sites {
        assert_eq!(s.economy.water_service[1], 1.);
        assert_eq!(s.economy.waterworks_plan[3], 0.);
        assert!(s.economy.waterworks_capacity() > 0.);
    }
    assert!(resumed
        .civilizations
        .as_ref()
        .unwrap()
        .economy_residuals()
        .iter()
        .all(|v| v.abs() < 0.001));
    // A zero investment target leaves domestic demand active without installing assets.
    c.production.waterworks_target_fraction = 0.;
    unbuilt.configure_economy(c.clone()).unwrap();
    unbuilt.advance_history(1).unwrap();
    for s in &unbuilt.civilizations.as_ref().unwrap().sites {
        assert_eq!(s.economy.waterworks_capacity(), 0.);
        assert_eq!(s.economy.waterworks_plan[0], 0.);
        assert!(s.economy.water_service[0] > 0.);
    }
    c.production.waterworks_target_fraction = 1.;
    unbuilt.configure_economy(c.clone()).unwrap();
    // Previously unbuilt sites construct only after finite material imports.
    for s in &mut unbuilt.civilizations.as_mut().unwrap().sites {
        for (good, mass) in [(0, 250.), (5, 500.)] {
            s.economy.goods[good] += mass;
            s.economy.initial[good] += mass;
            for (k, ratio) in c.composition(good).iter().enumerate() {
                s.economy.external[k] += mass * ratio;
            }
        }
        // Supply adequate baseline shelter so housing does not take this fixture's work.
        s.economy.housing[2] = s.stocks.stock[0] * 2.;
    }
    unbuilt.advance_history(1).unwrap();
    for s in &unbuilt.civilizations.as_ref().unwrap().sites {
        assert!(s.economy.waterworks_plan[2] > 0.);
        assert!(s.economy.waterworks[2] <= s.economy.labor[3] * 0.1 + 0.001);
    }
    assert!(unbuilt
        .civilizations
        .as_ref()
        .unwrap()
        .economy_residuals()
        .iter()
        .all(|v| v.abs() < 0.001));
    // Empty recipe queues must not erase recurring operating demand.
    c.production.adaptive_labor = true;
    g.configure_economy(c.clone()).unwrap();
    g.advance_history(48).unwrap();
    for s in &g.civilizations.as_ref().unwrap().sites {
        assert!(s.economy.waterworks_plan[3] > 0.5);
    }
    let h = g.civilizations.as_mut().unwrap();
    for s in &mut h.sites {
        let pop = s.stocks.stock[0];
        s.stocks.people[1] += pop;
        s.demography.health[2] += pop;
        s.stocks.stock[0] = 0.;
        s.demography.ages[..3].fill(0.);
        s.abandoned = true;
    }
    let before: Vec<_> = h.sites.iter().map(|s| s.economy).collect();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    for (s, old) in h.sites.iter().zip(before) {
        assert!(s.economy.waterworks_capacity() < old.waterworks_capacity());
        assert_eq!(s.economy.waterworks_plan[3], 0.);
        assert_eq!(s.economy.water_service[0], old.water_service[0]);
        assert_eq!(s.economy.water_service[2], old.water_service[2]);
    }
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
fn waterworks_targets_are_bounded() {
    let mut c = EconomyCatalog::bundled().unwrap();
    for invalid in [-0.01, 1.01, f32::NAN, f32::INFINITY] {
        c.production.waterworks_target_fraction = invalid;
        assert!(c.validate().is_err());
    }
    for valid in [0., 0.5, 1.] {
        c.production.waterworks_target_fraction = valid;
        c.validate().unwrap();
    }
}

#[test]
#[ignore = "requires hardware GPU"]
fn inherited_waterworks_repairs_compete_with_housing_headroom() {
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
    let mut c = EconomyCatalog::bundled().unwrap();
    c.recipes.clear();
    c.production.adaptive_labor = false;
    c.production.workshops = false;
    c.production.persistent_storage = false;
    c.production.waterworks_target_fraction = 1.;
    c.production.waterworks_repair_priority = true;
    g.configure_economy(c.clone()).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        let pop = s.stocks.stock[0];
        s.economy.housing[2] = pop * 1.02; // Shelter is adequate; only headroom competes.
        s.economy.policy[3] = 0.;
        s.economy.waterworks_recovery[0] = pop;
        // Declared fixture: a formerly complete system lost half its material.
        for (j, good) in [0, 5].into_iter().enumerate() {
            let original = pop * [2., 4.][j];
            let lost = original * 0.5;
            s.economy.waterworks[j] = original - lost;
            s.economy.waterworks_plan[1] += lost;
            s.economy.used[good] += lost;
            s.economy.reserves[3] += lost;
            let supplies = pop * 10.;
            s.economy.goods[good] += supplies;
            s.economy.initial[good] += original + supplies;
            for (k, ratio) in c.composition(good).iter().enumerate() {
                s.economy.external[k] += (original + supplies) * ratio;
                s.economy.detritus[k] += lost * ratio;
            }
        }
    }
    let path = std::env::temp_dir().join(format!("recovery-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut control = Generator::load(g.gpu.clone(), &path).unwrap();
    let mut crowded = Generator::load(g.gpu.clone(), &path).unwrap();
    let mut marginal = Generator::load(g.gpu.clone(), &path).unwrap();
    let mut marginal_control = Generator::load(g.gpu.clone(), &path).unwrap();
    for world in [&mut marginal, &mut marginal_control] {
        for s in &mut world.civilizations.as_mut().unwrap().sites {
            s.economy.housing[2] = s.stocks.stock[0] - 0.01;
        }
    }
    let mut no_damage = Generator::load(g.gpu.clone(), &path).unwrap();
    let mut no_bricks = Generator::load(g.gpu.clone(), &path).unwrap();
    for s in &mut no_bricks.civilizations.as_mut().unwrap().sites {
        let removed = s.economy.goods[5];
        s.economy.goods[5] = 0.;
        s.economy.initial[5] -= removed;
        for (k, ratio) in c.composition(5).iter().enumerate() {
            s.economy.external[k] -= removed * ratio;
        }
    }

    c.production.waterworks_repair_priority = false;
    marginal_control.configure_economy(c.clone()).unwrap();
    control.configure_economy(c).unwrap();
    for s in &mut crowded.civilizations.as_mut().unwrap().sites {
        s.economy.housing[2] = 1.;
    }
    for s in &mut no_damage.civilizations.as_mut().unwrap().sites {
        s.economy.waterworks_recovery[0] = 0.;
    }
    g.advance_history(1).unwrap();
    control.advance_history(1).unwrap();
    crowded.advance_history(1).unwrap();
    marginal.advance_history(1).unwrap();
    marginal_control.advance_history(1).unwrap();
    for (s, old) in marginal
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .zip(&marginal_control.civilizations.as_ref().unwrap().sites)
    {
        let e = &s.economy;
        assert!(
            e.housing_plan[2] >= 0.009,
            "urgent shelter must be built first"
        );
        assert!(
            e.waterworks_recovery[2] > 0.,
            "resolved shelter shortage must release repair work"
        );
        assert!(e.waterworks_capacity() > old.economy.waterworks_capacity());
        assert!(e.housing_plan[2] < old.economy.housing_plan[2]);
        assert!((e.housing[0] / 2. - e.housing[1] / 3.).abs() < 0.001);
        assert!((e.housing_plan[2] * 0.2 + e.waterworks_plan[2] * 0.2) <= e.labor[3] * 0.1 + 0.001);
    }
    no_damage.advance_history(1).unwrap();
    no_bricks.advance_history(1).unwrap();
    assert!(no_bricks
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .all(|s| s.economy.waterworks_recovery[2] == 0.));
    for (s, old) in g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .zip(&control.civilizations.as_ref().unwrap().sites)
    {
        assert!(s.economy.waterworks_recovery[2] > 0.);
        assert!(s.economy.waterworks_capacity() > old.economy.waterworks_capacity());
        assert!(s.economy.housing_plan[2] < old.economy.housing_plan[2]);
        assert!(s.economy.waterworks_capacity() <= s.economy.waterworks_recovery[0] + 0.001);
        assert!(s.economy.waterworks_recovery[2] <= s.economy.labor[3] * 0.1 + 0.001);
    }
    assert!(crowded
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .all(|s| s.economy.waterworks_recovery[2] == 0.));
    // No old loss: only this month's actual wear is eligible, not service expansion.
    for s in &no_damage.civilizations.as_ref().unwrap().sites {
        assert!(s.economy.waterworks_recovery[2] <= s.economy.waterworks_plan[1] / 6. * 0.2);
        assert!(s.economy.waterworks_recovery[2] < s.stocks.stock[0] * 0.001);
    }
    g.advance_history(1).unwrap();
    control.advance_history(1).unwrap();
    for (s, old) in g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .zip(&control.civilizations.as_ref().unwrap().sites)
    {
        assert!(s.economy.waterworks_plan[3] > old.economy.waterworks_plan[3]);
        println!("site {} month 2: coverage repair {:.6}, control {:.6}; capacity {:.3}/{:.3}; cumulative priority work {:.3}", s.id, s.economy.waterworks_plan[3], old.economy.waterworks_plan[3], s.economy.waterworks_capacity(), old.economy.waterworks_capacity(), s.economy.waterworks_recovery[3]);
    }
    for world in [
        &g,
        &control,
        &crowded,
        &marginal,
        &marginal_control,
        &no_damage,
        &no_bricks,
    ] {
        assert!(world
            .civilizations
            .as_ref()
            .unwrap()
            .economy_residuals()
            .iter()
            .all(|v| v.abs() < 0.001));
    }
    marginal.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(&path).unwrap();
    marginal.advance_history(3).unwrap();
    for _ in 0..3 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&marginal.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
}
