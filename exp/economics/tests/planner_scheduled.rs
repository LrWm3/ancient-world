#[path = "support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    allocation::Policy as Allocation,
    composition::{
        Budget, Strategy,
        continuation::{Policy, persons::Persons, scheduled},
    },
    compute::Backend,
    cooperation,
    model::*,
    scenario::*,
    settlement,
    simulation::Simulation,
    town_market::{self, Boundary},
};
fn setup(backend: Backend) -> (Simulation, scheduled::Controller) {
    let sim = fixtures::trading_persons(backend, true).unwrap();
    let p = Persons::new(
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
    (sim, scheduled::Controller::new(p))
}
fn as_batch(sim: &Simulation, round: town_market::Round) -> Batch {
    let mut b = Batch::empty(&sim.state);
    b.transactions = round.transactions.clone();
    b.town_market = Some(Boundary::Market(round));
    b
}
fn admit(sim: &mut Simulation) -> cooperation::Contract {
    fixtures::acquire(sim).unwrap();
    let c = scheduled::proposal(sim).unwrap();
    let r = cooperation::evaluate_schedule(&sim.world, &sim.state, Some(&c), &[PERSON, 89].into())
        .unwrap();
    assert!(r.transactions.is_empty());
    let b = as_batch(sim, r);
    settlement::commit(
        &sim.world,
        &mut sim.state,
        &b,
        sim.backend,
        sim.effect_limit,
    )
    .unwrap();
    sim.ledger.push(b);
    c
}
#[test]
fn dated_terms_require_consents_and_survive_a_change_of_work_controller() {
    let (mut sim, _) = setup(Backend::Reference);
    fixtures::acquire(&mut sim).unwrap();
    let c = scheduled::proposal(&sim).unwrap();
    assert!(c.choices.is_empty());
    assert!(c.deliveries.iter().all(|d| d.month > sim.state.month));
    let before = sim.state.clone();
    for consent in [Default::default(), [PERSON].into()] {
        assert!(
            cooperation::evaluate_schedule(&sim.world, &sim.state, Some(&c), &consent).is_err()
        );
    }
    for fault in 0..5 {
        let mut bad = c.clone();
        match fault {
            0 => {
                bad.deliveries.push(bad.deliveries[0].clone());
            }
            1 => {
                bad.through += 1;
            }
            2 => {
                bad.deliveries[0].payment.amount.quantity += 1;
            }
            3 => {
                bad.deliveries[0].goods.to = 999999;
            }
            _ => {
                bad.deliveries[0].month = 0;
            }
        }
        assert!(
            cooperation::evaluate_schedule(
                &sim.world,
                &sim.state,
                Some(&bad),
                &[PERSON, 89].into()
            )
            .is_err()
        );
    }
    assert_eq!(before, sim.state);
    let c = admit(&mut sim);
    assert_eq!(cooperation::active_independent(&sim.state), Some(&c));
    assert!(
        economics_compute_smoke::agreements::for_agent(&sim.world, &sim.state, PERSON)
            .unwrap()
            .iter()
            .any(|v| matches!(v, economics_compute_smoke::agreements::View::Exchange(_)))
    );
    // The ordinary simulator must honor terms even without the accepting controller.
    while sim.state.month == 1 {
        sim.step().unwrap();
    }
    fixtures::acquire(&mut sim).unwrap();
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert_eq!(r.cooperation.as_ref().unwrap().terms.as_ref(), Some(&c));
    assert_eq!(r.cooperation.as_ref().unwrap().completed.len(), 2);
    assert!(r.selection.is_none() && r.selections.is_none());
    let withheld = town_market::evaluate_selections(
        &sim.world,
        &sim.state,
        &[(PERSON, Default::default()), (89, Default::default())].into(),
    )
    .unwrap();
    assert_eq!(withheld, r);
    let b = as_batch(&sim, r);
    settlement::commit(
        &sim.world,
        &mut sim.state,
        &b,
        sim.backend,
        sim.effect_limit,
    )
    .unwrap();
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &b,
            sim.backend,
            sim.effect_limit
        )
        .is_err()
    );
}
#[test]
fn missed_goods_cash_storage_or_admission_cancel_the_remainder_without_partial_payment() {
    for fault in 0..4 {
        let (mut sim, _) = setup(Backend::Reference);
        let c = admit(&mut sim);
        while sim.state.month == 1 {
            sim.step().unwrap();
        }
        fixtures::acquire(&mut sim).unwrap();
        match fault {
            0 => {
                sim.state.balances.insert((89, GRAIN), 0);
            }
            1 => {
                sim.state.balances.insert((PERSON, TOKEN), 0);
            }
            2 => {
                let used = economics_compute_smoke::storage::usage(&sim.world, &sim.state.balances)
                    [&PERSON];
                sim.world
                    .storage
                    .capacities
                    .insert(PERSON, i32::try_from(used).unwrap());
            }
            _ => {
                sim.state
                    .town_market
                    .admission
                    .as_mut()
                    .unwrap()
                    .eligible
                    .remove(&89);
            }
        }
        let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
        let b = r.cooperation.as_ref().unwrap();
        assert!(b.failure.is_some(), "fault {fault}");
        assert!(b.active.is_none());
        assert!(b.completed.is_empty() && r.transactions.is_empty());
        assert_eq!(b.terms.as_ref(), Some(&c));
        let batch = as_batch(&sim, r);
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &batch,
            sim.backend,
            sim.effect_limit,
        )
        .unwrap();
        assert!(cooperation::active_independent(&sim.state).is_none());
        while sim.state.month == 2 {
            sim.step().unwrap();
        }
        fixtures::acquire(&mut sim).unwrap();
        assert!(
            town_market::evaluate(&sim.world, &sim.state)
                .unwrap()
                .cooperation
                .is_none()
        );
        assert!(
            cooperation::evaluate_schedule(&sim.world, &sim.state, Some(&c), &[PERSON, 89].into())
                .is_err()
        );
    }
}
#[test]
fn forged_schedule_terms_consents_and_transfers_are_rejected_atomically() {
    let (mut sim, _) = setup(Backend::Reference);
    let c = admit(&mut sim);
    while sim.state.month == 1 {
        sim.step().unwrap();
    }
    fixtures::acquire(&mut sim).unwrap();
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    let good = as_batch(&sim, r);
    for fault in 0..4 {
        let mut bad = good.clone();
        let Some(Boundary::Market(r)) = &mut bad.town_market else {
            unreachable!()
        };
        match fault {
            0 => {
                r.cooperation
                    .as_mut()
                    .unwrap()
                    .terms
                    .as_mut()
                    .unwrap()
                    .deliveries[0]
                    .payment
                    .amount
                    .quantity += 1;
            }
            1 => {
                r.cooperation.as_mut().unwrap().consents.insert(PERSON);
            }
            2 => {
                r.transactions.clear();
                bad.transactions.clear();
            }
            _ => {
                r.cooperation.as_mut().unwrap().active = None;
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
        assert_eq!(before, sim.state);
    }
    let mut rewrite = c;
    rewrite.start = sim.state.month;
    rewrite.through += 1;
    assert!(
        cooperation::evaluate_schedule(
            &sim.world,
            &sim.state,
            Some(&rewrite),
            &[PERSON, 89].into()
        )
        .is_err()
    );
}
#[test]
fn composition_assesses_own_work_and_follows_dated_terms() {
    let (mut sim, mut driver) = setup(Backend::Reference);
    while sim.state.month <= 12 {
        driver.step(&mut sim).unwrap();
    }
    assert!(driver.history.iter().any(|r| r.accepted));
    for r in driver.history.iter().filter(|r| r.accepted) {
        assert_eq!(r.assessments.len(), 2);
        assert!(
            r.assessments
                .iter()
                .all(|a| a.acceptable && a.offered.broken_commitments == 0)
        );
        assert!(r.assessments[0].offered < r.assessments[0].outside);
        assert!(
            r.assessments
                .iter()
                .all(|a| a.requests.iter().all(|x| x.agent == a.actor))
        );
    }
    assert_eq!(
        sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
        12
    );
}
#[test]
fn cpu_checkpoint_and_reordered_registrations_preserve_consents_and_separate_books() {
    fn step(
        sim: &mut Simulation,
        p: &mut scheduled::Controller,
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
        let (mut sim, mut p) = setup(backend);
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
        resumed = Simulation::new(resumed.world.clone(), resumed.state.clone(), backend).unwrap();
        resumed.ledger = sim.ledger.clone();
        resumed.reports = sim.reports.clone();
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
#[test]
fn reconstructed_history_rejects_dropped_rewritten_or_resurrected_commitments() {
    let (mut sim, _) = setup(Backend::Reference);
    admit(&mut sim);
    while sim.state.month <= 2 {
        sim.step().unwrap();
    }
    for fault in 0..5 {
        let mut state = sim.state.clone();
        match fault {
            0 => {
                state.town_market.history[1].cooperation = None;
            }
            1 => {
                state.town_market.history[0]
                    .cooperation
                    .as_mut()
                    .unwrap()
                    .consents
                    .clear();
            }
            2 => {
                state.town_market.history[1]
                    .cooperation
                    .as_mut()
                    .unwrap()
                    .completed
                    .clear();
            }
            3 => {
                let b = state.town_market.history[0].cooperation.as_mut().unwrap();
                b.failure = Some("missed".into());
                b.active = None;
            }
            _ => {
                state.town_market.history[1]
                    .cooperation
                    .as_mut()
                    .unwrap()
                    .terms
                    .as_mut()
                    .unwrap()
                    .deliveries[0]
                    .month += 1;
            }
        }
        assert!(
            Simulation::new(sim.world.clone(), state, Backend::Reference).is_err(),
            "fault {fault}"
        );
    }
}
#[test]
fn closed_book_and_failed_commit_keep_the_fallback_and_controller_atomic() {
    let (mut sim, mut driver) = setup(Backend::Reference);
    sim.world.town_market.as_mut().unwrap().match_limit = Some(0);
    sim.world.town_market.as_mut().unwrap().additional[0].match_limit = Some(0);
    let (mut ordinary, mut persons) = (sim.clone(), driver.persons.clone());
    while sim.state.month < 5 {
        driver.step(&mut sim).unwrap();
        persons.step(&mut ordinary).unwrap();
    }
    assert_eq!(sim.state, ordinary.state);
    assert_eq!(sim.ledger, ordinary.ledger);
    assert!(driver.history.iter().all(|r| r.assessments.is_empty()));
    let (mut sim, mut driver) = setup(Backend::Reference);
    fixtures::acquire(&mut sim).unwrap();
    sim.effect_limit = 0;
    let (state, before) = (sim.state.clone(), driver.clone());
    assert!(driver.step(&mut sim).is_err());
    assert_eq!(sim.state, state);
    assert_eq!(driver, before);
}
#[test]
fn dated_planning_sustains_the_high_cash_pair_but_low_cash_spot_has_fewer_shortfalls() {
    for coins in [1, 6] {
        let mut outcomes = vec![];
        for scheduled in [false, true] {
            let (mut sim, mut driver) = setup(Backend::Reference);
            for person in [PERSON, 89] {
                sim.state.balances.insert((person, TOKEN), coins);
            }
            while sim.state.month <= 24 {
                if scheduled {
                    driver.step(&mut sim).unwrap();
                } else {
                    driver.persons.step(&mut sim).unwrap();
                }
            }
            let deficit: i32 = sim
                .reports
                .iter()
                .map(|r| r.deficit(NUTRITION) + r.deficit(WARMTH))
                .sum();
            let terminal = sim.reports.iter().filter(|r| r.terminal.is_some()).count();
            outcomes.push((deficit, terminal));
            assert_eq!(
                sim.state.balance(PERSON, TOKEN) + sim.state.balance(89, TOKEN),
                2 * coins
            );
            if scheduled {
                assert_eq!(terminal, 0);
                assert!(
                    !sim.state
                        .town_market
                        .history
                        .iter()
                        .any(|r| r.cooperation.as_ref().is_some_and(|b| b.failure.is_some()))
                );
                assert!(cooperation::active_independent(&sim.state).is_some());
            }
        }
        if coins == 1 {
            assert_eq!(outcomes[0].1, 0);
            assert!(outcomes[0].0 < outcomes[1].0);
        } else {
            assert!(outcomes[0].1 > 0);
        }
    }
}
#[test]
fn a_lost_delivery_is_cancelled_by_the_live_planner_without_undoing_prior_exchange() {
    let (mut sim, mut driver) = setup(Backend::Reference);
    while sim.state.month < 3 || sim.state.phase != Phase::Acquire {
        driver.step(&mut sim).unwrap();
    }
    let paid: Vec<_> = sim
        .state
        .town_market
        .history
        .iter()
        .flat_map(|r| r.cooperation.iter())
        .flat_map(|b| b.completed.iter())
        .cloned()
        .collect();
    assert_eq!(paid.len(), 2);
    let start = cooperation::active_independent(&sim.state).unwrap().start;
    sim.state.balances.insert((89, GRAIN), 0);
    let opening = sim.state.balances.clone();
    driver.step(&mut sim).unwrap();
    let receipt = driver.history.last().unwrap();
    assert!(receipt.continuing && receipt.failure.is_some());
    assert!(sim.ledger.last().unwrap().transactions.is_empty());
    assert_eq!(sim.state.balances, opening);
    assert!(cooperation::active_independent(&sim.state).is_none());
    while sim.state.month < 7 {
        driver.step(&mut sim).unwrap();
    }
    let actual: Vec<_> = sim
        .state
        .town_market
        .history
        .iter()
        .flat_map(|r| r.cooperation.iter())
        .filter(|b| b.agreement == Some(start))
        .flat_map(|b| b.completed.iter())
        .cloned()
        .collect();
    assert_eq!(actual, paid);
}
