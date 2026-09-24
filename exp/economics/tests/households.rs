use economics_compute_smoke::{
    activities::{Target, WorkOrder},
    compute::Backend,
    households::{self, Agreement, Purpose, Request},
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
    storage,
};
const HOME: u32 = 10000;
fn fixture(count: u32) -> (World, State) {
    let (mut w, s) = baseline();
    for i in 1..count {
        w.agents.push(Agent {
            id: PERSON + i,
            name: format!("adult {i}"),
        });
        let mut p = w.participants[0].clone();
        p.agent = PERSON + i;
        w.participants.push(p);
    }
    (w, s)
}
fn agreement(count: u32) -> Agreement {
    Agreement {
        id: 1,
        agent: HOME,
        adults: (PERSON..PERSON + count).collect(),
        governance: economics_compute_smoke::household_governance::Governance::legacy(PERSON),
        formed: 1,
        dwelling_process: None,
    }
}
fn request(member: u32, quantity: i32, benefit: i64, sequence: u64) -> Request {
    Request {
        household: HOME,
        member,
        resource: GRAIN,
        quantity,
        minimum: 1,
        individual_benefit: benefit,
        collective_benefit: benefit,
        sequence,
        purpose: Purpose::Need(NUTRITION),
    }
}
#[test]
fn formation_requires_one_to_four_distinct_unaffiliated_adults_and_is_atomic() {
    for count in [0, 1, 4, 5] {
        let (mut w, s) = fixture(count.max(1));
        let before = w.clone();
        let result = households::form(&mut w, &s, agreement(count));
        assert_eq!(result.is_ok(), (1..=4).contains(&count));
        if result.is_err() {
            assert_eq!(w, before);
        }
    }
    let (mut w, s) = fixture(2);
    households::form(&mut w, &s, agreement(2)).unwrap();
    let before = w.clone();
    let mut other = agreement(1);
    other.agent += 1;
    other.id += 1;
    assert!(households::form(&mut w, &s, other).is_err());
    assert_eq!(w, before);
    let (mut w, s) = fixture(1);
    let mut a = agreement(1);
    a.adults.push(PERSON);
    assert!(households::form(&mut w, &s, a).is_err());
    assert_eq!(households::GROWN_CHILD_ADULT_SLOTS, 4);
}
#[test]
fn reservations_rank_collective_gain_then_submission_order_and_do_not_double_reserve() {
    let (mut w, mut s) = fixture(3);
    households::form(&mut w, &s, agreement(3)).unwrap();
    s.balances.insert((HOME, GRAIN), 5);
    let (r, e) = households::allocate(
        &w,
        &s,
        vec![
            request(PERSON, 3, 10, 0),
            request(PERSON + 1, 3, 20, 1),
            request(PERSON + 2, 3, 10, 2),
        ],
    )
    .unwrap();
    assert_eq!(
        r.iter()
            .map(|x| (x.request.member, x.allocated))
            .collect::<Vec<_>>(),
        vec![(PERSON + 1, 3), (PERSON, 2), (PERSON + 2, 0)]
    );
    assert_eq!(e.iter().map(|x| x.delta).sum::<i32>(), 0);
    let (r, _) = households::allocate(
        &w,
        &s,
        vec![request(PERSON, 3, 20, 0), request(PERSON, 3, 10, 1)],
    )
    .unwrap();
    assert_eq!(r.iter().map(|x| x.allocated).sum::<i32>(), 3);
    let mut bad = request(PERSON, 3, 20, 0);
    bad.individual_benefit = 0;
    assert_eq!(
        households::allocate(&w, &s, vec![bad]).unwrap().0[0].allocated,
        0
    );
}
#[test]
fn storage_contributes_half_without_creating_space_and_coins_need_none() {
    let (mut w, mut s) = fixture(2);
    w.storage.weights.insert(GRAIN, 1);
    w.storage
        .capacities
        .extend([(PERSON, 10), (PERSON + 1, 10)]);
    s.balances.clear();
    households::form(&mut w, &s, agreement(2)).unwrap();
    s.balances.extend([
        ((PERSON, GRAIN), 5),
        ((PERSON + 1, GRAIN), 5),
        ((HOME, GRAIN), 10),
    ]);
    storage::validate(&w, &s).unwrap();
    assert_eq!(
        storage::room(&w, &storage::usage(&w, &s.balances), HOME, GRAIN),
        0
    );
    s.balances.insert((HOME, GRAIN), 11);
    assert!(storage::validate(&w, &s).is_err());
    s.balances.insert((HOME, GRAIN), 0);
    s.balances.insert((PERSON, GRAIN), 15);
    storage::validate(&w, &s).unwrap();
    s.balances.insert((PERSON, GRAIN), 16);
    assert!(storage::validate(&w, &s).is_err());
    assert_eq!(
        storage::room(&w, &storage::usage(&w, &s.balances), HOME, SEED),
        i32::MAX
    );
}
#[test]
fn actual_production_is_pooled_once_and_replay_rejects_tampering() {
    let (mut w, mut s) = fixture(2);
    for p in &mut w.participants {
        p.needs.clear();
    }
    w.definitions
        .iter_mut()
        .find(|d| d.id == REPAIR)
        .unwrap()
        .outputs[0]
        .quantity = 4;
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: REPAIR,
        priority: 0,
        target: Target::Stock(Amount::new(REPAIR_OUTPUT, 20)),
    });
    households::form(&mut w, &s, agreement(2)).unwrap();
    s.phase = Phase::Productive;
    s.balances.insert((PERSON, LABOR), 1);
    let opening = s.clone();
    let mut sim = Simulation::new(w.clone(), s, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, REPAIR_OUTPUT), 2);
    assert_eq!(sim.state.balance(HOME, REPAIR_OUTPUT), 2);
    let mut replay = opening.clone();
    commit(
        &w,
        &mut replay,
        &sim.ledger[0],
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    assert_eq!(replay, sim.state);
    let prior = replay.clone();
    assert!(
        commit(
            &w,
            &mut replay,
            &sim.ledger[0],
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(prior, replay);
    let mut altered = sim.ledger[0].clone();
    altered.household.as_mut().unwrap().after[0].delta -= 1;
    let mut untouched = opening.clone();
    assert!(
        commit(
            &w,
            &mut untouched,
            &altered,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(untouched, opening);
    assert!(commit(&w, &mut untouched, &sim.ledger[0], Backend::Reference, 1).is_err());
    assert_eq!(untouched, opening);
}
#[test]
fn member_can_consume_shared_food_without_payment_or_repooling() {
    let (mut w, mut s) = fixture(2);
    s.balances.insert((PERSON, GRAIN), 0);
    s.balances.insert((HOME, GRAIN), 2);
    households::form(&mut w, &s, agreement(2)).unwrap();
    s.phase = Phase::Consumption;
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.balance(HOME, GRAIN), 0);
    assert_eq!(sim.state.balance(PERSON, NUTRITION), 1);
    assert_eq!(sim.state.balance(PERSON + 1, NUTRITION), 1);
    assert!(sim.ledger[0].household.as_ref().unwrap().after.is_empty());
}
#[test]
fn household_assigns_only_spare_labor_and_waits_without_useful_work() {
    let (mut w, mut s) = fixture(2);
    for p in &mut w.participants {
        p.needs.clear();
    }
    households::form(&mut w, &s, agreement(2)).unwrap();
    s.phase = Phase::Productive;
    s.balances.insert((PERSON, LABOR), 0);
    s.balances.insert((PERSON + 1, LABOR), 2);
    let mut idle = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    idle.step().unwrap();
    assert!(idle.ledger[0].household.as_ref().unwrap().before.is_empty());
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: REPAIR,
        priority: 0,
        target: Target::Stock(Amount::new(REPAIR_OUTPUT, 20)),
    });
    w.activities.orders.push(WorkOrder {
        agent: PERSON + 1,
        definition: REPAIR,
        priority: 0,
        target: Target::Stock(Amount::new(REPAIR_OUTPUT, 20)),
    });
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, REPAIR_OUTPUT), 1);
    assert_eq!(sim.state.balance(PERSON + 1, REPAIR_OUTPUT), 1);
    assert_eq!(sim.state.balance(PERSON + 1, LABOR), 0);
    assert_eq!(
        sim.ledger[0]
            .household
            .as_ref()
            .unwrap()
            .before
            .iter()
            .filter(|e| e.account == (PERSON, LABOR))
            .map(|e| e.delta)
            .sum::<i32>(),
        1
    );
}
#[test]
fn cpu_checkpoint_and_monthly_batches_match_with_households() {
    let (mut w, s) = fixture(2);
    households::form(&mut w, &s, agreement(2)).unwrap();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    cpu.run_months(9).unwrap();
    for _ in 0..9 {
        reference.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    let mut resume = reference.clone();
    resume.step().unwrap();
    let mut checkpoint = resume.clone();
    resume.run_months(2).unwrap();
    checkpoint.run_months(2).unwrap();
    assert_eq!(resume.state, checkpoint.state);
    assert_eq!(resume.ledger, checkpoint.ledger);
}

#[test]
fn one_actual_dwelling_serves_the_household_and_wears_only_once() {
    use economics_compute_smoke::{activities::DurableKind, equipment::DurableAsset};
    let (mut w, mut s) = fixture(2);
    let ticket = 99;
    let shelter = 100;
    let occupy = 90;
    let house = 77;
    w.resources.extend([
        Resource {
            id: ticket,
            name: "occupancy".into(),
            kind: ResourceKind::Stock,
        },
        Resource {
            id: shelter,
            name: "shelter".into(),
            kind: ResourceKind::Fulfillment,
        },
    ]);
    w.activities.perishable.insert(ticket);
    w.activities.required.insert(occupy, house);
    w.activities.kinds.insert(
        house,
        DurableKind {
            name: "dwelling".into(),
            lifetime: 4,
            attached: false,
            monthly_decay: 0,
        },
    );
    w.definitions.extend([
        ProcessDefinition {
            id: occupy,
            name: "occupy dwelling".into(),
            execution: Execution::Productive,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "occupy".into(),
                months: 1,
                entry_inputs: vec![],
                monthly_services: vec![],
            }],
            outputs: vec![Amount::new(ticket, 1)],
        },
        ProcessDefinition {
            id: 91,
            name: "shelter consumption".into(),
            execution: Execution::Consumption,
            enabled: true,
            asset_kind: None,
            stages: vec![Stage {
                name: "consume".into(),
                months: 1,
                entry_inputs: vec![Amount::new(ticket, 1)],
                monthly_services: vec![],
            }],
            outputs: vec![Amount::new(shelter, 1)],
        },
    ]);
    for p in &mut w.participants {
        p.needs = vec![Requirement {
            resource: shelter,
            quantity: 1,
            priority: 0,
        }];
    }
    s.equipment.insert(
        1000,
        DurableAsset {
            id: 1000,
            owner: PERSON,
            kind: house,
            remaining_uses: 4,
            last_used_month: None,
            attached_to: None,
        },
    );
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: occupy,
        priority: 0,
        target: Target::Stock(Amount::new(ticket, 1)),
    });
    let mut a = agreement(2);
    a.dwelling_process = Some(occupy);
    households::form(&mut w, &s, a).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert!(sim.reports.iter().all(|r| r.deficit(shelter) == 0));
    assert_eq!(sim.state.equipment[&1000].remaining_uses, 3);
    sim.state.equipment.get_mut(&1000).unwrap().remaining_uses = 0;
    sim.run_months(1).unwrap();
    assert!(
        sim.reports
            .iter()
            .filter(|r| r.month == 2)
            .all(|r| r.deficit(shelter) == 1)
    );
}

