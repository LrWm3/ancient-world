//! Controlled reciprocal-income fixture. Work targets remain supplied opportunities.
use crate::{
    activities::{Target, WorkOrder},
    household_governance::Policy,
    households::market::{EXAMPLE_HOUSEHOLD, scenario as household_market},
    marketplace::{Market, Side},
    model::*,
    opportunities::{Action, PERSON_TYPE},
    scenario::{FUEL, GRAIN, LABOR, PERSON, SEED, TOKEN, USE_FUEL, WARMTH},
    town_market::Listing,
};

pub const FUEL_MARKET: u32 = 2;
pub const COLLECT_FUEL: DefinitionId = 98;
pub const PRODUCE_GRAIN: DefinitionId = 99;
pub const RUN_MONTHS: u32 = 36;
const PRIVATE_FUEL_RESERVE: i32 = 2;
const COLLECTIVE_FUEL_TARGET: i32 = 1;
const SUPPORT_LIMIT: i32 = 1;
const BUYER: AgentId = 89;
const SECOND_PRODUCER: AgentId = 92;
const MEMBER_LABOR: i32 = 5;
const PRODUCER_LABOR: i32 = 1;
const FUEL_WORK: i32 = 6;
const FUEL_OUTPUT: i32 = 2;
const GRAIN_OUTPUT: i32 = 4;
const FUEL_TARGET: i32 = 24;
const GRAIN_TARGET: i32 = 8;
const STORAGE: i32 = 64;
const PRICE: i32 = 40;
const BUYER_COINS: i32 = 100;

pub fn costs() -> crate::process_accounting::Costs {
    use crate::process_accounting::{Costs, Output};
    Costs {
        output_weights: [(
            PRODUCE_GRAIN,
            [
                (Output::Stock(GRAIN), GRAIN_OUTPUT as u32),
                (Output::Stock(SEED), 1),
            ]
            .into(),
        )]
        .into(),
        ..Default::default()
    }
}

pub fn scenario() -> Result<(World, State), String> {
    let (mut w, mut s) = household_market()?;
    let (catalog, _) = crate::scenario::with_warmth(false);
    w.resources.extend(
        catalog
            .resources
            .into_iter()
            .filter(|r| [SEED, FUEL, WARMTH].contains(&r.id)),
    );
    w.definitions
        .extend(catalog.definitions.into_iter().filter(|d| d.id == USE_FUEL));
    for (id, name, inputs, labor, outputs) in [
        (
            COLLECT_FUEL,
            "collect fuel",
            vec![],
            FUEL_WORK,
            vec![Amount::new(FUEL, FUEL_OUTPUT)],
        ),
        (
            PRODUCE_GRAIN,
            "produce grain",
            vec![Amount::new(SEED, 1)],
            PRODUCER_LABOR,
            vec![Amount::new(GRAIN, GRAIN_OUTPUT), Amount::new(SEED, 1)],
        ),
    ] {
        w.definitions.push(ProcessDefinition {
            id,
            name: name.into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "work".into(),
                months: 1,
                entry_inputs: inputs,
                monthly_services: vec![Amount::new(LABOR, labor)],
            }],
            outputs,
        });
    }
    let law = w.transaction_policy.as_mut().unwrap();
    for id in [USE_FUEL, COLLECT_FUEL, PRODUCE_GRAIN] {
        law.permissions.insert((PERSON_TYPE, Action::Process(id)));
    }
    w.households[0]
        .governance
        .constitution
        .permitted_policies
        .insert(Policy::NeedsThenIncome);
    w.households[0].governance.charter.initial_policy = Policy::NeedsThenIncome;
    w.households[0].governance.constitution.activities = Some([COLLECT_FUEL].into());
    for p in &mut w.participants {
        p.capacity.quantity = if [BUYER, SECOND_PRODUCER].contains(&p.agent) {
            PRODUCER_LABOR
        } else {
            MEMBER_LABOR
        };
        w.storage.capacities.insert(p.agent, STORAGE);
        if p.agent == BUYER {
            p.needs.push(Requirement {
                resource: WARMTH,
                quantity: 1,
                priority: 1,
            });
        }
    }
    for id in [BUYER, SECOND_PRODUCER] {
        s.balances.insert((id, SEED), 1);
        w.activities.orders.push(WorkOrder {
            agent: id,
            definition: PRODUCE_GRAIN,
            priority: 0,
            target: Target::Stock(Amount::new(GRAIN, GRAIN_TARGET)),
        });
    }
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: COLLECT_FUEL,
        priority: 0,
        target: Target::Stock(Amount::new(FUEL, FUEL_TARGET)),
    });
    s.balances.insert((BUYER, TOKEN), BUYER_COINS);
    w.storage.weights.insert(FUEL, 1);
    w.storage.weights.insert(SEED, 1);
    let config = w.town_market.as_mut().unwrap();
    // Fixed crossed quotes isolate the labor/income policy from price learning.
    for entry in &mut config.traders {
        entry.trader.limit = PRICE;
        entry.trader.opening_quote = PRICE;
    }
    let mut traders = config.traders.clone();
    for entry in &mut traders {
        entry.side = if entry.trader.agent == EXAMPLE_HOUSEHOLD {
            Side::Sell
        } else {
            Side::Buy
        };
    }
    config.additional.push(Listing {
        market: FUEL_MARKET,
        traders,
        match_limit: None,
    });
    w.marketplaces
        .iter_mut()
        .find(|v| v.agent == config.venue)
        .unwrap()
        .markets
        .push(Market {
            id: FUEL_MARKET,
            goods: Amount::new(FUEL, 1),
            payment: TOKEN,
            price_tick: 1,
        });
    crate::settlement::validate_world(&w, &s)?;
    Ok((w, s))
}

/// The same opening stocks, work targets, quotes and productive technology, with
/// an explicit member offer of surplus to support collective food purchases.
pub fn coordinated() -> Result<(World, State), String> {
    let (mut w, s) = scenario()?;
    super::super::support::authorize(
        &mut w,
        &s,
        EXAMPLE_HOUSEHOLD,
        PERSON,
        super::super::support::Mandate {
            member: PERSON,
            resource: FUEL,
            from: s.month,
            through: u32::MAX,
            revoked_from: None,
            reserve_months: 1,
            private_reserve: PRIVATE_FUEL_RESERVE,
            household_target: COLLECTIVE_FUEL_TARGET,
            monthly_limit: SUPPORT_LIMIT,
        },
    )?;
    Ok((w, s))
}
