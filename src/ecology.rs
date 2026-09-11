//! GPU ecological compartments. Stocks are kg C/N/P per m² of whole grid cell;
//! mixed coastal cells retain independent terrestrial and aquatic inventories.
use crate::{
    catalog::Catalog,
    config::Config,
    gpu::{read_buffer, ContextGpu},
};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

pub const ECO_BYTES: u64 = std::mem::size_of::<EcoCell>() as u64;
pub const ENVIRONMENT_BYTES: u64 = std::mem::size_of::<Environment>() as u64;
pub const GROWTH_LIMITS: [&str; 7] = [
    "energy",
    "nitrogen",
    "phosphorus",
    "habitat",
    "groundwater",
    "trace minerals",
    "oxidant",
];
pub const STOCKS: usize = 26;
pub const GUILD_NAMES: [&str; 12] = [
    "Small herbivores",
    "Browsers",
    "Grazers",
    "Giant herbivores",
    "Colossal herbivores",
    "Small predators",
    "Apex predators",
    "Detritivores",
    "Aquatic grazers",
    "Aquatic predators",
    "Migratory river animals",
    "Colonial waterbirds",
];
pub const COMPARTMENTS: [&str; 26] = [
    "Canopy",
    "Understory",
    "Root mat",
    "Shallow underground",
    "Deep fault",
    "Small herbivores",
    "Browsers",
    "Grazers",
    "Giant herbivores",
    "Colossal herbivores",
    "Small predators",
    "Apex predators",
    "Detritivores",
    "Aquatic grazers",
    "Aquatic predators",
    "Migratory river animals",
    "Colonial waterbirds",
    "Soil",
    "Detritus",
    "Groundwater",
    "Surface water",
    "Deep water",
    "Sediment",
    "Plankton",
    "Buried material",
    "Source rock",
];
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EcoCell {
    /// Guilds 5–16: xyz are C/N/P, w is outer-associated founder ancestry fraction.
    pub pools: [[f32; 4]; 38],
}
impl Default for EcoCell {
    fn default() -> Self {
        bytemuck::Zeroable::zeroed()
    }
}
impl EcoCell {
    pub fn inventory(&self) -> [f64; 3] {
        std::array::from_fn(|k| self.pools[..STOCKS].iter().map(|p| p[k] as f64).sum())
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Environment {
    /// 22–24: fine-edge land, open-water and river conductance (left/right/down/up).
    pub fields: [[f32; 4]; 25],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Intervention {
    GeochemicalSupply(bool),
    RemoveGuild(u32),
    RestoreGuild(u32),
    LakeMixing(f32),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioEvent {
    /// Coupled-history coordinates, absent for ecology-only/older events.
    #[serde(default)]
    pub history_month: Option<u32>,
    #[serde(default)]
    pub history_event: Option<u64>,
    pub month: u64,
    pub region: Option<u32>,
    pub intervention: Intervention,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EcologyClock {
    pub month: u64,
    pub epoch_month: u32,
    pub initialized: bool,
    pub events: Vec<ScenarioEvent>,
    pub imported_baseline: bool,
    /// 0: old untracked fauna; 1: explicit founders and ancestry fractions.
    pub wildlife_baseline: u32,
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    dims: [u32; 4],
    physical: [f32; 4],
    counts: [u32; 4],
    options: [u32; 4],
    event: [u32; 4],
    storms: [f32; 4],
    abundance: [f32; 4],
}
#[derive(Clone, Debug, Serialize)]
pub struct RegionReport {
    pub name: String,
    pub area_km2: f64,
    pub photo_kg_c_per_m2_year: f64,
    pub chemo_kg_c_per_m2_year: f64,
    pub biomass_kg_c_per_m2: f64,
    pub layer_biomass_kg_c_per_m2: [f64; 5],
    pub available_phosphorus_kg_per_m2: f64,
    pub reactive_rock_kg_per_m2: f64,
    pub geochemical_hotspot_fraction: f64,
    /// Nested fractions of regional land: enriched, active, vent. Not biomass quotas.
    pub geological_habitat_fraction: [f64; 3],
    /// Last-month rates annualized; integrate twelve samples for an annual result.
    pub layer_growth_kg_c_per_m2_year: [f64; 5],
    pub layer_loss_kg_c_per_m2_year: [f64; 5],
    pub layer_limit_fraction: [[f64; 7]; 5],
    pub diagnostic_area_fraction: f64,
    pub reacted_rock_kg_per_m2_year: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct BudgetReport {
    pub inventory_kg: [f64; 3],
    pub initial_kg: [f64; 3],
    pub external_kg: [f64; 3],
    pub residual_kg: [f64; 3],
    pub relative_error: [f64; 3],
    pub ecological_month: u64,
    pub water_inventory_m3: f64,
    pub water_initial_m3: f64,
    pub water_external_m3: f64,
    pub water_relative_error: f64,
    pub within_tolerance: bool,
    pub regions: Vec<RegionReport>,
}
pub struct Ecology {
    pub(crate) living: bool,
    pub(crate) living_weather: [u32; 4],
    pub(crate) living_storms: [f32; 4],
    pub(crate) buffers: [wgpu::Buffer; 2],
    pub(crate) environment: wgpu::Buffer,
    pub(crate) rivers: [wgpu::Buffer; 2],
    pub(crate) current: usize,
    pub clock: EcologyClock,
    uniform: wgpu::Buffer,
    query: Option<wgpu::QuerySet>,
    query_buffer: wgpu::Buffer,
    query_count: u32,
    timing: bool,
    pub last_gpu_ms: f64,
    pub last_pass_ms: std::collections::BTreeMap<String, f64>,
    timed_passes: Vec<String>,
    layout: wgpu::BindGroupLayout,
    pipelines: std::collections::BTreeMap<&'static str, wgpu::ComputePipeline>,
}
impl Ecology {
    pub fn new(gpu: &ContextGpu, config: &Config) -> Result<Self> {
        let d = &gpu.device;
        let size = config.eco_cells() as u64 * ECO_BYTES;
        ensure!(
            size <= d.limits().max_storage_buffer_binding_size as u64,
            "ecology buffer exceeds adapter storage binding limit; lower ecology resolution"
        );
        d.push_error_scope(wgpu::ErrorFilter::Validation);
        let buffer = |label, size| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let buffers = std::array::from_fn(|_| buffer("Ecological inventories", size));
        let environment = buffer(
            "Conservative terrain aggregation",
            config.eco_cells() as u64 * ENVIRONMENT_BYTES,
        );
        let rivers = std::array::from_fn(|_| {
            buffer(
                "Fine watershed nutrient transport",
                config.cells() as u64 * 16,
            )
        });
        let uniform = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ecological clocks and forcing"),
            contents: &[0u8; std::mem::size_of::<Params>()],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let entries: Vec<_> = (0..8)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: if binding == 7 {
                        wgpu::BufferBindingType::Uniform
                    } else {
                        wgpu::BufferBindingType::Storage {
                            read_only: binding == 2 || binding == 4 || binding == 5,
                        }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ecology bindings"),
            entries: &entries,
        });
        let pl = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let cell = include_str!("../shaders/simulation.wgsl")
            .split("struct Params")
            .next()
            .unwrap();
        let source = format!(
            "{cell}\n{}\n{}",
            include_str!("../shaders/sunlight.wgsl"),
            include_str!("../shaders/ecology.wgsl")
        );
        let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ecological cycles"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let mut pipelines = std::collections::BTreeMap::new();
        for name in [
            "reconcile_plots",
            "monthly_weather",
            "aggregate",
            "seed_ecology",
            "biology",
            "transport",
            "river_inject",
            "river_route",
            "river_collect",
            "river_clear",
            "feedback",
            "intervention",
            "replenish",
        ] {
            pipelines.insert(
                name,
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
        if let Some(e) = pollster::block_on(d.pop_error_scope()) {
            anyhow::bail!("Ecology shader: {e}");
        }
        let query = d
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
            .then(|| {
                d.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("Ecological pass timings"),
                    ty: wgpu::QueryType::Timestamp,
                    count: 64,
                })
            });
        let query_buffer = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ecological timestamp resolve"),
            size: 512,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        Ok(Self {
            living: false,
            living_weather: [0; 4],
            living_storms: [0.; 4],
            query,
            query_buffer,
            query_count: 0,
            timing: false,
            last_gpu_ms: 0.,
            last_pass_ms: Default::default(),
            timed_passes: Vec::new(),
            buffers,
            environment,
            rivers,
            current: 0,
            clock: EcologyClock::default(),
            uniform,
            layout,
            pipelines,
        })
    }
    fn params(&self, config: &Config, catalog: &Catalog, event: [u32; 4]) -> Params {
        Params {
            dims: [
                config.resolution,
                config.eco_resolution(),
                self.clock.month as u32,
                config.seed,
            ],
            physical: [
                config.radius_km,
                1. / 12.,
                config.lake_mixing,
                config.solar_scale,
            ],
            storms: self.living_storms,
            abundance: [
                config.island_phosphorus_scale,
                f32::from(catalog.producer_competition),
                f32::from(config.wildlife_open_barriers),
                config.axial_tilt.to_radians(), // abundance.w: solar obliquity
            ],
            counts: catalog.counts(),
            options: [
                catalog.biomes.len() as u32,
                catalog.guilds.len() as u32,
                catalog.microbes.len() as u32,
                u32::from(self.living),
            ],
            event: if self.living && event == [0; 4] {
                self.living_weather
            } else {
                event
            },
        }
    }
    #[allow(clippy::too_many_arguments)] // Explicit GPU resources and dispatch parameters.
    fn pass(
        &mut self,
        gpu: &ContextGpu,
        terrain: &wgpu::Buffer,
        table: &wgpu::Buffer,
        config: &Config,
        name: &str,
        river_side: usize,
        swap: bool,
    ) {
        let buffers = [
            terrain,
            &self.environment,
            &self.buffers[self.current],
            &self.buffers[1 - self.current],
            table,
            &self.rivers[river_side],
            &self.rivers[1 - river_side],
            &self.uniform,
        ];
        let entries: Vec<_> = buffers
            .iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
        let group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &self.layout,
            entries: &entries,
        });
        let mut e = gpu.device.create_command_encoder(&Default::default());
        {
            let mut p = e.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(name),
                timestamp_writes: if self.timing {
                    self.query
                        .as_ref()
                        .map(|query_set| wgpu::ComputePassTimestampWrites {
                            query_set,
                            beginning_of_pass_write_index: Some(self.query_count),
                            end_of_pass_write_index: Some(self.query_count + 1),
                        })
                } else {
                    None
                },
            });
            p.set_pipeline(&self.pipelines[name]);
            p.set_bind_group(0, &group, &[]);
            let n = if name.starts_with("river_") && name != "river_collect"
                || name == "feedback"
                || name == "monthly_weather"
                || name == "reconcile_plots"
            {
                config.resolution
            } else {
                config.eco_resolution()
            };
            p.dispatch_workgroups(n / 8, n / 8, 6);
        }
        if self.timing {
            self.query_count += 2;
            self.timed_passes.push(name.into());
        }
        gpu.queue.submit(Some(e.finish()));
        if swap {
            self.current = 1 - self.current;
        }
    }
    pub fn prepare(
        &mut self,
        gpu: &ContextGpu,
        terrain: &wgpu::Buffer,
        table: &wgpu::Buffer,
        config: &Config,
        catalog: &Catalog,
        replenish: bool,
    ) {
        gpu.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&self.params(config, catalog, [0; 4])),
        );
        self.pass(gpu, terrain, table, config, "aggregate", 0, false);
        if !self.clock.initialized {
            self.pass(gpu, terrain, table, config, "seed_ecology", 0, true);
            self.clock.initialized = true;
            self.clock.wildlife_baseline = 1;
        } else if replenish {
            self.pass(gpu, terrain, table, config, "replenish", 0, true);
        }
    }
    pub(crate) fn reconcile_plots(
        &mut self,
        gpu: &ContextGpu,
        terrain: &wgpu::Buffer,
        table: &wgpu::Buffer,
        config: &Config,
        catalog: &Catalog,
    ) {
        gpu.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&self.params(config, catalog, [0; 4])),
        );
        self.pass(gpu, terrain, table, config, "aggregate", 0, false);
        self.pass(gpu, terrain, table, config, "reconcile_plots", 0, false);
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(
            &self.rivers[1],
            0,
            &self.rivers[0],
            0,
            config.cells() as u64 * 16,
        );
        gpu.queue.submit(Some(encoder.finish()));
        self.pass(gpu, terrain, table, config, "aggregate", 0, false);
    }
    pub fn step(
        &mut self,
        gpu: &ContextGpu,
        terrain: &wgpu::Buffer,
        table: &wgpu::Buffer,
        config: &Config,
        catalog: &Catalog,
    ) -> Result<()> {
        ensure!(
            self.clock.month < u32::MAX as u64,
            "ecological clock exhausted"
        );
        self.query_count = 0;
        self.timed_passes.clear();
        self.last_pass_ms.clear();
        self.timing = true;
        if !self.clock.initialized {
            self.prepare(gpu, terrain, table, config, catalog, false);
        } else {
            gpu.queue.write_buffer(
                &self.uniform,
                0,
                bytemuck::bytes_of(&self.params(config, catalog, [0; 4])),
            );
        }
        self.pass(gpu, terrain, table, config, "monthly_weather", 0, false);
        self.pass(gpu, terrain, table, config, "aggregate", 0, false);
        self.pass(gpu, terrain, table, config, "river_inject", 0, false); // river 0 -> 1
        self.pass(gpu, terrain, table, config, "biology", 0, true);
        self.pass(gpu, terrain, table, config, "transport", 0, true);
        // Four bounded routing steps; terminal material is collected once below.
        for side in [1, 0, 1, 0] {
            self.pass(gpu, terrain, table, config, "river_route", side, false);
        }
        self.pass(gpu, terrain, table, config, "river_collect", 1, true);
        self.pass(gpu, terrain, table, config, "river_clear", 1, false); // canonical river 0
        self.pass(gpu, terrain, table, config, "feedback", 0, false);
        self.timing = false;
        if let Some(query) = &self.query {
            let mut e = gpu.device.create_command_encoder(&Default::default());
            e.resolve_query_set(query, 0..self.query_count, &self.query_buffer, 0);
            gpu.queue.submit(Some(e.finish()));
            let data = read_buffer(gpu, &self.query_buffer, 0, self.query_count as u64 * 8)?;
            let ticks: u64 = data
                .chunks_exact(16)
                .map(|v| {
                    u64::from_le_bytes(v[8..].try_into().unwrap())
                        .saturating_sub(u64::from_le_bytes(v[..8].try_into().unwrap()))
                })
                .sum();
            self.last_gpu_ms = ticks as f64 * gpu.queue.get_timestamp_period() as f64 / 1e6;
            for (name, v) in self.timed_passes.iter().zip(data.chunks_exact(16)) {
                let ticks = u64::from_le_bytes(v[8..].try_into().unwrap())
                    .saturating_sub(u64::from_le_bytes(v[..8].try_into().unwrap()));
                *self.last_pass_ms.entry(name.clone()).or_default() +=
                    ticks as f64 * gpu.queue.get_timestamp_period() as f64 / 1e6;
            }
        } else {
            gpu.device.poll(wgpu::Maintain::Wait);
            self.last_gpu_ms = 0.;
        }
        self.clock.month += 1;
        Ok(())
    }
    pub fn apply(
        &mut self,
        gpu: &ContextGpu,
        terrain: &wgpu::Buffer,
        table: &wgpu::Buffer,
        config: &Config,
        catalog: &Catalog,
        event: ScenarioEvent,
    ) -> Result<()> {
        ensure!(
            event.month == self.clock.month && event.region.is_none_or(|r| r < 4),
            "scenario must target the current monthly boundary and a valid region"
        );
        let (kind, value) = match event.intervention {
            Intervention::GeochemicalSupply(on) => (1, on as u32),
            Intervention::RemoveGuild(id) => {
                ensure!(id < 12, "invalid guild");
                (2, id)
            }
            Intervention::RestoreGuild(id) => {
                ensure!(id < 12, "invalid guild");
                (3, id)
            }
            Intervention::LakeMixing(v) => {
                ensure!(v.is_finite() && (0.0..=20.).contains(&v), "invalid mixing");
                (4, v.to_bits())
            }
        };
        self.prepare(gpu, terrain, table, config, catalog, false);
        gpu.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&self.params(
                config,
                catalog,
                [kind, value, event.region.unwrap_or(u32::MAX), 0],
            )),
        );
        self.pass(gpu, terrain, table, config, "intervention", 0, true);
        self.clock.events.push(event);
        Ok(())
    }
    pub fn snapshot(&self, gpu: &ContextGpu, config: &Config) -> Result<Vec<EcoCell>> {
        Ok(read_buffer(
            gpu,
            &self.buffers[self.current],
            0,
            config.eco_cells() as u64 * ECO_BYTES,
        )?
        .chunks_exact(ECO_BYTES as usize)
        .map(bytemuck::pod_read_unaligned)
        .collect())
    }
    pub fn inspect(&self, gpu: &ContextGpu, config: &Config, fine: u32) -> Result<EcoCell> {
        ensure!(fine < config.cells(), "cell index out of bounds");
        let n = config.resolution;
        let m = config.eco_resolution();
        let id = (fine / (n * n)) * m * m + (fine / n % n) / (n / m) * m + (fine % n) / (n / m);
        Ok(bytemuck::pod_read_unaligned(&read_buffer(
            gpu,
            &self.buffers[self.current],
            id as u64 * ECO_BYTES,
            ECO_BYTES,
        )?))
    }
    /// Terrain-derived habitats and the last completed biological step's diagnostics.
    /// fields[21].w is zero when aggregation has invalidated those diagnostics.
    pub fn inspect_environment(
        &self,
        gpu: &ContextGpu,
        config: &Config,
        fine: u32,
    ) -> Result<Environment> {
        ensure!(fine < config.cells(), "cell index out of bounds");
        let n = config.resolution;
        let m = config.eco_resolution();
        let id = fine / (n * n) * m * m + (fine / n % n) / (n / m) * m + (fine % n) / (n / m);
        Ok(bytemuck::pod_read_unaligned(&read_buffer(
            gpu,
            &self.environment,
            id as u64 * ENVIRONMENT_BYTES,
            ENVIRONMENT_BYTES,
        )?))
    }
    pub fn budget(&self, gpu: &ContextGpu, config: &Config) -> Result<BudgetReport> {
        let cells = self.snapshot(gpu, config)?;
        validate(&cells)?;
        let env = read_buffer(
            gpu,
            &self.environment,
            0,
            config.eco_cells() as u64 * ENVIRONMENT_BYTES,
        )?;
        let river = read_buffer(gpu, &self.rivers[0], 0, config.cells() as u64 * 16)?;
        let mut b = BudgetReport {
            inventory_kg: [0.; 3],
            initial_kg: [0.; 3],
            external_kg: [0.; 3],
            residual_kg: [0.; 3],
            relative_error: [0.; 3],
            ecological_month: self.clock.month,
            water_inventory_m3: 0.,
            water_initial_m3: 0.,
            water_external_m3: 0.,
            water_relative_error: 0.,
            within_tolerance: true,
            regions: [
                "Exterior ocean",
                "Great lake",
                "Inner continents",
                "Outer continent",
            ]
            .iter()
            .map(|name| RegionReport {
                name: name.to_string(),
                area_km2: 0.,
                photo_kg_c_per_m2_year: 0.,
                chemo_kg_c_per_m2_year: 0.,
                biomass_kg_c_per_m2: 0.,
                layer_biomass_kg_c_per_m2: [0.; 5],
                available_phosphorus_kg_per_m2: 0.,
                reactive_rock_kg_per_m2: 0.,
                geochemical_hotspot_fraction: 0.,
                geological_habitat_fraction: [0.; 3],
                layer_growth_kg_c_per_m2_year: [0.; 5],
                layer_loss_kg_c_per_m2_year: [0.; 5],
                layer_limit_fraction: [[0.; 7]; 5],
                diagnostic_area_fraction: 0.,
                reacted_rock_kg_per_m2_year: 0.,
            })
            .collect(),
        };
        for (c, e) in cells
            .iter()
            .zip(env.chunks_exact(ENVIRONMENT_BYTES as usize))
        {
            let e: Environment = bytemuck::pod_read_unaligned(e);
            let area = e.fields[3][0] as f64;
            b.water_inventory_m3 += c.pools[25][3] as f64 * area;
            b.water_initial_m3 += c.pools[30][3] as f64 * area;
            b.water_external_m3 += c.pools[27][3] as f64 * area;
            for (r, report) in b.regions.iter_mut().enumerate() {
                let a = e.fields[0][r] as f64 * area;
                report.area_km2 += a / 1e6;
                if r >= 2 {
                    let land = (e.fields[0][2] + e.fields[0][3]) as f64;
                    let weight = a / land.max(1e-20);
                    for k in 0..3 {
                        report.geological_habitat_fraction[k] += e.fields[8][k] as f64 * weight;
                    }
                    if e.fields[21][3] == 1. {
                        report.diagnostic_area_fraction += a;
                        report.reacted_rock_kg_per_m2_year += e.fields[21][0] as f64 * weight * 12.;
                        for k in 0..5 {
                            let d = e.fields[16 + k];
                            report.layer_growth_kg_c_per_m2_year[k] += d[1] as f64 * weight * 12.;
                            report.layer_loss_kg_c_per_m2_year[k] += d[2] as f64 * weight * 12.;
                            report.layer_limit_fraction[k][(d[3] as usize).min(6)] += a;
                        }
                    }
                }
                for (k, value) in report.layer_biomass_kg_c_per_m2.iter_mut().enumerate() {
                    *value += c.pools[k][0] as f64 * a;
                }
                report.available_phosphorus_kg_per_m2 += c.pools[17][2] as f64 * a;
                report.reactive_rock_kg_per_m2 += c.pools[26][2] as f64 * a;
                if c.pools[28][1] > 0.00001 && c.pools[28][1] > c.pools[28][0] * 0.1 {
                    report.geochemical_hotspot_fraction += a;
                }
                report.photo_kg_c_per_m2_year += c.pools[28][0] as f64 * a * 12.;
                report.chemo_kg_c_per_m2_year += c.pools[28][1] as f64 * a * 12.;
                report.biomass_kg_c_per_m2 +=
                    c.pools[..17].iter().map(|p| p[0] as f64).sum::<f64>() * a;
            }
            let total = c.inventory();
            for (k, value) in total.iter().enumerate() {
                b.inventory_kg[k] += value * area;
                b.initial_kg[k] += c.pools[30][k] as f64 * area;
                b.external_kg[k] += c.pools[27][k] as f64 * area;
            }
        }
        for chunk in river.chunks_exact(16) {
            let mass: [f32; 4] = bytemuck::pod_read_unaligned(chunk);
            for (k, v) in mass[..3].iter().enumerate() {
                b.inventory_kg[k] += *v as f64;
            }
        }
        for k in 0..3 {
            b.residual_kg[k] = b.inventory_kg[k] - b.initial_kg[k] - b.external_kg[k];
            b.relative_error[k] =
                b.residual_kg[k].abs() / b.inventory_kg[k].abs().max(b.initial_kg[k].abs()).max(1.);
        }
        b.water_relative_error = (b.water_inventory_m3 - b.water_initial_m3 - b.water_external_m3)
            .abs()
            / b.water_inventory_m3.max(b.water_initial_m3).max(1.);
        b.within_tolerance =
            b.relative_error.iter().all(|v| *v <= 0.001) && b.water_relative_error <= 0.001;
        for r in &mut b.regions {
            let a = (r.area_km2 * 1e6).max(1.);
            r.photo_kg_c_per_m2_year /= a;
            r.chemo_kg_c_per_m2_year /= a;
            r.biomass_kg_c_per_m2 /= a;
            for value in &mut r.layer_biomass_kg_c_per_m2 {
                *value /= a;
            }
            r.available_phosphorus_kg_per_m2 /= a;
            r.reactive_rock_kg_per_m2 /= a;
            r.geochemical_hotspot_fraction /= a;
            r.diagnostic_area_fraction /= a;
            r.reacted_rock_kg_per_m2_year /= a;
            for v in &mut r.geological_habitat_fraction {
                *v /= a;
            }
            for k in 0..5 {
                r.layer_growth_kg_c_per_m2_year[k] /= a;
                r.layer_loss_kg_c_per_m2_year[k] /= a;
                for v in &mut r.layer_limit_fraction[k] {
                    *v /= a;
                }
            }
        }
        Ok(b)
    }
}
pub fn validate(cells: &[EcoCell]) -> Result<()> {
    for (i, c) in cells.iter().enumerate() {
        ensure!(
            c.pools.iter().flatten().all(|v| v.is_finite()),
            "non-finite ecology at {i}"
        );
        ensure!(
            (0.0..=1.).contains(&c.pools[31][0])
                && (0.0..=20.).contains(&c.pools[31][1])
                && (0.0..=1.).contains(&c.pools[31][2])
                && (0.0..=4095.).contains(&c.pools[31][3])
                && c.pools[31][3].fract() == 0.,
            "invalid ecological controls at {i}"
        );
        ensure!(
            c.pools[..5]
                .iter()
                .all(|v| v[3] >= 0. && v[3] <= 256. && v[3].fract() == 0.),
            "invalid ecological producer index at {i}"
        );
        ensure!(
            c.pools[32..].iter().all(|p| p[..2]
                .iter()
                .all(|id| *id >= 0. && *id <= 256. && id.fract() == 0.)
                && (0. ..=1.).contains(&p[2])
                && (p[3] == 0. || p[3] == 1.)),
            "invalid producer composition at {i}"
        );
        ensure!(
            (0. ..=1.).contains(&c.pools[24][3]),
            "invalid managed area fraction at {i}"
        );
        ensure!(
            c.pools[5..17].iter().all(|p| (0. ..=1.).contains(&p[3])),
            "invalid wildlife ancestry at {i}: {:?}",
            c.pools[5..17].iter().map(|p| p[3]).collect::<Vec<_>>()
        );
        ensure!(
            c.pools[..27]
                .iter()
                .all(|p| p[..3].iter().all(|v| *v >= 0.)),
            "negative ecological stock at {i}"
        );
    }
    Ok(())
}

