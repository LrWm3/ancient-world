use economics_compute_smoke::{
    acquisition,
    compute::Backend,
    credit, membership,
    model::*,
    negotiation::{self, Outcome, QuotePolicy},
    opportunities::{Action, PERSON_TYPE},
    scenario::{GRAIN, PERSON, STATE_AGENT, TOKEN},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    zip,
};

fn fixture(extra_cash: i32) -> (World, State) {
    let (mut w, mut s) = credit::scenario("repaid").unwrap();
    let (n, opening) = negotiation::scenario();
    w.agents.extend(
        n.agents
            .into_iter()
            .filter(|a| a.id != PERSON && a.id != STATE_AGENT),
    );
    w.resources
        .extend(n.resources.into_iter().filter(|r| r.id != TOKEN));
    w.marketplaces = n.marketplaces;
    w.negotiation = n.negotiation;
    w.transaction_policy = n.transaction_policy;
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::FinancedPurchase));
    w.storage = n.storage;
    s.balances = opening.balances;
    s.balances.insert((PERSON, TOKEN), extra_cash);
    (w, s)
}
fn acquire(w: World, s: State, backend: Backend) -> Simulation {
    let mut sim = Simulation::new(w, s, backend).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim
}

#[test]
fn downpayment_and_exchange_share_opening_cash_on_cpu() {
    for (extra, outcome) in [
        (0, Outcome::InsufficientPayment),
        (50, Outcome::Traded { price: 40 }),
    ] {
        let (w, s) = fixture(extra);
        let mut cpu = acquire(w, s, Backend::CubeCpu);
        let mut reference = cpu.clone();
        reference.backend = Backend::Reference;
        // Independently evaluated quotes would spend money already needed for land.
        assert_eq!(
            negotiation::evaluate(&cpu.world, &cpu.state)
                .unwrap()
                .unwrap()
                .outcome,
            Outcome::Traded { price: 40 }
        );
        cpu.step().unwrap();
        reference.step().unwrap();
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        let batch = cpu.ledger.last().unwrap();
        assert_eq!(batch.negotiation.as_ref().unwrap().outcome, outcome);
        assert_eq!(cpu.state.credit.loans.len(), 1);
        assert_eq!(
            cpu.state.balance(PERSON, TOKEN),
            if extra == 0 { 0 } else { 10 }
        );
        assert_eq!(
            cpu.state.balance(PERSON, GRAIN),
            if extra == 0 { 0 } else { 2 }
        );
        for resource in [TOKEN, GRAIN] {
            assert_eq!(
                batch
                    .transactions
                    .iter()
                    .flat_map(|t| &t.effects)
                    .filter(|e| e.account.1 == resource)
                    .map(|e| i64::from(e.delta))
                    .sum::<i64>(),
                0
            );
        }
    }
}

#[test]
fn incoming_sale_proceeds_cannot_fund_another_purchase_in_the_batch() {
    let (mut w, mut s) = fixture(0);
    w.credit.as_mut().unwrap().offers[0].sale.seller = 89;
    w.assets[0].owner = 89;
    let n = w.negotiation.as_mut().unwrap();
    n.buyer.agent = 89;
    n.seller.agent = PERSON;
    s.balances.insert((PERSON, GRAIN), 2);
    s.balances.insert((89, GRAIN), 0);
    let mut sim = acquire(w, s, Backend::CubeCpu);
    sim.step().unwrap();
    assert_eq!(
        sim.ledger
            .last()
            .unwrap()
            .negotiation
            .as_ref()
            .unwrap()
            .outcome,
        Outcome::InsufficientPayment
    );
    assert_eq!(sim.state.balance(89, TOKEN), 10_000);
    assert_eq!(sim.state.balance(PERSON, GRAIN), 2);
}

