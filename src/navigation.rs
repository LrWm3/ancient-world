//! GPU frontier searches. World topology stays implicit; only paths cross to CPU.
use crate::{
    civilization::History,
    gpu::{read_buffer, ContextGpu, Generator},
};
use anyhow::{ensure, Result};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationMode {
    #[default]
    Gpu,
    CpuReference,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum RouteKind {
    Road = 0,
    Harbor = 1,
    Sea = 2,
    Expedition = 3,
}
#[derive(Clone, Debug, Default)]
pub struct NavigationStats {
    pub queries: u64,
    pub reachable: u64,
    pub waves: u64,
    pub readback_bytes: u64,
    pub query_ms: f64,
    pub survey_bytes: u64,
    pub inspection_bytes: u64,
}
pub(crate) struct Navigation {
    gpu: ContextGpu,
    world: wgpu::Buffer,
    n: u32,
    radius: f32,
    pub(crate) terrain_buffer: usize,
    pub(crate) epoch: u32,
    distances: wgpu::Buffer,
    control: wgpu::Buffer,
    output: wgpu::Buffer,
    params: wgpu::Buffer,
    group: wgpu::BindGroup,
    pipelines: BTreeMap<&'static str, wgpu::ComputePipeline>,
    stats: Mutex<NavigationStats>,
    survey_pipeline: OnceLock<wgpu::ComputePipeline>,
    inspect_pipeline: OnceLock<wgpu::ComputePipeline>,
}
impl Navigation {
    pub(crate) fn new(g: &Generator) -> Result<Self> {
        let d = &g.gpu.device;
        let count = g.config.cells() as u64;
        ensure!(
            (count + 4) * 4 <= d.limits().max_storage_buffer_binding_size as u64,
            "navigation workspace exceeds GPU binding limit"
        );
        let make = |label, size, usage| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let rw = wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST;
        let distances = make(
            "Navigation distances / inner-continent roots",
            count * 4,
            rw,
        );
        let qa = make("Navigation frontier A", count * 4, rw);
        let qb = make("Navigation frontier B", count * 4, rw);
        let marks = make("Navigation frontier deduplication", count * 4, rw);
        let control = make("Navigation control", 48, rw | wgpu::BufferUsages::INDIRECT);
        let output = make("Navigation reconstructed path", (count + 4) * 4, rw);
        let params = make(
            "Navigation query",
            48,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let entries: Vec<_> = (0..8)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: if binding == 7 {
                        wgpu::BufferBindingType::Uniform
                    } else {
                        wgpu::BufferBindingType::Storage {
                            read_only: binding == 0,
                        }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let layout = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Navigation"),
            entries: &entries,
        });
        let pipeline_layout = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Navigation"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Frontier navigation"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/navigation.wgsl").into()),
        });
        let pipelines = [
            "initialize",
            "prepare",
            "relax",
            "finish",
            "choose",
            "choose_id",
            "reconstruct",
            "label_init",
            "hook",
            "compress",
        ]
        .into_iter()
        .map(|name| {
            (
                name,
                d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(name),
                    layout: Some(&pipeline_layout),
                    module: &shader,
                    entry_point: Some(name),
                    compilation_options: Default::default(),
                    cache: None,
                }),
            )
        })
        .collect();
        let world = g.buffers[g.current].clone();
        let buffers = [
            &world, &distances, &qa, &qb, &marks, &control, &output, &params,
        ];
        let entries: Vec<_> = buffers
            .iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
        let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Navigation"),
            layout: &layout,
            entries: &entries,
        });
        Ok(Self {
            gpu: g.gpu.clone(),
            world,
            n: g.config.resolution,
            radius: g.config.radius_km,
            terrain_buffer: g.current,
            epoch: g.progress.epoch,
            distances,
            control,
            output,
            params,
            group,
            pipelines,
            stats: Mutex::new(Default::default()),
            survey_pipeline: OnceLock::new(),
            inspect_pipeline: OnceLock::new(),
        })
    }
    fn count(&self) -> u32 {
        6 * self.n * self.n
    }
    fn write_query(&self, start: u32, end: u32, kind: RouteKind) {
        let limit = match kind {
            RouteKind::Road => 3_000_000,
            RouteKind::Harbor => 2_000_000,
            _ => 20_000_000,
        };
        self.gpu.queue.write_buffer(
            &self.params,
            0,
            bytemuck::cast_slice(&[
                self.n,
                start,
                end,
                kind as u32,
                self.radius.to_bits(),
                0,
                0,
                0,
                limit,
                self.count(),
                0,
                0,
            ]),
        );
    }
    fn dispatch(&self, e: &mut wgpu::CommandEncoder, name: &str, whole: bool) {
        let mut pass = e.begin_compute_pass(&Default::default());
        pass.set_pipeline(&self.pipelines[name]);
        pass.set_bind_group(0, &self.group, &[]);
        if name == "relax" {
            pass.dispatch_workgroups_indirect(&self.control, 16);
        } else {
            let groups = if whole { self.count().div_ceil(64) } else { 1 };
            pass.dispatch_workgroups(groups.min(65535), groups.div_ceil(65535), 1);
        }
    }
    /// Converged integer-distance search with descending-cost reconstruction.
    /// Separate GPU edge calculations and tie choices may change the returned path.
    pub(crate) fn route(
        &self,
        start: u32,
        end: Option<u32>,
        kind: RouteKind,
    ) -> Result<Option<(Vec<u32>, f32)>> {
        ensure!(
            start < self.count() && end.is_none_or(|i| i < self.count()),
            "navigation cell outside grid"
        );
        ensure!(
            matches!(kind, RouteKind::Harbor | RouteKind::Expedition) || end.is_some(),
            "route needs a destination"
        );
        let started = std::time::Instant::now();
        self.write_query(start, end.unwrap_or(u32::MAX), kind);
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        self.dispatch(&mut e, "initialize", true);
        self.gpu.queue.submit(Some(e.finish()));
        let mut waves = 0;
        let mut bytes = 0;
        loop {
            ensure!(
                waves < self.n * 32,
                "navigation unresolved after {waves} frontier waves"
            );
            let mut e = self.gpu.device.create_command_encoder(&Default::default());
            let batch = if waves == 0 { 8 } else { 64 };
            for _ in 0..batch {
                self.dispatch(&mut e, "prepare", false);
                self.dispatch(&mut e, "relax", false);
                self.dispatch(&mut e, "finish", false);
            }
            self.gpu.queue.submit(Some(e.finish()));
            waves += batch;
            let raw = read_buffer(&self.gpu, &self.control, 0, 32)?;
            bytes += 32;
            let c: &[u32] = bytemuck::cast_slice(&raw);
            ensure!(c[7] == 0, "navigation frontier capacity exceeded");
            if c[c[2] as usize] == 0 {
                break;
            }
        }
        self.gpu.queue.write_buffer(
            &self.control,
            0,
            bytemuck::cast_slice(&[u32::MAX, u32::MAX]),
        );
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        self.dispatch(&mut e, "choose", true);
        self.dispatch(&mut e, "choose_id", true);
        self.dispatch(&mut e, "reconstruct", false);
        self.gpu.queue.submit(Some(e.finish()));
        let raw = read_buffer(&self.gpu, &self.output, 0, 16)?;
        bytes += 16;
        let header: &[u32] = bytemuck::cast_slice(&raw);
        ensure!(
            header[3] == 0 && header[0] <= self.count(),
            "navigation path reconstruction failed: {header:?}"
        );
        let result = if header[0] == 0 {
            None
        } else {
            let raw = read_buffer(&self.gpu, &self.output, 16, header[0] as u64 * 4)?;
            bytes += raw.len() as u64;
            let mut path = bytemuck::cast_slice::<u8, u32>(&raw).to_vec();
            path.reverse();
            Some((path, header[1] as f32 / 1000.))
        };
        let mut stats = self.stats.lock().unwrap();
        stats.queries += 1;
        stats.reachable += u64::from(result.is_some());
        stats.waves += waves as u64;
        stats.readback_bytes += bytes;
        stats.query_ms += started.elapsed().as_secs_f64() * 1000.;
        Ok(result)
    }
    pub fn stats(&self) -> NavigationStats {
        self.stats.lock().unwrap().clone()
    }
    fn label_islands(&self) -> Result<()> {
        self.write_query(0, 0, RouteKind::Road);
        let mut e = self.gpu.device.create_command_encoder(&Default::default());
        self.dispatch(&mut e, "label_init", true);
        self.gpu.queue.submit(Some(e.finish()));
        for _ in 0..128 {
            self.gpu
                .queue
                .write_buffer(&self.control, 28, bytemuck::bytes_of(&0u32));
            let mut e = self.gpu.device.create_command_encoder(&Default::default());
            self.dispatch(&mut e, "hook", true);
            self.dispatch(&mut e, "compress", true);
            self.gpu.queue.submit(Some(e.finish()));
            self.stats.lock().unwrap().survey_bytes += 4;
            if bytemuck::pod_read_unaligned::<u32>(&read_buffer(&self.gpu, &self.control, 28, 4)?)
                == 0
            {
                return Ok(());
            }
        }
        anyhow::bail!("inner-continent labeling did not converge")
    }
}
impl Generator {
    pub fn set_navigation_mode(&mut self, mode: NavigationMode) {
        self.navigation_mode = mode;
        self.navigation = None;
        self.history_environment = None;
    }
    pub fn navigation_stats(&self) -> NavigationStats {
        self.navigation
            .as_ref()
            .map_or_else(Default::default, |n| n.stats())
    }
    pub(crate) fn navigation_service(&mut self) -> Result<Option<Arc<Navigation>>> {
        if self.navigation_mode == NavigationMode::CpuReference {
            return Ok(None);
        }
        if self
            .navigation
            .as_ref()
            .is_none_or(|n| n.terrain_buffer != self.current || n.epoch != self.progress.epoch)
        {
            self.navigation = Some(Arc::new(Navigation::new(self)?));
        }
        Ok(self.navigation.clone())
    }
    /// Diagnostic/public path query over current canonical terrain.
    pub fn survey_route(
        &mut self,
        start: u32,
        end: Option<u32>,
        kind: RouteKind,
    ) -> Result<Option<(Vec<u32>, f32)>> {
        ensure!(
            start < self.config.cells() && end.is_none_or(|i| i < self.config.cells()),
            "navigation cell outside grid"
        );
        ensure!(
            end.is_some() == matches!(kind, RouteKind::Road | RouteKind::Sea),
            "roads and sea lanes need a destination; harbor and expedition queries find a landing"
        );
        if let Some(n) = self.navigation_service()? {
            n.route(start, end, kind)
        } else {
            let cells = self.snapshot()?;
            Ok(match kind {
                RouteKind::Road => crate::society::terrain_path(
                    start,
                    end.ok_or_else(|| anyhow::anyhow!("road needs destination"))?,
                    self.config.resolution,
                    self.config.radius_km,
                    &cells,
                ),
                _ => crate::shipping::path(
                    start,
                    end,
                    self.config.resolution,
                    self.config.radius_km,
                    &cells,
                    kind == RouteKind::Expedition,
                ),
            })
        }
    }
}

