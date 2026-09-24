//! Physical issuance pilot: scripted or generated dated exchanges, paid capacity delegation,
//! then ordinary production. No incoming payment finances its own acquisition batch.
use crate::{
    acquisition::Resources,
    finance::Transfer,
    marketplace,
    model::*,
    opportunities::{self, Action},
};
use std::collections::{BTreeMap, BTreeSet};

pub mod orders;
pub mod provisioning;

pub const ISSUER: AgentId = 0;
pub const SUPPLIER: AgentId = 88;
pub const WORKER: AgentId = 91;
pub const VENUE: AgentId = 500;
pub const COIN: ResourceId = 101;
pub const WHEAT: ResourceId = 102;
pub const METAL: ResourceId = 103;
pub const HOURS: ResourceId = 104;
pub const FIREWOOD: ResourceId = 105;
pub const MINT: DefinitionId = 101;
pub const GATHER: DefinitionId = 102;
const SALE_MONTH: u32 = 1;
const MINT_MONTH: u32 = 2;
const OPENING_COINS: i32 = 6;
const WHEAT_LOT: i32 = 3;
const METAL_LOT: i32 = 2;
const LABOR_LOT: i32 = 2;
const WAGE: i32 = 4;
const COINS_PER_BATCH: i32 = 10;
const STORAGE_CAPACITY: i32 = 32;
pub const ORE: ResourceId = 106;
pub const REFINE: DefinitionId = 103;
const SECOND_MINT_MONTH: u32 = 4;
const THIRD_MINT_MONTH: u32 = 6;
const REPEATED_ORE: i32 = 6;
const REPEATED_FIREWOOD_TARGET: i32 = 6;
const LOW_MINT_YIELD: i32 = 4;
pub const NUTRITION: ResourceId = 107;
pub const LEISURE_TIME: ResourceId = 108;
pub const EAT: DefinitionId = 104;
pub const REST: DefinitionId = 105;
const PROVISION_MONTHS: u32 = 6;
const PROVISION_HORIZON: u32 = 3;
const FOOD_PRICE: i32 = 3;
const FOOD_PER_MONTH: i32 = 1;
const ADEQUATE_GRANARY: i32 = 18;
const SCARCE_GRANARY: i32 = 6;
const PROVISION_TREASURY: i32 = 12;
const INPUT_BID_CEILING: i32 = 6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deal {
    pub id: u32,
    pub month: u32,
    /// Indivisible packages, resolved by ascending ID from opening resources.
    pub package: u32,
    pub market: marketplace::MarketId,
    pub seller: AgentId,
    pub buyer: AgentId,
    pub price: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub issuer: AgentId,
    pub coin: ResourceId,
    pub definition: DefinitionId,
    pub venue: AgentId,
    pub deals: Vec<Deal>,
    pub order_policy: Option<orders::Policy>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub package: u32,
    pub deals: Vec<u32>,
    pub accepted: bool,
    pub reason: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boundary {
    pub month: u32,
    pub plan: Option<orders::Plan>,
    pub deals: Vec<Deal>,
    pub receipts: Vec<Receipt>,
    pub transactions: Vec<Transaction>,
}
fn transaction(w: &World, s: &State, c: &Config, d: &Deal) -> Result<Transaction, String> {
    let venue = marketplace::venue(w, c.venue).ok_or("missing venue")?;
    let market = venue
        .markets
        .iter()
        .find(|m| m.id == d.market)
        .ok_or("unlisted market")?;
    if ![d.buyer, d.seller]
        .iter()
        .all(|a| marketplace::eligible(w, s, c.venue, *a))
    {
        return Err("market admission or stock-trade permission denied".into());
    }
    if w.resources
        .iter()
        .any(|r| r.id == market.goods.resource && r.kind == ResourceKind::Capacity)
        && ![d.buyer, d.seller]
            .iter()
            .all(|a| opportunities::permits(w, s, *a, Action::CapacityTrade))
    {
        return Err("capacity-trade permission denied".into());
    }
    if d.buyer == c.issuer && !opportunities::permits(w, s, c.issuer, Action::Process(c.definition))
    {
        return Err("mint process permission denied".into());
    }
    let mut effects = Transfer {
        from: d.seller,
        to: d.buyer,
        amount: market.goods.clone(),
    }
    .effects()?;
    effects.extend(
        Transfer {
            from: d.buyer,
            to: d.seller,
            amount: Amount::new(market.payment, d.price),
        }
        .effects()?,
    );
    Ok(Transaction {
        cause: format!(
            "dated market deal {} at {} month {}",
            d.id, c.venue, s.month
        ),
        effects,
        process: None,
        technique_use: None,
        trade: None,
        stock_trade: None,
        forward: None,
        delivery: None,
        royalty: None,
    })
}
pub fn evaluate(w: &World, s: &State) -> Result<Option<Boundary>, String> {
    let Some(c) = &w.minting else {
        return Ok(None);
    };
    if s.phase != Phase::Acquire {
        return Ok(None);
    }
    let plan = c
        .order_policy
        .as_ref()
        .map(|p| orders::generate(w, s, c, p))
        .transpose()?;
    let mut deals = plan.as_ref().map(|p| p.deals.clone()).unwrap_or_else(|| {
        c.deals
            .iter()
            .filter(|d| d.month == s.month)
            .cloned()
            .collect()
    });
    deals.sort_by_key(|d| (d.package, d.id));
    let mut packages = BTreeMap::<u32, Vec<&Deal>>::new();
    for d in &deals {
        packages.entry(d.package).or_default().push(d);
    }
    let mut resources = Resources::opening(w, s);
    let mut out = Boundary {
        month: s.month,
        plan,
        deals: deals.clone(),
        receipts: vec![],
        transactions: vec![],
    };
    for (package, mut deals) in packages {
        deals.sort_by_key(|d| d.id);
        let mut staged = resources.clone();
        let result = (|| {
            let ts = deals
                .iter()
                .map(|d| transaction(w, s, c, d))
                .collect::<Result<Vec<_>, _>>()?;
            // Preflight named opening budgets for a useful shortfall receipt.
            for t in &ts {
                for e in t.effects.iter().filter(|e| e.delta < 0) {
                    let available = staged.available.get(&e.account).copied().unwrap_or(0);
                    if available < -e.delta {
                        return Err(format!(
                            "agent {} resource {} needs {}, opening available {}",
                            e.account.0, e.account.1, -e.delta, available
                        ));
                    }
                }
                staged.reserve(w, std::slice::from_ref(t))?;
            }
            Ok(ts)
        })();
        let reason = match result {
            Ok(ts) => {
                resources = staged;
                out.transactions.extend(ts);
                None
            }
            Err(e) => Some(e),
        };
        out.receipts.push(Receipt {
            package,
            deals: deals.iter().map(|d| d.id).collect(),
            accepted: reason.is_none(),
            reason,
        });
    }
    Ok(Some(out))
}
pub fn validate(w: &World) -> Result<(), String> {
    let Some(c) = &w.minting else {
        return Ok(());
    };
    // This first driver does not silently override other acquisition planners.
    if w.credit.is_some()
        || w.negotiation.is_some()
        || w.town_market.is_some()
        || w.production_market.is_some()
        || w.market.is_some()
        || w.competition.is_some()
        || w.pool_market.is_some()
        || w.work_choice.is_some()
        || !w.households.is_empty()
        || !w.issuance.is_empty()
        || !w.bids.is_empty()
        || !w.offers.is_empty()
        || !w.access_offers.is_empty()
        || !w.agreements.is_empty()
        || w.transaction_policy.is_none()
        || w.decision_horizon.is_some()
    {
        return Err(
            "physical minting requires an isolated acquisition driver and explicit policy".into(),
        );
    }
    let d = w
        .definitions
        .iter()
        .find(|d| d.id == c.definition)
        .ok_or("missing mint process")?;
    if !w.agents.iter().any(|a| a.id == c.issuer)
        || !w
            .resources
            .iter()
            .any(|r| r.id == c.coin && r.kind == ResourceKind::Stock)
        || d.execution != Execution::Productive
        || d.duration() != 1
        || d.stages.len() != 1
        || d.stages[0].entry_inputs.is_empty()
        || d.stages[0]
            .entry_inputs
            .iter()
            .any(|a| a.resource == c.coin)
        || d.stages[0].monthly_services.is_empty()
        || d.outputs.len() != 1
        || d.outputs[0].resource != c.coin
        || w.definitions.iter().any(|other| {
            other.id != c.definition && other.outputs.iter().any(|a| a.resource == c.coin)
        })
    {
        return Err("minting requires one material-and-labor process producing the coin".into());
    }
    let venue = marketplace::venue(w, c.venue).ok_or("missing mint market")?;
    if let Some(p) = &c.order_policy {
        orders::validate(w, c, p)?;
    }
    let mut ids = BTreeSet::new();
    for deal in &c.deals {
        let m = venue
            .markets
            .iter()
            .find(|m| m.id == deal.market)
            .ok_or("unlisted minting deal")?;
        if !ids.insert(deal.id)
            || deal.month == 0
            || deal.price <= 0
            || m.price_tick <= 0
            || deal.price % m.price_tick != 0
            || m.payment != c.coin
            || deal.buyer == deal.seller
            || ![deal.buyer, deal.seller]
                .iter()
                .all(|a| w.agents.iter().any(|x| x.id == *a))
        {
            return Err("invalid dated minting market terms".into());
        }
    }
    Ok(())
}
pub(crate) fn validate_batch(w: &World, s: &State, b: &Batch) -> Result<(), String> {
    if b.minting != evaluate(w, s)? {
        return Err("missing or altered minting acquisition receipt".into());
    }
    let Some(c) = &w.minting else {
        return Ok(());
    };
    if b.phase == Phase::Open {
        let mut expected = Batch::empty(s);
        crate::simulation::Simulation::open(w, s, &mut expected);
        if b.transactions != expected.transactions {
            return Err("minting opening differs from authorized capacity regeneration".into());
        }
    } else if b.phase != Phase::Acquire && b.transactions.iter().any(|t| t.process.is_none()) {
        return Err("physical minting requires a process for non-market effects".into());
    }
    if let Some(r) = &b.minting
        && r.transactions != b.transactions
    {
        return Err("minting transactions differ from reserved package".into());
    }
    let actual: i128 = b
        .transactions
        .iter()
        .flat_map(|t| &t.effects)
        .filter(|e| e.account.1 == c.coin)
        .map(|e| i128::from(e.delta))
        .sum();
    let mut authorized = 0i128;
    for change in b
        .transactions
        .iter()
        .filter_map(|t| t.process.as_ref())
        .filter(|p| p.after.definition == c.definition && p.after.status != Status::Aborted)
    {
        if change.after.operator != c.issuer || change.after.beneficiary != c.issuer {
            return Err("unauthorized physical issuer".into());
        }
        if change.after.status == Status::Completed {
            authorized += i128::from(w.definition(c.definition).outputs[0].quantity);
        }
    }
    if actual != authorized {
        return Err("coin supply differs from physical mint completion".into());
    }
    // Generic process validation below independently checks actual inputs, hours,
    // permissions, stage transitions and outputs before publishing any state.
    Ok(())
}

pub fn scenario(case: &str) -> Result<(World, State), String> {
    let (mut w, mut s) = crate::scenario::baseline();
    w.agents = [
        (ISSUER, "state"),
        (SUPPLIER, "metal supplier"),
        (WORKER, "worker"),
        (VENUE, "market"),
    ]
    .into_iter()
    .map(|(id, name)| Agent {
        id,
        name: name.into(),
    })
    .collect();
    w.participants = vec![
        Participant {
            agent: ISSUER,
            capacity: Amount::new(HOURS, 0),
            needs: vec![],
        },
        Participant {
            agent: SUPPLIER,
            capacity: Amount::new(HOURS, 0),
            needs: vec![],
        },
        Participant {
            agent: WORKER,
            capacity: Amount::new(HOURS, LABOR_LOT),
            needs: vec![],
        },
    ];
    w.resources = [
        (COIN, "minted coins", ResourceKind::Stock),
        (WHEAT, "wheat", ResourceKind::Stock),
        (METAL, "metal", ResourceKind::Stock),
        (HOURS, "labor hours", ResourceKind::Capacity),
        (FIREWOOD, "firewood", ResourceKind::Stock),
    ]
    .into_iter()
    .map(|(id, name, kind)| Resource {
        id,
        name: name.into(),
        kind,
    })
    .collect();
    w.assets.clear();
    w.rights.clear();
    s.balances.clear();
    w.definitions = vec![
        ProcessDefinition {
            id: MINT,
            name: "mint coins".into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "mint".into(),
                months: 1,
                entry_inputs: vec![Amount::new(METAL, METAL_LOT)],
                monthly_services: vec![Amount::new(HOURS, LABOR_LOT)],
            }],
            outputs: vec![Amount::new(COIN, COINS_PER_BATCH)],
        },
        ProcessDefinition {
            id: GATHER,
            name: "own firewood work".into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "collect".into(),
                months: 1,
                entry_inputs: vec![],
                monthly_services: vec![Amount::new(HOURS, LABOR_LOT)],
            }],
            outputs: vec![Amount::new(FIREWOOD, 1)],
        },
    ];
    w.scheduled_starts = vec![
        ScheduledStart {
            month: MINT_MONTH,
            agent: ISSUER,
            definition: MINT,
        },
        ScheduledStart {
            month: MINT_MONTH,
            agent: WORKER,
            definition: GATHER,
        },
    ];
    w.transaction_policy = Some(opportunities::Policy {
        authority: ISSUER,
        laws: vec![],
        agreement_forms: None,
        agreement_limits: Default::default(),
        membership_offers: vec![],
        membership_permissions: BTreeSet::new(),
        agent_types: BTreeMap::from([
            (ISSUER, opportunities::STATE_TYPE),
            (SUPPLIER, opportunities::PERSON_TYPE),
            (WORKER, opportunities::PERSON_TYPE),
            (VENUE, marketplace::MARKETPLACE_TYPE),
        ]),
        permissions: BTreeSet::from([
            (opportunities::STATE_TYPE, Action::StockTrade),
            (opportunities::PERSON_TYPE, Action::StockTrade),
            (opportunities::STATE_TYPE, Action::CapacityTrade),
            (opportunities::PERSON_TYPE, Action::CapacityTrade),
            (opportunities::STATE_TYPE, Action::Process(MINT)),
            (opportunities::PERSON_TYPE, Action::Process(GATHER)),
        ]),
    });
    w.marketplaces = vec![marketplace::Marketplace {
        agent: VENUE,
        allowed_types: BTreeSet::from([opportunities::PERSON_TYPE, opportunities::STATE_TYPE]),
        markets: vec![
            marketplace::Market {
                id: WHEAT,
                goods: Amount::new(WHEAT, WHEAT_LOT),
                payment: COIN,
                price_tick: 1,
            },
            marketplace::Market {
                id: METAL,
                goods: Amount::new(METAL, METAL_LOT),
                payment: COIN,
                price_tick: 1,
            },
            marketplace::Market {
                id: HOURS,
                goods: Amount::new(HOURS, LABOR_LOT),
                payment: COIN,
                price_tick: 1,
            },
        ],
    }];
    w.minting = Some(Config {
        issuer: ISSUER,
        coin: COIN,
        definition: MINT,
        venue: VENUE,
        order_policy: None,
        deals: vec![
            Deal {
                id: 1,
                month: SALE_MONTH,
                package: 1,
                market: WHEAT,
                seller: ISSUER,
                buyer: SUPPLIER,
                price: WHEAT_LOT,
            },
            Deal {
                id: 2,
                month: SALE_MONTH,
                package: 2,
                market: WHEAT,
                seller: ISSUER,
                buyer: WORKER,
                price: WHEAT_LOT,
            },
            Deal {
                id: 3,
                month: MINT_MONTH,
                package: 3,
                market: METAL,
                seller: SUPPLIER,
                buyer: ISSUER,
                price: METAL_LOT,
            },
            Deal {
                id: 4,
                month: MINT_MONTH,
                package: 3,
                market: HOURS,
                seller: WORKER,
                buyer: ISSUER,
                price: WAGE,
            },
        ],
    });
    s.balances = BTreeMap::from([
        ((ISSUER, WHEAT), WHEAT_LOT * 2),
        ((SUPPLIER, METAL), METAL_LOT),
        ((SUPPLIER, COIN), OPENING_COINS),
        ((WORKER, COIN), OPENING_COINS),
    ]);
    w.storage.weights = BTreeMap::from([(METAL, 1), (WHEAT, 1), (FIREWOOD, 1)]);
    w.storage.capacities = [ISSUER, SUPPLIER, WORKER]
        .into_iter()
        .map(|a| (a, STORAGE_CAPACITY))
        .collect();
    match case {
        "normal" => {}
        "treasury" => {
            s.balances.insert((ISSUER, WHEAT), WHEAT_LOT);
        }
        "metal" => {
            s.balances.insert((SUPPLIER, METAL), METAL_LOT - 1);
        }
        "labor" => {
            w.capacity_overrides
                .insert((MINT_MONTH, WORKER), LABOR_LOT - 1);
        }
        _ => return Err("unknown physical minting case".into()),
    }
    Ok((w, s))
}

