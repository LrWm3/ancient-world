use ancient_world::{
    catalog::Catalog,
    config::Config,
    ecology::{EcoCell, Intervention},
    gpu::{ContextGpu, Generator},
};
fn make(n: u32, m: u32) -> Generator {
    Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: n,
            ecology_resolution: m,
            ecology_years_per_epoch: 1,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap()
}
fn conserved(g: &Generator, tolerance: f64) {
    let b = g.ecology.budget(&g.gpu, &g.config).unwrap();
    assert!(b.relative_error.iter().all(|v| *v < tolerance), "{b:?}");
    assert!(b.water_relative_error < 2e-5, "water budget {b:?}");
    ancient_world::ecology::validate(&g.ecology.snapshot(&g.gpu, &g.config).unwrap()).unwrap();
}
#[test]
fn ecological_catalog_and_configuration() {
    let c = Catalog::bundled().unwrap();
    assert!(c.plants.len() >= 72);
    assert!(c.biomes.len() >= 24);
    assert_eq!(c.guilds.len(), 12);
    assert_eq!(c.microbes.len(), 8);
    assert!(
        c.plants
            .iter()
            .filter(|p| p.ecology.layer == 3 || p.ecology.layer == 4)
            .count()
            >= 12
    );
    let mut c = c;
    c.plants[0].ecology.phosphorus = 0.;
    assert!(c.validate().is_err());
    let config = Config {
        ecology_resolution: 63,
        ..Default::default()
    };
    assert!(config.validate().is_err());
    assert!(Config {
        resolution: 256,
        ecology_resolution: 512,
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(Config {
        resolution: 16,
        ..Default::default()
    }
    .validate()
    .is_ok());
}
#[test]
#[ignore = "requires hardware GPU"]
fn budgets_coarse_grid_scenarios_and_continuation() {
    let mut g = make(32, 16);
    g.run_epochs(1).unwrap();
    conserved(&g, 2e-5);
    let p = std::env::temp_dir().join(format!("ecology-checkpoint-{}.world", std::process::id()));
    g.scenario(Some(3), Intervention::RemoveGuild(3)).unwrap();
    g.scenario(None, Intervention::GeochemicalSupply(false))
        .unwrap();
    g.scenario(Some(1), Intervention::LakeMixing(12.)).unwrap();
    conserved(&g, 2e-5);
    g.save(&p).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &p).unwrap();
    for _ in 0..36 {
        g.advance_ecology().unwrap();
        resumed.advance_ecology().unwrap();
    }
    let a = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    let b = resumed
        .ecology
        .snapshot(&resumed.gpu, &resumed.config)
        .unwrap();
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&a),
        bytemuck::cast_slice::<_, u8>(&b)
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&g.snapshot().unwrap()),
        bytemuck::cast_slice::<_, u8>(&resumed.snapshot().unwrap())
    );
    assert_eq!(
        g.progress.geological_time_myr,
        resumed.progress.geological_time_myr
    );
    assert_eq!(g.ecology.clock.month, 48);
    conserved(&g, 4e-5);
    for c in &a {
        if c.pools[31][3] as u32 & 8 != 0 {
            assert_eq!(c.pools[8][0], 0.);
        }
    }
    g.scenario(None, Intervention::RestoreGuild(3)).unwrap();
    assert_eq!(g.ecology.clock.events.len(), 4);
    std::fs::remove_file(p).unwrap();
}
#[test]
#[ignore = "requires hardware GPU"]
fn no_energy_and_no_phosphorus_limit_growth() {
    let mut g = make(16, 16);
    g.run_epochs(1).unwrap();
    let mut cells = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    for c in &mut cells {
        c.pools[26] = [0.; 4];
        c.pools[31][0] = 0.;
        for k in 5..17 {
            let old = c.pools[k];
            for (j, v) in old[..3].iter().enumerate() {
                c.pools[18][j] += v;
            }
            c.pools[k] = [0.; 4];
        }
        c.pools[31][3] = 4095.;
    }
    g.restore_ecology(&cells).unwrap();
    g.config.solar_scale = 0.;
    let initial: f64 = cells
        .iter()
        .map(|c| c.pools[..5].iter().map(|p| p[0] as f64).sum::<f64>())
        .sum();
    for _ in 0..24 {
        g.advance_ecology().unwrap();
    }
    let result = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    let final_biomass: f64 = result
        .iter()
        .map(|c| c.pools[..5].iter().map(|p| p[0] as f64).sum::<f64>())
        .sum();
    assert!(final_biomass < initial);
    assert!(result
        .iter()
        .all(|c| c.pools[28][0] == 0. && c.pools[28][1] == 0.));
    conserved(&g, 5e-5);
    let mut cells = result;
    for c in &mut cells {
        for pool in &mut c.pools[..26] {
            pool[2] = 0.;
        }
        c.pools[26][0] = 10.;
        c.pools[26][3] = 10.;
    }
    g.restore_ecology(&cells).unwrap();
    g.config.solar_scale = 1.;
    g.advance_ecology().unwrap();
    let result = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    assert!(result
        .iter()
        .all(|c| c.pools[28][0] < 1e-8 && c.pools[28][1] < 1e-8));
    conserved(&g, 5e-5);
}
#[test]
#[ignore = "requires hardware GPU"]
fn lake_mixing_transfers_existing_phosphorus() {
    let mut g = make(16, 16);
    g.run_epochs(1).unwrap();
    let terrain = g.snapshot().unwrap();
    let mut cells = vec![EcoCell::default(); terrain.len()];
    for (c, t) in cells.iter_mut().zip(&terrain) {
        c.pools[31] = [0., 0., 1., 4095.];
        if t.meta[0] == 1 {
            c.pools[21] = [0., 1., 1., 0.];
        }
    }
    g.restore_ecology(&cells).unwrap();
    g.config.solar_scale = 0.;
    let p = std::env::temp_dir().join(format!("mixing-{}.world", std::process::id()));
    g.save(&p).unwrap();
    let mut mixed = Generator::load(g.gpu.clone(), &p).unwrap();
    mixed.scenario(None, Intervention::LakeMixing(20.)).unwrap();
    for _ in 0..12 {
        g.advance_ecology().unwrap();
        mixed.advance_ecology().unwrap();
    }
    let p_surface = |g: &Generator| {
        g.ecology
            .snapshot(&g.gpu, &g.config)
            .unwrap()
            .iter()
            .map(|c| c.pools[20][2] as f64)
            .sum::<f64>()
    };
    assert!(p_surface(&mixed) > p_surface(&g) + 0.00001);
    conserved(&mixed, 5e-5);
    conserved(&g, 5e-5);
    std::fs::remove_file(p).unwrap();
}

