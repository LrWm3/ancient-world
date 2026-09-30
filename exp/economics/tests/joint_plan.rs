use economics_compute_smoke::{
    borrowing,
    compute::Backend,
    credit, joint_plan,
    model::*,
    scenario::{GRAIN, GROW, NUTRITION, PERSON},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    stock_sale,
};
fn sim() -> Simulation {
    let (mut w, s) = stock_sale::joint_scenario("funded").unwrap();
    w.credit.as_mut().unwrap().purchase_policy = borrowing::Policy::Scripted;
    Simulation::new(w, s, Backend::Reference).unwrap()
}
fn acquire(s: &mut Simulation) {
    s.step().unwrap();
    s.step().unwrap();
    assert_eq!(s.state.phase, Phase::Acquire);
}
fn batch(s: &Simulation) -> Batch {
    let mut b = Batch::empty(&s.state);
    b.credit = credit::evaluate(&s.world, &s.state).unwrap();
    let c = b.credit.as_ref().unwrap();
    b.transactions = c.transactions.clone();
    b.production_plan = c.production_plan.clone();
    b
}
#[test]
fn joint_candidates_include_dated_production_and_future_sales_and_commit_selected_work() {
    let mut s = sim();
    acquire(&mut s);
    let b = batch(&s);
    let c = b.credit.as_ref().unwrap();
    let d = c.stock_sale.as_ref().unwrap().joint.as_ref().unwrap();
    assert!(d.feasible);
    assert!(
        d.alternatives
            .iter()
            .any(|a| matches!(a.work, joint_plan::Work::Produce(_))
                && a.starts.len() > 1
                && a.sales.len() > 1)
    );
    assert_eq!(d.alternatives[d.selected].deficits[&NUTRITION], 0);
    let plan = b.production_plan.clone().unwrap();
    settlement::commit(
        &s.world,
        &mut s.state,
        &b,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    assert_eq!(s.state.pending_production, Some(plan.clone()));
    s.step().unwrap();
    assert_eq!(s.ledger.last().unwrap(), plan.as_ref());
    assert!(
        s.state
            .processes
            .values()
            .any(|p| p.definition == GROW && p.status == Status::Active)
    );
}
#[test]
fn no_land_joint_fallback_does_not_sell_food_to_create_later_shortfall() {
    let mut s = sim();
    s.world.credit.as_mut().unwrap().purchase_policy = borrowing::Policy::Decline;
    s.run_months(18).unwrap();
    assert_eq!(s.state.credit.stock_spent, 0);
    assert!(
        s.ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .filter_map(|c| c.stock_sale.as_ref())
            .all(|r| r.goods == 0)
    );
    let deficit: i32 = s
        .reports
        .iter()
        .filter(|r| r.agent == PERSON)
        .map(|r| r.deficit(NUTRITION))
        .sum();
    assert_eq!(deficit, 7);
    assert!(
        s.ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .filter_map(|c| c.stock_sale.as_ref())
            .filter_map(|r| r.joint.as_ref())
            .all(|d| !d.feasible)
    );
}
#[test]
fn dated_plan_and_joint_receipt_cannot_be_forged_or_replayed() {
    let mut s = sim();
    acquire(&mut s);
    let b = batch(&s);
    let before = s.state.clone();
    for n in 0..3 {
        let mut bad = b.clone();
        match n {
            0 => bad.production_plan = None,
            1 => {
                bad.credit
                    .as_mut()
                    .unwrap()
                    .stock_sale
                    .as_mut()
                    .unwrap()
                    .joint
                    .as_mut()
                    .unwrap()
                    .selected = usize::MAX
            }
            _ => {
                let plan = bad.production_plan.as_mut().unwrap();
                plan.month += 1;
            }
        }
        assert!(
            settlement::commit(
                &s.world,
                &mut s.state,
                &bad,
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
        &b,
        Backend::Reference,
        DEFAULT_EFFECT_LIMIT,
    )
    .unwrap();
    let after = s.state.clone();
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
    assert_eq!(s.state, after);
    let mut altered = *s.state.pending_production.clone().unwrap();
    altered.receipts.clear();
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &altered,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, after);
}
#[test]
fn limits_and_hidden_shocks_preserve_bounded_observation() {
    let mut s = sim();
    acquire(&mut s);
    let expected = batch(&s);
    s.world.capacity_overrides.insert((2, PERSON), 0);
    assert_eq!(batch(&s), expected);
    s.world
        .credit
        .as_mut()
        .unwrap()
        .stock_sales
        .as_mut()
        .unwrap()
        .joint
        .as_mut()
        .unwrap()
        .horizon_months = 6;
    assert!(Simulation::new(s.world, s.state, Backend::Reference).is_err());
}
#[test]
fn cpu_checkpoint_monthly_and_catalog_order_preserve_joint_execution() {
    let mut cpu = sim();
    cpu.backend = Backend::CubeCpu;
    cpu.run_months(7).unwrap();
    let mut reference = cpu.clone();
    reference.backend = Backend::Reference;
    reference.world.definitions.reverse();
    reference.world.resources.reverse();
    let mut monthly = cpu.clone();
    cpu.run_months(11).unwrap();
    reference.run_months(11).unwrap();
    for _ in 0..11 {
        monthly.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.state, monthly.state);
    assert_eq!(cpu.ledger, monthly.ledger);
    assert!(cpu.reports.iter().all(|r| r.deficit(NUTRITION) == 0));
    assert_eq!(cpu.state.credit.loans[&1].status, credit::Status::Repaid);
    assert!(cpu.state.balance(PERSON, GRAIN) > 0);
}

#[test]
fn sixty_month_joint_execution_preserves_food_seed_and_finite_state_funding() {
    use economics_compute_smoke::scenario::{SEED, TOKEN};
    let mut s = sim();
    s.backend = Backend::CubeCpu;
    for _ in 0..60 {
        s.run_months(1).unwrap();
        let active = s
            .state
            .processes
            .values()
            .filter(|p| p.definition == GROW && p.status == Status::Active)
            .count() as i32;
        assert_eq!(s.state.balance(PERSON, SEED) + active, 1);
        assert!(s.state.credit.stock_spent <= 12000);
    }
    let harvests: Vec<_> = s
        .state
        .processes
        .values()
        .filter(|p| p.definition == GROW && p.status == Status::Completed)
        .map(|p| p.reserved_through)
        .collect();
    println!(
        "joint 60 months: harvests={harvests:?}, coins={}, grain={}, state_spent={}",
        s.state.balance(PERSON, TOKEN),
        s.state.balance(PERSON, GRAIN),
        s.state.credit.stock_spent
    );
    assert!(harvests.len() >= 5);
    assert!(s.reports.iter().all(|r| r.deficit(NUTRITION) == 0));
    assert_eq!(s.state.credit.loans[&1].status, credit::Status::Repaid);
    assert!(
        s.state
            .processes
            .values()
            .all(|p| p.status != Status::Aborted)
    );
}

#[test]
fn negotiated_seed_purchase_is_visible_to_joint_work_without_reusing_mortgage_cash() {
    use economics_compute_smoke::{
        negotiation,
        opportunities::{Action, PERSON_TYPE, STATE_TYPE},
        scenario::{SEED, STATE_AGENT, TOKEN},
    };
    for (funded, zip) in [(false, false), (true, false), (false, true), (true, true)] {
        let (mut w, mut s) = stock_sale::joint_scenario("funded").unwrap();
        let (venue, _) = negotiation::scenario();
        w.agents.extend(
            venue
                .agents
                .into_iter()
                .filter(|a| a.id != PERSON && a.id != STATE_AGENT),
        );
        w.marketplaces = venue.marketplaces;
        w.transaction_policy = venue.transaction_policy;
        w.negotiation = venue.negotiation;
        w.negotiation.as_mut().unwrap().goods = Amount::new(SEED, 1);
        if zip {
            let session = w.negotiation.as_mut().unwrap();
            session.buyer.policy = negotiation::QuotePolicy::Zip(Default::default());
            session.seller.policy = negotiation::QuotePolicy::Zip(Default::default());
            session.buyer.opening_quote = 50;
            session.seller.opening_quote = 30;
        }
        w.marketplaces[0].markets[0].goods = Amount::new(SEED, 1);
        let policy = w.transaction_policy.as_mut().unwrap();
        policy.permissions.extend([
            (PERSON_TYPE, Action::FinancedPurchase),
            (STATE_TYPE, Action::StockTrade),
        ]);
        policy.permissions.extend(
            w.definitions
                .iter()
                .map(|d| (PERSON_TYPE, Action::Process(d.id))),
        );
        s.balances.insert((PERSON, SEED), 0);
        s.balances.insert((89, SEED), 1);
        let c = w.credit.as_mut().unwrap();
        c.purchase_policy = borrowing::Policy::Scripted;
        if !funded {
            c.endowments
                .iter_mut()
                .find(|e| e.agent == PERSON)
                .unwrap()
                .amount
                .quantity = c.application.downpayment;
        }
        let joint = c.stock_sales.as_mut().unwrap().joint.as_mut().unwrap();
        joint.horizon_months = 12;
        joint.future_reserves = vec![6];
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.phase != Phase::Acquire {
                sim.step().unwrap();
            }
            let before = sim.state.clone();
            sim.step().unwrap();
            let accepted = sim.ledger.last().unwrap();
            assert_eq!(
                accepted.negotiation.as_ref().unwrap().outcome,
                if funded {
                    negotiation::Outcome::Traded { price: 40 }
                } else {
                    negotiation::Outcome::InsufficientPayment
                }
            );
            assert_eq!(sim.state.balance(PERSON, SEED), i32::from(funded));
            assert_eq!(sim.state.balance(89, TOKEN), if funded { 40 } else { 0 });
            assert_eq!(sim.state.credit.loans.len(), 1);
            let memory = &sim.state.marketplaces[&negotiation::MARKETPLACE];
            assert_eq!(memory.history.len(), 1);
            if zip {
                assert_eq!(memory.pricing.len(), 2);
                assert!(
                    memory
                        .pricing
                        .values()
                        .all(|p| p.learning.as_ref().unwrap().updates == u64::from(funded))
                );
            }
            let plan = accepted.production_plan.as_ref().unwrap().clone();
            assert_eq!(sim.state.pending_production, Some(plan.clone()));
            let mut tampered = accepted.clone();
            tampered.negotiation = None;
            let mut unchanged = before.clone();
            assert!(
                settlement::commit(
                    &sim.world,
                    &mut unchanged,
                    &tampered,
                    backend,
                    sim.effect_limit
                )
                .is_err()
            );
            assert_eq!(unchanged, before);
            let prefix = sim.ledger.len();
            let mut resumed =
                Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
            sim.step().unwrap();
            assert_eq!(sim.ledger.last().unwrap(), plan.as_ref());
            assert_eq!(
                sim.state
                    .processes
                    .values()
                    .any(|p| p.definition == GROW && p.status == Status::Active),
                funded
            );
            while sim.state.month < 2 {
                sim.step().unwrap();
            }
            while resumed.state.month < 2 {
                resumed.step().unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger[prefix..]),
                (&resumed.state, &resumed.ledger[..])
            );
            (sim.state, sim.ledger)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
