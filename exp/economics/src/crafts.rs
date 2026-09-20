//! Editable, deliberately abstract activity catalog and a provisioned integration fixture.
use crate::{
    activities::{CoinPayment, DurableKind, Outcome, Target, WorkOrder},
    equipment::Technique,
    model::*,
    pools::{Pool, PoolInput},
    scenario::*,
};

pub const STONE: ResourceId = 100;
pub const CLAY: ResourceId = 101;
pub const COPPER_ORE: ResourceId = 102;
pub const IRON_ORE: ResourceId = 103;
pub const SILVER_ORE: ResourceId = 104;
pub const GOLD_ORE: ResourceId = 105;
pub const COPPER: ResourceId = 106;
pub const IRON: ResourceId = 107;
pub const SILVER: ResourceId = 108;
pub const GOLD: ResourceId = 109;
pub const WOOL: ResourceId = 110;
pub const MILK: ResourceId = 111;
pub const FISH: ResourceId = 112;
pub const YOUNG_STOCK: ResourceId = 113;
pub const SHELTER_TICKET: ResourceId = 114;
pub const SHELTER: ResourceId = 115;
pub const HAY: ResourceId = 116;
pub const HAMMER_STONE: u32 = 1000;
pub const SOFT_HAMMER: u32 = 1001;
pub const PUNCH: u32 = 1002;
pub const KNIFE: u32 = 1003;
pub const STONE_STICK: u32 = 1004;
pub const SHOVEL: u32 = 1005;
pub const PICK: u32 = 1006;
pub const HOE: u32 = 1007;
pub const SPEAR: u32 = 1008;
pub const COMB: u32 = 1009;
pub const GRINDING_STONE: u32 = 1010;
pub const HOUSE: u32 = 1011;
pub const HERD: u32 = 1012;
pub const MINE_START: DefinitionId = 200;
pub const REFINE_START: DefinitionId = 210;
pub const CRAFT_START: DefinitionId = 300;
pub const REPAIR_START: DefinitionId = 400;
pub const BUILD_HOME: DefinitionId = 500;
pub const OCCUPY_HOME: DefinitionId = 501;
pub const RAISE_HERD: DefinitionId = 502;
pub const TEND_HERD: DefinitionId = 503;
pub const HUSBANDRY: DefinitionId = 504;
pub const FISHING: DefinitionId = 505;
pub const CUT_HAY: DefinitionId = 506;
pub const ACTIVITY_MONTHS: u32 = 36;
pub const SPECIALIZED_PERSON_COUNT: u32 = 32;
pub const SCENARIOS: &[&str] = &["specialized-activities", "specialized-32"];
const STATE_STORAGE_PER_GROUP: i32 = 512;
const DEPOSIT_RESOURCE_BASE: u32 = 2000;
const FISH_SUPPLY: ResourceId = 2100;
const GRASS_SUPPLY: ResourceId = 2101;
const TOOL_LIFETIME: u32 = 24;
const HOUSE_LIFETIME: u32 = 120;
const WORKER_LABOR: i32 = 6;

pub fn craft(kind: u32) -> DefinitionId {
    CRAFT_START + kind - HAMMER_STONE
}
pub fn repair(kind: u32) -> DefinitionId {
    REPAIR_START + kind - HAMMER_STONE
}
fn process(
    id: u32,
    name: &str,
    inputs: Vec<Amount>,
    outputs: Vec<Amount>,
    labor: i32,
    site: bool,
) -> ProcessDefinition {
    ProcessDefinition {
        id,
        name: name.into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: site.then_some(1),
        stages: vec![Stage {
            name: "work".into(),
            months: 1,
            entry_inputs: inputs,
            monthly_services: if labor > 0 {
                vec![Amount::new(LABOR, labor)]
            } else {
                vec![]
            },
        }],
        outputs,
    }
}
fn technique(world: &mut World, definition: u32, stage: usize, kind: u32, labor: i32) {
    world.techniques.push(Technique {
        output_multiplier: 1,
        id: 10000 + world.techniques.len() as u32,
        definition,
        stage,
        equipment_kind: Some(kind),
        competency: None,
        wear: 1,
        services: if labor > 0 {
            vec![Amount::new(LABOR, labor)]
        } else {
            vec![]
        },
    });
}
fn order(world: &mut World, agent: u32, definition: u32, priority: u32, target: Target) {
    world.activities.orders.push(WorkOrder {
        agent,
        definition,
        priority,
        target,
    });
}

