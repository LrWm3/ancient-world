//! Bounded recurring-operation acceptance. Terms and initial constitutions are
//! supplied; every delivery, hour, payment and decision uses ordinary execution.
use super::*;
use crate::{
    agency::{
        self, Command, Program,
        objectives::{Metric, Objective, Scope},
    },
    forward::direct,
    governance::Leadership,
};

pub const MONTHS: u32 = 48;
pub const CYCLE_MONTHS: u32 = 12;
const CONTRACT_THROUGH: u32 = MONTHS + CYCLE_MONTHS;
pub(super) const REFINING_HOURS: i32 = 1;
const ORE_LOT: i32 = 2;
pub(super) const ORE_PRICE: i32 = 2;
const ORE_BUFFER: i32 = 4;
const METAL_PRICE: i32 = 4;
const METAL_BUFFER: i32 = 4;
const SEED_REPLACEMENT: i32 = 1;
const ISSUER_RESERVE: i32 = 1000;
const FIRST_ORE_ORDER: u32 = 2;
const ORE_CONTRACT_BASE: u32 = 1000;
const ORGANIZATION_FORECAST: u32 = 4;
const STARTUP_PAYROLL_ID: u32 = 811;
const STARTUP_PAYROLL_SHORTFALL: i32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    Baseline,
    NoOreSupply,
    NoReplacementOrders,
    NoMetalAccess,
    LowMetalPrice,
    LatePayroll,
}
impl Case {
    pub const ALL: [Self; 6] = [
        Self::Baseline,
        Self::NoOreSupply,
        Self::NoReplacementOrders,
        Self::NoMetalAccess,
        Self::LowMetalPrice,
        Self::LatePayroll,
    ];
}

