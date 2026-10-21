use economics_compute_smoke::{
    acceptance::{
        self, TOWN,
        continuation::{self, Case},
        financing::Stocks,
        working_capital,
    },
    agency::financing,
    compute::Backend,
    minting::{COIN, ISSUER},
    opportunities::Action,
};
#[test]
fn consent_and_current_lender_funding_bound_candidates() {
    let (mut w, mut s) = working_capital::scenario(Case::Baseline, Stocks::Neither, true).unwrap();
    let before = (w.clone(), s.clone());
    let candidates = financing::candidates(&w, &s).unwrap();
    assert!(!candidates.is_empty());
    assert!(candidates.iter().all(|p| p.advance.debtor == ISSUER
        && p.advance.terms.creditor == TOWN
        && p.advance.month == 2));
    assert_eq!((w.clone(), s.clone()), before);
    assert!(financing::publish(&mut w, &s, &candidates[0]).is_err());
    assert_eq!(
        (w.clone(), s.clone()),
        before,
        "rejected proposals are atomic"
    );
    let mut off_cycle = w.clone();
    off_cycle.agency.get_mut(&TOWN).unwrap().config.review_every = 2;
    let mut second_month = s.clone();
    second_month.month = 2;
    assert!(
        financing::candidates(&off_cycle, &second_month)
            .unwrap()
            .is_empty()
    );
    s.balances.insert((TOWN, COIN), 0);
    assert!(financing::candidates(&w, &s).unwrap().is_empty());
    s = before.1;
    w.agency
        .get_mut(&TOWN)
        .unwrap()
        .config
        .financing
        .as_mut()
        .unwrap()
        .lend = false;
    assert!(financing::candidates(&w, &s).unwrap().is_empty());
    w = before.0;
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(acceptance::TOWN_TYPE, Action::Lend));
    assert!(financing::candidates(&w, &s).unwrap().is_empty());
}
#[test]
fn finite_working_capital_restores_neither_stock_startup() {
    let (w, s) = working_capital::scenario(Case::Baseline, Stocks::Neither, true).unwrap();
    let mut run = acceptance::Run::new(w, s, Backend::Reference).unwrap();
    run.through(48).unwrap();
    println!("EVIDENCE {:?}", run.evidence().unwrap());
    let check = continuation::check(&run).unwrap();
    println!("READINESS {check:?}");
    assert!(check.ready(), "{check:?}");
    assert_eq!(run.sim.world.agency[&ISSUER].financed_programs.len(), 1);
    let agreement = &run.sim.world.agency[&ISSUER].financed_programs[0];
    println!("FINANCING {agreement:?}");
    assert_eq!(agreement.month, 3);
    assert_eq!(
        run.sim.state.credit.loans[&agreement.proposal.advance.id].status,
        economics_compute_smoke::credit::Status::Repaid
    );
    assert!(
        !run.sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.events)
            .any(|e| matches!(e, economics_compute_smoke::credit::Event::Arrears { .. }))
    );
}