#[test]
fn household_can_fund_native_and_coin_taxes_without_changing_debtor_or_issuing_twice() {
    for coins in [false, true] {
        let (mut w, mut s) = named("annual-access").unwrap();
        w.priority = Priority::NeedFirst;
        w.participants[0].needs.clear();
        w.condition_rules.clear();
        s.conditions.clear();
        s.month = 13;
        s.phase = Phase::Due;
        s.balances.insert((PERSON, GRAIN), 0);
        if coins {
            w.resources.push(Resource {
                id: TOKEN,
                name: "coin".into(),
                kind: ResourceKind::Stock,
            });
            w.activities.coin_payments.insert(
                1,
                economics_compute_smoke::activities::CoinPayment {
                    resource: TOKEN,
                    coins_per_unit: 1,
                },
            );
            s.balances.insert((HOME, TOKEN), 1);
        } else {
            s.balances.insert((HOME, GRAIN), 1);
        }
        households::form(
            &mut w,
            &s,
            Agreement {
                formed: s.month,
                ..agreement(1)
            },
        )
        .unwrap();
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        sim.step().unwrap();
        assert_eq!(sim.state.obligations[&(1, 13)].paid, 1);
        assert_eq!(sim.world.agreements[0].debtor, PERSON);
        assert_eq!(
            sim.state
                .balance(STATE_AGENT, if coins { TOKEN } else { GRAIN }),
            1
        );
        sim.state.phase = Phase::ClearArrears;
        sim.step().unwrap();
        assert_eq!(
            sim.state
                .balance(STATE_AGENT, if coins { TOKEN } else { GRAIN }),
            1
        );
    }
}

