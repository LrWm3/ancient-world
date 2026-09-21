use economics_compute_smoke::{
    compute::Backend,
    membership::{self, CITIZEN},
    model::*,
    opportunities::{self, Action, Opportunity, PERSON_TYPE, STATE_TYPE},
    scenario::*,
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn sim(backend: Backend) -> Simulation {
    let (w, s) = membership::scenario().unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn acquire(s: &mut Simulation) {
    while s.state.phase != Phase::Acquire {
        s.step().unwrap();
    }
}
fn settle(s: &mut Simulation, b: &Batch) -> Result<(), String> {
    commit(
        &s.world,
        &mut s.state,
        b,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
}

#[test]
fn noncitizen_sees_obtainable_prerequisites_but_cannot_execute_them() {
    let mut s = sim(Backend::Reference);
    assert!(s.state.memberships.is_empty());
    assert!(!opportunities::permits(
        &s.world,
        &s.state,
        PERSON,
        Action::Process(GROW)
    ));
    assert!(!opportunities::permits(
        &s.world,
        &s.state,
        PERSON,
        Action::LandAccess
    ));
    let rows = opportunities::discover(&s.world, &s.state, PERSON);
    assert!(rows.iter().any(|o| matches!(o, Opportunity::Membership(_))));
    assert!(
        rows.iter()
            .any(|o| matches!(o,Opportunity::Environment(d) if d.id==GROW))
    );
    assert!(
        rows.iter()
            .any(|o| matches!(o, Opportunity::StateAccess(_)))
    );
    assert_eq!(
        opportunities::relevant_access(&s.world, &s.state, PERSON).len(),
        1
    );
    acquire(&mut s);
    let before = s.state.clone();
    let mut b = Batch::empty(&s.state);
    b.accept_access = Some(1);
    assert!(settle(&mut s, &b).is_err());
    assert_eq!(s.state, before);
}

#[test]
fn acceptance_is_agreement_and_scoped_relationship_without_changing_person_type() {
    let mut s = sim(Backend::Reference);
    acquire(&mut s);
    let balances = s.state.balances.clone();
    let mut b = Batch::empty(&s.state);
    b.accept_membership = Some((1, PERSON));
    settle(&mut s, &b).unwrap();
    let m = &s.state.memberships[&(PERSON, STATE_AGENT, CITIZEN)];
    assert_eq!((m.source_offer, m.accepted_month), (1, 1));
    assert_eq!(
        s.world.transaction_policy.as_ref().unwrap().agent_types[&PERSON],
        PERSON_TYPE
    );
    assert_eq!(s.state.balances, balances);
    assert!(s.state.accepted_agreements.is_empty());
    assert!(s.state.obligations.is_empty());
    assert!(opportunities::permits(
        &s.world,
        &s.state,
        PERSON,
        Action::LandAccess
    ));
    assert!(opportunities::permits(
        &s.world,
        &s.state,
        PERSON,
        Action::Process(GROW)
    ));
    assert!(!opportunities::permits(
        &s.world,
        &s.state,
        STATE_AGENT,
        Action::Process(GROW)
    ));
    // Identical role in another organization does not grant the state's permissions.
    s.world.agents.push(Agent {
        id: 99,
        name: "Other state".into(),
    });
    s.world.transaction_policy.as_mut().unwrap().authority = 99;
    assert!(!opportunities::permits(
        &s.world,
        &s.state,
        PERSON,
        Action::LandAccess
    ));
}

#[test]
fn full_chain_is_selected_once_and_cpu_matches_reference_and_checkpoint() {
    let mut cpu = sim(Backend::CubeCpu);
    acquire(&mut cpu);
    cpu.step().unwrap();
    let b = cpu.ledger.last().unwrap();
    assert_eq!(b.accept_membership, Some((1, PERSON)));
    assert_eq!(b.accept_access, Some(1));
    let d = b.decision.as_ref().unwrap();
    assert_eq!(
        d.alternatives[d.selected].plan.membership_offer(),
        Some((1, PERSON))
    );
    let mut checkpoint = cpu.clone();
    cpu.run_months(36).unwrap();
    for _ in 0..36 {
        checkpoint.run_months(1).unwrap();
    }
    let mut reference = sim(Backend::Reference);
    reference.run_months(36).unwrap();
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.state, checkpoint.state);
    assert_eq!(cpu.reports, checkpoint.reports);
    assert_eq!(cpu.state.memberships.len(), 1);
    assert_eq!(
        cpu.ledger
            .iter()
            .filter(|b| b.accept_membership.is_some())
            .count(),
        1
    );
    assert_eq!(cpu.state.obligations.len(), 2);
    assert!(
        cpu.state
            .obligations
            .values()
            .all(|o| o.paid == 2 && o.owed == 2)
    );
    assert!(
        cpu.reports
            .iter()
            .all(|r| r.deficit(NUTRITION) == 0 && r.deficit(WARMTH) == 0)
    );
    assert_eq!(
        cpu.state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Completed)
            .count(),
        5
    );
}

#[test]
fn membership_and_land_bundle_rolls_back_if_any_leg_fails() {
    let mut s = sim(Backend::Reference);
    acquire(&mut s);
    let before = s.state.clone();
    let mut b = Batch::empty(&s.state);
    b.accept_membership = Some((1, PERSON));
    b.accept_access = Some(999);
    assert!(settle(&mut s, &b).is_err());
    assert_eq!(s.state, before);
    b.accept_access = Some(1);
    settle(&mut s, &b).unwrap();
    assert_eq!(s.state.memberships.len(), 1);
    assert_eq!(s.state.accepted_agreements.len(), 1);
    let after = s.state.clone();
    assert!(settle(&mut s, &b).is_err());
    assert_eq!(s.state, after);
    s.run_months(1).unwrap();
    acquire(&mut s);
    let mut duplicate = Batch::empty(&s.state);
    duplicate.accept_membership = Some((1, PERSON));
    let before = s.state.clone();
    assert!(settle(&mut s, &duplicate).is_err());
    assert_eq!(s.state, before);
}

#[test]
fn no_offer_no_permission_and_ineligible_type_block_citizenship() {
    for control in 0..3 {
        let mut s = sim(Backend::Reference);
        let p = s.world.transaction_policy.as_mut().unwrap();
        match control {
            0 => p.membership_offers.clear(),
            1 => {
                p.permissions.remove(&(PERSON_TYPE, Action::Membership));
            }
            _ => {
                p.membership_offers[0].eligible_type = STATE_TYPE;
            }
        }
        acquire(&mut s);
        let mut b = Batch::empty(&s.state);
        b.accept_membership = Some((1, PERSON));
        assert!(settle(&mut s, &b).is_err());
        s.run_months(2).unwrap();
        assert!(s.state.memberships.is_empty());
        assert!(s.state.accepted_agreements.is_empty());
        assert!(!s.state.processes.values().any(|p| p.definition == GROW));
    }
}

#[test]
fn absent_seed_or_unaffordable_land_does_not_trigger_purposeless_membership() {
    for seedless in [true, false] {
        let mut s = sim(Backend::Reference);
        if seedless {
            s.state.balances.insert((PERSON, SEED), 0);
        } else {
            s.world.access_offers[0].payment.quantity = 100;
        }
        acquire(&mut s);
        s.step().unwrap();
        assert!(s.state.memberships.is_empty());
        assert!(s.state.accepted_agreements.is_empty());
    }
}

#[test]
fn citizenship_has_no_upkeep_and_land_arrears_do_not_revoke_it() {
    let mut s = sim(Backend::Reference);
    s.world.access_offers[0].payment.quantity = 100;
    acquire(&mut s);
    let mut b = Batch::empty(&s.state);
    b.accept_membership = Some((1, PERSON));
    b.accept_access = Some(1);
    settle(&mut s, &b).unwrap();
    let membership = s.state.memberships.clone();
    s.run_months(13).unwrap();
    assert!(s.state.obligations.values().any(|o| o.paid < o.owed));
    assert_eq!(s.state.memberships, membership);
    assert!(opportunities::permits(
        &s.world,
        &s.state,
        PERSON,
        Action::Process(GROW)
    ));
    assert!(!economics_compute_smoke::commitments::can_start(
        &s.world, &s.state, 1
    ));
}

#[test]
fn one_posted_offer_serves_multiple_people_without_creating_membership_dues() {
    let mut s = sim(Backend::Reference);
    s.world.priority = Priority::ContinuingFirst;
    let other = PERSON + 1;
    s.world.agents.push(Agent {
        id: other,
        name: "Second person".into(),
    });
    s.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .agent_types
        .insert(other, PERSON_TYPE);
    for person in [PERSON, other] {
        acquire(&mut s);
        let mut b = Batch::empty(&s.state);
        b.accept_membership = Some((1, person));
        settle(&mut s, &b).unwrap();
    }
    s.run_months(13).unwrap();
    assert_eq!(s.state.memberships.len(), 2);
    assert!(s.state.memberships.values().all(|m| m.source_offer == 1));
    assert_eq!(
        s.world
            .transaction_policy
            .as_ref()
            .unwrap()
            .membership_offers
            .len(),
        1
    );
    assert!(s.state.obligations.is_empty());
    assert!(s.state.accepted_agreements.is_empty());
}
