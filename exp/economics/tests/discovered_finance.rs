use economics_compute_smoke::{
    agency::objectives::{Metric, Objective, Scope},
    compute::Backend,
    discovery::scenario,
    minting::*,
    model::*,
    opportunities::{Action, PERSON_TYPE},
    simulation::Simulation,
};

fn surplus(quantity: i32) -> (World, State) {
    let (mut w, mut s) = scenario::scenario().unwrap();
    for p in &mut w.participants {
        p.needs.clear();
    }
    let c = w.discovery.as_mut().unwrap();
    c.land = None;
    c.household = None;
    c.state.as_mut().unwrap().objectives = vec![Objective {
        scope: Scope::Organization,
        metric: Metric::Reserve {
            resource: WHEAT,
            target: 1,
        },
    }];
    s.balances.insert((ISSUER, WHEAT), 0);
    s.balances.insert((ISSUER, COIN), 1);
    s.balances.insert((WORKER, WHEAT), quantity);
    (w, s)
}
fn run(w: World, s: State, backend: Backend) -> Simulation {
    let mut audit = scenario::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.month <= 10 {
        audit.step(&mut sim).unwrap();
    }
    sim
}

#[test]
fn exact_one_unit_surplus_can_be_discovered_and_delivered() {
    for quantity in [1, 2] {
        let (w, s) = surplus(quantity);
        let sim = run(w, s, Backend::CubeCpu);
        assert_eq!(sim.state.exchange.forwards.len(), 1);
        let f = sim.state.exchange.forwards.values().next().unwrap();
        assert_eq!((f.debtor, f.creditor, f.delivered), (WORKER, ISSUER, 1));
        assert_eq!(sim.state.balance(WORKER, WHEAT), quantity - 1);
        assert_eq!(sim.state.balance(ISSUER, WHEAT), 1);
    }
}

#[test]
fn absent_needed_committed_or_forbidden_stock_is_not_a_forward_surplus() {
    for case in ["absent", "needed", "committed", "forbidden"] {
        let (mut w, mut s) = surplus(if case == "absent" { 0 } else { 1 });
        if case == "needed" {
            w.participants
                .iter_mut()
                .find(|p| p.agent == WORKER)
                .unwrap()
                .needs = vec![Requirement {
                resource: NUTRITION,
                quantity: 1,
                priority: 0,
            }];
        }
        if case == "forbidden" {
            w.transaction_policy
                .as_mut()
                .unwrap()
                .permissions
                .remove(&(PERSON_TYPE, Action::StockTrade));
        }
        if case == "committed" {
            let t = economics_compute_smoke::forward::direct::Terms {
                id: 900,
                seller: WORKER,
                buyer: SUPPLIER,
                month: 1,
                due: 3,
                goods: Amount::new(WHEAT, 1),
                prepayment: Amount::new(COIN, 1),
            };
            s.exchange.forwards.insert(t.id, t.contract());
            w.prepaid_deliveries.push(t);
        }
        // Existing accepted opening claims use the ordinary simulation; fresh
        // positive cases above additionally reconcile complete financial books.
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(6).unwrap();
        assert!(
            sim.state
                .exchange
                .forwards
                .values()
                .all(|f| f.creditor != ISSUER || f.debtor != WORKER),
            "{case}"
        );
        if case == "committed" {
            assert_eq!(sim.state.exchange.forwards[&900].delivered, 1);
        }
        if case == "forbidden" {
            use economics_compute_smoke::discovery::finance::{Outcome, Performance};
            assert!(
                sim.world
                    .discovery
                    .as_ref()
                    .unwrap()
                    .financial
                    .iter()
                    .flat_map(|a| &a.attempts)
                    .any(|a| a.counterparty == WORKER
                        && a.outcome == Outcome::PerformanceShortfall
                        && a.performance == Some(Performance::NotAdmitted))
            );
        }
    }
}

fn mint_loan(held_metal: i32) -> (World, State) {
    let (mut w, mut s) = scenario::scenario().unwrap();
    let c = w.discovery.as_mut().unwrap();
    c.land = None;
    c.household = None;
    c.finance.as_mut().unwrap().unit_values.clear();
    // Isolate lot financing with an interest-free project and a term whose
    // first installment leaves the next-month procurement budget intact.
    c.finance.as_mut().unwrap().monthly_rate_bps = 0;
    c.finance.as_mut().unwrap().loan_months = 8;
    for p in &mut w.participants {
        p.needs.clear();
        if p.agent != WORKER {
            p.capacity.quantity = 0;
        }
    }
    s.balances.insert((ISSUER, COIN), 0);
    s.balances.insert((ISSUER, WHEAT), 0);
    s.balances.insert((ISSUER, METAL), held_metal);
    // A single candidate lender makes the funding boundary observable.
    s.balances.insert((WORKER, COIN), 0);
    (w, s)
}

