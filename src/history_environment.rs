//! Private CPU terrain view for history. Static geography is cached; all cells
//! consumed by monthly CPU history are refreshed. GPU navigation reads current
//! terrain directly; CPU reference searches require a full annual refresh. Never expose this partially refreshed view as a world snapshot.
use crate::{
    civilization::History,
    gpu::{Cell, Generator, CELL_BYTES},
    grid,
};
use anyhow::{ensure, Result};

// Keep the bitwise WGSL gather ABI synchronized with terrain storage.
const _: () = assert!(CELL_BYTES == 176);

/// Execution strategy only; not part of the simulated world or archive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HistoryReadbackMode {
    #[default]
    Gathered,
    /// Reference path for differential verification and diagnostics.
    Full,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HistoryReadbackStats {
    pub full_refreshes: u64,
    pub gathered_refreshes: u64,
    pub terrain_bytes: u64,
    pub observed_cells: u64,
    pub wall_ms: f64,
}

#[derive(Default)]
pub(crate) struct HistoryEnvironment {
    pub(crate) terrain: Vec<Cell>,
    month: Option<u32>,
    epoch: u32,
    buffer: usize,
    gather: Option<Gather>,
}

struct Gather {
    pipeline: wgpu::ComputePipeline,
    indices: wgpu::Buffer,
    observations: wgpu::Buffer,
    params: wgpu::Buffer,
    capacity: usize,
}
impl Gather {
    fn new(g: &Generator, count: usize) -> Result<Self> {
        let d = &g.gpu.device;
        let bytes = count as u64 * CELL_BYTES;
        ensure!(
            bytes <= d.limits().max_storage_buffer_binding_size as u64
                && bytes <= d.limits().max_buffer_size,
            "history observation buffer exceeds GPU limits"
        );
        let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("History environment gather"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/history_environment.wgsl").into(),
            ),
        });
        let pipeline = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("History environment gather"),
            layout: None,
            module: &shader,
            entry_point: Some("gather"),
            compilation_options: Default::default(),
            cache: None,
        });
        let buffer = |label, size, usage| {
            d.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        Ok(Self {
            pipeline,
            indices: buffer(
                "History observed cell IDs",
                count as u64 * 4,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            ),
            observations: buffer(
                "History observed cells",
                bytes,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            ),
            params: buffer(
                "History gather dimensions",
                16,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            capacity: count,
        })
    }
    fn read(&self, g: &Generator, ids: &[u32], month: u32) -> Result<Vec<u8>> {
        let groups = (ids.len() as u32).div_ceil(64);
        let x = groups.min(g.gpu.device.limits().max_compute_workgroups_per_dimension);
        let y = groups.div_ceil(x);
        ensure!(
            y <= g.gpu.device.limits().max_compute_workgroups_per_dimension,
            "history gather dispatch exceeds GPU limits"
        );
        g.gpu
            .queue
            .write_buffer(&self.indices, 0, bytemuck::cast_slice(ids));
        g.gpu.queue.write_buffer(
            &self.params,
            0,
            bytemuck::cast_slice(&[ids.len() as u32, x * 64, month, 0]),
        );
        let buffers = [
            &g.buffers[g.current],
            &self.indices,
            &self.observations,
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
        let group = g.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("History observed environment"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let mut encoder = g.gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups(x, y, 1);
        }
        g.gpu.queue.submit(Some(encoder.finish()));
        crate::gpu::read_buffer(&g.gpu, &self.observations, 0, ids.len() as u64 * CELL_BYTES)
    }
}

/// Includes prospective settlements and their neighbors, not just occupied towns.
/// Route cells are kept explicit in v1 rather than replacing hazard calculations
/// with a differently ordered reduction.
fn observed_cells(h: &History, include_routes: bool) -> Vec<u32> {
    let mut ids = Vec::new();
    for cell in h
        .sites
        .iter()
        .map(|s| s.cell)
        .chain(h.candidates.iter().map(|c| c.cell))
    {
        ids.push(cell);
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            ids.push(grid::neighbor(cell, h.terrain_resolution, dx, dy));
        }
    }
    if include_routes {
        if let Some(s) = &h.society {
            for r in &s.routes {
                ids.extend_from_slice(&r.cells);
            }
        }
        if let Some(s) = &h.shipping {
            for p in &s.ports {
                ids.push(p.water_cell);
                ids.extend_from_slice(&p.access);
            }
            for lane in &s.lanes {
                ids.extend_from_slice(&lane.cells);
            }
        }
    }
    if let Some(x) = &h.expeditions {
        for r in &x.routes {
            if include_routes {
                ids.extend_from_slice(&r.cells);
            } else if let Some(&cell) = r.cells.last() {
                ids.push(cell);
            }
        }
        if let Some(d) = &x.discoveries {
            ids.extend(d.sources.iter().map(|s| s.cell));
        }
    }
    if let Some(r) = &h.resources {
        ids.extend(r.sources.keys().copied());
    }
    ids.sort_unstable();
    ids.dedup();
    ids
}

