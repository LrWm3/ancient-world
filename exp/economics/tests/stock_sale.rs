use economics_compute_smoke::{
    borrowing::{self, Policy, Reason},
    compute::Backend,
    credit,
    model::*,
    scenario::{GRAIN, NUTRITION, PERSON, STATE_AGENT, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    stock_sale,
};
fn sim(case: &str, backend: Backend) -> Simulation {
    let (w, s) = stock_sale::scenario(case).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn acquire(s: &mut Simulation) {
    s.step().unwrap();
    s.step().unwrap();
    assert_eq!(s.state.phase, Phase::Acquire);
}
#[test]
fn production_sales_support_repayment_only_with_sufficient_bid_funding_and_price() {
    for (case, accept, missed) in [
        ("funded", true, None),
        ("limited", false, Some(9)),
        ("food-tight", false, Some(11)),
    ] {
        let mut s = sim(case, Backend::CubeCpu);
        acquire(&mut s);
        let d = borrowing::evaluate(&s.world, &s.state).unwrap();
        assert_eq!(d.accept, accept);
        assert_eq!(d.purchase.as_ref().unwrap().missed_payment, missed);
        assert_eq!(
            d.reason,
            if accept {
                Reason::Beneficial
            } else {
                Reason::InstallmentShortfall
            }
        );
        s.run_months(18).unwrap();
        let selected = if accept {
            d.purchase.as_ref().unwrap()
        } else {
            &d.decline
        };
        assert_eq!(s.state.balance(PERSON, TOKEN), selected.closing_coins);
        let deficits: i64 = s
            .reports
            .iter()
            .filter(|r| r.agent == PERSON)
            .map(|r| i64::from(r.deficit(NUTRITION)))
            .sum();
        assert_eq!(deficits, selected.deficits[&NUTRITION]);
        assert_eq!(s.state.credit.loans.len(), usize::from(accept));
        let sales: Vec<_> = s
            .ledger
            .iter()
            .filter_map(|b| {
                b.credit
                    .as_ref()
                    .and_then(|c| c.stock_sale.as_ref())
                    .filter(|r| r.goods > 0)
                    .map(|r| (b.month, r.goods, r.coins))
            })
            .collect();
        if accept {
            assert_eq!(
                sales,
                vec![(7, 2, 2400), (8, 2, 2400), (15, 2, 2400), (16, 2, 2400)]
            );
            assert_eq!(s.state.credit.stock_spent, 9600);
            assert_eq!(s.state.balance(PERSON, TOKEN), 5580);
            assert_eq!(s.state.credit.loans[&1].status, credit::Status::Repaid);
            assert_eq!(deficits, 0);
        }
        for b in &s.ledger {
            for t in &b.transactions {
                if t.stock_trade.is_some() {
                    for resource in [GRAIN, TOKEN] {
                        assert_eq!(
                            t.effects
                                .iter()
                                .filter(|e| e.account.1 == resource)
                                .map(|e| e.delta)
                                .sum::<i32>(),
                            0
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn sales_respect_food_quantity_treasury_storage_and_total_allowance() {
    for (stock, cash, capacity, spent, expected) in [
        (6, 10000, 100, 0, 0),
        (7, 10000, 100, 0, 1),
        (20, 10000, 100, 0, 2),
        (20, 1199, 100, 0, 0),
        (20, 10000, 1, 0, 1),
        (20, 10000, 100, 12000, 0),
    ] {
        let mut s = sim("funded", Backend::Reference);
        s.world.credit.as_mut().unwrap().purchase_policy = Policy::Decline;
        acquire(&mut s);
        s.state.balances.insert((PERSON, GRAIN), stock);
        s.state.balances.insert((STATE_AGENT, TOKEN), cash);
        s.world.storage.capacities.insert(STATE_AGENT, capacity);
        s.state.credit.stock_spent = spent;
        let b = credit::evaluate(&s.world, &s.state).unwrap().unwrap();
        let r = b.stock_sale.unwrap();
        assert_eq!(r.reserve, 6);
        assert_eq!(r.sold_lots, expected);
        assert!(stock - r.goods >= 6);
        assert_eq!(b.after.stock_spent, spent + r.coins);
    }
}
#[test]
fn later_harvest_cannot_pay_an_earlier_installment() {
    let mut s = sim("funded", Backend::Reference);
    acquire(&mut s);
    s.state.balances.insert((PERSON, TOKEN), 2000);
    let d = borrowing::evaluate(&s.world, &s.state).unwrap();
    assert!(!d.accept);
    assert_eq!(d.purchase.unwrap().missed_payment, Some(2));
}
#[test]
fn forged_sale_receipts_are_atomic_and_committed_sales_cannot_repeat() {
    let mut s = sim("funded", Backend::Reference);
    s.world.credit.as_mut().unwrap().purchase_policy = Policy::Decline;
    acquire(&mut s);
    s.state.balances.insert((PERSON, GRAIN), 10);
    let mut valid = Batch::empty(&s.state);
    valid.credit = credit::evaluate(&s.world, &s.state).unwrap();
    valid.transactions = valid.credit.as_ref().unwrap().transactions.clone();
    let before = s.state.clone();
    for variant in 0..3 {
        let mut b = valid.clone();
        let c = b.credit.as_mut().unwrap();
        match variant {
            0 => c.stock_sale = None,
            1 => c.stock_sale.as_mut().unwrap().goods += 1,
            _ => c.after.stock_spent += 1,
        }
        assert!(
            settlement::commit(
                &s.world,
                &mut s.state,
                &b,
                Backend::Reference,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(s.state, before);
    }
    settlement::commit(
        &s.world,
        &mut s.state,
        &valid,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    let after = s.state.clone();
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &valid,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, after);
}
#[test]
fn cpu_monthly_checkpoint_and_reordered_reference_match() {
    let mut cpu = sim("funded", Backend::CubeCpu);
    cpu.run_months(7).unwrap();
    let mut reference = cpu.clone();
    reference.backend = Backend::Reference;
    reference.world.definitions.reverse();
    reference.world.agents.reverse();
    reference.world.resources.reverse();
    let mut monthly = cpu.clone();
    cpu.run_months(53).unwrap();
    reference.run_months(53).unwrap();
    for _ in 0..53 {
        monthly.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.state, monthly.state);
    assert_eq!(cpu.ledger, monthly.ledger);
}

#[test]
fn removing_food_reserve_can_make_debt_payable_by_sacrificing_meals() {
    let mut s = sim("food-tight", Backend::Reference);
    s.world
        .credit
        .as_mut()
        .unwrap()
        .stock_sales
        .as_mut()
        .unwrap()
        .reserve_months = 0;
    acquire(&mut s);
    let d = borrowing::evaluate(&s.world, &s.state).unwrap();
    let p = d.purchase.unwrap();
    assert_eq!(p.missed_payment, None);
    assert_eq!(p.closing_debt, 0);
    assert_eq!(p.deficits[&NUTRITION], 9);
    assert_eq!(p.months[0].sold_stock, 2);
    assert!(!d.accept);
    assert_eq!(
        d.reason,
        Reason::NeedLimitExceeded {
            resource: NUTRITION,
            projected: 9,
            maximum: 0
        }
    );
}

#[test]
fn cultivation_right_duration_explains_late_hunger_without_a_planner_change() {
    use economics_compute_smoke::scenario::{GROW, SEED};
    let mut extended = sim("funded", Backend::CubeCpu);
    // Isolate the right-duration diagnosis from the newer absolute borrowing guard.
    if let Policy::Compare(c) = &mut extended.world.credit.as_mut().unwrap().purchase_policy {
        c.need_limits.clear();
    }
    let mut short = extended.clone();
    short.world.rights[0].through = 9;
    let mut same_world = short.world.clone();
    same_world.rights = extended.world.rights.clone();
    assert_eq!(same_world, extended.world);
    assert_eq!(short.state, extended.state);
    short.run_months(18).unwrap();
    let deficit: i32 = short
        .reports
        .iter()
        .filter(|r| r.agent == PERSON)
        .map(|r| r.deficit(NUTRITION))
        .sum();
    assert_eq!(deficit, 2);
    let old_harvests: Vec<_> = short
        .state
        .processes
        .values()
        .filter(|p| p.definition == GROW && p.status == Status::Completed)
        .map(|p| p.reserved_through)
        .collect();
    assert_eq!(old_harvests, vec![6]);
    assert_eq!(short.state.balance(PERSON, SEED), 1);

    // Check every committed boundary: seed is either held or invested in one crop.
    while extended.state.month <= 60 {
        extended.step().unwrap();
        let active = extended
            .state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Active)
            .count() as i32;
        assert_eq!(extended.state.balance(PERSON, SEED) + active, 1);
    }
    assert!(
        extended
            .reports
            .iter()
            .filter(|r| r.agent == PERSON)
            .all(|r| r.deficit(NUTRITION) == 0)
    );
    let harvests: Vec<_> = extended
        .state
        .processes
        .values()
        .filter(|p| p.definition == GROW && p.status == Status::Completed)
        .map(|p| p.reserved_through)
        .collect();
    assert_eq!(harvests, vec![6, 14, 22, 32, 44, 56]);
    assert_eq!(extended.state.credit.stock_spent, 12000);
    assert_eq!(extended.state.balance(PERSON, TOKEN), 7980);
    assert_eq!(
        extended.state.credit.loans[&1].status,
        credit::Status::Repaid
    );
    assert!(
        extended
            .state
            .processes
            .values()
            .all(|p| p.status != Status::Aborted)
    );
}
