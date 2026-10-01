#[path = "support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    composition::{self, Budget, Strategy},
    compute::Backend,
    marketplace::Side,
    model::*,
    production_market::WOOD_MARKET,
    scenario::*,
    town_market::{self, OrderSelection},
};
fn budget() -> Budget {
    Budget {
        expansions: 256,
        forecasts: 32,
        months: 6,
    }
}

#[test]
fn adapter_preserves_opening_cash_and_rejects_tampered_or_stale_receipts() {
    let (mut sim, _) = fixtures::wood_market(Backend::Reference, true).unwrap();
    sim.state.balances.insert((PERSON, GRAIN), 0);
    sim.state.balances.insert((PERSON, FUEL), 5);
    fixtures::acquire(&mut sim).unwrap();
    let grain_market = sim.world.town_market.as_ref().unwrap().market;
    let orders = OrderSelection {
        actor: PERSON,
        submit: [(grain_market, Side::Buy), (WOOD_MARKET, Side::Sell)].into(),
    };
    let batch = composition::market::prepare(&sim, &[], Some(&orders)).unwrap();
    let mut tampered = batch.clone();
    tampered.transactions.clear();
    let before = sim.state.clone();
    assert!(
        economics_compute_smoke::settlement::commit(
            &sim.world,
            &mut sim.state,
            &tampered,
            sim.backend,
            sim.effect_limit
        )
        .is_err()
    );
    assert_eq!(sim.state, before);
    economics_compute_smoke::settlement::commit(
        &sim.world,
        &mut sim.state,
        &batch,
        sim.backend,
        sim.effect_limit,
    )
    .unwrap();
    assert_eq!(sim.state.balance(PERSON, TOKEN), 1);
    assert_eq!(
        sim.state.balance(PERSON, GRAIN),
        0,
        "new sale receipts cannot finance this book"
    );
    fixtures::acquire(&mut sim).unwrap();
    let selection = composition::choose(
        &sim,
        &composition::Scope::Person(PERSON),
        Strategy::BestFirst,
        budget(),
    )
    .unwrap();
    let mut changed = sim.clone();
    changed.world.town_market.as_mut().unwrap().additional[0].match_limit = Some(0);
    let before = changed.state.clone();
    assert!(selection.accept(&mut changed).is_err());
    assert_eq!(changed.state, before);
    selection.accept(&mut sim).unwrap();
    assert!(sim.state.balance(PERSON, GRAIN) > 0);
}

#[test]
fn wood_sales_fund_food_and_missing_demand_removes_the_income() {
    for strategy in [Strategy::Beam, Strategy::BestFirst] {
        for present in [true, false] {
            let (mut sim, scope) = fixtures::wood_market(Backend::Reference, present).unwrap();
            while sim.state.month < 13 {
                if sim.state.phase == Phase::Acquire {
                    composition::choose(&sim, &scope, strategy, budget())
                        .unwrap()
                        .accept(&mut sim)
                        .unwrap();
                } else {
                    sim.step().unwrap();
                }
            }
            let food: i32 = sim
                .reports
                .iter()
                .filter(|r| r.agent == PERSON)
                .map(|r| r.deficit(NUTRITION))
                .sum();
            let volume: i32 = sim
                .state
                .town_market
                .history
                .iter()
                .map(|r| r.markets[&WOOD_MARKET].volume)
                .sum();
            eprintln!(
                "{strategy:?} buyer={present} food={food} wood={volume} coins={}",
                sim.state.balance(PERSON, TOKEN)
            );
            if present {
                assert_eq!(food, 0);
                assert!(volume > 0);
            } else {
                assert!(food > 0);
                assert_eq!(volume, 0);
                assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
            }
            assert!(sim.state.balances.values().all(|q| *q >= 0));
            assert_eq!(
                sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
                100
            );
        }
    }
}

