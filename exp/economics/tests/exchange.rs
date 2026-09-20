use economics_compute_smoke::{
    compute::Backend,
    crafts::*,
    equipment::DurableAsset,
    exchange,
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
    trading_scenario::{self, STOCK_UNIT},
};

fn work_fixture(rate: u32) -> Simulation {
    let (mut w, mut s) = trading_scenario::scenario(1, rate).unwrap();
    w.activities.orders.clear();
    w.condition_rules.clear();
    for p in &mut w.participants {
        for n in &mut p.needs {
            n.quantity = 0;
        }
    }
    let buyer = PERSON + 3;
    let provider = PERSON + 2;
    s.equipment.insert(
        9000,
        DurableAsset {
            id: 9000,
            owner: provider,
            attached_to: None,
            kind: COMB,
            remaining_uses: 24,
            last_used_month: None,
        },
    );
    s.equipment.insert(
        9001,
        DurableAsset {
            id: 9001,
            owner: buyer,
            attached_to: Some(PLOT + 3),
            kind: HERD,
            remaining_uses: 24,
            last_used_month: None,
        },
    );
    Simulation::new(w, s, Backend::Reference).unwrap()
}

#[test]
fn fractional_in_kind_shares_conserve_output_and_idle_tools_earn_nothing() {
    for rate in [0, 25, 100] {
        let mut sim = work_fixture(rate);
        sim.run_months(1).unwrap();
        assert_eq!(sim.state.equipment[&9000].owner, PERSON + 3);
        assert!(sim.state.exchange.earned.is_empty());
        sim.world.scheduled_starts.push(ScheduledStart {
            month: 2,
            agent: PERSON + 3,
            definition: HUSBANDRY,
        });
        sim.run_months(1).unwrap();
        let expected = [(MILK, 2 * STOCK_UNIT), (WOOL, STOCK_UNIT)];
        for (resource, produced) in expected {
            let provider = sim.state.balance(PERSON + 2, resource);
            let worker = sim.state.balance(PERSON + 3, resource);
            assert_eq!(provider, produced * rate as i32 / 100);
            assert_eq!(provider + worker, produced);
        }
        let before = sim.state.exchange.earned.clone();
        sim.run_months(1).unwrap();
        assert_eq!(sim.state.exchange.earned, before);
    }
}

#[test]
fn shares_preserve_replanting_seed_and_carry_earlier_tool_contribution() {
    let mut sim = work_fixture(25);
    sim.run_months(1).unwrap();
    let d = sim.world.definition(GROW);
    let p = ProcessInstance {
        id: 999,
        definition: GROW,
        operator: PERSON + 3,
        beneficiary: PERSON + 3,
        goal: None,
        asset: Some(PLOT + 3),
        right: Some(4),
        start: 1,
        reserved_through: d.duration(),
        stage: d.stages.len() - 1,
        elapsed: 0,
        status: Status::Active,
    };
    // The process remembers its supplying asset even if final work is manual.
    sim.state.exchange.contributors.insert(p.id, 9000);
    let (effects, royalty) = exchange::outputs(&sim.world, &sim.state, &p, None).unwrap();
    assert_eq!(
        effects
            .iter()
            .filter(|e| e.account == (PERSON + 3, SEED))
            .map(|e| e.delta)
            .sum::<i32>(),
        STOCK_UNIT
    );
    assert!(!royalty.unwrap().amounts.iter().any(|a| a.resource == SEED));
    sim.state.exchange.contributors.clear();
    let (_, manual) = exchange::outputs(&sim.world, &sim.state, &p, None).unwrap();
    assert!(manual.is_none());
}

