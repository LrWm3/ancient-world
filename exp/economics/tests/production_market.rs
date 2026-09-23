use economics_compute_smoke::{
    compute::Backend,
    marketplace::Side,
    model::*,
    production_market::{self as pm, Choice, Policy, Work},
    scenario::{GRAIN, GROW, NUTRITION, PERSON, SEED, TOKEN, WARMTH},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    town_market::{self, Boundary},
};
use std::collections::BTreeMap;

fn opening(w: World, s: State) -> Simulation {
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    sim
}
fn fixed() -> (World, State) {
    let (mut w, s) = pm::scenario(true);
    w.production_market.as_mut().unwrap().policy = Policy::Fixed(BTreeMap::new());
    (w, s)
}
#[test]
fn sides_follow_stock_and_buying_ahead_preserves_seller_buffer() {
    let (w, mut s) = fixed();
    for a in [88, 89, 91, 92] {
        s.balances.insert((a, GRAIN), 6);
    }
    s.balances.insert((88, GRAIN), 0);
    s.balances.insert((89, GRAIN), 8);
    let sim = opening(w.clone(), s.clone());
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert_eq!(r.volume, 2);
    assert!(
        r.orders
            .iter()
            .any(|o| o.agent == 88 && o.side == Side::Buy)
    );
    assert!(
        r.orders
            .iter()
            .any(|o| o.agent == 89 && o.side == Side::Sell)
    );
    // Registered sides do not prevent the same person selling in another state.
    s.balances.insert((88, GRAIN), 6);
    s.balances.insert((89, GRAIN), 0);
    let mut sim = opening(w, s);
    sim.step().unwrap();
    let r = &sim.state.town_market.history[0];
    assert!(
        r.orders
            .iter()
            .any(|o| o.agent == 88 && o.side == Side::Sell)
    );
    assert_eq!(sim.state.balance(88, GRAIN), 4);
    assert_eq!(
        [88, 89, 91, 92]
            .iter()
            .map(|a| sim.state.balance(*a, TOKEN))
            .sum::<i32>(),
        96
    );
}
#[test]
fn missing_seed_blocks_new_crop_and_wait_honors_existing_work() {
    let (mut w, mut s) = fixed();
    s.balances.insert((PERSON, SEED), 0);
    w.production_market.as_mut().unwrap().policy = Policy::Fixed(BTreeMap::from([(
        PERSON,
        Choice {
            work: Work::Produce(GROW),
            buy: false,
        },
    )]));
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert!(
        !sim.state
            .processes
            .values()
            .any(|p| p.operator == PERSON && p.definition == GROW)
    );
    let (w, s) = fixed();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.operator == PERSON && p.definition == GROW && p.status == Status::Active)
    );
    sim.world.production_market.as_mut().unwrap().policy = Policy::Fixed(BTreeMap::from([(
        PERSON,
        Choice {
            work: Work::Wait,
            buy: false,
        },
    )]));
    sim.run_months(2).unwrap();
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.operator == PERSON && p.definition == GROW && p.status == Status::Completed)
    );
    assert_eq!(sim.state.balance(PERSON, SEED), 1);
}
#[test]
fn decisions_are_dated_atomic_and_do_not_see_future_fixture_capacity() {
    let (w, s) = pm::scenario(true);
    let sim = opening(w, s);
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    let mut altered = sim.world.clone();
    altered.capacity_overrides.insert((2, PERSON), 0);
    assert_eq!(pm::choose(&altered, &sim.state).unwrap(), r.planning);
    let mut b = Batch::empty(&sim.state);
    b.transactions = r.transactions.clone();
    b.town_market = Some(Boundary::Market(r));
    if let Some(Boundary::Market(r)) = &mut b.town_market {
        r.planning.as_mut().unwrap().people[0].selected = usize::MAX;
    }
    let mut s = sim.state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut s,
            &b,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s, sim.state);
}
#[test]
fn observed_demand_expires_and_absent_buyers_create_no_cash() {
    let (w, mut s) = fixed();
    s.balances.insert((PERSON, GRAIN), 0);
    s.balances.insert((89, GRAIN), 10);
    let mut sim = opening(w, s);
    sim.step().unwrap();
    assert!(sim.state.town_market.history[0].volume > 0);
    // Current clearing results are not yesterday's observed demand.
    assert_eq!(pm::belief(&sim.world, &sim.state).lots_per_month, 0);
    sim.run_months(1).unwrap();
    assert!(pm::belief(&sim.world, &sim.state).lots_per_month > 0);
    let cash = sim
        .state
        .balances
        .iter()
        .filter(|((_, r), _)| *r == TOKEN)
        .map(|(a, q)| (*a, *q))
        .collect::<BTreeMap<_, _>>();
    for a in [88, 89, 91, 92] {
        sim.state.town_market.positions.insert(a, 100);
    }
    sim.run_months(7).unwrap();
    assert_eq!(pm::belief(&sim.world, &sim.state).lots_per_month, 0);
    assert_eq!(
        sim.state
            .balances
            .iter()
            .filter(|((_, r), _)| *r == TOKEN)
            .map(|(a, q)| (*a, *q))
            .collect::<BTreeMap<_, _>>(),
        cash
    );
    assert!(
        sim.state
            .town_market
            .history
            .iter()
            .skip(1)
            .all(|r| r.volume == 0 && r.posted_price.is_none())
    );
}
#[test]
fn cpu_monthly_checkpoint_and_reordering_match_reference_batch() {
    let (w, s) = pm::scenario(true);
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    reference.run_months(4).unwrap();
    let mut w = w;
    w.participants.reverse();
    w.agents.reverse();
    w.resources.reverse();
    w.definitions.reverse();
    w.town_market.as_mut().unwrap().traders.reverse();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    for _ in 0..4 {
        cpu.run_months(1).unwrap();
        let resumed =
            Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu).unwrap();
        cpu.state = resumed.state;
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    let mut bad = cpu.state.clone();
    bad.town_market.history[0].planning.as_mut().unwrap().people[0].selected = 999;
    assert!(Simulation::new(cpu.world, bad, Backend::Reference).is_err());
}
#[test]
fn trade_comparison_keeps_finite_money_and_repeated_crops() {
    let mut outcomes = vec![];
    for trading in [false, true] {
        let (w, s) = pm::scenario(trading);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(12).unwrap();
        assert_eq!(
            [88, 89, 91, 92]
                .iter()
                .map(|a| sim.state.balance(*a, TOKEN))
                .sum::<i32>(),
            96
        );
        assert!(sim.state.balances.values().all(|q| *q >= 0));
        assert!(
            sim.state
                .processes
                .values()
                .filter(|p| p.definition == GROW && p.status == Status::Completed)
                .count()
                > 4
        );
        let food = sim
            .reports
            .iter()
            .map(|r| r.deficit(NUTRITION))
            .sum::<i32>();
        let warmth = sim.reports.iter().map(|r| r.deficit(WARMTH)).sum::<i32>();
        let volume = sim
            .state
            .town_market
            .history
            .iter()
            .map(|r| r.volume)
            .sum::<i32>();
        outcomes.push((food, warmth, volume));
    }
    assert_eq!(outcomes[0].2, 0);
    assert!(outcomes[1].2 > 0);
    assert!(outcomes[1].0 <= outcomes[0].0);
}

