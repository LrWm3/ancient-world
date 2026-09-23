//! Editable initial catalog and deterministic scenario fixtures (abstract units).
use crate::{maintenance::ConditionRule, model::*};
use std::collections::BTreeMap;

pub const STATE_AGENT: AgentId = 0;
pub const PERSON: AgentId = 88;
pub const GRAIN: ResourceId = 1;
pub const SEED: ResourceId = 2;
pub const LABOR: ResourceId = 3;
pub const NUTRITION: ResourceId = 4;
pub const REPAIR_OUTPUT: ResourceId = 5;
pub const GROW: DefinitionId = 1;
pub const CONSUME: DefinitionId = 2;
pub const REPAIR: DefinitionId = 3;
pub const PLOT: AssetId = 412;
pub const SCENARIO_MONTHS: u32 = 9;
pub const REPEATED_MONTHS: u32 = 60;
const LONG_RIGHT_MONTHS: u32 = 240;
pub const RAW_WOOD: ResourceId = 6;
pub const FUEL: ResourceId = 7;
pub const WARMTH: ResourceId = 8;
pub const PREPARE_FUEL: DefinitionId = 4;
pub const USE_FUEL: DefinitionId = 5;

pub const LONG_SCENARIOS: &[&str] = &[
    "repeated-harvests",
    "repeated-no-seed",
    "repeated-short-horizon",
    "warmth-food-first",
    "warmth-first",
    "warmth-protect-active",
    "warmth-abundant-food-first",
    "warmth-abundant-warmth-first",
    "warmth-inactive",
    "warmth-no-wood",
];

pub fn scenario_months(name: &str) -> u32 {
    if name.starts_with("wood-market-") {
        return crate::pool_market::RUN_MONTHS;
    }
    if [
        "opportunity-farming",
        "opportunity-two-plots",
        "opportunity-one-plot",
    ]
    .contains(&name)
    {
        return crate::opportunities::RUN_MONTHS;
    }
    if name == "households-32" {
        return 24;
    }
    if [
        "trading-32",
        "trading-32-no-forward",
        "trading-32-share",
        "trading-32-plots",
        "trading-32-no-plots",
    ]
    .contains(&name)
    {
        return crate::trading_scenario::TRADING_MONTHS;
    }
    if crate::crafts::SCENARIOS.contains(&name) {
        return crate::crafts::ACTIVITY_MONTHS;
    }
    if MULTI_PERSON_SCENARIOS.contains(&name) || CURRENCY_SCENARIOS.contains(&name) {
        return REPEATED_MONTHS;
    }
    if LONG_SCENARIOS.contains(&name)
        || CONDITION_SCENARIOS.contains(&name)
        || PLANNING_SCENARIOS.contains(&name)
        || TOOL_SCENARIOS.contains(&name)
        || EXPERIENCE_SCENARIOS.contains(&name)
        || AGREEMENT_SCENARIOS.contains(&name)
        || ACCESS_SCENARIOS.contains(&name)
        || PAYMENT_SCENARIOS.contains(&name)
    {
        REPEATED_MONTHS
    } else {
        SCENARIO_MONTHS
    }
}

pub fn repeated() -> (World, State) {
    let (mut world, state) = baseline();
    world.rights[0].through = LONG_RIGHT_MONTHS;
    world
        .definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .outputs
        .push(Amount::new(SEED, 1));
    (world, state)
}

