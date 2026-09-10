//! Branch one checkpoint; close extraction for five years, then reopen it.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::Result;
use clap::Parser;
use serde_json::json;
use std::path::PathBuf;
#[derive(Parser)]
struct Args {
    #[arg(long, value_delimiter = ',', default_value = "17,81,256")]
    seeds: Vec<u32>,
    #[arg(long, default_value = "output/resource-counterfactual")]
    output: PathBuf,
}
fn sample(g: &Generator) -> serde_json::Value {
    let h = g.civilizations.as_ref().unwrap();
    json!({"month":h.month,"population":h.sites.iter().map(|s|s.stocks.stock[0]).sum::<f32>(),
        "farm_workers":h.sites.iter().map(|s|s.economy.labor[0]).sum::<f32>(),
        "extraction_workers":h.sites.iter().map(|s|s.economy.labor[2]).sum::<f32>(),
        "latest_food_output":h.sites.iter().map(|s|s.stocks.stock[2]).sum::<f32>(),
        "ore_made":h.sites.iter().map(|s|s.economy.made[1]).sum::<f32>(),
        "metal_made":h.sites.iter().map(|s|s.economy.made[2]).sum::<f32>(),
        "tools_made":h.sites.iter().map(|s|s.economy.made[3]).sum::<f32>(),
        "tool_stock":h.sites.iter().map(|s|s.economy.goods[3]).sum::<f32>(),
        "ore_mean_price":h.sites.iter().map(|s|s.economy.prices[1]).sum::<f32>()/h.sites.len() as f32,
        "shortage_sites":h.sites.iter().filter(|s|s.stocks.stock[3]>0.02).count(),
        "market_deliveries":h.events.iter().filter(|e|e.kind=="market_arrival").count(),
        "resource_residual":h.resources.as_ref().unwrap().residual(),
        "economy_residuals":h.economy_residuals(),"sources":h.resources})
}
fn main() -> Result<()> {
    let args = Args::parse();
    std::fs::create_dir_all(args.output.parent().unwrap_or(std::path::Path::new(".")))?;
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut runs = vec![];
    for seed in args.seeds {
        let mut baseline = Generator::new(
            gpu.clone(),
            Config {
                resolution: 64,
                ecology_resolution: 32,
                seed,
                crop_yield_scale: 0.33,
                ..Default::default()
            },
            Catalog::bundled()?,
        )?;
        baseline.run_epochs(1)?;
        baseline.found_civilizations(16)?;
        baseline.enable_society()?;
        baseline.enable_shared_resources()?;
        baseline.enable_living_history()?;
        baseline.advance_history(12)?;
        let checkpoint = args.output.with_extension(format!("{seed}.world"));
        baseline.save(&checkpoint)?;
        let mut treatment = Generator::load(gpu.clone(), &checkpoint)?;
        let initial = sample(&baseline);
        for site in 0..treatment.civilizations.as_ref().unwrap().sites.len() as u32 {
            treatment.set_mine_closed(site, true)?;
        }
        let mut trajectory = vec![];
        for year in 1..=10 {
            if year == 6 {
                let sites: Vec<_> = treatment
                    .civilizations
                    .as_ref()
                    .unwrap()
                    .resources
                    .as_ref()
                    .unwrap()
                    .closed_sites
                    .iter()
                    .copied()
                    .collect();
                for site in sites {
                    treatment.set_mine_closed(site, false)?;
                }
            }
            // Intervention applies to sites existing at the fork; new settlements remain autonomous.
            baseline.advance_history(12)?;
            treatment.advance_history(12)?;
            trajectory.push(json!({"year_after_fork":year,"baseline":sample(&baseline),"treatment":sample(&treatment)}));
        }
        eprintln!(
            "seed {seed}: {}",
            json!({"baseline":trajectory.last().unwrap()["baseline"]["tools_made"],"treatment":trajectory.last().unwrap()["treatment"]["tools_made"]})
        );
        runs.push(json!({"seed":seed,"initial":initial,"trajectory":trajectory}));
        std::fs::write(
            args.output.with_extension("json"),
            serde_json::to_vec_pretty(
                &json!({"version":1,"complete":false,"gpu":gpu.adapter_name,"runs":runs}),
            )?,
        )?;
    }
    std::fs::write(
        args.output.with_extension("json"),
        serde_json::to_vec_pretty(
            &json!({"version":1,"complete":true,"gpu":gpu.adapter_name,"runs":runs}),
        )?,
    )?;
    Ok(())
}
