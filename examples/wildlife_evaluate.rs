//! Reproducible connectivity ablation; simulation remains on the GPU.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::{ensure, Result};
fn main() -> Result<()> {
    let months = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "120".into())
        .parse::<u32>()?;
    let resolution = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "32".into())
        .parse::<u32>()?;
    ensure!((1..=6000).contains(&months), "months must be 1..6000");
    let threshold = std::env::args()
        .nth(3)
        .map(|v| v.parse::<f32>())
        .transpose()?;
    let basal = std::env::args()
        .nth(4)
        .map(|v| v.parse::<f32>())
        .transpose()?;
    let intake = std::env::args()
        .nth(5)
        .map(|v| v.parse::<f32>())
        .transpose()?;
    let river_intake = std::env::args()
        .nth(6)
        .map(|v| v.parse::<f32>())
        .transpose()?;
    let land_intake = std::env::args()
        .nth(7)
        .map(|v| v.parse::<f32>())
        .transpose()?;
    let aquatic_thermal_width_c = std::env::var("WILDLIFE_AQUATIC_THERMAL_WIDTH")
        .ok()
        .map(|s| s.parse::<f32>())
        .transpose()?
        .unwrap_or(15.);
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut rows = vec![];
    let seeds = std::env::args()
        .nth(8)
        .map(|v| v.parse::<u32>().map(|s| vec![s]))
        .transpose()?
        .unwrap_or_else(|| vec![17, 81, 256]);
    let seed_suffix = if seeds.len() == 1 {
        format!("-s{}", seeds[0])
    } else {
        String::new()
    };
    for seed in seeds {
        let barriers = if std::env::var("WILDLIFE_CLOSED_ONLY").as_deref() == Ok("1") {
            vec![false]
        } else {
            vec![false, true]
        };
        for open in barriers {
            let mut catalog = match std::env::var("WILDLIFE_CATALOG") {
                Ok(path) => Catalog::parse(&std::fs::read_to_string(path)?)?,
                Err(_) => Catalog::bundled()?,
            };
            if let Some(value) = threshold {
                catalog.guilds[9].food_half_saturation = value;
                catalog.guilds[11].food_half_saturation = value;
            }
            if let Some(value) = basal {
                catalog.guilds[8].food_half_saturation = value;
                catalog.guilds[10].food_half_saturation = value;
            }
            if let Some(value) = intake {
                for k in [0, 1, 2, 3, 4, 8, 10] {
                    catalog.guilds[k].feeding = value;
                }
            }
            if let Some(value) = river_intake {
                catalog.guilds[10].feeding = value;
            }
            if let Some(value) = land_intake {
                for k in 0..5 {
                    catalog.guilds[k].feeding = value;
                }
            }
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    seed,
                    resolution,
                    ecology_resolution: resolution / 2,
                    ecology_years_per_epoch: 1,
                    wildlife_open_barriers: false,
                    aquatic_thermal_width_c,
                    ..Default::default()
                },
                catalog,
            )?;
            g.run_epochs(1)?;
            let before = g.ecology.wildlife_report(&gpu, &g.config)?;
            g.config.wildlife_open_barriers = open;
            // Matched opening stocks; traits initialize on the first biological step.
            g.config.wildlife_ecotypes = std::env::var("WILDLIFE_ECOTYPES").as_deref() == Ok("1");
            let start = std::time::Instant::now();
            let mut timings = std::collections::BTreeMap::<String, f64>::new();
            let mut trajectory = Vec::new();
            for step in 0..months {
                g.advance_ecology()?;
                for (stage, ms) in &g.ecology.last_pass_ms {
                    *timings.entry(stage.clone()).or_default() += ms;
                }
                if (step + 1) % 120 == 0 {
                    trajectory.push(g.ecology.wildlife_report(&gpu, &g.config)?);
                }
            }
            let after = g.ecology.wildlife_report(&gpu, &g.config)?;
            ancient_world::ecology::validate(&g.ecology.snapshot(&gpu, &g.config)?)?;
            let budget = g.ecology.budget(&gpu, &g.config)?;
            ensure!(
                budget.within_tolerance,
                "budget failed: {:?}",
                budget.relative_error
            );
            eprintln!(
                "seed {seed}, open {open}: {:.1}s, residual {:?}",
                start.elapsed().as_secs_f64(),
                budget.relative_error
            );
            eprintln!(
                "lake predator fraction of consumers {:.6}, predator occupied {:.3}",
                after.carbon_kg[1][9]
                    / (after.carbon_kg[1][8] + after.carbon_kg[1][9] + after.carbon_kg[1][10])
                        .max(1e-30),
                after.occupied_fraction[1][9]
            );
            rows.push(serde_json::json!({"trajectory":trajectory,"guild_catalog":g.catalog.guilds,"feeding_by_guild":g.catalog.guilds.iter().map(|g|g.feeding).collect::<Vec<_>>(),"herbivore_intake":g.catalog.guilds[8].feeding,"grazer_river_half_saturation":g.catalog.guilds[8].food_half_saturation,"predator_bird_half_saturation":g.catalog.guilds[9].food_half_saturation,"config":g.config,"months":months,"before":before,"after":after,"budget":budget,"seconds":start.elapsed().as_secs_f64(),"gpu_stage_ms":timings}));
        }
    }
    std::fs::create_dir_all("output/wildlife")?;
    let suffix = format!(
        "{}{}{}{}{}",
        threshold.map(|t| format!("-h{t}")).unwrap_or_default(),
        basal.map(|t| format!("-b{t}")).unwrap_or_default(),
        intake.map(|t| format!("-i{t}")).unwrap_or_default(),
        river_intake.map(|t| format!("-r{t}")).unwrap_or_default(),
        land_intake.map(|t| format!("-l{t}")).unwrap_or_default()
    );
    std::fs::write(
        std::env::var("WILDLIFE_OUTPUT").unwrap_or_else(|_| {
            format!("output/wildlife/assembly-{resolution}-{months}{suffix}{seed_suffix}.json")
        }),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    Ok(())
}
