//! Matched household-nutrition ablation; generated monthly data belongs in output/.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::{ensure, Context, Result};
use clap::Parser;
use serde_json::json;
use std::{io::Write, path::PathBuf, time::Instant};
#[derive(Parser)]
struct Args {
    #[arg(long, value_delimiter = ',', default_value = "17,81,256")]
    seeds: Vec<u32>,
    #[arg(long, value_delimiter = ',', default_value = "0.33,0.15")]
    yields: Vec<f32>,
    #[arg(long, default_value_t = 20)]
    years: u32,
    #[arg(long, default_value_t = 32)]
    resolution: u32,
    #[arg(long, default_value = "output/nutrition-evaluation.jsonl")]
    output: PathBuf,
}
fn main() -> Result<()> {
    let args = Args::parse();
    ensure!(
        args.years > 0 && args.years <= 500 && !args.seeds.is_empty() && !args.yields.is_empty(),
        "invalid suite"
    );
    if let Some(p) = args.output.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut out = std::fs::File::create(&args.output)?;
    let gpu = pollster::block_on(ContextGpu::headless())?;
    writeln!(
        out,
        "{}",
        json!({"type":"settings","gpu":gpu.adapter_name,"years":args.years,"resolution":args.resolution,"seeds":args.seeds,"yields":args.yields,"living":false})
    )?;
    for &seed in &args.seeds {
        for &yield_scale in &args.yields {
            for enabled in [false, true] {
                let start = Instant::now();
                let mut g = Generator::new(
                    gpu.clone(),
                    Config {
                        seed,
                        resolution: args.resolution,
                        ecology_resolution: args.resolution,
                        crop_yield_scale: yield_scale,
                        ..Default::default()
                    },
                    Catalog::bundled()?,
                )?;
                g.run_epochs(1)?;
                g.found_civilizations(5)?;
                g.enable_society()?;
                g.enable_politics()?;
                g.enable_governance()?;
                g.enable_offices()?;
                g.enable_shipping()?;
                let h = g.civilizations.as_mut().unwrap();
                h.set_demographic_resolution(ancient_world::resolution::Mode::Individual, true)?;
                h.society
                    .as_mut()
                    .unwrap()
                    .household_economy
                    .as_mut()
                    .unwrap()
                    .individual_nutrition = enabled;
                let mut max_population_residual = 0f64;
                let mut max_food_residual = 0f64;
                for month in 1..=args.years * 12 {
                    g.advance_history(1).with_context(|| {
                        format!(
                            "seed {seed}, yield {yield_scale}, nutrition {enabled}, month {month}"
                        )
                    })?;
                    let h = g.civilizations.as_ref().unwrap();
                    let e = h
                        .society
                        .as_ref()
                        .unwrap()
                        .household_economy
                        .as_ref()
                        .unwrap();
                    let active: Vec<_> = e
                        .accounts
                        .iter()
                        .filter(|a| {
                            a.need > 0.
                                && a.food_site
                                    .is_some_and(|site| !h.sites[site as usize].abandoned)
                        })
                        .collect();
                    let need = active.iter().map(|a| a.need).sum::<f64>();
                    let hunger =
                        active.iter().map(|a| a.hunger * a.need).sum::<f64>() / need.max(1e-12);
                    let capacity = h.participation.as_ref().map_or(0., |p| {
                        p.residents.values().map(|r| r.capacity as f64).sum::<f64>()
                    });
                    let committed = h.participation.as_ref().map_or(0., |p| {
                        p.residents
                            .values()
                            .map(|r| r.committed as f64)
                            .sum::<f64>()
                    });
                    let population = h
                        .sites
                        .iter()
                        .map(|s| s.stocks.stock[0] as f64)
                        .sum::<f64>();
                    max_population_residual =
                        max_population_residual.max(h.population_residual().abs());
                    max_food_residual = max_food_residual.max(h.food_residual().abs());
                    writeln!(
                        out,
                        "{}",
                        json!({"type":"month","seed":seed,"yield":yield_scale,"enabled":enabled,"month":month,"population":population,"active_sites":h.sites.iter().filter(|s|!s.abandoned).count(),"need_weighted_hunger":hunger,"accounts":active.len(),"hungry_accounts":active.iter().filter(|a|a.hunger>0.5).count(),"personal_capacity":capacity,"committed":committed,"resolution":h.resolution.as_ref().map(|r|r.receipts.iter().map(|receipt|json!({"system":receipt.boundary.system,"site":receipt.boundary.site,"metrics":receipt.metrics})).collect::<Vec<_>>())})
                    )?;
                }
                let h = g.civilizations.as_ref().unwrap();
                let population = h
                    .sites
                    .iter()
                    .map(|s| s.stocks.stock[0] as f64)
                    .sum::<f64>();
                writeln!(
                    out,
                    "{}",
                    json!({"type":"result","seed":seed,"yield":yield_scale,"enabled":enabled,"population":population,"max_population_residual":max_population_residual,"max_food_residual":max_food_residual,"seconds":start.elapsed().as_secs_f64()})
                )?;
                out.flush()?;
                eprintln!("seed {seed} yield {yield_scale} nutrition {enabled}: {population:.1} people, residual {max_population_residual:.3e}, {:.1}s",start.elapsed().as_secs_f64());
            }
        }
    }
    Ok(())
}
