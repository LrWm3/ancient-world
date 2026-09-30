use economics_compute_smoke::{
    accounting::Account,
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::{Audit, Opening},
    model::*,
    scenario::{self, GRAIN, PERSON, SEED, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
fn fixture(extra: i32) -> (World, State, Opening) {
    let (mut w, mut s) = scenario::baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights.clear();
    w.agreements.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "reporting coin".into(),
        kind: ResourceKind::Stock,
    });
    w.storage.weights.insert(GRAIN, 1);
    w.lending.push(Advance {
        id: 1,
        debtor: PERSON,
        month: 1,
        principal: 4,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: GRAIN,
            max_principal: 4,
            monthly_rate_bps: 2500,
            term_months: 1,
            grace_months: 12,
        },
    });
    s.balances.clear();
    s.balances.insert((STATE_AGENT, GRAIN), 6);
    s.balances.insert((PERSON, GRAIN), extra);
    let mut opening = Opening {
        inventory: BTreeMap::from([((STATE_AGENT, GRAIN), 6)]),
        exchange_values: BTreeMap::from([(GRAIN, 3)]),
        assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
        ..Default::default()
    };
    if extra > 0 {
        opening
            .inventory
            .insert((PERSON, GRAIN), i128::from(extra * 2));
    }
    (w, s, opening)
}
#[test]
fn physical_advances_interest_and_repayments_reconcile_without_sales_or_cash() {
    for extra in [0, 1] {
        let (w, s, opening) = fixture(extra);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = Audit::with_opening(&w, &s, TOKEN, opening.clone()).unwrap();
            while sim.state.month <= 1 {
                a.step(&mut sim).unwrap();
            }
            let b = a.book().balances();
            assert_eq!(b[&(STATE_AGENT, Account::LoanReceivable(1))], 12);
            assert_eq!(b[&(PERSON, Account::LoanPayable(1))], -12);
            assert_eq!(b[&(STATE_AGENT, Account::SettlementGain)], -8);
            let mut resumed = Simulation::new(w.clone(), sim.state.clone(), backend).unwrap();
            let mut resumed_a = a.clone();
            while sim.state.month <= 2 {
                a.step(&mut sim).unwrap();
            }
            while resumed.state.month <= 2 {
                resumed_a.step(&mut resumed).unwrap();
            }
            assert_eq!(sim.state, resumed.state);
            assert_eq!(a, resumed_a);
            assert_eq!(sim.state.credit.loans[&1].principal, 1 - extra);
            let b = a.book().balances();
            assert_eq!(b[&(STATE_AGENT, Account::InterestIncome)], -3);
            assert_eq!(b[&(PERSON, Account::InterestExpense)], 3);
            for ((_, account), amount) in b {
                if matches!(
                    account,
                    Account::Sales | Account::CostOfSales | Account::Cash
                ) {
                    assert_eq!(*amount, 0);
                }
            }
            for agent in [STATE_AGENT, PERSON] {
                let statement = a.book().statements(agent, 1, 2).unwrap();
                assert_eq!(statement.closing_cash, 0);
            }
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
    }
}
#[test]
fn missing_commodity_value_rejects_reporting_atomically_before_loan_publication() {
    let (w, s, mut opening) = fixture(0);
    opening.exchange_values.clear();
    let mut a = Audit::with_opening(&w, &s, TOKEN, opening).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Acquire {
        a.step(&mut sim).unwrap();
    }
    let before = sim.state.clone();
    let before_a = a.clone();
    assert!(a.step(&mut sim).is_err());
    assert_eq!(before, sim.state);
    assert_eq!(before_a, a);
}

#[test]
fn physical_loan_guarantee_delivers_stock_and_creates_same_unit_recourse() {
    use economics_compute_smoke::recovery::{Guarantee, GuaranteedClaim};
    let (mut w, mut s, mut opening) = fixture(0);
    w.agents.push(Agent {
        id: 99,
        name: "grain guarantor".into(),
    });
    s.balances.insert((99, GRAIN), 2);
    opening.inventory.insert((99, GRAIN), 2);
    w.recovery.guarantees.push(Guarantee {
        follows_assignment: false,
        tender: economics_compute_smoke::recovery::GuaranteeTender::Native,
        security: economics_compute_smoke::recovery::RecourseSecurity::Unsecured,
        id: 1,
        claim: GuaranteedClaim::Loan(1),
        guarantor: 99,
        cap: 4,
        from: 1,
        through: 12,
        delay_months: 0,
        recourse: 101,
        priority: 0,
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = Audit::with_opening(&w, &s, TOKEN, opening.clone()).unwrap();
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.credit.loans[&1].principal, 0);
        let recourse = &sim.state.credit.loans[&101];
        assert_eq!(
            (recourse.principal, recourse.denomination, recourse.opened),
            (1, GRAIN, 2)
        );
        assert_eq!(sim.state.balance(99, GRAIN), 1);
        assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 7);
        let b = a.book().balances();
        assert_eq!(b[&(99, Account::LoanReceivable(101))], 3);
        assert_eq!(b[&(PERSON, Account::LoanPayable(101))], -3);
        assert_eq!(b[&(99, Account::SettlementGain)], -2);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
}