#[test]
#[ignore = "requires hardware GPU"]
fn migration_delivers_phosphorus_and_recovers_after_restoration() {
    let mut g = make(16, 16);
    g.run_epochs(1).unwrap();
    let mut terrain = g.snapshot().unwrap();
    let source = 100u32;
    let upland = ancient_world::grid::neighbor(source, 16, 1, 0) as usize;
    for c in &mut terrain {
        c.meta[0] = 2;
        c.terrain[0] = 500.;
        c.hydro = [500., 23., 1400., 0.];
        c.water = [0., 1., 0., 0.];
        c.routing[0] = u32::MAX;
    }
    terrain[upland].meta[0] = 3;
    terrain[upland].terrain[0] = 1000.;
    terrain[upland].hydro[0] = 1000.;
    g.restore_cells(&terrain, g.progress.epoch).unwrap();
    g.advance_ecology().unwrap();
    let mut cells = vec![EcoCell::default(); terrain.len()];
    for c in &mut cells {
        c.pools[31] = [0., 0., 1., 0.];
    }
    cells[source as usize].pools[8] = [1., 0.12, 0.015, 0.];
    g.restore_ecology(&cells).unwrap();
    g.config.solar_scale = 0.;
    let p = std::env::temp_dir().join(format!("migration-{}.world", std::process::id()));
    g.save(&p).unwrap();
    let mut removed = Generator::load(g.gpu.clone(), &p).unwrap();
    removed
        .scenario(Some(3), Intervention::RemoveGuild(3))
        .unwrap();
    for _ in 0..36 {
        g.advance_ecology().unwrap();
        removed.advance_ecology().unwrap();
    }
    let phosphorus =
        |g: &Generator| g.ecology.snapshot(&g.gpu, &g.config).unwrap()[upland].pools[17][2];
    assert!(
        phosphorus(&g) > phosphorus(&removed) + 1e-8,
        "upland receives P only through migrating consumers"
    );
    let before = phosphorus(&removed);
    removed
        .scenario(Some(3), Intervention::RestoreGuild(3))
        .unwrap();
    for _ in 0..36 {
        removed.advance_ecology().unwrap();
    }
    assert!(phosphorus(&removed) > before + 1e-8);
    conserved(&g, 5e-5);
    conserved(&removed, 5e-5);
    std::fs::remove_file(p).unwrap();
}

