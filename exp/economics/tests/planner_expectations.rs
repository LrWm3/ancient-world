#[path = "support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    allocation::Policy as Allocation,
    composition::{
        Budget, Strategy,
        continuation::{Policy, persons::Persons},
        expectations::{Policy as Counterparties, Snapshot},
    },
    compute::Backend,
    marketplace::Side,
    model::*,
    scenario::*,
    simulation::Simulation,
    town_market::{self, OrderReason},
};

fn setup(backend: Backend, policy: Counterparties) -> (Simulation, Persons) {
    let mut sim = fixtures::trading_persons(backend, true).unwrap();
    for a in [PERSON, 89] {
        sim.state.balances.insert((a, TOKEN), 1);
    }
    let mut p = Persons::new(
        [PERSON, 89],
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
    .unwrap();
    p.counterparty_policy = policy;
    (sim, p)
}

#[test]
fn observation_distinguishes_withholding_ineligibility_and_unfilled_submission() {
    let (mut sim, _) = setup(Backend::Reference, Counterparties::Ordinary);
    sim.state.balances.extend([
        ((PERSON, GRAIN), 0),
        ((PERSON, FUEL), 5),
        ((89, GRAIN), 10),
        ((89, FUEL), 0),
    ]);
    fixtures::acquire(&mut sim).unwrap();
    let mut round = town_market::evaluate(&sim.world, &sim.state).unwrap();
    let key = (89, 1, Side::Sell);
    let withheld = round
        .order_receipts
        .iter_mut()
        .find(|r| (r.agent, r.market, r.side) == key)
        .unwrap();
    withheld.reason = OrderReason::PlannerWithheld;
    // Synthetic receipts isolate evidence semantics from the live matcher.
    round.attempts.clear();
    sim.state.town_market.history = vec![round.clone()];
    sim.state.month = 2;
    let policy = Counterparties::RecentSubmission { memory_months: 3 };
    let frozen = Snapshot::observe(&sim.state, PERSON, policy).unwrap();
    assert!(!frozen.expects_submission(key, 2));
    assert!(!frozen.expects_submission(key, 4));
    assert!(frozen.expects_submission(key, 5));
    assert!(frozen.evidence.keys().all(|k| k.0 != PERSON));
    assert!(frozen.expects_submission((99, 1, Side::Sell), 2));

    round.month = 2;
    round
        .order_receipts
        .iter_mut()
        .find(|r| (r.agent, r.market, r.side) == key)
        .unwrap()
        .reason = OrderReason::ProtectedStock;
    sim.state.town_market.history.push(round.clone());
    sim.state.month = 3;
    let absent = Snapshot::observe(&sim.state, PERSON, policy).unwrap();
    assert_eq!(absent.evidence[&key].month, 1);
    assert!(!absent.expects_submission(key, 3));

    // An eligible submitted but unfilled order is willingness, not a guarantee.
    round
        .order_receipts
        .iter_mut()
        .find(|r| (r.agent, r.market, r.side) == key)
        .unwrap()
        .reason = OrderReason::Submitted;
    sim.state.town_market.history.pop();
    sim.state.town_market.history.push(round.clone());
    let submitted = Snapshot::observe(&sim.state, PERSON, policy).unwrap();
    assert!(submitted.expects_submission(key, 3));
    assert_eq!(submitted.evidence[&key].filled, 0);
    assert!(
        !frozen.expects_submission(key, 3),
        "frozen hypotheses do not learn from later/imagined books"
    );

    sim.state.month = 2;
    assert_eq!(
        Snapshot::observe(&sim.state, PERSON, policy).unwrap(),
        frozen,
        "same-month books cannot inform this simultaneous decision"
    );
    round.month = 100;
    sim.state.town_market.history.push(round);
    assert_eq!(
        Snapshot::observe(&sim.state, PERSON, policy).unwrap(),
        frozen
    );
    sim.state.month = 6;
    assert!(
        !Snapshot::observe(&sim.state, PERSON, policy)
            .unwrap()
            .evidence
            .contains_key(&key)
    );
}

#[test]
fn cold_start_matches_baseline_and_only_previous_live_books_inform_later_search() {
    let mut openings = vec![];
    for policy in [
        Counterparties::Ordinary,
        Counterparties::RecentSubmission { memory_months: 3 },
    ] {
        let (mut sim, mut p) = setup(Backend::Reference, policy);
        while sim.state.phase != Phase::Acquire {
            p.step(&mut sim).unwrap();
        }
        p.step(&mut sim).unwrap();
        openings.push((sim.state.clone(), sim.ledger.clone()));
        while sim.state.month < 7 {
            let before = sim.state.clone();
            p.step(&mut sim).unwrap();
            if before.phase != Phase::Acquire {
                continue;
            }
            let round = p.history.last().unwrap();
            let town_market::Boundary::Market(book) =
                sim.ledger.last().unwrap().town_market.as_ref().unwrap()
            else {
                panic!()
            };
            for (&actor, x) in &round.exchanges {
                assert_eq!(
                    x.expectations,
                    Snapshot::observe(&before, actor, policy).unwrap()
                );
                assert_eq!(
                    book.selections.as_ref().unwrap()[&actor],
                    x.submit,
                    "live consent comes from each actor, not another's hypothesis"
                );
                assert!(round.proposals[&actor].iter().all(|r| r.agent == actor));
            }
            assert_eq!(
                sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
                2
            );
        }
    }
    assert_eq!(openings[0], openings[1]);
}

#[test]
fn invalid_memory_and_failed_settlement_do_not_publish_learning_or_decisions() {
    let (mut sim, mut p) = setup(
        Backend::Reference,
        Counterparties::RecentSubmission { memory_months: 0 },
    );
    let before = sim.state.clone();
    let old = p.clone();
    assert!(p.step(&mut sim).unwrap_err().contains("memory"));
    assert_eq!(sim.state, before);
    assert_eq!(p, old);
    p.counterparty_policy = Counterparties::RecentSubmission { memory_months: 3 };
    while sim.state.month < 3 || sim.state.phase != Phase::Acquire {
        p.step(&mut sim).unwrap();
    }
    assert!(!sim.state.town_market.history.is_empty());
    sim.effect_limit = 0;
    let before = sim.state.clone();
    let old = p.clone();
    assert!(p.step(&mut sim).is_err());
    assert_eq!(sim.state, before);
    assert_eq!(p, old);
}

#[test]
fn learned_forecasts_resume_and_reorder_on_cpu_with_separate_accounting() {
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
    let mut outcomes = vec![];
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let (mut sim, mut p) = setup(
            backend,
            Counterparties::RecentSubmission { memory_months: 3 },
        );
        if matches!(backend, Backend::CubeCpu) {
            sim.world.participants.reverse();
            sim.world.definitions.reverse();
            sim.world.rights.reverse();
            let market = sim.world.town_market.as_mut().unwrap();
            market.traders.reverse();
            for l in &mut market.additional {
                l.traders.reverse();
            }
        }
        let mut audit = fixtures::audit(&sim).unwrap();
        while sim.state.month < 5 || sim.state.phase != Phase::Productive {
            step(&mut sim, &mut p, &mut audit);
        }
        assert!(
            p.history
                .iter()
                .flat_map(|r| r.exchanges.values())
                .any(|x| !x.expectations.evidence.is_empty())
        );
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
        outcomes.push((sim.state, sim.ledger, p.history, audit));
    }
    assert_eq!(outcomes[0], outcomes[1]);
}

