use ancient_world::{
    catalog::{Catalog, DepositSetting},
    config::Config,
    gpu::{ContextGpu, Generator, Stage, NONE},
    grid,
};

#[test]
fn deposit_catalog_defaults_and_bounds() {
    let c = Catalog::bundled().unwrap();
    assert!(c.geological_provinces);
    assert!(c
        .minerals
        .iter()
        .all(|m| m.deposit_setting != DepositSetting::Legacy));
    for bad in [0., -1., f32::NAN, f32::INFINITY, 5001.] {
        let mut c = c.clone();
        c.minerals[0].province_scale_km = bad;
        assert!(c.validate().is_err());
    }
    let mut value = serde_json::to_value(&c).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .remove("geological_provinces");
    for m in value["minerals"].as_array_mut().unwrap() {
        m.as_object_mut().unwrap().remove("deposit_setting");
        m.as_object_mut().unwrap().remove("province_scale_km");
    }
    let old: Catalog = serde_json::from_value(value).unwrap();
    old.validate().unwrap();
    assert!(!old.geological_provinces);
    assert!(old
        .minerals
        .iter()
        .all(|m| m.deposit_setting == DepositSetting::Legacy));
}
fn config(seed: u32) -> Config {
    Config {
        seed,
        resolution: 64,
        ecology_resolution: 32,
        ecology_years_per_epoch: 1,
        ..Default::default()
    }
}

#[test]
#[ignore = "requires hardware GPU"]
fn provinces_improve_neighbor_coherence_and_preserve_host_rules() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    for seed in [17, 81, 256] {
        let mut agreement = vec![];
        for enabled in [false, true] {
            let mut cat = Catalog::bundled().unwrap();
            cat.geological_provinces = enabled;
            let mut g = Generator::new(gpu.clone(), config(seed), cat).unwrap();
            g.run_epochs(1).unwrap();
            let cells = g.snapshot().unwrap();
            let mut matches = 0;
            let mut seam_matches = 0;
            let mut seams = 0;
            let mut deposits = 0;
            let mut sum = 0.;
            for (i, c) in cells.iter().enumerate() {
                let j = grid::neighbor(i as u32, 64, 1, 0) as usize;
                matches += usize::from(c.ids[0] == cells[j].ids[0]);
                if i / (64 * 64) != j / (64 * 64) {
                    seams += 1;
                    seam_matches += usize::from(c.ids[0] == cells[j].ids[0]);
                }
                if c.meta[1] != NONE {
                    let m = &g.catalog.minerals[c.meta[1] as usize];
                    assert!(m.hosts.contains(&g.catalog.rocks[c.ids[0] as usize].id));
                    assert!((0. ..=if enabled { 1. } else { 1.5 }).contains(&c.geology[2]));
                    assert!(c.geology[3] >= 0.);
                    deposits += 1;
                    sum += c.geology[2] as f64;
                }
            }
            let fraction = matches as f64 / cells.len() as f64;
            agreement.push(fraction);
            eprintln!("geology seed {seed} enabled {enabled}: rock agreement {fraction:.4}, seam {:.4}, deposits {deposits}, mean potential {:.5}",seam_matches as f64/seams as f64,sum/cells.len() as f64);
            assert!(deposits > 0);
            if enabled {
                assert!(seam_matches as f64 / seams as f64 > 0.3);
                let id = cells
                    .iter()
                    .position(|c| c.meta[0] == 2 && c.meta[1] != NONE)
                    .unwrap();
                let region = g
                    .generate_region(grid::cell_direction(id as u32, 64), 100., 16)
                    .unwrap();
                assert_eq!(region.version, 3);
                for c in &region.cells {
                    let parent = cells[c.route[2] as usize];
                    assert_eq!(
                        c.mineral_index(),
                        (parent.meta[1] != NONE).then_some(parent.meta[1] as usize)
                    );
                    assert_eq!(c.forcing[2], parent.geology[2]);
                    assert_eq!(c.strata, parent.strata);
                    assert_eq!(
                        c.rocks[..3],
                        [parent.ids[0], parent.meta[2], parent.meta[3]]
                    );
                    assert!(c.forcing[3].is_finite() && c.forcing[3] >= 0.);
                }
                let decoded: ancient_world::region::Region =
                    serde_json::from_slice(&serde_json::to_vec(&region).unwrap()).unwrap();
                assert_eq!(
                    bytemuck::cast_slice::<_, u8>(&region.cells),
                    bytemuck::cast_slice::<_, u8>(&decoded.cells)
                );
            }
        }
        assert!(agreement[1] > agreement[0] + 0.1, "{agreement:?}");
    }
}

#[test]
#[ignore = "requires hardware GPU"]
fn magmatic_activity_changes_potential_without_bypassing_hosts() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let mut cat = Catalog::bundled().unwrap();
    let mineral = cat
        .minerals
        .iter()
        .position(|m| m.id == "magnetite")
        .unwrap();
    let host = cat.rocks.iter().position(|r| r.id == "gabbro").unwrap();
    for (i, m) in cat.minerals.iter_mut().enumerate() {
        m.abundance = if i == mineral { 1. } else { 0. };
    }
    let mut g = Generator::new(gpu, config(17), cat).unwrap();
    let mut cells = g.snapshot().unwrap();
    for c in &mut cells {
        c.ids[0] = host as u32;
        c.terrain[3] = 1500.;
        c.geology[0] = 0.;
    }
    let mut means = vec![];
    for activity in [0., 1.] {
        for c in &mut cells {
            c.geology[0] = activity;
        }
        g.restore_cells(&cells, 0).unwrap();
        g.progress.stage = Stage::Water;
        g.advance().unwrap();
        let out = g.snapshot().unwrap();
        means.push(out.iter().map(|c| c.geology[2] as f64).sum::<f64>());
        assert!(out
            .iter()
            .all(|c| c.meta[1] == NONE || c.meta[1] == mineral as u32));
    }
    assert!(means[1] > means[0] * 3., "{means:?}");
}