pub fn with_warmth(warmth_first: bool) -> (World, State) {
    let (mut world, mut state) = repeated();
    world.priority = Priority::NeedFirst;
    world.participants[0].needs[0].priority = u32::from(warmth_first);
    world.participants[0].needs.push(Requirement {
        resource: WARMTH,
        quantity: 1,
        priority: u32::from(!warmth_first),
    });
    world.resources.extend([
        Resource {
            id: RAW_WOOD,
            name: "raw wood".into(),
            kind: ResourceKind::Stock,
        },
        Resource {
            id: FUEL,
            name: "fuel".into(),
            kind: ResourceKind::Stock,
        },
        Resource {
            id: WARMTH,
            name: "warmth".into(),
            kind: ResourceKind::Fulfillment,
        },
    ]);
    state.balances.insert((PERSON, RAW_WOOD), 40);
    state.balances.insert((PERSON, FUEL), 1);
    world.definitions.extend([
        ProcessDefinition {
            id: PREPARE_FUEL,
            name: "prepare fuel".into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "prepare".into(),
                months: 1,
                entry_inputs: vec![Amount::new(RAW_WOOD, 1)],
                monthly_services: vec![Amount::new(LABOR, 1)],
            }],
            outputs: vec![Amount::new(FUEL, 2)],
        },
        ProcessDefinition {
            id: USE_FUEL,
            name: "use fuel for warmth".into(),
            execution: Execution::Consumption,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "burn".into(),
                months: 1,
                entry_inputs: vec![Amount::new(FUEL, 1)],
                monthly_services: vec![],
            }],
            outputs: vec![Amount::new(WARMTH, 1)],
        },
    ]);
    (world, state)
}

pub fn baseline() -> (World, State) {
    let world = World {
        work_choice: None,
        credit: None,
        marketplaces: vec![],
        negotiation: None,
        town_market: None,
        need_orders: None,
        pool_market: None,
        competition: None,
        open_access_offers: Default::default(),
        agent_search: Default::default(),
        transaction_policy: None,
        households: vec![],
        market: None,
        activities: Default::default(),
        storage: Default::default(),
        issuance: vec![],
        bids: vec![],
        pools: vec![],
        pool_inputs: vec![],
        offers: vec![],
        techniques: vec![],
        practice_rules: vec![],
        agreements: vec![],
        access_offers: vec![],
        decision_horizon: None,
        payment_policy: Default::default(),
        condition_rules: Vec::new(),
        agents: vec![
            Agent {
                id: STATE_AGENT,
                name: "State".into(),
            },
            Agent {
                id: PERSON,
                name: "Person 88".into(),
            },
        ],
        participants: vec![Participant {
            agent: PERSON,
            capacity: Amount::new(LABOR, 2),
            needs: vec![Requirement {
                resource: NUTRITION,
                quantity: 1,
                priority: 0,
            }],
        }],
        resources: vec![
            Resource {
                id: GRAIN,
                name: "grain".into(),
                kind: ResourceKind::Stock,
            },
            Resource {
                id: SEED,
                name: "seed".into(),
                kind: ResourceKind::Stock,
            },
            Resource {
                id: LABOR,
                name: "labor".into(),
                kind: ResourceKind::Capacity,
            },
            Resource {
                id: NUTRITION,
                name: "nutrition".into(),
                kind: ResourceKind::Fulfillment,
            },
            Resource {
                id: REPAIR_OUTPUT,
                name: "repair material".into(),
                kind: ResourceKind::Stock,
            },
        ],
        assets: vec![Asset {
            id: PLOT,
            owner: STATE_AGENT,
            kind: 1,
        }],
        rights: vec![UseRight {
            id: 1,
            holder: PERSON,
            asset: PLOT,
            from: 1,
            through: SCENARIO_MONTHS,
            output_owner: PERSON,
        }],
        definitions: vec![
            ProcessDefinition {
                id: GROW,
                name: "grow grain".into(),
                execution: Execution::Productive,
                enabled: true,
                asset_kind: Some(1),
                stages: vec![
                    Stage {
                        name: "planting".into(),
                        months: 1,
                        entry_inputs: vec![Amount::new(SEED, 1)],
                        monthly_services: vec![Amount::new(LABOR, 2)],
                    },
                    Stage {
                        name: "growth".into(),
                        months: 4,
                        entry_inputs: vec![],
                        monthly_services: vec![Amount::new(LABOR, 1)],
                    },
                    Stage {
                        name: "harvest".into(),
                        months: 1,
                        entry_inputs: vec![],
                        monthly_services: vec![Amount::new(LABOR, 2)],
                    },
                ],
                outputs: vec![Amount::new(GRAIN, 8)],
            },
            ProcessDefinition {
                id: CONSUME,
                name: "consume grain".into(),
                execution: Execution::Consumption,
                enabled: true,
                asset_kind: None,
                stages: vec![Stage {
                    name: "consume".into(),
                    months: 1,
                    entry_inputs: vec![Amount::new(GRAIN, 1)],
                    monthly_services: vec![],
                }],
                outputs: vec![Amount::new(NUTRITION, 1)],
            },
            ProcessDefinition {
                id: REPAIR,
                name: "make repair material".into(),
                execution: Execution::Productive,
                enabled: true,
                asset_kind: None,
                stages: vec![Stage {
                    name: "make".into(),
                    months: 1,
                    entry_inputs: vec![],
                    monthly_services: vec![Amount::new(LABOR, 1)],
                }],
                outputs: vec![Amount::new(REPAIR_OUTPUT, 1)],
            },
        ],
        horizon: 6,
        priority: Priority::ContinuingFirst,
        capacity_overrides: BTreeMap::new(),
        scheduled_starts: Vec::new(),
    };
    let state = State {
        credit: Default::default(),
        marketplaces: Default::default(),
        town_market: Default::default(),
        memberships: Default::default(),
        household_remainders: BTreeMap::new(),
        exchange: Default::default(),
        month: 1,
        phase: Phase::Open,
        next_batch: 0,
        balances: BTreeMap::from([((PERSON, GRAIN), 5), ((PERSON, SEED), 1)]),
        processes: BTreeMap::new(),
        conditions: BTreeMap::new(),
        terminal: BTreeMap::new(),
        equipment: BTreeMap::new(),
        practice: BTreeMap::new(),
        filled_offers: Default::default(),
        pending_production: None,
        obligations: BTreeMap::new(),
        accepted_agreements: BTreeMap::new(),
    };
    (world, state)
}

