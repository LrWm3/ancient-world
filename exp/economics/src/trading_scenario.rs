//! Controlled provider-count experiment; economic quantities are hundredths.
use crate::{
    activities::{Outcome, Target, WorkOrder},
    crafts::*,
    currency::Bid,
    exchange::{DEFAULT_CAPTURE_PERCENT, Market, ToolRequest},
    model::*,
    scenario::*,
};
use std::collections::BTreeSet;

pub const STOCK_UNIT: i32 = 100;
pub const TRADING_MONTHS: u32 = 72;
pub const MAX_PROVIDERS: usize = 8;
// Selected using examples/trading_audit; minimum one even if none is self-supporting.
pub const DEFAULT_PROVIDERS: usize = 3;
pub const EVALUATION_MONTHS: u32 = 24;
const FOOD_RESERVE: i32 = 6;
const MATERIAL_RESERVE: i32 = 2;
const PROVIDER_MATERIAL_TARGET: i32 = 4;
const COIN_TARGET: i32 = 4;
pub const STATE_BUY_TICKS: i32 = 75;
pub const STATE_SELL_TICKS: i32 = 150;
pub const STATE_FORWARD_TICKS: i32 = 50;
pub const LABOR_TICKS_PER_UNIT: i32 = 4;
pub const TOOL_SERVICE_DIVISOR: i32 = 2;
pub const HARVEST_OUTPUT_MULTIPLIER: u32 = 2;

pub fn providers(count: usize) -> Vec<AgentId> {
    (0..count)
        .map(|i| PERSON + 2 + PERSON_COUNT * i as u32)
        .collect()
}

