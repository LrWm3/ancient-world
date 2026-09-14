//! Matched growth experiments with durable annual evidence and declining-growth gates.
use ancient_world::{
    catalog::Catalog,
    civilization::growth::Growth,
    config::Config,
    gpu::{ContextGpu, Generator},
    resolution::Mode as Demography,
    systems::{System, Systems},
};
use anyhow::{ensure, Context, Result};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

const HEALTH_BACKGROUND_SCALE: f64 = 0.85;
const YIELD_MULTIPLIER: f32 = 1.15;
const EXPANSION_POPULATION_SCALE: f32 = 0.85;
const EXPANSION_RESERVE_MONTHS: f32 = 9.;
const MONTHS_PER_YEAR: u32 = 12;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum Mode {
    Baseline,
    Health,
    Access,
    Yield,
    Expansion,
    Combined,
}
impl Mode {
    fn label(self) -> String {
        serde_json::to_value(self).unwrap().as_str().unwrap().into()
    }
    fn policy(self, limit: usize) -> Growth {
        Growth {
            background_mortality_scale: if matches!(self, Self::Health | Self::Combined) {
                HEALTH_BACKGROUND_SCALE
            } else {
                1.
            },
            founding_population_scale: if matches!(self, Self::Expansion | Self::Combined) {
                EXPANSION_POPULATION_SCALE
            } else {
                1.
            },
            founding_reserve_months: if matches!(self, Self::Expansion | Self::Combined) {
                EXPANSION_RESERVE_MONTHS
            } else {
                12.
            },
            settlement_limit: limit,
            audit_enabled: true,
            ..Default::default()
        }
    }
    fn apply(self, g: &mut Generator, limit: usize) -> Result<()> {
        g.civilizations.as_mut().unwrap().growth = self.policy(limit);
        if matches!(self, Self::Yield | Self::Combined) {
            g.config.crop_yield_scale *= YIELD_MULTIPLIER;
        }
        if matches!(self, Self::Access | Self::Combined) {
            g.apply_registered_policies(&Systems {
                overrides: [
                    (System::MunicipalFoodRelief, true),
                    (System::MunicipalWelfareReserves, true),
                ]
                .into(),
            })?;
        }
        g.config.validate()?;
        g.civilizations.as_ref().unwrap().growth.validate()
    }
}
#[derive(Parser, Serialize)]
struct Args {
    #[arg(long, value_delimiter = ',', default_value = "17,81,256")]
    seeds: Vec<u32>,
    #[arg(
        long,
        value_enum,
        value_delimiter = ',',
        default_value = "baseline,health,access,yield,expansion,combined"
    )]
    modes: Vec<Mode>,
    #[arg(long, default_value_t = 256)]
    resolution: u32,
    #[arg(long, default_value_t = 256)]
    ecology_resolution: u32,
    #[arg(long, default_value_t = 16)]
    civilizations: u32,
    #[arg(long, default_value_t = 1)]
    epochs: u32,
    #[arg(long, value_delimiter = ',', default_value = "100,200,500,1000")]
    gates: Vec<u32>,
    /// Trailing annual window used at gates; includes both window endpoints.
    #[arg(long, default_value_t = 20)]
    trend_years: u32,
    /// Ignore smaller endpoint losses as noise; slope must also be negative.
    #[arg(long, default_value_t = 0.01)]
    reversal_fraction: f64,
    /// Experimental soft cap, no larger than existing GPU capacity (2560).
    #[arg(long, default_value_t = 2560)]
    settlement_cap: usize,
    #[arg(long, default_value = "output/growth-ladder")]
    output: PathBuf,
    /// Use aggregate demographic resolution instead of complete individual rosters.
    #[arg(long)]
    aggregate: bool,
    #[arg(long)]
    resume: bool,
    /// Keep evaluating later gates even if a gate reports decline; can resume stopped arms.
    #[arg(long)]
    continue_on_decline: bool,
    /// At each arm's first gate verify actual one-month checkpoint continuation.
    #[arg(long)]
    verify_resume: bool,
    /// Write monthly site-level food/production boundary evidence (under output/).
    #[arg(long)]
    food_diagnostics: bool,
    /// Diagnostic intervention: release existing geological P faster, never import it.
    #[arg(long)]
    phosphorus_release: Option<f32>,
    /// Diagnostic intervention: allocate existing town food by need without purchase.
    #[arg(long)]
    needs_based_food: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Sample {
    year: u32,
    population: f64,
    births: f64,
    deaths: f64,
    records: usize,
    active: usize,
    abandoned: usize,
    candidates: usize,
    unused_candidates: usize,
    food_kg: f64,
    produced_food_kg: f64,
    consumed_food_kg: f64,
    hungry_sites: usize,
    resettlements: usize,
    named_records: usize,
    council_cash: f64,
    household_food_gap_kg: f64,
    municipal_relief_paid_latest_month: f64,
    events: usize,
    population_residual: f64,
    food_residual: f64,
    economy_residuals: [f64; 6],
    founding: Option<ancient_world::civilization::growth::Review>,
}
fn sample(g: &Generator) -> Sample {
    let h = g.civilizations.as_ref().unwrap();
    Sample {
        year: h.month / MONTHS_PER_YEAR,
        population: h.growth_population(),
        births: h.sites.iter().map(|s| s.stocks.people[0] as f64).sum(),
        deaths: h.sites.iter().map(|s| s.stocks.people[1] as f64).sum(),
        records: h.sites.len(),
        active: h.sites.iter().filter(|s| !s.abandoned).count(),
        abandoned: h.sites.iter().filter(|s| s.abandoned).count(),
        candidates: h.candidates.len(),
        unused_candidates: h
            .candidates
            .iter()
            .filter(|c| h.sites.iter().all(|s| s.cell != c.cell))
            .count(),
        food_kg: h.sites.iter().map(|s| s.stocks.stock[1] as f64).sum(),
        produced_food_kg: h.sites.iter().map(|s| s.stocks.ledger[0] as f64).sum(),
        consumed_food_kg: h.sites.iter().map(|s| s.stocks.ledger[1] as f64).sum(),
        hungry_sites: h
            .sites
            .iter()
            .filter(|s| !s.abandoned && s.stocks.stock[3] > 0.1)
            .count(),
        resettlements: h
            .society
            .as_ref()
            .map_or(0, |s| s.relocation.resettlement.occupations.len()),
        named_records: h.people.len(),
        council_cash: h
            .society
            .as_ref()
            .map_or(0., |s| s.councils.iter().map(|c| c.treasury).sum()),
        household_food_gap_kg: h.society.as_ref().map_or(0., |s| {
            s.household_economy.as_ref().map_or(0., |e| {
                e.accounts
                    .iter()
                    .map(|a| (a.need - a.common_food - a.purchased_food).max(0.))
                    .sum()
            })
        }),
        municipal_relief_paid_latest_month: h.society.as_ref().map_or(0., |s| {
            s.household_economy.as_ref().map_or(0., |e| {
                e.municipal_relief.receipts.iter().map(|r| r.paid).sum()
            })
        }),
        events: h.events.len(),
        population_residual: h.population_residual(),
        food_residual: h.food_residual(),
        economy_residuals: h.economy_residuals(),
        founding: h.growth.review.clone(),
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Trend {
    from_year: u32,
    through_year: u32,
    slope_people_per_year: f64,
    fractional_change: f64,
    declining: bool,
}
fn trend(samples: &[Sample], window: u32, tolerance: f64) -> Trend {
    let last = samples.last().unwrap();
    let points: Vec<_> = samples
        .iter()
        .filter(|s| s.year >= last.year.saturating_sub(window))
        .collect();
    let first = points[0];
    let mean_x = points.iter().map(|s| s.year as f64).sum::<f64>() / points.len() as f64;
    let mean_y = points.iter().map(|s| s.population).sum::<f64>() / points.len() as f64;
    let denom = points
        .iter()
        .map(|s| (s.year as f64 - mean_x).powi(2))
        .sum::<f64>();
    let slope = if denom > 0. {
        points
            .iter()
            .map(|s| (s.year as f64 - mean_x) * (s.population - mean_y))
            .sum::<f64>()
            / denom
    } else {
        0.
    };
    let change = (last.population - first.population) / first.population.max(1.);
    Trend {
        from_year: first.year,
        through_year: last.year,
        slope_people_per_year: slope,
        fractional_change: change,
        declining: points.len() > 1 && slope < 0. && change < -tolerance,
    }
}
fn gate_status(
    population: f64,
    declining: bool,
    final_gate: bool,
    continue_on_decline: bool,
) -> &'static str {
    if population <= 0. {
        "extinct"
    } else if declining && !continue_on_decline {
        "growth_reversed"
    } else if final_gate {
        "complete"
    } else {
        "running"
    }
}
#[derive(Serialize, Deserialize)]
struct Progress {
    year: u32,
    checkpoint: PathBuf,
    status: String,
    gates: Vec<Trend>,
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(tmp, path)?;
    Ok(())
}
fn save(g: &Generator, path: &Path) -> Result<()> {
    let tmp = path.with_extension("world.tmp");
    g.save(&tmp)?;
    fs::rename(tmp, path)?;
    Ok(())
}
fn verify_continuation(g: &mut Generator, gpu: &ContextGpu, path: &Path) -> Result<()> {
    let mut resumed = Generator::load(gpu.clone(), path)?;
    ensure!(
        serde_json::to_value(&g.civilizations)? == serde_json::to_value(&resumed.civilizations)?,
        "checkpoint state differs"
    );
    g.advance_history(1)?;
    resumed.advance_history(1)?;
    ensure!(
        serde_json::to_value(&g.civilizations)? == serde_json::to_value(&resumed.civilizations)?,
        "checkpoint continuation differs"
    );
    *g = Generator::load(gpu.clone(), path)?;
    Ok(())
}
fn run_arm(args: &Args, gpu: &ContextGpu, base: &Path, seed: u32, mode: Mode) -> Result<()> {
    let folder = args.output.join(format!("seed-{seed}")).join(mode.label());
    fs::create_dir_all(&folder)?;
    let progress_path = folder.join("progress.json");
    let mut progress: Progress = if args.resume && progress_path.exists() {
        serde_json::from_slice(&fs::read(&progress_path)?)?
    } else {
        Progress {
            year: 0,
            checkpoint: base.into(),
            status: "running".into(),
            gates: vec![],
        }
    };
    if progress.status == "extinct"
        || progress.status == "complete"
        || (progress.status == "growth_reversed" && !args.continue_on_decline)
    {
        return Ok(());
    }
    let mut g = Generator::load(gpu.clone(), &progress.checkpoint)?;
    if progress.year == 0 {
        mode.apply(&mut g, args.settlement_cap)?;
        if let Some(rate) = args.phosphorus_release {
            let catalog = g
                .civilizations
                .as_mut()
                .unwrap()
                .economy_catalog
                .as_mut()
                .unwrap();
            catalog.production.phosphorus_release_monthly_fraction = rate;
            catalog.validate()?;
        }
        if args.needs_based_food {
            g.civilizations
                .as_mut()
                .unwrap()
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .needs_based_food = true;
        }
    }
    if args.food_diagnostics {
        g.civilizations
            .as_mut()
            .unwrap()
            .demographic_audit
            .get_or_insert_with(Default::default);
    }
    // Resume keeps only observations at/before the durable checkpoint, just like annual rows.
    let diagnostic_path = folder.join("food-monthly.jsonl");
    let mut diagnostic_out = if args.food_diagnostics {
        let retained = if args.resume && diagnostic_path.exists() {
            fs::read_to_string(&diagnostic_path)?
                .lines()
                .map(|line| {
                    Ok((
                        serde_json::from_str::<serde_json::Value>(line)?,
                        line.to_owned(),
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter(|(v, _)| {
                    v["month"]
                        .as_u64()
                        .is_some_and(|m| m <= u64::from(progress.year * MONTHS_PER_YEAR))
                })
                .map(|(_, line)| line)
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let mut file = fs::File::create(&diagnostic_path)?;
        for line in retained {
            writeln!(file, "{line}")?;
        }
        Some(file)
    } else {
        None
    };
    ensure!(
        g.civilizations.as_ref().unwrap().month == progress.year * MONTHS_PER_YEAR,
        "checkpoint month differs from progress"
    );
    write_json(
        &folder.join("effective-settings.json"),
        &json!({"mode":mode,"growth":g.civilizations.as_ref().unwrap().growth,
        "config":g.config,"systems":g.config.systems,"base_checkpoint":base,
        "production":g.civilizations.as_ref().unwrap().economy_catalog.as_ref().map(|c|&c.production),
        "needs_based_food":g.civilizations.as_ref().unwrap().society.as_ref()
            .and_then(|s|s.household_economy.as_ref()).map(|e|e.needs_based_food)}),
    )?;
    let annual = folder.join("annual.jsonl");
    let mut samples: Vec<Sample> = if args.resume && annual.exists() {
        fs::read_to_string(&annual)?
            .lines()
            .take(progress.year as usize + 1)
            .map(serde_json::from_str::<Sample>)
            .collect::<std::result::Result<Vec<_>, _>>()?
    } else {
        vec![]
    };
    if samples.is_empty() {
        samples.push(sample(&g));
    }
    ensure!(
        samples.last().unwrap().year == progress.year,
        "annual evidence missing at checkpoint"
    );
    ensure!(
        samples.iter().enumerate().all(|(i, s)| s.year == i as u32),
        "annual evidence has missing or duplicate years"
    );
    // Discard uncheckpointed tail before replay after interruption.
    let mut out = fs::File::create(&annual)?;
    for s in &samples {
        writeln!(out, "{}", serde_json::to_string(s)?)?;
    }
    for &gate in args
        .gates
        .iter()
        .filter(|&&y| y > progress.year)
        .collect::<Vec<_>>()
    {
        let start = Instant::now();
        while g.civilizations.as_ref().unwrap().month < gate * MONTHS_PER_YEAR {
            for _ in 0..if args.food_diagnostics {
                MONTHS_PER_YEAR
            } else {
                1
            } {
                g.advance_history(if args.food_diagnostics {
                    1
                } else {
                    MONTHS_PER_YEAR
                })
                .with_context(|| {
                    format!(
                        "seed {seed} mode {} year {}",
                        mode.label(),
                        g.civilizations.as_ref().unwrap().month / MONTHS_PER_YEAR
                    )
                })?;
                if let Some(out) = &mut diagnostic_out {
                    let h = g.civilizations.as_ref().unwrap();
                    let audit = h.demographic_audit.as_ref().unwrap();
                    let rows: Vec<_> = audit
                        .food
                        .months
                        .iter()
                        .filter(|r| r.month == h.month)
                        .collect();
                    let sites: Vec<_> = h.sites.iter().map(|s| json!({
                    "site":s.id,"population":s.stocks.stock[0],"ages":s.demography.ages,
                    "births_deaths":s.stocks.people,"crop_stocks":s.economy.crops,
                    "water_service":s.economy.water_service,"soil":s.economy.soil,
                    "reserves":s.economy.reserves,"agriculture":s.economy.agriculture,
                    "labor":s.economy.labor,"food_labor":s.economy.food_labor,
                    "finance":s.economy.finance,"prices":s.economy.prices.as_slice(),
                    "waterworks":s.economy.waterworks_plan,
                    "households":h.society.as_ref().and_then(|s|s.household_economy.as_ref()).map(|e|
                        e.accounts.iter().filter(|a|a.food_site==Some(s.id)).collect::<Vec<_>>())
                })).collect();
                    writeln!(
                        out,
                        "{}",
                        json!({"month":h.month,"food":rows,"sites":sites,
                            "food_requests":h.trade_contact.food_requests})
                    )?;
                    out.flush()?;
                }
            }
            let row = sample(&g);
            writeln!(out, "{}", serde_json::to_string(&row)?)?;
            out.flush()?;
            samples.push(row);
        }
        let t = trend(&samples, args.trend_years, args.reversal_fraction);
        let checkpoint = folder.join(format!("year-{gate}.world"));
        save(&g, &checkpoint)?;
        if args.verify_resume && progress.gates.is_empty() {
            verify_continuation(&mut g, gpu, &checkpoint)?;
        }
        progress.year = gate;
        progress.checkpoint = checkpoint;
        progress.gates.push(t.clone());
        progress.status = gate_status(
            samples.last().unwrap().population,
            t.declining,
            gate == *args.gates.last().unwrap(),
            args.continue_on_decline,
        )
        .into();
        write_json(&progress_path, &progress)?;
        let last = samples.last().unwrap();
        let milestones:Vec<_>=[0.25,0.5,0.75,0.9,1.].into_iter().map(|fraction|json!({"fraction":fraction,
            "year":samples.iter().find(|s|s.records as f64>=args.settlement_cap as f64*fraction).map(|s|s.year)})).collect();
        write_json(
            &folder.join("summary.json"),
            &json!({"seed":seed,"mode":mode,"status":progress.status,"gate":gate,"trend":t,
            "latest":last,"milestones":milestones,"stage_seconds":start.elapsed().as_secs_f64(),
            "max_economy_residual":samples.iter().flat_map(|s|s.economy_residuals).map(f64::abs).fold(0.,f64::max)}),
        )?;
        eprintln!(
            "seed {seed} {} year {gate}: population {:.0}, sites {}/{}, slope {:.2}/year, {}",
            mode.label(),
            last.population,
            last.active,
            last.records,
            t.slope_people_per_year,
            progress.status
        );
        if progress.status == "growth_reversed" || progress.status == "extinct" {
            break;
        }
    }
    if folder.join("error.json").exists() {
        fs::remove_file(folder.join("error.json"))?;
    }
    Ok(())
}
fn main() -> Result<()> {
    let args = Args::parse();
    ensure!(
        !args.seeds.is_empty()
            && args
                .seeds
                .iter()
                .enumerate()
                .all(|(i, x)| !args.seeds[..i].contains(x))
            && args
                .modes
                .iter()
                .enumerate()
                .all(|(i, x)| !args.modes[..i].contains(x))
            && !args.modes.is_empty()
            && !args.gates.is_empty()
            && args.gates[0] > 0
            && args.gates.windows(2).all(|w| w[1] > w[0])
            && *args.gates.last().unwrap() <= 1000
            && args.trend_years > 0
            && (0. ..1.).contains(&args.reversal_fraction)
            && (1..=16).contains(&args.civilizations)
            && args.settlement_cap >= args.civilizations as usize,
        "invalid experiment settings"
    );
    Mode::Baseline.policy(args.settlement_cap).validate()?;
    let root = std::env::current_dir()?.join("output");
    fs::create_dir_all(&root)?;
    fs::create_dir_all(&args.output)?;
    ensure!(
        args.output
            .canonicalize()?
            .starts_with(root.canonicalize()?),
        "experiment outputs must be under ignored output/"
    );
    let mut spec = json!({"version":1,"seeds":args.seeds,"modes":args.modes,"resolution":args.resolution,"ecology_resolution":args.ecology_resolution,
        "civilizations":args.civilizations,"epochs":args.epochs,"gates":args.gates,"trend_years":args.trend_years,
        "reversal_fraction":args.reversal_fraction,"settlement_cap":args.settlement_cap,"aggregate":args.aggregate,
        "health_scale":HEALTH_BACKGROUND_SCALE,"yield_multiplier":YIELD_MULTIPLIER,
        "expansion_population_scale":EXPANSION_POPULATION_SCALE,"expansion_reserve_months":EXPANSION_RESERVE_MONTHS});
    if args.phosphorus_release.is_some() || args.needs_based_food {
        spec["diagnostic_interventions"] = json!({"phosphorus_release":args.phosphorus_release,
            "needs_based_food":args.needs_based_food});
    }
    let spec_path = args.output.join("suite.json");
    if spec_path.exists() {
        ensure!(
            args.resume,
            "output already exists; use --resume or a new directory"
        );
        ensure!(
            serde_json::from_slice::<serde_json::Value>(&fs::read(&spec_path)?)? == spec,
            "resume settings differ; choose a new output directory"
        );
    } else {
        write_json(&spec_path, &spec)?;
    }
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let revision = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());
    let provenance = args.output.join("environment.json");
    if !provenance.exists() {
        write_json(
            &provenance,
            &json!({"gpu":gpu.adapter_name,"revision":revision,"settings":spec}),
        )?;
    }
    let mut failures = vec![];
    for &seed in &args.seeds {
        let base = args.output.join(format!("seed-{seed}")).join("base.world");
        fs::create_dir_all(base.parent().unwrap())?;
        if !base.exists() {
            let mut g = Generator::new(
                gpu.clone(),
                Config {
                    seed,
                    resolution: args.resolution,
                    ecology_resolution: args.ecology_resolution,
                    ..Default::default()
                },
                Catalog::bundled()?,
            )?;
            g.run_epochs(args.epochs)?;
            g.found_civilizations(args.civilizations)?;
            g.apply_systems(&Systems::default())?;
            g.civilizations
                .as_mut()
                .unwrap()
                .set_demographic_resolution(
                    if args.aggregate {
                        Demography::Aggregate
                    } else {
                        Demography::Individual
                    },
                    false,
                )?;
            save(&g, &base)?;
        }
        for &mode in &args.modes {
            if let Err(e) = run_arm(&args, &gpu, &base, seed, mode) {
                let message = format!("{e:#}");
                eprintln!("FAILED seed {seed} {}: {message}", mode.label());
                let folder = args.output.join(format!("seed-{seed}")).join(mode.label());
                fs::create_dir_all(&folder)?;
                write_json(
                    &folder.join("error.json"),
                    &json!({"seed":seed,"mode":mode,"error":message}),
                )?;
                failures.push((seed, mode, message));
            }
        }
    }
    ensure!(
        failures.is_empty(),
        "{} experiment arms failed; inspect error.json files",
        failures.len()
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn series(values: &[f64]) -> Vec<Sample> {
        values.iter().enumerate().map(|(year,&population)| {
        serde_json::from_value(json!({"year":year,"population":population,"births":0.,"deaths":0.,"records":16,"active":16,"abandoned":0,
            "candidates":30,"unused_candidates":14,"food_kg":0.,"produced_food_kg":0.,"consumed_food_kg":0.,"hungry_sites":0,"resettlements":0,
            "named_records":0,"council_cash":0.,"household_food_gap_kg":0.,"municipal_relief_paid_latest_month":0.,"events":0,"population_residual":0.,"food_residual":0.,"economy_residuals":[0.,0.,0.,0.,0.,0.],"founding":null})).unwrap()
    }).collect()
    }
    #[test]
    fn gate_uses_recent_decline_not_a_single_bad_year_or_founding_baseline() {
        assert!(!trend(&series(&[100., 110., 120., 119.]), 3, 0.01).declining);
        assert!(trend(&series(&[100., 200., 190., 170.]), 2, 0.01).declining);
        assert!(trend(&series(&[100., 90., 80.]), 20, 0.01).declining);
        assert!(!trend(&series(&[100., 100., 100.]), 20, 0.01).declining);
        assert!(!trend(&series(&[100., 99.9, 99.8]), 20, 0.01).declining);
    }
    #[test]
    fn gate_status_stops_declines_allows_override_and_stops_extinction() {
        assert_eq!(gate_status(80., true, false, false), "growth_reversed");
        assert_eq!(gate_status(80., true, false, true), "running");
        assert_eq!(gate_status(80., false, false, false), "running");
        assert_eq!(gate_status(80., false, true, false), "complete");
        assert_eq!(gate_status(0., false, false, true), "extinct");
    }
    #[test]
    fn modes_are_separate_and_combined_has_declared_parameters() {
        assert_eq!(Mode::Health.policy(2560).founding_population_scale, 1.);
        assert_eq!(Mode::Expansion.policy(2560).background_mortality_scale, 1.);
        let c = Mode::Combined.policy(2560);
        assert_eq!(c.background_mortality_scale, 0.85);
        assert_eq!(c.founding_reserve_months, 9.);
    }
}
