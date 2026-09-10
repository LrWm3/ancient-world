use ancient_world::{
    catalog::Catalog,
    config::Config,
    economy::EconomyCatalog,
    gpu::{ContextGpu, Generator},
};

#[test]
#[ignore = "requires hardware GPU"]
fn illness_limits_work_before_mortality_and_preserves_ledgers() {
    let mut baseline = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 32,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    baseline.found_civilizations(5).unwrap();
    baseline.enable_society().unwrap();
    let mut catalog = EconomyCatalog::bundled().unwrap();
    catalog.agriculture = None;
    catalog.production.adaptive_labor = false;
    catalog.production.diagnostic_fixed_labor = true;
    catalog.production.food_security_labor = false;
    baseline.configure_economy(catalog).unwrap();
    for site in &mut baseline.civilizations.as_mut().unwrap().sites {
        site.demography.health[0] = 0.;
    }
    let path = std::env::temp_dir().join(format!("health-labor-{}.world", std::process::id()));
    let opening = baseline.civilizations.as_ref().unwrap().sites.clone();
    baseline.save(&path).unwrap();
    baseline.advance_history(1).unwrap();
    for (burden, fraction) in [(0., 1.), (0.2, 0.9), (0.4, 0.8), (1., 0.75)] {
        let mut sick = Generator::load(baseline.gpu.clone(), &path).unwrap();
        for site in &mut sick.civilizations.as_mut().unwrap().sites {
            site.demography.health[0] = burden;
        }
        sick.advance_history(1).unwrap();
        let h = sick.civilizations.as_ref().unwrap();
        for (site, well) in h
            .sites
            .iter()
            .zip(&baseline.civilizations.as_ref().unwrap().sites)
        {
            let work: f32 = site.economy.labor.iter().sum();
            let healthy_work: f32 = well.economy.labor.iter().sum();
            assert!(healthy_work > 0.);
            assert!(
                (work / healthy_work - fraction).abs() < 1e-5,
                "burden {burden}: {work}/{healthy_work}"
            );
            let before = &opening[site.id as usize];
            let expected_area = before.stocks.habitat[1]
                .min(before.demography.ages[1] * 0.8 * 0.62 * 1.5 * fraction);
            assert!(
                (site.economy.production_probe[1] - expected_area).abs() < 0.001,
                "site {} burden {burden}: area {} expected {expected_area}",
                site.id,
                site.economy.production_probe[1]
            );
        }
        assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
        assert!(h.population_residual().abs() < 0.01);
        println!(
            "burden {burden}: work fraction {fraction}; crop-area response and ledgers passed"
        );
        if burden == 0.4 {
            let resume_path = path.with_extension("resume.world");
            sick.save(&resume_path).unwrap();
            let mut resumed = Generator::load(sick.gpu.clone(), &resume_path).unwrap();
            std::fs::remove_file(resume_path).unwrap();
            sick.advance_history(3).unwrap();
            for _ in 0..3 {
                resumed.advance_history(1).unwrap();
            }
            assert_eq!(
                serde_json::to_value(&sick.civilizations).unwrap(),
                serde_json::to_value(&resumed.civilizations).unwrap()
            );
        }
    }
    std::fs::remove_file(path).unwrap();
}
