use economics_compute_smoke::{
    agency, compute::Backend, model::*, scenario::*, simulation::Simulation, state_governance,
};

fn mint_audit(w: &World, s: &State) -> economics_compute_smoke::financial_reporting::Audit {
    use economics_compute_smoke::{
        financial_reporting::{Audit, Opening},
        issuance_accounting,
        minting::COIN,
    };
    Audit::with_opening(
        w,
        s,
        COIN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| {
                    *r != COIN
                        && **q > 0
                        && w.resources
                            .iter()
                            .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
                })
                .map(|(k, q)| (*k, i128::from(*q)))
                .collect(),
            processes: Some(Default::default()),
            issuance: Some(issuance_accounting::Policy::NonRedeemableEquity),
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn state_opens_land_admissions_to_meet_citizen_needs() {
    let (w, s) = agency::scenario::farming().unwrap();
    let mut control = w.clone();
    control.agency.clear();
    let mut control = Simulation::new(control, s.clone(), Backend::Reference).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(8).unwrap();
    control.run_months(8).unwrap();
    assert!(
        sim.world.agency[&STATE_AGENT]
            .history
            .iter()
            .any(|d| d.chosen == Some(0))
    );
    assert_eq!(sim.state.accepted_agreements.len(), 2);
    assert!(control.state.accepted_agreements.is_empty());
    assert!(
        sim.state
            .processes
            .values()
            .any(|p| p.definition == GROW && p.status == Status::Completed)
    );
    assert!(sim.state.terminal.is_empty());
    assert!(
        state_governance::authority(&sim.world, &sim.state)
            .unwrap()
            .governor
            .is_some()
    );
}

#[test]
fn state_funds_physical_minting_toward_a_finite_reserve() {
    let (w, s) = agency::scenario::minting().unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(6).unwrap();
    assert_eq!(
        sim.state.balance(
            economics_compute_smoke::minting::ISSUER,
            economics_compute_smoke::minting::COIN
        ),
        14
    );
    let history = &sim.world.agency[&economics_compute_smoke::minting::ISSUER].history;
    assert_eq!(history.iter().filter(|d| d.chosen.is_some()).count(), 1);
    assert!(history.last().unwrap().chosen.is_none());
}

#[test]
fn same_controller_selects_household_priorities() {
    let (w, s) = agency::scenario::household().unwrap();
    let mut fixed = w.clone();
    fixed.agency.clear();
    let mut control = Simulation::new(fixed, s.clone(), Backend::Reference).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(4).unwrap();
    control.run_months(4).unwrap();
    let history = &sim.world.agency
        [&economics_compute_smoke::household_governance::scenario::HOUSEHOLD]
        .history;
    assert!(history.iter().any(|d| d.chosen == Some(1)));
    let deficit = |s: &Simulation| s.reports.iter().map(|r| r.deficit(NUTRITION)).sum::<i32>();
    assert!(deficit(&sim) < deficit(&control));
}

#[test]
fn simultaneous_household_and_state_decisions_preserve_separate_accounting() {
    use economics_compute_smoke::{
        financial_reporting::{Audit, Opening},
        household_governance::scenario::FOOD_PROCESS,
        process_accounting::{BeneficiaryPolicy, Costs, Output},
    };
    let (w, s) = agency::scenario::household().unwrap();
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
                    FOOD_PROCESS,
                    [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                )]
                .into(),
                ..Default::default()
            }),
            ..Default::default()
        },
    )
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while sim.state.month <= 4 {
        audit.step(&mut sim).unwrap();
    }
    for (agent, c) in &sim.world.agency {
        assert_eq!(c.history.len(), 4);
        let statement = audit.book().statements(*agent, 1, 4).unwrap();
        assert_eq!(statement.assets, statement.liabilities + statement.equity);
    }
}

#[test]
fn mint_decisions_reconcile_and_resume_identically_on_cpu() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let (w, s) = agency::scenario::minting().unwrap();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut a = mint_audit(&reference.world, &reference.state);
    let mut b = a.clone();
    let mut observer = Observer::new(
        vec![],
        "agency",
        Config {
            settlement: true,
            ..Default::default()
        },
    )
    .unwrap();
    while reference.state.month <= 6 {
        a.step(&mut reference).unwrap();
    }
    while cpu.state.month <= 6 {
        observer.step_audited(&mut cpu, &mut b).unwrap();
        let mut resumed =
            Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu).unwrap();
        resumed.ledger = cpu.ledger;
        resumed.reports = cpu.reports;
        cpu = resumed;
    }
    assert_eq!(reference.world, cpu.world);
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.ledger, cpu.ledger);
    assert_eq!(a, b);
    assert!(cpu.world.governance_observation.is_none());
    for agent in &cpu.world.agents {
        let statement = b.book().statements(agent.id, 1, 6).unwrap();
        assert_eq!(statement.assets, statement.liabilities + statement.equity);
    }
    let logs = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert_eq!(
        logs.lines()
            .filter(|l| l.contains("organization_decision"))
            .count(),
        6
    );
}

