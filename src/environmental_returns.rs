//! Managed land and runoff return to canonical ecology at coupled monthly boundaries.
use crate::gpu::{Generator, Stage};
use anyhow::{ensure, Result};
use wgpu::util::DeviceExt;
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Transfer {
    ids: [u32; 4],
    flow: [f32; 4],
}
impl Generator {
    pub fn enable_environmental_returns(&mut self) -> Result<()> {
        self.validate_living_boundary()?;
        ensure!(
            self.progress.stage == Stage::Boundary,
            "returns require a completed boundary"
        );
        let h = self
            .civilizations
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("no history"))?;
        let l = h
            .living
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("enable living history first"))?;
        if !l.environmental_returns {
            l.environmental_returns = true;
            h.event("environmental_returns",None,None,"Managed runoff now feeds its fine river cell; abandoned land returns finite environmental stocks to wilderness".into());
        }
        Ok(())
    }
    pub(crate) fn commit_environmental_returns(&mut self) -> Result<()> {
        let h = self.civilizations.as_ref().unwrap();
        let transfers: Vec<_> = h
            .sites
            .iter()
            .filter(|s| s.economy.return_flow.iter().any(|v| *v > 0.))
            .map(|s| Transfer {
                ids: [
                    s.cell,
                    s.economy.claim[0] as u32,
                    s.economy.claim[2].to_bits(),
                    0,
                ],
                flow: s.economy.return_flow,
            })
            .collect();
        if transfers.is_empty() {
            return Ok(());
        }
        ensure!(
            transfers.iter().all(|t| t.ids[0] < self.config.cells()
                && t.ids[1] < self.config.eco_cells()
                && f32::from_bits(t.ids[2]) > 0.
                && t.flow.iter().all(|v| v.is_finite() && *v >= 0.)),
            "invalid environmental transfer"
        );
        let d = &self.gpu.device;
        let pipeline = self.return_pipeline.get_or_insert_with(|| {
            let shader = d.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Managed returns"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../shaders/managed_returns.wgsl").into(),
                ),
            });
            d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Managed returns"),
                layout: None,
                module: &shader,
                entry_point: Some("commit"),
                compilation_options: Default::default(),
                cache: None,
            })
        });
        let data = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Managed return transfers"),
            contents: bytemuck::cast_slice(&transfers),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let bindings = [
            &self.ecology.rivers[0],
            &self.ecology.buffers[self.ecology.current],
            &data,
        ];
        let group = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &bindings
                .iter()
                .enumerate()
                .map(|(i, b)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: b.as_entire_binding(),
                })
                .collect::<Vec<_>>(),
        });
        let mut encoder = d.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        self.gpu.queue.submit(Some(encoder.finish()));
        for s in &mut self.civilizations.as_mut().unwrap().sites {
            s.economy.return_flow = [0.; 4];
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::Catalog,
        config::Config,
        gpu::{read_buffer, ContextGpu},
    };
    #[test]
    #[ignore = "requires hardware GPU"]
    fn transfers_enter_the_receiving_fine_cell_once() {
        let mut g = Generator::new(
            pollster::block_on(ContextGpu::headless()).unwrap(),
            Config {
                resolution: 32,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled().unwrap(),
        )
        .unwrap();
        g.advance_ecology().unwrap();
        g.found_civilizations(5).unwrap();
        g.enable_living_history().unwrap();
        g.enable_environmental_returns().unwrap();
        let s = &g.civilizations.as_ref().unwrap().sites[0];
        let cell = s.cell;
        let area = s.economy.claim[2];
        let before = g.ecology.inspect(&g.gpu, &g.config, cell).unwrap();
        let read = |g: &Generator| -> [f32; 4] {
            bytemuck::pod_read_unaligned(
                &read_buffer(&g.gpu, &g.ecology.rivers[0], u64::from(cell) * 16, 16).unwrap(),
            )
        };
        let river_before = read(&g);
        // Large enough to resolve against planetary float32 inventories.
        let payload = [area * 0.01, area * 0.001, area * 0.0001, area * 0.1];
        g.civilizations.as_mut().unwrap().sites[0]
            .economy
            .return_flow = payload;
        assert!(g.validate_living_boundary().is_err());
        g.commit_environmental_returns().unwrap();
        let after = g.ecology.inspect(&g.gpu, &g.config, cell).unwrap();
        let river_after = read(&g);
        for k in 0..4 {
            assert!((river_after[k] - river_before[k] - payload[k]).abs() / payload[k] < 1e-5);
            assert!(
                ((after.pools[27][k] - before.pools[27][k]) * area - payload[k]).abs() / payload[k]
                    < 0.002
            );
        }
        g.commit_environmental_returns().unwrap();
        assert_eq!(read(&g), river_after);
        g.validate_living_boundary().unwrap();
    }
}
