//! Finite productive capital in the voluntary-employment economy.
use super::{Run, WORKSHOP, WORKSHOP_TYPE, voluntary_employment};
use crate::{
    activities::{DurableKind, Outcome},
    agency::{
        self,
        objectives::{Metric, Objective, Scope},
    },
    compute::Backend,
    equipment::DurableAsset,
    minting::{HOURS, ORE, REFINE},
    model::*,
    opportunities::Action,
};
pub const REPAIR: DefinitionId = 130;
pub const MACHINE_KIND: u32 = 130;
pub const MACHINE: AssetId = 130;
pub const LIFETIME: u32 = 4;
const CONDITION_RESERVE: u32 = 1;
const REPAIR_ORE: i32 = 1;
const REPAIR_HOURS: i32 = 1;
const OPENING_MACHINE_COST: i128 = 4;
const FULL_CYCLES_PER_YEAR: u32 = super::continuation::CYCLE_MONTHS / (LIFETIME + 1);
const MIN_BATCHES_PER_YEAR: usize = (FULL_CYCLES_PER_YEAR * LIFETIME) as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Discovered,
    Supplied,
    NoAuthority,
    NoRepair,
    NoConditionGoal,
}

pub fn scenario(case: Case) -> Result<(World, State), String> {
    let (mut w, mut s) = voluntary_employment::scenario()?;
    w.activities.kinds.insert(
        MACHINE_KIND,
        DurableKind {
            name: "repairable refining equipment".into(),
            lifetime: LIFETIME,
            attached: false,
            monthly_decay: 0,
        },
    );
    w.activities.required.insert(REFINE, MACHINE_KIND);
    w.definitions.push(ProcessDefinition {
        id: REPAIR,
        name: "repair productive equipment".into(),
        execution: Execution::Productive,
        enabled: case != Case::NoRepair,
        asset_kind: None,
        stages: vec![Stage {
            name: "repair".into(),
            months: 1,
            entry_inputs: vec![Amount::new(ORE, REPAIR_ORE)],
            monthly_services: vec![Amount::new(HOURS, REPAIR_HOURS)],
        }],
        outputs: vec![],
    });
    w.activities.outcomes.insert(
        REPAIR,
        Outcome::Repair {
            kind: MACHINE_KIND,
            restore: LIFETIME,
        },
    );
    s.equipment.insert(
        MACHINE,
        DurableAsset {
            id: MACHINE,
            owner: WORKSHOP,
            kind: MACHINE_KIND,
            remaining_uses: LIFETIME,
            last_used_month: None,
            attached_to: None,
        },
    );
    let old = w.agency.remove(&WORKSHOP).unwrap();
    let mut agreement = old.operating_agreement().unwrap().clone();
    agreement.processes.insert(REPAIR);
    if case != Case::NoAuthority {
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((WORKSHOP_TYPE, Action::Process(REPAIR)));
    }
    let mut config = old.config;
    if case != Case::NoConditionGoal {
        for goals in std::iter::once(&mut config.objectives).chain(config.preferences.values_mut())
        {
            let index = goals
                .iter()
                .position(|o| matches!(o.metric, Metric::ProcessShortfall { .. }))
                .map_or(0, |i| i + 1);
            goals.insert(
                index,
                Objective {
                    scope: Scope::Organization,
                    metric: Metric::DurableReserve {
                        kind: MACHINE_KIND,
                        target: CONDITION_RESERVE,
                    },
                },
            );
        }
    }
    agency::organization::install_discovering_initial(
        &mut w,
        &s,
        agreement.clone(),
        &agreement.members,
        config,
    )?;
    if case == Case::Supplied {
        // Freeze the founding static menu; unlike request discovery, it does not
        // adapt its alternatives to later observed demand.
        let catalog = agency::discovery::programs(&w, WORKSHOP)?;
        let old = w.agency.remove(&WORKSHOP).unwrap();
        let mut config = old.config;
        config.programs = catalog;
        config.work_requests = agency::work_requests::Policy::Static;
        agency::organization::install_initial(
            &mut w,
            &s,
            agreement.clone(),
            &agreement.members,
            config,
        )?;
    }
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}

