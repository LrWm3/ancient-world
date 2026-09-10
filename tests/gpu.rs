//! Hardware integration tests are explicit: cargo test --test gpu -- --ignored --test-threads=1
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{validate_cells, Cell, ContextGpu, Generator, Stage, NONE},
    grid,
    viewer::{Camera, MapRenderer},
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};
fn make(n: u32, seed: u32) -> Generator {
    Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: n,
            seed,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap()
}
fn check_routes(g: &Generator, cells: &[Cell]) {
    for (i, c) in cells.iter().enumerate() {
        if c.meta[0] < 2 {
            assert_eq!(c.routing[0], NONE);
            continue;
        }
        let j = c.routing[0] as usize;
        assert!(j < cells.len());
        let b = cells[j];
        assert!(
            b.hydro[0] < c.hydro[0] || (b.hydro[0] == c.hydro[0] && b.routing[1] < c.routing[1]),
            "cycle rank at {i}"
        );
        assert!([(-1, 0), (1, 0), (0, -1), (0, 1)]
            .iter()
            .any(|&(dx, dy)| grid::neighbor(i as u32, g.config.resolution, dx, dy) == j as u32));
    }
}
#[test]
#[ignore = "requires a hardware GPU"]
fn gpu_drainage_matches_priority_flood() {
    let mut g = make(16, 42);
    let mut cells = g.snapshot().unwrap();
    // A flat world with nested bowls, two terminals, and routes across cube faces.
    for (i, c) in cells.iter_mut().enumerate() {
        c.meta[0] = 2;
        c.terrain[0] = 500. + ((i * 31) % 7) as f32 * 10.;
        c.water = [0.; 4];
        if i % 11 == 0 {
            c.terrain[0] = 300.;
        }
        if i % 37 == 0 {
            c.terrain[0] = 700.;
        }
    }
    for i in [0, 1100] {
        cells[i].meta[0] = 0;
        cells[i].terrain[0] = -100.;
    }
    g.restore_cells(&cells, 0).unwrap();
    g.rebuild_drainage().unwrap();
    let result = g.snapshot().unwrap();
    check_routes(&g, &result);
    let mut best = vec![u32::MAX; cells.len()];
    let mut heap = BinaryHeap::new();
    for (i, c) in cells.iter().enumerate() {
        if c.meta[0] == 0 {
            best[i] = 0;
            heap.push(Reverse((0u32, i)));
        }
    }
    while let Some(Reverse((height, i))) = heap.pop() {
        if height != best[i] {
            continue;
        }
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let j = grid::neighbor(i as u32, 16, dx, dy) as usize;
            let h = height.max(cells[j].terrain[0].max(0.) as u32);
            if h < best[j] {
                best[j] = h;
                heap.push(Reverse((h, j)));
            }
        }
    }
    for (i, c) in result.iter().enumerate() {
        assert_eq!(c.hydro[0], best[i] as f32, "spill mismatch at {i}");
    }
}
#[test]
#[ignore = "requires a hardware GPU"]
fn gpu_history_checkpoints_and_exports() {
    let mut g = make(32, 79);
    g.run_epochs(2).unwrap();
    let first = g.snapshot().unwrap();
    validate_cells(&first, &g.catalog).unwrap();
    let path =
        std::env::temp_dir().join(format!("ancient-world-test-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    g.run_epochs(2).unwrap();
    resumed.run_epochs(2).unwrap();
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&g.snapshot().unwrap()),
        bytemuck::cast_slice::<_, u8>(&resumed.snapshot().unwrap())
    );
    assert_eq!(g.planet_state().unwrap(), resumed.planet_state().unwrap());
    let renderer = MapRenderer::new(&g, 257, 129).unwrap();
    for layer in 0..31 {
        renderer.render(&g, layer, Camera::atlas(), None);
    }
    let png = path.with_extension("png");
    renderer.export_png(&g, &png).unwrap();
    assert_eq!(image::image_dimensions(&png).unwrap(), (257, 129));
    let regional = MapRenderer::new(&g, 128, 128).unwrap();
    let center = grid::cell_direction(
        first.iter().position(|c| c.meta[0] == 2).unwrap() as u32,
        32,
    );
    regional.export_region(&g, center, 800., &png).unwrap();
    let pixels = std::fs::read(&png).unwrap();
    regional.export_region(&g, center, 800., &png).unwrap();
    assert_eq!(pixels, std::fs::read(&png).unwrap());
    assert_eq!(image::image_dimensions(&png).unwrap(), (128, 128));
    assert!(regional.export_region(&g, center, 0., &png).is_err());
    let mut bytes = std::fs::read(&path).unwrap();
    let idx = bytes.len() - 20;
    bytes[idx] ^= 1;
    std::fs::write(&path, &bytes).unwrap();
    assert!(Generator::load(g.gpu.clone(), &path).is_err());
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(png.with_extension("region.json")).unwrap();
    std::fs::remove_file(png).unwrap();
    for c in g.snapshot().unwrap() {
        if c.ids[2] != NONE {
            let p = &g.catalog.plants[c.ids[2] as usize];
            assert!(c.hydro[1] >= p.temp_min && c.hydro[1] <= p.temp_max);
            assert!(c.hydro[2] >= p.rain_min && c.hydro[2] <= p.rain_max);
            assert!(c.life[1] >= p.soil_min);
            assert!(!p.outer || c.meta[0] == 3);
        }
    }
}
#[test]
#[ignore = "requires a hardware GPU"]
fn gpu_geography_seed_suite_and_long_run() {
    let mut g = make(64, 0);
    for seed in [0, 42, 999] {
        g = Generator::new(
            g.gpu.clone(),
            Config {
                resolution: 64,
                seed,
                ..Default::default()
            },
            g.catalog.clone(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        let cells = g.snapshot().unwrap();
        let n = g.config.resolution;
        let mut visited = vec![false; cells.len()];
        let mut components = [0u32; 4];
        let mut island_areas = Vec::new();
        let mut lake_area = 0.;
        for start in 0..cells.len() {
            if visited[start] {
                continue;
            }
            let region = cells[start].meta[0];
            components[region as usize] += 1;
            let mut q = VecDeque::from([start]);
            visited[start] = true;
            let mut component_area = 0.;
            while let Some(i) = q.pop_front() {
                component_area += grid::solid_angle(i as u32, n);
                if region == 2 {
                    // Outer shore starts at >= .867 rad: reserve > 1,200 km of open lake.
                    assert!(grid::cell_direction(i as u32, n)[2].acos() < 0.67);
                }
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let j = grid::neighbor(i as u32, n, dx, dy) as usize;
                    assert!(
                        !((region == 0 && cells[j].meta[0] == 1)
                            || (region == 1 && cells[j].meta[0] == 0)),
                        "great lake opens to ocean"
                    );
                    if !visited[j] && cells[j].meta[0] == region {
                        visited[j] = true;
                        q.push_back(j);
                    }
                }
            }
            if region == 2 {
                island_areas.push(component_area);
            }
            if region == 1 {
                lake_area = component_area;
            }
        }
        assert_eq!(components, [1, 1, 5, 1]);
        island_areas.sort_by(f64::total_cmp);
        assert!(
            island_areas[4] > island_areas[0] * 1.8,
            "islands too uniform: {island_areas:?}"
        );
        assert!(
            island_areas.iter().sum::<f64>() < lake_area * 0.18,
            "inner land crowds the lake"
        );
        assert!(cells.iter().any(|c| c.meta[0] == 2 && c.water[3] > 0.));
        assert!(cells
            .iter()
            .any(|c| c.meta[0] >= 2 && c.hydro[0] > c.terrain[0] + 0.1));
    }
    g.run_epochs(30).unwrap();
    validate_cells(&g.snapshot().unwrap(), &g.catalog).unwrap();
    assert_eq!(g.progress.stage, Stage::Boundary);
    let state = g.planet_state().unwrap();
    // Multiple ecological years can reach the explicit artistic lake-level bound.
    assert!((85.0..=240.0).contains(&state[0]));
    assert!(state[2].is_finite());
    assert!(state[1] >= 0.);
}
#[test]
#[ignore = "requires a hardware GPU"]
fn nonconvergence_is_an_error() {
    let mut g = make(32, 3);
    let mut cells = g.snapshot().unwrap();
    for c in &mut cells {
        c.meta[0] = 2;
        c.terrain[0] = 500.;
    }
    cells[0].meta[0] = 0;
    cells[0].terrain[0] = -100.;
    g.restore_cells(&cells, 0).unwrap();
    g.config.max_drainage_iterations = 17;
    assert!(g.rebuild_drainage().is_err());
    assert!(g.error.is_some());
    assert_ne!(g.progress.stage, Stage::Boundary);
    assert!(g
        .save(std::env::temp_dir().join("should-not-exist.world"))
        .is_err());
    // Resume an odd-length compact-buffer batch without restarting or changing routes.
    assert_eq!(g.progress.iteration, 17);
    g.config.max_drainage_iterations = 4096;
    g.error = None;
    while g.progress.stage != Stage::Climate {
        g.advance().unwrap();
    }
    check_routes(&g, &g.snapshot().unwrap());
}

#[test]
#[ignore = "requires a hardware GPU"]
fn water_and_sediment_budgets() {
    let mut g = make(32, 42);
    while g.progress.stage != Stage::LakeCollect {
        g.advance().unwrap();
    }
    let before = g.snapshot().unwrap();
    let n = g.config.resolution;
    let radius = g.config.radius_km as f64 * 1000.;
    let area = |i: usize| grid::solid_angle(i as u32, n) * radius * radius;
    let mut runoff = 0.;
    let mut terminal = 0.;
    let mut retained = 0.;
    for (i, c) in before.iter().enumerate() {
        runoff += c.life[3] as f64 * area(i);
        if c.routing[0] == NONE {
            terminal += c.water[3] as f64 * 31557600.;
        } else {
            let mut incoming = c.life[3] as f64 * area(i);
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let j = grid::neighbor(i as u32, n, dx, dy) as usize;
                if before[j].routing[0] == i as u32 {
                    incoming += before[j].water[3] as f64 * 31557600.;
                }
            }
            retained +=
                incoming.min(((c.hydro[0] - c.terrain[0] - c.water[0]).max(0.)) as f64 * area(i));
        }
    }
    assert!(
        (runoff - terminal - retained).abs() / runoff.max(1.) < 0.002,
        "runoff {runoff}, terminal {terminal}, retained {retained}"
    );
    while g.progress.stage != Stage::Water {
        g.advance().unwrap();
    }
    let old = g.snapshot().unwrap();
    g.advance().unwrap();
    let new = g.snapshot().unwrap();
    let mut source = 0.;
    let mut storage = 0.;
    let mut moved_out = 0.;
    let mut moved_in = 0.;
    for (i, (a, b)) in old.iter().zip(&new).enumerate() {
        if a.meta[0] >= 2 {
            source += (a.budget[0] - a.budget[1]) as f64 * area(i);
            storage += ((b.water[0] + b.water[1] + b.water[2])
                - (a.water[0] + a.water[1] + a.water[2])) as f64
                * area(i);
        }
        moved_out += b.budget[2] as f64 * area(i);
        moved_in += b.budget[3] as f64 * area(i);
    }
    assert!(
        (source - storage - terminal).abs() / source.max(1.) < 0.003,
        "precip minus evaporation {source}; retained {storage}; discharged {terminal}"
    );
    assert!(
        (moved_out - moved_in).abs() / moved_out.max(1.) < 0.002,
        "sediment transfer mismatch"
    );
}

