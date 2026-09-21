use economics_compute_smoke::{
    compute::Backend,
    model::*,
    opportunities::{self, Action, Opportunity, PERSON_TYPE, STATE_TYPE},
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn sim(backend: Backend) -> Simulation {
    let (w, s) = opportunities::scenario().unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn acquire(s: &mut Simulation) {
    while s.state.phase != Phase::Acquire {
        s.step().unwrap();
    }
}
fn deny(s: &mut Simulation, action: Action) {
    s.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(PERSON_TYPE, action));
}

#[test]
fn discovers_linked_offers_without_granting_rights() {
    let s = sim(Backend::Reference);
    assert_eq!(opportunities::discover(&s.world, &s.state, PERSON).len(), 5);
    assert!(opportunities::discover(&s.world, &s.state, STATE_AGENT).is_empty());
    assert_eq!(
        opportunities::relevant_access(&s.world, &s.state, PERSON)
            .into_iter()
            .collect::<Vec<_>>(),
        vec![1]
    );
    assert!(!economics_compute_smoke::commitments::can_start(
        &s.world, &s.state, 1
    ));
    assert!(s.state.processes.is_empty());
}

#[test]
fn cpu_repeated_harvests_and_taxes_match_reference_and_checkpoint() {
    let mut cpu = sim(Backend::CubeCpu);
    let mut reference = sim(Backend::Reference);
    cpu.run_months(12).unwrap();
    let mut resumed = cpu.clone();
    cpu.run_months(24).unwrap();
    for _ in 0..24 {
        resumed.run_months(1).unwrap();
    }
    reference.run_months(36).unwrap();
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.state, resumed.state);
    assert_eq!(cpu.reports, resumed.reports);
    assert_eq!(cpu.state.accepted_agreements.len(), 1);
    assert!(cpu.state.terminal.is_empty());
    assert!(
        cpu.reports
            .iter()
            .all(|r| r.deficit(NUTRITION) == 0 && r.deficit(WARMTH) == 0)
    );
    let harvests = cpu
        .state
        .processes
        .values()
        .filter(|p| p.definition == GROW && p.status == Status::Completed)
        .count();
    assert!(harvests >= 5, "harvests={harvests}");
    assert_eq!(cpu.state.obligations.len(), 2);
    assert!(
        cpu.state
            .obligations
            .values()
            .all(|o| o.owed == 2 && o.paid == 2)
    );
    assert!(
        cpu.state
            .processes
            .values()
            .any(|p| p.definition == PREPARE_FUEL)
    );
    println!(
        "36 months: {harvests} harvests; grain {}; seed {}; fuel {}; state grain {}; zero need deficits",
        cpu.state.balance(PERSON, GRAIN),
        cpu.state.balance(PERSON, SEED),
        cpu.state.balance(PERSON, FUEL),
        cpu.state.balance(STATE_AGENT, GRAIN)
    );
}

#[test]
fn denied_access_cannot_be_accepted_by_forged_batch() {
    let mut s = sim(Backend::Reference);
    deny(&mut s, Action::LandAccess);
    assert!(
        !opportunities::discover(&s.world, &s.state, PERSON)
            .iter()
            .any(|o| matches!(o, Opportunity::StateAccess(_)))
    );
    acquire(&mut s);
    let before = s.state.clone();
    let mut b = Batch::empty(&s.state);
    b.accept_access = Some(1);
    assert!(
        commit(
            &s.world,
            &mut s.state,
            &b,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, before);
    s.run_months(2).unwrap();
    assert!(s.state.accepted_agreements.is_empty());
    assert!(!s.state.processes.values().any(|p| p.definition == GROW));
}

#[test]
fn denied_farming_breaks_chain_and_scheduled_start_cannot_bypass_policy() {
    let mut s = sim(Backend::Reference);
    deny(&mut s, Action::Process(GROW));
    assert!(opportunities::relevant_access(&s.world, &s.state, PERSON).is_empty());
    s.world.scheduled_starts.push(ScheduledStart {
        month: 1,
        agent: PERSON,
        definition: GROW,
    });
    s.run_months(1).unwrap();
    assert!(s.state.accepted_agreements.is_empty());
    assert!(!s.state.processes.values().any(|p| p.definition == GROW));
    assert!(
        s.ledger
            .iter()
            .flat_map(|b| &b.receipts)
            .any(|r| r.reason == Reason::NotPermitted)
    );
}

#[test]
fn permission_is_rechecked_at_settlement_and_type_changes_deny_by_default() {
    let mut s = sim(Backend::Reference);
    acquire(&mut s);
    s.step().unwrap();
    let before = s.state.clone();
    let batch = *s.state.pending_production.clone().unwrap();
    deny(&mut s, Action::Process(GROW));
    assert!(
        commit(
            &s.world,
            &mut s.state,
            &batch,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, before);
    s.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .agent_types
        .insert(PERSON, STATE_TYPE);
    assert!(opportunities::discover(&s.world, &s.state, PERSON).is_empty());
}

#[test]
fn absent_seed_and_unaffordable_tax_do_not_attract_access_commitment() {
    for no_seed in [true, false] {
        let mut s = sim(Backend::Reference);
        if no_seed {
            s.state.balances.insert((PERSON, SEED), 0);
        } else {
            s.world.access_offers[0].payment.quantity = 100;
        }
        acquire(&mut s);
        s.step().unwrap();
        assert!(s.state.accepted_agreements.is_empty(), "no_seed={no_seed}");
    }
}

#[test]
fn wood_permission_changes_warmth_outcome() {
    let mut s = sim(Backend::Reference);
    deny(&mut s, Action::Process(PREPARE_FUEL));
    s.run_months(6).unwrap();
    assert!(
        !s.state
            .processes
            .values()
            .any(|p| p.definition == PREPARE_FUEL)
    );
    assert!(s.reports.iter().any(|r| r.deficit(WARMTH) > 0));
}