#[test]
fn publication_admission_activation_and_authority_are_separate() {
    use economics_compute_smoke::{agency, credit, model::Phase, settlement::validate_world};
    let (w, s) = working_capital::scenario(Case::Baseline, Stocks::Neither, true).unwrap();
    let mut sim =
        economics_compute_smoke::simulation::Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.world.agency[&ISSUER].financed_programs.is_empty() {
        assert!(sim.state.month <= 4);
        sim.step().unwrap();
    }
    let agreement = sim.world.agency[&ISSUER].financed_programs[0].clone();
    let advance = &agreement.proposal.advance;
    assert_eq!(agreement.month, 3);
    assert_eq!(advance.month, 4);
    assert_eq!(agreement.status, financing::Status::Pending);
    assert!(!sim.state.credit.loans.contains_key(&advance.id));
    assert!(!agency::accepted_programs(&sim.world, ISSUER).any(|p| p == &agreement.program));
    assert!(!agency::forwards::execution::reviewable(
        &sim.world, &sim.state, TOWN
    ));
    for all in [false, true] {
        let mut frozen = sim.world.clone();
        if all {
            for c in frozen.agency.values_mut() {
                c.autonomous = false;
            }
        } else {
            frozen.agency.get_mut(&ISSUER).unwrap().autonomous = false;
        }
        // Freezing future decisions cannot erase already required mutual consent.
        frozen.agency.get_mut(&ISSUER).unwrap().financed_programs[0]
            .assessment
            .comparisons
            .clear();
        assert!(validate_world(&frozen, &sim.state).is_err());
    }
    let mut wrong_party = sim.world.clone();
    let comparisons = &mut wrong_party
        .agency
        .get_mut(&ISSUER)
        .unwrap()
        .financed_programs[0]
        .assessment
        .comparisons;
    let consent = comparisons.remove(&TOWN).unwrap();
    comparisons.insert(acceptance::WORKSHOP, consent);
    assert!(validate_world(&wrong_party, &sim.state).is_err());
    let mut wrong_officer = sim.world.clone();
    wrong_officer
        .agency
        .get_mut(&ISSUER)
        .unwrap()
        .financed_programs[0]
        .assessment
        .comparisons
        .get_mut(&ISSUER)
        .unwrap()
        .governor = acceptance::SMITH;
    assert!(validate_world(&wrong_officer, &sim.state).is_err());
    for actor in [ISSUER, TOWN] {
        let mut changed = sim.world.clone();
        changed
            .agency
            .get_mut(&actor)
            .unwrap()
            .config
            .financing
            .as_mut()
            .unwrap()
            .max_principal += 1;
        assert!(validate_world(&changed, &sim.state).is_err());
    }
    while !(sim.state.month == advance.month && sim.state.phase == Phase::Acquire) {
        sim.step().unwrap();
    }
    let pending_review = sim.world.agency[&ISSUER].history.last().unwrap();
    assert!(pending_review.chosen.is_none());
    assert!(pending_review.reason.contains("awaiting agreement"));
    assert!(
        !pending_review.ballots.is_empty(),
        "pending loan must not suppress elections"
    );
    let mut unfunded = sim.clone();
    // Deliberate external shock after consent, before ordinary credit admission.
    unfunded.state.balances.insert((TOWN, COIN), 0);
    sim.step().unwrap();
    assert_eq!(
        sim.state.credit.loans[&advance.id].principal,
        advance.principal
    );
    assert!(
        sim.ledger
            .last()
            .unwrap()
            .credit
            .as_ref()
            .unwrap()
            .events
            .iter()
            .any(|e| matches!(e, credit::Event::Advanced { loan, .. } if *loan == advance.id))
    );
    assert_eq!(
        sim.world.agency[&ISSUER].financed_programs[0].status,
        financing::Status::Pending
    );
    while !(sim.state.month == advance.month + 1 && sim.state.phase == Phase::Open) {
        sim.step().unwrap();
    }
    while !(unfunded.state.month == advance.month + 1 && unfunded.state.phase == Phase::Open) {
        unfunded.step().unwrap();
    }
    unfunded.step().unwrap();
    assert_eq!(
        unfunded.world.agency[&ISSUER].financed_programs[0].status,
        financing::Status::AdmissionRejected(advance.month + 1)
    );
    assert!(!unfunded.state.credit.loans.contains_key(&advance.id));
    let mut prohibited = sim.clone();
    let mint = sim.world.minting.as_ref().unwrap().definition;
    let kind = sim.world.transaction_policy.as_ref().unwrap().agent_types[&ISSUER];
    prohibited
        .world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(kind, Action::Process(mint)));
    prohibited.step().unwrap();
    assert_eq!(
        prohibited.world.agency[&ISSUER].financed_programs[0].status,
        financing::Status::WorkRejected(advance.month + 1)
    );
    assert!(prohibited.state.credit.loans[&advance.id].debt().unwrap() > 0);
    // No elected successor is a legitimate future boundary in a private forecast.
    // Accepted commitments survive that vacancy; future discretionary choices do not.
    let mut vacant = sim.clone();
    vacant
        .world
        .state_governance
        .as_mut()
        .unwrap()
        .ballots
        .retain(|b| b.ballot.term_start < advance.month + 1);
    for c in vacant.world.agency.values_mut() {
        c.autonomous = false;
    }
    assert!(agency::governor(&vacant.world, &vacant.state, ISSUER).is_none());
    vacant.step().unwrap();
    assert_eq!(
        vacant.world.agency[&ISSUER].financed_programs[0].status,
        financing::Status::Activated(advance.month + 1)
    );
    assert!(agency::accepted_programs(&vacant.world, ISSUER).any(|p| p == &agreement.program));
}
