use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    compute::Backend,
    credit::{Advance, LoanOffer},
    finance::CollectionPolicy,
    financial_reporting::Audit,
    issuance_accounting::Policy,
    model::*,
    recovery::{ProceedingTerms, Stage},
    scenario::{self, PERSON, PLOT, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
use std::collections::BTreeMap;
const OTHER: AgentId = 97;
const BUYER: AgentId = 98;
const ESTATE: AgentId = 99;
fn advance(id: u32, creditor: AgentId) -> Advance {
    Advance {
        id,
        debtor: PERSON,
        terms: LoanOffer {
            creditor,
            denomination: TOKEN,
            max_principal: 10,
            monthly_rate_bps: 0,
            term_months: 1,
            grace_months: 10,
        },
        principal: 10,
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
    for id in [OTHER, BUYER, ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = PERSON;
    w.collection_policy = CollectionPolicy::Proportional;
    w.lending = vec![advance(10, STATE_AGENT), advance(11, OTHER)];
    s.balances.clear();
    for id in [STATE_AGENT, OTHER] {
        s.balances.insert((id, TOKEN), 10);
    }
    s.balances.insert((BUYER, TOKEN), 8);
    (w, s)
}
fn distressed(w: World, s: State, backend: Backend) -> Simulation {
    let mut sim = Simulation::new(w, s, backend).unwrap();
    sim.run_months(1).unwrap();
    // A controlled external loss, identical between policies/backends.
    sim.state.balances.insert((PERSON, TOKEN), 0);
    sim
}
fn land_estate(alternative: bool) -> Simulation {
    use economics_compute_smoke::{
        activities::CoinPayment, commitments::Agreement, currency::Issuance,
    };
    let (mut w, s) = fixture();
    w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = STATE_AGENT;
    w.rights = scenario::baseline().0.rights;
    w.rights[0].through = 60;
    w.agreements.push(Agreement {
        id: 1,
        right: 1,
        creditor: STATE_AGENT,
        debtor: PERSON,
        activated: 1,
        payment: Amount::new(scenario::GRAIN, 2),
    });
    w.issuance.push(Issuance {
        agreement: 1,
        token: TOKEN,
        collected_per_token: 2,
    });
    if alternative {
        w.activities.coin_payments.insert(
            1,
            CoinPayment {
                resource: TOKEN,
                coins_per_unit: 2,
            },
        );
    }
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 14,
        earliest_close: 15,
        assets: vec![],
        discharge_deficiency: true,
    });
    let mut sim = distressed(w, s, Backend::Reference);
    sim.run_months(12).unwrap();
    assert_eq!(sim.state.month, 14);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 0);
    sim
}

