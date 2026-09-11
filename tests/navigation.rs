use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
    grid,
    navigation::{NavigationMode, RouteKind},
};
#[test]
#[ignore = "requires hardware GPU"]
fn gpu_routes_are_valid_and_compare_with_cpu() {
    let resolution = std::env::var("NAVIGATION_RESOLUTION")
        .map(|s| s.parse().unwrap())
        .unwrap_or(64);
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    for seed in [17, 81, 256] {
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                seed,
                resolution,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.advance_ecology().unwrap();
        let checkpoint = format!(
            "output/navigation-initial-{}-{seed}.world",
            std::process::id()
        );
        std::fs::create_dir_all("output").unwrap();
        g.save(std::path::Path::new(&checkpoint)).unwrap();
        g.set_navigation_mode(NavigationMode::CpuReference);
        g.found_civilizations(16).unwrap();
        let founding = serde_json::to_value(&g.civilizations).unwrap();
        g = Generator::load(gpu.clone(), std::path::Path::new(&checkpoint)).unwrap();
        std::fs::remove_file(checkpoint).unwrap();
        g.set_navigation_mode(NavigationMode::Gpu);
        g.found_civilizations(16).unwrap();
        let surveyed = serde_json::to_value(&g.civilizations).unwrap();
        for key in ["candidates", "sites", "culture"] {
            assert!(
                founding[key] == surveyed[key],
                "compact founding survey seed {seed}: {key}"
            );
        }
        g.set_navigation_mode(NavigationMode::CpuReference);
        g.enable_society().unwrap();
        g.enable_politics().unwrap();
        g.enable_governance().unwrap();
        g.enable_shipping().unwrap();
        g.enable_expeditions().unwrap();
        let h = g.civilizations.as_ref().unwrap();
        let mut queries: Vec<_> = h
            .society
            .as_ref()
            .unwrap()
            .routes
            .iter()
            .take(3)
            .map(|r| {
                (
                    g.civilizations.as_ref().unwrap().sites[r.from as usize].cell,
                    Some(g.civilizations.as_ref().unwrap().sites[r.to as usize].cell),
                    RouteKind::Road,
                )
            })
            .collect();
        let sh = h.shipping.as_ref().unwrap();
        for p in sh.ports.iter().take(2) {
            queries.push((h.sites[p.site as usize].cell, None, RouteKind::Harbor));
            queries.push((p.water_cell, None, RouteKind::Expedition));
        }
        for lane in sh.lanes.iter().take(2) {
            queries.push((
                *lane.cells.first().unwrap(),
                lane.cells.last().copied(),
                RouteKind::Sea,
            ));
        }
        let refs: Vec<_> = queries
            .iter()
            .map(|&(s, t, k)| g.survey_route(s, t, k).unwrap())
            .collect();
        let terrain = g.snapshot().unwrap();
        g.set_navigation_mode(NavigationMode::Gpu);
        for ((start, end, kind), reference) in queries.into_iter().zip(refs) {
            let result = g.survey_route(start, end, kind).unwrap();
            assert_eq!(
                result.is_some(),
                reference.is_some(),
                "{kind:?} reachability"
            );
            if let (Some((path, cost)), Some((old, old_cost))) = (result, reference) {
                assert_eq!(path.first(), Some(&start));
                if let Some(end) = end {
                    assert_eq!(path.last(), Some(&end));
                }
                assert_eq!(
                    path.len(),
                    path.iter().collect::<std::collections::BTreeSet<_>>().len()
                );
                assert!(path.windows(2).all(|w| [(-1, 0), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .any(|(x, y)| grid::neighbor(w[0], resolution, x, y) == w[1])));
                for (idx, &id) in path.iter().enumerate() {
                    let c = &terrain[id as usize];
                    let lake = c.meta[0] == 1 && c.water[0] > 0.25;
                    let land = c.meta[0] == 2 && c.water[0] < 0.25;
                    let outer = c.meta[0] == 3 && c.water[0] < 0.25;
                    assert!(match kind {
                        RouteKind::Road => land,
                        RouteKind::Harbor => land || lake,
                        RouteKind::Sea => lake,
                        RouteKind::Expedition => lake || (idx + 1 == path.len() && outer),
                    });
                }
                println!("{kind:?}: CPU {old_cost:.3} km / {} cells; GPU {cost:.3} km / {} cells; same path {}",old.len(),path.len(),old==path);
                assert!((cost - old_cost).abs() / old_cost.max(1.) < 0.02);
            }
        }
        println!("{:?}", g.navigation_stats());
        // Explicitly block the destination and check failure is a proven no-route result.
        let start = g.civilizations.as_ref().unwrap().sites[0].cell;
        let end = grid::neighbor(start, resolution, 1, 0);
        let mut changed = terrain;
        changed[end as usize].water[0] = 10.;
        g.restore_cells(&changed, 0).unwrap();
        assert!(g
            .survey_route(start, Some(end), RouteKind::Road)
            .unwrap()
            .is_none());
        if seed == 17 {
            // One permitted edge across a cube face; every alternative is water.
            let a = resolution - 1;
            let b = grid::neighbor(a, resolution, 1, 0);
            assert_ne!(a / (resolution * resolution), b / (resolution * resolution));
            for cell in &mut changed {
                cell.meta[0] = 0;
                cell.water[0] = 10.;
            }
            for id in [a, b] {
                changed[id as usize].meta[0] = 2;
                changed[id as usize].water = [0.; 4];
                changed[id as usize].terrain[0] = 100.;
            }
            g.restore_cells(&changed, 0).unwrap();
            let (path, cost) = g
                .survey_route(a, Some(b), RouteKind::Road)
                .unwrap()
                .unwrap();
            assert_eq!(path, vec![a, b]);
            assert!(cost.is_finite() && cost > 0.);
            assert!(g.survey_route(u32::MAX, Some(b), RouteKind::Road).is_err());
        }
    }
}

#[test]
#[ignore = "requires hardware GPU; compares outcomes, not identical path shapes"]
fn gpu_navigation_history_comparison() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    std::fs::create_dir_all("output").unwrap();
    for seed in [17, 81, 256] {
        let config = Config {
            seed,
            resolution: 64,
            ecology_resolution: 16,
            ..Default::default()
        };
        let mut reference =
            Generator::new(gpu.clone(), config, Catalog::bundled().unwrap()).unwrap();
        reference.advance_ecology().unwrap();
        let path = format!(
            "output/navigation-comparison-{}-{seed}.world",
            std::process::id()
        );
        reference.save(std::path::Path::new(&path)).unwrap();
        let mut candidate = Generator::load(gpu.clone(), std::path::Path::new(&path)).unwrap();
        std::fs::remove_file(path).unwrap();
        reference.set_navigation_mode(NavigationMode::CpuReference);
        for (label, world) in [("CPU", &mut reference), ("GPU", &mut candidate)] {
            let start = std::time::Instant::now();
            world.found_civilizations(16).unwrap();
            world.enable_society().unwrap();
            world.enable_politics().unwrap();
            world.enable_governance().unwrap();
            world.enable_shipping().unwrap();
            world.enable_expeditions().unwrap();
            world.advance_history(2).unwrap();
            world.enable_living_history().unwrap();
            world.enable_environmental_returns().unwrap();
            let setup = start.elapsed().as_secs_f64();
            let start = std::time::Instant::now();
            world.advance_history(36).unwrap();
            let elapsed = start.elapsed().as_secs_f64();
            let h = world.civilizations.as_ref().unwrap();
            let population: f64 = h.sites.iter().map(|s| s.stocks.stock[0] as f64).sum();
            let roads = h.society.as_ref().unwrap().routes.len();
            let shipping = h.shipping.as_ref().unwrap();
            assert!(population.is_finite() && population >= 0.);
            assert!(
                world
                    .ecology
                    .budget(&gpu, &world.config)
                    .unwrap()
                    .within_tolerance
            );
            println!("seed {seed} {label}: setup {setup:.3}s; 36 months {elapsed:.3}s; population {population:.3}; sites {}; roads {roads}; ports {}; lanes {}; expeditions {}; events {}; {:?}", h.sites.len(), shipping.ports.len(), shipping.lanes.len(), h.expeditions.as_ref().unwrap().routes.len(), h.events.len(), world.navigation_stats());
        }
        let a = reference.civilizations.as_ref().unwrap();
        let b = candidate.civilizations.as_ref().unwrap();
        let event_differences = a
            .events
            .iter()
            .zip(&b.events)
            .filter(|(a, b)| serde_json::to_value(a).unwrap() != serde_json::to_value(b).unwrap())
            .count()
            + a.events.len().abs_diff(b.events.len());
        println!("seed {seed}: differing event records {event_differences}");
    }
}
