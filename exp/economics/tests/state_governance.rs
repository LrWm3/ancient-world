use economics_compute_smoke::{
    competition::SECOND_PERSON,
    compute::Backend,
    financial_reporting::{Audit, Opening},
    governance::{Ballot, ElectionTieBreak, Leadership},
    household_governance as household, households,
    laws::{self, Reason},
    maintenance::TerminalTransition,
    membership::{self, CITIZEN},
    model::*,
    opportunities::{self, Action, PERSON_TYPE},
    process_accounting::{BeneficiaryPolicy, Costs, Output},
    scenario::*,
    settlement,
    simulation::Simulation,
    state_governance::{
        self as g, PolicyChange,
        scenario::{OPEN, PAUSE_ADMISSIONS},
    },
    telemetry::{Config, Observer},
};

const HOME: AgentId = 10000;

fn change(month: u32, authorized_by: AgentId, policy: g::PolicyId) -> PolicyChange {
    PolicyChange {
        month,
        authorized_by,
        policy,
    }
}

fn death(s: &mut State, id: AgentId, month: u32) {
    s.terminal.insert(
        id,
        TerminalTransition {
            month,
            subject: id,
            reason: NUTRITION,
            state: "dead".into(),
        },
    );
}

#[test]
fn elected_citizen_sets_future_policy_without_changing_founding_parameters_or_balances() {
    let (mut w, mut s) = g::scenario::pair().unwrap();
    let charter = w.state_governance.as_ref().unwrap().charter.clone();
    let constitution = w.state_governance.as_ref().unwrap().constitution.clone();
    let before = s.clone();
    g::schedule(&mut w, &s, change(2, PERSON, PAUSE_ADMISSIONS)).unwrap();
    assert_eq!(s, before);
    assert!(laws::evaluate(&w, &s, PERSON, Action::LandAccess).allowed);
    s.month = 2;
    assert_eq!(
        laws::evaluate(&w, &s, PERSON, Action::LandAccess).reasons,
        vec![Reason::StatePolicy {
            policy: PAUSE_ADMISSIONS
        }]
    );
    s.month = 3;
    let a = g::authority(&w, &s).unwrap();
    assert_eq!(a.governor, Some(SECOND_PERSON));
    assert_eq!(a.election.unwrap().votes[&SECOND_PERSON], 2);
    let rejected = w.clone();
    assert!(g::schedule(&mut w, &s, change(4, PERSON, OPEN)).is_err());
    assert_eq!(w, rejected);
    g::schedule(&mut w, &s, change(4, SECOND_PERSON, OPEN)).unwrap();
    assert!(!laws::evaluate(&w, &s, PERSON, Action::LandAccess).allowed);
    s.month = 4;
    assert!(laws::evaluate(&w, &s, PERSON, Action::LandAccess).allowed);
    let a = g::authority(&w, &s).unwrap();
    assert_eq!(a.effective_since, 4);
    assert_eq!(a.instruction.unwrap().issued_month, 3);
    assert_eq!(w.state_governance.as_ref().unwrap().charter, charter);
    assert_eq!(
        w.state_governance.as_ref().unwrap().constitution,
        constitution
    );
    g::validate(&w, &s).unwrap();
}

#[test]
fn authority_dates_constitution_and_duplicate_instructions_are_enforced_atomically() {
    let (mut w, s) = g::scenario::pair().unwrap();
    for c in [
        change(2, SECOND_PERSON, OPEN),
        change(1, PERSON, OPEN),
        change(0, PERSON, OPEN),
        change(2, PERSON, 999),
        change(2, STATE_AGENT, OPEN),
    ] {
        let before = w.clone();
        assert!(g::schedule(&mut w, &s, c).is_err());
        assert_eq!(w, before);
    }
    g::schedule(&mut w, &s, change(2, PERSON, PAUSE_ADMISSIONS)).unwrap();
    let before = w.clone();
    assert!(g::schedule(&mut w, &s, change(2, PERSON, OPEN)).is_err());
    assert_eq!(w, before);
    // Checkpoint validation catches bypasses of the acceptance API too.
    w.state_governance.as_mut().unwrap().changes[0]
        .change
        .authorized_by = SECOND_PERSON;
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
}

