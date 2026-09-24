use economics_compute_smoke::{
    accounting::Account as A,
    compute::Backend,
    financial_reporting::{Audit, Opening},
    model::*,
    process_accounting::{BeneficiaryPolicy, Costs, Output},
    scenario::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;

fn fixture() -> (World, State) {
    let (mut w, s) = baseline();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    w.rights[0].output_owner = STATE_AGENT;
    w.scheduled_starts.push(ScheduledStart {
        month: 1,
        agent: PERSON,
        definition: GROW,
    });
    (w, s)
}
fn opening(w: &World, s: &State) -> Opening {
    Opening {
        assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
        inventory: s
            .balances
            .iter()
            .filter(|((_, r), q)| {
                *r != TOKEN
                    && **q > 0
                    && w.resources
                        .iter()
                        .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
            })
            .map(|(k, q)| (*k, if k.1 == SEED { 12 } else { i128::from(*q) * 2 }))
            .collect(),
        processes: Some(Costs {
            beneficiary_policy: Some(BeneficiaryPolicy::TransferAtCost),
            ..Default::default()
        }),
        ..Default::default()
    }
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim)
            .unwrap_or_else(|e| panic!("{e} at {} {:?}", sim.state.month, sim.state.phase));
    }
}
fn balance(a: &Audit, owner: AgentId, account: A) -> i128 {
    a.book()
        .balances()
        .get(&(owner, account))
        .copied()
        .unwrap_or(0)
}
#[test]
fn joint_output_transfers_only_on_completion_and_cpu_restart_preserves_cost() {
    let (mut w, s) = fixture();
    w.definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .outputs
        .push(Amount::new(SEED, 1));
    let mut o = opening(&w, &s);
    o.processes.as_mut().unwrap().output_weights.insert(
        GROW,
        BTreeMap::from([(Output::Stock(GRAIN), 3), (Output::Stock(SEED), 1)]),
    );
    let mut a = Audit::with_opening(&w, &s, TOKEN, o).unwrap();
    let mut b = a.clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut cpu, 1);
    through(&mut b, &mut reference, 1);
    assert_eq!(a, b);
    let p = cpu
        .state
        .processes
        .values()
        .find(|p| p.definition == GROW)
        .unwrap();
    assert_eq!(p.operator, PERSON);
    assert_eq!(p.beneficiary, STATE_AGENT);
    assert_eq!(balance(&a, PERSON, A::WorkInProgress(p.id)), 12);
    assert_eq!(balance(&a, STATE_AGENT, A::TransferIncome), 0);
    let mut resumed = a.clone();
    let mut checkpoint = cpu.clone();
    // A historical opening also needs the operator's exact unfinished carrying cost.
    let mut historical = opening(&cpu.world, &cpu.state);
    historical
        .processes
        .as_mut()
        .unwrap()
        .work
        .insert(p.id, (PERSON, 12));
    historical
        .processes
        .as_mut()
        .unwrap()
        .output_weights
        .insert(
            GROW,
            BTreeMap::from([(Output::Stock(GRAIN), 3), (Output::Stock(SEED), 1)]),
        );
    let mut reopened = Audit::with_opening(&cpu.world, &cpu.state, TOKEN, historical).unwrap();
    let mut restarted = cpu.clone();
    through(&mut a, &mut cpu, 6);
    through(&mut b, &mut reference, 6);
    through(&mut resumed, &mut checkpoint, 6);
    through(&mut reopened, &mut restarted, 6);
    assert_eq!(a, b);
    assert_eq!(a, resumed);
    assert_eq!(cpu.state, reference.state);
    for book in [&a, &reopened] {
        assert_eq!(balance(book, STATE_AGENT, A::Inventory(GRAIN)), 9);
        assert_eq!(balance(book, STATE_AGENT, A::Inventory(SEED)), 3);
        assert_eq!(balance(book, PERSON, A::TransferExpense), 12);
        assert_eq!(balance(book, STATE_AGENT, A::TransferIncome), -12);
        for agent in [PERSON, STATE_AGENT] {
            let report = book.book().statements(agent, 2, 6).unwrap();
            assert!(report.cash_flows.values().all(|v| *v == 0));
        }
    }
    assert_eq!(cpu.state.balance(STATE_AGENT, GRAIN), 8);
    a.finalize_through(6).unwrap();
}
#[test]
fn missed_work_or_blocked_recipient_leaves_loss_with_operator() {
    for storage_block in [false, true] {
        let (mut w, s) = fixture();
        if storage_block {
            w.storage.weights.insert(GRAIN, 1);
            w.storage.capacities.insert(PERSON, 100);
            w.storage.capacities.insert(STATE_AGENT, 0);
        } else {
            w.capacity_overrides.insert((2, PERSON), 0);
        }
        let mut a = Audit::with_opening(&w, &s, TOKEN, opening(&w, &s)).unwrap();
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        through(&mut a, &mut sim, 6);
        assert_eq!(balance(&a, PERSON, A::ProductionLoss), 12);
        assert_eq!(balance(&a, PERSON, A::TransferExpense), 0);
        assert_eq!(balance(&a, STATE_AGENT, A::TransferIncome), 0);
        assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 0);
        assert!(
            sim.state
                .processes
                .values()
                .any(|p| p.definition == GROW && p.status == Status::Aborted)
        );
    }
}
#[test]
fn missing_policy_and_wrong_historical_cost_owner_are_rejected() {
    let (w, s) = fixture();
    let mut o = opening(&w, &s);
    o.processes.as_mut().unwrap().beneficiary_policy = None;
    let mut a = Audit::with_opening(&w, &s, TOKEN, o).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    loop {
        assert_eq!(sim.state.month, 1);
        let old = a.clone();
        let state = sim.state.clone();
        if let Err(e) = a.step(&mut sim) {
            assert!(e.contains("beneficiary policy"), "{e}");
            assert_eq!(old, a);
            assert_eq!(state, sim.state);
            break;
        }
    }
    let (w, s) = fixture();
    let mut a = Audit::with_opening(&w, &s, TOKEN, opening(&w, &s)).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    let p = sim
        .state
        .processes
        .values()
        .find(|p| p.definition == GROW)
        .unwrap();
    let mut o = opening(&sim.world, &sim.state);
    o.processes
        .as_mut()
        .unwrap()
        .work
        .insert(p.id, (STATE_AGENT, 12));
    assert!(Audit::with_opening(&sim.world, &sim.state, TOKEN, o).is_err());
}
#[test]
fn completed_joint_durable_and_stock_belong_to_beneficiary_and_decay_there() {
    use economics_compute_smoke::crafts::{self, BUILD_HOME, HOUSE, STONE};
    let (mut w, s) = crafts::scenario().unwrap();
    w.activities.orders.clear();
    w.agreements.clear();
    w.issuance.clear();
    w.activities.coin_payments.clear();
    w.condition_rules.clear();
    for p in &mut w.participants {
        for n in &mut p.needs {
            n.quantity = 0;
        }
    }
    w.priority = Priority::ContinuingFirst;
    w.rights
        .iter_mut()
        .find(|r| r.holder == PERSON)
        .unwrap()
        .output_owner = STATE_AGENT;
    w.activities.kinds.get_mut(&HOUSE).unwrap().lifetime = 2;
    w.activities.kinds.get_mut(&HOUSE).unwrap().monthly_decay = 1;
    w.definitions
        .iter_mut()
        .find(|d| d.id == BUILD_HOME)
        .unwrap()
        .outputs
        .push(Amount::new(STONE, 1));
    w.scheduled_starts = vec![ScheduledStart {
        month: 1,
        agent: PERSON,
        definition: BUILD_HOME,
    }];
    let mut o = opening(&w, &s);
    // Manufacture fixture uses unit historical costs: eight material ticks in total.
    o.inventory = s
        .balances
        .iter()
        .filter(|((_, r), q)| {
            *r != TOKEN
                && **q > 0
                && w.resources
                    .iter()
                    .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
        })
        .map(|(k, q)| (*k, i128::from(*q)))
        .collect();
    o.processes.as_mut().unwrap().output_weights.insert(
        BUILD_HOME,
        BTreeMap::from([(Output::Durable(HOUSE), 3), (Output::Stock(STONE), 1)]),
    );
    let mut a = Audit::with_opening(&w, &s, TOKEN, o).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 2);
    let home = sim
        .state
        .equipment
        .values()
        .find(|v| v.kind == HOUSE)
        .unwrap();
    assert_eq!(home.owner, STATE_AGENT);
    assert_eq!(balance(&a, STATE_AGENT, A::Tangible(home.id)), 6);
    assert_eq!(balance(&a, STATE_AGENT, A::Inventory(STONE)), 2);
    assert_eq!(balance(&a, PERSON, A::TransferExpense), 8);
    assert_eq!(balance(&a, STATE_AGENT, A::TransferIncome), -8);
    through(&mut a, &mut sim, 4);
    assert_eq!(balance(&a, STATE_AGENT, A::Depreciation), 6);
    assert_eq!(balance(&a, PERSON, A::Depreciation), 0);
}

