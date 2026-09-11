use ancient_world::{
    catalog::Catalog,
    config::Config,
    economy::{EconomyCatalog, Recipe},
    gpu::{ContextGpu, Generator},
    grid,
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
fn relative(a: f64, b: f64) -> f64 {
    (a - b).abs() / a.abs().max(b.abs()).max(1.)
}
#[test]
fn economy_catalog_rejects_free_material_and_carbon() {
    let mut e = ancient_world::economy::Economy {
        claim: [0., 1., 1., 1.],
        ..Default::default()
    };
    assert!(e.valid());
    e.labor[0] = -0.001;
    assert!(!e.valid());
    let mut c = EconomyCatalog::bundled().unwrap();
    c.validate().unwrap();
    c.recipes[0].output[6] = 3.;
    assert!(c.validate().is_err());
    let mut c = EconomyCatalog::bundled().unwrap();
    c.goods[0].id = "ore".into();
    assert!(c.validate().is_err());
    let mut c = EconomyCatalog::bundled().unwrap();
    c.recipes[0].work[0] = 0.;
    assert!(c.validate().is_err());
    let mut c = EconomyCatalog::bundled().unwrap();
    c.recipes[0].work[2] = 4.;
    assert!(c.validate().is_err());
    c.recipes[0].work[2] = 1.5;
    assert!(c.validate().is_err());
}
#[test]
#[ignore = "requires a hardware GPU"]
fn plot_reservations_and_gpu_recipes_conserve() {
    let mut g = world();
    // A large diagnostic withdrawal stays above float32 subtraction noise when
    // comparing global planetary inventory differences against claimed stocks.
    g.config.settlement_plot_hectares = 5000.;
    g.advance_ecology().unwrap();
    let before = g.ecology.budget(&g.gpu, &g.config).unwrap();
    g.found_civilizations(16).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().any(|a| h
        .sites
        .iter()
        .any(|b| a.id != b.id && a.economy.claim[0] == b.economy.claim[0])));
    let claimed: [f64; 3] =
        std::array::from_fn(|k| h.sites.iter().map(|s| s.economy.baseline[k] as f64).sum());
    let after = g.ecology.budget(&g.gpu, &g.config).unwrap();
    assert!(after.within_tolerance);
    for (k, amount) in claimed.iter().enumerate() {
        assert!(
            relative(before.inventory_kg[k] - after.inventory_kg[k], *amount) < 0.005,
            "claim {k}"
        );
    }
    let mut c = EconomyCatalog::bundled().unwrap();
    c.recipes = vec![Recipe {
        input: {
            let mut v = [0.; ancient_world::economy::GOODS];
            v[2] = 1.;
            v
        },
        output: {
            let mut v = [0.; ancient_world::economy::GOODS];
            v[3] = 1.;
            v
        },
        work: [0.1, 0., 0., 0.],
    }];
    g.configure_economy(c).unwrap();
    let s = &mut g.civilizations.as_mut().unwrap().sites[0];
    s.economy.goods[2] = 10.;
    s.economy.initial[2] = 10.;
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let s = &h.sites[0];
    assert!((s.economy.made[3] - 10.).abs() < 0.001);
    assert!((s.economy.used[2] - 10.).abs() < 0.001);
    assert!(s.economy.goods[2] < 0.001);
    assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
}
#[test]
#[ignore = "requires a hardware GPU"]
fn no_phosphorus_blocks_crops_and_manure_retains_it() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let s = &mut g.civilizations.as_mut().unwrap().sites[0];
    // Declared removal from all farm-available P compartments; no hidden fertilizer source.
    let removed = s.economy.soil[2] + s.economy.detritus[2] + s.economy.reserves[0];
    s.economy.external[2] -= removed;
    s.economy.soil[2] = 0.;
    s.economy.detritus[2] = 0.;
    s.economy.reserves[0] = 0.;
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.recipes.clear();
    catalog.agriculture =
        Some(ancient_world::agriculture::AgricultureCatalog::seasonal_experiment());
    g.configure_economy(catalog).unwrap();
    // Declare an established mid-season stand by moving existing seed, not adding biomass.
    let site = &mut g.civilizations.as_mut().unwrap().sites[0];
    site.demography.crops[2] = 4.;
    for crop in &mut site.economy.crops {
        crop[1] += crop[2];
        crop[2] = 0.;
    }
    g.set_site_policy(0, [0.15, 0., 0.25, 0.]).unwrap();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites[0].economy.agriculture[0] < 0.001);
    assert_eq!(h.sites[0].economy.diagnostics[0], 2.);
    assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
    g.set_site_policy(0, [0.15, 1., 0.25, 0.]).unwrap();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites[0].economy.detritus[2] > 0.);
    g.advance_history(1).unwrap();
    // Diverse crops accumulate living biomass before their individual harvest months.
    assert!(
        g.civilizations.as_ref().unwrap().sites[0]
            .economy
            .agriculture[0]
            > 0.
    );
}
#[test]
#[ignore = "requires a hardware GPU"]
fn markets_reserve_cargo_pay_once_and_respect_closure() {
    let mut g = world();
    g.found_civilizations(16).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let n = g.config.resolution;
    let (seller, buyer) = h
        .sites
        .iter()
        .flat_map(|a| h.sites.iter().map(move |b| (a, b)))
        .find(|(a, b)| {
            let x = grid::cell_direction(a.cell, n);
            let y = grid::cell_direction(b.cell, n);
            a.id != b.id
                && a.island == b.island
                && x.iter()
                    .zip(y)
                    .map(|(x, y)| x * y)
                    .sum::<f32>()
                    .clamp(-1., 1.)
                    .acos()
                    * g.config.radius_km
                    < 1500.
        })
        .map(|(a, b)| (a.id as usize, b.id as usize))
        .unwrap();
    for i in 0..16 {
        g.set_site_policy(i, [0.15, 0.85, 0.25, 0.]).unwrap();
    }
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.recipes.clear();
    g.configure_economy(catalog).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    h.sites[seller].economy.goods[3] += 2000.;
    h.sites[seller].economy.initial[3] += 2000.;
    let h = g.civilizations.as_mut().unwrap();
    let tools = h.sites[buyer].economy.goods[3];
    h.sites[buyer].economy.goods[3] = 0.;
    h.sites[buyer].economy.used[3] += tools;
    h.sites[buyer].economy.reserves[3] += tools;
    g.advance_history(3).unwrap();
    assert!(g.civilizations.as_ref().unwrap().cargo.is_empty());
    g.set_site_policy(seller as u32, [0.15, 0.85, 0.25, 1.])
        .unwrap();
    g.set_site_policy(buyer as u32, [0.15, 0.85, 0.25, 1.])
        .unwrap();
    g.advance_history(3).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let shipment = h
        .cargo
        .iter()
        .find(|c| c.from as usize == seller && c.to as usize == buyer && c.good == 3)
        .unwrap()
        .clone();
    assert!(h.sites[seller].economy.finance[2] >= shipment.paid);
    assert!(
        h.sites[buyer].economy.goods[3] < 0.001,
        "cargo cannot arrive at departure"
    );
    let file = std::env::temp_dir().join(format!("market-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    let months = shipment.arrives - h.month + 1;
    g.advance_history(months).unwrap();
    for _ in 0..months {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_vec(&g.civilizations).unwrap(),
        serde_json::to_vec(&resumed.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites[buyer].economy.goods[3] > 0.);
    assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
}

#[test]
#[ignore = "requires a hardware GPU"]
fn farming_needs_sunlight_even_with_stored_nutrients() {
    let mut g = world();
    g.config.solar_scale = 0.;
    g.found_civilizations(5).unwrap();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().all(|s| s.economy.agriculture[0] == 0.
        && s.economy.soil[1] > 0.
        && s.economy.soil[2] > 0.));
    assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
}

#[test]
#[ignore = "requires a hardware GPU"]
fn excess_grain_is_bounded_and_remains_in_the_nutrient_ledger() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    let s = &mut h.sites[0];
    let added = s.stocks.stock[0] * 18. * 240.;
    s.stocks.stock[1] += added;
    h.initial_food += added as f64;
    for (k, ratio) in [0.45, 0.02, 0.003].into_iter().enumerate() {
        s.economy.external[k] += added * ratio;
    }
    let detritus = s.economy.detritus[2];
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let s = &h.sites[0];
    assert!(s.stocks.stock[1] <= s.stocks.stock[0] * 18. * 24.1);
    assert!(s.stocks.ledger[2] > added * 0.8);
    assert!(s.economy.detritus[2] > detritus + added * 0.002);
    assert!(h.food_residual().abs() < 0.001);
    assert!(h.economy_residuals().iter().all(|r| r.abs() < 0.001));
}

#[test]
#[ignore = "requires a hardware GPU"]
fn worn_tools_recycle_with_fuel_and_irrecoverable_losses() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    // This fixture measures conversion yields under explicit continuous orders.
    catalog.production.enabled = false;
    catalog.recipes.clear();
    g.configure_economy(catalog).unwrap();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    for s in &h.sites {
        assert!((s.economy.made[29] - s.economy.used[3] * 0.9).abs() < 1e-5);
        assert!(s.economy.made[29] < s.economy.used[3]);
    }
    let mut catalog = EconomyCatalog::bundled().unwrap();
    // This fixture measures conversion yields under explicit continuous orders.
    catalog.production.enabled = false;
    catalog.recipes.retain(|r| r.input[29] > 0.);
    g.configure_economy(catalog).unwrap();
    let e = &mut g.civilizations.as_mut().unwrap().sites[0].economy;
    e.goods[6] += 1.;
    e.initial[6] += 1.;
    e.external[0] += 1.; // Declared fuel import for this fixture.
    let scrap = e.goods[29];
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!((h.sites[0].economy.made[2] - scrap * 0.9).abs() < 1e-5);
    assert!(h.sites[0].economy.used[6] >= scrap * 0.1 - 1e-5);
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
#[ignore = "requires a hardware GPU"]
fn planned_overstock_stops_extraction_without_destroying_legacy_inventory() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.recipes.clear();
    catalog.production.specialized_workshops = false;
    g.configure_economy(catalog.clone()).unwrap();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    // Low ore stocks under orders must not trigger the legacy mining rush and starve farms.
    for s in &h.sites {
        assert!(s.economy.labor[0] >= s.economy.labor.iter().sum::<f32>() * 0.60);
    }
    for s in &mut h.sites {
        s.economy.policy[3] = 0.;
        for k in [0, 13] {
            let imported = 100000.;
            s.economy.goods[k] += imported;
            s.economy.initial[k] += imported;
            for n in 0..3 {
                s.economy.external[n] += imported * catalog.composition(k)[n];
            }
        }
    }
    let before: Vec<_> = h
        .sites
        .iter()
        .map(|s| {
            (
                s.economy.goods[0],
                s.economy.made[0],
                s.economy.made[1],
                s.economy.made[4],
            )
        })
        .collect();
    g.advance_history(2).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    for (s, b) in h.sites.iter().zip(before) {
        assert_eq!(
            (
                s.economy.goods[0],
                s.economy.made[0],
                s.economy.made[1],
                s.economy.made[4]
            ),
            b
        );
        assert!(s.economy.goods[13] >= 100000.);
    }
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
#[ignore = "requires a hardware GPU"]
fn ore_and_clay_share_one_physical_mining_budget() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.production.enabled = false;
    catalog.recipes.clear();
    g.configure_economy(catalog).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.reserves[1] = 1e6;
        s.economy.reserves[2] = 1e6;
        s.economy.policy[3] = 0.;
    }
    let before: Vec<_> = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.economy.made[1] + s.economy.made[4])
        .collect();
    g.advance_history(1).unwrap();
    for (s, before) in g.civilizations.as_ref().unwrap().sites.iter().zip(before) {
        let extracted = s.economy.made[1] + s.economy.made[4] - before;
        assert!(extracted > 0.);
        assert!(
            extracted <= s.economy.labor[2] * 5. + 0.001,
            "{} > {}",
            extracted,
            s.economy.labor[2] * 5.
        );
    }
}

