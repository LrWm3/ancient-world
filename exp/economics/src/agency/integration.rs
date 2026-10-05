//! One finite economy for exercising organizational decisions through shared settlement.
use super::{Command, Controller, Program, objectives::*, scenario};
use crate::{
    activities::{Target, WorkOrder},
    commitments, credit, financial_reporting, forward, household_governance as h, households,
    minting::*,
    model::*,
    opportunities::{Action, HOUSEHOLD_TYPE, PERSON_TYPE, STATE_TYPE},
};

pub const HOUSEHOLD: AgentId = 800;
pub const GROWER: AgentId = 89;
pub const GROW: DefinitionId = 110;
pub const SEED: ResourceId = 110;
pub const PLOT: AssetId = 777;
pub const RUN_MONTHS: u32 = 14;
const MEMBER_HOURS: i32 = 5;
const JOINT_WORK_HOURS: i32 = 6;
const FOOD_OUTPUT: i32 = 4;
const NONESSENTIAL_OUTPUT: i32 = 10;
const OPENING_SEED: i32 = 4;
const TERM_MONTHS: u32 = 2;
const STATE_LOAN: i32 = 6;
const LOAN_MONTHS: u32 = 6;
const DELIVERY_MONTH: u32 = 4;
const DELIVERY_QUANTITY: i32 = 2;
const LAND_DUES: i32 = 1;
const STORAGE_CAPACITY: i32 = 32;
const WOOD_TARGET: i32 = 100;
const FORECAST_MONTHS: u32 = 4;
const SUPPORT_RESERVE_MONTHS: u32 = 1;
const LAND_TERM_MONTHS: u32 = 24;
const SEED_INPUT: i32 = 1;
const SEED_OUTPUT: i32 = 2;
const NUTRITION_PER_MONTH: i32 = 1;
const OPENING_PLOT_COST: i128 = 10;

pub fn scenario() -> Result<(World, State), String> {
    let (mut w, mut s) = scenario::minting()?;
    w.agents.push(Agent {
        id: GROWER,
        name: "household grower".into(),
    });
    w.resources.extend([
        Resource {
            id: NUTRITION,
            name: "nutrition".into(),
            kind: ResourceKind::Fulfillment,
        },
        Resource {
            id: SEED,
            name: "seed".into(),
            kind: ResourceKind::Stock,
        },
    ]);
    w.participants.push(Participant {
        agent: GROWER,
        capacity: Amount::new(HOURS, MEMBER_HOURS),
        needs: vec![Requirement {
            resource: NUTRITION,
            quantity: NUTRITION_PER_MONTH,
            priority: 0,
        }],
    });
    let supplier = w
        .participants
        .iter_mut()
        .find(|p| p.agent == SUPPLIER)
        .unwrap();
    supplier.capacity.quantity = MEMBER_HOURS;
    supplier.needs = vec![Requirement {
        resource: NUTRITION,
        quantity: NUTRITION_PER_MONTH,
        priority: 0,
    }];
    w.storage.capacities.insert(GROWER, STORAGE_CAPACITY);
    w.storage.weights.insert(SEED, 1);
    s.balances.insert((GROWER, SEED), OPENING_SEED);
    w.assets.push(Asset {
        id: PLOT,
        owner: ISSUER,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: 77,
        holder: GROWER,
        asset: PLOT,
        from: 1,
        through: LAND_TERM_MONTHS,
        output_owner: GROWER,
    });
    w.agreements.push(commitments::Agreement {
        id: 77,
        right: 77,
        creditor: ISSUER,
        debtor: GROWER,
        activated: 1,
        payment: Amount::new(COIN, LAND_DUES),
    });
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
                entry_inputs: vec![Amount::new(WHEAT, NUTRITION_PER_MONTH)],
                monthly_services: vec![],
            }],
            outputs: vec![Amount::new(NUTRITION, NUTRITION_PER_MONTH)],
        },
        ProcessDefinition {
            id: GROW,
            name: "joint plot work".into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: Some(1),
            stages: vec![Stage {
                name: "grow".into(),
                months: 1,
                entry_inputs: vec![Amount::new(SEED, SEED_INPUT)],
                monthly_services: vec![Amount::new(HOURS, JOINT_WORK_HOURS)],
            }],
            outputs: vec![
                Amount::new(WHEAT, FOOD_OUTPUT),
                Amount::new(SEED, SEED_OUTPUT),
            ],
        },
    ]);
    let gather = w.definitions.iter_mut().find(|d| d.id == GATHER).unwrap();
    gather.stages[0].monthly_services[0].quantity = JOINT_WORK_HOURS;
    gather.outputs[0].quantity = NONESSENTIAL_OUTPUT;
    w.scheduled_starts.retain(|p| p.agent == ISSUER);
    w.activities.orders.push(WorkOrder {
        agent: SUPPLIER,
        definition: GATHER,
        priority: 0,
        target: Target::Stock(Amount::new(FIREWOOD, WOOD_TARGET)),
    });
    let law = w.transaction_policy.as_mut().unwrap();
    law.agent_types.insert(GROWER, PERSON_TYPE);
    law.permissions.extend([
        (PERSON_TYPE, Action::Process(EAT)),
        (PERSON_TYPE, Action::Process(GROW)),
        (PERSON_TYPE, Action::FoundHousehold),
        (PERSON_TYPE, Action::Lend),
        (STATE_TYPE, Action::Borrow),
        (HOUSEHOLD_TYPE, Action::StockTrade),
    ]);
    // Retain explicit legal/financial agreements: policy selection does not invent consent.
    let offer = w.transaction_policy.as_ref().unwrap().membership_offers[0].id;
    s.memberships.insert(
        (GROWER, ISSUER, crate::membership::CITIZEN),
        crate::membership::Agreement {
            member: GROWER,
            organization: ISSUER,
            role: crate::membership::CITIZEN,
            source_offer: offer,
            accepted_month: s.month,
        },
    );
    w.state_governance.as_mut().unwrap().constitution.leadership = h::Leadership::Elected;
    let mut governance = h::Governance::elected(SUPPLIER, TERM_MONTHS);
    governance.charter.initial_policy = h::Policy::NetOutput;
    households::form(
        &mut w,
        &s,
        households::Agreement {
            id: 1,
            agent: HOUSEHOLD,
            adults: vec![SUPPLIER, GROWER],
            formed: s.month,
            governance,
            dwelling_process: None,
            admission: None,
            membership: vec![],
            asset_sales: vec![],
            equipment_retirements: vec![],
            support: vec![],
        },
    )?;
    let mut home = scenario::config(
        vec![Objective {
            scope: Scope::Members,
            metric: Metric::NeedDeficit(NUTRITION),
        }],
        [(
            1,
            Program {
                name: "feed members before discretionary output".into(),
                commands: vec![Command::HouseholdPolicy(h::Policy::NeedsFirst)],
            },
        )]
        .into(),
    );
    home.horizon = FORECAST_MONTHS;
    for id in [SUPPLIER, GROWER] {
        home.preferences.insert(id, home.objectives.clone());
    }
    w.agency.insert(HOUSEHOLD, Controller::new(home));
    let state = &mut w.agency.get_mut(&ISSUER).unwrap().config;
    for id in [SUPPLIER, GROWER, WORKER] {
        state.preferences.insert(id, state.objectives.clone());
    }
    w.lending.push(credit::Advance {
        id: 10,
        debtor: ISSUER,
        principal: STATE_LOAN,
        month: 1,
        collateral: None,
        priority: 0,
        terms: credit::LoanOffer {
            creditor: SUPPLIER,
            denomination: COIN,
            max_principal: STATE_LOAN,
            monthly_rate_bps: 0,
            term_months: LOAN_MONTHS,
            grace_months: 1,
        },
    });
    w.prepaid_deliveries.push(forward::direct::Terms {
        id: 20,
        seller: HOUSEHOLD,
        buyer: ISSUER,
        month: 2,
        due: DELIVERY_MONTH,
        goods: Amount::new(WHEAT, DELIVERY_QUANTITY),
        prepayment: Amount::new(COIN, DELIVERY_QUANTITY),
    });
    Ok((w, s))
}

