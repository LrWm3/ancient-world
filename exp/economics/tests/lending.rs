use economics_compute_smoke::{
    agreements,
    compute::Backend,
    credit::{self, Advance, LoanOffer},
    finance::{ContractId, Execution, Transfer},
    model::*,
    scenario::{self, GRAIN, PERSON, STATE_AGENT, TOKEN},
    settlement,
    simulation::Simulation,
};

fn advance(
    id: u32,
    creditor: AgentId,
    debtor: AgentId,
    resource: ResourceId,
    principal: i32,
) -> Advance {
    Advance {
        id,
        debtor,
        terms: LoanOffer {
            creditor,
            denomination: resource,
            max_principal: principal,
            monthly_rate_bps: 0,
            term_months: 2,
            grace_months: 1,
        },
        principal,
        month: 1,
        collateral: None,
        priority: 0,
    }
}
fn fixture() -> (World, State) {
    let (mut w, mut s) = scenario::baseline();
    w.participants.clear();
    w.definitions.clear();
    w.rights.clear();
    w.agreements.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coins".into(),
        kind: ResourceKind::Stock,
    });
    w.storage.weights.insert(GRAIN, 1);
    w.lending = vec![advance(10, STATE_AGENT, PERSON, TOKEN, 10)];
    s.balances.clear();
    s.balances.insert((STATE_AGENT, TOKEN), 10);
    (w, s)
}
#[test]
fn unsecured_loans_use_the_existing_book_balances_and_cpu_settlement() {
    let (w, s) = fixture();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.balance(PERSON, TOKEN), 10);
    let loan = &sim.state.credit.loans[&10];
    assert!(loan.collateral.is_none());
    assert_eq!(loan.principal, 10);
    assert_eq!((loan.debtor, loan.creditor), (PERSON, STATE_AGENT));
    let views = agreements::for_agent(&sim.world, &sim.state, PERSON).unwrap();
    let agreements::View::Loan(view) = &views[0] else {
        panic!("missing loan view")
    };
    assert_eq!(view.title_holder(), None);
    assert_eq!(view.on_default(), None);
    let mut checkpoint = sim.clone();
    sim.run_months(2).unwrap();
    checkpoint.run_months(1).unwrap();
    checkpoint.run_months(1).unwrap();
    assert_eq!(sim.state, checkpoint.state);
    assert_eq!(sim.ledger, checkpoint.ledger);
    assert_eq!(sim.state.credit.loans[&10].status, credit::Status::Repaid);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 10);
}
#[test]
fn advance_cannot_rehypothecate_same_boundary_receipts_or_overdraw_lender() {
    let (mut w, s) = fixture();
    w.agents.push(Agent {
        id: 99,
        name: "another lender or borrower".into(),
    });
    w.lending.push(advance(11, PERSON, 99, TOKEN, 10));
    w.lending.push(advance(12, STATE_AGENT, 99, TOKEN, 10));
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.credit.loans.len(), 1);
    assert_eq!(sim.state.balance(99, TOKEN), 0);
}
#[test]
fn commodity_loan_honors_receiving_storage_and_leaves_no_debt_on_rejection() {
    let (mut w, mut s) = fixture();
    w.lending[0] = advance(10, STATE_AGENT, PERSON, GRAIN, 1000);
    s.balances.clear();
    s.balances.insert((STATE_AGENT, GRAIN), 1000);
    // Set real storage limits explicitly, independent of the baseline catalog.
    w.storage.capacities.insert(PERSON, 1);
    w.storage.capacities.insert(STATE_AGENT, 1000);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert!(sim.state.credit.loans.is_empty());
    assert_eq!(sim.state.balance(STATE_AGENT, GRAIN), 1000);
}
#[test]
fn shared_execution_does_not_retain_reservations_from_failed_atomic_exchange() {
    let (w, s) = fixture();
    let mut e = Execution::opening(&w, &s);
    let before = e.available.clone();
    let legs = vec![
        Transfer {
            from: STATE_AGENT,
            to: PERSON,
            amount: Amount::new(TOKEN, 5),
        },
        Transfer {
            from: PERSON,
            to: STATE_AGENT,
            amount: Amount::new(TOKEN, 5),
        },
    ];
    assert!(e.exchange(&w, &legs).is_err());
    assert_eq!(e.available, before);
    assert!(e.exchange(&w, &legs[..1]).is_ok());
    assert_eq!(e.available[&(STATE_AGENT, TOKEN)], 5);
}
#[test]
fn creditor_rank_changes_recovery_without_changing_total_resources() {
    fn run(prioritized: bool) -> Simulation {
        let (mut w, mut s) = fixture();
        w.agents.push(Agent {
            id: 99,
            name: "second creditor".into(),
        });
        s.balances.insert((99, TOKEN), 10);
        w.lending.push(advance(11, 99, PERSON, TOKEN, 10));
        if prioritized {
            w.claim_priorities.insert(ContractId::Loan(10), 1);
        }
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(1).unwrap();
        sim.state.balances.insert((PERSON, TOKEN), 5);
        sim.step().unwrap();
        sim.step().unwrap();
        sim
    }
    let a = run(false);
    let b = run(true);
    assert_eq!(a.state.credit.loans[&10].principal, 5);
    assert_eq!(a.state.credit.loans[&11].principal, 10);
    assert_eq!(b.state.credit.loans[&10].principal, 10);
    assert_eq!(b.state.credit.loans[&11].principal, 5);
    assert_eq!(a.state.balance(PERSON, TOKEN), 0);
    assert_eq!(b.state.balance(PERSON, TOKEN), 0);
}
#[test]
fn a_tampered_advance_rejects_without_publishing_money_or_debt() {
    let (w, s) = fixture();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.step().unwrap();
    sim.step().unwrap();
    let mut b = Batch::empty(&sim.state);
    b.credit = credit::evaluate(&sim.world, &sim.state).unwrap();
    b.transactions = b.credit.as_ref().unwrap().transactions.clone();
    b.credit
        .as_mut()
        .unwrap()
        .after
        .loans
        .get_mut(&10)
        .unwrap()
        .principal = 9;
    let before = sim.state.clone();
    assert!(settlement::commit(&sim.world, &mut sim.state, &b, Backend::Reference, 4096).is_err());
    assert_eq!(sim.state, before);
}

