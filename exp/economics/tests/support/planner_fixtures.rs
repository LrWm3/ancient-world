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

/// Frozen per-person endowments; plot count varies independently. Shared wood
/// scales with population in the abundant control rather than duplicating it.
pub fn persons(people: u32, plots: u32, backend: Backend) -> Result<Simulation, String> {
    if !(2..=32).contains(&people) || plots == 0 || plots > people {
        return Err("person fixture needs 2..32 people and 1..people plots".into());
    }
    let (mut w, mut s) = economics_compute_smoke::competition::scenario(plots.min(2), 7)?;
    w.priority = Priority::ContinuingFirst;
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    for offset in 2..people {
        let agent = PERSON + offset;
        w.agents.push(Agent {
            id: agent,
            name: format!("person {agent}"),
        });
        let mut participant = w.participants[0].clone();
        participant.agent = agent;
        w.participants.push(participant);
        let rules: Vec<_> = w
            .condition_rules
            .iter()
            .filter(|r| r.subject == PERSON)
            .cloned()
            .collect();
        for mut r in rules {
            r.subject = agent;
            w.condition_rules.push(r);
        }
        let stocks: Vec<_> = s
            .balances
            .iter()
            .filter(|((a, _), _)| *a == PERSON)
            .map(|((_, r), q)| (*r, *q))
            .collect();
        for (r, q) in stocks {
            s.balances.insert((agent, r), q);
        }
        if let Some(capacity) = w.storage.capacities.get(&PERSON).copied() {
            w.storage.capacities.insert(agent, capacity);
        }
        let policy = w.transaction_policy.as_mut().unwrap();
        policy
            .agent_types
            .insert(agent, policy.agent_types[&PERSON]);
    }
    for offset in 2..plots {
        let mut asset = w.assets[0].clone();
        asset.id += offset;
        w.assets.push(asset);
        let mut right = w.rights[0].clone();
        right.id += offset;
        right.asset += offset;
        w.rights.push(right);
        let mut offer = w.access_offers[0].clone();
        offer.id += offset;
        offer.right += offset;
        w.access_offers.push(offer);
    }
    w.open_access_offers = w.access_offers.iter().map(|a| a.id).collect();
    for pool in &mut w.pools {
        pool.capacity = 12 * people as i32;
        pool.monthly_regeneration = people as i32;
        s.balances.insert(pool.account, pool.capacity);
    }
    Simulation::new(w, s, backend)
}

/// B4b: finite stationary grain merchant also needs wood. No production or
/// omniscient future decisions are delegated to that counterparty by the planner.
pub fn wood_market(backend: Backend, buyer_present: bool) -> Result<(Simulation, Scope), String> {
    use economics_compute_smoke::{marketplace::Side, production_market::WOOD_MARKET};
    let (mut w, mut s) = economics_compute_smoke::production_market::reciprocal_scenario(true);
    w.production_market = None;
    w.participants.retain(|p| [PERSON, 89].contains(&p.agent));
    w.techniques.clear();
    w.practice_rules.clear();
    s.practice.clear();
    w.definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .enabled = false;
    let wood = w
        .definitions
        .iter_mut()
        .find(|d| d.id == PREPARE_FUEL)
        .unwrap();
    wood.outputs = vec![Amount::new(FUEL, 3)];
    wood.stages[0].monthly_services = vec![Amount::new(LABOR, 1)];
    for p in &mut w.participants {
        p.capacity.quantity = if p.agent == PERSON { 1 } else { 0 };
        w.storage.capacities.insert(p.agent, 200);
    }
    s.balances.clear();
    s.balances.extend([
        ((PERSON, GRAIN), 3),
        ((PERSON, FUEL), 2),
        ((PERSON, TOKEN), 0),
        ((89, GRAIN), 100),
        ((89, FUEL), 0),
        ((89, TOKEN), 100),
    ]);
    let m = w.town_market.as_mut().unwrap();
    m.adaptive = false;
    m.order_horizon = economics_compute_smoke::town_market::OrderHorizon::Aligned(2);
    for t in &mut m.traders {
        t.side = if t.trader.agent == PERSON {
            Side::Buy
        } else {
            Side::Sell
        };
        t.trader.limit = 1;
        t.trader.opening_quote = 1;
    }
    m.traders.retain(|t| [PERSON, 89].contains(&t.trader.agent));
    for l in &mut m.additional {
        assert_eq!(l.market, WOOD_MARKET);
        l.traders.retain(|t| [PERSON, 89].contains(&t.trader.agent));
        for t in &mut l.traders {
            t.side = if t.trader.agent == PERSON {
                Side::Sell
            } else {
                Side::Buy
            };
            t.trader.limit = 1;
            t.trader.opening_quote = 1;
        }
        if !buyer_present {
            l.match_limit = Some(0);
        }
    }
    for v in &mut w.marketplaces {
        for m in &mut v.markets {
            m.goods.quantity = 1;
            m.price_tick = 1;
        }
    }
    add_condition_rules(&mut w, "dead");
    w.condition_rules.retain(|r| r.subject == PERSON);
    Ok((Simulation::new(w, s, backend)?, Scope::Person(PERSON)))
}