#[test]
fn last_adult_death_deactivates_household_but_preserves_estate_and_history() {
    let (mut w, mut s) = named("conditions-warmth-first").unwrap();
    households::form(&mut w, &s, agreement(1)).unwrap();
    s.balances.insert((HOME, SEED), 3);
    s.balances.insert((PERSON, GRAIN), 0);
    s.balances.insert((PERSON, SEED), 0);
    s.balances.insert((PERSON, FUEL), 0);
    s.balances.insert((PERSON, RAW_WOOD), 0);
    for d in &mut w.definitions {
        if d.execution == Execution::Productive {
            d.enabled = false;
        }
    }
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(24).unwrap();
    assert!(sim.state.terminal.contains_key(&PERSON));
    assert_eq!(households::parent(&sim.world, &sim.state, PERSON), None);
    assert_eq!(sim.state.balance(HOME, SEED), 3);
    assert!(
        sim.ledger
            .last()
            .unwrap()
            .household
            .as_ref()
            .unwrap()
            .inactive
            .contains(&HOME)
    );
    assert!(
        households::form(
            &mut sim.world,
            &sim.state,
            Agreement {
                id: 2,
                agent: HOME + 1,
                ..agreement(1)
            }
        )
        .is_err()
    );
}

#[test]
fn mandatory_pool_storage_is_reserved_before_production() {
    let (mut w, mut s) = fixture(1);
    w.participants[0].needs.clear();
    w.storage.weights.insert(REPAIR_OUTPUT, 1);
    w.storage.capacities.insert(PERSON, 4);
    w.definitions
        .iter_mut()
        .find(|d| d.id == REPAIR)
        .unwrap()
        .outputs[0]
        .quantity = 2;
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: REPAIR,
        priority: 0,
        target: Target::Stock(Amount::new(REPAIR_OUTPUT, 10)),
    });
    s.balances.insert((HOME, REPAIR_OUTPUT), 2);
    s.balances.insert((PERSON, LABOR), 1);
    s.phase = Phase::Productive;
    households::form(&mut w, &s, agreement(1)).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, REPAIR_OUTPUT), 0);
    assert_eq!(sim.state.balance(HOME, REPAIR_OUTPUT), 2);
    assert!(
        sim.ledger[0]
            .receipts
            .iter()
            .any(|r| r.reason == Reason::InsufficientStorage)
    );
}

