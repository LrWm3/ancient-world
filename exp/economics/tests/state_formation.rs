use economics_compute_smoke::{
    agreements::{self, Identity, View},
    competition::SECOND_PERSON,
    compute::Backend,
    financial_reporting::{Audit, Opening},
    governance::{Ballot, Leadership, Rules},
    household_governance, households, laws,
    maintenance::{Condition, TerminalTransition},
    membership::{self, CITIZEN},
    model::*,
    offers::{self, Id, Request},
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    process_accounting::{BeneficiaryPolicy, Costs, Output},
    scenario::*,
    settlement,
    simulation::Simulation,
    state_governance::{self as g, formation as f},
    telemetry::{Config, Observer},
};
use std::collections::BTreeSet;

const NEW_STATE: AgentId = 9000;
const LATE_PERSON: AgentId = 90;

fn fixture() -> (World, State, households::Agreement) {
    // Reuse the productive two-person household control, before either
    // institution exists. All initial stocks remain the people's property.
    let (mut w, s) = household_governance::scenario::pair().unwrap();
    let household = w.households.remove(0);
    w.agents
        .retain(|a| a.id != STATE_AGENT && a.id != household.agent);
    let mut law = w.transaction_policy.take().unwrap();
    law.authority = NEW_STATE;
    law.agent_types.remove(&STATE_AGENT);
    law.agent_types.remove(&household.agent);
    law.agent_types.insert(NEW_STATE, STATE_TYPE);
    law.membership_offers = vec![membership::Offer {
        id: 1,
        organization: NEW_STATE,
        role: CITIZEN,
        eligible_type: PERSON_TYPE,
    }];
    law.permissions.insert((PERSON_TYPE, Action::Membership));
    law.permissions
        .remove(&(PERSON_TYPE, Action::FoundHousehold));
    law.membership_permissions
        .insert((CITIZEN, Action::FoundHousehold));
    w.state_founding = Some(f::Template {
        agent: Agent {
            id: NEW_STATE,
            name: "Founded state".into(),
        },
        eligible_founders: [PERSON, SECOND_PERSON].into(),
        minimum_founders: 2,
        constitution: g::Constitution {
            leadership: Leadership::Elected,
            policies: [
                (
                    0,
                    g::LegalPolicy {
                        name: "open".into(),
                        prohibited: BTreeSet::new(),
                    },
                ),
                (
                    1,
                    g::LegalPolicy {
                        name: "pause household formation".into(),
                        prohibited: [(Some(PERSON_TYPE), Action::FoundHousehold)].into(),
                    },
                ),
            ]
            .into(),
        },
        charter: g::Charter {
            founder: PERSON,
            term_months: 2,
            election: Rules::default(),
            initial_policy: 0,
        },
        law,
        citizenship_offer: 1,
    });
    settlement::validate_world(&w, &s).unwrap();
    (w, s, household)
}

fn found(w: &mut World, s: &mut State) -> f::Agreement {
    let a = f::propose(w, s, &[SECOND_PERSON, PERSON]).unwrap();
    f::accept(w, s, a.clone()).unwrap();
    a
}

fn death(w: &mut World, s: &mut State, id: AgentId) {
    add_condition_rules(w, "dead");
    let threshold = w
        .condition_rules
        .iter()
        .find(|r| r.subject == id && r.provision == NUTRITION)
        .unwrap()
        .terminal_at;
    s.conditions.insert(
        (id, NUTRITION),
        Condition {
            deprivation: threshold,
            adverse_months: 1,
        },
    );
    s.terminal.insert(
        id,
        TerminalTransition {
            month: s.month - 1,
            subject: id,
            reason: NUTRITION,
            state: "dead".into(),
        },
    );
}

#[test]
fn founding_is_read_only_until_atomic_acceptance_and_creates_no_wealth_or_labor() {
    let (mut w, mut s, _) = fixture();
    w.assets.push(Asset {
        id: 700,
        owner: PERSON,
        kind: 1,
    });
    let before_w = w.clone();
    let before_s = s.clone();
    let a = f::propose(&w, &s, &[SECOND_PERSON, PERSON]).unwrap();
    assert_eq!(a, f::propose(&w, &s, &[PERSON, SECOND_PERSON]).unwrap());
    assert_eq!(w, before_w);
    assert_eq!(s, before_s);
    f::accept(&mut w, &mut s, a.clone()).unwrap();
    assert_eq!(w.participants, before_w.participants);
    assert_eq!(w.assets, before_w.assets);
    assert_eq!(w.rights, before_w.rights);
    assert_eq!(w.ownership_rights, before_w.ownership_rights);
    let mut without_memberships = s.clone();
    without_memberships.memberships.clear();
    assert_eq!(without_memberships, before_s);
    assert_eq!(g::authority(&w, &s).unwrap().governor, Some(PERSON));
    for id in [NEW_STATE, PERSON, SECOND_PERSON] {
        let views = agreements::for_agent(&w, &s, id).unwrap();
        let view = views
            .iter()
            .find(|v| v.identity() == Identity::StateFormation(NEW_STATE))
            .unwrap();
        assert_eq!(view.parties(), vec![PERSON, SECOND_PERSON, NEW_STATE]);
        assert_eq!(view.accepted_month(), 1);
        assert!(view.claims().unwrap().is_empty());
        assert_eq!(view.holder(), None);
        assert_eq!(view.grantor(), None);
        assert!(matches!(view, View::StateFormation(v) if v.terms == &a));
    }
    let before = (w.clone(), s.clone());
    assert!(f::accept(&mut w, &mut s, a).is_err());
    assert_eq!((w, s), before);
}

