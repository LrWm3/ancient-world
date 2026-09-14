//! Finite geological phosphorus transfer and archive-compatible calibration controls.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    economy::EconomyCatalog,
    gpu::{ContextGpu, Generator},
    production::{
        DEFAULT_PHOSPHORUS_RELEASE_MONTHLY_FRACTION, MAX_PHOSPHORUS_RELEASE_MONTHLY_FRACTION,
    },
};

#[test]
fn nutrient_release_validation_and_legacy_default() {
    let catalog = EconomyCatalog::bundled().unwrap();
    let mut old = serde_json::to_value(&catalog).unwrap();
    old["production"]
        .as_object_mut()
        .unwrap()
        .remove("phosphorus_release_monthly_fraction");
    let restored: EconomyCatalog = serde_json::from_value(old).unwrap();
    assert_eq!(
        restored.production.phosphorus_release_monthly_fraction,
        DEFAULT_PHOSPHORUS_RELEASE_MONTHLY_FRACTION
    );
    for fraction in [-1., f32::NAN, f32::INFINITY, 1.] {
        let mut c = catalog.clone();
        c.production.phosphorus_release_monthly_fraction = fraction;
        assert!(c.validate().is_err());
    }
    for fraction in [
        0.,
        DEFAULT_PHOSPHORUS_RELEASE_MONTHLY_FRACTION,
        MAX_PHOSPHORUS_RELEASE_MONTHLY_FRACTION,
    ] {
        let mut c = catalog.clone();
        c.production.phosphorus_release_monthly_fraction = fraction;
        c.validate().unwrap();
    }
}

#[test]
#[ignore = "requires GPU; analytical finite-source transfer and checkpoint continuation"]
fn geological_release_consumes_source_and_resumes() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    for rate in [0., MAX_PHOSPHORUS_RELEASE_MONTHLY_FRACTION] {
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                resolution: 32,
                ecology_resolution: 32,
                seed: 1024,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        g.found_civilizations(5).unwrap();
        g.advance_history(1).unwrap(); // Complete initial ecological claims.
        g.civilizations
            .as_mut()
            .unwrap()
            .economy_catalog
            .as_mut()
            .unwrap()
            .production
            .phosphorus_release_monthly_fraction = rate;
        let before: Vec<_> = g
            .civilizations
            .as_ref()
            .unwrap()
            .sites
            .iter()
            .map(|s| s.economy.reserves[0])
            .collect();
        assert!(before.iter().all(|p| *p > 1000.));
        g.advance_history(1).unwrap();
        let h = g.civilizations.as_ref().unwrap();
        for (s, old) in h.sites.iter().zip(before) {
            let expected = old - old * rate;
            assert!(
                (s.economy.reserves[0] - expected).abs() <= old * 2e-7,
                "source transfer: {} versus {expected}",
                s.economy.reserves[0]
            );
        }
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
        let path =
            std::env::temp_dir().join(format!("farm-nutrients-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut resumed = Generator::load(gpu.clone(), &path).unwrap();
        std::fs::remove_file(path).unwrap();
        g.advance_history(3).unwrap();
        for _ in 0..3 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
    }
}

#[test]
#[ignore = "requires GPU; 200-year positive-growth scenario, not a default-balance guarantee"]
fn nutrient_and_food_access_scenario_supports_continued_expansion() {
    use ancient_world::systems::System;
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let mut g = Generator::new(
        gpu,
        Config {
            resolution: 32,
            ecology_resolution: 32,
            seed: 1024,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.run_epochs(1).unwrap();
    g.found_civilizations(5).unwrap();
    let mut systems = g.config.systems.clone();
    systems.select(System::NeedsBasedFood, true);
    systems.select(System::DemographicAudit, true);
    g.apply_systems(&systems).unwrap();
    g.civilizations
        .as_mut()
        .unwrap()
        .economy_catalog
        .as_mut()
        .unwrap()
        .production
        .phosphorus_release_monthly_fraction = 5e-7;
    let ids: Vec<_> = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| (s.id, s.economy.policy))
        .collect();
    for (id, mut policy) in ids {
        policy[1] = 0.95;
        g.set_site_policy(id, policy).unwrap();
    }
    assert_eq!(g.civilizations.as_ref().unwrap().initial_population, 600.);
    g.advance_history(2400).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let population: f32 = h.sites.iter().map(|s| s.stocks.stock[0]).sum();
    let active = h
        .sites
        .iter()
        .filter(|s| !s.abandoned && s.stocks.stock[0] > 0.)
        .count();
    let audit = h.demographic_audit.as_ref().unwrap();
    assert_eq!(audit.years.len(), 200);
    let late_births: f64 = audit
        .years
        .iter()
        .filter(|r| r.year > 190)
        .map(|r| r.observed_births)
        .sum();
    let late_deaths: f64 = audit
        .years
        .iter()
        .filter(|r| r.year > 190)
        .map(|r| r.observed_deaths)
        .sum();
    assert!(
        population > 1500.,
        "only {population} people after 200 years"
    );
    assert!(active >= 8, "only {active} active settlements");
    assert!(
        late_births > late_deaths,
        "late growth ceased: {late_births} births, {late_deaths} deaths"
    );
    assert!(h.sites.iter().any(|s| s.founded > 1200 && !s.abandoned));
    assert!(h
        .culture
        .as_ref()
        .unwrap()
        .institutions
        .iter()
        .any(|i| i.active));
    assert!(h.population_residual().abs() < 0.001);
    assert!(h.food_residual().abs() < 0.001);
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    println!("growth scenario: {population:.2} people, {active} active towns; final-decade births/deaths {late_births:.2}/{late_deaths:.2}");
}