#[test]
fn household_delivers_a_members_forward_once_from_pooled_inventory() {
    use economics_compute_smoke::{crafts::HOE, equipment::DurableAsset, trading_scenario};
    let (mut w, mut s) = trading_scenario::cash_scenario(1, true).unwrap();
    let market = w.market.as_mut().unwrap();
    market.tools.retain(|r| r.buyer == PERSON && r.kind == HOE);
    market.targets.clear();
    let provider = market.tools[0].provider;
    s.equipment.insert(
        9000,
        DurableAsset {
            id: 9000,
            owner: provider,
            kind: HOE,
            remaining_uses: 24,
            last_used_month: None,
            attached_to: None,
        },
    );
    for n in &mut w
        .participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .needs
    {
        n.quantity = 0;
    }
    s.balances.insert((PERSON, TOKEN), 0);
    s.balances.insert((STATE_AGENT, TOKEN), 10000);
    households::form(&mut w, &s, agreement(1)).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim.step().unwrap();
    let contract = sim.state.exchange.forwards[&9000].clone();
    let protected = sim
        .world
        .market
        .as_ref()
        .unwrap()
        .cash
        .as_ref()
        .unwrap()
        .protected
        .get(&contract.goods.resource)
        .copied()
        .unwrap_or(0);
    sim.state.month = contract.due;
    sim.state.phase = Phase::Acquire;
    sim.state
        .balances
        .insert((PERSON, contract.goods.resource), protected);
    sim.state
        .balances
        .insert((HOME, contract.goods.resource), contract.goods.quantity);
    let before = sim.state.balance(STATE_AGENT, contract.goods.resource);
    sim.step().unwrap();
    assert_eq!(
        sim.state.exchange.forwards[&9000].delivered,
        contract.goods.quantity
    );
    assert_eq!(sim.state.balance(HOME, contract.goods.resource), 0);
    assert_eq!(
        sim.state.balance(STATE_AGENT, contract.goods.resource) - before,
        contract.goods.quantity
    );
    assert_eq!(
        sim.state.balance(PERSON, contract.goods.resource),
        protected
    );
}

