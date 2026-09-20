use economics_compute_smoke::{
    commitments,
    compute::Backend,
    currency::{self, StockTrade},
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
fn settle(sim: &mut Simulation, batch: &Batch) -> Result<(), String> {
    commit(
        &sim.world,
        &mut sim.state,
        batch,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
}
fn due() -> Simulation {
    let mut sim = new("storage-exchange", Backend::Reference);
    sim.state.month = 13;
    sim.state.phase = Phase::Due;
    sim
}
fn pressure() -> Simulation {
    let mut sim = new("storage-exchange", Backend::Reference);
    let (_, mut state) = named("forage-harvest").unwrap();
    state.balances.insert((PERSON, GRAIN), 1);
    state.balances.insert((STATE_AGENT, GRAIN), 2);
    state.balances.insert((STATE_AGENT, TOKEN), 1);
    sim.state = state;
    sim.step().unwrap(); // Open resets period fulfillment and capacities.
    sim.step().unwrap(); // Due: this harvest predates the first annual bill.
    assert_eq!(sim.state.phase, Phase::Acquire);
    sim
}
fn completions(sim: &Simulation, definition: DefinitionId) -> usize {
    sim.ledger
        .iter()
        .flat_map(|b| &b.transactions)
        .filter_map(|t| t.process.as_ref())
        .filter(|p| p.after.definition == definition && p.after.status == Status::Completed)
        .count()
}

#[test]
fn storage_is_shared_weighted_and_tokens_take_no_space() {
    let mut sim = new("storage-exchange", Backend::Reference);
    sim.world.storage.capacities.insert(PERSON, 7);
    sim.state.balances.insert((PERSON, TOKEN), i32::MAX);
    storage::validate(&sim.world, &sim.state).unwrap();
    assert_eq!(storage::usage(&sim.world, &sim.state.balances)[&PERSON], 7);
    sim.state.balances.insert((PERSON, WILD_FOOD), 1);
    assert!(storage::validate(&sim.world, &sim.state).is_err());
    sim.state.balances.insert((PERSON, WILD_FOOD), 0);
    sim.world.storage.weights.insert(SEED, 2);
    assert!(storage::validate(&sim.world, &sim.state).is_err());
    sim.world.storage.weights.insert(TOKEN, 1);
    assert!(currency::validate(&sim.world).is_err());
}

#[test]
fn issuance_uses_actual_annual_collection_partial_payments_and_never_purchases() {
    let mut sim = due();
    sim.state.balances.insert((PERSON, GRAIN), 1);
    sim.step().unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 1);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
    // A later committed harvest can clear the balance at the existing boundary.
    sim.state.phase = Phase::ClearArrears;
    sim.state.balances.insert((PERSON, GRAIN), 1);
    sim.step().unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 2);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 1);
    sim.state.phase = Phase::ClearArrears;
    sim.step().unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 1);

    sim.state.phase = Phase::Acquire;
    sim.state.balances.insert((PERSON, GRAIN), 1);
    let mut batch = Batch::empty(&sim.state);
    batch.transactions.push(
        currency::transaction(
            &sim.world,
            &sim.state,
            StockTrade {
                bid: 1,
                seller: PERSON,
            },
        )
        .unwrap(),
    );
    settle(&mut sim, &batch).unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 3);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 1);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 2);
    sim.state.phase = Phase::ClearArrears;
    sim.step().unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
    // Next annual installment creates exactly one additional token.
    sim.state.month = 25;
    sim.state.phase = Phase::Due;
    sim.state.balances.insert((PERSON, GRAIN), 2);
    sim.step().unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 1);
}

#[test]
fn state_storage_caps_collection_and_unpaid_tax_does_not_issue_tokens() {
    let mut sim = due();
    sim.world.storage.capacities.insert(STATE_AGENT, 1);
    sim.step().unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 1);
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 1);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
    sim.state.phase = Phase::ClearArrears;
    sim.step().unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 1);
    sim.world.storage.capacities.insert(STATE_AGENT, 2);
    sim.state.phase = Phase::ClearArrears;
    sim.step().unwrap();
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 1);
}

#[test]
fn stock_trade_is_atomic_and_needs_money_goods_and_receiving_space() {
    let mut sim = pressure();
    let trade = StockTrade {
        bid: 1,
        seller: PERSON,
    };
    let valid = currency::transaction(&sim.world, &sim.state, trade.clone()).unwrap();
    let before = sim.state.clone();
    let mut batch = Batch::empty(&sim.state);
    batch.transactions = vec![valid.clone(), valid.clone()];
    assert!(settle(&mut sim, &batch).is_err());
    assert_eq!(sim.state, before);
    batch.transactions = vec![valid];
    batch.transactions[0].effects[3].delta = 2;
    assert!(settle(&mut sim, &batch).is_err());
    assert_eq!(sim.state, before);
    sim.world.storage.capacities.insert(STATE_AGENT, 2);
    assert!(currency::transaction(&sim.world, &sim.state, trade.clone()).is_err());
    sim.world.storage.capacities.insert(STATE_AGENT, 32);
    sim.state.balances.insert((STATE_AGENT, TOKEN), 0);
    assert!(currency::transaction(&sim.world, &sim.state, trade.clone()).is_err());
    sim.state.balances.insert((STATE_AGENT, TOKEN), 1);
    sim.state.balances.insert((PERSON, GRAIN), 0);
    assert!(currency::transaction(&sim.world, &sim.state, trade).is_err());
}

