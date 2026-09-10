use ancient_world::{
    catalog::Catalog,
    config::Config,
    economy::EconomyCatalog,
    gpu::{ContextGpu, Generator},
};

fn fixture() -> Generator {
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.advance_ecology().unwrap();
    let mut eco = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
    for cell in &mut eco {
        cell.pools[5..17].fill([0.; 4]);
        cell.pools[13] = [0.01, 0.0012, 0.00015, 0.];
    }
    g.restore_ecology(&eco).unwrap();
    g.found_civilizations(16).unwrap();
    let mut c = EconomyCatalog::bundled().unwrap();
    c.recipes.clear();
    c.agriculture.as_mut().unwrap().fishery.adaptive = true;
    g.configure_economy(c).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.policy[3] = 0.;
    }
    g
}
fn supply(g: &mut Generator, good: usize, kg: f32) {
    let h = g.civilizations.as_mut().unwrap();
    let chemistry = h.economy_catalog.as_ref().unwrap().composition(good);
    for s in &mut h.sites {
        s.economy.goods[good] += kg;
        s.economy.initial[good] += kg;
        for (k, c) in chemistry.iter().enumerate() {
            s.economy.external[k] += kg * c;
        }
    }
}
fn budgets(g: &Generator) {
    let h = g.civilizations.as_ref().unwrap();
    assert!(
        h.economy_residuals().iter().all(|v| v.abs() < 0.001),
        "{:?}",
        h.economy_residuals()
    );
    assert!(h.sites.iter().all(|s| s.economy.valid()));
    assert!(
        g.ecology
            .budget(&g.gpu, &g.config)
            .unwrap()
            .within_tolerance
    );
}
#[test]
fn adaptive_settings_are_bounded_and_old_archives_default_off() {
    let mut c = EconomyCatalog::bundled().unwrap();
    assert!(!c.agriculture.as_ref().unwrap().fishery.adaptive);
    let mut v = serde_json::to_value(&c).unwrap();
    v["agriculture"].as_object_mut().unwrap().remove("fishery");
    let old: EconomyCatalog = serde_json::from_value(v).unwrap();
    assert!(!old.agriculture.unwrap().fishery.adaptive);
    for bad in [f32::NAN, -1., 0., 0.3] {
        c.agriculture.as_mut().unwrap().fishery.max_worker_share = bad;
        assert!(c.validate().is_err());
    }
}
#[test]
#[ignore = "requires hardware GPU"]
fn equipment_demand_labor_and_catches_are_finite_and_resume() {
    let mut g = fixture();
    // High starting provisions must not create an unnecessary fishing fleet.
    g.advance_history(1).unwrap();
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .all(|s| s.economy.fishery_plan[3] == 0.));
    let count = g.civilizations.as_ref().unwrap().sites.len();
    for i in 0..count {
        g.spoil_site_food(i as u32, 1e9).unwrap();
    }
    g.advance_history(1).unwrap();
    assert!(
        g.civilizations
            .as_ref()
            .unwrap()
            .sites
            .iter()
            .all(|s| s.economy.fishery_plan[3] == 0.),
        "no fiber or gear means no catch"
    );
    for good in [0, 3, 16] {
        supply(&mut g, good, 1000.);
    }
    let workers: Vec<_> = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.stocks.stock[0] * 0.5)
        .collect();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().any(|s| s.economy.fishery_plan[3] > 5.));
    for (s, w) in h.sites.iter().zip(workers) {
        let e = &s.economy;
        let fishing = e.fishery_plan[1] + e.fishery_plan[2];
        assert!(fishing <= w * 0.15 + 0.001);
        assert!(
            e.labor.iter().sum::<f32>() + fishing <= w + 0.001,
            "workforce overdraw"
        );
        assert!(
            e.fishery_plan[1]
                <= (e.fishery[0] / 40.)
                    .min(e.fishery[1])
                    .min(e.fishery[2] / 4.)
                    + 0.001
        );
    }
    budgets(&g);
    let path = std::env::temp_dir().join(format!("adaptive-fishery-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    g.advance_history(6).unwrap();
    for _ in 0..6 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&g.ecology.snapshot(&g.gpu, &g.config).unwrap()),
        bytemuck::cast_slice::<_, u8>(&resumed.ecology.snapshot(&g.gpu, &g.config).unwrap())
    );
    budgets(&g);
    resumed.enable_living_history().unwrap();
    resumed
        .scenario(None, ancient_world::ecology::Intervention::RemoveGuild(8))
        .unwrap();
    for i in 0..count {
        resumed.spoil_site_food(i as u32, 1e9).unwrap();
    }
    resumed.advance_history(1).unwrap();
    assert!(
        resumed
            .civilizations
            .as_ref()
            .unwrap()
            .sites
            .iter()
            .all(|s| s.economy.fishery_plan[1..4] == [0.; 3] && s.economy.fishery_stats[3] == 0.),
        "depleted water must release workers despite installed gear"
    );
    budgets(&resumed);
    // Existing raw food shuts demand off without deleting the fleet.
    supply(&mut g, 28, 100000.);
    g.advance_history(1).unwrap();
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .all(|s| s.economy.fishery_plan[3] == 0.));
    budgets(&g);
    let gear: f32 = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.economy.fishery[0])
        .sum();
    assert!(gear > 0.);
    let mut c = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    c.agriculture.as_mut().unwrap().fisheries_enabled = false;
    g.configure_economy(c).unwrap();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h
        .sites
        .iter()
        .all(|s| s.economy.fishery_plan[1..4] == [0.; 3]));
    let remaining: f32 = h.sites.iter().map(|s| s.economy.fishery[0]).sum();
    assert!((remaining - gear * 0.997).abs() < 0.01);
    budgets(&g);
}