pub fn scenario(count: usize, capture_percent: u32) -> Result<(World, State), String> {
    if !(1..=MAX_PROVIDERS).contains(&count) || capture_percent > 100 {
        return Err("expected 1..=8 providers and 0..=100 capture percent".into());
    }
    let (mut w, mut s) = population_scenario(SPECIALIZED_PERSON_COUNT)?;
    let makers: BTreeSet<_> = providers(count).into_iter().collect();
    let farmer_orders: Vec<_> = w
        .activities
        .orders
        .iter()
        .filter(|o| o.agent == PERSON)
        .cloned()
        .collect();
    // Reassign excess providers to a tested productive profile, retaining plots/needs.
    for agent in providers(MAX_PROVIDERS) {
        if !makers.contains(&agent) {
            w.activities.orders.retain(|o| o.agent != agent);
            for order in &farmer_orders {
                let mut o = order.clone();
                o.agent = agent;
                w.activities.orders.push(o);
            }
            w.agreements
                .iter_mut()
                .find(|a| a.debtor == agent)
                .unwrap()
                .payment
                .resource = GRAIN;
            let id = w.agreements.iter().find(|a| a.debtor == agent).unwrap().id;
            w.activities.coin_payments.insert(
                id,
                crate::activities::CoinPayment {
                    resource: TOKEN,
                    coins_per_unit: 1,
                },
            );
            w.issuance.push(crate::currency::Issuance {
                agreement: id,
                token: TOKEN,
                collected_per_token: 2,
            });
        }
    }
    let mut market = Market {
        capture_percent,
        ..Default::default()
    };
    let maker_ids: Vec<_> = makers.iter().copied().collect();
    let customers: Vec<_> = w
        .participants
        .iter()
        .map(|p| p.agent)
        .filter(|id| !makers.contains(id))
        .collect();
    for (index, &buyer) in customers.iter().enumerate() {
        let provider = maker_ids[index % count];
        // Every customer may select subsistence crops. Additional tools reflect work orders.
        let mut kinds = BTreeSet::from([HOE]);
        match (buyer - PERSON) % PERSON_COUNT {
            1 => {
                kinds.insert(PICK);
            }
            3 => {
                kinds.insert(COMB);
                kinds.insert(KNIFE);
            }
            _ => {
                kinds.insert(SPEAR);
            }
        }
        for kind in kinds {
            market.tools.push(ToolRequest {
                buyer,
                provider,
                kind,
            });
        }
    }
    w.activities.orders.retain(|o| {
        let makes_tool = matches!(w.activities.outcomes.get(&o.definition), Some(Outcome::Create(k)) if (HAMMER_STONE..=GRINDING_STONE).contains(k));
        let repairs_provider_tool = makers.contains(&o.agent) && matches!(w.activities.outcomes.get(&o.definition), Some(Outcome::Repair {kind, ..}) if *kind != HOUSE);
        !(makes_tool || repairs_provider_tool)
    });
    for &agent in &maker_ids {
        let kinds: BTreeSet<_> = market
            .tools
            .iter()
            .filter(|r| r.provider == agent)
            .map(|r| r.kind)
            .chain([HAMMER_STONE])
            .collect();
        for kind in kinds {
            w.activities.orders.push(WorkOrder {
                agent,
                definition: craft(kind),
                priority: 4,
                target: Target::Assets { kind, count: 1 },
            });
        }
        w.activities.orders.push(WorkOrder {
            agent,
            definition: repair(HAMMER_STONE),
            priority: 3,
            target: Target::Maintain {
                kind: HAMMER_STONE,
                below: 4,
            },
        });
    }

    // Scale stocks/money and physical storage together; labor, time and fulfillment
    // retain their units. Thus consuming 100 grain ticks still satisfies one need.
    let stock: BTreeSet<_> = w
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Stock)
        .map(|r| r.id)
        .collect();
    for ((_, resource), q) in &mut s.balances {
        if stock.contains(resource) {
            *q *= STOCK_UNIT;
        }
    }
    for d in &mut w.definitions {
        for stage in &mut d.stages {
            for a in &mut stage.entry_inputs {
                a.quantity *= STOCK_UNIT;
            }
        }
        for a in &mut d.outputs {
            if stock.contains(&a.resource) {
                a.quantity *= STOCK_UNIT;
            }
        }
    }
    for pool in &mut w.pools {
        pool.capacity *= STOCK_UNIT;
        pool.monthly_regeneration *= STOCK_UNIT;
    }
    for q in w.storage.capacities.values_mut() {
        *q *= STOCK_UNIT;
    }
    for a in &mut w.agreements {
        a.payment.quantity *= STOCK_UNIT;
    }
    for o in &mut w.activities.orders {
        if let Target::Stock(a) = &mut o.target {
            a.quantity *= STOCK_UNIT;
        }
    }
    // Stock-to-stock barter: providers replenish inputs with earned grain.
    for &agent in &maker_ids {
        for resource in [STONE, RAW_WOOD, COPPER, IRON] {
            let id = w.bids.len() as u32 + 1;
            w.bids.push(Bid {
                id,
                buyer: agent,
                goods: Amount::new(resource, STOCK_UNIT),
                payment: Amount::new(GRAIN, STOCK_UNIT),
            });
            market
                .targets
                .insert(id, PROVIDER_MATERIAL_TARGET * STOCK_UNIT);
        }
        market
            .reserves
            .insert((agent, GRAIN), FOOD_RESERVE * STOCK_UNIT);
        market
            .payment_caps
            .insert((agent, TOKEN), COIN_TARGET * STOCK_UNIT);
    }
    for &agent in &customers {
        for resource in [STONE, RAW_WOOD, COPPER, IRON] {
            market
                .reserves
                .insert((agent, resource), MATERIAL_RESERVE * STOCK_UNIT);
        }
    }
    // Only providers sell excess grain here, up to their cash target. State payment
    // is bounded by tokens actually minted from collected native grain taxes.
    let id = w.bids.len() as u32 + 1;
    w.bids.push(Bid {
        id,
        buyer: STATE_AGENT,
        goods: Amount::new(GRAIN, STOCK_UNIT),
        payment: Amount::new(TOKEN, STOCK_UNIT),
    });
    market
        .targets
        .insert(id, w.storage.capacities[&STATE_AGENT]);
    w.market = Some(market);
    Ok((w, s))
}