#[test]
#[ignore = "requires hardware GPU"]
fn isolated_lake_exchange_matches_cpu_two_box_reference() {
    let mut g = make(16, 16);
    g.run_epochs(1).unwrap();
    let mut terrain = g.snapshot().unwrap();
    let lake = 50usize;
    for c in &mut terrain {
        c.meta[0] = 2;
        c.routing[0] = u32::MAX;
        c.water = [0.; 4];
        c.hydro = [500., 0., 0., 0.];
        c.terrain[0] = 500.;
    }
    terrain[lake].meta[0] = 1;
    terrain[lake].terrain[0] = -1000.;
    terrain[lake].water[0] = 999.;
    g.restore_cells(&terrain, g.progress.epoch).unwrap();
    g.advance_ecology().unwrap();
    let mut cells = vec![EcoCell::default(); terrain.len()];
    for c in &mut cells {
        c.pools[31] = [0., 2., 1., 4095.];
    }
    cells[lake].pools[21][2] = 1.;
    g.restore_ecology(&cells).unwrap();
    g.config.solar_scale = 0.;
    g.advance_ecology().unwrap();
    let direction = ancient_world::grid::cell_direction(lake as u32, 16);
    let peripheral = (-((direction[2].clamp(-1., 1.).acos() - 0.90) / 0.16).powi(2)).exp();
    let exchange = (0.02 + peripheral * 2.) * 2. / 12.;
    let expected = exchange / 900.;
    let result = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    assert!((result[lake].pools[20][2] - expected).abs() < 1e-7);
    assert!((result[lake].pools[21][2] - (1. - expected)).abs() < 1e-7);
    conserved(&g, 5e-5);
}