#[test]
fn absent_inputs_or_legal_permission_cannot_be_replaced_by_an_objective() {
    use economics_compute_smoke::{
        minting::*,
        opportunities::{Action, STATE_TYPE},
    };
    for prohibited in [false, true] {
        let (mut w, mut s) = agency::scenario::minting().unwrap();
        if prohibited {
            w.transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(STATE_TYPE, Action::Process(MINT)));
        } else {
            s.balances.insert((SUPPLIER, METAL), 0);
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(4).unwrap();
        assert!(sim.state.balance(ISSUER, COIN) < 14);
        assert!(
            sim.world.agency[&ISSUER]
                .history
                .iter()
                .all(|d| d.chosen.is_none())
        );
        assert!(sim.state.balances.values().all(|q| *q >= 0));
        if prohibited {
            assert!(
                sim.world.agency[&ISSUER].history[0]
                    .alternatives
                    .iter()
                    .any(|a| a.failure.as_deref() == Some("organization process is prohibited"))
            );
        }
    }
}

#[test]
fn elections_do_not_depend_on_policy_review_frequency_and_do_not_invent_votes() {
    use economics_compute_smoke::competition::SECOND_PERSON;
    let (mut w, s) = agency::scenario::farming().unwrap();
    w.agency.get_mut(&STATE_AGENT).unwrap().config.review_every = 4;
    w.agency
        .get_mut(&STATE_AGENT)
        .unwrap()
        .config
        .preferences
        .remove(&SECOND_PERSON);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    let decision = &sim.world.agency[&STATE_AGENT].history[1];
    assert_eq!(decision.ballots.len(), 1);
    assert_eq!(decision.ballots[0].voter, PERSON);
    assert_eq!(
        decision.reason,
        "election observation; policy review not due"
    );
    assert_eq!(
        sim.world.state_governance.as_ref().unwrap().ballots.len(),
        1
    );
}

#[test]
fn household_elections_use_the_same_supplied_preference_adapter() {
    use economics_compute_smoke::household_governance::{self as h, scenario::HOUSEHOLD};
    let (mut w, s) = agency::scenario::household().unwrap();
    w.households[0].governance.ballots.clear();
    let c = &mut w.agency.get_mut(&HOUSEHOLD).unwrap().config;
    for person in [PERSON, PERSON + 1] {
        c.preferences.insert(person, c.objectives.clone());
    }
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(4).unwrap();
    assert_eq!(sim.world.households[0].governance.ballots.len(), 4);
    assert_eq!(
        h::authority(&sim.world.households[0], &sim.state).leader,
        Some(PERSON)
    );
    assert_eq!(
        sim.world.households[0].governance.policy(sim.state.month),
        h::Policy::NeedsFirst
    );
}

#[test]
fn frozen_political_history_survives_omitted_peer_economics() {
    use economics_compute_smoke::{
        competition::SECOND_PERSON, forecast::ForecastContext, maintenance::TerminalTransition,
    };
    let (w, mut s) = state_governance::scenario::pair().unwrap();
    s.month = 4;
    s.terminal.insert(
        SECOND_PERSON,
        TerminalTransition {
            month: 3,
            subject: SECOND_PERSON,
            reason: NUTRITION,
            state: "dead".into(),
        },
    );
    let expected = state_governance::authority(&w, &s);
    assert_eq!(expected.as_ref().unwrap().governor, None);
    let (f, mut local) = ForecastContext::new(&w, &s).into_parts();
    local.memberships.retain(|(id, _, _), _| *id == PERSON);
    local.terminal.clear();
    assert_eq!(state_governance::authority(&f, &local), expected);
    state_governance::validate(&f, &local).unwrap();
    assert!(w.governance_observation.is_none());
}

#[test]
fn failed_open_rolls_back_decisions_and_static_mandate_cannot_be_rewritten() {
    let (w, s) = agency::scenario::household().unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.effect_limit = 0;
    let before = sim.clone();
    assert!(sim.step().is_err());
    assert_eq!(sim.world, before.world);
    assert_eq!(sim.state, before.state);
    assert_eq!(sim.ledger, before.ledger);
    sim.effect_limit = 4096;
    sim.step().unwrap();
    let home = economics_compute_smoke::household_governance::scenario::HOUSEHOLD;
    let original = sim.world.agency[&home].history[0].accepted.clone();
    sim.world
        .agency
        .get_mut(&home)
        .unwrap()
        .config
        .programs
        .get_mut(&1)
        .unwrap()
        .name = "rewritten".into();
    assert_eq!(sim.world.agency[&home].history[0].accepted, original);
    assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
}