impl Navigation {
    fn survey_records(&self, scores: Option<&wgpu::Buffer>) -> Result<Vec<[u32; 8]>> {
        use wgpu::util::DeviceExt;
        self.label_islands()?;
        self.gpu.queue.write_buffer(
            &self.params,
            12,
            bytemuck::bytes_of(&u32::from(scores.is_none())),
        );
        let d = &self.gpu.device;
        ensure!(
            self.count() as u64 * 32 <= d.limits().max_storage_buffer_binding_size as u64,
            "survey workspace exceeds GPU binding limit"
        );
        let dummy = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Unused coast scores"),
            contents: &[0u8; 16],
            usage: wgpu::BufferUsages::STORAGE,
        });
        let output = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compact survey records"),
            size: self.count() as u64 * 32,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let count = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Survey count"),
            contents: &[0u8; 4],
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let pipeline = self.survey_pipeline.get_or_init(|| {
            let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Survey compaction"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../shaders/navigation_survey.wgsl").into(),
                ),
            });
            d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Survey compaction"),
                layout: None,
                module: &shader,
                entry_point: Some("survey"),
                compilation_options: Default::default(),
                cache: None,
            })
        });
        let buffers = [
            &self.world,
            scores.unwrap_or(&dummy),
            &self.distances,
            &output,
            &count,
            &self.params,
        ];
        let entries: Vec<_> = buffers
            .iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
        let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Survey compaction"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut e = d.create_command_encoder(&Default::default());
        {
            let mut pass = e.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &group, &[]);
            let groups = self.count().div_ceil(64);
            pass.dispatch_workgroups(groups.min(65535), groups.div_ceil(65535), 1);
        }
        self.gpu.queue.submit(Some(e.finish()));
        let n = bytemuck::pod_read_unaligned::<u32>(&read_buffer(&self.gpu, &count, 0, 4)?);
        ensure!(n <= self.count(), "survey output overflow");
        self.stats.lock().unwrap().survey_bytes += 4 + n as u64 * 32;
        if n == 0 {
            return Ok(vec![]);
        }
        let bytes = read_buffer(&self.gpu, &output, 0, n as u64 * 32)?;
        let mut records: Vec<[u32; 8]> = bytemuck::cast_slice(&bytes).to_vec();
        records.sort_by_key(|r| r[0]);
        Ok(records)
    }
    pub(crate) fn candidates(
        &self,
        scores: &wgpu::Buffer,
        plot_hectares: f64,
    ) -> Result<Vec<crate::civilization::Candidate>> {
        use crate::naming::Landmark;
        Ok(self
            .survey_records(Some(scores))?
            .iter()
            .map(|r| crate::civilization::Candidate {
                cell: r[0],
                island: r[1],
                naming_landmark: Some(match r[2] {
                    1 => Landmark::Hill,
                    2 => Landmark::Water,
                    3 => Landmark::Shore,
                    _ => Landmark::Field,
                }),
                yield_kg: f32::from_bits(r[4]),
                score: f32::from_bits(r[5]),
                hectares: (crate::grid::solid_angle(r[0], self.n)
                    * (self.radius as f64 * 1000.).powi(2)
                    / 10000.
                    * 0.01)
                    .min(plot_hectares) as f32,
            })
            .collect())
    }
    pub(crate) fn island_coasts(&self) -> Result<BTreeMap<u32, Vec<u32>>> {
        let mut coasts: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for r in self.survey_records(None)? {
            coasts.entry(r[1]).or_default().push(r[0]);
        }
        Ok(coasts)
    }
    /// Exact current hazard predicates, in road / port / sea-lane order.
    pub(crate) fn inspect_routes(&self, h: &History) -> Result<Vec<bool>> {
        use wgpu::util::DeviceExt;
        let mut ids = Vec::new();
        let mut spans = Vec::new();
        let mut add = |path: &[u32], kind: u32, water: u32| {
            spans.push([ids.len() as u32, path.len() as u32, kind, water]);
            ids.extend_from_slice(path);
        };
        if let Some(s) = &h.society {
            for r in &s.routes {
                add(&r.cells, 0, 0);
            }
        }
        if let Some(s) = &h.shipping {
            for p in &s.ports {
                add(&p.access, 1, p.water_cell);
            }
            for l in &s.lanes {
                add(&l.cells, 2, 0);
            }
        }
        if spans.is_empty() {
            return Ok(vec![]);
        }
        ensure!(
            ids.iter().all(|&i| i < self.count())
                && spans.iter().all(|s| s[2] != 1 || s[3] < self.count()),
            "invalid route inspection cell"
        );
        if ids.is_empty() {
            ids.push(0);
        }
        let d = &self.gpu.device;
        ensure!(
            ids.len() as u64 * 4 <= d.limits().max_storage_buffer_binding_size as u64
                && spans.len() as u64 * 16 <= d.limits().max_storage_buffer_binding_size as u64
                && spans.len().div_ceil(64) <= 65535,
            "route inspection exceeds GPU limits"
        );
        let input = |label, bytes: &[u8]| {
            d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytes,
                usage: wgpu::BufferUsages::STORAGE,
            })
        };
        let indices = input("Inspected route cells", bytemuck::cast_slice(&ids));
        let spans_buffer = input("Inspected route spans", bytemuck::cast_slice(&spans));
        let output = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Route hazard summaries"),
            size: spans.len() as u64 * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let pipeline = self.inspect_pipeline.get_or_init(|| {
            let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Route hazard summaries"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../shaders/navigation_inspect.wgsl").into(),
                ),
            });
            d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Route hazard summaries"),
                layout: None,
                module: &shader,
                entry_point: Some("inspect"),
                compilation_options: Default::default(),
                cache: None,
            })
        });
        let buffers = [&self.world, &indices, &spans_buffer, &output];
        let entries: Vec<_> = buffers
            .iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
        let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Route hazard summaries"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut e = d.create_command_encoder(&Default::default());
        {
            let mut pass = e.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups((spans.len() as u32).div_ceil(64), 1, 1);
        }
        self.gpu.queue.submit(Some(e.finish()));
        self.stats.lock().unwrap().inspection_bytes += spans.len() as u64 * 4;
        Ok(bytemuck::cast_slice::<u8, u32>(&read_buffer(
            &self.gpu,
            &output,
            0,
            spans.len() as u64 * 4,
        )?)
        .iter()
        .map(|&v| v != 0)
        .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires hardware GPU"]
    fn route_inspections_match_cpu_after_water_changes() {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            crate::config::Config {
                seed: 17,
                resolution: 64,
                ecology_resolution: 16,
                ..Default::default()
            },
            crate::catalog::Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.advance_ecology().unwrap();
        g.found_civilizations(16).unwrap();
        g.enable_society().unwrap();
        g.enable_shipping().unwrap();
        g.enable_living_history().unwrap();
        let h = g.civilizations.as_ref().unwrap().clone();
        assert!(!h.society.as_ref().unwrap().routes.is_empty());
        assert!(!h.shipping.as_ref().unwrap().lanes.is_empty());
        let mut cells = g.snapshot().unwrap();
        for wet in [true, false] {
            for route in &h.society.as_ref().unwrap().routes {
                for &id in &route.cells {
                    let c = &mut cells[id as usize];
                    c.routing[0] = id;
                    c.hydro[0] = c.terrain[0];
                    c.water[3] = 10.;
                    c.water[0] = if wet { 0.02 } else { 0. };
                }
            }
            for port in &h.shipping.as_ref().unwrap().ports {
                cells[port.water_cell as usize].water[0] = if wet { 0. } else { 10. };
            }
            g.restore_cells(&cells, 0).unwrap();
            let result = g
                .navigation_service()
                .unwrap()
                .unwrap()
                .inspect_routes(&h)
                .unwrap();
            assert!(result.iter().any(|&v| v) || !wet);
            let mut cpu = h.clone();
            let mut gpu = h.clone();
            cpu.environmental_month(&cells);
            gpu.environmental_month_with_inspections(&cells, Some(&result));
            assert!(
                serde_json::to_value(cpu).unwrap() == serde_json::to_value(gpu).unwrap(),
                "water intervention {wet}"
            );
        }
    }
}