/// Same production target and initial stocks as the fixed-term control.
pub fn order_scenario(case: &str) -> Result<(World, State), String> {
    use marketplace::Side;
    let (mut w, s) = scenario(case)?;
    let c = w.minting.as_mut().unwrap();
    c.deals.clear();
    c.order_policy = Some(orders::Policy {
        month: MINT_MONTH,
        additional_months: BTreeSet::new(),
        provisioning: None,
        sale_market: WHEAT,
        sale_limit: WHEAT_LOT,
        input_limits: BTreeMap::from([(METAL, METAL_LOT), (HOURS, WAGE)]),
        quotes: vec![
            orders::Quote {
                agent: SUPPLIER,
                market: WHEAT,
                side: Side::Buy,
                limit: WHEAT_LOT,
                holding: WHEAT_LOT,
            },
            orders::Quote {
                agent: WORKER,
                market: WHEAT,
                side: Side::Buy,
                limit: WHEAT_LOT,
                holding: WHEAT_LOT,
            },
            orders::Quote {
                agent: SUPPLIER,
                market: METAL,
                side: Side::Sell,
                limit: METAL_LOT,
                holding: 0,
            },
            orders::Quote {
                agent: WORKER,
                market: HOURS,
                side: Side::Sell,
                limit: WAGE,
                holding: 0,
            },
        ],
    });
    Ok((w, s))
}

