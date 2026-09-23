use economics_compute_smoke::{
    calibration,
    compute::Backend,
    cooperation::{self, Discovery},
    model::*,
    production_market::Policy,
    scenario::{NUTRITION, TOKEN, WARMTH},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    town_market,
};

fn simulation(mode: Discovery, backend: Backend, shock: bool) -> Simulation {
    let (mut w, s) = calibration::scenario(true);
    w.production_market.as_mut().unwrap().policy = Policy::Cooperate(mode);
    if shock {
        w.capacity_overrides
            .insert((3, calibration::CROP_PERSON), 0);
    }
    Simulation::new(w, s, backend).unwrap()
}

#[test]
fn both_mechanisms_discover_reciprocal_exchange_and_keep_finite_money_for_72_months() {
    for mode in [Discovery::Mutual, Discovery::Posted] {
        let mut sim = simulation(mode, Backend::CubeCpu, false);
        for _ in 0..72 {
            sim.run_months(1).unwrap();
            let r = sim.state.town_market.history.last().unwrap();
            let c = r.cooperation.as_ref().unwrap();
            assert!(c.failure.is_none());
            assert_eq!(
                sim.state.balance(calibration::CROP_PERSON, TOKEN)
                    + sim.state.balance(calibration::WOOD_PERSON, TOKEN),
                48
            );
            if c.event == "Accepted" {
                assert!(c.assessments.iter().all(|a| a.acceptable));
                assert!(c.assessments.iter().any(|a| a.proposed < a.baseline));
            }
            if c.event == "Completed" {
                assert_eq!(sim.state.balance(calibration::CROP_PERSON, TOKEN), 24);
                assert_eq!(sim.state.balance(calibration::WOOD_PERSON, TOKEN), 24);
            }
            for t in &r.transactions {
                let mut sums = std::collections::BTreeMap::new();
                for e in &t.effects {
                    *sums.entry(e.account.1).or_insert(0) += e.delta;
                }
                assert!(sums.values().all(|v| *v == 0));
            }
        }
        assert!(
            sim.reports
                .iter()
                .all(|r| r.deficit(NUTRITION) == 0 && r.deficit(WARMTH) == 0)
        );
        let history = &sim.state.town_market.history;
        assert!(
            history
                .iter()
                .filter(|r| r.month > 60)
                .any(|r| !r.transactions.is_empty())
        );
        assert!(
            history
                .iter()
                .any(|r| r.cooperation.as_ref().unwrap().event == "Declined")
        );
        if mode == Discovery::Posted {
            assert!(
                history
                    .iter()
                    .any(|r| !r.cooperation.as_ref().unwrap().offers.is_empty())
            );
            for b in history.iter().filter_map(|r| r.cooperation.as_ref()) {
                let mut proposers = std::collections::BTreeSet::new();
                assert!(b.offers.iter().all(|o| proposers.insert(o.proposer)));
            }
        }
    }
}

#[test]
fn unanticipated_lost_harvest_cancels_future_deliveries_without_reversing_previous_trades() {
    for mode in [Discovery::Mutual, Discovery::Posted] {
        let mut normal = simulation(mode, Backend::Reference, false);
        let mut shock = simulation(mode, Backend::CubeCpu, true);
        normal.run_months(1).unwrap();
        shock.run_months(1).unwrap();
        assert_eq!(
            normal.state.town_market.history,
            shock.state.town_market.history
        );
        shock.run_months(3).unwrap();
        let failed = shock.state.town_market.history.last().unwrap();
        let c = failed.cooperation.as_ref().unwrap();
        assert_eq!(failed.month, 4);
        assert_eq!(c.event, "Failed");
        assert!(c.active.is_none() && c.completed.is_empty() && failed.transactions.is_empty());
        assert!(c.failure.is_some());
        assert_eq!(shock.state.balance(calibration::CROP_PERSON, TOKEN), 22);
        assert_eq!(shock.state.balance(calibration::WOOD_PERSON, TOKEN), 26);
        assert!(
            shock
                .state
                .processes
                .values()
                .any(|p| p.status == Status::Aborted)
        );
        shock.run_months(1).unwrap();
        assert_ne!(
            shock
                .state
                .town_market
                .history
                .last()
                .unwrap()
                .cooperation
                .as_ref()
                .unwrap()
                .event,
            "Continuing"
        );
    }
}

#[test]
fn unfunded_promises_are_rejected_without_forecast_endowments_leaking() {
    for mode in [Discovery::Mutual, Discovery::Posted] {
        let mut sim = simulation(mode, Backend::Reference, false);
        for agent in [calibration::CROP_PERSON, calibration::WOOD_PERSON] {
            sim.state.balances.insert((agent, TOKEN), 0);
        }
        sim.run_months(1).unwrap();
        let r = sim.state.town_market.history.last().unwrap();
        assert_eq!(r.cooperation.as_ref().unwrap().event, "Declined");
        assert!(r.transactions.is_empty());
        assert_eq!(sim.state.balance(calibration::CROP_PERSON, TOKEN), 0);
        assert_eq!(sim.state.balance(calibration::WOOD_PERSON, TOKEN), 0);
    }
}

