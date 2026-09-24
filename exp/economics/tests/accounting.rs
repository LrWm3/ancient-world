use economics_compute_smoke::{
    accounting::{Account as A, Book, Entry, Flow, Line},
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::Audit,
    model::*,
    scenario::{self, PERSON, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
fn fixture() -> (World, State) {
    let (mut w, mut s) = scenario::baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights.clear();
    w.agreements.clear();
    w.assets.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    w.lending.push(Advance {
        id: 10,
        debtor: PERSON,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: 10,
            monthly_rate_bps: 1000,
            term_months: 1,
            grace_months: 10,
        },
        principal: 10,
        month: 1,
        collateral: None,
        priority: 0,
    });
    s.balances = BTreeMap::from([((STATE_AGENT, TOKEN), 100), ((PERSON, TOKEN), 20)]);
    (w, s)
}
fn through(a: &mut Audit, s: &mut Simulation, month: u32) {
    while s.state.month <= month {
        a.step(s).unwrap();
    }
}
#[test]
fn lending_statements_separate_principal_from_income_and_reconcile_on_cpu() {
    let (w, s) = fixture();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut a = Audit::new(&w, &s, TOKEN).unwrap();
    let mut b = a.clone();
    through(&mut a, &mut reference, 2);
    through(&mut b, &mut cpu, 2);
    assert_eq!(a, b);
    assert_eq!(reference.state, cpu.state);
    let lender = a.book().statements(STATE_AGENT, 1, 2).unwrap();
    let borrower = a.book().statements(PERSON, 1, 2).unwrap();
    assert_eq!(
        (
            lender.assets,
            lender.liabilities,
            lender.equity,
            lender.net_income
        ),
        (101, 0, 101, 1)
    );
    assert_eq!(
        (
            borrower.assets,
            borrower.liabilities,
            borrower.equity,
            borrower.net_income
        ),
        (19, 0, 19, -1)
    );
    assert_eq!(borrower.cash_flows[&Flow::Financing], 0);
    assert_eq!(borrower.cash_flows[&Flow::Operating], -1);
    assert_eq!(lender.cash_flows[&Flow::Investing], 0);
    assert_eq!(lender.cash_flows[&Flow::Operating], 1);
    assert_eq!(
        a.book().statements(STATE_AGENT, 2, 2).unwrap().opening_cash,
        90
    );
    let mut resumed = Audit::new(&w, &cpu.state, TOKEN).unwrap();
    // Starting a new book intentionally rebases opening equity; cloning preserves history.
    let mut continued = a.clone();
    let mut sim = reference.clone();
    through(&mut continued, &mut reference, 3);
    through(&mut a, &mut sim, 3);
    assert_eq!(continued, a);
    resumed.step(&mut cpu).unwrap();
}
#[test]
fn guarantees_create_an_asset_not_an_expense_and_keep_interest_in_operating_cash() {
    use economics_compute_smoke::recovery::Guarantee;
    let (mut w, mut s) = fixture();
    w.agents.push(Agent {
        id: 7,
        name: "guarantor".into(),
    });
    w.recovery.guarantees.push(Guarantee {
        id: 1,
        loan: 10,
        guarantor: 7,
        cap: 11,
        from: 1,
        through: 5,
        delay_months: 0,
        recourse: 20,
        priority: 0,
    });
    s.balances.insert((7, TOKEN), 11);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, TOKEN), 0); // Explicit loss before this reporting opening.
    let mut a = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
    through(&mut a, &mut sim, 2);
    let g = a.book().statements(7, 2, 2).unwrap();
    assert_eq!((g.assets, g.net_income, g.closing_cash), (11, 0, 0));
    assert_eq!(g.trial_balance[&A::LoanReceivable(20)], 11);
    assert_eq!(g.cash_flows[&Flow::Investing], -11);
    let l = a.book().statements(STATE_AGENT, 2, 2).unwrap();
    assert_eq!(
        (
            l.net_income,
            l.cash_flows[&Flow::Operating],
            l.cash_flows[&Flow::Investing]
        ),
        (1, 1, 10)
    );
}
#[test]
fn estate_custody_is_not_income_and_explicit_discharge_has_no_cash_flow() {
    use economics_compute_smoke::recovery::ProceedingTerms;
    let (w, s) = fixture();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, TOKEN), 0);
    sim.run_months(1).unwrap();
    sim.world.agents.push(Agent {
        id: 9,
        name: "estate".into(),
    });
    sim.world.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: 9,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![],
        discharge_deficiency: true,
    });
    sim.state.balances.insert((PERSON, TOKEN), 5); // Part of explicit reporting opening.
    let mut a = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
    through(&mut a, &mut sim, 3);
    let estate = a.book().statements(9, 3, 3).unwrap();
    assert_eq!(
        (
            estate.assets,
            estate.liabilities,
            estate.equity,
            estate.closing_cash
        ),
        (5, 5, 0, 0)
    );
    let borrower = a.book().statements(PERSON, 3, 3).unwrap();
    assert_eq!(
        (borrower.closing_cash, borrower.cash_flows[&Flow::Internal]),
        (5, 0)
    );
    through(&mut a, &mut sim, 4);
    let lender = a.book().statements(STATE_AGENT, 3, 4).unwrap();
    assert_eq!(
        (lender.net_income, lender.expenses[&A::CreditLoss]),
        (-6, 6)
    );
    assert_eq!(lender.cash_flows[&Flow::Operating], 1);
    assert_eq!(lender.cash_flows[&Flow::Investing], 4);
    let borrower = a.book().statements(PERSON, 3, 4).unwrap();
    assert_eq!(borrower.income[&A::DebtRelief], 6);
    assert_eq!(
        (
            borrower.opening_equity,
            borrower.equity,
            borrower.closing_cash
        ),
        (-6, 0, 0)
    );
}
fn line(a: A, debit: i128, flow: Option<Flow>) -> Line {
    Line {
        agent: PERSON,
        account: a,
        debit,
        flow,
    }
}
#[test]
fn full_statements_support_inventory_sales_and_explicit_capital_without_income_plugs() {
    let mut b = Book::open(
        TOKEN,
        BTreeMap::from([((PERSON, A::Cash), 20), ((PERSON, A::Inventory(1)), 8)]),
    )
    .unwrap();
    b.post(Entry {
        id: "sale".into(),
        month: 1,
        batch: Some(1),
        description: "Inventory sale at cost 8, price 12".into(),
        lines: vec![
            line(A::Cash, 12, Some(Flow::Operating)),
            line(A::Sales, -12, None),
            line(A::CostOfSales, 8, None),
            line(A::Inventory(1), -8, None),
        ],
    })
    .unwrap();
    b.post(Entry {
        id: "distribution".into(),
        month: 2,
        batch: Some(2),
        description: "Capital distribution".into(),
        lines: vec![
            line(A::Cash, -5, Some(Flow::Financing)),
            line(A::Capital, 5, None),
        ],
    })
    .unwrap();
    let s = b.statements(PERSON, 1, 2).unwrap();
    assert_eq!(
        (
            s.income[&A::Sales],
            s.expenses[&A::CostOfSales],
            s.net_income
        ),
        (12, 8, 4)
    );
    assert_eq!((s.opening_equity, s.capital_change, s.equity), (28, -5, 27));
    assert_eq!((s.opening_cash, s.closing_cash), (20, 27));
    assert_eq!(s.trial_balance.values().sum::<i128>(), 0);
    let second = b.statements(PERSON, 2, 2).unwrap();
    assert_eq!(
        (second.opening_equity, second.net_income, second.equity),
        (32, 0, 27)
    );
}
#[test]
fn journal_rejects_entity_cross_netting_duplicates_overflow_and_unclassified_cash_atomically() {
    let mut b = Book::open(TOKEN, BTreeMap::from([((PERSON, A::Cash), 20)])).unwrap();
    for lines in [
        vec![
            line(A::Cash, 1, Some(Flow::Operating)),
            Line {
                agent: STATE_AGENT,
                account: A::Sales,
                debit: -1,
                flow: None,
            },
        ],
        vec![line(A::Cash, 1, None), line(A::Sales, -1, None)],
        vec![
            line(A::Cash, -21, Some(Flow::Financing)),
            line(A::Capital, 21, None),
        ],
        vec![
            line(A::Cash, i128::MAX, Some(Flow::Financing)),
            line(A::Capital, -i128::MAX, None),
        ],
    ] {
        let before = b.clone();
        assert!(
            b.post(Entry {
                id: "bad".into(),
                month: 1,
                batch: None,
                description: "bad".into(),
                lines
            })
            .is_err()
        );
        assert_eq!(b, before);
    }
    let e = Entry {
        id: "ok".into(),
        month: 1,
        batch: None,
        description: "income".into(),
        lines: vec![
            line(A::Cash, 1, Some(Flow::Operating)),
            line(A::Sales, -1, None),
        ],
    };
    b.post(e.clone()).unwrap();
    let before = b.clone();
    assert!(b.post(e).is_err());
    assert_eq!(b, before);
}
#[test]
fn reporting_rejects_missing_valuation_and_unjournaled_state_changes() {
    let (mut w, s) = fixture();
    let mut audit = Audit::new(&w, &s, TOKEN).unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    sim.state.balances.insert((PERSON, TOKEN), 21);
    let before = sim.state.clone();
    let old = audit.clone();
    assert!(audit.step(&mut sim).is_err());
    assert_eq!(sim.state, before);
    assert_eq!(audit, old);
    w.assets.push(Asset {
        id: 99,
        owner: PERSON,
        kind: 1,
    });
    assert!(Audit::new(&w, &s, TOKEN).is_err());
    let (w, mut s) = fixture();
    s.balances.insert((PERSON, scenario::GRAIN), 1);
    assert!(Audit::new(&w, &s, TOKEN).is_err());
}

