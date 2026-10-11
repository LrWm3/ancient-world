//! A bounded mixed-organization acceptance fixture, not a new execution driver.
use crate::{
    activities::{Target, WorkOrder},
    agency::integration,
    compute::Backend,
    employment,
    financial_reporting::Audit,
    marketplace::Side,
    minting::{COIN, FIREWOOD, HOURS, ISSUER, METAL, MINT, NUTRITION, ORE, REFINE, SUPPLIER},
    model::*,
    opportunities::{Action, AgentType, PERSON_TYPE},
    simulation::Simulation,
};

pub const RUN_MONTHS: u32 = integration::RUN_MONTHS;

pub const WORKSHOP: AgentId = 810;
pub const SMITH: AgentId = 811;
pub const WORKSHOP_TYPE: AgentType = 20;
pub const WORKSHOP_EMPLOYMENT: u32 = 810;
const WORKSHOP_TERM_MONTHS: u32 = 4;
const REFINING_LOT: i32 = 2;
const SMITH_HOURS: i32 = 2;
const WAGE_PER_HOUR: i32 = 1;
const WORKSHOP_OPENING_COINS: i32 = 8;
const WORKSHOP_ORE: i32 = 8;
const WORKSHOP_STORAGE: i32 = 32;
pub const TOWN: AgentId = 820;
pub const TOWN_TYPE: AgentType = 21;
pub const TOWN_EMPLOYMENT: u32 = 820;
pub const MUNICIPAL_WORK: DefinitionId = 120;
const TOWN_OPENING_COINS: i32 = 10;
const MUNICIPAL_HOURS: i32 = 1;
const MUNICIPAL_OUTPUT: i32 = 1;
const MUNICIPAL_STOCK_TARGET: i32 = 14;
const TOWN_STORAGE: i32 = 32;

pub fn scenario() -> Result<(World, State), String> {
    let (mut w, mut s) = integration::commitment_scenario()?;
    workshop(&mut w, &mut s);
    municipality(&mut w, &mut s);
    Ok((w, s))
}

/// Supplied operating mandate, not yet company founding, ownership or governance.
fn workshop(w: &mut World, s: &mut State) {
    for (id, name, hours, kind) in [
        (WORKSHOP, "metal workshop", 0, WORKSHOP_TYPE),
        (SMITH, "employed smith", SMITH_HOURS, PERSON_TYPE),
    ] {
        w.agents.push(Agent {
            id,
            name: name.into(),
        });
        w.participants.push(Participant {
            agent: id,
            capacity: Amount::new(HOURS, hours),
            needs: vec![],
        });
        w.storage.capacities.insert(id, WORKSHOP_STORAGE);
        w.transaction_policy
            .as_mut()
            .unwrap()
            .agent_types
            .insert(id, kind);
    }
    w.resources.push(Resource {
        id: ORE,
        name: "finite workshop ore".into(),
        kind: ResourceKind::Stock,
    });
    w.storage.weights.insert(ORE, 1);
    s.balances.remove(&(SUPPLIER, METAL));
    s.balances.insert((WORKSHOP, ORE), WORKSHOP_ORE);
    s.balances.insert((WORKSHOP, COIN), WORKSHOP_OPENING_COINS);
    w.definitions.push(ProcessDefinition {
        id: REFINE,
        name: "refine workshop ore".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "refine".into(),
            months: 1,
            entry_inputs: vec![Amount::new(ORE, REFINING_LOT)],
            monthly_services: vec![Amount::new(HOURS, SMITH_HOURS)],
        }],
        outputs: vec![Amount::new(METAL, REFINING_LOT)],
    });
    w.activities.orders.push(WorkOrder {
        agent: WORKSHOP,
        definition: REFINE,
        priority: 0,
        target: Target::Stock(Amount::new(METAL, REFINING_LOT)),
    });
    w.employment.push(employment::Terms {
        id: WORKSHOP_EMPLOYMENT,
        employer: WORKSHOP,
        worker: SMITH,
        from: 1,
        through: WORKSHOP_TERM_MONTHS,
        capacity: Amount::new(HOURS, SMITH_HOURS),
        wage_per_unit: Amount::new(COIN, WAGE_PER_HOUR),
        on_arrears: employment::ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    w.transaction_policy.as_mut().unwrap().permissions.extend([
        (WORKSHOP_TYPE, Action::StockTrade),
        (WORKSHOP_TYPE, Action::CapacityTrade),
        (WORKSHOP_TYPE, Action::Process(REFINE)),
    ]);
    w.marketplaces[0].allowed_types.insert(WORKSHOP_TYPE);
    for q in &mut w
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes
    {
        if q.market == METAL && q.side == Side::Sell {
            q.agent = WORKSHOP;
        }
    }
}

/// An operating municipal treasury with a supplied mandate; no elected town yet.
fn municipality(w: &mut World, s: &mut State) {
    w.agents.push(Agent {
        id: TOWN,
        name: "municipal treasury".into(),
    });
    w.participants.push(Participant {
        agent: TOWN,
        capacity: Amount::new(HOURS, 0),
        needs: vec![],
    });
    w.storage.capacities.insert(TOWN, TOWN_STORAGE);
    s.balances.insert((TOWN, COIN), TOWN_OPENING_COINS);
    w.assets
        .iter_mut()
        .find(|a| a.id == integration::PLOT)
        .unwrap()
        .owner = TOWN;
    w.agreements
        .iter_mut()
        .find(|a| a.id == 77)
        .unwrap()
        .creditor = TOWN;
    let law = w.transaction_policy.as_mut().unwrap();
    law.agent_types.insert(TOWN, TOWN_TYPE);
    law.permissions.extend([
        (TOWN_TYPE, Action::CapacityTrade),
        (TOWN_TYPE, Action::Process(MUNICIPAL_WORK)),
    ]);
    w.definitions.push(ProcessDefinition {
        id: MUNICIPAL_WORK,
        name: "collect municipal fuel reserve".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "collect".into(),
            months: 1,
            entry_inputs: vec![],
            monthly_services: vec![Amount::new(HOURS, MUNICIPAL_HOURS)],
        }],
        outputs: vec![Amount::new(FIREWOOD, MUNICIPAL_OUTPUT)],
    });
    w.activities.orders.push(WorkOrder {
        agent: TOWN,
        definition: MUNICIPAL_WORK,
        priority: 0,
        target: Target::Stock(Amount::new(FIREWOOD, MUNICIPAL_STOCK_TARGET)),
    });
    w.employment.push(employment::Terms {
        id: TOWN_EMPLOYMENT,
        employer: TOWN,
        worker: SMITH,
        from: 1,
        through: RUN_MONTHS,
        capacity: Amount::new(HOURS, MUNICIPAL_HOURS),
        wage_per_unit: Amount::new(COIN, WAGE_PER_HOUR),
        on_arrears: employment::ArrearsPolicy::SuspendDelivery,
        rank: 1,
    });
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evidence {
    pub months: u32,
    pub people_food_deficit: i32,
    pub completed_mints: usize,
    pub household_delivered: i32,
    pub land_dues_paid: i32,
    pub issuance: i128,
    pub reconciled_agents: usize,
    pub workshop_batches: usize,
    pub workshop_wages_paid: i32,
    pub municipal_batches: usize,
    pub municipal_wages_paid: i32,
    pub municipal_cash: i32,
}

