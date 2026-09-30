use economics_compute_smoke::{
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::{Audit, Opening},
    household_governance::Governance,
    households::{self, Agreement},
    model::*,
    offers::{self, Id, Request},
    recovery::{ProceedingTerms, receivables},
    scenario::*,
    settlement,
    simulation::Simulation,
};
const BORROWER: AgentId = 97;
const BUYER: AgentId = 98;
const ESTATE: AgentId = 99;
const HOME: AgentId = 10000;

fn fixture(household: bool, funded: bool, room: bool) -> (World, State, AgentId) {
    let (mut w, mut s) = baseline();
    w.definitions.clear();
    w.assets.clear();
    w.rights.clear();
    w.agreements.clear();
    w.participants[0].needs.clear();
    w.participants[0].capacity.quantity = 0;
    s.balances.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    for id in [BORROWER, BUYER, ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    let seller = if household {
        let mut governance = Governance::contributed(PERSON);
        governance.constitution.allow_dissolution = true;
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: vec![PERSON],
                governance,
                formed: 1,
                dwelling_process: None,
                admission: None,
                membership: vec![],
                asset_sales: vec![],
                equipment_retirements: vec![],
                support: vec![],
            },
        )
        .unwrap();
        s.balances.insert((PERSON, TOKEN), 5);
        HOME
    } else {
        PERSON
    };
    for (id, debtor, creditor, principal, denomination, rate, term) in [
        (10, seller, STATE_AGENT, 100, TOKEN, 1000, 1),
        (11, BORROWER, seller, 2, SEED, 0, 6),
    ] {
        w.lending.push(Advance {
            id,
            debtor,
            principal,
            month: 1,
            collateral: None,
            priority: 0,
            terms: LoanOffer {
                creditor,
                denomination,
                max_principal: principal,
                monthly_rate_bps: rate,
                term_months: term,
                grace_months: 12,
            },
        });
    }
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: seller,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![],
        discharge_deficiency: true,
    });
    w.recovery.receivable_listings.push(receivables::Listing {
        id: 1,
        proceeding: 1,
        loan: 11,
        coins_per_unit: 2,
    });
    w.recovery.receivable_bids.push(receivables::Bid {
        id: 1,
        listing: 1,
        buyer: BUYER,
        month: 3,
        price: 4,
    });
    w.storage.weights.insert(SEED, 1);
    w.storage.capacities.insert(BUYER, if room { 2 } else { 0 });
    s.balances.insert((STATE_AGENT, TOKEN), 100);
    s.balances.insert((seller, SEED), 2);
    s.balances
        .insert((BUYER, TOKEN), if funded { 4 } else { 3 });
    (w, s, seller)
}
fn opening(seller: AgentId, unit: i128) -> Opening {
    Opening {
        inventory: [((seller, SEED), 4)].into(),
        exchange_values: [(SEED, unit)].into(),
        ..Opening::default()
    }
}

#[test]
fn estate_sells_native_claim_for_coins_and_buyer_collects_goods_with_real_storage() {
    assignment(false);
}

#[test]
fn native_guarantees_follow_sold_claims_and_leave_recourse_with_the_original_borrower() {
    assignment(true);
}