#[test]
fn eligibility_does_not_make_a_non_signer_a_citizen_or_founder() {
    let (mut w, mut s, _) = fixture();
    w.state_founding.as_mut().unwrap().minimum_founders = 1;
    let proposal = f::propose(&w, &s, &[PERSON]).unwrap();
    f::accept(&mut w, &mut s, proposal).unwrap();
    assert!(s.memberships.contains_key(&(PERSON, NEW_STATE, CITIZEN)));
    assert!(
        !s.memberships
            .contains_key(&(SECOND_PERSON, NEW_STATE, CITIZEN))
    );
    assert!(!laws::evaluate(&w, &s, SECOND_PERSON, Action::FoundHousehold).allowed);
    assert!(
        !agreements::for_agent(&w, &s, SECOND_PERSON)
            .unwrap()
            .iter()
            .any(|v| v.identity() == Identity::StateFormation(NEW_STATE))
    );
}

#[test]
fn invalid_signatures_terms_and_citizenship_law_reject_without_publication() {
    let (w, s, _) = fixture();
    for signatures in [
        vec![],
        vec![PERSON],
        vec![PERSON, PERSON],
        vec![PERSON, 999],
    ] {
        assert!(f::propose(&w, &s, &signatures).is_err());
    }
    for case in 0..9 {
        let mut w = w.clone();
        let t = w.state_founding.as_mut().unwrap();
        match case {
            0 => t.minimum_founders = 0,
            1 => t.charter.term_months = 0,
            2 => t.charter.founder = 999,
            3 => t
                .law
                .agent_types
                .insert(PERSON, STATE_TYPE)
                .map(|_| ())
                .unwrap(),
            4 => t.citizenship_offer = 999,
            5 => {
                t.law.permissions.remove(&(PERSON_TYPE, Action::Membership));
            }
            6 => t.constitution.policies.clear(),
            7 => t.agent.id = PERSON,
            _ => t
                .law
                .membership_offers
                .push(t.law.membership_offers[0].clone()),
        }
        let before = w.clone();
        assert!(
            f::propose(&w, &s, &[PERSON, SECOND_PERSON]).is_err(),
            "case {case}"
        );
        assert_eq!(w, before);
    }
}

#[test]
fn acceptance_rechecks_death_date_phase_offer_and_existing_authority() {
    for case in 0..6 {
        let (mut w, mut s, _) = fixture();
        s.month = 2;
        let a = f::propose(&w, &s, &[PERSON, SECOND_PERSON]).unwrap();
        match case {
            0 => death(&mut w, &mut s, SECOND_PERSON),
            1 => s.month += 1,
            2 => s.phase = Phase::Acquire,
            3 => w.state_founding.as_mut().unwrap().charter.term_months += 1,
            4 => w.transaction_policy = Some(a.terms.law.clone()),
            _ => w.agents.push(a.terms.agent.clone()),
        }
        let before = (w.clone(), s.clone());
        assert!(f::accept(&mut w, &mut s, a).is_err(), "case {case}");
        assert_eq!((w, s), before);
    }
}

#[test]
fn accepted_terms_are_immutable_but_later_death_does_not_erase_founding() {
    let (mut w, mut s, _) = fixture();
    found(&mut w, &mut s);
    for case in 0..4 {
        let (mut w, mut s) = (w.clone(), s.clone());
        match case {
            0 => w.state_governance.as_mut().unwrap().charter.term_months += 1,
            1 => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .insert((PERSON_TYPE, Action::Lend));
            }
            2 => {
                s.memberships
                    .get_mut(&(PERSON, NEW_STATE, CITIZEN))
                    .unwrap()
                    .source_offer = 999
            }
            _ => {
                w.state_governance.as_mut().unwrap().constitution.leadership = Leadership::Rotating
            }
        }
        assert!(settlement::validate_world(&w, &s).is_err());
    }
    s.month = 3;
    death(&mut w, &mut s, PERSON);
    settlement::validate_world(&w, &s).unwrap();
    assert_eq!(g::authority(&w, &s).unwrap().governor, None);
    assert!(
        agreements::for_agent(&w, &s, PERSON)
            .unwrap()
            .iter()
            .any(|v| v.identity() == Identity::StateFormation(NEW_STATE))
    );
}