#[cfg(test)]
mod flood_tests {
    use super::*;
    use crate::gpu::{Generator, NONE};

    #[test]
    #[ignore = "requires a hardware GPU"]
    fn routed_overflow_is_finite_and_drains_back_to_runoff() {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                // Isolate reservoir drainage: seasonal warming must not evaporate
                // part of the prescribed pulse before its exact half-release.
                axial_tilt: 0.,
                resolution: 64,
                ecology_resolution: 64,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        let mut cells = g.snapshot().unwrap();
        let i = cells
            .iter()
            .position(|c| c.meta[0] == 2 && c.routing[0] != NONE)
            .unwrap();
        cells[i].water = [0.; 4];
        cells[i].hydro[0] = cells[i].terrain[0];
        cells[i].hydro[1] = -5.;
        cells[i].hydro[2] = 0.;
        g.restore_cells(&cells, 0).unwrap();
        g.ecology.living = true;
        g.ecology.prepare(
            &g.gpu,
            &g.buffers[g.current],
            &g.catalog_buffer,
            &g.config,
            &g.catalog,
            false,
        );
        let env: Environment = bytemuck::pod_read_unaligned(
            &crate::gpu::read_buffer(
                &g.gpu,
                &g.ecology.environment,
                i as u64 * ENVIRONMENT_BYTES,
                ENVIRONMENT_BYTES,
            )
            .unwrap(),
        );
        let area = env.fields[3][0];
        let payload = [area * 0.01, area * 0.02, area * 0.03, area * 0.6];
        g.gpu.queue.write_buffer(
            &g.ecology.rivers[0],
            i as u64 * 16,
            bytemuck::bytes_of(&payload),
        );
        let soil_before = g
            .ecology
            .inspect(&g.gpu, &g.config, i as u32)
            .unwrap()
            .pools[17];
        g.ecology.pass(
            &g.gpu,
            &g.buffers[g.current],
            &g.catalog_buffer,
            &g.config,
            "river_collect",
            0,
            true,
        );
        g.ecology.pass(
            &g.gpu,
            &g.buffers[g.current],
            &g.catalog_buffer,
            &g.config,
            "river_clear",
            0,
            false,
        );
        let after = g.snapshot().unwrap()[i];
        let routed: [f32; 4] = bytemuck::pod_read_unaligned(
            &crate::gpu::read_buffer(&g.gpu, &g.ecology.rivers[1], i as u64 * 16, 16).unwrap(),
        );
        assert!((after.water[0] - 0.6).abs() < 1e-5);
        assert!((after.water[0] * area + routed[3] - payload[3]).abs() / payload[3] < 1e-6);
        let soil_after = g
            .ecology
            .inspect(&g.gpu, &g.config, i as u32)
            .unwrap()
            .pools[17];
        for k in 0..3 {
            assert!(
                ((soil_after[k] - soil_before[k]) * area + routed[k] - payload[k]).abs()
                    / payload[k]
                    < 1e-5
            );
        }
        g.ecology.pass(
            &g.gpu,
            &g.buffers[g.current],
            &g.catalog_buffer,
            &g.config,
            "monthly_weather",
            0,
            false,
        );
        let drained = g.snapshot().unwrap()[i];
        assert!(drained.budget[0].abs() < 1e-6);
        assert!(drained.budget[1].abs() < 1e-6);
        assert!((drained.water[0] - 0.3).abs() < 1e-5);
        assert!((drained.life[3] - 0.3).abs() < 1e-5);
        assert!((drained.water[0] + drained.life[3] - after.water[0]).abs() < 1e-6);
        // A bankfull channel retains its through-flow; a narrow floodplain has
        // bounded storage and sends the rest onward without deleting water.
        for (flow, expected_depth) in [(1e12, 0.), (area * 0.3 / (31557600. / 12.) / 2., 0.075)] {
            cells[i].water[3] = flow;
            g.restore_cells(&cells, 0).unwrap();
            g.ecology.pass(
                &g.gpu,
                &g.buffers[g.current],
                &g.catalog_buffer,
                &g.config,
                "river_clear",
                0,
                false,
            );
            let c = g.snapshot().unwrap()[i];
            let r: [f32; 4] = bytemuck::pod_read_unaligned(
                &crate::gpu::read_buffer(&g.gpu, &g.ecology.rivers[1], i as u64 * 16, 16).unwrap(),
            );
            assert!((c.water[0] - expected_depth).abs() < 1e-5);
            assert!((c.water[0] * area + r[3] - payload[3]).abs() / payload[3] < 1e-6);
        }
    }
}

