//! Export a generated regional surface centered on a dry mountain cell.
use ancient_world::{
    gpu::{ContextGpu, Generator},
    grid,
    viewer::MapRenderer,
};
use anyhow::{Context, Result};
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let source = args
        .next()
        .context("usage: region WORLD OUTPUT.png [CELL_ID] [WIDTH_KM]")?;
    let output = args.next().context("missing output PNG path")?;
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let g = Generator::load(gpu, source)?;
    let cell = if let Some(id) = args.next() {
        id.parse()?
    } else {
        g.snapshot()?
            .iter()
            .enumerate()
            .filter(|(_, c)| c.meta[0] >= 2 && c.water[0] < 0.1)
            .max_by(|(_, a), (_, b)| a.geology[0].total_cmp(&b.geology[0]))
            .context("no dry terrain")?
            .0 as u32
    };
    anyhow::ensure!(cell < g.config.cells(), "cell index outside world");
    let width = args.next().unwrap_or("600".into()).parse()?;
    let renderer = MapRenderer::new(&g, 1024, 1024)?;
    renderer.export_region(
        &g,
        grid::cell_direction(cell, g.config.resolution),
        width,
        output,
    )?;
    println!("Regional surface centered on cell {cell}, requested width {width} km");
    Ok(())
}
