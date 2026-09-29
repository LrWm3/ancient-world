use economics_compute_smoke::{
    compute::Backend,
    employment::{ArrearsPolicy, Terms},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
    model::*,
    opportunities::{Action, PERSON_TYPE},
    scenario::{LABOR, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const PAY: ResourceId = 100;
const STORED: ResourceId = 101;
fn wages(room: i32) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    for (id, name) in [(PAY, "wage stock"), (STORED, "stored stock")] {
        w.resources.push(Resource {
            id,
            name: name.into(),
            kind: ResourceKind::Stock,
        });
        w.storage.weights.insert(id, 1);
    }
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == PERSON { 5 } else { 0 };
    }
    w.storage.capacities.insert(PERSON, 100);
    w.storage.capacities.insert(91, 2);
    w.storage.capacities.insert(89, 100);
    s.balances.insert((HOME, STORED), 51 - room);
    s.balances.insert((89, PAY), 4);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::CapacityTrade));
    w.employment.push(Terms {
        id: 1,
        employer: 89,
        worker: PERSON,
        from: 1,
        through: 3,
        capacity: Amount::new(LABOR, 4),
        wage_per_unit: Amount::new(PAY, 1),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    (w, s)
}
#[test]
fn physical_wages_fit_both_private_and_collective_storage_and_preserve_arrears() {
    for (room, paid) in [(0, 1), (1, 3), (2, 4)] {
        let (w, s) = wages(room);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            sim.run_months(1).unwrap();
            assert_eq!(sim.state.employment.earned[&(1, 1)].claim.settled, paid);
            assert_eq!(sim.state.balance(HOME, PAY), paid / 2);
            assert_eq!(sim.state.balance(PERSON, PAY), paid - paid / 2);
            assert_eq!(sim.state.balance(89, PAY), 4 - paid);
            let mut replay = s.clone();
            for b in &sim.ledger {
                commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(replay, sim.state);
            if paid < 4 {
                sim.run_months(1).unwrap();
                assert!(!sim.state.employment.earned.contains_key(&(1, 2)));
                assert_eq!(sim.state.employment.earned[&(1, 1)].claim.settled, paid);
            }
            (sim.state, sim.ledger)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

fn wage_audit(
    w: &World,
    s: &State,
    value: Option<i128>,
) -> Result<economics_compute_smoke::financial_reporting::Audit, String> {
    use economics_compute_smoke::financial_reporting::{Audit, Opening};
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            exchange_values: value.map(|v| [(PAY, v)].into()).unwrap_or_default(),
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| {
                    *r != TOKEN
                        && **q > 0
                        && w.resources
                            .iter()
                            .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
                })
                .map(|(key, q)| (*key, i128::from(*q) * if key.1 == PAY { 2 } else { 1 }))
                .collect(),
            processes: Some(Default::default()),
            services: Some(Default::default()),
            ..Default::default()
        },
    )
}
#[test]
fn physical_wages_value_claims_once_pool_costed_inventory_and_clear_after_room_returns() {
    use economics_compute_smoke::accounting::Account as A;
    let (w, s) = wages(1);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = wage_audit(&w, &s, Some(3)).unwrap();
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        let balance = |id, account| {
            a.book()
                .balances()
                .get(&(id, account))
                .copied()
                .unwrap_or(0)
        };
        assert_eq!(balance(PERSON, A::WagesReceivable(1, 1)), 3);
        assert_eq!(balance(89, A::WagesPayable(1, 1)), -3);
        assert_eq!(balance(PERSON, A::ServiceIncome), -12);
        assert_eq!(balance(89, A::ServiceExpense), 12);
        assert_eq!(balance(89, A::Sales), -9);
        assert_eq!(balance(89, A::CostOfSales), 6);
        assert_eq!(balance(HOME, A::Inventory(PAY)), 3);
        assert!(
            a.book()
                .statements(PERSON, 1, 2)
                .unwrap()
                .cash_flows
                .is_empty()
        );
        sim.world.storage.capacities.insert(91, 4);
        let (mut resumed, mut saved) = (sim.clone(), a.clone());
        while sim.state.month == 3 {
            a.step(&mut sim).unwrap();
            saved.step(&mut resumed).unwrap();
        }
        assert_eq!((&sim.state, &a), (&resumed.state, &saved));
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
        assert_eq!(sim.state.balance(HOME, PAY), 2);
        assert_eq!(sim.state.balance(PERSON, PAY), 2);
        for id in [PERSON, HOME, 89] {
            let f = a.book().statements(id, 1, 3).unwrap();
            assert_eq!(f.assets, f.liabilities + f.equity);
        }
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn missing_or_overflowing_wage_values_cannot_publish_financial_or_simulation_state() {
    let (w, s) = wages(2);
    assert!(wage_audit(&w, &s, None).is_err());
    let mut a = wage_audit(&w, &s, Some(i128::MAX)).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Acquire {
        a.step(&mut sim).unwrap();
    }
    let before = (sim.state.clone(), a.clone());
    assert!(a.step(&mut sim).unwrap_err().contains("overflow"));
    assert_eq!(before, (sim.state, a));
}

fn loans(support: bool, collective_due: bool) -> (World, State) {
    use economics_compute_smoke::credit::{Advance, Loan, LoanOffer, Status};
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = 0;
    }
    s.balances.clear();
    s.month = 2;
    s.balances.insert((HOME, TOKEN), 5);
    w.households[0].governance.charter.support_member_loans = support;
    for (id, debtor) in [(1, PERSON), (2, HOME)] {
        if id == 2 && !collective_due {
            continue;
        }
        w.lending.push(Advance {
            id,
            debtor,
            principal: 4,
            month: 1,
            priority: id,
            collateral: None,
            terms: LoanOffer {
                creditor: 89,
                denomination: TOKEN,
                max_principal: 4,
                monthly_rate_bps: 2500,
                term_months: 1,
                grace_months: 10,
            },
        });
        s.credit.loans.insert(
            id,
            Loan {
                id,
                creditor: 89,
                debtor,
                denomination: TOKEN,
                original_principal: 4,
                principal: 4,
                interest: 0,
                interest_remainder: 0,
                monthly_rate_bps: 2500,
                opened: 1,
                last_accrued: 1,
                term_months: 1,
                grace_months: 10,
                first_unpaid: None,
                status: Status::Active,
                collateral: None,
                priority: id,
            },
        );
    }
    (w, s)
}
#[test]
fn opted_household_support_pays_current_interest_without_assuming_member_debt() {
    use economics_compute_smoke::financial_reporting::Audit;
    for support in [false, true] {
        for collective_due in [false, true] {
            let (w, s) = loans(support, collective_due);
            let run = |backend| {
                let mut audit =
                    Audit::with_inventory(&w, &s, TOKEN, Default::default(), Default::default())
                        .unwrap();
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while sim.state.month == 2 {
                    audit.step(&mut sim).unwrap();
                }
                let member_paid = support && !collective_due;
                assert_eq!(
                    sim.state.credit.loans[&1].principal,
                    if member_paid { 0 } else { 4 }
                );
                assert_eq!(
                    sim.state.credit.loans[&1].interest,
                    if member_paid { 0 } else { 1 }
                );
                assert_eq!(
                    sim.state.balance(89, TOKEN),
                    if member_paid || collective_due { 5 } else { 0 }
                );
                if collective_due {
                    assert_eq!(sim.state.credit.loans[&2].principal, 0);
                }
                for id in [HOME, PERSON, 89] {
                    let report = audit.book().statements(id, 2, 2).unwrap();
                    assert_eq!(report.assets, report.liabilities + report.equity);
                }
                let mut replay = s.clone();
                for b in &sim.ledger {
                    commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
                }
                assert_eq!(replay, sim.state);
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}
#[test]
fn exited_member_receives_no_collective_loan_support() {
    let (mut w, s) = loans(true, false);
    households::membership::leave(&mut w, &s, HOME, 91).unwrap();
    // The continuing governor keeps authority, but this debtor has left.
    w.lending[0].debtor = 91;
    let mut s = s;
    s.credit.loans.get_mut(&1).unwrap().debtor = 91;
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.balance(HOME, TOKEN), 5);
    assert_eq!(sim.state.credit.loans[&1].principal, 4);
}

fn financed_market(buy: bool, support: bool) -> (World, State) {
    use economics_compute_smoke::{negotiation::QuotePolicy, scenario::GRAIN};
    let (mut w, mut s) = loans(support, false);
    let (mw, ms) = households::market::scenario().unwrap();
    w.town_market = mw.town_market;
    s.town_market = ms.town_market;
    w.households[0].governance.charter.fund_due_loans = buy;
    w.lending[0].terms.monthly_rate_bps = 0;
    s.credit.loans.get_mut(&1).unwrap().monthly_rate_bps = 0;
    s.balances.clear();
    s.balances.insert((HOME, GRAIN), 2);
    s.balances.insert((92, TOKEN), 4);
    let c = w.town_market.as_mut().unwrap();
    c.traders.retain(|t| [HOME, 92].contains(&t.trader.agent));
    for entry in &mut c.traders {
        entry.trader.limit = 2;
        entry.trader.opening_quote = 2;
        entry.trader.policy = QuotePolicy::Fixed;
    }
    let market = &mut w
        .marketplaces
        .iter_mut()
        .find(|m| m.agent == c.venue)
        .unwrap()
        .markets[0];
    market.goods = Amount::new(TOKEN, 4);
    market.payment = GRAIN;
    market.price_tick = 1;
    (w, s)
}
#[test]
fn collective_market_acquires_payment_stock_for_next_due_without_backdating_settlement() {
    use economics_compute_smoke::{
        financial_reporting::{Audit, Opening},
        scenario::GRAIN,
    };
    for buy in [false, true] {
        for support in [false, true] {
            let (w, s) = financed_market(buy, support);
            let run = |backend| {
                let mut audit = Audit::with_opening(
                    &w,
                    &s,
                    TOKEN,
                    Opening {
                        inventory: [((HOME, GRAIN), 2)].into(),
                        exchange_values: [(GRAIN, 2)].into(),
                        ..Default::default()
                    },
                )
                .unwrap();
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while sim.state.month == 2 {
                    audit.step(&mut sim).unwrap();
                }
                let traded = buy && support;
                assert_eq!(sim.state.balance(HOME, TOKEN), if traded { 4 } else { 0 });
                assert_eq!(sim.state.balance(92, TOKEN), if traded { 0 } else { 4 });
                assert_eq!(sim.state.balance(92, GRAIN), if traded { 2 } else { 0 });
                assert_eq!(sim.state.credit.loans[&1].principal, 4);
                let checkpoint = (sim.clone(), audit.clone());
                while sim.state.month == 3 {
                    audit.step(&mut sim).unwrap();
                }
                assert_eq!(
                    sim.state.credit.loans[&1].principal,
                    if traded { 0 } else { 4 }
                );
                assert_eq!(sim.state.balance(89, TOKEN), if traded { 4 } else { 0 });
                let (mut resumed, mut saved) = checkpoint;
                while resumed.state.month == 3 {
                    saved.step(&mut resumed).unwrap();
                }
                assert_eq!(sim.state, resumed.state);
                assert_eq!(audit, saved);
                let mut replay = s.clone();
                for b in &sim.ledger {
                    commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
                }
                assert_eq!(replay, sim.state);
                for id in [HOME, PERSON, 89, 92] {
                    let report = audit.book().statements(id, 2, 3).unwrap();
                    assert_eq!(report.assets, report.liabilities + report.equity);
                }
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

#[test]
fn collective_own_dues_generate_orders_but_unfunded_bids_do_not_create_payment_stock() {
    use economics_compute_smoke::scenario::GRAIN;
    for funded in [false, true] {
        let (mut w, mut s) = financed_market(true, false);
        w.lending[0].debtor = HOME;
        s.credit.loans.get_mut(&1).unwrap().debtor = HOME;
        if !funded {
            s.balances.insert((HOME, GRAIN), 0);
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        sim.run_months(1).unwrap();
        assert_eq!(sim.state.balance(HOME, TOKEN), if funded { 4 } else { 0 });
        assert_eq!(sim.state.credit.loans[&1].principal, 4);
        sim.run_months(1).unwrap();
        assert_eq!(
            sim.state.credit.loans[&1].principal,
            if funded { 0 } else { 4 }
        );
    }
}

#[test]
fn scarce_member_debt_support_has_an_explicit_policy_and_stable_claim_priority() {
    use economics_compute_smoke::household_governance::DebtSupportPolicy;
    for policy in [
        DebtSupportPolicy::ReservationOrder,
        DebtSupportPolicy::ClaimPriority,
    ] {
        let (mut w, mut s) = loans(true, false);
        w.households[0].governance.charter.debt_support = policy;
        let mut advance = w.lending[0].clone();
        advance.id = 2;
        advance.debtor = 91;
        advance.priority = 0;
        w.lending.push(advance);
        let mut loan = s.credit.loans[&1].clone();
        loan.id = 2;
        loan.debtor = 91;
        loan.priority = 0;
        s.credit.loans.insert(2, loan);
        let run = |w: World, backend| {
            let mut sim = Simulation::new(w, s.clone(), backend).unwrap();
            sim.run_months(1).unwrap();
            assert_eq!(sim.state.balance(89, TOKEN), 5);
            let paid = if policy == DebtSupportPolicy::ReservationOrder {
                1
            } else {
                2
            };
            assert_eq!(sim.state.credit.loans[&paid].principal, 0);
            assert_eq!(sim.state.credit.loans[&(3 - paid)].principal, 4);
            let reservations: Vec<_> = sim
                .ledger
                .iter()
                .filter_map(|b| b.household.as_ref())
                .flat_map(|b| &b.reservations)
                .filter(|r| matches!(r.request.purpose, households::Purpose::LoanSupport { .. }))
                .collect();
            assert_eq!(reservations.iter().map(|r| r.allocated).sum::<i32>(), 5);
            assert!(reservations.iter().all(|r| matches!(r.request.purpose, households::Purpose::LoanSupport { policy: p, .. } if p == policy)));
            (sim.state, sim.ledger)
        };
        let reference = run(w.clone(), Backend::Reference);
        assert_eq!(reference, run(w.clone(), Backend::CubeCpu));
        if policy == DebtSupportPolicy::ClaimPriority {
            w.households[0].adults.reverse();
            w.participants.reverse();
            w.lending.reverse();
            assert_eq!(reference.0, run(w, Backend::CubeCpu).0);
        }
    }
}

#[test]
fn earned_wages_pool_then_pay_another_members_loan_on_the_next_boundary() {
    use economics_compute_smoke::financial_reporting::{Audit, Opening};
    let (mut w, mut s) = loans(true, false);
    s.balances.clear();
    s.balances.insert((89, TOKEN), 4);
    w.participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .capacity
        .quantity = 5;
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::CapacityTrade));
    let (payroll, _) = wages(2);
    let mut terms = payroll.employment[0].clone();
    terms.wage_per_unit.resource = TOKEN;
    terms.from = 2;
    terms.through = 2;
    w.employment.push(terms);
    w.lending[0].debtor = 91;
    w.lending[0].terms.monthly_rate_bps = 0;
    let loan = s.credit.loans.get_mut(&1).unwrap();
    loan.debtor = 91;
    loan.monthly_rate_bps = 0;
    let run = |backend| {
        let mut audit = Audit::with_opening(
            &w,
            &s,
            TOKEN,
            Opening {
                services: Some(Default::default()),
                ..Default::default()
            },
        )
        .unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while sim.state.month == 2 {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.balance(HOME, TOKEN), 2);
        assert_eq!(sim.state.credit.loans[&1].principal, 4);
        let checkpoint = (sim.clone(), audit.clone());
        while sim.state.month == 3 {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.credit.loans[&1].principal, 2);
        assert_eq!(sim.state.balance(HOME, TOKEN), 0);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 2);
        assert_eq!(sim.state.balance(89, TOKEN), 2);
        let (mut resumed, mut saved) = checkpoint;
        while resumed.state.month == 3 {
            saved.step(&mut resumed).unwrap();
        }
        assert_eq!(sim.state, resumed.state);
        assert_eq!(audit, saved);
        let mut replay = s.clone();
        for b in &sim.ledger {
            commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        }
        assert_eq!(replay, sim.state);
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn debt_support_receipts_cannot_be_relabelled_after_reservation() {
    let (w, s) = loans(true, false);
    let mut sim = Simulation::new(w.clone(), s, Backend::Reference).unwrap();
    sim.step().unwrap();
    let before = sim.state.clone();
    assert_eq!(before.phase, Phase::Due);
    sim.step().unwrap();
    let mut forged = sim.ledger.last().unwrap().clone();
    let request = &mut forged
        .household
        .as_mut()
        .unwrap()
        .reservations
        .iter_mut()
        .find(|r| matches!(r.request.purpose, households::Purpose::LoanSupport { .. }))
        .unwrap()
        .request;
    request.purpose = households::Purpose::Obligation;
    let mut unchanged = before.clone();
    assert!(
        commit(
            &w,
            &mut unchanged,
            &forged,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(unchanged, before);
}
