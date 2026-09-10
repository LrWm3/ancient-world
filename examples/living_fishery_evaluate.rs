//! Natural-stock, matched-checkpoint fishery intervention trajectories.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    ecology::Intervention,
    gpu::{ContextGpu, Generator},
};
use anyhow::{ensure, Result};
use serde_json::json;
use std::{collections::BTreeMap, io::Write};

fn sample(
    g: &Generator,
    branch: &str,
    elapsed: u32,
    prior: &mut BTreeMap<u32, f64>,
) -> Result<serde_json::Value> {
    let h = g.civilizations.as_ref().unwrap();
    let cells = g.ecology.snapshot(&g.gpu, &g.config)?;
    ancient_world::ecology::validate(&cells)?;
    let mut sites = Vec::new();
    for s in &h.sites {
        let total = f64::from(s.economy.agriculture[2]);
        let catch = total - prior.insert(s.id, total).unwrap_or(total);
        let receiving =
            (s.economy.management[1] >= 1.).then(|| s.economy.management[1] as usize - 1);
        let accessible_carbon = receiving.map(|i| {
            [
                cells[i].pools[13][0],
                cells[i].pools[14][0],
                cells[i].pools[15][0],
            ]
        });
        let social = h
            .society
            .as_ref()
            .and_then(|s| s.indicators.as_ref())
            .and_then(|state| state.sites.get(s.id as usize));
        sites.push(json!({"id":s.id,"population":s.stocks.stock[0],"abandoned":s.abandoned,"catch_kg":catch,"food_reserve_equivalent_kg":s.stocks.stock[1],"ration_need":s.demography.ration_need,"ration_eaten":s.demography.ration_eaten,"fish_price":s.economy.prices[28],"receiving_ecology_cell":receiving,"accessible_guild_carbon_kg_m2":accessible_carbon,"household_stress":social.map(|c| c.household_stress),"food_security_households":social.map(|c| c.food_security),"fishery_equipment":s.economy.fishery,"fishery_traps":s.economy.fishery_traps,"fishery_choice":s.economy.fishery_choice,"fishery_inputs_kg":[s.economy.goods[0],s.economy.goods[3],s.economy.goods[16]],"fishery_input_targets_kg":[s.economy.targets[0],s.economy.targets[3],s.economy.targets[16]],"fishery_plan":s.economy.fishery_plan,"fishery_stats":s.economy.fishery_stats,"labor":s.economy.labor}));
    }
    let residuals = h.economy_residuals();
    ensure!(
        residuals.iter().all(|v| v.is_finite() && v.abs() < 0.001),
        "economy budget failed: {residuals:?}"
    );
    let budget = if elapsed % 12 == 0 {
        let b = g.ecology.budget(&g.gpu, &g.config)?;
        ensure!(
            b.within_tolerance,
            "ecology budget failed: {:?}",
            b.relative_error
        );
        Some(b)
    } else {
        None
    };
    let wildlife = if elapsed % 12 == 0 {
        Some(g.ecology.wildlife_report(&g.gpu, &g.config)?)
    } else {
        None
    };
    Ok(
        json!({"seed":g.config.seed,"branch":branch,"elapsed_months":elapsed,"history_month":h.month,"ecology_month":g.ecology.clock.month,"sites":sites,"economy_residuals":residuals,"budget":budget,"wildlife":wildlife}),
    )
}
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let months = args
        .get(1)
        .map(|s| s.parse::<u32>())
        .transpose()?
        .unwrap_or(120);
    let seeds = args
        .get(2)
        .map(|s| s.parse::<u32>().map(|s| vec![s]))
        .transpose()?
        .unwrap_or(vec![17, 81, 256]);
    let eco = args
        .get(3)
        .map(|s| s.parse::<u32>())
        .transpose()?
        .unwrap_or(16);
    let yield_scale = args
        .get(4)
        .map(|s| s.parse::<f32>())
        .transpose()?
        .unwrap_or(0.5);
    ensure!(
        (24..=1200).contains(&months) && months % 24 == 0,
        "months must be a multiple of 24 in 24..1200"
    );
    let output =
        std::env::var("FISHERY_OUTPUT").unwrap_or_else(|_| "output/living-fishery-natural".into());
    std::fs::create_dir_all(&output)?;
    let gpu = pollster::block_on(ContextGpu::headless())?;
    for seed in seeds {
        let started = std::time::Instant::now();
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                seed,
                resolution: 64,
                ecology_resolution: eco,
                ecology_years_per_epoch: 1,
                crop_yield_scale: yield_scale,
                ..Default::default()
            },
            Catalog::bundled()?,
        )?;
        g.run_epochs(1)?;
        for _ in 0..120 {
            g.advance_ecology()?;
        }
        g.found_civilizations(16)?;
        g.enable_society()?;
        g.enable_living_history()?;
        // Let receiving-water addresses and household observations establish before branching.
        g.advance_history(12)?;
        if std::env::var("FISHERY_ADAPTIVE").as_deref() == Ok("1") {
            let mut c = g
                .civilizations
                .as_ref()
                .unwrap()
                .economy_catalog
                .clone()
                .unwrap();
            c.agriculture.as_mut().unwrap().fishery.adaptive = true;
            c.agriculture.as_mut().unwrap().fishery.opportunity_cost =
                std::env::var("FISHERY_OPPORTUNITY").as_deref() == Ok("1");
            c.agriculture.as_mut().unwrap().fishery.primitive_gear =
                std::env::var("FISHERY_PRIMITIVE").as_deref() == Ok("1");
            if let Ok(value) = std::env::var("FISHERY_DENSITY_HALF") {
                c.agriculture
                    .as_mut()
                    .unwrap()
                    .fishery
                    .half_saturation_kg_c_m2 = value.parse()?;
            }
            g.configure_economy(c)?;
        }
        let checkpoint = format!("{output}/seed-{seed}.world");
        g.save(&checkpoint)?;
        let mut branches = vec![
            "baseline",
            "sham",
            "removed",
            "restored",
            "closed",
            "removed_closed",
        ];
        if std::env::var("FISHERY_OPPORTUNITY").as_deref() == Ok("1") {
            branches.push("work_ablation");
        }
        std::fs::write(
            format!("{output}/metadata-{seed}.json"),
            serde_json::to_vec_pretty(
                &json!({"branches":branches,"config":g.config,"catalog":g.catalog,"economy_catalog":g.civilizations.as_ref().unwrap().economy_catalog,"months":months,"restore_after_months":months/2,"natural_spinup_months":120,"history_warmup_months":12,"intervention_region":1,"guilds":[8,9,10],"baseline_commit":std::process::Command::new("git").args(["rev-parse","HEAD"]).output().ok().map(|o|String::from_utf8_lossy(&o.stdout).trim().to_owned())}),
            )?,
        )?;
        drop(g);
        for branch in branches {
            let mut g = Generator::load(gpu.clone(), &checkpoint)?;
            let mut file = std::io::BufWriter::new(std::fs::File::create(format!(
                "{output}/{seed}-{branch}.jsonl"
            ))?);
            let mut prior = BTreeMap::new();
            writeln!(file, "{}", sample(&g, branch, 0, &mut prior)?)?;
            if branch.contains("removed") || branch == "restored" {
                for guild in [8, 9, 10] {
                    g.scenario(Some(1), Intervention::RemoveGuild(guild))?;
                }
            } else if branch == "sham" {
                // Clear already-clear masks: same event count, no ecological state change.
                for guild in [8, 9, 10] {
                    g.scenario(Some(1), Intervention::RestoreGuild(guild))?;
                }
            }
            if branch == "work_ablation" {
                let mut c = g
                    .civilizations
                    .as_ref()
                    .unwrap()
                    .economy_catalog
                    .clone()
                    .unwrap();
                c.agriculture.as_mut().unwrap().fishery.opportunity_cost = false;
                g.configure_economy(c)?;
            }
            if branch.contains("closed") {
                let mut c = g
                    .civilizations
                    .as_ref()
                    .unwrap()
                    .economy_catalog
                    .clone()
                    .unwrap();
                c.agriculture.as_mut().unwrap().fisheries_enabled = false;
                g.configure_economy(c)?;
            }
            for elapsed in 1..=months {
                if branch == "restored" && elapsed == months / 2 + 1 {
                    for guild in [8, 9, 10] {
                        g.scenario(Some(1), Intervention::RestoreGuild(guild))?;
                    }
                }
                g.advance_history(1)?;
                writeln!(file, "{}", sample(&g, branch, elapsed, &mut prior)?)?;
                if elapsed % 12 == 0 {
                    file.flush()?;
                }
            }
            file.flush()?;
            g.save(format!("{output}/{seed}-{branch}-final.world"))?;
            eprintln!(
                "seed {seed} {branch}: {months} months completed, {:.1}s cumulative",
                started.elapsed().as_secs_f64()
            );
        }
    }
    Ok(())
}
