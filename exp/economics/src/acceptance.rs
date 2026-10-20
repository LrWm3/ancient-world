//! A bounded mixed-organization acceptance fixture, not a new execution driver.
use crate::{
    activities::{Target, WorkOrder},
    agency::integration,
    compute::Backend,
    employment,
    financial_reporting::Audit,
    marketplace::Side,
    minting::{
        COIN, FIREWOOD, GATHER, HOURS, ISSUER, METAL, MINT, NUTRITION, ORE, REFINE, SUPPLIER,
        WHEAT, WORKER,
    },
    model::*,
    opportunities::{Action, AgentType, HOUSEHOLD_TYPE, PERSON_TYPE},
    simulation::Simulation,
};

pub mod buffers;
pub mod continuation;
pub mod expectations;
pub mod financing;
pub mod governed;
pub mod production_preferences;
pub mod publication;
pub mod sales;

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
const FOOD_PER_MONTH: i32 = 1;
const OPENING_WORKER_FOOD: i32 = 1;
const OPENING_SMITH_COINS: i32 = 1;
const FOOD_QUOTE_TARGET: i32 = 3;
const FOOD_LOT_PRICE: i32 = 3;
const SALE_PROTECTION_MONTHS: u32 = 1;
const ACCEPTANCE_HARVEST_WHEAT: i32 = 6;
const TIGHT_HARVEST_WHEAT: i32 = 4;
const ACCEPTANCE_HARVEST_HOURS: i32 = 4;
const COMMERCIAL_GRAIN_TARGET: i32 = 12;
const PRIVATE_FOOD_RESERVE: i32 = 2;
const BOUNDED_PRIVATE_FUEL_TARGET: i32 = 4;
const CROWDED_PRIVATE_FUEL_TARGET: i32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Baseline,
    TightFood,
    NoSalesTarget,
    NoPrivateAsks,
    CrowdedWood,
    NoOre,
    NoSmithHours,
    WorkshopExcluded,
    WorkshopProhibited,
    TownFirst,
    UnfundedWorkshop,
    UnfundedTown,
}
impl Case {
    pub const ALL: [Self; 12] = [
        Self::Baseline,
        Self::TightFood,
        Self::NoSalesTarget,
        Self::NoPrivateAsks,
        Self::CrowdedWood,
        Self::NoOre,
        Self::NoSmithHours,
        Self::WorkshopExcluded,
        Self::WorkshopProhibited,
        Self::TownFirst,
        Self::UnfundedWorkshop,
        Self::UnfundedTown,
    ];
}

pub fn scenario() -> Result<(World, State), String> {
    scenario_for(Case::Baseline)
}

pub fn scenario_for(case: Case) -> Result<(World, State), String> {
    let (mut w, mut s) = integration::commitment_scenario()?;
    workshop(&mut w, &mut s);
    municipality(&mut w, &mut s);
    worker_consumption(&mut w, &mut s);
    w.definitions
        .iter_mut()
        .find(|d| d.id == integration::GROW)
        .unwrap()
        .outputs
        .iter_mut()
        .find(|a| a.resource == WHEAT)
        .unwrap()
        .quantity = match case {
        Case::TightFood => TIGHT_HARVEST_WHEAT,
        _ => ACCEPTANCE_HARVEST_WHEAT,
    };
    let crop = w
        .definitions
        .iter_mut()
        .find(|d| d.id == integration::GROW)
        .unwrap();
    crop.name = "market-oriented plot work".into();
    crop.stages[0].monthly_services[0].quantity = ACCEPTANCE_HARVEST_HOURS;
    if case != Case::NoSalesTarget {
        w.activities.orders.push(WorkOrder {
            agent: integration::GROWER,
            definition: integration::GROW,
            priority: 0,
            target: Target::Stock(Amount::new(WHEAT, COMMERCIAL_GRAIN_TARGET)),
        });
    }
    w.activities
        .orders
        .iter_mut()
        .find(|o| o.agent == SUPPLIER && o.definition == GATHER)
        .unwrap()
        .target = Target::Stock(Amount::new(FIREWOOD, BOUNDED_PRIVATE_FUEL_TARGET));
    match case {
        Case::NoPrivateAsks => w
            .minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap()
            .quotes
            .retain(|q| ![integration::HOUSEHOLD, integration::GROWER].contains(&q.agent)),
        Case::CrowdedWood => {
            w.activities
                .orders
                .iter_mut()
                .find(|o| o.agent == SUPPLIER && o.definition == GATHER)
                .unwrap()
                .target = Target::Stock(Amount::new(FIREWOOD, CROWDED_PRIVATE_FUEL_TARGET))
        }
        Case::NoOre => {
            s.balances.insert((WORKSHOP, ORE), 0);
        }
        Case::NoSmithHours => {
            w.participants
                .iter_mut()
                .find(|p| p.agent == SMITH)
                .unwrap()
                .capacity
                .quantity = 0
        }
        Case::WorkshopExcluded => {
            w.marketplaces[0].allowed_types.remove(&WORKSHOP_TYPE);
        }
        Case::WorkshopProhibited => {
            w.transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(WORKSHOP_TYPE, Action::Process(REFINE)));
        }
        Case::TownFirst => {
            w.employment
                .iter_mut()
                .find(|t| t.id == WORKSHOP_EMPLOYMENT)
                .unwrap()
                .rank = 1;
            w.employment
                .iter_mut()
                .find(|t| t.id == TOWN_EMPLOYMENT)
                .unwrap()
                .rank = 0;
        }
        Case::UnfundedWorkshop => {
            s.balances.insert((WORKSHOP, COIN), 0);
        }
        Case::UnfundedTown => {
            s.balances.insert((TOWN, COIN), 0);
        }
        Case::Baseline | Case::TightFood | Case::NoSalesTarget => {}
    }
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