#[test]
#[ignore = "requires hardware GPU"]
fn legacy_import_is_explicit_and_preserves_terrain() {
    let mut g = make(16, 16);
    g.run_epochs(1).unwrap();
    let path = std::env::temp_dir().join(format!("legacy-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let data = std::fs::read(&path).unwrap();
    let length = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
    let mut header: serde_json::Value = serde_json::from_slice(&data[16..16 + length]).unwrap();
    header["version"] = 1.into();
    header.as_object_mut().unwrap().remove("ecology");
    header["catalog"]["version"] = 1.into();
    header["catalog"].as_object_mut().unwrap().remove("guilds");
    header["catalog"]
        .as_object_mut()
        .unwrap()
        .remove("microbes");
    header["catalog"]["plants"]
        .as_array_mut()
        .unwrap()
        .truncate(48);
    header["catalog"]["biomes"]
        .as_array_mut()
        .unwrap()
        .truncate(16);
    for plant in header["catalog"]["plants"].as_array_mut().unwrap() {
        plant.as_object_mut().unwrap().remove("ecology");
    }
    let mut cells = g.snapshot().unwrap();
    for c in &mut cells {
        c.strata = [0.; 4];
        c.ids[2] = u32::MAX;
        c.ids[3] = u32::MAX;
    }
    let metadata = serde_json::to_vec(&header).unwrap();
    let payload: Vec<u8> = cells
        .iter()
        .flat_map(|c| bytemuck::bytes_of(c)[..160].iter().copied())
        .collect();
    let mut legacy = b"ANCIENT1".to_vec();
    legacy.extend((metadata.len() as u64).to_le_bytes());
    legacy.extend(&metadata);
    legacy.extend(&payload);
    let sum = metadata
        .iter()
        .chain(&payload)
        .fold(0xcbf29ce484222325u64, |h, b| {
            (h ^ *b as u64).wrapping_mul(0x100000001b3)
        });
    legacy.extend(sum.to_le_bytes());
    std::fs::write(&path, &legacy).unwrap();
    assert!(Generator::load(g.gpu.clone(), &path).is_err());
    let imported = Generator::import_v1(g.gpu.clone(), &path).unwrap();
    assert!(imported.ecology.clock.imported_baseline);
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&imported.snapshot().unwrap()),
        bytemuck::cast_slice::<_, u8>(&cells)
    );
    assert_eq!(std::fs::read(&path).unwrap(), legacy);
    assert_eq!(imported.catalog.guilds.len(), 12);
    assert_eq!(imported.catalog.plants.len(), 72);
    conserved(&imported, 5e-5);
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires hardware GPU"]
fn maintenance_returns_excess_body_nutrients_without_creating_them() {
    let mut g = make(16, 16);
    g.run_epochs(1).unwrap();
    let terrain = g.snapshot().unwrap();
    let land = terrain.iter().position(|c| c.meta[0] >= 2).unwrap();
    let mut cells = vec![EcoCell::default(); terrain.len()];
    for c in &mut cells {
        c.pools[31] = [0., 0., 1., 4094.];
    }
    cells[land].pools[5] = [1., 0.12, 0.015, 0.];
    g.restore_ecology(&cells).unwrap();
    g.config.solar_scale = 0.;
    g.advance_ecology().unwrap();
    let after = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    assert!(after[land].pools[17][1] > 0.0001);
    assert!(after[land].pools[17][2] > 0.00001);
    assert!(after[land].pools[5][1] <= after[land].pools[5][0] * 0.12 + 1e-6);
    conserved(&g, 5e-5);
}

#[test]
#[ignore = "requires hardware GPU"]
fn dry_month_evaporates_secondary_lake_storage() {
    let mut g = make(16, 16);
    g.run_epochs(1).unwrap();
    let mut terrain = g.snapshot().unwrap();
    let lake = terrain.iter().position(|c| c.meta[0] >= 2).unwrap();
    for c in &mut terrain {
        c.routing[2] = u32::MAX;
        c.climate[2] = 0.;
        c.hydro[2] = 0.;
        c.water[2] = 0.;
    }
    terrain[lake].terrain[0] = 500.;
    terrain[lake].hydro = [600., 23., 0., 0.];
    terrain[lake].water = [10., 0., 0., 0.];
    terrain[lake].routing = [u32::MAX, u32::MAX, lake as u32 + 2, 0];
    g.restore_cells(&terrain, g.progress.epoch).unwrap();
    g.restore_ecology(&vec![EcoCell::default(); terrain.len()])
        .unwrap();
    g.advance_ecology().unwrap();
    let after = g.inspect(lake as u32).unwrap();
    assert!(
        (after.water[0] - (10. - after.budget[1])).abs() < 0.001
            && after.budget[1] > 0.02
            && after.budget[0] == 0.,
        "stored lake water did not evaporate: {}",
        after.water[0]
    );
    conserved(&g, 5e-5);
}

#[test]
#[ignore = "requires a hardware GPU"]
fn island_phosphorus_changes_sources_without_changing_outer_inventory() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    for resolution in [16, 32] {
        let mut baseline = Vec::new();
        for scale in [1., 0.25] {
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    resolution: 32,
                    ecology_resolution: resolution,
                    island_phosphorus_scale: scale,
                    ..Default::default()
                },
                Catalog::bundled().unwrap(),
            )
            .unwrap();
            g.advance_ecology().unwrap();
            let cells = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
            conserved(&g, 0.001);
            if scale == 1. {
                baseline = cells;
                continue;
            }
            let mut depleted = 0;
            let mut unchanged = 0;
            let terrain = g.snapshot().unwrap();
            for (i, (a, b)) in baseline.iter().zip(cells).enumerate() {
                let i = i as u32;
                let ratio = 32 / resolution;
                let face = i / (resolution * resolution);
                let x = (i % resolution) * ratio;
                let y = (i / resolution % resolution) * ratio;
                let contains_island = (0..ratio).any(|dy| {
                    (0..ratio).any(|dx| {
                        terrain[(face * 1024 + (y + dy) * 32 + x + dx) as usize].meta[0] == 2
                    })
                });
                if !contains_island {
                    assert_eq!(a.pools[30], b.pools[30]);
                }
                // Recorded initial inventory includes all compartments, before any
                // ecological transfers. C/N are unchanged; only island P changes.
                assert_eq!(a.pools[30][0], b.pools[30][0]);
                assert_eq!(a.pools[30][1], b.pools[30][1]);
                assert!(b.pools[30][2] <= a.pools[30][2]);
                if b.pools[30][2] < a.pools[30][2] {
                    depleted += 1;
                } else {
                    unchanged += 1;
                }
            }
            assert!(depleted > 0 && unchanged > 0);
        }
    }
}

