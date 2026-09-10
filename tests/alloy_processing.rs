use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
    grid,
};
fn world(seed: u32) -> Generator {
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            seed,
            resolution: 32,
            ecology_resolution: 16,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.found_civilizations(5).unwrap();
    let mut terrain = g.snapshot().unwrap();
    for (s, id) in g.civilizations.as_ref().unwrap().sites.iter().zip([
        "chalcopyrite",
        "malachite",
        "cassiterite",
        "hematite",
        "bituminous_coal",
    ]) {
        terrain[s.cell as usize].meta[1] =
            g.catalog.minerals.iter().position(|m| m.id == id).unwrap() as u32;
    }
    g.restore_cells(&terrain, 0).unwrap();
    for s in &mut g.civilizations.as_mut().unwrap().sites {
        s.economy.reserves[1] = 100.;
        s.economy.goods[6] += 100.;
        s.economy.initial[6] += 100.;
        s.economy.external[0] += 100.;
    }
    g.enable_shared_resources().unwrap();
    g.enable_mineral_processing().unwrap();
    g.enable_alloy_processing().unwrap();
    g
}
#[test]
#[ignore = "requires hardware GPU"]
fn residues_limit_processing_persist_in_ruins_and_survive_checkpoints() {
    for seed in [17, 81, 256] {
        let mut g = world(seed);
        let mut c = g
            .civilizations
            .as_ref()
            .unwrap()
            .economy_catalog
            .clone()
            .unwrap();
        c.production.enabled = false;
        c.recipes
            .retain(|r| r.input[32..38].iter().sum::<f32>() > 0.);
        g.configure_economy(c).unwrap();
        for s in &mut g.civilizations.as_mut().unwrap().sites {
            s.economy.residue[2] = 1.;
        }
        let path =
            std::env::temp_dir().join(format!("alloy-residue-{seed}-{}.world", std::process::id()));
        g.save(&path).unwrap();
        let mut open = Generator::load(g.gpu.clone(), &path).unwrap();
        for s in &mut open.civilizations.as_mut().unwrap().sites {
            s.economy.residue[2] = 100.;
        }
        g.advance_history(12).unwrap();
        open.advance_history(12).unwrap();
        let h = g.civilizations.as_ref().unwrap();
        for (i, s) in h.sites.iter().enumerate() {
            let deposit = s.economy.residue[0];
            if i < 4 {
                assert!((deposit - 1.).abs() < 1e-4);
                assert!(s.economy.residue[3] > 0.);
                assert!(
                    open.civilizations.as_ref().unwrap().sites[i]
                        .economy
                        .residue[0]
                        > deposit
                );
                let raw = [35, 36, 37, 32][i];
                let metal = [38, 38, 39, 2][i];
                assert!((s.economy.used[raw] - s.economy.made[metal] - deposit).abs() < 0.001);
            } else {
                assert_eq!(deposit, 0.);
            }
        }
        assert_eq!(h.processing_deposits().len(), 4);
        assert!(h
            .processing_deposits()
            .iter()
            .all(|d| d.founding_event.is_some()));
        let cell = h.sites[0].cell;
        let region = g
            .generate_region(grid::cell_direction(cell, 32), 10., 16)
            .unwrap();
        assert_eq!(
            region
                .processing_deposits
                .iter()
                .find(|d| d.site == 0)
                .unwrap()
                .kg,
            h.sites[0].economy.residue[0]
        );
        g.save(&path).unwrap();
        let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
        g.advance_history(3).unwrap();
        for _ in 0..3 {
            resumed.advance_history(1).unwrap();
        }
        assert_eq!(
            serde_json::to_value(&g.civilizations).unwrap(),
            serde_json::to_value(&resumed.civilizations).unwrap()
        );
        let s = &mut g.civilizations.as_mut().unwrap().sites[0];
        let deposit = s.economy.residue[0];
        let pop = s.stocks.stock[0];
        s.stocks.people[1] += pop;
        s.demography.health[2] += pop;
        s.stocks.stock[0] = 0.;
        s.demography.ages[..3].fill(0.);
        s.abandoned = true;
        g.advance_history(1).unwrap();
        assert_eq!(
            g.civilizations.as_ref().unwrap().sites[0].economy.residue[0],
            deposit
        );
        let residual = g.civilizations.as_ref().unwrap().economy_residuals();
        assert!(residual.iter().all(|v| v.abs() < 0.001), "{residual:?}");
        std::fs::remove_file(path).unwrap();
    }
}
#[test]
#[ignore = "requires hardware GPU"]
fn bronze_requires_both_metals_and_wears_into_its_own_scrap() {
    let mut g = world(17);
    let mut c = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    c.production.enabled = false;
    c.recipes.retain(|r| {
        r.output[40] > 0. || r.output[41] > 0. || r.output[43] > 0. || r.input[44] > 0.
    });
    g.configure_economy(c.clone()).unwrap();
    for (i, s) in g
        .civilizations
        .as_mut()
        .unwrap()
        .sites
        .iter_mut()
        .enumerate()
    {
        s.economy.goods[38] += 9.;
        s.economy.initial[38] += 9.;
        if i != 0 {
            s.economy.goods[39] += 1.;
            s.economy.initial[39] += 1.;
        }
    }
    g.advance_history(12).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert_eq!(h.sites[0].economy.made[40], 0.);
    assert_eq!(h.sites[0].economy.made[41], 0.);
    assert!(h.sites[0].economy.made[43] > 0.);
    assert!(h.sites[0].economy.made[44] > 0.);
    assert!((h.sites[0].economy.made[44] - h.sites[0].economy.used[43] * 0.9).abs() < 0.001);
    for s in &h.sites[1..] {
        assert!(s.economy.made[41] > 0.);
        assert!(s.economy.made[42] > 0.);
        assert!((s.economy.made[42] - s.economy.used[41] * 0.9).abs() < 0.001);
    }
    let residual = h.economy_residuals();
    assert!(residual.iter().all(|v| v.abs() < 0.001), "{residual:?}");
    let mut invalid = c.clone();
    let mut bad = invalid.recipes[0];
    bad.input.fill(0.);
    bad.output.fill(0.);
    bad.input[38] = 1.;
    bad.output[40] = 1.;
    invalid.recipes.push(bad);
    assert!(invalid.validate().is_err());
    let mut invalid = c.clone();
    let mut bad = invalid.recipes[0];
    bad.input.fill(0.);
    bad.output.fill(0.);
    bad.input[42] = 1.;
    bad.output[2] = 1.;
    invalid.recipes.push(bad);
    assert!(invalid.validate().is_err());
}
