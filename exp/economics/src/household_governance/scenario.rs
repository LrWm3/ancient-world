//! Small lawful founding/election/policy fixture, with supplied political choices.
use crate::{
    activities::{Target, WorkOrder},
    household_governance::{self as g, elections},
    laws,
    model::*,
    opportunities::{self as o, Action},
    scenario::*,
};

pub const HOUSEHOLD: AgentId = 10_000;
pub const FOOD_PROCESS: DefinitionId = 99;
pub const TERM_MONTHS: u32 = 2;
pub const RUN_MONTHS: u32 = 6;
const LABOR_PER_PERSON: i32 = 5;
const JOB_LABOR: i32 = 6;
const REPAIR_OUTPUT_QUANTITY: i32 = 10;
const FOOD_OUTPUT_QUANTITY: i32 = 2;

pub fn pair() -> Result<(World, State), String> {
    let (mut w, mut s) = baseline();
    w.assets.clear();
    w.rights.clear();
    w.definitions.retain(|d| d.id != GROW);
    w.agents.push(Agent {
        id: PERSON + 1,
        name: "second adult".into(),
    });
    let mut second = w.participants[0].clone();
    second.agent = PERSON + 1;
    w.participants.push(second);
    for p in &mut w.participants {
        p.capacity.quantity = LABOR_PER_PERSON;
    }
    s.balances.clear();
    s.balances.insert((PERSON + 1, SEED), 1);
    w.resources.push(Resource {
        id: TOKEN,
        name: "reporting coin".into(),
        kind: ResourceKind::Stock,
    });
    let repair = w.definitions.iter_mut().find(|d| d.id == REPAIR).unwrap();
    repair.stages[0].monthly_services = vec![Amount::new(LABOR, JOB_LABOR)];
    repair.outputs = vec![Amount::new(REPAIR_OUTPUT, REPAIR_OUTPUT_QUANTITY)];
    let mut food = repair.clone();
    food.id = FOOD_PROCESS;
    food.name = "prepare food and retain seed".into();
    food.stages[0].entry_inputs = vec![Amount::new(SEED, 1)];
    food.outputs = vec![
        Amount::new(GRAIN, FOOD_OUTPUT_QUANTITY),
        Amount::new(SEED, 1),
    ];
    w.definitions.push(food);
    w.activities.orders = vec![WorkOrder {
        agent: PERSON,
        definition: REPAIR,
        priority: 0,
        target: Target::Stock(Amount::new(
            REPAIR_OUTPUT,
            REPAIR_OUTPUT_QUANTITY * RUN_MONTHS as i32,
        )),
    }];
    let rules = laws::households::Rules {
        leadership: [g::Leadership::Elected].into(),
        min_term_months: TERM_MONTHS,
        max_term_months: TERM_MONTHS,
        ..Default::default()
    };
    w.transaction_policy = Some(o::Policy {
        authority: STATE_AGENT,
        laws: vec![],
        agreement_forms: Some([laws::AgreementForm::Household].into()),
        agreement_limits: laws::AgreementLimits {
            household: Some(rules),
            ..Default::default()
        },
        membership_offers: vec![],
        membership_permissions: Default::default(),
        agent_types: [
            (STATE_AGENT, o::STATE_TYPE),
            (PERSON, o::PERSON_TYPE),
            (PERSON + 1, o::PERSON_TYPE),
        ]
        .into(),
        permissions: w
            .definitions
            .iter()
            .map(|d| (o::PERSON_TYPE, Action::Process(d.id)))
            .chain([(o::PERSON_TYPE, Action::FoundHousehold)])
            .collect(),
    });
    let mut governance = g::Governance::elected(PERSON, TERM_MONTHS);
    governance.charter.initial_policy = g::Policy::NetOutput;
    crate::households::form(
        &mut w,
        &s,
        crate::households::Agreement {
            id: 1,
            agent: HOUSEHOLD,
            adults: vec![PERSON, PERSON + 1],
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
    for term_start in [1 + TERM_MONTHS, 1 + TERM_MONTHS * 2] {
        for voter in [PERSON, PERSON + 1] {
            elections::cast(
                &mut w,
                &s,
                HOUSEHOLD,
                elections::Ballot {
                    term_start,
                    voter,
                    candidate: Some(PERSON + 1),
                },
            )?;
        }
    }
    Ok((w, s))
}

/// Supplied incoming governor instruction; caller invokes it at Open month three.
pub fn incoming_policy(w: &mut World, s: &State) -> Result<(), String> {
    if s.month != 1 + TERM_MONTHS || s.phase != Phase::Open {
        return Err("fixture governor instruction belongs at Open month three".into());
    }
    g::schedule_allocation(
        w,
        s,
        HOUSEHOLD,
        g::PolicyChange {
            month: s.month + 1,
            authorized_by: PERSON + 1,
            policy: g::Policy::NeedsFirst,
        },
        Some(g::TieBreak::MemberId),
    )
}