#[test]
fn financed_purchase_and_fixed_repossession_recognize_noncash_assets_and_losses() {
    use economics_compute_smoke::{credit, scenario::PLOT};
    for case in ["repaid", "default", "surplus", "downpayment"] {
        let (w, s) = credit::scenario(case).unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
        let mut audit =
            Audit::with_assets(&w, &s, TOKEN, BTreeMap::from([(PLOT, 10_000)])).unwrap();
        through(&mut audit, &mut sim, 1);
        let buyer = audit.book().statements(PERSON, 1, 1).unwrap();
        if case != "downpayment" {
            assert_eq!(buyer.assets, 10_000);
            assert_eq!(buyer.liabilities, 8_000);
            assert_eq!(buyer.net_income, 0);
            assert_eq!(buyer.trial_balance[&A::LoanPayable(1)], -8_000);
            let lender = audit.book().statements(STATE_AGENT, 1, 1).unwrap();
            assert_eq!(lender.trial_balance[&A::LoanReceivable(1)], 8_000);
            assert_eq!(lender.closing_cash, 102_000);
            assert_eq!(lender.equity, 110_000);
            assert_eq!(buyer.equity, 2_000);
            assert_eq!(buyer.cash_flows[&Flow::Financing], 2_000);
            assert_eq!(buyer.cash_flows[&Flow::Investing], -2_000);
        }
        through(&mut audit, &mut sim, 5);
        for agent in [PERSON, STATE_AGENT] {
            let report = audit.book().statements(agent, 1, 5).unwrap();
            assert_eq!(report.assets - report.liabilities, report.equity);
            let expected = match (case, agent) {
                ("default", PERSON) => -2160,
                ("default", STATE_AGENT) => 110160,
                ("surplus", PERSON) => 1840,
                ("surplus", STATE_AGENT) => 110160,
                ("downpayment", PERSON) => 1999,
                ("downpayment", STATE_AGENT) => 110000,
                ("repaid", PERSON) => 10200,
                ("repaid", STATE_AGENT) => 101800,
                _ => unreachable!(),
            };
            assert_eq!(report.equity, expected, "{case} agent={agent}");
        }
        if case == "default" {
            let buyer = audit.book().statements(PERSON, 1, 5).unwrap();
            assert_eq!(buyer.expenses[&A::DisposalLoss], 4_000);
            assert_eq!(buyer.cash_flows[&Flow::Investing], -2_000);
        }
    }
}

