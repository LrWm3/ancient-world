use economics_compute_smoke::{
    accounting::Account,
    compute::Backend,
    employment::{ArrearsPolicy, Terms},
    financial_reporting::{Audit, Opening},
    minting::{self, COIN, HOURS, ISSUER, SUPPLIER, WORKER},
    model::*,
    recovery::{Guarantee, GuaranteedClaim, Receipt},
    simulation::Simulation,
};

fn fixture() -> (World, State) {
    let (mut w, mut s) = minting::scenario("normal").unwrap();
    w.minting = None;
    w.transaction_policy = None;
    w.scheduled_starts.clear();
    w.capacity_overrides.clear();
    w.priority = Priority::ContinuingFirst;
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == WORKER { 2 } else { 0 };
    }
    s.balances.clear();
    s.balances.insert((SUPPLIER, COIN), 3);
    w.employment.push(Terms {
        id: 1,
        employer: ISSUER,
        worker: WORKER,
        from: 1,
        through: 1,
        capacity: Amount::new(HOURS, 2),
        wage_per_unit: Amount::new(COIN, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    w.recovery.guarantees.push(Guarantee {
        follows_assignment: false,
        tender: economics_compute_smoke::recovery::GuaranteeTender::Native,
        security: economics_compute_smoke::recovery::RecourseSecurity::Unsecured,
        id: 1,
        claim: GuaranteedClaim::Wages {
            agreement: 1,
            earned_month: 1,
        },
        guarantor: SUPPLIER,
        cap: 4,
        from: 1,
        through: 8,
        delay_months: 0,
        recourse: 101,
        priority: 0,
    });
    (w, s)
}
fn through(a: &mut Audit, s: &mut Simulation, month: u32) {
    while s.state.month <= month {
        a.step(s)
            .unwrap_or_else(|e| panic!("{} {:?}: {e}", s.state.month, s.state.phase));
    }
}
#[test]
fn earned_wage_guarantee_pays_only_the_residual_and_creates_matching_recourse() {
    let (w, s) = fixture();
    let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
    let mut resumed =
        Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 3);
    reference.run_months(3).unwrap();
    resumed.run_months(2).unwrap();
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(sim.state, resumed.state);
    assert_eq!(sim.state.balance(WORKER, COIN), 3);
    assert_eq!(sim.state.balance(SUPPLIER, COIN), 0);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 1);
    let recourse = &sim.state.credit.loans[&101];
    assert_eq!(
        (recourse.debtor, recourse.creditor, recourse.principal),
        (ISSUER, SUPPLIER, 3)
    );
    assert_eq!(recourse.opened, 2);
    let b = a.book().balances();
    assert_eq!(b[&(WORKER, Account::WagesReceivable(1, 1))], 1);
    assert_eq!(b[&(ISSUER, Account::WagesPayable(1, 1))], -1);
    assert_eq!(b[&(SUPPLIER, Account::LoanReceivable(101))], 3);
    assert_eq!(b[&(ISSUER, Account::LoanPayable(101))], -3);
    assert_eq!(b[&(WORKER, Account::ServiceIncome)], -4);
    assert_eq!(b[&(ISSUER, Account::ServiceExpense)], 4);
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.recovery)
            .any(|r| matches!(
                r,
                Receipt::Guaranteed {
                    paid: 3,
                    requested: 4,
                    ..
                }
            ))
    );
}
#[test]
fn unearned_work_has_no_call_and_ordinary_payment_reduces_coverage() {
    for (hours, employer_cash, expected) in [(0, 0, 0), (2, 1, 3), (2, 4, 0)] {
        let (mut w, mut s) = fixture();
        w.participants
            .iter_mut()
            .find(|p| p.agent == WORKER)
            .unwrap()
            .capacity
            .quantity = hours;
        s.balances.insert((ISSUER, COIN), employer_cash);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(2).unwrap();
        assert_eq!(
            sim.state
                .credit
                .recovery
                .paid_guarantees
                .get(&1)
                .copied()
                .unwrap_or(0),
            expected
        );
        if expected > 0 {
            assert_eq!(sim.state.credit.loans[&101].principal, expected);
        } else {
            assert!(!sim.state.credit.loans.contains_key(&101));
        }
    }
}

fn land_fixture() -> (World, State) {
    let (mut w, s) = fixture();
    w.employment.clear();
    w.assets.push(Asset {
        id: 900,
        owner: WORKER,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: 900,
        holder: ISSUER,
        asset: 900,
        from: 1,
        through: 30,
        output_owner: ISSUER,
    });
    w.agreements
        .push(economics_compute_smoke::commitments::Agreement {
            id: 900,
            right: 900,
            creditor: WORKER,
            debtor: ISSUER,
            activated: 1,
            payment: Amount::new(COIN, 4),
        });
    w.recovery.guarantees[0].claim = GuaranteedClaim::Land {
        agreement: 900,
        due: 13,
    };
    w.recovery.guarantees[0].through = 30;
    (w, s)
}
#[test]
fn land_guarantee_settles_one_original_bill_without_renewing_or_paying_later_bills() {
    let (w, s) = land_fixture();
    let opening = Opening {
        assets: [(900, 0)].into(),
        dues: Some(Default::default()),
        ..Default::default()
    };
    let mut a = Audit::with_opening(&w, &s, COIN, opening).unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 12);
    assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
    through(&mut a, &mut sim, 25);
    reference.run_months(25).unwrap();
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    let first = &sim.state.obligations[&(900, 13)];
    assert_eq!(
        (
            first.owed,
            first.paid,
            first.in_kind_paid,
            first.outstanding()
        ),
        (4, 3, 3, 1)
    );
    assert_eq!(sim.state.obligations[&(900, 25)].outstanding(), 4);
    assert_eq!(sim.state.credit.loans[&101].principal, 3);
    assert_eq!(sim.state.balance(WORKER, COIN), 3);
    let b = a.book().balances();
    assert_eq!(b[&(ISSUER, Account::DuesPayable(900, 13))], -1);
    assert_eq!(b[&(ISSUER, Account::LoanPayable(101))], -3);
    assert_eq!(b[&(WORKER, Account::DuesReceivable(900, 13))], 1);
    assert_eq!(b[&(SUPPLIER, Account::LoanReceivable(101))], 3);
    assert_eq!(b[&(ISSUER, Account::DuesExpense)], 8);
    assert_eq!(b[&(WORKER, Account::DuesIncome)], -8);
}
#[test]
fn guarantee_terms_reject_unidentified_dates_and_allow_physical_wages() {
    let (mut w, s) = land_fixture();
    w.recovery.guarantees[0].claim = GuaranteedClaim::Land {
        agreement: 900,
        due: 14,
    };
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
    let (mut w, s) = fixture();
    let physical = economics_compute_smoke::minting::FIREWOOD;
    w.employment[0].wage_per_unit.resource = physical;
    assert!(Simulation::new(w, s, Backend::Reference).is_ok());
}

const OTHER_WORKER: AgentId = 92;
const OTHER_GUARANTOR: AgentId = 93;
fn competing() -> (World, State) {
    let (mut w, mut s) = fixture();
    w.agents.push(Agent {
        id: OTHER_WORKER,
        name: "second worker".into(),
    });
    let mut participant = w
        .participants
        .iter()
        .find(|p| p.agent == WORKER)
        .unwrap()
        .clone();
    participant.agent = OTHER_WORKER;
    w.participants.push(participant);
    let mut job = w.employment[0].clone();
    job.id = 2;
    job.worker = OTHER_WORKER;
    w.employment.push(job);
    let mut g = w.recovery.guarantees[0].clone();
    g.id = 2;
    g.claim = GuaranteedClaim::Wages {
        agreement: 2,
        earned_month: 1,
    };
    g.recourse = 102;
    w.recovery.guarantees.push(g);
    s.balances.insert((SUPPLIER, COIN), 6);
    (w, s)
}
#[test]
fn explicit_guarantee_allocation_compares_identical_requests_and_finite_funds() {
    use economics_compute_smoke::finance::CollectionPolicy::{Proportional, Stable};
    for (policy, high_second, expected) in [
        (Stable, false, (4, 2)),
        (Proportional, false, (3, 3)),
        (Proportional, true, (2, 4)),
    ] {
        let (mut w, s) = competing();
        w.recovery.guarantee_policy = policy;
        if high_second {
            w.recovery.guarantees[0].priority = 1;
        }
        let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
        w.recovery.guarantees.reverse();
        w.employment.reverse();
        let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut sim, 2);
        reference.run_months(2).unwrap();
        assert_eq!(sim.state, reference.state);
        assert_eq!(sim.ledger, reference.ledger);
        assert_eq!(
            (
                sim.state.balance(WORKER, COIN),
                sim.state.balance(OTHER_WORKER, COIN)
            ),
            expected
        );
        assert_eq!(sim.state.balance(SUPPLIER, COIN), 0);
        let receipts: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.recovery)
            .filter_map(|r| {
                if let Receipt::Guaranteed {
                    requested,
                    allocated,
                    paid,
                    ..
                } = r
                {
                    Some((*requested, *allocated, *paid))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(receipts[0].0, 4);
        assert_eq!(receipts[1].0, 4);
        assert_eq!(receipts.iter().map(|r| r.2).sum::<i32>(), 6);
        assert!(
            receipts
                .iter()
                .all(|r| r.1.is_some() == (policy == Proportional))
        );
    }
}
#[test]
fn overlapping_coverage_releases_unneeded_grants_without_double_payment() {
    let (mut w, mut s) = competing();
    w.recovery.guarantee_policy = economics_compute_smoke::finance::CollectionPolicy::Proportional;
    w.agents.push(Agent {
        id: OTHER_GUARANTOR,
        name: "another guarantor".into(),
    });
    s.balances.insert((OTHER_GUARANTOR, COIN), 4);
    s.balances.insert((SUPPLIER, COIN), 4);
    let mut duplicate = w.recovery.guarantees[0].clone();
    duplicate.id = 3;
    duplicate.recourse = 103;
    w.recovery.guarantees[0].guarantor = OTHER_GUARANTOR;
    w.recovery.guarantees.push(duplicate);
    let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    w.recovery.guarantees.reverse();
    let mut reordered = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 2);
    reordered.run_months(2).unwrap();
    assert_eq!(sim.state, reordered.state);
    assert_eq!(sim.ledger, reordered.ledger);
    assert_eq!(sim.state.balance(WORKER, COIN), 4);
    assert_eq!(sim.state.balance(OTHER_WORKER, COIN), 4);
    assert_eq!(sim.state.credit.loans[&101].principal, 4);
    assert_eq!(sim.state.credit.loans[&102].principal, 4);
    assert!(!sim.state.credit.loans.contains_key(&103));
    assert!(
        sim.state
            .employment
            .earned
            .values()
            .all(|e| e.claim.outstanding() == 0)
    );
}