#[test]
fn fractional_contributions_carry_across_months_instead_of_vanishing() {
    let (mut w, s) = fixture(1);
    w.participants[0].needs.clear();
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: REPAIR,
        priority: 0,
        target: Target::Stock(Amount::new(REPAIR_OUTPUT, 20)),
    });
    households::form(&mut w, &s, agreement(1)).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.balance(HOME, REPAIR_OUTPUT), 0);
    assert_eq!(sim.state.household_remainders[&(PERSON, REPAIR_OUTPUT)], 1);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.balance(HOME, REPAIR_OUTPUT), 1);
    assert_eq!(sim.state.balance(PERSON, REPAIR_OUTPUT), 1);
    assert_eq!(sim.state.household_remainders[&(PERSON, REPAIR_OUTPUT)], 0);
}

#[test]
fn an_unfillable_reservation_leaves_goods_for_a_smaller_useful_request() {
    let (mut w, mut s) = fixture(2);
    s.balances.insert((HOME, GRAIN), 2);
    households::form(&mut w, &s, agreement(2)).unwrap();
    let mut large = request(PERSON, 3, 20, 0);
    large.minimum = 3;
    let (receipts, _) =
        households::allocate(&w, &s, vec![large, request(PERSON + 1, 2, 10, 1)]).unwrap();
    assert_eq!(receipts[0].allocated, 0);
    assert_eq!(receipts[1].allocated, 2);
}

#[test]
fn household_need_priority_keeps_external_claim_as_arrears_until_funded() {
    let (mut w, mut s) = named("annual-access").unwrap();
    w.priority = Priority::NeedFirst;
    s.month = 13;
    s.phase = Phase::Due;
    s.balances.insert((PERSON, GRAIN), 0);
    s.balances.insert((HOME, GRAIN), 1);
    households::form(
        &mut w,
        &s,
        Agreement {
            formed: 13,
            ..agreement(1)
        },
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 1);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 0);
    sim.state.balances.insert((HOME, GRAIN), 1);
    sim.state.phase = Phase::ClearArrears;
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 1);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 1);
}

#[test]
fn need_driven_processes_can_request_common_inputs_without_a_work_order() {
    let (mut w, mut s) = with_warmth(true);
    w.priority = Priority::NeedFirst;
    s.phase = Phase::Productive;
    s.balances.insert((PERSON, RAW_WOOD), 0);
    s.balances.insert((PERSON, FUEL), 0);
    s.balances.insert((PERSON, LABOR), 4);
    s.balances.insert((HOME, RAW_WOOD), 1);
    households::form(&mut w, &s, agreement(1)).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.step().unwrap();
    assert!(
        sim.ledger[0]
            .household
            .as_ref()
            .unwrap()
            .reservations
            .iter()
            .any(|r| r.request.resource == RAW_WOOD && r.allocated == 1)
    );
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, WARMTH), 1);
}