#[test]
fn malformed_mandates_and_forged_decision_authority_fail_closed() {
    let (w, s) = agency::scenario::minting().unwrap();
    for case in 0..5 {
        let mut w = w.clone();
        let c = &mut w.agency.get_mut(&STATE_AGENT).unwrap().config;
        match case {
            0 => c.horizon = 0,
            1 => c.review_every = 0,
            2 => c.emergency = Some((99, 0)),
            3 => {
                c.programs.get_mut(&1).unwrap().commands = vec![agency::Command::StatePolicy(99)];
            }
            _ => {
                c.preferences.insert(STATE_AGENT, c.objectives.clone());
            }
        }
        assert!(
            Simulation::new(w, s.clone(), Backend::Reference).is_err(),
            "case {case}"
        );
    }
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    sim.world.agency.get_mut(&STATE_AGENT).unwrap().history[1].authorized_by = Some(STATE_AGENT);
    assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
}

#[test]
fn state_work_targets_use_paid_individual_hours_with_separate_books() {
    use economics_compute_smoke::minting::*;
    let (w, s) = agency::scenario::public_work().unwrap();
    let mut unfunded = w.clone();
    unfunded.employment.clear();
    let mut control = Simulation::new(unfunded, s.clone(), Backend::Reference).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut audit = mint_audit(&sim.world, &sim.state);
    let opening_worker_cash = sim.state.balance(WORKER, COIN);
    while sim.state.month <= 4 {
        audit.step(&mut sim).unwrap();
    }
    control.run_months(4).unwrap();
    assert_eq!(sim.world.agency[&ISSUER].history[0].chosen, Some(1));
    assert_eq!(sim.state.balance(ISSUER, FIREWOOD), 2);
    assert_eq!(sim.state.balance(ISSUER, COIN), 0);
    assert_eq!(sim.state.balance(WORKER, COIN), opening_worker_cash + 4);
    assert_eq!(
        sim.world
            .participants
            .iter()
            .find(|p| p.agent == ISSUER)
            .unwrap()
            .capacity
            .quantity,
        0
    );
    assert!(
        sim.state
            .employment
            .earned
            .values()
            .all(|e| e.claim.outstanding() == 0)
    );
    assert_eq!(control.state.balance(ISSUER, FIREWOOD), 0);
    assert!(
        control.world.agency[&ISSUER]
            .history
            .iter()
            .all(|d| d.chosen.is_none())
    );
    for agent in &sim.world.agents {
        let statement = audit.book().statements(agent.id, 1, 4).unwrap();
        assert_eq!(statement.assets, statement.liabilities + statement.equity);
    }
}

#[test]
fn vacancy_keeps_accepted_work_and_waits_without_fabricating_authority() {
    let (mut w, s) = agency::scenario::public_work().unwrap();
    w.state_governance.as_mut().unwrap().constitution.leadership =
        economics_compute_smoke::governance::Leadership::Elected;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(4).unwrap();
    assert_eq!(
        sim.world.agency[&STATE_AGENT].history[2].reason,
        "vacant office"
    );
    assert_eq!(
        sim.state
            .balance(STATE_AGENT, economics_compute_smoke::minting::FIREWOOD),
        2
    );
    assert!(
        sim.world.agency[&STATE_AGENT].history[2..]
            .iter()
            .all(|d| d.chosen.is_none())
    );
}

#[test]
fn membership_need_reopens_a_posted_offer_but_person_still_accepts_independently() {
    use economics_compute_smoke::{
        agency::objectives::{Metric, Objective, Scope},
        competition::SECOND_PERSON,
        membership::CITIZEN,
        opportunities::{Action, PERSON_TYPE},
    };
    let (mut w, mut s) = agency::scenario::farming().unwrap();
    s.memberships.remove(&(SECOND_PERSON, STATE_AGENT, CITIZEN));
    let g = w.state_governance.as_mut().unwrap();
    g.constitution.leadership = economics_compute_smoke::governance::Leadership::FixedFounder;
    g.constitution
        .policies
        .get_mut(&state_governance::scenario::PAUSE_ADMISSIONS)
        .unwrap()
        .prohibited = [(Some(PERSON_TYPE), Action::Membership)].into();
    let c = &mut w.agency.get_mut(&STATE_AGENT).unwrap().config;
    c.objectives = vec![Objective {
        scope: Scope::Members,
        metric: Metric::MembershipShortfall(2),
    }];
    c.preferences.clear();
    c.horizon = 3;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.world.agency[&STATE_AGENT].history[0].chosen, Some(0));
    assert!(
        !sim.state
            .memberships
            .contains_key(&(SECOND_PERSON, STATE_AGENT, CITIZEN))
    );
    sim.run_months(3).unwrap();
    assert!(
        sim.state
            .memberships
            .contains_key(&(SECOND_PERSON, STATE_AGENT, CITIZEN))
    );
    assert!(
        sim.state
            .accepted_agreements
            .values()
            .any(|a| a.debtor == SECOND_PERSON)
    );
}
