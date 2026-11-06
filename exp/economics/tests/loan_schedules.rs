use economics_compute_smoke::{
    accounting::Account,
    agreements,
    compute::Backend,
    credit::{self, Advance, Event, LoanOffer, RepaymentSchedule, Status},
    financial_reporting::Audit,
    model::*,
    scenario::{self, PERSON, PLOT, STATE_AGENT, TOKEN},
    simulation::Simulation,
};

fn fixture(
    schedule: RepaymentSchedule,
    principal: i32,
    cash: i32,
    rate: u32,
    term: u32,
) -> (World, State) {
    let (mut w, _) = scenario::baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights.clear();
    w.assets.clear();
    w.condition_rules.clear();
    w.resources = vec![Resource {
        id: TOKEN,
        name: "cash".into(),
        kind: ResourceKind::Stock,
    }];
    w.lending = vec![Advance {
        id: 1,
        debtor: PERSON,
        principal,
        month: 1,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            repayment: schedule,
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: principal,
            monthly_rate_bps: rate,
            term_months: term,
            grace_months: 2,
        },
    }];
    let mut s = State::empty();
    s.balances.insert((STATE_AGENT, TOKEN), principal);
    s.balances.insert((PERSON, TOKEN), cash);
    (w, s)
}

fn audited(w: World, s: State, backend: Backend) -> (Simulation, Audit) {
    let audit = Audit::new(&w, &s, TOKEN).unwrap();
    (Simulation::new(w, s, backend).unwrap(), audit)
}

fn until(sim: &mut Simulation, audit: &mut Audit, next_month: u32) {
    while sim.state.month < next_month {
        audit.step(sim).unwrap();
    }
    assert_eq!(sim.state.phase, Phase::Open);
}

#[test]
fn maturity_defers_collection_but_not_interest_or_audited_liabilities() {
    let (w, s) = fixture(RepaymentSchedule::AtMaturity, 100, 5, 100, 3);
    let (mut sim, mut audit) = audited(w, s, Backend::Reference);
    until(&mut sim, &mut audit, 2);
    let before = sim.clone();
    for _ in 0..2 {
        assert_eq!(
            credit::projection::dues(&sim.world, &sim.state, PERSON, TOKEN, 4).unwrap(),
            [(4, 103)].into()
        );
    }
    assert_eq!(sim.state, before.state);
    assert_eq!(sim.ledger, before.ledger);
    for month in 2..=3 {
        audit.step(&mut sim).unwrap(); // Open -> Due
        assert_eq!(sim.state.phase, Phase::Due);
        audit.step(&mut sim).unwrap();
        let loan = &sim.state.credit.loans[&1];
        assert_eq!(loan.last_accrued, month);
        assert_eq!(loan.principal, 100);
        assert_eq!(loan.interest, (month - 1) as i32);
        assert_eq!(loan.first_unpaid, None);
        assert_eq!(loan.due(month).unwrap(), 0);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 105);
        assert_eq!(
            audit.book().balances()[&(STATE_AGENT, Account::InterestReceivable(1))],
            i128::from(month - 1)
        );
        assert_eq!(
            audit.book().balances()[&(PERSON, Account::InterestPayable(1))],
            -i128::from(month - 1)
        );
        let views = agreements::for_agent(&sim.world, &sim.state, PERSON).unwrap();
        let agreements::View::Loan(view) = &views[0] else {
            panic!("missing loan")
        };
        assert!(view.claim().unwrap().is_none());
        until(&mut sim, &mut audit, month + 1);
    }
    audit.step(&mut sim).unwrap();
    audit.step(&mut sim).unwrap(); // Maturity Due: interest first, then principal.
    assert_eq!(sim.state.credit.loans[&1].status, Status::Repaid);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 103);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 2);
    let paid: Vec<_> = sim
        .ledger
        .iter()
        .filter_map(|b| b.credit.as_ref())
        .flat_map(|b| &b.events)
        .filter_map(|e| match e {
            Event::Paid {
                loan,
                interest,
                principal,
            } => Some((*loan, *interest, *principal)),
            _ => None,
        })
        .collect();
    assert_eq!(paid, vec![(1, 3, 100)]);
    assert_eq!(
        audit
            .book()
            .statements(STATE_AGENT, 1, 4)
            .unwrap()
            .net_income,
        3
    );
    assert_eq!(
        audit.book().statements(PERSON, 1, 4).unwrap().net_income,
        -3
    );
    assert_eq!(sim.state.balances.values().sum::<i32>(), 105);
}