pub const EXPERIENCE_SCENARIOS: &[&str] =
    &["experience-manual", "experience-tool", "experience-no-seed"];

pub const AGREEMENT_SCENARIOS: &[&str] = &["annual-access", "annual-free", "annual-arrears"];

pub const ACCESS_SCENARIOS: &[&str] =
    &["offer-useful", "offer-no-seed", "offer-short", "offer-long"];

pub const PAYMENT_SCENARIOS: &[&str] = &[
    "payment-debt",
    "payment-protected",
    "payment-trap-debt",
    "payment-trap-protected",
];

pub fn named(name: &str) -> Result<(World, State), String> {
    if let Some(supply) = name.strip_prefix("wood-market-") {
        let (supply, policy) = supply
            .strip_suffix("-lottery")
            .map_or((supply, crate::allocation::Policy::PriorityLottery), |s| {
                (s, crate::allocation::Policy::Lottery)
            });
        return crate::pool_market::scenario(supply, policy);
    }
    if ["opportunity-two-plots", "opportunity-one-plot"].contains(&name) {
        return crate::competition::scenario(
            if name == "opportunity-two-plots" {
                2
            } else {
                1
            },
            crate::competition::DEFAULT_SEED,
        );
    }
    if name == "opportunity-farming" {
        return crate::membership::scenario();
    }
    if name == "households-32" {
        return crate::households::scenario();
    }
    if ["trading-32-plots", "trading-32-no-plots"].contains(&name) {
        return crate::plots::scenario(name == "trading-32-plots");
    }
    if name == "trading-32-share" {
        return crate::trading_scenario::default_scenario();
    }
    if ["trading-32", "trading-32-no-forward"].contains(&name) {
        return crate::trading_scenario::cash_scenario(
            crate::trading_scenario::DEFAULT_PROVIDERS,
            name == "trading-32",
        );
    }
    if crate::crafts::SCENARIOS.contains(&name) {
        return crate::crafts::population_scenario(if name == "specialized-32" {
            crate::crafts::SPECIALIZED_PERSON_COUNT
        } else {
            PERSON_COUNT
        });
    }
    if MULTI_PERSON_SCENARIOS.contains(&name) {
        return multi_person(name);
    }
    if CURRENCY_SCENARIOS.contains(&name) {
        return currency_scenario(name);
    }
    if FORAGING_SCENARIOS.contains(&name) {
        return foraging(name);
    }
    if PAYMENT_SCENARIOS.contains(&name) {
        let (mut world, mut state) = named("annual-access")?;
        if name.contains("trap") {
            state.month = 13;
            state.balances.insert((PERSON, GRAIN), 1);
            state.balances.insert((PERSON, FUEL), 6);
        }
        if name.ends_with("protected") {
            world.payment_policy = crate::commitments::PaymentPolicy::ProtectEssentials;
        }
        return Ok((world, state));
    }
    if ACCESS_SCENARIOS.contains(&name) {
        let (mut world, mut state) = named("annual-access")?;
        world.access_offers = std::mem::take(&mut world.agreements);
        if name == "offer-no-seed" {
            state.balances.insert((PERSON, SEED), 0);
        }
        if matches!(name, "offer-short" | "offer-long") {
            world.access_offers[0].payment.quantity = 8;
            let mut right = world.rights[0].clone();
            right.id += 1;
            let mut alternative = world.access_offers[0].clone();
            alternative.id = 2;
            alternative.right = right.id;
            alternative.payment.quantity = 1;
            world.rights.push(right);
            world.access_offers.push(alternative);
        }
        if name == "offer-long" {
            world.decision_horizon = Some(18);
        }
        return Ok((world, state));
    }
    if AGREEMENT_SCENARIOS.contains(&name) {
        let (mut world, mut state) = named(if name == "annual-arrears" {
            "tool-unavailable"
        } else {
            "forecast-harvest"
        })?;
        if name == "annual-arrears" {
            state.month += 7;
            for p in state.processes.values_mut() {
                p.start += 7;
                p.reserved_through += 7;
            }
            state.balances.insert((PERSON, GRAIN), 0);
        }
        if name != "annual-free" {
            world.agreements.push(crate::commitments::Agreement {
                id: 1,
                right: world.rights[0].id,
                creditor: STATE_AGENT,
                debtor: PERSON,
                activated: world.rights[0].from,
                payment: Amount::new(GRAIN, 1),
            });
        }
        return Ok((world, state));
    }
    if EXPERIENCE_SCENARIOS.contains(&name) {
        let (mut world, mut state) = named(if name == "experience-tool" {
            "tool-beneficial"
        } else {
            "forecast-harvest"
        })?;
        world.practice_rules.push(crate::equipment::PracticeRule {
            definition: GROW,
            stage: 2,
            competency: 1,
            points: 1,
        });
        world.techniques.push(crate::equipment::Technique {
            output_multiplier: 1,
            id: 2,
            definition: GROW,
            stage: 2,
            equipment_kind: None,
            competency: Some((1, 4)),
            wear: 0,
            services: vec![Amount::new(LABOR, 1)],
        });
        if name == "experience-no-seed" {
            state.balances.insert((PERSON, SEED), 0);
        }
        return Ok((world, state));
    }

    if TOOL_SCENARIOS.contains(&name) {
        let (mut world, mut state) = named(if name == "tool-food-risk" {
            "forecast-harvest"
        } else {
            "forecast-cold"
        })?;
        world.techniques.push(crate::equipment::Technique {
            output_multiplier: 1,
            id: 1,
            definition: GROW,
            stage: 2,
            equipment_kind: Some(2),
            competency: None,
            wear: 1,
            services: vec![Amount::new(LABOR, 1)],
        });
        state.equipment.insert(
            TOOL,
            crate::equipment::DurableAsset {
                attached_to: None,
                id: TOOL,
                owner: STATE_AGENT,
                kind: 2,
                remaining_uses: 6,
                last_used_month: None,
            },
        );
        world.offers.push(crate::equipment::Offer {
            id: 1,
            seller: STATE_AGENT,
            asset: TOOL,
            price: Amount::new(GRAIN, 3),
        });
        if matches!(
            name,
            "tool-unavailable" | "tool-exhausted" | "tool-lifetime"
        ) {
            state.balances.insert((PERSON, GRAIN), 5);
            state.balances.insert((PERSON, FUEL), 6);
            state
                .conditions
                .insert((PERSON, WARMTH), Default::default());
            world.participants[0].capacity.quantity = 2;
        }
        if matches!(name, "tool-beneficial" | "tool-no-offer") {
            state.balances.insert((PERSON, GRAIN), 4);
        }
        match name {
            "tool-no-offer" | "tool-unavailable" => world.offers.clear(),
            "tool-exhausted" => state.equipment.get_mut(&TOOL).unwrap().remaining_uses = 0,
            "tool-lifetime" => {
                world.offers.clear();
                let tool = state.equipment.get_mut(&TOOL).unwrap();
                tool.owner = PERSON;
                tool.remaining_uses = 1;
            }
            _ => {}
        }
        return Ok((world, state));
    }
    if PLANNING_SCENARIOS.contains(&name) {
        let (mut world, mut state) = if name.contains("resupply") {
            named("institution-recovery")?
        } else if name.contains("cold") {
            let (world, initial) = named("conditions-warmth-food-first")?;
            let mut prior = crate::simulation::Simulation::new(
                world,
                initial,
                crate::compute::Backend::Reference,
            )?;
            prior.run_months(5)?;
            let mut world = prior.world;
            let mut state = prior.state;
            // A controlled opening boundary: harvest due, abundant grain but no fuel,
            // severe cold stress. Base four yields two actual labor after impairment.
            world.participants[0].capacity.quantity = 4;
            state.balances.insert((PERSON, GRAIN), 20);
            state.balances.insert((PERSON, FUEL), 0);
            state.conditions.insert(
                (PERSON, WARMTH),
                crate::maintenance::Condition {
                    deprivation: 10,
                    adverse_months: 5,
                },
            );
            (world, state)
        } else {
            named("conditions-warmth-first")?
        };
        if name.contains("scarcity") {
            for resource in [GRAIN, SEED, FUEL, RAW_WOOD] {
                state.balances.insert((PERSON, resource), 0);
            }
        }
        if name.starts_with("forecast-") {
            world.priority = Priority::ConsequenceAware;
        }
        return Ok((world, state));
    }
    if CONDITION_SCENARIOS.contains(&name) {
        if name.starts_with("institution-") {
            return Ok(institution(name));
        }
        let base = name.strip_prefix("conditions-").unwrap();
        let (mut world, state) = named(base)?;
        add_condition_rules(&mut world, "dead");
        return Ok((world, state));
    }
    if LONG_SCENARIOS.contains(&name) {
        let (mut world, mut state) = if name.starts_with("warmth-") {
            with_warmth(matches!(
                name,
                "warmth-first" | "warmth-abundant-warmth-first" | "warmth-protect-active"
            ))
        } else {
            repeated()
        };
        match name {
            "repeated-no-seed" => {
                state.balances.insert((PERSON, SEED), 0);
            }
            "repeated-short-horizon" => world.horizon = 3,
            "warmth-abundant-food-first" | "warmth-abundant-warmth-first" => {
                world.participants[0].capacity.quantity = 3
            }
            "warmth-protect-active" => world.priority = Priority::ContinuingFirst,
            "warmth-inactive" => {
                world.participants[0]
                    .needs
                    .iter_mut()
                    .find(|n| n.resource == WARMTH)
                    .unwrap()
                    .quantity = 0
            }
            "warmth-no-wood" => {
                state.balances.insert((PERSON, RAW_WOOD), 0);
            }
            _ => {}
        }
        return Ok((world, state));
    }
    let (mut world, mut state) = baseline();
    match name {
        "baseline" => {}
        "short-food" => {
            state.balances.insert((PERSON, GRAIN), 2);
        }
        "no-seed" => {
            state.balances.insert((PERSON, SEED), 0);
        }
        "no-right" => world.rights.clear(),
        "short-right" => world.rights[0].through = 5,
        "missed-work" => {
            world.capacity_overrides.insert((3, PERSON), 0);
        }
        "no-need" => world.participants[0].needs[0].quantity = 0,
        "disabled" => world.definitions[0].enabled = false,
        "continuing-first" | "new-first" => {
            world.scheduled_starts.push(ScheduledStart {
                month: 6,
                agent: PERSON,
                definition: REPAIR,
            });
            if name == "new-first" {
                world.priority = Priority::NewFirst;
            }
        }
        _ => return Err(format!("unknown scenario: {name}")),
    }
    Ok((world, state))
}

