use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{validate_cells, ContextGpu, Generator},
    viewer::{self, Camera, MapRenderer},
};
use anyhow::{Context, Result};
use clap::Parser;
use std::{path::PathBuf, time::Instant};
#[derive(Parser)]
#[command(about = "GPU natural-history planet generator and explorer")]
struct Args {
    #[arg(long)]
    headless: bool,
    /// Found this many central-island civilizations (1–16), then optionally evolve history.
    #[arg(long)]
    civilizations: Option<u32>,
    /// Editable patron archetypes for a new founding; the full catalog is archived.
    #[arg(long, requires = "civilizations")]
    patron_catalog: Option<PathBuf>,
    /// Matched founding control: retain patrons and provisions but disable practical aid.
    #[arg(long, requires = "civilizations")]
    no_patron_aid: bool,
    /// Explicitly upgrade a saved first-beta history to managed ecological farming and markets.
    #[arg(long)]
    upgrade_economy: bool,
    /// Enable households, seasonal crops, councils and terrain routes at a new baseline.
    #[arg(long)]
    society: bool,
    /// Start explicit genealogy, factions, claims and territorial warfare on social history.
    #[arg(long)]
    politics: bool,
    /// Enable local legitimacy, administration costs and diplomacy on political history.
    #[arg(long)]
    governance: bool,
    /// Establish local offices after enabling governance.
    #[arg(long)]
    offices: bool,
    /// Enable surveyed inter-island shipping, requiring social history.
    #[arg(long)]
    shipping: bool,
    /// Enable finite research and rescue voyages (requires shipping and governance).
    #[arg(long)]
    expeditions: bool,
    /// Enable finite specimens and research workshops on an expedition baseline.
    #[arg(long)]
    discoveries: bool,
    /// Advance the GPU seasonal environment with each social month.
    #[arg(long)]
    living_world: bool,
    /// Editable economy TOML; archived with the history.
    #[arg(long)]
    economy_catalog: Option<PathBuf>,
    /// Advance social years; --living-world couples monthly planetary ecology.
    #[arg(long, default_value_t = 0)]
    history_years: u32,
    /// Export civilization records, settlements and events as JSON.
    #[arg(long)]
    history_export: Option<PathBuf>,
    /// Independent ecology grid edge (default 256, capped by terrain resolution).
    #[arg(long)]
    ecology_resolution: Option<u32>,
    #[arg(long)]
    ecology_years: Option<u32>,
    /// Advance ecology-only months after geological history.
    #[arg(long, default_value_t = 0)]
    months: u32,
    /// JSON array of timestamped ecological interventions for the ecology-only interval.
    #[arg(long)]
    scenarios: Option<PathBuf>,
    #[arg(long)]
    budget: Option<PathBuf>,
    /// Explicitly import a v1 terrain archive and initialize a new ecological baseline.
    #[arg(long)]
    import_v1: Option<PathBuf>,
    /// Evolve two epochs in the desktop, capture its rendered viewport, and exit.
    #[arg(long)]
    smoke_test: Option<PathBuf>,
    #[arg(long, default_value_t = 512)]
    resolution: u32,
    #[arg(long, default_value_t = 42)]
    seed: u32,
    /// Epochs to run in headless mode (desktop starts paused).
    #[arg(long, default_value_t = 1)]
    epochs: u32,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    catalog: Option<PathBuf>,
    #[arg(long)]
    load: Option<PathBuf>,
    #[arg(long)]
    save: Option<PathBuf>,
    #[arg(long)]
    export: Option<PathBuf>,
    /// Export layer index (0 natural world ... 14 vegetation; 15–30 ecological budgets).
    #[arg(long,default_value_t=0,value_parser=clap::value_parser!(u32).range(0..=30))]
    layer: u32,
    /// Run one epoch at each production resolution and write a JSON timing report.
    #[arg(long)]
    benchmark: bool,
}
fn main() -> Result<()> {
    let args = Args::parse();
    anyhow::ensure!(
        args.headless
            || (args.civilizations.is_none()
                && args.history_years == 0
                && args.history_export.is_none()
                && !args.society
                && !args.politics
                && !args.governance
                && !args.shipping
                && !args.expeditions
                && !args.discoveries
                && !args.living_world
                && !args.upgrade_economy
                && args.economy_catalog.is_none()),
        "civilization CLI options require --headless; use the civilization panel in desktop mode"
    );
    let mut config = if let Some(path) = args.config {
        toml::from_str(&std::fs::read_to_string(path)?)?
    } else {
        Config {
            resolution: args.resolution,
            seed: args.seed,
            ..Default::default()
        }
    };
    if let Some(n) = args.ecology_resolution {
        config.ecology_resolution = n;
    }
    if let Some(y) = args.ecology_years {
        config.ecology_years_per_epoch = y;
    }
    config.validate()?;
    let catalog = if let Some(path) = args.catalog {
        Catalog::parse(&std::fs::read_to_string(path)?)?
    } else {
        Catalog::bundled()?
    };
    if args.benchmark {
        let gpu = pollster::block_on(ContextGpu::headless())?;
        let mut reports = Vec::new();
        for n in [256, 512, 1024] {
            let start = Instant::now();
            let mut generator = Generator::new(
                gpu.clone(),
                Config {
                    resolution: n,
                    ..config.clone()
                },
                catalog.clone(),
            )?;
            generator.run_epochs(args.epochs)?;
            let report = serde_json::json!({"resolution":n,"cells":generator.config.cells(),"epochs":args.epochs,"seconds":start.elapsed().as_secs_f64(),"estimated_gpu_bytes":generator.config.estimated_bytes(),"gpu":gpu.adapter_name,"timings":generator.progress.stage_ms,"gpu_timestamps":generator.progress.timestamp_supported});
            println!("{report}");
            reports.push(report);
        }
        std::fs::create_dir_all("output")?;
        std::fs::write(
            "output/benchmark.json",
            serde_json::to_string_pretty(&reports)?,
        )?;
        return Ok(());
    }
    anyhow::ensure!(
        args.headless
            || (args.import_v1.is_none()
                && args.scenarios.is_none()
                && args.months == 0
                && args.budget.is_none()),
        "import-v1, months, scenarios and budget require --headless"
    );
    if !args.headless {
        return viewer::run(config, catalog, args.load, args.smoke_test);
    }
    let gpu = pollster::block_on(ContextGpu::headless())?;
    eprintln!("GPU: {}", gpu.adapter_name);
    let start = Instant::now();
    let mut generator = if let Some(path) = args.import_v1 {
        Generator::import_v1(gpu, path)?
    } else if let Some(path) = args.load {
        Generator::load(gpu, path)?
    } else {
        Generator::new(gpu, config, catalog)?
    };
    let target = generator
        .progress
        .epoch
        .checked_add(args.epochs)
        .context("epoch target overflow")?;
    let mut last = Instant::now();
    while generator.progress.epoch < target {
        generator.advance()?;
        if last.elapsed().as_secs() >= 5 {
            eprintln!(
                "epoch {} · {} · pass {} · {} changed",
                generator.progress.epoch,
                generator.progress.stage.label(),
                generator.progress.iteration,
                generator.progress.changed
            );
            last = Instant::now();
        }
    }
    let events: Vec<ancient_world::ecology::ScenarioEvent> = if let Some(path) = args.scenarios {
        serde_json::from_str(&std::fs::read_to_string(path)?)?
    } else {
        vec![]
    };
    let end = generator.ecology.clock.month + args.months as u64;
    anyhow::ensure!(
        events
            .iter()
            .all(|e| e.month >= generator.ecology.clock.month && e.month <= end),
        "scenario timestamp outside ecology-only interval"
    );
    loop {
        for event in events
            .iter()
            .filter(|e| e.month == generator.ecology.clock.month)
            .cloned()
            .collect::<Vec<_>>()
        {
            generator.scenario(event.region, event.intervention)?;
        }
        if generator.ecology.clock.month == end {
            break;
        }
        generator.advance_ecology()?;
    }
    if let Some(count) = args.civilizations {
        let patrons = if let Some(path) = args.patron_catalog {
            toml::from_str(&std::fs::read_to_string(path)?)?
        } else {
            ancient_world::culture::PatronCatalog::bundled()?
        };
        generator.found_civilizations_with_catalog(
            count,
            ancient_world::culture::FoundingOptions {
                aid_enabled: !args.no_patron_aid,
                ..Default::default()
            },
            patrons,
        )?;
    }
    if args.upgrade_economy {
        generator.upgrade_economy()?;
    }
    if let Some(path) = args.economy_catalog {
        generator.configure_economy(toml::from_str(&std::fs::read_to_string(path)?)?)?;
    }
    if args.society {
        generator.enable_society()?;
    }
    if args.politics {
        generator.enable_politics()?;
    }
    if args.governance {
        generator.enable_governance()?;
    }
    if args.offices {
        generator.enable_offices()?;
    }
    if args.shipping {
        generator.enable_shipping()?;
    }
    if args.expeditions {
        generator.enable_expeditions()?;
    }
    if args.discoveries {
        generator.enable_discoveries()?;
    }
    if args.living_world {
        generator.enable_living_history()?;
    }
    if args.history_years > 0 {
        generator.advance_history(
            args.history_years
                .checked_mul(12)
                .context("history duration overflow")?,
        )?;
    }
    validate_cells(&generator.snapshot()?, &generator.catalog)?;
    ancient_world::ecology::validate(
        &generator
            .ecology
            .snapshot(&generator.gpu, &generator.config)?,
    )?;
    let budget = generator
        .ecology
        .budget(&generator.gpu, &generator.config)?;
    if let Some(path) = args.budget {
        if let Some(parent) = path.parent().filter(|v| !v.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(&budget)?)?;
    }
    if !budget.within_tolerance {
        eprintln!("Ecological budget residual exceeds the 0.1% reporting tolerance");
    }
    eprintln!(
        "Ecological C/N/P relative budget residual: {:?}",
        budget.relative_error
    );
    if let Some(h) = &generator.civilizations {
        eprintln!("Civilization year {}: {} settlements, {:.0} people; food/population residual {:.6}% / {:.6}%",
            h.month / 12, h.sites.len(), h.sites.iter().map(|s| s.stocks.stock[0]).sum::<f32>(),
            h.food_residual() * 100., h.population_residual() * 100.);
    }
    if let Some(h) = generator.civilizations.as_ref().filter(|h| h.version == 2) {
        eprintln!(
            "Managed C/N/P, water, money, goods residuals: {:?}",
            h.economy_residuals()
        );
    }
    if let Some(path) = args.history_export {
        let history = generator
            .civilizations
            .as_ref()
            .context("no civilizations to export")?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(history)?)?;
    }
    if let Some(path) = args.save {
        generator.save(path)?;
    }
    if let Some(path) = args.export {
        let map = MapRenderer::new(&generator, 2048, 1024)?;
        map.render(&generator, args.layer, Camera::atlas(), None);
        map.export_png(&generator, path)?;
    }
    println!(
        "{}",
        serde_json::json!({"epochs":generator.progress.epoch,"cells":generator.config.cells(),"seconds":start.elapsed().as_secs_f64(),"stage_ms":generator.progress.stage_ms,"gpu_timestamps":generator.progress.timestamp_supported})
    );
    Ok(())
}
