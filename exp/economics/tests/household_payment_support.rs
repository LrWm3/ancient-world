use economics_compute_smoke::{
    accounting::Account as A,
    compute::Backend,
    employment::{ArrearsPolicy, Earned, Terms},
    finance::{Condition, FailureRule, Obligation, Transfer},
    financial_reporting::{Audit, Opening},
    household_governance::TieBreak,
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME, support},
    model::*,
    scenario::{GRAIN, LABOR, NUTRITION, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

const WORKER: AgentId = 89;
const LENDER: AgentId = 92;

fn fixture(enabled: bool, resource: ResourceId, private: i32) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    s.month = 2;
    s.balances.clear();
    s.balances.insert((PERSON, resource), private);
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = 0;
        w.storage.capacities.insert(p.agent, 100);
    }
    let c = &mut w.households[0].governance.charter;
    c.accept_payment_support = enabled;
    c.hiring_budget = Some(Amount::new(resource, 6));
    c.tie_break = TieBreak::MemberId;
    w.employment.push(Terms {
        id: 1,
        employer: HOME,
        worker: WORKER,
        from: 1,
        through: 1,
        capacity: Amount::new(LABOR, 3),
        wage_per_unit: Amount::new(resource, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    s.employment.earned.insert(
        (1, 1),
        Earned {
            delivered: 3,
            claim: Obligation {
                transfer: Transfer {
                    from: HOME,
                    to: WORKER,
                    amount: Amount::new(resource, 6),
                },
                settled: 0,
                condition: Condition::OnOrAfterMonth(2),
                failure: FailureRule::CarryArrears,
            },
        },
    );
    support::authorize(
        &mut w,
        &s,
        HOME,
        PERSON,
        support::Mandate {
            member: PERSON,
            resource,
            from: 2,
            through: 6,
            revoked_from: None,
            reserve_months: 1,
            private_reserve: 2,
            household_target: 20,
            monthly_limit: 20,
        },
    )
    .unwrap();
    (w, s)
}

fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| *r == GRAIN && **q > 0)
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            exchange_values: [(GRAIN, 1)].into(),
            services: Some(Default::default()),
            processes: Some(Default::default()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
fn value(a: &Audit, who: AgentId, account: A) -> i128 {
    a.book()
        .balances()
        .get(&(who, account))
        .copied()
        .unwrap_or(0)
}
fn replay(w: &World, s: &State, sim: &Simulation) {
    let mut state = s.clone();
    for b in &sim.ledger {
        commit(w, &mut state, b, sim.backend, DEFAULT_EFFECT_LIMIT).unwrap();
    }
    assert_eq!(state, sim.state);
}

#[test]
fn signed_surplus_funds_only_the_real_wage_gap_and_preserves_separate_books() {
    for (enabled, private, expected) in [(false, 10, 0), (true, 10, 6), (true, 5, 3)] {
        let (w, s) = fixture(enabled, TOKEN, private);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            while sim.state.phase != Phase::Productive {
                a.step(&mut sim).unwrap();
            }
            let before = sim.state.clone();
            a.step(&mut sim).unwrap();
            let b = sim.ledger.last().unwrap();
            let r = &b.household.as_ref().unwrap().support[0];
            assert_eq!(r.accepted, expected);
            assert_eq!(sim.state.balance(HOME, TOKEN), expected);
            // Funding is not debt settlement, and does not give the donor a claim.
            assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 6);
            assert_eq!(sim.state.balance(PERSON, TOKEN), private - expected);
            if enabled {
                assert_eq!(r.payment_funding.as_ref().unwrap().shortfall, 6);
                assert_eq!(
                    r.payment_funding.as_ref().unwrap().projected_shortfall,
                    i128::from(6 - expected)
                );
                let mut forged = b.clone();
                forged.household.as_mut().unwrap().support[0]
                    .payment_funding
                    .as_mut()
                    .unwrap()
                    .due += 1;
                let mut unchanged = before.clone();
                assert!(
                    commit(&w, &mut unchanged, &forged, backend, DEFAULT_EFFECT_LIMIT).is_err()
                );
                assert_eq!(unchanged, before);
            }
            let (mut resumed, mut ra) = (sim.clone(), a.clone());
            through(&mut a, &mut sim, 3);
            through(&mut ra, &mut resumed, 3);
            assert_eq!(sim.state.balance(WORKER, TOKEN), expected);
            assert_eq!(
                sim.state.employment.earned[&(1, 1)].claim.outstanding(),
                6 - expected
            );
            assert_eq!(value(&a, PERSON, A::TransferExpense), i128::from(expected));
            assert_eq!(value(&a, PERSON, A::WagesPayable(1, 1)), 0);
            assert_eq!(value(&a, HOME, A::TransferIncome), -i128::from(expected));
            assert_eq!(
                (sim.state.clone(), sim.ledger.clone(), a.clone()),
                (resumed.state, resumed.ledger, ra)
            );
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn physical_payment_support_protects_personal_food_and_retains_unstorable_payroll() {
    let (mut w, s) = fixture(true, GRAIN, 10);
    w.participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .needs
        .push(Requirement {
            resource: NUTRITION,
            quantity: 6,
            priority: 0,
        });
    w.storage.capacities.insert(WORKER, 2);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut a, &mut sim, 2);
        let r = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.support)
            .next()
            .unwrap();
        assert_eq!((r.protected, r.accepted), (6, 4));
        assert_eq!(
            sim.reports
                .iter()
                .find(|r| r.agent == PERSON)
                .unwrap()
                .deficit(NUTRITION),
            0
        );
        assert_eq!(sim.state.balance(WORKER, GRAIN), 2);
        assert_eq!(sim.state.balance(HOME, GRAIN), 2);
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 4);
        assert_eq!(value(&a, HOME, A::Inventory(GRAIN)), 2);
        assert_eq!(value(&a, PERSON, A::TransferExpense), 4);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn payment_support_requires_live_consent_membership_and_earned_claims() {
    for case in 0..4 {
        let (mut w, mut s) = fixture(true, TOKEN, 10);
        match case {
            0 => w.households[0].support.clear(),
            1 => {
                support::revoke(&mut w, &s, HOME, PERSON, TOKEN, 2).unwrap();
                s.month = 3;
            }
            2 => households::membership::leave(&mut w, &s, HOME, PERSON).unwrap(),
            _ => {
                s.employment.earned.clear();
                w.employment[0].from = 2;
                w.employment[0].through = 4;
                w.participants
                    .iter_mut()
                    .find(|p| p.agent == WORKER)
                    .unwrap()
                    .capacity
                    .quantity = 3;
            }
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut a, &mut sim, s.month);
            assert_eq!(sim.state.balance(PERSON, TOKEN), 10);
            assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            assert_eq!(sim.state.balance(WORKER, TOKEN), 0);
            assert!(
                sim.ledger
                    .iter()
                    .filter_map(|b| b.household.as_ref())
                    .flat_map(|h| &h.support)
                    .all(|r| r.accepted == 0)
            );
            if case == 3 {
                assert!(sim.state.employment.earned.is_empty());
            }
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn donors_own_wages_are_protected_before_supporting_collective_wages() {
    let (mut w, mut s) = fixture(true, TOKEN, 10);
    let mut terms = w.employment[0].clone();
    terms.id = 2;
    terms.employer = PERSON;
    terms.worker = LENDER;
    w.employment.push(terms);
    let mut wage = s.employment.earned[&(1, 1)].clone();
    wage.claim.transfer.from = PERSON;
    wage.claim.transfer.to = LENDER;
    s.employment.earned.insert((2, 1), wage);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut a, &mut sim, 2);
        let r = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.support)
            .next()
            .unwrap();
        assert_eq!((r.protected, r.accepted), (6, 4));
        assert_eq!(sim.state.balance(WORKER, TOKEN), 4);
        assert_eq!(sim.state.balance(LENDER, TOKEN), 6);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 2);
        assert_eq!(sim.state.employment.earned[&(2, 1)].claim.outstanding(), 0);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn successive_member_offers_share_one_payment_gap_in_stable_policy_order() {
    let (mut w, mut s) = fixture(true, TOKEN, 5);
    s.balances.insert((91, TOKEN), 10);
    let mut mandate = w.households[0].support[0].clone();
    mandate.member = 91;
    support::authorize(&mut w, &s, HOME, 91, mandate).unwrap();
    let run = |mut w: World, backend| {
        if matches!(backend, Backend::CubeCpu) {
            w.households[0].adults.reverse();
            w.households[0].support.reverse();
        }
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut a, &mut sim, 2);
        let receipts: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.support)
            .collect();
        assert_eq!(
            receipts.iter().map(|r| r.accepted).collect::<Vec<_>>(),
            vec![3, 3]
        );
        assert_eq!(receipts[1].payment_funding.as_ref().unwrap().shortfall, 3);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 2);
        assert_eq!(sim.state.balance(91, TOKEN), 7);
        assert_eq!(sim.state.balance(WORKER, TOKEN), 6);
        assert_eq!(sim.state.balance(HOME, TOKEN), 0);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(w.clone(), Backend::Reference), run(w, Backend::CubeCpu));
}