#[test]
fn delivered_tool_and_royalty_tampering_fail_atomically_and_storage_bounds_output() {
    let mut sim = work_fixture(25);
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    let before = sim.state.clone();
    sim.step().unwrap();
    let mut batch = sim.ledger.last().unwrap().clone();
    batch
        .transactions
        .iter_mut()
        .find_map(|t| t.delivery.as_mut())
        .unwrap()
        .capture_percent = 100;
    let mut replay = before.clone();
    assert!(
        commit(
            &sim.world,
            &mut replay,
            &batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, before);
    while sim.state.month == 1 {
        sim.step().unwrap();
    }
    sim.world.scheduled_starts.push(ScheduledStart {
        month: 2,
        agent: PERSON + 3,
        definition: HUSBANDRY,
    });
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    let before = sim.state.clone();
    sim.step().unwrap();
    let mut bad = sim.ledger.last().unwrap().clone();
    bad.transactions
        .iter_mut()
        .find_map(|t| t.royalty.as_mut())
        .unwrap()
        .amounts[0]
        .quantity += 1;
    let mut replay = before.clone();
    assert!(
        commit(
            &sim.world,
            &mut replay,
            &bad,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(replay, before);
    let used =
        economics_compute_smoke::storage::usage(&sim.world, &before.balances)[&(PERSON + 2)] as i32;
    sim.world.storage.capacities.insert(PERSON + 2, used);
    let mut blocked = Simulation::new(sim.world, before, Backend::Reference).unwrap();
    blocked.step().unwrap();
    assert!(
        blocked
            .ledger
            .last()
            .unwrap()
            .transactions
            .iter()
            .all(|t| t.royalty.is_none())
    );
    // Manual work remains feasible: no tool benefit, no share, full output to worker.
    assert_eq!(blocked.state.balance(PERSON + 3, MILK), 2 * STOCK_UNIT);
    assert_eq!(blocked.state.equipment[&9000].remaining_uses, 24);
}

#[test]
fn posted_material_barter_respects_reserves_storage_and_opening_budget() {
    let mut sim = work_fixture(25);
    sim.state.balances.insert((PERSON + 2, STONE), 0);
    sim.state.balances.insert((PERSON + 2, GRAIN), STOCK_UNIT);
    sim.run_months(1).unwrap();
    let trades: Vec<_> = sim
        .ledger
        .iter()
        .flat_map(|b| &b.transactions)
        .filter(|t| t.stock_trade.is_some())
        .collect();
    assert_eq!(trades.len(), 1);
    assert_eq!(sim.state.balance(PERSON + 2, STONE), STOCK_UNIT);
    assert_eq!(sim.state.balance(PERSON + 2, GRAIN), 0);
    for t in trades {
        for resource in [GRAIN, STONE] {
            assert_eq!(
                t.effects
                    .iter()
                    .filter(|e| e.account.1 == resource)
                    .map(|e| e.delta)
                    .sum::<i32>(),
                0
            );
        }
    }
}

#[test]
fn trading_cpu_replay_reordering_and_midmonth_continuation() {
    let (w, s) = trading_scenario::default_scenario().unwrap();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    cpu.run_months(14).unwrap();
    let mut reversed = w.clone();
    reversed.participants.reverse();
    reversed.activities.orders.reverse();
    reversed.definitions.reverse();
    reversed.bids.reverse();
    reversed.market.as_mut().unwrap().tools.reverse();
    let mut reference = Simulation::new(reversed, s.clone(), Backend::Reference).unwrap();
    for _ in 0..14 {
        reference.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    let mut replay = s;
    let mut checkpoint = None;
    for b in &cpu.ledger {
        commit(&w, &mut replay, b, Backend::Reference, DEFAULT_EFFECT_LIMIT).unwrap();
        if b.month == 7 && b.phase == Phase::Acquire {
            checkpoint = Some(replay.clone());
        }
    }
    assert_eq!(replay, cpu.state);
    let mut resumed = Simulation::new(w, checkpoint.unwrap(), Backend::Reference).unwrap();
    while resumed.state.month < cpu.state.month {
        resumed.step().unwrap();
    }
    assert_eq!(resumed.state, cpu.state);
    assert!(!cpu.state.exchange.contracts.is_empty());
    assert!(!cpu.state.exchange.earned.is_empty());
}

#[test]
fn provider_selection_keeps_one_when_unsupported_and_preserves_population() {
    assert_eq!(
        trading_scenario::select_provider_count(&[(1, false), (2, false)]),
        1
    );
    assert_eq!(
        trading_scenario::select_provider_count(&[(1, true), (3, true), (8, false)]),
        3
    );
    for count in 1..=trading_scenario::MAX_PROVIDERS {
        let (w, s) = trading_scenario::scenario(count, 25).unwrap();
        assert_eq!(w.participants.len(), 32);
        let makers: std::collections::BTreeSet<_> = w
            .market
            .as_ref()
            .unwrap()
            .tools
            .iter()
            .map(|r| r.provider)
            .collect();
        assert_eq!(makers.len(), count);
        Simulation::new(w, s, Backend::Reference).unwrap();
    }
    assert!(trading_scenario::scenario(0, 25).is_err());
    assert!(trading_scenario::scenario(1, 101).is_err());
}
