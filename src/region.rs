//! On-demand GPU regional terrain. Results are a derived snapshot, not a second planet ledger.
use crate::gpu::{read_buffer, Generator};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use wgpu::util::DeviceExt;
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RegionalCell {
    /// Bed elevation m, sediment m, soil depth m, spherical area m².
    pub surface: [f32; 4],
    /// Temperature °C, precipitation mm/year, fertility, cover.
    pub climate: [f32; 4],
    /// Water depth m, routing spill m, annual outflow m³, local annual runoff m³.
    pub water: [f32; 4],
    /// Rock, soil, plant (u32::MAX if absent), planet region.
    pub ids: [u32; 4],
    /// Downstream local cell, decreasing rank, parent planet cell, inherited water outlet.
    pub route: [u32; 4],
    /// Initial water volume m³, mineral index + 1 (0 absent), inherited potential, target depth m.
    pub forcing: [f32; 4],
    /// Inherited parent column: three thicknesses and cumulative removed bedrock, m.
    #[serde(default)]
    pub strata: [f32; 4],
    #[serde(default)]
    pub rocks: [u32; 4],
}
impl RegionalCell {
    /// Inherited prospect only; regional patches never create extraction reserves.
    pub fn mineral_index(&self) -> Option<usize> {
        (self.forcing[1] >= 1.).then(|| self.forcing[1] as usize - 1)
    }
}
#[derive(Serialize, Deserialize)]
pub struct Region {
    #[serde(default)]
    pub spatial: Option<crate::spatial::SurveyRef>,
    /// Canonical site/object snapshots, localized only to their parent cells.
    #[serde(default)]
    pub historical_sites: Vec<crate::civilization::Site>,
    #[serde(default)]
    pub historical_artifacts: Vec<crate::culture::Artifact>,
    #[serde(default)]
    pub regional_mines: Vec<crate::regional_mining::RegionalMine>,
    #[serde(default)]
    pub processing_deposits: Vec<crate::metallurgy::ResidueDeposit>,
    /// Read-only canonical accessible sources at this history boundary.
    #[serde(default)]
    pub resource_sources: Vec<crate::resources::Source>,
    #[serde(default)]
    pub history_month: Option<u32>,
    pub version: u32,
    pub seed: u32,
    pub epoch: u32,
    pub ecological_month: u64,
    pub catalog: crate::catalog::Catalog,
    pub resolution: u32,
    pub width_km: f32,
    pub center: [f32; 3],
    pub drainage_iterations: u32,
    pub flow_iterations: u32,
    pub pool_iterations: u32,
    pub cells: Vec<RegionalCell>,
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    dims: [u32; 4],
    center: [f32; 4],
    east: [f32; 4],
    north: [f32; 4],
    counts: [u32; 4],
}
impl Generator {
    pub fn generate_region(
        &self,
        center: [f32; 3],
        width_km: f32,
        resolution: u32,
    ) -> Result<Region> {
        ensure!(
            resolution.is_power_of_two() && (16..=1024).contains(&resolution),
            "regional resolution must be 16–1024, power of two"
        );
        ensure!(
            width_km.is_finite() && width_km >= 1. && width_km <= self.config.radius_km * 0.5,
            "regional extent must be 1 km to half the planet radius"
        );
        let length = center.iter().map(|x| x * x).sum::<f32>().sqrt();
        ensure!(length.is_finite() && length > 0., "invalid regional center");
        let center = center.map(|x| x / length);
        let up = if center[1].abs() > 0.95 {
            [1., 0., 0.]
        } else {
            [0., 1., 0.]
        };
        let cross = |a: [f32; 3], b: [f32; 3]| {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let mut east = cross(up, center);
        let norm = east.iter().map(|x| x * x).sum::<f32>().sqrt();
        east = east.map(|x| x / norm);
        let north = cross(center, east);
        let d = &self.gpu.device;
        let size =
            resolution as u64 * resolution as u64 * std::mem::size_of::<RegionalCell>() as u64;
        ensure!(
            size <= d.limits().max_storage_buffer_binding_size as u64
                && self.config.estimated_bytes() + size * 2 < 4 * 1024 * 1024 * 1024,
            "regional buffers exceed GPU memory budget"
        );
        let u = Uniforms {
            dims: [resolution, self.config.resolution, self.config.seed, 0],
            center: [center[0], center[1], center[2], width_km * 1000.],
            east: [east[0], east[1], east[2], self.config.radius_km * 1000.],
            north: [north[0], north[1], north[2], 0.],
            counts: self.catalog.counts(),
        };
        let uniform = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Regional settings"),
            contents: bytemuck::bytes_of(&u),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let buffers: [wgpu::Buffer; 2] = std::array::from_fn(|_| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Regional fields"),
                size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            })
        });
        let flags = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let entries = (0..6)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: if binding == 3 {
                        wgpu::BufferBindingType::Uniform
                    } else {
                        wgpu::BufferBindingType::Storage {
                            read_only: matches!(binding, 0 | 1 | 5),
                        }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect::<Vec<_>>();
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });
        let groups: [wgpu::BindGroup; 2] = std::array::from_fn(|i| {
            let resources = [
                &self.buffers[self.current],
                &buffers[i],
                &buffers[1 - i],
                &uniform,
                &flags,
                &self.catalog_buffer,
            ];
            d.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &resources
                    .iter()
                    .enumerate()
                    .map(|(binding, b)| wgpu::BindGroupEntry {
                        binding: binding as u32,
                        resource: b.as_entire_binding(),
                    })
                    .collect::<Vec<_>>(),
            })
        });
        let layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        d.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Regional simulation"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/region_compute.wgsl").into()),
        });
        let mut side = 0;
        let mut iterations = [0; 3];
        for (stage, name) in ["generate", "drainage", "flow", "pools", "habitat"]
            .iter()
            .enumerate()
        {
            let pipeline = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                module: &module,
                entry_point: Some(name),
                compilation_options: Default::default(),
                cache: None,
            });
            if stage == 0 {
                if let Some(error) = pollster::block_on(d.pop_error_scope()) {
                    anyhow::bail!("regional GPU pipeline: {error}");
                }
            }
            let iterative = (1..=3).contains(&stage);
            let limit = if iterative {
                resolution * resolution.min(64)
            } else {
                1
            };
            let mut converged = !iterative;
            for iteration in 0..limit {
                let mut encoder = d.create_command_encoder(&Default::default());
                encoder.clear_buffer(&flags, 0, None);
                {
                    let mut pass = encoder.begin_compute_pass(&Default::default());
                    pass.set_pipeline(&pipeline);
                    pass.set_bind_group(0, &groups[side], &[]);
                    pass.dispatch_workgroups(resolution / 8, resolution / 8, 1);
                }
                self.gpu.queue.submit(Some(encoder.finish()));
                side = 1 - side;
                if iterative {
                    iterations[stage - 1] = iteration + 1;
                    let result = read_buffer(&self.gpu, &flags, 0, 4)?;
                    if u32::from_le_bytes(result.try_into().unwrap()) == 0 {
                        converged = true;
                        break;
                    }
                }
            }
            ensure!(
                converged,
                "regional {name} failed to converge after {limit} iterations"
            );
        }
        let bytes = read_buffer(&self.gpu, &buffers[side], 0, size)?;
        let cells = bytemuck::cast_slice::<u8, RegionalCell>(&bytes).to_vec();
        ensure!(
            cells.iter().all(|c| c
                .surface
                .iter()
                .chain(c.climate.iter())
                .chain(c.water.iter())
                .all(|v| v.is_finite())
                && c.water[0] >= 0.),
            "invalid regional state"
        );
        let parents: std::collections::BTreeSet<_> = cells.iter().map(|c| c.route[2]).collect();
        let resource_sources = self
            .civilizations
            .as_ref()
            .and_then(|h| h.resources.as_ref())
            .map_or_else(Vec::new, |r| {
                r.sources
                    .values()
                    .filter(|s| parents.contains(&s.cell))
                    .cloned()
                    .collect()
            });
        Ok(Region {
            spatial: Some(crate::spatial::SurveyRef {
                id: crate::spatial::new_world_id()
                    .bytes()
                    .fold(0xcbf29ce484222325u64, |hash, byte| {
                        (hash ^ byte as u64).wrapping_mul(0x100000001b3)
                    }),
                grid: crate::spatial::GridRef {
                    world: self
                        .config
                        .spatial_world_id
                        .clone()
                        .expect("generator world identity"),
                    resolution: self.config.resolution,
                },
                radius_m: self.config.radius_km as f64 * 1000.,
            }),
            historical_sites: self.civilizations.as_ref().map_or_else(Vec::new, |h| {
                h.sites
                    .iter()
                    .filter(|s| parents.contains(&s.cell))
                    .cloned()
                    .collect()
            }),
            historical_artifacts: self.civilizations.as_ref().map_or_else(Vec::new, |h| {
                h.culture.as_ref().map_or_else(Vec::new, |c| {
                    c.artifacts
                        .iter()
                        .filter(|a| {
                            a.site
                                .is_some_and(|s| parents.contains(&h.sites[s as usize].cell))
                        })
                        .cloned()
                        .collect()
                })
            }),
            regional_mines: self
                .civilizations
                .as_ref()
                .and_then(|h| h.resources.as_ref())
                .map_or_else(Vec::new, |r| {
                    r.regional_mines
                        .values()
                        .chain(r.retired_regional_mines.iter())
                        .filter(|m| parents.contains(&m.cell))
                        .cloned()
                        .collect()
                }),
            processing_deposits: self.civilizations.as_ref().map_or_else(Vec::new, |h| {
                h.processing_deposits()
                    .into_iter()
                    .filter(|d| parents.contains(&d.cell))
                    .collect()
            }),
            resource_sources,
            history_month: self.civilizations.as_ref().map(|h| h.month),
            version: 3,
            seed: self.config.seed,
            epoch: self.progress.epoch,
            ecological_month: self.ecology.clock.month,
            catalog: self.catalog.clone(),
            resolution,
            width_km,
            center,
            drainage_iterations: iterations[0],
            flow_iterations: iterations[1],
            pool_iterations: iterations[2],
            cells,
        })
    }
}
impl Region {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, serde_json::to_vec(self)?)?;
        Ok(())
    }
    pub fn image(&self) -> image::RgbImage {
        let n = self.resolution;
        let mut image = image::RgbImage::new(n, n);
        for (i, c) in self.cells.iter().enumerate() {
            let mut color = if c.water[0] > 0.5 {
                [25., 85., 125.]
            } else if c.water[2] > 1e7 {
                [40., 110., 140.]
            } else {
                let v = c.climate[3];
                [135. - 90. * v, 115. + 25. * v, 75. - 10. * v]
            };
            let other = &self.cells[if i % n as usize > 0 { i - 1 } else { i }];
            let shade = (1. + (c.surface[0] - other.surface[0]) / c.surface[3].sqrt() * 0.8)
                .clamp(0.55, 1.4);
            if c.climate[0] < -2. && c.water[0] < 0.05 {
                color = [220., 230., 235.];
            }
            image.put_pixel(
                i as u32 % n,
                n - 1 - i as u32 / n,
                image::Rgb(color.map(|x| (x * shade).clamp(0., 255.) as u8)),
            );
        }
        image
    }
    pub fn export_png(&self, path: impl AsRef<Path>) -> Result<()> {
        self.image().save(path)?;
        Ok(())
    }
}