#[cfg(test)]
mod hotspot_tests {
    use super::*;
    use crate::gpu::Generator;

    #[test]
    #[ignore = "requires hardware GPU"]
    fn narrow_habitats_preserve_area_reaction_and_conditional_climate() {
        let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
        let catalog = Catalog::bundled().unwrap();
        let rock = catalog
            .rocks
            .iter()
            .position(|r| r.id == "peridotite")
            .unwrap() as u32;
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                resolution: 16,
                ecology_resolution: 16,
                ecology_years_per_epoch: 1,
                ..Default::default()
            },
            catalog.clone(),
        )
        .unwrap();
        g.run_epochs(1).unwrap();
        let mut terrain = g.snapshot().unwrap();
        for (i, c) in terrain.iter_mut().enumerate() {
            let hotspot = i % 16 % 2 == 0 && i / 16 % 16 % 2 == 0;
            c.meta[0] = 3;
            c.ids[0] = rock;
            c.terrain[0] = 500.;
            c.geology[0] = if hotspot { 0.9 } else { 0. };
            c.hydro[1] = if hotspot { 20. } else { -30. };
            c.hydro[2] = 1400.;
            c.water = [0., 1., 0., 0.];
        }
        let mut total = Vec::new();
        for m in [16, 8] {
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    resolution: 16,
                    ecology_resolution: m,
                    ..Default::default()
                },
                catalog.clone(),
            )
            .unwrap();
            g.restore_cells(&terrain, 0).unwrap();
            g.ecology.prepare(
                &g.gpu,
                &g.buffers[g.current],
                &g.catalog_buffer,
                &g.config,
                &g.catalog,
                false,
            );
            let mut area = [0f64; 4];
            for face in 0..6 {
                for y in 0..m {
                    for x in 0..m {
                        let fine = face * 256 + y * (16 / m) * 16 + x * (16 / m);
                        let e = g
                            .ecology
                            .inspect_environment(&g.gpu, &g.config, fine)
                            .unwrap();
                        let a = e.fields[3][0] as f64;
                        area[0] += e.fields[8][0] as f64 * a;
                        area[1] += e.fields[8][1] as f64 * a;
                        area[2] += e.fields[8][2] as f64 * a;
                        // Initial substrate * substrate-weighted annual reaction rate.
                        area[3] += e.fields[9][0] as f64 * 10000. * a;
                        if e.fields[8][0] > 0. {
                            assert!((e.fields[10][0] - 20.).abs() < 1e-4);
                            assert!((e.fields[13][0] - 20.).abs() < 1e-4);
                            if m == 8 {
                                assert!(e.fields[1][0] < -10.);
                                assert!(e.fields[8][0] < 0.4);
                            }
                        }
                    }
                }
            }
            // Fresh underground layers must establish in the preserved warm patch.
            g.ecology.pass(
                &g.gpu,
                &g.buffers[g.current],
                &g.catalog_buffer,
                &g.config,
                "biology",
                0,
                true,
            );
            let e = g.ecology.inspect_environment(&g.gpu, &g.config, 0).unwrap();
            assert!(e.fields[19][1] > 0. && e.fields[20][1] > 0., "{e:?}");
            assert_eq!(e.fields[21][3], 1.);
            total.push(area);
        }
        for k in 0..4 {
            assert!(
                (total[0][k] - total[1][k]).abs() / total[0][k] < 2e-5,
                "{total:?}"
            );
        }
    }
}