#[test]
fn coin_proceeding_admits_unsecured_native_arrears_without_converting_or_accruing_them() {
    use economics_compute_smoke::{
        credit::Status,
        recovery::{ProceedingTerms, Stage},
    };
    let (mut w, mut s, opening) = fixture(0);
    w.agents.push(Agent {
        id: 999,
        name: "coin custodian".into(),
    });
    s.balances.insert((PERSON, TOKEN), 10);
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: 999,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 3,
        assets: vec![],
        discharge_deficiency: true,
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = Audit::with_opening(&w, &s, TOKEN, opening.clone()).unwrap();
        while sim.state.month <= 5 {
            a.step(&mut sim).unwrap();
        }
        let loan = &sim.state.credit.loans[&1];
        assert_eq!(
            (loan.principal, loan.interest, loan.last_accrued),
            (1, 0, 5)
        );
        assert_eq!(loan.status, Status::Stayed);
        let case = &sim.state.credit.recovery.proceedings[&1];
        assert_eq!((case.stage, case.cash), (Stage::Active, 0));
        assert_eq!(sim.state.balance(PERSON, TOKEN), 10);
        assert_eq!(a.book().balances()[&(PERSON, Account::LoanPayable(1))], -3);
        assert_eq!(
            a.book().balances()[&(STATE_AGENT, Account::LoanReceivable(1))],
            3
        );
        assert_eq!(a.book().balances()[&(PERSON, Account::InterestExpense)], 3);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::CubeCpu), run(Backend::Reference));
}

