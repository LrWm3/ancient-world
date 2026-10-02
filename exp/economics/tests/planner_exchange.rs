#[path = "support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    allocation::Policy as Allocation,
    composition::{
        Budget, Strategy,
        continuation::{Policy, persons::Persons},
        market,
    },
    compute::Backend,
    marketplace::Side,
    model::*,
    offers::{Id, Request},
    production_market::WOOD_MARKET,
    scenario::*,
    settlement,
    simulation::Simulation,
    town_market::{self, OrderSelections},
};
fn planner(sim: &Simulation) -> Persons {
    Persons::new(
        sim.world.participants.iter().map(|p| p.agent),
        Strategy::BestFirst,
        Budget {
            expansions: 256,
            forecasts: 32,
            months: 6,
        },
        Policy::Monthly,
        Allocation::StablePriority,
        7,
    )
    .unwrap()
}
fn setup(backend: Backend, trading: bool, coins: i32) -> (Simulation, Persons) {
    let mut sim = fixtures::trading_persons(backend, trading).unwrap();
    for a in [PERSON, 89] {
        sim.state.balances.insert((a, TOKEN), coins);
    }
    let p = planner(&sim);
    (sim, p)
}
fn advance(sim: &mut Simulation, p: &mut Persons, end: u32) {
    while sim.state.month < end {
        p.step(sim).unwrap();
    }
}
fn all_orders() -> OrderSelections {
    [
        (PERSON, [(1, Side::Buy), (WOOD_MARKET, Side::Sell)].into()),
        (89, [(1, Side::Sell), (WOOD_MARKET, Side::Buy)].into()),
    ]
    .into()
}
fn liquid_book() -> Simulation {
    let (mut sim, _) = setup(Backend::Reference, true, 1);
    sim.state.balances.extend([
        ((PERSON, GRAIN), 0),
        ((PERSON, FUEL), 5),
        ((89, GRAIN), 10),
        ((89, FUEL), 0),
    ]);
    fixtures::acquire(&mut sim).unwrap();
    sim
}
#[test]
fn active_people_complete_repeated_production_and_exchange_with_finite_coins() {
    for trading in [true, false] {
        let (mut sim, mut p) = setup(Backend::Reference, trading, 1);
        advance(&mut sim, &mut p, 25);
        for person in [PERSON, 89] {
            let reports: Vec<_> = sim.reports.iter().filter(|r| r.agent == person).collect();
            let food: i32 = reports.iter().map(|r| r.deficit(NUTRITION)).sum();
            let warmth: i32 = reports.iter().map(|r| r.deficit(WARMTH)).sum();
            if trading {
                assert!(food <= 1 && warmth <= 1, "{person} {food} {warmth}");
                assert!(reports.iter().all(|r| r.terminal.is_none()));
                assert!(
                    sim.state
                        .processes
                        .values()
                        .filter(|x| x.operator == person
                            && x.status == Status::Completed
                            && sim.world.definition(x.definition).execution
                                == Execution::Productive)
                        .count()
                        >= 3
                );
            } else {
                assert!(food + warmth > 0);
                assert!(reports.iter().any(|r| r.terminal.is_some()));
            }
        }
        for id in [1, WOOD_MARKET] {
            let volume: i32 = sim
                .state
                .town_market
                .history
                .iter()
                .map(|r| r.markets[&id].volume)
                .sum();
            if trading {
                assert!(volume >= 12);
            } else {
                assert_eq!(volume, 0);
            }
        }
        assert!(sim.state.balances.values().all(|q| *q >= 0));
        assert_eq!(
            sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
            2
        );
        assert!(p.history.iter().all(|r| r.exchanges.len() == 2));
        assert!(p.history.iter().all(|r| {
            r.proposals
                .iter()
                .all(|(a, requests)| requests.iter().all(|x| x.agent == *a))
        }));
        assert!(p.controllers.values().all(|c| c.history.iter().all(|r| {
            r.search
                .as_ref()
                .is_some_and(|m| m.forecasts <= 32 && m.expansions <= 256)
        })));
    }
}
#[test]
fn independent_withholding_is_visible_and_predictions_are_not_fills() {
    let (mut sim, mut p) = setup(Backend::Reference, true, 6);
    advance(&mut sim, &mut p, 13);
    let missed = p
        .history
        .iter()
        .find(|r| r.exchanges.values().any(|x| x.expected != x.actual))
        .unwrap();
    let live = sim
        .state
        .town_market
        .history
        .iter()
        .find(|r| r.month == missed.month)
        .unwrap();
    assert!(
        live.order_receipts
            .iter()
            .any(|r| r.reason == town_market::OrderReason::PlannerWithheld)
    );
    assert!(live.selections.is_some());
    // A bid from one person never submits its counterparty's ask automatically.
    for order in &live.orders {
        assert!(
            missed.exchanges[&order.agent]
                .submit
                .contains(&(order.market, order.side))
        );
    }
    assert_eq!(
        sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
        12
    );
    assert!(
        sim.reports
            .iter()
            .any(|r| r.deficit(NUTRITION) > 0 || r.deficit(WARMTH) > 0)
    );
}
#[test]
fn joint_masks_require_every_actor_and_do_not_override_eligibility() {
    let mut sim = liquid_book();
    let mut masks = all_orders();
    masks.remove(&89);
    assert!(
        town_market::evaluate_selections(&sim.world, &sim.state, &masks)
            .unwrap_err()
            .contains("every participant")
    );
    masks.insert(89, Default::default());
    let round = town_market::evaluate_selections(&sim.world, &sim.state, &masks).unwrap();
    assert!(round.transactions.is_empty());
    assert!(
        round
            .order_receipts
            .iter()
            .any(|r| r.agent == 89 && r.reason == town_market::OrderReason::PlannerWithheld)
    );
    masks.get_mut(&PERSON).unwrap().insert((999, Side::Buy));
    assert!(town_market::evaluate_selections(&sim.world, &sim.state, &masks).is_err());
    // Admission is captured at Open; a supplied mask cannot reopen the venue.
    sim.state
        .town_market
        .admission
        .as_mut()
        .unwrap()
        .eligible
        .remove(&89);
    let round = town_market::evaluate_selections(&sim.world, &sim.state, &all_orders()).unwrap();
    assert!(round.transactions.is_empty());
    assert!(
        round
            .order_receipts
            .iter()
            .any(|r| r.agent == 89 && r.reason == town_market::OrderReason::NotAdmitted)
    );
}
#[test]
fn same_book_sale_income_is_not_opening_purchase_money() {
    let mut sim = liquid_book();
    sim.state.balances.insert((PERSON, TOKEN), 0);
    let batch = market::prepare_all(&sim, &[], &all_orders()).unwrap();
    settlement::commit(
        &sim.world,
        &mut sim.state,
        &batch,
        sim.backend,
        sim.effect_limit,
    )
    .unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 1);
    assert_eq!(sim.state.balance(89, FUEL), 1);
}
#[test]
fn unfilled_input_order_cannot_authorize_new_production() {
    let mut sim = liquid_book();
    sim.world
        .definitions
        .iter_mut()
        .find(|d| d.id == PREPARE_FUEL)
        .unwrap()
        .stages[0]
        .entry_inputs = vec![Amount::new(GRAIN, 1)];
    let work = [Request::new(Id::Process(PREPARE_FUEL), PERSON)];
    let mut masks = all_orders();
    masks.get_mut(&89).unwrap().clear();
    let before = sim.state.clone();
    assert!(
        market::prepare_all(&sim, &work, &masks)
            .unwrap_err()
            .contains("offer bundle")
    );
    assert_eq!(sim.state, before);
    let batch = market::prepare_all(&sim, &work, &all_orders()).unwrap();
    settlement::commit(
        &sim.world,
        &mut sim.state,
        &batch,
        sim.backend,
        sim.effect_limit,
    )
    .unwrap();
    assert_eq!(
        sim.state.balance(PERSON, GRAIN),
        1,
        "Acquire delivers the input; work remains reserved"
    );
    sim.step().unwrap();
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
    assert!(sim.state.processes.values().any(|p| p.operator == PERSON
        && p.definition == PREPARE_FUEL
        && p.status == Status::Completed));
}
#[test]
fn tampered_trade_receipt_and_failed_coordinator_leave_state_unchanged() {
    let mut sim = liquid_book();
    let batch = market::prepare_all(&sim, &[], &all_orders()).unwrap();
    assert!(!batch.transactions.is_empty());
    for alter_mask in [false, true] {
        let mut bad = batch.clone();
        if alter_mask {
            let Some(town_market::Boundary::Market(r)) = &mut bad.town_market else {
                unreachable!()
            };
            r.selections.as_mut().unwrap().get_mut(&89).unwrap().clear();
        } else {
            bad.transactions.clear();
        }
        let before = sim.state.clone();
        assert!(
            settlement::commit(
                &sim.world,
                &mut sim.state,
                &bad,
                sim.backend,
                sim.effect_limit
            )
            .is_err()
        );
        assert_eq!(sim.state, before);
    }
    sim.effect_limit = 0;
    let mut p = planner(&sim);
    let before = sim.state.clone();
    let old = p.clone();
    assert!(p.step(&mut sim).is_err());
    assert_eq!(sim.state, before);
    assert_eq!(p, old);
}
#[test]
fn market_continuation_rejects_unsupported_reviews_before_advancing() {
    for review in [Policy::RetainRepair, Policy::ScheduledReview] {
        let (mut sim, _) = setup(Backend::Reference, true, 1);
        let mut p = Persons::new(
            [PERSON, 89],
            Strategy::BestFirst,
            Budget {
                expansions: 256,
                forecasts: 32,
                months: 6,
            },
            review,
            Allocation::StablePriority,
            7,
        )
        .unwrap();
        let before = sim.state.clone();
        assert!(p.step(&mut sim).unwrap_err().contains("monthly review"));
        assert_eq!(sim.state, before);
        assert!(p.history.is_empty());
    }
}
#[test]
fn cpu_reordering_and_checkpoint_resume_preserve_exchange_and_separate_books() {
    fn step(
        sim: &mut Simulation,
        p: &mut Persons,
        audit: &mut economics_compute_smoke::financial_reporting::Audit,
    ) {
        let before = sim.state.clone();
        p.step(sim).unwrap();
        audit
            .record(&sim.world, &before, sim.ledger.last().unwrap(), &sim.state)
            .unwrap();
    }
    let mut results = vec![];
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let (mut sim, mut p) = setup(backend, true, 1);
        if matches!(backend, Backend::CubeCpu) {
            sim.world.participants.reverse();
            sim.world.definitions.reverse();
            sim.world.rights.reverse();
            let m = sim.world.town_market.as_mut().unwrap();
            m.traders.reverse();
            for l in &mut m.additional {
                l.traders.reverse();
            }
        }
        let mut audit = fixtures::audit(&sim).unwrap();
        while sim.state.month < 5 || sim.state.phase != Phase::Productive {
            step(&mut sim, &mut p, &mut audit);
        }
        assert!(sim.state.pending_production.is_some());
        let (mut resumed, mut rp, mut ra) = (sim.clone(), p.clone(), audit.clone());
        while sim.state.month < 13 {
            step(&mut sim, &mut p, &mut audit);
        }
        for end in 6..=13 {
            while resumed.state.month < end {
                step(&mut resumed, &mut rp, &mut ra);
            }
        }
        assert_eq!(sim.state, resumed.state);
        assert_eq!(sim.ledger, resumed.ledger);
        assert_eq!(p, rp);
        assert_eq!(audit, ra);
        audit.finalize_through(12).unwrap();
        results.push((sim.state, sim.ledger, p.history, audit));
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn standing_order_forecast_improves_the_low_cash_control_without_changing_endowments() {
    let mut outcomes = vec![];
    for forecast in [
        market::OrderForecast::CurrentBoundaryOnly,
        market::OrderForecast::StandingPolicy,
    ] {
        let (mut sim, mut p) = setup(Backend::Reference, true, 1);
        p.order_forecast = forecast;
        advance(&mut sim, &mut p, 25);
        let deficits: i32 = sim
            .reports
            .iter()
            .map(|r| r.deficit(NUTRITION) + r.deficit(WARMTH))
            .sum();
        let terminal = sim.reports.iter().filter(|r| r.terminal.is_some()).count();
        outcomes.push((deficits, terminal));
        assert_eq!(
            sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
            2
        );
    }
    assert!(outcomes[1].0 < outcomes[0].0);
    assert_eq!(outcomes[1].1, 0);
    assert!(outcomes[0].1 > 0);
}