#[test]
fn identical_openings_compare_schedules_and_one_month_terms_agree() {
    assert_eq!(
        RepaymentSchedule::default(),
        RepaymentSchedule::Installments
    );
    for term in [1, 3] {
        let (wi, si) = fixture(RepaymentSchedule::Installments, 100, 5, 100, term);
        let (wm, sm) = fixture(RepaymentSchedule::AtMaturity, 100, 5, 100, term);
        assert_eq!(si, sm);
        let mut installments = Simulation::new(wi, si, Backend::Reference).unwrap();
        let mut maturity = Simulation::new(wm, sm, Backend::Reference).unwrap();
        installments.run_months(2).unwrap();
        maturity.run_months(2).unwrap();
        if term == 1 {
            assert_eq!(installments.state.balances, maturity.state.balances);
            assert_eq!(maturity.state.credit.loans[&1].status, Status::Repaid);
        } else {
            assert_eq!(installments.state.credit.loans[&1].principal, 67);
            assert_eq!(installments.state.balance(PERSON, TOKEN), 71);
            assert_eq!(maturity.state.credit.loans[&1].principal, 100);
            assert_eq!(maturity.state.balance(PERSON, TOKEN), 105);
        }
    }
}

#[test]
fn scarce_maturity_payment_is_interest_first_and_unsecured_arrears_remain_finite() {
    let (w, s) = fixture(RepaymentSchedule::AtMaturity, 100, 0, 100, 3);
    let (mut sim, mut audit) = audited(w, s, Backend::Reference);
    until(&mut sim, &mut audit, 5);
    let loan = &sim.state.credit.loans[&1];
    assert_eq!((loan.principal, loan.interest), (3, 0));
    assert_eq!(loan.first_unpaid, Some(4));
    assert_eq!(loan.status, Status::Active);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 100);
    until(&mut sim, &mut audit, 6);
    let loan = &sim.state.credit.loans[&1];
    assert_eq!(loan.principal, 3);
    assert_eq!(loan.last_accrued, 5);
    assert_eq!(loan.interest_remainder, 300);
    assert_eq!(loan.first_unpaid, Some(4));
    assert_eq!(sim.state.balances.values().sum::<i32>(), 100);
}

#[test]
fn future_maturity_claim_does_not_collect_early_in_the_shared_pool() {
    let (mut w, mut s) = fixture(RepaymentSchedule::AtMaturity, 100, 0, 0, 3);
    w.collection_policy = economics_compute_smoke::finance::CollectionPolicy::Proportional;
    w.agents.push(Agent {
        id: 99,
        name: "later creditor".into(),
    });
    s.balances.insert((99, TOKEN), 100);
    let mut second = w.lending[0].clone();
    second.id = 2;
    second.terms.creditor = 99;
    second.terms.term_months = 5;
    w.lending.push(second);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(4).unwrap();
    assert_eq!(sim.state.credit.loans[&1].status, Status::Repaid);
    assert_eq!(sim.state.credit.loans[&2].principal, 100);
    assert_eq!(sim.state.credit.loans[&2].first_unpaid, None);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 100);
    assert_eq!(sim.state.balance(99, TOKEN), 0);
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.credit.loans[&2].status, Status::Repaid);
    assert_eq!(sim.state.balance(99, TOKEN), 100);
}

#[test]
fn accepted_schedule_is_frozen_and_fractional_interest_carries_until_collection() {
    let (w, s) = fixture(RepaymentSchedule::AtMaturity, 3, 3, 3333, 3);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    let mut changed = sim.clone();
    changed.world.lending[0].terms.repayment = RepaymentSchedule::Installments;
    for (interest, remainder) in [(0, 9999), (1, 9998)] {
        sim.run_months(1).unwrap();
        changed.run_months(1).unwrap();
        assert_eq!(sim.state, changed.state);
        assert_eq!(sim.ledger, changed.ledger);
        let loan = &sim.state.credit.loans[&1];
        assert_eq!(loan.repayment, RepaymentSchedule::AtMaturity);
        assert_eq!(
            (loan.interest, loan.interest_remainder),
            (interest, remainder)
        );
        assert_eq!(sim.state.balance(PERSON, TOKEN), 6);
    }
    sim.run_months(1).unwrap();
    changed.run_months(1).unwrap();
    assert_eq!(sim.state, changed.state);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 1);
    assert_eq!(sim.state.credit.loans[&1].interest_remainder, 0);
}