#[test]
fn opening_period_and_extreme_values_are_rejected_without_panics() {
    assert!(
        Book::open(
            TOKEN,
            BTreeMap::from([((PERSON, A::LoanPayable(1)), i128::MIN)])
        )
        .is_err()
    );
    let b = Book::open_at(TOKEN, 4, BTreeMap::from([((PERSON, A::Cash), 2)])).unwrap();
    assert!(b.statements(PERSON, 4, 5).is_err());
    assert_eq!(b.statements(PERSON, 5, 5).unwrap().opening_cash, 2);
}

#[test]
fn actual_resale_retains_borrower_asset_until_sale_and_records_only_received_cash() {
    use economics_compute_smoke::{credit, resale, scenario::PLOT, work_choice};
    for sell in [false, true] {
        let (mut w, s) = credit::scenario("default").unwrap();
        w.agents.push(Agent {
            id: resale::BUYER,
            name: "buyer".into(),
        });
        w.resources.extend([
            Resource {
                id: scenario::LABOR,
                name: "labor".into(),
                kind: ResourceKind::Capacity,
            },
            Resource {
                id: scenario::GRAIN,
                name: "grain".into(),
                kind: ResourceKind::Stock,
            },
        ]);
        w.participants.push(Participant {
            agent: resale::BUYER,
            capacity: Amount::new(scenario::LABOR, 1),
            needs: vec![],
        });
        let c = w.credit.as_mut().unwrap();
        c.offers[0].collateral.settlement = credit::CollateralSettlement::ResaleProceeds {
            minimum_price: 7_500,
        };
        c.endowments.push(credit::Endowment {
            agent: resale::BUYER,
            amount: Amount::new(TOKEN, 10_000),
        });
        if sell {
            c.resale_buyer = Some(resale::Buyer {
                from_month: 1,
                land_value: 9_000,
                preferences: work_choice::Config {
                    agent: resale::BUYER,
                    horizon: 1,
                    values: BTreeMap::from([(scenario::GRAIN, 1)]),
                },
            });
        }
        let mut audit =
            Audit::with_assets(&w, &s, TOKEN, BTreeMap::from([(PLOT, 10_000)])).unwrap();
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        through(&mut audit, &mut sim, 3);
        assert_eq!(
            audit.book().statements(PERSON, 1, 3).unwrap().trial_balance[&A::Tangible(PLOT)],
            10_000
        );
        assert_eq!(
            sim.state.credit.loans[&1].status,
            credit::Status::PendingSale
        );
        let lender = audit.book().statements(STATE_AGENT, 1, 3).unwrap();
        let debtor = audit.book().statements(PERSON, 1, 3).unwrap();
        assert_eq!(
            lender
                .trial_balance
                .get(&A::Tangible(PLOT))
                .copied()
                .unwrap_or(0),
            0
        );
        assert_eq!(lender.trial_balance[&A::LoanReceivable(1)], 8_000);
        assert_eq!(lender.trial_balance[&A::InterestReceivable(1)], 160);
        assert_eq!(debtor.trial_balance[&A::LoanPayable(1)], -8_000);
        assert_eq!(debtor.trial_balance[&A::InterestPayable(1)], -160);
        through(&mut audit, &mut sim, 4);
        let debtor = audit.book().statements(PERSON, 4, 4).unwrap();
        if sell {
            assert_eq!(debtor.expenses[&A::DisposalLoss], 1_000);
            assert_eq!(debtor.closing_cash, 840);
            assert_eq!(debtor.cash_flows[&Flow::Investing], 840);
            let lender = audit.book().statements(STATE_AGENT, 4, 4).unwrap();
            assert_eq!(lender.cash_flows[&Flow::Operating], 160);
            assert_eq!(lender.cash_flows[&Flow::Investing], 8_000);
            let buyer = audit.book().statements(resale::BUYER, 4, 4).unwrap();
            assert_eq!(buyer.trial_balance[&A::Tangible(PLOT)], 9_000);
            assert_eq!(buyer.cash_flows[&Flow::Investing], -9_000);
        } else {
            assert_eq!(debtor.net_income, 0);
            assert_eq!(debtor.trial_balance[&A::Tangible(PLOT)], 10_000);
        }
    }
}