#[test]
#[ignore = "requires a hardware GPU"]
fn idle_industries_release_workers_without_consuming_new_resources() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.recipes.clear();
    catalog.production.specialized_workshops = false;
    g.configure_economy(catalog.clone()).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.policy[3] = 0.;
        let imported = 100000.;
        s.economy.goods[0] += imported;
        s.economy.initial[0] += imported;
        for k in 0..3 {
            s.economy.external[k] += imported * catalog.composition(0)[k];
        }
    }
    g.advance_history(24).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    for s in &h.sites {
        let workers = s.economy.labor.iter().sum::<f32>();
        assert!(s.economy.labor[0] > workers * 0.9, "{:?}", s.economy.labor);
        assert!(s.economy.labor[1] + s.economy.labor[2] < workers * 0.01);
        assert!(s.economy.goods[0] >= 100000.);
    }
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
#[ignore = "requires a hardware GPU"]
fn workshops_require_materials_and_hold_them_through_checkpoint_and_wear() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.recipes.clear();
    catalog.production.specialized_workshops = false;
    g.configure_economy(catalog.clone()).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.policy[3] = 0.;
        s.economy.workshop_plan[0] = 10.;
    }
    g.advance_history(1).unwrap();
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .all(|s| s.economy.workshop[0] == 0.));
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        for good in [0, 5, 3] {
            let mass = 1000.;
            s.economy.goods[good] += mass;
            s.economy.initial[good] += mass;
            for k in 0..3 {
                s.economy.external[k] += mass * catalog.composition(good)[k];
            }
        }
    }
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().all(|s| s.economy.workshop[0] > 0.));
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    let checkpoint = std::env::temp_dir().join(format!("workshop-{}.world", std::process::id()));
    g.save(&checkpoint).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &checkpoint).unwrap();
    std::fs::remove_file(checkpoint).unwrap();
    g.advance_history(12).unwrap();
    for _ in 0..12 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().all(|s| s.economy.workshop_plan[1] > 0.));
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
#[ignore = "requires a hardware GPU"]
fn industrial_work_obeys_installed_capacity() {
    // Seed 17 exhausted capacity between recipes and exposed a floating-point
    // subtraction edge case: the next recipe must never receive negative work.
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            seed: 17,
            resolution: 64,
            ecology_resolution: 64,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.run_epochs(1).unwrap();
    g.found_civilizations(16).unwrap();
    g.advance_history(120).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let mut worked = 0.;
    for s in &h.sites {
        let e = &s.economy;
        let units = (e.workshop[0] / 20.)
            .min(e.workshop[1] / 30.)
            .min(e.workshop[2] / 2.);
        // Available workers include the tiny separately allocated shore fishery.
        let workers = e.labor.iter().sum::<f32>() / 0.999;
        assert!(e.workshop_plan[3] <= (workers * 0.025).max(1.) + units * 4. + 0.01);
        let unequipped_work: f32 = e
            .workshop_types
            .iter()
            .map(|t| (t[2] - 4. * t[0]).max(0.))
            .sum();
        assert!(unequipped_work <= (workers * 0.025).max(1.) + 0.01);
        worked += e.workshop_plan[3];
    }
    assert!(worked > 0.);
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
#[ignore = "requires a hardware GPU"]
fn specialized_capacity_cannot_make_an_unrelated_good() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut c = EconomyCatalog::bundled().unwrap();
    c.recipes.retain(|r| r.output[3] > 0.);
    c.production.adaptive_labor = false;
    g.configure_economy(c.clone()).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        let e = &mut s.economy;
        e.policy[3] = 0.;
        for (k, mass) in [(2, 10000.), (0, 100.), (5, 150.), (3, 10.)] {
            e.initial[k] += mass;
            if k == 2 {
                e.goods[k] += mass;
            }
            for j in 0..3 {
                e.external[j] += mass * c.composition(k)[j];
            }
        }
        e.workshop = [100., 150., 10., 1.];
        e.workshop_types[1][0] = 5.;
    }
    let path = std::env::temp_dir().join(format!("specialized-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut wrong = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    for s in &mut wrong.civilizations.as_mut().unwrap().sites {
        s.economy.workshop_types[1][0] = 0.;
        s.economy.workshop_types[2][0] = 5.;
    }
    g.advance_history(1).unwrap();
    wrong.advance_history(1).unwrap();
    let production = |g: &Generator| {
        g.civilizations
            .as_ref()
            .unwrap()
            .sites
            .iter()
            .map(|s| s.economy.made[3])
            .sum::<f32>()
    };
    assert!(production(&g) > production(&wrong) + 1.);
    // With no operating orders, installed equipment wears rather than requesting
    // automatic repairs forever. No material is duplicated by family assignment.
    c.recipes.clear();
    g.configure_economy(c).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        for t in &mut s.economy.workshop_types {
            t[1] = 0.;
        }
    }
    let before: f32 = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.economy.workshop[0])
        .sum();
    g.advance_history(1).unwrap();
    let after: f32 = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.economy.workshop[0])
        .sum();
    assert!((after - before * 0.998).abs() < 0.001);
    for g in [&g, &wrong] {
        let h = g.civilizations.as_ref().unwrap();
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
        assert!(h.sites.iter().all(|s| s.economy.valid()));
    }
}

