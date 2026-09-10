//! Paired-seed history experiments; emits machine-readable data and a concise Markdown report.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::{ensure, Context, Result};
use clap::Parser;
use serde_json::json;
use std::{collections::BTreeMap, io::Write, path::PathBuf, time::Instant};
#[derive(Parser)]
struct Args {
    #[arg(long, value_delimiter = ',', default_value = "0,7,42,99,999")]
    seeds: Vec<u32>,
    #[arg(long, default_value_t = 64)]
    resolution: u32,
    #[arg(long, default_value_t = 1)]
    epochs: u32,
    #[arg(long, default_value_t = 100)]
    years: u32,
    #[arg(long, default_value_t = 16)]
    civilizations: u32,
    #[arg(long)]
    island_phosphorus_scale: Option<f32>,
    #[arg(long)]
    settlement_plot_hectares: Option<f32>,
    #[arg(long)]
    crop_yield_scale: Option<f32>,
    #[arg(long, default_value = "output/history-evaluation")]
    output: PathBuf,
    #[arg(long, default_value = "current")]
    label: String,
    /// Run the archived original market rules as a paired control.
    #[arg(long)]
    legacy_markets: bool,
    #[arg(long)]
    legacy_production: bool,
    /// Retain demand orders but use the previous fixed workforce shares.
    #[arg(long)]
    fixed_labor: bool,
    /// Disable industrial capital while retaining demand orders and staffing.
    #[arg(long)]
    no_workshops: bool,
    /// Paired control with population-scaled dry storage.
    #[arg(long)]
    no_persistent_storage: bool,
    #[arg(long)]
    no_persistent_housing: bool,
    #[arg(long)]
    no_waterworks: bool,
    /// Prioritize inherited water service after urgent shelter.
    #[arg(long)]
    waterworks_repair_priority: bool,
    /// Simultaneous planned land cargo capacity in kg per resident.
    #[arg(long)]
    land_freight_kg_per_person: Option<f32>,
    /// Planned coverage; zero is an unbuilt control retaining domestic demand and health exposure.
    #[arg(long)]
    waterworks_target_fraction: Option<f32>,
    #[arg(long)]
    initial_housing_per_person: Option<f32>,
    /// Initial dry-yard allowance; also bounds expansion planning relative to local workforce.
    #[arg(long)]
    storage_kg_per_person: Option<f32>,
    #[arg(long)]
    no_export_contracts: bool,
    #[arg(long)]
    no_supplier_profitability: bool,
    #[arg(long)]
    no_specialized_workshops: bool,
    #[arg(long)]
    no_patron_aid: bool,
    #[arg(long)]
    no_relocation: bool,
    /// Paired control without social pressure memory or its decision modifiers.
    #[arg(long)]
    no_social_indicators: bool,
    /// Control retaining scenario-only autonomy rather than council crisis responses.
    #[arg(long)]
    no_negotiated_autonomy: bool,
    /// Establish persistent local offices with finite terms and tax-collection effects.
    #[arg(long)]
    offices: bool,
    #[arg(long)]
    occupation_months: Option<u32>,
    #[arg(long)]
    no_household_economy: bool,
    /// Matched control retaining communal workshop operation.
    #[arg(long)]
    no_enterprises: bool,
    /// Matched control retaining equal household wages.
    #[arg(long)]
    equal_payroll: bool,
    #[arg(long)]
    no_targeted_relief: bool,
    /// Prioritize children and elders during shortages; retains a common ration floor.
    #[arg(long)]
    protect_vulnerable_rations: bool,
    #[arg(long)]
    legacy_relief: bool,
    #[arg(long)]
    legacy_farming: bool,
    /// Enable finite inter-island commercial shipping.
    #[arg(long)]
    shipping: bool,
    /// Enable shipping plus chartered outer-continent voyages.
    #[arg(long)]
    expeditions: bool,
    /// Include finite specimens and workshop applications (implies expeditions).
    #[arg(long)]
    discoveries: bool,
    #[arg(long)]
    living_world: bool,
    #[arg(long, requires = "living_world")]
    storm_probability: Option<f32>,
    /// Controlled expedition hazard override, 0–5.
    #[arg(long)]
    expedition_hazard: Option<f32>,
    /// Disable multi-year droughts for an original-simulation control.
    #[arg(long)]
    legacy_weather: bool,
    /// Override the bundled drought severity for a calibration or stress experiment (0–1).
    #[arg(long, conflicts_with = "legacy_weather")]
    drought_severity: Option<f32>,
    #[arg(long, conflicts_with = "legacy_weather")]
    drought_probability: Option<f32>,
    #[arg(long, conflicts_with = "legacy_weather")]
    drought_regime_months: Option<u32>,
    /// Compare against a report with matching geography, seeds and history length.
    #[arg(long)]
    compare: Option<PathBuf>,
    /// Save each final world alongside the report for inspection in the desktop explorer.
    #[arg(long)]
    save_worlds: bool,
}
fn main() -> Result<()> {
    let args = Args::parse();
    let expedition_enabled = args.expeditions || args.discoveries;
    ensure!(
        args.expedition_hazard.is_none() || expedition_enabled,
        "--expedition-hazard requires --expeditions or --discoveries"
    );
    ensure!(
        !args.seeds.is_empty() && args.years > 0 && args.years <= 10000,
        "invalid experiment interval"
    );
    let comparison: Option<serde_json::Value> = args
        .compare
        .as_ref()
        .map(|p| -> Result<_> { Ok(serde_json::from_slice(&std::fs::read(p)?)?) })
        .transpose()?;
    if let Some(c) = &comparison {
        ensure!(
            c["resolution"] == args.resolution
                && c["epochs"] == args.epochs
                && c["years"] == args.years
                && c["requested_civilizations"] == args.civilizations
                && c["seeds"] == json!(args.seeds)
                && c["runs"]
                    .as_array()
                    .is_some_and(|r| r.len() == args.seeds.len()),
            "comparison experiment configuration mismatch or incomplete report"
        );
    }
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut runs = vec![];
    let mut markdown = format!("# History evaluation: {}\n\nGPU: {}. Terrain/ecology {}. {} geological epochs, {} social years, {} requested civilizations.\n\nCross-border deliveries use administrators at annual observation boundaries; changes within a sample can affect attribution. Shortages and recovery are annual observations, not all monthly incidents. Household wealth inequality is not yet modeled independently. No outcome quotas are imposed.\n\n| Seed | Population | Active sites | Deliveries | Cross-border | Sea deliveries | Ports ready | Wars | Treaties | Shortage site-years | Recoveries | Max residual | Seconds |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n", args.label, gpu.adapter_name, args.resolution, args.epochs, args.years, args.civilizations);
    for &seed in &args.seeds {
        let start = Instant::now();
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                resolution: args.resolution,
                ecology_resolution: args.resolution,
                seed,
                crop_yield_scale: args
                    .crop_yield_scale
                    .unwrap_or(Config::default().crop_yield_scale),
                island_phosphorus_scale: args
                    .island_phosphorus_scale
                    .unwrap_or(Config::default().island_phosphorus_scale),
                settlement_plot_hectares: args
                    .settlement_plot_hectares
                    .unwrap_or(Config::default().settlement_plot_hectares),
                ..Default::default()
            },
            Catalog::bundled()?,
        )?;
        g.run_epochs(args.epochs)?;
        let climate = json!({"converged":g.progress.climate_converged,"cycles":g.progress.climate_cycles,"limit":g.config.climate_cycles});
        eprintln!("seed {seed}: initial climate {climate}");
        g.found_civilizations_with_options(
            args.civilizations,
            ancient_world::culture::FoundingOptions {
                aid_enabled: !args.no_patron_aid,
                ..Default::default()
            },
        )?;
        if args.legacy_farming {
            g.set_diversified_farming(false)?;
        }
        if args.legacy_production
            || args.fixed_labor
            || args.no_workshops
            || args.no_persistent_storage
            || args.no_persistent_housing
            || args.no_waterworks
            || args.waterworks_repair_priority
            || args.land_freight_kg_per_person.is_some()
            || args.waterworks_target_fraction.is_some()
            || args.initial_housing_per_person.is_some()
            || args.storage_kg_per_person.is_some()
            || args.no_export_contracts
            || args.no_supplier_profitability
            || args.no_specialized_workshops
            || args.legacy_markets
            || args.legacy_weather
            || args.drought_severity.is_some()
            || args.drought_probability.is_some()
            || args.drought_regime_months.is_some()
            || args.storm_probability.is_some()
        {
            let mut catalog = g
                .civilizations
                .as_ref()
                .unwrap()
                .economy_catalog
                .clone()
                .unwrap();
            if args.waterworks_repair_priority {
                catalog.production.waterworks_repair_priority = true;
            }
            if let Some(capacity) = args.land_freight_kg_per_person {
                catalog.production.land_freight_kg_per_person = capacity;
            }
            if args.no_specialized_workshops {
                catalog.production.specialized_workshops = false;
            }
            if args.no_supplier_profitability {
                catalog.production.supplier_profitability = false;
            }
            if args.no_export_contracts {
                catalog.production.export_contracts = false;
            }
            if let Some(capacity) = args.storage_kg_per_person {
                catalog.production.storage_kg_per_person = capacity;
            }
            if let Some(fraction) = args.waterworks_target_fraction {
                catalog.production.waterworks_target_fraction = fraction;
            }
            if args.no_waterworks {
                catalog.production.waterworks = false;
            }
            if args.no_persistent_housing {
                catalog.production.persistent_housing = false;
            }
            if let Some(allowance) = args.initial_housing_per_person {
                catalog.production.initial_housing_per_person = allowance;
            }
            if args.no_persistent_storage {
                catalog.production.persistent_storage = false;
            }
            if args.no_workshops {
                catalog.production.workshops = false;
            }
            if args.fixed_labor {
                catalog.production.adaptive_labor = false;
            }
            if args.legacy_production {
                catalog.production.enabled = false;
            }
            if args.legacy_markets {
                catalog.market = Default::default();
            }
            if args.legacy_weather {
                catalog.weather = Default::default();
            }
            if let Some(probability) = args.storm_probability {
                catalog.weather.storm_probability = probability;
            }
            if let Some(severity) = args.drought_severity {
                catalog.weather.drought_severity = severity;
            }
            if let Some(v) = args.drought_probability {
                catalog.weather.drought_probability = v;
            }
            if let Some(v) = args.drought_regime_months {
                catalog.weather.regime_months = v;
            }
            g.configure_economy(catalog)?;
        }
        g.enable_society()?;
        if args.no_enterprises {
            g.set_enterprises(false)?;
        }
        if args.equal_payroll {
            g.set_occupational_payroll(false)?;
        }
        if args.no_targeted_relief {
            g.configure_household_relief(0., 0.75)?;
        }
        if args.no_household_economy {
            g.civilizations
                .as_mut()
                .unwrap()
                .society
                .as_mut()
                .unwrap()
                .household_economy = None;
        }
        if args.protect_vulnerable_rations {
            for site in 0..g.civilizations.as_ref().unwrap().sites.len() {
                g.set_ration_priority(site as u32, [3., 0., 3.])?;
            }
        }
        if args.no_social_indicators {
            g.civilizations
                .as_mut()
                .unwrap()
                .society
                .as_mut()
                .unwrap()
                .indicators = None;
        }
        g.civilizations
            .as_mut()
            .unwrap()
            .set_household_relocation(!args.no_relocation)?;
        g.civilizations
            .as_mut()
            .unwrap()
            .set_witnessed_relief(!args.legacy_relief)?;
        g.enable_politics()?;
        if let Some(months) = args.occupation_months {
            g.set_occupation_months(months)?;
        }
        g.enable_governance()?;
        if args.offices {
            g.enable_offices()?;
        }
        if args.no_negotiated_autonomy {
            g.set_negotiated_autonomy(false)?;
        }
        if args.shipping || expedition_enabled {
            g.enable_shipping()?;
        }
        if expedition_enabled {
            g.enable_expeditions()?;
            if let Some(hazard) = args.expedition_hazard {
                let rules = ancient_world::expeditions::Rules {
                    hazard_scale: hazard,
                    ..Default::default()
                };
                g.configure_expeditions(rules)?;
            }
        }
        if args.discoveries {
            g.enable_discoveries()?;
        }
        if args.living_world {
            g.enable_living_history()?;
        }
        let mut terrain = g.snapshot()?;
        let mut counts = BTreeMap::<String, u64>::new();
        let mut cursor = g.civilizations.as_ref().unwrap().events.len();
        let mut cross = 0u64;
        let mut shortage_years = 0usize;
        let mut recoveries = 0usize;
        let mut previous_shortages = vec![];
        let mut previous_controllers: Vec<_> = g
            .civilizations
            .as_ref()
            .unwrap()
            .sites
            .iter()
            .map(|s| g.civilizations.as_ref().unwrap().controller(s.id))
            .collect();
        let mut territory_changes = 0;
        let mut max_residual = 0f64;
        let initial_sites: Vec<_> = g
            .civilizations
            .as_ref()
            .unwrap()
            .sites
            .iter()
            .map(|s| json!({"id":s.id,"population":s.stocks.stock[0]}))
            .collect();
        let mut samples = vec![];
        if let Some(parent) = args.output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut journal =
            std::fs::File::create(args.output.with_extension(format!("{seed}.jsonl")))?;
        for year in 0..args.years {
            g.advance_history(12).with_context(|| {
                format!(
                    "seed {seed}, month {}",
                    g.civilizations.as_ref().unwrap().month
                )
            })?;
            let h = g.civilizations.as_ref().unwrap();
            if (year + 1) % 50 == 0 {
                eprintln!(
                    "seed {seed}: year {}, {:.0} people, {} events",
                    year + 1,
                    h.sites
                        .iter()
                        .map(|s| s.stocks.stock[0] as f64)
                        .sum::<f64>(),
                    h.events.len()
                );
            }
            if args.living_world {
                terrain = g.snapshot()?;
            }
            h.validate(&terrain)?;
            if args.living_world {
                anyhow::ensure!(
                    h.candidates
                        .iter()
                        .all(
                            |c| ancient_world::hazards::flood_depth(&terrain[c.cell as usize])
                                < 0.25
                        ),
                    "unsafe settlement candidate retained"
                );
                anyhow::ensure!(
                    g.ecology.budget(&g.gpu, &g.config)?.within_tolerance,
                    "living environment budget failed"
                );
            }
            for e in &h.events[cursor..] {
                *counts.entry(e.kind.clone()).or_default() += 1;
                if e.kind == "market_arrival"
                    && e.site
                        .zip(e.other)
                        .is_some_and(|(a, b)| h.controller(a) != h.controller(b))
                {
                    cross += 1;
                }
            }
            cursor = h.events.len();
            let shortage: Vec<_> = h
                .sites
                .iter()
                .map(|s| !s.abandoned && s.stocks.stock[3] > 0.01)
                .collect();
            shortage_years += shortage.iter().filter(|&&v| v).count();
            recoveries += previous_shortages
                .iter()
                .zip(&shortage)
                .zip(&h.sites)
                .filter(|((a, b), site)| **a && !**b && !site.abandoned)
                .count();
            previous_shortages = shortage;
            let controllers: Vec<_> = h.sites.iter().map(|s| h.controller(s.id)).collect();
            territory_changes += previous_controllers
                .iter()
                .zip(&controllers)
                .filter(|(a, b)| a != b)
                .count();
            previous_controllers = controllers;
            let mut residuals = h.economy_residuals().to_vec();
            residuals.extend([h.food_residual(), h.population_residual()]);
            ensure!(
                residuals.iter().all(|v| v.is_finite() && v.abs() < 0.001),
                "seed {seed} month {} conservation failure: {residuals:?}",
                h.month
            );
            max_residual = residuals.iter().fold(max_residual, |m, v| m.max(v.abs()));
            let population: f64 = h.sites.iter().map(|s| s.stocks.stock[0] as f64).sum();
            let active = h.sites.iter().filter(|s| !s.abandoned).count();
            let unrest: f32 = h
                .governance
                .as_ref()
                .unwrap()
                .administrations
                .iter()
                .map(|a| a.unrest)
                .sum::<f32>()
                / h.sites.len() as f32;
            let tools_per_person = h
                .sites
                .iter()
                .map(|s| s.economy.goods[3] as f64)
                .sum::<f64>()
                / population.max(1.);
            let tool_sufficiency = h
                .sites
                .iter()
                .map(|s| {
                    (s.economy.goods[3] as f64 / (s.stocks.stock[0] as f64 * 0.5).max(1.)).min(1.)
                        * s.stocks.stock[0] as f64
                })
                .sum::<f64>()
                / population.max(1.);
            let network = h.trade_distances().unwrap();
            let seas = h.sea_quotes(&network);
            let n = h.sites.len();
            let connected_foreign_pairs = (0..n)
                .flat_map(|a| (a + 1..n).map(move |b| (a, b)))
                .filter(|&(a, b)| {
                    h.controller(a as u32) != h.controller(b as u32)
                        && (network[a * n + b].is_finite() || seas[a * n + b].is_some())
                })
                .count();
            samples.push(json!({"roads":h.society.as_ref().map(|s|json!({"material_kg":s.routes.iter().map(|r|r.road_bricks).sum::<f64>(),"weathered_kg":s.routes.iter().filter_map(|r|r.upkeep.as_ref()).map(|c|c.lost_kg).sum::<f64>(),"work":s.routes.iter().filter_map(|r|r.upkeep.as_ref()).map(|c|c.work).sum::<f64>(),"impaired":s.routes.iter().filter_map(|r|r.upkeep.as_ref()).filter(|c|c.impaired).count()})),"enterprises":h.enterprise_summary(),"integration":integration_sample(h),"month":h.month,"relocation":h.household_relocations().map(|r|json!({"enabled":r.enabled,"journeys":r.journeys.len(),"travelers":r.journeys.iter().map(|j|j.population()).sum::<f32>(),"food_in_transit":r.journeys.iter().map(|j|j.food).sum::<f32>()})),"settlements":h.sites.iter().map(|s|json!({"id":s.id,"name":s.name,"population":s.stocks.stock[0],"abandoned":s.abandoned,"settlement_size":s.lifecycle.size.label(),"shortage":s.stocks.stock[3],"ration_need":s.demography.ration_need,"ration_eaten":s.demography.ration_eaten,"consecutive_shortage_months":s.demography.health[1],"food_months":s.stocks.stock[1]/(s.stocks.stock[0]*18.).max(1.),"deaths":s.stocks.people[1],"disease":s.demography.health[0],"waterlogging":s.economy.soil[3]})).collect::<Vec<_>>(),"production":h.production_summary(),"floods":h.living.as_ref().map(|l|json!({"site_months":l.floods.values().map(|f|f.flooded_months).sum::<u32>(),"food_lost_kg":l.floods.values().map(|f|f.food_lost_kg).sum::<f64>(),"crops_lost_kg":l.floods.values().map(|f|f.crops_lost_kg).sum::<f64>(),"active_sites":l.floods.values().filter(|f|f.flooded).count(),"persistent_sites":l.floods.values().filter(|f|f.persistent).count(),"waiting_cargo":h.cargo.iter().filter(|c|c.weather_delay_months>0).count(),"max_cargo_delay_months":h.cargo.iter().map(|c|c.weather_delay_months).max().unwrap_or(0),"closed_roads":h.society.as_ref().map_or(0,|s|s.routes.iter().filter(|r|r.flood_months>0).count()),"closed_ports":h.shipping.as_ref().map_or(0,|s|s.ports.iter().filter(|p|p.flood_months>0).count())})),"ecology_month":g.ecology.clock.month,"ecology_budget":if args.living_world {Some(g.ecology.budget(&g.gpu, &g.config)?)} else {None},"discoveries":h.expeditions.as_ref().and_then(|x|x.discoveries.as_ref().map(|d|json!({"collected":d.collected,"studied":d.studied,"processed":d.processed,"remedy_made":d.remedy_made,"remedy_used":d.remedy_used,"remedy_expired":d.remedy_expired,"phosphorus_applied":d.phosphorus_applied,"worker_months":d.worker_months,"worker_months_reserved":d.worker_months_reserved,"residuals":d.residuals(x),"depleted_sources":d.sources.iter().filter(|s|s.remaining.iter().all(|v|*v<=0.)).count()}))),"expeditions":h.expeditions.as_ref().map(|x|json!({"launched":x.voyages.len(),"active":x.voyages.iter().filter(|e|e.phase.active()).count(),"crew_away":x.voyages.iter().filter(|e|e.phase.active()).map(|e|e.survivors()).sum::<usize>(),"knowledge":x.knowledge,"returned":x.voyages.iter().filter(|e|e.phase==ancient_world::expeditions::Phase::Returned).count(),"lost":x.voyages.iter().filter(|e|e.phase==ancient_world::expeditions::Phase::Lost).count()})),"ports":h.shipping.as_ref().map_or(0,|s|s.ports.len()),"operational_ports":h.shipping.as_ref().map_or(0,|s|s.ports.iter().filter(|p|p.capacity()>=1.).count()),"population":population,"active_sites":active,"mean_unrest":unrest,"tools_kg_per_person":tools_per_person,"population_weighted_tool_sufficiency":tool_sufficiency,"connected_foreign_site_pairs":connected_foreign_pairs,"drought_sites":h.sites.iter().filter(|s| !s.abandoned && s.demography.ages[3]<0.999).count(),"cumulative_events":counts,"cross_border_deliveries":cross,"territory_changes":territory_changes,"residuals_cnp_water_money_goods_food_population":residuals}));
            serde_json::to_writer(&mut journal, samples.last().unwrap())?;
            writeln!(journal)?;
            journal.flush()?;
        }
        if args.save_worlds {
            if let Some(p) = args.output.parent() {
                std::fs::create_dir_all(p)?;
            }
            g.save(args.output.with_extension(format!("{seed}.world")))?;
        }
        let last = samples.last().unwrap();
        let count = |s: &str| counts.get(s).copied().unwrap_or(0);
        let seconds = start.elapsed().as_secs_f64();
        markdown.push_str(&format!("| {seed} | {:.0} | {} | {} | {cross} | {} | {} | {} | {} | {shortage_years} | {recoveries} | {max_residual:.2e} | {seconds:.2} |\n", last["population"].as_f64().unwrap(),last["active_sites"],count("market_arrival"),count("sea_arrival"),last["operational_ports"],count("war_declared"),count("treaty_signed")));
        eprintln!("seed {seed}: {} deliveries, {cross} cross-border, {} wars, {} treaties ({seconds:.1}s)",count("market_arrival"),count("war_declared"),count("treaty_signed"));
        if let Some(c) = &comparison {
            let baseline = c["runs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["seed"] == seed)
                .ok_or_else(|| anyhow::anyhow!("missing baseline seed"))?;
            let last_before = baseline["samples"]
                .as_array()
                .and_then(|s| s.last())
                .ok_or_else(|| anyhow::anyhow!("missing baseline samples"))?;
            eprintln!(
                "paired seed {seed}: population delta {:.1}, cross-border delivery delta {}",
                last["population"].as_f64().unwrap() - last_before["population"].as_f64().unwrap(),
                cross as i64 - last_before["cross_border_deliveries"].as_i64().unwrap()
            );
        }
        runs.push(json!({"harbors":g.civilizations.as_ref().unwrap().shipping.as_ref().map(|s|&s.ports),"roads":g.civilizations.as_ref().unwrap().society.as_ref().map(|s|&s.routes),"enterprises":g.civilizations.as_ref().unwrap().enterprises,"household_economy":g.civilizations.as_ref().unwrap().society.as_ref().and_then(|s|s.household_economy.as_ref()),"social_state":g.civilizations.as_ref().unwrap().society.as_ref().and_then(|s|s.indicators.as_ref()),"seed":seed,"initial_sites":initial_sites,"abandonments":g.civilizations.as_ref().unwrap().events.iter().filter(|e| e.kind=="abandoned").collect::<Vec<_>>(),"timings_ms":g.progress.stage_ms,"culture":g.civilizations.as_ref().unwrap().cultural_summary(),"agriculture":g.civilizations.as_ref().unwrap().sites.iter().map(|s|json!({"site":s.id,"crops":s.economy.crops,"herds":s.economy.herds,"totals":s.economy.agriculture,"roles":g.civilizations.as_ref().unwrap().settlement_roles(s.id)})).collect::<Vec<_>>(),"initial_climate":climate,"seconds":seconds,"samples":samples,"events":counts,"shortage_site_years":shortage_years,"recoveries":recoveries,"territory_changes":territory_changes,"max_relative_residual":max_residual}));
        if let Some(p) = args.output.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(
            args.output.with_extension("json"),
            serde_json::to_vec_pretty(
                &json!({"version":1,"label":args.label,"enterprises":!args.no_enterprises&&!args.no_household_economy,"occupational_payroll":!args.equal_payroll,"household_economy":!args.no_household_economy,"protect_vulnerable_rations":args.protect_vulnerable_rations,"negotiated_autonomy":!args.no_negotiated_autonomy,"offices":args.offices,"occupation_months":g.civilizations.as_ref().unwrap().politics.as_ref().unwrap().occupation_months,"social_indicators":!args.no_social_indicators,"witnessed_relief":!args.legacy_relief,"gpu":gpu.adapter_name,"resolution":args.resolution,"epochs":args.epochs,"crop_yield_scale":g.config.crop_yield_scale,"island_phosphorus_scale":g.config.island_phosphorus_scale,"settlement_plot_hectares":g.config.settlement_plot_hectares,"years":args.years,"requested_civilizations":args.civilizations,"shipping":args.shipping||expedition_enabled,"discoveries":args.discoveries,"living_world":args.living_world,"expeditions":expedition_enabled,"expedition_hazard":args.expedition_hazard,"expedition_rules":g.civilizations.as_ref().unwrap().expeditions.as_ref().map(|x|&x.rules),"production_rules":g.civilizations.as_ref().unwrap().economy_catalog.as_ref().unwrap().production,"legacy_markets":args.legacy_markets,"legacy_weather":args.legacy_weather,"weather_rules":g.civilizations.as_ref().unwrap().economy_catalog.as_ref().unwrap().weather,"market_rules":g.civilizations.as_ref().unwrap().economy_catalog.as_ref().unwrap().market,"complete":runs.len()==args.seeds.len(),"seeds":args.seeds,"runs":runs}),
            )?,
        )?;
        std::fs::write(args.output.with_extension("md"), &markdown)?;
    }
    if args.living_world {
        markdown.push_str("\n## Floods and recovery\n\nFlood counts and site-months cover the entire run; losses are cumulative kg. Recovery counts can trail floods when another flood interrupts cleanup or a town is still recovering.\n\n| Seed | Flood episodes | Recovered | Flooded site-months | Stored food lost kg | Standing crops lost kg | Road closures | Port closures | Cargo delays | Cargo recovered |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
        for r in &runs {
            let f = &r["samples"].as_array().unwrap().last().unwrap()["floods"];
            let count = |key: &str| r["events"][key].as_u64().unwrap_or(0);
            markdown.push_str(&format!(
                "| {} | {} | {} | {} | {:.1} | {:.1} | {} | {} | {} | {} |\n",
                r["seed"],
                count("flood"),
                count("flood_recovery"),
                f["site_months"],
                f["food_lost_kg"].as_f64().unwrap(),
                f["crops_lost_kg"].as_f64().unwrap(),
                count("road_flood_closed"),
                count("port_weather_closed"),
                count("cargo_weather_delay"),
                count("cargo_weather_recovered")
            ));
        }
        std::fs::write(args.output.with_extension("md"), &markdown)?;
    }
    if expedition_enabled {
        markdown.push_str("\n## Expeditions\n\nResidents exclude crew away. Returned counts include rescue vessels; rescued parties are listed separately and do not duplicate survivors. Findings count as knowledge only after arrival home.\n\n| Seed | Charters | Returned | Rescued parties | Lost parties | Crew deaths | Crew away | Confirmed knowledge |\n|---|---:|---:|---:|---:|---:|---:|---:|\n");
        for r in &runs {
            let last = r["samples"].as_array().unwrap().last().unwrap();
            let e = &last["expeditions"];
            let count = |key: &str| r["events"][key].as_u64().unwrap_or(0);
            let knowledge: f64 = e["knowledge"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .sum();
            markdown.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {:.1} |\n",
                r["seed"],
                e["launched"],
                e["returned"],
                count("expedition_rescue"),
                e["lost"],
                count("expedition_casualty"),
                e["crew_away"],
                knowledge
            ));
        }
        std::fs::write(args.output.with_extension("md"), &markdown)?;
    }
    if args.discoveries {
        markdown.push_str("\n## Specimen applications\n\nAll quantities are kg except labor (worker-months). Research and processing consume actual specimens; accessible specimen stocks have no replenishment in this version, including during living history.\n\n| Seed | Resin collected | Mineral collected | Remedy made | Remedy used | P applied | Research/craft labor | Max specimen residual |\n|---|---:|---:|---:|---:|---:|---:|---:|\n");
        for r in &runs {
            let samples = r["samples"].as_array().unwrap();
            let d = &samples.last().unwrap()["discoveries"];
            let residual = samples
                .iter()
                .flat_map(|v| v["discoveries"]["residuals"].as_array().unwrap())
                .map(|v| v.as_f64().unwrap().abs())
                .fold(0f64, f64::max);
            markdown.push_str(&format!(
                "| {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.2} | {:.1} | {:.2e} |\n",
                r["seed"],
                d["collected"][0].as_f64().unwrap(),
                d["collected"][1].as_f64().unwrap(),
                d["remedy_made"].as_f64().unwrap(),
                d["remedy_used"].as_f64().unwrap(),
                d["phosphorus_applied"].as_f64().unwrap(),
                d["worker_months"].as_f64().unwrap(),
                residual
            ));
        }
        std::fs::write(args.output.with_extension("md"), &markdown)?;
    }
    if let Some(c) = comparison {
        markdown.push_str("\n## Paired changes\n\n| Seed | Population delta | Cross-border delivery delta | War delta | Treaty delta |\n|---|---:|---:|---:|---:|\n");
        for r in &runs {
            let b = c["runs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|b| b["seed"] == r["seed"])
                .unwrap();
            let last = |v: &serde_json::Value, key: &str| {
                v["samples"].as_array().unwrap().last().unwrap()[key]
                    .as_f64()
                    .unwrap()
            };
            let event = |v: &serde_json::Value, key: &str| v["events"][key].as_i64().unwrap_or(0);
            markdown.push_str(&format!(
                "| {} | {:.1} | {:.0} | {} | {} |\n",
                r["seed"],
                last(r, "population") - last(b, "population"),
                last(r, "cross_border_deliveries") - last(b, "cross_border_deliveries"),
                event(r, "war_declared") - event(b, "war_declared"),
                event(r, "treaty_signed") - event(b, "treaty_signed")
            ));
        }
        std::fs::write(args.output.with_extension("md"), markdown)?;
    }
    Ok(())
}

