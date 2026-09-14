//! Hardware boundary fixture: observations cannot alter history or miss roster months.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
    resolution::Mode,
    systems::Systems,
};

#[test]
#[ignore = "requires GPU"]
fn roster_food_probes_are_observational_and_match_resource_bounds() -> anyhow::Result<()> {
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut g = Generator::new(
        gpu.clone(),
        Config {
            resolution: 32,
            ecology_resolution: 32,
            seed: 17,
            ..Default::default()
        },
        Catalog::bundled()?,
    )?;
    g.run_epochs(1)?;
    g.found_civilizations(3)?;
    g.apply_systems(&Systems::default())?;
    g.civilizations
        .as_mut()
        .unwrap()
        .set_demographic_resolution(Mode::Individual, false)?;
    let dir = std::path::PathBuf::from(format!(
        "output/food-diagnostic-test-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("base.world");
    g.save(&path)?;
    let mut observed = Generator::load(gpu.clone(), &path)?;
    observed.civilizations.as_mut().unwrap().demographic_audit = Some(Default::default());
    g.advance_history(2)?;
    observed.advance_history(1)?;
    observed.save(&path)?;
    let mut observed = Generator::load(gpu, &path)?;
    observed.advance_history(1)?;
    let audit = observed
        .civilizations
        .as_mut()
        .unwrap()
        .demographic_audit
        .take()
        .unwrap();
    assert!(!audit.food.months.is_empty());
    assert!(audit.food.months.iter().any(|r| r.month == 1));
    assert!(audit.food.months.iter().any(|r| r.month == 2));
    for r in &audit.food.months {
        assert!(r.opening.is_some() && r.closing.is_some());
        let [demand, supply, output] = r.crop_probe;
        let fraction = (0..3)
            .filter(|&i| demand[i] > 0.)
            .map(|i| (supply[i] / demand[i].max(0.000001)).min(1.))
            .fold(1., f32::min);
        assert!((supply[3] - fraction).abs() < 1e-5);
        assert!((output[3] - demand[3] * fraction).abs() < 0.02);
        let gaps = r.shortfalls_kg();
        assert!((gaps[0] + gaps[1] - (r.need_kg - r.eaten_kg).max(0.)).abs() < 1e-8);
    }
    g.civilizations.as_mut().unwrap().demographic_audit = None;
    assert_eq!(
        serde_json::to_value(&g.civilizations)?,
        serde_json::to_value(&observed.civilizations)?
    );
    std::fs::remove_dir_all(dir)?;
    Ok(())
}