#[test]
fn learned_policy_preserves_low_cash_survival_but_does_not_solve_high_cash_coordination() {
    let mut high_cash = vec![];
    for policy in [
        Counterparties::Ordinary,
        Counterparties::RecentSubmission { memory_months: 3 },
    ] {
        for coins in [1, 6] {
            let (mut sim, mut p) = setup(Backend::Reference, policy);
            for a in [PERSON, 89] {
                sim.state.balances.insert((a, TOKEN), coins);
            }
            while sim.state.month < 25 {
                p.step(&mut sim).unwrap();
            }
            assert_eq!(
                sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
                2 * coins
            );
            let terminal = sim.reports.iter().filter(|r| r.terminal.is_some()).count();
            let volume: i32 = sim
                .state
                .town_market
                .history
                .iter()
                .flat_map(|r| r.markets.values())
                .map(|m| m.volume)
                .sum();
            if coins == 1 {
                assert_eq!(terminal, 0);
                assert!(volume >= 36);
                assert!(
                    p.history
                        .iter()
                        .flat_map(|r| r.exchanges.values())
                        .flat_map(|x| x.expectations.evidence.values())
                        .any(|e| e.filled > 0)
                        || policy == Counterparties::Ordinary
                );
            } else {
                assert!(
                    terminal > 0,
                    "neither forecast model demonstrates reliable high-cash coordination"
                );
                high_cash.push((terminal, volume));
            }
        }
    }
    assert!(high_cash[1].0 > high_cash[0].0);
    assert!(
        high_cash[1].1 < high_cash[0].1,
        "remembering refusals can entrench inactivity"
    );
}

#[test]
fn peer_hypotheses_are_independent_of_the_own_order_forecast_variant() {
    for own in [
        economics_compute_smoke::composition::market::OrderForecast::CurrentBoundaryOnly,
        economics_compute_smoke::composition::market::OrderForecast::StandingPolicy,
    ] {
        let (mut sim, mut p) = setup(
            Backend::Reference,
            Counterparties::RecentSubmission { memory_months: 1 },
        );
        p.order_forecast = own;
        while sim.state.month < 7 {
            p.step(&mut sim).unwrap();
        }
        assert_eq!(p.history.len(), 6);
        assert!(
            p.history
                .iter()
                .flat_map(|r| r.exchanges.values())
                .any(|x| !x.expectations.evidence.is_empty())
        );
        assert_eq!(
            sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
            2
        );
        assert!(
            p.controllers
                .values()
                .flat_map(|c| &c.history)
                .filter_map(|r| r.search.as_ref())
                .all(|m| m.forecasts <= 32 && m.expansions <= 256)
        );
    }
}
