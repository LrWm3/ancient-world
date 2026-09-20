use economics_compute_smoke::{
    compute::Backend,
    crafts::HOE,
    equipment::DurableAsset,
    exchange,
    forward::{self, Event, Reason},
    model::*,
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
    trading_scenario,
};

fn fixture(cash: i32, treasury: i32, enabled: bool) -> Simulation {
    let (mut w, mut s) = trading_scenario::cash_scenario(1, enabled).unwrap();
    let market = w.market.as_mut().unwrap();
    market.tools.retain(|r| r.buyer == PERSON && r.kind == HOE);
    assert_eq!(market.tools.len(), 1);
    market.targets.clear();
    let provider = market.tools[0].provider;
    s.equipment.insert(
        9000,
        DurableAsset {
            id: 9000,
            owner: provider,
            kind: HOE,
            remaining_uses: 24,
            attached_to: None,
            last_used_month: None,
        },
    );
    // Isolate financing from consumption; production remains driven by work targets.
    for need in &mut w
        .participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .needs
    {
        need.quantity = 0;
    }
    s.balances.insert((PERSON, TOKEN), cash);
    s.balances.insert((STATE_AGENT, TOKEN), treasury);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim
}

#[test]
fn upfront_cash_or_exact_forward_transfers_coins_and_never_charges_royalties() {
    for cash in [0, 10000] {
        let mut sim = fixture(cash, 10000, true);
        let before = sim.state.clone();
        sim.step().unwrap();
        assert!(
            sim.state.exchange.contracts.contains_key(&9000),
            "projection {:?}; ledger {:?}",
            forward::project(&sim.world, &before, PERSON, 9000),
            sim.ledger.last()
        );
        let delivery = &sim.state.exchange.contracts[&9000];
        let purchase = delivery.purchase.as_ref().unwrap();
        let price = purchase.price.quantity;
        assert!(price > 0);
        assert_eq!(delivery.capture_percent, 0);
        assert_eq!(sim.state.equipment[&9000].owner, PERSON);
        assert_eq!(
            sim.state.balance(delivery.provider, TOKEN) - before.balance(delivery.provider, TOKEN),
            price
        );
        if cash == 0 {
            let c = &sim.state.exchange.forwards[&9000];
            assert_eq!(c.advance.quantity, price);
            assert_eq!(c.goods.quantity, 2 * c.advance.quantity);
            let spot = &forward::policy(&sim.world).unwrap().prices[&c.goods.resource];
            let foregone = c.goods.quantity * spot.coins / spot.goods;
            assert!(purchase.projection.incremental_value > foregone);

            assert_eq!(c.goods.quantity * c.price.coins, price * c.price.goods);
            assert_eq!(c.due, c.issued + 12);
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 10000 - price);
            assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
            assert!(purchase.projection.surplus[&c.goods.resource] >= c.goods.quantity);
        } else {
            assert!(purchase.advance.is_none());
            assert_eq!(sim.state.balance(PERSON, TOKEN), cash - price);
        }
        let sum = |s: &State| {
            s.balances
                .iter()
                .filter(|((_, r), _)| *r == TOKEN)
                .map(|(_, q)| i64::from(*q))
                .sum::<i64>()
        };
        assert_eq!(sum(&before), sum(&sim.state));
        sim.run_months(12).unwrap();
        assert!(sim.state.exchange.earned.is_empty());
        assert!(
            sim.ledger
                .iter()
                .flat_map(|b| &b.transactions)
                .all(|t| t.royalty.is_none())
        );
    }
}

#[test]
fn finite_treasury_disabled_advances_and_unknown_prices_reject_cleanly() {
    for (cash, treasury, enabled, reason) in [
        (0, 0, true, Reason::NoCoinFunding),
        (0, 1, true, Reason::TreasuryShortfall),
        (0, 10000, false, Reason::AdvancesDisabled),
    ] {
        let mut sim = fixture(cash, treasury, enabled);
        let before = sim.state.balances.clone();
        sim.step().unwrap();
        assert!(sim.state.exchange.contracts.is_empty());
        assert_eq!(sim.state.balances, before);
        assert!(
            sim.ledger
                .last()
                .unwrap()
                .transactions
                .iter()
                .any(|t| matches!(&t.forward, Some(Event::Rejected {reason:r,..}) if *r == reason))
        );
    }
    let sim = fixture(0, 10000, true);
    let mut projection = forward::project(&sim.world, &sim.state, PERSON, 9000).unwrap();
    projection.assisted.insert(u32::MAX, 1);
    assert!(forward::quote(forward::policy(&sim.world).unwrap(), &projection, 25).is_err());
}