#[test]
fn guaranteed_member_wages_pool_actual_receipts_once_and_preserve_household_recourse() {
    use economics_compute_smoke::{
        households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
        opportunities::{Action, PERSON_TYPE},
        scenario::{LABOR, PERSON, TOKEN},
    };
    const EMPLOYER: AgentId = 89;
    const OUTSIDER: AgentId = 92;
    for guarantor in [OUTSIDER, HOME] {
        let (mut w, mut s) = households::market::scenario().unwrap();
        w.town_market = None;
        s.town_market = Default::default();
        s.balances.clear();
        s.balances.insert((guarantor, TOKEN), 4);
        w.activities.orders.clear();
        for p in &mut w.participants {
            p.needs.clear();
            p.capacity.quantity = if p.agent == PERSON { 5 } else { 0 };
        }
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((PERSON_TYPE, Action::CapacityTrade));
        w.employment.push(Terms {
            id: 1,
            employer: EMPLOYER,
            worker: PERSON,
            from: 1,
            through: 1,
            capacity: Amount::new(LABOR, 2),
            wage_per_unit: Amount::new(TOKEN, 2),
            on_arrears: ArrearsPolicy::Continue,
            rank: 0,
        });
        w.recovery.guarantees.push(Guarantee {
            follows_assignment: false,
            tender: economics_compute_smoke::recovery::GuaranteeTender::Native,
            security: economics_compute_smoke::recovery::RecourseSecurity::Unsecured,
            id: 1,
            claim: GuaranteedClaim::Wages {
                agreement: 1,
                earned_month: 1,
            },
            guarantor,
            cap: 4,
            from: 1,
            through: 8,
            delay_months: 0,
            recourse: 101,
            priority: 0,
        });
        let mut a = Audit::with_opening(
            &w,
            &s,
            TOKEN,
            Opening {
                assets: w.assets.iter().map(|asset| (asset.id, 0)).collect(),
                ..Default::default()
            },
        )
        .unwrap();
        let mut b = a.clone();
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
        let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut a, &mut sim, 4);
        through(&mut b, &mut reference, 4);
        assert_eq!(sim.state, reference.state);
        assert_eq!(sim.ledger, reference.ledger);
        assert_eq!(a.book().balances(), b.book().balances());
        assert_eq!(sim.state.balance(PERSON, TOKEN), 2);
        assert_eq!(sim.state.balance(HOME, TOKEN), 2);
        assert_eq!(sim.state.balance(OUTSIDER, TOKEN), 0);
        assert_eq!(sim.state.credit.loans[&101].principal, 4);
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
        assert_eq!(
            a.book().balances()[&(guarantor, Account::LoanReceivable(101))],
            4
        );
        assert_eq!(
            a.book().balances()[&(EMPLOYER, Account::LoanPayable(101))],
            -4
        );
    }
}

const ESTATE: AgentId = 999;
fn authorize(w: &mut World, month: u32) {
    w.agents.push(Agent {
        id: ESTATE,
        name: "custodian".into(),
    });
    w.recovery
        .proceedings
        .push(economics_compute_smoke::recovery::ProceedingTerms {
            id: 1,
            debtor: ISSUER,
            authority: ISSUER,
            estate: ESTATE,
            denomination: COIN,
            opening_month: month,
            earliest_close: month,
            assets: vec![],
            discharge_deficiency: false,
        });
}
#[test]
fn accepted_relief_changes_guarantee_calls_without_extending_the_guarantee_term() {
    use economics_compute_smoke::{
        claim_relief::{Action, Terms as Relief},
        finance::ContractId,
    };
    for expires in [false, true] {
        let (mut w, s) = fixture();
        authorize(&mut w, 3);
        w.recovery.guarantees[0].delay_months = 2;
        if expires {
            w.recovery.guarantees[0].through = 6;
        }
        w.recovery.claim_relief = vec![
            Relief {
                id: 1,
                proceeding: 1,
                contract: ContractId::Wages(1),
                original_due: 2,
                debtor: ISSUER,
                creditor: WORKER,
                month: 3,
                expected_due: 2,
                expected_remaining: 4,
                action: Action::Extend { due: 5 },
            },
            Relief {
                id: 2,
                proceeding: 1,
                contract: ContractId::Wages(1),
                original_due: 2,
                debtor: ISSUER,
                creditor: WORKER,
                month: 6,
                expected_due: 5,
                expected_remaining: 4,
                action: Action::WriteOff { quantity: 1 },
            },
        ];
        let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        through(&mut a, &mut sim, 6);
        assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 3);
        let mut resumed =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
        through(&mut a, &mut sim, 8);
        resumed.run_months(2).unwrap();
        assert_eq!(sim.state, resumed.state);
        let claim = &sim.state.employment.earned[&(1, 1)].claim;
        if expires {
            assert_eq!(claim.outstanding(), 3);
            assert!(!sim.state.credit.loans.contains_key(&101));
            assert_eq!(sim.state.balance(SUPPLIER, COIN), 3);
        } else {
            assert_eq!(claim.outstanding(), 0);
            assert_eq!(claim.settled, 3);
            assert_eq!(sim.state.credit.loans[&101].principal, 3);
            assert_eq!(sim.state.credit.loans[&101].opened, 7);
        }
        assert_eq!(a.book().balances()[&(WORKER, Account::CreditLoss)], 1);
        assert_eq!(a.book().balances()[&(ISSUER, Account::DebtRelief)], -1);
    }
}
#[test]
fn same_boundary_guarantee_payment_invalidates_stale_relief_consent() {
    use economics_compute_smoke::{
        claim_relief::{Action, Terms as Relief},
        finance::ContractId,
    };
    let (mut w, s) = fixture();
    authorize(&mut w, 3);
    w.recovery.guarantees[0].from = 3;
    w.recovery.claim_relief.push(Relief {
        id: 1,
        proceeding: 1,
        contract: ContractId::Wages(1),
        original_due: 2,
        debtor: ISSUER,
        creditor: WORKER,
        month: 3,
        expected_due: 2,
        expected_remaining: 4,
        action: Action::WriteOff { quantity: 4 },
    });
    let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 3);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 1);
    assert!(sim.state.employment.earned[&(1, 1)].relief.is_empty());
    assert_eq!(sim.state.credit.loans[&101].principal, 3);
    assert!(
        sim.ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.recovery)
            .any(|r| matches!(
                r,
                Receipt::ClaimRelief {
                    rejection: Some(_),
                    written_off: None,
                    ..
                }
            ))
    );
}

#[test]
fn later_guarantee_advances_cannot_collect_from_the_estate_in_the_same_boundary() {
    let (mut w, mut s) = fixture();
    s.balances.insert((SUPPLIER, COIN), 2);
    w.agents.push(Agent {
        id: OTHER_GUARANTOR,
        name: "funded service buyer".into(),
    });
    s.balances.insert((OTHER_GUARANTOR, COIN), 4);
    authorize(&mut w, 3);
    // The debtor earns estate funding in month 2. The worker later buys a real
    // service from the guarantor, funding the second guarantee advance in month 4.
    for (id, employer, worker, month, wage) in [
        (2, OTHER_GUARANTOR, ISSUER, 2, 4),
        (3, WORKER, SUPPLIER, 3, 2),
    ] {
        w.capacity_overrides.insert((month, worker), 1);
        w.employment.push(Terms {
            id,
            employer,
            worker,
            from: month,
            through: month,
            capacity: Amount::new(HOURS, 1),
            wage_per_unit: Amount::new(COIN, wage),
            on_arrears: ArrearsPolicy::Continue,
            rank: 0,
        });
    }
    let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut sim, 3);
    assert_eq!(sim.state.credit.loans[&101].principal, 2);
    assert_eq!(sim.state.balance(ESTATE, COIN), 4);
    assert_eq!(sim.state.balance(SUPPLIER, COIN), 2);
    let mut resumed =
        Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
    loop {
        a.step(&mut sim).unwrap();
        if sim
            .ledger
            .last()
            .is_some_and(|b| b.month == 4 && b.phase == Phase::Due)
        {
            break;
        }
    }
    let views =
        economics_compute_smoke::agreements::for_agent(&sim.world, &sim.state, SUPPLIER).unwrap();
    let view = views
        .iter()
        .find(|v| v.identity() == economics_compute_smoke::agreements::Identity::Loan(101))
        .unwrap();
    assert!(view.claims().unwrap().is_empty());
    let mut forged = sim.state.clone();
    forged.credit.recovery.guarantee_advances.remove(&(1, 4));
    assert!(Simulation::new(sim.world.clone(), forged, Backend::Reference).is_err());
    let mut forged = sim.state.clone();
    forged
        .employment
        .earned
        .get_mut(&(1, 1))
        .unwrap()
        .claim
        .settled = 0;
    assert!(Simulation::new(sim.world.clone(), forged, Backend::Reference).is_err());
    through(&mut a, &mut sim, 4);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
    assert_eq!(sim.state.credit.loans[&101].original_principal, 4);
    assert_eq!(sim.state.credit.loans[&101].principal, 2);
    assert_eq!(sim.state.balance(ESTATE, COIN), 2);
    assert_eq!(sim.state.balance(SUPPLIER, COIN), 2);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        economics_compute_smoke::recovery::Stage::Active
    );
    through(&mut a, &mut sim, 5);
    resumed.run_months(2).unwrap();
    assert_eq!(sim.state, resumed.state);
    assert_eq!(sim.state.credit.loans[&101].principal, 0);
    assert_eq!(sim.state.balance(ESTATE, COIN), 0);
    assert_eq!(sim.state.balance(SUPPLIER, COIN), 4);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        economics_compute_smoke::recovery::Stage::Closed
    );
}

#[test]
fn household_guarantees_share_cash_across_loan_wage_and_land_claims_with_separate_books() {
    household_mixed_guarantees(false);
}

#[test]
fn household_guarantees_share_native_wages_and_coin_tender_without_recycling_pooled_receipts() {
    household_mixed_guarantees(true);
}