fn governed_fixture() -> (World, State) {
    use economics_compute_smoke::household_governance::Governance;
    let (mut w, mut s) = fixture(2);
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = 5;
    }
    w.definitions.retain(|d| d.id == REPAIR);
    let d = &mut w.definitions[0];
    d.stages[0].monthly_services = vec![Amount::new(LABOR, 6)];
    d.outputs = vec![Amount::new(REPAIR_OUTPUT, 10)];
    for person in [PERSON, PERSON + 1] {
        w.activities.orders.push(WorkOrder {
            agent: person,
            definition: REPAIR,
            priority: 0,
            target: Target::Stock(Amount::new(REPAIR_OUTPUT, 100)),
        });
        s.balances.insert((person, LABOR), 5);
    }
    let mut a = agreement(2);
    a.governance = Governance::contributed(PERSON);
    households::form(&mut w, &s, a).unwrap();
    s.phase = Phase::Productive;
    (w, s)
}

#[test]
fn charter_contributions_are_bounded_and_ties_rotate_independently_of_signatures() {
    for month in [1, 2] {
        let (mut w, mut s) = governed_fixture();
        w.households[0].adults.reverse();
        s.month = month;
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.step().unwrap();
        let d = &sim.ledger[0].household.as_ref().unwrap().labor[0];
        assert_eq!(d.recipient, Some(PERSON + month - 1));
        assert_eq!(d.granted, 2);
        assert_eq!(d.contributions.len(), 2);
        assert!(
            d.contributions
                .iter()
                .all(|c| c.available == 5 && c.reserved == 1 && c.directed == 1 && c.returned == 0)
        );
        let produced: i32 = [PERSON, PERSON + 1, HOME]
            .into_iter()
            .map(|a| sim.state.balance(a, REPAIR_OUTPUT))
            .sum();
        assert_eq!(produced, 10);
        assert_eq!(
            [PERSON, PERSON + 1]
                .into_iter()
                .map(|a| sim.state.balance(a, LABOR))
                .sum::<i32>(),
            4
        );
    }
}