#[test]
fn maturity_execution_matches_cpu_monthly_batch_and_clone_continuation() {
    let (w, s) = fixture(RepaymentSchedule::AtMaturity, 100, 5, 100, 3);
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut monthly = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut batched = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    reference.run_months(5).unwrap();
    batched.run_months(5).unwrap();
    for _ in 0..2 {
        monthly.run_months(1).unwrap();
    }
    let mut resumed = monthly.clone();
    for _ in 0..3 {
        monthly.run_months(1).unwrap();
    }
    resumed.run_months(3).unwrap();
    for other in [&monthly, &batched, &resumed] {
        assert_eq!(reference.state, other.state);
        assert_eq!(reference.ledger, other.ledger);
    }
}

#[test]
fn collateral_grace_starts_at_maturity_and_explicit_acceleration_is_preserved() {
    let (mut w, s) = credit::scenario("default").unwrap();
    w.credit.as_mut().unwrap().offers[0].loan.repayment = RepaymentSchedule::AtMaturity;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(4).unwrap();
    assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
    assert_eq!(sim.state.credit.loans[&1].first_unpaid, None);
    let mut accelerated = sim.state.credit.loans[&1].clone();
    accelerated.status = Status::Enforced;
    assert_eq!(accelerated.due(4).unwrap(), 8240);
    accelerated.status = Status::Stayed;
    assert_eq!(accelerated.due(4).unwrap(), 8240);
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.credit.loans[&1].first_unpaid, Some(5));
    assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.credit.loans[&1].status, Status::Enforced);
    assert_eq!(sim.state.credit.loans[&1].debt().unwrap(), 2400);
    assert_eq!(
        credit::owner(&sim.world, &sim.state, PLOT),
        Some(STATE_AGENT)
    );
}

#[test]
fn guarantee_waits_for_maturity_and_only_covers_the_unpaid_residual() {
    use economics_compute_smoke::recovery::{
        Guarantee, GuaranteeTender, GuaranteedClaim, RecourseSecurity,
    };
    let (mut w, mut s) = fixture(RepaymentSchedule::AtMaturity, 100, 0, 100, 3);
    w.agents.push(Agent {
        id: 99,
        name: "guarantor".into(),
    });
    s.balances.insert((99, TOKEN), 10);
    w.recovery.guarantees.push(Guarantee {
        id: 1,
        claim: GuaranteedClaim::Loan(1),
        guarantor: 99,
        cap: 10,
        from: 1,
        through: 6,
        delay_months: 0,
        recourse: 101,
        priority: 0,
        follows_assignment: false,
        tender: GuaranteeTender::Native,
        security: RecourseSecurity::Unsecured,
    });
    let (mut sim, mut audit) = audited(w, s, Backend::Reference);
    until(&mut sim, &mut audit, 4);
    assert!(sim.state.credit.recovery.paid_guarantees.is_empty());
    assert!(!sim.state.credit.loans.contains_key(&101));
    assert_eq!(sim.state.balance(99, TOKEN), 10);
    until(&mut sim, &mut audit, 5);
    assert_eq!(sim.state.credit.loans[&1].status, Status::Repaid);
    assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], 3);
    let recourse = &sim.state.credit.loans[&101];
    assert_eq!(recourse.principal, 3);
    assert_eq!(recourse.opened, 4);
    assert_eq!(recourse.repayment, RepaymentSchedule::Installments);
    assert_eq!(recourse.first_unpaid, None);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 103);
    assert_eq!(sim.state.balance(99, TOKEN), 7);
    assert_eq!(sim.state.balances.values().sum::<i32>(), 110);
}

