use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub resolution: u32,
    pub ecology_resolution: u32,
    pub ecology_years_per_epoch: u32,
    /// Diagnostic ablation: allow wildlife across all neighboring habitats.
    pub wildlife_open_barriers: bool,
    pub lake_mixing: f32,
    pub solar_scale: f32,
    #[serde(default = "legacy_unit_scale")]
    pub island_phosphorus_scale: f32,
    #[serde(default = "legacy_plot_hectares")]
    pub settlement_plot_hectares: f32,
    #[serde(default = "legacy_unit_scale")]
    pub crop_yield_scale: f32,
    pub seed: u32,
    pub radius_km: f32,
    pub axial_tilt: f32,
    pub inner_continents: u32,
    pub target_epochs: u32,
    pub geological_step_myr: f32,
    pub climate_iterations: u32,
    pub climate_cycles: u32,
    pub max_drainage_iterations: u32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            resolution: 512,
            ecology_resolution: 256,
            ecology_years_per_epoch: 10,
            wildlife_open_barriers: false,
            lake_mixing: 1.,
            solar_scale: 1.,
            island_phosphorus_scale: 0.25,
            settlement_plot_hectares: 160.,
            crop_yield_scale: 0.5,
            seed: 42,
            radius_km: 6371.,
            axial_tilt: 23.44,
            inner_continents: 5,
            target_epochs: 1000,
            geological_step_myr: 0.01,
            climate_iterations: 24,
            climate_cycles: 32,
            max_drainage_iterations: 16384,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (0.1..=1.).contains(&self.crop_yield_scale),
            "crop yield scale must be 0.1–1"
        );
        ensure!(
            (0.01..=1.).contains(&self.island_phosphorus_scale),
            "island phosphorus scale must be 0.01–1"
        );
        ensure!(
            (20. ..=5000.).contains(&self.settlement_plot_hectares),
            "settlement plot must be 20–5000 hectares"
        );
        ensure!(
            self.resolution.is_power_of_two() && (8..=1024).contains(&self.resolution),
            "resolution must be a power of two from 8 to 1024 (8–128 are diagnostic sizes)"
        );
        ensure!(
            self.radius_km.is_finite() && (100.0..=100000.0).contains(&self.radius_km),
            "radius must be 100–100000 km"
        );
        ensure!(
            self.axial_tilt.is_finite() && (0.0..=90.0).contains(&self.axial_tilt),
            "axial tilt must be 0–90 degrees"
        );
        ensure!(
            (2..=8).contains(&self.inner_continents),
            "choose 2–8 inner continents"
        );
        ensure!(
            self.geological_step_myr.is_finite()
                && (0.00001..=1.0).contains(&self.geological_step_myr),
            "geological step must be 0.00001–1 Myr"
        );
        ensure!(
            (4..=4096).contains(&self.climate_iterations),
            "climate iterations must be 4–4096"
        );
        ensure!(
            (1..=64).contains(&self.climate_cycles),
            "climate cycles must be 1–64"
        );
        ensure!(
            (16..=1000000).contains(&self.max_drainage_iterations),
            "drainage iteration limit must be 16–1000000"
        );
        ensure!(
            self.ecology_resolution.is_power_of_two()
                && (8..=1024).contains(&self.ecology_resolution),
            "invalid ecology resolution"
        );
        ensure!(self.ecology_resolution <= self.resolution || (self.ecology_resolution == 256 && self.resolution < 256), "ecology resolution cannot exceed terrain resolution; only the default 256 ecology grid automatically follows smaller diagnostic terrain grids");
        ensure!(
            (1..=1000).contains(&self.ecology_years_per_epoch),
            "ecological years must be 1–1000"
        );
        ensure!(
            self.lake_mixing.is_finite()
                && (0.0..=20.0).contains(&self.lake_mixing)
                && self.solar_scale.is_finite()
                && (0.0..=4.0).contains(&self.solar_scale),
            "invalid ecological forcing"
        );
        Ok(())
    }
    pub fn cells(&self) -> u32 {
        6 * self.resolution * self.resolution
    }
    pub fn eco_resolution(&self) -> u32 {
        self.ecology_resolution.min(self.resolution)
    }
    pub fn eco_cells(&self) -> u32 {
        6 * self.eco_resolution().pow(2)
    }
    pub fn estimated_bytes(&self) -> u64 {
        self.cells() as u64 * (crate::gpu::CELL_BYTES * 2 + 32 + 32)
            + self.eco_cells() as u64
                * (crate::ecology::ECO_BYTES * 2 + crate::ecology::ENVIRONMENT_BYTES)
            + 32 * 1024 * 1024
    }
}

fn legacy_unit_scale() -> f32 {
    1.
}
fn legacy_plot_hectares() -> f32 {
    5000.
}
