use economics_compute_smoke::{
    compute::Backend,
    credit::{Advance, LoanOffer},
    model::*,
    offers::{self, Id, Request},
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    scenario::*,
    settlement,
    simulation::Simulation,
};

fn fixture(funded: bool) -> (World, State) {
    let (mut w, mut s) = named("opportunity-farming").unwrap();
    w.lending.push(Advance {
        id: 10,
        debtor: PERSON,
        principal: 1,
        month: 1,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: SEED,
            max_principal: 1,
            monthly_rate_bps: 0,
            term_months: 6,
            grace_months: 12,
        },
    });
    let policy = w.transaction_policy.as_mut().unwrap();
    policy.permissions.insert((PERSON_TYPE, Action::Borrow));
    policy.permissions.insert((STATE_TYPE, Action::Lend));
    s.balances.insert((PERSON, SEED), 0);
    s.balances.insert((PERSON, GRAIN), 5);
    s.balances.insert((STATE_AGENT, SEED), i32::from(funded));
    (w, s)
}

#[test]
fn accepted_seed_credit_and_land_search_share_one_dated_production_plan() {
    for funded in [false, true] {
        let (w, s) = fixture(funded);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.phase != Phase::Acquire {
                sim.step().unwrap();
            }
            let mut resumed = sim.clone();
            sim.step().unwrap();
            assert_eq!(sim.state.credit.loans.contains_key(&10), funded);
            let batch = sim.ledger.last().unwrap();
            assert!(batch.credit.is_some());
            assert!(batch.decision.is_some());
            assert!(batch.production_plan.is_some());
            assert_eq!(
                batch.transactions,
                batch.credit.as_ref().unwrap().transactions
            );
            if funded {
                assert_eq!(sim.state.accepted_agreements.len(), 1);
                assert_eq!(sim.state.memberships.len(), 1);
            }
            let plan = sim.state.pending_production.clone().unwrap();
            sim.step().unwrap();
            assert_eq!(sim.ledger.last().unwrap(), plan.as_ref());
            while sim.state.month < 5 {
                sim.step().unwrap();
            }
            while resumed.state.month < 5 {
                resumed.step().unwrap();
            }
            assert_eq!(
                sim.state.processes.values().any(|p| p.definition == GROW),
                funded
            );
            assert_eq!((&sim.state, &sim.ledger), (&resumed.state, &resumed.ledger));
            (sim.state, sim.ledger)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn explicit_crop_bundle_uses_delivered_credit_and_rejects_altered_boundaries() {
    for funded in [false, true] {
        let (w, s) = fixture(funded);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        while sim.state.phase != Phase::Acquire {
            sim.step().unwrap();
        }
        let opening = sim.state.clone();
        let requests = [
            Request::new(Id::Membership(1), PERSON),
            Request::new(Id::Land(1), PERSON),
            Request::new(Id::Process(GROW), PERSON),
        ];
        let prepared = offers::prepare(&sim, &requests);
        assert_eq!(sim.state, opening);
        if !funded {
            assert!(prepared.is_err());
            assert!(offers::accept(&mut sim, &requests).is_err());
            assert_eq!(sim.state, opening);
            continue;
        }
        let prepared = prepared.unwrap();
        for alteration in 0..6 {
            let mut bad = prepared.clone();
            match alteration {
                0 => bad.production_plan.as_mut().unwrap().month += 1,
                1 => bad.production_plan.as_mut().unwrap().id += 1,
                2 => bad.production_plan.as_mut().unwrap().phase = Phase::Close,
                3 => {
                    bad.production_plan.as_mut().unwrap().production_plan =
                        prepared.production_plan.clone()
                }
                4 => bad.credit.as_mut().unwrap().after.loans.clear(),
                _ => bad.transactions.clear(),
            }
            let mut state = opening.clone();
            assert!(
                settlement::commit(&sim.world, &mut state, &bad, sim.backend, sim.effect_limit,)
                    .is_err()
            );
            assert_eq!(state, opening);
        }
        offers::accept(&mut sim, &requests).unwrap();
        assert_eq!(sim.state.balance(PERSON, SEED), 1);
        assert!(sim.state.processes.is_empty());
        sim.step().unwrap();
        assert_eq!(sim.state.balance(PERSON, SEED), 0);
        assert!(sim.state.processes.values().any(|p| p.definition == GROW));
        let after = sim.state.clone();
        assert!(
            settlement::commit(
                &sim.world,
                &mut sim.state,
                &prepared,
                sim.backend,
                sim.effect_limit,
            )
            .is_err()
        );
        assert_eq!(sim.state, after);
    }
}

#[test]
fn competing_applicants_share_funded_advances_without_canceling_losers_debt() {
    use economics_compute_smoke::{
        allocation::Policy,
        competition::{self, Application, SECOND_PERSON},
    };
    for seeds in [0, 1, 2] {
        for policy in [Policy::StablePriority, Policy::PriorityLottery] {
            let (mut w, mut s) = competition::scenario(1, 7).unwrap();
            for (id, agent) in [(10, PERSON), (11, SECOND_PERSON)] {
                let mut advance = fixture(true).0.lending.remove(0);
                advance.id = id;
                advance.debtor = agent;
                w.lending.push(advance);
                s.balances.insert((agent, SEED), 0);
            }
            s.balances.insert((STATE_AGENT, SEED), seeds);
            let permissions = &mut w.transaction_policy.as_mut().unwrap().permissions;
            permissions.insert((PERSON_TYPE, Action::Borrow));
            permissions.insert((STATE_TYPE, Action::Lend));
            let run = |backend| {
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while sim.state.phase != Phase::Acquire {
                    sim.step().unwrap();
                }
                let requests: Vec<_> = [PERSON, SECOND_PERSON]
                    .into_iter()
                    .map(|agent| Application {
                        agent,
                        requests: vec![
                            Request::new(Id::Membership(1), agent),
                            Request::new(Id::Land(1), agent),
                            Request::new(Id::Process(GROW), agent),
                        ],
                    })
                    .collect();
                let batch = competition::prepare(&sim, 1, 7, policy, &requests).unwrap();
                let mut reversed = sim.clone();
                reversed.world.lending.reverse();
                assert_eq!(
                    batch,
                    competition::prepare(
                        &reversed,
                        1,
                        7,
                        policy,
                        &requests.iter().cloned().rev().collect::<Vec<_>>()
                    )
                    .unwrap()
                );
                let checkpoint = sim.clone();
                competition::accept(&mut sim, 1, 7, policy, &requests).unwrap();
                assert_eq!(sim.state.credit.loans.len(), seeds as usize);
                assert_eq!(sim.state.accepted_agreements.len(), usize::from(seeds > 0));
                let mut resumed = checkpoint;
                competition::accept(&mut resumed, 1, 7, policy, &requests).unwrap();
                sim.step().unwrap();
                resumed.step().unwrap();
                assert_eq!(
                    sim.state
                        .processes
                        .values()
                        .filter(|p| p.definition == GROW)
                        .count(),
                    usize::from(seeds > 0)
                );
                if seeds > 0 {
                    let winner = sim.state.accepted_agreements[&1].debtor;
                    assert_eq!(sim.state.balance(winner, SEED), 0);
                    if seeds == 1 {
                        assert_eq!(winner, PERSON);
                    } else {
                        let loser = if winner == PERSON {
                            SECOND_PERSON
                        } else {
                            PERSON
                        };
                        assert_eq!(sim.state.balance(loser, SEED), 1);
                        assert!(sim.state.credit.loans.values().any(|l| l.debtor == loser));
                    }
                }
                assert_eq!((&sim.state, &sim.ledger), (&resumed.state, &resumed.ledger));
                (sim.state, sim.ledger)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

#[test]
fn autonomous_competition_keeps_credit_on_later_uncontested_boundaries() {
    use economics_compute_smoke::competition::{self, SECOND_PERSON};
    let (mut w, mut s) = competition::scenario(1, 7).unwrap();
    for (id, agent) in [(10, PERSON), (11, SECOND_PERSON)] {
        let mut advance = fixture(true).0.lending.remove(0);
        advance.id = id;
        advance.debtor = agent;
        w.lending.push(advance);
        s.balances.insert((agent, SEED), 0);
    }
    s.balances.insert((STATE_AGENT, SEED), 2);
    let permissions = &mut w.transaction_policy.as_mut().unwrap().permissions;
    permissions.insert((PERSON_TYPE, Action::Borrow));
    permissions.insert((STATE_TYPE, Action::Lend));
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while sim.state.phase != Phase::Acquire {
            sim.step().unwrap();
        }
        let mut resumed = sim.clone();
        sim.step().unwrap();
        assert_eq!(sim.state.accepted_agreements.len(), 1);
        assert_eq!(sim.state.credit.loans.len(), 2);
        assert!(sim.ledger.last().unwrap().allocation.is_some());
        while sim.state.month < 3 {
            sim.step().unwrap();
        }
        while resumed.state.month < 3 {
            resumed.step().unwrap();
        }
        assert_eq!((&sim.state, &sim.ledger), (&resumed.state, &resumed.ledger));
        (sim.state, sim.ledger)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn prepaid_harvest_and_seed_credit_remain_distinct_through_search_and_delivery() {
    use economics_compute_smoke::{
        dues_accounting::Valuation,
        financial_reporting::{Audit, Opening},
        forward::direct,
        process_accounting::{Costs, Output},
    };
    for (credit, funded, term) in [
        (false, true, 6),
        (true, true, 24),
        (true, true, 6),
        (true, false, 6),
    ] {
        let (mut w, mut s) = fixture(true);
        w.lending[0].terms.term_months = term;
        if !credit {
            w.lending.clear();
            s.balances.insert((PERSON, SEED), 1);
        }
        w.resources.push(Resource {
            id: TOKEN,
            name: "coin".into(),
            kind: ResourceKind::Stock,
        });
        s.balances
            .insert((STATE_AGENT, TOKEN), if funded { 2 } else { 0 });
        w.prepaid_deliveries.push(direct::Terms {
            id: 20,
            seller: PERSON,
            buyer: STATE_AGENT,
            month: 1,
            due: 9,
            goods: Amount::new(GRAIN, 2),
            prepayment: Amount::new(TOKEN, 2),
        });
        let permissions = &mut w.transaction_policy.as_mut().unwrap().permissions;
        permissions.insert((PERSON_TYPE, Action::StockTrade));
        permissions.insert((STATE_TYPE, Action::StockTrade));
        let run = |backend| {
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
                    inventory: s
                        .balances
                        .iter()
                        .filter(|((_, r), q)| {
                            **q > 0
                                && *r != TOKEN
                                && w.resources
                                    .iter()
                                    .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
                        })
                        .map(|(a, q)| (*a, i128::from(*q)))
                        .collect(),
                    exchange_values: [(GRAIN, 1), (SEED, 1), (RAW_WOOD, 1), (FUEL, 1)].into(),
                    processes: Some(Costs {
                        output_weights: [(
                            GROW,
                            [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                        )]
                        .into(),
                        ..Costs::default()
                    }),
                    dues: Some(Valuation([(1, 1)].into())),
                    ..Opening::default()
                },
            )
            .unwrap();
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.phase != Phase::Acquire {
                audit.step(&mut sim).unwrap();
            }
            let mut resumed = sim.clone();
            let mut ra = audit.clone();
            audit.step(&mut sim).unwrap();
            if credit && funded && term == 6 {
                let d = sim.ledger.last().unwrap().decision.as_ref().unwrap();
                assert!(
                    d.rejection_reasons
                        .iter()
                        .any(|r| r.contains("land payment is forecast to remain unpaid"))
                );
            }
            assert_eq!(sim.state.exchange.forwards.contains_key(&20), funded);
            assert_eq!(sim.state.credit.loans.contains_key(&10), credit);
            assert!(sim.ledger.last().unwrap().decision.is_some());
            assert_eq!(sim.state.balance(PERSON, TOKEN), if funded { 2 } else { 0 });
            while sim.state.month < 10 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month < 10 {
                ra.step(&mut resumed).unwrap();
            }
            if funded {
                let delivered = if credit && term == 6 { 0 } else { 2 };
                assert_eq!(sim.state.exchange.forwards[&20].delivered, delivered);
                assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), delivered);
                assert_eq!(
                    sim.state.exchange.forwards[&20].claim().outstanding(),
                    2 - delivered
                );
            }
            assert_eq!((&sim.state, &sim.ledger), (&resumed.state, &resumed.ledger));
            assert_eq!(audit, ra);
            for agent in &w.agents {
                let report = audit.book().statements(agent.id, 1, 9).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn forward_acceptance_does_not_depend_on_winning_land_and_receipts_remain_exact() {
    use economics_compute_smoke::{
        allocation::Policy,
        competition::{self, Application, SECOND_PERSON},
        forward::{Event, direct},
    };
    let (mut w, mut s) = competition::scenario(1, 7).unwrap();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    s.balances.insert((STATE_AGENT, TOKEN), 2);
    w.prepaid_deliveries.push(direct::Terms {
        id: 20,
        seller: SECOND_PERSON,
        buyer: STATE_AGENT,
        month: 1,
        due: 3,
        goods: Amount::new(GRAIN, 2),
        prepayment: Amount::new(TOKEN, 2),
    });
    let permissions = &mut w.transaction_policy.as_mut().unwrap().permissions;
    permissions.insert((PERSON_TYPE, Action::StockTrade));
    permissions.insert((STATE_TYPE, Action::StockTrade));
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    // Supplied consent remains binding when no applicant wins, and when a
    // different person wins. Neither round can erase the prepaid delivery.
    for applications in [
        vec![],
        vec![Application {
            agent: PERSON,
            requests: vec![
                Request::new(Id::Membership(1), PERSON),
                Request::new(Id::Land(1), PERSON),
                Request::new(Id::Process(GROW), PERSON),
            ],
        }],
    ] {
        let batch =
            competition::prepare(&sim, 1, 7, Policy::StablePriority, &applications).unwrap();
        let opening = sim.state.clone();
        let mut bad = batch.clone();
        for tx in &mut bad.transactions {
            if let Some(Event::Accepted(c)) = &mut tx.forward {
                c.goods.quantity += 1;
            }
        }
        let mut state = opening.clone();
        assert!(
            settlement::commit(&sim.world, &mut state, &bad, sim.backend, sim.effect_limit)
                .is_err()
        );
        assert_eq!(state, opening);
        let mut branch = sim.clone();
        competition::accept(&mut branch, 1, 7, Policy::StablePriority, &applications).unwrap();
        assert_eq!(branch.state.exchange.forwards[&20].debtor, SECOND_PERSON);
        assert_eq!(branch.state.balance(SECOND_PERSON, TOKEN), 2);
        assert_eq!(branch.state.accepted_agreements.len(), applications.len());
        assert!(
            !branch
                .state
                .accepted_agreements
                .values()
                .any(|a| a.debtor == SECOND_PERSON)
        );
        branch.step().unwrap();
    }
}

#[test]
fn common_requests_can_name_financial_and_productive_acceptances_atomically() {
    use economics_compute_smoke::forward::direct;
    for funded in [false, true] {
        let (mut w, mut s) = fixture(true);
        w.resources.push(Resource {
            id: TOKEN,
            name: "coin".into(),
            kind: ResourceKind::Stock,
        });
        s.balances
            .insert((STATE_AGENT, TOKEN), if funded { 2 } else { 0 });
        w.prepaid_deliveries.push(direct::Terms {
            id: 20,
            seller: PERSON,
            buyer: STATE_AGENT,
            month: 1,
            due: 9,
            goods: Amount::new(GRAIN, 2),
            prepayment: Amount::new(TOKEN, 2),
        });
        let permissions = &mut w.transaction_policy.as_mut().unwrap().permissions;
        permissions.insert((PERSON_TYPE, Action::StockTrade));
        permissions.insert((STATE_TYPE, Action::StockTrade));
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        while sim.state.phase != Phase::Acquire {
            sim.step().unwrap();
        }
        let requests = vec![
            Request::new(Id::Advance(10), PERSON),
            Request::new(Id::PrepaidDelivery(20), PERSON),
            Request::new(Id::Membership(1), PERSON),
            Request::new(Id::Land(1), PERSON),
            Request::new(Id::Process(GROW), PERSON),
        ];
        let opening = sim.state.clone();
        let ledger = sim.ledger.clone();
        let batch = offers::prepare(&sim, &requests);
        assert_eq!(sim.state, opening);
        if !funded {
            assert!(batch.is_err());
            assert!(offers::accept(&mut sim, &requests).is_err());
            assert_eq!((&sim.state, &sim.ledger), (&opening, &ledger));
            continue;
        }
        let batch = batch.unwrap();
        let mut reordered = requests.clone();
        reordered.swap(0, 1);
        assert_eq!(batch, offers::prepare(&sim, &reordered).unwrap());
        for change in 0..4 {
            let mut invalid = requests.clone();
            match change {
                0 => invalid.push(invalid[0].clone()),
                1 => invalid[0].agent = STATE_AGENT,
                2 => invalid[0].need = Some(NUTRITION),
                _ => invalid.swap(2, 3),
            }
            assert!(offers::accept(&mut sim, &invalid).is_err());
            assert_eq!((&sim.state, &sim.ledger), (&opening, &ledger));
        }
        offers::accept(&mut sim, &requests).unwrap();
        assert_eq!(sim.state.credit.loans.len(), 1);
        assert_eq!(sim.state.exchange.forwards.len(), 1);
        assert_eq!(sim.state.accepted_agreements.len(), 1);
        assert_eq!(sim.state.balance(PERSON, SEED), 1);
        sim.step().unwrap();
        assert_eq!(sim.state.balance(PERSON, SEED), 0);
        assert!(sim.state.processes.values().any(|p| p.definition == GROW));
    }
}
