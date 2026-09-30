use economics_compute_smoke::{
    accounting::Account as A,
    compute::Backend,
    financial_reporting::{Audit, Opening},
    forward::{
        Event,
        direct::{Rejection, Terms},
    },
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME, support},
    model::*,
    negotiation::QuotePolicy,
    opportunities::{Action, HOUSEHOLD_TYPE},
    scenario::{GRAIN, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const BUYER: AgentId = 89;
const SELLER: AgentId = 92;
const CONTRACT: u32 = 60000;
fn fixture(fund: bool) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    s.balances.clear();
    s.balances.insert((BUYER, TOKEN), 4);
    s.balances.insert((SELLER, GRAIN), 12);
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = 0;
        w.storage.capacities.insert(p.agent, 100);
    }
    w.households[0].governance.charter.fund_forward_deliveries = fund;
    w.prepaid_deliveries.push(Terms {
        id: CONTRACT,
        seller: HOME,
        buyer: BUYER,
        month: 1,
        due: 3,
        goods: Amount::new(GRAIN, 4),
        prepayment: Amount::new(TOKEN, 4),
    });
    let c = w.town_market.as_mut().unwrap();
    c.traders
        .retain(|e| [HOME, SELLER].contains(&e.trader.agent));
    for e in &mut c.traders {
        e.trader.limit = 2;
        e.trader.opening_quote = 2;
        e.trader.policy = QuotePolicy::Fixed;
    }
    let m = &mut w
        .marketplaces
        .iter_mut()
        .find(|m| m.agent == c.venue)
        .unwrap()
        .markets[0];
    m.goods = Amount::new(GRAIN, 4);
    m.payment = TOKEN;
    m.price_tick = 1;
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| *r != TOKEN && **q > 0)
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            exchange_values: [(GRAIN, 1)].into(),
            dues: Some(economics_compute_smoke::dues_accounting::Valuation(
                w.agreements.iter().map(|a| (a.id, 1)).collect(),
            )),
            ..Default::default()
        },
    )
    .unwrap()
}
fn balance(a: &Audit, agent: AgentId, account: A) -> i128 {
    a.book()
        .balances()
        .get(&(agent, account))
        .copied()
        .unwrap_or(0)
}
fn through(sim: &mut Simulation, a: &mut Audit, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
fn replay(w: &World, s: &State, sim: &Simulation) {
    let mut state = s.clone();
    for b in &sim.ledger {
        commit(w, &mut state, b, sim.backend, DEFAULT_EFFECT_LIMIT).unwrap();
    }
    assert_eq!(state, sim.state);
}

#[test]
fn prepaid_household_delivery_uses_market_funding_with_dated_arrears_and_separate_books() {
    for enabled in [false, true] {
        let (w, s) = fixture(enabled);
        let run = |mut w: World, backend| {
            if matches!(backend, Backend::CubeCpu) {
                w.households[0].adults.reverse();
                w.town_market.as_mut().unwrap().traders.reverse();
            }
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut sim, &mut a, 2);
            assert_eq!(sim.state.balance(HOME, TOKEN), 4);
            assert_eq!(sim.state.balance(HOME, GRAIN), 0);
            assert_eq!(balance(&a, HOME, A::DeferredRevenue(CONTRACT)), -4);
            assert_eq!(balance(&a, BUYER, A::ForwardPrepayment(CONTRACT)), 4);
            through(&mut sim, &mut a, 3);
            assert_eq!(sim.state.balance(HOME, GRAIN), if enabled { 4 } else { 0 });
            assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, 0);
            // Acquire delivery ran before the purchase: stock can be delivered next month.
            let (mut resumed, mut ra) = (sim.clone(), a.clone());
            through(&mut sim, &mut a, 5);
            through(&mut resumed, &mut ra, 5);
            assert_eq!(
                (sim.state.clone(), sim.ledger.clone(), a.clone()),
                (resumed.state, resumed.ledger, ra)
            );
            let delivered = if enabled { 4 } else { 0 };
            assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, delivered);
            assert_eq!(sim.state.balance(BUYER, GRAIN), delivered);
            assert_eq!(sim.state.balance(HOME, TOKEN), if enabled { 2 } else { 4 });
            assert_eq!(sim.state.balance(SELLER, GRAIN), 12 - delivered);
            assert_eq!(
                balance(&a, HOME, A::DeferredRevenue(CONTRACT)),
                i128::from(delivered - 4)
            );
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(w.clone(), Backend::Reference), run(w, Backend::CubeCpu));
    }
}

