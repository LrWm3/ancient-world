use economics_compute_smoke::{
    acquisition,
    compute::Backend,
    credit,
    model::*,
    need_orders,
    negotiation::{self, Outcome, QuotePolicy},
    opportunities::{Action, PERSON_TYPE},
    scenario::{CONSUME, GRAIN, NUTRITION, PERSON, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    zip,
};

fn opening(w: World, s: State, backend: Backend) -> Simulation {
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim
}
fn preview(sim: &Simulation) -> Batch {
    acquisition::evaluate(&sim.world, &sim.state).unwrap()
}
fn outcome(sim: &Simulation) -> Outcome {
    preview(sim).negotiation.unwrap().outcome
}

#[test]
fn generated_orders_meet_needs_then_stop_at_own_stock_and_seller_reserves() {
    let (w, s) = need_orders::scenario();
    let mut generated = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut supplied = w;
    supplied.need_orders = None;
    let mut supplied = Simulation::new(supplied, s, Backend::Reference).unwrap();
    generated.run_months(6).unwrap();
    supplied.run_months(6).unwrap();
    let rounds: Vec<_> = generated
        .ledger
        .iter()
        .filter_map(|b| b.negotiation.as_ref())
        .collect();
    assert_eq!(
        rounds.iter().map(|r| r.outcome.clone()).collect::<Vec<_>>(),
        vec![
            Outcome::Traded { price: 40 },
            Outcome::NoDemand,
            Outcome::Traded { price: 40 },
            Outcome::NoDemand,
            Outcome::NoSurplus,
            Outcome::NoSurplus,
        ]
    );
    assert!(
        rounds
            .iter()
            .all(|r| r.orders.as_ref().unwrap().month == r.month)
    );
    let deficits = |sim: &Simulation, agent| {
        sim.reports
            .iter()
            .filter(|r| r.agent == agent)
            .map(|r| r.deficit(NUTRITION))
            .sum::<i32>()
    };
    assert_eq!(deficits(&generated, PERSON), 2);
    assert_eq!(deficits(&supplied, PERSON), 4);
    assert_eq!(deficits(&generated, 89), 0);
    assert_eq!(generated.state.balance(PERSON, TOKEN), 120);
    assert_eq!(generated.state.balance(89, TOKEN), 80);
    assert_eq!(
        generated.state.balance(PERSON, GRAIN) + generated.state.balance(89, GRAIN),
        0
    );
}

#[test]
fn orders_are_intents_and_still_need_cash_storage_and_trade_permission() {
    for expected in [
        Outcome::InsufficientPayment,
        Outcome::InsufficientStorage,
        Outcome::Ineligible,
    ] {
        let (mut w, mut s) = need_orders::scenario();
        match expected {
            Outcome::InsufficientPayment => {
                s.balances.insert((PERSON, TOKEN), 0);
            }
            Outcome::InsufficientStorage => {
                w.storage.capacities.insert(PERSON, 1);
            }
            _ => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(PERSON_TYPE, Action::StockTrade));
            }
        }
        let mut sim = opening(w, s, Backend::CubeCpu);
        let before = sim.state.balances.clone();
        sim.step().unwrap();
        let round = sim.ledger.last().unwrap().negotiation.as_ref().unwrap();
        assert_eq!(round.outcome, expected);
        if expected != Outcome::Ineligible {
            assert!(round.orders.as_ref().unwrap().buy.is_some());
        }
        assert_eq!(sim.state.balances, before);
    }
}

#[test]
fn recipes_permissions_and_substitutes_determine_demand_without_food_ids() {
    for case in 0..4 {
        let (mut w, mut s) = need_orders::scenario();
        match case {
            0 => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(PERSON_TYPE, Action::Process(CONSUME)));
            }
            1 => {
                w.definitions[0].enabled = false;
            }
            2 => {
                w.resources.push(Resource {
                    id: 99,
                    name: "alternate food".into(),
                    kind: ResourceKind::Stock,
                });
                let mut recipe = w.definitions[0].clone();
                recipe.id = 99;
                recipe.stages[0].entry_inputs[0].resource = 99;
                w.definitions.push(recipe);
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .insert((PERSON_TYPE, Action::Process(99)));
                s.balances.insert((PERSON, 99), 1);
            }
            _ => {
                for r in &mut w.resources {
                    if r.id == NUTRITION {
                        r.id = 100;
                        r.name = "warmth".into();
                    }
                }
                w.definitions[0].outputs[0].resource = 100;
                for p in &mut w.participants {
                    p.needs[0].resource = 100;
                }
            }
        }
        let sim = opening(w, s, Backend::Reference);
        assert_eq!(
            outcome(&sim),
            if case == 3 {
                Outcome::Traded { price: 40 }
            } else {
                Outcome::NoDemand
            }
        );
    }
}

