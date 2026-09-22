use economics_compute_smoke::{
    borrowing::{self, Policy, Reason},
    compute::Backend,
    credit,
    finance::Transfer,
    model::*,
    offers,
    scenario::{GRAIN, NUTRITION, PERSON, STATE_AGENT, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
};
fn sim(case: &str, backend: Backend) -> Simulation {
    let (w, s) = borrowing::scenario(case).unwrap();
    Simulation::new(w, s, backend).unwrap()
}
fn acquire(s: &mut Simulation) {
    s.step().unwrap();
    s.step().unwrap();
    assert_eq!(s.state.phase, Phase::Acquire);
}
fn batch(s: &Simulation) -> Batch {
    let mut b = Batch::empty(&s.state);
    b.credit = credit::evaluate(&s.world, &s.state).unwrap();
    b.transactions = b.credit.as_ref().unwrap().transactions.clone();
    b
}
#[test]
fn cpu_selects_affordable_benefit_and_declines_unpayable_or_unhelpful_loans() {
    for (case, expected, debt, deficits) in [
        ("affordable", Reason::Beneficial, 0, 0),
        ("unaffordable", Reason::InstallmentShortfall, 2160, 4),
        ("unhelpful", Reason::NotBeneficial, 0, 4),
    ] {
        let mut s = sim(case, Backend::CubeCpu);
        acquire(&mut s);
        let opening = s.clone();
        let d = borrowing::evaluate(&s.world, &s.state).unwrap();
        assert_eq!(s.state, opening.state);
        assert_eq!(s.ledger, opening.ledger);
        assert_eq!(s.world, opening.world);
        assert_eq!(d.reason, expected);
        let purchase = d.purchase.as_ref().unwrap();
        assert_eq!(purchase.closing_debt, debt);
        assert_eq!(purchase.deficits[&NUTRITION], deficits);
        assert_eq!(d.decline.deficits[&NUTRITION], 3);
        assert_eq!(purchase.months.len(), 12);
        assert!(purchase.months.iter().all(|m| m.labor <= 2));
        let request = offers::Request::new(offers::Id::FinancedPurchase(1), PERSON);
        assert_eq!(offers::feasible(&s, &[request]).is_ok(), d.accept);
        s.run_months(12).unwrap();
        let selected = if d.accept { purchase } else { &d.decline };
        assert_eq!(s.state.balance(PERSON, TOKEN), selected.closing_coins);
        assert_eq!(s.state.credit.loans.len(), usize::from(d.accept));
        let actual_deficit: i64 = s
            .reports
            .iter()
            .filter(|r| r.agent == PERSON)
            .map(|r| i64::from(r.deficit(NUTRITION)))
            .sum();
        assert_eq!(actual_deficit, selected.deficits[&NUTRITION]);
        assert_eq!(
            s.ledger
                .iter()
                .filter_map(|b| b.credit.as_ref())
                .filter(|b| b.decision.is_some())
                .count(),
            1
        );
        if d.accept {
            assert_eq!(s.state.credit.loans[&1].status, credit::Status::Repaid);
            assert_eq!(s.state.balance(PERSON, TOKEN), 100);
        }
    }
}
#[test]
fn grain_and_hidden_gifts_do_not_pay_installments_and_short_horizons_are_rejected() {
    let mut s = sim("unaffordable", Backend::Reference);
    acquire(&mut s);
    let expected = borrowing::evaluate(&s.world, &s.state).unwrap();
    s.world
        .credit
        .as_mut()
        .unwrap()
        .transfers
        .push(credit::ScheduledTransfer {
            month: 2,
            transfer: Transfer {
                from: STATE_AGENT,
                to: PERSON,
                amount: Amount::new(TOKEN, 10000),
            },
        });
    s.world.capacity_overrides.insert((2, PERSON), 0);
    assert_eq!(expected, borrowing::evaluate(&s.world, &s.state).unwrap());
    s.state.balances.insert((PERSON, GRAIN), 100);
    let d = borrowing::evaluate(&s.world, &s.state).unwrap();
    assert_eq!(d.reason, Reason::InstallmentShortfall);
    assert_eq!(d.purchase.unwrap().missed_payment, Some(2));
    s.world.credit.as_mut().unwrap().purchase_policy =
        Policy::Compare(borrowing::Config { horizon_months: 4 });
    assert!(Simulation::new(s.world, s.state, Backend::Reference).is_err());
}
#[test]
fn failed_acceptance_and_forged_decision_publish_nothing() {
    let mut s = sim("affordable", Backend::CubeCpu);
    acquire(&mut s);
    let valid = batch(&s);
    let before = s.state.clone();
    for missing in [true, false] {
        let mut forged = valid.clone();
        let boundary = forged.credit.as_mut().unwrap();
        if missing {
            boundary.decision = None;
        } else {
            boundary.decision.as_mut().unwrap().accept = false;
        }
        assert!(
            settlement::commit(
                &s.world,
                &mut s.state,
                &forged,
                Backend::CubeCpu,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(s.state, before);
    }
    s.state.balances.insert((PERSON, TOKEN), 1);
    let d = borrowing::evaluate(&s.world, &s.state).unwrap();
    assert!(matches!(d.reason, Reason::Infeasible(_)));
    assert!(!d.accept);
    assert!(d.purchase.is_none());
}
#[test]
fn monthly_batch_checkpoint_and_reordered_reference_agree() {
    for case in ["affordable", "unaffordable", "unhelpful"] {
        let mut cpu = sim(case, Backend::CubeCpu);
        acquire(&mut cpu);
        let mut resumed = cpu.clone();
        resumed.backend = Backend::Reference;
        resumed.world.agents.reverse();
        resumed.world.definitions.reverse();
        resumed.world.resources.reverse();
        resumed.world.participants.reverse();
        let mut monthly = cpu.clone();
        cpu.run_months(12).unwrap();
        resumed.run_months(12).unwrap();
        for _ in 0..12 {
            monthly.run_months(1).unwrap();
        }
        assert_eq!(cpu.state, resumed.state);
        assert_eq!(cpu.ledger, resumed.ledger);
        assert_eq!(cpu.state, monthly.state);
        assert_eq!(cpu.ledger, monthly.ledger);
    }
}
