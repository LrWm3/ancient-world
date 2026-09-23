//! Bounded accept/decline projections using ordinary production and debt settlement.
use crate::{
    compute::Backend, credit, forecast::ForecastContext, model::*, offers, simulation::Simulation,
};
use std::collections::BTreeMap;

const MAX_HORIZON_MONTHS: u32 = 24;
const DECISION_MONTHS: u32 = 12;
const FUNDED_COINS: i32 = 10_300;
const DOWNPAYMENT_ONLY_COINS: i32 = 2_000;
const INITIAL_GRAIN: i32 = 5;
const FALLBACK_PROCESS: DefinitionId = 50;
const FORAGE_MONTHS: u32 = 3;
const FORAGE_OUTPUT: i32 = 1;
const FORAGE_LABOR: i32 = 2;
const LOW_CROP_OUTPUT: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Policy {
    Scripted,
    Decline,
    Compare(Config),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub horizon_months: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Month {
    pub sold_stock: i32,
    pub sale_coins: i32,
    pub month: u32,
    pub coins: i32,
    pub debt: i32,
    pub deficits: BTreeMap<ResourceId, i32>,
    pub labor: i64,
    pub first_unpaid: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection {
    pub months: Vec<Month>,
    pub deficits: BTreeMap<ResourceId, i64>,
    pub failed_processes: usize,
    pub terminal: bool,
    pub closing_coins: i32,
    pub closing_debt: i32,
    pub missed_payment: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    Beneficial,
    InstallmentShortfall,
    Infeasible(String),
    NotBeneficial,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub agent: AgentId,
    pub offer: u32,
    pub month: u32,
    pub through: u32,
    pub decline: Projection,
    pub purchase: Option<Projection>,
    pub accept: bool,
    pub reason: Reason,
}

pub fn validate(world: &World) -> Result<(), String> {
    let Some(c) = &world.credit else {
        return Ok(());
    };
    let Policy::Compare(config) = &c.purchase_policy else {
        return Ok(());
    };
    let offer = c
        .offers
        .iter()
        .find(|o| o.id == c.application.offer)
        .ok_or("missing borrowing offer")?;
    if c.offers.len() != 1
        || config.horizon_months <= offer.loan.term_months
        || config.horizon_months > MAX_HORIZON_MONTHS
        || !world
            .participants
            .iter()
            .any(|p| p.agent == c.application.buyer && !p.needs.is_empty())
        || world.work_choice.is_some()
        || world.decision_horizon.is_some()
        || world.priority != Priority::ContinuingFirst
        || c.resale_buyer.is_some()
        || !world.scheduled_starts.is_empty()
    {
        return Err("borrowing comparison requires a needs-bearing buyer, ordinary work policy and a horizon covering all installments".into());
    }
    Ok(())
}

fn project(context: &ForecastContext, accept: bool, horizon: u32) -> Result<Projection, String> {
    let (mut w, s) = context.clone().into_parts();
    let c = w.credit.as_mut().ok_or("missing credit configuration")?;
    c.purchase_policy = if accept {
        Policy::Scripted
    } else {
        Policy::Decline
    };
    let agent = c.application.buyer;
    let offer = c.application.offer;
    let coin = c
        .offers
        .iter()
        .find(|o| o.id == offer)
        .ok_or("missing offer")?
        .loan
        .denomination;
    let labor = w
        .participants
        .iter()
        .find(|p| p.agent == agent)
        .ok_or("missing buyer")?
        .capacity
        .resource;
    let end = s
        .month
        .checked_add(horizon)
        .ok_or("borrowing horizon overflow")?;
    let mut sim = Simulation::new(w, s, Backend::Reference)?;
    if accept {
        // Same common acceptance and atomic settlement as the live purchase.
        offers::accept(
            &mut sim,
            &[offers::Request::new(
                offers::Id::FinancedPurchase(offer),
                agent,
            )],
        )?;
    } else {
        sim.step()?; // Commit the same Acquire boundary with no purchase.
    }
    let mut result = Projection {
        months: vec![],
        deficits: BTreeMap::new(),
        failed_processes: 0,
        terminal: false,
        closing_coins: 0,
        closing_debt: 0,
        missed_payment: None,
    };
    let sale = sim
        .ledger
        .last()
        .and_then(|b| b.credit.as_ref())
        .and_then(|b| b.stock_sale.as_ref());
    let mut sold_stock = sale.map_or(0, |s| s.goods);
    let mut sale_coins = sale.map_or(0, |s| s.coins);
    let mut monthly_labor = 0;
    while sim.state.month < end {
        let month = sim.state.month;
        sim.step()?;
        let batch = sim.ledger.last().unwrap();
        if let Some(sale) = batch.credit.as_ref().and_then(|b| b.stock_sale.as_ref()) {
            sold_stock = sold_stock
                .checked_add(sale.goods)
                .ok_or("projected sales overflow")?;
            sale_coins = sale_coins
                .checked_add(sale.coins)
                .ok_or("projected sale coins overflow")?;
        }
        monthly_labor += batch
            .transactions
            .iter()
            .flat_map(|t| &t.effects)
            .filter(|e| {
                matches!(batch.phase, Phase::Productive | Phase::Consumption)
                    && e.account == (agent, labor)
                    && e.delta < 0
            })
            .map(|e| -i64::from(e.delta))
            .sum::<i64>();
        let loan = sim.state.credit.loans.get(&offer);
        if let Some(since) = loan.and_then(|l| l.first_unpaid) {
            result.missed_payment = Some(result.missed_payment.map_or(since, |m| m.min(since)));
        }
        // A zero-grace enforcement may clear first_unpaid in the same boundary.
        if batch.credit.as_ref().is_some_and(|b| {
            b.events
                .iter()
                .any(|e| matches!(e, credit::Event::Arrears { .. }))
        }) {
            result.missed_payment = Some(result.missed_payment.map_or(month, |m| m.min(month)));
        }
        if sim.state.month != month {
            let report = sim
                .reports
                .iter()
                .rev()
                .find(|r| r.agent == agent && r.month == month)
                .ok_or("missing borrowing month report")?;
            let deficits: BTreeMap<_, _> =
                report.needs.iter().map(|(r, n)| (*r, n.deficit)).collect();
            for (r, n) in &deficits {
                *result.deficits.entry(*r).or_default() += i64::from(*n);
            }
            result.months.push(Month {
                sold_stock,
                sale_coins,
                month,
                coins: sim.state.balance(agent, coin),
                debt: loan.map(|l| l.debt()).transpose()?.unwrap_or(0),
                deficits,
                labor: monthly_labor,
                first_unpaid: loan.and_then(|l| l.first_unpaid),
            });
            monthly_labor = 0;
            sold_stock = 0;
            sale_coins = 0;
        }
    }
    result.failed_processes = sim
        .state
        .processes
        .values()
        .filter(|p| {
            p.operator == agent && p.start >= context.state().month && p.status == Status::Aborted
        })
        .count();
    result.terminal = sim.state.terminal.contains_key(&agent);
    result.closing_coins = sim.state.balance(agent, coin);
    result.closing_debt = sim
        .state
        .credit
        .loans
        .get(&offer)
        .map(|l| l.debt())
        .transpose()?
        .unwrap_or(0);
    Ok(result)
}

pub fn evaluate(world: &World, state: &State) -> Result<Decision, String> {
    validate(world)?;
    let c = world
        .credit
        .as_ref()
        .ok_or("missing credit configuration")?;
    let Policy::Compare(config) = &c.purchase_policy else {
        return Err("borrowing comparison policy is not enabled".into());
    };
    if state.phase != Phase::Acquire
        || c.application.month != state.month
        || state.credit.loans.contains_key(&c.application.offer)
    {
        return Err("borrowing comparison outside the unfilled application boundary".into());
    }
    let context = ForecastContext::new(world, state);
    let decline = project(&context, false, config.horizon_months)?;
    let purchase = project(&context, true, config.horizon_months);
    let mut needs = world
        .participants
        .iter()
        .find(|p| p.agent == c.application.buyer)
        .ok_or("missing borrower")?
        .needs
        .clone();
    needs.sort_by_key(|n| (n.priority, n.resource));
    let score = |p: &Projection| {
        (
            p.terminal,
            needs
                .iter()
                .map(|n| p.deficits.get(&n.resource).copied().unwrap_or(0))
                .collect::<Vec<_>>(),
            p.failed_processes,
            i64::from(p.closing_debt) - i64::from(p.closing_coins),
        )
    };
    let reason = match &purchase {
        Err(e) => Reason::Infeasible(e.clone()),
        Ok(p) if p.missed_payment.is_some() || p.closing_debt > 0 => Reason::InstallmentShortfall,
        Ok(p) if score(p) < score(&decline) => Reason::Beneficial,
        Ok(_) => Reason::NotBeneficial,
    };
    Ok(Decision {
        agent: c.application.buyer,
        offer: c.application.offer,
        month: state.month,
        through: state
            .month
            .checked_add(config.horizon_months - 1)
            .ok_or("borrowing horizon overflow")?,
        decline,
        purchase: purchase.ok(),
        accept: reason == Reason::Beneficial,
        reason,
    })
}

pub fn scenario(case: &str) -> Result<(World, State), String> {
    use crate::scenario::*;
    let (mut w, mut s) = credit::crop_scenario(false)?;
    let (catalog, _) = baseline();
    w.scheduled_starts.clear();
    w.resources.push(
        catalog
            .resources
            .iter()
            .find(|r| r.id == NUTRITION)
            .unwrap()
            .clone(),
    );
    w.definitions.push(catalog.definition(CONSUME).clone());
    w.participants.retain(|p| p.agent == PERSON);
    w.participants[0].needs = vec![Requirement {
        resource: NUTRITION,
        quantity: 1,
        priority: 0,
    }];
    w.definitions.push(ProcessDefinition {
        id: FALLBACK_PROCESS,
        name: "slow food gathering".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "gather".into(),
            months: FORAGE_MONTHS,
            entry_inputs: vec![],
            monthly_services: vec![Amount::new(LABOR, FORAGE_LABOR)],
        }],
        outputs: vec![Amount::new(GRAIN, FORAGE_OUTPUT)],
    });
    s.balances.insert((PERSON, GRAIN), INITIAL_GRAIN);
    let c = w.credit.as_mut().unwrap();
    c.purchase_policy = Policy::Compare(Config {
        horizon_months: DECISION_MONTHS,
    });
    c.endowments
        .iter_mut()
        .find(|e| e.agent == PERSON)
        .unwrap()
        .amount
        .quantity = match case {
        "affordable" | "unhelpful" => FUNDED_COINS,
        "unaffordable" => DOWNPAYMENT_ONLY_COINS,
        _ => return Err("unknown borrowing case".into()),
    };
    if case == "unhelpful" {
        w.definitions
            .iter_mut()
            .find(|d| d.id == GROW)
            .unwrap()
            .outputs
            .iter_mut()
            .find(|a| a.resource == GRAIN)
            .unwrap()
            .quantity = LOW_CROP_OUTPUT;
    }
    Ok((w, s))
}