pub fn scenario(case: Case) -> Result<(World, State), String> {
    let (mut w, mut s) = super::scenario()?;
    w.rights.iter_mut().find(|r| r.id == 77).unwrap().through = CONTRACT_THROUGH;
    for h in &mut w.households {
        for support in &mut h.support {
            support.through = CONTRACT_THROUGH;
        }
    }
    for term in &mut w.employment {
        term.through = CONTRACT_THROUGH;
        if term.employer == WORKSHOP {
            term.capacity.quantity = REFINING_HOURS;
        }
    }
    w.definitions
        .iter_mut()
        .find(|d| d.id == REFINE)
        .unwrap()
        .stages[0]
        .monthly_services[0]
        .quantity = REFINING_HOURS;
    s.balances.insert((WORKSHOP, ORE), ORE_BUFFER);
    // A supplied working inventory bridges next-month governance and acquisition.
    s.balances.insert((WORKSHOP, METAL), REFINING_LOT);
    w.definitions
        .iter_mut()
        .find(|d| d.id == integration::GROW)
        .unwrap()
        .outputs
        .iter_mut()
        .find(|a| a.resource == integration::SEED)
        .unwrap()
        .quantity = SEED_REPLACEMENT;
    let mine = w
        .definitions
        .iter_mut()
        .find(|d| d.id == MUNICIPAL_WORK)
        .unwrap();
    mine.name = "municipal ore extraction".into();
    mine.outputs = vec![Amount::new(ORE, ORE_LOT)];
    w.transaction_policy.as_mut().unwrap().permissions.extend([
        (TOWN_TYPE, Action::StockTrade),
        (PERSON_TYPE, Action::GovernOrganization(WORKSHOP_TYPE)),
        (PERSON_TYPE, Action::GovernOrganization(TOWN_TYPE)),
    ]);
    if case != Case::NoReplacementOrders {
        for month in FIRST_ORE_ORDER..CONTRACT_THROUGH {
            w.prepaid_deliveries.push(direct::Terms {
                id: ORE_CONTRACT_BASE + month,
                seller: TOWN,
                buyer: WORKSHOP,
                month,
                due: month + 1,
                goods: Amount::new(ORE, ORE_LOT),
                prepayment: Amount::new(COIN, ORE_PRICE),
            });
        }
    }
    let price = if case == Case::LowMetalPrice {
        REFINING_LOT
    } else {
        METAL_PRICE
    };
    let p = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    p.input_limits.insert(METAL, price);
    for q in &mut p.quotes {
        if q.market == METAL && q.side == Side::Sell {
            q.limit = price;
        }
    }
    let c = &mut w.agency.get_mut(&ISSUER).unwrap().config;
    c.objectives = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Reserve {
            resource: COIN,
            target: ISSUER_RESERVE,
        },
    }];
    for preferences in c.preferences.values_mut() {
        *preferences = c.objectives.clone();
    }
    w.activities
        .orders
        .retain(|o| ![WORKSHOP, TOWN].contains(&o.agent));
    for (agent, founder, members, definition, resource, target, leadership) in [
        (
            WORKSHOP,
            SMITH,
            [SMITH].into(),
            REFINE,
            METAL,
            METAL_BUFFER,
            Leadership::FixedFounder,
        ),
        (
            TOWN,
            SUPPLIER,
            [SUPPLIER, integration::GROWER].into(),
            MUNICIPAL_WORK,
            ORE,
            ORE_BUFFER,
            Leadership::Rotating,
        ),
    ] {
        let agreement = agency::organization::Agreement {
            organization: agent,
            members,
            founder,
            leadership,
            term_months: CYCLE_MONTHS,
            processes: [definition].into(),
            stock_markets: Default::default(),
            forwards: Default::default(),
        };
        let mut config = agency::scenario::config(
            vec![Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve { resource, target },
            }],
            [(
                1,
                Program {
                    name: "maintain contracted supply buffer".into(),
                    commands: vec![Command::WorkTargets(vec![WorkOrder {
                        agent,
                        definition,
                        priority: 0,
                        target: Target::Stock(Amount::new(resource, target)),
                    }])],
                },
            )]
            .into(),
        );
        config.horizon = ORGANIZATION_FORECAST;
        config.review_every = CYCLE_MONTHS;
        let signatures = agreement.members.clone();
        agency::organization::install_initial(&mut w, &s, agreement, &signatures, config)?;
    }
    match case {
        Case::NoOreSupply => {
            w.definitions
                .iter_mut()
                .find(|d| d.id == MUNICIPAL_WORK)
                .unwrap()
                .enabled = false;
        }
        Case::NoMetalAccess => {
            w.marketplaces[0].allowed_types.remove(&WORKSHOP_TYPE);
        }
        Case::LatePayroll => {
            let ordinary = w
                .employment
                .iter_mut()
                .find(|t| t.id == WORKSHOP_EMPLOYMENT)
                .unwrap();
            ordinary.from = 2;
            ordinary.on_arrears = crate::employment::ArrearsPolicy::Continue;
            let mut startup = ordinary.clone();
            startup.id = STARTUP_PAYROLL_ID;
            startup.from = 1;
            startup.through = 1;
            startup.wage_per_unit.quantity = WORKSHOP_OPENING_COINS + STARTUP_PAYROLL_SHORTFALL;
            w.employment.push(startup);
        }
        _ => {}
    }
    Ok((w, s))
}