#[test]
fn another_overdue_claim_can_open_an_estate_before_maturity() {
    use economics_compute_smoke::recovery::{ProceedingTerms, Stage};
    let (mut w, mut s) = fixture(RepaymentSchedule::AtMaturity, 100, 0, 100, 5);
    let mut earlier = w.lending[0].clone();
    earlier.id = 2;
    earlier.principal = 10;
    earlier.terms.max_principal = 10;
    earlier.terms.monthly_rate_bps = 0;
    earlier.terms.term_months = 1;
    w.lending.push(earlier);
    s.balances.insert((STATE_AGENT, TOKEN), 110);
    w.agents.push(Agent {
        id: 99,
        name: "estate".into(),
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: 99,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![],
        discharge_deficiency: false,
    });
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    // Controlled external loss before opening the audited distress interval.
    sim.state.balances.insert((PERSON, TOKEN), 0);
    let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
    until(&mut sim, &mut audit, 3);
    assert_eq!(sim.state.credit.loans[&2].first_unpaid, Some(2));
    assert_eq!(sim.state.credit.loans[&1].first_unpaid, None);
    assert_eq!(sim.state.credit.loans[&1].due(3).unwrap(), 0);
    until(&mut sim, &mut audit, 4);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    let loan = &sim.state.credit.loans[&1];
    assert_eq!(loan.status, Status::Stayed);
    assert_eq!(loan.repayment, RepaymentSchedule::AtMaturity);
    assert_eq!(loan.last_accrued, 3);
    assert_eq!(loan.interest, 1); // Estate opening freezes further interest.
    assert_eq!(loan.due(3).unwrap(), 101); // Full estate claim before month 6.
    assert!(
        credit::projection::dues(&sim.world, &sim.state, PERSON, TOKEN, 3)
            .unwrap()
            .is_empty()
    ); // Ordinary collection remains stayed.
}

#[test]
fn invalid_maturity_date_cannot_enter_the_book() {
    let (w, s) = fixture(RepaymentSchedule::AtMaturity, 100, 5, 100, 3);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.state.month = u32::MAX - 1;
    let loan = sim.state.credit.loans.get_mut(&1).unwrap();
    loan.opened = sim.state.month;
    loan.last_accrued = sim.state.month;
    assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
}

#[cfg(feature = "ancient-world-host")]
#[test]
fn fully_funded_maturity_case_matches_main_engine_monthly_collection() {
    use ancient_world::{
        civilization::History,
        credit::{Account as HostAccount, RepaymentSource, SHARED_CURRENCY, Terms},
    };
    let mut h:History=serde_json::from_value(serde_json::json!({
        "version":2,"seed":17,"terrain_resolution":16,"source_epoch":0,"month":1,
        "civilizations":[],"sites":[{"id":0,"civilization":0,"island":0,"cell":0,
          "name":"borrower","founded":0,"abandoned":false,
          "stocks":{"stock":[0.,0.,0.,0.],"habitat":[0.,0.,0.,0.],"ledger":[0.,0.,0.,0.],"people":[0.,0.,0.,0.]}}],
        "people":[],"events":[],"shipments":[],"candidates":[],"initial_food":0.,"initial_population":0.,
        "society":{"version":1,"started":0,"households":[],"councils":[{"civilization":0,
          "treasury":100.,"tax_rate":0.,"relief_paid":0.}],"routes":[],"routed_sites":0,"raids":[],"next_raid":0}
    })).unwrap();
    h.sites[0].economy.finance[0] = 5.;
    h.credit.servicing_policy.available_cash_share = 1.;
    h.commit_credit_loan(
        Terms {
            lender: HostAccount::Council(0),
            borrower: HostAccount::Town(0),
            currency: SHARED_CURRENCY,
            source: RepaymentSource::AnnualTax {
                council: 0,
                collection_month: 4,
            },
            annual_simple_rate: 0.12,
            maturity_month: 4,
            grace_months: 2,
        },
        100.,
    )
    .unwrap()
    .unwrap();
    let (w, s) = fixture(RepaymentSchedule::AtMaturity, 100, 5, 100, 3);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    for month in 2..=4 {
        h.month = month;
        h.service_credit_month().unwrap();
        h.validate_credit().unwrap();
        sim.run_months(1).unwrap();
        let loan = &sim.state.credit.loans[&1];
        assert_eq!(
            f64::from(loan.principal),
            h.credit.loans[0].outstanding_principal
        );
        assert_eq!(f64::from(loan.interest), h.credit.loans[0].interest_due);
        assert_eq!(
            f64::from(sim.state.balance(PERSON, TOKEN)),
            f64::from(h.sites[0].economy.finance[0])
        );
        assert_eq!(
            f64::from(sim.state.balance(STATE_AGENT, TOKEN)),
            h.society.as_ref().unwrap().councils[0].treasury
        );
    }
}