#[test]
fn fixed_rotating_and_elected_offices_keep_authority_distinct_from_citizenship() {
    for leadership in [
        Leadership::FixedFounder,
        Leadership::Rotating,
        Leadership::Elected,
    ] {
        let (mut w, mut s) = g::scenario::pair().unwrap();
        w.state_governance.as_mut().unwrap().constitution.leadership = leadership;
        if leadership != Leadership::Elected {
            w.state_governance.as_mut().unwrap().ballots.clear();
        }
        s.month = 3;
        g::validate(&w, &s).unwrap();
        assert_eq!(
            g::authority(&w, &s).unwrap().governor,
            Some(if leadership == Leadership::FixedFounder {
                PERSON
            } else {
                SECOND_PERSON
            })
        );
        assert!(laws::evaluate(&w, &s, PERSON, Action::Process(GROW)).allowed);
        assert!(!laws::evaluate(&w, &s, STATE_AGENT, Action::Process(GROW)).allowed);
    }
}

#[test]
fn vacancies_keep_law_but_allow_no_policy_instruction() {
    for tie in [false, true] {
        let (mut w, mut s) = g::scenario::pair().unwrap();
        w.state_governance.as_mut().unwrap().ballots.clear();
        w.state_governance
            .as_mut()
            .unwrap()
            .charter
            .election
            .tie_break = ElectionTieBreak::Vacant;
        g::schedule(&mut w, &s, change(2, PERSON, PAUSE_ADMISSIONS)).unwrap();
        if tie {
            for voter in [PERSON, SECOND_PERSON] {
                g::cast(
                    &mut w,
                    &s,
                    Ballot {
                        term_start: 3,
                        voter,
                        candidate: Some(voter),
                    },
                )
                .unwrap();
            }
        }
        s.month = 3;
        let a = g::authority(&w, &s).unwrap();
        assert_eq!(a.governor, None);
        assert_eq!(a.policy, PAUSE_ADMISSIONS);
        assert!(g::schedule(&mut w, &s, change(4, PERSON, OPEN)).is_err());
        assert!(g::schedule(&mut w, &s, change(4, SECOND_PERSON, OPEN)).is_err());
        g::validate(&w, &s).unwrap();
    }
}

#[test]
fn ballots_reject_non_citizens_non_persons_dead_participants_and_bad_dates() {
    let (mut w, mut s) = g::scenario::pair().unwrap();
    for ballot in [
        Ballot {
            term_start: 3,
            voter: PERSON,
            candidate: Some(SECOND_PERSON),
        }, // duplicate
        Ballot {
            term_start: 2,
            voter: PERSON,
            candidate: Some(SECOND_PERSON),
        },
        Ballot {
            term_start: 1,
            voter: PERSON,
            candidate: Some(SECOND_PERSON),
        },
        Ballot {
            term_start: 5,
            voter: STATE_AGENT,
            candidate: Some(PERSON),
        },
        Ballot {
            term_start: 5,
            voter: PERSON,
            candidate: Some(STATE_AGENT),
        },
    ] {
        let before = w.clone();
        assert!(g::cast(&mut w, &s, ballot).is_err());
        assert_eq!(w, before);
    }
    w.state_governance.as_mut().unwrap().ballots.clear();
    s.memberships.remove(&(SECOND_PERSON, STATE_AGENT, CITIZEN));
    assert!(
        g::cast(
            &mut w,
            &s,
            Ballot {
                term_start: 3,
                voter: SECOND_PERSON,
                candidate: Some(PERSON)
            }
        )
        .is_err()
    );
    death(&mut s, PERSON, 1);
    assert!(
        g::cast(
            &mut w,
            &s,
            Ballot {
                term_start: 3,
                voter: PERSON,
                candidate: Some(PERSON)
            }
        )
        .is_err()
    );
    assert!(g::schedule(&mut w, &s, change(2, PERSON, OPEN)).is_err());
}