/// Separate controls retain the earlier consequence-free scenarios unchanged.
pub const CONDITION_SCENARIOS: &[&str] = &[
    "conditions-warmth-food-first",
    "conditions-warmth-first",
    "conditions-warmth-no-wood",
    "conditions-repeated-no-seed",
    "institution-upkeep",
    "institution-recovery",
    "institution-supplied",
];
pub const INSTITUTION: AgentId = 90;
pub const UPKEEP_SUPPLY: ResourceId = 20;
pub const UPKEEP: ResourceId = 21;
pub const ADMINISTRATION: ResourceId = 22;
pub const SUPPLY_RESERVE: ResourceId = 23;

/// Illustrative catalog values, not empirical physiology or organizational calibration.
pub fn add_condition_rules(world: &mut World, terminal_state: &str) {
    world.condition_rules = world
        .participants
        .iter()
        .flat_map(|p| {
            p.needs.iter().map(move |n| ConditionRule {
                subject: p.agent,
                provision: n.resource,
                name: format!("deprivation of provision {}", n.resource),
                shortfall_cost: 2,
                recovery: 1,
                impaired_at: 4,
                retained_capacity_permille: 500,
                affected_capacity: p.capacity.resource,
                terminal_at: 12,
                terminal_state: terminal_state.into(),
            })
        })
        .collect();
}