fn household_mixed_guarantees(alternative: bool) {
    use economics_compute_smoke::{
        commitments::Agreement,
        credit::{Advance, LoanOffer},
        finance::CollectionPolicy,
        households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
        scenario::{GRAIN, LABOR, PERSON, STATE_AGENT, TOKEN},
        settlement::{DEFAULT_EFFECT_LIMIT, commit},
        telemetry::{Config, Observer},
    };
    const DEBTOR: AgentId = 89;
    const LENDER: AgentId = 92;
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.transaction_policy = None;
    w.town_market = None;
    s.town_market = Default::default();
    w.activities.orders.clear();
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == PERSON { 5 } else { 0 };
    }
    s.balances.clear();
    s.balances.insert((HOME, TOKEN), 4);
    s.balances.insert((LENDER, TOKEN), 4);
    w.assets.push(Asset {
        id: 900,
        owner: STATE_AGENT,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: 900,
        asset: 900,
        holder: DEBTOR,
        from: 1,
        through: 24,
        output_owner: DEBTOR,
    });
    w.agreements.push(Agreement {
        id: 900,
        right: 900,
        creditor: STATE_AGENT,
        debtor: DEBTOR,
        activated: 1,
        payment: if alternative {
            Amount::new(GRAIN, 2)
        } else {
            Amount::new(TOKEN, 4)
        },
    });
    w.lending.push(Advance {
        id: 10,
        debtor: DEBTOR,
        principal: 4,
        month: 1,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: LENDER,
            denomination: TOKEN,
            max_principal: 4,
            monthly_rate_bps: 0,
            term_months: 12,
            grace_months: 0,
        },
    });
    for (id, month) in [(1, 1), (2, 12)] {
        w.employment.push(Terms {
            id,
            employer: DEBTOR,
            worker: PERSON,
            from: month,
            through: month,
            capacity: Amount::new(LABOR, 2),
            wage_per_unit: Amount::new(TOKEN, 2),
            on_arrears: ArrearsPolicy::Continue,
            rank: 0,
        });
    }
    for (id, claim) in [
        (1, GuaranteedClaim::Loan(10)),
        (
            2,
            GuaranteedClaim::Wages {
                agreement: 2,
                earned_month: 12,
            },
        ),
        (
            3,
            GuaranteedClaim::Land {
                agreement: 900,
                due: 13,
            },
        ),
    ] {
        w.recovery.guarantees.push(Guarantee {
            follows_assignment: false,
            tender: economics_compute_smoke::recovery::GuaranteeTender::Native,
            security: economics_compute_smoke::recovery::RecourseSecurity::Unsecured,
            id,
            claim,
            guarantor: HOME,
            cap: 4,
            from: 13,
            through: 13,
            delay_months: 0,
            recourse: 100 + id,
            priority: 0,
        });
    }
    if alternative {
        w.activities.coin_payments.insert(
            900,
            economics_compute_smoke::activities::CoinPayment {
                resource: TOKEN,
                coins_per_unit: 2,
            },
        );
        w.recovery
            .guarantees
            .iter_mut()
            .find(|g| g.id == 3)
            .unwrap()
            .tender = economics_compute_smoke::recovery::GuaranteeTender::AcceptedLandCoins;
    }
    w.recovery.guarantee_policy = CollectionPolicy::Proportional;
    let mut a = Audit::with_opening(
        &w,
        &s,
        TOKEN,
        Opening {
            assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
            dues: Some(if alternative {
                economics_compute_smoke::dues_accounting::Valuation([(900, 3)].into())
            } else {
                Default::default()
            }),
            exchange_values: if alternative {
                [(GRAIN, 3)].into()
            } else {
                Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();
    let mut reference_audit = a.clone();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    w.recovery.guarantees.reverse();
    w.employment.reverse();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    let mut observer = Observer::new(
        Vec::new(),
        "mixed-guarantees",
        Config {
            settlement: true,
            agents: if alternative {
                [PERSON, HOME].into()
            } else {
                [PERSON].into()
            },
            ..Default::default()
        },
    )
    .unwrap();
    while sim.state.month <= 12 {
        observer.step_audited(&mut sim, &mut a).unwrap();
    }
    assert_eq!(sim.state.balance(HOME, TOKEN), 6);
    let mut resumed =
        Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
    let mut forged_checked = false;
    while sim.state.month <= 13 {
        let before = sim.state.clone();
        let before_audit = a.clone();
        observer.step_audited(&mut sim, &mut a).unwrap();
        let batch = sim.ledger.last().unwrap();
        if batch.phase == Phase::Due {
            let mut bad = batch.clone();
            let r = bad
                .credit
                .as_mut()
                .unwrap()
                .recovery
                .iter_mut()
                .find(|r| matches!(r, Receipt::Guaranteed { paid: 2, .. }))
                .unwrap();
            if let Receipt::Guaranteed { paid, .. } = r {
                *paid += 1;
            }
            let mut candidate = before.clone();
            assert!(
                commit(
                    &sim.world,
                    &mut candidate,
                    &bad,
                    Backend::Reference,
                    DEFAULT_EFFECT_LIMIT
                )
                .is_err()
            );
            assert_eq!(candidate, before);
            let mut reporting = before_audit.clone();
            assert!(
                reporting
                    .record(&sim.world, &before, &bad, &sim.state)
                    .is_err()
            );
            assert_eq!(reporting.book().balances(), before_audit.book().balances());
            forged_checked = true;
        }
    }
    assert!(forged_checked);
    through(&mut reference_audit, &mut reference, 13);
    resumed.run_months(1).unwrap();
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(sim.state, resumed.state);
    assert_eq!(a.book().balances(), reference_audit.book().balances());
    assert_eq!(sim.state.balance(HOME, TOKEN), 1); // one coin of actual wage pooling, not recycled at Due
    assert_eq!(sim.state.balance(PERSON, TOKEN), 3);
    assert_eq!(sim.state.balance(LENDER, TOKEN), 2);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 2);
    assert_eq!(sim.state.credit.loans[&10].principal, 2);
    assert_eq!(sim.state.employment.earned[&(2, 12)].claim.outstanding(), 2);
    assert_eq!(
        sim.state.obligations[&(900, 13)].outstanding(),
        if alternative { 1 } else { 2 }
    );
    assert_eq!(
        sim.state.obligations[&(900, 13)].in_kind_paid,
        if alternative { 0 } else { 2 }
    );
    for id in [101, 102, 103] {
        let native = alternative && id == 103;
        assert_eq!(
            sim.state.credit.loans[&id].principal,
            if native { 1 } else { 2 }
        );
        assert_eq!(
            sim.state.credit.loans[&id].denomination,
            if native { GRAIN } else { TOKEN }
        );
        assert_eq!(
            a.book().balances()[&(HOME, Account::LoanReceivable(id))],
            if native { 3 } else { 2 }
        );
        assert_eq!(
            a.book().balances()[&(DEBTOR, Account::LoanPayable(id))],
            if native { -3 } else { -2 }
        );
    }
    let records = String::from_utf8(observer.finish().unwrap()).unwrap();
    let calls: Vec<serde_json::Value> = records
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|r: &serde_json::Value| r["kind"] == "guarantee_payment")
        .collect();
    if alternative {
        assert!(calls.iter().any(|r| r["guarantee"] == 3
            && r["resource"] == GRAIN
            && r["paid"] == 1
            && r["tender_resource"] == TOKEN
            && r["tender_paid"] == 2));
        assert_eq!(a.book().balances()[&(HOME, Account::SettlementGain)], -1);
    }
    let calls: Vec<_> = calls.iter().filter(|r| r["creditor"] == PERSON).collect();
    assert!(!calls.is_empty());
    assert!(
        calls
            .iter()
            .all(|r| r["creditor"] == PERSON && r["resource"] == TOKEN)
    );
    assert_eq!(
        calls
            .iter()
            .map(|r| r["paid"].as_i64().unwrap())
            .sum::<i64>(),
        2
    );
}

#[test]
fn physical_wage_guarantees_reserve_member_and_collective_storage_before_payment() {
    use economics_compute_smoke::{
        household_governance::Governance, households, minting::FIREWOOD,
    };
    for pooled_room in [0, 2] {
        let (mut w, mut s) = fixture();
        w.employment[0].wage_per_unit.resource = FIREWOOD;
        s.balances.clear();
        s.balances.insert((SUPPLIER, FIREWOOD), 4);
        w.storage.capacities.insert(WORKER, 4);
        households::form(
            &mut w,
            &s,
            households::Agreement {
                id: 1,
                agent: 800,
                governance: Governance::contributed(WORKER),
                adults: vec![WORKER],
                membership: vec![],
                asset_sales: vec![],
                equipment_retirements: vec![],
                support: vec![],
                formed: 1,
                dwelling_process: None,
                admission: None,
            },
        )
        .unwrap();
        s.balances.insert((800, FIREWOOD), 2 - pooled_room);
        // Enough private room for two units. At most one can be paid if the
        // collective half-share would overflow; its odd-unit carry persists.
        let mut opening = Opening {
            exchange_values: [(FIREWOOD, 3)].into(),
            inventory: [((SUPPLIER, FIREWOOD), 4)].into(),
            ..Default::default()
        };
        if pooled_room == 0 {
            opening.inventory.insert((800, FIREWOOD), 2);
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = Audit::with_opening(&w, &s, COIN, opening.clone()).unwrap();
            through(&mut a, &mut sim, 2);
            let paid = if pooled_room == 0 { 1 } else { 4 };
            assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], paid);
            assert_eq!(
                sim.state.employment.earned[&(1, 1)].claim.outstanding(),
                4 - paid
            );
            assert_eq!(sim.state.credit.loans[&101].principal, paid);
            assert_eq!(sim.state.balance(800, FIREWOOD), 2 - pooled_room + paid / 2);
            assert_eq!(sim.state.balance(WORKER, FIREWOOD), (paid + 1) / 2);
            assert_eq!(
                a.book().balances()[&(SUPPLIER, Account::LoanReceivable(101))],
                i128::from(paid * 3)
            );
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
    }
}