#[test]
#[ignore = "requires a hardware GPU"]
fn secondary_lakes_share_surface_and_conserve_volume_across_seam() {
    let mut g = make(16, 42);
    let mut cells = g.snapshot().unwrap();
    for c in &mut cells {
        c.meta[0] = 0;
        c.routing[2] = 0;
        c.water[0] = 0.;
    }
    let a = 15u32 + 8 * 16;
    let b = grid::neighbor(a, 16, 1, 0);
    let dry = grid::neighbor(b, 16, 0, 1);
    assert_ne!(a / (16 * 16), b / (16 * 16));
    for (i, h, w) in [(a, 300., 40.), (b, 310., 0.), (dry, 350., 0.)] {
        let c = &mut cells[i as usize];
        c.meta[0] = 2;
        c.routing[2] = a + 2;
        c.terrain[0] = h;
        c.water[0] = w;
        c.hydro[0] = 400.;
    }
    let ids = [a, b, dry];
    let volume = |state: &[ancient_world::gpu::Cell]| {
        ids.iter()
            .map(|&i| state[i as usize].water[0] as f64 * grid::solid_angle(i, 16))
            .sum::<f64>()
    };
    let before = volume(&cells);
    g.restore_cells(&cells, 0).unwrap();
    g.equilibrate_lakes().unwrap();
    let result = g.snapshot().unwrap();
    assert!((volume(&result) - before).abs() / before < 0.0001);
    let surface = |i: u32| result[i as usize].terrain[0] + result[i as usize].water[0];
    assert!((surface(a) - surface(b)).abs() < 0.002);
    assert_eq!(result[dry as usize].water[0], 0.);
    g.equilibrate_lakes().unwrap();
    assert!((volume(&g.snapshot().unwrap()) - before).abs() / before < 0.0001);
}