#[test]
fn partial_lot_financing_matches_real_input_orders_and_repays() {
    for (held, principal) in [(0, 6), (1, 6), (2, 4)] {
        let (w, s) = mint_loan(held);
        let reference = run(w.clone(), s.clone(), Backend::Reference);
        let cpu = run(w, s, Backend::CubeCpu);
        assert_eq!(cpu.world, reference.world);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        let loan = cpu.state.credit.loans.values().next().unwrap_or_else(|| {
            panic!(
                "held={held}: {:?}",
                cpu.world.discovery.as_ref().unwrap().receipts
            )
        });
        assert_eq!(loan.original_principal, principal);
        assert_eq!(loan.status, economics_compute_smoke::credit::Status::Repaid);
        assert!(
            cpu.state
                .processes
                .values()
                .any(|p| p.definition == MINT && p.status == Status::Completed)
        );
        assert_eq!(
            cpu.state.balance(ISSUER, METAL),
            if held == 1 { 1 } else { 0 }
        );
    }
}

#[test]
fn rounded_input_funding_requires_real_lender_money_and_supply() {
    for case in ["cash", "metal", "storage", "early installment"] {
        let (mut w, mut s) = mint_loan(1);
        match case {
            "cash" => {
                s.balances.insert((SUPPLIER, COIN), 5);
            }
            "metal" => {
                s.balances.insert((SUPPLIER, METAL), 0);
            }
            "storage" => {
                w.storage.capacities.insert(ISSUER, 1);
            }
            "early installment" => {
                w.discovery
                    .as_mut()
                    .unwrap()
                    .finance
                    .as_mut()
                    .unwrap()
                    .loan_months = 4;
            }
            _ => unreachable!(),
        }
        let sim = run(w, s, Backend::Reference);
        assert!(sim.state.credit.loans.is_empty(), "{case}");
        assert!(
            !sim.state
                .processes
                .values()
                .any(|p| p.definition == MINT && p.status == Status::Completed)
        );
    }
}

#[test]
fn mint_underwriting_cannot_treat_grain_as_procurement_coins() {
    let (mut w, s) = mint_loan(1);
    w.discovery
        .as_mut()
        .unwrap()
        .finance
        .as_mut()
        .unwrap()
        .denomination = WHEAT;
    let err = match Simulation::new(w, s, Backend::Reference) {
        Ok(_) => panic!("mixed-unit underwriting was admitted"),
        Err(err) => err,
    };
    assert!(err.contains("procurement currency"), "{err}");
}

#[test]
fn reserve_objectives_cannot_publish_duplicate_forward_requests() {
    for other_resource in [WHEAT, METAL] {
        let mut results = vec![];
        for reverse in [false, true] {
            let (mut w, mut s) = surplus(2);
            s.balances.insert((ISSUER, COIN), 2);
            let c = w.discovery.as_mut().unwrap();
            c.finance
                .as_mut()
                .unwrap()
                .unit_values
                .insert(other_resource, 1);
            c.state.as_mut().unwrap().objectives.push(Objective {
                scope: Scope::Organization,
                metric: Metric::Reserve {
                    resource: other_resource,
                    target: 1,
                },
            });
            if reverse {
                c.state.as_mut().unwrap().objectives.reverse();
            }
            let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
            sim.run_months(2).unwrap();
            assert_eq!(sim.world.prepaid_deliveries.len(), 1);
            assert_eq!(sim.state.exchange.forwards.len(), 1);
            results.push(sim.world.prepaid_deliveries.clone());
        }
        assert_eq!(results[0], results[1]);
    }
}

#[test]
fn financial_assessments_distinguish_no_supply_from_published_delivery() {
    use economics_compute_smoke::discovery::finance::{Instrument, Outcome};
    for quantity in [0, 1] {
        let (w, s) = surplus(quantity);
        let sim = run(w, s, Backend::Reference);
        let assessments: Vec<_> = sim
            .world
            .discovery
            .as_ref()
            .unwrap()
            .financial
            .iter()
            .filter(|a| a.instrument == Instrument::Forward && a.requester == ISSUER)
            .collect();
        assert!(!assessments.is_empty());
        if quantity == 0 {
            assert!(
                assessments
                    .iter()
                    .all(|a| a.candidate_count == 0 && a.attempts.is_empty())
            );
        } else {
            let a = assessments
                .iter()
                .find(|a| a.attempts.iter().any(|p| p.outcome == Outcome::Published))
                .unwrap();
            let p = a
                .attempts
                .iter()
                .find(|p| p.outcome == Outcome::Published)
                .unwrap();
            assert_eq!(
                (a.resource, a.denomination, p.quantity, p.prepayment),
                (WHEAT, COIN, 1, Some(1))
            );
            assert_eq!(sim.state.exchange.forwards[&p.candidate_id].delivered, 1);
            assert_eq!(
                p.performance,
                Some(
                    economics_compute_smoke::discovery::finance::Performance::Forward {
                        delivered: 1,
                        outstanding: 0
                    }
                )
            );
        }
    }
}