#[test]
fn forged_component_receipts_and_independently_funded_trades_fail_atomically() {
    let (w, s) = fixture(0);
    let sim = acquire(w, s, Backend::Reference);
    let valid = acquisition::evaluate(&sim.world, &sim.state).unwrap();
    for case in 0..4 {
        let mut b = valid.clone();
        match case {
            0 => b.credit = None,
            1 => b.negotiation = None,
            2 => b.transactions.push(b.transactions[0].clone()),
            _ => {
                b.negotiation = negotiation::evaluate(&sim.world, &sim.state).unwrap();
                b.transactions.extend(
                    negotiation::transactions(&sim.world, &sim.state, &b.negotiation).unwrap(),
                );
            }
        }
        let mut state = sim.state.clone();
        assert!(
            settlement::commit(
                &sim.world,
                &mut state,
                &b,
                Backend::CubeCpu,
                DEFAULT_EFFECT_LIMIT
            )
            .is_err()
        );
        assert_eq!(state, sim.state);
    }
    let mut state = sim.state.clone();
    assert!(settlement::commit(&sim.world, &mut state, &valid, Backend::CubeCpu, 1).is_err());
    assert_eq!(state, sim.state);
}

#[test]
fn citizenship_can_authorize_origination_and_revocation_does_not_erase_debt() {
    for citizen in [false, true] {
        let (mut w, mut s) = fixture(50);
        let p = w.transaction_policy.as_mut().unwrap();
        p.permissions
            .remove(&(PERSON_TYPE, Action::FinancedPurchase));
        p.membership_permissions
            .insert((membership::CITIZEN, Action::FinancedPurchase));
        p.membership_offers.push(membership::Offer {
            id: 1,
            organization: STATE_AGENT,
            role: membership::CITIZEN,
            eligible_type: PERSON_TYPE,
        });
        if citizen {
            s.memberships.insert(
                (PERSON, STATE_AGENT, membership::CITIZEN),
                membership::Agreement {
                    member: PERSON,
                    organization: STATE_AGENT,
                    role: membership::CITIZEN,
                    source_offer: 1,
                    accepted_month: 1,
                },
            );
        }
        let mut sim = acquire(w, s, Backend::CubeCpu);
        sim.step().unwrap();
        assert_eq!(sim.state.credit.loans.len(), usize::from(citizen));
        if citizen {
            sim.world
                .transaction_policy
                .as_mut()
                .unwrap()
                .membership_permissions
                .clear();
            sim.run_months(2).unwrap();
            assert!(sim.state.credit.loans[&1].principal < 8_000);
        } else {
            assert!(
                sim.ledger
                    .last()
                    .unwrap()
                    .credit
                    .as_ref()
                    .unwrap()
                    .events
                    .iter()
                    .any(|e| matches!(
                        e,
                        credit::Event::Rejected {
                            reason: credit::Rejection::Ineligible,
                            ..
                        }
                    ))
            );
        }
    }
}

#[test]
fn zip_credit_monthly_batch_resume_and_table_order_agree() {
    let (mut w, s) = fixture(50);
    let n = w.negotiation.as_mut().unwrap();
    n.buyer.policy = QuotePolicy::Zip(zip::Config::default());
    n.seller.policy = QuotePolicy::Zip(zip::Config::default());
    n.buyer.opening_quote = 40;
    n.seller.opening_quote = 40;
    let mut monthly = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut batched = monthly.clone();
    w.agents.reverse();
    w.resources.reverse();
    let mut reversed = Simulation::new(w, s, Backend::Reference).unwrap();
    monthly.step().unwrap();
    let mut resumed = monthly.clone();
    for _ in 0..6 {
        monthly.run_months(1).unwrap();
    }
    for sim in [&mut batched, &mut resumed, &mut reversed] {
        sim.run_months(6).unwrap();
        assert_eq!(sim.state, monthly.state);
        assert_eq!(sim.ledger, monthly.ledger);
    }
    let memory = &monthly.state.marketplaces[&negotiation::MARKETPLACE];
    assert_eq!(memory.history.len(), 1);
    assert!(memory.pricing.values().all(|p| p.learning.is_some()));
}