#[test]
#[ignore = "requires hardware GPU"]
fn version_three_environment_upgrade_preserves_stocks_and_continuation() {
    let mut g = make(16, 8);
    g.catalog.producer_competition = false;
    // Old worlds had no column inventory. Keep that baseline throughout this fixture.
    let mut initial = g.snapshot().unwrap();
    for c in &mut initial {
        c.strata = [0.; 4];
    }
    g.restore_cells(&initial, 0).unwrap();
    g.run_epochs(1).unwrap();
    let path = std::env::temp_dir().join(format!("ecology-v3-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let data = std::fs::read(&path).unwrap();
    let len = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
    let mut header: serde_json::Value = serde_json::from_slice(&data[16..16 + len]).unwrap();
    assert_eq!(header["version"], 8);
    header["version"] = 3.into();
    let metadata = serde_json::to_vec(&header).unwrap();
    let terrain_bytes = g.config.cells() as usize * ancient_world::gpu::CELL_BYTES as usize;
    let prefix =
        terrain_bytes + g.config.eco_cells() as usize * ancient_world::ecology::ECO_BYTES as usize;
    let stride = ancient_world::ecology::ENVIRONMENT_BYTES as usize;
    let env_end = 16 + len + prefix + g.config.eco_cells() as usize * stride;
    let mut payload = vec![];
    for c in data[16 + len..16 + len + terrain_bytes]
        .chunks_exact(ancient_world::gpu::CELL_BYTES as usize)
    {
        payload.extend(&c[..160]);
    }
    for cell in data[16 + len + terrain_bytes..16 + len + prefix]
        .chunks_exact(ancient_world::ecology::ECO_BYTES as usize)
    {
        payload.extend(&cell[..512]);
    }
    for env in data[16 + len + prefix..env_end].chunks_exact(stride) {
        payload.extend(&env[..128]);
    }
    payload.extend(&data[env_end..data.len() - 8]);
    let checksum = metadata
        .iter()
        .chain(&payload)
        .fold(0xcbf29ce484222325u64, |h, b| {
            (h ^ *b as u64).wrapping_mul(0x100000001b3)
        });
    let mut old = b"ANCIENT2".to_vec();
    old.extend((metadata.len() as u64).to_le_bytes());
    old.extend(metadata);
    old.extend(payload);
    old.extend(checksum.to_le_bytes());
    std::fs::write(&path, old).unwrap();
    let mut loaded = Generator::load(g.gpu.clone(), &path).unwrap();
    let exact = |a: &Generator, b: &Generator| {
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&a.ecology.snapshot(&a.gpu, &a.config).unwrap()),
            bytemuck::cast_slice::<_, u8>(&b.ecology.snapshot(&b.gpu, &b.config).unwrap())
        );
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&a.snapshot().unwrap()),
            bytemuck::cast_slice::<_, u8>(&b.snapshot().unwrap())
        );
    };
    exact(&g, &loaded);
    assert_eq!(
        loaded
            .ecology
            .inspect_environment(&loaded.gpu, &loaded.config, 0)
            .unwrap()
            .fields[21][3],
        0.
    );
    for _ in 0..12 {
        g.advance_ecology().unwrap();
        loaded.advance_ecology().unwrap();
    }
    exact(&g, &loaded);
    conserved(&loaded, 5e-5);
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires hardware GPU"]
fn hydrogen_symbiosis_uses_finite_reserves_and_declines_without_supply() {
    let mut g = make(16, 16);
    g.run_epochs(1).unwrap();
    let rock = g
        .catalog
        .rocks
        .iter()
        .position(|r| r.id == "peridotite")
        .unwrap() as u32;
    let mut terrain = g.snapshot().unwrap();
    for c in &mut terrain {
        c.meta[0] = 3;
        c.ids[0] = rock;
        c.terrain[0] = 500.;
        c.geology[0] = 0.9;
        c.hydro = [500., 23., 1400., 0.];
        c.water = [0., 1., 0., 0.];
        c.budget = [0.; 4];
        c.routing[0] = u32::MAX;
    }
    g.restore_cells(&terrain, 0).unwrap();
    let mut cells = vec![EcoCell::default(); terrain.len()];
    for c in &mut cells {
        c.pools[17] = [0., 1., 1., 0.];
        c.pools[26] = [1., 0., 100., 0.]; // H2 reserve, no secondary energy or stored oxidant
        c.pools[31] = [0., 0., 1., 4095.]; // No fresh chemical supply or animal predation
    }
    g.restore_ecology(&cells).unwrap();
    g.config.solar_scale = 0.;
    g.advance_ecology().unwrap();
    let first = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    // Too much growth to be explained by the small monthly aerobic replenishment.
    assert!(first.iter().any(|c| c.pools[3][0] + c.pools[4][0] > 0.001));
    let biomass = |g: &Generator| {
        g.ecology
            .snapshot(&g.gpu, &g.config)
            .unwrap()
            .iter()
            .map(|c| (c.pools[3][0] + c.pools[4][0]) as f64)
            .sum::<f64>()
    };
    for _ in 0..59 {
        g.advance_ecology().unwrap();
    }
    let peak = biomass(&g);
    for _ in 0..300 {
        g.advance_ecology().unwrap();
    }
    assert!(biomass(&g) < peak * 0.5);
    let last = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    assert!(last.iter().all(|c| c.pools[26][2] == 100.));
    assert!(last.iter().all(|c| c.pools[26][0] < 0.01));
    conserved(&g, 1e-4);
}