#[test]
fn work_uses_accepted_decision_and_rejects_altered_execution() {
    let (w, s) = pm::scenario(true);
    let mut sim = opening(w, s);
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Productive);
    let checkpoint = sim.state.clone();
    let mut resumed =
        Simulation::new(sim.world.clone(), checkpoint.clone(), Backend::Reference).unwrap();
    sim.step().unwrap();
    resumed.step().unwrap();
    assert_eq!(sim.state, resumed.state);
    let mut b = sim.ledger.last().unwrap().clone();
    b.transactions.clear();
    let mut state = checkpoint.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut state,
            &b,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(state, checkpoint);
}

#[test]
fn incompatible_or_unbounded_planners_are_rejected() {
    for mode in 0..4 {
        let (mut w, s) = pm::scenario(true);
        match mode {
            0 => w.production_market.as_mut().unwrap().horizon = 1,
            1 => w.production_market.as_mut().unwrap().horizon = 13,
            2 => w.town_market.as_mut().unwrap().adaptive = false,
            _ => {
                w.production_market.as_mut().unwrap().policy = Policy::Fixed(BTreeMap::from([(
                    PERSON,
                    Choice {
                        work: Work::Produce(999),
                        buy: true,
                    },
                )]))
            }
        }
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}

#[test]
fn observed_price_values_unsold_stock_without_creating_spendable_cash() {
    let (w, mut s) = fixed();
    s.balances.insert((PERSON, GRAIN), 0);
    s.balances.insert((89, GRAIN), 10);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.world.production_market.as_mut().unwrap().policy = Policy::Plan;
    sim.step().unwrap();
    let a = pm::choose(&sim.world, &sim.state).unwrap().unwrap();
    let mut observed = sim.state.clone();
    observed.town_market.history[0].posted_price = Some(8);
    let b = pm::choose(&sim.world, &observed).unwrap().unwrap();
    assert_eq!(a.belief.price, Some(4));
    assert_eq!(b.belief.price, Some(8));
    let mut valued = false;
    for (a, b) in a.people.iter().zip(&b.people) {
        for (a, b) in a.alternatives.iter().zip(&b.alternatives) {
            assert_eq!(a.closing_coins, b.closing_coins);
            assert_eq!((a.sales, a.purchases), (b.sales, b.purchases));
            if a.stock_value > 0 {
                assert_eq!(b.stock_value, 2 * a.stock_value);
                valued = true;
            }
        }
    }
    assert!(valued);
}