pub fn catalog(world: &mut World, state: &mut State) {
    for (id, name) in [
        (STONE, "stone"),
        (CLAY, "clay"),
        (COPPER_ORE, "copper ore"),
        (IRON_ORE, "iron ore"),
        (SILVER_ORE, "silver ore"),
        (GOLD_ORE, "gold ore"),
        (COPPER, "copper"),
        (IRON, "iron"),
        (SILVER, "silver"),
        (GOLD, "gold"),
        (WOOL, "wool"),
        (MILK, "milk"),
        (FISH, "fish"),
        (YOUNG_STOCK, "young livestock"),
        (SHELTER_TICKET, "current-month shelter"),
        (SHELTER, "shelter satisfaction"),
        (HAY, "hay"),
    ] {
        world.resources.push(Resource {
            id,
            name: name.into(),
            kind: if id == SHELTER {
                ResourceKind::Fulfillment
            } else {
                ResourceKind::Stock
            },
        });
        if id != SHELTER && id != SHELTER_TICKET {
            world.storage.weights.insert(id, 1);
        }
    }
    world.activities.perishable.insert(SHELTER_TICKET);
    for (i, (resource, name)) in [
        (STONE, "quarry stone"),
        (CLAY, "dig clay"),
        (COPPER_ORE, "mine copper ore"),
        (IRON_ORE, "mine iron ore"),
        (SILVER_ORE, "mine silver ore"),
        (GOLD_ORE, "mine gold ore"),
    ]
    .into_iter()
    .enumerate()
    {
        let deposit = DEPOSIT_RESOURCE_BASE + i as u32;
        world.resources.push(Resource {
            id: deposit,
            name: format!("{name} deposit"),
            kind: ResourceKind::Stock,
        });
        world.pools.push(Pool {
            account: (STATE_AGENT, deposit),
            capacity: 120,
            monthly_regeneration: 0,
        });
        state.balances.insert((STATE_AGENT, deposit), 120);
        let id = MINE_START + i as u32;
        world.pool_inputs.push(PoolInput {
            definition: id,
            account: (STATE_AGENT, deposit),
        });
        world.definitions.push(process(
            id,
            name,
            vec![Amount::new(deposit, 1)],
            vec![Amount::new(resource, 2)],
            4,
            true,
        ));
        for (tool, cost) in [(STONE_STICK, 3), (SHOVEL, 2), (PICK, 1)] {
            technique(world, id, 0, tool, cost);
        }
    }
    for (i, (ore, metal, name)) in [
        (COPPER_ORE, COPPER, "refine copper"),
        (IRON_ORE, IRON, "refine iron"),
        (SILVER_ORE, SILVER, "refine silver"),
        (GOLD_ORE, GOLD, "refine gold"),
    ]
    .into_iter()
    .enumerate()
    {
        world.definitions.push(process(
            REFINE_START + i as u32,
            name,
            vec![Amount::new(ore, 1), Amount::new(FUEL, 1)],
            vec![Amount::new(metal, 1)],
            2,
            false,
        ));
    }
    let tools = [
        (HAMMER_STONE, "hammer stone", STONE),
        (SOFT_HAMMER, "soft hammer", RAW_WOOD),
        (PUNCH, "punch", COPPER),
        (KNIFE, "knife", STONE),
        (STONE_STICK, "stone and stick mining tool", STONE),
        (SHOVEL, "shovel", RAW_WOOD),
        (PICK, "pick", IRON),
        (HOE, "stone hoe", STONE),
        (SPEAR, "fishing spear", STONE),
        (COMB, "livestock comb", RAW_WOOD),
        (GRINDING_STONE, "grinding stone", STONE),
    ];
    for (kind, name, material) in tools {
        world.activities.kinds.insert(
            kind,
            DurableKind {
                name: name.into(),
                lifetime: TOOL_LIFETIME,
                attached: false,
                monthly_decay: 0,
            },
        );
        world.definitions.push(process(
            craft(kind),
            &format!("make {name}"),
            vec![Amount::new(material, 1), Amount::new(RAW_WOOD, 1)],
            vec![],
            3,
            false,
        ));
        world
            .activities
            .outcomes
            .insert(craft(kind), Outcome::Create(kind));
        world.definitions.push(process(
            repair(kind),
            &format!("repair {name}"),
            vec![],
            vec![],
            4,
            false,
        ));
        world
            .activities
            .outcomes
            .insert(repair(kind), Outcome::Repair { kind, restore: 12 });
        for tool in [HAMMER_STONE, SOFT_HAMMER, PUNCH, KNIFE] {
            technique(world, craft(kind), 0, tool, 1);
        }
        technique(world, repair(kind), 0, GRINDING_STONE, 1);
    }
    for stage in 0..world.definition(GROW).stages.len() {
        technique(world, GROW, stage, HOE, 1);
    }
    world.activities.kinds.insert(
        HOUSE,
        DurableKind {
            name: "home".into(),
            lifetime: HOUSE_LIFETIME,
            attached: true,
            monthly_decay: 0,
        },
    );
    let mut home = process(
        BUILD_HOME,
        "build home",
        vec![
            Amount::new(STONE, 2),
            Amount::new(CLAY, 2),
            Amount::new(RAW_WOOD, 4),
        ],
        vec![],
        3,
        true,
    );
    home.stages.push(Stage {
        name: "finish".into(),
        months: 1,
        entry_inputs: vec![],
        monthly_services: vec![Amount::new(LABOR, 3)],
    });
    world.definitions.push(home);
    world
        .activities
        .outcomes
        .insert(BUILD_HOME, Outcome::Create(HOUSE));
    for stage in 0..2 {
        for tool in [HAMMER_STONE, KNIFE, SHOVEL] {
            technique(world, BUILD_HOME, stage, tool, 2);
        }
    }
    world.definitions.push(process(
        repair(HOUSE),
        "maintain home",
        vec![Amount::new(CLAY, 1)],
        vec![],
        1,
        true,
    ));
    world.activities.outcomes.insert(
        repair(HOUSE),
        Outcome::Repair {
            kind: HOUSE,
            restore: 24,
        },
    );
    world.activities.shared_sites.insert(repair(HOUSE));
    world.definitions.push(process(
        OCCUPY_HOME,
        "occupy home",
        vec![],
        vec![Amount::new(SHELTER_TICKET, 1)],
        0,
        true,
    ));
    world.activities.required.insert(OCCUPY_HOME, HOUSE);
    world.activities.shared_sites.insert(OCCUPY_HOME);
    world.activities.kinds.insert(
        HERD,
        DurableKind {
            name: "livestock herd".into(),
            lifetime: 24,
            attached: true,
            monthly_decay: 1,
        },
    );
    world.definitions.push(process(
        RAISE_HERD,
        "raise livestock",
        vec![Amount::new(YOUNG_STOCK, 1), Amount::new(HAY, 2)],
        vec![],
        3,
        true,
    ));
    world
        .activities
        .outcomes
        .insert(RAISE_HERD, Outcome::Create(HERD));
    world.definitions.push(process(
        TEND_HERD,
        "tend livestock",
        vec![Amount::new(HAY, 1)],
        vec![],
        1,
        true,
    ));
    world.activities.outcomes.insert(
        TEND_HERD,
        Outcome::Repair {
            kind: HERD,
            restore: 12,
        },
    );
    world.definitions.push(process(
        HUSBANDRY,
        "feed and gather milk and wool",
        vec![Amount::new(HAY, 1)],
        vec![Amount::new(MILK, 2), Amount::new(WOOL, 1)],
        4,
        true,
    ));
    world.activities.required.insert(HUSBANDRY, HERD);
    technique(world, HUSBANDRY, 0, COMB, 2);
    for (id, source, name, output) in [
        (FISHING, FISH_SUPPLY, "fish", FISH),
        (CUT_HAY, GRASS_SUPPLY, "cut hay", HAY),
    ] {
        world.resources.push(Resource {
            id: source,
            name: format!("{name} supply"),
            kind: ResourceKind::Stock,
        });
        world.pools.push(Pool {
            account: (STATE_AGENT, source),
            capacity: 24,
            monthly_regeneration: 2,
        });
        state.balances.insert((STATE_AGENT, source), 24);
        world.pool_inputs.push(PoolInput {
            definition: id,
            account: (STATE_AGENT, source),
        });
        world.definitions.push(process(
            id,
            name,
            vec![Amount::new(source, 1)],
            vec![Amount::new(output, 2)],
            4,
            false,
        ));
    }
    technique(world, FISHING, 0, SPEAR, 2);
    technique(world, CUT_HAY, 0, KNIFE, 2);
    for (id, input, output, name) in [
        (600, MILK, NUTRITION, "drink milk"),
        (601, FISH, NUTRITION, "eat fish"),
        (602, SHELTER_TICKET, SHELTER, "use shelter"),
    ] {
        let mut d = process(
            id,
            name,
            vec![Amount::new(input, 1)],
            vec![Amount::new(output, 1)],
            0,
            false,
        );
        d.execution = Execution::Consumption;
        world.definitions.push(d);
    }
}