/// Two productive persons, finite coins, supplied rights and fixed unit prices.
/// Separate plot/woodland rights permit complementary production. No counterparty work
/// is injected into their private search. Trading-off changes only match limits.
pub fn trading_persons(backend: Backend, trading: bool) -> Result<Simulation, String> {
    use economics_compute_smoke::{marketplace::Side, production_market::WOOD_MARKET};
    let (mut w, mut s) = economics_compute_smoke::production_market::reciprocal_scenario(true);
    w.production_market = None;
    w.participants.retain(|p| [PERSON, 89].contains(&p.agent));
    w.condition_rules
        .retain(|r| [PERSON, 89].contains(&r.subject));
    w.techniques.clear();
    w.practice_rules.clear();
    s.practice.clear();
    w.rights.retain(|r| r.holder == 89);
    w.assets.retain(|a| a.id == 89);
    w.assets.push(Asset {
        id: PERSON,
        owner: STATE_AGENT,
        kind: 2,
    });
    w.rights.push(UseRight {
        id: PERSON,
        holder: PERSON,
        asset: PERSON,
        from: 1,
        through: 240,
        output_owner: PERSON,
    });
    let crop = w.definitions.iter_mut().find(|d| d.id == GROW).unwrap();
    for stage in &mut crop.stages {
        stage.monthly_services = vec![Amount::new(LABOR, 1)];
    }
    crop.outputs = vec![Amount::new(GRAIN, 8), Amount::new(SEED, 1)];
    let wood = w
        .definitions
        .iter_mut()
        .find(|d| d.id == PREPARE_FUEL)
        .unwrap();
    wood.asset_kind = Some(2);
    wood.stages[0].monthly_services = vec![Amount::new(LABOR, 1)];
    wood.outputs = vec![Amount::new(FUEL, 3)];
    for p in &mut w.participants {
        p.capacity.quantity = 1;
        w.storage.capacities.insert(p.agent, 64);
    }
    s.balances.clear();
    s.balances.extend([
        ((PERSON, GRAIN), 3),
        ((PERSON, FUEL), 4),
        ((PERSON, TOKEN), 6),
        ((89, GRAIN), 6),
        ((89, FUEL), 3),
        ((89, SEED), 1),
        ((89, TOKEN), 6),
    ]);
    let m = w.town_market.as_mut().unwrap();
    m.adaptive = false;
    m.order_horizon = economics_compute_smoke::town_market::OrderHorizon::Aligned(2);
    m.match_limit = (!trading).then_some(0);
    m.traders.retain(|t| [PERSON, 89].contains(&t.trader.agent));
    for t in &mut m.traders {
        t.side = if t.trader.agent == PERSON {
            Side::Buy
        } else {
            Side::Sell
        };
        t.trader.limit = 1;
        t.trader.opening_quote = 1;
    }
    for l in &mut m.additional {
        assert_eq!(l.market, WOOD_MARKET);
        l.match_limit = (!trading).then_some(0);
        l.traders.retain(|t| [PERSON, 89].contains(&t.trader.agent));
        for t in &mut l.traders {
            t.side = if t.trader.agent == PERSON {
                Side::Sell
            } else {
                Side::Buy
            };
            t.trader.limit = 1;
            t.trader.opening_quote = 1;
        }
    }
    for venue in &mut w.marketplaces {
        for market in &mut venue.markets {
            market.goods.quantity = 1;
        }
    }
    add_condition_rules(&mut w, "dead");
    Simulation::new(w, s, backend)
}
