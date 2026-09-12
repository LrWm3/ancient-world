//! Monthly accounting with decadal summaries; raw outputs belong under ignored output/.
use ancient_world::{
    catalog::Catalog,
    config::Config,
    gpu::{ContextGpu, Generator},
};
use anyhow::{Context, Result};
use clap::Parser;
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf, time::Instant};
#[derive(Parser)]
struct Args {
    /// Keep initial distribution shares fixed while leaving elections and tax politics active.
    #[arg(long)]
    fixed_distribution: bool,
    /// Have institutions buy writing supplies for heritage interpretation.
    #[arg(long)]
    funded_heritage_study: bool,
    /// Separate core operating space from whole-membership expansion demand.
    #[arg(long)]
    institution_working_core: bool,
    /// Prefer eligible institutional students on alternate cultural quarters.
    #[arg(long)]
    institutional_students: bool,
    /// Fund basic upkeep and member administration before additional repairs.
    #[arg(long, requires = "named_institution_administration")]
    essential_institution_work: bool,
    /// Assign an institution member to administration independently of the generic culture bundle.
    #[arg(long, conflicts_with = "legacy_participation")]
    named_institution_administration: bool,
    /// Quote proportional institution operating budgets against bounded town funding.
    #[arg(long)]
    operating_institutions: bool,
    /// Rotate institutional priority quarterly within election and upkeep requests.
    #[arg(long, conflicts_with = "legacy_participation")]
    rotating_institutions: bool,
    /// Retain dated household food/income observations for distributional diagnosis.
    #[arg(long)]
    household_diagnostics: bool,
    /// Local kin gifts from surplus household wallets, before council relief.
    #[arg(long)]
    family_support: bool,
    #[arg(long)]
    legacy_resident_payroll: bool,
    /// Ablate personal food exposure while retaining the same individual population.
    #[arg(long)]
    no_individual_nutrition: bool,
    /// Use age-band mortality while retaining personal hunger effects on work.
    #[arg(long, conflicts_with = "no_individual_nutrition")]
    no_household_mortality: bool,
    /// Override the long-run common entitlement without changing food production.
    #[arg(long)]
    common_share: Option<f32>,
    /// Override the existing finite council-to-household relief budget.
    #[arg(long, requires = "household_relief_target")]
    household_relief_share: Option<f32>,
    /// Dietary entitlement targeted by council relief, without changing common food.
    #[arg(long, requires = "household_relief_share")]
    household_relief_target: Option<f32>,
    #[arg(long, default_value_t = 0.5)]
    crop_yield_scale: f32,
    /// Matched control: use the configured common share immediately from founding.
    #[arg(long)]
    no_founding_access: bool,
    #[arg(long, conflicts_with = "individual_demography")]
    aggregate_resolution: bool,
    #[arg(long)]
    compare_resolution: bool,
    #[arg(long, requires = "individual_demography")]
    workshop_refinement: bool,
    #[arg(long, requires_all = ["individual_demography", "workshop_refinement"])]
    agriculture_refinement: bool,
    #[arg(long, requires = "agriculture_refinement")]
    extraction_refinement: bool,
    #[arg(long, requires = "extraction_refinement")]
    construction_refinement: bool,
    #[arg(long)]
    individual_demography: bool,
    /// Identify all whole residents at the initial boundary; does not replace cohort demography.
    #[arg(long)]
    resident_baseline: bool,
    #[arg(long)]
    legacy_participation: bool,
    #[arg(long)]
    no_domestic_care: bool,
    #[arg(long)]
    legacy_named_demography: bool,
    #[arg(long)]
    strict_identities: bool,
    #[arg(long, value_delimiter = ',', default_value = "17,81,256")]
    seeds: Vec<u32>,
    #[arg(long, default_value_t = 100)]
    years: u32,
    #[arg(long, default_value_t = 32)]
    resolution: u32,
    #[arg(long, default_value = "output/cultural-work-baseline.json")]
    output: PathBuf,
}
fn main() -> Result<()> {
    let args = Args::parse();
    anyhow::ensure!(
        !args.compare_resolution || args.individual_demography || args.aggregate_resolution,
        "comparison requires a demographic resolution mode"
    );
    anyhow::ensure!(args.years > 0 && args.years <= 500, "years must be 1..500");
    for value in [args.household_relief_share, args.household_relief_target]
        .into_iter()
        .flatten()
    {
        anyhow::ensure!(
            value.is_finite() && (0. ..=1.).contains(&value),
            "household relief fractions must be in 0..1"
        );
    }
    let gpu = pollster::block_on(ContextGpu::headless())?;
    let mut rows = vec![];
    for seed in &args.seeds {
        let start = Instant::now();
        let mut g = Generator::new(
            gpu.clone(),
            Config {
                seed: *seed,
                crop_yield_scale: args.crop_yield_scale,
                resolution: args.resolution,
                ecology_resolution: 16,
                ..Default::default()
            },
            Catalog::bundled()?,
        )?;
        g.run_epochs(1)?;
        g.found_civilizations(16)?;
        g.civilizations
            .as_mut()
            .unwrap()
            .set_individual_participation(!args.legacy_participation)?;
        g.civilizations
            .as_mut()
            .unwrap()
            .culture
            .as_mut()
            .unwrap()
            .focused_work_identities = !args.strict_identities;
        g.civilizations
            .as_mut()
            .unwrap()
            .culture
            .as_mut()
            .unwrap()
            .institutional_students = args.institutional_students;
        g.civilizations
            .as_mut()
            .unwrap()
            .culture
            .as_mut()
            .unwrap()
            .institution_working_core = args.institution_working_core;
        g.civilizations
            .as_mut()
            .unwrap()
            .culture
            .as_mut()
            .unwrap()
            .funded_heritage_study = args.funded_heritage_study;
        if args.named_institution_administration {
            g.civilizations
                .as_mut()
                .unwrap()
                .culture
                .as_mut()
                .unwrap()
                .named_administration = true;
        }
        if args.essential_institution_work {
            g.civilizations
                .as_mut()
                .unwrap()
                .culture
                .as_mut()
                .unwrap()
                .institution_work_policy =
                ancient_world::institution_capacity::WorkPolicy::EssentialFirst;
        }
        if args.operating_institutions {
            g.civilizations
                .as_mut()
                .unwrap()
                .culture
                .as_mut()
                .unwrap()
                .institution_funding = ancient_world::institution_funding::Policy::Operating;
        }
        if args.rotating_institutions {
            g.civilizations
                .as_mut()
                .unwrap()
                .culture
                .as_mut()
                .unwrap()
                .institution_priority = ancient_world::institution_capacity::Priority::Rotating;
        }
        g.civilizations
            .as_mut()
            .unwrap()
            .set_domestic_households(!args.no_domestic_care)?;
        g.civilizations
            .as_mut()
            .unwrap()
            .set_named_demography(!args.legacy_named_demography)?;
        g.enable_society()?;
        g.civilizations
            .as_mut()
            .unwrap()
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .resident_payroll = !args.legacy_resident_payroll;
        g.civilizations
            .as_mut()
            .unwrap()
            .society
            .as_mut()
            .unwrap()
            .household_economy
            .as_mut()
            .unwrap()
            .political_distribution = !args.fixed_distribution;
        if let Some(share) = args.common_share {
            anyhow::ensure!(
                share.is_finite() && (0. ..=1.).contains(&share),
                "invalid common share"
            );
            g.civilizations
                .as_mut()
                .unwrap()
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .common_share = share;
        }
        if args.family_support {
            g.civilizations
                .as_mut()
                .unwrap()
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .family_support =
                Some(ancient_world::household_economy::FamilySupportPolicy::default());
        }
        if let (Some(share), Some(target)) =
            (args.household_relief_share, args.household_relief_target)
        {
            let e = g
                .civilizations
                .as_mut()
                .unwrap()
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap();
            e.relief_share = share;
            e.relief_target = target;
        }
        if args.no_individual_nutrition {
            g.civilizations
                .as_mut()
                .unwrap()
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .individual_nutrition = false;
        }
        if args.no_household_mortality {
            g.civilizations
                .as_mut()
                .unwrap()
                .society
                .as_mut()
                .unwrap()
                .household_economy
                .as_mut()
                .unwrap()
                .household_mortality = false;
        }
        if args.no_founding_access {
            g.configure_founding_food_access(None)?;
        }
        g.enable_politics()?;
        g.enable_governance()?;
        g.enable_offices()?;
        g.enable_shipping()?;
        g.enable_expeditions()?;
        g.enable_discoveries()?;
        g.enable_living_history()?;
        if args.individual_demography || args.aggregate_resolution {
            g.civilizations
                .as_mut()
                .unwrap()
                .set_demographic_resolution(
                    if args.individual_demography {
                        ancient_world::resolution::Mode::Individual
                    } else {
                        ancient_world::resolution::Mode::Aggregate
                    },
                    args.compare_resolution,
                )?;
        }
        if args.workshop_refinement {
            g.civilizations
                .as_mut()
                .unwrap()
                .set_workshop_refinement(true)?;
        }
        if args.agriculture_refinement {
            g.civilizations
                .as_mut()
                .unwrap()
                .set_agriculture_refinement(true)?;
        }
        if args.resident_baseline {
            g.civilizations
                .as_mut()
                .unwrap()
                .identify_resident_baseline()?;
        }
        let mut samples = vec![];
        let mut changes = BTreeMap::<String, u64>::new();
        let mut actions = BTreeMap::<String, u64>::new();
        let (mut requested, mut granted, mut used, mut cancelled_work) = (0., 0., 0., 0.);
        let (mut funded, mut cancelled) = (0u64, 0u64);
        let mut food = [0f64; 6];
        let mut max_population_residual = 0f64;
        if args.extraction_refinement {
            g.civilizations
                .as_mut()
                .unwrap()
                .set_extraction_refinement(true)?;
        }
        if args.construction_refinement {
            g.civilizations
                .as_mut()
                .unwrap()
                .set_construction_refinement(true)?;
        }
        let mut production_work = [[0f64; 3]; 4];
        let mut max_food_residual = 0f64;
        let mut mortality_by_age = [[0f64; 3]; 2];
        let mut demographic_months = 0u64;
        let mut lesson_opportunities = [0u64; 6];
        let mut lesson_observations = 0u64;
        let mut service_room = [[0f64; 4]; 3];
        let mut service_counts = [[0u64; 4]; 3];
        let mut agriculture_audit_rounding_cases = 0u64;
        let mut opening_relief_room = 0f64;
        let mut opening_relief_dispatches = 0u64;
        let mut institution_work = [[0f64; 3]; 3];
        let mut institution_funding = [0f64; 3];
        let mut institution_funding_attempts = [0u64; 2];
        let mut institution_multi_request_quarters = [0u64; 3];
        let mut institution_shortfall_quarters = [0u64; 3];
        for month in 1..=args.years * 12 {
            if let Err(error) = g.advance_history(1) {
                // The history transaction may have rolled back while ecology already
                // advanced. Preserve evidence, explicitly not a resumable checkpoint.
                let diagnostic = args
                    .output
                    .with_extension(format!("seed-{seed}-month-{month}.failure.json"));
                let write = (|| -> Result<()> {
                    if let Some(parent) = diagnostic.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(
                        &diagnostic,
                        serde_json::to_vec_pretty(&json!({
                            "diagnostic_only": true, "seed": seed, "attempted_month": month,
                            "error": format!("{error:#}"), "history": g.civilizations,
                            "completed_samples": samples,
                        }))?,
                    )?;
                    Ok(())
                })();
                if let Err(write_error) = write {
                    eprintln!("Could not preserve failure evidence: {write_error:#}");
                }
                return Err(error).with_context(|| format!("seed {seed}, month {month}"));
            }
            let h = g.civilizations.as_ref().unwrap();
            if let Some(a) = h.resolution.as_ref().and_then(|r| r.agriculture.as_ref()) {
                for p in &a.plans {
                    if p.granted() > p.requested + 1e-4 && p.grants_within_request() {
                        agriculture_audit_rounding_cases += 1;
                        eprintln!("seed {seed}, month {month}: grant audit rounding, site {}, sector {}, requested {}, f32 {}, stored {}",
                            p.site,p.sector,p.requested,p.granted(),
                            p.assignments.iter().map(|a|a.granted as f64).sum::<f64>());
                    }
                }
            }
            let c = h.culture.as_ref().unwrap();
            if let Some(resolution) = &h.resolution {
                for receipt in &resolution.receipts {
                    let Some(snapshot) = &receipt.demographic_snapshot else {
                        continue;
                    };
                    anyhow::ensure!(snapshot.month == h.month, "stale demographic diagnostic");
                    demographic_months += 1;
                    let projection = &snapshot.projection;
                    let mut personal = std::array::from_fn::<_, 3, _>(|b| {
                        snapshot.anonymous[b] * projection.mortality[b]
                    });
                    if let Some(people) = &snapshot.people {
                        for (id, band, _) in people {
                            personal[*band] += snapshot
                                .personal_mortality
                                .get(id)
                                .copied()
                                .unwrap_or(projection.mortality[*band]);
                        }
                    } else {
                        personal = std::array::from_fn(|b| {
                            projection.opening[b] * projection.mortality[b]
                        });
                    }
                    for (b, value) in personal.into_iter().enumerate() {
                        mortality_by_age[0][b] += projection.opening[b] * projection.mortality[b];
                        mortality_by_age[1][b] += value;
                    }
                }
            }
            max_population_residual = max_population_residual.max(h.population_residual().abs());
            max_food_residual = max_food_residual.max(h.food_residual().abs());
            if let Some(a) = h.resolution.as_ref().and_then(|r| r.agriculture.as_ref()) {
                for plan in &a.plans {
                    anyhow::ensure!(
                        plan.month == h.month && plan.settled && plan.sector < 4,
                        "production diagnostics require a completed current plan"
                    );
                    production_work[plan.sector][0] += plan.requested as f64;
                    production_work[plan.sector][1] += plan.granted() as f64;
                    production_work[plan.sector][2] +=
                        plan.assignments.iter().map(|a| a.used as f64).sum::<f64>();
                }
            }
            for site in &h.sites {
                let d = &site.demography;
                if d.household_food[1] <= 0.5 {
                    continue;
                }
                let need = d.ration_need[3] as f64;
                let available = d.household_food[2] as f64;
                let eaten = d.ration_eaten[3] as f64;
                let physical = (need - available).max(0.);
                let access = (need.min(available) - eaten).max(0.);
                anyhow::ensure!(
                    (need - eaten - physical - access).abs() <= 1e-4 * (1. + need),
                    "food gap decomposition failed"
                );
                for (total, value) in food.iter_mut().zip([
                    need,
                    available,
                    d.household_food[0] as f64,
                    eaten,
                    physical,
                    access,
                ]) {
                    *total += value;
                }
            }
            if let Some(p) = &h.participation {
                let completed: f64 = p.residents.values().map(|r| r.completed[0]).sum();
                anyhow::ensure!(
                    (completed - c.labor_spent).abs() <= 0.001 + c.labor_spent * 1e-6,
                    "personal and cultural lifetime work disagree: {completed} vs {}",
                    c.labor_spent
                );
                if let Some(d) = h.expeditions.as_ref().and_then(|x| x.discoveries.as_ref()) {
                    let completed: f64 = p.residents.values().map(|r| r.completed[1]).sum();
                    anyhow::ensure!(
                        (completed - d.worker_months).abs() <= 0.001 + d.worker_months * 1e-6,
                        "personal and research lifetime work disagree"
                    );
                }
            }
            for mission in c
                .religious_relief
                .missions
                .iter()
                .filter(|m| m.dispatched == h.month)
            {
                if let Some(room) = &mission.room {
                    opening_relief_room += room.used;
                    opening_relief_dispatches += 1;
                }
            }
            requested += c.work_receipt.requested;
            granted += c.work_receipt.granted;
            used += c.work_receipt.used;
            for p in &c.work_plans {
                anyhow::ensure!(p.month == h.month, "stale institutional diagnostic");
                if let Some(counts) = p.lesson_opportunities {
                    lesson_observations += 1;
                    for (total, count) in lesson_opportunities.iter_mut().zip(counts) {
                        *total += u64::from(count);
                    }
                }
                for plan in p.services.iter().flatten() {
                    anyhow::ensure!(
                        plan.month == h.month && plan.closed,
                        "service diagnostics require a closed current plan"
                    );
                    for r in &plan.receipts {
                        use ancient_world::institution_services::Service;
                        let k = match r.service {
                            Service::Lesson { .. } => 0,
                            Service::HeritageStudy { .. } => 1,
                            Service::PetitionHearing { .. } => 2,
                        };
                        for (total, value) in service_room[k].iter_mut().zip([
                            r.requested,
                            r.opening_grant(),
                            r.granted,
                            r.used,
                        ]) {
                            *total += value;
                        }
                        for (total, positive) in service_counts[k].iter_mut().zip([
                            true,
                            r.opening_grant() > 0.,
                            r.granted > 0.,
                            r.used > 0.,
                        ]) {
                            *total += u64::from(positive);
                        }
                    }
                }
                if let Some(b) = &p.funding {
                    for r in &b.requests {
                        if r.ceiling > 0. {
                            institution_funding_attempts[0] += 1;
                            institution_funding_attempts[1] += u64::from(r.settled);
                        }
                        for (total, value) in
                            institution_funding
                                .iter_mut()
                                .zip([r.requested, r.ceiling, r.paid])
                        {
                            *total += value;
                        }
                    }
                }
                for (k, plans) in [&p.elections, &p.upkeep, &p.administration]
                    .into_iter()
                    .enumerate()
                {
                    if let Some(plans) = plans {
                        let requested = plans.iter().map(|u| u.requested as f64).sum::<f64>();
                        let granted = plans.iter().map(|u| u.granted as f64).sum::<f64>();
                        let used = plans.iter().map(|u| u.used as f64).sum::<f64>();
                        for (total, value) in institution_work[k]
                            .iter_mut()
                            .zip([requested, granted, used])
                        {
                            *total += value;
                        }
                        if plans.len() > 1 {
                            institution_multi_request_quarters[k] += 1;
                            if granted + 1e-6 < requested {
                                institution_shortfall_quarters[k] += 1;
                            }
                        }
                    }
                }
                if p.granted <= 0. {
                    continue;
                }
                funded += 1;
                if p.cancellation.is_some() {
                    cancelled += 1;
                    cancelled_work += p.cancelled_work as f64;
                    for key in &p.changed_identities {
                        *changes.entry(key.clone()).or_default() += 1;
                    }
                    for (a, _) in &p.actions {
                        *actions.entry(a.clone()).or_default() += 1;
                    }
                }
            }
            if month % 120 == 0 || month == args.years * 12 {
                let household_observations =
                    args.household_diagnostics.then(|| {
                        h.society
                            .as_ref()
                            .and_then(|s| s.household_economy.as_ref())
                            .map(|e| {
                                e.accounts.iter().enumerate().map(|(id, a)| json!({
                            "household": id, "food_site": a.food_site,
                            "need_kg": a.need, "common_kg": a.common_food,
                            "purchased_kg": a.purchased_food, "hunger": a.hunger,
                            "cash": a.cash, "sector_wages_this_month": a.sector_wages,
                            "lifetime_wages": a.wages, "lifetime_relief": a.relief,
                            "lifetime_dividends": a.dividends,
                            "lifetime_food_spending": a.food_spending,
                        })).collect::<Vec<_>>()
                            })
                    });
                // Preserve the actual gates and stocks behind the operational count.
                // These are read-only samples, not additional monthly updates.
                let institution_state = h.institution_state_report();
                let mut row = json!({"institution_funding_attempts":institution_funding_attempts,"institution_funding":institution_funding,"institution_state":institution_state,"institution_work":institution_work,"institution_multi_request_quarters":institution_multi_request_quarters,"institution_shortfall_quarters":institution_shortfall_quarters,"operational_institutions":c.institutions.iter().filter(|n| n.operational()).count(),"production_work":production_work,"household_observations":household_observations,"year":month/12,"food_totals":food,"max_population_residual":max_population_residual,"max_food_residual":max_food_residual,"recent_trade_pairs":h.trade_contact.receipts.len(),"demographic_site_months":demographic_months,"expected_deaths_by_age":mortality_by_age,"ages":(0..3).map(|b|h.sites.iter().map(|s|s.demography.ages[b] as f64).sum::<f64>()).collect::<Vec<_>>(),"heritage_recognitions":c.heritage_renown.len(),"heritage_witnesses":c.heritage_renown.iter().map(|r|r.witnesses.len()).sum::<usize>(),"domestic":h.domestic.as_ref().map(|d|json!({"groups":d.units.iter().filter(|u|u.ended.is_none()).count(),"members":d.membership.len(),"completed":d.care_completed,"care":d.care})),"funded_bundles":funded,"cancelled_bundles":cancelled,"requested":requested,"granted":granted,"used":used,"cancelled_work":cancelled_work,"population":h.sites.iter().map(|s|s.stocks.stock[0] as f64).sum::<f64>(),"active_sites":h.sites.iter().filter(|s|!s.abandoned).count(),"institutions":c.institutions.iter().filter(|n|n.active).count(),"knowledge_links":c.agents.iter().map(|a|a.knowledge.len()).sum::<usize>(),"artifacts":c.artifacts.len(),"individuals":h.participation.as_ref().map(|p|json!({"known":p.residents.len(),"available_adults":p.residents.values().filter(|r|r.capacity>0.).count(),"culture_work":p.residents.values().map(|r|r.completed[0]).sum::<f64>(),"research_work":p.residents.values().map(|r|r.completed[1]).sum::<f64>()}))});
                eprintln!("seed {seed}: {} years, cancelled {cancelled}/{funded}, work {used:.1}/{granted:.1}, {:.1}s",month/12,start.elapsed().as_secs_f64());
                let society = h.society.as_ref().unwrap();
                let accounts = &society.household_economy.as_ref().unwrap().accounts;
                let heritage_funding: Vec<_> = h
                    .expeditions
                    .iter()
                    .flat_map(|x| &x.voyages)
                    .filter_map(|v| v.heritage.as_ref()?.find.as_ref())
                    .flat_map(|f| &f.studies)
                    .filter_map(|s| s.funding.as_ref())
                    .collect();
                row["heritage_study_funding"] = json!({"studies": heritage_funding.len(), "paid": heritage_funding.iter().map(|f| f.paid).sum::<f64>(), "writing_kg": heritage_funding.iter().map(|f| f.kg as f64).sum::<f64>()});
                row["distribution_policies"] = json!(society.councils.iter().map(|c| json!({
                    "civilization":c.civilization,
                    "active":c.distribution.unwrap_or_else(|| ancient_world::household_economy::policy::DistributionPolicy::baseline(society.household_economy.as_ref().unwrap())),
                    "pending":c.pending_distribution,
                    "treasury":c.treasury,
                })).collect::<Vec<_>>());
                row["agriculture_audit_rounding_cases"] = json!(agriculture_audit_rounding_cases);
                row["distribution_changes"] = json!(h
                    .events
                    .iter()
                    .filter(|e| e.kind == "distribution_policy_effective")
                    .count());
                row["council_funding"] = json!(society.council_funding);
                row["civic_petitions"] = json!(h.governance.as_ref().map(|g| &g.petitions));
                row["administrations"] = json!(h.governance.as_ref().map(|g| &g.administrations));
                row["road_state"] = json!({
                    "routes": society.routes.len(),
                    "passable": society.routes.iter().filter(|r| r.passable()).count(),
                    "bricks": society.routes.iter().map(|r| r.road_bricks).sum::<f64>(),
                });
                row["household_fiscal"] = json!({
                    "family_recipient_households": accounts.iter().filter(|a| a.family_received>0.).count(),
                    "family_donor_households": accounts.iter().filter(|a| a.family_sent>0.).count(),
                    "cumulative_family_received": accounts.iter().map(|a| a.family_received).sum::<f64>(),
                    "cumulative_family_sent": accounts.iter().map(|a| a.family_sent).sum::<f64>(),
                    "cumulative_relief": accounts.iter().map(|a| a.relief).sum::<f64>(),
                    "cumulative_wages": accounts.iter().map(|a| a.wages).sum::<f64>(),
                    "cumulative_food_spending": accounts.iter().map(|a| a.food_spending).sum::<f64>(),
                    "wallet_cash": accounts.iter().map(|a| a.cash).sum::<f64>(),
                    "council_treasury": society.councils.iter().map(|c| c.treasury).sum::<f64>(),
                    "council_relief_paid": society.councils.iter().map(|c| c.relief_paid).sum::<f64>(),
                    "town_cash": h.sites.iter().map(|s| s.economy.finance[0] as f64).sum::<f64>(),
                });
                row["lesson_opportunities"] = json!(lesson_opportunities);
                row["lesson_observations"] = json!(lesson_observations);
                row["service_room"] = json!(service_room);
                row["service_counts"] = json!(service_counts);
                row["opening_relief_room"] = json!(opening_relief_room);
                row["opening_relief_dispatches"] = json!(opening_relief_dispatches);
                samples.push(row);
            }
        }
        let h = g.civilizations.as_ref().unwrap();
        let mut events = BTreeMap::<String, u64>::new();
        for e in &h.events {
            *events.entry(e.kind.clone()).or_default() += 1;
        }
        rows.push(json!({"seed":seed,"seconds":start.elapsed().as_secs_f64(),"samples":samples,"changed_identities":changes,"cancelled_actions":actions,"events":events,"travel":h.expeditions.as_ref().map(|x|json!({"voyages":x.voyages.len(),"active_people":h.person_duties.len(),"identified_at_recruitment":x.voyages.iter().flat_map(|e|&e.crew).filter(|c|c.identified_from_cohort).filter_map(|c|c.person).collect::<std::collections::BTreeSet<_>>().len(),"crew_person_ids":x.voyages.iter().flat_map(|e|&e.crew).filter_map(|c|c.person).collect::<std::collections::BTreeSet<_>>().len()})),"military":{"active_people":h.military.duties.len(),"people_ever_served":h.military.careers.len(),"service_months":h.military.careers.values().map(|c|c.months_served as u64).sum::<u64>(),"named_deaths":h.events.iter().filter(|e|e.kind=="military_deaths").flat_map(|e|&e.subjects).filter(|(k,_)|k=="person").count()},"resolution":h.resolution_report(),"population_reconciliation":h.population_reconciliation(),"residuals":h.economy_residuals()}));
        if let Some(parent) = args.output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut report = json!({"institution_funding_fields":["requested","conditional_ceiling","paid"],"institution_funding_unit":"abstract currency","operating_institutions":args.operating_institutions,"institution_work_classes":["election","upkeep","administration"],"institution_work_fields":["requested","granted","used"],"institution_work_unit":"worker-months","rotating_institutions":args.rotating_institutions,"construction_refinement":args.construction_refinement,"extraction_refinement":args.extraction_refinement,"agriculture_refinement":args.agriculture_refinement,"household_diagnostics":args.household_diagnostics,"resident_payroll":!args.legacy_resident_payroll,"individual_nutrition":!args.no_individual_nutrition,"household_mortality":!args.no_individual_nutrition && !args.no_household_mortality,"common_share_override":args.common_share,"observation_interval_months":1,"mortality_diagnostic_rows":["age_band_exposure","household_exposure"],"mortality_age_bands":["child","adult","elder"],"production_sectors":["farming","forestry","mining","construction"],"production_work_fields":["requested","granted","completed"],"production_work_unit":"worker-months","food_fields":["need","available","funded","eaten","physical_gap","access_gap"],"crop_yield_scale":args.crop_yield_scale,"founding_access":!args.no_founding_access,"aggregate_resolution":args.aggregate_resolution,"compare_resolution":args.compare_resolution,"workshop_refinement":args.workshop_refinement,"individual_demography":args.individual_demography,"resident_baseline":args.resident_baseline,"legacy_named_demography":args.legacy_named_demography,"no_domestic_care":args.no_domestic_care,"legacy_participation":args.legacy_participation,"strict_identities":args.strict_identities,"years":args.years,"resolution":args.resolution,"ecology_resolution":16,"epochs":1,"seeds":args.seeds,"gpu":gpu.adapter_name,"complete":rows.len()==args.seeds.len(),"runs":rows});
        report["funded_heritage_study"] = json!(args.funded_heritage_study);
        report["institution_working_core"] = json!(args.institution_working_core);
        report["institutional_students"] = json!(args.institutional_students);
        report["family_support"] = json!(args.family_support);
        report["political_distribution"] = json!(!args.fixed_distribution);
        report["household_relief_share_override"] = json!(args.household_relief_share);
        report["household_relief_target_override"] = json!(args.household_relief_target);
        report["lesson_opportunity_fields"] = json!([
            "present",
            "local_members",
            "present_source",
            "operational_source",
            "selected_source",
            "selected_source_with_faith"
        ]);
        report["lesson_opportunity_unit"] =
            json!("person-opportunities summed over opening site plans");
        report["service_classes"] = json!(["lesson", "heritage_study", "petition_hearing"]);
        report["service_fields"] =
            json!(["requested", "opening_space_granted", "work_backed", "used"]);
        report["service_room_unit"] = json!("room-months");
        report["service_counts_unit"] = json!("requests");
        report["opening_relief_room_unit"] = json!("room-months");
        report["essential_institution_work"] = json!(args.essential_institution_work);
        report["institution_funding_attempt_fields"] =
            json!(["positive_ceiling_requests", "collection_executed"]);
        report["named_institution_administration"] = json!(args.named_institution_administration);
        std::fs::write(&args.output, serde_json::to_vec_pretty(&report)?)?;
    }
    Ok(())
}