pub fn run(case: Case, backend: Backend) -> Result<Run, String> {
    let (w, s) = scenario(case)?;
    let mut opening = agency::integration::opening(&w, &s);
    opening.assets.insert(MACHINE, OPENING_MACHINE_COST);
    Run::with_opening(w, s, backend, opening)
}

/// Two full four-use/one-repair cycles fit each twelve-month operating window.
/// Retain the original economy's obligation gates; do not pretend downtime is output.
pub fn check(run: &Run) -> Result<super::continuation::Readiness, String> {
    use super::continuation::{self, CYCLE_MONTHS, MONTHS, Readiness};
    let mut failures = continuation::obligations(run)?;
    let periods = (1..MONTHS / CYCLE_MONTHS)
        .map(|c| continuation::period(run, c * CYCLE_MONTHS + 1, (c + 1) * CYCLE_MONTHS))
        .collect::<Result<Vec<_>, _>>()?;
    let mut mature_ore_consumed = 0_i64;
    let mut mature_ore_delivered = 0_i64;
    for p in &periods {
        let repairs = run
            .sim
            .state
            .processes
            .values()
            .filter(|v| {
                v.operator == WORKSHOP
                    && v.definition == REPAIR
                    && v.status == Status::Completed
                    && v.start >= p.from
                    && v.start <= p.through
            })
            .count();
        if repairs < FULL_CYCLES_PER_YEAR as usize
            || p.workshop_batches < MIN_BATCHES_PER_YEAR
            || p.metal_sales < MIN_BATCHES_PER_YEAR
            || p.municipal_batches < MIN_BATCHES_PER_YEAR
        {
            failures.push("missing recurring wear-repair-production-sales cycles".into());
        }
        if p.workshop_hours != (p.workshop_batches + repairs) as i32
            || p.workshop_wages != p.workshop_hours
            || p.municipal_hours != super::MUNICIPAL_HOURS * p.municipal_batches as i32
            || p.municipal_wages != p.municipal_hours
        {
            failures.push("paid labor differs from completed work".into());
        }
        let refining_ore: i32 = run
            .sim
            .world
            .definition(REFINE)
            .stages
            .iter()
            .flat_map(|s| &s.entry_inputs)
            .filter(|a| a.resource == ORE)
            .map(|a| a.quantity)
            .sum();
        let inputs =
            MIN_BATCHES_PER_YEAR as i32 * refining_ore + FULL_CYCLES_PER_YEAR as i32 * REPAIR_ORE;
        mature_ore_consumed += p.workshop_batches as i64 * i64::from(refining_ore)
            + repairs as i64 * i64::from(REPAIR_ORE);
        mature_ore_delivered += i64::from(p.ore_delivered);
        if p.ore_delivered < inputs {
            failures.push("replacement and repair materials not replenished".into());
        }
        if p.workshop_closing_cash
            < i128::from(
                continuation::ORE_PRICE + continuation::REFINING_HOURS * super::WAGE_PER_HOUR,
            )
            || p.municipal_closing_cash < i128::from(super::MUNICIPAL_HOURS * super::WAGE_PER_HOUR)
        {
            failures.push("next operating payment not covered".into());
        }
        if p.workshop_income <= 0
            || p.municipal_income <= 0
            || p.workshop_closing_cash < p.workshop_opening_cash
            || p.municipal_closing_cash < p.municipal_opening_cash
        {
            failures.push("maintenance economy consumes operating reserves".into());
        }
    }
    // Whole-lot purchases and five-month cycles straddle annual boundaries.
    // Across all mature years, actual use must be replaced without drawing down
    // the inventory accumulated during startup.
    if mature_ore_delivered < mature_ore_consumed {
        failures.push("mature operation draws down opening material inventory".into());
    }
    failures.sort();
    failures.dedup();
    Ok(Readiness { periods, failures })
}