/// Repeated targets with finite ore, ordinary mining, and competing worker activity.
pub fn repeated_scenario(case: &str) -> Result<(World, State), String> {
    use crate::activities::{Target, WorkOrder};
    let (mut w, mut s) = order_scenario("normal")?;
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .additional_months = BTreeSet::from([SECOND_MINT_MONTH, THIRD_MINT_MONTH]);
    w.scheduled_starts = [MINT_MONTH, SECOND_MINT_MONTH, THIRD_MINT_MONTH]
        .into_iter()
        .map(|month| ScheduledStart {
            month,
            agent: ISSUER,
            definition: MINT,
        })
        .collect();
    w.resources.push(Resource {
        id: ORE,
        name: "finite ore stock".into(),
        kind: ResourceKind::Stock,
    });
    w.storage.weights.insert(ORE, 1);
    s.balances.remove(&(SUPPLIER, METAL));
    s.balances.insert((SUPPLIER, ORE), REPEATED_ORE);
    w.participants
        .iter_mut()
        .find(|p| p.agent == SUPPLIER)
        .unwrap()
        .capacity
        .quantity = LABOR_LOT;
    w.definitions.push(ProcessDefinition {
        id: REFINE,
        name: "work ore into mint metal".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "refine".into(),
            months: 1,
            entry_inputs: vec![Amount::new(ORE, METAL_LOT)],
            monthly_services: vec![Amount::new(HOURS, LABOR_LOT)],
        }],
        outputs: vec![Amount::new(METAL, METAL_LOT)],
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((opportunities::PERSON_TYPE, Action::Process(REFINE)));
    w.activities.orders = vec![
        WorkOrder {
            agent: SUPPLIER,
            definition: REFINE,
            priority: 0,
            target: Target::Stock(Amount::new(METAL, METAL_LOT)),
        },
        WorkOrder {
            agent: WORKER,
            definition: GATHER,
            priority: 0,
            target: Target::Stock(Amount::new(FIREWOOD, REPEATED_FIREWOOD_TARGET)),
        },
    ];
    match case {
        "normal" => {}
        "ore" => {
            s.balances.insert((SUPPLIER, ORE), METAL_LOT);
        }
        "labor" => {
            w.capacity_overrides
                .insert((SECOND_MINT_MONTH, WORKER), LABOR_LOT - 1);
        }
        "low_yield" => {
            w.definitions
                .iter_mut()
                .find(|d| d.id == MINT)
                .unwrap()
                .outputs[0]
                .quantity = LOW_MINT_YIELD;
        }
        _ => return Err("unknown repeated minting case".into()),
    }
    Ok((w, s))
}

