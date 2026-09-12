//! Capture/replay local lake diagnostics. Raw cell fixtures are not world archives.
//! Keep fixtures under ignored output/; see docs/default-generation-performance.md.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{Cell, ContextGpu, Generator},
    grid,
};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// Local raw cell fixture to read, or write when --capture-epochs is given.
    fixture: PathBuf,
    #[arg(long, default_value_t = 512)]
    resolution: u32,
    #[arg(long, default_value_t = 42)]
    seed: u32,
    /// Capture on completion or error; an unresolved solve still returns an error.
    #[arg(long)]
    capture_epochs: Option<u32>,
    /// Zero uses the automatic resolution-scaled budget.
    #[arg(long, default_value_t = 0)]
    max_lake_iterations: u32,
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config {
        resolution: args.resolution,
        seed: args.seed,
        max_lake_iterations: args.max_lake_iterations,
        ..Default::default()
    };
    config.validate()?;
    let mut g = Generator::new(
        pollster::block_on(ContextGpu::headless())?,
        config,
        Catalog::bundled()?,
    )?;
    if let Some(epochs) = args.capture_epochs {
        anyhow::ensure!(epochs > 0, "capture needs at least one epoch");
        let result = g.run_epochs(epochs);
        let cells = g.snapshot()?;
        if let Some(parent) = args.fixture.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&args.fixture, bytemuck::cast_slice(&cells))?;
        println!(
            "captured epoch={} iterations={} changed={} max_unresolved_change_m={}",
            g.progress.epoch,
            g.progress.lake_iterations,
            g.progress.lake_changed_cells,
            g.progress.lake_max_change_m
        );
        return result;
    }
    let bytes = std::fs::read(&args.fixture)?;
    anyhow::ensure!(
        bytes.len() == g.config.cells() as usize * std::mem::size_of::<Cell>(),
        "fixture size mismatch"
    );
    let cells: Vec<Cell> = bytes
        .chunks_exact(std::mem::size_of::<Cell>())
        .map(bytemuck::pod_read_unaligned)
        .collect();
    g.restore_cells(&cells, 0)?;
    let volume = |cells: &[Cell]| {
        cells
            .iter()
            .enumerate()
            .map(|(i, c)| c.water[0] as f64 * grid::solid_angle(i as u32, args.resolution))
            .sum::<f64>()
    };
    let before = volume(&cells);
    let result = g.equilibrate_lakes();
    let after = g.snapshot()?;
    println!("iterations={} wall_ms={} changed={} max_unresolved_change_m={} relative_volume_error={:.9e}",
        g.progress.lake_iterations, g.progress.stage_ms["secondary_lakes_wall"],
        g.progress.lake_changed_cells, g.progress.lake_max_change_m,
        (volume(&after)-before)/before.max(1e-30));
    // This global residual includes large terminal reservoirs; the focused GPU
    // fixtures separately check small closed pools, saddles and seam transfers.
    result
}
