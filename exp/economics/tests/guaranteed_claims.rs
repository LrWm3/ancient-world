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
    use economics_compute_smoke::{
        commitments::Agreement,
        credit::{Advance, LoanOffer},
        finance::CollectionPolicy,
        households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
        scenario::{LABOR, PERSON, STATE_AGENT, TOKEN},
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
        payment: Amount::new(TOKEN, 4),
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
    w.recovery.guarantee_policy = CollectionPolicy::Proportional;
    let mut a = Audit::with_opening(
        &w,
        &s,
        TOKEN,
        Opening {
            assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
            dues: Some(Default::default()),
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
            agents: [PERSON].into(),
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
    assert_eq!(sim.state.obligations[&(900, 13)].outstanding(), 2);
    for id in [101, 102, 103] {
        assert_eq!(sim.state.credit.loans[&id].principal, 2);
        assert_eq!(a.book().balances()[&(HOME, Account::LoanReceivable(id))], 2);
        assert_eq!(a.book().balances()[&(DEBTOR, Account::LoanPayable(id))], -2);
    }
    let records = String::from_utf8(observer.finish().unwrap()).unwrap();
    let calls: Vec<serde_json::Value> = records
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .filter(|r: &serde_json::Value| r["kind"] == "guarantee_payment")
        .collect();
    assert!(!calls.is_empty());
    assert!(calls.iter().all(|r| r["creditor"] == PERSON));
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