#[test]
fn loans_and_annual_land_claims_share_one_ranked_opening_budget() {
    use economics_compute_smoke::commitments;
    fn run(land_first: bool) -> Simulation {
        let (mut w, mut s) = scenario::named("annual-access").unwrap();
        w.participants.clear();
        w.condition_rules.clear();
        w.definitions.clear();
        w.priority = Priority::ContinuingFirst;
        w.lending = vec![advance(10, STATE_AGENT, PERSON, GRAIN, 10)];
        w.lending[0].month = 12;
        w.lending[0].terms.term_months = 1;
        if land_first {
            w.claim_priorities.insert(ContractId::Loan(10), 1);
        }
        w.payment_policy = commitments::PaymentPolicy::DebtFirst;
        s.month = 12;
        s.balances.clear();
        s.balances.insert((STATE_AGENT, GRAIN), 10);
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        sim.run_months(1).unwrap();
        sim.state.balances.insert((PERSON, GRAIN), 1);
        sim.step().unwrap();
        sim.step().unwrap();
        sim
    }
    let loans = run(false);
    let land = run(true);
    assert_eq!(loans.state.obligations[&(1, 13)].paid, 0);
    assert_eq!(land.state.obligations[&(1, 13)].paid, 1);
    assert_eq!(loans.state.credit.loans[&10].principal, 9);
    assert_eq!(land.state.credit.loans[&10].principal, 10);
    assert_eq!(loans.state.balance(PERSON, GRAIN), 0);
    assert_eq!(land.state.balance(PERSON, GRAIN), 0);
}

#[test]
fn laws_gate_general_borrowing_and_lending_without_stopping_existing_service() {
    use economics_compute_smoke::{
        laws,
        opportunities::{Action, PERSON_TYPE, Policy, STATE_TYPE},
    };
    use std::collections::{BTreeMap, BTreeSet};
    let (mut w, s) = fixture();
    w.transaction_policy = Some(Policy {
        authority: STATE_AGENT,
        laws: vec![],
        agreement_forms: Some(BTreeSet::from([laws::AgreementForm::Loan])),
        agreement_limits: Default::default(),
        membership_offers: vec![],
        membership_permissions: BTreeSet::new(),
        agent_types: BTreeMap::from([(PERSON, PERSON_TYPE), (STATE_AGENT, STATE_TYPE)]),
        permissions: BTreeSet::from([(PERSON_TYPE, Action::Borrow)]),
    });
    let mut denied = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    denied.run_months(1).unwrap();
    assert!(denied.state.credit.loans.is_empty());
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((STATE_TYPE, Action::Lend));
    let mut allowed = Simulation::new(w, s, Backend::Reference).unwrap();
    allowed.run_months(1).unwrap();
    assert_eq!(allowed.state.credit.loans.len(), 1);
    allowed
        .world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .clear();
    allowed.run_months(2).unwrap();
    assert_eq!(
        allowed.state.credit.loans[&10].status,
        credit::Status::Repaid
    );
}