/// Same opening economy, with explicitly permitted commitment preparation and
/// independently signed member surplus offers. The baseline remains available.
pub fn commitment_scenario() -> Result<(World, State), String> {
    let (mut w, s) = scenario()?;
    let policy = h::Policy::NeedsThenCommitments {
        months: FORECAST_MONTHS,
    };
    let household = &mut w.households[0];
    household
        .governance
        .constitution
        .permitted_policies
        .insert(policy);
    household.governance.charter.accept_payment_support = true;
    w.agency
        .get_mut(&HOUSEHOLD)
        .unwrap()
        .config
        .programs
        .get_mut(&1)
        .unwrap()
        .commands = vec![Command::HouseholdPolicy(policy)];
    let config = &mut w.agency.get_mut(&HOUSEHOLD).unwrap().config;
    config.programs.get_mut(&1).unwrap().name =
        "feed members and prepare accepted commitments".into();
    config.objectives.push(Objective {
        scope: Scope::Organization,
        metric: Metric::FundingGap {
            resource: WHEAT,
            months: FORECAST_MONTHS,
        },
    });
    for member in [SUPPLIER, GROWER] {
        config.preferences.insert(member, config.objectives.clone());
    }
    for member in [SUPPLIER, GROWER] {
        households::support::authorize(
            &mut w,
            &s,
            HOUSEHOLD,
            member,
            households::support::Mandate {
                member,
                resource: WHEAT,
                from: s.month,
                through: RUN_MONTHS,
                revoked_from: None,
                reserve_months: SUPPORT_RESERVE_MONTHS,
                private_reserve: NUTRITION_PER_MONTH,
                household_target: DELIVERY_QUANTITY,
                monthly_limit: DELIVERY_QUANTITY,
            },
        )?;
    }
    Ok((w, s))
}

/// Deliberately separate entity statements, with supplied opening carrying costs.
pub fn audit(w: &World, s: &State) -> Result<financial_reporting::Audit, String> {
    financial_reporting::Audit::with_opening(w, s, COIN, opening(w, s))
}

pub fn opening(w: &World, s: &State) -> financial_reporting::Opening {
    use crate::process_accounting::{Costs, Output};
    financial_reporting::Opening {
        assets: w.assets.iter().map(|a| (a.id, OPENING_PLOT_COST)).collect(),
        inventory: s
            .balances
            .iter()
            .filter(|((_, r), q)| *r != COIN && **q > 0)
            .map(|(k, q)| (*k, i128::from(*q)))
            .collect(),
        processes: Some(Costs {
            output_weights: [(
                GROW,
                [
                    (Output::Stock(WHEAT), FOOD_OUTPUT as u32),
                    (Output::Stock(SEED), SEED_OUTPUT as u32),
                ]
                .into(),
            )]
            .into(),
            ..Default::default()
        }),
        dues: Some(crate::dues_accounting::Valuation(
            w.agreements.iter().map(|a| (a.id, 1)).collect(),
        )),
        issuance: Some(crate::issuance_accounting::Policy::NonRedeemableEquity),
        ..Default::default()
    }
}
