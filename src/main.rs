use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{validate_cells, ContextGpu, Generator},
    systems::System,
    viewer::{self, Camera, MapRenderer},
};
use anyhow::{Context, Result};
use clap::Parser;
use std::{path::PathBuf, time::Instant};

const BENCHMARK_RESOLUTIONS: [u32; 3] = [256, 512, 1024];
const PROGRESS_INTERVAL_SECONDS: u64 = 5;
const EXPORT_ATLAS_WIDTH_PX: u32 = 2048;
const EXPORT_ATLAS_HEIGHT_PX: u32 = 1024;

#[derive(Parser)]
#[command(about = "GPU natural-history planet generator and explorer")]
struct Args {
    #[arg(long)]
    headless: bool,
    /// Explicitly enable startup systems (comma-separated; all default on for new histories).
    #[arg(long, value_enum, value_delimiter = ',')]
    enable_system: Vec<System>,
    /// Disable startup systems and their dependents before founding.
    #[arg(long, value_enum, value_delimiter = ',')]
    disable_system: Vec<System>,
    /// Found this many civilizations on the inner continents (1–16), then optionally evolve history.
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
    /// Enable surveyed shipping between the inner continents, requiring social history.
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
    /// Advance social years; new histories include monthly planetary ecology by default.
    #[arg(long, default_value_t = 0)]
    history_years: u32,
    /// Override managed crop yield for subsequent history (0.1–1), including loaded worlds.
    #[arg(long)]
    crop_yield_scale: Option<f32>,
    /// Experimental council tax-bridge lending; false stops new loans, not repayment.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    council_credit: Option<bool>,
    /// Allow surplus local institutional offers when council credit is enabled.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    institution_credit_lenders: Option<bool>,
    /// Reserve institutional annual operating costs instead of the council cash floor.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    institution_credit_operating_reserve: Option<bool>,
    /// Experimental town working-capital loans against delivery-paid exports.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    commercial_credit: Option<bool>,
    /// Include funded workshop orders when commercial credit is enabled.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    service_order_credit: Option<bool>,
    /// Reserve surplus town cash for next-month workshop service orders.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    service_order_procurement: Option<bool>,
    /// Include due service contracts in desired workshop shifts; cash and worker caps remain.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    contract_workshop_staffing: Option<bool>,
    /// Cap workshop shifts by current recipe demand; preserves old policy if omitted.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    demand_workshop_staffing: Option<bool>,
    /// Allow local recorded-descendant inheritance of vacant household estates.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    household_estate_inheritance: Option<bool>,
    /// Purchase household cloth from finite town stocks after protecting food cash.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    household_clothing: Option<bool>,
    /// Annual progressive household cash tax above protected food reserves.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    household_wealth_tax: Option<bool>,
    /// Cover full household food budgets using council reserves above administrative needs.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    council_welfare_reserves: Option<bool>,
    /// Let full dietary need access existing town food regardless of wallet cash.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    needs_based_food: Option<bool>,
    /// Use accumulated nutritional stress for aggregate mortality and illness.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    gradual_nutrition: Option<bool>,
    /// Fund local experiments to recover missing production knowledge.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    practical_research: Option<bool>,
    /// Review unclaimed household cash using delivered named office work.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    household_estate_reclamation: Option<bool>,
    /// Buy retained bulk stocks using buyer-provided round-trip land freight.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    abandoned_stock_recovery: Option<bool>,
    /// Reserve and settle actual named administrative attendance.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    named_office_service: Option<bool>,
    /// Override the fraction of surplus town cash available to service procurement (0–1).
    /// Does not itself enable procurement; omission preserves the archived share.
    #[arg(long, value_parser = parse_procurement_share)]
    service_procurement_share: Option<f64>,
    /// Recover old export defaults from bounded newly received delivery proceeds.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    export_default_recovery: Option<bool>,
    /// Bounded, dated shared-currency issuance into council treasuries.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    shared_issuance: Option<bool>,
    /// Newly funded export orders pay on delivery; false retains dispatch payment.
    #[arg(long, num_args = 0..=1, default_missing_value = "true")]
    delivery_paid_exports: Option<bool>,
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
    #[arg(long, default_value_t = ancient_world::config::DEFAULT_RESOLUTION)]
    resolution: u32,
    #[arg(long, default_value_t = ancient_world::config::DEFAULT_SEED)]
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
fn parse_procurement_share(value: &str) -> std::result::Result<f64, String> {
    let share: f64 = value
        .parse()
        .map_err(|_| "expected a numeric procurement share")?;
    if !share.is_finite() || !(0.0..=1.0).contains(&share) {
        return Err("procurement share must be finite and within 0–1".into());
    }
    Ok(share)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut overrides = std::collections::BTreeMap::new();
    for &system in &args.disable_system {
        overrides.insert(system, false);
    }
    let mut enable = args.enable_system.clone();
    for (requested, system) in [
        (args.society, System::Society),
        (args.politics, System::Politics),
        (args.governance, System::Governance),
        (args.offices, System::Offices),
        (args.shipping, System::Shipping),
        (args.expeditions, System::Expeditions),
        (args.discoveries, System::Discoveries),
        (args.living_world, System::LivingWorld),
    ] {
        if requested {
            enable.push(system);
        }
    }
    for system in enable {
        anyhow::ensure!(
            overrides.get(&system) != Some(&false),
            "{} was both enabled and disabled",
            system.label()
        );
        overrides.insert(system, true);
    }
    anyhow::ensure!(
        args.headless
            || (args.civilizations.is_none()
                && args.history_years == 0
                && args.crop_yield_scale.is_none()
                && args.history_export.is_none()
                && args.council_credit.is_none()
                && args.institution_credit_lenders.is_none()
                && args.institution_credit_operating_reserve.is_none()
                && args.commercial_credit.is_none()
                && args.service_order_credit.is_none()
                && args.service_order_procurement.is_none()
                && args.contract_workshop_staffing.is_none()
                && args.demand_workshop_staffing.is_none()
                && args.household_estate_inheritance.is_none()
                && args.household_clothing.is_none()
                && args.household_wealth_tax.is_none()
                && args.council_welfare_reserves.is_none()
                && args.needs_based_food.is_none()
                && args.gradual_nutrition.is_none()
                && args.practical_research.is_none()
                && args.household_estate_reclamation.is_none()
                && args.abandoned_stock_recovery.is_none()
                && args.named_office_service.is_none()
                && args.service_procurement_share.is_none()
                && args.export_default_recovery.is_none()
                && args.shared_issuance.is_none()
                && args.delivery_paid_exports.is_none()
                && !args.society
                && !args.politics
                && !args.offices
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
    config.systems.overrides.extend(overrides.clone());
    if let Some(scale) = args.crop_yield_scale {
        config.crop_yield_scale = scale;
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
        for n in BENCHMARK_RESOLUTIONS {
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
            let report = serde_json::json!({"resolution":n,"cells":generator.config.cells(),"epochs":args.epochs,"seconds":start.elapsed().as_secs_f64(),"estimated_gpu_bytes":generator.config.estimated_bytes(),"gpu":gpu.adapter_name,"timings":generator.progress.stage_ms,"gpu_timestamps":generator.progress.timestamp_supported,"lake_iterations":generator.progress.lake_iterations,"lake_changed_cells":generator.progress.lake_changed_cells,"lake_max_change_m":generator.progress.lake_max_change_m});
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
    let requested_systems = config.systems.clone();
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
    // Loading restores the archived config; apply the explicit intervention to
    // that completed boundary as well. Monthly production uploads this field.
    if let Some(scale) = args.crop_yield_scale {
        generator.config.crop_yield_scale = scale;
        generator.config.validate()?;
    }
    let target = generator
        .progress
        .epoch
        .checked_add(args.epochs)
        .context("epoch target overflow")?;
    let mut last = Instant::now();
    while generator.progress.epoch < target {
        generator.advance()?;
        if last.elapsed().as_secs() >= PROGRESS_INTERVAL_SECONDS {
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
    if args.civilizations.is_some() || !requested_systems.overrides.is_empty() {
        let mut systems = generator.config.systems.clone();
        systems.overrides.extend(requested_systems.overrides);
        generator.apply_systems(&systems)?;
    }
    if let Some(enabled) = args.council_credit {
        generator
            .civilizations
            .as_mut()
            .context("council credit requires a history")?
            .credit
            .council_policy
            .enabled = enabled;
    }
    if let Some(enabled) = args.institution_credit_lenders {
        generator
            .civilizations
            .as_mut()
            .context("institution credit requires a history")?
            .credit
            .council_policy
            .institution_lenders = enabled;
    }
    if let Some(enabled) = args.institution_credit_operating_reserve {
        generator
            .civilizations
            .as_mut()
            .context("institution credit requires a history")?
            .credit
            .council_policy
            .institution_reserve = if enabled {
            ancient_world::credit::councils::InstitutionReserve::AnnualOperatingCosts
        } else {
            ancient_world::credit::councils::InstitutionReserve::CouncilFloor
        };
    }
    if let Some(enabled) = args.shared_issuance {
        generator
            .civilizations
            .as_mut()
            .context("shared issuance requires a history")?
            .configure_shared_issuance(enabled)?;
    }
    if let Some(enabled) = args.commercial_credit {
        generator
            .civilizations
            .as_mut()
            .context("commercial credit requires a history")?
            .credit
            .commercial_policy
            .enabled = enabled;
    }
    if let Some(enabled) = args.service_order_credit {
        generator
            .civilizations
            .as_mut()
            .context("service order credit requires a history")?
            .credit
            .commercial_policy
            .service_orders = enabled;
    }
    if let Some(enabled) = args.service_order_procurement {
        generator
            .civilizations
            .as_mut()
            .context("service procurement requires a history")?
            .enterprises
            .as_mut()
            .context("service procurement requires enterprises")?
            .procurement
            .enabled = enabled;
    }
    if let Some(enabled) = args.contract_workshop_staffing {
        generator
            .civilizations
            .as_mut()
            .context("contract workshop staffing requires a history")?
            .enterprises
            .as_mut()
            .context("contract workshop staffing requires enterprises")?
            .procurement
            .contract_staffing = enabled;
    }
    if let Some(enabled) = args.demand_workshop_staffing {
        generator
            .civilizations
            .as_mut()
            .context("demand workshop staffing requires a history")?
            .enterprises
            .as_mut()
            .context("demand workshop staffing requires enterprises")?
            .procurement
            .demand_staffing = enabled;
    }
    for (setting, value) in [(0, args.needs_based_food), (1, args.gradual_nutrition)] {
        if let Some(enabled) = value {
            let h = generator
                .civilizations
                .as_mut()
                .context("food experiments require history")?;
            anyhow::ensure!(
                !(setting == 1 && enabled && h.individual_demography_enabled()),
                "gradual nutrition currently requires aggregate demography"
            );
            let e = h
                .society
                .as_mut()
                .and_then(|s| s.household_economy.as_mut())
                .context("food experiments require household accounts")?;
            if setting == 0 {
                e.needs_based_food = enabled;
            } else {
                e.gradual_nutrition = enabled;
            }
        }
    }
    if let Some(enabled) = args.council_welfare_reserves {
        generator
            .civilizations
            .as_mut()
            .context("welfare requires history")?
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
            .context("welfare requires household accounts")?
            .council_allocation = if enabled {
            ancient_world::household_economy::council_allocation::Policy::NeedsFirst
        } else {
            ancient_world::household_economy::council_allocation::Policy::Existing
        };
    }
    if let Some(enabled) = args.household_wealth_tax {
        generator
            .civilizations
            .as_mut()
            .context("wealth tax requires history")?
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
            .context("wealth tax requires household accounts")?
            .wealth_tax
            .enabled = enabled;
    }
    if let Some(enabled) = args.practical_research {
        generator
            .civilizations
            .as_mut()
            .context("research requires history")?
            .culture
            .as_mut()
            .context("research requires culture")?
            .practical_research = enabled;
    }
    if let Some(enabled) = args.household_clothing {
        generator
            .civilizations
            .as_mut()
            .context("clothing requires history")?
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
            .context("clothing requires household accounts")?
            .clothing_enabled = enabled;
    }
    if let Some(enabled) = args.household_estate_inheritance {
        generator
            .civilizations
            .as_mut()
            .context("estate inheritance requires a history")?
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
            .context("estate inheritance requires household accounts")?
            .inheritance
            .enabled = enabled;
    }
    if let Some(enabled) = args.named_office_service {
        generator
            .civilizations
            .as_mut()
            .context("office service requires a history")?
            .set_office_service(enabled)?;
    }
    if let Some(enabled) = args.abandoned_stock_recovery {
        generator
            .civilizations
            .as_mut()
            .context("stock recovery requires a history")?
            .society
            .as_mut()
            .context("stock recovery requires society")?
            .stock_recovery = enabled;
    }
    if let Some(enabled) = args.household_estate_reclamation {
        generator
            .civilizations
            .as_mut()
            .context("estate reclamation requires a history")?
            .society
            .as_mut()
            .and_then(|s| s.household_economy.as_mut())
            .context("estate reclamation requires household accounts")?
            .reclamation
            .enabled = enabled;
    }
    if let Some(share) = args.service_procurement_share {
        generator
            .civilizations
            .as_mut()
            .context("service procurement share requires a history")?
            .enterprises
            .as_mut()
            .context("service procurement share requires enterprises")?
            .procurement
            .surplus_share = share;
    }
    if let Some(enabled) = args.export_default_recovery {
        generator
            .civilizations
            .as_mut()
            .context("export default recovery requires a history")?
            .credit
            .export_recovery
            .policy
            .enabled = enabled;
    }
    if let Some(enabled) = args.delivery_paid_exports {
        generator
            .civilizations
            .as_mut()
            .context("delivery-paid exports require a history")?
            .export_payment_timing = if enabled {
            ancient_world::export_contracts::payments::Timing::Delivery
        } else {
            ancient_world::export_contracts::payments::Timing::Dispatch
        };
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
        let map = MapRenderer::new(&generator, EXPORT_ATLAS_WIDTH_PX, EXPORT_ATLAS_HEIGHT_PX)?;
        map.render(&generator, args.layer, Camera::atlas(), None);
        map.export_png(&generator, path)?;
    }
    println!(
        "{}",
        serde_json::json!({"epochs":generator.progress.epoch,"cells":generator.config.cells(),"seconds":start.elapsed().as_secs_f64(),"stage_ms":generator.progress.stage_ms,"gpu_timestamps":generator.progress.timestamp_supported,"lake_iterations":generator.progress.lake_iterations,"lake_changed_cells":generator.progress.lake_changed_cells,"lake_max_change_m":generator.progress.lake_max_change_m})
    );
    Ok(())
}

#[cfg(test)]
mod args_tests {
    use super::*;
    #[test]
    fn issuance_flag_preserves_archive_when_omitted() {
        for (arguments, expected) in [
            (vec!["ancient-world"], None),
            (vec!["ancient-world", "--shared-issuance"], Some(true)),
            (
                vec!["ancient-world", "--shared-issuance=false"],
                Some(false),
            ),
        ] {
            assert_eq!(
                Args::try_parse_from(arguments).unwrap().shared_issuance,
                expected
            );
        }
    }
    #[test]
    fn commercial_credit_flag_preserves_archive_when_omitted() {
        for (arguments, expected) in [
            (vec!["ancient-world"], None),
            (vec!["ancient-world", "--commercial-credit"], Some(true)),
            (
                vec!["ancient-world", "--commercial-credit=false"],
                Some(false),
            ),
        ] {
            assert_eq!(
                Args::try_parse_from(arguments).unwrap().commercial_credit,
                expected
            );
        }
    }
    #[test]
    fn service_procurement_flag_is_independent_and_preserves_archive() {
        for (args, expected) in [
            (vec!["ancient-world"], None),
            (
                vec!["ancient-world", "--service-order-procurement"],
                Some(true),
            ),
            (
                vec!["ancient-world", "--service-order-procurement=false"],
                Some(false),
            ),
        ] {
            let args = Args::try_parse_from(args).unwrap();
            assert_eq!(args.service_order_procurement, expected);
            assert_eq!(args.service_order_credit, None);
        }
    }

    #[test]
    fn estate_inheritance_override_preserves_omission() {
        for (arguments, expected) in [
            (vec!["ancient-world"], None),
            (
                vec!["ancient-world", "--household-estate-inheritance"],
                Some(true),
            ),
            (
                vec!["ancient-world", "--household-estate-inheritance=false"],
                Some(false),
            ),
        ] {
            let args = Args::try_parse_from(arguments).unwrap();
            assert_eq!(args.household_estate_inheritance, expected);
            assert_eq!(args.commercial_credit, None);
        }
    }

    #[test]
    fn demand_staffing_override_is_independent() {
        for (arguments, expected) in [
            (vec!["ancient-world"], None),
            (
                vec!["ancient-world", "--demand-workshop-staffing"],
                Some(true),
            ),
            (
                vec!["ancient-world", "--demand-workshop-staffing=false"],
                Some(false),
            ),
        ] {
            let args = Args::try_parse_from(arguments).unwrap();
            assert_eq!(args.demand_workshop_staffing, expected);
            assert_eq!(args.contract_workshop_staffing, None);
            assert_eq!(args.service_order_procurement, None);
        }
    }

    #[test]
    fn contract_staffing_override_preserves_omission_and_independent_policies() {
        for (arguments, expected) in [
            (vec!["ancient-world"], None),
            (
                vec!["ancient-world", "--contract-workshop-staffing"],
                Some(true),
            ),
            (
                vec!["ancient-world", "--contract-workshop-staffing=false"],
                Some(false),
            ),
        ] {
            let args = Args::try_parse_from(arguments).unwrap();
            assert_eq!(args.contract_workshop_staffing, expected);
            assert_eq!(args.service_order_procurement, None);
            assert_eq!(args.service_order_credit, None);
        }
    }

    #[test]
    fn service_procurement_share_is_bounded_and_does_not_enable_policy() {
        assert_eq!(
            Args::try_parse_from(["ancient-world"])
                .unwrap()
                .service_procurement_share,
            None
        );
        for (text, expected) in [("0", 0.0), ("0.1", 0.1), ("1", 1.0)] {
            let args = Args::try_parse_from(["ancient-world", "--service-procurement-share", text])
                .unwrap();
            assert_eq!(args.service_procurement_share, Some(expected));
            assert_eq!(args.service_order_procurement, None);
            assert_eq!(args.service_order_credit, None);
        }
        for text in ["NaN", "inf", "-0.1", "1.01", "abc"] {
            assert!(Args::try_parse_from([
                "ancient-world",
                &format!("--service-procurement-share={text}")
            ])
            .is_err());
        }
    }

    #[test]
    fn service_order_credit_flag_is_explicit_and_preserves_archive() {
        for (args, expected) in [
            (vec!["ancient-world"], None),
            (vec!["ancient-world", "--service-order-credit"], Some(true)),
            (
                vec!["ancient-world", "--service-order-credit=false"],
                Some(false),
            ),
        ] {
            assert_eq!(
                Args::try_parse_from(args).unwrap().service_order_credit,
                expected
            );
        }
    }

    #[test]
    fn export_recovery_flag_preserves_archive_when_omitted() {
        for (arguments, expected) in [
            (vec!["ancient-world"], None),
            (
                vec!["ancient-world", "--export-default-recovery"],
                Some(true),
            ),
            (
                vec!["ancient-world", "--export-default-recovery=false"],
                Some(false),
            ),
        ] {
            assert_eq!(
                Args::try_parse_from(arguments)
                    .unwrap()
                    .export_default_recovery,
                expected
            );
        }
    }
    #[test]
    fn delivery_payment_flag_preserves_archive_when_omitted() {
        for (arguments, expected) in [
            (vec!["ancient-world"], None),
            (vec!["ancient-world", "--delivery-paid-exports"], Some(true)),
            (
                vec!["ancient-world", "--delivery-paid-exports=false"],
                Some(false),
            ),
        ] {
            assert_eq!(
                Args::try_parse_from(arguments)
                    .unwrap()
                    .delivery_paid_exports,
                expected
            );
        }
    }
    #[test]
    fn council_credit_can_be_enabled_disabled_or_left_as_archived() {
        assert_eq!(
            Args::try_parse_from(["ancient-world"])
                .unwrap()
                .council_credit,
            None
        );
        assert_eq!(
            Args::try_parse_from(["ancient-world", "--council-credit"])
                .unwrap()
                .council_credit,
            Some(true)
        );
        assert_eq!(
            Args::try_parse_from(["ancient-world", "--council-credit=false"])
                .unwrap()
                .council_credit,
            Some(false)
        );
    }
    #[test]
    fn institution_lender_switch_preserves_unspecified_archived_policy() {
        assert_eq!(
            Args::try_parse_from(["ancient-world"])
                .unwrap()
                .institution_credit_lenders,
            None
        );
        assert_eq!(
            Args::try_parse_from(["ancient-world", "--institution-credit-lenders"])
                .unwrap()
                .institution_credit_lenders,
            Some(true)
        );
        assert_eq!(
            Args::try_parse_from(["ancient-world", "--institution-credit-lenders=false"])
                .unwrap()
                .institution_credit_lenders,
            Some(false)
        );
    }
    #[test]
    fn institutional_operating_reserve_can_be_selected_or_preserved() {
        assert_eq!(
            Args::try_parse_from(["ancient-world"])
                .unwrap()
                .institution_credit_operating_reserve,
            None
        );
        assert_eq!(
            Args::try_parse_from(["ancient-world", "--institution-credit-operating-reserve"])
                .unwrap()
                .institution_credit_operating_reserve,
            Some(true)
        );
        assert_eq!(
            Args::try_parse_from([
                "ancient-world",
                "--institution-credit-operating-reserve=false"
            ])
            .unwrap()
            .institution_credit_operating_reserve,
            Some(false)
        );
    }
    #[test]
    fn system_flags_are_explicit_and_validate_names() {
        let args = Args::try_parse_from([
            "ancient-world",
            "--disable-system",
            "society,living-world",
            "--enable-system",
            "adaptive-prices",
        ])
        .unwrap();
        assert_eq!(
            args.disable_system,
            vec![System::Society, System::LivingWorld]
        );
        assert_eq!(args.enable_system, vec![System::AdaptivePrices]);
        assert!(Args::try_parse_from(["ancient-world", "--disable-system", "typo"]).is_err());
        assert!(
            Args::try_parse_from(["ancient-world", "--society"])
                .unwrap()
                .society
        );
    }
}
