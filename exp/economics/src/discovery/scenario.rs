//! Physical endowments and institutional/technology rules, without signed deals.
use super::*;
use crate::{agency::integration, minting::*};

pub const RUN_MONTHS: u32 = 14;
const ANNUAL_DUES: i32 = 1;
const LAND_DURATION: u32 = 24;
const FORECAST_MONTHS: u32 = 4;
const WHEAT_RESERVE: i32 = 4;
const LOAN_MONTHS: u32 = 4;
const MONTHLY_INTEREST_BPS: u32 = 1_000;
const DELIVERY_MONTHS: u32 = 2;
const WHEAT_COIN_VALUE: i32 = 1;
const SURPLUS_STATE_COIN: i32 = 20;
const SURPLUS_CROP_OUTPUT: i32 = 6;
const SURPLUS_WHEAT_RESERVE: i32 = 8;
const CIRCULATION_TREASURY: i32 = 6;
const CIRCULATION_GRANARY: i32 = 16;
const WORKER_FOOD_BUFFER: i32 = 3;
const COMPETING_LABOR_HOURS: i32 = 1;
const WORKER_MONTHLY_NUTRITION: i32 = 1;
const FINANCED_OPENING_TREASURY: i32 = 0;
const FINANCED_INTEREST_BPS: u32 = 0;
const FINANCED_ALTERNATIVE_TERM: u32 = 8;
const FINANCED_LABOR_LOT: i32 = 5;

pub fn scenario() -> Result<(World, State), String> {
    // Reuse physical catalog/endowments of the integration control, then remove
    // every arranged relationship, opportunity instance and operational choice.
    let (mut w, mut s) = integration::scenario()?;
    let household = w.households.remove(0);
    let state = w.state_governance.take().unwrap();
    let mut objectives = w.agency[&ISSUER].config.objectives.clone();
    objectives.push(agency::objectives::Objective {
        scope: agency::objectives::Scope::Organization,
        metric: agency::objectives::Metric::Reserve {
            resource: WHEAT,
            target: WHEAT_RESERVE,
        },
    });
    w.agency.clear();
    w.agents.retain(|a| a.id != household.agent);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .agent_types
        .remove(&household.agent);
    w.lending.clear();
    w.prepaid_deliveries.clear();
    w.agreements.clear();
    w.rights.clear();
    w.access_offers.clear();
    w.scheduled_starts.clear();
    w.activities.orders.clear();
    s.memberships.clear();
    let law = w.transaction_policy.as_mut().unwrap();
    law.membership_offers.clear();
    law.permissions
        .insert((opportunities::PERSON_TYPE, Action::Membership));
    for action in [
        Action::LandAccess,
        Action::Process(integration::GROW),
        Action::FoundHousehold,
    ] {
        law.permissions
            .remove(&(opportunities::PERSON_TYPE, action));
        law.membership_permissions
            .insert((crate::membership::CITIZEN, action));
    }
    let policy = w.minting.as_mut().unwrap().order_policy.as_mut().unwrap();
    policy.month = 0; // idle until an authorized agent chooses a dated target
    policy.additional_months.clear();
    policy.quotes.clear();
    let mut constitution = household.governance.constitution;
    constitution
        .permitted_policies
        .insert(h::Policy::NeedsThenCommitments {
            months: FORECAST_MONTHS,
        });
    w.discovery = Some(super::Config {
        enabled: true,
        horizon: FORECAST_MONTHS,
        land: Some(LandRule {
            asset_kind: 1,
            duration: LAND_DURATION,
            annual_payment: Amount::new(WHEAT, ANNUAL_DUES),
        }),
        land_allocation: crate::allocation::Policy::StablePriority,
        land_seed: 0,
        household: Some(HouseholdRule {
            constitution,
            charter: household.governance.charter,
        }),
        state: Some(StateRule {
            constitution: state.constitution,
            term_months: state.charter.term_months,
            objectives,
        }),
        finance: Some(FinanceRule {
            alternative_loan_months: Default::default(),
            denomination: COIN,
            loan_months: LOAN_MONTHS,
            monthly_rate_bps: MONTHLY_INTEREST_BPS,
            delivery_months: DELIVERY_MONTHS,
            forward_horizon: None,
            unit_values: [(WHEAT, WHEAT_COIN_VALUE)].into(),
        }),
        through: 0,
        receipts: vec![],
        financial: vec![],
        supply: vec![],
        public_sales: false,
        private_sales: false,
    });
    Ok((w, s))
}

