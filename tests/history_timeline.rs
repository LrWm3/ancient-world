use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};

#[test]
#[ignore = "requires hardware GPU"]
fn recorded_history_matches_monthly_and_checkpoint_continuation() {
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless()).unwrap(),
        Config {
            resolution: 64,
            ecology_resolution: 16,
            ecology_years_per_epoch: 1,
            ..Default::default()
        },
        Catalog::bundled().unwrap(),
    )
    .unwrap();
    g.found_civilizations(5).unwrap();
    assert!(g.civilizations.as_ref().unwrap().sites.iter().all(|s| s
        .lifecycle
        .timeline
        .samples
        .is_empty()));
    g.advance_history(6).unwrap();
    let path = std::env::temp_dir().join(format!("timeline-{}.world", std::process::id()));
    g.save(&path).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &path).unwrap();
    std::fs::remove_file(path).unwrap();
    g.advance_history(18).unwrap();
    for _ in 0..18 {
        resumed.advance_history(1).unwrap();
    }
    let a = g.civilizations.as_ref().unwrap();
    let b = resumed.civilizations.as_ref().unwrap();
    assert_eq!(
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap()
    );
    let ctx = eframe::egui::Context::default();
    let mut view = ancient_world::history_timeline::TimelineView::default();
    let mut selected = Some(0);
    let output = ctx.run(Default::default(), |ctx| {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            assert!(view.show(ui, a, &mut selected).is_none());
        });
    });
    assert!(!output.shapes.is_empty());
    for site in a.sites.iter().filter(|s| s.founded == 0) {
        let samples = &site.lifecycle.timeline.samples;
        assert_eq!(samples.len(), 24);
        assert_eq!(samples.front().unwrap().month, 1);
        assert_eq!(samples.back().unwrap().month, 24);
        assert_eq!(samples.back().unwrap().population, site.stocks.stock[0]);
    }
}