#[test]
fn estate_asset_sale_recognizes_disposal_loss_and_custody_then_discharge() {
    use economics_compute_smoke::recovery::{Bid, Listing, ProceedingTerms};
    let (mut w, mut s) = fixture();
    w.agents.extend([
        Agent {
            id: 8,
            name: "buyer".into(),
        },
        Agent {
            id: 9,
            name: "estate".into(),
        },
    ]);
    w.assets.push(Asset {
        id: 99,
        owner: PERSON,
        kind: 1,
    });
    s.balances.insert((8, TOKEN), 8);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, TOKEN), 0);
    sim.run_months(1).unwrap();
    sim.world.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: 9,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![Listing {
            asset: 99,
            minimum_price: 6,
        }],
        discharge_deficiency: true,
    });
    sim.world.recovery.bids.push(Bid {
        id: 1,
        proceeding: 1,
        buyer: 8,
        asset: 99,
        month: 3,
        price: 8,
    });
    let mut audit =
        Audit::with_assets(&sim.world, &sim.state, TOKEN, BTreeMap::from([(99, 10)])).unwrap();
    through(&mut audit, &mut sim, 3);
    let debtor = audit.book().statements(PERSON, 3, 3).unwrap();
    assert_eq!(debtor.expenses[&A::DisposalLoss], 2);
    assert_eq!(debtor.cash_flows[&Flow::Investing], 8);
    assert_eq!(audit.book().statements(9, 3, 3).unwrap().equity, 0);
    through(&mut audit, &mut sim, 4);
    let debtor = audit.book().statements(PERSON, 3, 4).unwrap();
    assert_eq!(debtor.income[&A::DebtRelief], 3);
    assert_eq!(debtor.equity, 0);
    assert_eq!(debtor.closing_cash, 0);
    assert_eq!(
        audit.book().statements(STATE_AGENT, 3, 4).unwrap().expenses[&A::CreditLoss],
        3
    );
}

