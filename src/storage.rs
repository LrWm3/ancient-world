use crate::ecology::ENVIRONMENT_BYTES;
use crate::{
    catalog::Catalog,
    config::Config,
    gpu::{validate_cells, Cell, ContextGpu, Generator, Progress, Stage},
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};
const MAGIC: &[u8; 8] = b"ANCIENT2";
#[derive(Serialize, Deserialize)]
struct Header {
    version: u32,
    #[serde(default)]
    civilizations: Option<crate::civilization::History>,
    planet: [f32; 4],
    config: Config,
    catalog: Catalog,
    progress: Progress,
    grid: String,
    units: String,
    rng: String,
    #[serde(default)]
    ecology: Option<crate::ecology::EcologyClock>,
    #[serde(default)]
    ecological_fields: Vec<String>,
}
fn checksum(bytes: &[u8], mut value: u64) -> u64 {
    for b in bytes {
        value ^= *b as u64;
        value = value.wrapping_mul(0x100000001b3);
    }
    value
}
impl Generator {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        ensure!(
            self.progress.stage == Stage::Boundary,
            "checkpoints must be saved at an epoch boundary"
        );
        self.validate_living_boundary()?;
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let mut ecological_fields: Vec<String> = crate::ecology::COMPARTMENTS
            .iter()
            .map(|n| format!("{n}: kg C,N,P per m²; fourth component reserved or producer index"))
            .collect();
        for field in &mut ecological_fields[5..17] {
            *field = field.replace(
                "fourth component reserved or producer index",
                "fourth component: outer-associated founder ancestry fraction (baseline 1 only)",
            );
        }
        ecological_fields[24] =
            "buried/sorbed C,N,P kg/m²; fourth component: reserved managed fraction of whole cell"
                .into();
        ecological_fields.extend(
            [
                "energy: H2 kg/m², secondary MJ/m², reactive rock kg/m², oxidant kg/m²",
                "external ledger: kg C,N,P per m²; water m",
                "production: photo C, chemo C, respiration C, fixed N in kg/m²/month",
                "circulation: tangent x,y; vertical exchange m/month; limiting resource code",
                "initial inventory: kg C,N,P per m²; water m",
                "controls: geochemical supply; mixing; oxygen availability; disabled guild bitmask",
            ]
            .map(str::to_string),
        );
        ecological_fields.extend((0..6).map(|k| format!("producer composition {k}: first ID+1, second ID+1, first biomass fraction, initialized")));
        let header=Header {
            version:7,civilizations:self.civilizations.clone(),ecology:Some(self.ecology.clock.clone()),ecological_fields,
            planet:self.planet_state()?, config:self.config.clone(), catalog:self.catalog.clone(), progress:self.progress.clone(),
            grid:"cube-sphere; faces +X,-X,+Y,-Y,+Z,-Z; row-major cells; payload: terrain[176 bytes/cell; final vec4: top/middle/basement thickness and cumulative removed bedrock in m], ecology[608 bytes/ecocell; final six vec4 are producer composition metadata], environment[400 bytes/ecocell; habitat fractions, conditional habitats, last-step diagnostics, wildlife edge conductance], routed C/N/P/water[16 bytes/cell, kg/kg/kg/m3]".into(),
            units:"terrain:m,m,m,Myr; climate:C,mm/year,mm,m/s; water:m,m,m,m3/s; life:fraction,fraction,g/kg,m/step; geology:stress,km,probability,m; hydro:spill_m,mean_C,mean_mm/year,net_m/step; budget:rain_m,evap_m,eroded_m,deposited_m; ecological stocks normalized by total cell area, with separate land/water compartments; source-rock fourth component records total local water inventory including routed water; civilization v2 managed C/N/P and goods in kg, water in m3, plots in m2, cash in abstract currency, recipe labor in worker-months".into(),
            rng:"counter hash(seed, cell, epoch, stream); ecological forcing keyed by month; no hidden mutable RNG".into()
        };
        let metadata = serde_json::to_vec(&header)?;
        ensure!(
            metadata.len() <= 256 * 1024 * 1024,
            "archive metadata exceeds 256 MiB"
        );
        let cells = self.snapshot()?;
        let temporary = path.with_extension("world.tmp");
        let file = File::create(&temporary)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(MAGIC)?;
        writer.write_all(&(metadata.len() as u64).to_le_bytes())?;
        writer.write_all(&metadata)?;
        let mut sum = checksum(&metadata, 0xcbf29ce484222325);
        ensure!(
            cfg!(target_endian = "little"),
            "world archives currently require a little-endian host"
        );
        for chunk in cells.chunks(4096) {
            let bytes = bytemuck::cast_slice(chunk);
            writer.write_all(bytes)?;
            sum = checksum(bytes, sum);
        }
        let eco = self.ecology.snapshot(&self.gpu, &self.config)?;
        let env = crate::gpu::read_buffer(
            &self.gpu,
            &self.ecology.environment,
            0,
            self.config.eco_cells() as u64 * ENVIRONMENT_BYTES,
        )?;
        let rivers = crate::gpu::read_buffer(
            &self.gpu,
            &self.ecology.rivers[0],
            0,
            self.config.cells() as u64 * 16,
        )?;
        for bytes in [
            bytemuck::cast_slice(&eco),
            env.as_slice(),
            rivers.as_slice(),
        ] {
            writer.write_all(bytes)?;
            sum = checksum(bytes, sum);
        }
        writer.write_all(&sum.to_le_bytes())?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        drop(writer);
        fs::rename(temporary, path)?;
        Ok(())
    }
    pub fn load(gpu: ContextGpu, path: impl AsRef<Path>) -> Result<Self> {
        Self::load_archive(gpu, path.as_ref(), false)
    }
    pub fn import_v1(gpu: ContextGpu, path: impl AsRef<Path>) -> Result<Self> {
        Self::load_archive(gpu, path.as_ref(), true)
    }
    fn load_archive(gpu: ContextGpu, path: &Path, allow_legacy: bool) -> Result<Self> {
        let file = File::open(path).context("opening world archive")?;
        let length = file.metadata()?.len();
        ensure!(length <= 4_500_000_000, "world archive exceeds size limit");
        let mut r = BufReader::new(file);
        let mut magic = [0; 8];
        r.read_exact(&mut magic)?;
        let legacy = &magic == b"ANCIENT1";
        ensure!(&magic==MAGIC || (legacy&&allow_legacy), "unsupported archive; use explicit import-v1 for an Ancient World v1 ecological baseline");
        let mut bytes = [0; 8];
        r.read_exact(&mut bytes)?;
        let len = u64::from_le_bytes(bytes);
        ensure!(len <= 256 * 1024 * 1024, "archive metadata is too large");
        let mut metadata = vec![0; len as usize];
        r.read_exact(&mut metadata)?;
        let mut header: Header = serde_json::from_slice(&metadata)?;
        ensure!(
            (if legacy {
                header.version == 1
            } else {
                (2..=7).contains(&header.version)
            }) && header.progress.stage == Stage::Boundary,
            "unsupported or incomplete checkpoint"
        );
        let terrain_stride = if header.version >= 6 {
            crate::gpu::CELL_BYTES as usize
        } else {
            160
        };
        let eco_stride = if header.version >= 5 {
            crate::ecology::ECO_BYTES as usize
        } else {
            512
        };
        let env_stride = if header.version >= 7 {
            ENVIRONMENT_BYTES
        } else if header.version >= 4 {
            352
        } else {
            128
        };
        header.config.validate()?;
        ensure!(
            header.progress.geological_time_myr.is_finite()
                && header.progress.geological_time_myr >= 0.,
            "invalid geological clock"
        );
        header.catalog.validate()?;
        ensure!(
            length
                == 16
                    + len
                    + header.config.cells() as u64 * terrain_stride as u64
                    + 8
                    + if legacy {
                        0
                    } else {
                        header.config.eco_cells() as u64 * (eco_stride as u64 + env_stride)
                            + header.config.cells() as u64 * 16
                    },
            "archive length does not match grid"
        );
        let mut cells = vec![Cell::default(); header.config.cells() as usize];
        let mut bytes = vec![0u8; cells.len() * terrain_stride];
        r.read_exact(&mut bytes)?;
        let mut sum = checksum(&bytes, checksum(&metadata, 0xcbf29ce484222325));
        for (cell, chunk) in cells.iter_mut().zip(bytes.chunks_exact(terrain_stride)) {
            bytemuck::bytes_of_mut(cell)[..terrain_stride].copy_from_slice(chunk);
        }
        let mut eco = vec![];
        let mut env = vec![];
        let mut rivers = vec![];
        if !legacy {
            eco.resize(header.config.eco_cells() as usize * eco_stride, 0);
            env.resize(header.config.eco_cells() as usize * env_stride as usize, 0);
            rivers.resize(header.config.cells() as usize * 16, 0);
            for b in [&mut eco, &mut env, &mut rivers] {
                r.read_exact(b)?;
                sum = checksum(b, sum);
            }
            let ecocells: Vec<crate::ecology::EcoCell> = eco
                .chunks_exact(eco_stride)
                .map(|bytes| {
                    let mut c = crate::ecology::EcoCell::default();
                    bytemuck::bytes_of_mut(&mut c)[..eco_stride].copy_from_slice(bytes);
                    c
                })
                .collect();
            crate::ecology::validate(&ecocells)?;
            ensure!(
                env.chunks_exact(4)
                    .chain(rivers.chunks_exact(4))
                    .all(|x| f32::from_le_bytes(x.try_into().unwrap()).is_finite()),
                "invalid ecological archive data"
            );
        }
        let mut stored = [0; 8];
        r.read_exact(&mut stored)?;
        ensure!(
            sum == u64::from_le_bytes(stored),
            "archive checksum mismatch"
        );
        if !legacy && eco_stride != crate::ecology::ECO_BYTES as usize {
            let mut expanded =
                vec![0; header.config.eco_cells() as usize * crate::ecology::ECO_BYTES as usize];
            for (old, new) in eco
                .chunks_exact(eco_stride)
                .zip(expanded.chunks_exact_mut(crate::ecology::ECO_BYTES as usize))
            {
                new[..eco_stride].copy_from_slice(old);
            }
            eco = expanded;
        }
        // Older environments keep their exact physical fields; derived habitats are
        // rebuilt at the next monthly aggregation. No stocks or clocks are reset.
        if !legacy && env_stride != ENVIRONMENT_BYTES {
            let mut expanded =
                vec![0; header.config.eco_cells() as usize * ENVIRONMENT_BYTES as usize];
            for (old, new) in env
                .chunks_exact(env_stride as usize)
                .zip(expanded.chunks_exact_mut(ENVIRONMENT_BYTES as usize))
            {
                new[..env_stride as usize].copy_from_slice(old);
            }
            env = expanded;
        }
        validate_cells(&cells, &header.catalog)?;
        if legacy {
            let bundled = Catalog::bundled()?;
            header.catalog.guilds = bundled.guilds;
            header.catalog.microbes = bundled.microbes;
            for plant in bundled
                .plants
                .into_iter()
                .filter(|v| v.id.starts_with("eco_"))
            {
                if !header.catalog.plants.iter().any(|v| v.id == plant.id) {
                    header.catalog.plants.push(plant);
                }
            }
            for biome in bundled
                .biomes
                .into_iter()
                .filter(|v| v.ecological_habitat > 0)
            {
                if !header.catalog.biomes.iter().any(|v| v.id == biome.id) {
                    header.catalog.biomes.push(biome);
                }
            }
            header.catalog.version = 2;
        }
        let mut generator = Self::new(gpu, header.config, header.catalog)?;
        generator.restore_cells(&cells, header.progress.epoch)?;
        ensure!(
            header.planet.iter().all(|x| x.is_finite())
                && (85.0..=240.0).contains(&header.planet[0])
                && header.planet[1] >= 0.,
            "invalid planet reservoir state"
        );
        generator
            .gpu
            .queue
            .write_buffer(&generator.planet, 0, bytemuck::bytes_of(&header.planet));
        generator.progress = header.progress;
        generator.progress.timestamp_supported = generator
            .gpu
            .device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY);
        if legacy {
            generator.ecology.clock.imported_baseline = true;
            generator.ecology.prepare(
                &generator.gpu,
                &generator.buffers[generator.current],
                &generator.catalog_buffer,
                &generator.config,
                &generator.catalog,
                false,
            );
        } else {
            let clock = header.ecology.context("archive lacks ecological clocks")?;
            ensure!(
                clock.epoch_month == 0
                    && clock.month <= u32::MAX as u64
                    && clock
                        .events
                        .iter()
                        .all(|e| e.month <= clock.month && e.region.is_none_or(|r| r < 4)),
                "invalid ecological clock or event history"
            );
            generator.gpu.queue.write_buffer(
                &generator.ecology.buffers[generator.ecology.current],
                0,
                &eco,
            );
            generator
                .gpu
                .queue
                .write_buffer(&generator.ecology.environment, 0, &env);
            generator
                .gpu
                .queue
                .write_buffer(&generator.ecology.rivers[0], 0, &rivers);
            generator.ecology.clock = clock;
        }
        if let Some(mut history) = header.civilizations {
            if header.version < 3 {
                for c in &mut history.cargo {
                    if c.good == 8 {
                        c.good = crate::economy::FOOD as u32;
                    }
                }
            }
            history.initialize_legacy_culture()?;
            ensure!(
                history.seed == generator.config.seed
                    && history.terrain_resolution == generator.config.resolution
                    && history.source_epoch == generator.progress.epoch,
                "civilization provenance does not match planet"
            );
            history.validate(&generator.snapshot()?)?;
            generator.validate_economic_grid(&history)?;
            generator.civilizations = Some(history);
        }
        generator.ecology.living = generator
            .civilizations
            .as_ref()
            .is_some_and(|h| h.living.is_some());
        generator.validate_living_boundary()?;
        Ok(generator)
    }
}