#[test]
fn physical_land_guarantee_uses_guarantor_inventory_and_preserves_native_dues() {
    use economics_compute_smoke::{dues_accounting::Valuation, minting::FIREWOOD};
    let (mut w, mut s) = land_fixture();
    s.month = 12;
    w.agreements[0].payment.resource = FIREWOOD;
    s.balances.clear();
    s.balances.insert((SUPPLIER, FIREWOOD), 3);
    let opening = Opening {
        assets: [(900, 0)].into(),
        dues: Some(Valuation([(900, 3)].into())),
        exchange_values: [(FIREWOOD, 3)].into(),
        inventory: [((SUPPLIER, FIREWOOD), 3)].into(),
        ..Default::default()
    };
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = Audit::with_opening(&w, &s, COIN, opening.clone()).unwrap();
        through(&mut a, &mut sim, 14);
        let bill = &sim.state.obligations[&(900, 13)];
        assert_eq!(
            (bill.paid, bill.in_kind_paid, bill.outstanding()),
            (3, 3, 1)
        );
        assert_eq!(sim.state.balance(ISSUER, FIREWOOD), 0);
        assert_eq!(sim.state.balance(WORKER, FIREWOOD), 3);
        assert_eq!(sim.state.credit.loans[&101].principal, 3);
        let b = a.book().balances();
        assert_eq!(b[&(ISSUER, Account::DuesPayable(900, 13))], -3);
        assert_eq!(b[&(ISSUER, Account::LoanPayable(101))], -9);
        assert_eq!(b[&(WORKER, Account::Inventory(FIREWOOD))], 9);
        assert_eq!(b[&(SUPPLIER, Account::SettlementGain)], -6);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
    let mut mismatched = opening;
    mismatched.exchange_values.insert(FIREWOOD, 4);
    assert!(Audit::with_opening(&w, &s, COIN, mismatched).is_err());
}

#[test]
fn forward_guarantee_waits_for_delivery_window_and_keeps_historical_prepaid_cost() {
    use economics_compute_smoke::{forward::direct::Terms as Forward, minting::WHEAT};
    let (mut w, mut s) = fixture();
    w.employment.clear();
    w.prepaid_deliveries.push(Forward {
        id: 50,
        seller: ISSUER,
        buyer: WORKER,
        month: 1,
        due: 2,
        goods: Amount::new(WHEAT, 4),
        prepayment: Amount::new(COIN, 5),
    });
    w.recovery.guarantees[0].claim = GuaranteedClaim::Forward(50);
    s.balances.clear();
    s.balances.insert((WORKER, COIN), 5);
    s.balances.insert((ISSUER, WHEAT), 1);
    s.balances.insert((SUPPLIER, WHEAT), 3);
    let opening = Opening {
        inventory: [((ISSUER, WHEAT), 1), ((SUPPLIER, WHEAT), 3)].into(),
        exchange_values: [(WHEAT, 3)].into(),
        ..Default::default()
    };
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = Audit::with_opening(&w, &s, COIN, opening.clone()).unwrap();
        through(&mut a, &mut sim, 2);
        assert_eq!(sim.state.exchange.forwards[&50].delivered, 1);
        assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
        let mut resumed = Simulation::new(w.clone(), sim.state.clone(), backend).unwrap();
        let mut resumed_a = a.clone();
        through(&mut a, &mut sim, 4);
        through(&mut resumed_a, &mut resumed, 4);
        assert_eq!(sim.state, resumed.state);
        assert_eq!(a, resumed_a);
        assert_eq!(sim.state.exchange.forwards[&50].delivered, 4);
        assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], 3);
        assert_eq!(
            (
                sim.state.credit.loans[&101].opened,
                sim.state.credit.loans[&101].principal
            ),
            (3, 3)
        );
        assert_eq!(sim.state.balance(WORKER, WHEAT), 4);
        assert_eq!(sim.state.balance(ISSUER, WHEAT), 0);
        let b = a.book().balances();
        assert_eq!(b[&(WORKER, Account::Inventory(WHEAT))], 5);
        assert_eq!(b[&(ISSUER, Account::LoanPayable(101))], -9);
        assert_eq!(b[&(SUPPLIER, Account::LoanReceivable(101))], 9);
        assert_eq!(b[&(ISSUER, Account::Sales)], -5);
        assert_eq!(b[&(ISSUER, Account::CostOfSales)], 10);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
}

#[test]
fn overlapping_forward_guarantees_keep_rounding_and_relief_dates_consistent() {
    use economics_compute_smoke::{
        delivery_relief, finance::CollectionPolicy, forward::direct::Terms as Forward,
        minting::WHEAT,
    };
    for extended in [false, true] {
        let (mut w, mut s) = fixture();
        w.employment.clear();
        w.recovery.guarantee_policy = CollectionPolicy::Proportional;
        w.prepaid_deliveries.push(Forward {
            id: 50,
            seller: ISSUER,
            buyer: WORKER,
            month: 1,
            due: 2,
            goods: Amount::new(WHEAT, 4),
            prepayment: Amount::new(COIN, 5),
        });
        w.recovery.guarantees[0].claim = GuaranteedClaim::Forward(50);
        w.recovery.guarantees[0].through = 4;
        w.recovery.guarantees[0].delay_months = 1;
        w.agents.push(Agent {
            id: 99,
            name: "second grain guarantor".into(),
        });
        let mut second = w.recovery.guarantees[0].clone();
        second.id = 2;
        second.guarantor = 99;
        second.recourse = 102;
        w.recovery.guarantees.push(second);
        s.balances.clear();
        s.balances.insert((WORKER, COIN), 5);
        s.balances.insert((SUPPLIER, WHEAT), 2);
        s.balances.insert((99, WHEAT), 2);
        if extended {
            authorize(&mut w, 3);
            w.recovery.delivery_relief.push(delivery_relief::Terms {
                id: 90,
                proceeding: 1,
                expected_due: 2,
                contract: 50,
                debtor: ISSUER,
                creditor: WORKER,
                month: 3,
                expected_remaining: 4,
                action: delivery_relief::Action::Extend { due: 4 },
            });
        }
        let opening = Opening {
            inventory: [((SUPPLIER, WHEAT), 2), ((99, WHEAT), 2)].into(),
            exchange_values: [(WHEAT, 3)].into(),
            ..Default::default()
        };
        let run = |world: &World, backend| {
            let mut sim = Simulation::new(world.clone(), s.clone(), backend).unwrap();
            let mut a = Audit::with_opening(world, &s, COIN, opening.clone()).unwrap();
            through(&mut a, &mut sim, 5);
            assert_eq!(
                sim.state.exchange.forwards[&50].delivered,
                if extended { 0 } else { 4 }
            );
            if extended {
                assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
                assert_eq!(sim.state.exchange.forwards[&50].effective_due(), 4);
            } else {
                assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], 2);
                assert_eq!(sim.state.credit.recovery.paid_guarantees[&2], 2);
                assert_eq!(a.book().balances()[&(WORKER, Account::Inventory(WHEAT))], 5);
                assert_eq!(
                    sim.state.credit.loans[&101].principal + sim.state.credit.loans[&102].principal,
                    4
                );
            }
            (sim.state, sim.ledger, a)
        };
        let result = run(&w, Backend::CubeCpu);
        w.recovery.guarantees.reverse();
        assert_eq!(result, run(&w, Backend::Reference));
    }
}

#[test]
fn native_recourse_in_coin_estate_waits_for_actual_goods_without_conversion_or_discharge() {
    use economics_compute_smoke::{
        credit::Status, finance::ContractId, minting::FIREWOOD, recovery::Stage,
    };
    for discharge in [false, true] {
        let (mut w, mut s) = fixture();
        w.employment[0].wage_per_unit.resource = FIREWOOD;
        w.recovery.guarantees[0].from = 3;
        authorize(&mut w, 3);
        w.recovery.proceedings[0].discharge_deficiency = discharge;
        s.balances.clear();
        s.balances.insert((ISSUER, COIN), 5);
        s.balances.insert((SUPPLIER, FIREWOOD), 4);
        // Real work earns goods after the guarantee call. Month 4 Close wages
        // cannot fund that month's Due collection; repayment is month 5.
        w.capacity_overrides.insert((4, ISSUER), 1);
        w.employment.push(Terms {
            id: 2,
            employer: WORKER,
            worker: ISSUER,
            from: 4,
            through: 4,
            capacity: Amount::new(HOURS, 1),
            wage_per_unit: Amount::new(FIREWOOD, 4),
            on_arrears: ArrearsPolicy::Continue,
            rank: 0,
        });
        let opening = Opening {
            inventory: [((SUPPLIER, FIREWOOD), 4)].into(),
            exchange_values: [(FIREWOOD, 3)].into(),
            ..Default::default()
        };
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = Audit::with_opening(&w, &s, COIN, opening.clone()).unwrap();
            through(&mut a, &mut sim, 4);
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Active
            );
            assert_eq!(sim.state.credit.recovery.proceedings[&1].cash, 0);
            assert_eq!(sim.state.balance(ISSUER, COIN), 5);
            assert_eq!(sim.state.balance(ISSUER, FIREWOOD), 4);
            assert_eq!(sim.state.credit.loans[&101].principal, 4);
            assert_eq!(sim.state.credit.loans[&101].status, Status::Stayed);
            assert!(sim.ledger.iter().filter_map(|b| b.credit.as_ref())
                .flat_map(|b| &b.recovery).any(|r| matches!(r,
                    Receipt::ClosureDeferred { claims, .. } if claims.iter().any(|c|
                        c.contract == ContractId::Loan(101) && c.remaining == Amount::new(FIREWOOD, 4)))));
            let mut forged = sim.state.clone();
            let case = forged.credit.recovery.proceedings.get_mut(&1).unwrap();
            case.stage = Stage::Closed;
            case.closed = Some(4);
            assert!(Simulation::new(w.clone(), forged, backend).is_err());
            let mut resumed = Simulation::new(w.clone(), sim.state.clone(), backend).unwrap();
            let mut resumed_a = a.clone();
            through(&mut a, &mut sim, 5);
            through(&mut resumed_a, &mut resumed, 5);
            assert_eq!(sim.state, resumed.state);
            assert_eq!(a, resumed_a);
            assert_eq!(sim.state.credit.loans[&101].status, Status::Repaid);
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Closed
            );
            assert_eq!(sim.state.balance(SUPPLIER, FIREWOOD), 4);
            assert_eq!(sim.state.balance(ISSUER, COIN), 5);
            let b = a.book().balances();
            assert_eq!(
                b.get(&(ISSUER, Account::LoanPayable(101)))
                    .copied()
                    .unwrap_or(0),
                0
            );
            assert_eq!(
                b.get(&(SUPPLIER, Account::LoanReceivable(101)))
                    .copied()
                    .unwrap_or(0),
                0
            );
            assert_eq!(b[&(SUPPLIER, Account::Inventory(FIREWOOD))], 12);
            assert_eq!(b[&(ISSUER, Account::Cash)], 5);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
    }
}

