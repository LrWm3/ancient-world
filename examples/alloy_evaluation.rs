//! Short natural-seed diagnostic for coupled alloy production and residue accumulation.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::Result;
use serde_json::json;
fn main() -> Result<()> {
    let gpu = pollster::block_on(ContextGpu::headless())?;
    std::fs::create_dir_all("output")?;
    let mut runs = vec![];
    for seed in [17, 81, 256] {
        let mut g = Generator::new(
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
        g.run_epochs(1)?;
        g.found_civilizations(16)?;
        g.enable_society()?;
        g.enable_shared_resources()?;
        g.enable_mineral_processing()?;
        g.enable_alloy_processing()?;
        g.enable_living_history()?;
        g.enable_environmental_returns()?;
        let mut years = vec![];
        for year in 1..=5 {
            g.advance_history(12)?;
            let h = g.civilizations.as_ref().unwrap();
            let budget = g.ecology.budget(&g.gpu, &g.config)?;
            anyhow::ensure!(budget.within_tolerance, "ecological budget failure");
            let sum = |k: usize| {
                h.sites
                    .iter()
                    .map(|s| s.economy.made[k] as f64)
                    .sum::<f64>()
            };
            years.push(json!({"year":year,"copper_kg":sum(38),"copper_tools_kg":sum(43),"copper_scrap_kg":sum(44),"tin_kg":sum(39),"bronze_kg":sum(40),"bronze_tools_kg":sum(41),"iron_tools_kg":sum(3),"bronze_scrap_kg":sum(42),"residue_kg":h.processing_deposits().iter().map(|d|d.kg as f64).sum::<f64>(),"blocked_recipes":h.sites.iter().map(|s|s.economy.residue[3] as f64).sum::<f64>(),"population":h.sites.iter().map(|s|s.stocks.stock[0]).sum::<f32>(),"economy_residuals":h.economy_residuals(),"source_residual":h.resources.as_ref().unwrap().residual(),"ecology_relative_error":budget.relative_error}));
        }
        let h = g.civilizations.as_ref().unwrap();
        let sources: Vec<_> = h
            .resources
            .as_ref()
            .unwrap()
            .sources
            .values()
            .map(|s| json!({"cell":s.cell,"mineral":s.mineral,"extracted":s.extracted}))
            .collect();
        runs.push(json!({"seed":seed,"sources":sources,"years":years}));
        std::fs::write(
            "output/alloy-evaluation.json",
            serde_json::to_vec_pretty(&json!({"complete":false,"runs":runs}))?,
        )?;
        println!("seed {seed} complete");
    }
    std::fs::write(
        "output/alloy-evaluation.json",
        serde_json::to_vec_pretty(&json!({"complete":true,"runs":runs}))?,
    )?;
    Ok(())
}
