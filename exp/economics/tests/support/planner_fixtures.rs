#![allow(dead_code)]
use economics_compute_smoke::{
    composition::Scope,
    compute::Backend,
    financial_reporting::{Audit, Opening},
    household_governance::Governance,
    households::{self, Agreement},
    model::*,
    opportunities::{Action, PERSON_TYPE},
    process_accounting::{Costs, Output},
    scenario::*,
    simulation::Simulation,
};
pub const HOME: AgentId = 10000;
pub const CASES: &[&str] = &[
    "B1",
    "B2-no-land",
    "B2-no-seed",
    "B2-scarcity",
    "B3",
    "B5",
    "B5-one-seed",
];
pub fn fixture(case: &str, backend: Backend) -> Result<(Simulation, Scope), String> {
    if case.starts_with("B5") {
        let (mut w, mut s) = economics_compute_smoke::competition::scenario(2, 7)?;
        w.competition = None;
        w.priority = Priority::ContinuingFirst;
        w.decision_horizon = None;
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((PERSON_TYPE, Action::FoundHousehold));
        w.resources.push(Resource {
            id: TOKEN,
            name: "coin".into(),
            kind: ResourceKind::Stock,
        });
        for p in &mut w.participants {
            p.capacity.quantity = 5;
            w.storage.capacities.insert(p.agent, 40);
            s.balances.insert((p.agent, SEED), 0);
        }
        w.storage.weights.extend([(GRAIN, 1), (SEED, 1), (FUEL, 1)]);
        let adults: Vec<_> = w.participants.iter().map(|p| p.agent).collect();
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: adults.clone(),
                governance: Governance::contributed(PERSON),
                formed: 1,
                dwelling_process: None,
                admission: None,
                membership: vec![],
                asset_sales: vec![],
                equipment_retirements: vec![],
                support: vec![],
            },
        )?;
        s.balances.extend([
            ((HOME, SEED), if case == "B5-one-seed" { 1 } else { 2 }),
            ((HOME, GRAIN), 10),
            ((HOME, TOKEN), 4),
        ]);
        return Ok((
            Simulation::new(w, s, backend)?,
            Scope::Household {
                agent: HOME,
                consenting_members: adults.into_iter().collect(),
            },
        ));
    }
    let (mut w, mut s) = economics_compute_smoke::membership::scenario()?;
    w.decision_horizon = Some(12);
    match case {
        "B1" => {}
        "B2-no-land" => {
            w.access_offers.clear();
            w.rights.clear();
        }
        "B2-no-seed" => {
            s.balances.insert((PERSON, SEED), 0);
            w.access_offers.clear();
            w.rights[0].holder = PERSON;
        }
        "B2-scarcity" => {
            w.access_offers.clear();
            w.rights.clear();
            s.balances.insert((PERSON, SEED), 0);
            for p in &mut w.pools {
                p.monthly_regeneration = 0;
                s.balances.insert(p.account, 0);
            }
        }
        "B3" => {
            s.balances.insert((PERSON, SEED), 2);
            w.definitions
                .iter_mut()
                .find(|d| d.id == GROW)
                .unwrap()
                .stages[0]
                .monthly_services[0]
                .quantity = 1;
            let mut second = w.definition(GROW).clone();
            second.id = 99;
            second.name = "late labor intensive crop".into();
            second.stages.last_mut().unwrap().monthly_services[0].quantity = 3;
            w.definitions.push(second);
            w.transaction_policy
                .as_mut()
                .unwrap()
                .membership_permissions
                .insert((
                    economics_compute_smoke::membership::CITIZEN,
                    Action::Process(99),
                ));
            let mut asset = w.assets[0].clone();
            asset.id += 1;
            w.assets.push(asset);
            let mut right = w.rights[0].clone();
            right.id += 1;
            right.asset += 1;
            w.rights.push(right);
            let mut offer = w.access_offers[0].clone();
            offer.id += 1;
            offer.right += 1;
            w.access_offers.push(offer);
        }
        _ => return Err("unknown planner fixture".into()),
    }
    Ok((Simulation::new(w, s, backend)?, Scope::Person(PERSON)))
}
pub fn audit(sim: &Simulation) -> Result<Audit, String> {
    let w = &sim.world;
    let s = &sim.state;
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| **q > 0 && *r != TOKEN)
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            exchange_values: w
                .resources
                .iter()
                .filter(|r| r.kind == ResourceKind::Stock && r.id != TOKEN)
                .map(|r| (r.id, 1))
                .collect(),
            processes: Some(Costs {
                output_weights: [(
                    GROW,
                    [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                )]
                .into(),
                ..Costs::default()
            }),
            dues: Some(economics_compute_smoke::dues_accounting::Valuation(
                w.access_offers.iter().map(|a| (a.id, 1)).collect(),
            )),
            ..Opening::default()
        },
    )
}
pub fn acquire(sim: &mut Simulation) -> Result<(), String> {
    while sim.state.phase != Phase::Acquire {
        sim.step()?;
    }
    Ok(())
}