#[test]
fn current_delivery_can_use_consented_member_surplus_without_duplicate_donations() {
    let (mut w, mut s) = fixture(false);
    w.town_market = None;
    s.town_market = Default::default();
    w.households[0].governance.charter.accept_payment_support = true;
    s.balances.insert((PERSON, GRAIN), 10);
    support::authorize(
        &mut w,
        &s,
        HOME,
        PERSON,
        support::Mandate {
            member: PERSON,
            resource: GRAIN,
            from: 1,
            through: 6,
            revoked_from: None,
            reserve_months: 1,
            private_reserve: 2,
            household_target: 20,
            monthly_limit: 20,
        },
    )
    .unwrap();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut sim, &mut a, 2);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 10);
        through(&mut sim, &mut a, 3);
        assert_eq!(sim.state.balance(HOME, GRAIN), 4);
        assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, 0);
        through(&mut sim, &mut a, 5);
        assert_eq!(sim.state.balance(PERSON, GRAIN), 6);
        assert_eq!(sim.state.balance(BUYER, GRAIN), 4);
        assert_eq!(balance(&a, PERSON, A::TransferExpense), 4);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn admission_checks_law_cash_storage_and_existing_delivery_before_creating_a_claim() {
    for control in 0..5 {
        let (mut w, mut s) = fixture(false);
        w.town_market = None;
        s.town_market = Default::default();
        let reason = match control {
            0 => {
                s.balances.insert((BUYER, TOKEN), 3);
                Rejection::FundingShortfall
            }
            1 => {
                w.storage.capacities.insert(BUYER, 3);
                Rejection::StorageShortfall
            }
            2 => {
                w.transaction_policy
                    .as_mut()
                    .unwrap()
                    .permissions
                    .remove(&(HOUSEHOLD_TYPE, Action::StockTrade));
                Rejection::Ineligible
            }
            3 => {
                w.transaction_policy.as_mut().unwrap().agreement_forms = Some(Default::default());
                Rejection::Ineligible
            }
            _ => {
                let mut second = w.prepaid_deliveries[0].clone();
                second.id += 1;
                w.prepaid_deliveries.push(second);
                Rejection::ExistingDelivery
            }
        };
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut sim, &mut a, 1);
            assert_eq!(sim.state.exchange.forwards.len(), usize::from(control == 4));
            assert!(sim.ledger.iter().flat_map(|b| &b.transactions).any(
                |t| matches!(&t.forward,Some(Event::AdmissionRejected{reason:r,..}) if *r==reason)
            ));
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn land_and_forward_orders_fund_one_shared_stock_gap_and_keep_collection_boundaries() {
    use economics_compute_smoke::commitments::Agreement;
    let (mut w, s) = fixture(true);
    w.prepaid_deliveries[0].due = 13;
    let c = w.town_market.as_mut().unwrap();
    for t in &mut c.traders {
        t.trader.limit = 4;
        t.trader.opening_quote = 4;
    }
    w.marketplaces
        .iter_mut()
        .find(|m| m.agent == c.venue)
        .unwrap()
        .markets[0]
        .goods
        .quantity = 8;
    w.households[0].governance.charter.fund_land_dues = true;
    w.assets.push(Asset {
        id: 60001,
        owner: BUYER,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: 60001,
        holder: HOME,
        asset: 60001,
        from: 1,
        through: 25,
        output_owner: HOME,
    });
    w.agreements.push(Agreement {
        id: 60001,
        right: 60001,
        creditor: BUYER,
        debtor: HOME,
        activated: 1,
        payment: Amount::new(GRAIN, 4),
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut sim, &mut a, 13);
        assert_eq!(sim.state.balance(HOME, GRAIN), 4);
        assert_eq!(sim.state.balance(HOME, TOKEN), 0);
        assert_eq!(sim.state.obligations[&(60001, 13)].paid, 4);
        assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, 0);
        through(&mut sim, &mut a, 14);
        assert_eq!(sim.state.balance(BUYER, GRAIN), 8);
        assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, 4);
        assert_eq!(balance(&a, HOME, A::DeferredRevenue(CONTRACT)), 0);
        assert_eq!(balance(&a, HOME, A::DuesPayable(60001, 13)), 0);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn storage_blocked_partial_delivery_retains_stock_arrears_and_funding_does_not_repeat() {
    let (w, s) = fixture(true);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut sim, &mut a, 3);
        // The receiver loses storage after admission. New space is not promised by the contract.
        sim.world.storage.capacities.insert(BUYER, 2);
        let checkpoint = sim.clone();
        through(&mut sim, &mut a, 5);
        assert_eq!(sim.state.balance(HOME, GRAIN), 2);
        assert_eq!(sim.state.balance(HOME, TOKEN), 2);
        assert_eq!(sim.state.balance(BUYER, GRAIN), 2);
        assert_eq!(
            sim.state.exchange.forwards[&CONTRACT].claim().outstanding(),
            2
        );
        assert_eq!(balance(&a, HOME, A::DeferredRevenue(CONTRACT)), -2);
        assert_eq!(balance(&a, BUYER, A::ForwardPrepayment(CONTRACT)), 2);
        let mut replayed = checkpoint.state;
        for b in sim.ledger.iter().skip(checkpoint.ledger.len()) {
            commit(&sim.world, &mut replayed, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        }
        assert_eq!(replayed, sim.state);
        sim.world.storage.capacities.insert(BUYER, 4);
        through(&mut sim, &mut a, 6);
        assert_eq!(sim.state.balance(HOME, TOKEN), 2);
        assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, 4);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn credit_and_prepayment_reserve_one_opening_budget_and_receipts_cannot_be_forged() {
    use economics_compute_smoke::{
        credit::{Advance, LoanOffer},
        opportunities::PERSON_TYPE,
    };
    let (mut w, s) = fixture(false);
    w.transaction_policy.as_mut().unwrap().permissions.extend([
        (HOUSEHOLD_TYPE, Action::Borrow),
        (PERSON_TYPE, Action::Lend),
    ]);
    w.lending.push(Advance {
        id: 1,
        debtor: HOME,
        principal: 3,
        month: 1,
        priority: 0,
        collateral: None,
        terms: LoanOffer {
            creditor: BUYER,
            denomination: TOKEN,
            max_principal: 3,
            monthly_rate_bps: 0,
            term_months: 6,
            grace_months: 6,
        },
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        while sim.state.phase != Phase::Acquire {
            a.step(&mut sim).unwrap();
        }
        let before = sim.state.clone();
        a.step(&mut sim).unwrap();
        assert!(sim.state.exchange.forwards.is_empty());
        assert_eq!(sim.state.balance(BUYER, TOKEN), 1);
        assert_eq!(sim.state.balance(HOME, TOKEN), 3);
        let mut forged = sim.ledger.last().unwrap().clone();
        let t = forged
            .transactions
            .iter_mut()
            .find(|t| matches!(t.forward, Some(Event::AdmissionRejected { .. })))
            .unwrap();
        t.forward = Some(Event::Accepted(Box::new(
            w.prepaid_deliveries[0].contract(),
        )));
        let mut rejected = before.clone();
        assert!(commit(&w, &mut rejected, &forged, backend, DEFAULT_EFFECT_LIMIT).is_err());
        assert_eq!(rejected, before);
        through(&mut sim, &mut a, 1);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn incoming_prepayment_cannot_fund_another_acceptance_at_the_same_boundary() {
    let (mut w, mut s) = fixture(false);
    w.town_market = None;
    s.town_market = Default::default();
    w.prepaid_deliveries.push(Terms {
        id: CONTRACT + 1,
        seller: SELLER,
        buyer: HOME,
        month: 1,
        due: 3,
        goods: Amount::new(GRAIN, 4),
        prepayment: Amount::new(TOKEN, 4),
    });
    let run = |mut w: World, backend| {
        if matches!(backend, Backend::CubeCpu) {
            w.prepaid_deliveries.reverse();
        }
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut sim, &mut a, 1);
        assert_eq!(sim.state.exchange.forwards.len(), 1);
        assert!(
            sim.ledger
                .iter()
                .flat_map(|b| &b.transactions)
                .any(|t| matches!(
                    t.forward,
                    Some(Event::AdmissionRejected {
                        reason: Rejection::FundingShortfall,
                        ..
                    })
                ))
        );
        assert_eq!(sim.state.balance(HOME, TOKEN), 4);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(w.clone(), Backend::Reference), run(w, Backend::CubeCpu));
}

#[test]
fn household_buyers_share_real_storage_and_existing_performance_survives_law_changes() {
    for room in [2, 4] {
        let (mut w, mut s) = fixture(false);
        w.town_market = None;
        s.town_market = Default::default();
        s.balances.insert((HOME, TOKEN), 4);
        s.balances.insert((BUYER, GRAIN), 4);
        w.storage.capacities.remove(&HOME);
        for member in &w.households[0].adults {
            w.storage.capacities.insert(*member, room);
        }
        w.prepaid_deliveries[0].seller = BUYER;
        w.prepaid_deliveries[0].buyer = HOME;
        let mut later = w.prepaid_deliveries[0].clone();
        later.id += 1;
        later.month = 4;
        later.due = 5;
        w.prepaid_deliveries.push(later);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut sim, &mut a, 2);
            assert_eq!(sim.state.exchange.forwards.len(), usize::from(room == 4));
            let p = sim.world.transaction_policy.as_mut().unwrap();
            p.agreement_forms = Some(Default::default());
            p.permissions.retain(|(_, a)| *a != Action::StockTrade);
            let checkpoint = sim.clone();
            through(&mut sim, &mut a, 4);
            assert_eq!(
                sim.state.balance(HOME, GRAIN),
                if room == 4 { 4 } else { 0 }
            );
            assert!(!sim.state.exchange.forwards.contains_key(&(CONTRACT + 1)));
            if room == 4 {
                assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, 4);
            }
            let mut replayed = checkpoint.state;
            for b in sim.ledger.iter().skip(checkpoint.ledger.len()) {
                commit(&sim.world, &mut replayed, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(sim.state, replayed);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn malformed_terms_and_unsupported_tool_driver_fail_before_execution() {
    for control in 0..6 {
        let (mut w, s) = fixture(false);
        match control {
            0 => w.prepaid_deliveries[0].prepayment.quantity = 0,
            1 => w.prepaid_deliveries[0].due = 1,
            2 => w.prepaid_deliveries[0].buyer = HOME,
            3 => w.prepaid_deliveries.push(w.prepaid_deliveries[0].clone()),
            4 => w.prepaid_deliveries[0].goods.resource = TOKEN,
            _ => w.market = Some(Default::default()),
        }
        assert!(Simulation::new(w, s, Backend::Reference).is_err());
    }
    let (w, s) = fixture(false);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    sim.state
        .exchange
        .forwards
        .get_mut(&CONTRACT)
        .unwrap()
        .advance
        .quantity += 1;
    assert!(Simulation::new(sim.world, sim.state, Backend::Reference).is_err());
}

#[test]
fn direct_delivery_recovery_extends_then_writes_off_only_the_unfulfilled_claim() {
    use economics_compute_smoke::{
        delivery_relief::{Action as Relief, Terms as ReliefTerms},
        households::dissolution,
        recovery::{ProceedingTerms, Stage},
        scenario::STATE_AGENT,
    };
    const ESTATE: AgentId = 60002;
    let (mut w, mut s) = fixture(false);
    w.town_market = None;
    s.town_market = Default::default();
    w.households[0].governance.constitution.allow_dissolution = true;
    w.households[0].adults = vec![PERSON];
    w.households[0]
        .admission
        .as_mut()
        .unwrap()
        .founders
        .retain(|(id, _)| *id == PERSON);
    s.balances.insert((HOME, GRAIN), 2);
    w.agents.push(Agent {
        id: ESTATE,
        name: "estate custody".into(),
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: HOME,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 5,
        earliest_close: 5,
        assets: vec![],
        discharge_deficiency: true,
    });
    w.recovery.delivery_relief = vec![
        ReliefTerms {
            id: 1,
            proceeding: 1,
            contract: CONTRACT,
            debtor: HOME,
            creditor: BUYER,
            month: 5,
            expected_due: 3,
            expected_remaining: 2,
            action: Relief::Extend { due: 7 },
        },
        ReliefTerms {
            id: 2,
            proceeding: 1,
            contract: CONTRACT,
            debtor: HOME,
            creditor: BUYER,
            month: 8,
            expected_due: 7,
            expected_remaining: 2,
            action: Relief::WriteOff { quantity: 2 },
        },
    ];
    let mut later = w.prepaid_deliveries[0].clone();
    later.id += 1;
    later.month = 6;
    later.due = 9;
    w.prepaid_deliveries.push(later);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut sim, &mut a, 3);
        assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, 2);
        dissolution::request(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
        through(&mut sim, &mut a, 5);
        assert_eq!(sim.state.exchange.forwards[&CONTRACT].effective_due(), 7);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Active
        );
        assert_eq!(balance(&a, HOME, A::DeferredRevenue(CONTRACT)), -2);
        assert!(dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).is_err());
        let (mut resumed, mut ra) = (sim.clone(), a.clone());
        through(&mut sim, &mut a, 9);
        through(&mut resumed, &mut ra, 9);
        assert_eq!(
            (&sim.state, &sim.ledger, &a),
            (&resumed.state, &resumed.ledger, &ra)
        );
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Closed
        );
        assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, 2);
        assert_eq!(
            sim.state.exchange.forwards[&CONTRACT].claim().outstanding(),
            0
        );
        assert_eq!(sim.state.balance(BUYER, GRAIN), 2);
        assert_eq!(balance(&a, HOME, A::DeferredRevenue(CONTRACT)), 0);
        assert_eq!(balance(&a, BUYER, A::ForwardPrepayment(CONTRACT)), 0);
        assert_eq!(balance(&a, BUYER, A::CreditLoss), 2);
        assert_eq!(balance(&a, HOME, A::DebtRelief), -2);
        assert!(!sim.state.exchange.forwards.contains_key(&(CONTRACT + 1)));
        dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
        through(&mut sim, &mut a, 10);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    for seller in [false, true] {
        let mut bad = w.clone();
        if seller {
            bad.prepaid_deliveries[0].seller = ESTATE;
        } else {
            bad.prepaid_deliveries[0].buyer = ESTATE;
        }
        assert!(Simulation::new(bad, s.clone(), Backend::Reference).is_err());
    }
}

