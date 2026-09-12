use crate::{catalog::Catalog, config::Config};
use anyhow::{anyhow, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::mpsc, time::Instant};
use wgpu::util::DeviceExt;

pub const CELL_BYTES: u64 = std::mem::size_of::<Cell>() as u64;
pub const NONE: u32 = u32::MAX;
#[repr(C)]
#[derive(
    Clone, Copy, Debug, Default, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable,
)]
pub struct Cell {
    pub terrain: [f32; 4],
    pub climate: [f32; 4],
    pub water: [f32; 4],
    pub life: [f32; 4],
    pub geology: [f32; 4],
    pub hydro: [f32; 4],
    pub ids: [u32; 4],
    pub routing: [u32; 4],
    pub meta: [u32; 4],
    pub budget: [f32; 4],
    /// Top, middle, basement thickness m; cumulative bedrock removed m. Zero depths mean unavailable.
    #[serde(default)]
    pub strata: [f32; 4],
}
impl Cell {
    /// Rock at depth below the top of bedrock (loose sediment is excluded).
    /// Returns None for an unavailable/exhausted column or a depth beyond its base.
    pub fn rock_at_depth(&self, depth_m: f32) -> Option<u32> {
        if !depth_m.is_finite() || depth_m < 0. {
            return None;
        }
        let mut bottom = 0.;
        for (rock, thickness) in [self.ids[0], self.meta[2], self.meta[3]]
            .into_iter()
            .zip(self.strata[..3].iter())
        {
            bottom += thickness;
            if depth_m < bottom {
                return Some(rock);
            }
        }
        None
    }
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    dims: [u32; 4],
    physical: [f32; 4],
    counts: [u32; 4],
    aux: [u32; 4],
    tuning: [f32; 4],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    Boundary,
    Tectonics,
    DrainInit,
    Drainage,
    BasinInit,
    Basins,
    ClimateStart,
    Climate,
    ClimateCheck,
    FlowInit,
    Flow,
    LakeCollect,
    LakeReduce,
    LakeUpdate,
    Water,
    Ecology,
}
impl Stage {
    pub fn label(self) -> &'static str {
        match self {
            Self::Boundary => "Epoch complete",
            Self::Tectonics => "Tectonic evolution",
            Self::DrainInit => "Preparing drainage",
            Self::Drainage => "Converging drainage",
            Self::BasinInit => "Finding lakes",
            Self::Basins => "Labeling lake basins",
            Self::ClimateStart => "Starting climate cycle",
            Self::Climate => "Seasonal climate",
            Self::ClimateCheck => "Checking climate convergence",
            Self::FlowInit => "Runoff",
            Self::Flow => "Accumulating rivers",
            Self::LakeCollect => "Lake water balance",
            Self::LakeReduce => "Lake volume reduction",
            Self::LakeUpdate => "Updating inland sea",
            Self::Water => "Water and erosion",
            Self::Ecology => "Ecological succession",
        }
    }
    fn kernel(self) -> &'static str {
        match self {
            Self::Boundary => "initialize",
            Self::Tectonics => "tectonics",
            Self::DrainInit => "drain_init",
            Self::Drainage => "drain_relax",
            Self::BasinInit => "basin_init",
            Self::Basins => "basin_relax",
            Self::ClimateStart => "climate_start",
            Self::Climate => "climate",
            Self::ClimateCheck => "climate_check",
            Self::FlowInit => "flow_init",
            Self::Flow => "flow_relax",
            Self::LakeCollect => "lake_collect",
            Self::LakeReduce => "lake_reduce",
            Self::LakeUpdate => "lake_update",
            Self::Water => "water_erosion",
            Self::Ecology => "ecology",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub epoch: u32,
    pub geological_time_myr: f64,
    pub stage: Stage,
    pub iteration: u32,
    pub changed: u32,
    #[serde(default)]
    pub climate_cycles: u32,
    #[serde(default)]
    pub lake_iterations: u32,
    #[serde(default)]
    pub lake_changed_cells: u32,
    #[serde(default)]
    pub lake_max_change_m: f32,
    #[serde(default)]
    pub climate_converged: bool,
    pub stage_ms: BTreeMap<String, f64>,
    pub timestamp_supported: bool,
}
#[derive(Clone)]
pub struct ContextGpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_name: String,
    pub adapter_info: wgpu::AdapterInfo,
}
impl ContextGpu {
    pub async fn headless() -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let adapter=instance.request_adapter(&wgpu::RequestAdapterOptions{power_preference:wgpu::PowerPreference::HighPerformance,force_fallback_adapter:false,compatible_surface:None}).await.context("No compute-capable GPU available; CPU simulation fallback is intentionally disabled")?;
        ensure!(
            adapter.get_info().device_type != wgpu::DeviceType::Cpu,
            "Software GPU adapter rejected; hardware compute is required"
        );
        let desc = device_descriptor(&adapter);
        let (device, queue) = adapter
            .request_device(&desc, None)
            .await
            .context("requesting GPU device")?;
        Ok(Self {
            device,
            queue,
            adapter_name: adapter.get_info().name,
            adapter_info: adapter.get_info(),
        })
    }
}
pub fn device_descriptor(adapter: &wgpu::Adapter) -> wgpu::DeviceDescriptor<'static> {
    let supported = adapter.limits();
    let limits = wgpu::Limits {
        max_storage_buffer_binding_size: supported.max_storage_buffer_binding_size,
        max_buffer_size: supported.max_buffer_size,
        ..Default::default()
    };
    wgpu::DeviceDescriptor {
        label: Some("Ancient World GPU"),
        required_features: adapter.features() & wgpu::Features::TIMESTAMP_QUERY,
        required_limits: limits,
        memory_hints: wgpu::MemoryHints::MemoryUsage,
    }
}
pub struct Generator {
    pub(crate) navigation: Option<std::sync::Arc<crate::navigation::Navigation>>,
    pub(crate) navigation_mode: crate::navigation::NavigationMode,
    pub(crate) history_environment: Option<crate::history_environment::HistoryEnvironment>,
    pub(crate) history_readback_mode: crate::history_environment::HistoryReadbackMode,
    pub(crate) history_readback_stats: crate::history_environment::HistoryReadbackStats,
    #[cfg(test)]
    pub(crate) terrain_snapshot_count: std::sync::atomic::AtomicU64,
    pub(crate) return_pipeline: Option<wgpu::ComputePipeline>,
    pub(crate) history_engine: Option<crate::civilization::Engine>,
    pub civilizations: Option<crate::civilization::History>,
    pub gpu: ContextGpu,
    pub config: Config,
    pub catalog: Catalog,
    pub progress: Progress,
    pub(crate) buffers: [wgpu::Buffer; 2],
    pub(crate) current: usize,
    pub(crate) catalog_buffer: wgpu::Buffer,
    uniform: wgpu::Buffer,
    flags: wgpu::Buffer,
    groups: [wgpu::BindGroup; 2],
    pipelines: BTreeMap<String, wgpu::ComputePipeline>,
    query: Option<wgpu::QuerySet>,
    query_buffer: wgpu::Buffer,
    pub error: Option<String>,
    pub ecology: crate::ecology::Ecology,
    pub(crate) planet: wgpu::Buffer,
    reduction_count: u32,
    reduction_side: u32,
}
impl Generator {
    pub fn new(gpu: ContextGpu, mut config: Config, catalog: Catalog) -> Result<Self> {
        config.validate()?;
        config
            .spatial_world_id
            .get_or_insert_with(crate::spatial::new_world_id);
        catalog.validate()?;
        let size = config.cells() as u64 * std::mem::size_of::<Cell>() as u64;
        ensure!(size<=gpu.device.limits().max_storage_buffer_binding_size as u64 && size<=gpu.device.limits().max_buffer_size,"{}² per face needs a {} MiB storage binding; this GPU exposes only {} MiB. Select a lower resolution.",config.resolution,size/1048576,gpu.device.limits().max_storage_buffer_binding_size/1048576);
        // A conservative user-visible budget, independent of optional vendor memory queries.
        ensure!(
            config.estimated_bytes() <= 4 * 1024 * 1024 * 1024,
            "estimated simulation allocation exceeds 4 GiB safety budget"
        );
        let d = &gpu.device;
        d.push_error_scope(wgpu::ErrorFilter::Validation);
        d.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let buffers = std::array::from_fn(|_| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Planet fields"),
                size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        let uniform = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Simulation settings"),
            size: 80,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let flags = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Convergence flags"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let catalog_buffer = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material tables"),
            contents: bytemuck::cast_slice(&catalog.entries()),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let scratch = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Deterministic lake reduction"),
            size: config.cells() as u64 * 32,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let planet = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Planet reservoir state"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let entries = (0..7)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: if binding == 2 {
                        wgpu::BufferBindingType::Uniform
                    } else {
                        wgpu::BufferBindingType::Storage {
                            read_only: binding == 0 || binding == 4,
                        }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect::<Vec<_>>();
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Simulation bindings"),
            entries: &entries,
        });
        let groups = std::array::from_fn(|i| {
            d.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Planet ping pong"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: buffers[1 - i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: flags.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: catalog_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: scratch.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: planet.as_entire_binding(),
                    },
                ],
            })
        });
        let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Planet simulation"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/simulation.wgsl").into()),
        });
        let pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let mut pipelines = BTreeMap::new();
        for name in [
            "pool_init",
            "pool_even",
            "pool_odd",
            "pool_scatter",
            "initialize",
            "tectonics",
            "drain_init",
            "drain_even",
            "drain_odd",
            "drain_scatter",
            "basin_init",
            "basin_relax",
            "climate",
            "flow_init",
            "flow_relax",
            "water_erosion",
            "ecology",
            "lake_collect",
            "lake_reduce",
            "lake_update",
            "coast_cleanup",
            "climate_start",
            "climate_check",
        ] {
            pipelines.insert(
                name.into(),
                d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(name),
                    layout: Some(&pl),
                    module: &module,
                    entry_point: Some(name),
                    compilation_options: Default::default(),
                    cache: None,
                }),
            );
        }
        let timestamp = d.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let query = timestamp.then(|| {
            d.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Stage timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: 16,
            })
        });
        let query_buffer = d.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 256,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let oom = pollster::block_on(d.pop_error_scope());
        let validation = pollster::block_on(d.pop_error_scope());
        if let Some(e) = oom.or(validation) {
            return Err(anyhow!("GPU setup failed: {e}"));
        }
        let ecology = crate::ecology::Ecology::new(&gpu, &config)?;
        let mut s = Self {
            #[cfg(test)]
            terrain_snapshot_count: Default::default(),
            navigation: None,
            navigation_mode: Default::default(),
            history_environment: None,
            history_readback_mode: Default::default(),
            history_readback_stats: Default::default(),
            civilizations: None,
            history_engine: None,
            return_pipeline: None,
            ecology,
            gpu,
            config,
            catalog,
            progress: Progress {
                epoch: 0,
                geological_time_myr: 0.,
                stage: Stage::Boundary,
                iteration: 0,
                changed: 0,
                climate_cycles: 0,
                lake_iterations: 0,
                lake_changed_cells: 0,
                lake_max_change_m: 0.,
                climate_converged: false,
                stage_ms: BTreeMap::new(),
                timestamp_supported: timestamp,
            },
            buffers,
            current: 0,
            catalog_buffer,
            uniform,
            flags,
            groups,
            pipelines,
            query,
            query_buffer,
            error: None,
            planet,
            reduction_count: 0,
            reduction_side: 0,
        };
        s.dispatch("initialize", 1)?;
        s.dispatch("coast_cleanup", 8)?;
        Ok(s)
    }
    fn params(&self) -> Params {
        Params {
            dims: [
                self.config.resolution,
                self.config.seed,
                self.progress.epoch,
                self.config.inner_continents,
            ],
            physical: [
                self.config.radius_km,
                self.config.axial_tilt,
                self.config.geological_step_myr,
                self.progress.iteration as f32 / self.config.climate_iterations as f32,
            ],
            counts: self.catalog.counts(),
            aux: [
                self.catalog.biomes.len() as u32,
                self.progress.iteration,
                self.reduction_count,
                self.reduction_side,
            ],
            tuning: [
                120.,
                1.,
                self.progress.geological_time_myr as f32,
                if self.catalog.geological_provinces {
                    if self.catalog.process_geology {
                        2.
                    } else {
                        1.
                    }
                } else {
                    0.
                },
            ],
        }
    }
    fn dispatch(&mut self, name: &str, batch: u32) -> Result<u32> {
        let start = Instant::now();
        let d = &self.gpu.device;
        let q = &self.gpu.queue;
        q.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&self.params()));
        let read = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Small diagnostics readback"),
            size: 16 + 16 * batch as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            d.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(name) });
        for pass_index in 0..batch {
            encoder.clear_buffer(&self.flags, 0, None);
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(name),
                timestamp_writes: self.query.as_ref().map(|query_set| {
                    wgpu::ComputePassTimestampWrites {
                        query_set,
                        beginning_of_pass_write_index: Some(pass_index * 2),
                        end_of_pass_write_index: Some(pass_index * 2 + 1),
                    }
                }),
            });
            let kernel = if name == "drain_relax" {
                if (self.progress.iteration + pass_index) % 2 == 0 {
                    "drain_even"
                } else {
                    "drain_odd"
                }
            } else {
                name
            };
            pass.set_pipeline(&self.pipelines[kernel]);
            pass.set_bind_group(0, &self.groups[self.current], &[]);
            if name == "lake_reduce" {
                pass.dispatch_workgroups(self.reduction_count.div_ceil(256), 1, 1);
            } else {
                pass.dispatch_workgroups(self.config.resolution / 8, self.config.resolution / 8, 6);
            }
            drop(pass);
            if name != "lake_reduce" && name != "drain_relax" {
                self.current = 1 - self.current;
            }
        }
        encoder.copy_buffer_to_buffer(&self.flags, 0, &read, 0, 16);
        if let Some(query) = &self.query {
            encoder.resolve_query_set(query, 0..batch * 2, &self.query_buffer, 0);
            encoder.copy_buffer_to_buffer(&self.query_buffer, 0, &read, 16, 16 * batch as u64);
        }
        q.submit(Some(encoder.finish()));
        let data = map_buffer(d, &read)?;
        let changed = u32::from_le_bytes(data[0..4].try_into()?);
        let ms = if self.query.is_some() {
            let ticks: u64 = data[16..]
                .chunks_exact(16)
                .map(|pair| {
                    let a = u64::from_le_bytes(pair[..8].try_into().unwrap());
                    let b = u64::from_le_bytes(pair[8..].try_into().unwrap());
                    b.saturating_sub(a)
                })
                .sum();
            ticks as f64 * self.gpu.queue.get_timestamp_period() as f64 / 1e6
        } else {
            start.elapsed().as_secs_f64() * 1000.
        };
        *self.progress.stage_ms.entry(name.into()).or_default() += ms;
        Ok(changed)
    }
    /// Relax connected water through actual neighboring saddles, with conservative
    /// area-weighted transfers. Never shares water across a dry internal ridge.
    pub fn equilibrate_lakes(&mut self) -> Result<()> {
        self.history_environment = None;
        self.navigation = None;
        let start = Instant::now();
        self.gpu
            .queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&self.params()));
        let mut init = self.gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = init.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipelines["pool_init"]);
            pass.set_bind_group(0, &self.groups[self.current], &[]);
            pass.dispatch_workgroups(self.config.resolution / 8, self.config.resolution / 8, 6);
        }
        self.gpu.queue.submit(Some(init.finish()));
        // Reuse one tiny staging allocation throughout this solve. The flag copy
        // joins the existing batch submission; map_buffer unmaps before reuse.
        let readback = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lake convergence readback"),
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut converged = false;
        let mut iterations = 0;
        let limit = self.config.lake_iteration_limit();
        for _ in 0..limit / 16 {
            let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
            for step in 0..16 {
                encoder.clear_buffer(&self.flags, 0, None);
                let mut pass = encoder.begin_compute_pass(&Default::default());
                pass.set_pipeline(
                    &self.pipelines[if step % 2 == 0 {
                        "pool_even"
                    } else {
                        "pool_odd"
                    }],
                );
                pass.set_bind_group(0, &self.groups[self.current], &[]);
                pass.dispatch_workgroups(self.config.resolution / 8, self.config.resolution / 8, 6);
                iterations += 1;
            }
            encoder.copy_buffer_to_buffer(&self.flags, 0, &readback, 0, 16);
            self.gpu.queue.submit(Some(encoder.finish()));
            let flags = map_buffer(&self.gpu.device, &readback)?;
            self.progress.lake_changed_cells = u32::from_le_bytes(flags[..4].try_into()?);
            self.progress.lake_max_change_m = f32::from_le_bytes(flags[4..8].try_into()?);
            if u32::from_le_bytes(flags[..4].try_into()?) == 0 {
                converged = true;
                break;
            }
        }
        // Every batch has an even number of passes: the completed state is side zero.
        let mut scatter = self.gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = scatter.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipelines["pool_scatter"]);
            pass.set_bind_group(0, &self.groups[self.current], &[]);
            pass.dispatch_workgroups(self.config.resolution / 8, self.config.resolution / 8, 6);
        }
        self.gpu.queue.submit(Some(scatter.finish()));
        self.current = 1 - self.current;
        *self
            .progress
            .stage_ms
            .entry("secondary_lakes_wall".into())
            .or_default() += start.elapsed().as_secs_f64() * 1000.;
        self.progress.lake_iterations = iterations;
        if !converged {
            let message =
                format!("secondary lake surface flow unresolved after {iterations} iterations: {} cells still changing, maximum change {} m", self.progress.lake_changed_cells, self.progress.lake_max_change_m);
            self.error = Some(message.clone());
            anyhow::bail!(message);
        }
        Ok(())
    }
    /// Advance a bounded dispatch batch. Returns true at an epoch boundary.
    pub fn advance(&mut self) -> Result<bool> {
        self.history_environment = None;
        self.navigation = None;
        ensure!(self.civilizations.is_none(), "Geological epochs are locked after founding; enable living history for monthly environmental evolution");
        if let Some(e) = &self.error {
            return Err(anyhow!(e.clone()));
        }
        if self.progress.stage == Stage::Boundary {
            ensure!(self.progress.epoch < u32::MAX, "epoch counter exhausted");
            self.progress.climate_cycles = 0;
            self.progress.climate_converged = false;
            self.progress.stage = Stage::Tectonics;
            self.progress.iteration = 0;
        }
        let stage = self.progress.stage;
        if stage == Stage::Ecology {
            if self.ecology.clock.epoch_month == 0 {
                self.ecology.prepare(
                    &self.gpu,
                    &self.buffers[self.current],
                    &self.catalog_buffer,
                    &self.config,
                    &self.catalog,
                    true,
                );
            }
            let start = Instant::now();
            self.ecology.step(
                &self.gpu,
                &self.buffers[self.current],
                &self.catalog_buffer,
                &self.config,
                &self.catalog,
            )?;
            *self
                .progress
                .stage_ms
                .entry("ecology_month_wall".into())
                .or_default() += start.elapsed().as_secs_f64() * 1000.;
            *self
                .progress
                .stage_ms
                .entry("ecological_gpu".into())
                .or_default() += self.ecology.last_gpu_ms;
            for (name, ms) in &self.ecology.last_pass_ms {
                *self
                    .progress
                    .stage_ms
                    .entry(format!("eco/{name}"))
                    .or_default() += ms;
            }
            self.ecology.clock.epoch_month += 1;
            self.progress.iteration = self.ecology.clock.epoch_month;
            if self.ecology.clock.epoch_month < self.config.ecology_years_per_epoch * 12 {
                return Ok(false);
            }
            self.equilibrate_lakes()?;
            self.ecology.clock.epoch_month = 0;
            self.progress.epoch += 1;
            self.progress.geological_time_myr += self.config.geological_step_myr as f64;
            self.progress.stage = Stage::Boundary;
            self.progress.iteration = 0;
            return Ok(true);
        }
        let iterative = matches!(stage, Stage::Drainage | Stage::Basins | Stage::Flow);
        let batch = if iterative {
            8.min(
                self.config
                    .max_drainage_iterations
                    .saturating_sub(self.progress.iteration),
            )
        } else {
            1
        };
        let changed = self.dispatch(stage.kernel(), batch)?;
        self.progress.changed = changed;
        self.progress.iteration += batch;
        if iterative && changed > 0 {
            if self.progress.iteration >= self.config.max_drainage_iterations {
                let e=format!("{} did not converge after {} passes ({} cells changed). World remains paused at this stage; increase the limit and retry, or regenerate.",stage.label(),self.progress.iteration,changed);
                self.error = Some(e.clone());
                return Err(anyhow!(e));
            }
            return Ok(false);
        }
        if stage == Stage::Drainage {
            self.dispatch("drain_scatter", 1)?;
        }
        if stage == Stage::LakeReduce {
            self.reduction_count = self.reduction_count.div_ceil(256);
            self.reduction_side = 1 - self.reduction_side;
            if self.reduction_count > 1 {
                return Ok(false);
            }
        }
        if stage == Stage::Climate && self.progress.iteration < self.config.climate_iterations {
            return Ok(false);
        }
        self.progress.stage = match stage {
            Stage::Tectonics => Stage::DrainInit,
            Stage::DrainInit => Stage::Drainage,
            Stage::Drainage => Stage::BasinInit,
            Stage::BasinInit => Stage::Basins,
            Stage::Basins => Stage::ClimateStart,
            Stage::ClimateStart => Stage::Climate,
            Stage::Climate => Stage::ClimateCheck,
            Stage::ClimateCheck => {
                self.progress.climate_cycles += 1;
                self.progress.climate_converged = changed == 0;
                if changed > 0 && self.progress.climate_cycles < self.config.climate_cycles {
                    Stage::ClimateStart
                } else {
                    Stage::FlowInit
                }
            }
            Stage::FlowInit => Stage::Flow,
            Stage::Flow => Stage::LakeCollect,
            Stage::LakeCollect => {
                self.reduction_count = self.config.cells();
                self.reduction_side = 0;
                Stage::LakeReduce
            }
            Stage::LakeReduce => Stage::LakeUpdate,
            Stage::LakeUpdate => Stage::Water,
            Stage::Water => {
                self.dispatch("ecology", 1)?;
                Stage::Ecology
            }
            Stage::Ecology => {
                self.progress.epoch += 1;
                self.progress.geological_time_myr += self.config.geological_step_myr as f64;
                Stage::Boundary
            }
            Stage::Boundary => unreachable!(),
        };
        self.progress.iteration = 0;
        Ok(self.progress.stage == Stage::Boundary)
    }
    pub fn run_epochs(&mut self, count: u32) -> Result<()> {
        let target = self
            .progress
            .epoch
            .checked_add(count)
            .context("epoch overflow")?;
        while self.progress.epoch < target {
            self.advance()?;
        }
        Ok(())
    }
    /// Advance one ecological month with fixed geological terrain. Seasonal climate and water
    /// remain active; geological uplift and erosion are not advanced by this interface.
    pub fn advance_ecology(&mut self) -> Result<()> {
        ensure!(
            self.civilizations.is_none(),
            "Advance the environment through living history to keep the social and ecological clocks synchronized"
        );
        ensure!(
            self.progress.stage == Stage::Boundary,
            "ecology-only evolution requires an epoch boundary"
        );
        let saved = self.progress.iteration;
        self.progress.iteration =
            (self.ecology.clock.month % self.config.climate_iterations as u64) as u32;
        self.dispatch("climate", 1)?;
        self.progress.iteration = saved;
        let start = Instant::now();
        self.ecology.step(
            &self.gpu,
            &self.buffers[self.current],
            &self.catalog_buffer,
            &self.config,
            &self.catalog,
        )?;
        *self
            .progress
            .stage_ms
            .entry("ecological_gpu".into())
            .or_default() += self.ecology.last_gpu_ms;
        *self
            .progress
            .stage_ms
            .entry("ecology_month_wall".into())
            .or_default() += start.elapsed().as_secs_f64() * 1000.;
        for (name, ms) in &self.ecology.last_pass_ms {
            *self
                .progress
                .stage_ms
                .entry(format!("eco/{name}"))
                .or_default() += ms;
        }
        self.equilibrate_lakes()?;
        Ok(())
    }
    pub fn scenario(
        &mut self,
        region: Option<u32>,
        intervention: crate::ecology::Intervention,
    ) -> Result<()> {
        ensure!(
            self.progress.stage == Stage::Boundary,
            "scenarios require a completed epoch or synchronized monthly boundary"
        );
        if let Some(h) = &self.civilizations {
            ensure!(
                h.living.is_some(),
                "Enable living history before applying ecological scenarios to civilizations"
            );
            self.validate_living_boundary()?;
        }
        let history_month = self.civilizations.as_ref().map(|h| h.month);
        let history_event = self.civilizations.as_ref().map(|h| h.events.len() as u64);
        let description = format!("Environmental intervention {:?}, region {:?}, ecology month {}, history month {:?}; applies before the next monthly evolution", intervention, region, self.ecology.clock.month, history_month);
        let event = crate::ecology::ScenarioEvent {
            history_month,
            history_event,
            month: self.ecology.clock.month,
            region,
            intervention,
        };
        self.ecology.apply(
            &self.gpu,
            &self.buffers[self.current],
            &self.catalog_buffer,
            &self.config,
            &self.catalog,
            event,
        )?;
        if let Some(h) = &mut self.civilizations {
            h.event("ecological_intervention", None, None, description);
        }
        Ok(())
    }
    /// Upload a diagnostic ecological fixture, recording its starting inventory.
    /// This is not used by simulation kernels or checkpoint continuation.
    pub fn restore_ecology(&mut self, cells: &[crate::ecology::EcoCell]) -> Result<()> {
        ensure!(
            self.progress.stage == Stage::Boundary
                && cells.len() == self.config.eco_cells() as usize,
            "invalid ecological fixture boundary or dimensions"
        );
        crate::ecology::validate(cells)?;
        let mut cells = cells.to_vec();
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        for river in &self.ecology.rivers {
            encoder.clear_buffer(river, 0, None);
        }
        self.gpu.queue.submit(Some(encoder.finish()));
        self.ecology.prepare(
            &self.gpu,
            &self.buffers[self.current],
            &self.catalog_buffer,
            &self.config,
            &self.catalog,
            false,
        );
        let environment = read_buffer(
            &self.gpu,
            &self.ecology.environment,
            0,
            self.config.eco_cells() as u64 * crate::ecology::ENVIRONMENT_BYTES,
        )?;
        for (c, bytes) in cells
            .iter_mut()
            .zip(environment.chunks_exact(crate::ecology::ENVIRONMENT_BYTES as usize))
        {
            let e: crate::ecology::Environment = bytemuck::pod_read_unaligned(bytes);
            let total = c.inventory();
            c.pools[30] = [
                total[0] as f32,
                total[1] as f32,
                total[2] as f32,
                e.fields[7][3],
            ];
            c.pools[25][3] = e.fields[7][3];
            c.pools[27] = [0.; 4];
        }
        self.gpu.queue.write_buffer(
            &self.ecology.buffers[self.ecology.current],
            0,
            bytemuck::cast_slice(&cells),
        );
        self.ecology.clock.initialized = true;
        Ok(())
    }
    pub fn inspect(&self, index: u32) -> Result<Cell> {
        ensure!(index < self.config.cells(), "cell index out of bounds");
        let bytes = read_buffer(
            &self.gpu,
            &self.buffers[self.current],
            index as u64 * CELL_BYTES,
            CELL_BYTES,
        )?;
        Ok(bytemuck::pod_read_unaligned(&bytes))
    }
    pub fn planet_state(&self) -> Result<[f32; 4]> {
        let bytes = read_buffer(&self.gpu, &self.planet, 0, 16)?;
        Ok(bytemuck::pod_read_unaligned(&bytes))
    }
    pub fn snapshot(&self) -> Result<Vec<Cell>> {
        #[cfg(test)]
        self.terrain_snapshot_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let bytes = read_buffer(
            &self.gpu,
            &self.buffers[self.current],
            0,
            self.config.cells() as u64 * CELL_BYTES,
        )?;
        Ok(bytes
            .chunks_exact(CELL_BYTES as usize)
            .map(bytemuck::pod_read_unaligned)
            .collect())
    }
    /// Uploads a validated checkpoint or a diagnostic fixture, never used for simulation.
    pub fn restore_cells(&mut self, cells: &[Cell], epoch: u32) -> Result<()> {
        self.history_environment = None;
        self.navigation = None;
        ensure!(
            cells.len() == self.config.cells() as usize,
            "checkpoint cell count mismatch"
        );
        validate_cells(cells, &self.catalog)?;
        self.gpu
            .queue
            .write_buffer(&self.buffers[self.current], 0, bytemuck::cast_slice(cells));
        self.progress.epoch = epoch;
        self.progress.geological_time_myr = epoch as f64 * self.config.geological_step_myr as f64;
        self.progress.stage = Stage::Boundary;
        self.progress.iteration = 0;
        self.error = None;
        Ok(())
    }
    pub fn rebuild_drainage(&mut self) -> Result<()> {
        self.progress.stage = Stage::DrainInit;
        self.progress.iteration = 0;
        while self.progress.stage != Stage::Climate {
            self.advance()?;
        }
        Ok(())
    }
}
pub fn map_buffer(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Result<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv().context("GPU readback callback lost")??;
    let data = buffer.slice(..).get_mapped_range().to_vec();
    buffer.unmap();
    Ok(data)
}
pub fn read_buffer(
    gpu: &ContextGpu,
    buffer: &wgpu::Buffer,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>> {
    let read = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Explicit field readback"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut e = gpu.device.create_command_encoder(&Default::default());
    e.copy_buffer_to_buffer(buffer, offset, &read, 0, size);
    gpu.queue.submit(Some(e.finish()));
    map_buffer(&gpu.device, &read)
}
pub fn validate_cells(cells: &[Cell], catalog: &Catalog) -> Result<()> {
    for (i, c) in cells.iter().enumerate() {
        ensure!(
            c.terrain
                .iter()
                .chain(&c.climate)
                .chain(&c.water)
                .chain(&c.life)
                .chain(&c.geology)
                .chain(&c.hydro)
                .chain(&c.budget)
                .chain(&c.strata)
                .all(|v| v.is_finite()),
            "non-finite field at cell {i}"
        );
        ensure!(
            c.strata.iter().all(|v| *v >= 0.)
                && c.water.iter().all(|v| *v >= 0.)
                && c.terrain[1] >= 0.
                && c.terrain[2] >= 0.,
            "negative storage at cell {i}"
        );
        ensure!(
            c.ids[0] < (catalog.rocks.len() as u32)
                && c.ids[1] < (catalog.soils.len() as u32)
                && (c.ids[2] == NONE || c.ids[2] < (catalog.plants.len() as u32))
                && (c.ids[3] == NONE || c.ids[3] < (catalog.biomes.len() as u32)),
            "invalid catalog index at cell {i}"
        );
        ensure!(
            c.meta[0] < 4
                && (c.meta[1] == NONE || c.meta[1] < (catalog.minerals.len() as u32))
                && c.meta[2] < (catalog.rocks.len() as u32)
                && c.meta[3] < (catalog.rocks.len() as u32)
                && (c.routing[0] == NONE || c.routing[0] < (cells.len() as u32)),
            "invalid metadata at cell {i}"
        );
    }
    Ok(())
}