fn posted_guarantee_fixture() -> (World, State) {
    use economics_compute_smoke::{
        laws::{AgreementForm, AgreementLimits},
        opportunities::{Action, PERSON_TYPE, Policy},
    };
    let (mut w, s) = fixture();
    w.recovery.posted_guarantees.insert(1);
    w.recovery.guarantee_applications.push(
        economics_compute_smoke::recovery::admission::Application {
            guarantee: 1,
            month: 2,
        },
    );
    w.transaction_policy = Some(Policy {
        authority: ISSUER,
        laws: vec![],
        agreement_forms: Some([AgreementForm::Guarantee].into()),
        agreement_limits: AgreementLimits::default(),
        membership_offers: vec![],
        membership_permissions: Default::default(),
        agent_types: [
            (ISSUER, PERSON_TYPE),
            (WORKER, PERSON_TYPE),
            (SUPPLIER, PERSON_TYPE),
        ]
        .into(),
        permissions: [
            (PERSON_TYPE, Action::CapacityTrade),
            (PERSON_TYPE, Action::Guarantee),
        ]
        .into(),
    });
    (w, s)
}

#[test]
fn posted_guarantee_uses_common_discovery_acceptance_and_later_performance() {
    use economics_compute_smoke::{
        agreements::{self, Identity},
        offers::{self, Id, Request},
        opportunities::{Action, PERSON_TYPE},
        settlement::{DEFAULT_EFFECT_LIMIT, commit},
    };
    let (w, s) = posted_guarantee_fixture();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
        assert!(
            offers::discover(&w, &s, SUPPLIER)
                .iter()
                .any(|o| o.id == Id::Guarantee(1))
        );
        assert!(
            !offers::discover(&w, &s, WORKER)
                .iter()
                .any(|o| o.id == Id::Guarantee(1))
        );
        assert!(
            !agreements::for_agent(&w, &s, SUPPLIER)
                .unwrap()
                .iter()
                .any(|v| v.identity() == Identity::Guarantee(1))
        );
        while sim.state.month < 2 || sim.state.phase != Phase::Acquire {
            a.step(&mut sim).unwrap();
        }
        assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
        let request = Request::new(Id::Guarantee(1), SUPPLIER);
        let before = sim.state.clone();
        let mut premature = before.clone();
        premature.credit.recovery.accepted_guarantees.insert(1, 2);
        assert!(Simulation::new(w.clone(), premature, backend).is_err());
        let prepared = offers::prepare(&sim, &[request]).unwrap();
        assert_eq!(sim.state, before);
        let mut forged = prepared.clone();
        forged
            .credit
            .as_mut()
            .unwrap()
            .after
            .recovery
            .accepted_guarantees
            .insert(1, 1);
        let mut unchanged = before.clone();
        assert!(commit(&w, &mut unchanged, &forged, backend, DEFAULT_EFFECT_LIMIT).is_err());
        assert_eq!(unchanged, before);
        a.step(&mut sim).unwrap();
        assert_eq!(*sim.ledger.last().unwrap(), prepared);
        assert_eq!(sim.state.credit.recovery.accepted_guarantees[&1], 2);
        assert_eq!(
            agreements::for_agent(&w, &sim.state, SUPPLIER)
                .unwrap()
                .iter()
                .find(|v| v.identity() == Identity::Guarantee(1))
                .unwrap()
                .accepted_month(),
            2
        );
        assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
        assert!(
            commit(
                &w,
                &mut sim.state.clone(),
                &prepared,
                backend,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        // A later withdrawal of formation permission cannot erase accepted exposure.
        sim.world
            .transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .remove(&(PERSON_TYPE, Action::Guarantee));
        sim.world
            .transaction_policy
            .as_mut()
            .unwrap()
            .agreement_forms = Some(Default::default());
        let mut resumed = Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
        let mut resumed_a = a.clone();
        through(&mut a, &mut sim, 3);
        through(&mut resumed_a, &mut resumed, 3);
        assert_eq!(sim.state, resumed.state);
        assert_eq!(a, resumed_a);
        assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], 3);
        assert_eq!(sim.state.credit.loans[&101].principal, 3);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
}

#[test]
fn unrecognized_or_unpermitted_posted_guarantees_leave_no_contingent_commitment() {
    use economics_compute_smoke::{
        opportunities::{Action, PERSON_TYPE},
        telemetry::{Config, Observer},
    };
    for unrecognized in [false, true] {
        let (mut w, s) = posted_guarantee_fixture();
        if unrecognized {
            w.transaction_policy.as_mut().unwrap().agreement_forms = Some(Default::default());
        } else {
            w.transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(PERSON_TYPE, Action::Guarantee));
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        let mut observer = Observer::new(
            vec![],
            "guarantee-admission",
            Config {
                settlement: true,
                agents: [SUPPLIER].into(),
                ..Default::default()
            },
        )
        .unwrap();
        observer.run_months(&mut sim, 3).unwrap();
        assert!(sim.state.credit.recovery.accepted_guarantees.is_empty());
        assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
        assert!(sim.state.credit.loans.is_empty());
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
        let records = String::from_utf8(observer.finish().unwrap()).unwrap();
        let admissions: Vec<serde_json::Value> = records
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .filter(|r: &serde_json::Value| r["kind"] == "guarantee_admission")
            .collect();
        assert_eq!(admissions.len(), 1);
        assert_eq!(admissions[0]["accepted"], false);
        assert_eq!(admissions[0]["rejection"], "NotPermitted");
    }
}

#[test]
fn unaccepted_household_guarantees_do_not_block_wind_down_but_accepted_ones_do() {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, dissolution as d},
    };
    for winding in [false, true] {
        let (mut w, mut s) = posted_guarantee_fixture();
        w.transaction_policy = None;
        let mut governance = Governance::contributed(SUPPLIER);
        governance.constitution.allow_dissolution = true;
        households::form(
            &mut w,
            &s,
            households::Agreement {
                id: 1,
                agent: 800,
                governance,
                adults: vec![SUPPLIER],
                membership: vec![],
                asset_sales: vec![],
                equipment_retirements: vec![],
                support: vec![],
                formed: 1,
                dwelling_process: None,
                admission: None,
            },
        )
        .unwrap();
        w.recovery.guarantees[0].guarantor = 800;
        s.balances.insert((SUPPLIER, COIN), 0);
        s.balances.insert((800, COIN), 3);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        let mut a = Audit::with_opening(&sim.world, &sim.state, COIN, Opening::default()).unwrap();
        through(&mut a, &mut sim, 1);
        assert!(!d::blockers(&sim.world, &sim.state, 800).contains(&d::Blocker::Guarantee));
        if winding {
            d::request(&mut sim.world, &sim.state, 800, SUPPLIER).unwrap();
        }
        if !winding {
            while sim.state.phase != Phase::Acquire {
                a.step(&mut sim).unwrap();
            }
            let prepared = economics_compute_smoke::offers::prepare(
                &sim,
                &[economics_compute_smoke::offers::Request::new(
                    economics_compute_smoke::offers::Id::Guarantee(1),
                    800,
                )],
            )
            .unwrap();
            a.step(&mut sim).unwrap();
            assert_eq!(sim.ledger.last(), Some(&prepared));
        }
        through(&mut a, &mut sim, 2);
        assert_eq!(
            sim.state
                .credit
                .recovery
                .accepted_guarantees
                .contains_key(&1),
            !winding
        );
        assert_eq!(
            d::blockers(&sim.world, &sim.state, 800).contains(&d::Blocker::Guarantee),
            !winding
        );
        if winding {
            d::finish(&mut sim.world, &sim.state, 800, SUPPLIER).unwrap();
            through(&mut a, &mut sim, 3);
            assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
        } else {
            d::request(&mut sim.world, &sim.state, 800, SUPPLIER).unwrap();
            assert!(d::finish(&mut sim.world, &sim.state, 800, SUPPLIER).is_err());
            through(&mut a, &mut sim, 3);
            assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], 3);
            assert_eq!(sim.state.credit.loans[&101].creditor, 800);
            assert_eq!(a.book().balances()[&(800, Account::LoanReceivable(101))], 3);
        }
    }
}

#[test]
fn guarantee_accepted_at_end_of_term_cannot_backdate_a_missed_call() {
    let (mut w, s) = posted_guarantee_fixture();
    w.recovery.guarantees[0].through = 2;
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut a = Audit::with_opening(&w, &s, COIN, Opening::default()).unwrap();
    through(&mut a, &mut sim, 3);
    assert_eq!(sim.state.credit.recovery.accepted_guarantees[&1], 2);
    assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
}

#[test]
fn land_guarantee_can_pay_accepted_coins_without_delivering_goods_or_converting_recourse() {
    use economics_compute_smoke::{
        activities::CoinPayment, dues_accounting::Valuation, finance::CollectionPolicy,
        minting::FIREWOOD, recovery::GuaranteeTender,
    };
    for policy in [CollectionPolicy::Stable, CollectionPolicy::Proportional] {
        for alternative in [false, true] {
            for rate in [2, 4] {
                let (mut w, mut s) = land_fixture();
                s.month = 12;
                w.agreements[0].payment.resource = FIREWOOD;
                w.activities.coin_payments.insert(
                    900,
                    CoinPayment {
                        resource: COIN,
                        coins_per_unit: rate,
                    },
                );
                w.recovery.guarantee_policy = policy;
                w.recovery.guarantees[0].tender = if alternative {
                    GuaranteeTender::AcceptedLandCoins
                } else {
                    GuaranteeTender::Native
                };
                s.balances.clear();
                s.balances.insert((SUPPLIER, COIN), 5);
                let opening = Opening {
                    assets: [(900, 0)].into(),
                    dues: Some(Valuation([(900, 3)].into())),
                    exchange_values: [(FIREWOOD, 3)].into(),
                    ..Default::default()
                };
                let run = |backend| {
                    let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                    let mut audit = Audit::with_opening(&w, &s, COIN, opening.clone()).unwrap();
                    through(&mut audit, &mut sim, 13);
                    let paid = if alternative { 5 / rate } else { 0 };
                    let bill = &sim.state.obligations[&(900, 13)];
                    assert_eq!(
                        (bill.paid, bill.in_kind_paid, bill.outstanding()),
                        (paid, 0, 4 - paid)
                    );
                    assert_eq!(sim.state.balance(SUPPLIER, COIN), 5 - paid * rate);
                    assert_eq!(sim.state.balance(WORKER, COIN), paid * rate);
                    assert_eq!(sim.state.balance(WORKER, FIREWOOD), 0);
                    if paid > 0 {
                        let l = &sim.state.credit.loans[&101];
                        assert_eq!(
                            (l.principal, l.denomination, l.debtor, l.creditor),
                            (paid, FIREWOOD, ISSUER, SUPPLIER)
                        );
                        let books = audit.book().balances();
                        assert_eq!(
                            books[&(SUPPLIER, Account::LoanReceivable(101))],
                            i128::from(paid * 3)
                        );
                        assert_eq!(
                            books[&(ISSUER, Account::LoanPayable(101))],
                            -i128::from(paid * 3)
                        );
                        let gain = paid * (3 - rate);
                        let account = if gain > 0 {
                            Account::SettlementGain
                        } else {
                            Account::SettlementLoss
                        };
                        assert_eq!(books[&(SUPPLIER, account.clone())], -i128::from(gain));
                        assert_eq!(books.get(&(ISSUER, account)).copied().unwrap_or(0), 0);
                        assert!(sim.ledger.iter().filter_map(|b| b.credit.as_ref()).flat_map(|b| &b.recovery).any(|r| matches!(r,
                            Receipt::Guaranteed { paid: q, tender, .. } if *q == paid && *tender == Amount::new(COIN, paid * rate))));
                    } else {
                        assert!(sim.state.credit.loans.is_empty());
                    }
                    let mut resumed =
                        Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
                    let mut ra = audit.clone();
                    through(&mut audit, &mut sim, 14);
                    through(&mut ra, &mut resumed, 14);
                    assert_eq!((&sim.state, &audit), (&resumed.state, &ra));
                    (sim.state, sim.ledger, audit)
                };
                assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
            }
        }
    }
}