/// Neutral ancestry tracer, not a species count or an evolved trait.
#[derive(Clone, Debug, Serialize)]
pub struct WildlifeReport {
    pub baseline: u32,
    pub month: u64,
    /// Ocean, great lake, central land, outer land; kg carbon by guild.
    pub carbon_kg: [[f64; 12]; 4],
    pub outer_founder_fraction: [[f64; 12]; 4],
    /// Area with >1e-10 kg C/m², divided by geographic region area.
    pub occupied_fraction: [[f64; 12]; 4],
    /// C/N/P in compartments 0–25, area-weighted with the same coarse region estimate.
    /// Includes producer, dissolved, sediment and buried stocks for trophic diagnosis.
    pub compartment_cnp_kg: [[[f64; 3]; 26]; 4],
}
impl Ecology {
    pub fn wildlife_report(&self, gpu: &ContextGpu, config: &Config) -> Result<WildlifeReport> {
        let cells = self.snapshot(gpu, config)?;
        let bytes = read_buffer(
            gpu,
            &self.environment,
            0,
            config.eco_cells() as u64 * ENVIRONMENT_BYTES,
        )?;
        let mut report = WildlifeReport {
            baseline: self.clock.wildlife_baseline,
            month: self.clock.month,
            carbon_kg: [[0.; 12]; 4],
            outer_founder_fraction: [[0.; 12]; 4],
            occupied_fraction: [[0.; 12]; 4],
            compartment_cnp_kg: [[[0.; 3]; 26]; 4],
        };
        let mut areas = [0.; 4];
        for (c, raw) in cells
            .iter()
            .zip(bytes.chunks_exact(ENVIRONMENT_BYTES as usize))
        {
            let e: Environment = bytemuck::pod_read_unaligned(raw);
            for (r, area) in areas.iter_mut().enumerate() {
                let fraction = e.fields[0][r] as f64;
                let a = e.fields[3][0] as f64;
                *area += fraction * a;
                for (k, pool) in c.pools[..26].iter().enumerate() {
                    for (element, amount) in pool[..3].iter().enumerate() {
                        report.compartment_cnp_kg[r][k][element] += *amount as f64 * a * fraction;
                    }
                }
                for k in 0..12 {
                    // Region allocation is a coarse-cell estimate, not fine-scale occupancy.
                    let carbon = c.pools[k + 5][0] as f64 * a * fraction;
                    report.carbon_kg[r][k] += carbon;
                    report.outer_founder_fraction[r][k] += carbon * c.pools[k + 5][3] as f64;
                    if c.pools[k + 5][0] > 1e-10 {
                        report.occupied_fraction[r][k] += fraction * a;
                    }
                }
            }
        }
        for (r, area) in areas.iter().enumerate() {
            for k in 0..12 {
                report.outer_founder_fraction[r][k] /= report.carbon_kg[r][k].max(1e-30);
                report.occupied_fraction[r][k] /= area.max(1.);
            }
        }
        Ok(report)
    }
}