pub struct Run {
    pub sim: Simulation,
    pub audit: Audit,
    opening_coins: i128,
}

fn coins(s: &State) -> i128 {
    s.balances
        .iter()
        .filter(|((_, r), _)| *r == COIN)
        .map(|(_, q)| i128::from(*q))
        .sum()
}

impl Run {
    pub fn new(w: World, s: State, backend: Backend) -> Result<Self, String> {
        let audit = integration::audit(&w, &s)?;
        let opening_coins = coins(&s);
        Ok(Self {
            sim: Simulation::new(w, s, backend)?,
            audit,
            opening_coins,
        })
    }

    pub fn through(&mut self, month: u32) -> Result<(), String> {
        while self.sim.state.month <= month {
            self.audit.step(&mut self.sim)?;
        }
        Ok(())
    }

    /// Reconcile actual completed boundaries; labels and forecasts are not evidence.
    pub fn evidence(&self) -> Result<Evidence, String> {
        let sim = &self.sim;
        if sim.state.phase != Phase::Open || sim.state.month <= 1 {
            return Err("acceptance evidence requires a completed month".into());
        }
        let months = sim.state.month - 1;
        for agent in &sim.world.agents {
            let statement = self.audit.book().statements(agent.id, 1, months)?;
            if statement.assets != statement.liabilities + statement.equity
                || statement.closing_cash != i128::from(sim.state.balance(agent.id, COIN))
            {
                return Err(format!("unreconciled agent {}", agent.id));
            }
        }
        let issuance = self
            .audit
            .book()
            .statements(ISSUER, 1, months)?
            .issuance_change;
        if coins(&sim.state) - self.opening_coins != issuance {
            return Err("coin supply does not reconcile to issuance".into());
        }
        Ok(Evidence {
            months,
            people_food_deficit: sim.reports.iter().map(|r| r.deficit(NUTRITION)).sum(),
            completed_mints: sim
                .state
                .processes
                .values()
                .filter(|p| p.definition == MINT && p.status == Status::Completed)
                .count(),
            household_delivered: sim
                .state
                .exchange
                .forwards
                .get(&20)
                .map_or(0, |f| f.delivered),
            land_dues_paid: sim.state.obligations.get(&(77, 13)).map_or(0, |c| c.paid),
            issuance,
            reconciled_agents: sim.world.agents.len(),
            municipal_batches: sim
                .state
                .processes
                .values()
                .filter(|p| {
                    p.operator == TOWN
                        && p.definition == MUNICIPAL_WORK
                        && p.status == Status::Completed
                })
                .count(),
            municipal_wages_paid: sim
                .state
                .employment
                .earned
                .iter()
                .filter(|((id, _), _)| *id == TOWN_EMPLOYMENT)
                .map(|(_, e)| e.claim.settled)
                .sum(),
            municipal_cash: sim.state.balance(TOWN, COIN),
            workshop_batches: sim
                .state
                .processes
                .values()
                .filter(|p| {
                    p.operator == WORKSHOP
                        && p.definition == REFINE
                        && p.status == Status::Completed
                })
                .count(),
            workshop_wages_paid: sim
                .state
                .employment
                .earned
                .iter()
                .filter(|((id, _), _)| *id == WORKSHOP_EMPLOYMENT)
                .map(|(_, e)| e.claim.settled)
                .sum(),
        })
    }
}