/// Food provision and purchasing-power-sensitive work in the same isolated market.
pub fn provision_scenario(case: &str) -> Result<(World, State), String> {
    use crate::activities::{Target, WorkOrder};
    let (mut w, mut s) = repeated_scenario("normal")?;
    w.resources.extend([
        Resource {
            id: NUTRITION,
            name: "nutrition".into(),
            kind: ResourceKind::Fulfillment,
        },
        Resource {
            id: LEISURE_TIME,
            name: "leisure hours this month".into(),
            kind: ResourceKind::Stock,
        },
    ]);
    w.activities.perishable.insert(LEISURE_TIME);
    w.definitions.extend([
        ProcessDefinition {
            id: EAT,
            name: "eat wheat".into(),
            execution: Execution::Consumption,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "eat".into(),
                months: 1,
                entry_inputs: vec![Amount::new(WHEAT, FOOD_PER_MONTH)],
                monthly_services: vec![],
            }],
            outputs: vec![Amount::new(NUTRITION, FOOD_PER_MONTH)],
        },
        ProcessDefinition {
            id: REST,
            name: "take leisure".into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "leisure".into(),
                months: 1,
                entry_inputs: vec![],
                monthly_services: vec![Amount::new(HOURS, LABOR_LOT)],
            }],
            outputs: vec![Amount::new(LEISURE_TIME, LABOR_LOT)],
        },
    ]);
    for a in w.participants.iter_mut().filter(|a| a.agent != ISSUER) {
        a.needs = vec![Requirement {
            resource: NUTRITION,
            quantity: FOOD_PER_MONTH,
            priority: 0,
        }];
    }
    let policy = w.transaction_policy.as_mut().unwrap();
    policy
        .permissions
        .insert((opportunities::PERSON_TYPE, Action::Process(EAT)));
    policy
        .permissions
        .insert((opportunities::PERSON_TYPE, Action::Process(REST)));
    w.marketplaces[0]
        .markets
        .iter_mut()
        .find(|m| m.id == WHEAT)
        .unwrap()
        .goods
        .quantity = FOOD_PER_MONTH;
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.month = 1;
    p.additional_months = (2..=PROVISION_MONTHS).collect();
    p.sale_limit = FOOD_PRICE;
    p.input_limits = BTreeMap::from([(METAL, INPUT_BID_CEILING), (HOURS, INPUT_BID_CEILING)]);
    p.provisioning = Some(provisioning::Policy {
        consumption: EAT,
        horizon_months: PROVISION_HORIZON,
        leisure: REST,
        earning: BTreeMap::from([(SUPPLIER, Some(REFINE)), (WORKER, None)]),
    });
    for q in p.quotes.iter_mut().filter(|q| q.market == WHEAT) {
        q.limit = FOOD_PRICE;
    }
    w.scheduled_starts = (1..=PROVISION_MONTHS)
        .map(|month| ScheduledStart {
            month,
            agent: ISSUER,
            definition: MINT,
        })
        .collect();
    w.activities.orders = vec![
        WorkOrder {
            agent: SUPPLIER,
            definition: REFINE,
            priority: 0,
            target: Target::Stock(Amount::new(METAL, METAL_LOT)),
        },
        WorkOrder {
            agent: SUPPLIER,
            definition: REST,
            priority: 0,
            target: Target::Stock(Amount::new(LEISURE_TIME, LABOR_LOT)),
        },
        WorkOrder {
            agent: WORKER,
            definition: REST,
            priority: 0,
            target: Target::Stock(Amount::new(LEISURE_TIME, LABOR_LOT)),
        },
    ];
    s.balances.insert((SUPPLIER, METAL), METAL_LOT);
    s.balances.insert((ISSUER, COIN), PROVISION_TREASURY);
    s.balances.insert((ISSUER, WHEAT), ADEQUATE_GRANARY);
    for a in [SUPPLIER, WORKER] {
        s.balances.insert((a, COIN), FOOD_PRICE);
    }
    match case {
        "adequate" => {}
        "scarce" => {
            s.balances.insert((ISSUER, WHEAT), SCARCE_GRANARY);
        }
        "empty" => {
            s.balances.insert((ISSUER, WHEAT), 0);
        }
        "endowed" => {
            for a in [SUPPLIER, WORKER] {
                s.balances
                    .insert((a, WHEAT), PROVISION_MONTHS as i32 * FOOD_PER_MONTH);
            }
        }
        _ => return Err("unknown provision case".into()),
    }
    Ok((w, s))
}