#[test]
fn either_party_in_direct_forward_recovery_cannot_accept_a_new_prepayment() {
    use economics_compute_smoke::{recovery::ProceedingTerms, scenario::STATE_AGENT};
    let (mut w, mut s) = fixture(false);
    w.town_market = None;
    s.town_market = Default::default();
    s.balances.insert((SELLER, GRAIN), 0);
    w.prepaid_deliveries[0].seller = SELLER;
    w.agents.push(Agent {
        id: 60002,
        name: "custody".into(),
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: SELLER,
        authority: STATE_AGENT,
        estate: 60002,
        denomination: TOKEN,
        opening_month: 4,
        earliest_close: 4,
        assets: vec![],
        discharge_deficiency: true,
    });
    for (id, seller, buyer) in [(CONTRACT + 1, SELLER, BUYER), (CONTRACT + 2, BUYER, SELLER)] {
        let mut t = w.prepaid_deliveries[0].clone();
        t.id = id;
        t.seller = seller;
        t.buyer = buyer;
        t.month = 5;
        t.due = 6;
        w.prepaid_deliveries.push(t);
    }
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut sim, &mut a, 5);
        assert_eq!(sim.state.exchange.forwards.len(), 1);
        let rejected: Vec<_> = sim
            .ledger
            .iter()
            .flat_map(|b| &b.transactions)
            .filter_map(|t| match &t.forward {
                Some(Event::AdmissionRejected {
                    contract,
                    reason: Rejection::Ineligible,
                }) => Some(*contract),
                _ => None,
            })
            .collect();
        assert_eq!(rejected, vec![CONTRACT + 1, CONTRACT + 2]);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn town_admission_is_rechecked_after_recovery_opens_and_custody_cannot_trade() {
    use economics_compute_smoke::{
        recovery::ProceedingTerms, scenario::STATE_AGENT, town_market::OrderReason,
    };
    let (mut w, mut s) = fixture(false);
    s.balances.insert((SELLER, GRAIN), 0);
    w.prepaid_deliveries[0].seller = SELLER;
    w.agents.push(Agent {
        id: 60002,
        name: "custody".into(),
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: SELLER,
        authority: STATE_AGENT,
        estate: 60002,
        denomination: TOKEN,
        opening_month: 4,
        earliest_close: 4,
        assets: vec![],
        discharge_deficiency: true,
    });
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut sim, &mut a, 3);
        // Enough free stock to sell, even after the overdue four-unit delivery.
        // An active case must still prevent ordinary market disposal at Acquire.
        sim.state.balances.insert((SELLER, GRAIN), 12);
        a = audit(&sim.world, &sim.state);
        while sim.state.phase != Phase::Acquire {
            a.step(&mut sim).unwrap();
        }
        assert!(
            sim.state
                .town_market
                .admission
                .as_ref()
                .unwrap()
                .eligible
                .contains(&SELLER)
        );
        let checkpoint = (sim.clone(), a.clone());
        through(&mut sim, &mut a, 4);
        assert_eq!(sim.state.exchange.forwards[&CONTRACT].delivered, 4);
        let book = sim.state.town_market.history.last().unwrap();
        assert!(
            book.order_receipts
                .iter()
                .any(|r| r.agent == SELLER && r.reason == OrderReason::Inactive)
        );
        assert_eq!(sim.state.balance(SELLER, GRAIN), 8);
        let (mut resumed, mut ra) = checkpoint;
        through(&mut resumed, &mut ra, 4);
        assert_eq!(
            (&sim.state, &sim.ledger, &a),
            (&resumed.state, &resumed.ledger, &ra)
        );
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    let mut bad = w;
    bad.town_market.as_mut().unwrap().traders[0].trader.agent = 60002;
    assert!(Simulation::new(bad, s, Backend::Reference).is_err());
}