#[test]
fn explicit_submission_can_withhold_only_own_eligible_orders() {
    let (mut sim, _) = fixtures::wood_market(Backend::Reference, true).unwrap();
    sim.state.balances.insert((PERSON, FUEL), 5);
    fixtures::acquire(&mut sim).unwrap();
    let empty = OrderSelection {
        actor: PERSON,
        submit: Default::default(),
    };
    let round = town_market::evaluate_selected(&sim.world, &sim.state, &empty).unwrap();
    let mut reversed = sim.world.clone();
    reversed.participants.reverse();
    reversed.town_market.as_mut().unwrap().traders.reverse();
    for listing in &mut reversed.town_market.as_mut().unwrap().additional {
        listing.traders.reverse();
    }
    assert_eq!(
        round,
        town_market::evaluate_selected(&reversed, &sim.state, &empty).unwrap()
    );
    assert!(round.orders.iter().all(|o| o.agent != PERSON));
    assert!(round.orders.iter().any(|o| o.agent == 89));
    assert!(
        round
            .order_receipts
            .iter()
            .any(|r| r.agent == PERSON && r.reason == town_market::OrderReason::PlannerWithheld)
    );
    let invalid = OrderSelection {
        actor: PERSON,
        submit: [(999, Side::Buy)].into(),
    };
    assert!(town_market::evaluate_selected(&sim.world, &sim.state, &invalid).is_err());
    assert!(
        composition::market::prepare(
            &sim,
            &[economics_compute_smoke::offers::Request::new(
                economics_compute_smoke::offers::Id::Process(PREPARE_FUEL),
                89
            )],
            Some(&empty)
        )
        .unwrap_err()
        .contains("another actor")
    );
    sim.world
        .participants
        .iter_mut()
        .find(|p| p.agent == 89)
        .unwrap()
        .capacity
        .quantity = 1;
    assert!(
        composition::choose(
            &sim,
            &composition::Scope::Person(PERSON),
            Strategy::Beam,
            budget()
        )
        .unwrap_err()
        .contains("passive counterparties")
    );
}

#[test]
fn market_work_composition_replays_on_cpu_with_separate_books() {
    fn advance(
        sim: &mut economics_compute_smoke::simulation::Simulation,
        audit: &mut economics_compute_smoke::financial_reporting::Audit,
        end: u32,
    ) {
        while sim.state.month < end {
            if sim.state.phase == Phase::Acquire {
                let selected = composition::choose(
                    sim,
                    &composition::Scope::Person(PERSON),
                    Strategy::BestFirst,
                    budget(),
                )
                .unwrap();
                let before = sim.state.clone();
                selected.accept(sim).unwrap();
                audit
                    .record(&sim.world, &before, &selected.batch, &sim.state)
                    .unwrap();
            } else {
                audit.step(sim).unwrap();
            }
        }
    }
    let mut results = vec![];
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let (mut checkpoint, _) = fixtures::wood_market(backend, true).unwrap();
        let mut checkpoint_audit = fixtures::audit(&checkpoint).unwrap();
        advance(&mut checkpoint, &mut checkpoint_audit, 4);
        while checkpoint.state.phase != Phase::Acquire {
            checkpoint_audit.step(&mut checkpoint).unwrap();
        }
        let selected = composition::choose(
            &checkpoint,
            &composition::Scope::Person(PERSON),
            Strategy::BestFirst,
            budget(),
        )
        .unwrap();
        let before = checkpoint.state.clone();
        selected.accept(&mut checkpoint).unwrap();
        checkpoint_audit
            .record(
                &checkpoint.world,
                &before,
                &selected.batch,
                &checkpoint.state,
            )
            .unwrap();
        let mut resumed = checkpoint.clone();
        let mut resumed_audit = checkpoint_audit.clone();
        advance(&mut checkpoint, &mut checkpoint_audit, 13);
        for end in 5..=13 {
            advance(&mut resumed, &mut resumed_audit, end);
        }
        assert_eq!(checkpoint.state, resumed.state);
        assert_eq!(checkpoint.ledger, resumed.ledger);
        assert_eq!(checkpoint_audit, resumed_audit);
        checkpoint_audit.finalize_through(12).unwrap();
        results.push((checkpoint.state, checkpoint.ledger, checkpoint_audit));
    }
    assert_eq!(results[0], results[1]);
}