fn assignment(guaranteed: bool) {
    for household in [false, true] {
        for funded in [false, true] {
            for room in [false, true] {
                let (mut w, s, seller) = fixture(household, funded, room);
                const GUARANTOR: AgentId = 96;
                if guaranteed {
                    use economics_compute_smoke::{
                        employment::{ArrearsPolicy, Terms},
                        recovery,
                    };
                    w.agents.push(Agent {
                        id: GUARANTOR,
                        name: "worker and guarantor".into(),
                    });
                    w.participants.push(Participant {
                        agent: GUARANTOR,
                        capacity: Amount::new(LABOR, 1),
                        needs: vec![],
                    });
                    // Actual work earns the borrowed seeds. The original borrower
                    // spends them as wages and cannot also deliver loan repayments.
                    w.employment.push(Terms {
                        id: 1,
                        employer: BORROWER,
                        worker: GUARANTOR,
                        from: 1,
                        through: 1,
                        capacity: Amount::new(LABOR, 1),
                        wage_per_unit: Amount::new(SEED, 2),
                        on_arrears: ArrearsPolicy::Continue,
                        rank: 0,
                    });
                    w.recovery.guarantees.push(recovery::Guarantee {
                        id: 1,
                        follows_assignment: true,
                        tender: recovery::GuaranteeTender::Native,
                        security: recovery::RecourseSecurity::Unsecured,
                        claim: recovery::GuaranteedClaim::Loan(11),
                        guarantor: GUARANTOR,
                        cap: 2,
                        from: 4,
                        through: 8,
                        delay_months: 0,
                        recourse: 200,
                        priority: 0,
                    });
                }
                let run = |backend| {
                    let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                    let mut audit = Audit::with_opening(&w, &s, TOKEN, opening(seller, 2)).unwrap();
                    while sim.state.month < 2 {
                        audit.step(&mut sim).unwrap();
                    }
                    if household {
                        households::dissolution::request(&mut sim.world, &sim.state, HOME, PERSON)
                            .unwrap();
                    }
                    while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
                        audit.step(&mut sim).unwrap();
                    }
                    let discovered = receivables::discover(&sim.world, &sim.state, BUYER);
                    assert_eq!(discovered.len(), 1);
                    assert_eq!(discovered[0].remaining, Amount::new(SEED, 2));
                    assert_eq!(discovered[0].payment_resource, TOKEN);
                    assert_eq!(discovered[0].listing.coins_per_unit, 2);
                    let requests = [Request::new(Id::ReceivableLiquidationBid(1), BUYER)];
                    let before = sim.state.clone();
                    let prepared = offers::prepare(&sim, &requests);
                    assert_eq!(sim.state, before);
                    if funded {
                        let prepared = prepared.unwrap();
                        let mut bad = prepared.clone();
                        bad.credit
                            .as_mut()
                            .unwrap()
                            .after
                            .loans
                            .get_mut(&11)
                            .unwrap()
                            .denomination = TOKEN;
                        assert!(
                            settlement::commit(
                                &sim.world,
                                &mut sim.state,
                                &bad,
                                backend,
                                sim.effect_limit
                            )
                            .is_err()
                        );
                        assert_eq!(sim.state, before);
                        offers::accept(&mut sim, &requests).unwrap();
                        audit
                            .record(&sim.world, &before, &prepared, &sim.state)
                            .unwrap();
                    } else {
                        assert!(prepared.is_err());
                        assert!(offers::accept(&mut sim, &requests).is_err());
                        assert_eq!(sim.state, before);
                        audit.step(&mut sim).unwrap();
                    }
                    assert_eq!(
                        sim.state.credit.loans[&11].creditor,
                        if funded { BUYER } else { seller }
                    );
                    assert_eq!(sim.state.credit.loans[&11].denomination, SEED);
                    assert_eq!(sim.state.balance(BUYER, SEED), 0);
                    assert_eq!(sim.state.balance(ESTATE, TOKEN), if funded { 4 } else { 0 });
                    let mut resumed =
                        Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
                    let mut ra = audit.clone();
                    while sim.state.month < 8 {
                        audit.step(&mut sim).unwrap();
                    }
                    while resumed.state.month < 8 {
                        ra.step(&mut resumed).unwrap();
                    }
                    assert_eq!((&sim.state, &audit), (&resumed.state, &ra));
                    assert_eq!(
                        sim.state.balance(BUYER, SEED),
                        if funded && room { 2 } else { 0 }
                    );
                    assert_eq!(
                        sim.state.credit.loans[&11].principal,
                        if funded && !room { 2 } else { 0 }
                    );
                    assert_eq!(
                        sim.state.balance(STATE_AGENT, TOKEN),
                        if funded { 104 } else { 100 }
                    );
                    assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
                    if guaranteed {
                        let paid = if funded && !room { 0 } else { 2 };
                        assert_eq!(
                            sim.state
                                .credit
                                .recovery
                                .paid_guarantees
                                .get(&1)
                                .copied()
                                .unwrap_or(0),
                            paid
                        );
                        assert_eq!(sim.state.balance(GUARANTOR, SEED), 2 - paid);
                        assert_eq!(sim.state.balance(BORROWER, SEED), 0);
                        if paid > 0 {
                            let recourse = &sim.state.credit.loans[&200];
                            assert_eq!(
                                (
                                    recourse.debtor,
                                    recourse.creditor,
                                    recourse.denomination,
                                    recourse.principal
                                ),
                                (BORROWER, GUARANTOR, SEED, 2)
                            );
                        } else {
                            assert!(!sim.state.credit.loans.contains_key(&200));
                        }
                    }
                    if household {
                        assert_eq!(sim.state.balance(PERSON, TOKEN), 5);
                    }
                    (sim.state, sim.ledger, audit)
                };
                assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
            }
        }
    }
}

#[test]
fn native_assignment_requires_positive_quote_and_matching_fixed_report_value() {
    let (w, s, seller) = fixture(false, true, true);
    let mut zero = w.clone();
    zero.recovery.receivable_listings[0].coins_per_unit = 0;
    assert!(Simulation::new(zero, s.clone(), Backend::Reference).is_err());
    assert!(Audit::with_opening(&w, &s, TOKEN, opening(seller, 1)).is_err());
    let mut stale = w.clone();
    stale.recovery.receivable_bids[0].price = 3;
    let mut sim = Simulation::new(stale, s, Backend::Reference).unwrap();
    while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
        sim.step().unwrap();
    }
    let before = sim.state.clone();
    assert!(
        offers::accept(
            &mut sim,
            &[Request::new(Id::ReceivableLiquidationBid(1), BUYER)]
        )
        .is_err()
    );
    assert_eq!(sim.state, before);
}
