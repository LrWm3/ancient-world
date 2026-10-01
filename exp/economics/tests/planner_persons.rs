#[path = "support/planner_fixtures.rs"]
mod fixtures;
use economics_compute_smoke::{
    allocation::{Outcome, Policy as Allocation},
    competition,
    composition::{
        Budget, Strategy,
        continuation::{Policy, Reason, persons::Persons},
    },
    compute::Backend,
    model::*,
    offers::Id,
    scenario::*,
    simulation::Simulation,
};

fn setup(
    plots: u32,
    backend: Backend,
    review: Policy,
    policy: Allocation,
    seed: u64,
) -> (Simulation, Persons) {
    let (mut w, s) = competition::scenario(plots, seed).unwrap();
    w.priority = Priority::ContinuingFirst;
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    let controllers = Persons::new(
        w.participants.iter().map(|p| p.agent),
        Strategy::BestFirst,
        Budget {
            expansions: 256,
            forecasts: 32,
            months: 12,
        },
        review,
        policy,
        seed,
    )
    .unwrap();
    (Simulation::new(w, s, backend).unwrap(), controllers)
}
fn advance(sim: &mut Simulation, p: &mut Persons, end: u32) {
    while sim.state.month < end {
        p.step(sim).unwrap();
    }
}

#[test]
fn two_people_submit_before_one_plot_is_awarded_and_loser_keeps_seed() {
    let (mut sim, mut p) = setup(
        1,
        Backend::Reference,
        Policy::RetainRepair,
        Allocation::StablePriority,
        7,
    );
    fixtures::acquire(&mut sim).unwrap();
    let before = sim.state.clone();
    p.step(&mut sim).unwrap();
    let round = p.history.last().unwrap();
    assert_eq!(round.proposals.len(), 2);
    for (&agent, requests) in &round.proposals {
        assert!(requests.iter().all(|r| r.agent == agent));
        assert!(requests.iter().any(|r| matches!(r.offer, Id::Land(1))));
    }
    assert_eq!(sim.state.accepted_agreements.len(), 1);
    assert_eq!(
        round
            .admission
            .iter()
            .filter(|r| matches!(r.outcome, Outcome::Reserved(_)))
            .count(),
        1
    );
    let loser = competition::SECOND_PERSON;
    assert!(
        round.fallback[&loser]
            .iter()
            .any(|r| r.offer == Id::Process(PREPARE_FUEL))
    );
    assert_eq!(
        sim.state.balance(PERSON, SEED),
        before.balance(PERSON, SEED),
        "Acquire reserves; Productive spends"
    );
    p.step(&mut sim).unwrap();
    assert_eq!(sim.state.balance(loser, SEED), before.balance(loser, SEED));
    assert_eq!(
        sim.state
            .processes
            .values()
            .filter(|x| x.definition == GROW)
            .count(),
        1
    );
    advance(&mut sim, &mut p, 3);
    assert_eq!(
        p.controllers[&loser].history[1].reason,
        Reason::AdmissionRejected
    );
}