#[test]
fn maturity_shortfall_storage_and_repeated_collection_remain_real_transfers() {
    let mut sim = fixture(0, 10000, true);
    sim.step().unwrap();
    let c = sim.state.exchange.forwards[&9000].clone();
    let collect = |sim: &Simulation| {
        forward::settle(
            &sim.world,
            &sim.state,
            &mut sim.state.balances.clone(),
            &mut economics_compute_smoke::storage::usage(&sim.world, &sim.state.balances),
        )
        .unwrap()
    };
    assert!(collect(&sim).is_empty());
    sim.state.month = c.due;
    sim.state.balances.insert((PERSON, c.goods.resource), 0);
    assert!(collect(&sim).is_empty());
    assert_eq!(sim.state.exchange.forwards[&9000].delivered, 0);
    let protected = forward::policy(&sim.world)
        .unwrap()
        .protected
        .get(&c.goods.resource)
        .copied()
        .unwrap_or(0);
    sim.state
        .balances
        .insert((PERSON, c.goods.resource), protected + c.goods.quantity - 1);
    let used = economics_compute_smoke::storage::usage(&sim.world, &sim.state.balances)
        [&STATE_AGENT] as i32;
    let capacity = sim
        .world
        .storage
        .capacities
        .insert(STATE_AGENT, used)
        .unwrap();
    assert!(collect(&sim).is_empty());
    sim.world.storage.capacities.insert(STATE_AGENT, capacity);
    let partial = collect(&sim);
    assert_eq!(partial.len(), 1);
    assert_eq!(partial[0].effects.iter().map(|e| e.delta).sum::<i32>(), 0);
    exchange::record(&mut sim.state, &partial[0]);
    for e in &partial[0].effects {
        *sim.state.balances.entry(e.account).or_default() += e.delta;
    }
    assert!(collect(&sim).is_empty());
    // A later unit of actual production can settle the remaining unit.
    *sim.state
        .balances
        .entry((PERSON, c.goods.resource))
        .or_default() += 1;
    assert_eq!(
        sim.state.exchange.forwards[&9000].delivered,
        c.goods.quantity - 1
    );
    let last = collect(&sim);
    assert_eq!(last[0].effects[1].delta, 1);
    exchange::record(&mut sim.state, &last[0]);
    assert!(collect(&sim).is_empty());
}

#[test]
fn tampered_advance_fails_atomically_and_cpu_replay_matches() {
    let mut sim = fixture(0, 10000, true);
    let initial = sim.state.clone();
    sim.step().unwrap();
    let mut bad = sim.ledger.last().unwrap().clone();
    bad.transactions
        .iter_mut()
        .find_map(|t| t.delivery.as_mut())
        .unwrap()
        .purchase
        .as_mut()
        .unwrap()
        .advance
        .as_mut()
        .unwrap()
        .goods
        .quantity += 1;
    let mut replay = initial.clone();
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
    assert_eq!(replay, initial);
    let mut cpu = Simulation::new(sim.world.clone(), initial.clone(), Backend::CubeCpu).unwrap();
    cpu.run_months(14).unwrap();
    let mut w = sim.world.clone();
    w.participants.reverse();
    w.activities.orders.reverse();
    w.definitions.reverse();
    w.bids.reverse();
    let mut reference = Simulation::new(w, initial.clone(), Backend::Reference).unwrap();
    for _ in 0..14 {
        reference.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    let mut replay = initial;
    let mut checkpoint = None;
    for b in &cpu.ledger {
        commit(
            &sim.world,
            &mut replay,
            b,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT,
        )
        .unwrap();
        if b.month == 7 && b.phase == Phase::Acquire {
            checkpoint = Some(replay.clone());
        }
    }
    assert_eq!(replay, cpu.state);
    let mut resumed = Simulation::new(sim.world, checkpoint.unwrap(), Backend::Reference).unwrap();
    while resumed.state.month < cpu.state.month {
        resumed.step().unwrap();
    }
    assert_eq!(resumed.state, cpu.state);
}

#[test]
fn projection_ignores_future_shocks_and_reservations_cannot_reuse_treasury() {
    let sim = fixture(0, 10000, true);
    let expected = forward::project(&sim.world, &sim.state, PERSON, 9000).unwrap();
    let mut shocked = sim.world.clone();
    // A scripted future event is not information available to today's buyer.
    shocked.scheduled_starts.push(ScheduledStart {
        month: sim.state.month + 1,
        agent: PERSON,
        definition: GROW,
    });
    assert_eq!(
        forward::project(&shocked, &sim.state, PERSON, 9000).unwrap(),
        expected
    );
    let price = forward::quote(forward::policy(&sim.world).unwrap(), &expected, 25).unwrap();
    let request = &sim.world.market.as_ref().unwrap().tools[0];
    let asset = &sim.state.equipment[&9000];
    let mut available = sim.state.balances.clone();
    available.insert((STATE_AGENT, TOKEN), price);
    let first = forward::purchase(&sim.world, &sim.state, request, asset, &mut available).unwrap();
    assert!(first.delivery.is_some());
    assert_eq!(available[&(STATE_AGENT, TOKEN)], 0);
    let second = forward::purchase(&sim.world, &sim.state, request, asset, &mut available).unwrap();
    assert!(second.delivery.is_none());
    assert!(matches!(
        second.forward,
        Some(Event::Rejected {
            reason: Reason::NoCoinFunding,
            ..
        })
    ));
}

#[test]
fn forged_maturity_delivery_is_rejected_without_mutation() {
    let mut sim = fixture(0, 10000, true);
    sim.step().unwrap();
    let due = sim.state.exchange.forwards[&9000].due;
    while sim.state.month < due || sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    let opening = sim.state.clone();
    sim.step().unwrap();
    let mut batch = sim.ledger.last().unwrap().clone();
    let payment = batch
        .transactions
        .iter_mut()
        .find_map(|t| match t.forward.as_mut() {
            Some(Event::Delivery { quantity, .. }) => Some(quantity),
            _ => None,
        })
        .expect("the successful-production fixture should deliver at maturity");
    *payment += 1;
    let mut replay = opening.clone();
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
    assert_eq!(replay, opening);
}

#[test]
fn affordable_but_unproductive_tools_are_declined() {
    let mut sim = fixture(10000, 10000, true);
    for t in &mut sim.world.techniques {
        if t.equipment_kind.is_some() {
            t.output_multiplier = 1;
            for service in &mut t.services {
                service.quantity *= trading_scenario::TOOL_SERVICE_DIVISOR;
            }
        }
    }
    sim.step().unwrap();
    assert!(sim.state.exchange.contracts.is_empty());
    assert!(
        sim.ledger
            .last()
            .unwrap()
            .transactions
            .iter()
            .any(|t| matches!(
                &t.forward,
                Some(Event::Rejected {
                    reason: Reason::NotEconomic,
                    ..
                })
            ))
    );
}