#[test]
fn unused_or_out_of_scope_reservations_return_without_creating_labor() {
    for restricted in [false, true] {
        let (mut w, s) = governed_fixture();
        if restricted {
            w.households[0].governance.constitution.activities = Some(Default::default());
        } else {
            w.activities.orders.clear();
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.step().unwrap();
        let d = &sim.ledger[0].household.as_ref().unwrap().labor[0];
        assert_eq!(d.recipient, None);
        assert_eq!(d.granted, 0);
        assert!(
            d.contributions
                .iter()
                .all(|c| c.reserved == 1 && c.directed == 0 && c.returned == 1)
        );
        assert_eq!(sim.state.balance(PERSON, LABOR), 5);
        assert_eq!(sim.state.balance(PERSON + 1, LABOR), 5);
    }
}

#[test]
fn governor_may_schedule_only_constitutionally_permitted_future_policies() {
    use economics_compute_smoke::household_governance::{self as g, Policy, PolicyChange};
    let (mut w, s) = governed_fixture();
    let constitution = w.households[0].governance.constitution.clone();
    let charter = w.households[0].governance.charter.clone();
    for (actor, month) in [(PERSON + 1, 2), (PERSON, 1)] {
        let before = w.clone();
        assert!(
            g::schedule(
                &mut w,
                &s,
                HOME,
                PolicyChange {
                    month,
                    authorized_by: actor,
                    policy: Policy::NetOutput
                }
            )
            .is_err()
        );
        assert_eq!(w, before);
    }
    g::schedule(
        &mut w,
        &s,
        HOME,
        PolicyChange {
            month: 2,
            authorized_by: PERSON,
            policy: Policy::NetOutput,
        },
    )
    .unwrap();
    assert_eq!(
        w.households[0].governance.policy(1),
        Policy::PreserveCommittedWork
    );
    assert_eq!(w.households[0].governance.policy(2), Policy::NetOutput);
    assert_eq!(w.households[0].governance.constitution, constitution);
    assert_eq!(w.households[0].governance.charter, charter);
    let before = w.clone();
    assert!(
        g::schedule(
            &mut w,
            &s,
            HOME,
            PolicyChange {
                month: 2,
                authorized_by: PERSON,
                policy: Policy::NetOutput
            }
        )
        .is_err()
    );
    assert_eq!(w, before);
    w.households[0]
        .governance
        .constitution
        .permitted_policies
        .remove(&Policy::NetOutput);
    assert!(households::validate(&w, &s).is_err());
}

#[test]
fn contributed_work_replays_on_cpu_and_rejects_altered_reservations_atomically() {
    let (w, s) = governed_fixture();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    reference.step().unwrap();
    cpu.step().unwrap();
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.ledger, cpu.ledger);
    let mut forged = reference.ledger[0].clone();
    forged.household.as_mut().unwrap().labor[0].contributions[0].directed += 1;
    let mut untouched = s.clone();
    assert!(
        commit(
            &w,
            &mut untouched,
            &forged,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(untouched, s);
    let mut resumed = Simulation::new(w, reference.state.clone(), Backend::Reference).unwrap();
    reference.run_months(3).unwrap();
    for _ in 0..3 {
        resumed.run_months(1).unwrap();
    }
    assert_eq!(reference.state, resumed.state);
}

#[test]
fn commitment_protection_and_output_policy_differ_on_identical_opening_work() {
    use economics_compute_smoke::household_governance::Policy;
    let (mut w, mut s) = governed_fixture();
    let mut committed = w.definitions[0].clone();
    committed.id = 99;
    committed.stages[0].monthly_services = vec![Amount::new(LABOR, 5)];
    committed.outputs[0].quantity = 1;
    w.definitions.push(committed);
    w.activities.orders.retain(|o| o.agent == PERSON);
    s.processes.insert(
        900,
        ProcessInstance {
            id: 900,
            definition: 99,
            operator: PERSON + 1,
            beneficiary: PERSON + 1,
            goal: None,
            asset: None,
            right: None,
            start: 1,
            reserved_through: 1,
            stage: 0,
            elapsed: 0,
            status: Status::Active,
        },
    );
    let mut preserve = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    w.households[0].governance.charter.initial_policy = Policy::NetOutput;
    let mut output = Simulation::new(w, s, Backend::Reference).unwrap();
    preserve.step().unwrap();
    output.step().unwrap();
    assert_eq!(preserve.state.processes[&900].status, Status::Completed);
    assert_eq!(output.state.processes[&900].status, Status::Aborted);
    assert_eq!(
        preserve.ledger[0].household.as_ref().unwrap().labor[0].recipient,
        None
    );
    assert_eq!(
        output.ledger[0].household.as_ref().unwrap().labor[0].recipient,
        Some(PERSON)
    );
}

#[test]
fn contribution_rounding_uses_current_capacity_and_does_not_bank_unused_hours() {
    let (w, mut s) = governed_fixture();
    s.balances.insert((PERSON, LABOR), 4);
    s.balances.insert((PERSON + 1, LABOR), 0);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    let d = &sim.ledger[0].household.as_ref().unwrap().labor[0];
    assert!(d.contributions.iter().all(|c| c.reserved == 0));
    assert_eq!(sim.state.balance(PERSON, LABOR), 4);
    sim.run_months(2).unwrap();
    let decisions: Vec<_> = sim
        .ledger
        .iter()
        .filter_map(|b| b.household.as_ref())
        .flat_map(|h| &h.labor)
        .collect();
    assert!(decisions.iter().skip(1).all(|d| {
        d.contributions
            .iter()
            .all(|c| c.available == 5 && c.reserved == 1)
    }));
}

#[test]
fn household_observer_reports_member_contributions_without_affecting_execution() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let (w, s) = governed_fixture();
    let mut observed = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut plain = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut observer = Observer::new(
        vec![],
        "governance",
        Config {
            settlement: true,
            agents: [PERSON + 1].into_iter().collect(),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut observed, 1).unwrap();
    plain.run_months(1).unwrap();
    assert_eq!(observed.state, plain.state);
    assert_eq!(observed.ledger, plain.ledger);
    let log = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert!(log.lines().any(|l| l.contains("household_labor")
        && l.contains("\"reserved\":1")
        && l.contains("PreserveCommittedWork")));
}
