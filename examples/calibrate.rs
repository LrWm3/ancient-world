//! Reproducible regional comparisons with a full year's production sampling.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::{ensure, Context, Result};
fn main() -> Result<()> {
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let epochs: u32 = std::env::args().nth(1).unwrap_or("10".into()).parse()?;
    let resolution: u32 = std::env::args().nth(2).unwrap_or("64".into()).parse()?;
    let ecology_resolution: u32 = std::env::args()
        .nth(3)
        .unwrap_or(resolution.to_string())
        .parse()?;
    let seeds: Vec<u32> = std::env::args()
        .nth(4)
        .unwrap_or("0,7,42,99,999".into())
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()?;
    let output = std::env::args()
        .nth(5)
        .unwrap_or("output/calibration.json".into());
    let warmup_months: u32 = std::env::args().nth(6).unwrap_or("0".into()).parse()?;
    let mut reports = Vec::new();
    for seed in seeds {
        let started = std::time::Instant::now();
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                resolution,
                ecology_resolution,
                seed,
                ..Default::default()
            },
            Catalog::bundled()?,
        )?;
        g.run_epochs(epochs).with_context(|| {
            format!(
                "seed {seed}, geological epoch {}, stage {:?}",
                g.progress.epoch, g.progress.stage
            )
        })?;
        for month in 0..warmup_months {
            g.advance_ecology()
                .with_context(|| format!("seed {seed}, ecology warmup month {month}"))?;
            if month % 120 == 119 {
                ensure!(
                    g.ecology.budget(&g.gpu, &g.config)?.within_tolerance,
                    "warmup budget failure, seed {seed}, month {month}"
                );
            }
        }
        let mut annual = [[0.; 2]; 4];
        let mut growth = [[0.; 5]; 4];
        let mut losses = [[0.; 5]; 4];
        let mut limits = [[[0.; 7]; 5]; 4];
        let mut timings = std::collections::BTreeMap::<String, f64>::new();
        for _ in 0..12 {
            g.advance_ecology()?;
            let b = g.ecology.budget(&g.gpu, &g.config)?;
            ensure!(b.within_tolerance, "budget failure for seed {seed}");
            for (name, ms) in &g.ecology.last_pass_ms {
                *timings.entry(name.clone()).or_default() += ms / 12.;
            }
            for (j, (r, a)) in b.regions.iter().zip(&mut annual).enumerate() {
                a[0] += r.photo_kg_c_per_m2_year / 12.;
                a[1] += r.chemo_kg_c_per_m2_year / 12.;
                for k in 0..5 {
                    growth[j][k] += r.layer_growth_kg_c_per_m2_year[k] / 12.;
                    losses[j][k] += r.layer_loss_kg_c_per_m2_year[k] / 12.;
                    for q in 0..7 {
                        limits[j][k][q] += r.layer_limit_fraction[k][q] / 12.;
                    }
                }
            }
        }
        ensure!(
            annual[2][0] > annual[2][1] * 19.,
            "inner continents no longer primarily solar, seed {seed}"
        );
        ensure!(
            annual[3][1] > annual[2][1],
            "outer geochemical supply not differentiated, seed {seed}"
        );
        let report = serde_json::json!({"seed":seed,"epochs":epochs,"ecology_only_warmup_months":warmup_months,"terrain_resolution":resolution,"ecology_resolution":ecology_resolution,"elapsed_seconds":started.elapsed().as_secs_f64(),"progress":g.progress,"mean_monthly_gpu_ms":timings,"annual_layer_growth_kg_c_per_m2":growth,"annual_layer_loss_kg_c_per_m2":losses,"annual_layer_limit_fraction":limits,"limit_names":ancient_world::ecology::GROWTH_LIMITS,"annual_production_kg_c_per_m2":annual,"budget":g.ecology.budget(&g.gpu,&g.config)?});
        println!("{report}");
        reports.push(report);
    }
    std::fs::create_dir_all("output")?;
    std::fs::write(output, serde_json::to_string_pretty(&reports)?)?;
    Ok(())
}
