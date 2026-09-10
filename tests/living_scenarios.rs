use ancient_world::{
    catalog::Catalog,
    config::Config,
    ecology::Intervention,
    gpu::{ContextGpu, Generator, Stage},
};

fn fixture(gpu: ContextGpu, seed: u32) -> Generator {
    let mut g = Generator::new(
        gpu,
        Config {
            seed,
            resolution: 64,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.advance_ecology().unwrap();
    // Declared diagnostic inventory, shared by every branch; not natural calibration.
    let mut cells = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    for c in &mut cells {
        c.pools[5..17].fill([0.; 4]);
        c.pools[15] = [0.1, 0.012, 0.0015, 1.];
    }
    g.restore_ecology(&cells).unwrap();
    g.found_civilizations(16).unwrap();
    g
}

#[test]
#[ignore = "requires a hardware GPU"]
fn interventions_cross_the_living_boundary_and_change_finite_fish_supply() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let mut rows = vec![];
    for seed in [17, 81, 256] {
        let mut initial = fixture(gpu.clone(), seed);
        assert!(initial
            .scenario(None, Intervention::RemoveGuild(10))
            .is_err());
        initial.advance_history(2).unwrap(); // Explicit nonzero history/ecology offset.
        initial.enable_living_history().unwrap();
        let path = std::env::temp_dir().join(format!(
            "living-scenario-{}-{seed}.world",
            std::process::id()
        ));
        initial.save(&path).unwrap();
        let mut catches = vec![];
        let mut food = vec![];
        for (label, remove, closed) in [
            ("baseline", false, false),
            ("removed", true, false),
            ("negative_control", false, false),
            ("fishery_closed", false, true),
            ("removed_closed", true, true),
        ] {
            let mut g = Generator::load(gpu.clone(), &path).unwrap();
            if closed {
                let mut catalog = g
                    .civilizations
                    .as_ref()
                    .unwrap()
                    .economy_catalog
                    .clone()
                    .unwrap();
                catalog.agriculture.as_mut().unwrap().fisheries_enabled = false;
                g.configure_economy(catalog).unwrap();
            }
            let before = g.ecology.snapshot(&gpu, &g.config).unwrap();
            let ecology_month = g.ecology.clock.month;
            let history_month = g.civilizations.as_ref().unwrap().month;
            if remove {
                g.scenario(None, Intervention::RemoveGuild(10)).unwrap();
            }
            if label == "negative_control" {
                g.scenario(None, Intervention::RemoveGuild(0)).unwrap();
            }
            assert_eq!(g.ecology.clock.month, ecology_month);
            assert_eq!(g.civilizations.as_ref().unwrap().month, history_month);
            if remove {
                let after = g.ecology.snapshot(&gpu, &g.config).unwrap();
                for (a, b) in before.iter().zip(&after) {
                    assert_eq!(b.pools[15], [0.; 4]);
                    for k in 0..3 {
                        let returned =
                            b.pools[18][k] + b.pools[22][k] - a.pools[18][k] - a.pools[22][k];
                        assert!((returned - a.pools[15][k]).abs() < 1e-5);
                    }
                }
                let event = g.ecology.clock.events.last().unwrap();
                assert_eq!(event.history_month, Some(history_month));
                let record = &g.civilizations.as_ref().unwrap().events
                    [event.history_event.unwrap() as usize];
                assert_eq!(record.kind, "ecological_intervention");
                assert_eq!(record.month, history_month);
            }
            let prior_catch = g
                .civilizations
                .as_ref()
                .unwrap()
                .sites
                .iter()
                .map(|s| f64::from(s.economy.agriculture[2]))
                .sum::<f64>();
            g.advance_history(1).unwrap();
            let h = g.civilizations.as_ref().unwrap();
            let catch = h
                .sites
                .iter()
                .map(|s| f64::from(s.economy.agriculture[2]))
                .sum::<f64>()
                - prior_catch;
            let reserve = h
                .sites
                .iter()
                .map(|s| f64::from(s.stocks.stock[1]))
                .sum::<f64>();
            let budget = g.ecology.budget(&gpu, &g.config).unwrap();
            assert!(budget.within_tolerance);
            assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
            catches.push(catch);
            food.push(reserve);
            rows.push(serde_json::json!({"seed":seed,"branch":label,"history_month":h.month,"ecology_month":g.ecology.clock.month,"catch_kg":catch,"food_reserve_equivalent_kg":reserve,"population":h.sites.iter().map(|s|s.stocks.stock[0] as f64).sum::<f64>(),"economy_residuals":h.economy_residuals(),"ecology_budget":budget}));
        }
        assert!(catches[0] > 0.);
        assert_eq!(catches[1], 0.);
        assert!((catches[2] - catches[0]).abs() < 1e-5);
        assert_eq!(catches[3], 0.);
        assert_eq!(catches[4], 0.);
        assert_eq!(
            food[0], food[2],
            "absent terrestrial guild is a negative control"
        );
        assert_eq!(
            food[3], food[4],
            "harvest closure blocks the immediate food effect"
        );
        assert!(
            food[0] > food[1],
            "catch must reach food supplies: {food:?}"
        );
        std::fs::remove_file(path).unwrap();
    }
    std::fs::create_dir_all("output/living-scenarios").unwrap();
    std::fs::write(
        "output/living-scenarios/fishery.json",
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "requires a hardware GPU"]
fn scenario_validation_and_checkpoint_continuation_are_atomic() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let mut g = fixture(gpu.clone(), 17);
    g.enable_living_history().unwrap();
    let initial = g.ecology.snapshot(&gpu, &g.config).unwrap();
    let count = g.ecology.clock.events.len();
    let history_count = g.civilizations.as_ref().unwrap().events.len();
    for (region, event) in [
        (Some(4), Intervention::RemoveGuild(0)),
        (None, Intervention::RemoveGuild(12)),
        (None, Intervention::LakeMixing(f32::NAN)),
    ] {
        assert!(g.scenario(region, event).is_err());
    }
    g.progress.stage = Stage::Ecology;
    assert!(g.scenario(None, Intervention::RemoveGuild(10)).is_err());
    g.progress.stage = Stage::Boundary;
    g.civilizations
        .as_mut()
        .unwrap()
        .living
        .as_mut()
        .unwrap()
        .incomplete = true;
    assert!(g.scenario(None, Intervention::RemoveGuild(10)).is_err());
    g.civilizations
        .as_mut()
        .unwrap()
        .living
        .as_mut()
        .unwrap()
        .incomplete = false;
    assert_eq!(count, g.ecology.clock.events.len());
    assert_eq!(
        history_count,
        g.civilizations.as_ref().unwrap().events.len()
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&initial),
        bytemuck::cast_slice::<_, u8>(&g.ecology.snapshot(&gpu, &g.config).unwrap())
    );
    // A pure-water coarse cell must receive aquatic, not terrestrial, detritus.
    let terrain = g.snapshot().unwrap();
    let n = g.config.resolution;
    let m = g.config.eco_resolution();
    let r = n / m;
    let water_cell = (0..6 * m * m)
        .find(|&i| {
            (0..r).all(|y| {
                (0..r).all(|x| {
                    let fine = i / (m * m) * n * n + ((i / m % m) * r + y) * n + (i % m) * r + x;
                    terrain[fine as usize].meta[0] < 2
                })
            })
        })
        .unwrap() as usize;
    g.scenario(None, Intervention::RemoveGuild(10)).unwrap();
    let removed = g.ecology.snapshot(&gpu, &g.config).unwrap();
    assert_eq!(removed[water_cell].pools[18], initial[water_cell].pools[18]);
    for k in 0..3 {
        assert!(
            (removed[water_cell].pools[22][k]
                - initial[water_cell].pools[22][k]
                - initial[water_cell].pools[15][k])
                .abs()
                < 1e-5
        );
    }
    let path = std::env::temp_dir().join(format!("scenario-resume-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(gpu.clone(), &path).unwrap();
    g.advance_history(3).unwrap();
    for _ in 0..3 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&g.ecology.clock).unwrap(),
        serde_json::to_value(&resumed.ecology.clock).unwrap()
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&g.ecology.snapshot(&gpu, &g.config).unwrap()),
        bytemuck::cast_slice::<_, u8>(&resumed.ecology.snapshot(&gpu, &g.config).unwrap())
    );
    g.scenario(None, Intervention::RestoreGuild(10)).unwrap();
    g.advance_history(1).unwrap();
    assert!(g
        .ecology
        .snapshot(&gpu, &g.config)
        .unwrap()
        .iter()
        .all(|c| c.pools[15][0] == 0.));
    resumed
        .ecology
        .clock
        .events
        .last_mut()
        .unwrap()
        .history_event = Some(u64::MAX);
    assert!(resumed.save(&path).is_err());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn old_scenarios_and_agriculture_retain_compatible_defaults() {
    let event: ancient_world::ecology::ScenarioEvent = serde_json::from_value(
        serde_json::json!({"month":4,"region":null,"intervention":{"RemoveGuild":10}}),
    )
    .unwrap();
    assert_eq!(event.history_month, None);
    assert_eq!(event.history_event, None);
    let catalog = ancient_world::agriculture::AgricultureCatalog::bundled();
    assert!(catalog.fisheries_enabled);
    let mut value = serde_json::to_value(&catalog).unwrap();
    value.as_object_mut().unwrap().remove("fisheries_enabled");
    let old: ancient_world::agriculture::AgricultureCatalog =
        serde_json::from_value(value).unwrap();
    assert!(old.fisheries_enabled);
}
