#[path = "support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    allocation::Policy,
    competition::{self, Application},
    composition::{self, Budget, Scope, Strategy},
    compute::Backend,
    forecast::ForecastContext,
    model::*,
    offers::{Id, Request},
    scenario::*,
    simulation::Simulation,
};
fn budget() -> Budget {
    Budget {
        expansions: 1024,
        forecasts: 128,
        months: 12,
    }
}
#[test]
fn dated_descriptions_are_read_only_and_match_authoritative_terms() {
    let (mut sim, _) = fixtures::fixture("B1", Backend::Reference).unwrap();
    fixtures::acquire(&mut sim).unwrap();
    let before = sim.clone();
    let context = ForecastContext::new(&sim.world, &sim.state);
    let rows = composition::describe(&context, PERSON).unwrap();
    let crop = rows
        .iter()
        .find(|d| d.request.offer == Id::Process(GROW))
        .unwrap();
    assert_eq!(
        crop.requirements,
        economics_compute_smoke::agreements::ProductionTerms::from_definition(
            sim.world.definition(GROW)
        )
        .schedule(1)
        .unwrap()
    );
    assert_eq!(
        crop.outputs,
        vec![(6, sim.world.definition(GROW).outputs.clone())]
    );
    assert!(crop.requirements[0].1.contains(&Amount::new(SEED, 1)));
    let land = rows
        .iter()
        .find(|d| d.request.offer == Id::Land(1))
        .unwrap();
    assert_eq!(land.payments[0].first_due, 13);
    assert_eq!(land.payments[0].transfer.amount, Amount::new(GRAIN, 2));
    assert_eq!(sim.state, before.state);
    assert_eq!(sim.world, before.world);
    assert_eq!(sim.ledger, before.ledger);
}
#[test]
fn both_searches_match_the_tiny_exhaustive_reference_and_stale_acceptance_is_atomic() {
    let (mut sim, scope) = fixtures::fixture("B1", Backend::Reference).unwrap();
    fixtures::acquire(&mut sim).unwrap();
    let mut descriptions =
        composition::describe(&ForecastContext::new(&sim.world, &sim.state), PERSON).unwrap();
    descriptions.sort_by_key(|d| d.request.offer);
    let mut oracle = None;
    for mask in 0..1 << descriptions.len() {
        let requests: Vec<_> = descriptions
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, d)| d.request.clone())
            .collect();
        if let Ok((_, score, _)) = composition::forecast_package(&sim, &requests, 12)
            && oracle.as_ref().is_none_or(|b| score < *b)
        {
            oracle = Some(score);
        }
    }
    for strategy in [Strategy::Beam, Strategy::BestFirst] {
        let selection = composition::choose(&sim, &scope, strategy, budget()).unwrap();
        assert_eq!(Some(selection.score.clone()), oracle);
        assert!(selection.metrics.expansions <= budget().expansions);
        assert!(selection.metrics.forecasts <= budget().forecasts);
        let mut hidden = sim.clone();
        hidden.world.capacity_overrides.insert((2, PERSON), 0);
        let unobserved = composition::choose(&hidden, &scope, strategy, budget()).unwrap();
        assert_eq!(selection.requests, unobserved.requests);
        assert_eq!(selection.score, unobserved.score);
        assert!(
            selection
                .requests
                .iter()
                .any(|r| r.offer == Id::Membership(1))
        );
        assert!(selection.requests.iter().any(|r| r.offer == Id::Land(1)));
        let mut stale = sim.clone();
        stale.state.balances.insert((PERSON, SEED), 0);
        let before = stale.state.clone();
        assert!(selection.accept(&mut stale).unwrap_err().contains("stale"));
        assert_eq!(stale.state, before);
        let mut changed = sim.clone();
        changed.world.definitions[0].outputs[0].quantity += 1;
        assert!(selection.accept(&mut changed).is_err());
        let mut live = sim.clone();
        selection.accept(&mut live).unwrap();
        live.run_months(6).unwrap();
        assert!(
            live.state
                .processes
                .values()
                .any(|p| p.definition == GROW && p.status == Status::Completed)
        );
    }
}
#[test]
fn budgets_reordering_and_scarcity_keep_lawful_fallbacks() {
    for case in ["B1", "B2-no-land", "B2-no-seed", "B2-scarcity"] {
        let (mut sim, scope) = fixtures::fixture(case, Backend::Reference).unwrap();
        fixtures::acquire(&mut sim).unwrap();
        let mut reverse = sim.clone();
        reverse.world.definitions.reverse();
        reverse.world.access_offers.reverse();
        for strategy in [Strategy::Beam, Strategy::BestFirst] {
            let b = Budget {
                expansions: 0,
                forecasts: 1,
                months: 6,
            };
            let tiny = composition::choose(&sim, &scope, strategy, b).unwrap();
            assert!(tiny.requests.is_empty());
            assert_eq!(tiny.metrics.forecasts, 1);
            assert_eq!(tiny.metrics.expansions, 0);
            assert!(tiny.metrics.exhausted);
            let selected = composition::choose(&sim, &scope, strategy, budget()).unwrap();
            let reordered = composition::choose(&reverse, &scope, strategy, budget()).unwrap();
            assert_eq!(selected.requests, reordered.requests);
            assert_eq!(selected.score, reordered.score);
            if case != "B1" {
                assert!(
                    !selected
                        .requests
                        .iter()
                        .any(|r| r.offer == Id::Process(GROW))
                );
            }
        }
    }
}
#[test]
fn critical_month_conflict_is_seen_by_rollout_not_total_labor() {
    let (mut sim, scope) = fixtures::fixture("B3", Backend::Reference).unwrap();
    fixtures::acquire(&mut sim).unwrap();
    let requests = vec![
        Request::new(Id::Membership(1), PERSON),
        Request::new(Id::Land(1), PERSON),
        Request::new(Id::Land(2), PERSON),
        Request::new(Id::Process(GROW), PERSON),
        Request::new(Id::Process(99), PERSON),
    ];
    let (_, bad, branch) = composition::forecast_package(&sim, &requests, 12).unwrap();
    assert!(
        branch
            .state
            .processes
            .values()
            .any(|p| p.status == Status::Aborted)
    );
    for strategy in [Strategy::Beam, Strategy::BestFirst] {
        let selected = composition::choose(&sim, &scope, strategy, budget()).unwrap();
        assert!(selected.score < bad);
        assert!(
            !(selected
                .requests
                .iter()
                .any(|r| r.offer == Id::Process(GROW))
                && selected.requests.iter().any(|r| r.offer == Id::Process(99)))
        );
    }
}
#[test]
fn household_discovery_uses_envelope_and_separate_accounting_on_cpu_and_reference() {
    fn advance(
        sim: &mut Simulation,
        audit: &mut economics_compute_smoke::financial_reporting::Audit,
        scope: &Scope,
        end: u32,
    ) {
        while sim.state.month < end {
            if sim.state.phase == Phase::Acquire {
                let choice = composition::choose(
                    sim,
                    scope,
                    Strategy::BestFirst,
                    Budget {
                        expansions: 256,
                        forecasts: 32,
                        months: 12,
                    },
                )
                .unwrap();
                let before = sim.state.clone();
                choice.accept(sim).unwrap();
                audit
                    .record(&sim.world, &before, &choice.batch, &sim.state)
                    .unwrap();
            } else {
                audit.step(sim).unwrap();
            }
        }
    }
    let mut outcomes = Vec::new();
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let (mut sim, scope) = fixtures::fixture("B5", backend).unwrap();
        let mut audit = fixtures::audit(&sim).unwrap();
        while sim.state.phase != Phase::Acquire {
            audit.step(&mut sim).unwrap();
        }
        let choice = composition::choose(
            &sim,
            &scope,
            Strategy::BestFirst,
            Budget {
                expansions: 256,
                forecasts: 32,
                months: 12,
            },
        )
        .unwrap();
        assert!(
            choice
                .requests
                .iter()
                .any(|r| matches!(r.offer, Id::Land(_)))
        );
        let before = sim.state.clone();
        choice.accept(&mut sim).unwrap();
        audit
            .record(&sim.world, &before, &choice.batch, &sim.state)
            .unwrap();
        let mut resumed = sim.clone();
        let mut resumed_audit = audit.clone();
        advance(&mut sim, &mut audit, &scope, 25);
        for end in 2..=25 {
            advance(&mut resumed, &mut resumed_audit, &scope, end);
        }
        assert_eq!(audit, resumed_audit);
        assert_eq!(sim.state, resumed.state);
        assert_eq!(sim.ledger, resumed.ledger);
        audit.finalize_through(24).unwrap();
        let statements: Vec<_> = [
            PERSON,
            competition::SECOND_PERSON,
            fixtures::HOME,
            STATE_AGENT,
        ]
        .into_iter()
        .map(|a| audit.book().finalized_statements(a, 1, 24).unwrap())
        .collect();
        assert!(
            sim.state
                .processes
                .values()
                .any(|p| p.definition == GROW && p.status == Status::Completed)
        );
        outcomes.push((sim.state, sim.reports, audit.book().clone(), statements));
    }
    assert_eq!(outcomes[0], outcomes[1]);
}
#[test]
fn household_consent_and_seed_are_not_inferred_from_membership() {
    let (mut sim, _) = fixtures::fixture("B5-one-seed", Backend::Reference).unwrap();
    fixtures::acquire(&mut sim).unwrap();
    let scope = Scope::Household {
        agent: fixtures::HOME,
        consenting_members: [PERSON].into(),
    };
    let choice = composition::choose(&sim, &scope, Strategy::BestFirst, budget()).unwrap();
    assert!(choice.requests.iter().all(|r| r.agent == PERSON));
    assert!(
        choice
            .requests
            .iter()
            .filter(|r| r.offer == Id::Process(GROW))
            .count()
            <= 1
    );
    let mut outsiders = sim.clone();
    outsiders.world.participants.push(Participant {
        agent: STATE_AGENT,
        capacity: Amount::new(LABOR, 5),
        needs: vec![],
    });
    assert!(
        composition::choose(&outsiders, &scope, Strategy::BestFirst, budget())
            .unwrap_err()
            .contains("isolated household")
    );
    let invalid = Scope::Household {
        agent: fixtures::HOME,
        consenting_members: [STATE_AGENT].into(),
    };
    assert!(composition::choose(&sim, &invalid, Strategy::BestFirst, budget()).is_err());
    assert!(
        composition::choose(&sim, &Scope::Person(PERSON), Strategy::BestFirst, budget()).is_err()
    );
}
#[test]
fn independent_plans_do_not_allocate_the_contested_plot() {
    let (w, s) = competition::scenario(1, 7).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    fixtures::acquire(&mut sim).unwrap();
    let mut applications = Vec::new();
    for agent in [PERSON, competition::SECOND_PERSON] {
        let mut local = sim.clone();
        local.world.competition = None;
        local.world.participants.retain(|p| p.agent == agent);
        local.world.condition_rules.retain(|r| r.subject == agent);
        local.state.conditions.retain(|(a, _), _| *a == agent);
        let choice =
            composition::choose(&local, &Scope::Person(agent), Strategy::BestFirst, budget())
                .unwrap();
        assert!(choice.requests.iter().all(|r| r.agent == agent));
        assert!(choice.requests.iter().any(|r| r.offer == Id::Land(1)));
        applications.push(Application {
            agent,
            requests: choice.requests,
        });
    }
    assert!(sim.state.accepted_agreements.is_empty());
    for policy in [Policy::StablePriority, Policy::PriorityLottery] {
        let mut winners = std::collections::BTreeMap::new();
        for seed in 0..20 {
            let batch = competition::prepare(&sim, 1, seed, policy, &applications).unwrap();
            let mut reverse = applications.clone();
            reverse.reverse();
            assert_eq!(
                batch,
                competition::prepare(&sim, 1, seed, policy, &reverse).unwrap()
            );
            let mut live = sim.clone();
            competition::accept(&mut live, 1, seed, policy, &applications).unwrap();
            assert_eq!(live.state.accepted_agreements.len(), 1);
            live.step().unwrap();
            let winner = live.state.accepted_agreements[&1].debtor;
            *winners.entry((seed >= 10, winner)).or_insert(0) += 1;
            assert!(
                live.state
                    .processes
                    .values()
                    .filter(|p| p.definition == GROW)
                    .all(|p| p.operator == winner)
            );
            assert!(live.state.balances.values().all(|q| *q >= 0));
            let loser = if winner == PERSON {
                competition::SECOND_PERSON
            } else {
                PERSON
            };
            assert_eq!(
                live.state.balance(loser, SEED),
                sim.state.balance(loser, SEED)
            );
            assert!(
                live.state
                    .processes
                    .values()
                    .any(|p| p.operator == loser && p.definition == PREPARE_FUEL)
            );
        }
        eprintln!("B6 {policy:?} winners by (held_out,person): {winners:?}");
    }
}
#[test]
fn unsupported_market_driver_is_explicit() {
    let (w, s) = economics_compute_smoke::production_market::reciprocal_scenario(true);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    fixtures::acquire(&mut sim).unwrap();
    let before = sim.clone();
    let markets =
        composition::describe_markets(&ForecastContext::new(&sim.world, &sim.state), PERSON);
    assert_eq!(markets.len(), 2);
    assert!(
        markets
            .iter()
            .all(|m| m.admitted && m.registered && m.last_completed.is_none())
    );
    assert_eq!(sim.state, before.state);
    assert_eq!(sim.world, before.world);
    assert!(
        composition::choose(&sim, &Scope::Person(PERSON), Strategy::Beam, budget())
            .unwrap_err()
            .contains("does not support")
    );
}
