//! Matched policy experiments; generated output belongs in ignored output/.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::{Context, Result};
use clap::Parser;
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf, time::Instant};
#[derive(Parser)]
struct Args {
    #[arg(long, value_delimiter = ',', default_value = "17,81,256,409,1024")]
    seeds: Vec<u32>,
    #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u32).range(1..))]
    years: u32,
    #[arg(long, default_value = "output/civic-balance.json")]
    output: PathBuf,
    #[arg(long, default_value_t = 1)]
    epochs: u32,
    #[arg(long, default_value_t = 0.5)]
    crop_yield_scale: f32,
}
fn main() -> Result<()> {
    let args = Args::parse();
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut rows = vec![];
    for seed in args.seeds {
        for enabled in [false, true] {
            let start = Instant::now();
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    resolution: 64,
                    ecology_resolution: 16,
                    seed,
                    crop_yield_scale: args.crop_yield_scale,
                    ..Default::default()
                },
                Catalog::bundled()?,
            )?;
            g.run_epochs(args.epochs)?;
            g.found_civilizations(16)?;
            g.enable_society()?;
            g.enable_politics()?;
            g.enable_governance()?;
            g.enable_offices()?;
            g.enable_shipping()?;
            g.civilizations
                .as_mut()
                .unwrap()
                .governance
                .as_mut()
                .unwrap()
                .petitions_enabled = enabled;
            let mut samples = vec![];
            let mut residual = 0f64;
            for year in 1..=args.years {
                g.advance_history(12)
                    .with_context(|| format!("seed {seed}, petitions {enabled}, year {year}"))?;
                let h = g.civilizations.as_ref().unwrap();
                residual = h
                    .economy_residuals()
                    .iter()
                    .fold(residual, |a, v| a.max(v.abs() as f64));
                let c = h.culture.as_ref().unwrap();
                let gov = h.governance.as_ref().unwrap();
                samples.push(json!({"year":year,"population":h.sites.iter().map(|s|s.stocks.stock[0] as f64).sum::<f64>(),
     "active_sites":h.sites.iter().filter(|s|!s.abandoned).count(),
     "hungry_sites":h.sites.iter().filter(|s|!s.abandoned && s.stocks.stock[3]>0.3).count(),
     "institutions":c.institutions.iter().filter(|n|n.active).count(),
     "operational":c.institutions.iter().filter(|n|n.operational()).count(),
     "petitions":gov.petitions.len(),"honored":gov.petitions.iter().filter(|p|p.honored).count(),
     "paid":gov.petitions.iter().map(|p|p.paid).sum::<f64>(),
     "council_cash":h.society.as_ref().unwrap().councils.iter().map(|c|c.treasury).sum::<f64>(),
     "administrative_pay":gov.administrations.iter().map(|a|a.wages_paid).sum::<f64>(),
     "max_support":h.politics.as_ref().unwrap().factions.iter().map(|f|f.support).fold(0f32,f32::max)}));
            }
            let h = g.civilizations.as_ref().unwrap();
            let mut events = BTreeMap::<String, usize>::new();
            for e in &h.events {
                *events.entry(e.kind.clone()).or_default() += 1;
            }
            let row = json!({"seed":seed,"enabled":enabled,"seconds":start.elapsed().as_secs_f64(),"max_residual":residual,
       "samples":samples,"petitions":h.governance.as_ref().unwrap().petitions,"events":events});
            eprintln!(
                "seed {seed} enabled {enabled}: {} petitions, {} honored, {:.1}s",
                row["petitions"].as_array().unwrap().len(),
                row["samples"].as_array().unwrap().last().unwrap()["honored"],
                start.elapsed().as_secs_f64()
            );
            rows.push(row);
            if let Some(parent) = args.output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(
                &args.output,
                serde_json::to_vec_pretty(
                    &json!({"gpu":gpu.adapter_name,"years":args.years,"epochs":args.epochs,"crop_yield_scale":args.crop_yield_scale,"runs":rows}),
                )?,
            )?;
        }
    }
    Ok(())
}
