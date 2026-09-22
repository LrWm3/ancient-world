use economics_compute_smoke::{
    compute::Backend,
    marketplace::{self, Side},
    model::*,
    negotiation::{self, GRAIN_MARKET, MARKETPLACE, Outcome, QuotePolicy},
    scenario::{GRAIN, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    zip::{self, Event, Learning},
};

fn config() -> zip::Config {
    zip::Config::default()
}
fn scenario(backend: Backend) -> Simulation {
    let (mut w, mut s) = negotiation::scenario();
    w.storage.capacities.insert(88, 24);
    s.balances.insert((89, GRAIN), 24);
    s.balances.insert((88, TOKEN), 600);
    let session = w.negotiation.as_mut().unwrap();
    session.buyer.policy = QuotePolicy::Zip(config());
    session.seller.policy = QuotePolicy::Zip(config());
    session.max_rounds = negotiation::MAX_QUOTE_ROUNDS;
    Simulation::new(w, s, backend).unwrap()
}
fn preview(sim: &Simulation) -> Batch {
    let mut b = Batch::empty(&sim.state);
    b.negotiation = negotiation::evaluate(&sim.world, &sim.state).unwrap();
    b.transactions = negotiation::transactions(&sim.world, &sim.state, &b.negotiation).unwrap();
    b
}
#[test]
fn widrow_hoff_momentum_has_a_hand_calculated_two_event_control() {
    let c = zip::Config {
        learning_rate: 500_000,
        momentum: 250_000,
        relative_target: 0,
        absolute_ticks: 0,
        seed: 7,
    };
    let mut l = Learning::new(c, 20, 30, &[1]);
    l.observe(c, Side::Sell, 20, 1, &Event::Trade { price: 40 });
    // Error correction .5*(40-30)=5; smoothed adjustment .75*5=3.75.
    assert_eq!(l.previous_adjustment, 3_750_000);
    assert_eq!(l.margin, 687_500);
    l.observe(c, Side::Sell, 20, 1, &Event::Trade { price: 40 });
    // .25*3.75 + .75*.5*(40-33.75) = 3.28125 payment units.
    assert_eq!(l.previous_adjustment, 3_281_250);
    assert_eq!(l.margin, 851_562);
    assert_eq!(l.updates, 2);
}
#[test]
fn public_event_branches_distinguish_success_rejection_and_failed_settlement() {
    let c = config();
    let buyer = Learning::new(c, 50, 40, &[1]);
    let seller = Learning::new(c, 30, 40, &[2]);
    for (side, limit, original) in [(Side::Buy, 50, buyer), (Side::Sell, 30, seller)] {
        let mut rejected = original.clone();
        rejected.observe(c, side, limit, 1, &Event::Rejected { side, price: 40 });
        let mut accepted = original.clone();
        accepted.observe(c, side, limit, 1, &Event::Trade { price: 40 });
        match side {
            Side::Buy => {
                assert!(rejected.margin > original.margin);
                assert!(accepted.margin < original.margin);
            }
            Side::Sell => {
                assert!(rejected.margin < original.margin);
                assert!(accepted.margin > original.margin);
            }
        }
        let mut ignored = original.clone();
        ignored.observe(c, side, limit, 1, &Event::SettlementFailed { price: 40 });
        assert_eq!(ignored, original);
        ignored.observe(
            c,
            side,
            limit,
            1,
            &Event::Rejected {
                side: if side == Side::Buy {
                    Side::Sell
                } else {
                    Side::Buy
                },
                price: 40,
            },
        );
        assert_eq!(ignored, original);
    }
}
#[test]
fn cpu_trades_preserve_private_limits_and_learning_survives_reordering_and_resume() {
    let mut cpu = scenario(Backend::CubeCpu);
    let mut reference = cpu.clone();
    reference.backend = Backend::Reference;
    reference.world.agents.reverse();
    reference.world.resources.reverse();
    cpu.step().unwrap();
    reference.step().unwrap();
    let opening = cpu.state.clone();
    let b = preview(&cpu);
    assert_eq!(cpu.state, opening);
    let mut resumed = cpu.clone();
    for sim in [&mut cpu, &mut reference, &mut resumed] {
        for _ in 0..4 {
            sim.world.negotiation.as_mut().unwrap().month = sim.state.month;
            sim.run_months(1).unwrap();
        }
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.state, resumed.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.ledger, resumed.ledger);
    let m = &cpu.state.marketplaces[&MARKETPLACE];
    assert_eq!(m.history.len(), 4);
    for r in &m.history {
        assert!(
            r.quotes
                .iter()
                .all(|q| q.bid > 0 && q.bid <= 50 && q.ask >= 30)
        );
        let Outcome::Traded { price } = r.outcome else {
            panic!("expected funded trade")
        };
        assert!((30..=50).contains(&price));
        assert_eq!(r.events.last(), Some(&Event::Trade { price }));
    }
    assert!(
        m.pricing[&(88, GRAIN_MARKET, Side::Buy)]
            .learning
            .as_ref()
            .unwrap()
            .updates
            > 4
    );
    assert_eq!(
        cpu.state.balance(88, GRAIN) + cpu.state.balance(89, GRAIN),
        24
    );
    assert_eq!(
        cpu.state.balance(88, TOKEN) + cpu.state.balance(89, TOKEN),
        600
    );
    assert!(b.negotiation.unwrap().buyer_learning.is_some());
}
#[test]
fn sub_tick_learning_is_retained_and_does_not_trigger_fixed_quote_early_stop() {
    let mut sim = scenario(Backend::Reference);
    sim.world.negotiation.as_mut().unwrap().max_rounds = 2;
    sim.step().unwrap();
    sim.step().unwrap();
    let r = sim.state.marketplaces[&MARKETPLACE].history.last().unwrap();
    assert_eq!(r.quotes.len(), 2);
    assert_eq!(r.outcome, Outcome::NoAgreement);
    assert_eq!(r.buyer_learning.as_ref().unwrap().updates, 2);
    assert_eq!(r.seller_learning.as_ref().unwrap().updates, 2);
    assert_ne!(r.buyer_learning.as_ref().unwrap().margin, -600_000);
}
#[test]
fn settlement_failure_is_not_a_trade_and_tampered_learning_is_atomic() {
    let mut sim = scenario(Backend::CubeCpu);
    let s = sim.world.negotiation.as_mut().unwrap();
    s.buyer.opening_quote = 45;
    s.seller.opening_quote = 35;
    sim.state.balances.insert((88, TOKEN), 0);
    sim.step().unwrap();
    let valid = preview(&sim);
    let r = valid.negotiation.as_ref().unwrap();
    assert_eq!(r.outcome, Outcome::InsufficientPayment);
    assert_eq!(r.events, vec![Event::SettlementFailed { price: 40 }]);
    assert_eq!(r.buyer_learning.as_ref().unwrap().updates, 0);
    for case in 0..3 {
        let mut b = valid.clone();
        let r = b.negotiation.as_mut().unwrap();
        match case {
            0 => r.buyer_learning.as_mut().unwrap().margin += 1,
            1 => r.seller_learning.as_mut().unwrap().random_state += 1,
            _ => r.events[0] = Event::Trade { price: 40 },
        }
        let mut state = sim.state.clone();
        assert!(
            settlement::commit(
                &sim.world,
                &mut state,
                &b,
                Backend::CubeCpu,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(state, sim.state);
    }
    sim.step().unwrap();
    assert_eq!(sim.state.balance(88, TOKEN), 0);
    assert_eq!(sim.state.balance(88, GRAIN), 0);
}
#[test]
fn new_limits_reprice_margin_policy_changes_reset_and_incompatible_limits_never_cross() {
    let mut sim = scenario(Backend::Reference);
    sim.run_months(1).unwrap();
    let s = sim.world.negotiation.as_mut().unwrap();
    s.month = 2;
    s.buyer.limit = 80;
    s.seller.limit = 50;
    let learner = marketplace::learning(&sim.state, s, Side::Buy).unwrap();
    let expected = learner.quote(80, 1, Side::Buy);
    sim.run_months(1).unwrap();
    assert_eq!(
        sim.state.marketplaces[&MARKETPLACE].history[1].quotes[0].bid,
        expected
    );
    let s = sim.world.negotiation.as_mut().unwrap();
    s.month = 3;
    s.buyer.policy = QuotePolicy::Fixed;
    s.seller.policy = QuotePolicy::Fixed;
    sim.run_months(1).unwrap();
    let m = &sim.state.marketplaces[&MARKETPLACE];
    assert_eq!(
        (m.history[2].quotes[0].bid, m.history[2].quotes[0].ask),
        (20, 60)
    );
    assert!(m.pricing[&(88, GRAIN_MARKET, Side::Buy)].learning.is_none());
    let mut disjoint = scenario(Backend::CubeCpu);
    disjoint.world.negotiation.as_mut().unwrap().buyer.limit = 25;
    disjoint.run_months(1).unwrap();
    let r = &disjoint.state.marketplaces[&MARKETPLACE].history[0];
    assert_eq!(r.outcome, Outcome::NoAgreement);
    assert_eq!(r.quotes.len(), 64);
    assert!(r.quotes.iter().all(|q| q.bid <= 25 && q.ask >= 30));
}
#[test]
fn seeded_randomness_is_repeatable_and_bad_parameters_or_memory_are_rejected() {
    let mut a = Learning::new(config(), 50, 20, &[1]);
    let mut b = a.clone();
    let mut other = Learning::new(
        zip::Config {
            seed: 19,
            ..config()
        },
        50,
        20,
        &[1],
    );
    for _ in 0..10 {
        let event = Event::Rejected {
            side: Side::Buy,
            price: 25,
        };
        a.observe(config(), Side::Buy, 50, 1, &event);
        b.observe(config(), Side::Buy, 50, 1, &event);
        other.observe(config(), Side::Buy, 50, 1, &event);
    }
    assert_eq!(a, b);
    assert_ne!(a.margin, other.margin);
    for case in 0..3 {
        let mut sim = scenario(Backend::Reference);
        let mut c = config();
        match case {
            0 => c.learning_rate = 0,
            1 => c.momentum = zip::SCALE as u32,
            _ => c.relative_target = zip::SCALE as u32 + 1,
        }
        sim.world.negotiation.as_mut().unwrap().buyer.policy = QuotePolicy::Zip(c);
        assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
    }
    let mut sim = scenario(Backend::Reference);
    sim.run_months(1).unwrap();
    sim.state
        .marketplaces
        .get_mut(&MARKETPLACE)
        .unwrap()
        .pricing
        .get_mut(&(88, GRAIN_MARKET, Side::Buy))
        .unwrap()
        .learning
        .as_mut()
        .unwrap()
        .margin = 1;
    assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
}