#[test]
fn diets_reject_duplicate_self_and_nonfood_sources() {
    use ancient_world::catalog::DietItem;
    for diet in [
        vec![DietItem {
            efficiency: 1.,
            prey: 5,
            weight: 1.,
        }],
        vec![DietItem {
            efficiency: 1.,
            prey: 17,
            weight: 1.,
        }],
        vec![
            DietItem {
                efficiency: 1.,
                prey: 1,
                weight: 0.5,
            },
            DietItem {
                efficiency: 1.,
                prey: 1,
                weight: 0.5,
            },
        ],
    ] {
        let mut c = Catalog::bundled().unwrap();
        c.guilds[0].diet = diet;
        assert!(c.validate().is_err());
    }
    let mut c = Catalog::bundled().unwrap();
    for g in &mut c.guilds {
        g.diet.clear();
    }
    c.validate().unwrap();
}

#[test]
#[ignore = "requires hardware GPU"]
fn alternate_aquatic_food_supports_grazers_without_creating_nutrients() {
    use ancient_world::catalog::DietItem;
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let mut outcomes = vec![];
    for (alternate, efficiency, half_saturation) in [
        (false, 1., 0.),
        (true, 1., 0.),
        (true, 0.2, 0.),
        (true, 1., 10.),
    ] {
        let mut catalog = Catalog::bundled().unwrap();
        catalog.guilds[8].food_half_saturation = half_saturation;
        catalog.guilds[8].diet = if alternate {
            vec![DietItem {
                efficiency,
                prey: 22,
                weight: 1.,
            }]
        } else {
            vec![]
        };
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                resolution: 16,
                ecology_resolution: 16,
                ecology_years_per_epoch: 1,
                solar_scale: 0.,
                ..Default::default()
            },
            catalog,
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        let terrain = g.snapshot().unwrap();
        let mut cells = vec![EcoCell::default(); terrain.len()];
        for (c, t) in cells.iter_mut().zip(&terrain) {
            c.pools[31] = [0., 0., 1., (4095u32 ^ (1 << 8)) as f32];
            if t.meta[0] < 2 {
                c.pools[22] = [1., 0.12, 0.015, 0.];
                c.pools[13] = [0.01, 0.0012, 0.00015, 0.];
            }
        }
        g.restore_ecology(&cells).unwrap();
        g.advance_ecology().unwrap();
        conserved(&g, 5e-5);
        outcomes.push(
            g.ecology
                .snapshot(&gpu, &g.config)
                .unwrap()
                .iter()
                .map(|c| c.pools[13][0] as f64)
                .sum::<f64>(),
        );
    }
    assert!(outcomes[1] > outcomes[0] * 1.01, "{outcomes:?}");
    assert!(
        outcomes[2] < outcomes[1],
        "poor food must reduce growth: {outcomes:?}"
    );
    assert!(
        outcomes[3] < outcomes[1],
        "scarcity must reduce intake: {outcomes:?}"
    );
}