#[test]
fn state_bid_and_negotiation_share_goods_and_net_storage() {
    use economics_compute_smoke::{opportunities::STATE_TYPE, stock_sale};
    for selling in [false, true] {
        let (mut w, mut s) = stock_sale::scenario("funded").unwrap();
        w.credit.as_mut().unwrap().purchase_policy =
            economics_compute_smoke::borrowing::Policy::Decline;
        w.credit
            .as_mut()
            .unwrap()
            .stock_sales
            .as_mut()
            .unwrap()
            .reserve_months = 0;
        let (n, _) = negotiation::scenario();
        w.agents.extend(
            n.agents
                .into_iter()
                .filter(|a| a.id != PERSON && a.id != STATE_AGENT),
        );
        w.marketplaces = n.marketplaces;
        w.negotiation = n.negotiation;
        w.transaction_policy = n.transaction_policy;
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((STATE_TYPE, Action::StockTrade));
        s.balances.insert((PERSON, GRAIN), 2);
        s.balances.insert((89, GRAIN), 2);
        s.balances.insert((89, TOKEN), 100);
        w.storage.weights.clear();
        w.storage.weights.insert(GRAIN, 1);
        w.storage.capacities.insert(PERSON, 2);
        if selling {
            let n = w.negotiation.as_mut().unwrap();
            n.buyer.agent = 89;
            n.seller.agent = PERSON;
        }
        let mut sim = acquire(w, s, Backend::CubeCpu);
        sim.step().unwrap();
        let batch = sim.ledger.last().unwrap();
        assert_eq!(
            batch
                .credit
                .as_ref()
                .unwrap()
                .stock_sale
                .as_ref()
                .unwrap()
                .sold_lots,
            2
        );
        assert_eq!(
            batch.negotiation.as_ref().unwrap().outcome,
            if selling {
                Outcome::InsufficientGoods
            } else {
                Outcome::Traded { price: 40 }
            }
        );
        assert_eq!(
            sim.state.balance(PERSON, GRAIN),
            if selling { 0 } else { 2 }
        );
    }
}

#[test]
fn state_stock_bid_obeys_both_parties_trade_permissions() {
    use economics_compute_smoke::{
        opportunities::{Policy, STATE_TYPE},
        stock_sale,
    };
    use std::collections::{BTreeMap, BTreeSet};
    let (mut w, s) = stock_sale::scenario("funded").unwrap();
    w.credit.as_mut().unwrap().purchase_policy =
        economics_compute_smoke::borrowing::Policy::Decline;
    w.credit
        .as_mut()
        .unwrap()
        .stock_sales
        .as_mut()
        .unwrap()
        .reserve_months = 0;
    w.transaction_policy = Some(Policy {
        authority: STATE_AGENT,
        membership_offers: vec![],
        laws: vec![],
        agreement_forms: None,
        agreement_limits: Default::default(),
        membership_permissions: BTreeSet::new(),
        agent_types: BTreeMap::from([(PERSON, PERSON_TYPE), (STATE_AGENT, STATE_TYPE)]),
        permissions: BTreeSet::from([(PERSON_TYPE, Action::StockTrade)]),
    });
    let mut sim = acquire(w, s, Backend::Reference);
    sim.step().unwrap();
    assert_eq!(
        sim.ledger
            .last()
            .unwrap()
            .credit
            .as_ref()
            .unwrap()
            .stock_sale
            .as_ref()
            .unwrap()
            .sold_lots,
        0
    );
}

#[test]
fn common_offer_acceptance_uses_the_same_shared_boundary_as_monthly_execution() {
    use economics_compute_smoke::offers::{self, Id, Request};
    let (w, s) = fixture(50);
    let mut monthly = acquire(w, s, Backend::CubeCpu);
    let mut explicit = monthly.clone();
    offers::accept(
        &mut explicit,
        &[Request::new(Id::FinancedPurchase(1), PERSON)],
    )
    .unwrap();
    monthly.step().unwrap();
    assert_eq!(explicit.state, monthly.state);
    assert_eq!(explicit.ledger, monthly.ledger);
}