#[cfg(test)]
mod wildlife_tests {
    use super::*;
    use crate::gpu::Generator;
    fn fixture() -> Generator {
        Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 16,
                ecology_resolution: 8,
                ecology_years_per_epoch: 1,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap()
    }
    fn dispatch(g: &mut Generator, name: &str) {
        g.ecology.pass(
            &g.gpu,
            &g.buffers[g.current],
            &g.catalog_buffer,
            &g.config,
            name,
            0,
            true,
        );
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn seasonal_light_reverses_actual_aquatic_production() {
        let mut g = fixture();
        g.run_epochs(1).unwrap();
        let count = g.config.eco_cells() as usize;
        let mut env = Environment {
            fields: [[0.; 4]; 25],
        };
        env.fields[0][1] = 1.; // identical open lake habitat
        env.fields[1] = [23., 1400., 1., 0.];
        env.fields[3] = [1., 0., 100., 0.75];
        let mut cell = EcoCell::default();
        cell.pools[20] = [0., 100., 100., 0.];
        cell.pools[23] = [0.1, 0.003, 0.0003, 0.];
        cell.pools[31] = [0., 0., 1., 0.];
        let initial = vec![cell; count];
        let mut production = Vec::new();
        for (month, latitude, scale, living) in [
            (0, 0.75, 1., false),
            (6, 0.75, 1., false),
            (0, -0.75, 1., false),
            (6, -0.75, 1., false),
            (6, 0.75, 0., false),
            (11, 1., 1., false),
            (6, 0.75, 1., true),
        ] {
            g.restore_ecology(&initial).unwrap();
            env.fields[3][3] = latitude;
            g.gpu.queue.write_buffer(
                &g.ecology.environment,
                0,
                bytemuck::cast_slice(&vec![env; count]),
            );
            g.config.solar_scale = scale;
            g.ecology.clock.month = month;
            g.ecology.living = living;
            g.ecology.living_weather[3] = month as u32 + 1;
            g.gpu.queue.write_buffer(
                &g.ecology.uniform,
                0,
                bytemuck::bytes_of(&g.ecology.params(&g.config, &g.catalog, [0; 4])),
            );
            dispatch(&mut g, "biology");
            production.push(g.ecology.snapshot(&g.gpu, &g.config).unwrap()[0].pools[28][0]);
        }
        assert!(
            production[1] > production[0] && production[0] > 0.,
            "{production:?}"
        );
        assert!(production[2] > production[3]);
        assert!((production[0] - production[3]).abs() < 1e-6);
        assert!((production[1] - production[2]).abs() < 1e-6);
        assert_eq!(production[4], 0.);
        assert_eq!(production[5], 0.);
        assert_eq!(production[6], production[1]);
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn empty_fauna_does_not_reappear_after_restoration() {
        let mut g = fixture();
        g.run_epochs(1).unwrap();
        let mut cells = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
        for c in &mut cells {
            for p in &mut c.pools[5..17] {
                *p = [0.; 4];
            }
        }
        g.restore_ecology(&cells).unwrap();
        for _ in 0..12 {
            g.advance_ecology().unwrap();
        }
        assert!(g
            .ecology
            .snapshot(&g.gpu, &g.config)
            .unwrap()
            .iter()
            .all(|c| c.pools[5..17].iter().all(|p| p[..3] == [0.; 3])));
        // Restore permits immigration; it is not a reintroduction.
        g.scenario(None, Intervention::RestoreGuild(0)).unwrap();
        g.advance_ecology().unwrap();
        assert!(g
            .ecology
            .snapshot(&g.gpu, &g.config)
            .unwrap()
            .iter()
            .all(|c| c.pools[5][0] == 0.));
    }
    #[test]
    #[ignore = "requires hardware GPU"]
    fn phosphorus_limited_intake_matches_analytical_growth() {
        let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
        for (feeding, scale) in [(0.8, 1.), (2.4, 1.), (2.4, 1e-8)] {
            let mut catalog = Catalog::bundled().unwrap();
            for plant in &mut catalog.plants {
                if plant.ecology.layer == 5 {
                    plant.ecology.nitrogen = 0.03;
                    plant.ecology.phosphorus = 0.001;
                    plant.ecology.maintenance = 0.;
                    plant.ecology.turnover = 0.;
                    plant.ecology.fixation = 0.;
                }
            }
            catalog.guilds[8].feeding = feeding;
            catalog.guilds[8].food_half_saturation = 0.;
            catalog.guilds[8].diet = vec![crate::catalog::DietItem {
                prey: 23,
                weight: 1.,
                efficiency: 1.,
            }];
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    resolution: 16,
                    ecology_resolution: 16,
                    ecology_years_per_epoch: 1,
                    solar_scale: 0.,
                    ..Default::default()
                },
                catalog,
            )
            .unwrap();
            g.run_epochs(1).unwrap();
            let mut terrain = g.snapshot().unwrap();
            for c in &mut terrain {
                c.meta[0] = 1;
                c.terrain[0] = -100.;
                c.hydro = [0., 20., 1000., 0.];
                c.water = [100., 0., 0., 0.];
            }
            g.restore_cells(&terrain, 1).unwrap();
            let mut cells = vec![EcoCell::default(); g.config.eco_cells() as usize];
            for c in &mut cells {
                c.pools[31] = [0., 0., 1., 0.];
                c.pools[23] = [scale, 0.03 * scale, 0.001 * scale, 0.];
                c.pools[13] = [0.01 * scale, 0.0012 * scale, 0.00015 * scale, 0.];
            }
            g.restore_ecology(&cells).unwrap();
            dispatch(&mut g, "biology");
            let after = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
            let expected = (0.01 + 0.01 * feeding / 12. * 0.001 / 0.015)
                * (1. - 0.1 / 12.)
                * (1. - 0.02 / 12.);
            assert!(
                (after[0].pools[13][0] / scale - expected).abs() < 1e-7,
                "feeding {feeding}: {} vs {expected}",
                after[0].pools[13][0]
            );
            assert_eq!(after[0].pools[13][0] > 0.01 * scale, feeding > 1.);
            let budget = g.ecology.budget(&g.gpu, &g.config).unwrap();
            assert!(budget.relative_error.iter().all(|v| v.abs() < 1e-5));
        }
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn animal_refuge_matches_capture_and_conserves_nutrients() {
        let gpu = pollster::block_on(ContextGpu::headless()).unwrap();
        for (food, refuge) in [(0.01, 0.), (0.01, 1e-4), (1e-6, 0.), (1e-6, 1e-4)] {
            let mut catalog = Catalog::bundled().unwrap();
            catalog.guilds[8].feeding = 0.;
            catalog.guilds[8].maintenance = 0.;
            catalog.guilds[9].maintenance = 0.;
            catalog.guilds[9].food_half_saturation = 1e-4;
            catalog.guilds[9].animal_prey_refuge = refuge;
            catalog.guilds[9].diet = vec![crate::catalog::DietItem {
                prey: 13,
                weight: 1.,
                efficiency: 1.,
            }];
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    resolution: 16,
                    ecology_resolution: 16,
                    ecology_years_per_epoch: 1,
                    solar_scale: 0.,
                    ..Default::default()
                },
                catalog,
            )
            .unwrap();
            g.run_epochs(1).unwrap();
            let mut terrain = g.snapshot().unwrap();
            for c in &mut terrain {
                c.meta[0] = 1;
                c.terrain[0] = -100.;
                c.hydro = [0., 20., 1000., 0.];
                c.water = [100., 0., 0., 0.];
            }
            g.restore_cells(&terrain, 1).unwrap();
            let consumer = food * 0.01;
            let mut cells = vec![EcoCell::default(); g.config.eco_cells() as usize];
            for c in &mut cells {
                c.pools[31] = [0., 0., 1., 0.];
                c.pools[13] = [food, food * 0.12, food * 0.015, 0.];
                c.pools[14] = [consumer, consumer * 0.12, consumer * 0.015, 0.];
            }
            g.restore_ecology(&cells).unwrap();
            dispatch(&mut g, "biology");
            let after = g.ecology.snapshot(&gpu, &g.config).unwrap();
            // Prey mortality happens before predator feeding in this monthly batch.
            let available = food * (1. - 0.02 / 12.);
            let effective = available * available / (available + refuge);
            let bite = consumer * 0.8 / 12. * effective / (effective + 1e-4);
            let expected = (consumer + bite * 0.4) * (1. - 0.02 / 12.);
            assert!((after[0].pools[14][0] / expected - 1.).abs() < 1e-5);
            assert!(((available - after[0].pools[13][0]) - bite).abs() < food * 1e-6);
            if food < 1e-5 && refuge > 0. {
                let no_refuge = consumer * 0.8 / 12. * available / (available + 1e-4);
                assert!(
                    bite < no_refuge * 0.02,
                    "rare prey must receive meaningful protection"
                );
                let path = std::env::temp_dir()
                    .join(format!("wildlife-refuge-{}.world", std::process::id()));
                g.save(&path).unwrap();
                let mut resumed = Generator::load(gpu.clone(), &path).unwrap();
                std::fs::remove_file(path).unwrap();
                assert_eq!(resumed.catalog.guilds[9].animal_prey_refuge, refuge);
                g.advance_ecology().unwrap();
                resumed.advance_ecology().unwrap();
                let a = g.ecology.snapshot(&gpu, &g.config).unwrap();
                let b = resumed.ecology.snapshot(&gpu, &resumed.config).unwrap();
                assert_eq!(
                    bytemuck::cast_slice::<EcoCell, u8>(&a),
                    bytemuck::cast_slice::<EcoCell, u8>(&b)
                );
            }
            assert!(g.ecology.budget(&gpu, &g.config).unwrap().within_tolerance);
        }
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn numerical_extinction_returns_remaining_nutrients() {
        let mut g = fixture();
        g.run_epochs(1).unwrap();
        let mut cells = vec![EcoCell::default(); g.config.eco_cells() as usize];
        for c in &mut cells {
            c.pools[31] = [0., 0., 1., 0.];
            c.pools[5] = [1e-14, 1.2e-15, 1.5e-16, 1.];
        }
        g.restore_ecology(&cells).unwrap();
        dispatch(&mut g, "biology");
        let after = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
        for c in &after {
            assert_eq!(c.pools[5], [0.; 4]);
            let inventory = c.inventory();
            assert!((inventory[1] / 1.2e-15 - 1.).abs() < 1e-5);
            assert!((inventory[2] / 1.5e-16 - 1.).abs() < 1e-5);
        }
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn edge_conductance_matches_fine_cpu_reference_including_seams() {
        let mut g = fixture();
        g.run_epochs(1).unwrap();
        let mut terrain = g.snapshot().unwrap();
        for (i, c) in terrain.iter_mut().enumerate() {
            c.meta[0] = ((i * 7 + i / 16) % 4) as u32;
            c.terrain[0] = (i % 13) as f32 * 110.;
        }
        g.restore_cells(&terrain, 1).unwrap();
        g.ecology.prepare(
            &g.gpu,
            &g.buffers[g.current],
            &g.catalog_buffer,
            &g.config,
            &g.catalog,
            false,
        );
        let bytes = read_buffer(
            &g.gpu,
            &g.ecology.environment,
            0,
            g.config.eco_cells() as u64 * ENVIRONMENT_BYTES,
        )
        .unwrap();
        let offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (i, raw) in bytes.chunks_exact(ENVIRONMENT_BYTES as usize).enumerate() {
            let e: Environment = bytemuck::pod_read_unaligned(raw);
            for (d, (dx, dy)) in offsets.iter().enumerate() {
                let mut land = 0.;
                let mut water = 0.;
                for q in 0..2 {
                    let x = if d < 2 {
                        if d == 0 {
                            0
                        } else {
                            1
                        }
                    } else {
                        q
                    };
                    let y = if d >= 2 {
                        if d == 2 {
                            0
                        } else {
                            1
                        }
                    } else {
                        q
                    };
                    let fine = (i / 64) * 256 + (i / 8 % 8 * 2 + y) * 16 + (i % 8 * 2 + x);
                    let other = crate::grid::neighbor(fine as u32, 16, *dx, *dy) as usize;
                    let a = terrain[fine];
                    let b = terrain[other];
                    if a.meta[0] >= 2 && b.meta[0] >= 2 {
                        land += 0.5 / (1. + (a.terrain[0] - b.terrain[0]).abs() / 1000.);
                    }
                    if a.meta[0] < 2 && a.meta[0] == b.meta[0] {
                        water += 0.5;
                    }
                }
                assert!((e.fields[22][d] - land).abs() < 1e-6, "cell {i} edge {d}");
                assert_eq!(e.fields[23][d], water);
            }
        }
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn fine_edges_block_false_coastal_bridges_and_preserve_ancestry() {
        let mut g = fixture();
        g.run_epochs(1).unwrap();
        let mut terrain = g.snapshot().unwrap();
        for c in &mut terrain {
            c.meta[0] = 1;
            c.water = [10., 0., 0., 0.];
            c.terrain[0] = -10.;
            c.routing[0] = u32::MAX;
        }
        // Adjacent coarse cells each contain land, but their facing columns are water.
        let i = 18usize;
        let j = 19usize;
        for y in 4..6 {
            for x in [4usize, 7] {
                let c = &mut terrain[y * 16 + x];
                c.meta[0] = 2;
                c.terrain[0] = 10.;
                c.water = [0.; 4];
            }
        }
        g.restore_cells(&terrain, 1).unwrap();
        g.ecology.prepare(
            &g.gpu,
            &g.buffers[g.current],
            &g.catalog_buffer,
            &g.config,
            &g.catalog,
            false,
        );
        let mut cells = vec![EcoCell::default(); g.config.eco_cells() as usize];
        for c in &mut cells {
            c.pools[31] = [0., 0., 1., 0.];
            c.pools[1] = [1., 0.12, 0.015, 0.];
        }
        cells[i].pools[5] = [1., 0.12, 0.015, 1.];
        g.restore_ecology(&cells).unwrap();
        dispatch(&mut g, "transport");
        assert_eq!(
            g.ecology.snapshot(&g.gpu, &g.config).unwrap()[j].pools[5][0],
            0.
        );
        // Open only the actual shared land edge; the previously absent population arrives.
        for y in 4..6 {
            for x in 5..7 {
                let c = &mut terrain[y * 16 + x];
                c.meta[0] = 2;
                c.terrain[0] = 10.;
                c.water = [0.; 4];
            }
        }
        g.restore_cells(&terrain, 1).unwrap();
        g.ecology.prepare(
            &g.gpu,
            &g.buffers[g.current],
            &g.catalog_buffer,
            &g.config,
            &g.catalog,
            false,
        );
        g.restore_ecology(&cells).unwrap();
        dispatch(&mut g, "transport");
        let arrived = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
        assert!(arrived[j].pools[5][0] > 0.);
        assert_eq!(arrived[j].pools[5][3], 1.);
        let b = g.ecology.budget(&g.gpu, &g.config).unwrap();
        assert!(b.relative_error.iter().all(|x| *x < 2e-6));
        // Equal source/destination carbon, different ancestry: analytical two-box mix.
        cells[j].pools[5] = [1., 0.12, 0.015, 0.];
        g.restore_ecology(&cells).unwrap();
        dispatch(&mut g, "transport");
        let mixed = g.ecology.snapshot(&g.gpu, &g.config).unwrap();
        let ea = g
            .ecology
            .inspect_environment(&g.gpu, &g.config, 4 * 16 + 4)
            .unwrap();
        let eb = g
            .ecology
            .inspect_environment(&g.gpu, &g.config, 4 * 16 + 6)
            .unwrap();
        // Guild 0 sees understory weight .6; other prey are empty.
        let flux = (1. / 12.) * 0.1 * (0.6 / (0.6 + 0.01));
        let expected = flux * ea.fields[3][0].min(eb.fields[3][0]) / eb.fields[3][0];
        assert!((mixed[j].pools[5][3] - expected).abs() < 1e-7);
    }
}
