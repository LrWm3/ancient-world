#[path = "support/release_evidence.rs"]
mod release_evidence;
use economics_compute_smoke::{
    allocation::Policy,
    competition::{self, Application, SECOND_PERSON},
    compute::Backend,
    financial_reporting::{Audit, Opening},
    household_governance::Governance,
    households::{self, Agreement},
    model::*,
    offers::{self, Id, Request},
    process_accounting::{Costs, Output},
    scenario::*,
    settlement,
    simulation::Simulation,
};
const HOME: AgentId = 10000;
fn fixture(plots: u32, backend: Backend) -> (Simulation, Audit) {
    let (mut w, mut s) = competition::scenario(plots, 7).unwrap();
    w.competition = None; // Explicit bundles, then ordinary fixed-priority work.
    w.priority = Priority::ContinuingFirst;
    w.decision_horizon = None;
    w.transaction_policy.as_mut().unwrap().permissions.insert((
        economics_compute_smoke::opportunities::PERSON_TYPE,
        economics_compute_smoke::opportunities::Action::FoundHousehold,
    ));
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    for person in &mut w.participants {
        person.capacity.quantity = 5; // One contributed hour and four private hours.
        w.storage.capacities.insert(person.agent, 40);
        s.balances.insert((person.agent, SEED), 0);
    }
    w.storage.weights.extend([(GRAIN, 1), (SEED, 1), (FUEL, 1)]);
    households::form(
        &mut w,
        &s,
        Agreement {
            id: 1,
            agent: HOME,
            adults: vec![PERSON, SECOND_PERSON],
            governance: Governance::contributed(PERSON),
            formed: 1,
            dwelling_process: None,
            admission: None,
            membership: vec![],
            asset_sales: vec![],
            equipment_retirements: vec![],
            support: vec![],
        },
    )
    .unwrap();
    s.balances.insert((HOME, SEED), 2);
    s.balances.insert((HOME, GRAIN), 10);
    s.balances.insert((HOME, TOKEN), 4);
    let audit = Audit::with_opening(
        &w,
        &s,
        TOKEN,
        Opening {
            assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| **q > 0 && *r != TOKEN)
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            exchange_values: w
                .resources
                .iter()
                .filter(|r| r.kind == ResourceKind::Stock && r.id != TOKEN)
                .map(|r| (r.id, 1))
                .collect(),
            processes: Some(Costs {
                output_weights: [(
                    GROW,
                    [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                )]
                .into(),
                ..Costs::default()
            }),
            dues: Some(economics_compute_smoke::dues_accounting::Valuation(
                w.access_offers.iter().map(|a| (a.id, 1)).collect(),
            )),
            ..Opening::default()
        },
    )
    .unwrap();
    (Simulation::new(w, s, backend).unwrap(), audit)
}
fn acquire(sim: &mut Simulation, audit: &mut Audit) {
    while sim.state.phase != Phase::Acquire {
        audit.step(sim).unwrap();
    }
}
fn bundle(agent: AgentId, plot: u32) -> Vec<Request> {
    vec![
        Request::new(Id::Membership(1), agent),
        Request::new(Id::Land(plot), agent),
        Request::new(Id::Process(GROW), agent),
    ]
}
fn joint() -> Vec<Request> {
    vec![
        Request::new(Id::Membership(1), PERSON),
        Request::new(Id::Land(1), PERSON),
        Request::new(Id::Membership(1), SECOND_PERSON),
        Request::new(Id::Land(2), SECOND_PERSON),
        Request::new(Id::Process(GROW), PERSON),
        Request::new(Id::Process(GROW), SECOND_PERSON),
    ]
}
fn accept(sim: &mut Simulation, audit: &mut Audit, requests: &[Request]) {
    let before = sim.state.clone();
    let ledger = sim.ledger.clone();
    let batch = offers::prepare(sim, requests).unwrap();
    assert_eq!(sim.state, before);
    assert_eq!(sim.ledger, ledger);
    offers::accept(sim, requests).unwrap();
    audit
        .record(&sim.world, &before, &batch, &sim.state)
        .unwrap();
}
#[test]
fn household_prerequisites_reserve_once_and_repeated_harvests_reconcile() {
    let run = |backend| {
        let (mut sim, mut audit) = fixture(2, backend);
        acquire(&mut sim, &mut audit);
        accept(&mut sim, &mut audit, &joint());
        assert_eq!(sim.state.accepted_agreements.len(), 2);
        assert_eq!(sim.state.balance(HOME, SEED), 2);
        assert!(sim.state.processes.is_empty());
        assert!(
            sim.state
                .pending_production
                .as_ref()
                .unwrap()
                .household
                .is_some()
        );
        let mut resumed = Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
        let mut resumed_audit = audit.clone();
        let prefix = sim.ledger.len();
        while sim.state.month <= 24 {
            audit.step(&mut sim).unwrap();
        }
        while resumed.state.month <= 24 {
            resumed_audit.step(&mut resumed).unwrap();
        }
        assert_eq!(sim.state, resumed.state);
        assert_eq!(audit, resumed_audit);
        assert_eq!(&sim.ledger[prefix..], resumed.ledger.as_slice());
        let completed = sim
            .state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Completed)
            .count();
        assert!(completed >= 2, "completed {completed} crops");
        let pooled_grain: i32 = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.after)
            .filter(|e| e.account == (HOME, GRAIN) && e.delta > 0)
            .map(|e| e.delta)
            .sum();
        let yield_per_crop = sim
            .world
            .definition(GROW)
            .outputs
            .iter()
            .find(|a| a.resource == GRAIN)
            .unwrap()
            .quantity;
        assert_eq!(pooled_grain, completed as i32 * yield_per_crop / 2);
        assert!(
            sim.ledger
                .iter()
                .filter_map(|b| b.household.as_ref())
                .flat_map(|h| &h.before)
                .any(|e| e.account == (HOME, SEED) && e.delta < 0)
        );
        for agent in [PERSON, SECOND_PERSON, HOME, STATE_AGENT] {
            audit.book().statements(agent, 1, 24).unwrap();
        }
        println!(
            "S1 {backend:?}: months=24 crops={completed} grain_pooled={pooled_grain} reports={} reconciled=true",
            sim.reports.len()
        );
        release_evidence::record("S1 accepted household farming", &sim, &audit);
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn household_packages_reject_missing_permission_seed_and_overcommitted_labor_atomically() {
    for control in 0..4 {
        let (mut sim, mut audit) = fixture(2, Backend::Reference);
        acquire(&mut sim, &mut audit);
        let mut requests = joint();
        match control {
            0 => {
                sim.world
                    .transaction_policy
                    .as_mut()
                    .unwrap()
                    .membership_permissions
                    .remove(&(
                        economics_compute_smoke::membership::CITIZEN,
                        economics_compute_smoke::opportunities::Action::Process(GROW),
                    ));
            }
            1 => {
                sim.state.balances.insert((HOME, SEED), 1);
            }
            2 => requests.extend((0..4).map(|_| Request::new(Id::Process(PREPARE_FUEL), PERSON))),
            _ => {
                for agent in [PERSON, SECOND_PERSON] {
                    sim.world.storage.capacities.insert(agent, 12);
                    requests.push(Request::new(Id::Process(PREPARE_FUEL), agent));
                }
            }
        }
        let before = sim.state.clone();
        let ledger = sim.ledger.clone();
        assert!(
            offers::accept(&mut sim, &requests).is_err(),
            "control {control}"
        );
        assert_eq!(sim.state, before);
        assert_eq!(sim.ledger, ledger);
    }
}
#[test]
fn household_stale_or_forged_receipts_publish_nothing() {
    let (mut sim, mut audit) = fixture(2, Backend::CubeCpu);
    acquire(&mut sim, &mut audit);
    let batch = offers::prepare(&sim, &joint()).unwrap();
    let mut stale = sim.state.clone();
    stale.balances.insert((HOME, SEED), 1);
    let unchanged = stale.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut stale,
            &batch,
            sim.backend,
            sim.effect_limit
        )
        .is_err()
    );
    assert_eq!(stale, unchanged);
    let mut forged = batch.clone();
    forged
        .production_plan
        .as_mut()
        .unwrap()
        .household
        .as_mut()
        .unwrap()
        .before
        .clear();
    let mut unchanged = sim.state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut unchanged,
            &forged,
            sim.backend,
            sim.effect_limit
        )
        .is_err()
    );
    assert_eq!(unchanged, sim.state);
    let mut bad = batch.clone();
    bad.household.as_mut().unwrap().before.push(Effect {
        account: (PERSON, TOKEN),
        delta: 1,
    });
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
    accept(&mut sim, &mut audit, &joint());
    let before = sim.state.clone();
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &batch,
            sim.backend,
            sim.effect_limit
        )
        .is_err()
    );
    assert_eq!(sim.state, before);
    let mut bad = *sim.state.pending_production.clone().unwrap();
    bad.household.as_mut().unwrap().after.push(Effect {
        account: (HOME, GRAIN),
        delta: 1,
    });
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
    audit.step(&mut sim).unwrap();
}
#[test]
fn household_plot_competition_keeps_loser_free_of_rights_and_claims() {
    let (mut sim, mut audit) = fixture(1, Backend::CubeCpu);
    acquire(&mut sim, &mut audit);
    let applications: Vec<_> = [PERSON, SECOND_PERSON]
        .into_iter()
        .map(|agent| Application {
            agent,
            requests: bundle(agent, 1),
        })
        .collect();
    let batch = competition::prepare(&sim, 1, 7, Policy::PriorityLottery, &applications).unwrap();
    let mut reverse = applications.clone();
    reverse.reverse();
    assert_eq!(
        batch,
        competition::prepare(&sim, 1, 7, Policy::PriorityLottery, &reverse).unwrap()
    );
    let before = sim.state.clone();
    competition::accept(&mut sim, 1, 7, Policy::PriorityLottery, &applications).unwrap();
    audit
        .record(&sim.world, &before, &batch, &sim.state)
        .unwrap();
    assert_eq!(sim.state.accepted_agreements.len(), 1);
    assert_eq!(sim.state.memberships.len(), 1);
    let winner = sim.state.accepted_agreements[&1].debtor;
    audit.step(&mut sim).unwrap();
    assert!(
        sim.state
            .processes
            .values()
            .filter(|p| p.definition == GROW)
            .all(|p| p.operator == winner)
    );
}
#[test]
fn household_missed_work_keeps_seed_sunk_and_records_failed_agreement() {
    let (mut sim, mut audit) = fixture(2, Backend::CubeCpu);
    for agent in [PERSON, SECOND_PERSON] {
        sim.world.capacity_overrides.insert((2, agent), 0);
    }
    acquire(&mut sim, &mut audit);
    accept(&mut sim, &mut audit, &joint());
    while sim.state.month <= 2 {
        audit.step(&mut sim).unwrap();
    }
    assert_eq!(
        sim.state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Aborted)
            .count(),
        2
    );
    assert_eq!(
        [HOME, PERSON, SECOND_PERSON]
            .iter()
            .map(|a| sim.state.balance(*a, SEED))
            .sum::<i32>(),
        0
    );
}
