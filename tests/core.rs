use ancient_world::{
    catalog::Catalog,
    config::Config,
    grid,
    viewer::{pick, Camera},
};
#[test]
fn cube_sphere_topology_and_area() {
    for n in [8, 16, 32] {
        let mut area = 0.;
        for i in 0..6 * n * n {
            assert_eq!(grid::index(grid::cell_direction(i, n), n), i);
            area += grid::solid_angle(i, n);
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let j = grid::neighbor(i, n, dx, dy);
                assert_ne!(i, j);
                assert!(
                    [(-1, 0), (1, 0), (0, -1), (0, 1)]
                        .iter()
                        .any(|&(x, y)| grid::neighbor(j, n, x, y) == i),
                    "nonreciprocal edge {i} → {j} at {n}"
                );
            }
        }
        assert!((area - 4. * std::f64::consts::PI).abs() < 1e-10);
    }
}
#[test]
fn validate_catalog_and_configuration() {
    let mut c = Catalog::bundled().unwrap();
    assert_eq!(c.plants.len(), 72);
    c.minerals[0].hosts = vec!["unobtainium".into()];
    assert!(c.validate().is_err());
    let mut c = Catalog::bundled().unwrap();
    c.plants[0].temp_min = f32::NAN;
    assert!(c.validate().is_err());
    assert!(Config {
        resolution: 333,
        ..Default::default()
    }
    .validate()
    .is_err());
    assert!(Config {
        radius_km: f32::NAN,
        ..Default::default()
    }
    .validate()
    .is_err());
}
#[test]
fn picking_agrees_between_views() {
    let globe = pick([0.5, 0.5], 1., Camera::globe(), 32);
    let atlas = pick([0.5, 0.5], 2., Camera::atlas(), 32);
    assert_eq!(globe, atlas);
    assert!(pick([0., 0.], 1., Camera::globe(), 32).is_none());
    let mut c = Camera::atlas();
    c.pan = [1., 0.];
    assert_eq!(pick([0.5, 0.5], 2., c, 32), atlas);
}

#[test]
fn regional_focus_preserves_selection_on_all_faces() {
    for id in [0, 255, 450, 777, 1024, 1535] {
        let d = grid::cell_direction(id, 16);
        for mut camera in [Camera::globe(), Camera::atlas()] {
            camera.focus(d);
            assert_eq!(
                pick([0.5, 0.5], if camera.globe { 1. } else { 2. }, camera, 16),
                Some(id)
            );
        }
    }
}

#[test]
fn abundance_settings_validate_and_old_archives_keep_their_supply_rules() {
    let modern = Config::default();
    assert_eq!(modern.island_phosphorus_scale, 0.25);
    let mut value = serde_json::to_value(&modern).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .remove("island_phosphorus_scale");
    value
        .as_object_mut()
        .unwrap()
        .remove("settlement_plot_hectares");
    value.as_object_mut().unwrap().remove("crop_yield_scale");
    let old: Config = serde_json::from_value(value).unwrap();
    assert_eq!(old.crop_yield_scale, 1.);
    assert_eq!(old.island_phosphorus_scale, 1.);
    assert_eq!(old.settlement_plot_hectares, 5000.);
    for bad in [f32::NAN, 0., 1.1] {
        let mut config = modern.clone();
        config.island_phosphorus_scale = bad;
        assert!(config.validate().is_err());
        config.island_phosphorus_scale = modern.island_phosphorus_scale;
        config.crop_yield_scale = bad;
        assert!(config.validate().is_err());
    }
    for bad in [f32::NAN, 0., 5001.] {
        let mut config = modern.clone();
        config.settlement_plot_hectares = bad;
        assert!(config.validate().is_err());
    }
}