#[test]
#[ignore = "requires hardware GPU"]
fn finite_columns_expose_bedrock_and_lithify_without_creating_volume() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let mut cfg = config(17);
    cfg.resolution = 16;
    cfg.ecology_resolution = 16;
    cfg.geological_step_myr = 1.;
    let mut g = Generator::new(gpu, cfg, Catalog::bundled().unwrap()).unwrap();
    let rock = |id: &str| g.catalog.rocks.iter().position(|r| r.id == id).unwrap() as u32;
    let top = rock("limestone");
    let middle = rock("sandstone");
    let base = rock("granite");
    for deposition in [false, true] {
        let mut cells = g.snapshot().unwrap();
        for c in &mut cells {
            c.terrain = [500., if deposition { 105. } else { 0. }, 0., 100.];
            c.strata = if deposition {
                [10., 20., 100., 0.]
            } else {
                [0.000001, 0.000001, 10., 0.]
            };
            c.ids[0] = top;
            c.meta = [3, NONE, middle, base];
            c.water = [0.; 4];
            c.life[3] = 0.;
            c.hydro = [500., 20., 0., 0.];
            c.routing[0] = NONE;
        }
        g.restore_cells(&cells, 0).unwrap();
        g.progress.stage = Stage::Water;
        g.advance().unwrap();
        let out = g.snapshot().unwrap();
        for (old, new) in cells.iter().zip(&out) {
            let volume = |c: &ancient_world::gpu::Cell| {
                c.terrain[1] as f64 + c.strata[..3].iter().map(|v| *v as f64).sum::<f64>()
            };
            assert!(
                (volume(old) - volume(new)).abs() < 0.00005,
                "{} vs {}",
                volume(old),
                volume(new)
            );
            assert!(new.strata[3] > 0.);
            assert!(new.strata.iter().all(|v| v.is_finite() && *v >= 0.));
            if deposition {
                assert!(new.terrain[1] < old.terrain[1]);
            } else {
                assert_eq!(new.ids[0], base);
                assert_eq!(new.strata[1], 0.);
                assert_eq!(new.strata[2], 0.);
            }
        }
    }
}

#[test]
fn column_depth_queries_respect_contacts_and_finite_base() {
    let c = ancient_world::gpu::Cell {
        ids: [1, 0, 0, 0],
        meta: [0, 0, 2, 3],
        strata: [10., 20., 30., 0.],
        ..Default::default()
    };
    assert_eq!(c.rock_at_depth(0.), Some(1));
    assert_eq!(c.rock_at_depth(10.), Some(2));
    assert_eq!(c.rock_at_depth(30.), Some(3));
    for depth in [60., -1., f32::NAN, f32::INFINITY] {
        assert_eq!(c.rock_at_depth(depth), None);
    }
    assert_eq!(ancient_world::gpu::Cell::default().rock_at_depth(0.), None);
}

#[test]
fn rock_settings_and_archive_defaults_are_validated() {
    let c = Catalog::bundled().unwrap();
    assert!(c.process_geology);
    assert!(c.rocks.iter().all(|r| r.setting > 0));
    let mut bad = c.clone();
    bad.rocks[0].setting = 4;
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(&c).unwrap();
    value.as_object_mut().unwrap().remove("process_geology");
    for r in value["rocks"].as_array_mut().unwrap() {
        r.as_object_mut().unwrap().remove("setting");
    }
    let old: Catalog = serde_json::from_value(value).unwrap();
    old.validate().unwrap();
    assert!(!old.process_geology);
}

#[test]
#[ignore = "requires hardware GPU"]
fn process_regions_reduce_fragmentation_without_erasing_diversity() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    for seed in [17, 81, 256] {
        let mut agreement = Vec::new();
        for enabled in [false, true] {
            let mut c = Catalog::bundled().unwrap();
            c.process_geology = enabled;
            let mut g = Generator::new(gpu.clone(), config(seed), c).unwrap();
            g.run_epochs(1).unwrap();
            let cells = g.snapshot().unwrap();
            let mut matched = 0;
            let mut count = 0;
            let mut kinds = std::collections::BTreeSet::new();
            for (i, c) in cells.iter().enumerate() {
                if c.meta[0] < 2 {
                    continue;
                }
                kinds.insert(c.ids[0]);
                let n = &cells[grid::neighbor(i as u32, 64, 1, 0) as usize];
                if n.meta[0] < 2 {
                    continue;
                }
                count += 1;
                matched += usize::from(c.ids[0] == n.ids[0]);
                if c.meta[1] != NONE {
                    assert!(g.catalog.minerals[c.meta[1] as usize]
                        .hosts
                        .contains(&g.catalog.rocks[c.ids[0] as usize].id));
                }
            }
            let rate = matched as f64 / count as f64;
            agreement.push(rate);
            eprintln!("process geology seed={seed} enabled={enabled} land_neighbor_agreement={rate:.4} rock_types={}",kinds.len());
            assert!(kinds.len() >= 8);
        }
        assert!(agreement[1] > agreement[0] + 0.15);
    }
}