#[test]
fn seller_protection_counts_shared_inputs_once_and_retains_partial_recipe_stocks() {
    let (mut w, mut s) = need_orders::scenario();
    // Two distinct needs use the same grain. Four units over two months are protected.
    w.resources.push(Resource {
        id: 100,
        name: "warmth".into(),
        kind: ResourceKind::Fulfillment,
    });
    let mut warmth = w.definitions[0].clone();
    warmth.id = 100;
    warmth.outputs[0].resource = 100;
    w.definitions.push(warmth);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::Process(100)));
    w.participants
        .iter_mut()
        .find(|p| p.agent == 89)
        .unwrap()
        .needs
        .push(Requirement {
            resource: 100,
            quantity: 1,
            priority: 1,
        });
    s.balances.insert((89, GRAIN), 5);
    let sim = opening(w, s, Backend::Reference);
    let r = preview(&sim).negotiation.unwrap();
    assert_eq!(r.orders.unwrap().protected[&(89, GRAIN)], 4);
    assert_eq!(r.outcome, Outcome::NoSurplus);

    let (mut w, mut s) = need_orders::scenario();
    w.definitions[0].stages[0].entry_inputs[0].quantity = 3;
    s.balances.insert((PERSON, GRAIN), 1);
    s.balances.insert((89, GRAIN), 2);
    let sim = opening(w, s, Backend::Reference);
    let r = preview(&sim).negotiation.unwrap();
    assert!(r.orders.as_ref().unwrap().buy.is_some()); // one held + two requested completes recipe
    assert_eq!(r.orders.unwrap().protected[&(89, GRAIN)], 2); // preserve partial stock
    assert_eq!(r.outcome, Outcome::NoSurplus);
}

#[test]
fn active_process_inputs_are_protected_until_the_stage_has_consumed_them() {
    let (mut w, mut s) = need_orders::scenario();
    let d = ProcessDefinition {
        id: 101,
        name: "stock commitment".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "work".into(),
            months: 2,
            entry_inputs: vec![Amount::new(GRAIN, 3)],
            monthly_services: vec![],
        }],
        outputs: vec![Amount::new(GRAIN, 1)],
    };
    w.definitions.push(d);
    s.balances.insert((89, GRAIN), 4);
    s.processes.insert(
        1,
        ProcessInstance {
            id: 1,
            definition: 101,
            operator: 89,
            beneficiary: 89,
            goal: None,
            asset: None,
            right: None,
            start: 1,
            reserved_through: 2,
            stage: 0,
            elapsed: 0,
            status: Status::Active,
        },
    );
    let mut sim = opening(w, s, Backend::Reference);
    assert_eq!(outcome(&sim), Outcome::NoSurplus);
    // Already paid entry inputs must not be charged again in the reserve calculation.
    sim.state.processes.get_mut(&1).unwrap().elapsed = 1;
    assert_eq!(outcome(&sim), Outcome::Traded { price: 40 });
}

#[test]
fn generated_buy_order_cannot_spend_cash_reserved_for_a_mortgage() {
    let (mut w, mut s) = need_orders::scenario();
    let (c, _) = credit::scenario("repaid").unwrap();
    w.credit = c.credit;
    w.assets = c.assets;
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::FinancedPurchase));
    s.balances.insert((PERSON, TOKEN), 0);
    let mut sim = opening(w, s, Backend::CubeCpu);
    sim.step().unwrap();
    let b = sim.ledger.last().unwrap();
    assert!(
        b.negotiation
            .as_ref()
            .unwrap()
            .orders
            .as_ref()
            .unwrap()
            .buy
            .is_some()
    );
    assert_eq!(
        b.negotiation.as_ref().unwrap().outcome,
        Outcome::InsufficientPayment
    );
    assert_eq!(sim.state.credit.loans.len(), 1);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
}

