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