/// An institution has the same participant/requirement components, with no participant
/// or farming-specific behavior. Supplies are consumed through an ordinary process.
fn institution(name: &str) -> (World, State) {
    let (mut world, mut state) = baseline();
    world.agents = vec![Agent {
        id: INSTITUTION,
        name: "Institution".into(),
    }];
    world.participants = vec![Participant {
        agent: INSTITUTION,
        capacity: Amount::new(ADMINISTRATION, 4),
        needs: vec![Requirement {
            resource: UPKEEP,
            quantity: 1,
            priority: 0,
        }],
    }];
    world.assets.clear();
    world.rights.clear();
    world.resources = vec![
        Resource {
            id: UPKEEP_SUPPLY,
            name: "upkeep supplies".into(),
            kind: ResourceKind::Stock,
        },
        Resource {
            id: UPKEEP,
            name: "upkeep".into(),
            kind: ResourceKind::Fulfillment,
        },
        Resource {
            id: ADMINISTRATION,
            name: "administration capacity".into(),
            kind: ResourceKind::Capacity,
        },
        Resource {
            id: SUPPLY_RESERVE,
            name: "supply reserve".into(),
            kind: ResourceKind::Stock,
        },
    ];
    world.definitions = vec![ProcessDefinition {
        id: 1,
        name: "perform upkeep".into(),
        execution: Execution::Consumption,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "upkeep".into(),
            months: 1,
            entry_inputs: vec![Amount::new(UPKEEP_SUPPLY, 1)],
            monthly_services: vec![],
        }],
        outputs: vec![Amount::new(UPKEEP, 1)],
    }];
    state.balances = BTreeMap::from([(
        (INSTITUTION, UPKEEP_SUPPLY),
        if name == "institution-supplied" {
            60
        } else {
            2
        },
    )]);
    if name == "institution-recovery" {
        // A dated, finite resupply fixture bypasses the planner, not settlement.
        state.balances.insert((INSTITUTION, SUPPLY_RESERVE), 60);
        world.definitions.push(ProcessDefinition {
            id: 2,
            name: "release supply reserve".into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "delivery".into(),
                months: 1,
                entry_inputs: vec![Amount::new(SUPPLY_RESERVE, 60)],
                monthly_services: vec![],
            }],
            outputs: vec![Amount::new(UPKEEP_SUPPLY, 60)],
        });
        world.definitions[1].stages[0].months = 5;
        state.processes.insert(
            1,
            ProcessInstance {
                id: 1,
                definition: 2,
                operator: INSTITUTION,
                beneficiary: INSTITUTION,
                goal: None,
                asset: None,
                right: None,
                start: 1,
                reserved_through: 5,
                stage: 0,
                elapsed: 0,
                status: Status::Active,
            },
        );
    }
    add_condition_rules(&mut world, "dissolved");
    (world, state)
}

