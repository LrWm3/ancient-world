use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub spatial_world_id: Option<String>,
    pub systems: crate::systems::Systems,
    pub resolution: u32,
    pub ecology_resolution: u32,
    pub ecology_years_per_epoch: u32,
    /// Diagnostic ablation: allow wildlife across all neighboring habitats.
    pub wildlife_open_barriers: bool,
    /// Experimental inherited thermal preferences; aggregate guild identity is unchanged.
    pub wildlife_ecotypes: bool,
    /// Half-response temperature mismatch for aquatic ecotype feeding (toy degrees C).
    pub aquatic_thermal_width_c: f32,
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
    /// Zero selects a resolution-scaled, bounded surface-water relaxation budget.
    pub max_lake_iterations: u32,
    /// Convergence readback interval; changes numerical stopping points, not physical time.
    pub lake_poll_passes: u32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            spatial_world_id: None,
            systems: Default::default(),
            resolution: 512,
            ecology_resolution: 256,
            ecology_years_per_epoch: 10,
            wildlife_open_barriers: false,
            wildlife_ecotypes: false,
            aquatic_thermal_width_c: 15.,
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
            max_lake_iterations: 0,
            lake_poll_passes: 16,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        self.systems.validate()?;
        ensure!(
            matches!(self.lake_poll_passes, 16 | 32 | 64 | 128),
            "lake polling must be 16, 32, 64 or 128 passes"
        );
        ensure!(
            (1. ..=60.).contains(&self.aquatic_thermal_width_c),
            "aquatic thermal width must be finite and within 1..60 degrees C"
        );
        ensure!(
            self.max_lake_iterations == 0
                || ((16..=1048576).contains(&self.max_lake_iterations)
                    && self.max_lake_iterations % 16 == 0),
            "lake iteration limit must be zero (automatic) or a multiple of 16 in 16–1048576"
        );
        ensure!(
            (0.1..=1.).contains(&self.crop_yield_scale),
            "crop yield scale must be 0.1–1"
        );
        ensure!(
            (0.01..=1.).contains(&self.island_phosphorus_scale),
            "inner-continent phosphorus scale must be 0.01–1"
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
    pub fn lake_iteration_limit(&self) -> u32 {
        if self.max_lake_iterations == 0 {
            self.resolution.saturating_mul(self.resolution).max(16384)
        } else {
            self.max_lake_iterations
        }
    }
    pub fn eco_cells(&self) -> u32 {
        6 * self.eco_resolution().pow(2)
    }
    pub fn estimated_bytes(&self) -> u64 {
        // Includes 20 bytes/cell of navigation workspace and a transient 32-byte survey record.
        self.cells() as u64 * (crate::gpu::CELL_BYTES * 2 + 32 + 32 + 52)
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

#[cfg(test)]
mod lake_budget_tests {
    use super::*;
    #[test]
    fn aquatic_width_is_bounded_and_legacy_compatible() {
        let legacy: Config = toml::from_str("seed = 42").unwrap();
        assert_eq!(legacy.aquatic_thermal_width_c, 15.);
        for width in [1., 15., 30., 60., 0., 61., f32::NAN, f32::INFINITY] {
            let c = Config {
                aquatic_thermal_width_c: width,
                ..Default::default()
            };
            assert_eq!(
                c.validate().is_ok(),
                width.is_finite() && (1. ..=60.).contains(&width)
            );
        }
    }
    #[test]
    fn automatic_lake_budget_scales_without_changing_explicit_limits() {
        for (resolution, expected) in [(16, 16384), (256, 65536), (512, 262144), (1024, 1048576)] {
            let config = Config {
                resolution,
                ..Default::default()
            };
            assert_eq!(config.lake_iteration_limit(), expected);
        }
        let mut config = Config {
            max_lake_iterations: 32,
            ..Default::default()
        };
        assert_eq!(config.lake_iteration_limit(), 32);
        assert!(config.validate().is_ok());
        config.max_lake_iterations = 17;
        assert!(config.validate().is_err());
        let legacy: Config = toml::from_str("seed = 42").unwrap();
        assert_eq!(legacy.max_lake_iterations, 0);
        assert_eq!(legacy.lake_poll_passes, 16);
        for interval in [0, 15, 16, 32, 64, 128, 256] {
            let config = Config {
                lake_poll_passes: interval,
                ..Default::default()
            };
            assert_eq!(
                config.validate().is_ok(),
                matches!(interval, 16 | 32 | 64 | 128)
            );
        }
    }
}