fn audit(sim: &Simulation) -> Audit {
    let costs = sim
        .state
        .balances
        .iter()
        .filter(|((_, r), q)| *r != TOKEN && **q > 0)
        .map(|(key, q)| (*key, i128::from(*q)))
        .collect();
    Audit::with_dues(
        &sim.world,
        &sim.state,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        costs,
        BTreeMap::from([(1, 3)]),
    )
    .unwrap()
    .with_issuance_policy(Policy::NonRedeemableEquity)
    .unwrap()
}
fn through(a: &mut Audit, sim: &mut Simulation, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
#[test]
fn estate_tender_reduces_claims_and_restricted_cash_without_new_dues_income() {
    let mut reference = land_estate(true);
    reference.state.balances.insert((PERSON, TOKEN), 12);
    let mut a = audit(&reference);
    let mut cpu = Simulation::new(
        reference.world.clone(),
        reference.state.clone(),
        Backend::CubeCpu,
    )
    .unwrap();
    let mut b = a.clone();
    through(&mut a, &mut reference, 14);
    through(&mut b, &mut cpu, 14);
    assert_eq!(a, b);
    let debtor = a.book().statements(PERSON, 14, 14).unwrap();
    assert_eq!(debtor.trial_balance[&A::RestrictedCash(1)], 12);
    assert_eq!(debtor.cash_flows.values().sum::<i128>(), 0);
    let custodian = a.book().statements(ESTATE, 14, 14).unwrap();
    assert_eq!(custodian.assets, 12);
    assert_eq!(custodian.liabilities, 12);
    assert_eq!(custodian.equity, 0);
    let mut saved = a.clone();
    let mut checkpoint = reference.clone();
    through(&mut a, &mut reference, 15);
    through(&mut b, &mut cpu, 15);
    through(&mut saved, &mut checkpoint, 15);
    assert_eq!(a, b);
    assert_eq!(a, saved);
    let debtor = a.book().statements(PERSON, 14, 15).unwrap();
    assert_eq!(debtor.trial_balance[&A::DuesPayable(1, 13)], -3);
    assert_eq!(debtor.cash_flows[&Flow::Operating], -2);
    assert_eq!(debtor.cash_flows[&Flow::Financing], -10);
    assert_eq!(debtor.income[&A::SettlementGain], 1);
    assert!(!debtor.expenses.contains_key(&A::DuesExpense));
    let creditor = a.book().statements(STATE_AGENT, 14, 15).unwrap();
    assert_eq!(creditor.trial_balance[&A::DuesReceivable(1, 13)], 3);
    assert_eq!(creditor.expenses[&A::SettlementLoss], 1);
    assert!(!creditor.income.contains_key(&A::DuesIncome));
    assert_eq!(creditor.issuance_change, 0);
    let estate = a.book().statements(ESTATE, 14, 15).unwrap();
    assert_eq!(
        (estate.assets, estate.liabilities, estate.net_income),
        (0, 0, 0)
    );
    assert_eq!(
        reference.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
}
#[test]
fn native_coin_dues_and_untendered_goods_keep_distinct_estate_treatment() {
    for native_coin in [true, false] {
        let mut sim = land_estate(false);
        if native_coin {
            sim.world.agreements[0].payment.resource = TOKEN;
            sim.world.issuance.clear();
        }
        sim.state.balances.insert((PERSON, TOKEN), 12);
        sim = Simulation::new(sim.world, sim.state, Backend::CubeCpu).unwrap();
        let mut a = audit(&sim);
        through(&mut a, &mut sim, 15);
        let r = a.book().statements(PERSON, 14, 15).unwrap();
        if native_coin {
            assert!(sim.state.obligations[&(1, 13)].paid > 0);
            assert_eq!(
                r.cash_flows[&Flow::Operating],
                -i128::from(sim.state.obligations[&(1, 13)].paid)
            );
            assert!(!r.income.contains_key(&A::SettlementGain));
        } else {
            assert_eq!(sim.state.obligations[&(1, 13)].paid, 0);
            assert_eq!(r.trial_balance[&A::DuesPayable(1, 13)], -6);
            assert_eq!(r.cash_flows.get(&Flow::Operating).copied().unwrap_or(0), 0);
            assert_eq!(sim.state.balance(STATE_AGENT, scenario::GRAIN), 0);
        }
    }
}
#[test]
fn forged_estate_dues_receipt_cannot_publish_accounting() {
    let mut sim = land_estate(true);
    sim.state.balances.insert((PERSON, TOKEN), 12);
    let mut a = audit(&sim);
    through(&mut a, &mut sim, 14);
    while sim.state.phase != Phase::Due {
        a.step(&mut sim).unwrap();
    }
    let mut preview = sim.clone();
    preview.step().unwrap();
    let mut bad = preview.ledger.last().unwrap().clone();
    let receipt = bad
        .credit
        .as_mut()
        .unwrap()
        .recovery
        .iter_mut()
        .find(|r| {
            matches!(
                r,
                economics_compute_smoke::recovery::Receipt::LandDistributed { .. }
            )
        })
        .unwrap();
    if let economics_compute_smoke::recovery::Receipt::LandDistributed { tender, .. } = receipt {
        tender.quantity += 1;
    }
    let old = a.clone();
    assert!(
        a.record(&sim.world, &sim.state, &bad, &preview.state)
            .is_err()
    );
    assert_eq!(a, old);
    a.step(&mut sim).unwrap();
}

#[test]
fn ranked_full_dues_payment_composes_with_loan_discharge_and_estate_closure() {
    use economics_compute_smoke::finance::ContractId;
    let mut sim = land_estate(true);
    sim.state.balances.insert((PERSON, TOKEN), 12);
    sim.world.claim_priorities.insert(ContractId::Land(1), 0);
    for id in [10, 11] {
        sim.world.claim_priorities.insert(ContractId::Loan(id), 1);
    }
    let mut a = audit(&sim);
    through(&mut a, &mut sim, 15);
    assert_eq!(sim.state.obligations[&(1, 13)].paid, 2);
    assert_eq!(sim.state.obligations[&(1, 13)].in_kind_paid, 0);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    let r = a.book().statements(PERSON, 14, 15).unwrap();
    assert_eq!((r.assets, r.liabilities, r.equity), (0, 0, 0));
    assert_eq!(r.income[&A::SettlementGain], 2);
    assert_eq!(r.income[&A::DebtRelief], 12);
    assert_eq!(r.cash_flows[&Flow::Operating], -4);
    assert_eq!(r.cash_flows[&Flow::Financing], -8);
    let creditor = a.book().statements(STATE_AGENT, 14, 15).unwrap();
    assert_eq!(creditor.expenses[&A::SettlementLoss], 2);
    assert_eq!(creditor.expenses[&A::CreditLoss], 6);
    assert_eq!(creditor.issuance_change, 0);
    let estate = a.book().statements(ESTATE, 14, 15).unwrap();
    assert_eq!(
        (estate.assets, estate.liabilities, estate.net_income),
        (0, 0, 0)
    );
    through(&mut a, &mut sim, 16);
    assert_eq!(a.book().statements(PERSON, 16, 16).unwrap().net_income, 0);
}

fn land_relief(quantity: i32) -> economics_compute_smoke::claim_relief::Terms {
    economics_compute_smoke::claim_relief::Terms {
        id: 1,
        proceeding: 1,
        contract: economics_compute_smoke::finance::ContractId::Land(1),
        original_due: 13,
        debtor: PERSON,
        creditor: STATE_AGENT,
        month: 14,
        expected_due: 13,
        expected_remaining: 2,
        action: economics_compute_smoke::claim_relief::Action::WriteOff { quantity },
    }
}
#[test]
fn native_land_writeoff_preserves_bill_payment_and_issuance_history() {
    let mut reference = land_estate(false);
    reference.world.recovery.claim_relief.push(land_relief(2));
    let mut a = audit(&reference);
    let mut b = a.clone();
    let mut cpu = Simulation::new(
        reference.world.clone(),
        reference.state.clone(),
        Backend::CubeCpu,
    )
    .unwrap();
    let opening = reference.state.balances.clone();
    through(&mut a, &mut reference, 14);
    through(&mut b, &mut cpu, 14);
    let o = &reference.state.obligations[&(1, 13)];
    assert_eq!(
        (
            o.owed,
            o.paid,
            o.in_kind_paid,
            o.written_off(),
            o.outstanding()
        ),
        (2, 0, 0, 2, 0)
    );
    assert_eq!(reference.state.balances, opening);
    assert_eq!(a.book().balances()[&(PERSON, A::DebtRelief)], -6);
    assert_eq!(a.book().balances()[&(STATE_AGENT, A::CreditLoss)], 6);
    let mut resumed = reference.clone();
    let mut c = a.clone();
    through(&mut a, &mut reference, 16);
    through(&mut b, &mut cpu, 16);
    through(&mut c, &mut resumed, 16);
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.state, resumed.state);
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(
        reference.state.credit.recovery.proceedings[&1].stage,
        Stage::Closed
    );
    assert_eq!(reference.state.balance(PERSON, TOKEN), 0);
    assert_eq!(reference.state.balance(STATE_AGENT, TOKEN), 0);
}
#[test]
fn partial_land_relief_leaves_collectible_native_claim_and_stale_terms_do_not_apply() {
    use economics_compute_smoke::recovery::Receipt;
    let mut sim = land_estate(true);
    sim.world.recovery.claim_relief.push(land_relief(1));
    let mut a = audit(&sim);
    through(&mut a, &mut sim, 15);
    assert_eq!(sim.state.obligations[&(1, 13)].outstanding(), 1);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert_eq!(a.book().balances()[&(PERSON, A::DuesPayable(1, 13))], -3);
    let mut stale = land_estate(false);
    stale.world.recovery.claim_relief.push(land_relief(2));
    stale.state.balances.insert((PERSON, scenario::GRAIN), 1);
    let mut b = audit(&stale);
    through(&mut b, &mut stale, 14);
    let bill = &stale.state.obligations[&(1, 13)];
    assert_eq!(
        (
            bill.paid,
            bill.in_kind_paid,
            bill.written_off(),
            bill.outstanding()
        ),
        (1, 1, 0, 1)
    );
    assert!(
        stale
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.recovery)
            .any(|r| matches!(
                r,
                Receipt::ClaimRelief {
                    rejection: Some(_),
                    written_off: None,
                    ..
                }
            ))
    );
    let mut forged = sim.state.clone();
    forged.obligations.get_mut(&(1, 13)).unwrap().relief[0]
        .terms
        .action = economics_compute_smoke::claim_relief::Action::WriteOff { quantity: 2 };
    assert!(Simulation::new(sim.world.clone(), forged, Backend::Reference).is_err());
}