#[test]
fn direct_lending_composes_with_existing_market_reservations() {
    use economics_compute_smoke::{currency, exchange};
    let (mut w, mut s) = fixture();
    w.agents.push(Agent {
        id: 99,
        name: "seller".into(),
    });
    s.balances.insert((99, GRAIN), 1);
    w.market = Some(exchange::Market::default());
    let m = w.market.as_mut().unwrap();
    m.targets.insert(1, 1);
    m.reserves.insert((99, GRAIN), 0);
    w.bids.push(currency::Bid {
        id: 1,
        buyer: STATE_AGENT,
        goods: Amount::new(GRAIN, 1),
        payment: Amount::new(TOKEN, 10),
    });
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.credit.loans.len(), 1);
    assert_eq!(sim.state.balance(99, GRAIN), 1);
    assert_eq!(sim.state.balance(99, TOKEN), 0);
    // Loan repayment becomes spendable at a later boundary; the market remains live.
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.balance(99, GRAIN), 0);
    assert_eq!(sim.state.balance(99, TOKEN), 10);
}

#[test]
fn direct_secured_advance_reuses_mortgage_enforcement_and_keeps_deficiency() {
    use economics_compute_smoke::scenario::PLOT;
    let (mut w, s) = fixture();
    w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = PERSON;
    w.lending[0].collateral = Some(credit::Collateral {
        asset: PLOT,
        priority: 0,
        pledged: true,
        settlement: credit::CollateralSettlement::FixedValue { value: 6 },
    });
    w.lending[0].terms.grace_months = 0;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    // External loss control: the borrower cannot meet the first scheduled payment.
    sim.state.balances.insert((PERSON, TOKEN), 0);
    sim.run_months(1).unwrap();
    assert_eq!(
        credit::owner(&sim.world, &sim.state, PLOT),
        Some(STATE_AGENT)
    );
    assert_eq!(sim.state.credit.loans[&10].principal, 4);
    assert_eq!(sim.state.credit.loans[&10].status, credit::Status::Enforced);
    assert_eq!(sim.state.credit.values[&PLOT], 6);
}

#[test]
fn direct_collateral_preserves_and_transfers_the_existing_crop_commitment() {
    use economics_compute_smoke::scenario::PLOT;
    let (mut w, mut s) = credit::crop_scenario(true).unwrap();
    let purchase = w.credit.take().unwrap();
    let offer = &purchase.offers[0];
    w.ownership_rights = purchase.attached_rights;
    w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = PERSON;
    w.lending = vec![Advance {
        id: 10,
        debtor: PERSON,
        terms: offer.loan.clone(),
        principal: 8000,
        month: 1,
        collateral: Some(offer.collateral.clone()),
        priority: 0,
    }];
    s.balances.insert((STATE_AGENT, TOKEN), 8000);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, TOKEN), 0);
    sim.run_months(1).unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Due);
    let before = sim.state.processes.values().next().unwrap().clone();
    sim.step().unwrap();
    let after = &sim.state.processes[&before.id];
    assert_eq!(after.operator, STATE_AGENT);
    assert_eq!(after.beneficiary, STATE_AGENT);
    assert_eq!(
        (after.stage, after.elapsed, after.status),
        (before.stage, before.elapsed, before.status)
    );
}

#[test]
fn essential_stock_protection_applies_to_loan_collection_too() {
    let (mut w, mut s) = scenario::named("payment-trap-protected").unwrap();
    w.priority = Priority::ContinuingFirst;
    let mut a = advance(10, STATE_AGENT, PERSON, GRAIN, 2);
    a.month = s.month;
    w.lending = vec![a];
    s.balances.insert((STATE_AGENT, GRAIN), 2);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, GRAIN), 1);
    sim.step().unwrap();
    let before = sim.state.clone();
    sim.step().unwrap();
    assert_eq!(sim.state.credit.loans[&10].principal, 2);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 1);
    let receipt = &sim
        .ledger
        .last()
        .unwrap()
        .credit
        .as_ref()
        .unwrap()
        .collections[0];
    assert_eq!((receipt.requested.quantity, receipt.paid), (1, 0));
    let mut unprotected = sim.world.clone();
    unprotected.payment_policy = economics_compute_smoke::commitments::PaymentPolicy::DebtFirst;
    let collection = credit::evaluate(&unprotected, &before).unwrap().unwrap();
    assert_eq!(collection.after.loans[&10].principal, 1);
}

