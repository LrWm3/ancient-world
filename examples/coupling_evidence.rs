//! Matched mine-access × staffing experiment. Model evidence, not empirical calibration.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::{ensure, Result};
use clap::Parser;
use serde_json::{json, Value};
use std::{io::Write, path::PathBuf};
#[derive(Parser)]
struct Args {
    #[arg(long, value_delimiter = ',', default_value = "17,81,256")]
    seeds: Vec<u32>,
    #[arg(long, value_delimiter = ',', default_value = "1")]
    tool_fractions: Vec<f64>,
    #[arg(long)]
    food_security: bool,
    /// Compare adaptive, fixed, food-security, and food-pressure-only staffing.
    #[arg(long, conflicts_with = "food_security")]
    policy_suite: bool,
    #[arg(long, default_value_t = 60)]
    closure_months: u32,
    #[arg(long, default_value_t = 60)]
    recovery_months: u32,
    #[arg(long, default_value = "output/coupling-evidence")]
    output: PathBuf,
}
fn sample(g: &Generator) -> Result<Value> {
    let h = g.civilizations.as_ref().unwrap();
    let residual = h.economy_residuals();
    let population_residual = h.population_residual();
    ensure!(
        population_residual.is_finite() && population_residual.abs() < 0.001,
        "population budget failure at {}",
        h.month
    );
    let travelers = h.society.as_ref().map_or(0., |s| {
        s.raids.iter().map(|r| r.soldiers as f64).sum::<f64>()
            + s.relocation
                .journeys
                .iter()
                .map(|j| j.population() as f64)
                .sum::<f64>()
    }) + h.expeditions.as_ref().map_or(0., |x| {
        x.voyages
            .iter()
            .filter(|v| v.phase.active())
            .map(|v| v.survivors() as f64)
            .sum::<f64>()
    });
    ensure!(
        residual.iter().all(|v| v.is_finite() && v.abs() < 0.001),
        "economy budget failure at {}",
        h.month
    );
    let source = h.resources.as_ref().unwrap().residual();
    ensure!(
        source.is_finite() && source.abs() < 0.001,
        "source budget failure"
    );
    let ecology = g.ecology.budget(&g.gpu, &g.config)?;
    ensure!(
        ecology.within_tolerance,
        "ecology budget failure at {}",
        h.month
    );
    let sites:Vec<_>=h.sites.iter().map(|s|json!({"site":s.id,"population":s.stocks.stock[0],"food_stock_kg":s.stocks.stock[1],"production_kg":s.stocks.stock[2],"shortage_fraction":s.stocks.stock[3],"food_ledger_kg":s.stocks.ledger,"demographic_ledger_people":s.stocks.people,"labor_workers":s.economy.labor,"ages_people":s.demography.ages,"crop_constraint":s.economy.diagnostics[0],"managed_crop_growth_cumulative_kg":s.economy.agriculture[0],"standing_crops":s.economy.crops,"production_probe":s.economy.production_probe,"food_labor":s.economy.food_labor,"restricted_tools_kg":h.experimental_tool_reserves.get(&s.id).copied().unwrap_or(0.),"ration_need_kg":s.demography.ration_need,"ration_eaten_kg":s.demography.ration_eaten,"household_food":s.demography.household_food,"tool_stock_kg":s.economy.goods[3],"ore_price":s.economy.prices[1],"ore_made_kg":s.economy.made[1],"metal_made_kg":s.economy.made[2],"tool_made_kg":s.economy.made[3]})).collect();
    Ok(
        json!({"month":h.month,"population_in_transit":travelers,"population_residual":population_residual,"sites":sites,"economy_residuals":residual,"source_residual":source,"ecology_relative_error":ecology.relative_error,"ecology_water_relative_error":ecology.water_relative_error,"sources":h.resources}),
    )
}
fn main() -> Result<()> {
    let args = Args::parse();
    ensure!(
        !args.seeds.is_empty() && args.closure_months > 0,
        "need seeds and a positive closure interval"
    );
    ensure!(
        !args.tool_fractions.is_empty()
            && args
                .tool_fractions
                .iter()
                .all(|f| f.is_finite() && (0. ..=1.).contains(f)),
        "tool fractions must be in [0,1]"
    );
    ensure!(
        args.seeds
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == args.seeds.len(),
        "duplicate seeds are not independent runs"
    );
    let mut fractions = args.tool_fractions.clone();
    fractions.sort_by(f64::total_cmp);
    fractions.dedup();
    ensure!(
        fractions.len() == args.tool_fractions.len(),
        "duplicate tool fractions"
    );
    std::fs::create_dir_all(&args.output)?;
    let path = args.output.join("results.json");
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&json!({"complete":false,"runs":[]}))?,
    )?;
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let backend = format!("{:?}", gpu.adapter_info.backend);
    let driver = format!(
        "{} {}",
        gpu.adapter_info.driver, gpu.adapter_info.driver_info
    );
    let mut runs = vec![];
    for seed in args.seeds {
        let mut initial = Generator::new(
            gpu.clone(),
            Config {
                seed,
                resolution: 64,
                ecology_resolution: 32,
                crop_yield_scale: 0.33,
                ..Default::default()
            },
            Catalog::bundled()?,
        )?;
        initial.run_epochs(1)?;
        initial.found_civilizations(16)?;
        initial.enable_society()?;
        initial.enable_shared_resources()?;
        initial.enable_living_history()?;
        initial.advance_history(12)?;
        let checkpoint = args.output.join(format!("{seed}.world"));
        initial.save(&checkpoint)?;
        let start = sample(&initial)?;
        let config = initial.config.clone();
        let economy_catalog = initial
            .civilizations
            .as_ref()
            .unwrap()
            .economy_catalog
            .clone();
        // A no-op intervention uses the public access API on already-open mines.
        let mut noop = Generator::load(gpu.clone(), &checkpoint)?;
        let ids: Vec<_> = initial
            .civilizations
            .as_ref()
            .unwrap()
            .sites
            .iter()
            .map(|s| s.id)
            .collect();
        for id in &ids {
            noop.set_mine_closed(*id, false)?;
            noop.set_tool_stock_access(*id, 1.)?;
        }
        initial.advance_history(1)?;
        noop.advance_history(1)?;
        ensure!(
            serde_json::to_value(&initial.civilizations)?
                == serde_json::to_value(&noop.civilizations)?,
            "no-op changed history"
        );
        ensure!(
            sample(&initial)? == sample(&noop)?,
            "no-op changed measured environment"
        );
        drop(initial);
        drop(noop);
        for tool_fraction in &args.tool_fractions {
            let policies = if args.policy_suite {
                vec![
                    (false, false, true),
                    (true, false, true),
                    (false, true, true),
                    (false, true, false),
                ]
            } else {
                vec![(false, args.food_security, true), (true, false, true)]
            };
            for (fixed, food_security, maintenance) in policies {
                let policy = if fixed {
                    "fixed"
                } else if !food_security {
                    "adaptive"
                } else if maintenance {
                    "food-maintenance"
                } else {
                    "food-only"
                };
                let mut branches = vec![];
                for closed in [false, true] {
                    let mut g = Generator::load(gpu.clone(), &checkpoint)?;
                    let mut catalog = g
                        .civilizations
                        .as_ref()
                        .unwrap()
                        .economy_catalog
                        .clone()
                        .unwrap();
                    ensure!(
                        catalog.production.enabled && catalog.production.adaptive_labor,
                        "experiment requires an adaptive planned baseline"
                    );
                    catalog.production.diagnostic_fixed_labor = fixed;
                    catalog.production.food_security_labor = food_security;
                    catalog.production.food_security_maintenance = maintenance;
                    g.configure_economy(catalog)?;
                    for id in &ids {
                        g.set_tool_stock_access(*id, *tool_fraction)?;
                    }
                    let after_stock_intervention = sample(&g)?;
                    if closed {
                        for id in &ids {
                            g.set_mine_closed(*id, true)?;
                        }
                    }
                    let mut months = vec![];
                    let mut journal = std::fs::File::create(args.output.join(format!(
                        "{seed}-tools-{tool_fraction}-policy-{policy}-closed-{closed}.jsonl"
                    )))?;
                    for month in 1..=args.closure_months + args.recovery_months {
                        if closed && month == args.closure_months + 1 {
                            for id in &ids {
                                g.set_mine_closed(*id, false)?;
                            }
                        }
                        g.advance_history(1)?;
                        let measured = sample(&g)?;
                        writeln!(journal, "{}", serde_json::to_string(&measured)?)?;
                        months.push(measured);
                        if month % 12 == 0 {
                            eprintln!("seed {seed}, tools={tool_fraction}, policy={policy}, closed={closed}, month {month}");
                        }
                    }
                    branches.push(json!({"closed":closed,"after_stock_intervention":after_stock_intervention,"months":months}));
                }
                runs.push(json!({"seed":seed,"tool_fraction":tool_fraction,"config":config,"baseline_economy_catalog":economy_catalog,"fixed_labor":fixed,"food_security":food_security,"maintenance":maintenance,"policy":policy,"initial":start,"no_op_passed":true,"branches":branches}));
                std::fs::write(
                    &path,
                    serde_json::to_vec_pretty(
                        &json!({"version":3,"complete":false,"gpu":gpu.adapter_name,"backend":backend,"driver":driver,"closure_months":args.closure_months,"recovery_months":args.recovery_months,"runs":runs}),
                    )?,
                )?;
            }
        }
    }
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(
            &json!({"version":3,"complete":true,"gpu":gpu.adapter_name,"backend":backend,"driver":driver,"closure_months":args.closure_months,"recovery_months":args.recovery_months,"runs":runs}),
        )?,
    )?;
    Ok(())
}