#[test]
fn land_extension_delays_estate_tender_without_moving_annual_billing() {
    use economics_compute_smoke::{
        claim_relief::Action,
        finance::ContractId,
        recovery::{Bid, Listing},
    };
    const SOLD_ASSET: AssetId = 901;
    let mut reference = land_estate(true);
    reference.state.balances.insert((PERSON, TOKEN), 12);
    let mut relief = land_relief(2);
    relief.action = Action::Extend { due: 16 };
    reference.world.recovery.claim_relief.push(relief);
    reference.world.assets.push(Asset {
        id: SOLD_ASSET,
        owner: PERSON,
        kind: 1,
    });
    reference.world.recovery.proceedings[0]
        .assets
        .push(Listing {
            asset: SOLD_ASSET,
            minimum_price: 4,
        });
    reference.world.recovery.bids.push(Bid {
        id: 1,
        proceeding: 1,
        buyer: BUYER,
        asset: SOLD_ASSET,
        month: 15,
        price: 4,
    });
    reference
        .world
        .claim_priorities
        .insert(ContractId::Land(1), 0);
    for id in [10, 11] {
        reference
            .world
            .claim_priorities
            .insert(ContractId::Loan(id), 1);
    }
    let mut a = Audit::with_dues(
        &reference.world,
        &reference.state,
        TOKEN,
        [(PLOT, 0), (SOLD_ASSET, 0)].into(),
        BTreeMap::new(),
        [(1, 3)].into(),
    )
    .unwrap()
    .with_issuance_policy(Policy::NonRedeemableEquity)
    .unwrap();
    let mut b = a.clone();
    let mut cpu = Simulation::new(
        reference.world.clone(),
        reference.state.clone(),
        Backend::CubeCpu,
    )
    .unwrap();
    through(&mut a, &mut reference, 15);
    through(&mut b, &mut cpu, 15);
    let bill = &reference.state.obligations[&(1, 13)];
    assert_eq!(
        (
            bill.due,
            bill.effective_due(),
            bill.paid,
            bill.outstanding()
        ),
        (13, 16, 0, 2)
    );
    assert_eq!(reference.state.balance(ESTATE, TOKEN), 4);
    assert_eq!(
        reference.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert!(
        reference.world.agreements[0]
            .contract(&reference.world, &reference.state)
            .unwrap()
            .evaluate(15)
            .breaches
            .is_empty()
    );
    let mut resumed = reference.clone();
    let mut c = a.clone();
    through(&mut a, &mut reference, 25);
    through(&mut b, &mut cpu, 25);
    through(&mut c, &mut resumed, 25);
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.state, resumed.state);
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(reference.state.obligations[&(1, 13)].paid, 2);
    assert_eq!(reference.state.obligations[&(1, 13)].in_kind_paid, 0);
    assert_eq!(reference.state.obligations[&(1, 13)].written_off(), 0);
    assert_eq!(reference.state.obligations[&(1, 25)].outstanding(), 2);
    assert_eq!(
        reference.state.credit.recovery.proceedings[&1].closed,
        Some(16)
    );
    assert_eq!(reference.state.balance(PERSON, TOKEN), 0);
}
