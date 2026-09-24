use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Event},
    model::*,
    resale,
    scenario::{GRAIN, PERSON, PLOT, SEED, STATE_AGENT, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
fn sim(case: &str, backend: Backend) -> Simulation {
    let (w, s) = resale::scenario(case).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn pending(s: &mut Simulation) {
    s.run_months(2).unwrap();
    s.step().unwrap(); // Open
    s.step().unwrap(); // Due repossession
    assert_eq!(s.state.credit.loans[&1].status, credit::Status::PendingSale);
    assert_eq!(s.state.phase, Phase::Acquire);
}
fn sale_boundary(s: &mut Simulation) {
    pending(s);
    while s.state.month < 4 {
        s.step().unwrap();
    }
    s.step().unwrap();
    s.step().unwrap();
    assert_eq!((s.state.month, s.state.phase), (4, Phase::Acquire));
}
fn batch(s: &Simulation) -> Batch {
    let mut b = Batch::empty(&s.state);
    b.credit = credit::evaluate(&s.world, &s.state).unwrap();
    b.transactions = b.credit.as_ref().unwrap().transactions.clone();
    b
}
fn cash(s: &Simulation) -> i32 {
    s.state
        .balances
        .iter()
        .filter(|((_, r), _)| *r == TOKEN)
        .map(|(_, n)| *n)
        .sum()
}
#[test]
fn repossession_defers_credit_and_freezes_interest_until_sale() {
    let mut s = sim("funded", Backend::CubeCpu);
    pending(&mut s);
    let loan = &s.state.credit.loans[&1];
    assert_eq!(
        (loan.principal, loan.interest, loan.last_accrued),
        (8000, 160, 3)
    );
    assert_eq!(credit::owner(&s.world, &s.state, PLOT), Some(STATE_AGENT));
    assert_eq!((loan.debtor, loan.creditor), (PERSON, STATE_AGENT));
    assert!(!s.state.credit.pending_sales.is_empty());
    s.step().unwrap(); // listing cannot sell immediately
    assert_eq!(s.state.credit.loans[&1].status, credit::Status::PendingSale);
    assert_eq!(s.state.credit.loans[&1].debt().unwrap(), 8160);
}
#[test]
fn actual_sale_pays_surplus_or_retains_deficiency_and_transfers_crop_atomically() {
    for (case, price, surplus, deficiency) in
        [("funded", 9500, 1340, 0), ("deficiency", 8000, 0, 160)]
    {
        let mut s = sim(case, Backend::CubeCpu);
        sale_boundary(&mut s);
        let before = s
            .state
            .processes
            .values()
            .find(|p| p.status == Status::Active)
            .unwrap()
            .clone();
        assert_eq!(
            (before.operator, before.stage, before.elapsed),
            (STATE_AGENT, 2, 0)
        );
        let opening_cash = cash(&s);
        let lender_cash = s.state.balance(STATE_AGENT, TOKEN);
        let buyer_cash = s.state.balance(resale::BUYER, TOKEN);
        let quoted = batch(&s);
        let quote = quoted
            .credit
            .as_ref()
            .unwrap()
            .events
            .iter()
            .find_map(|e| {
                if let Event::ResaleBid { quote, .. } = e {
                    Some(quote)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(
            (quote.with_asset, quote.without_asset, quote.crop_value),
            (2100, 1600, 500)
        );
        assert_eq!(quote.amount, price);
        s.step().unwrap();
        assert_eq!(cash(&s), opening_cash);
        assert_eq!(s.state.balance(PERSON, TOKEN), surplus);
        assert_eq!(
            s.state.balance(STATE_AGENT, TOKEN),
            lender_cash + price.min(8160)
        );
        assert_eq!(s.state.balance(resale::BUYER, TOKEN), buyer_cash - price);
        assert_eq!(s.state.credit.loans[&1].debt().unwrap(), deficiency);
        assert!(s.state.credit.pending_sales.is_empty());
        assert_eq!(credit::owner(&s.world, &s.state, PLOT), Some(resale::BUYER));
        let mut expected = before.clone();
        expected.operator = resale::BUYER;
        expected.beneficiary = resale::BUYER;
        expected.goal = None;
        assert_eq!(s.state.processes[&before.id], expected);
        assert_eq!(s.state.credit.values[&PLOT], price);
        s.run_months(3).unwrap();
        assert_eq!(s.state.balance(resale::BUYER, GRAIN), 8);
        assert_eq!(s.state.balance(resale::BUYER, SEED), 1);
        assert_eq!(s.state.credit.loans[&1].debt().unwrap(), deficiency);
        assert_eq!(
            s.ledger
                .iter()
                .filter_map(|b| b.credit.as_ref())
                .flat_map(|c| &c.events)
                .filter(|e| matches!(e, Event::Resold { .. }))
                .count(),
            1
        );
    }
}
#[test]
fn insufficient_or_missing_buyer_preserves_debt_and_lender_can_maintain_crop() {
    for case in ["insufficient", "no-buyer"] {
        let mut s = sim(case, Backend::CubeCpu);
        sale_boundary(&mut s);
        let before = s.state.clone();
        let b = batch(&s);
        assert!(b.credit.as_ref().unwrap().attachments.is_empty());
        assert!(b.transactions.is_empty());
        s.step().unwrap();
        assert_eq!(s.state.balances, before.balances);
        assert_eq!(s.state.credit, before.credit);
        assert_eq!(s.state.processes, before.processes);
        s.run_months(3).unwrap();
        assert_eq!(s.state.credit.loans[&1].status, credit::Status::PendingSale);
        assert_eq!(s.state.credit.loans[&1].debt().unwrap(), 8160);
        assert_eq!(s.state.credit.loans[&1].last_accrued, 3);
        assert_eq!(s.state.balance(STATE_AGENT, GRAIN), 8);
        assert_eq!(s.state.balance(STATE_AGENT, SEED), 1);
    }
}
#[test]
fn forged_proceeds_quote_or_attachment_reject_atomically_and_replay_cannot_sell_twice() {
    let mut s = sim("funded", Backend::CubeCpu);
    sale_boundary(&mut s);
    let opening = s.state.clone();
    let original = batch(&s);
    for mode in 0..3 {
        let mut b = original.clone();
        match mode {
            0 => b.credit.as_mut().unwrap().attachments.clear(),
            1 => b.transactions[0].effects[0].delta += 1,
            _ => {
                for e in &mut b.credit.as_mut().unwrap().events {
                    if let Event::ResaleBid { quote, .. } = e {
                        quote.crop_value += 1;
                    }
                }
            }
        }
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
        assert_eq!(s.state, opening);
    }
    s.step().unwrap();
    let sold = s.state.clone();
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &original,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, sold);
}
#[test]
fn buyer_capacity_and_future_shocks_affect_only_known_valuation_and_sale_can_retry() {
    let mut s = sim("funded", Backend::Reference);
    sale_boundary(&mut s);
    let before = batch(&s);
    s.world.capacity_overrides.insert((5, resale::BUYER), 0);
    assert_eq!(batch(&s), before);
    s.state
        .balances
        .insert((resale::BUYER, economics_compute_smoke::scenario::LABOR), 0);
    let b = batch(&s);
    let quote = b
        .credit
        .unwrap()
        .events
        .into_iter()
        .find_map(|e| {
            if let Event::ResaleBid { quote, .. } = e {
                Some(quote)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!((quote.crop_value, quote.amount), (0, 9000));

    let mut retry = sim("insufficient", Backend::CubeCpu);
    retry
        .world
        .credit
        .as_mut()
        .unwrap()
        .transfers
        .push(credit::ScheduledTransfer {
            month: 5,
            transfer: economics_compute_smoke::finance::Transfer {
                from: STATE_AGENT,
                to: resale::BUYER,
                amount: Amount::new(TOKEN, 3000),
            },
        });
    retry.run_months(6).unwrap();
    assert_eq!(
        credit::owner(&retry.world, &retry.state, PLOT),
        Some(resale::BUYER)
    );
    assert_eq!(retry.state.balance(PERSON, TOKEN), 840); // crop already harvested by lender; bare-land bid 90
    assert!(retry.state.credit.pending_sales.is_empty());
    assert_eq!(retry.state.balance(STATE_AGENT, GRAIN), 8);
}
#[test]
fn monthly_reference_checkpoint_and_table_order_preserve_resale_results() {
    for case in ["funded", "deficiency", "insufficient", "no-buyer"] {
        let mut cpu = sim(case, Backend::CubeCpu);
        cpu.run_months(6).unwrap();
        let mut reference = sim(case, Backend::Reference);
        reference.world.agents.reverse();
        reference.world.participants.reverse();
        reference.world.definitions.reverse();
        for _ in 0..6 {
            reference.run_months(1).unwrap();
        }
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        let mut prefix = sim(case, Backend::CubeCpu);
        sale_boundary(&mut prefix);
        let mut resumed =
            Simulation::new(prefix.world.clone(), prefix.state.clone(), Backend::CubeCpu).unwrap();
        resumed.run_months(3).unwrap();
        assert_eq!(cpu.state, resumed.state);
        assert_eq!(&cpu.ledger[prefix.ledger.len()..], resumed.ledger);
    }
}

#[test]
fn reserve_price_rejects_low_valuation_and_fixed_rule_remains_immediate() {
    let mut low = sim("funded", Backend::CubeCpu);
    low.world
        .credit
        .as_mut()
        .unwrap()
        .resale_buyer
        .as_mut()
        .unwrap()
        .land_value = 6000;
    sale_boundary(&mut low);
    let opening = low.state.clone();
    let b = batch(&low);
    assert!(b.credit.as_ref().unwrap().events.iter().any(|e| matches!(
        e,
        Event::ResaleRejected {
            bid: 6500,
            minimum_price: 7500,
            ..
        }
    )));
    low.step().unwrap();
    assert_eq!(low.state.credit, opening.credit);
    assert_eq!(low.state.balances, opening.balances);

    let mut fixed = sim("funded", Backend::CubeCpu);
    fixed.world.credit.as_mut().unwrap().offers[0]
        .collateral
        .settlement = credit::CollateralSettlement::FixedValue { value: 6000 };
    fixed.run_months(3).unwrap();
    assert_eq!(
        fixed.state.credit.loans[&1].status,
        credit::Status::Enforced
    );
    assert_eq!(fixed.state.credit.loans[&1].debt().unwrap(), 2160);
    assert!(fixed.state.credit.pending_sales.is_empty());
    assert!(
        !fixed
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|c| &c.events)
            .any(|e| matches!(e, Event::ResaleBid { .. }))
    );
}