#[test]
#[ignore = "requires a hardware GPU"]
fn nested_pools_wait_for_saddle_and_overflow_conservatively() {
    let mut g = make(16, 42);
    let mut cells = g.snapshot().unwrap();
    for c in &mut cells {
        c.meta[0] = 2;
        c.terrain[0] = 1000.;
        c.water[0] = 0.;
        c.routing = [NONE, 0, NONE, 0];
        c.hydro[0] = 1000.;
    }
    let ids = [8 * 16 + 6, 8 * 16 + 7, 8 * 16 + 8];
    for (&i, h) in ids.iter().zip([300., 350., 300.]) {
        cells[i].terrain[0] = h;
        cells[i].hydro[0] = 400.;
        cells[i].routing[2] = ids[0] as u32 + 2;
    }
    cells[ids[0]].water[0] = 30.;
    g.restore_cells(&cells, 0).unwrap();
    g.equilibrate_lakes().unwrap();
    let low = g.snapshot().unwrap();
    assert_eq!(low[ids[2]].water[0], 0., "water crossed a dry saddle");
    assert!((low[ids[0]].water[0] - 30.).abs() < 0.001);
    cells[ids[0]].water[0] = 160.;
    let volume = |cs: &[ancient_world::gpu::Cell]| {
        ids.iter()
            .map(|&i| cs[i].water[0] as f64 * grid::solid_angle(i as u32, 16))
            .sum::<f64>()
    };
    let before = volume(&cells);
    g.restore_cells(&cells, 0).unwrap();
    g.equilibrate_lakes().unwrap();
    let high = g.snapshot().unwrap();
    assert!(high[ids[2]].water[0] > 40.);
    assert!((volume(&high) - before).abs() / before < 0.0001);
    let levels = ids.map(|i| high[i].terrain[0] + high[i].water[0]);
    assert!((levels[0] - levels[2]).abs() < 0.002);
}

