use economics_compute_smoke::{
    compute::Backend,
    laws::{self, Reason, Requirement, Rule},
    membership,
    model::*,
    opportunities::{self, Action, Opportunity, PERSON_TYPE},
    scenario::*,
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};

fn rule(id: u32, action: Action, requirement: Requirement) -> Rule {
    Rule {
        id,
        name: format!("rule {id}"),
        agent_type: Some(PERSON_TYPE),
        action,
        requirement,
    }
}

#[test]
fn laws_constrain_type_and_membership_grants_and_explain_denial() {
    let (mut w, mut s) = membership::scenario().unwrap();
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::LandAccess));
    w.transaction_policy.as_mut().unwrap().laws.push(rule(
        2,
        Action::LandAccess,
        Requirement::Membership(membership::CITIZEN),
    ));
    let d = laws::evaluate(&w, &s, PERSON, Action::LandAccess);
    assert!(!d.allowed);
    assert_eq!(
        d.reasons,
        vec![Reason::MembershipRequired {
            rule: 2,
            role: membership::CITIZEN
        }]
    );
    assert!(
        opportunities::discover(&w, &s, PERSON)
            .iter()
            .any(|o| matches!(o, Opportunity::StateAccess(_)))
    );
    let p = w.transaction_policy.as_ref().unwrap();
    let offer = &p.membership_offers[0];
    s.memberships.insert(
        (PERSON, p.authority, offer.role),
        membership::Agreement {
            member: PERSON,
            organization: p.authority,
            role: offer.role,
            source_offer: offer.id,
            accepted_month: s.month,
        },
    );
    assert!(laws::evaluate(&w, &s, PERSON, Action::LandAccess).allowed);
    w.transaction_policy.as_mut().unwrap().laws.push(rule(
        1,
        Action::LandAccess,
        Requirement::Prohibited,
    ));
    assert_eq!(
        laws::evaluate(&w, &s, PERSON, Action::LandAccess).reasons,
        vec![Reason::Prohibited { rule: 1 }]
    );
    assert!(
        !opportunities::discover(&w, &s, PERSON)
            .iter()
            .any(|o| matches!(o, Opportunity::StateAccess(_)))
    );
    s.memberships.clear();
    let before = laws::evaluate(&w, &s, PERSON, Action::LandAccess);
    assert_eq!(
        before.reasons,
        vec![
            Reason::Prohibited { rule: 1 },
            Reason::MembershipRequired {
                rule: 2,
                role: membership::CITIZEN
            },
        ]
    );
    w.transaction_policy.as_mut().unwrap().laws.reverse();
    assert_eq!(before, laws::evaluate(&w, &s, PERSON, Action::LandAccess));
    // Restore membership to isolate the unconditional prohibition below.
    let offer = w.transaction_policy.as_ref().unwrap().membership_offers[0].clone();
    s.memberships.insert(
        (PERSON, offer.organization, offer.role),
        membership::Agreement {
            member: PERSON,
            organization: offer.organization,
            role: offer.role,
            source_offer: offer.id,
            accepted_month: s.month,
        },
    );
    assert_eq!(
        laws::evaluate(&w, &s, PERSON, Action::LandAccess).reasons,
        vec![Reason::Prohibited { rule: 1 }]
    );
}

#[test]
fn prohibited_membership_cannot_be_used_as_a_discovery_prerequisite() {
    let (mut w, s) = membership::scenario().unwrap();
    w.transaction_policy.as_mut().unwrap().laws.push(rule(
        1,
        Action::Membership,
        Requirement::Prohibited,
    ));
    assert!(
        !opportunities::discover(&w, &s, PERSON)
            .iter()
            .any(
                |o| matches!(o, Opportunity::Membership(_) | Opportunity::StateAccess(_))
                    || matches!(o, Opportunity::Environment(d) if d.id == GROW)
            )
    );
}

#[test]
fn prohibited_access_rejects_forged_acceptance_atomically() {
    let (mut w, s) = opportunities::scenario().unwrap();
    w.transaction_policy.as_mut().unwrap().laws.push(rule(
        1,
        Action::LandAccess,
        Requirement::Prohibited,
    ));
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    assert_eq!(sim.state.phase, Phase::Acquire);
    let before = sim.state.clone();
    let mut b = Batch::empty(&sim.state);
    b.accept_access = Some(1);
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &b,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, sim.state);
}

#[test]
fn legal_alternatives_run_on_cpu_and_survive_checkpoints() {
    for prohibited in [false, true] {
        let (mut w, s) = membership::scenario().unwrap();
        let p = w.transaction_policy.as_mut().unwrap();
        p.laws.push(rule(
            2,
            Action::LandAccess,
            Requirement::Membership(membership::CITIZEN),
        ));
        if prohibited {
            p.laws
                .push(rule(1, Action::LandAccess, Requirement::Prohibited));
        }
        let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
        reference.run_months(6).unwrap();
        w.transaction_policy.as_mut().unwrap().laws.reverse();
        let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        for _ in 0..6 {
            cpu.step().unwrap();
            cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                .unwrap()
                .state;
            cpu.run_months(1).unwrap();
        }
        assert_eq!(reference.state, cpu.state);
        assert_eq!(reference.ledger, cpu.ledger);
        assert_eq!(reference.reports, cpu.reports);
        assert_eq!(cpu.state.accepted_agreements.is_empty(), prohibited);
        let food: i32 = cpu.reports.iter().map(|r| r.deficit(NUTRITION)).sum();
        let warmth: i32 = cpu.reports.iter().map(|r| r.deficit(WARMTH)).sum();
        println!(
            "land prohibited={prohibited}: access={}, memberships={}, food deficit={food}, warmth deficit={warmth}",
            cpu.state.accepted_agreements.len(),
            cpu.state.memberships.len()
        );
        if !prohibited {
            assert!(!cpu.state.memberships.is_empty());
            assert!(cpu.state.processes.values().any(|p| p.definition == GROW));
        }
    }
}

#[test]
fn malformed_laws_fail_validation_and_legacy_permissions_remain_explicit() {
    let (mut w, s) = opportunities::scenario().unwrap();
    let r = rule(1, Action::LandAccess, Requirement::Prohibited);
    w.transaction_policy.as_mut().unwrap().laws = vec![r.clone(), r];
    assert!(Simulation::new(w.clone(), s.clone(), Backend::Reference).is_err());
    w.transaction_policy.as_mut().unwrap().laws =
        vec![rule(1, Action::Process(u32::MAX), Requirement::Prohibited)];
    assert!(Simulation::new(w.clone(), s.clone(), Backend::Reference).is_err());
    let mut unnamed = rule(1, Action::LandAccess, Requirement::Prohibited);
    unnamed.name.clear();
    w.transaction_policy.as_mut().unwrap().laws = vec![unnamed];
    assert!(Simulation::new(w.clone(), s.clone(), Backend::Reference).is_err());
    w.transaction_policy = None;
    assert_eq!(
        laws::evaluate(&w, &s, PERSON, Action::LandAccess).reasons,
        vec![Reason::UnrestrictedLegacy]
    );
}

#[test]
fn law_is_rechecked_when_an_already_prepared_process_batch_settles() {
    let (w, s) = opportunities::scenario().unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim.step().unwrap();
    let batch = *sim.state.pending_production.clone().unwrap();
    let before = sim.state.clone();
    sim.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .laws
        .push(rule(1, Action::Process(GROW), Requirement::Prohibited));
    assert!(
        settlement::commit(
            &sim.world,
            &mut sim.state,
            &batch,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(before, sim.state);
}