#[test]
fn alternative_guarantees_allocate_whole_payment_lots_from_one_cash_pool() {
    use economics_compute_smoke::{
        activities::CoinPayment,
        dues_accounting::Valuation,
        finance::CollectionPolicy,
        minting::FIREWOOD,
        recovery::GuaranteeTender,
        settlement::{self, DEFAULT_EFFECT_LIMIT},
    };
    for first_rate in [2, 6] {
        for policy in [CollectionPolicy::Stable, CollectionPolicy::Proportional] {
            let (mut w, mut s) = land_fixture();
            s.month = 12;
            w.agreements[0].payment.resource = FIREWOOD;
            w.recovery.guarantees[0].tender = GuaranteeTender::AcceptedLandCoins;
            w.recovery.guarantee_policy = policy;
            let mut asset = w.assets.iter().find(|a| a.id == 900).unwrap().clone();
            asset.id = 901;
            w.assets.push(asset);
            let mut right = w.rights.iter().find(|r| r.id == 900).unwrap().clone();
            right.id = 901;
            right.asset = 901;
            w.rights.push(right);
            let mut agreement = w.agreements[0].clone();
            agreement.id = 901;
            agreement.right = 901;
            w.agreements.push(agreement);
            let mut g = w.recovery.guarantees[0].clone();
            g.id = 2;
            g.recourse = 102;
            g.claim = GuaranteedClaim::Land {
                agreement: 901,
                due: 13,
            };
            w.recovery.guarantees.push(g);
            for (id, rate) in [(900, first_rate), (901, 3)] {
                w.activities.coin_payments.insert(
                    id,
                    CoinPayment {
                        resource: COIN,
                        coins_per_unit: rate,
                    },
                );
            }
            s.balances.clear();
            s.balances.insert((SUPPLIER, COIN), 5);
            let opening = Opening {
                assets: [(900, 0), (901, 0)].into(),
                dues: Some(Valuation([(900, 3), (901, 3)].into())),
                exchange_values: [(FIREWOOD, 3)].into(),
                ..Default::default()
            };
            let mut invalid = w.clone();
            invalid.activities.coin_payments.remove(&900);
            assert!(Simulation::new(invalid, s.clone(), Backend::Reference).is_err());
            let mut invalid = w.clone();
            invalid
                .activities
                .coin_payments
                .get_mut(&900)
                .unwrap()
                .coins_per_unit = 0;
            assert!(Simulation::new(invalid, s.clone(), Backend::Reference).is_err());
            let run = |backend, reversed| {
                let mut world = w.clone();
                if reversed {
                    world.recovery.guarantees.reverse();
                    world.agreements.reverse();
                }
                let mut sim = Simulation::new(world, s.clone(), backend).unwrap();
                let mut audit =
                    Audit::with_opening(&sim.world, &sim.state, COIN, opening.clone()).unwrap();
                while (sim.state.month, sim.state.phase) != (13, Phase::Due) {
                    audit.step(&mut sim).unwrap();
                }
                let before = sim.state.clone();
                let before_audit = audit.clone();
                audit.step(&mut sim).unwrap();
                let expected = if first_rate == 6 {
                    [0, 1]
                } else if policy == CollectionPolicy::Stable {
                    [2, 0]
                } else {
                    [1, 1]
                };
                for (id, q) in [(900, expected[0]), (901, expected[1])] {
                    assert_eq!(sim.state.obligations[&(id, 13)].paid, q);
                    assert_eq!(sim.state.obligations[&(id, 13)].in_kind_paid, 0);
                }
                let paid_coins = expected[0] * first_rate + expected[1] * 3;
                assert_eq!(sim.state.balance(SUPPLIER, COIN), 5 - paid_coins);
                assert_eq!(sim.state.balance(WORKER, COIN), paid_coins);
                let mut forged = sim.ledger.last().unwrap().clone();
                let receipt = forged
                    .credit
                    .as_mut()
                    .unwrap()
                    .recovery
                    .iter_mut()
                    .find(|r| matches!(r, Receipt::Guaranteed { paid, .. } if *paid > 0))
                    .unwrap();
                if let Receipt::Guaranteed { tender, .. } = receipt {
                    tender.quantity += 1;
                }
                let mut candidate = before.clone();
                assert!(
                    settlement::commit(
                        &sim.world,
                        &mut candidate,
                        &forged,
                        backend,
                        DEFAULT_EFFECT_LIMIT
                    )
                    .is_err()
                );
                assert_eq!(candidate, before);
                let mut reporting = before_audit.clone();
                assert!(
                    reporting
                        .record(&sim.world, &before, &forged, &sim.state)
                        .is_err()
                );
                assert_eq!(reporting, before_audit);
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference, false), run(Backend::CubeCpu, true));
        }
    }
    let (mut w, s) = fixture();
    w.recovery.guarantees[0].tender = GuaranteeTender::AcceptedLandCoins;
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
}

#[test]
fn accepted_posted_guarantee_terms_cannot_be_rewritten_in_the_offer_catalog() {
    let (w, s) = posted_guarantee_fixture();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.credit.recovery.accepted_guarantees[&1], 2);
    assert_eq!(
        sim.state.credit.recovery.accepted_guarantee_terms[&1].terms,
        sim.world.recovery.guarantees[0]
    );
    for missing in [false, true] {
        let mut changed = sim.state.clone();
        if missing {
            changed.credit.recovery.accepted_guarantee_terms.clear();
        } else {
            changed
                .credit
                .recovery
                .accepted_guarantee_terms
                .get_mut(&1)
                .unwrap()
                .terms
                .cap += 1;
        }
        assert!(Simulation::new(sim.world.clone(), changed, Backend::Reference).is_err());
    }
    for mode in 0..5 {
        let mut changed = sim.world.clone();
        let g = &mut changed.recovery.guarantees[0];
        match mode {
            0 => g.cap += 1,
            1 => g.through += 1,
            2 => g.delay_months += 1,
            3 => g.recourse += 1,
            _ => g.priority += 1,
        }
        assert!(
            Simulation::new(changed, sim.state.clone(), Backend::Reference).is_err(),
            "accepted terms changed: {mode}"
        );
    }
}