/// Matched scenarios differ only in productive allocation policy.
pub const PLANNING_SCENARIOS: &[&str] = &[
    "forecast-harvest",
    "forecast-cold",
    "static-cold",
    "forecast-resupply",
    "forecast-scarcity",
    "static-scarcity",
];

/// One finite barter offer and an optional tool-assisted harvest stage.
pub const TOOL: AssetId = 500;
pub const TOOL_SCENARIOS: &[&str] = &[
    "tool-beneficial",
    "tool-no-offer",
    "tool-food-risk",
    "tool-unavailable",
    "tool-exhausted",
    "tool-lifetime",
];

pub const WILD_FOOD: ResourceId = 30;
pub const WILD_SUPPLY: ResourceId = 31;
pub const FORAGE: DefinitionId = 30;
pub const EAT_WILD: DefinitionId = 31;
const FORAGE_LABOR: i32 = 2;
const WILD_POOL_CAPACITY: i32 = 6;
const WILD_REGENERATION: i32 = 1;
pub const FORAGING_SCENARIOS: &[&str] = &[
    "forage-abundant",
    "forage-bridge",
    "forage-conflict",
    "forage-conflict-long",
    "forage-urgent",
    "forage-harvest",
];

pub fn foraging(name: &str) -> Result<(World, State), String> {
    let (mut world, mut state) = named("forecast-harvest")?;
    if name != "forage-abundant" {
        // Prepare a real active crop through ordinary settlement, then expose
        // the controlled opening shortage. No scheduled future rescue is used.
        let elapsed = if name == "forage-harvest" { 5 } else { 4 };
        let mut sim =
            crate::simulation::Simulation::new(world, state, crate::compute::Backend::Reference)?;
        sim.run_months(elapsed)?;
        world = sim.world;
        state = sim.state;
        state.balances.insert((PERSON, GRAIN), 0);
        state.balances.insert((PERSON, FUEL), 6);
        if name == "forage-bridge" {
            world.participants[0].capacity.quantity = 3;
        }
        if name == "forage-urgent" {
            let rule = world
                .condition_rules
                .iter_mut()
                .find(|r| r.provision == NUTRITION)
                .unwrap();
            rule.shortfall_cost = rule.terminal_at;
        }
    } else {
        state.balances.insert((PERSON, GRAIN), 80);
    }
    if name == "forage-conflict-long" {
        world.decision_horizon = Some(18);
    }
    world.resources.extend([
        Resource {
            id: WILD_FOOD,
            name: "wild food".into(),
            kind: ResourceKind::Stock,
        },
        Resource {
            id: WILD_SUPPLY,
            name: "harvestable wild supply".into(),
            kind: ResourceKind::Stock,
        },
    ]);
    world.definitions.extend([
        ProcessDefinition {
            id: FORAGE,
            name: "forage".into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "gather".into(),
                months: 1,
                entry_inputs: vec![Amount::new(WILD_SUPPLY, 1)],
                monthly_services: vec![Amount::new(LABOR, FORAGE_LABOR)],
            }],
            outputs: vec![Amount::new(WILD_FOOD, 1)],
        },
        ProcessDefinition {
            id: EAT_WILD,
            name: "eat wild food".into(),
            execution: Execution::Consumption,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "eat".into(),
                months: 1,
                entry_inputs: vec![Amount::new(WILD_FOOD, 1)],
                monthly_services: vec![],
            }],
            outputs: vec![Amount::new(NUTRITION, 1)],
        },
    ]);
    world.pools.push(crate::pools::Pool {
        account: (STATE_AGENT, WILD_SUPPLY),
        capacity: WILD_POOL_CAPACITY,
        monthly_regeneration: WILD_REGENERATION,
    });
    world.pool_inputs.push(crate::pools::PoolInput {
        definition: FORAGE,
        account: (STATE_AGENT, WILD_SUPPLY),
    });
    state
        .balances
        .insert((STATE_AGENT, WILD_SUPPLY), WILD_POOL_CAPACITY);
    Ok((world, state))
}