pub fn scenario() -> Result<(World, State), String> {
    population_scenario(PERSON_COUNT)
}

/// Repeat the four configured work profiles with proportional shared supplies.
pub fn population_scenario(count: u32) -> Result<(World, State), String> {
    if ![PERSON_COUNT, SPECIALIZED_PERSON_COUNT].contains(&count) {
        return Err("specialized fixtures support four or 32 people".into());
    }
    let groups = (count / PERSON_COUNT) as i32;
    let (mut world, mut state) = population(true, count)?;
    let existing_pools = world.pools.len();
    catalog(&mut world, &mut state);
    for pool in &mut world.pools[existing_pools..] {
        pool.capacity *= groups;
        pool.monthly_regeneration *= groups;
        *state.balances.get_mut(&pool.account).unwrap() *= groups;
    }
    world.priority = Priority::NeedFirst;
    world.bids.clear();
    world
        .storage
        .capacities
        .insert(STATE_AGENT, STATE_STORAGE_PER_GROUP * groups);
    let ids: Vec<_> = world.participants.iter().map(|p| p.agent).collect();
    for &agent in &ids {
        world
            .participants
            .iter_mut()
            .find(|p| p.agent == agent)
            .unwrap()
            .capacity
            .quantity = WORKER_LABOR;
        world
            .participants
            .iter_mut()
            .find(|p| p.agent == agent)
            .unwrap()
            .needs
            .push(Requirement {
                resource: SHELTER,
                quantity: 1,
                priority: 2,
            });
        let mut rule = world
            .condition_rules
            .iter()
            .find(|r| r.subject == agent && r.provision == WARMTH)
            .unwrap()
            .clone();
        rule.provision = SHELTER;
        rule.name = "exposure without shelter".into();
        world.condition_rules.push(rule);
        world.storage.capacities.insert(agent, 256);
        for (r, q) in [
            (GRAIN, 40),
            (FUEL, 40),
            (RAW_WOOD, 60),
            (STONE, 8),
            (CLAY, 8),
            (COPPER, 2),
            (IRON, 2),
            (TOKEN, 4),
            (HAY, 8),
            (YOUNG_STOCK, 1),
        ] {
            state.balances.insert((agent, r), q);
        }
        order(
            &mut world,
            agent,
            BUILD_HOME,
            3,
            Target::Assets {
                kind: HOUSE,
                count: 1,
            },
        );
        order(
            &mut world,
            agent,
            repair(HOUSE),
            1,
            Target::Maintain {
                kind: HOUSE,
                below: 24,
            },
        );
    }
    let taxes = [GRAIN, STONE, TOKEN, WOOL];
    world
        .issuance
        .retain(|r| (r.agreement - 1) % PERSON_COUNT == 0);
    for (index, a) in world.agreements.iter_mut().enumerate() {
        a.payment = Amount::new(taxes[index % PERSON_COUNT as usize], 2);
        if a.payment.resource != TOKEN {
            world.activities.coin_payments.insert(
                a.id,
                CoinPayment {
                    resource: TOKEN,
                    coins_per_unit: 1,
                },
            );
        }
    }
    for first in (PERSON..PERSON + count).step_by(PERSON_COUNT as usize) {
        order(
            &mut world,
            first,
            craft(HOE),
            4,
            Target::Assets {
                kind: HOE,
                count: 1,
            },
        );
        order(
            &mut world,
            first,
            repair(HOE),
            4,
            Target::Maintain {
                kind: HOE,
                below: 4,
            },
        );
        order(
            &mut world,
            first,
            GROW,
            10,
            Target::Stock(Amount::new(GRAIN, 60)),
        );
        order(
            &mut world,
            first,
            craft(SPEAR),
            5,
            Target::Assets {
                kind: SPEAR,
                count: 1,
            },
        );
        order(
            &mut world,
            first,
            FISHING,
            20,
            Target::Stock(Amount::new(FISH, 4)),
        );
        order(
            &mut world,
            first + 1,
            craft(PICK),
            4,
            Target::Assets {
                kind: PICK,
                count: 1,
            },
        );
        order(
            &mut world,
            first + 1,
            repair(PICK),
            4,
            Target::Maintain {
                kind: PICK,
                below: 4,
            },
        );
        for (i, r) in [STONE, CLAY, COPPER_ORE, IRON_ORE, SILVER_ORE, GOLD_ORE]
            .into_iter()
            .enumerate()
        {
            order(
                &mut world,
                first + 1,
                MINE_START + i as u32,
                10 + i as u32,
                Target::Stock(Amount::new(r, 12)),
            );
        }
        for (i, r) in [COPPER, IRON, SILVER, GOLD].into_iter().enumerate() {
            order(
                &mut world,
                first + 1,
                REFINE_START + i as u32,
                20 + i as u32,
                Target::Stock(Amount::new(r, 4)),
            );
        }
        for kind in HAMMER_STONE..=GRINDING_STONE {
            order(
                &mut world,
                first + 2,
                craft(kind),
                4,
                Target::Assets { kind, count: 1 },
            );
            order(
                &mut world,
                first + 2,
                repair(kind),
                5,
                Target::Maintain { kind, below: 4 },
            );
        }
        order(
            &mut world,
            first + 3,
            craft(COMB),
            4,
            Target::Assets {
                kind: COMB,
                count: 1,
            },
        );
        order(
            &mut world,
            first + 3,
            RAISE_HERD,
            5,
            Target::Assets {
                kind: HERD,
                count: 1,
            },
        );
        order(
            &mut world,
            first + 3,
            TEND_HERD,
            5,
            Target::Maintain {
                kind: HERD,
                below: 8,
            },
        );
        order(
            &mut world,
            first + 3,
            CUT_HAY,
            6,
            Target::Stock(Amount::new(HAY, 8)),
        );
        order(
            &mut world,
            first + 3,
            HUSBANDRY,
            10,
            Target::Stock(Amount::new(WOOL, 12)),
        );
    }
    Ok((world, state))
}
