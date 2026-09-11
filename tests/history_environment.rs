//! Differential execution test: the reference still reads the entire world.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
    history_environment::HistoryReadbackMode,
};

fn assert_same(a: &Generator, b: &Generator, seed: u32, month: u32) {
    assert_eq!(
        serde_json::to_value(&a.civilizations).unwrap(),
        serde_json::to_value(&b.civilizations).unwrap(),
        "history seed {seed}, month {month}"
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&a.snapshot().unwrap()),
        bytemuck::cast_slice::<_, u8>(&b.snapshot().unwrap()),
        "terrain seed {seed}, month {month}"
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&a.ecology.snapshot(&a.gpu, &a.config).unwrap()),
        bytemuck::cast_slice::<_, u8>(&b.ecology.snapshot(&b.gpu, &b.config).unwrap()),
        "ecology seed {seed}, month {month}"
    );
    assert_eq!(
        serde_json::to_value(&a.ecology.clock).unwrap(),
        serde_json::to_value(&b.ecology.clock).unwrap()
    );
}

#[test]
#[ignore = "requires hardware GPU; matched seeds with full-state comparisons"]
fn gathered_history_matches_full_readbacks() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    std::fs::create_dir_all("output").unwrap();
    let resolution = std::env::var("HISTORY_GATHER_RESOLUTION")
        .map(|s| s.parse::<u32>().unwrap())
        .unwrap_or(64);
    let ecology_resolution = std::env::var("HISTORY_GATHER_ECOLOGY_RESOLUTION")
        .map(|s| s.parse::<u32>().unwrap())
        .unwrap_or(16);
    let seeds = std::env::var("HISTORY_GATHER_SEEDS")
        .unwrap_or_else(|_| "17,81,256".into())
        .split(',')
        .map(|s| s.parse::<u32>().unwrap())
        .collect::<Vec<_>>();
    println!(
        "adapter {:?}; terrain {resolution}; ecology {ecology_resolution}",
        gpu.adapter_info
    );
    for seed in seeds {
        let mut compact = Generator::new(
            gpu.clone(),
            Config {
                seed,
                resolution,
                ecology_resolution,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        compact.advance_ecology().unwrap();
        compact.found_civilizations(16).unwrap();
        compact.enable_society().unwrap();
        assert!(
            !compact
                .civilizations
                .as_ref()
                .unwrap()
                .society
                .as_ref()
                .unwrap()
                .routes
                .is_empty(),
            "fixture must exercise roads as well as sea routes"
        );
        compact.enable_politics().unwrap();
        compact.enable_governance().unwrap();
        compact.enable_shipping().unwrap();
        compact.enable_expeditions().unwrap();
        compact.enable_discoveries().unwrap();
        compact.enable_shared_resources().unwrap();
        compact.advance_history(2).unwrap();
        compact.enable_living_history().unwrap();
        compact.enable_environmental_returns().unwrap();
        let path = format!("output/history-gather-{}-{seed}.world", std::process::id());
        compact.save(std::path::Path::new(&path)).unwrap();
        let mut full = Generator::load(gpu.clone(), std::path::Path::new(&path)).unwrap();
        compact.set_history_readback_mode(HistoryReadbackMode::Gathered);
        full.set_history_readback_mode(HistoryReadbackMode::Full);
        // Include strong deterministic wet/dry forcing, not just benign weather.
        for g in [&mut compact, &mut full] {
            let w = &mut g
                .civilizations
                .as_mut()
                .unwrap()
                .economy_catalog
                .as_mut()
                .unwrap()
                .weather;
            w.drought_probability = 0.5;
            w.drought_severity = 0.9;
            w.storm_probability = 0.5;
            w.storm_multiplier = 4.;
        }
        let mut compact_ms = 0.;
        let mut full_ms = 0.;
        for month in 1..=36 {
            let start = std::time::Instant::now();
            compact.advance_history(1).unwrap();
            compact_ms += start.elapsed().as_secs_f64() * 1000.;
            let start = std::time::Instant::now();
            full.advance_history(1).unwrap();
            full_ms += start.elapsed().as_secs_f64() * 1000.;
            assert_same(&compact, &full, seed, month);
        }
        println!("seed {seed}: history wall compact {compact_ms:.3} ms, full {full_ms:.3} ms (excluding comparison readbacks)");
        let a = compact.history_readback_stats();
        let b = full.history_readback_stats();
        println!("seed {seed}: compact {a:?}; full {b:?}; sites {}, road cells {}, sea cells {}, expedition routes {}",
            compact.civilizations.as_ref().unwrap().sites.len(),
            compact.civilizations.as_ref().unwrap().society.as_ref().unwrap().routes.iter().map(|r|r.cells.len()).sum::<usize>(),
            compact.civilizations.as_ref().unwrap().shipping.as_ref().unwrap().lanes.iter().map(|r|r.cells.len()).sum::<usize>(),
            compact.civilizations.as_ref().unwrap().expeditions.as_ref().unwrap().routes.len());
        assert!(a.gathered_refreshes >= 30, "{a:?}");
        assert_eq!(b.full_refreshes, 36);
        assert!(a.terrain_bytes < b.terrain_bytes / 2, "{a:?} vs {b:?}");
        assert!(
            compact
                .ecology
                .budget(&gpu, &compact.config)
                .unwrap()
                .within_tolerance
        );
        // Resume with an empty CPU cache, using a different execution batch size.
        compact.save(std::path::Path::new(&path)).unwrap();
        let mut resumed = Generator::load(gpu.clone(), std::path::Path::new(&path)).unwrap();
        compact.advance_history(4).unwrap();
        for _ in 0..4 {
            resumed.advance_history(1).unwrap();
            full.advance_history(1).unwrap();
        }
        assert_same(&compact, &resumed, seed, 40);
        assert_same(&compact, &full, seed, 40);
        std::fs::remove_file(path).unwrap();
    }
}
