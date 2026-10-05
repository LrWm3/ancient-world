//! Small controls sharing the same decision loop across legal and economic work.
use super::*;
use crate::{
    membership::{self, CITIZEN},
    opportunities::{Action, PERSON_TYPE},
    state_governance as g,
};
use objectives::{Metric, Scope};

const HORIZON_MONTHS: u32 = 8;
const REVIEW_MONTHS: u32 = 1;
const HOLD_MONTHS: u32 = 1;
const FOUNDING_TERM_MONTHS: u32 = 2;
const FARM_OPENING_GRAIN: i32 = 8;
const MINT_RESERVE: i32 = 14;
const SUPPLIER_METAL: i32 = 10;
const WORK_TREASURY: i32 = 4;
const WORK_RESERVE: i32 = 2;
const WORK_START: u32 = 2;
const WORK_END: u32 = 3;
const WORK_HOURS: i32 = 2;
const WAGE_PER_HOUR: i32 = 1;

pub fn config(objectives: Vec<Objective>, programs: BTreeMap<u32, Program>) -> Config {
    Config {
        horizon: HORIZON_MONTHS,
        review_every: REVIEW_MONTHS,
        minimum_tenure: HOLD_MONTHS,
        emergency: None,
        objectives,
        preferences: BTreeMap::new(),
        programs,
    }
}

pub fn farming() -> Result<(World, State), String> {
    use crate::scenario::*;
    let (mut w, mut s) = g::scenario::pair()?;
    let g = w.state_governance.as_mut().unwrap();
    g.charter.initial_policy = g::scenario::PAUSE_ADMISSIONS;
    g.ballots.clear();
    for agent in [PERSON, crate::competition::SECOND_PERSON] {
        s.balances.insert((agent, GRAIN), FARM_OPENING_GRAIN);
    }
    let c = config(
        vec![
            Objective {
                scope: Scope::Members,
                metric: Metric::Deaths,
            },
            Objective {
                scope: Scope::Members,
                metric: Metric::NeedDeficit(NUTRITION),
            },
            Objective {
                scope: Scope::Members,
                metric: Metric::NeedDeficit(WARMTH),
            },
        ],
        [
            (
                0,
                Program {
                    name: "open admissions".into(),
                    commands: vec![Command::StatePolicy(g::scenario::OPEN)],
                },
            ),
            (
                1,
                Program {
                    name: "pause admissions".into(),
                    commands: vec![Command::StatePolicy(g::scenario::PAUSE_ADMISSIONS)],
                },
            ),
        ]
        .into(),
    );
    // Supplied stable preferences, actual votes and policy choices are generated.
    let mut c = c;
    for agent in [PERSON, crate::competition::SECOND_PERSON] {
        c.preferences.insert(agent, c.objectives.clone());
    }
    w.agency.insert(STATE_AGENT, Controller::new(c));
    Ok((w, s))
}

pub fn govern(w: &mut World, s: &mut State, governor: AgentId, citizens: &[AgentId]) {
    let p = w.transaction_policy.as_mut().unwrap();
    let state = p.authority;
    p.membership_offers.push(membership::Offer {
        id: 900,
        organization: state,
        role: CITIZEN,
        eligible_type: PERSON_TYPE,
    });
    p.permissions.insert((PERSON_TYPE, Action::Membership));
    for &member in citizens {
        s.memberships.insert(
            (member, state, CITIZEN),
            membership::Agreement {
                member,
                organization: state,
                role: CITIZEN,
                source_offer: 900,
                accepted_month: s.month,
            },
        );
    }
    w.state_governance = Some(g::Governance {
        formation: None,
        state,
        formed: s.month,
        constitution: g::Constitution {
            leadership: crate::governance::Leadership::FixedFounder,
            policies: [(
                0,
                g::LegalPolicy {
                    name: "ordinary law".into(),
                    prohibited: BTreeSet::new(),
                },
            )]
            .into(),
        },
        charter: g::Charter {
            founder: governor,
            term_months: FOUNDING_TERM_MONTHS,
            election: Default::default(),
            initial_policy: 0,
        },
        ballots: vec![],
        changes: vec![],
    });
}