#[test]
fn new_citizens_and_later_deaths_do_not_rewrite_elections_or_accepted_instructions() {
    let (mut w, mut s) = g::scenario::pair().unwrap();
    s.month = 3;
    let opening = g::authority(&w, &s).unwrap().election.unwrap();
    g::schedule(&mut w, &s, change(4, SECOND_PERSON, PAUSE_ADMISSIONS)).unwrap();
    let entrant = 900;
    w.agents.push(Agent {
        id: entrant,
        name: "new citizen".into(),
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .agent_types
        .insert(entrant, PERSON_TYPE);
    s.memberships.insert(
        (entrant, STATE_AGENT, CITIZEN),
        membership::Agreement {
            member: entrant,
            organization: STATE_AGENT,
            role: CITIZEN,
            source_offer: 1,
            accepted_month: 3,
        },
    );
    assert_eq!(
        g::authority(&w, &s).unwrap().election,
        Some(opening.clone())
    );
    g::cast(
        &mut w,
        &s,
        Ballot {
            term_start: 5,
            voter: entrant,
            candidate: Some(entrant),
        },
    )
    .unwrap();
    death(&mut s, SECOND_PERSON, 3);
    assert_eq!(g::authority(&w, &s).unwrap().governor, None);
    s.month = 4;
    let a = g::authority(&w, &s).unwrap();
    assert_eq!(a.election, Some(opening));
    assert_eq!(a.governor, None);
    assert_eq!(a.policy, PAUSE_ADMISSIONS);
    g::validate(&w, &s).unwrap();
    s.month = 5;
    assert_eq!(g::authority(&w, &s).unwrap().governor, Some(entrant));
    g::validate(&w, &s).unwrap();
}

#[test]
fn governor_cannot_override_base_law_or_create_permissions() {
    let (mut w, mut s) = g::scenario::pair().unwrap();
    w.transaction_policy
        .as_mut()
        .unwrap()
        .laws
        .push(laws::Rule {
            id: 1,
            name: "fixed prohibition".into(),
            agent_type: Some(PERSON_TYPE),
            action: Action::FoundHousehold,
            requirement: laws::Requirement::Prohibited,
        });
    g::schedule(&mut w, &s, change(2, PERSON, OPEN)).unwrap();
    s.month = 2;
    assert_eq!(
        laws::evaluate(&w, &s, PERSON, Action::FoundHousehold).reasons,
        vec![Reason::Prohibited { rule: 1 }]
    );
    assert!(!laws::evaluate(&w, &s, PERSON, Action::Borrow).allowed);
    assert!(!laws::evaluate(&w, &s, STATE_AGENT, Action::LandAccess).allowed);
}

#[test]
fn malformed_state_charters_and_unsupported_policy_definitions_fail_closed() {
    let (w, s) = g::scenario::pair().unwrap();
    for case in 0..8 {
        let mut w = w.clone();
        let g = w.state_governance.as_mut().unwrap();
        match case {
            0 => g.charter.term_months = 0,
            1 => g.charter.founder = STATE_AGENT,
            2 => g.charter.initial_policy = 999,
            3 => g.charter.election.minimum_turnout_percent = 101,
            4 => g.formed = 0,
            5 => g.state = PERSON,
            6 => {
                g.constitution
                    .policies
                    .get_mut(&OPEN)
                    .unwrap()
                    .prohibited
                    .insert((None, Action::Process(999)));
            }
            _ => {
                w.transaction_policy = None;
            }
        }
        assert!(
            Simulation::new(w, s.clone(), Backend::Reference).is_err(),
            "case {case}"
        );
    }
}

#[test]
fn farm_rights_harvests_and_taxes_continue_when_new_admissions_close() {
    let (w, s) = g::scenario::pair().unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.accepted_agreements.len(), 2);
    let mut control = sim.clone();
    g::schedule(
        &mut sim.world,
        &sim.state,
        change(3, PERSON, PAUSE_ADMISSIONS),
    )
    .unwrap();
    sim.run_months(24).unwrap();
    control.run_months(24).unwrap();
    assert_eq!(sim.state, control.state);
    assert_eq!(sim.reports, control.reports);
    assert_eq!(sim.ledger, control.ledger);
    assert!(sim.state.terminal.is_empty());
    assert!(
        sim.state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Completed)
            .count()
            >= 4
    );
    assert!(!sim.state.obligations.is_empty());
    assert!(sim.state.obligations.values().all(|o| o.paid == o.owed));
    println!(
        "continued farms: agreements={}, harvests={}, dues_paid={}",
        sim.state.accepted_agreements.len(),
        sim.state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Completed)
            .count(),
        sim.state.obligations.values().map(|o| o.paid).sum::<i32>()
    );
}

