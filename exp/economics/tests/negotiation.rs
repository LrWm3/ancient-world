use economics_compute_smoke::{
    compute::Backend,
    model::*,
    negotiation::{self, Outcome, QuotePolicy},
    opportunities::{Action, PERSON_TYPE, Policy},
    scenario::{GRAIN, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
use std::collections::{BTreeMap, BTreeSet};

fn opening() -> Simulation {
    let (world, state) = negotiation::scenario();
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Acquire);
    sim
}
fn preview(sim: &Simulation) -> Batch {
    let mut batch = Batch::empty(&sim.state);
    batch.negotiation = negotiation::evaluate(&sim.world, &sim.state).unwrap();
    batch.transactions =
        negotiation::transactions(&sim.world, &sim.state, &batch.negotiation).unwrap();
    batch
}

#[test]
fn concessions_cross_within_private_limits_and_cpu_settles_both_legs() {
    let mut reference = opening();
    let before = reference.state.clone();
    let batch = preview(&reference);
    assert_eq!(reference.state, before);
    let r = batch.negotiation.as_ref().unwrap();
    assert_eq!(r.outcome, Outcome::Traded { price: 40 });
    assert_eq!(
        r.quotes.iter().map(|q| (q.bid, q.ask)).collect::<Vec<_>>(),
        vec![(20, 60), (25, 55), (30, 50), (35, 45), (40, 40)]
    );
    let s = reference.world.negotiation.as_ref().unwrap();
    assert!(
        r.quotes
            .iter()
            .all(|q| q.bid <= s.buyer.limit && q.ask >= s.seller.limit)
    );
    for resource in [GRAIN, TOKEN] {
        assert_eq!(
            batch
                .transactions
                .iter()
                .flat_map(|t| &t.effects)
                .filter(|e| e.account.1 == resource)
                .map(|e| e.delta)
                .sum::<i32>(),
            0
        );
    }
    let mut cpu = reference.clone();
    cpu.backend = Backend::CubeCpu;
    reference.step().unwrap();
    cpu.step().unwrap();
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.state.balance(88, GRAIN), 2);
    assert_eq!(cpu.state.balance(89, GRAIN), 0);
    assert_eq!(cpu.state.balance(88, TOKEN), 60);
    assert_eq!(cpu.state.balance(89, TOKEN), 40);
    assert_eq!(cpu.state.phase, Phase::Productive);
}

#[test]
fn fixed_quotes_disjoint_limits_and_deadlines_leave_resources_untouched() {
    for case in 0..3 {
        let mut sim = opening();
        let s = sim.world.negotiation.as_mut().unwrap();
        match case {
            0 => {
                s.buyer.policy = QuotePolicy::Fixed;
                s.seller.policy = QuotePolicy::Fixed;
            }
            1 => s.buyer.limit = 25,
            _ => s.max_rounds = 2,
        }
        let before = sim.state.balances.clone();
        sim.step().unwrap();
        let b = sim.ledger.last().unwrap();
        let r = b.negotiation.as_ref().unwrap();
        assert_eq!(r.outcome, Outcome::NoAgreement);
        assert!(r.quotes.len() <= s_max(&sim));
        assert!(b.transactions.is_empty());
        assert_eq!(sim.state.balances, before);
    }
}
fn s_max(sim: &Simulation) -> usize {
    sim.world.negotiation.as_ref().unwrap().max_rounds as usize
}

#[test]
fn crossed_quotes_still_require_stock_money_storage_and_permission() {
    for expected in [
        Outcome::InsufficientGoods,
        Outcome::InsufficientPayment,
        Outcome::InsufficientStorage,
        Outcome::Ineligible,
    ] {
        let mut sim = opening();
        match expected {
            Outcome::InsufficientGoods => {
                sim.state.balances.insert((89, GRAIN), 1);
            }
            Outcome::InsufficientPayment => {
                sim.state.balances.insert((88, TOKEN), 39);
            }
            Outcome::InsufficientStorage => {
                sim.world.storage.capacities.insert(88, 1);
            }
            _ => {
                sim.world.transaction_policy = Some(Policy {
                    authority: 89,
                    membership_offers: vec![],
                    laws: vec![],
                    membership_permissions: BTreeSet::new(),
                    agent_types: BTreeMap::from([(88, PERSON_TYPE), (89, PERSON_TYPE)]),
                    permissions: BTreeSet::new(),
                });
            }
        }
        let before = sim.state.balances.clone();
        sim.backend = Backend::CubeCpu;
        sim.step().unwrap();
        let b = sim.ledger.last().unwrap();
        assert_eq!(b.negotiation.as_ref().unwrap().outcome, expected);
        assert!(b.transactions.is_empty());
        assert_eq!(sim.state.balances, before);
        if expected == Outcome::Ineligible {
            let policy = sim.world.transaction_policy.as_mut().unwrap();
            policy.permissions.insert((PERSON_TYPE, Action::StockTrade));
            // A permission changed after Acquire cannot retroactively transact.
            assert!(
                negotiation::evaluate(&sim.world, &sim.state)
                    .unwrap()
                    .is_none()
            );
        }
    }
}

