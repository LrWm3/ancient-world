//! Paired ecological assembly audit: no CPU simulation or biological fallback.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::{ensure, Result};
fn main() -> Result<()> {
    let months = std::env::args()
        .nth(1)
        .map(|s| s.parse::<u32>())
        .transpose()?
        .unwrap_or(120);
    ensure!((1..=12000).contains(&months), "months must be 1..12000");
    let audit = std::env::args().nth(2).unwrap_or_default();
    let tuning = audit == "tuning";
    let aquatic_audit = audit == "aquatic" || tuning || audit == "tuned";
    let modes: &[&str] = if tuning {
        &["moderate", "permissive"]
    } else if audit == "tuned" {
        &["tuned"]
    } else if aquatic_audit {
        &["previous_diets", "revised_diets"]
    } else {
        &["single", "competition", "foodweb"]
    };
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut rows = vec![];
    let seeds = if let Some(seed) = std::env::args().nth(3) {
        vec![seed.parse::<u32>()?]
    } else {
        vec![17, 81, 256]
    };
    for seed in seeds {
        for &mode in modes {
            let mut catalog = Catalog::bundled()?;
            catalog.producer_competition = mode != "single";
            if !aquatic_audit && mode != "foodweb" {
                for g in &mut catalog.guilds {
                    g.diet.clear();
                }
            }
            if mode == "previous_diets" {
                for guild in &mut catalog.guilds {
                    guild.food_half_saturation = 0.;
                    for food in &mut guild.diet {
                        food.efficiency = 1.;
                    }
                }
                catalog.guilds[9].diet = vec![
                    ancient_world::catalog::DietItem {
                        prey: 13,
                        weight: 0.7,
                        efficiency: 1.,
                    },
                    ancient_world::catalog::DietItem {
                        prey: 15,
                        weight: 0.2,
                        efficiency: 1.,
                    },
                    ancient_world::catalog::DietItem {
                        prey: 23,
                        weight: 0.1,
                        efficiency: 1.,
                    },
                ];
            }
            if tuning {
                let threshold = if mode == "moderate" {
                    0.000005
                } else {
                    0.0000005
                };
                catalog.guilds[9].food_half_saturation = threshold;
                catalog.guilds[11].food_half_saturation = threshold;
            }
            let thresholds = [
                catalog.guilds[9].food_half_saturation,
                catalog.guilds[11].food_half_saturation,
            ];
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    seed,
                    resolution: 64,
                    ecology_resolution: 32,
                    ecology_years_per_epoch: 1,
                    ..Default::default()
                },
                catalog,
            )?;
            g.run_epochs(1)?;
            let before = g.ecology.snapshot(&gpu, &g.config)?;
            let start = std::time::Instant::now();
            for _ in 0..months {
                g.advance_ecology()?;
            }
            let seconds = start.elapsed().as_secs_f64();
            let cells = g.ecology.snapshot(&gpu, &g.config)?;
            ancient_world::ecology::validate(&cells)?;
            let budget = g.ecology.budget(&gpu, &g.config)?;
            ensure!(
                budget.relative_error.iter().all(|v| *v < 1e-4),
                "budget error: {:?}",
                budget.relative_error
            );
            let mut mixed = [0usize; 6];
            let mut changed = [0usize; 6];
            let mut guilds = [0f64; 12];
            for (old, c) in before.iter().zip(&cells) {
                for k in 0..6 {
                    let p = c.pools[32 + k];
                    let pool = if k == 5 { 23 } else { k };
                    if p[0] > 0.
                        && p[1] > 0.
                        && p[2] > 0.05
                        && p[2] < 0.95
                        && c.pools[pool][0] > 1e-6
                    {
                        mixed[k] += 1;
                    }
                    if (old.pools[32 + k][2] - p[2]).abs() > 0.05 {
                        changed[k] += 1;
                    }
                }
                for (k, biomass) in guilds.iter_mut().enumerate() {
                    *biomass += c.pools[k + 5][0] as f64 / cells.len() as f64;
                }
            }
            eprintln!(
                "seed {seed} {mode}: {seconds:.2}s, mixed {mixed:?}, residual {:?}",
                budget.relative_error
            );
            eprintln!("aquatic guild carbon {:?}", &guilds[8..]);
            rows.push(serde_json::json!({"seed":seed,"mode":mode,"months":months,"predator_bird_half_saturation":thresholds,"seconds":seconds,"mixed_cells_by_layer":mixed,"composition_changed_cells":changed,"guild_mean_carbon_kg_m2_unweighted":guilds,"budget":budget,"ecology_cell_bytes":ancient_world::ecology::ECO_BYTES}));
        }
    }
    std::fs::create_dir_all("output/foodweb")?;
    std::fs::write(
        format!(
            "output/foodweb/{}-{months}.json",
            if tuning {
                "aquatic-tuning"
            } else if audit == "tuned" {
                "aquatic-tuned"
            } else if aquatic_audit {
                "aquatic-diets"
            } else {
                "comparison"
            }
        ),
        serde_json::to_vec_pretty(&rows)?,
    )?;
    Ok(())
}
