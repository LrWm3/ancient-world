use economics_compute_smoke::{
    compute::Backend,
    finance::{CollectionPolicy, ContractId},
    model::*,
    scenario::{self, GRAIN, PERSON, STATE_AGENT},
    simulation::Simulation,
};
const OTHER: AgentId = 99;
const COIN: ResourceId = 800;
fn fixture(policy: CollectionPolicy) -> (World, State) {
    let (mut w, mut s) = scenario::named("annual-access").unwrap();
    w.participants.clear();
    w.condition_rules.clear();
    w.definitions.clear();
    w.priority = Priority::ContinuingFirst;
    w.collection_policy = policy;
    w.storage.weights.insert(GRAIN, 1);
    w.agreements[0].payment.quantity = 4;
    w.agents.push(Agent {
        id: OTHER,
        name: "other landlord".into(),
    });
    w.assets.push(Asset {
        id: 999,
        owner: OTHER,
        kind: 1,
    });
    let mut right = w.rights[0].clone();
    right.id = 2;
    right.asset = 999;
    w.rights.push(right);
    let mut a = w.agreements[0].clone();
    a.id = 2;
    a.right = 2;
    a.creditor = OTHER;
    w.agreements.push(a);
    s.month = 12;
    s.balances.clear();
    s.balances.insert((PERSON, GRAIN), 5);
    (w, s)
}
#[test]
fn standalone_land_uses_explicit_proportional_policy_and_stable_default() {
    for (policy, paid) in [
        (CollectionPolicy::Stable, (4, 1)),
        (CollectionPolicy::Proportional, (3, 2)),
    ] {
        let (w, s) = fixture(policy);
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
        let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(1).unwrap();
        let mut checkpoint =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
        sim.world.agreements.reverse();
        sim.run_months(2).unwrap();
        checkpoint.run_months(1).unwrap();
        checkpoint.run_months(1).unwrap();
        reference.run_months(3).unwrap();
        assert_eq!(sim.state, reference.state);
        assert_eq!(sim.state, checkpoint.state);
        assert_eq!(sim.ledger, reference.ledger);
        assert_eq!(
            (
                sim.state.obligations[&(1, 13)].paid,
                sim.state.obligations[&(2, 13)].paid
            ),
            paid
        );
        assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    }
}
#[test]
fn standalone_rank_and_storage_limits_constrain_actual_land_payments() {
    for blocked in [false, true] {
        let (mut w, s) = fixture(CollectionPolicy::Proportional);
        if blocked {
            w.storage.capacities.insert(STATE_AGENT, 0);
        } else {
            w.claim_priorities.insert(ContractId::Land(1), 1);
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        sim.run_months(2).unwrap();
        assert_eq!(sim.state.obligations[&(2, 13)].paid, 4);
        assert_eq!(
            sim.state.obligations[&(1, 13)].paid,
            if blocked { 0 } else { 1 }
        );
        assert_eq!(
            sim.state.balance(PERSON, GRAIN),
            if blocked { 1 } else { 0 }
        );
    }
}
#[test]
fn standalone_alternative_tenders_allocate_whole_conversion_lots() {
    use economics_compute_smoke::activities::CoinPayment;
    let (mut w, mut s) = fixture(CollectionPolicy::Proportional);
    w.resources.push(Resource {
        id: COIN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    for a in &w.agreements {
        w.activities.coin_payments.insert(
            a.id,
            CoinPayment {
                resource: COIN,
                coins_per_unit: 2,
            },
        );
    }
    s.balances.insert((PERSON, GRAIN), 2);
    s.balances.insert((PERSON, COIN), 7);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(2).unwrap();
    let bills: Vec<_> = sim.state.obligations.values().collect();
    assert_eq!(bills.iter().map(|o| o.in_kind_paid).sum::<i32>(), 2);
    assert_eq!(bills.iter().map(|o| o.paid).sum::<i32>(), 5);
    assert!(bills.iter().all(|o| o.paid >= 2));
    assert_eq!(sim.state.balance(PERSON, COIN), 1);
}

#[test]
fn standalone_receipts_and_observers_distinguish_requested_granted_and_paid() {
    use economics_compute_smoke::{
        commitments, settlement,
        telemetry::{Config, Observer},
    };
    let (w, s) = fixture(CollectionPolicy::Proportional);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Due);
    let mut batch = Batch::empty(&sim.state);
    let mut result = commitments::evaluate(&sim.world, &sim.state).unwrap();
    assert_eq!(
        result
            .collections
            .iter()
            .map(|r| (r.requested.quantity, r.allocated, r.paid))
            .collect::<Vec<_>>(),
        vec![(4, Some(3), 3), (4, Some(2), 2)]
    );
    batch.transactions = result.transactions.clone();
    result.collections[0].allocated = Some(4);
    batch.commitments = Some(result);
    let before = sim.state.clone();
    assert!(
        settlement::commit(&sim.world, &mut sim.state, &batch, Backend::CubeCpu, 4096).is_err()
    );
    assert_eq!(sim.state, before);
    let mut observer = Observer::new(
        Vec::new(),
        "standalone",
        Config {
            settlement: true,
            ..Default::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 1).unwrap();
    let bytes = observer.finish().unwrap();
    let rows: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let paid: Vec<_> = rows
        .iter()
        .filter(|r| r["kind"] == "claim_collection" && r["paid"].as_i64().unwrap_or(0) > 0)
        .collect();
    assert_eq!(paid.len(), 2);
    assert_eq!(
        paid.iter()
            .map(|r| r["paid"].as_i64().unwrap())
            .sum::<i64>(),
        5
    );
}

fn forwards(concurrent: bool, policy: CollectionPolicy) -> (World, State) {
    use economics_compute_smoke::forward::direct::{AdmissionPolicy, Terms};
    let (mut w, mut s) = fixture(policy);
    w.agreements.clear();
    w.rights.clear();
    w.assets.clear();
    w.resources.push(Resource {
        id: COIN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    w.prepaid_admission = if concurrent {
        AdmissionPolicy::Concurrent
    } else {
        AdmissionPolicy::SingleOutstanding
    };
    w.prepaid_deliveries = [STATE_AGENT, OTHER]
        .into_iter()
        .enumerate()
        .map(|(i, buyer)| Terms {
            id: i as u32 + 1,
            seller: PERSON,
            buyer,
            month: 12,
            due: 13,
            goods: Amount::new(GRAIN, 4),
            prepayment: Amount::new(COIN, 4),
        })
        .collect();
    for buyer in [STATE_AGENT, OTHER] {
        s.balances.insert((buyer, COIN), 4);
    }
    (w, s)
}
#[test]
fn concurrent_forward_admission_is_opt_in_and_reserves_real_money_and_storage() {
    for concurrent in [false, true] {
        let (w, s) = forwards(concurrent, CollectionPolicy::Stable);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        sim.run_months(1).unwrap();
        assert_eq!(
            sim.state.exchange.forwards.len(),
            if concurrent { 2 } else { 1 }
        );
        assert_eq!(
            sim.state.balance(PERSON, COIN),
            if concurrent { 8 } else { 4 }
        );
        assert_eq!(sim.state.balance(PERSON, GRAIN), 5);
    }
    let (mut w, s) = forwards(true, CollectionPolicy::Stable);
    w.prepaid_deliveries[1].buyer = STATE_AGENT;
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.exchange.forwards.len(), 1); // same four coins cannot fund two advances
    let mut funded = s;
    funded.balances.insert((STATE_AGENT, COIN), 8);
    w.storage.capacities.insert(STATE_AGENT, 4);
    let mut sim = Simulation::new(w, funded, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.exchange.forwards.len(), 1); // four prospective slots cannot cover eight goods
    assert_eq!(sim.state.balance(STATE_AGENT, COIN), 4);
}