#[test]
#[ignore = "requires a hardware GPU"]
fn regional_fields_have_acyclic_routes_and_accounted_runoff() {
    let g = make(32, 42);
    let parent = g.snapshot().unwrap();
    let id = parent.iter().position(|c| c.meta[0] == 2).unwrap() as u32;
    let center = grid::cell_direction(id, 32);
    let region = g.generate_region(center, 300., 64).unwrap();
    let again = g.generate_region(center, 300., 64).unwrap();
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&region.cells),
        bytemuck::cast_slice::<_, u8>(&again.cells)
    );
    // Independent CPU priority flood reference on the generated physical bed.
    let n = region.resolution as usize;
    let mut spill = vec![f32::INFINITY; n * n];
    let mut heap = BinaryHeap::new();
    for (i, c) in region.cells.iter().enumerate() {
        if i % n == 0 || i % n == n - 1 || i / n == 0 || i / n == n - 1 || c.route[3] == 1 {
            spill[i] = c.surface[0];
            heap.push(Reverse((spill[i].to_bits(), i)));
        }
    }
    while let Some(Reverse((bits, i))) = heap.pop() {
        let h = f32::from_bits(bits);
        if h > spill[i] {
            continue;
        }
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let x = (i % n) as i32 + dx;
                let y = (i / n) as i32 + dy;
                if x < 0 || y < 0 || x >= n as i32 || y >= n as i32 {
                    continue;
                }
                let j = y as usize * n + x as usize;
                let next = h.max(region.cells[j].surface[0]);
                if next < spill[j] {
                    spill[j] = next;
                    heap.push(Reverse((next.to_bits(), j)));
                }
            }
        }
    }
    for (c, expected) in region.cells.iter().zip(spill) {
        assert!((c.water[1] - expected).abs() < 0.002);
    }
    let mut input = 0f64;
    let mut accounted = 0f64;
    let mut branching = false;
    let mut incoming = vec![0; region.cells.len()];
    for (i, c) in region.cells.iter().enumerate() {
        input += c.water[3] as f64 + c.forcing[0] as f64;
        accounted += c.water[0] as f64 * c.surface[3] as f64;
        if c.route[0] == NONE {
            accounted += c.water[2] as f64;
        } else {
            let j = c.route[0] as usize;
            let b = &region.cells[j];
            assert!(
                b.water[1] < c.water[1] || (b.water[1] == c.water[1] && b.route[1] < c.route[1]),
                "cycle at {i}"
            );
            incoming[j] += 1;
            branching |= incoming[j] > 1;
        }
        if c.ids[2] != NONE {
            let plant = &g.catalog.plants[c.ids[2] as usize];
            assert!(c.climate[0] >= plant.temp_min && c.climate[0] <= plant.temp_max);
            assert!(c.climate[1] >= plant.rain_min && c.climate[1] <= plant.rain_max);
            assert!(!plant.outer || c.ids[3] == 3);
        }
    }
    assert!(branching);
    assert!(
        (input - accounted).abs() / input.max(1.) < 0.0001,
        "{input} versus {accounted}"
    );
    assert!(g.generate_region(center, 0., 64).is_err());
    assert!(g.generate_region(center, 300., 63).is_err());
}