#[test]
#[ignore = "requires hardware GPU"]
fn identical_competitors_share_one_production_budget() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let mut outcomes = vec![];
    for competition in [false, true] {
        let mut catalog = Catalog::bundled().unwrap();
        catalog.producer_competition = competition;
        let template = catalog.plants[0].clone();
        for p in &mut catalog.plants {
            let id = p.id.clone();
            let name = p.name.clone();
            let outer = p.outer;
            *p = template.clone();
            p.id = id;
            p.name = name;
            p.outer = outer;
            p.substrate = 3;
            p.soil_min = 0.;
            p.temp_min = -50.;
            p.temp_max = 60.;
            p.rain_min = 0.;
            p.rain_max = 10000.;
        }
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                resolution: 16,
                ecology_resolution: 16,
                ecology_years_per_epoch: 1,
                ..Default::default()
            },
            catalog,
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        conserved(&g, 5e-5);
        let cells = g.ecology.snapshot(&gpu, &g.config).unwrap();
        if competition {
            assert!(cells
                .iter()
                .any(|c| c.pools[32][1] > 0. && c.pools[0][0] > 0. && c.pools[32][2] == 0.5));
        }
        outcomes.push(cells.iter().map(|c| c.pools[28][0] as f64).sum::<f64>());
    }
    assert!(
        (outcomes[0] - outcomes[1]).abs() / outcomes[0].max(1e-8) < 1e-5,
        "{outcomes:?}"
    );
}

#[test]
fn aquatic_feeding_traits_validate_and_old_diets_default() {
    let mut c = Catalog::bundled().unwrap();
    assert!(c.guilds[9].diet.iter().all(|d| d.prey != 23));
    for bad in [-1., f32::NAN, f32::INFINITY, 1.1] {
        c.guilds[9].diet[0].efficiency = bad;
        assert!(c.validate().is_err());
    }
    c.guilds[9].diet[0].efficiency = 1.;
    for bad in [-1., f32::NAN, f32::INFINITY] {
        c.guilds[9].food_half_saturation = bad;
        assert!(c.validate().is_err());
    }
    let item: ancient_world::catalog::DietItem = toml::from_str("prey = 13\nweight = 1.0").unwrap();
    assert_eq!(item.efficiency, 1.);
}