#[test]
#[ignore = "requires a hardware GPU"]
fn diagnostic_fixed_staffing_preserves_worker_shares_and_budgets() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut c = EconomyCatalog::bundled().unwrap();
    c.production.diagnostic_fixed_labor = true;
    g.configure_economy(c).unwrap();
    g.advance_history(3).unwrap();
    let mut measured = 0;
    for s in &g.civilizations.as_ref().unwrap().sites {
        let l = s.economy.labor;
        if l[2] > 0. {
            measured += 1;
            assert!((l[0] / l[2] - 6.2).abs() < 1e-4);
            assert!((l[3] / l[2] - 2.).abs() < 1e-4);
        }
    }
    assert!(measured > 0);
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_residuals()
        .iter()
        .all(|v| v.abs() < 0.001));
}

#[test]
#[ignore = "requires a hardware GPU"]
fn restricted_tools_conserve_custody_and_resume_without_becoming_usable() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut c = EconomyCatalog::bundled().unwrap();
    c.recipes.clear();
    c.production.specialized_workshops = false;
    g.configure_economy(c).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.policy[3] = 0.;
    }
    let before = serde_json::to_value(&g.civilizations).unwrap();
    g.set_tool_stock_access(0, 1.).unwrap();
    assert_eq!(before, serde_json::to_value(&g.civilizations).unwrap());
    assert!(g.set_tool_stock_access(0, f64::NAN).is_err());
    assert!(g.set_tool_stock_access(999, 0.).is_err());
    let original: Vec<_> = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.economy.goods[3])
        .collect();
    assert!(original.iter().sum::<f32>() > 0.);
    for id in 0..original.len() as u32 {
        g.set_tool_stock_access(id, 0.).unwrap();
    }
    let stored = g
        .civilizations
        .as_ref()
        .unwrap()
        .experimental_tool_reserves
        .clone();
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_residuals()
        .iter()
        .all(|x| x.abs() < 0.001));
    g.advance_history(1).unwrap();
    for s in &g.civilizations.as_ref().unwrap().sites {
        assert_eq!(s.economy.production_probe[0], 0.75);
        assert_eq!(s.economy.production_probe[2], 0.);
        assert!(s.economy.production_probe[1] >= 0.);
    }
    let path = std::env::temp_dir().join(format!("tool-custody-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    g.advance_history(2).unwrap();
    resumed.advance_history(1).unwrap();
    resumed.advance_history(1).unwrap();
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    assert_eq!(
        g.civilizations.as_ref().unwrap().experimental_tool_reserves,
        stored
    );
    for id in 0..original.len() as u32 {
        g.set_tool_stock_access(id, 1.).unwrap();
    }
    for (s, kg) in g.civilizations.as_ref().unwrap().sites.iter().zip(original) {
        assert_eq!(s.economy.goods[3], kg);
    }
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .experimental_tool_reserves
        .is_empty());
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_residuals()
        .iter()
        .all(|x| x.abs() < 0.001));
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires a hardware GPU"]
fn food_security_staffing_reacts_without_free_workers_and_resumes() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.production.food_security_labor = true;
    g.configure_economy(catalog.clone()).unwrap();
    for id in 0..g.civilizations.as_ref().unwrap().sites.len() as u32 {
        g.set_tool_stock_access(id, 0.).unwrap();
    }
    // A controlled remembered-pressure intervention changes no physical inventory.
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.food_labor[0] = 1.;
    }
    let checkpoint = std::env::temp_dir().join(format!("food-labor-{}.world", std::process::id()));
    g.save(&checkpoint).unwrap();
    let mut control = Generator::load(g.gpu.clone(), &checkpoint).unwrap();
    catalog.production.food_security_labor = false;
    control.configure_economy(catalog).unwrap();
    g.advance_history(1).unwrap();
    control.advance_history(1).unwrap();
    let mut responsive = 0;
    for (s, old) in g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .zip(&control.civilizations.as_ref().unwrap().sites)
    {
        let labor = s.economy.labor;
        let total: f32 = labor.iter().sum();
        let old_total: f32 = old.economy.labor.iter().sum();
        assert!(labor.iter().all(|v| *v >= 0.));
        assert!((total - old_total).abs() < 0.001);
        if total > 0. {
            responsive += usize::from(labor[0] / total > old.economy.labor[0] / old_total + 0.01);
            assert!(labor[0] / total >= 0.62);
            assert!(labor[1..].iter().sum::<f32>() > 0.);
        }
        assert!((0.75..=1.).contains(&s.economy.food_labor[0]));
    }
    assert!(
        responsive > 0,
        "pressure must change staffing in the controlled fixture"
    );
    g.save(&checkpoint).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &checkpoint).unwrap();
    g.advance_history(2).unwrap();
    resumed.advance_history(1).unwrap();
    resumed.advance_history(1).unwrap();
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    for g in [&g, &control] {
        assert!(g
            .civilizations
            .as_ref()
            .unwrap()
            .economy_residuals()
            .iter()
            .all(|v| v.abs() < 0.001));
    }
    std::fs::remove_file(checkpoint).unwrap();
}

