use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Advance, CollateralSettlement, LoanOffer},
    financial_reporting::{Audit, Opening},
    household_governance::Governance,
    households::{self, Agreement, dissolution},
    model::*,
    offers::{self, Id, Request},
    process_accounting::{Costs, Output},
    recovery::{self, Bid, Listing, ProceedingTerms, receivables},
    scenario::*,
    simulation::Simulation,
};
const HOME: AgentId = 10000;
const MEMBER: AgentId = 98;
const INVESTOR: AgentId = 96;
const PROPERTY_ESTATE: AgentId = 97;
const HOME_ESTATE: AgentId = 99;

fn fixture(funded: bool) -> (World, State) {
    let (mut w, mut s) = credit::crop_scenario(false).unwrap();
    let c = w.credit.as_mut().unwrap();
    c.endowments.clear();
    c.application.downpayment = 2;
    let o = &mut c.offers[0];
    o.sale.price.quantity = 8;
    o.minimum_downpayment = 2;
    o.loan.creditor = HOME;
    o.loan.max_principal = 6;
    o.loan.monthly_rate_bps = 0;
    o.loan.term_months = 1;
    o.loan.grace_months = 0;
    o.collateral.settlement = CollateralSettlement::AuthorizedLiquidation;
    for id in [MEMBER, INVESTOR, PROPERTY_ESTATE, HOME_ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    w.participants.push(Participant {
        agent: MEMBER,
        capacity: Amount::new(LABOR, 2),
        needs: vec![],
    });
    let mut governance = Governance::contributed(MEMBER);
    governance.constitution.allow_dissolution = true;
    households::form(
        &mut w,
        &s,
        Agreement {
            id: 1,
            agent: HOME,
            adults: vec![MEMBER],
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
    s.balances.insert((HOME, TOKEN), 6);
    s.balances.insert((PERSON, TOKEN), 2);
    s.balances.insert((STATE_AGENT, TOKEN), 120);
    s.balances.insert((MEMBER, TOKEN), 4);
    s.balances
        .insert((INVESTOR, TOKEN), if funded { 6 } else { 5 });
    w.lending.push(Advance {
        id: 10,
        debtor: HOME,
        principal: 100,
        month: 1,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: 100,
            monthly_rate_bps: 1000,
            term_months: 1,
            grace_months: 12,
        },
    });
    w.recovery.proceedings = vec![
        ProceedingTerms {
            id: 1,
            debtor: PERSON,
            authority: STATE_AGENT,
            estate: PROPERTY_ESTATE,
            denomination: TOKEN,
            opening_month: 3,
            earliest_close: 5,
            assets: vec![Listing {
                asset: PLOT,
                minimum_price: 4,
            }],
            discharge_deficiency: false,
        },
        ProceedingTerms {
            id: 2,
            debtor: HOME,
            authority: STATE_AGENT,
            estate: HOME_ESTATE,
            denomination: TOKEN,
            opening_month: 3,
            earliest_close: 5,
            assets: vec![],
            discharge_deficiency: true,
        },
    ];
    w.recovery.bids.push(Bid {
        id: 1,
        proceeding: 1,
        buyer: MEMBER,
        asset: PLOT,
        month: 4,
        price: 4,
    });
    w.recovery.receivable_listings.push(receivables::Listing {
        id: 1,
        proceeding: 2,
        loan: 1,
        coins_per_unit: 1,
    });
    w.recovery.receivable_bids.push(receivables::Bid {
        id: 1,
        listing: 1,
        buyer: INVESTOR,
        month: 3,
        price: 6,
    });
    (w, s)
}

#[test]
fn winding_household_sells_mortgage_claim_and_new_holder_receives_actual_collateral_proceeds() {
    assignment(None, false);
}

#[test]
fn mortgage_assignment_retains_guarantee_consent_and_inherited_liens_across_custody() {
    for from in [4, 5] {
        for shared in [false, true] {
            assignment(Some(from), shared);
        }
    }
}

fn assignment(guarantee_from: Option<u32>, shared_custody: bool) {
    for funded in [false, true] {
        let (mut w, s) = fixture(funded);
        if shared_custody {
            w.recovery.proceedings[0].estate = HOME_ESTATE;
        }
        if let Some(from) = guarantee_from {
            w.recovery.guarantees.push(recovery::Guarantee {
                id: 1,
                claim: recovery::GuaranteedClaim::Loan(1),
                guarantor: STATE_AGENT,
                cap: 2,
                from,
                through: from,
                delay_months: 0,
                recourse: 200,
                priority: 0,
                follows_assignment: true,
                tender: recovery::GuaranteeTender::Native,
                security: recovery::RecourseSecurity::InheritLiquidationLien,
            });
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    assets: [(PLOT, 8)].into(),
                    inventory: [((PERSON, SEED), i128::from(s.balance(PERSON, SEED)))].into(),
                    exchange_values: [(SEED, 1), (GRAIN, 1)].into(),
                    processes: Some(Costs {
                        output_weights: [(
                            GROW,
                            [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                        )]
                        .into(),
                        ..Costs::default()
                    }),
                    ..Opening::default()
                },
            )
            .unwrap();
            while sim.state.month < 2 {
                audit.step(&mut sim).unwrap();
            }
            dissolution::request(&mut sim.world, &sim.state, HOME, MEMBER).unwrap();
            while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
                audit.step(&mut sim).unwrap();
            }
            let before = sim.state.clone();
            let request = [Request::new(Id::ReceivableLiquidationBid(1), INVESTOR)];
            let discovered = receivables::discover(&sim.world, &sim.state, INVESTOR);
            assert_eq!(discovered.len(), 1);
            assert_eq!(discovered[0].loan.collateral.as_ref().unwrap().asset, PLOT);
            let prepared = offers::prepare(&sim, &request);
            assert_eq!(sim.state, before);
            if funded {
                let prepared = prepared.unwrap();
                offers::accept(&mut sim, &request).unwrap();
                audit
                    .record(&sim.world, &before, &prepared, &sim.state)
                    .unwrap();
                assert_eq!(sim.state.credit.loans[&1].creditor, INVESTOR);
                assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(PERSON));
                assert_eq!(sim.state.balance(HOME_ESTATE, TOKEN), 6);
                assert_eq!(sim.state.credit.loans[&10].principal, 10);
            } else {
                assert!(prepared.is_err());
                audit.step(&mut sim).unwrap();
            }
            let mut resumed =
                Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
            let mut ra = audit.clone();
            let start = sim.ledger.len();
            while sim.state.month < 8 {
                audit.step(&mut sim).unwrap();
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(sim.state, resumed.state);
            assert_eq!(&sim.ledger[start..], resumed.ledger.as_slice());
            assert_eq!(audit, ra);
            let recovered = if guarantee_from == Some(4) { 6 } else { 4 };
            assert_eq!(sim.state.credit.loans[&1].principal, 6 - recovered);
            if let Some(from) = guarantee_from {
                let recourse = &sim.state.credit.loans[&200];
                assert_eq!(recourse.creditor, STATE_AGENT);
                assert_eq!(recourse.debtor, PERSON);
                assert_eq!(recourse.principal, if from == 4 { 2 } else { 0 });
                assert_eq!(recourse.collateral.as_ref().unwrap().asset, PLOT);
                assert!(sim.ledger.iter().filter(|b| b.month == from).all(|b| {
                    b.credit.as_ref().is_none_or(|c| {
                        c.recovery
                            .iter()
                            .all(|r| !matches!(r, recovery::Receipt::Distributed { loan: 200, paid, .. } if *paid > 0))
                    })
                }));
            }
            assert_eq!(
                sim.state.credit.loans[&1].creditor,
                if funded { INVESTOR } else { HOME }
            );
            assert_eq!(
                sim.state.balance(INVESTOR, TOKEN),
                if funded { recovered } else { 5 }
            );
            assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(MEMBER));
            assert_eq!(sim.state.balance(MEMBER, TOKEN), 0);
            assert!(
                sim.state
                    .processes
                    .values()
                    .any(|p| p.definition == GROW && p.status == Status::Completed)
            );
            assert_eq!(
                recovery::active(&sim.world, &sim.state.credit, HOME).is_none(),
                funded || recovered == 6
            );
            if funded || recovered == 6 {
                assert!(dissolution::blockers(&sim.world, &sim.state, HOME).is_empty());
                dissolution::finish(&mut sim.world, &sim.state, HOME, MEMBER).unwrap();
            } else {
                assert!(dissolution::finish(&mut sim.world, &sim.state, HOME, MEMBER).is_err());
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