#[test]
fn land_admission_reopens_after_election_on_cpu_and_checkpoint() {
    let (mut w, mut s) = g::scenario::pair().unwrap();
    w.state_governance.as_mut().unwrap().charter.initial_policy = PAUSE_ADMISSIONS;
    // Three extra food units cover the three-month admission pause; no aid is
    // created by governance. The unprovisioned control below fails to start.
    for person in [PERSON, SECOND_PERSON] {
        *s.balances.get_mut(&(person, GRAIN)).unwrap() += 3;
    }
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    reference.run_months(2).unwrap();
    assert!(reference.state.accepted_agreements.is_empty());
    g::schedule(
        &mut reference.world,
        &reference.state,
        change(4, SECOND_PERSON, OPEN),
    )
    .unwrap();
    reference.run_months(1).unwrap();
    assert!(reference.state.accepted_agreements.is_empty());
    reference.run_months(3).unwrap();
    assert!(!reference.state.accepted_agreements.is_empty());
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    cpu.run_months(2).unwrap();
    g::schedule(&mut cpu.world, &cpu.state, change(4, SECOND_PERSON, OPEN)).unwrap();
    for _ in 0..4 {
        cpu.step().unwrap();
        // Reconstruct mid-month with the pending instruction, then resume.
        let mut resumed =
            Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu).unwrap();
        resumed.ledger = cpu.ledger;
        resumed.reports = cpu.reports;
        cpu = resumed;
        cpu.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    assert_eq!(cpu.world, reference.world);
}

fn household_pair() -> (World, State) {
    let (mut w, mut s) = household::scenario::pair().unwrap();
    let (state_world, state_state) = g::scenario::pair().unwrap();
    w.state_governance = state_world.state_governance;
    s.memberships = state_state.memberships;
    w.transaction_policy.as_mut().unwrap().membership_offers =
        state_world.transaction_policy.unwrap().membership_offers;
    (w, s)
}