pub fn audit(w: &World, s: &State) -> Result<crate::financial_reporting::Audit, String> {
    let mut opening = integration::opening(w, s);
    opening
        .exchange_values
        .insert(WHEAT, i128::from(WHEAT_COIN_VALUE));
    crate::financial_reporting::Audit::with_opening(w, s, COIN, opening)
}

/// Finite wages-to-food control. Other workers cannot fill a two-hour mint lot;
/// agriculture, household formation and borrowing are outside this fixture.
pub fn circulation() -> Result<(World, State), String> {
    let (mut w, mut s) = scenario()?;
    let c = w.discovery.as_mut().unwrap();
    c.public_sales = true;
    c.finance = None;
    c.household = None;
    c.land = None;
    s.balances.insert((ISSUER, COIN), CIRCULATION_TREASURY);
    s.balances.insert((ISSUER, WHEAT), CIRCULATION_GRANARY);
    s.balances.insert((WORKER, COIN), 0);
    s.balances.insert((WORKER, WHEAT), WORKER_FOOD_BUFFER);
    for p in &mut w.participants {
        if p.agent != ISSUER && p.agent != WORKER {
            p.capacity.quantity = COMPETING_LABOR_HOURS;
        }
        if p.agent == WORKER {
            p.needs = vec![Requirement {
                resource: NUTRITION,
                quantity: WORKER_MONTHLY_NUTRITION,
                priority: 0,
            }];
        }
    }
    Ok((w, s))
}

/// A materially richer buyer and harvest make a future delivery useful to both sides.
pub fn surplus() -> Result<(World, State), String> {
    let (mut w, mut s) = scenario()?;
    s.balances.insert((ISSUER, COIN), SURPLUS_STATE_COIN);
    w.definitions
        .iter_mut()
        .find(|d| d.id == integration::GROW)
        .unwrap()
        .outputs
        .iter_mut()
        .find(|a| a.resource == WHEAT)
        .unwrap()
        .quantity = SURPLUS_CROP_OUTPUT;
    w.discovery
        .as_mut()
        .unwrap()
        .state
        .as_mut()
        .unwrap()
        .objectives
        .insert(
            0,
            agency::objectives::Objective {
                scope: agency::objectives::Scope::Organization,
                metric: agency::objectives::Metric::Reserve {
                    resource: WHEAT,
                    target: SURPLUS_WHEAT_RESERVE,
                },
            },
        );
    Ok((w, s))
}

/// Keep farming households and credit in the finite wages-to-food control.
/// Five-hour mint work cannot use a household member's four uncommitted hours;
/// the independent worker can supply a whole lot. Technology and lot sizes are
/// scenario assumptions, not a change to the marketplace's allocation policy.
pub fn financed_circulation() -> Result<(World, State), String> {
    let (mut w, mut s) = scenario()?;
    let c = w.discovery.as_mut().unwrap();
    c.public_sales = true;
    let finance = c.finance.as_mut().unwrap();
    finance.monthly_rate_bps = FINANCED_INTEREST_BPS;
    finance
        .alternative_loan_months
        .insert(FINANCED_ALTERNATIVE_TERM);
    s.balances.insert((ISSUER, COIN), FINANCED_OPENING_TREASURY);
    s.balances.insert((ISSUER, WHEAT), CIRCULATION_GRANARY);
    s.balances.insert((WORKER, COIN), 0);
    s.balances.insert((WORKER, WHEAT), WORKER_FOOD_BUFFER);
    let worker = w
        .participants
        .iter_mut()
        .find(|p| p.agent == WORKER)
        .unwrap();
    worker.capacity.quantity = FINANCED_LABOR_LOT;
    worker.needs = vec![Requirement {
        resource: NUTRITION,
        quantity: WORKER_MONTHLY_NUTRITION,
        priority: 0,
    }];
    w.definitions
        .iter_mut()
        .find(|d| d.id == MINT)
        .unwrap()
        .stages[0]
        .monthly_services
        .iter_mut()
        .find(|a| a.resource == HOURS)
        .unwrap()
        .quantity = FINANCED_LABOR_LOT;
    w.marketplaces
        .iter_mut()
        .find(|v| v.agent == VENUE)
        .unwrap()
        .markets
        .iter_mut()
        .find(|m| m.goods.resource == HOURS)
        .unwrap()
        .goods
        .quantity = FINANCED_LABOR_LOT;
    Ok((w, s))
}
