use economics_compute_smoke::{
    compute::Backend,
    credit::{Advance, LoanOffer},
    dues_accounting::Valuation,
    financial_reporting::{Audit, Opening},
    model::*,
    offers::{self, Id, Request},
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    process_accounting::{Costs, Output},
    recovery::{self, ProceedingTerms, Stage, inventory},
    scenario::*,
    settlement,
    simulation::Simulation,
};
const DEBTOR: AgentId = 90;
const ESTATE: AgentId = 99;

#[test]
fn purchased_estate_seed_funds_a_dated_crop_without_spending_new_custody_receipts() {
    productive_claim(None);
}

#[test]
fn priced_claims_share_acquisition_cash_with_seed_and_dated_cultivation() {
    for price in [1, 3] {
        productive_claim(Some(price));
    }
}

fn productive_claim(price: Option<i32>) {
    for funded in [false, true] {
        for buy_claim in [false, true] {
            if price.is_some() && !buy_claim {
                continue;
            }
            let (mut w, mut s) = named("opportunity-farming").unwrap();
            w.resources.push(Resource {
                id: TOKEN,
                name: "coins".into(),
                kind: ResourceKind::Stock,
            });
            for (id, name) in [(DEBTOR, "seed owner"), (ESTATE, "custodian")] {
                w.agents.push(Agent {
                    id,
                    name: name.into(),
                });
            }
            let policy = w.transaction_policy.as_mut().unwrap();
            policy.agent_types.insert(DEBTOR, PERSON_TYPE);
            policy.permissions.insert((PERSON_TYPE, Action::Borrow));
            policy.permissions.insert((PERSON_TYPE, Action::StockTrade));
            policy.permissions.insert((STATE_TYPE, Action::Lend));
            s.balances.insert((PERSON, SEED), 0);
            s.balances.insert((PERSON, GRAIN), 10);
            s.balances.insert(
                (PERSON, TOKEN),
                if funded {
                    if buy_claim { price.unwrap_or(2) + 1 } else { 1 }
                } else {
                    price.unwrap_or(0)
                },
            );
            s.balances.insert((STATE_AGENT, TOKEN), 100);
            s.balances.insert((DEBTOR, SEED), 1);
            w.lending.push(Advance {
                id: 10,
                debtor: DEBTOR,
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
            w.recovery.proceedings.push(ProceedingTerms {
                id: 1,
                debtor: DEBTOR,
                authority: STATE_AGENT,
                estate: ESTATE,
                denomination: TOKEN,
                opening_month: 3,
                earliest_close: 4,
                assets: vec![],
                discharge_deficiency: false,
            });
            w.recovery.inventory_listings.push(inventory::Listing {
                id: 1,
                proceeding: 1,
                goods: Amount::new(SEED, 1),
                minimum_price: 1,
            });
            w.recovery.inventory_bids.push(inventory::Bid {
                id: 1,
                listing: 1,
                buyer: PERSON,
                month: 3,
                price: 1,
            });
            if buy_claim {
                let borrower = 98;
                w.agents.push(Agent {
                    id: borrower,
                    name: "claim counterparty".into(),
                });
                let p = w.transaction_policy.as_mut().unwrap();
                p.agent_types.insert(borrower, PERSON_TYPE);
                p.permissions.insert((PERSON_TYPE, Action::Lend));
                p.permissions.insert((PERSON_TYPE, Action::AssetTrade));
                let mut loan = w.lending[0].clone();
                loan.id = 11;
                loan.debtor = borrower;
                loan.principal = 2;
                loan.terms.creditor = DEBTOR;
                loan.terms.max_principal = 2;
                loan.terms.monthly_rate_bps = 0;
                loan.terms.term_months = 6;
                w.lending.push(loan);
                s.balances.insert((DEBTOR, TOKEN), 2);
                w.recovery
                    .receivable_listings
                    .push(recovery::receivables::Listing {
                        coins_per_unit: 1,
                        id: 1,
                        proceeding: 1,
                        loan: 11,
                    });
                w.recovery.receivable_bids.push(recovery::receivables::Bid {
                    id: 1,
                    listing: 1,
                    buyer: PERSON,
                    month: 3,
                    price: price.unwrap_or(2),
                });
                if price.is_some() {
                    w.recovery.receivable_price_floors.insert(1, 1);
                }
            }
            let run = |backend| {
                let mut audit = Audit::with_opening(
                    &w,
                    &s,
                    TOKEN,
                    Opening {
                        assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
                        inventory: s
                            .balances
                            .iter()
                            .filter(|((_, r), q)| **q > 0 && *r != TOKEN)
                            .map(|(a, q)| (*a, i128::from(*q)))
                            .collect(),
                        exchange_values: [(GRAIN, 1), (SEED, 1), (RAW_WOOD, 1), (FUEL, 1)].into(),
                        processes: Some(Costs {
                            output_weights: [(
                                GROW,
                                [(Output::Stock(GRAIN), 1), (Output::Stock(SEED), 1)].into(),
                            )]
                            .into(),
                            ..Costs::default()
                        }),
                        dues: Some(Valuation([(1, 1)].into())),
                        ..Opening::default()
                    },
                )
                .unwrap();
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
                    audit.step(&mut sim).unwrap();
                }
                assert_eq!(
                    sim.state.credit.recovery.proceedings[&1].stage,
                    Stage::Active
                );
                assert!(
                    offers::discover(&sim.world, &sim.state, PERSON)
                        .iter()
                        .any(|o| o.id == Id::InventoryLiquidationBid(1))
                );
                let mut requests = vec![Request::new(Id::InventoryLiquidationBid(1), PERSON)];
                if buy_claim {
                    requests.push(Request::new(Id::ReceivableLiquidationBid(1), PERSON));
                }
                if sim.state.memberships.is_empty() {
                    requests.push(Request::new(Id::Membership(1), PERSON));
                }
                if sim.state.accepted_agreements.is_empty() {
                    requests.push(Request::new(Id::Land(1), PERSON));
                }
                requests.push(Request::new(Id::Process(GROW), PERSON));
                let before = sim.state.clone();
                let prepared = offers::prepare(&sim, &requests);
                assert_eq!(sim.state, before);
                if !funded {
                    assert!(prepared.is_err());
                    assert!(offers::accept(&mut sim, &requests).is_err());
                    assert_eq!(sim.state, before);
                    assert_eq!(sim.state.balance(DEBTOR, SEED), 1);
                    return (sim.state, sim.ledger, audit);
                }
                let prepared = prepared.unwrap();
                let mut bad = prepared.clone();
                bad.credit
                    .as_mut()
                    .unwrap()
                    .recovery
                    .retain(|r| !matches!(r, recovery::Receipt::InventorySold { .. }));
                let mut unchanged = before.clone();
                assert!(
                    settlement::commit(&sim.world, &mut unchanged, &bad, backend, sim.effect_limit)
                        .is_err()
                );
                assert_eq!(unchanged, before);
                offers::accept(&mut sim, &requests).unwrap();
                audit
                    .record(&sim.world, &before, &prepared, &sim.state)
                    .unwrap();
                assert_eq!(sim.state.balance(PERSON, SEED), 1);
                assert_eq!(
                    sim.state.balance(ESTATE, TOKEN),
                    if buy_claim { price.unwrap_or(2) + 1 } else { 1 }
                );
                if buy_claim {
                    assert_eq!(sim.state.credit.loans[&11].creditor, PERSON);
                    assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
                }
                assert_eq!(sim.state.credit.loans[&10].principal, 10);
                audit.step(&mut sim).unwrap();
                assert_eq!(sim.state.balance(PERSON, SEED), 0);
                let mut resumed =
                    Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
                let mut ra = audit.clone();
                while sim.state.month < 10 {
                    audit.step(&mut sim).unwrap();
                }
                while resumed.state.month < 10 {
                    ra.step(&mut resumed).unwrap();
                }
                assert_eq!((&sim.state, &audit), (&resumed.state, &ra));
                assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
                assert_eq!(
                    sim.state.credit.loans[&10].principal,
                    if buy_claim { 9 - price.unwrap_or(2) } else { 9 }
                );
                if buy_claim {
                    assert_eq!(sim.state.credit.loans[&11].principal, 0);
                    assert_eq!(sim.state.balance(PERSON, TOKEN), 2);
                    if let Some(price) = price {
                        use economics_compute_smoke::accounting::Account;
                        let balance = |who, account| {
                            audit
                                .book()
                                .balances()
                                .get(&(who, account))
                                .copied()
                                .unwrap_or(0)
                        };
                        assert_eq!(balance(PERSON, Account::LoanBasisAdjustment(11)), 0);
                        assert_eq!(
                            balance(PERSON, Account::SettlementGain)
                                + balance(PERSON, Account::SettlementLoss),
                            i128::from(price - 2)
                        );
                        assert_eq!(
                            balance(DEBTOR, Account::DisposalGain)
                                + balance(DEBTOR, Account::DisposalLoss),
                            i128::from(2 - price)
                        );
                    }
                }
                assert!(
                    sim.state
                        .processes
                        .values()
                        .any(|p| p.definition == GROW && p.status == Status::Completed)
                );
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}