#[test]
fn posted_land_guarantee_captures_the_accepted_external_tender_rate() {
    use economics_compute_smoke::{
        activities::CoinPayment, commitments, recovery::GuaranteeTender, settlement,
    };
    let (mut w, s) = posted_guarantee_fixture();
    w.assets.push(Asset {
        id: 900,
        owner: WORKER,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: 900,
        holder: ISSUER,
        asset: 900,
        from: 1,
        through: 24,
        output_owner: ISSUER,
    });
    w.agreements.push(commitments::Agreement {
        id: 900,
        right: 900,
        creditor: WORKER,
        debtor: ISSUER,
        activated: 1,
        payment: Amount::new(minting::FIREWOOD, 1),
    });
    w.activities.coin_payments.insert(
        900,
        CoinPayment {
            resource: COIN,
            coins_per_unit: 2,
        },
    );
    let g = &mut w.recovery.guarantees[0];
    g.claim = GuaranteedClaim::Land {
        agreement: 900,
        due: 13,
    };
    g.through = 24;
    g.tender = GuaranteeTender::AcceptedLandCoins;
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while (sim.state.month, sim.state.phase) != (2, Phase::Acquire) {
            sim.step().unwrap();
        }
        let before = sim.state.clone();
        sim.step().unwrap();
        let accepted = &sim.state.credit.recovery.accepted_guarantee_terms[&1];
        assert_eq!(
            accepted.tender,
            Some(CoinPayment {
                resource: COIN,
                coins_per_unit: 2
            })
        );
        let mut altered = sim.ledger.last().unwrap().clone();
        altered
            .credit
            .as_mut()
            .unwrap()
            .after
            .recovery
            .accepted_guarantee_terms
            .clear();
        let mut rejected = before.clone();
        assert!(
            settlement::commit(&w, &mut rejected, &altered, backend, sim.effect_limit).is_err()
        );
        assert_eq!(rejected, before);
        let mut changed = sim.world.clone();
        changed
            .activities
            .coin_payments
            .get_mut(&900)
            .unwrap()
            .coins_per_unit = 3;
        assert!(Simulation::new(changed, sim.state.clone(), backend).is_err());
        let mut resumed = Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
        let prefix = sim.ledger.len();
        sim.run_months(2).unwrap();
        resumed.run_months(2).unwrap();
        assert_eq!(
            (&sim.state, &sim.ledger[prefix..]),
            (&resumed.state, &resumed.ledger[..])
        );
        (sim.state, sim.ledger)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn agreed_coin_wage_guarantees_keep_native_claims_and_actual_payment_statements() {
    use economics_compute_smoke::{minting::FIREWOOD, recovery::GuaranteeTender, settlement};
    const REPORT_COIN: ResourceId = 900;
    for reporting_other in [false, true] {
        for rate in [2, 4] {
            for funds in [rate - 1, 5] {
                let (mut w, mut s) = fixture();
                w.employment[0].wage_per_unit.resource = FIREWOOD;
                s.balances.insert((SUPPLIER, COIN), funds);
                w.recovery.guarantees[0].tender = GuaranteeTender::AgreedCoins {
                    resource: COIN,
                    coins_per_unit: rate,
                };
                let reporting = if reporting_other { REPORT_COIN } else { COIN };
                let mut opening = Opening {
                    exchange_values: [(FIREWOOD, 3)].into(),
                    ..Opening::default()
                };
                if reporting_other {
                    w.resources.push(Resource {
                        id: REPORT_COIN,
                        name: "reporting unit".into(),
                        kind: ResourceKind::Stock,
                    });
                    opening.exchange_values.insert(COIN, 2);
                    opening
                        .inventory
                        .insert((SUPPLIER, COIN), i128::from(funds));
                }
                let run = |backend| {
                    let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                    let mut audit =
                        Audit::with_opening(&w, &s, reporting, opening.clone()).unwrap();
                    through(&mut audit, &mut sim, 1);
                    let checkpoint = (sim.clone(), audit.clone());
                    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
                    let mut before_due = None;
                    while sim.state.month <= 2 {
                        if sim.state.phase == Phase::Due {
                            before_due = Some(sim.state.clone());
                        }
                        audit.step(&mut sim).unwrap();
                    }
                    let paid = (funds / rate).min(4);
                    assert_eq!(sim.state.balance(WORKER, COIN), paid * rate);
                    assert_eq!(sim.state.balance(SUPPLIER, COIN), funds - paid * rate);
                    assert_eq!(sim.state.balance(WORKER, FIREWOOD), 0);
                    assert_eq!(
                        sim.state.employment.earned[&(1, 1)].claim.outstanding(),
                        4 - paid
                    );
                    assert_eq!(
                        sim.state.credit.loans.get(&101).map_or(0, |l| l.principal),
                        paid
                    );
                    if paid > 0 {
                        let l = &sim.state.credit.loans[&101];
                        assert_eq!(
                            (l.denomination, l.opened, l.debtor, l.creditor),
                            (FIREWOOD, 2, ISSUER, SUPPLIER)
                        );
                        assert_eq!(
                            audit.book().balances()[&(SUPPLIER, Account::LoanReceivable(101))],
                            i128::from(paid * 3)
                        );
                    }
                    assert_eq!(
                        audit.book().balances()[&(WORKER, Account::ServiceIncome)],
                        -12
                    );
                    let difference = paid * (rate * if reporting_other { 2 } else { 1 } - 3);
                    let account = if difference > 0 {
                        Account::SettlementGain
                    } else {
                        Account::SettlementLoss
                    };
                    assert_eq!(
                        audit
                            .book()
                            .balances()
                            .get(&(WORKER, account))
                            .copied()
                            .unwrap_or(0),
                        -i128::from(difference)
                    );
                    let due = sim
                        .ledger
                        .iter()
                        .find(|b| b.month == 2 && b.phase == Phase::Due)
                        .unwrap();
                    let mut forged = due.clone();
                    let r = forged
                        .credit
                        .as_mut()
                        .unwrap()
                        .recovery
                        .iter_mut()
                        .find(|r| matches!(r, Receipt::Guaranteed { guarantee: 1, .. }))
                        .unwrap();
                    let Receipt::Guaranteed { tender, .. } = r else {
                        unreachable!()
                    };
                    tender.quantity += 1;
                    let before = before_due.unwrap();
                    let mut unchanged = before.clone();
                    assert!(
                        settlement::commit(&w, &mut unchanged, &forged, backend, sim.effect_limit)
                            .is_err()
                    );
                    assert_eq!(unchanged, before);
                    let (saved, mut ra) = checkpoint;
                    let mut resumed = Simulation::new(saved.world, saved.state, backend).unwrap();
                    through(&mut ra, &mut resumed, 2);
                    assert_eq!((&sim.state, &audit), (&resumed.state, &ra));
                    (sim.state, sim.ledger, audit)
                };
                assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
            }
        }
    }
}

#[test]
fn household_wage_guarantor_pools_actual_coins_once_and_retains_native_recourse_at_exit() {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, dissolution as d},
        minting::FIREWOOD,
        recovery::GuaranteeTender,
    };
    const HOME: AgentId = 800;
    for cash in [1, 5] {
        let (mut w, mut s) = fixture();
        w.employment[0].wage_per_unit.resource = FIREWOOD;
        w.recovery.guarantees[0].guarantor = HOME;
        w.recovery.guarantees[0].through = 2;
        w.recovery.guarantees[0].tender = GuaranteeTender::AgreedCoins {
            resource: COIN,
            coins_per_unit: 2,
        };
        let mut governance = Governance::contributed(WORKER);
        governance.constitution.allow_dissolution = true;
        households::form(
            &mut w,
            &s,
            households::Agreement {
                id: 1,
                agent: HOME,
                adults: vec![WORKER],
                governance,
                formed: 1,
                dwelling_process: None,
                admission: None,
                membership: vec![],
                asset_sales: vec![],
                equipment_retirements: vec![],
                support: vec![],
            },
        )
        .unwrap();
        s.balances.insert((HOME, COIN), cash);
        s.balances.insert((WORKER, COIN), 10);
        let opening = Opening {
            exchange_values: [(FIREWOOD, 3)].into(),
            ..Opening::default()
        };
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut audit = Audit::with_opening(&w, &s, COIN, opening.clone()).unwrap();
            through(&mut audit, &mut sim, 2);
            let paid = cash / 2;
            // Returned household contributions cannot finance another call in
            // the same Due boundary, and private opening cash never funds it.
            assert_eq!(sim.state.balance(HOME, COIN), cash - paid);
            assert_eq!(sim.state.balance(WORKER, COIN), 10 + paid);
            assert_eq!(
                sim.state.employment.earned[&(1, 1)].claim.outstanding(),
                4 - paid
            );
            assert_eq!(
                sim.state.credit.loans.get(&101).map_or(0, |l| l.principal),
                paid
            );
            assert_eq!(sim.state.balance(HOME, FIREWOOD), 0);
            assert_eq!(sim.state.balance(WORKER, FIREWOOD), 0);
            let due = sim
                .ledger
                .iter()
                .find(|b| b.month == 2 && b.phase == Phase::Due)
                .unwrap();
            let h = due.household.as_ref().unwrap();
            assert!(h.after.iter().all(|e| e.account.1 == COIN));
            assert_eq!(
                h.after
                    .iter()
                    .filter(|e| e.account == (HOME, COIN))
                    .map(|e| e.delta)
                    .sum::<i32>(),
                paid
            );
            if paid > 0 {
                assert_eq!(
                    audit.book().balances()[&(HOME, Account::LoanReceivable(101))],
                    i128::from(paid * 3)
                );
                assert_eq!(
                    audit.book().balances()[&(ISSUER, Account::LoanPayable(101))],
                    -i128::from(paid * 3)
                );
            }
            assert_eq!(
                audit.book().balances()[&(WORKER, Account::WagesReceivable(1, 1))],
                i128::from((4 - paid) * 3)
            );
            assert_eq!(
                d::blockers(&sim.world, &sim.state, HOME).contains(&d::Blocker::Loan),
                paid > 0
            );
            d::request(&mut sim.world, &sim.state, HOME, WORKER).unwrap();
            let mut resumed =
                Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
            let mut ra = audit.clone();
            through(&mut audit, &mut sim, 3);
            through(&mut ra, &mut resumed, 3);
            assert_eq!((&sim.state, &audit), (&resumed.state, &ra));
            if paid > 0 {
                assert!(d::finish(&mut sim.world, &sim.state, HOME, WORKER).is_err());
            } else {
                d::finish(&mut sim.world, &sim.state, HOME, WORKER).unwrap();
                assert_eq!(sim.state.balance(WORKER, COIN), 11);
                assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
            }
            (sim.world, sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn forward_coin_guarantees_discharge_native_units_without_inventing_delivery() {
    use economics_compute_smoke::{
        forward::direct::Terms as Forward, minting::WHEAT, recovery::GuaranteeTender, settlement,
    };
    const PAYMENT: ResourceId = 901;
    const SECOND: AgentId = 99;
    for other_currency in [false, true] {
        for advance in [1, 5] {
            for rate in [1, 4] {
                for funds in [0, 9] {
                    let (mut w, mut s) = fixture();
                    w.employment.clear();
                    w.prepaid_deliveries.push(Forward {
                        id: 50,
                        seller: ISSUER,
                        buyer: WORKER,
                        month: 1,
                        due: 2,
                        goods: Amount::new(WHEAT, 4),
                        prepayment: Amount::new(COIN, advance),
                    });
                    let payment = if other_currency { PAYMENT } else { COIN };
                    let g = &mut w.recovery.guarantees[0];
                    g.claim = GuaranteedClaim::Forward(50);
                    g.tender = GuaranteeTender::AgreedCoins {
                        resource: payment,
                        coins_per_unit: rate,
                    };
                    g.priority = 0;
                    // The second native guarantee follows substitute tender in the
                    // same boundary: inventory receives its exact remaining basis.
                    let mut native = g.clone();
                    native.id = 2;
                    native.guarantor = SECOND;
                    native.recourse = 102;
                    native.priority = 1;
                    native.tender = GuaranteeTender::Native;
                    w.recovery.guarantees.push(native);
                    w.agents.push(Agent {
                        id: SECOND,
                        name: "native guarantor".into(),
                    });
                    s.balances.clear();
                    s.balances.insert((WORKER, COIN), advance);
                    s.balances.insert((ISSUER, WHEAT), 1);
                    s.balances.insert((SECOND, WHEAT), 1);
                    s.balances.insert((SUPPLIER, payment), funds);
                    let mut opening = Opening {
                        inventory: [((ISSUER, WHEAT), 1), ((SECOND, WHEAT), 1)].into(),
                        exchange_values: [(WHEAT, 3)].into(),
                        ..Opening::default()
                    };
                    if other_currency {
                        w.resources.push(Resource {
                            id: PAYMENT,
                            name: "payment coin".into(),
                            kind: ResourceKind::Stock,
                        });
                        opening.exchange_values.insert(PAYMENT, 2);
                        if funds > 0 {
                            opening
                                .inventory
                                .insert((SUPPLIER, PAYMENT), i128::from(funds));
                        }
                    }
                    let run = |backend| {
                        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                        let mut audit = Audit::with_opening(&w, &s, COIN, opening.clone()).unwrap();
                        through(&mut audit, &mut sim, 2);
                        let saved = (sim.clone(), audit.clone());
                        let mut before_due = None;
                        while sim.state.month <= 3 {
                            if sim.state.phase == Phase::Due {
                                before_due = Some(sim.state.clone());
                            }
                            audit.step(&mut sim).unwrap();
                        }
                        let substituted = (funds / rate).min(3);
                        let native = (3 - substituted).min(1);
                        let c = &sim.state.exchange.forwards[&50];
                        assert_eq!(
                            (c.delivered, c.substituted, c.claim().outstanding()),
                            (1 + native, substituted, 3 - substituted - native)
                        );
                        assert_eq!(sim.state.balance(WORKER, WHEAT), 1 + native);
                        assert_eq!(sim.state.balance(WORKER, payment), substituted * rate);
                        assert_eq!(
                            sim.state.credit.loans.get(&101).map_or(0, |l| l.principal),
                            substituted
                        );
                        let book = audit.book().balances();
                        let basis_before = advance / 4;
                        let basis_after_coins = advance * (1 + substituted) / 4;
                        let basis_after_native = advance * (1 + substituted + native) / 4;
                        assert_eq!(
                            book.get(&(WORKER, Account::Inventory(WHEAT)))
                                .copied()
                                .unwrap_or(0),
                            i128::from(basis_before + basis_after_native - basis_after_coins)
                        );
                        let difference = basis_after_coins
                            - basis_before
                            - substituted * rate * if other_currency { 2 } else { 1 };
                        let account = if difference > 0 {
                            Account::SettlementLoss
                        } else {
                            Account::SettlementGain
                        };
                        assert_eq!(
                            book.get(&(WORKER, account)).copied().unwrap_or(0),
                            i128::from(difference)
                        );
                        assert_eq!(
                            book.get(&(WORKER, Account::ForwardPrepayment(50)))
                                .copied()
                                .unwrap_or(0),
                            i128::from(advance - basis_after_native)
                        );
                        assert_eq!(
                            book.get(&(SUPPLIER, Account::LoanReceivable(101)))
                                .copied()
                                .unwrap_or(0),
                            i128::from(substituted * 3)
                        );
                        let due = sim
                            .ledger
                            .iter()
                            .find(|b| b.month == 3 && b.phase == Phase::Due)
                            .unwrap();
                        let mut forged = due.clone();
                        let change = forged
                            .credit
                            .as_mut()
                            .unwrap()
                            .forward_changes
                            .get_mut(&50)
                            .unwrap();
                        change.delivered += change.substituted;
                        change.substituted = 0;
                        if substituted > 0 {
                            let before = before_due.unwrap();
                            let mut unchanged = before.clone();
                            assert!(
                                settlement::commit(
                                    &w,
                                    &mut unchanged,
                                    &forged,
                                    backend,
                                    sim.effect_limit
                                )
                                .is_err()
                            );
                            assert_eq!(before, unchanged);
                            let mut bad = sim.state.clone();
                            let c = bad.exchange.forwards.get_mut(&50).unwrap();
                            c.delivered += c.substituted;
                            c.substituted = 0;
                            assert!(Simulation::new(w.clone(), bad, backend).is_err());
                        }
                        let (saved, mut ra) = saved;
                        let mut resumed =
                            Simulation::new(saved.world, saved.state, backend).unwrap();
                        through(&mut ra, &mut resumed, 3);
                        assert_eq!((&sim.state, &audit), (&resumed.state, &ra));
                        (sim.state, sim.ledger, audit)
                    };
                    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
                }
            }
        }
    }
}

#[test]
fn forward_relief_uses_same_boundary_substitute_performance_and_rejects_stale_consent() {
    use economics_compute_smoke::{
        delivery_relief, forward::direct::Terms as Forward, minting::WHEAT,
        recovery::GuaranteeTender,
    };
    for stale in [false, true] {
        let (mut w, mut s) = fixture();
        w.employment.clear();
        w.prepaid_deliveries.push(Forward {
            id: 50,
            seller: ISSUER,
            buyer: WORKER,
            month: 1,
            due: 2,
            goods: Amount::new(WHEAT, 4),
            prepayment: Amount::new(COIN, 5),
        });
        let g = &mut w.recovery.guarantees[0];
        g.claim = GuaranteedClaim::Forward(50);
        g.cap = 2;
        g.tender = GuaranteeTender::AgreedCoins {
            resource: COIN,
            coins_per_unit: 2,
        };
        s.balances.clear();
        s.balances.insert((WORKER, COIN), 5);
        s.balances.insert((SUPPLIER, COIN), 4);
        authorize(&mut w, 3);
        w.recovery.delivery_relief.push(delivery_relief::Terms {
            id: 90,
            proceeding: 1,
            contract: 50,
            debtor: ISSUER,
            creditor: WORKER,
            month: 3,
            expected_due: 2,
            expected_remaining: if stale { 4 } else { 2 },
            action: delivery_relief::Action::WriteOff { quantity: 2 },
        });
        let opening = Opening {
            exchange_values: [(WHEAT, 3)].into(),
            ..Opening::default()
        };
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = Audit::with_opening(&w, &s, COIN, opening.clone()).unwrap();
            through(&mut a, &mut sim, 2);
            let (saved, mut ra) = (sim.clone(), a.clone());
            through(&mut a, &mut sim, 4);
            let c = &sim.state.exchange.forwards[&50];
            assert_eq!(
                (
                    c.delivered,
                    c.substituted,
                    c.written_off(),
                    c.claim().outstanding()
                ),
                (0, 2, if stale { 0 } else { 2 }, if stale { 2 } else { 0 })
            );
            assert_eq!(sim.state.balance(WORKER, WHEAT), 0);
            assert_eq!(sim.state.balance(WORKER, COIN), 4);
            assert_eq!(sim.state.credit.loans[&101].principal, 2);
            assert_eq!(sim.state.balance(ESTATE, COIN), 0);
            assert_eq!(sim.state.balance(ISSUER, COIN), 5);
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                economics_compute_smoke::recovery::Stage::Active
            );
            if !stale {
                assert_eq!(c.relief[0].substituted, 2);
                assert_eq!(a.book().balances()[&(WORKER, Account::CreditLoss)], 3);
            }
            assert!(sim.ledger.iter().flat_map(|b| b.credit.iter()).flat_map(|c| &c.recovery).any(|r|
                matches!(r, Receipt::DeliveryRelief { applied, .. } if *applied != stale)));
            let mut resumed = Simulation::new(saved.world, saved.state, backend).unwrap();
            through(&mut ra, &mut resumed, 4);
            assert_eq!((&sim.state, &a), (&resumed.state, &ra));
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn member_guarantee_of_household_forward_preserves_native_debt_after_delivery_relief() {
    use economics_compute_smoke::{
        delivery_relief,
        forward::direct::Terms as Forward,
        household_governance::Governance,
        households::{self, dissolution as d},
        minting::WHEAT,
        recovery::GuaranteeTender,
    };
    const HOME: AgentId = 800;
    for funded in [false, true] {
        let (mut w, mut s) = fixture();
        let mut governance = Governance::contributed(WORKER);
        governance.constitution.allow_dissolution = true;
        households::form(
            &mut w,
            &s,
            households::Agreement {
                id: 1,
                agent: HOME,
                adults: vec![WORKER],
                governance,
                formed: 1,
                dwelling_process: None,
                admission: None,
                membership: vec![],
                asset_sales: vec![],
                equipment_retirements: vec![],
                support: vec![],
            },
        )
        .unwrap();
        w.employment.clear();
        w.prepaid_deliveries.push(Forward {
            id: 50,
            seller: HOME,
            buyer: SUPPLIER,
            month: 1,
            due: 2,
            goods: Amount::new(WHEAT, 4),
            prepayment: Amount::new(COIN, 5),
        });
        let g = &mut w.recovery.guarantees[0];
        g.claim = GuaranteedClaim::Forward(50);
        g.guarantor = WORKER;
        g.cap = 2;
        g.through = 3;
        g.tender = GuaranteeTender::AgreedCoins {
            resource: COIN,
            coins_per_unit: 2,
        };
        s.balances.clear();
        s.balances.insert((SUPPLIER, COIN), 5);
        s.balances
            .insert((WORKER, COIN), if funded { 4 } else { 0 });
        authorize(&mut w, 3);
        w.recovery.proceedings[0].debtor = HOME;
        w.recovery.delivery_relief.push(delivery_relief::Terms {
            id: 90,
            proceeding: 1,
            contract: 50,
            debtor: HOME,
            creditor: SUPPLIER,
            month: 3,
            expected_due: 2,
            expected_remaining: if funded { 2 } else { 4 },
            action: delivery_relief::Action::WriteOff {
                quantity: if funded { 2 } else { 4 },
            },
        });
        let opening = Opening {
            exchange_values: [(WHEAT, 3)].into(),
            ..Opening::default()
        };
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = Audit::with_opening(&w, &s, COIN, opening.clone()).unwrap();
            through(&mut a, &mut sim, 1);
            d::request(&mut sim.world, &sim.state, HOME, WORKER).unwrap();
            through(&mut a, &mut sim, 2);
            let (saved, mut ra) = (sim.clone(), a.clone());
            through(&mut a, &mut sim, 4);
            let c = &sim.state.exchange.forwards[&50];
            assert_eq!(
                (c.delivered, c.substituted, c.claim().outstanding()),
                (0, if funded { 2 } else { 0 }, 0)
            );
            assert_eq!(sim.state.balance(SUPPLIER, WHEAT), 0);
            assert_eq!(sim.state.balance(HOME, COIN), if funded { 5 } else { 0 });
            assert_eq!(sim.state.balance(WORKER, COIN), if funded { 0 } else { 5 });
            assert_eq!(
                sim.state.balance(SUPPLIER, COIN),
                if funded { 4 } else { 0 }
            );
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                if funded {
                    economics_compute_smoke::recovery::Stage::Active
                } else {
                    economics_compute_smoke::recovery::Stage::Closed
                }
            );
            assert_eq!(
                d::blockers(&sim.world, &sim.state, HOME).contains(&d::Blocker::Loan),
                funded
            );
            if funded {
                let l = &sim.state.credit.loans[&101];
                assert_eq!(
                    (l.creditor, l.debtor, l.denomination, l.principal),
                    (WORKER, HOME, WHEAT, 2)
                );
                let b = a.book().balances();
                assert_eq!(b[&(WORKER, Account::LoanReceivable(101))], 6);
                assert_eq!(b[&(HOME, Account::LoanPayable(101))], -6);
                assert!(d::finish(&mut sim.world, &sim.state, HOME, WORKER).is_err());
            }
            let mut resumed = Simulation::new(saved.world, saved.state, backend).unwrap();
            through(&mut ra, &mut resumed, 4);
            assert_eq!((&sim.state, &a), (&resumed.state, &ra));
            if !funded {
                d::finish(&mut sim.world, &sim.state, HOME, WORKER).unwrap();
                assert_eq!(sim.state.balance(WORKER, COIN), 5);
            }
            (sim.world, sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