pub fn minting() -> Result<(World, State), String> {
    use crate::minting::*;
    let (mut w, mut s) = crate::minting::order_scenario("normal")?;
    govern(&mut w, &mut s, SUPPLIER, &[SUPPLIER, WORKER]);
    s.balances.insert((SUPPLIER, METAL), SUPPLIER_METAL);
    let mut c = config(
        vec![Objective {
            scope: Scope::Organization,
            metric: Metric::Reserve {
                resource: COIN,
                target: MINT_RESERVE,
            },
        }],
        [(
            1,
            Program {
                name: "fund another mint batch".into(),
                commands: vec![Command::StartProcess(MINT)],
            },
        )]
        .into(),
    );
    c.horizon = 4;
    w.agency.insert(ISSUER, Controller::new(c));
    Ok((w, s))
}

pub fn household() -> Result<(World, State), String> {
    use crate::{household_governance as h, scenario::*};
    let (mut w, mut s) = h::scenario::pair()?;
    govern(&mut w, &mut s, PERSON, &[PERSON, PERSON + 1]);
    let c = config(
        vec![Objective {
            scope: Scope::Members,
            metric: Metric::NeedDeficit(NUTRITION),
        }],
        [(
            1,
            Program {
                name: "feed members first".into(),
                commands: vec![Command::HouseholdPolicy(h::Policy::NeedsFirst)],
            },
        )]
        .into(),
    );
    w.agency.insert(h::scenario::HOUSEHOLD, Controller::new(c));
    // State separately protects its citizens using its own constitutional menu.
    let c = config(
        vec![Objective {
            scope: Scope::Members,
            metric: Metric::NeedDeficit(NUTRITION),
        }],
        [(
            0,
            Program {
                name: "ordinary law".into(),
                commands: vec![Command::StatePolicy(0)],
            },
        )]
        .into(),
    );
    w.agency.insert(STATE_AGENT, Controller::new(c));
    Ok((w, s))
}

/// The state owns no labor endowment. An already accepted wage agreement supplies
/// its work capacity; the shared controller decides how to use those paid hours.
pub fn public_work() -> Result<(World, State), String> {
    use crate::{
        activities::{Target, WorkOrder},
        employment,
        minting::*,
    };
    let (mut w, mut s) = minting()?;
    w.minting = None;
    w.scheduled_starts.clear();
    w.activities.orders.clear();
    s.balances.insert((ISSUER, COIN), WORK_TREASURY);
    w.transaction_policy.as_mut().unwrap().permissions.extend([
        (crate::opportunities::STATE_TYPE, Action::Process(GATHER)),
        (crate::opportunities::STATE_TYPE, Action::CapacityTrade),
        (PERSON_TYPE, Action::CapacityTrade),
    ]);
    w.employment.push(employment::Terms {
        id: 1,
        employer: ISSUER,
        worker: WORKER,
        from: WORK_START,
        through: WORK_END,
        capacity: Amount::new(HOURS, WORK_HOURS),
        wage_per_unit: Amount::new(COIN, WAGE_PER_HOUR),
        on_arrears: employment::ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    let mut c = config(
        vec![
            Objective {
                scope: Scope::Organization,
                metric: Metric::FundingGap {
                    resource: COIN,
                    months: 1,
                },
            },
            Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: FIREWOOD,
                    target: WORK_RESERVE,
                },
            },
        ],
        [(
            1,
            Program {
                name: "use contracted hours for firewood reserve".into(),
                commands: vec![Command::WorkTargets(vec![WorkOrder {
                    agent: ISSUER,
                    definition: GATHER,
                    priority: 0,
                    target: Target::Stock(Amount::new(FIREWOOD, WORK_RESERVE)),
                }])],
            },
        )]
        .into(),
    );
    c.horizon = 4;
    w.agency.insert(ISSUER, Controller::new(c));
    Ok((w, s))
}