#[test]
fn beneficiary_supplied_pool_inputs_keep_their_cost_across_both_transfers() {
    use economics_compute_smoke::pools::{Pool, PoolInput};
    let (mut w, mut s) = fixture();
    s.balances.remove(&(PERSON, SEED));
    s.balances.insert((STATE_AGENT, SEED), 1);
    w.pools.push(Pool {
        account: (STATE_AGENT, SEED),
        capacity: 1,
        monthly_regeneration: 0,
    });
    w.pool_inputs.push(PoolInput {
        definition: GROW,
        account: (STATE_AGENT, SEED),
    });
    let mut a = Audit::with_opening(&w, &s, TOKEN, opening(&w, &s)).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 1);
    assert_eq!(balance(&a, STATE_AGENT, A::TransferExpense), 12);
    assert_eq!(balance(&a, PERSON, A::TransferIncome), -12);
    assert_eq!(balance(&a, STATE_AGENT, A::TransferIncome), 0);
    through(&mut a, &mut sim, 6);
    assert_eq!(balance(&a, STATE_AGENT, A::Inventory(GRAIN)), 12);
    assert_eq!(balance(&a, PERSON, A::TransferExpense), 12);
    assert_eq!(balance(&a, STATE_AGENT, A::TransferIncome), -12);
    assert_eq!(
        a.book().statements(STATE_AGENT, 1, 6).unwrap().net_income,
        0
    );
}
