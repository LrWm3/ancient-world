#[path = "support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    allocation::Policy as Allocation,
    composition::{
        Budget, Strategy,
        continuation::{Policy, persons::Persons, posted},
    },
    compute::Backend,
    marketplace::Side,
    model::*,
    production_market::WOOD_MARKET,
    scenario::*,
    settlement,
    simulation::Simulation,
    town_market::{self, OrderSelections},
};
fn setup(
    backend: Backend,
    strategy: Strategy,
    coins: i32,
    trading: bool,
) -> (Simulation, posted::Controller) {
    let mut sim = fixtures::trading_persons(backend, trading).unwrap();
    for a in [PERSON, 89] {
        sim.state.balances.insert((a, TOKEN), coins);
    }
    let p = Persons::new(
        [PERSON, 89],
        strategy,
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
    (sim, posted::Controller::new(p))
}
fn masks() -> OrderSelections {
    [
        (PERSON, [(1, Side::Buy), (WOOD_MARKET, Side::Sell)].into()),
        (89, [(1, Side::Sell), (WOOD_MARKET, Side::Buy)].into()),
    ]
    .into()
}
fn liquid() -> Simulation {
    let (mut sim, _) = setup(Backend::Reference, Strategy::BestFirst, 1, true);
    sim.state.balances.extend([
        ((PERSON, GRAIN), 0),
        ((PERSON, FUEL), 5),
        ((89, GRAIN), 10),
        ((89, FUEL), 0),
    ]);
    fixtures::acquire(&mut sim).unwrap();
    sim
}
fn batch(sim: &Simulation, round: town_market::Round) -> Batch {
    let mut b = Batch::empty(&sim.state);
    b.transactions = round.transactions.clone();
    b.town_market = Some(town_market::Boundary::Market(round));
    b
}
#[test]
fn reciprocal_spot_requires_both_funded_deliveries_and_exact_consents() {
    let sim = liquid();
    let ordinary = town_market::evaluate_selections(&sim.world, &sim.state, &masks()).unwrap();
    let terms = town_market::spot_deliveries(&ordinary);
    assert_eq!(terms.len(), 2);
    let accepted =
        town_market::evaluate_conditional(&sim.world, &sim.state, &masks(), &terms).unwrap();
    assert_eq!(accepted.transactions, ordinary.transactions);
    for fault in 0..6 {
        let mut bad = sim.clone();
        let mut consent = masks();
        let mut offer = terms.clone();
        match fault {
            0 => {
                bad.state.balances.insert((PERSON, TOKEN), 0);
            }
            1 => {
                bad.state.balances.insert((PERSON, FUEL), 0);
            }
            2 => {
                offer[0].month += 1;
            }
            3 => {
                offer[0].payment.amount.quantity += 1;
            }
            4 => {
                consent.get_mut(&89).unwrap().clear();
            }
            _ => {
                bad.state
                    .town_market
                    .admission
                    .as_mut()
                    .unwrap()
                    .eligible
                    .remove(&89);
            }
        }
        let before = bad.state.clone();
        assert!(
            town_market::evaluate_conditional(&bad.world, &bad.state, &consent, &offer).is_err()
        );
        assert_eq!(bad.state, before);
    }
    // The ordinary book would execute one sale despite the missing other payment.
    let mut poor = sim.clone();
    poor.state.balances.insert((PERSON, TOKEN), 0);
    assert_eq!(
        town_market::spot_deliveries(
            &town_market::evaluate_selections(&poor.world, &poor.state, &masks()).unwrap()
        )
        .len(),
        1
    );
    assert!(town_market::evaluate_conditional(&poor.world, &poor.state, &masks(), &terms).is_err());
}
#[test]
fn forged_conditions_and_replay_do_not_publish_any_trade() {
    let mut sim = liquid();
    let terms = town_market::spot_deliveries(
        &town_market::evaluate_selections(&sim.world, &sim.state, &masks()).unwrap(),
    );
    let round =
        town_market::evaluate_conditional(&sim.world, &sim.state, &masks(), &terms).unwrap();
    let good = batch(&sim, round);
    for fault in 0..3 {
        let mut bad = good.clone();
        let Some(town_market::Boundary::Market(r)) = &mut bad.town_market else {
            panic!()
        };
        match fault {
            0 => {
                r.conditional.as_mut().unwrap()[0].payment.amount.quantity += 1;
            }
            1 => {
                r.selections.as_mut().unwrap().get_mut(&89).unwrap().clear();
            }
            _ => {
                bad.transactions.pop();
            }
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
    settlement::commit(
        &sim.world,
        &mut sim.state,
        &good,
        sim.backend,
        sim.effect_limit,
    )
    .unwrap();
    let before = sim.state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &good,
            sim.backend,
            sim.effect_limit
        )
        .is_err()
    );
    assert_eq!(sim.state, before);
    let mut corrupt = sim.state.clone();
    corrupt
        .town_market
        .history
        .last_mut()
        .unwrap()
        .conditional
        .as_mut()
        .unwrap()[0]
        .month += 1;
    assert!(Simulation::new(sim.world.clone(), corrupt, Backend::Reference).is_err());
}
#[test]
fn independent_consents_publish_only_current_spot_terms_and_own_work() {
    let (mut sim, mut p) = setup(Backend::Reference, Strategy::Beam, 6, true);
    while sim.state.month < 13 {
        p.step(&mut sim).unwrap();
    }
    let accepted: Vec<_> = p.history.iter().filter(|r| r.accepted).collect();
    assert!(!accepted.is_empty());
    for r in accepted {
        assert_eq!(r.assessments.len(), 2);
        assert!(r.assessments.iter().all(|a| a.acceptable
            && a.offered <= a.outside
            && a.requests.iter().all(|w| w.agent == a.actor)));
        let proposer = r
            .assessments
            .iter()
            .find(|a| a.actor == r.proposer)
            .unwrap();
        assert!(proposer.offered < proposer.outside);
        let book = sim
            .state
            .town_market
            .history
            .iter()
            .find(|b| b.month == r.month)
            .unwrap();
        assert_eq!(book.conditional.as_ref().unwrap(), &r.terms);
        assert!(r.terms.iter().all(|d| d.month == r.month));
        assert_eq!(town_market::spot_deliveries(book), r.terms);
    }
    assert!(sim.state.credit.loans.is_empty());
    assert_eq!(
        sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
        12
    );
}
#[test]
fn offered_exchange_increases_high_cash_trade_but_does_not_establish_survival() {
    for coins in [1, 6] {
        let mut outcomes = vec![];
        for conditional in [false, true] {
            let (mut sim, mut p) = setup(Backend::Reference, Strategy::BestFirst, coins, true);
            while sim.state.month < 25 {
                if conditional {
                    p.step(&mut sim).unwrap();
                } else {
                    p.persons.step(&mut sim).unwrap();
                }
            }
            let volume: i32 = sim
                .state
                .town_market
                .history
                .iter()
                .flat_map(|r| r.markets.values())
                .map(|m| m.volume)
                .sum();
            let terminal = sim.reports.iter().filter(|r| r.terminal.is_some()).count();
            outcomes.push((volume, terminal));
            assert_eq!(
                sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
                2 * coins
            );
        }
        if coins == 1 {
            assert_eq!(outcomes[0], outcomes[1]);
            assert_eq!(outcomes[1].1, 0);
        } else {
            assert!(outcomes[1].0 > outcomes[0].0);
            assert!(outcomes[1].1 > 0);
        }
    }
}
#[test]
fn closed_books_fall_back_and_failed_boundaries_leave_driver_unchanged() {
    let (mut sim, mut p) = setup(Backend::Reference, Strategy::BestFirst, 6, false);
    let mut baseline = sim.clone();
    let mut ordinary = p.persons.clone();
    while sim.state.month < 5 {
        p.step(&mut sim).unwrap();
        ordinary.step(&mut baseline).unwrap();
    }
    assert_eq!(sim.state, baseline.state);
    assert_eq!(sim.ledger, baseline.ledger);
    assert!(
        p.history
            .iter()
            .all(|r| !r.accepted && r.assessments.is_empty())
    );
    while sim.state.phase != Phase::Acquire {
        p.step(&mut sim).unwrap();
    }
    sim.effect_limit = 0;
    let before = sim.state.clone();
    let driver = p.clone();
    assert!(p.step(&mut sim).is_err());
    assert_eq!(sim.state, before);
    assert_eq!(p, driver);
}
#[test]
fn cpu_checkpoint_and_reordered_registrations_preserve_consents_and_separate_books() {
    fn step(
        sim: &mut Simulation,
        p: &mut posted::Controller,
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
        let (mut sim, mut p) = setup(backend, Strategy::Beam, 6, true);
        if matches!(backend, Backend::CubeCpu) {
            sim.world.participants.reverse();
            sim.world.definitions.reverse();
            sim.world.rights.reverse();
            let c = sim.world.town_market.as_mut().unwrap();
            c.traders.reverse();
            for l in &mut c.additional {
                l.traders.reverse();
            }
        }
        let mut audit = fixtures::audit(&sim).unwrap();
        while !p.history.last().is_some_and(|r| r.accepted) || sim.state.phase != Phase::Productive
        {
            assert!(sim.state.month < 13);
            step(&mut sim, &mut p, &mut audit);
        }
        assert!(sim.state.pending_production.is_some());
        let (mut resumed, mut rp, mut ra) = (sim.clone(), p.clone(), audit.clone());
        while sim.state.month < 13 {
            step(&mut sim, &mut p, &mut audit);
        }
        while resumed.state.month < 13 {
            let end = resumed.state.month + 1;
            while resumed.state.month < end {
                step(&mut resumed, &mut rp, &mut ra);
            }
        }
        assert_eq!(sim.state, resumed.state);
        assert_eq!(sim.ledger, resumed.ledger);
        assert_eq!(p, rp);
        assert_eq!(audit, ra);
        audit.finalize_through(12).unwrap();
        results.push((sim.state, sim.ledger, p, audit));
    }
    assert_eq!(results[0], results[1]);
}
