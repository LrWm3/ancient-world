use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
fn world() -> Generator {
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
    g.found_civilizations(16).unwrap();
    g.enable_society().unwrap();
    g.enable_politics().unwrap();
    g.enable_governance().unwrap();
    g.set_negotiated_autonomy(false).unwrap(); // Existing manual policy comparisons.
    g
}
#[test]
#[ignore = "requires hardware GPU"]
fn treaties_expire_and_resume_without_changing_inventory() {
    let mut g = world();
    let h = g.civilizations.as_ref().unwrap();
    let r = h
        .society
        .as_ref()
        .unwrap()
        .routes
        .iter()
        .find(|r| r.cost_km < 900.)
        .unwrap()
        .clone();
    let a = h.controller(r.from);
    let b = h.controller(r.to);
    g.sign_nonaggression(a, b, 12).unwrap();
    let before = serde_json::to_vec(&g.civilizations).unwrap();
    assert!(g.declare_war(r.from, r.to).is_err());
    assert_eq!(before, serde_json::to_vec(&g.civilizations).unwrap());
    let file = std::env::temp_dir().join(format!("governance-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut resumed = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    g.advance_history(24).unwrap();
    for _ in 0..24 {
        resumed.advance_history(1).unwrap();
    }
    assert!(
        serde_json::to_vec(&g.civilizations).unwrap()
            == serde_json::to_vec(&resumed.civilizations).unwrap(),
        "governance continuation diverged"
    );
    let h = g.civilizations.as_ref().unwrap();
    assert!(h.governance.as_ref().unwrap().treaties[0].expired.is_some());
    assert!(h
        .events
        .iter()
        .any(|e| e.kind == "treaty_expired" && !e.causes.is_empty()));
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
    let mut bad = h.clone();
    bad.governance.as_mut().unwrap().administrations[0].loyalty = f32::NAN;
    assert!(bad.validate(&g.snapshot().unwrap()).is_err());
}
#[test]
#[ignore = "requires hardware GPU"]
fn unfunded_occupation_secedes_but_autonomy_retains_control() {
    let mut g = world();
    let h = g.civilizations.as_ref().unwrap();
    let r = h
        .society
        .as_ref()
        .unwrap()
        .routes
        .iter()
        .find(|r| r.cost_km < 900.)
        .unwrap()
        .clone();
    let culture = h.sites[r.to as usize].civilization;
    g.declare_war(r.from, r.to).unwrap();
    g.advance_history(12).unwrap();
    assert_ne!(g.civilizations.as_ref().unwrap().controller(r.to), culture);
    // Controlled fiscal crisis: transfer council money back to existing residents;
    // no currency or population is deleted to set up the comparison.
    let h = g.civilizations.as_mut().unwrap();
    let owner = h.controller(r.to) as usize;
    let cash = h.society.as_ref().unwrap().councils[owner].treasury;
    h.society.as_mut().unwrap().councils[owner].treasury = 0.;
    h.sites[r.from as usize].economy.finance[0] += cash as f32;
    let a = &mut h.governance.as_mut().unwrap().administrations[r.to as usize];
    a.loyalty = 0.05;
    a.unrest = 0.95;
    a.autonomy = 0.;
    let file = std::env::temp_dir().join(format!("occupation-{}.world", std::process::id()));
    g.save(&file).unwrap();
    let mut autonomous = Generator::load(g.gpu.clone(), &file).unwrap();
    std::fs::remove_file(file).unwrap();
    autonomous.set_autonomy(r.to, 1.).unwrap();
    g.advance_history(36).unwrap();
    autonomous.advance_history(36).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    assert_eq!(h.controller(r.to), culture);
    assert!(h
        .events
        .iter()
        .any(|e| e.kind == "secession" && !e.causes.is_empty()));
    let h = autonomous.civilizations.as_ref().unwrap();
    assert_ne!(
        h.controller(r.to),
        culture,
        "devolved self-rule should retain nominal administration"
    );
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}

#[test]
#[ignore = "requires hardware GPU"]
fn delivered_cross_border_cargo_builds_trust_and_automatic_agreement() {
    let mut g = world();
    let h = g.civilizations.as_mut().unwrap();
    let r = h
        .society
        .as_ref()
        .unwrap()
        .routes
        .iter()
        .find(|r| r.cost_km < 900.)
        .unwrap()
        .clone();
    let a = h.controller(r.from);
    let b = h.controller(r.to);
    // Schedule real tool transfers, removing the cargo from its origin once.
    // Zero-price deliveries are diplomatic gifts within the existing cargo contract.
    assert!(h.sites[r.from as usize].economy.goods[3] >= 12.);
    h.sites[r.from as usize].economy.goods[3] -= 12.;
    for i in 0..120 {
        h.cargo.push(ancient_world::economy::Cargo {
            sea_lane: None,
            weather_delay_months: 0,
            from: r.from,
            to: r.to,
            good: 3,
            kg: 0.1,
            paid: 0.,
            arrives: i / 10 + 1,
        });
    }
    g.advance_history(12).unwrap();
    let h = g.civilizations.as_ref().unwrap();
    let d = h.governance.as_ref().unwrap();
    assert!(d.protected(a, b, h.month));
    assert!(h
        .events
        .iter()
        .any(|e| e.kind == "treaty_signed" && !e.causes.is_empty()));
    assert!(h.economy_residuals().iter().all(|v| v.abs() < 0.001));
}
