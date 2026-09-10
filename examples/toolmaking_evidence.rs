//! Development comparison, not empirical calibration. Run with an unused output path.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::{ensure, Result};
use serde_json::json;
fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("output JSON path required");
    ensure!(
        !std::path::Path::new(&path).exists(),
        "refusing to overwrite evidence"
    );
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut runs = Vec::new();
    for seed in [17, 81, 256] {
        let mut base = Generator::new(
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
        base.run_epochs(1)?;
        base.found_civilizations(16)?;
        base.enable_society()?;
        base.enable_shared_resources()?;
        base.enable_living_history()?;
        base.advance_history(12)?;
        for id in 0..base.civilizations.as_ref().unwrap().sites.len() as u32 {
            base.set_tool_stock_access(id, 0.)?;
        }
        let checkpoint =
            std::env::temp_dir().join(format!("toolmaking-{}-{seed}.world", std::process::id()));
        base.save(&checkpoint)?;
        for (policy, jobs, skill) in [
            ("food-only", false, false),
            ("replacement-jobs", true, false),
            ("jobs-and-expertise", true, true),
        ] {
            let mut g = Generator::load(gpu.clone(), &checkpoint)?;
            let mut c = g
                .civilizations
                .as_ref()
                .unwrap()
                .economy_catalog
                .clone()
                .unwrap();
            c.production.food_security_labor = true;
            c.production.food_security_maintenance = false;
            c.production.replacement_tool_jobs = jobs;
            c.production.toolmaking_expertise = skill;
            g.configure_economy(c)?;
            let mut months = Vec::new();
            for _ in 0..120 {
                g.advance_history(1)?;
                let h = g.civilizations.as_ref().unwrap();
                let residual = h.economy_residuals();
                ensure!(
                    residual.iter().all(|v| v.is_finite() && v.abs() < 0.001),
                    "budget failed seed {seed}, {policy}, month {}",
                    h.month
                );
                ensure!(h.sites.iter().all(|s| s.economy.valid()), "invalid economy");
                let sites: Vec<_> = h.sites.iter().map(|s| json!({"site":s.id,"population":s.stocks.stock[0],"harvest":s.stocks.stock[2],"unmet_rations":s.demography.ration_need[3]-s.demography.ration_eaten[3],"tools":s.economy.goods[3],"tools_made":s.economy.made[3],"tool_factor":s.economy.production_probe[0],"craft":s.economy.tool_craft,"work":s.economy.tool_work,"labor":s.economy.labor})).collect();
                months.push(json!({"month":h.month,"sites":sites,"residual":residual}));
            }
            eprintln!("seed {seed}, {policy}: completed");
            runs.push(json!({"seed":seed,"policy":policy,"months":months}));
            std::fs::write(
                &path,
                serde_json::to_vec_pretty(
                    &json!({"complete":false,"gpu":gpu.adapter_name,"runs":runs}),
                )?,
            )?;
        }
        std::fs::remove_file(checkpoint)?;
    }
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&json!({"complete":true,"gpu":gpu.adapter_name,"runs":runs}))?,
    )?;
    Ok(())
}