fn worker_consumption(w: &mut World, s: &mut State) {
    for id in [SMITH, WORKER] {
        w.participants
            .iter_mut()
            .find(|p| p.agent == id)
            .unwrap()
            .needs = vec![Requirement {
            resource: NUTRITION,
            quantity: FOOD_PER_MONTH,
            priority: 0,
        }];
        s.balances.insert((id, WHEAT), OPENING_WORKER_FOOD);
    }
    s.balances.insert((SMITH, COIN), OPENING_SMITH_COINS);
    w.marketplaces[0].allowed_types.insert(HOUSEHOLD_TYPE);
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.private_sales = Some(SALE_PROTECTION_MONTHS);
    p.public_sale = Some(crate::minting::orders::StockSales {
        reserve: 0,
        claim_months: SALE_PROTECTION_MONTHS,
    });
    p.quotes.extend([
        crate::minting::orders::Quote {
            agent: integration::GROWER,
            market: WHEAT,
            side: Side::Sell,
            limit: FOOD_LOT_PRICE,
            holding: PRIVATE_FOOD_RESERVE,
            max_lots: None,
        },
        crate::minting::orders::Quote {
            agent: SMITH,
            market: WHEAT,
            side: Side::Buy,
            limit: FOOD_LOT_PRICE,
            holding: FOOD_QUOTE_TARGET,
            max_lots: None,
        },
        crate::minting::orders::Quote {
            agent: integration::HOUSEHOLD,
            market: WHEAT,
            side: Side::Sell,
            limit: FOOD_LOT_PRICE,
            holding: 0,
            max_lots: None,
        },
    ]);
}

/// Only deals whose entire package actually committed count as exchange evidence.
pub fn accepted_deals(sim: &Simulation) -> impl Iterator<Item = &crate::minting::Deal> {
    sim.ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .flat_map(|b| {
            b.deals.iter().filter(|d| {
                b.receipts
                    .iter()
                    .any(|r| r.accepted && r.deals.contains(&d.id))
            })
        })
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
    pub smith_food_deficit: i32,
    pub household_food_sales: usize,
    pub smith_food_purchases: usize,
    pub workshop_metal_sales: usize,
    pub household_on_time_delivery: i32,
    pub state_loan_outstanding: Option<i32>,
    pub state_loan_repaid: bool,
    pub grower_food_sales: usize,
    pub workshop_net_income: i128,
    pub wage_arrears: i32,
    pub grower_storage_rejections: usize,
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
        if s.month != 1 || s.phase != Phase::Open {
            return Err(
                "acceptance runner needs its month-one opening; preserve Run when restarting"
                    .into(),
            );
        }
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
            grower_food_sales: accepted_deals(sim)
                .filter(|d| d.seller == integration::GROWER && d.market == WHEAT)
                .count(),
            workshop_net_income: self
                .audit
                .book()
                .statements(WORKSHOP, 1, months)?
                .net_income,
            wage_arrears: sim
                .state
                .employment
                .earned
                .values()
                .map(|e| e.claim.outstanding())
                .sum(),
            grower_storage_rejections: sim
                .ledger
                .iter()
                .flat_map(|b| &b.receipts)
                .filter(|r| {
                    r.agent == integration::GROWER
                        && r.definition == Some(integration::GROW)
                        && r.reason == Reason::InsufficientStorage
                })
                .count(),
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
            smith_food_deficit: sim
                .reports
                .iter()
                .filter(|r| r.agent == SMITH)
                .map(|r| r.deficit(NUTRITION))
                .sum(),
            household_food_sales: accepted_deals(sim)
                .filter(|d| d.seller == integration::HOUSEHOLD && d.market == WHEAT)
                .count(),
            smith_food_purchases: accepted_deals(sim)
                .filter(|d| d.buyer == SMITH && d.market == WHEAT)
                .count(),
            workshop_metal_sales: accepted_deals(sim)
                .filter(|d| d.seller == WORKSHOP && d.buyer == ISSUER && d.market == METAL)
                .count(),
            household_on_time_delivery: sim
                .ledger
                .iter()
                .filter(|b| b.month <= 4)
                .flat_map(|b| &b.transactions)
                .filter_map(|t| match t.forward {
                    Some(crate::forward::Event::Delivery {
                        contract: 20,
                        quantity,
                    }) => Some(quantity),
                    _ => None,
                })
                .sum(),
            state_loan_repaid: sim
                .state
                .credit
                .loans
                .get(&10)
                .is_some_and(|l| l.status == crate::credit::Status::Repaid),
            state_loan_outstanding: sim.state.credit.loans.get(&10).map(|l| l.principal),
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