pub const TOKEN: ResourceId = 40;
const ANNUAL_GRAIN_TAX: i32 = 2;
const COLLECTED_GRAIN_PER_TOKEN: i32 = 2;
const PERSONAL_STORAGE: i32 = 15;
const TREASURY_STORAGE: i32 = 32;
pub const CURRENCY_SCENARIOS: &[&str] = &[
    "storage-tax-one",
    "storage-tax-two",
    "storage-issuance",
    "storage-exchange",
];

pub fn currency_scenario(name: &str) -> Result<(World, State), String> {
    let (mut world, mut state) = named("forage-abundant")?;
    state.balances.insert((PERSON, GRAIN), 5);
    let (annual, _) = named("annual-access")?;
    world.agreements = annual.agreements;
    world.agreements[0].payment.quantity = if name == "storage-tax-one" {
        1
    } else {
        ANNUAL_GRAIN_TAX
    };
    world.resources.push(Resource {
        id: TOKEN,
        name: "token".into(),
        kind: ResourceKind::Stock,
    });
    world.storage.weights =
        BTreeMap::from([(GRAIN, 1), (SEED, 1), (FUEL, 1), (WILD_FOOD, 1), (TOKEN, 0)]);
    world.storage.capacities =
        BTreeMap::from([(PERSON, PERSONAL_STORAGE), (STATE_AGENT, TREASURY_STORAGE)]);
    if matches!(name, "storage-issuance" | "storage-exchange") {
        world.issuance.push(crate::currency::Issuance {
            agreement: 1,
            token: TOKEN,
            collected_per_token: COLLECTED_GRAIN_PER_TOKEN,
        });
    }
    if name == "storage-exchange" {
        world.bids.push(crate::currency::Bid {
            id: 1,
            buyer: STATE_AGENT,
            goods: Amount::new(GRAIN, 1),
            payment: Amount::new(TOKEN, 1),
        });
    }
    Ok((world, state))
}