#[test]
fn third_party_mortgage_lender_and_seller_have_distinct_books() {
    use economics_compute_smoke::{credit, scenario::PLOT};
    let (mut w, mut s) = credit::scenario("default").unwrap();
    w.agents.push(Agent {
        id: 7,
        name: "bank".into(),
    });
    w.credit.as_mut().unwrap().offers[0].loan.creditor = 7;
    s.balances.insert((7, TOKEN), 8_000);
    let mut a = Audit::with_assets(&w, &s, TOKEN, BTreeMap::from([(PLOT, 7_000)])).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut sim, 1);
    let seller = a.book().statements(STATE_AGENT, 1, 1).unwrap();
    assert_eq!(seller.income[&A::DisposalGain], 3_000);
    assert_eq!(seller.cash_flows[&Flow::Investing], 10_000);
    let bank = a.book().statements(7, 1, 1).unwrap();
    assert_eq!((bank.net_income, bank.closing_cash), (0, 0));
    assert_eq!(bank.trial_balance[&A::LoanReceivable(1)], 8_000);
    assert_eq!(bank.cash_flows[&Flow::Investing], -8_000);
    assert_eq!(
        a.book().statements(PERSON, 1, 1).unwrap().cash_flows[&Flow::Financing],
        2_000
    );
}

#[test]
fn journal_archive_rebuilds_large_integer_balances_and_finalization_guards() {
    let large = i128::from(u64::MAX) + 123;
    let mut book = Book::open(TOKEN, BTreeMap::from([((PERSON, A::Cash), large)])).unwrap();
    let entry = |id: &str, month| Entry {
        id: id.into(),
        month,
        batch: None,
        description: "expense".into(),
        lines: vec![
            Line {
                agent: PERSON,
                account: A::Cash,
                debit: -1,
                flow: Some(Flow::Operating),
            },
            Line {
                agent: PERSON,
                account: A::ProductionExpense,
                debit: 1,
                flow: None,
            },
        ],
    };
    book.post(entry("one", 1)).unwrap();
    assert!(book.finalized_statements(PERSON, 1, 1).is_err());
    book.finalize_through(1).unwrap();
    let frozen = book.finalized_statements(PERSON, 1, 1).unwrap();
    let mut loaded = Book::from_json(&book.to_json().unwrap()).unwrap();
    assert_eq!(book, loaded);
    assert_eq!(loaded.balances()[&(PERSON, A::Cash)], large - 1);
    let old = loaded.clone();
    assert!(loaded.post(entry("late", 1)).is_err());
    assert!(loaded.finalize_through(2).is_err());
    assert!(loaded.finalize_through(0).is_err());
    assert_eq!(old, loaded);
    loaded.post(entry("two", 2)).unwrap();
    assert_eq!(frozen, loaded.finalized_statements(PERSON, 1, 1).unwrap());
    assert!(loaded.finalized_statements(PERSON, 1, 2).is_err());
    loaded.finalize_through(2).unwrap();
    assert_eq!(loaded, Book::from_json(&loaded.to_json().unwrap()).unwrap());
}

#[test]
fn malformed_journals_cannot_bypass_posting_validation() {
    let mut book = Book::open(TOKEN, BTreeMap::from([((PERSON, A::Cash), 10)])).unwrap();
    book.post(Entry {
        id: "one".into(),
        month: 1,
        batch: Some(1),
        description: "empty boundary".into(),
        lines: vec![],
    })
    .unwrap();
    let json = book.to_json().unwrap();
    for bad in [
        json.replace("\"version\": 1", "\"version\": 999"),
        json.replace("\"debit\": 10", "\"debit\": 11"),
        json.replace("\"id\": \"one\"", "\"id\": \"opening\""),
        json.replace("\"finalized_through\": null", "\"finalized_through\": 2"),
        json.replace("\"Cash\"", "\"Sales\""),
        "{}".into(),
    ] {
        assert!(Book::from_json(&bad).is_err(), "{bad}");
    }
}

