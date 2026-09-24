use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Event, Rejection, Status},
    finance::Transfer,
    model::*,
    scenario::{PERSON, PLOT, STATE_AGENT, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
fn sim(case: &str, backend: Backend) -> Simulation {
    let (w, s) = credit::scenario(case).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn to_acquire(sim: &mut Simulation) {
    sim.step().unwrap();
    sim.step().unwrap();
    assert_eq!(sim.state.phase, Phase::Acquire);
}
fn batch(sim: &Simulation) -> Batch {
    let mut b = Batch::empty(&sim.state);
    b.credit = credit::evaluate(&sim.world, &sim.state).unwrap();
    b.transactions = b.credit.as_ref().unwrap().transactions.clone();
    b
}
fn events(sim: &Simulation) -> Vec<&Event> {
    sim.ledger
        .iter()
        .filter_map(|b| b.credit.as_ref())
        .flat_map(|b| &b.events)
        .collect()
}
#[test]
fn purchase_is_atomic_and_both_balance_sheets_share_the_same_coin_debt() {
    let mut s = sim("repaid", Backend::CubeCpu);
    assert_eq!(credit::discover(&s.world, &s.state, PERSON).len(), 1);
    to_acquire(&mut s);
    let before = s.state.clone();
    let b = batch(&s);
    assert_eq!(s.state, before);
    assert!(s.state.credit.loans.is_empty());
    s.step().unwrap();
    let l = &s.state.credit.loans[&1];
    assert_eq!(l.principal, 8000);
    assert_eq!(l.interest, 0);
    assert_eq!(credit::owner(&s.world, &s.state, PLOT), Some(PERSON));
    assert!(l.collateral.as_ref().unwrap().pledged);
    let buyer = credit::balance_sheet(&s.world, &s.state, PERSON, TOKEN);
    let lender = credit::balance_sheet(&s.world, &s.state, STATE_AGENT, TOKEN);
    assert_eq!(
        (buyer.coins, buyer.assets, buyer.principal_payable),
        (0, 10000, 8000)
    );
    assert_eq!(
        (lender.coins, lender.assets, lender.principal_receivable),
        (102000, 0, 8000)
    );
    assert_eq!(buyer.equity(), 2000);
    assert_eq!(lender.equity(), 110000);
    assert!(
        b.credit
            .unwrap()
            .events
            .iter()
            .any(|e| matches!(e, Event::Purchased { advance: 8000, .. }))
    );
}
#[test]
fn lender_can_differ_from_seller_and_neither_downpayment_nor_advance_is_created() {
    let mut s = sim("repaid", Backend::CubeCpu);
    s.world.agents.push(Agent {
        id: 7,
        name: "independent lender".into(),
    });
    let c = s.world.credit.as_mut().unwrap();
    c.offers[0].loan.creditor = 7;
    c.transfers.clear();
    c.endowments.push(credit::Endowment {
        agent: 7,
        amount: Amount::new(TOKEN, 8000),
    });
    to_acquire(&mut s);
    let total_before: i32 = s.state.balances.values().sum();
    s.step().unwrap();
    assert_eq!(s.state.balance(7, TOKEN), 0);
    assert_eq!(s.state.balance(STATE_AGENT, TOKEN), 110000);
    assert_eq!(s.state.balances.values().sum::<i32>(), total_before);
    assert_eq!(
        credit::balance_sheet(&s.world, &s.state, 7, TOKEN).principal_receivable,
        8000
    );
    assert_eq!(s.state.credit.loans[&1].creditor, 7);
}
#[test]
fn missing_downpayment_or_lender_funding_leaves_title_and_debt_untouched() {
    for missing_downpayment in [true, false] {
        let mut s = sim(
            if missing_downpayment {
                "downpayment"
            } else {
                "repaid"
            },
            Backend::CubeCpu,
        );
        if !missing_downpayment {
            s.world.credit.as_mut().unwrap().endowments[1]
                .amount
                .quantity = 7999;
        }
        to_acquire(&mut s);
        let balances = s.state.balances.clone();
        s.step().unwrap();
        assert_eq!(s.state.balances, balances);
        assert!(s.state.credit.loans.is_empty());
        assert_eq!(credit::owner(&s.world, &s.state, PLOT), Some(STATE_AGENT));
        assert!(events(&s).contains(&&Event::Rejected {
            offer: 1,
            reason: if missing_downpayment {
                Rejection::Downpayment
            } else {
                Rejection::Funding
            }
        }));
    }
}
#[test]
fn repayment_and_recovery_charge_opening_principal_once_and_release_collateral() {
    for (case, interest) in [("repaid", 200), ("recovered", 220)] {
        let mut s = sim(case, Backend::CubeCpu);
        s.run_months(6).unwrap();
        let l = &s.state.credit.loans[&1];
        assert_eq!(l.status, Status::Repaid);
        assert_eq!(l.debt().unwrap(), 0);
        assert!(!l.collateral.as_ref().unwrap().pledged);
        assert_eq!(credit::owner(&s.world, &s.state, PLOT), Some(PERSON));
        let accrued: Vec<_> = events(&s)
            .into_iter()
            .filter_map(|e| {
                if let Event::Accrued { interest, .. } = e {
                    Some(*interest)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            accrued,
            if case == "repaid" {
                vec![80, 60, 40, 20]
            } else {
                vec![80, 80, 40, 20]
            }
        );
        assert_eq!(accrued.iter().sum::<i32>(), interest);
        assert_eq!(s.state.balance(PERSON, TOKEN), 400 - interest);
        assert_eq!(s.state.balances.values().sum::<i32>(), 102000);
    }
}
#[test]
fn default_transfers_collateral_credits_value_and_retains_deficiency_or_pays_surplus() {
    for (case, remaining, surplus) in [("default", 2160, 0), ("surplus", 0, 1840)] {
        let mut s = sim(case, Backend::CubeCpu);
        s.run_months(6).unwrap();
        let l = &s.state.credit.loans[&1];
        assert_eq!(l.debt().unwrap(), remaining);
        assert!(!l.collateral.as_ref().unwrap().pledged);
        assert_eq!(credit::owner(&s.world, &s.state, PLOT), Some(STATE_AGENT));
        assert_eq!(s.state.balance(PERSON, TOKEN), surplus);
        let enforcements: Vec<_> = events(&s)
            .into_iter()
            .filter(|e| matches!(e, Event::Enforced { .. }))
            .collect();
        assert_eq!(enforcements.len(), 1);
        assert_eq!(l.last_accrued, 3); // no interest on the enforced deficiency in this pilot
        let b = credit::balance_sheet(&s.world, &s.state, PERSON, TOKEN);
        let c = credit::balance_sheet(&s.world, &s.state, STATE_AGENT, TOKEN);
        assert_eq!(b.assets, 0);
        assert_eq!(b.principal_payable, c.principal_receivable);
        assert_eq!(b.interest_payable, c.interest_receivable);
        assert_eq!(c.assets, if case == "default" { 6000 } else { 10000 });
        assert_eq!(
            b.equity() + c.equity(),
            if case == "default" { 108000 } else { 112000 }
        );
    }
}
#[test]
fn surplus_must_be_funded_before_seizure_and_deficiency_can_be_repaid_later() {
    let mut s = sim("surplus", Backend::CubeCpu);
    s.world.agents.push(Agent {
        id: 7,
        name: "cash recipient".into(),
    });
    let c = s.world.credit.as_mut().unwrap();
    c.endowments[1].amount.quantity = 8000;
    c.transfers.push(credit::ScheduledTransfer {
        month: 2,
        transfer: Transfer {
            from: STATE_AGENT,
            to: 7,
            amount: Amount::new(TOKEN, 8500),
        },
    });
    s.run_months(3).unwrap();
    assert_eq!(credit::owner(&s.world, &s.state, PLOT), Some(PERSON));
    assert!(
        s.state.credit.loans[&1]
            .collateral
            .as_ref()
            .unwrap()
            .pledged
    );
    assert!(events(&s).contains(&&Event::EnforcementDeferred {
        loan: 1,
        surplus: 1840
    }));
    let mut d = sim("default", Backend::CubeCpu);
    d.world
        .credit
        .as_mut()
        .unwrap()
        .transfers
        .push(credit::ScheduledTransfer {
            month: 4,
            transfer: Transfer {
                from: STATE_AGENT,
                to: PERSON,
                amount: Amount::new(TOKEN, 2160),
            },
        });
    d.run_months(5).unwrap();
    assert_eq!(d.state.credit.loans[&1].status, Status::Repaid);
    assert_eq!(credit::owner(&d.world, &d.state, PLOT), Some(STATE_AGENT)); // paying deficiency doesn't repurchase the plot
}
#[test]
fn fractional_interest_carries_without_compounding_and_payment_is_interest_first() {
    let mut s = sim("default", Backend::Reference);
    let c = s.world.credit.as_mut().unwrap();
    c.offers[0].loan.monthly_rate_bps = 1;
    c.offers[0].loan.grace_months = 120;
    s.run_months(5).unwrap();
    let l = &s.state.credit.loans[&1];
    assert_eq!(l.interest, 3);
    assert_eq!(l.interest_remainder, 2000);
    assert_eq!(l.principal, 8000);
    let mut partial = sim("default", Backend::CubeCpu);
    let c = partial.world.credit.as_mut().unwrap();
    c.offers[0].loan.grace_months = 120;
    c.transfers.push(credit::ScheduledTransfer {
        month: 2,
        transfer: Transfer {
            from: STATE_AGENT,
            to: PERSON,
            amount: Amount::new(TOKEN, 30),
        },
    });
    partial.run_months(2).unwrap();
    let l = &partial.state.credit.loans[&1];
    assert_eq!(l.principal, 8000);
    assert_eq!(l.interest, 50);
    partial.run_months(1).unwrap();
    assert_eq!(partial.state.credit.loans[&1].interest, 130); // not 130.5
}
#[test]
fn tampered_origination_replay_and_due_receipts_reject_without_partial_state() {
    let mut s = sim("repaid", Backend::CubeCpu);
    to_acquire(&mut s);
    let valid = batch(&s);
    for case in 0..3 {
        let mut b = valid.clone();
        match case {
            0 => {
                b.credit
                    .as_mut()
                    .unwrap()
                    .after
                    .loans
                    .get_mut(&1)
                    .unwrap()
                    .principal = 1
            }
            1 => {
                b.credit
                    .as_mut()
                    .unwrap()
                    .after
                    .owners
                    .insert(PLOT, STATE_AGENT);
            }
            _ => b.transactions.clear(),
        }
        let mut state = s.state.clone();
        assert!(
            settlement::commit(
                &s.world,
                &mut state,
                &b,
                Backend::CubeCpu,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(state, s.state);
    }
    s.step().unwrap();
    let before = s.state.clone();
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &valid,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, before);
    while s.state.month < 2 || s.state.phase != Phase::Due {
        s.step().unwrap();
    }
    let mut b = batch(&s);
    b.credit
        .as_mut()
        .unwrap()
        .after
        .loans
        .get_mut(&1)
        .unwrap()
        .interest = 999;
    let before = s.state.clone();
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &b,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, before);
}
#[test]
fn monthly_batch_checkpoint_and_table_order_preserve_the_full_loan_ledger() {
    let mut batched = sim("recovered", Backend::CubeCpu);
    let mut monthly = batched.clone();
    let mut reference = batched.clone();
    reference.backend = Backend::Reference;
    reference.world.agents.reverse();
    reference
        .world
        .credit
        .as_mut()
        .unwrap()
        .endowments
        .reverse();
    // Endowment presentation order is catalog order; physical state is order-independent.
    batched.run_months(6).unwrap();
    for _ in 0..6 {
        monthly.run_months(1).unwrap();
    }
    assert_eq!(monthly.state, batched.state);
    assert_eq!(monthly.ledger, batched.ledger);
    reference.run_months(6).unwrap();
    assert_eq!(reference.state, batched.state);
    let mut checkpoint = sim("recovered", Backend::CubeCpu);
    checkpoint.run_months(2).unwrap();
    checkpoint.step().unwrap(); // month 3 Due, before recovery
    let mut resumed = checkpoint.clone();
    resumed.run_months(4).unwrap();
    assert_eq!(resumed.state, batched.state);
    assert_eq!(resumed.ledger, batched.ledger);
}