/// Four people total: the original plus three, with private stores and plots.
pub const MULTI_PERSON_SCENARIOS: &[&str] = &["four-person-exchange", "four-person-scaled"];
pub const PERSON_COUNT: u32 = 4;

pub fn multi_person(name: &str) -> Result<(World, State), String> {
    population(name == "four-person-scaled", PERSON_COUNT)
}

pub(crate) fn population(scaled: bool, count: u32) -> Result<(World, State), String> {
    let (mut world, mut state) = currency_scenario("storage-exchange")?;
    let person = world.participants[0].clone();
    let right = world.rights[0].clone();
    let agreement = world.agreements[0].clone();
    let rules = world.condition_rules.clone();
    let balances: Vec<_> = state
        .balances
        .iter()
        .filter(|((owner, _), _)| *owner == PERSON)
        .map(|((_, r), q)| (*r, *q))
        .collect();
    for offset in 1..count {
        let agent = PERSON + offset;
        world.agents.push(Agent {
            id: agent,
            name: format!("person {}", offset + 1),
        });
        let mut participant = person.clone();
        participant.agent = agent;
        world.participants.push(participant);
        world.assets.push(Asset {
            id: PLOT + offset,
            owner: STATE_AGENT,
            kind: 1,
        });
        let mut next_right = right.clone();
        next_right.id += offset;
        next_right.asset += offset;
        next_right.holder = agent;
        next_right.output_owner = agent;
        world.rights.push(next_right);
        let mut next_agreement = agreement.clone();
        next_agreement.id += offset;
        next_agreement.right += offset;
        next_agreement.debtor = agent;
        world.agreements.push(next_agreement);
        let mut issuance = world.issuance[0].clone();
        issuance.agreement += offset;
        world.issuance.push(issuance);
        for rule in &rules {
            let mut r = rule.clone();
            r.subject = agent;
            world.condition_rules.push(r);
        }
        for &(r, q) in &balances {
            state.balances.insert((agent, r), q);
        }
        world.storage.capacities.insert(agent, PERSONAL_STORAGE);
    }
    if scaled {
        world
            .storage
            .capacities
            .insert(STATE_AGENT, TREASURY_STORAGE * count as i32);
        for pool in &mut world.pools {
            pool.capacity *= count as i32;
            pool.monthly_regeneration *= count as i32;
            if let Some(q) = state.balances.get_mut(&pool.account) {
                *q *= count as i32;
            }
        }
    }
    Ok((world, state))
}
