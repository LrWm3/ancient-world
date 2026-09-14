use ancient_world::{
    catalog::Catalog,
    config::Config,
    economy::EconomyCatalog,
    gpu::{ContextGpu, Generator},
    systems::Systems,
};

#[test]
fn mobility_is_bounded_and_old_settings_preserve_flushing() -> anyhow::Result<()> {
    let old: ancient_world::production::ProductionSettings = serde_json::from_str("{}")?;
    assert_eq!(old.phosphorus_runoff_mobility, 1.);
    let mut catalog = EconomyCatalog::bundled()?;
    for value in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
        catalog.production.phosphorus_runoff_mobility = value;
        assert!(catalog.validate().is_err());
    }
    for value in [0., 0.1, 1.] {
        catalog.production.phosphorus_runoff_mobility = value;
        catalog.validate()?;
    }
    Ok(())
}

#[test]
#[ignore = "requires GPU"]
fn retained_phosphorus_stays_in_soil_and_continues_across_checkpoint() -> anyhow::Result<()> {
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut baseline = Generator::new(
        gpu.clone(),
        Config {
            resolution: 32,
            ecology_resolution: 32,
            seed: 17,
            ..Default::default()
        },
        Catalog::bundled()?,
    )?;
    baseline.run_epochs(1)?;
    baseline.found_civilizations(3)?;
    baseline.apply_systems(&Systems::default())?;
    let dir = std::path::PathBuf::from(format!("output/retention-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("world.world");
    baseline.save(&path)?;
    let mut retained = Generator::load(gpu.clone(), &path)?;
    retained
        .civilizations
        .as_mut()
        .unwrap()
        .economy_catalog
        .as_mut()
        .unwrap()
        .production
        .phosphorus_runoff_mobility = 0.1;
    baseline.advance_history(1)?;
    retained.advance_history(1)?;
    let mut exported = 0.;
    for (a, b) in baseline
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .zip(&retained.civilizations.as_ref().unwrap().sites)
    {
        let a = &a.economy;
        let b = &b.economy;
        exported += a.phosphorus_probe[0];
        assert!((b.phosphorus_probe[0] - a.phosphorus_probe[0] * 0.1).abs() < 0.01);
        // Same opening inputs: all withheld runoff P is present at the crop supply boundary.
        let kept = a.phosphorus_probe[0] - b.phosphorus_probe[0];
        assert!((b.crop_probe[1][1] - a.crop_probe[1][1] - kept).abs() < 0.1);
        assert_eq!(a.crop_probe[1][0], b.crop_probe[1][0]); // N is unaffected
        assert_eq!(a.crop_probe[1][2], b.crop_probe[1][2]); // water is unaffected
        assert_eq!(a.phosphorus_probe[1], b.phosphorus_probe[1]); // no additional release
    }
    assert!(exported > 0.); // non-vacuous runoff fixture
    assert!(retained
        .civilizations
        .as_ref()
        .unwrap()
        .economy_residuals()
        .iter()
        .all(|x| x.abs() < 1e-4));
    retained.save(&path)?;
    let mut resumed = Generator::load(gpu, &path)?;
    retained.advance_history(2)?;
    resumed.advance_history(1)?;
    resumed.advance_history(1)?;
    assert_eq!(
        serde_json::to_value(&retained.civilizations)?,
        serde_json::to_value(&resumed.civilizations)?
    );
    std::fs::remove_dir_all(dir)?;
    Ok(())
}
