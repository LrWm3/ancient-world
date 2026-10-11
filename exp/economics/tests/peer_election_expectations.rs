use economics_compute_smoke::{
    acceptance::{Run, WORKSHOP, continuation as c, governed as g},
    agency::expectations::{Policy, Snapshot},
    compute::Backend,
    forecast::ForecastContext,
    governance::Ballot,
    minting::ISSUER,
    model::*,
    simulation::Simulation,
};
const POLICY: Policy = Policy::RecentSuccessfulStartsAndBallots { memory_months: 6 };
fn observed() -> (World, State, Snapshot) {
    let (w, s) = g::scenario(c::Case::Baseline).unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(6).unwrap();
    let snapshot = Snapshot::observe(&run.sim.world, &run.sim.state, WORKSHOP, POLICY).unwrap();
    (run.sim.world, run.sim.state, snapshot)
}
#[test]
fn frozen_individual_ballots_support_future_terms_through_ordinary_tally() {
    let (w, s, snapshot) = observed();
    assert!(!snapshot.ballots.is_empty());
    assert!(
        snapshot
            .ballots
            .iter()
            .all(|b| b.term_start == 7 && b.issued_month == 6)
    );
    assert!(
        Snapshot::observe(&w, &s, ISSUER, POLICY)
            .unwrap()
            .ballots
            .is_empty()
    );
    let (world, state) = ForecastContext::new(&w, &s).into_parts();
    let mut sim = Simulation::new(world, state, Backend::Reference).unwrap();
    let mut votes = vec![];
    let mut starts = vec![];
    while sim.state.month < 11 {
        votes.extend(snapshot.apply_ballots(&mut sim.world, &sim.state).unwrap());
        starts.extend(
            snapshot
                .apply_hypotheses(&mut sim.world, &sim.state)
                .unwrap(),
        );
        sim.step().unwrap();
    }
    assert!(
        votes
            .iter()
            .all(|v| v.accepted && v.month > 7 && v.term_start == v.month + 1)
    );
    assert_eq!(votes.len(), snapshot.ballots.len() * 2);
    assert!(starts.iter().any(|a| a.month == 9));
    assert!(starts.iter().any(|a| a.month == 10));
    assert!(
        starts
            .iter()
            .all(|a| sim.state.processes.values().any(|p| p.start == a.month
                && p.operator == a.agent
                && p.definition == a.definition
                && p.status == Status::Completed))
    );
    assert!(
        w.state_governance
            .as_ref()
            .unwrap()
            .ballots
            .iter()
            .all(|b| b.issued_month <= 6)
    );
    assert_eq!(
        snapshot,
        Snapshot::observe(&w, &s, WORKSHOP, POLICY).unwrap()
    );
}

#[test]
fn existing_votes_abstentions_and_expiry_do_not_create_a_guaranteed_governor() {
    let (mut w, s, _) = observed();
    // The latest cohort abstained; earlier votes must not be borrowed.
    for b in &mut w.state_governance.as_mut().unwrap().ballots {
        if b.ballot.term_start == 7 {
            b.ballot.candidate = None;
        }
    }
    let snapshot = Snapshot::observe(&w, &s, WORKSHOP, POLICY).unwrap();
    assert!(snapshot.ballots.iter().all(|b| b.candidate.is_none()));
    let (mut w, mut s) = ForecastContext::new(&w, &s).into_parts();
    s.month = 8;
    let voter = snapshot.ballots[0].voter;
    economics_compute_smoke::state_governance::cast(
        &mut w,
        &s,
        Ballot {
            term_start: 9,
            voter,
            candidate: Some(voter),
        },
    )
    .unwrap();
    let accepted = w.state_governance.as_ref().unwrap().ballots.clone();
    let attempts = snapshot.apply_ballots(&mut w, &s).unwrap();
    assert!(
        attempts
            .iter()
            .all(|a| a.voter != voter && a.candidate.is_none())
    );
    assert!(snapshot.apply_ballots(&mut w, &s).unwrap().is_empty());
    assert!(
        accepted
            .iter()
            .all(|b| w.state_governance.as_ref().unwrap().ballots.contains(b))
    );
    s.month = 14;
    assert!(snapshot.apply_ballots(&mut w, &s).unwrap().is_empty());
    let mut absent = w.clone();
    absent.state_governance.as_mut().unwrap().ballots.clear();
    assert!(
        Snapshot::observe(&absent, &s, WORKSHOP, POLICY)
            .unwrap()
            .ballots
            .is_empty()
    );
}

#[test]
fn abstention_insufficient_turnout_and_ineligible_choices_leave_actual_vacancies() {
    let (w, s, _) = observed();
    for control in ["abstain", "turnout", "dead", "ineligible", "current open"] {
        let mut source = w.clone();
        let current = &mut source.state_governance.as_mut().unwrap().ballots;
        if control == "abstain" {
            for b in current.iter_mut().filter(|b| b.ballot.term_start == 7) {
                b.ballot.candidate = None;
            }
        }
        if control == "turnout" {
            let voter = current
                .iter()
                .find(|b| b.ballot.term_start == 7)
                .unwrap()
                .ballot
                .voter;
            current.retain(|b| b.ballot.term_start != 7 || b.ballot.voter == voter);
            source
                .state_governance
                .as_mut()
                .unwrap()
                .charter
                .election
                .minimum_turnout_percent = 100;
        }
        let (mut w, mut s) = ForecastContext::new(&source, &s).into_parts();
        let mut snapshot = Snapshot::observe(&source, &s, WORKSHOP, POLICY).unwrap();
        s.month = 8;
        if control == "current open" {
            snapshot = Snapshot::observe(&source, &s, WORKSHOP, POLICY).unwrap();
        }
        if control == "dead" {
            let candidate = snapshot.ballots[0].candidate.unwrap();
            s.terminal.insert(
                candidate,
                economics_compute_smoke::maintenance::TerminalTransition {
                    month: 8,
                    subject: candidate,
                    reason: 0,
                    state: "test death".into(),
                },
            );
        }
        if control == "ineligible" {
            for b in &mut snapshot.ballots {
                b.candidate = Some(WORKSHOP);
            }
        }
        let attempts = snapshot.apply_ballots(&mut w, &s).unwrap();
        if matches!(control, "dead" | "ineligible") {
            assert!(!attempts.is_empty());
            assert!(attempts.iter().all(|a| !a.accepted && a.failure.is_some()));
        }
        if control == "current open" {
            assert!(attempts.is_empty());
        }
        s.month = 9;
        assert!(
            economics_compute_smoke::state_governance::authority(&w, &s)
                .unwrap()
                .governor
                .is_none(),
            "{control}"
        );
    }
}