#[test]
fn repeated_person_plans_with_two_plots_produce_for_both_people() {
    for review in [
        Policy::Monthly,
        Policy::RetainRepair,
        Policy::ScheduledReview,
    ] {
        let (mut sim, mut p) = setup(2, Backend::Reference, review, Allocation::StablePriority, 7);
        advance(&mut sim, &mut p, 25);
        for person in [PERSON, competition::SECOND_PERSON] {
            let crops = sim
                .state
                .processes
                .values()
                .filter(|x| {
                    x.operator == person && x.definition == GROW && x.status == Status::Completed
                })
                .count();
            assert!(crops >= 2, "{review:?} {person}: {crops}");
            assert!(
                p.controllers[&person]
                    .history
                    .iter()
                    .filter_map(|h| h.search.as_ref())
                    .all(|m| m.forecasts <= 32 && m.expansions <= 256)
            );
        }
        assert_eq!(sim.state.accepted_agreements.len(), 2);
        assert!(sim.state.balances.values().all(|q| *q >= 0));
        eprintln!(
            "{review:?} search counts: {:?}",
            p.controllers
                .iter()
                .map(|(a, c)| (a, c.history.iter().filter(|r| r.search.is_some()).count()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn shared_wood_is_reserved_once_and_policy_can_change_the_winner() {
    let mut winners = std::collections::BTreeSet::new();
    for seed in 0..10 {
        let (mut sim, mut p) = setup(
            2,
            Backend::Reference,
            Policy::Monthly,
            Allocation::Lottery,
            seed,
        );
        sim.world
            .definitions
            .iter_mut()
            .find(|d| d.id == GROW)
            .unwrap()
            .enabled = false;
        for a in [PERSON, competition::SECOND_PERSON] {
            sim.state.balances.insert((a, FUEL), 0);
        }
        for pool in &mut sim.world.pools {
            pool.capacity = 1;
            pool.monthly_regeneration = 1;
            sim.state.balances.insert(pool.account, 1);
        }
        fixtures::acquire(&mut sim).unwrap();
        let pools: Vec<_> = sim.world.pools.iter().map(|p| p.account).collect();
        p.step(&mut sim).unwrap();
        let round = p.history.last().unwrap();
        assert!(
            round
                .proposals
                .values()
                .all(|r| r.iter().any(|r| r.offer == Id::Process(PREPARE_FUEL)))
        );
        assert_eq!(
            round
                .admission
                .iter()
                .filter(|r| matches!(r.outcome, Outcome::Reserved(_)))
                .count(),
            1
        );
        p.step(&mut sim).unwrap();
        let work: Vec<_> = sim
            .state
            .processes
            .values()
            .filter(|x| x.definition == PREPARE_FUEL && x.status == Status::Completed)
            .collect();
        assert_eq!(work.len(), 1);
        winners.insert(work[0].operator);
        for a in pools {
            assert_eq!(sim.state.balance(a.0, a.1), 0);
        }
    }
    assert_eq!(winners.len(), 2);
}

#[test]
fn cpu_checkpoint_accounting_and_input_reordering_agree() {
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
        let (mut sim, mut p) = setup(2, backend, Policy::RetainRepair, Allocation::Lottery, 3);
        if matches!(backend, Backend::CubeCpu) {
            sim.world.participants.reverse();
            sim.world.access_offers.reverse();
            sim.world.definitions.reverse();
            sim.world.rights.reverse();
        }
        let mut audit = fixtures::audit(&sim).unwrap();
        while sim.state.phase != Phase::Productive {
            step(&mut sim, &mut p, &mut audit);
        }
        let (mut resumed, mut saved, mut book) = (sim.clone(), p.clone(), audit.clone());
        while sim.state.month < 15 {
            step(&mut sim, &mut p, &mut audit);
        }
        for end in 2..=15 {
            while resumed.state.month < end {
                step(&mut resumed, &mut saved, &mut book);
            }
        }
        assert_eq!(sim.state, resumed.state);
        assert_eq!(sim.ledger, resumed.ledger);
        assert_eq!(p, saved);
        assert_eq!(audit, book);
        results.push((sim.state, sim.reports, sim.ledger, p.history, audit));
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn rejected_commit_is_atomic_for_all_controllers_and_people() {
    let (mut sim, mut p) = setup(
        2,
        Backend::Reference,
        Policy::RetainRepair,
        Allocation::StablePriority,
        7,
    );
    fixtures::acquire(&mut sim).unwrap();
    sim.effect_limit = 0;
    let before = sim.clone();
    let controllers = p.clone();
    assert!(p.step(&mut sim).is_err());
    assert_eq!(sim.state, before.state);
    assert_eq!(sim.ledger, before.ledger);
    assert_eq!(p, controllers);
}

#[test]
fn missing_controller_and_unsupported_scope_are_explicit() {
    let (mut sim, mut p) = setup(
        2,
        Backend::Reference,
        Policy::Monthly,
        Allocation::StablePriority,
        7,
    );
    let controller = p.controllers.remove(&PERSON).unwrap();
    let before = sim.state.clone();
    assert!(p.step(&mut sim).unwrap_err().contains("each participant"));
    assert_eq!(sim.state, before);
    p.controllers.insert(
        PERSON,
        economics_compute_smoke::composition::continuation::Controller::new(
            economics_compute_smoke::composition::Scope::Person(PERSON),
            Strategy::BestFirst,
            Budget {
                expansions: 256,
                forecasts: 32,
                months: 12,
            },
            Policy::Monthly,
        ),
    );
    assert!(p.step(&mut sim).unwrap_err().contains("land request limit"));
    assert_eq!(sim.state, before);
    p.controllers.insert(PERSON, controller);
    let (mut household, _) = fixtures::fixture("B5", Backend::Reference).unwrap();
    assert!(
        p.step(&mut household)
            .unwrap_err()
            .contains("without households")
    );
}

#[test]
fn observed_private_stock_loss_replans_only_its_owner() {
    let (mut sim, mut p) = setup(
        2,
        Backend::Reference,
        Policy::RetainRepair,
        Allocation::StablePriority,
        7,
    );
    advance(&mut sim, &mut p, 4);
    fixtures::acquire(&mut sim).unwrap();
    // External fixture intervention; not part of the financial audit case.
    *sim.state
        .balances
        .get_mut(&(competition::SECOND_PERSON, GRAIN))
        .unwrap() -= 1;
    p.step(&mut sim).unwrap();
    assert_eq!(
        p.controllers[&PERSON].history.last().unwrap().reason,
        Reason::Retained
    );
    assert_eq!(
        p.controllers[&competition::SECOND_PERSON]
            .history
            .last()
            .unwrap()
            .reason,
        Reason::ObservationChanged
    );
}

#[test]
fn four_independent_people_acquire_distinct_rights_and_repeat_harvests() {
    let mut sim = fixtures::persons(4, 4, Backend::Reference).unwrap();
    let mut p = Persons::new(
        sim.world.participants.iter().map(|p| p.agent),
        Strategy::BestFirst,
        Budget {
            expansions: 256,
            forecasts: 32,
            months: 12,
        },
        Policy::RetainRepair,
        Allocation::Lottery,
        7,
    )
    .unwrap();
    fixtures::acquire(&mut sim).unwrap();
    p.step(&mut sim).unwrap();
    assert_eq!(
        p.history[0].accepted.len(),
        4,
        "equally valued plots should admit all four opening packages"
    );
    assert_eq!(
        sim.state
            .accepted_agreements
            .values()
            .map(|a| a.debtor)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    advance(&mut sim, &mut p, 25);
    for person in &sim.world.participants {
        assert!(
            sim.state
                .processes
                .values()
                .filter(|x| x.operator == person.agent
                    && x.definition == GROW
                    && x.status == Status::Completed)
                .count()
                >= 2,
            "person {}",
            person.agent
        );
    }
    assert_eq!(
        sim.state
            .accepted_agreements
            .values()
            .map(|a| a.debtor)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert!(sim.state.balances.values().all(|q| *q >= 0));
}

#[test]
fn thirty_two_people_share_one_opening_plot_without_duplicate_publication() {
    let mut sim = fixtures::persons(32, 1, Backend::Reference).unwrap();
    let mut p = Persons::new(
        sim.world.participants.iter().map(|p| p.agent),
        Strategy::BestFirst,
        Budget {
            expansions: 256,
            forecasts: 32,
            months: 12,
        },
        Policy::RetainRepair,
        Allocation::Lottery,
        7,
    )
    .unwrap();
    fixtures::acquire(&mut sim).unwrap();
    p.step(&mut sim).unwrap();
    assert_eq!(p.history[0].proposals.len(), 32);
    assert_eq!(sim.state.accepted_agreements.len(), 1);
    p.step(&mut sim).unwrap();
    assert_eq!(
        sim.state
            .processes
            .values()
            .filter(|p| p.definition == GROW)
            .count(),
        1
    );
    assert!(sim.state.balances.values().all(|q| *q >= 0));
}