#[test]
fn income_policy_and_observer_describe_the_capped_payment_offer() {
    use economics_compute_smoke::{
        household_governance::Policy,
        telemetry::{Config, Observer},
    };
    let (mut w, mut s) = fixture(true, TOKEN, 10);
    let (market, opening) = households::market::scenario().unwrap();
    w.town_market = market.town_market;
    s.town_market = opening.town_market;
    let g = &mut w.households[0].governance;
    g.constitution
        .permitted_policies
        .insert(Policy::NeedsThenIncome);
    g.charter.initial_policy = Policy::NeedsThenIncome;
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        let mut observer = Observer::new(
            Vec::new(),
            "payment-support",
            Config {
                settlement: true,
                ..Default::default()
            },
        )
        .unwrap();
        while sim.state.month <= 2 {
            observer.step_audited(&mut sim, &mut a).unwrap();
        }
        let r = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.support)
            .next()
            .unwrap();
        assert_eq!((r.offered, r.accepted), (8, 6));
        assert_eq!(r.projected_income.as_ref().unwrap().opening_coins, 6);
        assert_eq!(r.projected_income.as_ref().unwrap().closing_coins, 0);
        assert_eq!(sim.state.balance(WORKER, TOKEN), 6);
        let logs = String::from_utf8(observer.finish().unwrap()).unwrap();
        assert!(
            logs.lines()
                .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
                .any(|r| r["kind"] == "household_support"
                    && r["receipt"]["payment_funding"]["shortfall"] == 6
                    && r["receipt"]["accepted"] == 6)
        );
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn own_loan_and_wage_funding_share_units_but_keep_their_collection_boundaries() {
    use economics_compute_smoke::credit::{Advance, LoanOffer};
    let (mut w, mut s) = fixture(true, TOKEN, 20);
    w.transaction_policy.as_mut().unwrap().permissions.extend([
        (
            economics_compute_smoke::opportunities::HOUSEHOLD_TYPE,
            economics_compute_smoke::opportunities::Action::Borrow,
        ),
        (
            economics_compute_smoke::opportunities::PERSON_TYPE,
            economics_compute_smoke::opportunities::Action::Lend,
        ),
    ]);
    let terms = std::mem::take(&mut w.employment);
    let wages = std::mem::take(&mut s.employment);
    s.month = 1;
    s.balances.insert((LENDER, TOKEN), 6);
    w.lending.push(Advance {
        id: 1,
        debtor: HOME,
        principal: 6,
        month: 1,
        priority: 0,
        collateral: None,
        terms: LoanOffer {
            creditor: LENDER,
            denomination: TOKEN,
            max_principal: 6,
            monthly_rate_bps: 0,
            term_months: 1,
            grace_months: 12,
        },
    });
    let mut origin = Simulation::new(w, s, Backend::Reference).unwrap();
    origin.run_months(1).unwrap();
    let (mut w, mut s) = (origin.world, origin.state);
    assert_eq!(s.credit.loans[&1].principal, 6);
    // Explicit pre-audit loss supplies an already distressed opening balance sheet.
    s.balances.insert((HOME, TOKEN), 0);
    w.employment = terms;
    s.employment = wages;
    let run = |mut w: World, backend| {
        if matches!(backend, Backend::CubeCpu) {
            w.households[0].adults.reverse();
        }
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut a, &mut sim, 2);
        let r = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.support)
            .next()
            .unwrap();
        assert_eq!(r.accepted, 12);
        assert_eq!(r.payment_funding.as_ref().unwrap().due, 12);
        assert_eq!(sim.state.balance(WORKER, TOKEN), 6);
        assert_eq!(sim.state.balance(HOME, TOKEN), 6);
        assert_eq!(sim.state.balance(LENDER, TOKEN), 0);
        assert_eq!(sim.state.credit.loans[&1].principal, 6);
        let (mut resumed, mut ra) = (sim.clone(), a.clone());
        through(&mut a, &mut sim, 3);
        through(&mut ra, &mut resumed, 3);
        assert_eq!(sim.state.balance(LENDER, TOKEN), 6);
        assert_eq!(sim.state.balance(HOME, TOKEN), 0);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 8);
        assert_eq!(value(&a, PERSON, A::TransferExpense), 12);
        assert_eq!(a.book().statements(HOME, 2, 3).unwrap().liabilities, 0);
        assert_eq!(
            (sim.state.clone(), sim.ledger.clone(), a.clone()),
            (resumed.state, resumed.ledger, ra)
        );
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(w.clone(), Backend::Reference), run(w, Backend::CubeCpu));
}