#[test]
fn loan_diagnostics_distinguish_funding_gain_and_publication_from_admission() {
    use economics_compute_smoke::discovery::finance::{Instrument, Outcome};
    for case in ["funded", "no funds", "early installment"] {
        let (mut w, mut s) = mint_loan(1);
        if case == "no funds" {
            s.balances.insert((SUPPLIER, COIN), 0);
        }
        if case == "early installment" {
            w.discovery
                .as_mut()
                .unwrap()
                .finance
                .as_mut()
                .unwrap()
                .loan_months = 4;
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        while sim.state.month < 2 {
            sim.step().unwrap();
        }
        sim.step().unwrap(); // Publishing Open is before loan admission at Acquire.
        let assessments: Vec<_> = sim
            .world
            .discovery
            .as_ref()
            .unwrap()
            .financial
            .iter()
            .filter(|a| a.instrument == Instrument::Loan && a.month == 2)
            .collect();
        assert_eq!(assessments.len(), 1, "{case}");
        let a = assessments[0];
        assert_eq!(
            (a.requester, a.resource, a.denomination),
            (ISSUER, COIN, COIN)
        );
        assert!(sim.state.credit.loans.is_empty());
        if case == "no funds" {
            assert_eq!(a.candidate_count, 0);
            assert!(a.attempts.is_empty());
        } else {
            assert_eq!(a.attempts.len(), 1);
            let p = &a.attempts[0];
            assert_eq!(
                (p.counterparty, p.quantity, p.prepayment),
                (SUPPLIER, 6, None)
            );
            match case {
                "funded" => {
                    assert_eq!(p.outcome, Outcome::Published);
                    assert_eq!(sim.world.lending.len(), 1);
                }
                "early installment" => {
                    assert_eq!(p.outcome, Outcome::NoMutualGain);
                    assert_eq!(
                        p.performance,
                        Some(
                            economics_compute_smoke::discovery::finance::Performance::Loan {
                                status: economics_compute_smoke::credit::Status::Repaid,
                                outstanding: 0,
                            }
                        )
                    );
                }
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn discovery_interest_rates_use_the_contract_execution_bounds() {
    for rate in [0, 10_000, 10_001, u32::MAX] {
        let (mut w, s) = mint_loan(1);
        w.discovery
            .as_mut()
            .unwrap()
            .finance
            .as_mut()
            .unwrap()
            .monthly_rate_bps = rate;
        let result = Simulation::new(w, s, Backend::Reference);
        if rate <= 10_000 {
            assert!(result.is_ok());
        } else {
            let error = result.expect_err("invalid rate was accepted");
            assert!(error.contains("discovery monthly interest rate"), "{error}");
        }
    }
}

#[test]
fn discovered_loans_respect_unused_purchase_offer_identities() {
    use economics_compute_smoke::{borrowing, credit};
    for (offer_id, recourse) in [(1, None), (99, None), (99, Some(100))] {
        let (mut w, s) = mint_loan(1);
        let asset = w.assets.iter().find(|a| a.owner == ISSUER).unwrap().id;
        w.credit = Some(credit::Config {
            stock_sales: None,
            purchase_policy: borrowing::Policy::Decline,
            resale_buyer: None,
            attached_rights: Default::default(),
            offers: vec![credit::Offer {
                id: offer_id,
                sale: credit::Sale {
                    asset,
                    seller: ISSUER,
                    price: Amount::new(COIN, 10),
                },
                loan: credit::LoanOffer {
                    creditor: ISSUER,
                    denomination: COIN,
                    max_principal: 9,
                    monthly_rate_bps: 0,
                    term_months: 8,
                    grace_months: 1,
                },
                minimum_downpayment: 1,
                collateral: credit::Collateral {
                    asset,
                    priority: 1,
                    settlement: credit::CollateralSettlement::FixedValue { value: 10 },
                    pledged: true,
                },
            }],
            application: credit::Application {
                offer: offer_id,
                buyer: WORKER,
                month: 1,
                downpayment: 1,
            },
            endowments: vec![],
            transfers: vec![],
        });
        if let Some(recourse) = recourse {
            use economics_compute_smoke::recovery::{
                Guarantee, GuaranteeTender, GuaranteedClaim, RecourseSecurity,
            };
            w.recovery.guarantees.push(Guarantee {
                id: 700,
                claim: GuaranteedClaim::Loan(offer_id),
                guarantor: SUPPLIER,
                follows_assignment: false,
                tender: GuaranteeTender::Native,
                security: RecourseSecurity::Unsecured,
                cap: 1,
                from: 1,
                through: 12,
                delay_months: 0,
                recourse,
                priority: 0,
            });
        }
        let reference = run(w.clone(), s.clone(), Backend::Reference);
        let cpu = run(w, s, Backend::CubeCpu);
        assert_eq!(cpu.world, reference.world);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        let loan = &cpu.state.credit.loans[&(recourse.unwrap_or(offer_id) + 1)];
        if let Some(recourse) = recourse {
            assert!(!cpu.state.credit.loans.contains_key(&recourse));
            assert!(cpu.state.credit.recovery.paid_guarantees.is_empty());
        }
        assert_eq!(
            (loan.original_principal, loan.status),
            (6, credit::Status::Repaid)
        );
        assert!(!cpu.state.credit.loans.contains_key(&offer_id));
        assert_eq!(credit::owner(&cpu.world, &cpu.state, asset), Some(ISSUER));
        assert!(
            cpu.state
                .processes
                .values()
                .any(|p| p.definition == MINT && p.status == Status::Completed)
        );
    }
}

#[test]
fn bounded_loan_duration_search_preserves_primary_preference_and_finds_viable_terms() {
    use economics_compute_smoke::{
        credit,
        discovery::finance::{Instrument, Outcome},
    };
    for (primary, alternatives, expected) in [
        (4, vec![], None),
        (4, vec![4, 8], Some(8)),
        (8, vec![4, 8], Some(8)),
    ] {
        let (mut w, s) = mint_loan(1);
        let rule = w.discovery.as_mut().unwrap().finance.as_mut().unwrap();
        rule.loan_months = primary;
        rule.alternative_loan_months = alternatives.into_iter().collect();
        let reference = run(w.clone(), s.clone(), Backend::Reference);
        let cpu = run(w, s, Backend::CubeCpu);
        assert_eq!(cpu.world, reference.world);
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        let assessments: Vec<_> = cpu
            .world
            .discovery
            .as_ref()
            .unwrap()
            .financial
            .iter()
            .filter(|a| a.month == 2 && a.instrument == Instrument::Loan)
            .collect();
        assert_eq!(assessments[0].duration, primary);
        assert_eq!(
            assessments.len(),
            if primary == 4 && expected.is_some() {
                2
            } else {
                1
            }
        );
        if let Some(term) = expected {
            assert_eq!(cpu.world.lending.len(), 1);
            assert_eq!(cpu.world.lending[0].terms.term_months, term);
            assert_eq!(
                cpu.state.credit.loans[&cpu.world.lending[0].id].status,
                credit::Status::Repaid
            );
            let published = assessments.last().unwrap();
            if primary == 4 {
                assert_eq!(assessments[0].attempts[0].outcome, Outcome::NoMutualGain);
                assert_eq!(
                    assessments[0].attempts[0].candidate_id,
                    published.attempts[0].candidate_id
                );
            }
            assert_eq!(published.duration, term);
            assert_eq!(published.horizon, term + 2);
            assert_eq!(published.attempts[0].outcome, Outcome::Published);
            assert!(
                cpu.state
                    .processes
                    .values()
                    .any(|p| p.definition == MINT && p.status == Status::Completed)
            );
        } else {
            assert!(cpu.world.lending.is_empty());
            assert_eq!(assessments[0].attempts[0].outcome, Outcome::NoMutualGain);
        }
    }
    for term in [0, 23, u32::MAX] {
        let (mut w, s) = mint_loan(1);
        w.discovery
            .as_mut()
            .unwrap()
            .finance
            .as_mut()
            .unwrap()
            .alternative_loan_months
            .insert(term);
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
}

#[test]
fn projected_unpaid_loan_is_distinct_from_nonadmission() {
    use economics_compute_smoke::discovery::finance::{Instrument, Outcome, Performance};
    let (mut w, s) = mint_loan(1);
    w.discovery
        .as_mut()
        .unwrap()
        .finance
        .as_mut()
        .unwrap()
        .monthly_rate_bps = 10_000;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.month < 2 {
        sim.step().unwrap();
    }
    sim.step().unwrap();
    let attempt = &sim
        .world
        .discovery
        .as_ref()
        .unwrap()
        .financial
        .iter()
        .find(|a| a.month == 2 && a.instrument == Instrument::Loan)
        .unwrap()
        .attempts[0];
    assert_eq!(attempt.outcome, Outcome::PerformanceShortfall);
    assert!(
        matches!(attempt.performance, Some(Performance::Loan {outstanding, ..}) if outstanding > 0),
        "{attempt:?}"
    );
    assert!(sim.world.lending.is_empty());
    assert!(sim.state.credit.loans.is_empty());
}