#[test]
#[ignore = "requires a hardware GPU"]
fn maintenance_ablation_changes_idle_industry_without_changing_pressure_or_workers() {
    let mut g = world();
    // Generate finite ore first: an unadvanced world has no feasible mining.
    g.run_epochs(1).unwrap();
    g.found_civilizations(5).unwrap();
    let mut c = EconomyCatalog::bundled().unwrap();
    c.production.food_security_labor = true;
    g.configure_economy(c.clone()).unwrap();
    for id in 0..g.civilizations.as_ref().unwrap().sites.len() as u32 {
        g.set_tool_stock_access(id, 0.).unwrap();
    }
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.food_labor[0] = 1.;
        // Previous occupation shares only; no new worker or material inventory.
        s.economy.labor = [50., 0., 0., 0.];
    }
    let path =
        std::env::temp_dir().join(format!("maintenance-ablation-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut without = Generator::load(g.gpu.clone(), &path).unwrap();
    c.production.food_security_maintenance = false;
    without.configure_economy(c).unwrap();
    g.advance_history(1).unwrap();
    without.advance_history(1).unwrap();
    let mut changed = 0;
    for (a, b) in g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .zip(&without.civilizations.as_ref().unwrap().sites)
    {
        assert_eq!(a.economy.food_labor, b.economy.food_labor);
        let total = |s: &ancient_world::civilization::Site| s.economy.labor.iter().sum::<f32>();
        assert!((total(a) - total(b)).abs() < 0.001);
        assert!(a.economy.labor.iter().all(|v| *v >= 0.));
        assert!(a.economy.labor[0] <= b.economy.labor[0] + 0.001);
        changed += usize::from(a.economy.labor[0] < b.economy.labor[0] - 0.001);
    }
    assert!(
        changed > 0,
        "reservation must bind in the idle-industry fixture"
    );
    without.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    without.advance_history(2).unwrap();
    resumed.advance_history(1).unwrap();
    resumed.advance_history(1).unwrap();
    assert_eq!(
        serde_json::to_value(&without.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    for g in [&g, &without] {
        assert!(g
            .civilizations
            .as_ref()
            .unwrap()
            .economy_residuals()
            .iter()
            .all(|v| v.abs() < 0.001));
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires a hardware GPU"]
fn replacement_jobs_and_toolmaking_learn_only_from_finite_completed_work() {
    let mut g = world();
    g.run_epochs(1).unwrap();
    g.found_civilizations(5).unwrap();
    let mut c = EconomyCatalog::bundled().unwrap();
    c.production.replacement_tool_jobs = true;
    c.production.toolmaking_expertise = true;
    g.configure_economy(c.clone()).unwrap();
    for id in 0..5 {
        g.set_tool_stock_access(id, 0.).unwrap();
    }
    // Initial metal is a declared fixture import, matched in material accounting.
    // Isolate craft execution from mining and knowledge acquisition delays.
    for site in &mut g.civilizations.as_mut().unwrap().sites {
        site.economy.goods[2] += 100.;
        site.economy.initial[2] += 100.;
    }
    let path = std::env::temp_dir().join(format!("tool-jobs-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut no_learning = Generator::load(g.gpu.clone(), &path).unwrap();
    c.production.toolmaking_expertise = false;
    no_learning.configure_economy(c.clone()).unwrap();
    let mut experienced = Generator::load(g.gpu.clone(), &path).unwrap();
    for s in &mut experienced.civilizations.as_mut().unwrap().sites {
        s.economy.tool_craft[0] = 1.;
    }
    experienced.advance_history(1).unwrap();
    g.advance_history(1).unwrap();
    no_learning.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().any(|s| s.economy.tool_work[3] > 0.));
    for (a, b) in h
        .sites
        .iter()
        .zip(&no_learning.civilizations.as_ref().unwrap().sites)
    {
        let e = &a.economy;
        assert!(e.tool_work[1] <= e.tool_work[0] + 0.001);
        assert!(e.tool_work[2] <= e.labor[3] + 0.001);
        assert_eq!(
            e.goods, b.economy.goods,
            "zero initial skill gives no free advantage"
        );
        let fish = if e.management[0] > 0.5 && e.management[1] >= 1. {
            0.001
        } else {
            0.
        };
        let workforce = e.labor.iter().sum::<f32>() / (1. - fish);
        let practice = (e.tool_work[2] / workforce.max(0.001)).clamp(0., 1.);
        assert!((e.tool_craft[0] - 0.04 * practice).abs() < 1e-6);
    }
    for (novice, expert) in h
        .sites
        .iter()
        .zip(&experienced.civilizations.as_ref().unwrap().sites)
    {
        let a = novice.economy.tool_work;
        let b = expert.economy.tool_work;
        if a[3] > 0. && b[3] > 0. {
            assert!((a[2] / a[3] - 0.1).abs() < 1e-6);
            assert!((b[2] / b[3] - 0.08).abs() < 1e-6);
        }
    }
    assert!(experienced
        .civilizations
        .as_ref()
        .unwrap()
        .economy_residuals()
        .iter()
        .all(|v| v.abs() < 0.001));
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
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
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_residuals()
        .iter()
        .all(|v| v.abs() < 0.001));
    // With no recipes, a large remembered skill cannot manufacture practice or output.
    c.production.toolmaking_expertise = true;
    c.recipes.clear();
    g.configure_economy(c).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.tool_craft[0] = 0.5;
    }
    g.advance_history(1).unwrap();
    for s in &g.civilizations.as_ref().unwrap().sites {
        assert_eq!(s.economy.tool_work[2], 0.);
        assert_eq!(s.economy.tool_work[3], 0.);
        assert!((s.economy.tool_craft[0] - 0.499).abs() < 1e-6);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires a hardware GPU"]
fn fisheries_can_use_migratory_animals_without_aquatic_grazers() {
    let mut g = world();
    g.advance_ecology().unwrap();
    let mut eco = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    for c in &mut eco {
        for p in &mut c.pools[5..17] {
            *p = [0.; 4];
        }
        c.pools[15] = [0.1, 0.012, 0.0015, 1.];
    }
    g.restore_ecology(&eco).unwrap();
    g.found_civilizations(16).unwrap();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let catch: f32 = h.sites.iter().map(|s| s.economy.agriculture[2]).sum();
    assert!(
        catch > 0.,
        "migratory animal stocks must support a connected fishery"
    );
    assert!(g
        .ecology
        .snapshot(&g.gpu, &g.config)
        .unwrap()
        .iter()
        .all(|c| c.pools[13][0] == 0.));
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    assert!(
        g.ecology
            .budget(&g.gpu, &g.config)
            .unwrap()
            .within_tolerance
    );
}

#[test]
fn seasonal_catalog_validates_and_legacy_remains_explicit() {
    let mut c = EconomyCatalog::bundled().unwrap();
    assert!(c
        .agriculture
        .as_ref()
        .unwrap()
        .crops
        .iter()
        .all(|c| c.season.is_none()));
    c.agriculture = Some(ancient_world::agriculture::AgricultureCatalog::seasonal_experiment());
    let a = c.agriculture.as_mut().unwrap();
    assert_eq!(a.crops[4].yield_scale, 1.);
    assert!(a.crops.iter().all(|c| c.season.is_some()));
    a.crops[0].season.as_mut().unwrap().harvest_index = 1.1;
    assert!(c.validate().is_err());
    let c = EconomyCatalog::bundled().unwrap();
    let mut value = serde_json::to_value(c.agriculture.as_ref().unwrap()).unwrap();
    for crop in value["crops"].as_array_mut().unwrap() {
        crop.as_object_mut().unwrap().remove("season");
    }
    let legacy: ancient_world::agriculture::AgricultureCatalog =
        serde_json::from_value(value).unwrap();
    legacy.validate(&c).unwrap();
    assert!(legacy.crops.iter().all(|c| c.season.is_none()));
    assert!(legacy.gpu(&c)[15..].iter().all(|v| *v == [0.; 4]));
}
#[test]
#[ignore = "requires hardware GPU"]
fn stored_seed_is_dormant_and_a_full_season_conserves_material() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.agriculture =
        Some(ancient_world::agriculture::AgricultureCatalog::seasonal_experiment());
    catalog.market.adaptive_prices = true;
    g.configure_economy(catalog).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    for site in &mut h.sites {
        // At month one, phase seven is post-harvest for every crop.
        site.demography.crops[2] = 0.;
    }
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().all(|s| s.economy.agriculture[0] == 0.));
    assert!(h
        .sites
        .iter()
        .all(|s| s.economy.crops.iter().all(|c| c[1] == 0.)));
    g.advance_history(6).unwrap();
    let checkpoint = std::env::temp_dir().join(format!("seasonal-{}.world", std::process::id()));
    g.save(&checkpoint).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &checkpoint).unwrap();
    std::fs::remove_file(checkpoint).unwrap();
    g.advance_history(12).unwrap();
    for _ in 0..12 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().any(|s| s.economy.agriculture[0] > 0.));
    assert!(
        h.economy_residuals().iter().all(|r| r.abs() < 0.001),
        "{:?}",
        h.economy_residuals()
    );
}

#[test]
#[ignore = "requires hardware GPU; paired 50-year model evaluation"]
fn seasonal_crops_and_adaptive_prices_seed_comparison() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let seeds: Vec<u32> = std::env::var("CROP_PRICE_SEEDS")
        .ok()
        .map(|v| v.split(',').map(|v| v.parse().unwrap()).collect())
        .unwrap_or_else(|| vec![17, 81, 256]);
    let months: u32 = std::env::var("CROP_PRICE_MONTHS")
        .ok()
        .map(|v| v.parse().unwrap())
        .unwrap_or(600);
    let combined = std::env::var_os("CROP_PRICE_COMBINED_ONLY").is_some();
    let living = std::env::var_os("CROP_PRICE_LIVING").is_some();
    for seed in seeds {
        let variants = [(false, false), (true, false), (false, true), (true, true)];
        for (crops, prices) in variants {
            if combined && !(crops && prices) {
                continue;
            }
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    seed,
                    resolution: 64,
                    ecology_resolution: 32,
                    ecology_years_per_epoch: 1,
                    ..Default::default()
                },
                Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.run_epochs(1).unwrap();
            g.found_civilizations(8).unwrap();
            let mut catalog = EconomyCatalog::bundled().unwrap();
            catalog.market.adaptive_prices = prices;
            if crops {
                catalog.agriculture =
                    Some(ancient_world::agriculture::AgricultureCatalog::seasonal_experiment());
            }
            if !crops {
                for c in &mut catalog.agriculture.as_mut().unwrap().crops {
                    c.season = None;
                    if c.good == "tubers" {
                        c.yield_scale = 6.;
                    }
                }
            }
            g.configure_economy(catalog).unwrap();
            g.enable_society().unwrap();
            g.enable_politics().unwrap();
            g.enable_governance().unwrap();
            g.enable_shipping().unwrap();
            if living {
                g.enable_living_history().unwrap();
            }
            if std::env::var_os("CROP_PRICE_MONTHLY").is_some() {
                for month in 1..=months {
                    g.advance_history(1).unwrap();
                    if month <= 36 || month % 120 == 0 {
                        let h = g.civilizations.as_ref().unwrap();
                        let s = &h.sites[0];
                        println!("month={month} seed={seed} seasonal={crops} adaptive={prices} pop={} food={} growth={} crop={:?} water={} labor={:?} cultivated={} shortage={}", s.stocks.stock[0],s.stocks.stock[1],s.economy.agriculture[0],s.economy.crops[0],s.economy.water[0],s.economy.labor,s.economy.production_probe[1],s.stocks.stock[3]);
                    }
                }
            } else {
                g.advance_history(months).unwrap();
            }
            let h = g.civilizations.as_ref().unwrap();
            let population: f32 = h.sites.iter().map(|s| s.stocks.stock[0]).sum();
            let harvest: f32 = h
                .sites
                .iter()
                .map(|s| {
                    s.economy
                        .crops
                        .iter()
                        .enumerate()
                        .map(|(i, c)| {
                            let catalog = h.economy_catalog.as_ref().unwrap();
                            let crop = &catalog.agriculture.as_ref().unwrap().crops[i];
                            c[3] * catalog.goods[catalog.index(&crop.good).unwrap()].food_energy
                        })
                        .sum::<f32>()
                })
                .sum();
            let residual = h
                .economy_residuals()
                .into_iter()
                .map(f64::abs)
                .fold(0., f64::max);
            assert!(residual < 0.001, "seed {seed}: {residual}");
            let sales = h.events.iter().filter(|e| e.kind == "market_sale").count();
            let wheat_range = h
                .sites
                .iter()
                .filter(|s| !s.abandoned)
                .fold([f32::INFINITY, 0_f32], |r, s| {
                    [r[0].min(s.economy.prices[8]), r[1].max(s.economy.prices[8])]
                });
            // Game-regression acceptance for these ordinary reference worlds:
            // balanced ledgers alone must not count a total collapse as success.
            assert!(
                h.sites
                    .iter()
                    .any(|s| !s.abandoned && s.stocks.stock[0] >= 20.),
                "seed {seed}: no viable settlement after {months} months"
            );
            if prices {
                assert!(
                    wheat_range[1] < 100.,
                    "seed {seed}: unsupported staple quote drift {wheat_range:?}"
                );
            }
            println!("sales={sales} active_wheat_range={wheat_range:?}");
            println!("seed={seed} seasonal={crops} adaptive_prices={prices} population={population:.1} occupied={} harvested_food_equivalent={harvest:.1} wheat_price={:.3} tools_price={:.3} residual={residual:.7}",h.sites.iter().filter(|s|!s.abandoned).count(),h.sites[0].economy.prices[8],h.sites[0].economy.prices[3]);
        }
    }
}