#[test]
fn audit_only_finalizes_completed_months_and_continues_on_cpu() {
    let (w, s) = fixture();
    let mut a = Audit::new(&w, &s, TOKEN).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    a.step(&mut sim).unwrap();
    let old = a.clone();
    assert!(a.finalize_through(1).is_err());
    assert_eq!(old, a);
    through(&mut a, &mut sim, 1);
    a.finalize_through(1).unwrap();
    let report = a.book().finalized_statements(PERSON, 1, 1).unwrap();
    let mut resumed = a.clone();
    let mut checkpoint = sim.clone();
    through(&mut a, &mut sim, 3);
    through(&mut resumed, &mut checkpoint, 3);
    assert_eq!(a, resumed);
    assert_eq!(report, a.book().finalized_statements(PERSON, 1, 1).unwrap());
    assert_eq!(
        *a.book(),
        Book::from_json(&a.book().to_json().unwrap()).unwrap()
    );
}

#[test]
fn explicit_separate_scope_preserves_related_agent_debts_and_labels_exports() {
    use economics_compute_smoke::accounting::ReportingScope;
    // The household owes a member ten coins. Other related agents stay outside
    // both separate statements, regardless of how a caller groups them.
    let household = 900;
    let member = 901;
    let subsidiary = 902;
    let mut book = Book::open(
        TOKEN,
        BTreeMap::from([
            ((household, A::Cash), 10),
            ((household, A::LoanPayable(1)), -10),
            ((member, A::LoanReceivable(1)), 10),
            ((subsidiary, A::Cash), 70),
        ]),
    )
    .unwrap();
    let scope = ReportingScope::Separate { agent: household };
    let report = book.statements_for_scope(&scope, 1, 1).unwrap();
    assert_eq!(report.scope, scope);
    assert_eq!(report.assets, 10);
    assert_eq!(report.liabilities, 10);
    assert_eq!(report, book.statements(household, 1, 1).unwrap());
    let member_report = book.statements(member, 1, 1).unwrap();
    assert_eq!(member_report.assets, 10);
    assert_eq!(member_report.trial_balance[&A::LoanReceivable(1)], 10);
    let text = report.markdown(TOKEN);
    assert!(text.contains("Reporting scope: Separate agent 900"));
    assert!(text.contains("LoanPayable(1)"));
    assert!(book.finalized_statements_for_scope(&scope, 1, 1).is_err());
    book.post(Entry {
        id: "close:1".into(),
        month: 1,
        batch: None,
        description: "Completed unchanged reporting period".into(),
        lines: vec![],
    })
    .unwrap();
    book.finalize_through(1).unwrap();
    assert_eq!(
        book.finalized_statements_for_scope(&scope, 1, 1).unwrap(),
        report
    );
    assert_eq!(book.finalized_statements(household, 1, 1).unwrap(), report);
}

#[test]
fn consolidated_scope_requires_eliminations_and_never_changes_individual_books() {
    use economics_compute_smoke::accounting::ReportingScope;
    use std::collections::BTreeSet;
    let mut book = Book::open(
        TOKEN,
        BTreeMap::from([
            ((1, A::LoanReceivable(7)), 10),
            ((2, A::LoanPayable(7)), -10),
            ((2, A::Cash), 10),
        ]),
    )
    .unwrap();
    book.post(Entry {
        id: "close:1".into(),
        month: 1,
        batch: None,
        description: "Completed unchanged reporting period".into(),
        lines: vec![],
    })
    .unwrap();
    book.finalize_through(1).unwrap();
    let before = book.clone();
    for entities in [BTreeSet::new(), BTreeSet::from([1]), BTreeSet::from([1, 2])] {
        let scope = ReportingScope::Consolidated { entities };
        let encoded = serde_json::to_string(&scope).unwrap();
        assert_eq!(
            serde_json::from_str::<ReportingScope>(&encoded).unwrap(),
            scope
        );
        assert!(
            book.statements_for_scope(&scope, 1, 1)
                .unwrap_err()
                .contains("elimination adapter")
        );
        assert!(book.finalized_statements_for_scope(&scope, 1, 1).is_err());
        assert_eq!(book, before);
    }
    let restored = Book::from_json(&book.to_json().unwrap()).unwrap();
    assert_eq!(restored.statements(1, 1, 1).unwrap().assets, 10);
    assert_eq!(restored.statements(2, 1, 1).unwrap().liabilities, 10);
}
