use economics_compute_smoke::{
    allocation::{Outcome, Policy},
    competition::{self, Application, SECOND_PERSON},
    compute::Backend,
    model::*,
    offers::{self, Id, Request},
    scenario::*,
    settlement,
    simulation::Simulation,
};
fn sim(backend: Backend, plots: u32) -> Simulation {
    let (w, s) = competition::scenario(plots, competition::DEFAULT_SEED).unwrap();
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim
}
fn applicants() -> Vec<Application> {
    [PERSON, SECOND_PERSON]
        .into_iter()
        .map(|agent| Application {
            agent,
            requests: vec![
                Request::new(Id::Membership(1), agent),
                Request::new(Id::Land(1), agent),
                Request::new(Id::Process(GROW), agent),
            ],
        })
        .collect()
}
#[test]
fn one_posting_two_conditional_applications_only_winner_spends_or_owes() {
    let mut s = sim(Backend::CubeCpu, 1);
    for agent in [PERSON, SECOND_PERSON] {
        assert!(
            offers::discover(&s.world, &s.state, agent)
                .iter()
                .any(|o| o.id == Id::Land(1))
        );
    }
    let opening = s.state.clone();
    let a = applicants();
    let batch = competition::prepare(&s, 1, 7, Policy::PriorityLottery, &a).unwrap();
    assert_eq!(s.state, opening);
    let mut reverse = a.clone();
    reverse.reverse();
    assert_eq!(
        batch,
        competition::prepare(&s, 1, 7, Policy::PriorityLottery, &reverse).unwrap()
    );
    let mut tampered = batch.clone();
    tampered.allocation.as_mut().unwrap().context.round += 1;
    assert!(
        settlement::commit(&s.world, &mut s.state, &tampered, s.backend, s.effect_limit).is_err()
    );
    assert_eq!(s.state, opening);
    competition::accept(&mut s, 1, 7, Policy::PriorityLottery, &a).unwrap();
    let winner = s.state.accepted_agreements[&1].debtor;
    let loser = if winner == PERSON {
        SECOND_PERSON
    } else {
        PERSON
    };
    assert_eq!(s.state.accepted_agreements.len(), 1);
    assert!(s.state.obligations.is_empty());
    s.step().unwrap();
    assert_eq!(s.state.balance(winner, SEED), 0);
    assert_eq!(s.state.balance(loser, SEED), 1);
    assert!(
        s.state
            .processes
            .values()
            .filter(|p| p.definition == GROW)
            .all(|p| p.operator == winner)
    );
    s.run_months(12).unwrap();
    assert_eq!(s.state.accepted_agreements[&1].debtor, winner); // no tenure redraw
    assert!(
        s.state
            .obligations
            .values()
            .all(|o| s.state.accepted_agreements[&o.agreement].debtor == winner)
    );
}
#[test]
fn infeasible_applicant_does_not_block_the_other_or_create_breach() {
    let mut s = sim(Backend::Reference, 1);
    s.state.balances.insert((PERSON, SEED), 0);
    let batch = competition::prepare(&s, 1, 0, Policy::StablePriority, &applicants()).unwrap();
    assert_eq!(batch.access_applicant, Some(SECOND_PERSON));
    assert!(
        batch
            .allocation
            .unwrap()
            .receipts
            .iter()
            .any(|r| r.claim.id == u64::from(PERSON) && matches!(r.outcome, Outcome::Rejected(_)))
    );
}
#[test]
fn autonomous_two_plot_control_and_scarcity_run_on_cpu_and_reference() {
    for plots in [1, 2] {
        let mut cpu = sim(Backend::CubeCpu, plots);
        let mut reference = sim(Backend::Reference, plots);
        cpu.run_months(18).unwrap();
        reference.run_months(18).unwrap();
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.reports, reference.reports);
        assert_eq!(cpu.state.accepted_agreements.len(), plots as usize);
        assert!(
            cpu.state
                .accepted_agreements
                .values()
                .all(|a| a.activated == 1)
        );
        if plots == 2 {
            assert!(cpu.state.terminal.is_empty());
            assert!(
                cpu.reports
                    .iter()
                    .all(|r| r.deficit(NUTRITION) == 0 && r.deficit(WARMTH) == 0)
            );
        }
        eprintln!(
            "plots={plots}: accepted={:?}, terminal={:?}",
            cpu.state.accepted_agreements, cpu.state.terminal
        );
    }
}