#[test]
fn proportional_creditors_share_shortage_and_preserve_continuation() {
    use economics_compute_smoke::finance::CollectionPolicy;
    let (mut w, mut s) = fixture();
    w.collection_policy = CollectionPolicy::Proportional;
    w.agents.push(Agent {
        id: 99,
        name: "second creditor".into(),
    });
    s.balances.insert((99, TOKEN), 10);
    w.lending.push(advance(11, 99, PERSON, TOKEN, 10));
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, TOKEN), 5);
    sim.step().unwrap();
    let boundary = credit::evaluate(&sim.world, &sim.state).unwrap().unwrap();
    assert_eq!(
        boundary
            .collections
            .iter()
            .map(|r| (r.allocated, r.paid))
            .collect::<Vec<_>>(),
        vec![(Some(3), 3), (Some(2), 2)]
    );
    let mut resumed = sim.clone();
    resumed.world.lending.reverse();
    sim.run_months(2).unwrap();
    resumed.run_months(1).unwrap();
    resumed.run_months(1).unwrap();
    assert_eq!(sim.state, resumed.state);
    assert_eq!(sim.ledger, resumed.ledger);
    assert_eq!(sim.state.credit.loans[&10].principal, 7);
    assert_eq!(sim.state.credit.loans[&11].principal, 8);
}

#[test]
fn proportional_land_and_loan_dues_share_the_same_native_pool() {
    use economics_compute_smoke::{commitments::PaymentPolicy, finance::CollectionPolicy};
    let (mut w, mut s) = scenario::named("annual-access").unwrap();
    w.participants.clear();
    w.condition_rules.clear();
    w.definitions.clear();
    w.priority = Priority::ContinuingFirst;
    w.collection_policy = CollectionPolicy::Proportional;
    w.payment_policy = PaymentPolicy::DebtFirst;
    w.agreements[0].payment.quantity = 10;
    w.lending = vec![advance(10, STATE_AGENT, PERSON, GRAIN, 10)];
    w.lending[0].month = 12;
    w.lending[0].terms.term_months = 1;
    s.month = 12;
    s.balances.clear();
    s.balances.insert((STATE_AGENT, GRAIN), 10);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, GRAIN), 6);
    sim.step().unwrap();
    let boundary = credit::evaluate(&sim.world, &sim.state).unwrap().unwrap();
    assert_eq!(
        boundary
            .collections
            .iter()
            .map(|r| (r.requested.quantity, r.allocated, r.paid))
            .collect::<Vec<_>>(),
        vec![(10, Some(3), 3), (10, Some(3), 3)]
    );
    sim.step().unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 3);
    assert_eq!(sim.state.credit.loans[&10].principal, 7);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 0);
}

#[test]
fn proportional_allocator_redistributes_storage_limits_and_respects_rank() {
    use economics_compute_smoke::finance::{
        self, CollectionRequest, Condition, FailureRule, Obligation,
    };
    let (mut w, mut s) = fixture();
    s.balances.clear();
    s.balances.insert((PERSON, GRAIN), 12);
    w.storage.capacities.insert(STATE_AGENT, 2);
    let request = |id, rank, creditor, quantity| CollectionRequest {
        contract: ContractId::Loan(id),
        rank,
        claim: Obligation {
            transfer: Transfer {
                from: PERSON,
                to: creditor,
                amount: Amount::new(GRAIN, quantity),
            },
            settled: 0,
            condition: Condition::OnOrAfterMonth(1),
            failure: FailureRule::CarryArrears,
        },
    };
    let requests = vec![
        request(10, 0, STATE_AGENT, 10),
        request(11, 0, STATE_AGENT, 10),
        request(12, 0, 99, 10),
        request(13, 1, 98, 10),
    ];
    let mut protected = std::collections::BTreeMap::new();
    protected.insert((PERSON, GRAIN), 3);
    let execution = Execution::opening(&w, &s);
    let grants = finance::proportional_grants(&w, 1, &execution, &protected, &requests).unwrap();
    assert_eq!(
        grants[&ContractId::Loan(10)] + grants[&ContractId::Loan(11)],
        2
    );
    assert_eq!(grants[&ContractId::Loan(12)], 7);
    assert_eq!(grants[&ContractId::Loan(13)], 0);
    assert_eq!(execution.available[&(PERSON, GRAIN)], 12);
    let mut reversed = requests.clone();
    reversed.reverse();
    assert_eq!(
        grants,
        finance::proportional_grants(&w, 1, &execution, &protected, &reversed).unwrap()
    );
}