impl HistoryEnvironment {
    pub(crate) fn refresh(&mut self, g: &mut Generator) -> Result<()> {
        let started = std::time::Instant::now();
        let h = g.civilizations.as_ref().unwrap();
        let month = h.month + 1;
        let full = g.history_readback_mode == HistoryReadbackMode::Full
            || self.terrain.len() != g.config.cells() as usize
            || self.epoch != g.progress.epoch
            || self.buffer != g.current
            || (g.navigation_mode == crate::navigation::NavigationMode::CpuReference
                && (month % 12 == 0
                    || h.society
                        .as_ref()
                        .is_some_and(|s| (s.routed_sites as usize) < h.sites.len())));
        let ids = if full {
            Vec::new()
        } else {
            observed_cells(
                h,
                g.navigation_mode == crate::navigation::NavigationMode::CpuReference,
            )
        };
        ensure!(
            ids.iter().all(|&id| id < g.config.cells()),
            "invalid history observation cell"
        );
        // A dense observation set has no transfer advantage over the reference path.
        let count = if full || ids.len() >= g.config.cells() as usize {
            self.terrain = g.snapshot()?;
            g.history_readback_stats.full_refreshes += 1;
            g.config.cells() as u64
        } else {
            if !ids.is_empty() {
                if self.gather.as_ref().is_none_or(|x| x.capacity < ids.len()) {
                    self.gather = Some(Gather::new(g, ids.len())?);
                }
                let bytes = self.gather.as_ref().unwrap().read(g, &ids, month)?;
                for (&id, bytes) in ids.iter().zip(bytes.chunks_exact(CELL_BYTES as usize)) {
                    self.terrain[id as usize] = bytemuck::pod_read_unaligned(bytes);
                }
            }
            g.history_readback_stats.gathered_refreshes += 1;
            ids.len() as u64
        };
        self.month = Some(month);
        self.epoch = g.progress.epoch;
        self.buffer = g.current;
        g.history_readback_stats.observed_cells += count;
        g.history_readback_stats.terrain_bytes += count * CELL_BYTES;
        g.history_readback_stats.wall_ms += started.elapsed().as_secs_f64() * 1000.;
        Ok(())
    }
    pub(crate) fn for_month(&self, month: u32) -> Result<&[Cell]> {
        ensure!(
            self.month == Some(month),
            "stale history environment observations"
        );
        Ok(&self.terrain)
    }
}

impl Generator {
    /// Select execution strategy. Full mode is retained for matched-run auditing.
    pub fn set_history_readback_mode(&mut self, mode: HistoryReadbackMode) {
        self.history_readback_mode = mode;
        self.history_environment = None;
        self.history_readback_stats = Default::default();
    }
    /// Terrain transfer counters only; excludes economic, ecological and save readbacks.
    pub fn history_readback_stats(&self) -> HistoryReadbackStats {
        self.history_readback_stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_observation_month_is_rejected() {
        let view = HistoryEnvironment {
            month: Some(4),
            ..Default::default()
        };
        assert!(view.for_month(4).is_ok());
        assert!(view.for_month(3).is_err());
        assert!(view.for_month(5).is_err());
        assert!(HistoryEnvironment::default().for_month(0).is_err());
    }

    #[test]
    #[ignore = "requires hardware GPU"]
    fn gather_is_bitwise_exact_at_seams_and_after_restore() {
        use crate::{catalog::Catalog, config::Config, gpu::ContextGpu};
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 64,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.advance_ecology().unwrap();
        let terrain = g.snapshot().unwrap();
        let mut ids = vec![0, 63, 64, 4095, 4096, g.config.cells() - 1];
        ids.extend((0..g.config.cells()).step_by(71));
        let gather = Gather::new(&g, ids.len()).unwrap();
        let compare = |g: &Generator, cells: &[Cell]| {
            let bytes = gather.read(g, &ids, 1).unwrap();
            let expected: Vec<Cell> = ids.iter().map(|&i| cells[i as usize]).collect();
            assert_eq!(bytes, bytemuck::cast_slice::<_, u8>(&expected));
        };
        compare(&g, &terrain);
        let mut changed = terrain;
        changed[0].climate[0] += 1.;
        changed[4096].life[0] = 0.5;
        g.history_environment = Some(HistoryEnvironment::default());
        g.restore_cells(&changed, 0).unwrap();
        assert!(g.history_environment.is_none());
        compare(&g, &changed);
    }
}
