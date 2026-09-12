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

#[test]
#[ignore = "requires hardware GPU; full frozen-month schedule equivalence"]
fn frozen_schedule_batch_and_checkpoint_equivalence() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    std::fs::create_dir_all("output").unwrap();
    for seed in [17, 81, 256] {
        let mut batch = Generator::new(
            gpu.clone(),
            Config {
                seed,
                resolution: 64,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        // Zero time must not initialize claims, read back terrain, or create history.
        let reads = batch.history_readback_stats();
        batch.advance_history(0).unwrap();
        assert!(batch.civilizations.is_none());
        assert_eq!(
            batch.history_readback_stats().terrain_bytes,
            reads.terrain_bytes
        );
        batch.advance_ecology().unwrap();
        batch.found_civilizations(8).unwrap();
        batch.enable_society().unwrap();
        batch.enable_politics().unwrap();
        batch.enable_governance().unwrap();
        batch.enable_shipping().unwrap();
        batch.enable_expeditions().unwrap();
        batch.enable_shared_resources().unwrap();
        // Exercise the new explicit allocation policy through the full scheduler,
        // including actual archive restoration, while retaining a priority arm.
        if seed != 81 {
            batch
                .civilizations
                .as_mut()
                .unwrap()
                .service_allocation
                .policy = ancient_world::service_allocation::Policy::Weighted {
                research: 1.,
                culture: 1.,
            };
        }
        batch
            .civilizations
            .as_mut()
            .unwrap()
            .set_demographic_resolution(ancient_world::resolution::Mode::Aggregate, seed != 81)
            .unwrap();
        // Exercise office attendance in two seeds while retaining a legacy arm.
        if seed != 81 {
            batch.enable_offices().unwrap();
            batch
                .civilizations
                .as_mut()
                .unwrap()
                .set_individual_participation(true)
                .unwrap();
            batch
                .civilizations
                .as_mut()
                .unwrap()
                .set_office_service(true)
                .unwrap();
        }
        if seed == 256 {
            batch
                .civilizations
                .as_mut()
                .unwrap()
                .culture
                .as_mut()
                .unwrap()
                .institution_priority = ancient_world::institution_capacity::Priority::Rotating;
            batch
                .civilizations
                .as_mut()
                .unwrap()
                .culture
                .as_mut()
                .unwrap()
                .institution_funding = ancient_world::institution_funding::Policy::Operating;
            batch
                .civilizations
                .as_mut()
                .unwrap()
                .culture
                .as_mut()
                .unwrap()
                .named_administration = true;
            batch
                .civilizations
                .as_mut()
                .unwrap()
                .culture
                .as_mut()
                .unwrap()
                .institution_work_policy =
                ancient_world::institution_capacity::WorkPolicy::EssentialFirst;
        }
        let path = format!("output/schedule-{}-{seed}.world", std::process::id());
        batch.save(std::path::Path::new(&path)).unwrap();
        let mut single = Generator::load(gpu.clone(), std::path::Path::new(&path)).unwrap();
        batch.advance_history(24).unwrap();
        for _ in 0..24 {
            single.advance_history(1).unwrap();
        }
        assert_same(&batch, &single, seed, 24);
        single.save(std::path::Path::new(&path)).unwrap();
        let mut resumed = Generator::load(gpu.clone(), std::path::Path::new(&path)).unwrap();
        batch.advance_history(12).unwrap();
        resumed.advance_history(5).unwrap();
        resumed.advance_history(7).unwrap();
        assert_same(&batch, &resumed, seed, 36);
        let before = serde_json::to_value(&batch.civilizations).unwrap();
        batch.advance_history(0).unwrap();
        assert_eq!(before, serde_json::to_value(&batch.civilizations).unwrap());
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
#[ignore = "requires hardware GPU; arrival timing intervention"]
fn due_food_cargo_prevents_current_consumption_shortage() {
    use ancient_world::economy::{Cargo, FOOD};
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    std::fs::create_dir_all("output").unwrap();
    let mut due = Generator::new(
        gpu.clone(),
        Config {
            resolution: 64,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    due.found_civilizations(5).unwrap();
    // Isolate arrival timing: identical cargo and source debit; only due date differs.
    let h = due.civilizations.as_mut().unwrap();
    h.society = None;
    h.sites[1].stocks.stock[1] += h.sites[0].stocks.stock[1];
    h.sites[0].stocks.stock[1] = 0.;
    h.sites[0].stocks.habitat[0] = 0.;
    let food = h.sites[1].stocks.stock[1].min(h.sites[0].stocks.stock[0] * 36.);
    assert!(food > 0.);
    h.sites[1].stocks.stock[1] -= food;
    h.cargo.push(Cargo {
        voyage_clock: None,
        from: 1,
        to: 0,
        good: FOOD as u32,
        kg: food,
        paid: 1.,
        arrives: h.month + 1,
        freight_stops: vec![],
        sea_lane: None,
        weather_delay_months: 0,
    });
    let path = format!("output/schedule-arrival-{}.world", std::process::id());
    due.save(std::path::Path::new(&path)).unwrap();
    let mut later = Generator::load(gpu, std::path::Path::new(&path)).unwrap();
    later
        .civilizations
        .as_mut()
        .unwrap()
        .cargo
        .last_mut()
        .unwrap()
        .arrives += 1;
    due.advance_history(1).unwrap();
    later.advance_history(1).unwrap();
    let a = &due.civilizations.as_ref().unwrap().sites[0].stocks;
    let b = &later.civilizations.as_ref().unwrap().sites[0].stocks;
    assert!(
        a.ledger[1] > b.ledger[1],
        "due grain must enter current consumption: {:?} {:?}",
        a,
        b
    );
    assert!(
        a.stock[3] < b.stock[3],
        "due grain must reduce current shortage"
    );
    assert!(later
        .civilizations
        .as_ref()
        .unwrap()
        .cargo
        .iter()
        .any(|c| c.to == 0 && c.kg == food));
    std::fs::remove_file(path).unwrap();
}