/// Annual observations of immediate mediators; no claim about unseen monthly maxima.
fn integration_sample(h: &ancient_world::civilization::History) -> serde_json::Value {
    json!({
        "cargo_kg": h.cargo.iter().map(|c| c.kg as f64).sum::<f64>(),
        "sites": h.sites.iter().filter(|s| !s.abandoned).map(|s| {
            let holders = h.culture.as_ref().map(|c| c.knowledge_holders(h, s.id));
            let a = h.governance.as_ref().and_then(|g| g.administrations.get(s.id as usize));
            let free = h.land_freight_capacity(s.id);
            json!({"site":s.id,"free_land_freight_kg":if free.is_finite(){Some(free)}else{None},
                "planned_land_freight":s.economy.logistics[3]>0.5,
                "knowledge_holders":holders,
                "office_capacity":h.office_capacity(s.id),"occupation_strength":h.occupation_strength(s.id),
                "office_holder":h.offices.as_ref().and_then(|o|o.seats.get(s.id as usize)).and_then(|o|o.holder()),
                "loyalty":a.map(|a|a.loyalty),"unrest":a.map(|a|a.unrest),
                "unpaid_months":a.map(|a|a.unpaid_months),"crisis_months":a.map(|a|a.crisis_months),
                "social":h.social_indicators(s.id),
                "water_coverage":s.economy.waterworks_plan[3],
                "water_priority_work":s.economy.waterworks_recovery[3],
                "crowding":s.economy.crowding(s.stocks.stock[0])})
        }).collect::<Vec<_>>()
    })
}
