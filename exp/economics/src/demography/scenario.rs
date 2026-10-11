//! Accelerated, abstract population-pressure control, not demographic calibration.
use super::*;
use crate::{
    maintenance::ConditionRule,
    pools::{Pool, PoolInput},
    scenario::{GRAIN, LABOR, NUTRITION, PERSON, STATE_AGENT, TOKEN},
};
pub const MONTHS: u32 = 36;
pub const FIRST_CHILD: AgentId = 1000;
pub const FORAGE: DefinitionId = 900;
pub const WILD_FOOD: ResourceId = 901;
const INITIAL_FOOD: i32 = 6;
const STORE: i32 = 12;
const FOOD_PER_TRIP: i32 = 3;
const BIRTH_INTERVAL: u32 = 6;
const MATURITY: u32 = 6;
const MAX_BIRTHS: usize = 12;
const POOL_CAPACITY: i32 = 8;
const ABUNDANT_REGEN: i32 = 8;
const SCARCE_REGEN: i32 = 1;

pub fn scenario(reproduce: bool, scarce: bool) -> (World, State) {
    let (mut w, mut s) = crate::scenario::baseline();
    w.assets.clear();
    w.rights.clear();
    w.definitions
        .retain(|d| d.execution == Execution::Consumption);
    s.balances.clear();
    w.horizon = 2;
    w.resources.extend([
        Resource {
            id: WILD_FOOD,
            name: "wild food source".into(),
            kind: ResourceKind::Stock,
        },
        Resource {
            id: TOKEN,
            name: "coin".into(),
            kind: ResourceKind::Stock,
        },
    ]);
    let person = w.participants[0].clone();
    w.agents.push(Agent {
        id: PERSON + 1,
        name: "second parent".into(),
    });
    w.participants.push(Participant {
        agent: PERSON + 1,
        ..person
    });
    let rule = ConditionRule {
        subject: PERSON,
        provision: NUTRITION,
        name: "hunger".into(),
        shortfall_cost: 1,
        recovery: 1,
        impaired_at: 3,
        retained_capacity_permille: 1000,
        affected_capacity: LABOR,
        terminal_at: 6,
        terminal_state: "dead".into(),
    };
    for p in &mut w.participants {
        p.capacity.quantity = 1;
        s.balances.insert((p.agent, GRAIN), INITIAL_FOOD);
        w.storage.capacities.insert(p.agent, STORE);
        w.condition_rules.push(ConditionRule {
            subject: p.agent,
            ..rule.clone()
        });
    }
    w.storage.weights.insert(GRAIN, 1);
    w.pools.push(Pool {
        account: (STATE_AGENT, WILD_FOOD),
        capacity: POOL_CAPACITY,
        monthly_regeneration: if scarce { SCARCE_REGEN } else { ABUNDANT_REGEN },
    });
    w.pool_inputs.push(PoolInput {
        definition: FORAGE,
        account: (STATE_AGENT, WILD_FOOD),
    });
    w.definitions.push(ProcessDefinition {
        id: FORAGE,
        name: "forage food".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "gather".into(),
            months: 1,
            entry_inputs: vec![Amount::new(WILD_FOOD, 1)],
            monthly_services: vec![Amount::new(LABOR, 1)],
        }],
        outputs: vec![Amount::new(GRAIN, FOOD_PER_TRIP)],
    });
    w.demography = Some(Config {
        enabled: reproduce,
        plan_support: false,
        parents: [PERSON, PERSON + 1].into(),
        descendants_reproduce: true,
        first_month: 1,
        interval_months: BIRTH_INTERVAL,
        maturity_months: MATURITY,
        max_births: MAX_BIRTHS,
        first_id: FIRST_CHILD,
        person: Template {
            capacity: Amount::new(LABOR, 1),
            needs: w.participants[0].needs.clone(),
            conditions: vec![rule],
            storage: STORE,
        },
        support: Amount::new(GRAIN, 1),
        birth_buffer_months: 2,
        parent_reserve: 1,
    });
    (w, s)
}