pub fn default_scenario() -> Result<(World, State), String> {
    scenario(DEFAULT_PROVIDERS, DEFAULT_CAPTURE_PERCENT)
}

#[derive(Clone, Debug)]
pub struct Assessment {
    pub provider: AgentId,
    pub food_income: i64,
    pub material_spend: i64,
    pub food_and_tax_cost: i64,
    pub supported: bool,
}

/// A conservative realized-income screen, not a marginal-price equilibrium.
/// Nonfood royalties have no imputed food value unless actually exchanged.
pub fn assess(sim: &crate::simulation::Simulation, count: usize) -> Vec<Assessment> {
    let end = sim.state.month - 1;
    let start = end.saturating_sub(EVALUATION_MONTHS);
    providers(count)
        .into_iter()
        .map(|provider| {
            let food_income: i64 = sim
                .ledger
                .iter()
                .filter(|b| b.month > start)
                .flat_map(|b| &b.transactions)
                .filter_map(|t| t.royalty.as_ref())
                .filter(|r| r.provider == provider)
                .flat_map(|r| &r.amounts)
                .filter(|a| [GRAIN, WILD_FOOD, MILK, FISH].contains(&a.resource))
                .map(|a| i64::from(a.quantity))
                .sum();
            let material_spend: i64 = sim
                .ledger
                .iter()
                .filter(|b| b.month > start)
                .flat_map(|b| &b.transactions)
                .filter_map(|t| t.stock_trade.as_ref())
                .filter_map(|t| sim.world.bids.iter().find(|b| b.id == t.bid))
                .filter(|b| b.buyer == provider && b.payment.resource == GRAIN)
                .map(|b| i64::from(b.payment.quantity))
                .sum();
            let tax: i64 = sim
                .state
                .obligations
                .values()
                .filter(|o| o.due > start && o.due <= end)
                .filter(|o| {
                    sim.world
                        .agreements
                        .iter()
                        .any(|a| a.id == o.agreement && a.debtor == provider)
                })
                .map(|o| i64::from(o.owed))
                .sum();
            let food_and_tax_cost = i64::from(EVALUATION_MONTHS) * i64::from(STOCK_UNIT) + tax;
            let healthy = sim
                .reports
                .iter()
                .filter(|r| r.agent == provider && r.month > start)
                .all(|r| {
                    r.deficit(NUTRITION) == 0
                        && r.deficit(WARMTH) == 0
                        && r.terminal.is_none()
                        && r.obligations.values().all(|o| o.owed == o.paid)
                });
            let next_tax_funded = sim
                .world
                .agreements
                .iter()
                .filter(|a| a.debtor == provider)
                .all(|a| sim.state.balance(provider, a.payment.resource) >= a.payment.quantity);
            Assessment {
                provider,
                food_income,
                material_spend,
                food_and_tax_cost,
                supported: end >= EVALUATION_MONTHS
                    && food_income - material_spend >= food_and_tax_cost
                    && healthy
                    && next_tax_funded,
            }
        })
        .collect()
}

pub fn select_provider_count(results: &[(usize, bool)]) -> usize {
    results
        .iter()
        .filter(|(_, supported)| *supported)
        .map(|(count, _)| *count)
        .max()
        .unwrap_or(1)
}

