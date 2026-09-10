use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
fn world() -> Generator {
    Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            // These non-seasonal fixtures require prosperous daughter towns to
            // exercise relief and migration, independently of balance defaults.
            island_phosphorus_scale: 1.,
            settlement_plot_hectares: 5000.,
            crop_yield_scale: 1.,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap()
}
#[test]
#[ignore = "requires a hardware GPU"]
fn central_history_conserves_and_continues_after_checkpoint() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    g.set_diversified_farming(false).unwrap();
    g.advance_history(1200).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites.len() > 5);
    assert!(h.events.iter().any(|e| e.kind == "succession"));
    assert!(h.events.iter().any(|e| e.kind == "migration"));
    assert!(h.food_residual().abs() < 0.001);
    assert!(h.population_residual().abs() < 0.001);
    let cells = g.snapshot().unwrap();
    assert!(h.sites.iter().all(|s| cells[s.cell as usize].meta[0] == 2));
    let file = std::env::temp_dir().join(format!("civilization-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    g.advance_history(24).unwrap();
    for _ in 0..24 {
        resumed.advance_history(1).unwrap();
    }
    assert_eq!(
        serde_json::to_vec(&g.civilizations).unwrap(),
        serde_json::to_vec(&resumed.civilizations).unwrap()
    );
    assert!(g.advance().is_err());
    assert!(g.advance_ecology().is_err());
}
#[test]
#[ignore = "requires a hardware GPU"]
fn famine_and_food_shipments_use_real_stocks() {
    let mut g = world();
    g.found_civilizations(5).unwrap();
    g.set_diversified_farming(false).unwrap();
    // Isolate relief from the regional drought experiment and commercial rule changes.
    let mut catalog = g
        .civilizations
        .as_ref()
        .unwrap()
        .economy_catalog
        .clone()
        .unwrap();
    catalog.weather = Default::default();
    catalog.market = Default::default();
    g.configure_economy(catalog).unwrap();
    g.advance_history(900).unwrap();
    let h = g.civilizations.as_mut().unwrap();
    let receiver = h
        .sites
        .iter()
        .enumerate()
        .find(|(i, s)| {
            h.sites
                .iter()
                .enumerate()
                .any(|(j, b)| i != &j && s.island == b.island)
        })
        .unwrap()
        .0;
    let lost = h.sites[receiver].stocks.stock[1];

    h.sites[receiver].stocks.habitat[0] = 0.;
    let before = h.sites[receiver].stocks.stock[0];
    g.spoil_site_food(receiver as u32, lost).unwrap();
    g.advance_history(36).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.sites[receiver].stocks.stock[0] < before);
    assert!(h.events.iter().any(|e| e.kind == "relief_sent"));
    assert!(h.events.iter().any(|e| e.kind == "arrival"));
    assert!(h.food_residual().abs() < 0.001);
    let mut invalid = h.clone();
    invalid.sites[0].cell = g
        .snapshot()
        .unwrap()
        .iter()
        .position(|c| c.meta[0] == 3)
        .unwrap() as u32;
    assert!(invalid.validate(&g.snapshot().unwrap()).is_err());
}
