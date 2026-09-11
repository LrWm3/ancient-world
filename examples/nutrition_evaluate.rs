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
    /// Isolate food access: nutrition stays enabled; vary common entitlements instead.
    #[arg(long)]
    affordability: bool,
    /// Include the default founding communal-to-household transition (otherwise static policy).
    #[arg(long)]
    founding_access: bool,
    #[arg(long, value_delimiter = ',', default_value = "0.5,0.75,1.0")]
    common_shares: Vec<f32>,
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
    ensure!(
        !args.common_shares.is_empty()
            && args
                .common_shares
                .iter()
                .all(|x| x.is_finite() && (0. ..=1.).contains(x)),
        "invalid common shares"
    );
    if let Some(p) = args.output.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut out = std::fs::File::create(&args.output)?;
    let gpu = pollster::block_on(ContextGpu::headless())?;
    writeln!(
        out,
        "{}",
        json!({"type":"settings","gpu":gpu.adapter_name,"years":args.years,"resolution":args.resolution,"seeds":args.seeds,"yields":args.yields,"living":false,"affordability":args.affordability,"founding_access":args.founding_access,"common_shares":args.common_shares})
    )?;
    for &seed in &args.seeds {
        for &yield_scale in &args.yields {
            let variants: Vec<_> = if args.affordability {
                args.common_shares.iter().map(|&s| (true, s)).collect()
            } else {
                vec![(false, 0.5), (true, 0.5)]
            };
            for (enabled, common_share) in variants {
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
                if !args.founding_access {
                    h.society
                        .as_mut()
                        .unwrap()
                        .household_economy
                        .as_mut()
                        .unwrap()
                        .founding_access = None;
                }
                h.set_demographic_resolution(ancient_world::resolution::Mode::Individual, true)?;
                h.society
                    .as_mut()
                    .unwrap()
                    .household_economy
                    .as_mut()
                    .unwrap()
                    .individual_nutrition = enabled;
                h.society
                    .as_mut()
                    .unwrap()
                    .household_economy
                    .as_mut()
                    .unwrap()
                    .common_share = common_share;
                let mut max_population_residual = 0f64;
                let mut max_food_residual = 0f64;
                for month in 1..=args.years * 12 {
                    let production_before = g
                        .civilizations
                        .as_ref()
                        .unwrap()
                        .sites
                        .iter()
                        .map(|s| s.stocks.ledger[0] as f64)
                        .sum::<f64>();
                    g.advance_history(1).with_context(|| {
                        format!(
                            "seed {seed}, yield {yield_scale}, nutrition {enabled}, common {common_share}, month {month}"
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
                    // Decompose locally: a surplus in one town cannot feed a different town instantly.
                    let mut food = [0f64; 6]; // need, available, funded, eaten, physical gap, access gap
                    for site in &h.sites {
                        let d = &site.demography;
                        if d.household_food[1] <= 0.5 {
                            continue;
                        }
                        let n = d.ration_need[3] as f64;
                        let a = d.household_food[2] as f64;
                        let c = d.ration_eaten[3] as f64;
                        let gaps = food_gaps(n, a, c);
                        ensure!(
                            (n - c - gaps[0] - gaps[1]).abs() <= 1e-4 * (1. + n),
                            "food gap decomposition failed"
                        );
                        for (total, value) in food.iter_mut().zip([
                            n,
                            a,
                            d.household_food[0] as f64,
                            c,
                            gaps[0],
                            gaps[1],
                        ]) {
                            *total += value;
                        }
                    }
                    let produced = h
                        .sites
                        .iter()
                        .map(|s| s.stocks.ledger[0] as f64)
                        .sum::<f64>()
                        - production_before;
                    writeln!(
                        out,
                        "{}",
                        json!({"type":"month","seed":seed,"yield":yield_scale,"enabled":enabled,"common_share":common_share,"food":food,"produced":produced,"month":month,"population":population,"active_sites":h.sites.iter().filter(|s|!s.abandoned).count(),"need_weighted_hunger":hunger,"accounts":active.len(),"hungry_accounts":active.iter().filter(|a|a.hunger>0.5).count(),"personal_capacity":capacity,"committed":committed,"resolution":h.resolution.as_ref().map(|r|r.receipts.iter().map(|receipt|json!({"system":receipt.boundary.system,"site":receipt.boundary.site,"metrics":receipt.metrics})).collect::<Vec<_>>())})
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
                    json!({"type":"result","seed":seed,"yield":yield_scale,"enabled":enabled,"common_share":common_share,"population":population,"max_population_residual":max_population_residual,"max_food_residual":max_food_residual,"seconds":start.elapsed().as_secs_f64()})
                )?;
                out.flush()?;
                eprintln!("seed {seed} yield {yield_scale} nutrition {enabled} common {common_share}: {population:.1} people, residual {max_population_residual:.3e}, {:.1}s",start.elapsed().as_secs_f64());
            }
        }
    }
    Ok(())
}

// These are consumption-boundary attribution quantities, not extra food transfers.
fn food_gaps(need: f64, available: f64, eaten: f64) -> [f64; 2] {
    [
        (need - available).max(0.),
        (need.min(available) - eaten).max(0.),
    ]
}
#[cfg(test)]
mod tests {
    use super::food_gaps;
    #[test]
    fn scarcity_and_access_are_distinct_and_add_to_unmet_need() {
        for (need, available, eaten, expected) in [
            (100., 200., 50., [0., 50.]),
            (100., 40., 40., [60., 0.]),
            (100., 40., 20., [60., 20.]),
            (100., 200., 100., [0., 0.]),
            (0., 200., 0., [0., 0.]),
        ] {
            let gaps = food_gaps(need, available, eaten);
            assert_eq!(gaps, expected);
            assert_eq!(gaps.iter().sum::<f64>(), need - eaten);
        }
        // Regional totals cannot substitute distant surplus for local access.
        assert_eq!(
            food_gaps(100., 0., 0.)[0] + food_gaps(100., 200., 100.)[0],
            100.
        );
    }
}