/// Upfront coin exchange; retain the output-share scenario as a comparison.
pub fn cash_scenario(count: usize, advances: bool) -> Result<(World, State), String> {
    let (mut w, mut s) = scenario(count, DEFAULT_CAPTURE_PERCENT)?;
    tune_tools(&mut w, &mut s);
    let market = w.market.as_mut().unwrap();
    for bid in &mut w.bids {
        if bid.buyer != STATE_AGENT {
            bid.payment.resource = TOKEN;
        }
    }
    let commodities = [
        GRAIN, RAW_WOOD, STONE, CLAY, COPPER_ORE, IRON_ORE, SILVER_ORE, GOLD_ORE, COPPER, IRON,
        SILVER, GOLD, WOOL, MILK, FISH, HAY,
    ];
    let mut prices = std::collections::BTreeMap::new();
    // Spot valuation, state resale and forward purchase prices are distinct.
    for resource in commodities {
        market.seller_prices.insert(
            (STATE_AGENT, resource),
            crate::forward::Price {
                goods: STOCK_UNIT,
                coins: STATE_SELL_TICKS,
            },
        );
        prices.insert(
            resource,
            crate::forward::Price {
                goods: STOCK_UNIT,
                coins: STATE_BUY_TICKS,
            },
        );
        if !w
            .bids
            .iter()
            .any(|b| b.buyer == STATE_AGENT && b.goods.resource == resource)
        {
            let id = w.bids.len() as u32 + 1;
            w.bids.push(Bid {
                id,
                buyer: STATE_AGENT,
                goods: Amount::new(resource, STOCK_UNIT),
                payment: Amount::new(TOKEN, STOCK_UNIT),
            });
            market
                .targets
                .insert(id, w.storage.capacities[&STATE_AGENT]);
        }
        market.reserves.insert((STATE_AGENT, resource), 0);
    }
    for bid in &mut w.bids {
        if bid.buyer == STATE_AGENT {
            bid.payment.quantity = STATE_BUY_TICKS;
        }
    }
    for p in &w.participants {
        for resource in commodities {
            market.reserves.insert(
                (p.agent, resource),
                if resource == GRAIN {
                    FOOD_RESERVE
                } else {
                    MATERIAL_RESERVE
                } * STOCK_UNIT,
            );
        }
        market
            .payment_caps
            .insert((p.agent, TOKEN), COIN_TARGET * STOCK_UNIT);
        let id = w.bids.len() as u32 + 1;
        w.bids.push(Bid {
            id,
            buyer: p.agent,
            goods: Amount::new(GRAIN, STOCK_UNIT),
            payment: Amount::new(TOKEN, STOCK_UNIT),
        });
        market.targets.insert(id, FOOD_RESERVE * STOCK_UNIT);
    }
    market.cash = Some(crate::forward::Policy {
        lender: STATE_AGENT,
        coin: TOKEN,
        months: crate::forward::PROJECTION_MONTHS,
        enabled: advances,
        advance_prices: prices
            .keys()
            .map(|&r| {
                (
                    r,
                    crate::forward::Price {
                        goods: STOCK_UNIT,
                        coins: STATE_FORWARD_TICKS,
                    },
                )
            })
            .collect(),
        prices,
        protected: std::collections::BTreeMap::from([(GRAIN, FOOD_RESERVE * STOCK_UNIT)]),
    });
    Ok((w, s))
}

/// Quarter-unit labor precision; manual work is unchanged in physical units.
fn tune_tools(world: &mut World, state: &mut State) {
    let capacity: BTreeSet<_> = world
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Capacity)
        .map(|r| r.id)
        .collect();
    for p in &mut world.participants {
        p.capacity.quantity *= LABOR_TICKS_PER_UNIT;
    }
    for ((_, r), q) in &mut state.balances {
        if capacity.contains(r) {
            *q *= LABOR_TICKS_PER_UNIT;
        }
    }
    for q in world.capacity_overrides.values_mut() {
        *q *= LABOR_TICKS_PER_UNIT;
    }
    for d in &mut world.definitions {
        for stage in &mut d.stages {
            for service in &mut stage.monthly_services {
                service.quantity *= LABOR_TICKS_PER_UNIT;
            }
        }
    }
    for t in &mut world.techniques {
        if t.equipment_kind.is_some() && [GROW, HUSBANDRY].contains(&t.definition) {
            t.output_multiplier = HARVEST_OUTPUT_MULTIPLIER;
        }
        for service in &mut t.services {
            service.quantity *= LABOR_TICKS_PER_UNIT;
            if t.equipment_kind.is_some() {
                service.quantity = (service.quantity / TOOL_SERVICE_DIVISOR).max(1);
            }
        }
    }
}