#[test]
fn mortgage_forward_and_negotiated_food_share_cash_and_dated_delivery() {
    use economics_compute_smoke::{
        financial_reporting::{Audit, Opening},
        forward::direct,
        offers,
    };
    for extra in [0, 50] {
        let (mut w, s) = fixture(extra);
        let law = w.transaction_policy.as_mut().unwrap();
        law.agent_types.insert(
            STATE_AGENT,
            economics_compute_smoke::opportunities::STATE_TYPE,
        );
        law.permissions.insert((
            economics_compute_smoke::opportunities::STATE_TYPE,
            Action::StockTrade,
        ));
        w.prepaid_deliveries.push(direct::Terms {
            id: 70,
            seller: PERSON,
            buyer: STATE_AGENT,
            month: 1,
            due: 2,
            goods: Amount::new(GRAIN, 1),
            prepayment: Amount::new(TOKEN, 40),
        });
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    assets: w.assets.iter().map(|a| (a.id, 10000)).collect(),
                    inventory: s
                        .balances
                        .iter()
                        .filter(|((_, r), q)| *r != TOKEN && **q > 0)
                        .map(|(a, q)| (*a, i128::from(*q)))
                        .collect(),
                    ..Default::default()
                },
            )
            .unwrap();
            while sim.state.phase != Phase::Acquire {
                audit.step(&mut sim).unwrap();
            }
            let requests = [
                offers::Request::new(offers::Id::FinancedPurchase(1), PERSON),
                offers::Request::new(offers::Id::PrepaidDelivery(70), PERSON),
            ];
            let prepared = offers::prepare(&sim, &requests).unwrap();
            let before = sim.state.clone();
            let mut forged = prepared.clone();
            forged.transactions.extend(
                prepared
                    .transactions
                    .iter()
                    .filter(|t| t.forward.is_some())
                    .cloned(),
            );
            let mut rejected = before.clone();
            assert!(
                settlement::commit(&w, &mut rejected, &forged, backend, DEFAULT_EFFECT_LIMIT)
                    .is_err()
            );
            assert_eq!(rejected, before);
            audit.step(&mut sim).unwrap();
            assert_eq!(sim.ledger.last(), Some(&prepared));
            assert_eq!(
                credit::owner(&w, &sim.state, economics_compute_smoke::scenario::PLOT),
                Some(PERSON)
            );
            assert_eq!(
                sim.state.balance(PERSON, GRAIN),
                if extra == 0 { 0 } else { 2 }
            );
            assert_eq!(
                sim.state.balance(PERSON, TOKEN),
                if extra == 0 { 40 } else { 50 }
            );
            let mut checkpoint = (sim.clone(), audit.clone());
            while sim.state.month <= 6 {
                audit.step(&mut sim).unwrap();
            }
            while checkpoint.0.state.month <= 6 {
                checkpoint.1.step(&mut checkpoint.0).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&checkpoint.0.state, &checkpoint.0.ledger, &checkpoint.1)
            );
            assert_eq!(sim.state.credit.loans[&1].status, credit::Status::Repaid);
            assert_eq!(
                sim.state.exchange.forwards[&70].delivered,
                if extra == 0 { 0 } else { 1 }
            );
            let first = sim.ledger.iter().find(|b| {
                b.transactions.iter().any(|t| {
                    matches!(
                        t.forward,
                        Some(economics_compute_smoke::forward::Event::Delivery {
                            contract: 70,
                            ..
                        })
                    )
                })
            });
            assert_eq!(
                first.map(|b| b.month),
                if extra == 0 { None } else { Some(2) }
            );
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn mortgage_seller_cannot_reuse_downpayment_as_forward_funding_at_acquire() {
    use economics_compute_smoke::forward::{self, direct};
    let (mut w, s) = fixture(0);
    let law = w.transaction_policy.as_mut().unwrap();
    law.agent_types.insert(
        STATE_AGENT,
        economics_compute_smoke::opportunities::STATE_TYPE,
    );
    law.permissions.insert((
        economics_compute_smoke::opportunities::STATE_TYPE,
        Action::StockTrade,
    ));
    w.prepaid_deliveries.push(direct::Terms {
        id: 70,
        seller: PERSON,
        buyer: STATE_AGENT,
        month: 1,
        due: 2,
        goods: Amount::new(GRAIN, 1),
        prepayment: Amount::new(TOKEN, 100001),
    });
    let mut sim = acquire(w, s, Backend::CubeCpu);
    sim.step().unwrap();
    assert!(sim.state.credit.loans.contains_key(&1));
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 102000);
    assert!(sim.state.exchange.forwards.is_empty());
    assert!(
        sim.ledger
            .last()
            .unwrap()
            .transactions
            .iter()
            .any(|t| matches!(
                t.forward,
                Some(forward::Event::AdmissionRejected {
                    reason: direct::Rejection::FundingShortfall,
                    ..
                })
            ))
    );
}
