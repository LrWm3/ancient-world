//! Count geographic inner-land cells, ignoring settlement eligibility and spacing.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
    grid,
};
use anyhow::Result;
use clap::Parser;
use std::time::Instant;

#[derive(Parser)]
struct Args {
    #[arg(long, value_delimiter = ',', default_value = "17,81,256")]
    seeds: Vec<u32>,
    #[arg(long, value_delimiter = ',', default_value = "256,1024")]
    resolutions: Vec<u32>,
}
fn main() -> Result<()> {
    let args = Args::parse();
    let gpu = pollster::block_on(ContextGpu::headless())?;
    eprintln!("GPU: {}", gpu.adapter_name);
    println!("seed,resolution,planet_cells,inner_land_cells,inner_land_km2,seconds");
    for resolution in args.resolutions {
        for &seed in &args.seeds {
            let start = Instant::now();
            let config = Config {
                seed,
                resolution,
                ..Default::default()
            };
            let generator = Generator::new(gpu.clone(), config, Catalog::bundled()?)?;
            // New performs initialization and coast cleanup. No suitability survey,
            // hydrology evolution, history or candidate ceiling participates here.
            let terrain = generator.snapshot()?;
            let mut count = 0u64;
            let mut solid_angle = 0f64;
            for (i, cell) in terrain.iter().enumerate() {
                if cell.meta[0] == 2 {
                    count += 1;
                    solid_angle += grid::solid_angle(i as u32, resolution);
                }
            }
            let area = solid_angle * f64::from(generator.config.radius_km).powi(2);
            println!(
                "{seed},{resolution},{},{count},{area:.3},{:.3}",
                terrain.len(),
                start.elapsed().as_secs_f64()
            );
        }
    }
    Ok(())
}