#[test]
#[ignore = "requires a hardware GPU"]
fn lake_outlet_only_releases_water_above_its_spill() {
    let mut g = make(16, 42);
    let mut cells = g.snapshot().unwrap();
    for c in &mut cells {
        c.meta[0] = 2;
        c.terrain[0] = 1000.;
        c.water[0] = 0.;
        c.routing = [NONE, 0, NONE, 0];
        c.hydro[0] = 1000.;
    }
    let a = 136usize;
    let b = 137usize;
    cells[a].terrain[0] = 300.;
    cells[a].hydro[0] = 350.;
    cells[a].water[0] = 100.;
    cells[a].routing = [b as u32, 1, a as u32 + 2, 0];
    cells[b].terrain[0] = 250.;
    cells[b].hydro[0] = 250.;
    let before = cells[a].water[0] as f64 * grid::solid_angle(a as u32, 16);
    g.restore_cells(&cells, 0).unwrap();
    g.equilibrate_lakes().unwrap();
    let result = g.snapshot().unwrap();
    assert!(
        (result[a].water[0] - 50.).abs() < 0.003,
        "outlet drained below spill: {}",
        result[a].water[0]
    );
    let after = [a, b]
        .iter()
        .map(|&i| result[i].water[0] as f64 * grid::solid_angle(i as u32, 16))
        .sum::<f64>();
    assert!((before - after).abs() / before < 0.0001);
}

#[test]
#[ignore = "requires a hardware GPU"]
fn default_climate_reaches_a_periodic_seasonal_state() {
    for seed in [17, 81, 256] {
        let mut g = make(64, seed);
        g.run_epochs(1).unwrap();
        assert!(
            g.progress.climate_converged,
            "seed {seed}: {:?}",
            g.progress
        );
        eprintln!(
            "seed {seed}: climate settled in {} cycles",
            g.progress.climate_cycles
        );
    }
    // A deliberately insufficient budget must still report failure to settle.
    let mut g = make(16, 17);
    g.config.climate_cycles = 1;
    g.run_epochs(1).unwrap();
    assert!(!g.progress.climate_converged);
}