#[test]
fn founded_law_controls_households_and_later_citizens_use_normal_offer_settlement() {
    let (mut w, mut s, h) = fixture();
    found(&mut w, &mut s);
    g::schedule(
        &mut w,
        &s,
        g::PolicyChange {
            month: 2,
            authorized_by: PERSON,
            policy: 1,
        },
    )
    .unwrap();
    s.month = 2;
    let mut h = h;
    h.formed = 2;
    let before = w.clone();
    assert!(households::form(&mut w, &s, h).is_err());
    assert_eq!(w, before);
    w.agents.push(Agent {
        id: LATE_PERSON,
        name: "later citizen".into(),
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .agent_types
        .insert(LATE_PERSON, PERSON_TYPE);
    s.phase = Phase::Acquire;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    assert!(
        offers::discover(&sim.world, &sim.state, LATE_PERSON)
            .iter()
            .any(|o| o.id == Id::Membership(1))
    );
    offers::accept(&mut sim, &[Request::new(Id::Membership(1), LATE_PERSON)]).unwrap();
    let w = sim.world;
    let s = sim.state;
    assert!(
        s.memberships
            .contains_key(&(LATE_PERSON, NEW_STATE, CITIZEN))
    );
    assert!(
        !agreements::for_agent(&w, &s, LATE_PERSON)
            .unwrap()
            .iter()
            .any(|v| v.identity() == Identity::StateFormation(NEW_STATE))
    );
    assert!(!laws::evaluate(&w, &s, LATE_PERSON, Action::FoundHousehold).allowed);
}

#[test]
fn audit_rejects_stale_boundaries_and_failed_founding_without_reopening_books() {
    let (mut w, mut s, _) = fixture();
    let mut audit = Audit::with_opening(
        &w,
        &s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            ..Opening::default()
        },
    )
    .unwrap();
    let proposal = f::propose(&w, &s, &[PERSON, SECOND_PERSON]).unwrap();
    let entries = audit.book().entries().to_vec();
    let original = s.clone();
    s.balances.insert((PERSON, TOKEN), 1);
    let before = (w.clone(), s.clone());
    assert!(
        audit
            .accept_state_founding(&mut w, &mut s, proposal.clone())
            .is_err()
    );
    assert_eq!((w.clone(), s.clone()), before);
    assert_eq!(audit.book().entries(), entries);
    s = original;
    let mut invalid = proposal.clone();
    invalid.founders.pop();
    let before = (w.clone(), s.clone());
    assert!(
        audit
            .accept_state_founding(&mut w, &mut s, invalid)
            .is_err()
    );
    assert_eq!((w.clone(), s.clone()), before);
    audit
        .accept_state_founding(&mut w, &mut s, proposal)
        .unwrap();
    assert_eq!(audit.book().entries(), entries);
}

#[test]
fn founding_cannot_legitimize_preallocated_property_for_the_new_identity() {
    for account in [true, false] {
        let (mut w, mut s, _) = fixture();
        let proposal = f::propose(&w, &s, &[PERSON, SECOND_PERSON]).unwrap();
        if account {
            s.balances.insert((NEW_STATE, TOKEN), 10);
        } else {
            w.assets.push(Asset {
                id: 700,
                owner: NEW_STATE,
                kind: 1,
            });
        }
        let before = (w.clone(), s.clone());
        assert!(f::accept(&mut w, &mut s, proposal).is_err());
        assert_eq!((w, s), before);
    }
}

#[test]
fn founded_state_household_and_person_loop_agree_on_cpu_and_checkpoint_continuation() {
    let (mut w, mut s, h) = fixture();
    let mut audit = Audit::with_opening(
        &w,
        &s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            processes: Some(Costs {
                beneficiary_policy: Some(BeneficiaryPolicy::TransferAtCost),
                output_weights: [(
                    household_governance::scenario::FOOD_PROCESS,
                    [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                )]
                .into(),
                ..Costs::default()
            }),
            ..Opening::default()
        },
    )
    .unwrap();
    let proposal = f::propose(&w, &s, &[PERSON, SECOND_PERSON]).unwrap();
    let opening_entries = audit.book().entries().to_vec();
    audit
        .accept_state_founding(&mut w, &mut s, proposal)
        .unwrap();
    assert_eq!(audit.book().entries(), opening_entries);
    households::form(&mut w, &s, h).unwrap();
    for voter in [PERSON, SECOND_PERSON] {
        g::cast(
            &mut w,
            &s,
            Ballot {
                term_start: 3,
                voter,
                candidate: Some(SECOND_PERSON),
            },
        )
        .unwrap();
    }
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut observer = Observer::new(vec![], "state-formation", Config::default()).unwrap();
    while cpu.state.month < 7 {
        observer.step_audited(&mut cpu, &mut audit).unwrap();
    }
    reference.run_months(2).unwrap();
    assert_eq!(
        g::authority(&reference.world, &reference.state)
            .unwrap()
            .governor,
        Some(SECOND_PERSON)
    );
    let mut resumed = Simulation::new(
        reference.world.clone(),
        reference.state.clone(),
        Backend::Reference,
    )
    .unwrap();
    for _ in 0..4 {
        resumed.run_months(1).unwrap();
    }
    reference.run_months(4).unwrap();
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    assert_eq!(resumed.state, reference.state);
    assert_eq!(resumed.world, reference.world);
    assert!(
        cpu.state
            .processes
            .values()
            .any(|p| p.status == Status::Completed)
    );
    assert!(cpu.state.terminal.is_empty());
}