/// Same economy with inventory-driven ore bids/asks instead of dated deliveries.
pub fn spot_scenario(case: Case) -> Result<(World, State), String> {
    let (mut w, s) = scenario(case)?;
    w.prepaid_deliveries
        .retain(|t| !(t.seller == TOWN && t.buyer == WORKSHOP && t.goods.resource == ORE));
    let venue = &mut w.marketplaces[0];
    venue.allowed_types.insert(TOWN_TYPE);
    venue.markets.push(crate::marketplace::Market {
        id: ORE,
        goods: Amount::new(ORE, ORE_LOT),
        payment: COIN,
        price_tick: 1,
    });
    let mut quotes = vec![crate::minting::orders::Quote {
        agent: TOWN,
        market: ORE,
        side: Side::Sell,
        limit: ORE_PRICE,
        holding: 0,
        max_lots: None,
    }];
    if case != Case::NoReplacementOrders {
        quotes.push(crate::minting::orders::Quote {
            agent: WORKSHOP,
            market: ORE,
            side: Side::Buy,
            limit: ORE_PRICE,
            holding: ORE_BUFFER,
            max_lots: None,
        });
    }
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .peer_stocks = Some(crate::minting::orders::peer_stocks::Policy {
        claim_months: SALE_PROTECTION_MONTHS,
        quotes,
    });
    Ok((w, s))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Period {
    pub from: u32,
    pub through: u32,
    pub workshop_batches: usize,
    pub municipal_batches: usize,
    pub metal_sales: usize,
    pub ore_delivered: i32,
    pub workshop_hours: i32,
    pub municipal_hours: i32,
    pub workshop_wages: i32,
    pub municipal_wages: i32,
    pub workshop_income: i128,
    pub municipal_income: i128,
    pub workshop_opening_cash: i128,
    pub workshop_closing_cash: i128,
    pub municipal_opening_cash: i128,
    pub municipal_closing_cash: i128,
}

pub fn period(run: &Run, from: u32, through: u32) -> Result<Period, String> {
    if from == 0 || from > through || through >= run.sim.state.month {
        return Err("period must be fully completed".into());
    }
    let w = run.audit.book().statements(WORKSHOP, from, through)?;
    let t = run.audit.book().statements(TOWN, from, through)?;
    let between = |month: u32| (from..=through).contains(&month);
    let completed = |agent, definition| {
        run.sim
            .ledger
            .iter()
            .filter(|b| between(b.month))
            .flat_map(|b| &b.transactions)
            .filter(|t| {
                t.process.as_ref().is_some_and(|p| {
                    p.after.operator == agent
                        && p.after.definition == definition
                        && p.after.status == Status::Completed
                        && p.before
                            .as_ref()
                            .is_none_or(|b| b.status != Status::Completed)
                })
            })
            .count()
    };
    let forward_ore: i32 = run
        .sim
        .ledger
        .iter()
        .filter(|b| between(b.month))
        .flat_map(|b| &b.transactions)
        .filter_map(|t| match t.forward {
            Some(crate::forward::Event::Delivery { contract, quantity })
                if run.sim.world.prepaid_deliveries.iter().any(|p| {
                    p.id == contract
                        && p.seller == TOWN
                        && p.buyer == WORKSHOP
                        && p.goods.resource == ORE
                }) =>
            {
                Some(quantity)
            }
            _ => None,
        })
        .sum();
    let spot_ore: i32 = accepted_deals(&run.sim)
        .filter(|d| between(d.month) && d.seller == TOWN && d.buyer == WORKSHOP)
        .filter_map(|d| {
            let venue = run.sim.world.minting.as_ref()?.venue;
            run.sim
                .world
                .marketplaces
                .iter()
                .find(|v| v.agent == venue)
                .and_then(|v| {
                    v.markets
                        .iter()
                        .find(|m| m.id == d.market && m.goods.resource == ORE)
                })
                .map(|m| m.goods.quantity)
        })
        .sum();
    let employment = |id| {
        run.sim
            .state
            .employment
            .earned
            .iter()
            .filter(|((agreement, month), _)| *agreement == id && between(*month))
            .fold((0, 0), |(hours, wages), (_, e)| {
                (hours + e.delivered, wages + e.claim.settled)
            })
    };
    let (workshop_hours, workshop_wages) = employment(WORKSHOP_EMPLOYMENT);
    let (municipal_hours, municipal_wages) = employment(TOWN_EMPLOYMENT);
    Ok(Period {
        workshop_hours,
        workshop_wages,
        municipal_hours,
        municipal_wages,
        from,
        through,
        workshop_batches: completed(WORKSHOP, REFINE),
        municipal_batches: completed(TOWN, MUNICIPAL_WORK),
        metal_sales: accepted_deals(&run.sim)
            .filter(|d| between(d.month) && d.seller == WORKSHOP && d.market == METAL)
            .count(),
        ore_delivered: forward_ore + spot_ore,
        workshop_income: w.net_income,
        municipal_income: t.net_income,
        workshop_opening_cash: w.opening_cash,
        workshop_closing_cash: w.closing_cash,
        municipal_opening_cash: t.opening_cash,
        municipal_closing_cash: t.closing_cash,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Readiness {
    pub periods: Vec<Period>,
    pub failures: Vec<String>,
}
impl Readiness {
    pub fn ready(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Fail closed: a functioning treasury or balanced books alone are insufficient.
pub(crate) fn obligations(run: &Run) -> Result<Vec<String>, String> {
    let e = run.evidence()?;
    if e.months != MONTHS {
        return Err("acceptance requires exactly 48 completed months".into());
    }
    let mut failures = vec![];
    let mut require = |pass: bool, reason: &str| {
        if !pass {
            failures.push(reason.into());
        }
    };
    require(e.people_food_deficit == 0, "unmet nutrition");
    require(e.wage_arrears == 0, "unpaid earned wages");
    require(e.household_on_time_delivery == 2, "late household forward");
    require(e.state_loan_repaid, "state bootstrap loan not repaid");
    require(e.reconciled_agents == 9, "missing entity accounts");
    for month in [13, 25, 37] {
        require(
            run.sim
                .reports
                .iter()
                .filter(|r| r.month == month)
                .any(|r| {
                    r.obligations
                        .get(&(77, month))
                        .is_some_and(|c| c.paid == 1 && c.outstanding() == 0)
                }),
            "missing or late annual rent",
        );
    }
    for terms in run
        .sim
        .world
        .prepaid_deliveries
        .iter()
        .filter(|t| t.due <= MONTHS)
    {
        let delivered: i32 = run
            .sim
            .ledger
            .iter()
            .filter(|b| b.month <= terms.due)
            .flat_map(|b| &b.transactions)
            .filter_map(|t| match t.forward {
                Some(crate::forward::Event::Delivery { contract, quantity })
                    if contract == terms.id =>
                {
                    Some(quantity)
                }
                _ => None,
            })
            .sum();
        require(
            delivered == terms.goods.quantity,
            "missing or late prepaid delivery",
        );
    }
    require(
        !run.sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.events)
            .any(|e| matches!(e, crate::credit::Event::Arrears { .. })),
        "loan entered arrears",
    );
    require(
        !run.sim
            .ledger
            .iter()
            .filter(|b| b.phase == Phase::Close)
            .filter_map(|b| b.employment.as_ref())
            .flat_map(|b| &b.receipts)
            .any(|r| {
                matches!(
                    r.reason,
                    crate::employment::Reason::Collection
                        | crate::employment::Reason::CollectionStayed
                ) && r.requested > r.paid
            }),
        "wages entered arrears",
    );
    for agent in [WORKSHOP, TOWN] {
        require(
            run.sim.world.agency.get(&agent).is_some_and(|c| {
                c.operating_agreement().is_some()
                    && agency::accepted_programs(&run.sim.world, agent)
                        .next()
                        .is_some()
            }),
            "no authorized organization work choice",
        );
    }
    Ok(failures)
}

pub fn check(run: &Run) -> Result<Readiness, String> {
    let mut failures = obligations(run)?;
    let mut require = |pass: bool, reason: &str| {
        if !pass {
            failures.push(reason.into());
        }
    };
    let periods = (1..MONTHS / CYCLE_MONTHS)
        .map(|cycle| period(run, cycle * CYCLE_MONTHS + 1, (cycle + 1) * CYCLE_MONTHS))
        .collect::<Result<Vec<_>, _>>()?;
    for p in &periods {
        require(
            p.workshop_hours == REFINING_HOURS * CYCLE_MONTHS as i32
                && p.municipal_hours == MUNICIPAL_HOURS * CYCLE_MONTHS as i32
                && p.workshop_wages == REFINING_HOURS * WAGE_PER_HOUR * CYCLE_MONTHS as i32
                && p.municipal_wages == MUNICIPAL_HOURS * WAGE_PER_HOUR * CYCLE_MONTHS as i32,
            "missing paid operating labor",
        );
        require(
            p.workshop_batches >= CYCLE_MONTHS as usize
                && p.municipal_batches >= CYCLE_MONTHS as usize
                && p.metal_sales >= CYCLE_MONTHS as usize,
            "recurring production or market demand absent",
        );
        require(
            p.ore_delivered >= ORE_LOT * CYCLE_MONTHS as i32,
            "replacement ore not delivered",
        );
        require(
            p.workshop_income > 0 && p.municipal_income > 0,
            "nonpositive operating income",
        );
        require(
            p.workshop_closing_cash >= p.workshop_opening_cash
                && p.municipal_closing_cash >= p.municipal_opening_cash,
            "operating reserves declining",
        );
        require(
            p.workshop_closing_cash >= i128::from(ORE_PRICE + REFINING_HOURS * WAGE_PER_HOUR)
                && p.municipal_closing_cash >= i128::from(MUNICIPAL_HOURS * WAGE_PER_HOUR),
            "next operating payment not covered",
        );
    }
    failures.sort();
    failures.dedup();
    Ok(Readiness { periods, failures })
}