#[test]
fn agreed_coin_guarantee_settles_native_loan_units_without_delivering_phantom_grain() {
    use economics_compute_smoke::{
        credit,
        recovery::{Guarantee, GuaranteeTender, GuaranteedClaim, RecourseSecurity},
        settlement,
    };
    for rate in [2, 3, 4] {
        for cash in [rate - 1, rate] {
            let (mut w, mut s, opening) = fixture(0);
            w.agents.push(Agent {
                id: 99,
                name: "coin guarantor".into(),
            });
            s.balances.insert((99, TOKEN), cash);
            w.recovery.guarantees.push(Guarantee {
                id: 1,
                claim: GuaranteedClaim::Loan(1),
                guarantor: 99,
                cap: 2,
                from: 1,
                through: 6,
                delay_months: 0,
                recourse: 101,
                priority: 0,
                follows_assignment: false,
                security: RecourseSecurity::Unsecured,
                tender: GuaranteeTender::AgreedCoins {
                    resource: TOKEN,
                    coins_per_unit: rate,
                },
            });
            let run = |backend| {
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                let mut audit = Audit::with_opening(&w, &s, TOKEN, opening.clone()).unwrap();
                while (sim.state.month, sim.state.phase) != (2, Phase::Due) {
                    audit.step(&mut sim).unwrap();
                }
                let before = sim.state.clone();
                let mut resumed = Simulation::new(w.clone(), before.clone(), backend).unwrap();
                let mut ra = audit.clone();
                let prefix = sim.ledger.len();
                audit.step(&mut sim).unwrap();
                let paid = i32::from(cash >= rate);
                assert_eq!(sim.state.credit.loans[&1].principal, 1 - paid);
                assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 6);
                assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
                assert_eq!(sim.state.balance(99, TOKEN), cash - paid * rate);
                assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), paid * rate);
                assert_eq!(sim.state.credit.loans.contains_key(&101), paid > 0);
                if paid > 0 {
                    let recourse = &sim.state.credit.loans[&101];
                    assert_eq!(
                        (recourse.principal, recourse.denomination, recourse.opened),
                        (1, GRAIN, 2)
                    );
                    let book = audit.book().balances();
                    assert_eq!(book[&(99, Account::LoanReceivable(101))], 3);
                    let difference = i128::from(rate - 3);
                    let gain_or_loss = if difference > 0 {
                        Account::SettlementLoss
                    } else {
                        Account::SettlementGain
                    };
                    assert_eq!(
                        book.get(&(99, gain_or_loss)).copied().unwrap_or(0),
                        difference
                    );
                    assert_eq!(
                        audit.book().statements(99, 1, 2).unwrap().closing_cash,
                        i128::from(cash - rate)
                    );
                }
                let accepted = sim.ledger.last().unwrap();
                let mut forged = accepted.clone();
                for r in &mut forged.credit.as_mut().unwrap().recovery {
                    if let economics_compute_smoke::recovery::Receipt::Guaranteed {
                        tender, ..
                    } = r
                    {
                        tender.quantity += 1;
                    }
                }
                let mut rejected = before.clone();
                assert!(
                    settlement::commit(&w, &mut rejected, &forged, backend, sim.effect_limit)
                        .is_err()
                );
                assert_eq!(rejected, before);
                while sim.state.month < 3 {
                    audit.step(&mut sim).unwrap();
                }
                while resumed.state.month < 3 {
                    ra.step(&mut resumed).unwrap();
                }
                assert_eq!(
                    (&sim.state, &sim.ledger[prefix..], &audit),
                    (&resumed.state, &resumed.ledger[..], &ra)
                );
                assert_eq!(
                    sim.state.credit.loans[&1].status == credit::Status::Repaid,
                    paid > 0
                );
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

#[test]
fn competing_alternative_guarantees_share_whole_payment_lots_and_cannot_pay_a_claim_twice() {
    use economics_compute_smoke::{
        finance::CollectionPolicy,
        recovery::{Guarantee, GuaranteeTender, GuaranteedClaim, RecourseSecurity},
    };
    for policy in [CollectionPolicy::Stable, CollectionPolicy::Proportional] {
        let (mut w, mut s, opening) = fixture(0);
        w.agents.push(Agent {
            id: 99,
            name: "shared guarantor".into(),
        });
        s.balances.insert((99, TOKEN), 3);
        w.recovery.guarantee_policy = policy;
        w.storage.weights.insert(SEED, 1);
        for id in [1, 2] {
            w.recovery.guarantees.push(Guarantee {
                id,
                claim: GuaranteedClaim::Loan(1),
                guarantor: 99,
                cap: 2,
                from: 1,
                through: 6,
                delay_months: 0,
                recourse: 100 + id,
                priority: 0,
                follows_assignment: false,
                security: RecourseSecurity::Unsecured,
                tender: GuaranteeTender::AgreedCoins {
                    resource: TOKEN,
                    coins_per_unit: 2,
                },
            });
        }
        let run = |backend, reverse| {
            let mut world = w.clone();
            if reverse {
                world.recovery.guarantees.reverse();
            }
            let mut sim = Simulation::new(world.clone(), s.clone(), backend).unwrap();
            let mut audit = Audit::with_opening(&world, &s, TOKEN, opening.clone()).unwrap();
            while sim.state.month < 3 {
                audit.step(&mut sim).unwrap();
            }
            assert_eq!(sim.state.credit.loans[&1].debt().unwrap(), 0);
            assert_eq!(
                sim.state
                    .credit
                    .recovery
                    .paid_guarantees
                    .values()
                    .sum::<i32>(),
                1
            );
            assert_eq!(
                sim.state
                    .credit
                    .loans
                    .values()
                    .filter(|l| l.id >= 100)
                    .map(|l| l.principal)
                    .sum::<i32>(),
                1
            );
            assert_eq!(sim.state.balance(99, TOKEN), 1);
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 2);
            assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 6);
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference, true), run(Backend::CubeCpu, false));
        for (resource, coins_per_unit) in [(TOKEN, 0), (GRAIN, 2), (SEED, 2)] {
            let mut invalid = w.clone();
            invalid.recovery.guarantees[0].tender = GuaranteeTender::AgreedCoins {
                resource,
                coins_per_unit,
            };
            assert!(Simulation::new(invalid, s.clone(), Backend::Reference).is_err());
        }
    }
}