#[test]
fn proportional_allocator_leaves_future_and_indivisible_claims_out() {
    use economics_compute_smoke::finance::{
        self, CollectionRequest, Condition, FailureRule, Obligation,
    };
    let (w, s) = fixture();
    let mut request = CollectionRequest {
        contract: ContractId::Loan(1),
        rank: 0,
        claim: Obligation {
            transfer: Transfer {
                from: STATE_AGENT,
                to: PERSON,
                amount: Amount::new(TOKEN, 20),
            },
            settled: 0,
            condition: Condition::OnOrAfterMonth(2),
            failure: FailureRule::CarryArrears,
        },
    };
    let execution = Execution::opening(&w, &s);
    let grants =
        finance::proportional_grants(&w, 1, &execution, &Default::default(), &[request.clone()])
            .unwrap();
    assert_eq!(grants[&request.contract], 0);
    request.claim.failure = FailureRule::RejectExchange;
    assert!(
        finance::proportional_grants(&w, 2, &execution, &Default::default(), &[request]).is_err()
    );
}

#[test]
fn collateral_surplus_cannot_spend_another_claims_reserved_payment() {
    use economics_compute_smoke::{finance::CollectionPolicy, scenario::PLOT};
    let (mut w, mut s) = fixture();
    w.collection_policy = CollectionPolicy::Proportional;
    w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = PERSON;
    w.lending[0].terms.grace_months = 0;
    w.lending[0].collateral = Some(credit::Collateral {
        asset: PLOT,
        priority: 0,
        pledged: true,
        settlement: credit::CollateralSettlement::FixedValue { value: 15 },
    });
    w.agents.push(Agent {
        id: 99,
        name: "treasury lender".into(),
    });
    s.balances.insert((99, TOKEN), 10);
    w.lending.push(advance(11, 99, STATE_AGENT, TOKEN, 10));
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, TOKEN), 0);
    sim.state.balances.insert((STATE_AGENT, TOKEN), 5);
    sim.step().unwrap();
    let boundary = credit::evaluate(&sim.world, &sim.state).unwrap().unwrap();
    assert!(
        boundary
            .events
            .contains(&credit::Event::EnforcementDeferred {
                loan: 10,
                surplus: 5
            })
    );
    assert_eq!(boundary.collections[1].paid, 5);
    sim.step().unwrap();
    assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
    assert_eq!(sim.state.credit.loans[&11].principal, 5);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
}

#[test]
fn alternative_tender_competes_with_coin_loans_without_double_settlement() {
    use economics_compute_smoke::{activities::CoinPayment, finance::CollectionPolicy};
    let (mut w, mut s) = scenario::named("annual-access").unwrap();
    w.participants.clear();
    w.condition_rules.clear();
    w.definitions.clear();
    w.priority = Priority::ContinuingFirst;
    w.collection_policy = CollectionPolicy::Proportional;
    w.resources.push(Resource {
        id: TOKEN,
        name: "coins".into(),
        kind: ResourceKind::Stock,
    });
    w.agreements[0].payment.quantity = 4;
    w.activities.coin_payments.insert(
        1,
        CoinPayment {
            resource: TOKEN,
            coins_per_unit: 2,
        },
    );
    w.lending = vec![advance(10, STATE_AGENT, PERSON, TOKEN, 4)];
    w.lending[0].month = 12;
    w.lending[0].terms.term_months = 1;
    s.month = 12;
    s.balances.clear();
    s.balances.insert((STATE_AGENT, TOKEN), 4);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(1).unwrap();
    sim.state.balances.insert((PERSON, TOKEN), 5);
    sim.state.balances.insert((PERSON, GRAIN), 2);
    sim.step().unwrap();
    let boundary = credit::evaluate(&sim.world, &sim.state).unwrap().unwrap();
    assert_eq!(boundary.collections[0].paid, 3);
    assert_eq!(boundary.collections[1].paid, 3); // 2 native + 1 via two coins
    assert_eq!(boundary.collections[1].allocated, Some(3));
    sim.step().unwrap();
    assert_eq!(sim.state.obligations[&(1, 13)].in_kind_paid, 2);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 3);
    assert_eq!(sim.state.credit.loans[&10].principal, 1);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 5);
}