#[test]
fn existing_tenure_lowers_priority_and_loser_replans_for_warmth() {
    let mut s = sim(Backend::Reference, 2);
    let mut incumbent = s.world.access_offers.remove(1);
    incumbent.debtor = PERSON;
    s.world.open_access_offers.remove(&incumbent.id);
    s.world.rights[1].holder = PERSON;
    s.world.rights[1].output_owner = PERSON;
    s.world.agreements.push(incumbent);
    for agent in [PERSON, SECOND_PERSON] {
        s.state.balances.insert((agent, FUEL), 0);
    }
    // Lowest ID loses despite a stable-ID tie breaker: existing tenure matters first.
    let batch = competition::prepare(&s, 1, 7, Policy::StablePriority, &applicants()).unwrap();
    assert_eq!(batch.access_applicant, Some(SECOND_PERSON));
    assert!(
        batch
            .production_plan
            .as_ref()
            .unwrap()
            .transactions
            .iter()
            .any(|t| t
                .process
                .as_ref()
                .is_some_and(|c| c.after.operator == PERSON && c.after.definition == PREPARE_FUEL))
    );
}

#[test]
fn reversed_people_and_checkpoint_continuation_preserve_outcomes() {
    let mut normal = sim(Backend::Reference, 2);
    let mut reversed = normal.clone();
    reversed.world.agents.reverse();
    reversed.world.participants.reverse();
    reversed.world.access_offers.reverse();
    normal.run_months(6).unwrap();
    reversed.run_months(6).unwrap();
    assert_eq!(normal.state, reversed.state);
    let mut resumed = normal.clone();
    normal.run_months(6).unwrap();
    for _ in 0..6 {
        resumed.run_months(1).unwrap();
    }
    assert_eq!(normal.state, resumed.state);
    assert_eq!(normal.ledger, resumed.ledger);
}

fn alternatives() -> Vec<Application> {
    let mut all = applicants();
    let other: Vec<_> = all
        .iter()
        .cloned()
        .map(|mut a| {
            for r in &mut a.requests {
                if r.offer == Id::Land(1) {
                    r.offer = Id::Land(2);
                }
            }
            a
        })
        .collect();
    all.extend(other);
    all
}

#[test]
fn simultaneous_acceptance_reserves_each_person_once_and_rejects_forged_extra_awards() {
    let mut s = sim(Backend::CubeCpu, 2);
    let before = s.state.clone();
    let requests = alternatives();
    let batch =
        competition::prepare_many(&s, &[1, 2], 7, Policy::PriorityLottery, &requests).unwrap();
    let mut reversed = requests;
    reversed.reverse();
    assert_eq!(
        batch,
        competition::prepare_many(&s, &[2, 1], 7, Policy::PriorityLottery, &reversed).unwrap()
    );
    assert_eq!(batch.allocation.as_ref().unwrap().awards.len(), 2);
    let mut forged = batch.clone();
    forged.additional_access.push((1, PERSON));
    assert!(
        settlement::commit(&s.world, &mut s.state, &forged, s.backend, s.effect_limit).is_err()
    );
    assert_eq!(s.state, before);
    settlement::commit(&s.world, &mut s.state, &batch, s.backend, s.effect_limit).unwrap();
    assert_eq!(s.state.memberships.len(), 2);
    assert!(
        s.state
            .accepted_agreements
            .values()
            .all(|a| a.activated == 1)
    );
    assert_eq!(s.state.accepted_agreements.len(), 2);
    for agent in [PERSON, SECOND_PERSON] {
        assert_eq!(s.state.balance(agent, SEED), 1);
    }
    // Checkpoint at the reservation barrier must execute the same complete plan.
    let mut checkpoint = s.clone();
    s.step().unwrap();
    checkpoint.step().unwrap();
    assert_eq!(s.state, checkpoint.state);
    for agent in [PERSON, SECOND_PERSON] {
        assert_eq!(s.state.balance(agent, SEED), 0);
        assert_eq!(
            s.state
                .processes
                .values()
                .filter(|p| p.operator == agent && p.definition == GROW)
                .count(),
            1
        );
    }
}

#[test]
fn alternatives_are_exclusive_and_joint_resource_shortfall_is_rechecked() {
    let mut s = sim(Backend::Reference, 2);
    let one: Vec<_> = alternatives()
        .into_iter()
        .filter(|a| a.agent == PERSON)
        .collect();
    let batch = competition::prepare_many(&s, &[1, 2], 7, Policy::StablePriority, &one).unwrap();
    assert_eq!(batch.allocation.as_ref().unwrap().awards.len(), 1);
    assert!(batch.additional_access.is_empty());
    s.state.balances.insert((STATE_AGENT, RAW_WOOD), 1);
    let mut requests = alternatives();
    for a in &mut requests {
        a.requests
            .push(Request::new(Id::Process(PREPARE_FUEL), a.agent));
        assert!(offers::feasible(&s, &a.requests).is_ok());
    }
    let opening = s.state.clone();
    let batch =
        competition::prepare_many(&s, &[1, 2], 7, Policy::StablePriority, &requests).unwrap();
    assert_eq!(s.state, opening);
    let round = batch.allocation.as_ref().unwrap();
    assert_eq!(round.awards.len(), 1); // Each requires the same last unit of wood.
    assert!(!round.rejections.is_empty());
    assert_eq!(
        round.awards.keys().copied().collect::<Vec<_>>(),
        [u64::from(PERSON)]
    );
}
