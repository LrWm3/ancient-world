use crate::{
    catalog::Catalog,
    config::Config,
    gpu::{self, Cell, ContextGpu, Generator, Stage},
};
use anyhow::{Context, Result};
use eframe::{egui, egui_wgpu};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;

pub const LAYERS: [&str; 31] = [
    "Natural world",
    "Elevation",
    "Tectonic plates",
    "Geology",
    "Resources",
    "Soil",
    "Temperature",
    "Precipitation",
    "Wind",
    "Snow",
    "Drainage",
    "Rivers",
    "Lakes",
    "Biomes",
    "Vegetation",
    "Solar production",
    "Geochemical production",
    "Available nitrogen",
    "Available phosphorus",
    "Growth constraint",
    "Canopy biomass",
    "Understory biomass",
    "Root mat biomass",
    "Shallow underground biomass",
    "Deep fault biomass",
    "Animal biomass",
    "Lake currents",
    "Upwelling",
    "Deep-water phosphorus",
    "Plankton biomass",
    "Recycling inventory",
];
const LEGENDS: [&str; 31] = [
    "Land cover · relief · water",
    "−4,000 m ocean → 6,000 m peaks",
    "16 moving plates · categorical colors",
    "Exposed rock family · blue water · select for strata",
    "Dark → gold: low → high deposit potential",
    "Soil type · select a cell for properties",
    "Blue −30 °C → red 45 °C",
    "Dry 0 → wet 3,000 mm/year",
    "Blue westward → orange eastward",
    "Dark 0 → white 2 m snow",
    "Basin colors · brighter land = longer route",
    "Logarithmic discharge · m³/s",
    "Logarithmic water depth · m",
    "Climate-defined habitat classes",
    "Bare 0% → green 100% cover",
    "kg C/m²/month · dark 0 → bright 0.1",
    "kg C/m²/month · dark 0 → bright 0.01",
    "kg N/m² · dark 0 → bright 0.02",
    "kg P/m² · dark 0 → bright 0.004",
    "Green energy/habitat · orange nitrogen · purple phosphorus",
    "kg C/m² · logarithmic scale",
    "kg C/m² · logarithmic scale",
    "kg C/m² · logarithmic scale",
    "kg C/m² · logarithmic scale",
    "kg C/m² · logarithmic scale",
    "kg C/m² · logarithmic scale",
    "Wind-driven basin circulation direction",
    "Vertical exchange m/month",
    "kg P/m² · dark 0 → bright 0.1",
    "kg C/m² · dark 0 → bright 0.01",
    "Detrital carbon kg/m²",
];
fn legend_colors(layer: u32) -> Option<([u8; 3], [u8; 3])> {
    match layer {
        4 => Some(([9, 11, 17], [242, 166, 51])),
        6 => Some(([41, 89, 204], [250, 77, 31])),
        7 => Some(([173, 107, 51], [33, 166, 204])),
        8 => Some(([51, 115, 204], [230, 140, 64])),
        9 => Some(([20, 31, 41], [235, 247, 255])),
        11 => Some(([31, 41, 36], [56, 204, 255])),
        12 => Some(([36, 46, 41], [38, 166, 230])),
        14 => Some(([102, 77, 51], [26, 166, 71])),
        15..=18 | 20..=25 | 27..=30 => Some(([6, 10, 20], [64, 230, 166])),
        _ => None,
    }
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewParams {
    dims: [u32; 4],
    camera: [f32; 4],
    pan: [f32; 4],
    counts: [u32; 4],
    extra: [u32; 4],
}
#[derive(Clone, Copy)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub pan: [f32; 2],
    pub globe: bool,
}
impl Camera {
    /// Center either projection on the same spherical point for regional inspection.
    pub fn focus(&mut self, direction: [f32; 3]) {
        self.yaw = direction[0].atan2(direction[2]);
        self.pitch = direction[1].clamp(-1., 1.).asin();
        self.pan = [
            self.yaw / std::f32::consts::TAU,
            -self.pitch / std::f32::consts::PI,
        ];
        self.zoom = if self.globe { 4. } else { 8. };
    }
    pub fn globe() -> Self {
        Self {
            yaw: 0.,
            pitch: 0.,
            zoom: 0.88,
            pan: [0.; 2],
            globe: true,
        }
    }
    pub fn atlas() -> Self {
        Self {
            yaw: 0.,
            pitch: 0.,
            zoom: 1.,
            pan: [0.; 2],
            globe: false,
        }
    }
}
pub struct MapRenderer {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
    uniform: wgpu::Buffer,
    groups: [wgpu::BindGroup; 4],
    pipeline: wgpu::ComputePipeline,
}
impl MapRenderer {
    pub fn new(generator: &Generator, width: u32, height: u32) -> Result<Self> {
        let d = &generator.gpu.device;
        d.push_error_scope(wgpu::ErrorFilter::Validation);
        let texture = d.create_texture(&wgpu::TextureDescriptor {
            label: Some("World map"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });
        let storage_view = texture.create_view(&Default::default());
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            ..Default::default()
        });
        let uniform = d.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Map settings"),
            contents: &[0; 80],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let source = format!(
            "{}\n{}\n{}",
            include_str!("../shaders/simulation.wgsl")
                .split("struct Params")
                .next()
                .unwrap(),
            include_str!("../shaders/view.wgsl"),
            include_str!("../shaders/regional.wgsl")
        );
        let module = d.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Globe and atlas"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let pipeline = d.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Map rendering"),
            layout: None,
            module: &module,
            entry_point: Some("render"),
            compilation_options: Default::default(),
            cache: None,
        });
        let layout = pipeline.get_bind_group_layout(0);
        let groups = std::array::from_fn(|i| {
            d.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: generator.buffers[i / 2].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: generator.catalog_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&storage_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: generator.ecology.buffers[i % 2].as_entire_binding(),
                    },
                ],
            })
        });
        if let Some(e) = pollster::block_on(d.pop_error_scope()) {
            anyhow::bail!("Map shader failed: {e}");
        }
        Ok(Self {
            texture,
            view,
            width,
            height,
            uniform,
            groups,
            pipeline,
        })
    }
    pub fn render(&self, generator: &Generator, layer: u32, camera: Camera, selected: Option<u32>) {
        let params = ViewParams {
            dims: [generator.config.resolution, self.width, self.height, layer],
            camera: [
                camera.yaw,
                camera.pitch,
                camera.zoom,
                if camera.globe { 1. } else { 0. },
            ],
            pan: [camera.pan[0], camera.pan[1], 0., 0.],
            counts: generator.catalog.counts(),
            extra: [
                generator.catalog.biomes.len() as u32,
                selected.unwrap_or(gpu::NONE),
                generator.config.eco_resolution(),
                generator.config.seed,
            ],
        };
        generator
            .gpu
            .queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&params));
        let mut encoder = generator
            .gpu
            .device
            .create_command_encoder(&Default::default());
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(
            0,
            &self.groups[generator.current * 2 + generator.ecology.current],
            &[],
        );
        pass.dispatch_workgroups(self.width.div_ceil(8), self.height.div_ceil(8), 1);
        drop(pass);
        generator.gpu.queue.submit(Some(encoder.finish()));
    }
    /// Generate fine regional fields and export both a PNG and regional archive.
    pub fn export_region(
        &self,
        generator: &Generator,
        center: [f32; 3],
        width_km: f32,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        anyhow::ensure!(
            self.width == self.height,
            "regional exports require a square renderer"
        );
        let length = center.iter().map(|v| v * v).sum::<f32>().sqrt();
        anyhow::ensure!(
            length.is_finite() && length > 0. && width_km.is_finite() && width_km > 0.,
            "invalid regional extent or center"
        );
        let region = generator.generate_region(
            center,
            width_km,
            self.width.min(1024).next_power_of_two(),
        )?;
        region.save(path.as_ref().with_extension("region.json"))?;
        region.export_png(path)
    }

    pub fn export_png(&self, generator: &Generator, path: impl AsRef<Path>) -> Result<()> {
        let padded = (self.width * 4).div_ceil(256) * 256;
        let size = padded as u64 * self.height as u64;
        let read = generator.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PNG export"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = generator
            .gpu
            .device
            .create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &read,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        generator.gpu.queue.submit(Some(encoder.finish()));
        let data = gpu::map_buffer(&generator.gpu.device, &read)?;
        let rgba = data
            .chunks_exact(padded as usize)
            .flat_map(|row| row[..self.width as usize * 4].iter().copied())
            .collect::<Vec<_>>();
        if let Some(parent) = path.as_ref().parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        image::save_buffer(
            path,
            &rgba,
            self.width,
            self.height,
            image::ColorType::Rgba8,
        )?;
        Ok(())
    }
}
pub fn pick(uv: [f32; 2], aspect: f32, camera: Camera, n: u32) -> Option<u32> {
    let d = if camera.globe {
        let x = (uv[0] - 0.5) * 2. * aspect / camera.zoom;
        let y = -(uv[1] - 0.5) * 2. / camera.zoom;
        let r = x * x + y * y;
        if r > 1. {
            return None;
        }
        let z = (1. - r).sqrt();
        let yy = y * camera.pitch.cos() + z * camera.pitch.sin();
        let zz = -y * camera.pitch.sin() + z * camera.pitch.cos();
        [
            x * camera.yaw.cos() + zz * camera.yaw.sin(),
            yy,
            -x * camera.yaw.sin() + zz * camera.yaw.cos(),
        ]
    } else {
        let lon = ((uv[0] - 0.5) / camera.zoom + camera.pan[0]) * std::f32::consts::TAU;
        let lat = (-(uv[1] - 0.5) / camera.zoom - camera.pan[1]) * std::f32::consts::PI;
        if lat.abs() > std::f32::consts::FRAC_PI_2 {
            return None;
        }
        [lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
    };
    Some(crate::grid::index(d, n))
}
struct App {
    history_selection: Option<usize>,
    history_search: String,
    history_tab: u8,
    timeline_view: crate::history_timeline::TimelineView,
    political_overlay: bool,
    historical_territory: bool,
    history_window_open: bool,
    journey_overlay: Option<u64>,
    expedition_overlay: Option<u32>,
    event_export_months: u32,
    territory_export_month: u32,
    cultural_overlay: u32,
    founding_options: crate::culture::FoundingOptions,
    region: Option<(crate::region::Region, egui::TextureHandle)>,
    regional_selection: Option<usize>,
    ecology_only: bool,
    baseline_report: Option<crate::ecology::BudgetReport>,
    current_report: Option<crate::ecology::BudgetReport>,
    scenario_guild: u32,
    scenario_region: u32,
    scenario_mixing: f32,
    generator: Generator,
    draft: Config,
    render_state: egui_wgpu::RenderState,
    maps: [MapRenderer; 2],
    textures: [egui::TextureId; 2],
    cameras: [Camera; 2],
    layer: u32,
    running: bool,
    continuous: bool,
    steps: u32,
    selected: Option<(u32, Cell)>,
    message: String,
    path: String,
    pending_save: bool,
    view_mode: u32,
    smoke: Option<std::path::PathBuf>,
    screenshot_requested: bool,
    smoke_started: Instant,
}
impl App {
    fn new(
        cc: &eframe::CreationContext<'_>,
        config: Config,
        catalog: Catalog,
        load: Option<std::path::PathBuf>,
        smoke: Option<std::path::PathBuf>,
    ) -> Result<Self> {
        let state = cc
            .wgpu_render_state
            .clone()
            .context("wgpu renderer unavailable")?;
        anyhow::ensure!(
            state.adapter.get_info().device_type != wgpu::DeviceType::Cpu,
            "hardware GPU required"
        );
        let gpu = ContextGpu {
            device: state.device.clone(),
            queue: state.queue.clone(),
            adapter_name: state.adapter.get_info().name,
            adapter_info: state.adapter.get_info(),
        };
        let requested_systems = config.systems.clone();
        let loaded = load.is_some();
        let mut generator = if let Some(path) = load {
            Generator::load(gpu, path)?
        } else {
            Generator::new(gpu, config, catalog)?
        };
        if loaded && !requested_systems.overrides.is_empty() {
            let mut systems = generator.config.systems.clone();
            systems.overrides.extend(requested_systems.overrides);
            if generator.civilizations.is_some() {
                generator.apply_systems(&systems)?;
            } else {
                systems.validate()?;
                generator.config.systems = systems;
            }
        }
        let maps = [
            MapRenderer::new(&generator, 768, 768)?,
            MapRenderer::new(&generator, 1536, 768)?,
        ];
        let textures = std::array::from_fn(|i| {
            state.renderer.write().register_native_texture(
                &state.device,
                &maps[i].view,
                wgpu::FilterMode::Linear,
            )
        });
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(17, 24, 32);
        visuals.window_fill = visuals.panel_fill;
        visuals.selection.bg_fill = egui::Color32::from_rgb(34, 95, 99);
        cc.egui_ctx.set_visuals(visuals);
        let running = smoke.is_some() && generator.civilizations.is_none();
        let history_selection = generator.civilizations.as_ref().map(|_| 0);
        Ok(Self {
            history_selection,
            history_search: String::new(),
            history_tab: 0,
            timeline_view: Default::default(),
            political_overlay: true,
            historical_territory: false,
            history_window_open: true,
            journey_overlay: None,
            expedition_overlay: None,
            event_export_months: 120,
            territory_export_month: 0,
            cultural_overlay: 0,
            founding_options: Default::default(),
            region: None,
            regional_selection: None,
            ecology_only: false,
            baseline_report: None,
            current_report: None,
            scenario_guild: 3,
            scenario_region: 4,
            scenario_mixing: 1.,
            draft: generator.config.clone(),
            generator,
            render_state: state,
            maps,
            textures,
            cameras: [Camera::globe(), Camera::atlas()],
            layer: 0,
            running,
            continuous: false,
            steps: 4,
            selected: None,
            message: "Paused. Evolve to advance time, or inspect the world.".into(),
            path: "output/planet.world".into(),
            pending_save: false,
            view_mode: 0,
            smoke,
            screenshot_requested: false,
            smoke_started: Instant::now(),
        })
    }
    fn replace(&mut self, generator: Generator) -> Result<()> {
        let maps = [
            MapRenderer::new(&generator, 768, 768)?,
            MapRenderer::new(&generator, 1536, 768)?,
        ];
        let mut renderer = self.render_state.renderer.write();
        for id in self.textures {
            renderer.free_texture(&id);
        }
        self.textures = std::array::from_fn(|i| {
            renderer.register_native_texture(
                &self.render_state.device,
                &maps[i].view,
                wgpu::FilterMode::Linear,
            )
        });
        drop(renderer);
        self.maps = maps;
        self.expedition_overlay = None;
        self.historical_territory = false;
        self.history_window_open = true;
        self.journey_overlay = None;
        self.draft = generator.config.clone();
        self.region = None;
        self.regional_selection = None;
        self.history_selection = None;
        self.generator = generator;
        self.selected = None;
        self.baseline_report = None;
        self.current_report = None;
        self.running = false;
        self.pending_save = false;
        Ok(())
    }
    fn report(&mut self, result: Result<()>, success: &str) {
        self.message = match result {
            Ok(()) => {
                if let Some((id, _)) = self.selected {
                    if let Ok(c) = self.generator.inspect(id) {
                        self.selected = Some((id, c));
                    }
                }
                success.into()
            }
            Err(e) => format!("{e:#}"),
        };
    }
    fn map(&mut self, ui: &mut egui::Ui, which: usize, size: egui::Vec2) {
        self.maps[which].render(
            &self.generator,
            self.layer,
            self.cameras[which],
            self.selected.map(|s| s.0),
        );
        let response = ui.add(
            egui::Image::new((self.textures[which], size)).sense(egui::Sense::click_and_drag()),
        );
        if which == 1 {
            if let Some(e) = self
                .generator
                .civilizations
                .as_ref()
                .and_then(|h| h.expeditions.as_ref())
                .and_then(|x| {
                    self.expedition_overlay
                        .and_then(|id| x.voyages.get(id as usize).map(|e| (x, e)))
                })
            {
                let (x, e) = e;
                let cells = e
                    .planned_cells
                    .as_deref()
                    .unwrap_or(&x.routes[e.route as usize].cells);
                let points: Vec<_> = cells
                    .iter()
                    .map(|&cell| {
                        let d = crate::grid::cell_direction(cell, self.generator.config.resolution);
                        [
                            d[0].atan2(d[2]).to_degrees() as f64,
                            d[1].asin().to_degrees() as f64,
                        ]
                    })
                    .collect();
                let camera = self.cameras[1];
                let project = |p: [f64; 2]| {
                    let q = egui::vec2(p[0] as f32 / 360. + 0.5, 0.5 - p[1] as f32 / 180.);
                    response.rect.min
                        + ((q - egui::vec2(0.5 + camera.pan[0], 0.5 + camera.pan[1])) * camera.zoom
                            + egui::vec2(0.5, 0.5))
                            * size
                };
                let painter = ui.painter().with_clip_rect(response.rect);
                for segment in crate::spatial::split_dateline(&points) {
                    painter.add(egui::Shape::line(
                        segment.into_iter().map(project).collect(),
                        egui::Stroke::new(2., egui::Color32::GOLD),
                    ));
                }
                if let Some(&p) = points.last() {
                    if matches!(
                        e.phase,
                        crate::expeditions::Phase::Camp | crate::expeditions::Phase::Stranded
                    ) {
                        let color = if e.phase == crate::expeditions::Phase::Stranded {
                            egui::Color32::RED
                        } else {
                            egui::Color32::LIGHT_GREEN
                        };
                        painter.circle_filled(project(p), 5., color);
                        painter.text(
                            project(p) + egui::vec2(8., 0.),
                            egui::Align2::LEFT_CENTER,
                            format!("{:?}", e.phase),
                            egui::FontId::proportional(12.),
                            color,
                        );
                    }
                    painter.circle_stroke(
                        project(p),
                        5.,
                        egui::Stroke::new(2., egui::Color32::GOLD),
                    );
                }
                if let Some(find) = e.heritage.as_ref().and_then(|c| c.find.as_ref()) {
                    let d =
                        crate::grid::cell_direction(find.cell, self.generator.config.resolution);
                    let pos = project([
                        d[0].atan2(d[2]).to_degrees() as f64,
                        d[1].asin().to_degrees() as f64,
                    ]);
                    painter.circle_filled(pos, 4., egui::Color32::LIGHT_BLUE);
                    painter.text(
                        pos + egui::vec2(8., 12.),
                        egui::Align2::LEFT_CENTER,
                        "Heritage find",
                        egui::FontId::proportional(12.),
                        egui::Color32::LIGHT_BLUE,
                    );
                }
            }
        }
        if which == 1 {
            if let Some(history) = &self.generator.civilizations {
                let camera = self.cameras[1];
                if self.political_overlay
                    && self.cultural_overlay == 0
                    && !self.historical_territory
                {
                    if let Some(p) = &history.politics {
                        let painter = ui.painter().with_clip_rect(response.rect);
                        for claim in &p.claims {
                            let d = crate::grid::cell_direction(
                                claim.cell,
                                self.generator.config.resolution,
                            );
                            let q = egui::vec2(
                                d[0].atan2(d[2]) / std::f32::consts::TAU + 0.5,
                                0.5 - d[1].asin() / std::f32::consts::PI,
                            );
                            let uv = (q - egui::vec2(0.5 + camera.pan[0], 0.5 + camera.pan[1]))
                                * camera.zoom
                                + egui::vec2(0.5, 0.5);
                            let pos = response.rect.min + uv * size;
                            let owners = p.claim_owners(claim);
                            let color = if owners.len() > 1 {
                                egui::Color32::WHITE
                            } else {
                                political_color(*owners.first().unwrap())
                            };
                            painter.circle_filled(
                                pos,
                                (size.x * camera.zoom / self.generator.config.resolution as f32
                                    * 0.2)
                                    .clamp(1.5, 10.),
                                color.gamma_multiply(0.45),
                            );
                        }
                    }
                }
                for site in history.sites.iter().filter(|_| !self.historical_territory) {
                    let d =
                        crate::grid::cell_direction(site.cell, self.generator.config.resolution);
                    let q = egui::vec2(
                        d[0].atan2(d[2]) / std::f32::consts::TAU + 0.5,
                        0.5 - d[1].asin() / std::f32::consts::PI,
                    );
                    let uv = (q - egui::vec2(0.5 + camera.pan[0], 0.5 + camera.pan[1]))
                        * camera.zoom
                        + egui::vec2(0.5, 0.5);
                    let pos = response.rect.min + uv * size;
                    if response.rect.contains(pos) {
                        let color = if site.abandoned {
                            egui::Color32::GRAY
                        } else if self.cultural_overlay == 1 {
                            political_color(
                                history
                                    .culture
                                    .as_ref()
                                    .map_or(site.civilization, |c| c.site_faith[site.id as usize]),
                            )
                        } else if self.cultural_overlay == 2 {
                            let role = history
                                .settlement_roles(site.id)
                                .first()
                                .map_or("", |r| r.0);
                            political_color(
                                role.bytes()
                                    .fold(0u32, |s, b| s.wrapping_mul(31).wrapping_add(b as u32)),
                            )
                        } else if history.politics.is_some() && self.political_overlay {
                            political_color(history.controller(site.id))
                        } else {
                            egui::Color32::from_rgb(255, 180, 70)
                        };
                        ui.painter().circle_filled(pos, 3., color);
                    }
                }
            }
        }
        if which == 1 {
            if let Some(h) = &self.generator.civilizations {
                crate::history_atlas::draw(
                    ui.painter(),
                    &crate::history_atlas::Projection {
                        rect: response.rect,
                        pan: self.cameras[1].pan,
                        zoom: self.cameras[1].zoom,
                        resolution: self.generator.config.resolution,
                    },
                    h,
                    self.historical_territory
                        .then_some(self.territory_export_month),
                    self.journey_overlay,
                );
            }
        }
        if response.dragged() {
            let delta = ui.input(|i| i.pointer.delta());
            let camera = &mut self.cameras[which];
            if camera.globe {
                camera.yaw -= delta.x * 0.006;
                camera.pitch = (camera.pitch + delta.y * 0.006).clamp(-1.5, 1.5);
            } else {
                camera.pan[0] -= delta.x / size.x / camera.zoom;
                camera.pan[1] -= delta.y / size.y / camera.zoom;
            }
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            self.cameras[which].zoom = (self.cameras[which].zoom * (scroll * 0.002).exp())
                .clamp(if which == 0 { 0.4 } else { 1. }, 12.);
        }
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let uv = [
                    (pos.x - response.rect.left()) / size.x,
                    (pos.y - response.rect.top()) / size.y,
                ];
                if let Some(id) = pick(
                    uv,
                    if which == 0 { 1. } else { 2. },
                    self.cameras[which],
                    self.generator.config.resolution,
                ) {
                    match self.generator.inspect(id) {
                        Ok(c) => {
                            self.selected = Some((id, c));
                            if let Some(h) = &self.generator.civilizations {
                                self.history_selection = h.sites.iter().position(|s| s.cell == id);
                            }
                        }
                        Err(e) => self.message = e.to_string(),
                    }
                }
            }
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(path) = self.smoke.clone() {
            ctx.request_repaint();
            if (self.generator.progress.epoch >= 2 || self.generator.civilizations.is_some())
                && !self.screenshot_requested
                && self.smoke_started.elapsed().as_secs_f32() > 1.
            {
                self.running = false;
                let id = crate::grid::index([0.45, 0.1, 0.9], self.generator.config.resolution);
                if let Ok(cell) = self.generator.inspect(id) {
                    self.selected = Some((id, cell));
                }
                if let Some(h) = &self.generator.civilizations {
                    if h.culture.is_some() {
                        self.history_tab = 1;
                    }
                    self.historical_territory = !h.territorial_history.is_empty();
                    self.territory_export_month = h.month;
                    self.journey_overlay = h
                        .events
                        .iter()
                        .rev()
                        .find(|e| e.planned_path.is_some())
                        .map(|e| e.id);
                    self.view_mode = 2;

                    if let Some(site) = h.sites.first() {
                        self.history_selection = Some(0);
                        let d = crate::grid::cell_direction(
                            site.cell,
                            self.generator.config.resolution,
                        );
                        for camera in &mut self.cameras {
                            camera.focus(d);
                        }
                    }
                    self.history_window_open = false;
                    self.cameras[1] = Camera::atlas();
                } else if let Ok(cells) = self.generator.snapshot() {
                    if let Some(id) = cells
                        .iter()
                        .position(|c| c.meta[0] == 2 && c.water[0] < 0.1)
                    {
                        let center = crate::grid::cell_direction(
                            id as u32,
                            self.generator.config.resolution,
                        );
                        match self.generator.generate_region(
                            center,
                            300f32.min(self.generator.config.radius_km * 0.5),
                            128,
                        ) {
                            Ok(region) => {
                                let image = region.image();
                                let texture = ctx.load_texture(
                                    "smoke regional terrain",
                                    egui::ColorImage::from_rgb([128, 128], image.as_raw()),
                                    egui::TextureOptions::LINEAR,
                                );
                                self.region = Some((region, texture));
                                self.regional_selection = Some(128 * 64 + 64);
                            }
                            Err(e) => {
                                eprintln!("Regional smoke test failed: {e}");
                                self.message = e.to_string();
                            }
                        }
                    }
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                self.screenshot_requested = true;
            }
            for event in ctx.input(|i| i.events.clone()) {
                if let egui::Event::Screenshot { image, .. } = event {
                    let result = (|| -> Result<()> {
                        if let Some(parent) = path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        let rgba = image
                            .pixels
                            .iter()
                            .flat_map(|p| p.to_array())
                            .collect::<Vec<_>>();
                        image::save_buffer(
                            &path,
                            &rgba,
                            image.size[0] as u32,
                            image.size[1] as u32,
                            image::ColorType::Rgba8,
                        )?;
                        Ok(())
                    })();
                    if let Err(e) = result {
                        eprintln!("Screenshot failed: {e}");
                    } else {
                        eprintln!("Desktop smoke test saved {}", path.display());
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            ctx.request_repaint();
        }
        if self.running {
            let start = Instant::now();
            for _ in 0..self.steps {
                if self.generator.civilizations.is_none()
                    && !self.ecology_only
                    && !self.continuous
                    && self.generator.progress.epoch >= self.generator.config.target_epochs
                    && self.generator.progress.stage == Stage::Boundary
                {
                    self.running = false;
                    break;
                }
                match if self
                    .generator
                    .civilizations
                    .as_ref()
                    .is_some_and(|h| h.living.is_some())
                {
                    self.generator.advance_history(1).map(|_| true)
                } else if self.ecology_only {
                    self.generator.advance_ecology().map(|_| true)
                } else {
                    self.generator.advance()
                } {
                    Ok(boundary) => {
                        if boundary {
                            if self.smoke.is_some() && self.generator.progress.epoch >= 2 {
                                self.running = false;
                                break;
                            }
                            if let Some((id, _)) = self.selected {
                                if let Ok(c) = self.generator.inspect(id) {
                                    self.selected = Some((id, c));
                                }
                            }
                            if self.pending_save {
                                self.running = false;
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        self.message = e.to_string();
                        self.running = false;
                        break;
                    }
                }
                if start.elapsed() > Duration::from_millis(24) {
                    break;
                }
            }
            ctx.request_repaint();
        }
        if self.pending_save && self.generator.progress.stage == Stage::Boundary {
            let result = self.generator.save(&self.path);
            self.report(result, "World saved at the epoch boundary.");
            self.pending_save = false;
        }
        egui::TopBottomPanel::top("masthead").show(ctx, |ui| {
            ui.add_space(10.);
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("ANCIENT WORLD")
                        .color(egui::Color32::from_rgb(222, 209, 170)),
                );
                ui.label(" /  A planet in the making");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(h) = self
                        .generator
                        .civilizations
                        .as_ref()
                        .filter(|h| h.living.is_some())
                    {
                        ui.label(format!(
                            "YEAR {} · MONTH {} · LIVING WORLD",
                            h.month / 12,
                            h.month % 12 + 1
                        ));
                    } else {
                        ui.label(format!(
                            "EPOCH {:04}   ·   {:0.2} Myr",
                            self.generator.progress.epoch,
                            self.generator.progress.geological_time_myr
                        ));
                    }
                });
            });
            ui.add_space(10.);
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.small(&self.message);
            });
        });
        egui::SidePanel::left("controls")
            .default_width(280.)
            .resizable(false)
            .show(ctx, |ui| self.sidebar(ui));
        egui::CentralPanel::default().show(ctx, |ui| self.central(ui));
        let mut region_open = true;
        let mut local_recovery = None;
        if let Some((region, texture)) = &self.region {
            egui::Window::new("Regional terrain").open(&mut region_open).vscroll(true).show(ctx, |ui| {
                ui.label(format!("{} × {} cells · {:.0} m nominal spacing · epoch {} snapshot", region.resolution, region.resolution, region.width_km*1000./region.resolution as f32, region.epoch));
                ui.small("Fine terrain, local drainage and one-year runoff. Boundary outlets leave this patch.");
                if let Some(month)=region.history_month {ui.small(format!("History month {month} snapshot; refresh after advancing history. Recovery uses the next available quarterly cultural work."));}
                let size=ui.available_width().min(640.);
                let response=ui.add(egui::Image::new((texture.id(),egui::vec2(size,size))).sense(egui::Sense::click()));
                if let Some(pos)=response.interact_pointer_pos().filter(|_|response.clicked()) {
                    let q=(pos-response.rect.min)/response.rect.size();let n=region.resolution as usize;
                    let x=(q.x*n as f32) as usize;let y=((1.-q.y)*n as f32) as usize;
                    self.regional_selection=Some(y.min(n-1)*n+x.min(n-1));
                }
                if let Some(i)=self.regional_selection {
                    let c=&region.cells[i];ui.label(format!("Cell {i}: elevation {:.1} m · soil {:.2} m · water {:.2} m",c.surface[0],c.surface[2],c.water[0]));
                    ui.label(format!("{:.1} °C · {:.0} mm/year · flow {:.3} m³/s · downstream {:?}",c.climate[0],c.climate[1],c.water[2]/31557600.,(c.route[0]!=u32::MAX).then_some(c.route[0])));
                    if c.strata.iter().any(|v|*v>0.) {
                        for k in 0..3 { if let Some(rock)=region.catalog.rocks.get(c.rocks[k] as usize) { ui.label(format!("Parent unit {}: {} · {:.2} m",k+1,rock.name,c.strata[k])); } }
                        ui.small("Inherited planetary column; no local folding or excavation mesh.");
                    }
                    if let Some(m)=c.mineral_index().and_then(|id|region.catalog.minerals.get(id)) {
                        ui.label(format!("Inherited prospect: {} · {} · {:.1}% potential · ~{:.0} m below local surface",m.name,m.deposit_setting.label(),c.forcing[2]*100.,c.forcing[3]));
                        ui.small("Regional prospect snapshot; registered shared sources retain canonical finite quantities.");
                    }
                    for d in region.processing_deposits.iter().filter(|d|d.cell==c.route[2]) {ui.small(format!("Site {} processing residue: {:.1}/{:.1} kg · {}",d.site,d.kg,d.capacity_kg,if d.abandoned {"retained in ruins"} else {"active deposit"}));}
                    let plant=region.catalog.plants.get(c.ids[2] as usize).map(|p|p.id.as_str()).unwrap_or("none");ui.label(format!("Plant: {plant} · parent planet cell {}",c.route[2]));

                    for place in region.historical_sites.iter().filter(|s|s.cell==c.route[2]) {
                        ui.separator();
                        ui.label(format!("{} · {} · founded month {}",place.name,if place.abandoned {"ruins"} else {"settlement"},place.founded));
                        ui.small("Historical snapshot at parent-cell scale; building positions have not been generated.");
                        for object in region.historical_artifacts.iter().filter(|a|a.site==Some(place.id)) {
                            ui.label(format!("{} · {} · {} recorded events",object.name,if object.destroyed {"destroyed remains"} else if object.lost {"lost in ruins"} else {"in custody"},object.events.len()));
                            if object.lost && !object.destroyed {
                                if let Some(h)=self.generator.civilizations.as_ref() {
                                    if let Some(culture)=h.culture.as_ref() {
                                        if let Some(actor)=culture.site_people(h,place.id).first() {
                                            if ui.button(format!("Commission recovery of object {}",object.id)).clicked() {local_recovery=Some((place.id,*actor,object.id));}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
        if let Some((site, person, artifact)) = local_recovery {
            if let Some((region, _)) = &self.region {
                let result = self
                    .generator
                    .request_local_recovery(region, site, person, artifact);
                self.report(result, "Recovery queued for available cultural work");
            }
        }
        self.history_window(ctx);
        if !region_open {
            self.region = None;
            self.regional_selection = None;
        }
    }
}
impl App {
    fn sidebar(&mut self, ui: &mut egui::Ui) {
        let living = self
            .generator
            .civilizations
            .as_ref()
            .is_some_and(|h| h.living.is_some());
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(12.);
            ui.label(
                egui::RichText::new(if living {
                    "LIVING HISTORY"
                } else {
                    "NATURAL HISTORY"
                })
                .strong()
                .color(egui::Color32::from_rgb(132, 185, 173)),
            );
            ui.add_space(8.);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        self.generator
                            .civilizations
                            .as_ref()
                            .is_none_or(|h| h.living.is_some()),
                        egui::Button::new(if self.running { "Pause" } else { "▶ Evolve" }),
                    )
                    .clicked()
                {
                    self.running = !self.running;
                }
                if ui
                    .add_enabled(
                        self.generator
                            .civilizations
                            .as_ref()
                            .is_none_or(|h| h.living.is_some()),
                        egui::Button::new("Step"),
                    )
                    .clicked()
                {
                    self.running = false;
                    let result = if self
                        .generator
                        .civilizations
                        .as_ref()
                        .is_some_and(|h| h.living.is_some())
                    {
                        self.generator.advance_history(1)
                    } else if self.ecology_only {
                        self.generator.advance_ecology()
                    } else {
                        self.generator.advance().map(|_| ())
                    };
                    self.report(result, "Advanced one simulation batch.");
                }
            });
            if !living {
                ui.checkbox(&mut self.continuous, "Continue beyond target");
            }
            ui.add_enabled(
                self.generator.progress.stage == Stage::Boundary
                    && self.generator.civilizations.is_none(),
                egui::Checkbox::new(&mut self.ecology_only, "Ecology only (monthly steps)"),
            );
            ui.small(format!(
                "Ecological time: {:.2} years · {}² / face",
                self.generator.ecology.clock.month as f64 / 12.,
                self.generator.config.eco_resolution()
            ));
            ui.add(egui::Slider::new(&mut self.steps, 1..=64).text(if living {
                "Months / frame"
            } else {
                "Batches / frame"
            }));
            if living {
                let h = self.generator.civilizations.as_ref().unwrap();
                ui.label(format!(
                    "Year {} · month {}",
                    h.month / 12,
                    h.month % 12 + 1
                ));
                ui.small("Seasonal ecology active · geological terrain fixed");
            } else {
                ui.label(self.generator.progress.stage.label());
                ui.small(format!(
                    "Pass {} · {} changed cells",
                    self.generator.progress.iteration, self.generator.progress.changed
                ));
                let fraction = (self.generator.progress.epoch as f32
                    / self.generator.config.target_epochs.max(1) as f32)
                    .min(1.);
                ui.add(egui::ProgressBar::new(fraction).text(format!(
                    "{} / {} epochs",
                    self.generator.progress.epoch, self.generator.config.target_epochs
                )));
            }
            ui.add_space(14.);
            ui.separator();
            ui.label(egui::RichText::new("WORLD LAYERS").strong());
            if let Some(h) = &self.generator.civilizations {
                if ui.button("Open history window").clicked() {
                    self.history_window_open = true;
                }
                if ui
                    .checkbox(
                        &mut self.historical_territory,
                        "Historical territory on atlas",
                    )
                    .changed()
                    && self.historical_territory
                {
                    self.view_mode = 2;
                    self.history_window_open = false;
                    self.expedition_overlay = None;
                }
                if self.historical_territory {
                    ui.add(
                        egui::Slider::new(&mut self.territory_export_month, 0..=h.month)
                            .text("History month"),
                    );
                    ui.horizontal(|ui| {
                        if ui.small_button("Previous change").clicked() {
                            if let Some(s) = h
                                .territorial_history
                                .iter()
                                .rev()
                                .find(|s| s.month < self.territory_export_month)
                            {
                                self.territory_export_month = s.month;
                            }
                        }
                        if ui.small_button("Next change").clicked() {
                            if let Some(s) = h
                                .territorial_history
                                .iter()
                                .find(|s| s.month > self.territory_export_month)
                            {
                                self.territory_export_month = s.month;
                            }
                        }
                    });
                }
                egui::ComboBox::from_label("Saved household journey")
                    .selected_text(
                        self.journey_overlay
                            .map_or("None".into(), |id| format!("Departure #{id}")),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.journey_overlay, None, "None");
                        for e in h
                            .events
                            .iter()
                            .rev()
                            .filter(|e| e.planned_path.is_some())
                            .take(128)
                        {
                            if ui
                                .selectable_value(
                                    &mut self.journey_overlay,
                                    Some(e.id),
                                    format!("#{} · Y{} M{}", e.id, e.month / 12, e.month % 12 + 1),
                                )
                                .clicked()
                            {
                                self.expedition_overlay = None;
                                self.history_window_open = false;
                                self.view_mode = 2;
                                self.cameras[1] = Camera::atlas();
                            }
                        }
                    });
                ui.small("Latest 128 plans here; older plans are accessible through the timeline.");
            }

            if self
                .generator
                .civilizations
                .as_ref()
                .is_some_and(|h| h.politics.is_some())
            {
                ui.checkbox(&mut self.political_overlay, "Political claims on atlas");
                egui::ComboBox::from_label("Settlement colors")
                    .selected_text(
                        ["Politics", "Religion", "Economic role"][self.cultural_overlay as usize],
                    )
                    .show_ui(ui, |ui| {
                        for (i, name) in
                            ["Politics", "Religion", "Economic role"].iter().enumerate()
                        {
                            ui.selectable_value(&mut self.cultural_overlay, i as u32, *name);
                        }
                    });
                ui.small("Colors: administration · white: disputed claims");
            }
            egui::ComboBox::from_id_salt("layer")
                .selected_text(LAYERS[self.layer as usize])
                .width(245.)
                .show_ui(ui, |ui| {
                    for (i, name) in LAYERS.iter().enumerate() {
                        ui.selectable_value(&mut self.layer, i as u32, *name);
                    }
                });
            ui.small(LEGENDS[self.layer as usize]);
            if self.layer == 3 {
                ui.horizontal(|ui| {
                    for (name, rgb) in [
                        ("Igneous", [171, 99, 79]),
                        ("Sedimentary", [186, 166, 107]),
                        ("Metamorphic", [117, 135, 163]),
                    ] {
                        ui.colored_label(egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]), name);
                    }
                });
                ui.collapsing("Rock color key", |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(180.)
                        .show(ui, |ui| {
                            for rock in &self.generator.catalog.rocks {
                                let rgb = crate::catalog::rock_color(rock);
                                ui.colored_label(
                                    egui::Color32::from_rgb(
                                        (rgb[0] * 255.) as u8,
                                        (rgb[1] * 255.) as u8,
                                        (rgb[2] * 255.) as u8,
                                    ),
                                    &rock.name,
                                );
                            }
                        });
                });
                ui.small("Submerged rock remains available through cell inspection.");
            }

            if let Some((low, high)) = legend_colors(self.layer) {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(245., 9.), egui::Sense::hover());
                for k in 0..64 {
                    let t = k as f32 / 63.;
                    let rgb: [u8; 3] = std::array::from_fn(|i| {
                        ((low[i] as f32) * (1. - t) + (high[i] as f32) * t) as u8
                    });
                    let r = egui::Rect::from_min_max(
                        egui::pos2(rect.left() + rect.width() * k as f32 / 64., rect.top()),
                        egui::pos2(
                            rect.left() + rect.width() * (k + 1) as f32 / 64.,
                            rect.bottom(),
                        ),
                    );
                    ui.painter().rect_filled(
                        r,
                        0.,
                        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                    );
                }
            }
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.view_mode, 0, "Both");
                ui.selectable_value(&mut self.view_mode, 1, "Globe");
                ui.selectable_value(&mut self.view_mode, 2, "Atlas");
            });
            if ui
                .add_enabled(
                    self.selected.is_some(),
                    egui::Button::new("Focus selected region"),
                )
                .clicked()
            {
                if let Some((id, _)) = self.selected {
                    let d = crate::grid::cell_direction(id, self.generator.config.resolution);
                    for camera in &mut self.cameras {
                        camera.focus(d);
                    }
                }
            }
            if ui.small_button("Reset cameras").clicked() {
                self.cameras = [Camera::globe(), Camera::atlas()];
            }
            ui.add_space(12.);
            ui.collapsing("Civilizations beta", |ui| self.civilization_panel(ui));
            ui.collapsing("Generation settings", |ui| self.settings(ui));
            ui.collapsing("Save, resume & export", |ui| self.files(ui));
            ui.collapsing("Ecological experiments", |ui| self.experiments(ui));
            ui.collapsing("GPU diagnostics", |ui| self.diagnostics(ui));
        });
    }
    fn experiments(&mut self, ui: &mut egui::Ui) {
        use crate::ecology::{Intervention, GUILD_NAMES};
        ui.small("Events apply at completed epoch or ecology-only monthly boundaries.");
        egui::ComboBox::from_id_salt("scenario_region")
            .selected_text(
                [
                    "Ocean",
                    "Great lake",
                    "Inner continents",
                    "Outer continent",
                    "Everywhere",
                ][self.scenario_region as usize],
            )
            .show_ui(ui, |ui| {
                for (i, n) in [
                    "Ocean",
                    "Great lake",
                    "Inner continents",
                    "Outer continent",
                    "Everywhere",
                ]
                .iter()
                .enumerate()
                {
                    ui.selectable_value(&mut self.scenario_region, i as u32, *n);
                }
            });
        let region = (self.scenario_region < 4).then_some(self.scenario_region);
        let mut event = None;
        let scenario_ready = self.generator.progress.stage == Stage::Boundary
            && self
                .generator
                .civilizations
                .as_ref()
                .is_none_or(|h| h.living.as_ref().is_some_and(|l| !l.incomplete));
        if let Some(h) = &self.generator.civilizations {
            ui.label(if h.living.is_some() {
                "Scenarios apply at the current shared monthly boundary."
            } else {
                "Enable living history to apply ecological scenarios."
            });
        }
        ui.add_enabled_ui(scenario_ready, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Disable geochemistry").clicked() {
                    event = Some(Intervention::GeochemicalSupply(false));
                }
                if ui.button("Restore supply").clicked() {
                    event = Some(Intervention::GeochemicalSupply(true));
                }
            });
            egui::ComboBox::from_id_salt("guild")
                .selected_text(GUILD_NAMES[self.scenario_guild as usize])
                .show_ui(ui, |ui| {
                    for (i, n) in GUILD_NAMES.iter().enumerate() {
                        ui.selectable_value(&mut self.scenario_guild, i as u32, *n);
                    }
                });
            ui.horizontal(|ui| {
                if ui.button("Remove guild").clicked() {
                    event = Some(Intervention::RemoveGuild(self.scenario_guild));
                }
                if ui.button("Allow recovery").clicked() {
                    event = Some(Intervention::RestoreGuild(self.scenario_guild));
                }
            });
            ui.add(egui::Slider::new(&mut self.scenario_mixing, 0.0..=20.).text("Lake mixing"));
            if ui.button("Apply mixing").clicked() {
                event = Some(Intervention::LakeMixing(self.scenario_mixing));
            }
        });
        if let Some(event) = event {
            self.generator.ecology.prepare(
                &self.generator.gpu,
                &self.generator.buffers[self.generator.current],
                &self.generator.catalog_buffer,
                &self.generator.config,
                &self.generator.catalog,
                false,
            );
            self.baseline_report = self
                .generator
                .ecology
                .budget(&self.generator.gpu, &self.generator.config)
                .ok();
            let result = self.generator.scenario(region, event);
            self.report(result, "Scenario recorded.");
        }
        if ui.button("Audit nutrient budgets").clicked() {
            match self
                .generator
                .ecology
                .budget(&self.generator.gpu, &self.generator.config)
            {
                Ok(b) => {
                    self.current_report = Some(b.clone());
                    self.message = format!(
                        "C/N/P relative budget residuals: {:.2e}, {:.2e}, {:.2e}",
                        b.relative_error[0], b.relative_error[1], b.relative_error[2]
                    )
                }
                Err(e) => self.message = e.to_string(),
            }
        }
        if let Some(report) = &self.current_report {
            if !report.within_tolerance {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    "Budget residual exceeds 0.1% tolerance",
                );
            }
            ui.small(format!(
                "Water residual {:.2e}",
                report.water_relative_error
            ));
            egui::Grid::new("regional_budget_comparison")
                .striped(true)
                .show(ui, |ui| {
                    ui.small("Region");
                    ui.small("Photo");
                    ui.small("Chemo");
                    ui.small("Biomass");
                    ui.end_row();
                    for r in &report.regions {
                        ui.small(&r.name);
                        ui.small(format!("{:.3}", r.photo_kg_c_per_m2_year));
                        ui.small(format!("{:.4}", r.chemo_kg_c_per_m2_year));
                        ui.small(format!("{:.3}", r.biomass_kg_c_per_m2));
                        ui.end_row();
                    }
                });
            for (i, r) in report.regions.iter().enumerate() {
                let before = self
                    .baseline_report
                    .as_ref()
                    .map(|b| b.regions[i].biomass_kg_c_per_m2);
                ui.small(format!(
                    "{}: photo {:.3} + chemo {:.4} kg C/m²/year; biomass {:.3}{}",
                    r.name,
                    r.photo_kg_c_per_m2_year,
                    r.chemo_kg_c_per_m2_year,
                    r.biomass_kg_c_per_m2,
                    before
                        .map(|v| format!(" (before {:.3})", v))
                        .unwrap_or_default()
                ));
            }
        }
        ui.small(format!(
            "{} recorded interventions",
            self.generator.ecology.clock.events.len()
        ));
    }
    fn settings(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("ecoresolution")
            .selected_text(format!("Ecology {}²", self.draft.ecology_resolution))
            .show_ui(ui, |ui| {
                for n in [64, 128, 256, 512, 1024] {
                    ui.selectable_value(
                        &mut self.draft.ecology_resolution,
                        n,
                        format!("{n}² / face"),
                    );
                }
            });
        ui.add(
            egui::DragValue::new(&mut self.draft.ecology_years_per_epoch)
                .range(1..=1000)
                .prefix("Ecological years / epoch "),
        );
        ui.label("Seed");
        ui.add(egui::DragValue::new(&mut self.draft.seed));
        egui::ComboBox::from_id_salt("resolution")
            .selected_text(format!(
                "{} × {} / face",
                self.draft.resolution, self.draft.resolution
            ))
            .show_ui(ui, |ui| {
                for n in [256, 512, 1024] {
                    ui.selectable_value(&mut self.draft.resolution, n, format!("{n} × {n}"));
                }
            });
        ui.add(egui::Slider::new(&mut self.draft.inner_continents, 2..=8).text("Inner continents"));
        ui.add(
            egui::DragValue::new(&mut self.draft.radius_km)
                .range(100.0..=100000.0)
                .prefix("Radius km "),
        );
        ui.add(egui::Slider::new(&mut self.draft.axial_tilt, 0.0..=90.0).text("Tilt °"));
        egui::CollapsingHeader::new("Island abundance").show(ui, |ui| {
            ui.small("Applies when generating a new planet. Existing inventories are preserved.");
            ui.add(
                egui::Slider::new(&mut self.draft.island_phosphorus_scale, 0.01..=1.)
                    .text("Island phosphorus"),
            );
            ui.add(
                egui::Slider::new(&mut self.draft.settlement_plot_hectares, 20. ..=5000.)
                    .logarithmic(true)
                    .text("Town plot hectares"),
            );
            ui.add(
                egui::Slider::new(&mut self.draft.crop_yield_scale, 0.1..=1.)
                    .text("Attainable crop yield"),
            );
        });
        ui.add(
            egui::DragValue::new(&mut self.draft.target_epochs)
                .range(1..=1000000)
                .prefix("Target epochs "),
        );
        ui.add(
            egui::DragValue::new(&mut self.draft.geological_step_myr)
                .range(0.00001..=1.)
                .speed(0.001)
                .prefix("Myr / epoch "),
        );
        ui.add(
            egui::DragValue::new(&mut self.draft.climate_iterations)
                .range(4..=4096)
                .prefix("Passes / season cycle "),
        );
        ui.add(
            egui::DragValue::new(&mut self.draft.climate_cycles)
                .range(1..=64)
                .prefix("Climate cycle limit "),
        );
        ui.add(
            egui::DragValue::new(&mut self.draft.max_drainage_iterations)
                .range(16..=1000000)
                .prefix("Drainage limit "),
        );
        ui.small(format!(
            "Estimated GPU memory: {:.0} MiB",
            self.draft.estimated_bytes() as f64 / 1048576.
        ));
        if ui.button("Generate new planet").clicked() {
            self.draft.spatial_world_id = None;
            let result = Generator::new(
                self.generator.gpu.clone(),
                self.draft.clone(),
                self.generator.catalog.clone(),
            )
            .and_then(|g| self.replace(g));
            self.report(result, "New planet initialized.");
        }
        if ui.button("Apply history controls").clicked() {
            match self.draft.validate() {
                Ok(()) => {
                    self.generator.config.target_epochs = self.draft.target_epochs;
                    self.generator.config.max_drainage_iterations =
                        self.draft.max_drainage_iterations;
                    if self.generator.progress.stage == Stage::Boundary {
                        self.generator.config.geological_step_myr = self.draft.geological_step_myr;
                        self.generator.config.climate_iterations = self.draft.climate_iterations;
                        self.generator.config.climate_cycles = self.draft.climate_cycles;
                        self.generator.config.ecology_years_per_epoch =
                            self.draft.ecology_years_per_epoch;
                        self.message = "History controls applied.".into();
                    } else {
                        self.message="Target and drainage limit applied. Pause at an epoch boundary to change physical steps.".into();
                    }
                    self.generator.error = None;
                }
                Err(e) => self.message = e.to_string(),
            }
        }
    }

    fn civilization_panel(&mut self, ui: &mut egui::Ui) {
        ui.small(
            "Central-island societies. Living world couples seasonal ecology to social months.",
        );
        if self.generator.civilizations.is_none() {
            ui.add(
                egui::Slider::new(&mut self.founding_options.animal_months, 12..=120)
                    .text("Animal patron base months"),
            );
            ui.add(
                egui::Slider::new(&mut self.founding_options.intelligent_months, 12..=120)
                    .text("Intelligent patron base months"),
            );
            ui.checkbox(
                &mut self.founding_options.aid_enabled,
                "Patron founding aid",
            );
            ui.collapsing("Optional systems (default on)", |ui| {
                for &system in crate::systems::System::ALL {
                    let mut enabled = self.generator.config.systems.enabled(system);
                    if ui.checkbox(&mut enabled, system.label()).changed() {
                        self.generator.config.systems.select(system, enabled);
                    }
                }
            });
            if ui
                .add_enabled(
                    self.generator.progress.stage == Stage::Boundary,
                    egui::Button::new("Found five civilizations"),
                )
                .clicked()
            {
                self.running = false;
                let result = self
                    .generator
                    .found_civilizations_with_options(5, self.founding_options.clone())
                    .and_then(|()| {
                        let systems = self.generator.config.systems.clone();
                        self.generator.apply_systems(&systems)
                    });
                self.report(
                    result,
                    "Civilizations founded. Open the history window to inspect them.",
                );
            }
            ui.small("Finish the current epoch before founding.");
        } else {
            if self
                .generator
                .civilizations
                .as_ref()
                .is_some_and(|h| h.living.is_none())
            {
                self.running = false;
            }
            if self
                .generator
                .civilizations
                .as_ref()
                .is_some_and(|h| h.version == 1)
                && ui.button("Enable farming, crafts and markets").clicked()
            {
                let result = self.generator.upgrade_economy();
                self.report(result, "Economy upgraded; ecological plots reserved.");
            }
            if self
                .generator
                .civilizations
                .as_ref()
                .is_some_and(|h| h.version == 2 && h.resources.is_none())
                && ui.button("Enable shared extraction sources").clicked()
            {
                let result = self.generator.enable_shared_resources();
                self.report(result, "Existing reserves transferred to shared sources.");
            }
            if self
                .generator
                .civilizations
                .as_ref()
                .and_then(|h| h.living.as_ref())
                .is_some_and(|l| !l.environmental_returns)
                && ui
                    .button("Connect runoff and abandoned land to ecology")
                    .clicked()
            {
                let result = self.generator.enable_environmental_returns();
                self.report(
                    result,
                    "Environmental returns enabled at monthly boundaries.",
                );
            }
            if self
                .generator
                .civilizations
                .as_ref()
                .and_then(|h| h.resources.as_ref())
                .is_some_and(|r| r.mineral_catalog.is_empty())
                && ui
                    .button("Enable mineral-specific iron processing")
                    .clicked()
            {
                let result = self.generator.enable_mineral_processing();
                self.report(
                    result,
                    "Sources identified; separate iron ore processing enabled.",
                );
            }
            if self
                .generator
                .civilizations
                .as_ref()
                .and_then(|h| h.resources.as_ref())
                .is_some_and(|r| !r.mineral_catalog.is_empty() && !r.alloy_processing)
                && ui
                    .button("Enable copper, tin, bronze and physical residue")
                    .clicked()
            {
                let result = self.generator.enable_alloy_processing();
                self.report(result, "Alloy chains and finite residue deposits enabled.");
            }
            if let Some(mut catalog) = self
                .generator
                .civilizations
                .as_ref()
                .and_then(|h| h.economy_catalog.clone())
            {
                let mut changed = ui
                    .checkbox(
                        &mut catalog.production.enabled,
                        "Demand-driven production and storage",
                    )
                    .changed();
                changed |= ui
                    .checkbox(
                        &mut catalog.production.waterworks_repair_priority,
                        "Repair inherited waterworks before spare housing (experimental)",
                    )
                    .changed();
                if let Some(agriculture) = &mut catalog.agriculture {
                    changed |= ui
                        .checkbox(
                            &mut agriculture.fisheries_enabled,
                            "Enable managed fishery harvest",
                        )
                        .changed();
                }
                if let Some(agriculture) = &mut catalog.agriculture {
                    changed |= ui
                        .checkbox(
                            &mut agriculture.fishery.adaptive,
                            "Adaptive fishing and finite equipment (experimental)",
                        )
                        .changed();
                    changed |= ui
                        .checkbox(
                            &mut agriculture.fishery.primitive_gear,
                            "Allow primitive timber fish traps (experimental)",
                        )
                        .changed();
                    changed |= ui
                        .checkbox(
                            &mut agriculture.fishery.opportunity_cost,
                            "Compare fishing with observed food returns (experimental)",
                        )
                        .changed();
                }
                changed |= ui
                    .add_enabled(
                        catalog.production.enabled,
                        egui::Checkbox::new(
                            &mut catalog.production.adaptive_labor,
                            "Adapt workers to available jobs",
                        ),
                    )
                    .changed();
                changed |= ui
                    .add_enabled(
                        catalog.production.enabled,
                        egui::Checkbox::new(
                            &mut catalog.production.workshops,
                            "Require workshop investment and upkeep",
                        ),
                    )
                    .changed();
                changed |= ui
                    .add_enabled(
                        catalog.production.enabled,
                        egui::Checkbox::new(
                            &mut catalog.production.export_contracts,
                            "Fund repeat export contracts",
                        ),
                    )
                    .changed();
                for (setting, label) in [
                    (
                        &mut catalog.production.replacement_tool_jobs,
                        "Prioritize local replacement-tool jobs (experimental)",
                    ),
                    (
                        &mut catalog.production.toolmaking_expertise,
                        "Learn toolmaking from completed work (experimental)",
                    ),
                ] {
                    changed |= ui
                        .add_enabled(
                            catalog.production.enabled,
                            egui::Checkbox::new(setting, label),
                        )
                        .changed();
                }
                changed |= ui
                    .add_enabled(
                        catalog.production.export_contracts,
                        egui::Checkbox::new(
                            &mut catalog.production.supplier_profitability,
                            "Require profitable supplier quotes",
                        ),
                    )
                    .changed();
                changed |= ui
                    .add_enabled(
                        catalog.production.workshops,
                        egui::Checkbox::new(
                            &mut catalog.production.specialized_workshops,
                            "Require industry-specific workshops",
                        ),
                    )
                    .changed();
                if changed {
                    let result = self.generator.configure_economy(catalog);
                    self.report(result, "Production rules updated for the next month; existing inventories retained.");
                }
            }
            ui.horizontal(|ui| {
                if self
                    .generator
                    .civilizations
                    .as_ref()
                    .is_some_and(|h| h.version == 2 && h.society.is_none())
                    && ui.button("Enable social history").clicked()
                {
                    let result = self.generator.enable_society();
                    self.report(result, "Households and social history enabled.");
                }
                if self.generator.civilizations.as_ref().is_some_and(|h|h.society.as_ref().is_some_and(|s|s.indicators.is_none())) && ui.button("Observe social pressures").clicked() {
                    let result=self.generator.enable_social_indicators();self.report(result,"Social observation baseline established.");
                }
                if self
                    .generator
                    .civilizations
                    .as_ref()
                    .is_some_and(|h| h.society.is_some() && h.politics.is_none())
                    && ui.button("Enable dynasties and politics").clicked()
                {
                    let result = self.generator.enable_politics();
                    self.report(result, "Genealogy and political baseline established.");
                }
                if self
                    .generator
                    .civilizations
                    .as_ref()
                    .is_some_and(|h| h.politics.is_some() && h.governance.is_none())
                    && ui.button("Enable governance and diplomacy").clicked()
                {
                    let result = self.generator.enable_governance();
                    self.report(result, "Governance baseline established.");
                }
                if self
                    .generator
                    .civilizations
                    .as_ref()
                    .is_some_and(|h| h.society.is_some() && h.shipping.is_none())
                    && ui.button("Enable lake shipping").clicked()
                {
                    let result = self.generator.enable_shipping();
                    self.report(
                        result,
                        "Coastal access surveyed; ports build from settlement materials.",
                    );
                }
                if self.generator.civilizations.as_ref().is_some_and(|h| {
                    h.shipping.is_some() && h.governance.is_some() && h.expeditions.is_none()
                }) && ui.button("Enable ancient-continent expeditions").clicked()
                {
                    let result = self.generator.enable_expeditions();
                    self.report(
                        result,
                        "Frontier routes surveyed; wealthy sponsors can fund expeditions.",
                    );
                }
                if self.generator.civilizations.as_ref().and_then(|h|h.expeditions.as_ref()).is_some_and(|x|x.discoveries.is_none()) && ui.button("Enable specimen research and applications").clicked() {
                    let result=self.generator.enable_discoveries();self.report(result,"Finite specimen baseline enabled; future collections can support local workshops.");
                }
                if self.generator.civilizations.as_ref().is_some_and(|h| h.living.is_none()) && ui.button("Enable living world").clicked() {
                    let result = self.generator.enable_living_history();
                    self.report(result, "World ecology now advances with social history.");
                }
                if ui.button("+1 month").clicked() {
                    let result = self.generator.advance_history(1);
                    self.report(result, "Social month advanced.");
                }
                for years in [1, 10] {
                    if ui.button(format!("+{years} years")).clicked() {
                        let result = self.generator.advance_history(years * 12);
                        self.report(result, "Social history advanced.");
                    }
                }
            });
            if let Some((site, mut closed)) = self.history_selection.and_then(|id| {
                let h = self.generator.civilizations.as_ref()?;
                let r = h.resources.as_ref()?;
                h.sites
                    .get(id)
                    .map(|s| (s.id, r.closed_sites.contains(&s.id)))
            }) {
                if ui
                    .checkbox(&mut closed, "Close selected town's ore and clay extraction")
                    .changed()
                {
                    let result = self.generator.set_mine_closed(site, closed);
                    self.report(result, "Extraction access updated at the history boundary.");
                }
            }
            if let Some(h) = &self.generator.civilizations {
                ui.small(if h.living.is_some() {
                    format!(
                        "Living world · ecology month {} · terrain fixed",
                        self.generator.ecology.clock.month
                    )
                } else {
                    "Snapshot history · world ecology paused".into()
                });
                ui.label(format!(
                    "Year {} · {} settlements · {:.0} people",
                    h.month / 12,
                    h.sites.len(),
                    h.sites.iter().map(|s| s.stocks.stock[0]).sum::<f32>()
                ));
                ui.small(format!(
                    "Food residual {:.5}% · population {:.5}%",
                    h.food_residual() * 100.,
                    h.population_residual() * 100.
                ));
            }
            if ui.button("Export history JSON").clicked() {
                let path = Path::new(&self.path).with_extension("history.json");
                let result = (|| -> Result<()> {
                    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(
                        &path,
                        serde_json::to_vec_pretty(&self.generator.civilizations)?,
                    )?;
                    Ok(())
                })();
                self.report(result, "History exported.");
            }
        }
    }
    fn history_window(&mut self, ctx: &egui::Context) {
        if !self.history_window_open {
            return;
        }
        let mut open = self.history_window_open;
        let Some(h) = &self.generator.civilizations else {
            return;
        };
        let mut focus = None;
        let mut policy_change = None;
        let mut spoil = None;
        let mut route_change = None;
        let mut relocation_change = None;
        let mut relief_change = None;
        let mut religious_relief_change = None;
        let mut war_target = None;
        let mut autonomy_change = None;
        let mut negotiation_change = None;
        let mut offices_init = false;
        let mut treaty_offer = None;
        let mut sea_policy = None;
        let mut expedition_launch = None;
        let mut expedition_recall = None;
        let mut expedition_rules = None;
        let mut specimen_policy = None;
        egui::Window::new("Civilizations and history")
            .open(&mut open)
            .default_width(480.)
            .default_height(600.)
            .vscroll(true)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ctx, |ui| {
                ui.heading(format!(
                    "Year {} · month {}",
                    h.month / 12,
                    h.month % 12 + 1
                ));
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.history_tab, 0, "Towns and economy");
                    ui.selectable_value(&mut self.history_tab, 1, "Foundings and culture");
                    ui.selectable_value(&mut self.history_tab, 2, "Chronicle");
                    ui.selectable_value(&mut self.history_tab, 3, "Timeline");
                });
                ui.separator();
                if self.history_tab == 0 {
                ui.label(if h.politics.is_some() { "Atlas colors show administration; white claims are disputed." } else { "Amber atlas dots are settlements; gray dots are ruins." });
                for civ in &h.civilizations {
                    ui.label(format!(
                        "{} · led by {}",
                        civ.name, h.people[civ.leader as usize].name
                    ));
                    if let Some(language) = &civ.language {
                        ui.collapsing(format!("{} naming language · {}", language.name, civ.name), |ui| {
                            ui.small("Fictional naming conventions only; no effect on beliefs, trade or politics.");
                            ui.label(format!("{} recorded names · consistent sound changes from a shared invented vocabulary", language.names.len()));
                            ui.collapsing("Evolving vocabulary", |ui| {
                                for (other, kg) in &language.trade_kg {
                                    ui.small(format!("Civilization {other}: {kg:.1} kg received/sent since last annual update"));
                                }
                                for (concept, options) in &language.lexicon {
                                    ui.label(format!("{concept}: {}", options.iter().enumerate().map(|(i,w)|
                                        format!("{} (weight {})",w.form,1 << i)).collect::<Vec<_>>().join(" · ")))
                                        .on_hover_text(options.iter().map(|w| format!("{} · month {} · {}{}", w.form,w.adopted,
                                            w.source.as_ref().map(|s| format!("{} {}: {}",s.kind,s.id,s.name)).unwrap_or_else(|| "original root".into()),
                                            format_args!("{} · {} · {}{}", w.borrowed_from.map(|c| format!("borrowed from civilization {c}")).unwrap_or_default(),
                                                w.basis, if w.adapted {"adapted"} else {"retained"}, w.original_form.as_ref().map(|f| format!(" from {f}")).unwrap_or_default())))
                                            .collect::<Vec<_>>().join("\n"));
                                }
                            });
                            let records: Vec<_> = language.names.iter().collect();
                            let row_height = ui.text_style_height(&egui::TextStyle::Body);
                            egui::ScrollArea::vertical().max_height(240.).id_salt(("names", civ.id)).show_rows(ui, row_height, records.len(), |ui, range| {
                                for index in range {
                                    let (key, record) = records[index];
                                    ui.label(&record.name).on_hover_text(format!("{} · {}\n{}{}", key, record.form, record.meanings.join(" + "), record.source.as_ref().map(|s| format!(" · named for {} ({} {})",s.name,s.kind,s.id)).unwrap_or_default()))
                                        .on_hover_text(record.words.iter().map(|w| format!("{} → {}{}", w.concept, w.word.form,
                                            w.word.source.as_ref().map(|s| format!(" · from {}",s.name)).unwrap_or_default())).collect::<Vec<_>>().join("\n"));
                                }
                            });
                        });
                    }

                }
                ui.separator();
                if let Some(r) = h.household_relocations() {
                    ui.collapsing("Household relocation", |ui| {
                        let mut enabled = r.enabled;
                        if ui.checkbox(&mut enabled, "Allow departures from sustained hardship").changed() { relocation_change = Some(enabled); }
                        ui.small("Existing journeys continue when departures are disabled. Travel uses finite provisions; newcomers require spare production, land and food reserves.");
                        let mut witnessed = r.witnessed_relief;
                        if ui.checkbox(&mut witnessed, "Require arriving families to request outside relief").changed() { relief_change = Some(witnessed); }
                        ui.label(format!("Witnessed relief: {} · {} appeals received", if r.witnessed_relief { "enabled" } else { "legacy relief" }, r.appeals.len()));
                        if let Some(c)=&h.culture {
                            let mut enabled=c.religious_relief.enabled;
                            if ui.checkbox(&mut enabled,"Allow funded religious relief").changed() {religious_relief_change=Some(enabled);}
                            ui.small(format!("Religious relief: {} missions; {:.1} kg received. Receipt builds local ties, not conversion.",c.religious_relief.missions.len(),c.religious_relief.missions.iter().filter_map(|m|m.delivered_kg).sum::<f32>()));
                        }
                        if let Some(c)=&h.culture {
                            ui.small(format!("{} dated local reports; {} communities practicing mutual aid",c.religious_relief.memory.reports.len(),c.religious_relief.memory.mutual_aid_sites.len()));
                            for a in &c.religious_relief.memory.aid {ui.small(format!("{} remembers {}: {:.0} kg aid received, {} deliveries, {} failures",h.sites[a.recipient as usize].name,h.sites[a.donor as usize].name,a.delivered_kg,a.successes,a.failures));}
                            for m in c.religious_relief.missions.iter().rev().take(5) {
                                ui.small(format!("{}; {:.1} kg prior assistance repaid, {:.1} kg returned against this mission", if m.reciprocal { "Reciprocal assistance" } else { "Gift" },m.repayment_kg,m.returned_kg));
                                let receipt = m.delivered_kg.map_or_else(|| "in transit".into(), |kg| format!("{kg:.1} kg received"));
                                ui.small(format!("{} → {}: {:.1} kg promised, {}; recipient trust {:.2}", c.institutions[m.institution as usize].name, h.sites[m.recipient as usize].name,m.promised_kg,receipt,c.religious_relief.institution_trust(m.recipient,m.institution,h.sites[m.recipient as usize].stocks.stock[0])));
                            }
                        }
                        for a in r.appeals.iter().rev().take(5) {
                            ui.label(format!("{} → {}: report from month {}, received {} · {}", h.sites[a.origin as usize].name, h.sites[a.host as usize].name, a.reported, a.received, if a.response.is_some() { "answered (see history)" } else { "awaiting response" }));
                        }
                        for j in &r.journeys {
                            ui.label(format!("Household #{}: {} → {} · {:.1} travelers · {:.1} kg food · {}", j.household, h.sites[j.from as usize].name, h.sites[if j.returning { j.from } else { j.to } as usize].name, j.population(), j.food, if j.returning { "returning" } else if j.blocked { "route blocked" } else { "traveling" }));
                        }
                    });
                }
                ui.label("Settlements");
                egui::ScrollArea::vertical()
                    .id_salt("sites")
                    .max_height(150.)
                    .show(ui, |ui| {
                        for (i, s) in h.sites.iter().enumerate() {
                            if ui
                                .selectable_label(
                                    self.history_selection == Some(i),
                                    format!(
                                        "{} · {:.0} people · {}",
                                        s.name,
                                        s.stocks.stock[0],
                                        if s.abandoned { "ruins" } else { s.lifecycle.size.label() }
                                    ),
                                )
                                .clicked()
                            {
                                self.history_selection = Some(i);
                                focus = Some(s.cell);
                            }
                        }
                    });
                if let Some(s) = self.history_selection.and_then(|i| h.sites.get(i)) {
                    ui.label(format!(
                        "{} · island {} · founded year {}",
                        s.name,
                        s.island,
                        s.founded / 12
                    ));
                    if !s.abandoned {
                        if let Some(size) = s.lifecycle.pending_size {
                            ui.small(format!("Possible {}: {} of 24 sustained months", size.label(), s.lifecycle.pending_months));
                        }
                        if s.lifecycle.depopulated_months > 0 {
                            ui.small(format!("Below one resident-equivalent for {} of 12 months before abandonment", s.lifecycle.depopulated_months));
                        }
                    }
                    ui.label(format!(
                        "Food {:.0} kg · production {:.0} kg/month · shortage {:.0}%",
                        s.stocks.stock[1],
                        s.stocks.stock[2],
                        s.stocks.stock[3] * 100.
                    ));
                    ui.small(format!(
                        "Farm limit {:.0} ha · potential yield {:.0} kg/ha/year",
                        s.stocks.habitat[1], s.stocks.habitat[0]
                    ));
                }
                if let Some(s) = self
                    .history_selection
                    .and_then(|i| h.sites.get(i))
                    .filter(|_| h.version == 2)
                {
                    let e = &s.economy;
                    let storage_bonus = (e.goods[7] / (s.stocks.stock[0] * 2.).max(1.)).clamp(0., 1.);
                    ui.small(format!("Grain storage capacity: {:.0} kg (12–24 demand months with pottery)", s.stocks.stock[0] * 18. * (12. + 12. * storage_bonus)));
                    if let Some(f) = h.living.as_ref().and_then(|l|l.floods.get(&s.id)) {
                        if f.granary_bricks > 0. {
                            ui.small(format!("Raised granaries: {:.0} kg bricks · {:.2} m protection for food stores", f.granary_bricks, (f.granary_bricks / (s.stocks.stock[0] as f64 * 20.).max(1.)).min(1.5)));
                        }
                        ui.label(format!("Flood exposure {:.2} m · {}", f.depth_m,
                            if f.persistent { format!("persistent inundation · {} / 12 dry months", f.dry_streak) }
                            else { format!("cleanup {} months", f.recovery_months) }));
                        ui.small(format!("Flood losses: {:.0} kg stores · {:.0} kg standing crops", f.food_lost_kg, f.crops_lost_kg));
                    }
                    egui::CollapsingHeader::new("Farming and workshops")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label(format!(
                                "Soil N {:.0} kg · P {:.0} kg · stored water {:.0} m³",
                                e.soil[1], e.soil[2], e.water[0]
                            ));
                            ui.label(format!(
                                "Growth constraint: {}",
                                ["labor/light", "nitrogen", "phosphorus", "water"]
                                    .get(e.diagnostics[0] as usize)
                                    .unwrap_or(&"unknown")
                            ));
                            ui.small(format!(
                            "Workers: farms {:.0}, forestry {:.0}, extraction {:.0}, crafts {:.0}",
                            e.labor[0], e.labor[1], e.labor[2], e.labor[3]
                        ));
                            if h.society.is_some() {
                                ui.small(format!(
                                    "Current illness burden {:.3} · {:.1}% less effective work next production month",
                                    s.demography.health[0],
                                    50. * s.demography.health[0].clamp(0., 0.5)
                                ));
                            }
                            ui.small(format!(
                                "Ore remaining {:.0} kg · clay {:.0} kg · wood carbon {:.0} kg",
                                h.accessible_resources(s.id as usize)[0], h.accessible_resources(s.id as usize)[1], e.forest[0]
                            ));
                            if e.extraction[1]>0.5 {ui.label(format!("Mineral residue {:.1}/{:.1} kg · blocked recipe attempts {:.0} · bronze tools {:.1} kg",e.residue[0],e.residue[2],e.residue[3],e.goods[41]));}
                            if e.land_return[0] > 0.5 {
                                ui.label(format!("Managed land: {}; cumulative release {:.1} ha", if e.land_return[1] > 0.5 { "returned to wilderness" } else { "reserved for town" }, e.land_return[3]));
                            }
                            let mut policy = e.policy;
                            ui.add(
                                egui::Slider::new(&mut policy[0], 0. ..=1.).text("Fallow fraction"),
                            );
                            ui.add(
                                egui::Slider::new(&mut policy[1], 0. ..=1.).text("Manure retained"),
                            );
                            ui.add(
                                egui::Slider::new(&mut policy[2], 0. ..=1.)
                                    .text("Rainwater storage"),
                            );
                            let mut open = policy[3] >= 0.5;
                            ui.checkbox(&mut open, "Market open");
                            policy[3] = if open { 1. } else { 0. };
                            if policy != e.policy {
                                policy_change = Some((s.id, policy));
                            }
                            if ui.button("Spoil half the food (scenario)").clicked() {
                                spoil = Some((s.id, s.stocks.stock[1] * 0.5));
                            }
                        });
                    ui.collapsing("Production and specialization",|ui|{
                        ui.label(format!("Roles: {:?}",h.settlement_roles(s.id)));
                        ui.small("Settlement roles follow annual activity with gradual adjustment.");
                        for c in h.export_contracts.iter().filter(|c| (c.buyer==s.id || c.seller==s.id) && c.escrow>0.) {
                            let good=&h.economy_catalog.as_ref().unwrap().goods[c.good as usize].name;
                            if c.estimated_unit_cost>0. {ui.small(format!("Estimated cost {:.2}/kg · agreed price {:.2}/kg",c.estimated_unit_cost,c.unit_price));}
                            ui.label(format!("Contract: {} → {} · {} {:.1} kg remaining · escrow {:.1} · expires month {}",h.sites[c.seller as usize].name,h.sites[c.buyer as usize].name,good,c.remaining_kg,c.escrow,c.expires));
                        }
                        ui.label(format!("Crop growth {:.0} kg · feed {:.1} kg · catch {:.1} kg",e.agriculture[0],e.agriculture[1],e.agriculture[2]));
                        if e.fishery[3] > 0.5 {
                            ui.label(format!("Food return: alternative {:.2}, fishing {:.2} kg/worker-month; crew factor {:.0}%",e.fishery_choice[0],e.fishery_choice[1],e.fishery_choice[2]*100.));
                            ui.label(format!("Timber traps: {:.1} kg installed; {:.1} kg worn",e.fishery_traps[0],e.fishery_traps[2]));
                            ui.label(format!("Fishing: {:.1} workers + {:.1} equipment work; {:.1} kg this month; stock response {:.0}%",e.fishery_plan[1],e.fishery_plan[2],e.fishery_plan[3],e.fishery_stats[3]*100.));
                            ui.label(format!("Fishing equipment: {:.1} kg timber, {:.1} kg tools, {:.1} kg fiber; worn {:.1} kg",e.fishery[0],e.fishery[1],e.fishery[2],e.fishery_stats[0]));
                        }
                        for (i,crop) in e.crops.iter().enumerate(){ui.label(format!("{}: {:.0}% land · {:.1} kg growing · {:.1} kg seed · {:.0} kg harvested",h.economy_catalog.as_ref().unwrap().goods.get(8+i).map_or("legacy crop",|g|g.name.as_str()),crop[0]*100.,crop[1],crop[2],crop[3]));}
                        for (i,herd) in e.herds.iter().enumerate(){ui.label(format!("{}: {:.1} kg live · {:.1} kg products",["Goats","Sheep","Fowl"][i],herd[0],herd[3]));}
                    });
                    ui.collapsing("Market and stockpiles", |ui| {
                        ui.label(format!(
                            "Treasury {:.0} · {} shipments in transit",
                            e.finance[0],
                            h.cargo
                                .iter()
                                .filter(|c| c.from == s.id || c.to == s.id)
                                .count()
                        ));
                        let delayed = h.cargo.iter().filter(|c| (c.from == s.id || c.to == s.id) && c.weather_delay_months > 0).count();
                        if delayed > 0 { ui.small(format!("{delayed} shipments held by weather closures")); }
                        if e.logistics[3] > 0.5 {
                            if e.waterworks[3] > 0.5 {
                                ui.label(format!("Waterworks {:.1} residents capacity · operating coverage {:.0}% · domestic shortfall {:.0}% · water used {:.1} m³",e.waterworks_capacity(),e.waterworks_plan[3]*100.,e.water_service[1]*100.,e.water_service[0]));
                            }
                            ui.small(format!("Peak waterworks capacity {:.1} · priority repairs {:.2} worker-months this month / {:.2} total", e.waterworks_recovery[0], e.waterworks_recovery[2], e.waterworks_recovery[3]));
                            if e.housing_plan[3] > 0.5 {
                                ui.label(format!("Shelter {:.1} places · crowding {:.0}% · timber/bricks {:.1}/{:.1} kg",e.housing_capacity(),e.crowding(s.stocks.stock[0])*100.,e.housing[0],e.housing[1]));
                            }
                            if let Some(catalog)=h.economy_catalog.as_ref().filter(|c|c.materials.is_some()) {ui.small(format!("Container storage bonus {:.0} kg · host resistance {:.2} · source depleted {:.0}%",e.container_capacity(catalog),e.extraction[2],e.extraction[3]*100.));}
                            if e.storage[2] > 0. {
                                ui.label(format!("Persistent yards {:.0} kg · warehouses {:.0} kg · timber/bricks {:.1}/{:.1} kg · cumulative wear {:.1} kg",e.storage[2],e.storage_capacity()-e.storage[2],e.storage[0],e.storage[1],e.storage_plan[1]));
                            }
                            ui.label(format!("Free inland freight {:.1} kg · journeys reserve inland stops and sea-port approaches until arrival or loss", h.land_freight_capacity(s.id)));
                            ui.label(format!("Dry storage capacity {:.0} kg · unused craft labor {:.1} worker-months", e.logistics[0], e.logistics[2]));
                            if e.workshop_types[0][3] > 0.5 {
                                for (j,name) in crate::production::WORKSHOP_NAMES.iter().enumerate() {
                                    let t=e.workshop_types[j];
                                    ui.label(format!("{name}: {:.2} units · target {:.2} · work {:.1} worker-months", t[0],t[1],t[2]));
                                }
                            }
                            if e.workshop[3] > 0.5 {
                                let units = (e.workshop[0]/20.).min(e.workshop[1]/30.).min(e.workshop[2]/2.);
                                ui.label(format!("Workshops {:.2} units · target {:.2} · industrial work {:.1} worker-months · cumulative wear {:.1} kg", units, e.workshop_plan[0], e.workshop_plan[3], e.workshop_plan[1]));
                                ui.small(format!("Toolmaking expertise {:.1}% · priority work {:.2}/{:.2} · tool work {:.2} worker-months · output {:.2} kg", e.tool_craft[0]*100., e.tool_work[1], e.tool_work[0], e.tool_work[2], e.tool_work[3]));
                            }
                            ui.small("Orders include useful reserves and recipe inputs. Full yards pause new extraction; existing surplus remains.");
                        }
                        egui::Grid::new("goods").striped(true).show(ui, |ui| {
                            ui.label("Good");
                            ui.label("Stored kg");
                            ui.label("Price/kg");
                            ui.label("Order target kg");
                            ui.end_row();
                            for (k, good) in h.economy_catalog.as_ref().unwrap().goods.iter().enumerate().filter(|(_,g)|!g.id.starts_with("reserved_")) {
                                ui.label(&good.name);
                                ui.label(format!("{:.1}", e.goods[k]));
                                ui.label(format!("{:.2}", e.prices[k]));
                                ui.label(format!("{:.1}", e.targets[k]));
                                ui.end_row();
                            }
                        });
                        ui.small(format!(
                            "C/N/P, water, money, goods residuals: {:?}",
                            h.economy_residuals().map(|v| format!("{:.4}%", v * 100.))
                        ));
                    });
                }
                ui.separator();
                if let Some(society) = &h.society {
                    ui.collapsing("People and institutions", |ui| {
                        if let Some(site) = self.history_selection.and_then(|i| h.sites.get(i)) {
                            let d = site.demography;
                            ui.label(format!(
                                "Children {:.0} · adults {:.0} · elders {:.0}",
                                d.ages[0], d.ages[1], d.ages[2]
                            ));
                            ui.label(format!(
                                "Standing crops {:.0} kg · seed {:.0} kg · harvest month {}",
                                d.crops[0],
                                d.crops[1],
                                d.crops[2] as u32 + 1
                            ));
                            if h.month > society.started && h.economy_catalog.as_ref().is_some_and(|c| c.weather.drought_probability > 0.) {
                                ui.small(format!("Regional rainfall/crop potential: {:.0}% of ordinary weather", d.ages[3]*100.));
                            }
                            ui.small(format!(
                                "Disease burden {:.2} · hungry months {:.0}",
                                d.health[0], d.health[1]
                            ));
                            ui.small(format!("Last monthly rations, child/adult/elder (kg food equivalent): {:.1}/{:.1}, {:.1}/{:.1}, {:.1}/{:.1}",d.ration_eaten[0],d.ration_need[0],d.ration_eaten[1],d.ration_need[1],d.ration_eaten[2],d.ration_need[2]));
                            ui.small(format!("Additional ration priorities: {:.1}/{:.1}/{:.1}",d.ration_priority[0],d.ration_priority[1],d.ration_priority[2]));
                            if let Some(c)=h.social_indicators(site.id) {
                                ui.small(format!("Social memory: hunger {:.0}% · disease {:.0}% · disruption {:.0}% · ownership inequality {:.0}%",c.pressure[0]*100.,c.pressure[1]*100.,c.pressure[2]*100.,c.pressure[3]*100.));
                                ui.small(format!("Collective morale proxy {:.0}% · migration pressure {:.0}% · {}",c.morale()*100.,c.migration_pressure()*100.,if c.status[2]==1 {"sustained strain"} else {"no strain episode"}));
                                ui.small(format!("Adult labor projection: farm {:.1}, forest {:.1}, mine {:.1}, craft {:.1}, other/care {:.1}",c.livelihoods[1][0],c.livelihoods[1][1],c.livelihoods[1][2],c.livelihoods[1][3],c.livelihoods[1][4]));
                                ui.small(format!("Households by relative ownership: {:?}",c.ownership));
                                ui.small(format!("Shelter {:.1} places · current crowding {:.0}% · remembered crowding {:.0}%",c.housing[0],c.housing[1]*100.,c.housing[2]*100.));
                                ui.small(format!("Cash buffers (<⅛, ¼, ½, 1, 2, 4, 8, ≥8 months of food): {:?}",c.cash_buffer));
                                ui.small(format!("Food security (secure / strained / deprived / severe): {:?}; {:.0} households observed",c.food_security,c.household_stress[3]));
                                ui.small(format!("Persistent severe deprivation: {:.0}%",c.household_stress[2]*100.));
                                for k in 0..4 {if let Some(t)=h.culture.as_ref().and_then(|culture|culture.traditions.get(c.traditions[k] as usize)) {ui.small(format!("{}: {:.0}% affiliation proxy",t.name,c.faith[k]*100.));}}
                                if c.faith[4]>0. {ui.small(format!("Other traditions: {:.0}%",c.faith[4]*100.));}
                                for (k,share) in c.factions.iter().chain(&c.additional_factions).enumerate() { if *share>0. {ui.small(format!("{}: {:.0}% local affiliation",crate::faction_interests::NAMES[k],share*100.));} }
                            }
                            if let Some(operators) = &h.enterprises {
                                for firm in operators.firms.iter().filter(|f| f.site == site.id && f.closed.is_none()) {
                                    ui.small(format!("Workshop operator #{} · household {} · family {} · leased {:.2} units · cash {:.1} · work {:.2}/{:.2} · revenue {:.1}, wages {:.1}, rent {:.1}, unpaid fees {:.1}", firm.id, firm.owner, firm.family, firm.leased_units, firm.cash, firm.last_completed_work, firm.last_funded_work, firm.revenue, firm.wages, firm.rent, firm.written_off));
                                }
                            }
                            for family in society.households.iter().filter(|f| f.site == site.id) {
                                if let Some(a)=h.household_account(family.id) {
                                    ui.small(format!("{} wallet {:.1} · lifetime wages {:.1}, dividends {:.1}, relief {:.1}, food spending {:.1} · latest food: common {:.1} + bought {:.1} / need {:.1}",family.name,a.cash,a.wages,a.dividends,a.relief,a.food_spending,a.common_food,a.purchased_food,a.need));
                                    if a.livelihood.is_some() {
                                        ui.small(format!("Latest wages by farming / forestry / mining / crafts: {:.1} / {:.1} / {:.1} / {:.1}", a.sector_wages[0], a.sector_wages[1], a.sector_wages[2], a.sector_wages[3]));
                                    }
                                }
                                ui.label(format!(
                                    "{} · generation {} · led by {}",
                                    family.name,
                                    family.generation + 1,
                                    h.people[family.head as usize].name
                                ));
                                ui.small(format!(
                                    "{:.1} residents · {:.1}% of private stocks · cash share {:.1}",
                                    site.stocks.stock[0] as f64 * family.share,
                                    family.share * 100.,
                                    site.economy.finance[0] as f64 * family.share
                                ));
                            }
                            let council = &society.councils[h.controller(site.id) as usize];
                            ui.label(format!(
                                "Council treasury {:.0} · annual tax {:.0}% · relief paid {:.0}",
                                council.treasury,
                                council.tax_rate * 100.,
                                council.relief_paid
                            ));
                        }
                    });
                    ui.collapsing("Routes and expeditions", |ui| {
                        for r in society.routes.iter().filter(|r| {
                            self.history_selection
                                .is_none_or(|i| r.from as usize == i || r.to as usize == i)
                        }) {
                            let mut open = r.open;
                            ui.checkbox(
                                &mut open,
                                format!(
                                    "{}–{} · {:.0} travel km · {:.0} kg road bricks",
                                    h.sites[r.from as usize].name,
                                    h.sites[r.to as usize].name,
                                    r.cost_km,
                                    r.road_bricks
                                ),
                            );
                            if r.flood_months > 0 { ui.small(format!("Flood closure: {} dry monthly steps until reopening", r.flood_months)); }
                            if let Some(c) = &r.upkeep { ui.small(format!("Weathered {:.1} kg · road work {:.2} worker-months · {}",c.lost_kg,c.work,if c.impaired {"deteriorated"}else{"maintained or developing"})); }
                            if open != r.open {
                                route_change = Some((r.id, open));
                            }
                        }
                        for r in &society.raids {
                            ui.label(format!(
                                "{} · {:.0} people · {:.0} kg provisions · {} · arrival {}",
                                r.war.map_or(format!("Raid {}",r.id),|w|format!("{}, army {}",h.politics.as_ref().and_then(|p|p.wars.get(w as usize)).map(|w|w.label()).unwrap_or_else(||format!("War {w}")),r.id)),
                                r.soldiers,
                                r.food,
                                if r.occupation_until.is_some() { "occupying until" } else if r.returning { "returning" } else { "outbound" },
                                r.arrives
                            ));
                        }
                    });
                }
                if let Some(p) = &h.politics {
                    ui.collapsing("Genealogy and marriages", |ui| {
                        if let Some(site)=self.history_selection {
                            let houses:std::collections::BTreeSet<_>=h.society.as_ref().unwrap().households.iter().filter(|f|f.site as usize==site).map(|f|f.id).collect();
                            egui::ScrollArea::vertical().id_salt("family_tree").max_height(220.).show(ui,|ui| {
                                for k in p.kin.iter().filter(|k|houses.contains(&k.household)) {
                                    let person=&h.people[k.person as usize];
                                    ui.label(format!("#{} {} · born Y{} · {}",person.id,person.name,person.born/12,person.died.map_or("living".into(),|m|format!("died Y{}",m/12))));
                                    ui.small(k.parents[0].map_or("Parents unknown at baseline".into(),|a|format!("Parents: #{} {} + #{} {}",a,h.people[a as usize].name,k.parents[1].unwrap(),h.people[k.parents[1].unwrap() as usize].name)));
                                    for m in p.marriages.iter().filter(|m|m.partners.contains(&person.id)) {ui.small(format!("Union #{} + #{} · Y{}–{} · {} recorded children",m.partners[0],m.partners[1],m.started/12,m.ended.map_or("present".into(),|v|(v/12).to_string()),m.children));}
                                }
                            });
                        }
                    });
                    ui.collapsing("Factions, claims and wars", |ui| {
                        if let Some(site)=self.history_selection {
                            let s=&h.sites[site]; let controller=h.controller(s.id);
                            ui.label(format!("Administration: {} · cultural affiliation: {}",h.civilizations[controller as usize].name,h.civilizations[s.civilization as usize].name));
                            for f in p.factions.iter().filter(|f|f.civilization==s.civilization) {
                                ui.label(format!("{} · support {:.0}% · dissent {:.0}% · cohesion {:.0}%{}",crate::faction_interests::name(f.interest),f.support*100.,f.dissent*100.,f.cohesion*100.,if p.governing[s.civilization as usize]==f.id {" · governing"}else{""}));
                            }
                            ui.label(format!("{} claimed cells · {} contested",p.claims.iter().filter(|c|c.sites.contains(&s.id)).count(),p.claims.iter().filter(|c|c.sites.contains(&s.id)&&p.claim_owners(c).len()>1).count()));
                            ui.small("Claims cover occupied hinterland and surveyed road corridors. Taxes go to the administrator.");
                            for r in &h.society.as_ref().unwrap().routes {
                                let target=if r.from==s.id {Some(r.to)}else if r.to==s.id {Some(r.from)}else{None};
                                if let Some(t)=target.filter(|&t|h.controller(t)!=controller) {
                                    if ui.button(format!("Demand territory: {}",h.sites[t as usize].name)).clicked(){war_target=Some((s.id,t));}
                                }
                            }
                        }
                        for w in p.wars.iter().rev().take(20) { ui.label(format!("{}: {} → {} · {} · Y{}–{}",w.label(),h.civilizations[w.attacker as usize].name,h.sites[w.goal as usize].name,w.outcome,w.started/12,w.ended.map_or("present".into(),|m|(m/12).to_string()))); }
                    });
                }
                if let Some(g)=&h.governance {
                    ui.collapsing("Civic petitions and remembered outcomes",|ui| {
                        for p in g.petitions.iter().rev().take(30) {
                            ui.small(format!("{}: {:?} · {} · {:.1}/{:.1} transferred · pressure {:.2}",h.sites[p.site as usize].name,p.demand,if p.honored {"honored"}else if p.resolved.is_some() {"lapsed"}else {"pending"},p.paid,p.requested,p.pressure));
                        }
                    });
                    ui.collapsing("Governance and diplomacy",|ui| {
                        if h.offices.is_none() { offices_init |= ui.button("Establish local offices").clicked(); }
                        let mut negotiate = g.negotiated_autonomy;
                        if ui.checkbox(&mut negotiate, "Councils may negotiate local self-rule").changed() { negotiation_change = Some(negotiate); }
                        if let Some(i)=self.history_selection {
                            let a=&g.administrations[i];
                            if let Some(o)=h.offices.as_ref().and_then(|x|x.seats.get(i)) {
                                ui.label(format!("{} · {:?} · collection capacity {:.0}%",o.title,o.selection,h.office_capacity(i as u32)*100.));
                                ui.label(o.holder().map_or("Vacant".into(),|p|format!("Holder: {}",h.people[p as usize].name)));
                                for t in o.tenures.iter().rev().take(4) {ui.small(format!("{}: month {}–{} (term due {})",h.people[t.person as usize].name,t.began,t.ended.map_or("present".into(),|m|m.to_string()),t.due));}
                            }
                            ui.label(format!("Loyalty {:.0}% · unrest {:.0}% · unpaid {} months",a.loyalty*100.,a.unrest*100.,a.unpaid_months));
                            let [hunger, disruption] = crate::governance::local_pressures(h.sites[i].stocks.stock[3], h.social_indicators(i as u32));
                            ui.small(format!("Current governance inputs: hunger {:.0}% · crowding/disruption {:.0}%", hunger * 100., disruption * 100.));
                            ui.small("Social pressures use the latest completed observation; recovery takes time after shortages end.");
                            ui.small(format!("Administration wages paid {:.0} · crisis duration {} months",a.wages_paid,a.crisis_months));
                            let mut autonomy=a.autonomy;
                            if ui.add(egui::Slider::new(&mut autonomy,0. ..=1.).text("Local autonomy")).changed(){autonomy_change=Some((i as u32,autonomy));}
                            ui.small("Autonomy reduces tax revenue and local resistance. Wages transfer council money to residents.");
                            for r in g.relations.iter().filter(|r|r.parties.contains(&a.controller)) {
                                let other=if r.parties[0]==a.controller {r.parties[1]}else{r.parties[0]};
                                ui.label(format!("{} · trust {:.0} · {} trade contacts",h.civilizations[other as usize].name,r.trust,r.trade_contacts));
                                if !g.protected(a.controller,other,h.month)&&ui.small_button(format!("Agree to 10-year non-aggression with {}",h.civilizations[other as usize].name)).clicked(){treaty_offer=Some((a.controller,other));}
                            }
                        }
                        for t in g.treaties.iter().rev().take(12) {ui.label(format!("Treaty {} · {} / {} · expires Y{}{}",t.id,h.civilizations[t.parties[0] as usize].name,h.civilizations[t.parties[1] as usize].name,t.expires/12,if t.expired.is_some(){" · expired"}else{""}));}
                    });
                }
                if let Some(shipping) = &h.shipping {
                    ui.collapsing("Lake shipping", |ui| {
                        ui.small("Regional harbors and pooled merchant capacity. Materials wear and require replacement. Policy closures allow existing contracts to finish. Flooded approaches delay cargo until recovery.");
                        for p in &shipping.ports {
                            if p.flood_months > 0 { ui.small(format!("{}: weather closure · {} dry steps to reopen", h.sites[p.site as usize].name, p.flood_months)); }
                            ui.label(format!("{}: {:.0} kg capacity · timber {:.0}/200 · tools {:.1}/10 · masonry {:.0}/100 · approach {:.0} travel km", h.sites[p.site as usize].name,p.capacity(),p.assets[0],p.assets[1],p.assets[2],p.access_km));
                            if let Some(f) = &p.fleet { for v in &f.vessels { ui.small(format!("{} · crew household {:?} · {:.2} worker-months · cumulative wages {:.1}",v.name,v.household,v.funded_work,v.wages_paid)); } }
                            if let Some(w) = &p.work { ui.small(format!("Harbor work {:.2} worker-months{}",w.worker_months,if w.impaired {" · deteriorated"}else{""})); }
                        }
                        for (id,l) in shipping.lanes.iter().enumerate() {
                            let mut open=l.open;
                            if ui.checkbox(&mut open,format!("{} ↔ {} · {:.0} sea km · {:.0} kg free",h.sites[shipping.ports[l.ports[0] as usize].site as usize].name,h.sites[shipping.ports[l.ports[1] as usize].site as usize].name,l.km,h.sea_capacity(id as u32))).changed() {sea_policy=Some((id as u32,open));}
                        }
                    });
                }
                if let Some(x)=&h.expeditions {
                    ui.collapsing("Ancient-continent expeditions",|ui| {
                        use crate::expeditions::{Objective,Phase};
                        let mut rules=x.rules.clone();
                        let changed=ui.checkbox(&mut rules.automatic,"Automatic wealthy-sponsor charters").changed()
                            | ui.add(egui::Slider::new(&mut rules.hazard_scale,0. ..=5.).text("Hazard multiplier")).changed()
                            | ui.add(egui::Slider::new(&mut rules.reserve_months,2..=24).text("Extra provision months")).changed();
                        if changed {expedition_rules=Some(rules);}
                        ui.small("Eight adults, 100 kg timber, 24 kg tools, voyage provisions and 600 money escrow. Civilian reserves remain protected. Temporary outer-shore research camps; no permanent settlements.");
                        for (id,r) in x.routes.iter().enumerate() {
                            let port=&h.shipping.as_ref().unwrap().ports[r.port as usize];
                            let sponsor=h.controller(port.site);
                            ui.label(format!("{} → outer cell {} · {} months each way · knowledge {:.1}",h.sites[port.site as usize].name,r.cells.last().unwrap(),r.travel_months,x.knowledge[sponsor as usize]));
                            ui.horizontal(|ui| {for (label,goal) in [("Chart voyage",Objective::Charts),("Geological survey",Objective::Geology),("Ecological survey",Objective::Ecology),("Seek patron",Objective::PatronSearch),("Study inscriptions",Objective::Inscriptions),("Seek old texts",Objective::OldLiterature)] {if ui.small_button(label).clicked(){expedition_launch=Some((id as u32,goal,None));}}});
                        }
                        for e in x.voyages.iter().rev().take(24) {
                            ui.collapsing(format!("Expedition {} · {:?} · {:?} · {}/{} survivors",e.id,e.objective,e.phase,e.survivors(),e.crew.len()),|ui| {
                                if let Some(c)=&e.heritage { ui.label(&c.motive); if let Some(f)=&c.find { ui.small(&f.description); ui.small(format!("Recovered artifact: {:?} · {} dated institutional readings",f.artifact,f.studies.len())); } }
                                ui.label(format!("Departed Y{} M{} · next milestone month {} · {:.0} kg food · {:.1} kg tools · {:.1} kg timber · {:.0} money escrow",e.departed/12,e.departed%12+1,e.due,e.food,e.tools,e.timber,e.purse));
                                ui.label(format!("{:.1} observation points · {} · effective research competence {:.0}%",e.findings,if e.confirmed {"delivered and confirmed"}else{"not confirmed at home"},e.research_skill()*100.));
                                if ui.button("Show planned route on atlas").clicked() { self.expedition_overlay=Some(e.id); self.history_window_open=false; self.journey_overlay=None; self.historical_territory=false; self.cameras[1]=Camera::atlas(); self.view_mode=2; }
                                ui.small(if e.planned_cells.is_some() { "Route saved at departure; endpoint is the planned destination." } else { "Legacy route association; original route was not saved." });
                                ui.small(format!("Specimens aboard: {:.2} kg resin · {:.2} kg phosphatic crust",e.samples[0],e.samples[1]));
                                for c in &e.crew {ui.small(format!("{} · {} · {} · competence {:.0}%",c.name,c.role,if c.alive {"survivor"}else{"deceased"},c.expertise.unwrap_or(e.skill)*100.));}
                                if matches!(e.phase,Phase::Outward|Phase::Camp)&&ui.button("Recall expedition").clicked(){expedition_recall=Some(e.id);}
                                if e.phase==Phase::Stranded&&e.objective!=Objective::Rescue&&ui.button("Fund rescue voyage").clicked(){expedition_launch=Some((e.route,Objective::Rescue,Some(e.id)));}
                            });
                        }
                    });
                }
                if let Some(x)=&h.expeditions {if let Some(d)=&x.discoveries {
                    ui.collapsing("Specimen research and applications",|ui| {
                        ui.small("Finite coastal sources. Workshops reserve up to two craft workers and consume tools and charcoal. Local assays or paid research exchange establish a method; fresh specimens are still required.");
                        ui.label(format!("Remedy made {:.1} kg · used {:.1} · expired {:.1} · farm phosphorus {:.2} kg · labor {:.1}/{:.1} worker-months used/reserved",d.remedy_made,d.remedy_used,d.remedy_expired,d.phosphorus_applied,d.worker_months,d.worker_months_reserved));
                        ui.small(format!("Specimen/remedy budget residuals: {:?}",d.residuals(x)));
                        for w in &d.workshops {let mut open=w.enabled;if ui.checkbox(&mut open,format!("{} workshop open",h.sites[w.site as usize].name)).changed(){specimen_policy=Some((w.site,open));}ui.label(format!("{}: resin {:.2} kg · mineral {:.2} kg · remedy {:.2} kg",h.sites[w.site as usize].name,w.samples[0],w.samples[1],w.remedy));ui.small(format!("Assay progress: resin {:.0}% · mineral {:.0}%",w.studied[0]/1.5*100.,w.studied[1]/1.5*100.));ui.small(format!("Copied methods: resin {} · mineral {}",w.learned[0].is_some(),w.learned[1].is_some()));}
                        for source in &d.sources {ui.small(format!("Coast {}: resin {:.1}/{:.1} kg · mineral {:.1}/{:.1} kg remaining",source.cell,source.remaining[0],source.initial[0],source.remaining[1],source.initial[1]));}
                    });
                }}
                }
                if self.history_tab == 1 {
                if let Some(c) = &h.culture {
                    egui::CollapsingHeader::new("Founding patrons and living traditions").default_open(true).show(ui, |ui| {
                        ui.small("Witnessed events and attributed religious accounts are separate. Patrons return to the ancient continent after their finite service; their mandate remains unknown.");
                        if c.traditions.len() >= 256 { ui.label("Tradition capacity reached: further schisms are paused; existing traditions continue evolving."); }
                        egui::CollapsingHeader::new("Patrons").default_open(true).show(ui, |ui| {
                        for p in &c.patrons { egui::CollapsingHeader::new(&p.name).id_salt(("patron",p.id)).default_open(p.id == 0).show(ui, |ui| {
                            ui.label(format!("{} · origin {} / landing {} / {}",c.catalog.patrons[p.archetype as usize].kind,p.origin,p.landing,h.sites[p.site as usize].name));
                            ui.label(format!("Return to ancient continent: month {} · {} · aid {:?}",p.departure_month,if p.departed.is_some(){"departed for the ancient continent"}else{"still guiding the island community"},p.effort));
                            ui.small(&c.catalog.patrons[p.archetype as usize].appearance);
                            ui.small(format!("Witnesses: {}", p.witnesses.iter().map(|&id| h.people[id as usize].name.as_str()).collect::<Vec<_>>().join(", ")));
                            if ui.small_button(format!("Read arrival #{}",p.arrival_event)).clicked(){self.history_tab=2; self.history_search=format!("#{}",p.arrival_event);}
                            for a in c.artifacts.iter().filter(|a| a.tradition==Some(p.id) && a.kind=="founding keepsake") {
                                ui.label(format!("Founding object: {}{}",a.name,if a.destroyed{" (destroyed)"}else if a.lost{" (lost in ruins)"}else{""}));
                            }
                        }); }
                        });
                        ui.collapsing(format!("Traditions ({})", c.traditions.len()), |ui| {
                        for t in &c.traditions { egui::CollapsingHeader::new(&t.name).id_salt(("tradition",t.id)).show(ui, |ui| {
                            ui.label(format!("Values: {}", t.themes.map(|i|crate::culture::THEMES[i as usize]).join(", ")));
                            ui.small(format!("Ancestral tradition: {} · Patron: {}",t.parent.map_or("Founding tradition",|id|c.traditions[id as usize].name.as_str()),t.patron.map_or("Unrecorded",|id|c.patrons[id as usize].name.as_str())));
                            for a in c.accounts.iter().filter(|a|a.tradition==t.id).rev().take(8){
                                let author=a.author.map(|id|h.people[id as usize].name.as_str()).or_else(||a.institution.map(|id|c.institutions[id as usize].name.as_str())).unwrap_or("Unrecorded transmitter");
                                ui.label(format!("Account attributed to {author}, Y{}: {}",a.month/12,a.text));
                                ui.horizontal_wrapped(|ui|{ for id in &a.facts {if ui.small_button(format!("Fact #{id}")).clicked(){self.history_tab=2; self.history_search=format!("#{id}");}} });
                            }
                        }); }
                        });
                        ui.collapsing("Institutions", |ui| {
                        for n in c.institutions.iter().filter(|n|n.active) { egui::CollapsingHeader::new(&n.name).id_salt(("institution",n.id)).show(ui, |ui| {
                            ui.label(format!("{} members · treasury {:.1} · dues {:.1} · expenditure {:.1}",n.members.len(),n.treasury,n.dues,n.expenses));
                            let local = c.site_people(h, n.site).iter().filter(|p| n.members.contains(p)).count();
                            ui.small(format!("{} locally present adult members · two provide full staffing support", local));
                            if let Some(tradition) = n.tradition {
                                ui.small(format!("Tradition: {}", c.traditions[tradition as usize].name));
                            }
                            if let Some(c)=&n.capacity {if let Some(b)=&c.building {if let Some(f)=&b.facility {ui.small(format!("Facility capacity {:.1} usable / {:.1} planned · expansion investment {:.1}",f.usable(),f.planned(),f.invested)); for room in &f.rooms { ui.small(format!("{} · {:.0} capacity · {:.2} work remaining",room.method,room.capacity,room.remaining_work));for p in &room.components {let name=h.economy_catalog.as_ref().and_then(|c|c.goods.get(p.good as usize)).map_or("unknown",|g|g.name.as_str());ui.small(format!("  {}: {:.1} kg, {:.0}% condition",name,p.kg,p.condition*100.));}}}ui.small(format!("Meeting place {:.0}% condition · construction remaining {:.2} worker-months · replacement material {:.3} kg · repairs paid {:.2}",b.condition*100.,b.construction_remaining,b.repaired_kg,b.repair_paid));} ui.small(format!("Readiness {:.0}% · upkeep paid {:.1} · work {:.2} · {}",c.readiness*100.,c.paid,c.work,if n.operational() {"operational"}else{"services limited"}));}
                            if let Some(m) = n.capacity.as_ref().and_then(|c| c.mandate.as_ref()) {
                                let eligible = c.institution_candidates(h, n.id).len();
                                ui.small(format!("{eligible} eligible local representatives{}", if eligible == 0 { " · no local constituency for succession" } else { "" }));
                                let holder = m.holder.map_or("Vacant", |id| h.people[id as usize].name.as_str());
                                ui.small(format!("Mandate: {holder} · {} · last ballot support {:.0}% · assembly work {:.3}", if m.contested {"contested"} else {"uncontested"}, m.support * 100., m.work));
                                for &event in m.events.iter().rev().take(5) {
                                    if ui.small_button(format!("Succession event #{event}")).clicked() { self.history_tab = 2; self.history_search = format!("#{event}"); }
                                }
                            } else {
                                ui.small(format!("Led by {} at {}",h.people[n.leader as usize].name,h.sites[n.site as usize].name));
                            }
                            ui.small(format!("Practices: {}",n.knowledge.iter().map(|&id| crate::culture::TOPICS[id as usize]).collect::<Vec<_>>().join(", ")));
                            for &id in &n.property { ui.label(&c.artifacts[id as usize].name); }
                        }); }
                        });
                        ui.collapsing("Local practical knowledge", |ui| {
                            ui.small("Living local representatives enable techniques. Institutional traditions and unread books preserve possible sources, not an automatic skilled workforce.");
                            for site in h.sites.iter().filter(|s| !s.abandoned) {
                                let holders=c.knowledge_holders(h,site.id);
                                let topics=crate::culture::TOPICS.iter().enumerate().filter(|(i,_)| holders[*i] > 0).map(|(i,name)| format!("{name} ({} holder{})", holders[i], if holders[i] == 1 { "; succession risk" } else { "s" })).collect::<Vec<_>>().join(", ");
                                ui.label(format!("{}: {}",site.name,topics));
                            }
                        });
                        ui.collapsing("Individual participation", |ui| {
                            ui.small("Named residents share cultural and research work. Population and ordinary employment still use settlement cohorts.");
                            if let Some(p)=&h.participation {
                                ui.label(format!("Opening-month roster: {} known people · {} commitments",p.residents.len(),p.commitments.len()));
                                for work in &p.commitments {
                                    egui::CollapsingHeader::new(format!("{} · {:?}: {:.2} / {:.2} worker-months",h.sites[work.site as usize].name,work.activity,work.used,work.granted)).id_salt(("participation",work.month,work.site,format!("{:?}",work.activity))).show(ui,|ui| {
                                        for &(id,time) in &work.people {ui.label(format!("{}: {:.2} reserved · current {:?}",h.people[id as usize].name,time,h.person_presence(id).1));}
                                        if let Some(reason)=&work.cancellation {ui.small(reason);}
                                        ui.small(if work.settled {"Settled; unused time expires"}else{"Awaiting execution"});
                                    });
                                }
                            } else {ui.label("Legacy aggregate work mode");}
                        });
                        ui.collapsing("People and learning", |ui| {
                            for a in c.agents.iter().filter(|a|a.actions>0).rev().take(32) { egui::CollapsingHeader::new(&h.people[a.person as usize].name).id_salt(("cultural_person",a.person)).show(ui,|ui| {
                                ui.label(format!("{} · {}",a.occupation,a.goal));
                                ui.small(format!("Ambition / generosity / piety / curiosity / loyalty / caution: {:?}",a.traits));
                                ui.small(format!("Known places: {} · recorded actions: {}",a.known_places.len(),a.actions));
                                for &topic in &a.knowledge { ui.horizontal(|ui| {
                                    ui.label(crate::culture::TOPICS[topic as usize]);
                                    if let Some(event)=a.knowledge_sources.get(&topic) { if ui.small_button(format!("Learned at #{event}")).clicked() {self.history_tab=2; self.history_search=format!("#{event}");} }
                                }); }
                            }); }
                        });
                        ui.collapsing("Objects and provenance", |ui| {
                        for a in c.artifacts.iter().rev() { egui::CollapsingHeader::new(&a.name).id_salt(("artifact",a.id)).show(ui, |ui| {
                            let owner=match a.owner {crate::culture::Owner::Person(id)=>h.people[id as usize].name.as_str(),crate::culture::Owner::Institution(id)=>c.institutions[id as usize].name.as_str(),crate::culture::Owner::Community(id)=>h.sites[id as usize].name.as_str()};
                            ui.label(format!("Owned by {owner} · {}",if a.destroyed{"destroyed"}else if a.lost{"lost in ruins; inaccessible"}else{"surviving"}));
                            ui.small(format!("Physical location: {} · Custodian: {}",a.site.map_or("Unknown",|id|h.sites[id as usize].name.as_str()),a.custodian.map_or("Site storage",|id|h.people[id as usize].name.as_str())));
                            ui.horizontal_wrapped(|ui|{for id in &a.events {if ui.small_button(format!("Event #{id}")).clicked(){self.history_tab=2; self.history_search=format!("#{id}");}}});
                        }); }
                        });
                    });
                }
                }
                if self.history_tab == 3 {
                    focus = self.timeline_view.show(ui, h, &mut self.history_selection);
                    if let Some(id)=self.timeline_view.take_route_request() {
                        self.journey_overlay=Some(id);
                        self.history_window_open=false;
                        self.expedition_overlay=None;
                        self.territory_export_month=h.events[id as usize].month;
                        self.historical_territory=true;
                        self.view_mode=2;
                        self.cameras[1]=Camera::atlas();
                    }

                }
                if self.history_tab == 2 {
                ui.label("Recorded events");
                ui.text_edit_singleline(&mut self.history_search);
                let filter = self.history_search.to_lowercase();
                egui::ScrollArea::vertical()
                    .id_salt("events")
                    .max_height(250.)
                    .show(ui, |ui| {
                        for e in h
                            .events
                            .iter()
                            .rev()
                            .filter(|e| {
                                filter.is_empty()
                                    || filter == format!("#{}", e.id)
                                    || e.detail.to_lowercase().contains(&filter)
                                    || e.kind.contains(&filter)
                            })
                            .take(100)
                        {
                            ui.label(format!(
                                "#{} · Y{} M{} · {} · {}: {}",
                                e.id,
                                e.month / 12,
                                e.month % 12 + 1,
                                e.site
                                    .and_then(|id| h.sites.get(id as usize))
                                    .map_or("Society", |s| s.name.as_str()),
                                e.kind,
                                e.detail
                            ));
                            if !e.causes.is_empty() {
                                ui.horizontal_wrapped(|ui| {
                                    ui.small("Linked causes:");
                                    for cause in &e.causes {
                                        if ui.small_button(format!("#{cause}")).clicked() {
                                            self.history_search = format!("#{cause}");
                                        }
                                    }
                                });
                            }
                        }
                    });
                }
            });
        self.history_window_open &= open;
        if let Some((site, open)) = specimen_policy {
            let result = self.generator.set_specimen_workshop_open(site, open);
            self.report(result, "Workshop policy updated.");
        }
        if let Some(rules) = expedition_rules {
            let result = self.generator.configure_expeditions(rules);
            self.report(result, "Expedition rules updated.");
        }
        if let Some((route, goal, rescue)) = expedition_launch {
            let result = self
                .generator
                .launch_expedition(route, goal, rescue)
                .map(|_| ());
            self.report(result, "Expedition provisioned and departed.");
        }
        if let Some(id) = expedition_recall {
            let result = self.generator.recall_expedition(id);
            self.report(result, "Recall recorded; crew must travel home.");
        }
        if let Some((lane, open)) = sea_policy {
            let result = self.generator.set_sea_lane_open(lane, open);
            self.report(result, "Sea lane policy updated.");
        }
        if offices_init {
            let result = self.generator.enable_offices();
            self.report(
                result,
                "Local offices established at this history boundary.",
            );
        }
        if let Some(enabled) = negotiation_change {
            let result = self.generator.set_negotiated_autonomy(enabled);
            self.report(result, "Council negotiation policy updated.");
        }
        if let Some((site, value)) = autonomy_change {
            let result = self.generator.set_autonomy(site, value);
            self.report(result, "Autonomy policy updated.");
        }
        if let Some((a, b)) = treaty_offer {
            let result = self.generator.sign_nonaggression(a, b, 120).map(|_| ());
            self.report(result, "Non-aggression treaty recorded.");
        }
        if let Some((origin, target)) = war_target {
            let result = self.generator.declare_war(origin, target).map(|_| ());
            self.report(result, "Territorial campaign mobilized.");
        }
        if let Some(enabled) = religious_relief_change {
            let result = self
                .generator
                .civilizations
                .as_mut()
                .unwrap()
                .set_religious_relief(enabled);
            self.report(result, "Religious relief policy updated.");
        }
        if let Some(enabled) = relief_change {
            let result = self
                .generator
                .civilizations
                .as_mut()
                .unwrap()
                .set_witnessed_relief(enabled);
            self.report(
                result,
                "Relief awareness policy updated; existing appeals and shipments continue.",
            );
        }
        if let Some(enabled) = relocation_change {
            let result = self
                .generator
                .civilizations
                .as_mut()
                .unwrap()
                .set_household_relocation(enabled);
            self.report(result, "Household relocation policy updated.");
        }
        if let Some((id, open)) = route_change {
            let result = self.generator.set_route_open(id, open);
            self.report(result, "Route access changed.");
        }
        if let Some((id, policy)) = policy_change {
            let result = self.generator.set_site_policy(id, policy);
            self.report(result, "Site policy updated.");
        }
        if let Some((id, kg)) = spoil {
            let result = self.generator.spoil_site_food(id, kg);
            self.report(result, "Food loss recorded.");
        }
        if let Some(id) = focus {
            let d = crate::grid::cell_direction(id, self.generator.config.resolution);
            for c in &mut self.cameras {
                c.focus(d);
            }
            if let Ok(c) = self.generator.inspect(id) {
                self.selected = Some((id, c));
            }
        }
    }
    fn files(&mut self, ui: &mut egui::Ui) {
        if ui
            .add_enabled(
                self.selected.is_some(),
                egui::Button::new("Generate explorable region"),
            )
            .clicked()
        {
            if let Some((id, _)) = self.selected {
                self.running = false;
                let center = crate::grid::cell_direction(id, self.generator.config.resolution);
                match self.generator.generate_region(
                    center,
                    600f32.min(self.generator.config.radius_km * 0.5),
                    512,
                ) {
                    Ok(region) => {
                        let image = region.image();
                        let texture = ui.ctx().load_texture(
                            "regional terrain",
                            egui::ColorImage::from_rgb([512, 512], image.as_raw()),
                            egui::TextureOptions::LINEAR,
                        );
                        self.region = Some((region, texture));
                        self.regional_selection = None;
                    }
                    Err(error) => self.message = error.to_string(),
                }
            }
        }

        if self.expedition_overlay.is_some() && ui.button("Hide expedition route").clicked() {
            self.expedition_overlay = None;
        }
        ui.text_edit_singleline(&mut self.path);
        if ui
            .add_enabled(
                self.region.is_some(),
                egui::Button::new("Export survey spatial features"),
            )
            .clicked()
        {
            let path = Path::new(&self.path).with_extension("survey.geojson");
            let result = self
                .region
                .as_ref()
                .unwrap()
                .0
                .spatial_features()
                .and_then(|f| f.geojson())
                .and_then(|value| {
                    std::fs::write(&path, serde_json::to_vec_pretty(&value)?)?;
                    Ok(())
                });
            self.report(result, &format!("Exported {}", path.display()));
        }
        ui.add(
            egui::DragValue::new(&mut self.event_export_months)
                .range(1..=120000)
                .prefix("Event export months "),
        );
        if ui
            .add_enabled(
                self.generator.civilizations.is_some(),
                egui::Button::new("Export recent event locations"),
            )
            .clicked()
        {
            let month = self.generator.civilizations.as_ref().unwrap().month;
            let path = Path::new(&self.path).with_extension("events.geojson");
            let result = self
                .generator
                .spatial_events(month.saturating_sub(self.event_export_months - 1)..=month)
                .and_then(|f| f.geojson())
                .and_then(|value| {
                    std::fs::write(&path, serde_json::to_vec_pretty(&value)?)?;
                    Ok(())
                });
            self.report(result, &format!("Exported {}", path.display()));
        }
        ui.add(egui::DragValue::new(&mut self.territory_export_month).prefix("Territory month "));
        if ui.button("Use current history month").clicked() {
            self.territory_export_month =
                self.generator.civilizations.as_ref().map_or(0, |h| h.month);
        }
        if ui.button("Export recorded territory").clicked() {
            let path = Path::new(&self.path).with_extension("territory.geojson");
            let result = self
                .generator
                .spatial_territory(self.territory_export_month)
                .and_then(|f| f.geojson())
                .and_then(|value| {
                    std::fs::write(&path, serde_json::to_vec_pretty(&value)?)?;
                    Ok(())
                });
            self.report(result, &format!("Exported {}", path.display()));
        }
        if ui.button("Export spatial features").clicked() {
            let path = Path::new(&self.path).with_extension("spatial.geojson");
            let result = self
                .generator
                .spatial_features()
                .and_then(|f| f.geojson())
                .and_then(|value| {
                    std::fs::write(&path, serde_json::to_vec_pretty(&value)?)?;
                    Ok(())
                });
            self.report(result, &format!("Exported {}", path.display()));
        }
        if ui.button("Save checkpoint").clicked() {
            self.pending_save = true;
            self.message =
                "Checkpoint queued for the next completed epoch. Evolve to reach it if paused."
                    .into();
        }
        if ui.button("Load checkpoint").clicked() {
            let result = Generator::load(self.generator.gpu.clone(), &self.path)
                .and_then(|g| self.replace(g));
            self.report(result, "Checkpoint loaded; evolution is paused.");
        }
        if ui.button("Export atlas PNG").clicked() {
            let path = Path::new(&self.path).with_extension("png");
            self.maps[1].render(&self.generator, self.layer, Camera::atlas(), None);
            let result = self.maps[1].export_png(&self.generator, &path);
            self.report(result, &format!("Exported {}", path.display()));
        }
        if ui
            .add_enabled(
                self.selected.is_some(),
                egui::Button::new("Export selected region PNG"),
            )
            .clicked()
        {
            if let Some((id, _)) = self.selected {
                let path = Path::new(&self.path).with_extension("region.png");
                let d = crate::grid::cell_direction(id, self.generator.config.resolution);
                let result = self.maps[0].export_region(
                    &self.generator,
                    d,
                    600f32.min(self.generator.config.radius_km * 0.5),
                    &path,
                );
                self.report(result, &format!("Exported {}", path.display()));
            }
        }
    }
    fn diagnostics(&self, ui: &mut egui::Ui) {
        ui.label(&self.generator.gpu.adapter_name);
        ui.small(format!(
            "Lake surface relaxation: {} passes",
            self.generator.progress.lake_iterations
        ));
        ui.small(format!(
            "{} cells · {:.0} MiB estimated",
            self.generator.config.cells(),
            self.generator.config.estimated_bytes() as f64 / 1048576.
        ));
        ui.small(if self.generator.progress.timestamp_supported {
            "GPU stage timings"
        } else {
            "Wall-clock stage timings"
        });
        let mut timings: Vec<_> = self
            .generator
            .progress
            .stage_ms
            .iter()
            .filter(|(name, _)| name.as_str() != "ecological_gpu" && !name.ends_with("_wall"))
            .collect();
        timings.sort_by(|a, b| b.1.total_cmp(a.1));
        ui.small("Individual kernels, largest first (aggregate timings omitted)");
        for (name, ms) in timings {
            ui.small(format!("{name}: {ms:.1} ms"));
        }
        let climate = if self.generator.progress.climate_converged {
            "converged"
        } else {
            "iteration budget / pending"
        };
        ui.small(format!(
            "Climate: {climate} after {} cycles. Drainage and rivers require convergence.",
            self.generator.progress.climate_cycles
        ));
    }
    fn central(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.);
        ui.horizontal(|ui| {
            ui.heading(LAYERS[self.layer as usize]);
            ui.label("Drag to explore · scroll to zoom · click to inspect");
        });
        if self.historical_territory {
            if let Some(h) = &self.generator.civilizations {
                ui.label(match h.territory_at(self.territory_export_month) {
                    Some(s)=>format!("Territory Y{} M{} · last change Y{} M{} · white: contested · colors: controllers",self.territory_export_month/12,self.territory_export_month%12+1,s.month/12,s.month%12+1),
                    None=>"No territorial record for the selected month; no current claims substituted.".into(),
                });
            }
            ui.small("Historical cells on current terrain. Claims are not continuous borders; terrain inspection remains current.");
        }
        if let Some(e) = self.journey_overlay.and_then(|id| {
            self.generator
                .civilizations
                .as_ref()?
                .events
                .get(id as usize)
        }) {
            ui.small(format!(
                "Blue: intended journey from event #{} · Y{} M{} · not a completed track",
                e.id,
                e.month / 12,
                e.month % 12 + 1
            ));
        }
        ui.add_space(12.);
        let available = ui.available_size();
        let h = (available.y - 145.).max(180.);
        match self.view_mode {
            1 => {
                let side = h.min(available.x);
                self.map(ui, 0, egui::vec2(side, side));
            }
            2 => {
                let w = available.x.min(h * 2.);
                self.map(ui, 1, egui::vec2(w, w * 0.5));
            }
            _ => {
                let side = (available.x * 0.37).min(h);
                let width = (available.x - side - 16.).max(128.);
                ui.horizontal(|ui| {
                    self.map(ui, 0, egui::vec2(side, side));
                    ui.vertical(|ui| {
                        ui.small("PLANETARY ATLAS");
                        self.map(ui, 1, egui::vec2(width, width * 0.5));
                        ui.small("The enclosed inland sea lies at the center of the atlas.");
                    });
                });
            }
        }
        ui.add_space(12.);
        let cfg = &self.generator.config;
        let mean_edge = (4. * std::f64::consts::PI * (cfg.radius_km as f64).powi(2)
            / cfg.cells() as f64)
            .sqrt();
        ui.small(format!("Terrain: {}²/face · mean cell width {:.1} km · Ecology: {}²/face. Regional relief is procedural; water and ecology retain their simulation grids.",cfg.resolution,mean_edge,cfg.eco_resolution()));
        ui.separator();
        self.inspector(ui);
    }
    fn inspector(&self, ui: &mut egui::Ui) {
        let Some((id, c)) = self.selected else {
            ui.label("Select a place on either map to inspect its climate, geology, resources, and plant life.");
            return;
        };
        if let Ok(e) =
            self.generator
                .ecology
                .inspect(&self.generator.gpu, &self.generator.config, id)
        {
            ui.label(format!("Ecology {}²: photo {:.4} + chemo {:.4} kg C/m²/month · soil N {:.5}, P {:.5} kg/m²",self.generator.config.eco_resolution(),e.pools[28][0],e.pools[28][1],e.pools[17][1],e.pools[17][2]));
            for (k, composition) in e.pools[32..38].iter().enumerate() {
                if composition[3] == 0. {
                    continue;
                }
                let name = |id: f32| {
                    self.generator
                        .catalog
                        .plants
                        .get((id as usize).wrapping_sub(1))
                        .map_or("none", |p| p.name.as_str())
                };
                ui.small(format!(
                    "{} competitors: {} {:.1}% / {} {:.1}%",
                    [
                        "Canopy",
                        "Understory",
                        "Root mat",
                        "Shallow underground",
                        "Deep fault",
                        "Aquatic producers"
                    ][k],
                    name(composition[0]),
                    composition[2] * 100.,
                    name(composition[1]),
                    (1. - composition[2]) * 100.
                ));
            }
            ui.small(format!("Layer biomass C: {:.3} / {:.3} / {:.3} / {:.3} / {:.3} · H₂ {:.4} kg/m² · reactive rock {:.1} kg/m²",e.pools[0][0],e.pools[1][0],e.pools[2][0],e.pools[3][0],e.pools[4][0],e.pools[26][0],e.pools[26][2]));
            ui.collapsing("Wildlife origins", |ui| {
                if self.generator.ecology.clock.wildlife_baseline == 0 {
                    ui.small("Legacy fauna: founder ancestry was not recorded.");
                } else {
                    ui.small(
                        "Share descended from initial outer-continent-associated populations.",
                    );
                    for k in 0..12 {
                        let animal = e.pools[k + 5];
                        if animal[0] > 1e-10 {
                            ui.small(format!(
                                "{}: {:.3e} kg C/m² · {:.1}% outer ancestry",
                                crate::ecology::GUILD_NAMES[k],
                                animal[0],
                                animal[3] * 100.
                            ));
                        }
                    }
                }
            });
            ui.small(format!("Deep-water P {:.5} kg/m² · upwelling {:.4} m/month · oxygen {:.2} · limiting factor {}",e.pools[21][2],e.pools[29][2],e.pools[31][2],["energy / habitat","nitrogen","phosphorus"][e.pools[29][3].clamp(0.,2.) as usize]));
            if let Ok(h) = self.generator.ecology.inspect_environment(
                &self.generator.gpu,
                &self.generator.config,
                id,
            ) {
                let land = (h.fields[0][2] + h.fields[0][3]).max(1e-6);
                ui.small(format!("Geological habitats (% land, nested): enriched {:.1} · active {:.1} · vents {:.2}",h.fields[8][0]/land*100.,h.fields[8][1]/land*100.,h.fields[8][2]/land*100.));
                if h.fields[21][3] == 1. {
                    for k in 3..5 {
                        let d = h.fields[16 + k];
                        ui.small(format!(
                            "{}: growth {:.6}, loss {:.6} kg C/m²/month · limited by {}",
                            crate::ecology::COMPARTMENTS[k],
                            d[1],
                            d[2],
                            crate::ecology::GROWTH_LIMITS[(d[3] as usize).min(6)]
                        ));
                    }
                } else {
                    ui.small("Growth diagnostics available after the next ecology step.");
                }
            }
        }
        if let Ok(e) =
            self.generator
                .ecology
                .inspect(&self.generator.gpu, &self.generator.config, id)
        {
            let names: Vec<_> = e.pools[..5]
                .iter()
                .map(|p| {
                    self.generator
                        .catalog
                        .plants
                        .get((p[3] as usize).wrapping_sub(1))
                        .map_or("none", |p| p.name.as_str())
                })
                .collect();
            ui.small(format!("Producers by layer: {}", names.join(" / ")));
            let animals: Vec<_> = self
                .generator
                .catalog
                .guilds
                .iter()
                .enumerate()
                .filter(|(i, _)| e.pools[i + 5][0] > 1e-6)
                .map(|(i, g)| {
                    format!(
                        "{} {:.2}/km²",
                        g.name,
                        e.pools[i + 5][0] * 2. * 1e6 / g.body_mass_kg
                    )
                })
                .collect();
            ui.small(format!("Guild equivalents: {}", animals.join(" · ")));
        }
        let cat = &self.generator.catalog;
        let region = [
            "Exterior ocean",
            "Great lake",
            "Inner continent",
            "Outer continent",
        ][c.meta[0] as usize];
        ui.label(egui::RichText::new(format!("CELL {id}   /   {region}")).strong());
        ui.label(format!("Elevation {:.0} m   ·   {:.1} °C   ·   {:.0} mm/year   ·   Water {:.2} m   ·   River {:.1} m³/s",c.terrain[0],c.hydro[1],c.hydro[2],c.water[0],c.water[3]));
        ui.label(format!(
            "Rock sequence: {} / {} / {}  ·  {}",
            cat.rocks[c.ids[0] as usize].name,
            cat.rocks[c.meta[2] as usize].name,
            cat.rocks[c.meta[3] as usize].name,
            cat.soils[c.ids[1] as usize].name
        ));
        if c.strata.iter().any(|v| *v > 0.) {
            ui.label(format!(
                "Column thickness: {:.2} / {:.2} / {:.2} m · bedrock removed {:.3} m",
                c.strata[0], c.strata[1], c.strata[2], c.strata[3]
            ));
        } else {
            ui.small("Column thickness unavailable in this legacy baseline.");
        }
        let plant = cat
            .plants
            .get(c.ids[2] as usize)
            .map_or("None", |p| p.name.as_str());
        let mineral = cat
            .minerals
            .get(c.meta[1] as usize)
            .map_or("None", |m| m.name.as_str());
        if let Some(source) = self
            .generator
            .civilizations
            .as_ref()
            .and_then(|h| h.resources.as_ref())
            .and_then(|r| r.sources.get(&id))
        {
            ui.label(format!(
                "Shared accessible source: ore {:.0}/{:.0} kg · clay {:.0}/{:.0} kg",
                source.remaining[0], source.initial[0], source.remaining[1], source.initial[1]
            ));
            if let Some(mineral) = &source.mineral {
                ui.small(format!(
                    "Source mineral: {mineral} · {}",
                    if source.ore_good.is_some() {
                        "compatible processing available"
                    } else {
                        "no compatible processing chain yet"
                    }
                ));
            } else {
                ui.small("Legacy economic ore/clay proxy; mineral composition unresolved.");
            }
        }
        ui.label(format!("Plant: {plant}  ·  Cover {:.0}%  ·  Resource: {mineral} ({:.0}% potential, {:.0} m depth)",c.life[0]*100.,c.geology[2]*100.,c.geology[3]));
        if let Some(m) = cat.minerals.get(c.meta[1] as usize) {
            ui.small(format!(
                "Formation setting: {} · province scale {:.0} km",
                m.deposit_setting.label(),
                m.province_scale_km
            ));
        }
        let downstream = if c.routing[0] == gpu::NONE {
            "terminal".into()
        } else {
            c.routing[0].to_string()
        };
        let basin = if c.routing[2] == gpu::NONE {
            "none".into()
        } else {
            c.routing[2].to_string()
        };
        ui.small(format!("Drainage → {downstream}  ·  Basin {basin}  ·  Groundwater {:.2} m  ·  Salinity {:.3} g/kg",c.water[1],c.life[2]));
    }
}

pub fn run(
    config: Config,
    catalog: Catalog,
    load: Option<std::path::PathBuf>,
    smoke: Option<std::path::PathBuf>,
) -> Result<()> {
    let setup = egui_wgpu::WgpuSetupCreateNew {
        device_descriptor: Arc::new(gpu::device_descriptor),
        ..Default::default()
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440., 900.])
            .with_min_inner_size([1000., 680.]),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: egui_wgpu::WgpuConfiguration {
            wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(setup),
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "Ancient World",
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, config, catalog, load, smoke)?))),
    )
    .map_err(|e| anyhow::anyhow!("Desktop application: {e}"))
}

pub(crate) fn political_color(id: u32) -> egui::Color32 {
    let hue = (id as f32 * 0.618_034) % 1.;
    egui::ecolor::Hsva::new(hue, 0.65, 0.95, 1.).into()
}