#[test]
#[ignore = "requires hardware GPU"]
fn scarce_crop_resources_are_not_awarded_in_catalog_order() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.recipes.clear();
    g.configure_economy(catalog.clone()).unwrap();
    for site in &mut g.civilizations.as_mut().unwrap().sites {
        let e = &mut site.economy;
        let removed = e.soil[2] + e.detritus[2] + e.reserves[0] - 0.01;
        e.external[2] -= removed;
        e.soil[2] = 0.01;
        e.detritus[2] = 0.;
        e.reserves[0] = 0.;
    }
    let checkpoint = std::env::temp_dir().join(format!("crop-order-{}.world", std::process::id()));
    g.save(&checkpoint).unwrap();
    let mut reversed = Generator::load(g.gpu.clone(), &checkpoint).unwrap();
    std::fs::remove_file(checkpoint).unwrap();
    catalog.agriculture.as_mut().unwrap().crops.reverse();
    reversed.configure_economy(catalog).unwrap();
    for site in &mut reversed.civilizations.as_mut().unwrap().sites {
        site.economy.crops.reverse();
    }
    g.advance_history(1).unwrap();
    reversed.advance_history(1).unwrap();
    let a = g.civilizations.as_ref().unwrap();
    let b = reversed.civilizations.as_ref().unwrap();
    assert!(a.sites.iter().any(|s| s.economy.agriculture[0] > 0.));
    for (a, b) in a.sites.iter().zip(&b.sites) {
        for j in 0..6 {
            for k in 1..4 {
                assert!(
                    relative(
                        a.economy.crops[j][k] as f64,
                        b.economy.crops[5 - j][k] as f64
                    ) < 0.0001,
                    "site {} crop {j} field {k}: {:?} vs {:?}",
                    a.id,
                    a.economy.crops[j],
                    b.economy.crops[5 - j]
                );
            }
        }
        assert!(a.economy.valid() && b.economy.valid());
    }
    for h in [a, b] {
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    }
}