#[test]
fn forged_quotes_effects_duplicate_spending_and_stale_replay_fail_atomically() {
    let sim = opening();
    let valid = preview(&sim);
    for case in 0..5 {
        let mut batch = valid.clone();
        match case {
            0 => batch.negotiation = None,
            1 => batch.negotiation.as_mut().unwrap().quotes[0].bid += 1,
            2 => batch.negotiation.as_mut().unwrap().outcome = Outcome::Traded { price: 1 },
            3 => batch.transactions[0].effects[0].delta = -1,
            _ => batch.transactions.push(batch.transactions[0].clone()),
        }
        let mut state = sim.state.clone();
        assert!(
            settlement::commit(
                &sim.world,
                &mut state,
                &batch,
                Backend::CubeCpu,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(state, sim.state);
    }
    let mut state = sim.state.clone();
    settlement::commit(
        &sim.world,
        &mut state,
        &valid,
        Backend::CubeCpu,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    let settled = state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut state,
            &valid,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(state, settled);
    // Revalidate opening availability rather than trusting an earlier preview.
    let mut state = sim.state.clone();
    state.balances.insert((88, TOKEN), 0);
    let before = state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut state,
            &valid,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(state, before);
}

#[test]
fn monthly_batched_resumed_and_reordered_execution_agree_without_repeat_trade() {
    let (world, state) = negotiation::scenario();
    let mut monthly = Simulation::new(world.clone(), state.clone(), Backend::CubeCpu).unwrap();
    let mut batched = monthly.clone();
    let mut reversed = world;
    reversed.agents.reverse();
    reversed.resources.reverse();
    let mut reversed = Simulation::new(reversed, state, Backend::Reference).unwrap();
    monthly.step().unwrap(); // checkpoint at Acquire, before any transfer
    let mut resumed = monthly.clone();
    for _ in 0..3 {
        monthly.run_months(1).unwrap();
    }
    for sim in [&mut batched, &mut resumed, &mut reversed] {
        sim.run_months(3).unwrap();
        assert_eq!(sim.state, monthly.state);
        assert_eq!(sim.ledger, monthly.ledger);
    }
    assert_eq!(
        monthly
            .ledger
            .iter()
            .filter(|b| b.negotiation.is_some())
            .count(),
        1
    );
    assert_eq!(monthly.state.balance(88, GRAIN), 2);
    assert_eq!(monthly.state.balance(88, TOKEN), 60);
}

#[test]
fn equal_limits_crossed_opening_quotes_and_overflow_sized_steps_are_bounded() {
    let mut sim = opening();
    {
        let s = sim.world.negotiation.as_mut().unwrap();
        s.buyer.limit = 40;
        s.seller.limit = 40;
        s.buyer.policy = QuotePolicy::Concede { ticks: i32::MAX };
        s.seller.policy = QuotePolicy::Concede { ticks: i32::MAX };
    }
    let round = negotiation::evaluate(&sim.world, &sim.state)
        .unwrap()
        .unwrap();
    assert_eq!(round.outcome, Outcome::Traded { price: 40 });
    assert_eq!(round.quotes.len(), 2);
    {
        let s = sim.world.negotiation.as_mut().unwrap();
        s.buyer.limit = 50;
        s.seller.limit = 30;
        s.buyer.opening_quote = 45;
        s.seller.opening_quote = 34;
    }
    let round = negotiation::evaluate(&sim.world, &sim.state)
        .unwrap()
        .unwrap();
    assert_eq!(round.outcome, Outcome::Traded { price: 39 });
    assert_eq!(round.quotes.len(), 1);
}

#[test]
fn invalid_terms_reject_before_any_execution() {
    for case in 0..7 {
        let (mut world, state) = negotiation::scenario();
        let s = world.negotiation.as_mut().unwrap();
        match case {
            0 => s.buyer.agent = s.seller.agent,
            1 => s.buyer.limit = 1,
            2 => s.seller.limit = 100,
            3 => s.buyer.policy = QuotePolicy::Concede { ticks: 0 },
            4 => s.goods.quantity = -1,
            5 => s.payment = s.goods.resource,
            _ => s.max_rounds = negotiation::MAX_QUOTE_ROUNDS + 1,
        }
        assert!(Simulation::new(world, state, Backend::Reference).is_err());
    }
}