#[test]
fn tampered_orders_protection_and_changed_needs_reject_without_publication() {
    let (w, s) = need_orders::scenario();
    let sim = opening(w, s, Backend::Reference);
    let valid = preview(&sim);
    for case in 0..4 {
        let mut w = sim.world.clone();
        let mut b = valid.clone();
        let r = b.negotiation.as_mut().unwrap();
        match case {
            0 => r.orders = None,
            1 => {
                r.orders.as_mut().unwrap().protected.clear();
            }
            2 => {
                r.orders
                    .as_mut()
                    .unwrap()
                    .buy
                    .as_mut()
                    .unwrap()
                    .goods
                    .quantity += 1;
            }
            _ => {
                w.participants[0].needs[0].quantity = 0;
            }
        }
        let mut state = sim.state.clone();
        assert!(
            settlement::commit(&w, &mut state, &b, Backend::CubeCpu, DEFAULT_EFFECT_LIMIT).is_err()
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
    let after = state.clone();
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
    assert_eq!(state, after);
}

#[test]
fn zip_cpu_reference_monthly_batch_resume_and_catalog_order_agree() {
    let (mut w, s) = need_orders::scenario();
    let n = w.negotiation.as_mut().unwrap();
    n.buyer.policy = QuotePolicy::Zip(zip::Config::default());
    n.seller.policy = QuotePolicy::Zip(zip::Config::default());
    n.max_rounds = negotiation::MAX_QUOTE_ROUNDS;
    let mut monthly = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut batched = monthly.clone();
    w.agents.reverse();
    w.resources.reverse();
    w.participants.reverse();
    w.definitions.reverse();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    monthly.step().unwrap();
    let mut resumed = monthly.clone();
    for _ in 0..6 {
        monthly.run_months(1).unwrap();
    }
    for sim in [&mut batched, &mut reference, &mut resumed] {
        sim.run_months(6).unwrap();
        assert_eq!(sim.state, monthly.state);
        assert_eq!(sim.ledger, monthly.ledger);
        assert_eq!(sim.reports, monthly.reports);
    }
    let history = &monthly.state.marketplaces[&negotiation::MARKETPLACE].history;
    assert_eq!(history.len(), 6);
    assert!(history[1].quotes.is_empty() && history[1].events.is_empty());
    assert_eq!(monthly.state.balance(PERSON, TOKEN), 128);
}

#[test]
fn consumption_uses_the_same_permitted_recipe_set_as_order_generation() {
    let (mut w, s) = need_orders::scenario();
    let mut forbidden = w.definitions[0].clone();
    forbidden.id = 99;
    forbidden.outputs[0].quantity = 2; // tempting higher yield, but not permitted
    w.definitions.push(forbidden);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert!(sim.reports.iter().all(|r| r.deficit(NUTRITION) == 0));
    assert!(sim.state.processes.values().all(|p| p.definition != 99));
}

#[test]
fn policy_validates_horizon_and_participants_and_respects_market_start_month() {
    for case in 0..4 {
        let (mut w, s) = need_orders::scenario();
        match case {
            0 => w.need_orders.as_mut().unwrap().reserve_months = 0,
            1 => w.need_orders.as_mut().unwrap().reserve_months = 25,
            2 => w.negotiation = None,
            _ => w.participants.clear(),
        }
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
    let (mut w, s) = need_orders::scenario();
    w.negotiation.as_mut().unwrap().month = 3;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(3).unwrap();
    let rounds: Vec<_> = sim
        .ledger
        .iter()
        .filter_map(|b| b.negotiation.as_ref())
        .collect();
    assert_eq!(rounds.len(), 1);
    assert_eq!(rounds[0].month, 3);
}

#[test]
fn dated_commodity_commitments_reduce_surplus_inside_the_protection_window() {
    use economics_compute_smoke::{
        commitments,
        scenario::{self, STATE_AGENT},
    };
    for months in [1, 2] {
        let (mut w, mut s) = need_orders::scenario();
        let (catalog, _) = scenario::baseline();
        w.assets = catalog.assets;
        w.rights = catalog.rights;
        w.rights[0].holder = 89;
        w.rights[0].output_owner = 89;
        w.rights[0].through = 24;
        w.agreements.push(commitments::Agreement {
            id: 1,
            right: w.rights[0].id,
            creditor: STATE_AGENT,
            debtor: 89,
            activated: 1,
            payment: Amount::new(GRAIN, 2),
        });
        w.need_orders.as_mut().unwrap().reserve_months = months;
        s.month = 12;
        s.balances.insert((89, GRAIN), 5);
        let sim = opening(w, s, Backend::Reference);
        let r = preview(&sim).negotiation.unwrap();
        assert_eq!(
            r.orders.unwrap().protected[&(89, GRAIN)],
            if months == 1 { 1 } else { 4 }
        );
        assert_eq!(
            r.outcome,
            if months == 1 {
                Outcome::Traded { price: 40 }
            } else {
                Outcome::NoSurplus
            }
        );
    }
}