#[test]
fn state_and_household_governors_operate_independently_and_observer_is_read_only() {
    let (mut w, s) = household_pair();
    g::schedule(&mut w, &s, change(2, PERSON, PAUSE_ADMISSIONS)).unwrap();
    let constitution = w.households[0].governance.constitution.clone();
    let mut observed = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut plain = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut audit = Audit::with_opening(
        &observed.world,
        &observed.state,
        TOKEN,
        Opening {
            inventory: observed
                .state
                .balances
                .iter()
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            processes: Some(Costs {
                beneficiary_policy: Some(BeneficiaryPolicy::TransferAtCost),
                output_weights: [(
                    household::scenario::FOOD_PROCESS,
                    [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                )]
                .into(),
                ..Costs::default()
            }),
            ..Opening::default()
        },
    )
    .unwrap();
    let mut observer = Observer::new(
        vec![],
        "state-household",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    for _ in 0..6 {
        if observed.state.month == 3 {
            for sim in [&mut observed, &mut plain] {
                g::schedule(&mut sim.world, &sim.state, change(4, SECOND_PERSON, OPEN)).unwrap();
                household::scenario::incoming_policy(&mut sim.world, &sim.state).unwrap();
            }
        }
        let end = observed.state.month + 1;
        while observed.state.month < end {
            observer.step_audited(&mut observed, &mut audit).unwrap();
        }
        plain.run_months(1).unwrap();
    }
    assert_eq!(observed.state, plain.state);
    assert_eq!(observed.ledger, plain.ledger);
    assert_eq!(observed.reports, plain.reports);
    assert_eq!(
        observed.world.households[0].governance.constitution,
        constitution
    );
    assert!(observed.state.processes.values().any(|p| p.definition
        == household::scenario::FOOD_PROCESS
        && p.status == Status::Completed));
    for agent in &observed.world.agents {
        let statement = audit.book().statements(agent.id, 1, 6).unwrap();
        assert_eq!(statement.assets, statement.liabilities + statement.equity);
    }
    let text = String::from_utf8(observer.finish().unwrap()).unwrap();
    let rows: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let state_rows: Vec<_> = rows
        .iter()
        .filter(|r| r["kind"] == "state_governance")
        .collect();
    assert_eq!(state_rows.len(), 6);
    assert_eq!(state_rows[1]["policy"], PAUSE_ADMISSIONS);
    assert_eq!(state_rows[2]["governor"], SECOND_PERSON);
    assert_eq!(state_rows[3]["policy"], OPEN);
    assert_eq!(state_rows[3]["authorized_by"], SECOND_PERSON);
    assert!(rows.iter().any(|r| r["kind"] == "household_governance"));
}

#[test]
fn closing_household_admission_blocks_new_formation_but_not_existing_households() {
    let (mut w, mut s) = household_pair();
    let mut new_home = w.households[0].clone();
    // Separate counterfactual: same adults have not yet founded their household.
    w.households.clear();
    w.agents.retain(|a| a.id != HOME);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .agent_types
        .remove(&HOME);
    new_home.governance.ballots.clear();
    g::schedule(&mut w, &s, change(2, PERSON, PAUSE_ADMISSIONS)).unwrap();
    s.month = 2;
    new_home.formed = 2;
    let before = w.clone();
    assert!(households::form(&mut w, &s, new_home.clone()).is_err());
    assert_eq!(w, before);
    s.month = 3;
    g::schedule(&mut w, &s, change(4, SECOND_PERSON, OPEN)).unwrap();
    s.month = 4;
    new_home.formed = 4;
    households::form(&mut w, &s, new_home).unwrap();
    assert_eq!(w.households[0].admission.as_ref().unwrap().month, 4);
    settlement::validate_world(&w, &s).unwrap();
    assert!(opportunities::permits(
        &w,
        &s,
        PERSON,
        Action::FoundHousehold
    ));
}

#[test]
fn reopening_without_food_cover_does_not_guarantee_a_viable_new_commitment() {
    let (mut w, s) = g::scenario::pair().unwrap();
    w.state_governance.as_mut().unwrap().charter.initial_policy = PAUSE_ADMISSIONS;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    g::schedule(&mut sim.world, &sim.state, change(4, SECOND_PERSON, OPEN)).unwrap();
    sim.run_months(1).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    assert!(laws::evaluate(&sim.world, &sim.state, PERSON, Action::LandAccess).allowed);
    // Inspect the same independent search used by competitive land admission.
    let mut local = sim.clone();
    local.world.competition = None;
    local.world.participants.retain(|p| p.agent == PERSON);
    local.world.condition_rules.retain(|r| r.subject == PERSON);
    local.state.conditions.retain(|(a, _), _| *a == PERSON);
    local.state.processes.retain(|_, p| p.operator == PERSON);
    let mut batch = Batch::empty(&local.state);
    economics_compute_smoke::planning::choose_with_search(
        &local,
        &mut batch,
        &economics_compute_smoke::search::NeedDirectedOpportunitySearch,
        Default::default(),
    )
    .unwrap();
    let reasons = &batch.decision.as_ref().unwrap().rejection_reasons;
    println!(
        "unprovisioned reopening: {} rejected plans (failed production or unpaid land dues)",
        reasons.len()
    );
    assert!(
        reasons
            .iter()
            .any(|r| r.contains("production commitment is forecast to fail"))
    );
    sim.run_months(3).unwrap();
    assert!(sim.state.accepted_agreements.is_empty());
}

#[test]
fn state_office_does_not_confer_household_office_or_the_reverse() {
    let (mut w, mut s) = household_pair();
    let state = w.state_governance.as_mut().unwrap();
    state.constitution.leadership = Leadership::FixedFounder;
    state.ballots.clear();
    s.month = 3;
    assert_eq!(g::authority(&w, &s).unwrap().governor, Some(PERSON));
    assert_eq!(
        household::authority(&w.households[0], &s).leader,
        Some(SECOND_PERSON)
    );
    let before = w.clone();
    assert!(g::schedule(&mut w, &s, change(4, SECOND_PERSON, PAUSE_ADMISSIONS)).is_err());
    assert!(
        household::schedule(
            &mut w,
            &s,
            HOME,
            household::PolicyChange {
                month: 4,
                authorized_by: PERSON,
                policy: household::Policy::NeedsFirst
            }
        )
        .is_err()
    );
    assert_eq!(w, before);
    g::schedule(&mut w, &s, change(4, PERSON, PAUSE_ADMISSIONS)).unwrap();
    household::scenario::incoming_policy(&mut w, &s).unwrap();
    settlement::validate_world(&w, &s).unwrap();
}

#[test]
fn incoming_governor_can_supersede_a_pending_policy_without_erasing_history() {
    let (mut w, mut s) = g::scenario::pair().unwrap();
    g::schedule(&mut w, &s, change(4, PERSON, PAUSE_ADMISSIONS)).unwrap();
    s.month = 3;
    g::schedule(&mut w, &s, change(4, SECOND_PERSON, OPEN)).unwrap();
    assert_eq!(g::authority(&w, &s).unwrap().policy, OPEN);
    s.month = 4;
    let a = g::authority(&w, &s).unwrap();
    assert_eq!(a.policy, OPEN);
    assert_eq!(a.instruction.unwrap().change.authorized_by, SECOND_PERSON);
    assert_eq!(w.state_governance.as_ref().unwrap().changes.len(), 2);
    // Catalog order cannot change effective precedence.
    w.state_governance.as_mut().unwrap().changes.reverse();
    assert_eq!(g::authority(&w, &s).unwrap().policy, OPEN);
    g::validate(&w, &s).unwrap();
}