#[test]
#[ignore = "requires hardware GPU"]
fn version_six_and_seven_upgrade_preserves_stocks_and_continuation() {
    for version in [6, 7] {
        let mut g = make(16, 8);
        g.catalog.producer_competition = false;
        // Old worlds had no column inventory. Keep that baseline throughout this fixture.
        let mut initial = g.snapshot().unwrap();
        for c in &mut initial {
            c.strata = [0.; 4];
        }
        g.restore_cells(&initial, 0).unwrap();
        g.run_epochs(1).unwrap();
        let path =
            std::env::temp_dir().join(format!("ecology-v{version}-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let data = std::fs::read(&path).unwrap();
        let len = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
        let mut header: serde_json::Value = serde_json::from_slice(&data[16..16 + len]).unwrap();
        assert_eq!(header["version"], 8);
        header["version"] = version.into();
        header["config"]
            .as_object_mut()
            .unwrap()
            .remove("wildlife_ecotypes");
        header["ecology"]
            .as_object_mut()
            .unwrap()
            .remove("wildlife_baseline");
        let metadata = serde_json::to_vec(&header).unwrap();
        let terrain_bytes = g.config.cells() as usize * ancient_world::gpu::CELL_BYTES as usize;
        let prefix = terrain_bytes
            + g.config.eco_cells() as usize * ancient_world::ecology::ECO_BYTES as usize;
        let stride = ancient_world::ecology::ENVIRONMENT_BYTES as usize;
        let env_end = 16 + len + prefix + g.config.eco_cells() as usize * stride;
        let mut payload = vec![];
        for c in data[16 + len..16 + len + terrain_bytes]
            .chunks_exact(ancient_world::gpu::CELL_BYTES as usize)
        {
            payload.extend(c);
        }
        for cell in data[16 + len + terrain_bytes..16 + len + prefix]
            .chunks_exact(ancient_world::ecology::ECO_BYTES as usize)
        {
            payload.extend(&cell[..608]);
        }
        for env in data[16 + len + prefix..env_end].chunks_exact(stride) {
            payload.extend(&env[..if version == 6 { 352 } else { 400 }]);
        }
        payload.extend(&data[env_end..data.len() - 8]);
        let checksum = metadata
            .iter()
            .chain(&payload)
            .fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ *b as u64).wrapping_mul(0x100000001b3)
            });
        let mut old = b"ANCIENT2".to_vec();
        old.extend((metadata.len() as u64).to_le_bytes());
        old.extend(metadata);
        old.extend(payload);
        old.extend(checksum.to_le_bytes());
        std::fs::write(&path, old).unwrap();
        let mut loaded = Generator::load(g.gpu.clone(), &path).unwrap();
        let exact = |a: &Generator, b: &Generator| {
            assert_eq!(
                bytemuck::cast_slice::<_, u8>(&a.ecology.snapshot(&a.gpu, &a.config).unwrap()),
                bytemuck::cast_slice::<_, u8>(&b.ecology.snapshot(&b.gpu, &b.config).unwrap())
            );
            assert_eq!(
                bytemuck::cast_slice::<_, u8>(&a.snapshot().unwrap()),
                bytemuck::cast_slice::<_, u8>(&b.snapshot().unwrap())
            );
        };
        exact(&g, &loaded);
        assert_eq!(loaded.ecology.clock.wildlife_baseline, 0);
        assert!(!loaded.config.wildlife_ecotypes);
        for _ in 0..12 {
            g.advance_ecology().unwrap();
            loaded.advance_ecology().unwrap();
        }
        exact(&g, &loaded);
        conserved(&loaded, 5e-5);
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn feeding_is_an_annual_intake_rate_not_a_probability() {
    let mut c = Catalog::bundled().unwrap();
    for rate in [0., 0.8, 2.4, 12.] {
        c.guilds[8].feeding = rate;
        assert!(c.validate().is_ok());
    }
    for rate in [-0.1, 12.1, f32::NAN, f32::INFINITY] {
        c.guilds[8].feeding = rate;
        assert!(c.validate().is_err());
    }
}

#[test]
fn animal_refuge_is_validated_packed_and_optional_for_old_catalogs() {
    let mut c = Catalog::bundled().unwrap();
    let old = toml::to_string(&c.guilds[9])
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("animal_prey_refuge"))
        .collect::<Vec<_>>()
        .join("\n");
    let old: ancient_world::catalog::Guild = toml::from_str(&old).unwrap();
    assert_eq!(old.animal_prey_refuge, 0.);
    for invalid in [-1., f32::NAN, f32::INFINITY] {
        c.guilds[9].animal_prey_refuge = invalid;
        assert!(c.validate().is_err());
    }
    c.guilds[9].animal_prey_refuge = 5e-6;
    c.guilds[9].food_half_saturation = 0.;
    assert!(
        c.validate().is_err(),
        "refuge requires an encounter-limited consumer"
    );
    c.guilds[9].food_half_saturation = 5e-6;
    c.validate().unwrap();
    let entries = c.entries();
    let offset = entries.len() - c.microbes.len() - c.guilds.len();
    assert_eq!(f32::from_bits(entries[offset + 9].ids[0]), 5e-6);
}

#[test]
fn bundled_guilds_are_not_intrinsically_starved_with_unlimited_food() {
    let catalog = Catalog::bundled().unwrap();
    for guild in catalog.guilds {
        // Optimistic bound: unlimited, nutrient-sufficient food, no migration losses.
        // This does not promise viability in any particular habitat.
        let best_efficiency = guild
            .diet
            .iter()
            .map(|d| d.efficiency)
            .fold(0_f32, f32::max);
        let multiplier = (1. + guild.feeding * guild.assimilation * best_efficiency / 12.)
            * (1. - guild.maintenance / 12.)
            * (1. - 0.02 / 12.);
        assert!(
            multiplier > 1.,
            "{} declines even at its energetic upper bound",
            guild.name
        );
    }
}