#[test]
#[ignore = "requires hardware GPU"]
fn timber_traps_bootstrap_without_fiber_and_preserve_work_and_materials() {
    let mut g = fixture();
    g.advance_history(1).unwrap();
    for i in 0..g.civilizations.as_ref().unwrap().sites.len() {
        g.spoil_site_food(i as u32, 1e9).unwrap();
    }
    supply(&mut g, 0, 1000.);
    g.advance_history(1).unwrap();
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .all(|s| s.economy.fishery_plan[3] == 0.));
    let mut c = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    c.agriculture.as_mut().unwrap().fishery.primitive_gear = true;
    g.configure_economy(c).unwrap();
    let workers: Vec<_> = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.stocks.stock[0] * 0.5)
        .collect();
    g.advance_history(1).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.iter().any(|s| s.economy.fishery_plan[3] > 1.));
    for (s, w) in h.sites.iter().zip(workers) {
        let e = &s.economy;
        assert_eq!(e.fishery[2], 0.);
        assert!(e.fishery_plan[1] <= e.fishery_traps[0] / 20. + 0.001);
        assert!(e.fishery_plan[3] <= e.fishery_plan[1] * 80. * 0.35 * e.fishery_stats[3] + 0.001);
        let work = e.fishery_plan[1] + e.fishery_plan[2];
        assert!(work <= w * 0.15 + 0.001);
        assert!(e.labor.iter().sum::<f32>() + work <= w + 0.001);
        assert!((e.fishery_traps[1] - e.fishery_traps[2] - e.fishery_traps[0]).abs() < 0.001);
    }
    budgets(&g);
    let path = std::env::temp_dir().join(format!("timber-traps-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    g.advance_history(6).unwrap();
    for _ in 0..6 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&g.ecology.snapshot(&g.gpu, &g.config).unwrap()),
        bytemuck::cast_slice::<_, u8>(
            &resumed
                .ecology
                .snapshot(&resumed.gpu, &resumed.config)
                .unwrap()
        )
    );
    let held: Vec<_> = g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .map(|s| s.economy.fishery_traps[0])
        .collect();
    let mut c = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    c.agriculture.as_mut().unwrap().fisheries_enabled = false;
    g.configure_economy(c).unwrap();
    g.advance_history(1).unwrap();
    for (s, before) in g.civilizations.as_ref().unwrap().sites.iter().zip(held) {
        assert_eq!(s.economy.fishery_plan[1..4], [0.; 3]);
        assert!((s.economy.fishery_traps[0] - before * 0.997).abs() < 0.001);
    }
    budgets(&g);
}

#[test]
#[ignore = "requires hardware GPU"]
fn opportunity_cost_releases_workers_and_resumes_deterministically() {
    let mut g = fixture();
    g.advance_history(1).unwrap();
    for i in 0..g.civilizations.as_ref().unwrap().sites.len() {
        g.spoil_site_food(i as u32, 1e9).unwrap();
    }
    for good in [0, 3, 16] {
        supply(&mut g, good, 1000.);
    }
    let mut c = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    c.agriculture.as_mut().unwrap().fishery.opportunity_cost = true;
    g.configure_economy(c).unwrap();
    // Synthetic observed return isolates the decision from ecological differences.
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.fishery_choice[0] = 1000.;
    }
    let path = std::env::temp_dir().join(format!("fish-opportunity-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut closed = Generator::load(g.gpu.clone(), &path).unwrap();
    let mut c = closed
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    c.agriculture.as_mut().unwrap().fisheries_enabled = false;
    closed.configure_economy(c).unwrap();
    closed.advance_history(1).unwrap();
    let mut ablated = Generator::load(g.gpu.clone(), &path).unwrap();
    let mut c = ablated
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    c.agriculture.as_mut().unwrap().fishery.opportunity_cost = false;
    ablated.configure_economy(c).unwrap();
    ablated.advance_history(1).unwrap();
    g.advance_history(1).unwrap();
    assert!(ablated
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .any(|s| s.economy.fishery_plan[3] > 1.));
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .all(|s| s.economy.fishery_plan[0..4] == [0.; 4]));
    for (a, b) in g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .zip(&closed.civilizations.as_ref().unwrap().sites)
    {
        assert_eq!(
            a.economy.labor, b.economy.labor,
            "idle adaptive fishing changes worker shares"
        );
        let ordinary = a.economy.labor.iter().sum::<f32>().max(1.);
        let expected = 1000. * (11. / 12.) + a.economy.diagnostics[1] / ordinary / 12.;
        assert!(
            (a.economy.fishery_choice[0] - expected).abs() < 0.001,
            "one-step exponential memory differs from analytical recurrence"
        );
    }
    budgets(&g);
    budgets(&ablated);
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.fishery_choice[0] = 0.;
    }
    g.advance_history(1).unwrap();
    assert!(g
        .civilizations
        .as_ref()
        .unwrap()
        .sites
        .iter()
        .any(|s| s.economy.fishery_plan[3] > 1.));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    g.advance_history(3).unwrap();
    for _ in 0..3 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_value(&g.civilizations).unwrap(),
        serde_json::to_value(&resumed.civilizations).unwrap()
    );
    assert_eq!(
        bytemuck::cast_slice::<_, u8>(&g.ecology.snapshot(&g.gpu, &g.config).unwrap()),
        bytemuck::cast_slice::<_, u8>(
            &resumed
                .ecology
                .snapshot(&resumed.gpu, &resumed.config)
                .unwrap()
        )
    );
    budgets(&g);
}
