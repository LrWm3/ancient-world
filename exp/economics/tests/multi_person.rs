use economics_compute_smoke::{
    compute::Backend,
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
    storage,
};

fn new(name: &str, backend: Backend) -> Simulation {
    let (w, s) = named(name).unwrap();
    Simulation::new(w, s, backend).unwrap()
}

#[test]
fn four_people_have_separate_stocks_rights_taxes_and_storage() {
    let sim = new("four-person-exchange", Backend::Reference);
    assert_eq!(sim.world.agents.len(), 5);
    assert_eq!(sim.world.participants.len(), 4);
    for offset in 0..PERSON_COUNT {
        let agent = PERSON + offset;
        assert_eq!(sim.state.balance(agent, GRAIN), 5);
        assert_eq!(sim.state.balance(agent, SEED), 1);
        assert_eq!(sim.world.storage.capacities[&agent], 15);
        assert!(
            sim.world
                .rights
                .iter()
                .any(|r| r.holder == agent && r.asset == PLOT + offset && r.output_owner == agent)
        );
        assert!(
            sim.world
                .agreements
                .iter()
                .any(|a| a.debtor == agent && a.payment.quantity == 2)
        );
    }
    assert_eq!(sim.world.storage.capacities[&STATE_AGENT], 32);
    assert_eq!(sim.world.pools[0].monthly_regeneration, 1);
    let mut world = sim.world;
    let mut extra = world.participants[0].clone();
    extra.agent = 999;
    world.agents.push(Agent {
        id: 999,
        name: "fifth person".into(),
    });
    world.participants.push(extra);
    assert!(Simulation::new(world, sim.state, Backend::Reference).is_err());
}

fn harvest_pressure(tokens: i32, state_space: i32) -> Simulation {
    let mut sim = new("four-person-exchange", Backend::Reference);
    let (_, single) = named("forage-harvest").unwrap();
    sim.state.month = single.month;
    let crop = single
        .processes
        .values()
        .find(|p| p.definition == GROW && p.status == Status::Active)
        .unwrap();
    for offset in 0..PERSON_COUNT {
        let agent = PERSON + offset;
        for ((owner, r), q) in &single.balances {
            if *owner == PERSON {
                sim.state.balances.insert((agent, *r), *q);
            }
        }
        sim.state.balances.insert((agent, GRAIN), 1);
        let mut p = crop.clone();
        p.id = u64::from(offset) + 1;
        p.operator = agent;
        p.beneficiary = agent;
        p.asset = Some(PLOT + offset);
        p.right = Some(1 + offset);
        sim.state.processes.insert(p.id, p);
    }
    sim.state.balances.insert((STATE_AGENT, GRAIN), 2);
    sim.state.balances.insert((STATE_AGENT, TOKEN), tokens);
    sim.world
        .storage
        .capacities
        .insert(STATE_AGENT, state_space);
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Acquire);
    sim
}

#[test]
fn simultaneous_sellers_share_treasury_money_and_receiving_space() {
    for (tokens, space, expected) in [(2, 32, 2), (4, 3, 1), (0, 32, 0)] {
        let mut sim = harvest_pressure(tokens, space);
        sim.step().unwrap();
        let batch = sim.ledger.last().unwrap();
        let sellers: Vec<_> = batch
            .transactions
            .iter()
            .filter_map(|t| t.stock_trade.as_ref().map(|v| v.seller))
            .collect();
        assert_eq!(sellers.len(), expected);
        assert_eq!(
            sim.state.balance(STATE_AGENT, TOKEN),
            tokens - expected as i32
        );
        assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 2 + expected as i32);
        assert_eq!(
            sellers,
            (PERSON..PERSON + expected as u32).collect::<Vec<_>>()
        );
        sim.step().unwrap();
        assert_eq!(
            sim.state
                .processes
                .values()
                .filter(|p| p.definition == GROW && p.status == Status::Completed)
                .count(),
            expected
        );
        storage::validate(&sim.world, &sim.state).unwrap();
    }
}

#[test]
fn foragers_cannot_multiply_shared_supply_or_monthly_regeneration() {
    let mut sim = new("four-person-exchange", Backend::Reference);
    sim.world.priority = Priority::ContinuingFirst;
    for p in &mut sim.world.participants {
        for need in &mut p.needs {
            need.quantity = 0;
        }
        sim.world.scheduled_starts.push(ScheduledStart {
            month: 1,
            agent: p.agent,
            definition: FORAGE,
        });
    }
    sim.step().unwrap();
    sim.state.balances.insert((STATE_AGENT, WILD_SUPPLY), 1);
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    sim.step().unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, WILD_SUPPLY), 0);
    assert_eq!(
        sim.world
            .participants
            .iter()
            .map(|p| sim.state.balance(p.agent, WILD_FOOD))
            .sum::<i32>(),
        1
    );
    assert_eq!(sim.state.balance(PERSON, WILD_FOOD), 1);
    while sim.state.month == 1 {
        sim.step().unwrap();
    }
    sim.step().unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, WILD_SUPPLY), 1);
}

#[test]
fn multi_person_cpu_replay_reordering_and_midmonth_continuation() {
    for &name in MULTI_PERSON_SCENARIOS {
        let mut cpu = new(name, Backend::CubeCpu);
        let initial = cpu.state.clone();
        let mut reversed = cpu.clone();
        reversed.backend = Backend::Reference;
        reversed.world.agents.reverse();
        reversed.world.participants.reverse();
        reversed.world.assets.reverse();
        reversed.world.rights.reverse();
        reversed.world.agreements.reverse();
        reversed.world.issuance.reverse();
        reversed.world.condition_rules.reverse();
        reversed.world.resources.reverse();
        reversed.world.definitions.reverse();
        for p in &mut reversed.world.participants {
            p.needs.reverse();
        }
        cpu.run_months(14).unwrap();
        for _ in 0..14 {
            reversed.run_months(1).unwrap();
        }
        assert_eq!(cpu.state, reversed.state);
        assert_eq!(cpu.ledger, reversed.ledger);
        assert_eq!(cpu.reports, reversed.reports);
        let mut replay = initial;
        let mut checkpoint = None;
        for b in &cpu.ledger {
            commit(
                &cpu.world,
                &mut replay,
                b,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT,
            )
            .unwrap();
            if b.month == 13 && b.phase == Phase::Acquire {
                checkpoint = Some(replay.clone());
            }
            if let Some(d) = &b.decision {
                for row in d.alternatives[d.selected]
                    .outcomes
                    .iter()
                    .filter(|r| r.month == b.month)
                {
                    assert_eq!(
                        Some(row),
                        cpu.reports
                            .iter()
                            .find(|r| r.month == row.month && r.agent == row.agent)
                    );
                }
            }
        }
        assert_eq!(replay, cpu.state);
        let mut resumed =
            Simulation::new(cpu.world.clone(), checkpoint.unwrap(), Backend::Reference).unwrap();
        while resumed.state.month < cpu.state.month {
            resumed.step().unwrap();
        }
        assert_eq!(resumed.state, cpu.state);
        assert_eq!(
            cpu.state.obligations.values().map(|o| o.paid).sum::<i32>(),
            8
        );
        assert_eq!(
            cpu.state
                .balances
                .iter()
                .filter(|((_, r), _)| *r == TOKEN)
                .map(|(_, q)| q)
                .sum::<i32>(),
            4
        );
    }
}