#[test]
fn sale_frees_storage_for_harvest_and_without_pressure_the_agent_keeps_food() {
    let mut sim = pressure();
    let mut blocked = sim.clone();
    blocked.world.bids.clear();
    let mut roomy = sim.clone();
    roomy.world.storage.capacities.insert(PERSON, 30);
    sim.run_months(1).unwrap();
    blocked.run_months(1).unwrap();
    roomy.run_months(1).unwrap();
    assert_eq!(sim.state.balance(PERSON, TOKEN), 1);
    assert_eq!(completions(&sim, GROW), 1);
    assert_eq!(completions(&blocked, GROW), 0);
    assert!(
        blocked
            .ledger
            .iter()
            .flat_map(|b| &b.receipts)
            .any(|r| r.reason == Reason::InsufficientStorage)
    );
    assert_eq!(roomy.state.balance(PERSON, TOKEN), 0);
    assert_eq!(completions(&roomy, GROW), 1);
    assert_eq!(sim.reports[0].deficit(NUTRITION), 0);
}

#[test]
fn issuance_tampering_overflow_and_storage_overflow_publish_nothing() {
    let mut sim = due();
    let before = sim.state.clone();
    let mut b = Batch::empty(&sim.state);
    let expected = commitments::evaluate(&sim.world, &sim.state).unwrap();
    b.transactions = expected.transactions.clone();
    b.commitments = Some(expected);
    b.transactions.last_mut().unwrap().effects[0].delta += 1;
    assert!(settle(&mut sim, &b).is_err());
    assert_eq!(sim.state, before);
    sim.state.balances.insert((STATE_AGENT, TOKEN), i32::MAX);
    let before = sim.state.clone();
    assert!(sim.step().is_err());
    assert_eq!(sim.state, before);

    let mut sim = pressure();
    let mut b = Batch::empty(&sim.state);
    b.transactions.push(Transaction {
        cause: "unbacked issuance".into(),
        effects: vec![Effect {
            account: (PERSON, TOKEN),
            delta: 1,
        }],
        process: None,
        technique_use: None,
        trade: None,
        stock_trade: None,
        forward: None,
        delivery: None,
        royalty: None,
    });
    let before = sim.state.clone();
    assert!(settle(&mut sim, &b).is_err());
    assert_eq!(sim.state, before);
    b.transactions[0].effects = vec![Effect {
        account: (PERSON, GRAIN),
        delta: 30,
    }];
    assert!(settle(&mut sim, &b).is_err());
    assert_eq!(sim.state, before);
}

#[test]
fn cpu_reference_replay_continuation_and_conservation_include_storage_and_currency() {
    for &name in CURRENCY_SCENARIOS {
        let mut cpu = new(name, Backend::CubeCpu);
        let initial = cpu.state.clone();
        cpu.run_months(36).unwrap();
        let mut reference = new(name, Backend::Reference);
        reference.world.resources.reverse();
        reference.world.definitions.reverse();
        for _ in 0..36 {
            reference.run_months(1).unwrap();
        }
        assert_eq!(cpu.state, reference.state, "{name}");
        assert_eq!(cpu.reports, reference.reports);
        assert_eq!(cpu.ledger, reference.ledger);
        let mut replay = initial.clone();
        let mut checkpoints = Vec::new();
        let mut minted = 0;
        let mut sold = 0;
        for b in &cpu.ledger {
            if b.month == 13 {
                checkpoints.push(replay.clone());
            }
            if b.commitments.is_some() {
                minted += b
                    .transactions
                    .iter()
                    .flat_map(|t| &t.effects)
                    .filter(|e| e.account.1 == TOKEN)
                    .map(|e| e.delta)
                    .sum::<i32>();
            }
            sold += b
                .transactions
                .iter()
                .filter(|t| t.stock_trade.is_some())
                .count() as i32;
            commit(
                &cpu.world,
                &mut replay,
                b,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT,
            )
            .unwrap();
            storage::validate(&cpu.world, &replay).unwrap();
            if let Some(d) = &b.decision {
                for r in d.alternatives[d.selected]
                    .outcomes
                    .iter()
                    .filter(|r| r.month == b.month)
                {
                    assert_eq!(
                        Some(r),
                        cpu.reports
                            .iter()
                            .find(|row| row.month == r.month && row.agent == r.agent)
                    );
                }
            }
        }
        assert_eq!(replay, cpu.state);
        assert_eq!(cpu.state.balance(PERSON, TOKEN), sold);
        assert_eq!(cpu.state.balance(STATE_AGENT, TOKEN) + sold, minted);
        let collected: i32 = cpu.state.obligations.values().map(|o| o.paid).sum();
        assert_eq!(cpu.state.balance(STATE_AGENT, GRAIN), collected + sold);
        if !cpu.world.issuance.is_empty() {
            assert_eq!(
                minted,
                cpu.state
                    .obligations
                    .values()
                    .map(|o| o.paid / 2)
                    .sum::<i32>()
            );
        }
        for state in checkpoints {
            let mut resumed =
                Simulation::new(cpu.world.clone(), state, Backend::Reference).unwrap();
            while resumed.state.month < cpu.state.month {
                resumed.step().unwrap();
            }
            assert_eq!(resumed.state, cpu.state);
        }
    }
}
