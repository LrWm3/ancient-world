use economics_compute_smoke::{
    agreements::{self, Consequence, Counterparty, Identity, LoanState, LoanView, View},
    commitments,
    compute::Backend,
    credit, membership,
    model::Phase,
    resale,
    scenario::{PERSON, PLOT, STATE_AGENT},
    simulation::Simulation,
};

fn loan(sim: &Simulation, agent: u32) -> LoanView<'_> {
    agreements::for_agent(&sim.world, &sim.state, agent)
        .unwrap()
        .into_iter()
        .find_map(|v| match v {
            View::Loan(v) => Some(v),
            _ => None,
        })
        .unwrap()
}

fn credit_sim(case: &str) -> Simulation {
    let (w, s) = credit::scenario(case).unwrap();
    Simulation::new(w, s, Backend::Reference).unwrap()
}

#[test]
fn accepted_terms_are_borrowed_from_book_and_namespaced_not_live_offers() {
    let mut s = credit_sim("default");
    assert!(
        agreements::for_agent(&s.world, &s.state, PERSON)
            .unwrap()
            .is_empty()
    );
    s.run_months(1).unwrap();
    let record = s.state.credit.loans[&1].clone();
    let offer = &mut s.world.credit.as_mut().unwrap().offers[0];
    offer.loan.monthly_rate_bps = 999;
    offer.loan.grace_months = 99;
    offer.collateral.settlement = credit::CollateralSettlement::FixedValue { value: 1 };
    let v = loan(&s, PERSON);
    assert!(std::ptr::eq(v.record(), &s.state.credit.loans[&1]));
    assert_eq!(v.record(), &record);
    assert_eq!(
        v.on_default().unwrap(),
        Consequence::RepossessCollateral {
            asset: PLOT,
            creditor: STATE_AGENT,
            grace_months: record.grace_months,
            settlement: record.collateral.as_ref().unwrap().settlement.clone(),
        }
    );
    let views = agreements::for_agent(&s.world, &s.state, PERSON).unwrap();
    assert_eq!(views[0].identity(), Identity::Loan(1));
    assert_ne!(views[0].identity(), Identity::Land(1));
    assert_eq!(views[0].holder(), PERSON);
    assert_eq!(views[0].grantor(), Counterparty::Agent(STATE_AGENT));
    assert_eq!(views[0].accepted_month(), 1);
    assert!(
        agreements::for_agent(&s.world, &s.state, 999)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn current_due_and_observed_arrears_are_distinct_then_fixed_enforcement_is_deficiency() {
    let mut s = credit_sim("default");
    s.step().unwrap();
    s.step().unwrap();
    s.step().unwrap(); // Accepted; no installment due in opening month.
    assert_eq!(loan(&s, PERSON).state(), LoanState::Current);
    assert!(loan(&s, PERSON).claim().unwrap().is_none());
    while s.state.month < 2 {
        s.step().unwrap();
    }
    let v = loan(&s, PERSON);
    assert_eq!(v.boundary(), (2, Phase::Open));
    assert_eq!(v.state(), LoanState::Current); // No failed collection yet.
    assert_eq!(v.claim().unwrap().unwrap().outstanding(), 2000);
    assert_eq!(v.record().interest, 0); // Query did not accrue this month's interest.
    s.step().unwrap();
    s.step().unwrap(); // Due commits interest and failed payment.
    let v = loan(&s, PERSON);
    assert_eq!(v.state(), LoanState::Overdue { since: 2 });
    assert_eq!(v.claim().unwrap().unwrap().outstanding(), 2080);
    assert_eq!(v.outstanding().unwrap().quantity, 8080);
    assert_eq!(v.title_holder(), Some(PERSON));
    while s.state.month < 3 {
        s.step().unwrap();
    }
    s.step().unwrap();
    s.step().unwrap();
    let v = loan(&s, PERSON);
    assert_eq!(v.state(), LoanState::Deficiency);
    assert_eq!(v.title_holder(), Some(STATE_AGENT));
    assert!(!v.record().collateral.as_ref().unwrap().pledged);
    assert_eq!(v.claim().unwrap().unwrap().outstanding(), 2160);
}

#[test]
fn resale_custody_pauses_claim_but_preserves_debt_then_reports_realized_outcome() {
    for (case, expected, debt) in [
        ("funded", LoanState::Repaid, 0),
        ("deficiency", LoanState::Deficiency, 160),
        ("no-buyer", LoanState::PendingSale { listed: 3 }, 8160),
    ] {
        let (w, state) = resale::scenario(case).unwrap();
        let mut s = Simulation::new(w, state, Backend::CubeCpu).unwrap();
        s.run_months(2).unwrap();
        s.step().unwrap();
        s.step().unwrap();
        let v = loan(&s, PERSON);
        assert_eq!(v.state(), LoanState::PendingSale { listed: 3 });
        assert_eq!(v.outstanding().unwrap().quantity, 8160);
        assert!(v.claim().unwrap().is_none());
        assert_eq!(v.title_holder(), Some(STATE_AGENT));
        assert_eq!(v, loan(&s, STATE_AGENT));
        assert_eq!(v.record(), &s.state.credit.loans[&1]);
        s.run_months(2).unwrap();
        let v = loan(&s, PERSON);
        assert_eq!(v.state(), expected);
        assert_eq!(v.outstanding().unwrap().quantity, debt);
        assert_eq!(
            v.claim().unwrap().is_some(),
            expected == LoanState::Deficiency
        );
    }
}

#[test]
fn inspection_preserves_every_boundary_and_matches_the_authoritative_contract() {
    for case in ["repaid", "recovered", "default", "surplus"] {
        let mut inspected = credit_sim(case);
        let mut control = inspected.clone();
        control.backend = Backend::CubeCpu;
        while inspected.state.month <= 6 {
            if !inspected.state.credit.loans.is_empty() {
                let v = loan(&inspected, PERSON);
                assert_eq!(v, loan(&inspected, STATE_AGENT));
                assert_eq!(
                    v.outstanding().unwrap().quantity,
                    inspected.state.credit.loans[&1].debt().unwrap()
                );
            }
            inspected.step().unwrap();
            control.step().unwrap();
            assert_eq!(inspected.state, control.state);
            assert_eq!(inspected.ledger, control.ledger);
            assert_eq!(inspected.reports, control.reports);
        }
        if case != "default" {
            assert_eq!(loan(&inspected, PERSON).state(), LoanState::Repaid);
        }
    }
}

#[test]
fn shared_entry_point_includes_existing_agreements_and_excludes_unaccepted_offers() {
    let (w, mut s) = membership::scenario().unwrap();
    assert!(agreements::for_agent(&w, &s, PERSON).unwrap().is_empty());
    s.phase = Phase::Acquire;
    let citizen = membership::acceptance(&w, &s, 1, PERSON).unwrap();
    s.memberships
        .insert((PERSON, STATE_AGENT, membership::CITIZEN), citizen);
    let land = commitments::acceptance(&w, &s, 1).unwrap();
    s.accepted_agreements.insert(land.id, land);
    let views = agreements::for_agent(&w, &s, PERSON).unwrap();
    assert_eq!(views.len(), 2);
    assert!(views.iter().any(|v| v.identity() == Identity::Land(1)));
    assert_eq!(views, agreements::for_agent(&w, &s, STATE_AGENT).unwrap());

    let (w, s) = credit::crop_scenario(true).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    let views = agreements::for_agent(&sim.world, &sim.state, PERSON).unwrap();
    assert!(
        views
            .iter()
            .any(|v| matches!(v.identity(), Identity::Process(_)))
    );
    assert!(views.iter().any(|v| v.identity() == Identity::Loan(1)));
}
