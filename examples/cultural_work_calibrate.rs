//! Quarterly observations; raw outputs belong under ignored output/.
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
    #[arg(long)]
    strict_identities: bool,
    #[arg(long, value_delimiter = ',', default_value = "17,81,256")]
    seeds: Vec<u32>,
    #[arg(long, default_value_t = 100)]
    years: u32,
    #[arg(long, default_value_t = 32)]
    resolution: u32,
    #[arg(long, default_value = "output/cultural-work-baseline.json")]
    output: PathBuf,
}
fn main() -> Result<()> {
    let args = Args::parse();
    anyhow::ensure!(args.years > 0 && args.years <= 500, "years must be 1..500");
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut rows = vec![];
    for seed in &args.seeds {
        let start = Instant::now();
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                seed: *seed,
                resolution: args.resolution,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled()?,
        )?;
        g.run_epochs(1)?;
        g.found_civilizations(16)?;
        g.civilizations
            .as_mut()
            .unwrap()
            .culture
            .as_mut()
            .unwrap()
            .focused_work_identities = !args.strict_identities;
        g.enable_society()?;
        g.enable_politics()?;
        g.enable_governance()?;
        g.enable_offices()?;
        g.enable_shipping()?;
        g.enable_expeditions()?;
        g.enable_discoveries()?;
        g.enable_living_history()?;
        let mut samples = vec![];
        let mut changes = BTreeMap::<String, u64>::new();
        let mut actions = BTreeMap::<String, u64>::new();
        let (mut requested, mut granted, mut used, mut cancelled_work) = (0., 0., 0., 0.);
        let (mut funded, mut cancelled) = (0u64, 0u64);
        for quarter in 1..=args.years * 4 {
            g.advance_history(3)
                .with_context(|| format!("seed {seed}, quarter {quarter}"))?;
            let h = g.civilizations.as_ref().unwrap();
            let c = h.culture.as_ref().unwrap();
            requested += c.work_receipt.requested;
            granted += c.work_receipt.granted;
            used += c.work_receipt.used;
            for p in &c.work_plans {
                if p.granted <= 0. {
                    continue;
                }
                funded += 1;
                if p.cancellation.is_some() {
                    cancelled += 1;
                    cancelled_work += p.cancelled_work as f64;
                    for key in &p.changed_identities {
                        *changes.entry(key.clone()).or_default() += 1;
                    }
                    for (a, _) in &p.actions {
                        *actions.entry(a.clone()).or_default() += 1;
                    }
                }
            }
            if quarter % 40 == 0 || quarter == args.years * 4 {
                let row = json!({"year":quarter/4,"funded_bundles":funded,"cancelled_bundles":cancelled,"requested":requested,"granted":granted,"used":used,"cancelled_work":cancelled_work,"population":h.sites.iter().map(|s|s.stocks.stock[0] as f64).sum::<f64>(),"active_sites":h.sites.iter().filter(|s|!s.abandoned).count(),"institutions":c.institutions.iter().filter(|n|n.active).count(),"knowledge_links":c.agents.iter().map(|a|a.knowledge.len()).sum::<usize>(),"artifacts":c.artifacts.len()});
                eprintln!("seed {seed}: {} years, cancelled {cancelled}/{funded}, work {used:.1}/{granted:.1}, {:.1}s",quarter/4,start.elapsed().as_secs_f64());
                samples.push(row);
            }
        }
        let h = g.civilizations.as_ref().unwrap();
        let mut events = BTreeMap::<String, u64>::new();
        for e in &h.events {
            *events.entry(e.kind.clone()).or_default() += 1;
        }
        rows.push(json!({"seed":seed,"seconds":start.elapsed().as_secs_f64(),"samples":samples,"changed_identities":changes,"cancelled_actions":actions,"events":events,"residuals":h.economy_residuals()}));
        if let Some(parent) = args.output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &args.output,
            serde_json::to_vec_pretty(
                &json!({"strict_identities":args.strict_identities,"years":args.years,"resolution":args.resolution,"ecology_resolution":16,"epochs":1,"seeds":args.seeds,"gpu":gpu.adapter_name,"complete":rows.len()==args.seeds.len(),"runs":rows}),
            )?,
        )?;
    }
    Ok(())
}
