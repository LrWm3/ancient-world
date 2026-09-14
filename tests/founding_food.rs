//! Opt-in end-to-end guard for the forgiving arrival inventory.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};

#[test]
#[ignore = "requires a GPU; five years across three seeds, two provision arms and both history clocks"]
fn founding_provisions_avoid_early_starvation() {
    let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
    let mut control_shortages = 0;
    for living in [false, true] {
        for months in [12, 48] {
            for seed in [1024, 256, 409] {
                let mut g = Generator::new(
                    gpu.clone(),
                    Config {
                        resolution: 32,
                        ecology_resolution: 32,
                        seed,
                        ..Default::default()
                    },
                    Catalog::bundled().unwrap(),
                )
                .unwrap();
                g.run_epochs(1).unwrap();
                if months == 48 {
                    g.found_civilizations(5).unwrap();
                } else {
                    g.found_civilizations_with_options(
                        5,
                        ancient_world::culture::FoundingOptions {
                            food_months: 12,
                            granary_months: 12.,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                }
                g.config
                    .systems
                    .select(ancient_world::systems::System::LivingWorld, living);
                g.apply_systems(&g.config.systems.clone()).unwrap();
                let h = g.civilizations.as_mut().unwrap();
                assert_eq!(
                    h.economy_catalog
                        .as_ref()
                        .unwrap()
                        .production
                        .base_granary_months,
                    months as f32
                );
                for p in &h.culture.as_ref().unwrap().patrons {
                    assert_eq!(p.initial_food, p.initial_people * 18. * months as f32);
                }
                h.demographic_audit = Some(Default::default());
                g.advance_history(60).unwrap();
                let h = g.civilizations.as_ref().unwrap();
                let audit = h.demographic_audit.as_ref().unwrap();
                assert_eq!(
                    audit.food.months.len(),
                    300,
                    "seed {seed}: missing town-month observations"
                );
                for row in &audit.food.months {
                    assert!(row.need_kg.is_finite() && row.need_kg > 0.);
                    assert!(row.eaten_kg.is_finite());
                    if months == 12 {
                        control_shortages += usize::from(row.need_kg - row.eaten_kg >= 0.01);
                        continue;
                    }
                    assert!(
                        row.need_kg - row.eaten_kg < 0.01,
                        "seed {seed}, month {}, town {}: need {} kg, ate {} kg, available {} kg",
                        row.month,
                        row.site,
                        row.need_kg,
                        row.eaten_kg,
                        row.physically_available_kg
                    );
                }
                let hunger_deaths: f64 = audit.years.iter().map(|y| y.nutrition_deaths).sum();
                assert!(
                    months == 12 || hunger_deaths < 0.01,
                    "seed {seed}: {hunger_deaths} nutrition-attributed deaths"
                );
                eprintln!("seed {seed}, living {living}, provisions {months}: population {:.2}; hunger deaths {hunger_deaths}", h.sites.iter().map(|s| s.stocks.stock[0] as f64).sum::<f64>());
            }
        }
    }
    assert!(
        control_shortages > 0,
        "legacy control no longer exposes founding hunger; revisit fixture sensitivity"
    );
    eprintln!("legacy control shortage town-months: {control_shortages}");
}