#[test]
fn forged_or_replayed_agreement_receipt_cannot_commit() {
    let mut sim = simulation(Discovery::Posted, Backend::CubeCpu, false);
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Acquire);
    let before = sim.state.clone();
    let round = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert_eq!(before, sim.state);
    let mut valid = Batch::empty(&sim.state);
    valid.transactions = round.transactions.clone();
    valid.town_market = Some(town_market::Boundary::Market(round));
    let mut forged = valid.clone();
    if let Some(town_market::Boundary::Market(r)) = &mut forged.town_market {
        r.cooperation
            .as_mut()
            .unwrap()
            .active
            .as_mut()
            .unwrap()
            .deliveries[0]
            .payment
            .amount
            .quantity += 1;
    }
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &forged,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, sim.state);
    settlement::commit(
        &sim.world,
        &mut sim.state,
        &valid,
        Backend::CubeCpu,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    let committed = sim.state.clone();
    let mut missing = committed.clone();
    missing.town_market.history.last_mut().unwrap().cooperation = None;
    assert!(Simulation::new(sim.world.clone(), missing, Backend::Reference).is_err());
    let mut changed = sim.world.clone();
    changed.production_market.as_mut().unwrap().policy = Policy::Plan;
    assert!(Simulation::new(changed.clone(), committed.clone(), Backend::Reference).is_err());
    changed.production_market.as_mut().unwrap().policy = Policy::Cooperate(Discovery::Mutual);
    assert!(Simulation::new(changed, committed.clone(), Backend::Reference).is_ok());
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &valid,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(committed, sim.state);
}

#[test]
fn cpu_checkpoint_and_catalog_order_match_reference_with_and_without_shock() {
    for mode in [Discovery::Mutual, Discovery::Posted] {
        for shock in [false, true] {
            let mut reference = simulation(mode, Backend::Reference, shock);
            reference.run_months(8).unwrap();
            let mut cpu = simulation(mode, Backend::CubeCpu, shock);
            cpu.world.participants.reverse();
            cpu.world.definitions.reverse();
            cpu.world.town_market.as_mut().unwrap().traders.reverse();
            cpu.world.town_market.as_mut().unwrap().additional[0]
                .traders
                .reverse();
            for _ in 0..8 {
                cpu.step().unwrap();
                cpu.step().unwrap();
                cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                    .unwrap()
                    .state;
                cpu.run_months(1).unwrap();
            }
            assert_eq!(cpu.state, reference.state);
            assert_eq!(cpu.ledger, reference.ledger);
            assert_eq!(cpu.reports, reference.reports);
        }
    }
}

#[test]
fn agreement_does_not_reuse_current_incoming_money_or_ignore_market_admission() {
    let mut sim = simulation(Discovery::Mutual, Backend::Reference, false);
    sim.run_months(1).unwrap();
    let c = sim.state.town_market.history[0]
        .cooperation
        .as_ref()
        .unwrap()
        .active
        .as_ref()
        .unwrap()
        .clone();
    sim.world.production_market.as_mut().unwrap().policy = Policy::Agreement(Box::new(c));
    // Month two: the grain seller is owed four coins, but must independently
    // fund its two-coin fuel purchase from the opening balance.
    sim.state
        .balances
        .insert((calibration::CROP_PERSON, TOKEN), 0);
    sim.step().unwrap();
    let r = town_market::evaluate(&sim.world, &sim.state).unwrap();
    assert!(r.transactions.is_empty());
    assert!(r.cooperation.unwrap().failure.is_some());
    sim.state
        .balances
        .insert((calibration::CROP_PERSON, TOKEN), 24);
    sim.state
        .town_market
        .admission
        .as_mut()
        .unwrap()
        .eligible
        .remove(&calibration::WOOD_PERSON);
    let r = cooperation::evaluate(&sim.world, &sim.state)
        .unwrap()
        .unwrap();
    assert_eq!(
        r.cooperation.unwrap().failure.as_deref(),
        Some("participant unavailable or outside marketplace")
    );
}

#[test]
fn observer_records_posted_terms_accepted_assessments_and_failed_delivery() {
    use economics_compute_smoke::telemetry::{Config, Observer, PlanningDetail};
    let mut sim = simulation(Discovery::Posted, Backend::CubeCpu, true);
    let mut observer = Observer::new(
        Vec::new(),
        "cooperation",
        Config {
            settlement: true,
            planning: PlanningDetail::Selected,
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 4).unwrap();
    let bytes = observer.finish().unwrap();
    let rows: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|r| serde_json::from_str(r).unwrap())
        .collect();
    let accepted = rows
        .iter()
        .find(|r| r["kind"] == "cooperation" && r["event"] == "Accepted")
        .unwrap();
    assert_eq!(accepted["through"], 6);
    assert_eq!(accepted["offers"].as_array().unwrap().len(), 2);
    assert_eq!(accepted["deliveries"].as_array().unwrap().len(), 9);
    assert!(
        accepted["assessments"]
            .as_array()
            .unwrap()
            .iter()
            .all(|a| a["acceptable"] == true)
    );
    let failed = rows
        .iter()
        .find(|r| r["kind"] == "cooperation" && r["event"] == "Failed")
        .unwrap();
    assert_eq!(failed["month"], 4);
    assert!(failed["failure"].is_string());
    assert!(failed["completed"].as_array().unwrap().is_empty());
    let mut plain = simulation(Discovery::Posted, Backend::CubeCpu, true);
    plain.run_months(4).unwrap();
    assert_eq!(sim.state, plain.state);
    assert_eq!(sim.ledger, plain.ledger);
}