#[test]
fn alternative_guarantee_currency_keeps_its_cost_basis_when_it_is_not_reporting_cash() {
    use economics_compute_smoke::recovery::{
        Guarantee, GuaranteeTender, GuaranteedClaim, RecourseSecurity,
    };
    const OTHER_COIN: ResourceId = 100;
    let (mut w, mut s, mut opening) = fixture(0);
    w.agents.push(Agent {
        id: 99,
        name: "other currency guarantor".into(),
    });
    w.resources.push(Resource {
        id: OTHER_COIN,
        name: "other coin".into(),
        kind: ResourceKind::Stock,
    });
    s.balances.insert((99, OTHER_COIN), 3);
    opening.inventory.insert((99, OTHER_COIN), 6);
    opening.exchange_values.insert(OTHER_COIN, 5);
    w.recovery.guarantees.push(Guarantee {
        id: 1,
        claim: GuaranteedClaim::Loan(1),
        guarantor: 99,
        cap: 1,
        from: 1,
        through: 6,
        delay_months: 0,
        recourse: 101,
        priority: 0,
        follows_assignment: false,
        security: RecourseSecurity::Unsecured,
        tender: GuaranteeTender::AgreedCoins {
            resource: OTHER_COIN,
            coins_per_unit: 1,
        },
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut audit = Audit::with_opening(&w, &s, TOKEN, opening.clone()).unwrap();
        while sim.state.month < 3 {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.credit.loans[&1].debt().unwrap(), 0);
        assert_eq!(sim.state.credit.loans[&101].denomination, GRAIN);
        assert_eq!(sim.state.balance(99, OTHER_COIN), 2);
        assert_eq!(sim.state.balance(STATE_AGENT, OTHER_COIN), 1);
        let b = audit.book().balances();
        assert_eq!(b[&(99, Account::LoanReceivable(101))], 3);
        assert_eq!(b[&(99, Account::Inventory(OTHER_COIN))], 4);
        assert_eq!(b[&(99, Account::SettlementGain)], -3);
        assert_eq!(b[&(99, Account::SettlementLoss)], 2);
        assert_eq!(b[&(STATE_AGENT, Account::Inventory(OTHER_COIN))], 5);
        assert_eq!(audit.book().statements(99, 1, 2).unwrap().closing_cash, 0);
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn household_alternative_guarantee_preserves_private_debt_and_blocks_exit_on_native_recourse() {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, dissolution as d},
        recovery::{Guarantee, GuaranteeTender, GuaranteedClaim, RecourseSecurity},
        scenario::LABOR,
    };
    const HOME: AgentId = 800;
    for cash in [1, 2] {
        let (mut w, mut s, opening) = fixture(0);
        w.participants.push(Participant {
            agent: PERSON,
            capacity: Amount::new(LABOR, 0),
            needs: vec![],
        });
        let mut governance = Governance::contributed(PERSON);
        governance.constitution.allow_dissolution = true;
        households::form(
            &mut w,
            &s,
            households::Agreement {
                id: 1,
                agent: HOME,
                adults: vec![PERSON],
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
        s.balances.insert((HOME, TOKEN), cash);
        s.balances.insert((PERSON, TOKEN), 10);
        w.recovery.guarantees.push(Guarantee {
            id: 1,
            claim: GuaranteedClaim::Loan(1),
            guarantor: HOME,
            cap: 1,
            from: 1,
            through: 2,
            delay_months: 0,
            recourse: 101,
            priority: 0,
            follows_assignment: false,
            security: RecourseSecurity::Unsecured,
            tender: GuaranteeTender::AgreedCoins {
                resource: TOKEN,
                coins_per_unit: 2,
            },
        });
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut audit = Audit::with_opening(&w, &s, TOKEN, opening.clone()).unwrap();
            while sim.state.month < 3 {
                audit.step(&mut sim).unwrap();
            }
            let funded = cash == 2;
            assert_eq!(sim.state.balance(PERSON, TOKEN), 10);
            assert_eq!(sim.state.balance(HOME, TOKEN), if funded { 0 } else { 1 });
            assert_eq!(sim.state.credit.loans[&1].principal, i32::from(!funded));
            assert_eq!(
                d::blockers(&sim.world, &sim.state, HOME).contains(&d::Blocker::Loan),
                funded
            );
            if funded {
                let loan = &sim.state.credit.loans[&101];
                assert_eq!(
                    (
                        loan.creditor,
                        loan.debtor,
                        loan.denomination,
                        loan.principal
                    ),
                    (HOME, PERSON, GRAIN, 1)
                );
                let b = audit.book().balances();
                assert_eq!(b[&(HOME, Account::LoanReceivable(101))], 3);
                assert_eq!(b[&(PERSON, Account::LoanPayable(101))], -3);
            }
            d::request(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
            let mut resumed =
                Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
            let mut ra = audit.clone();
            let prefix = sim.ledger.len();
            while sim.state.month < 4 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month < 4 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger[prefix..], &audit),
                (&resumed.state, &resumed.ledger[..], &ra)
            );
            let before = sim.world.clone();
            assert_eq!(
                d::finish(&mut sim.world, &sim.state, HOME, PERSON).is_ok(),
                !funded
            );
            if funded {
                assert_eq!(sim.world, before);
                assert_eq!(sim.state.credit.loans[&101].principal, 1);
            } else {
                audit.step(&mut sim).unwrap();
                assert!(d::closed_at(&sim.world.households[0], 4));
                assert_eq!(sim.state.balance(PERSON, TOKEN), 11);
                assert_eq!(sim.state.credit.loans[&1].principal, 1);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
