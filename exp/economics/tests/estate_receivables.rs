use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Advance, LoanOffer},
    financial_reporting::Audit,
    model::*,
    recovery::{self, ProceedingTerms, Receipt, Stage},
    recovery_claims,
    scenario::{self, PERSON, STATE_AGENT, TOKEN},
    simulation::Simulation,
};
const BORROWER: AgentId = 97;
const ESTATE: AgentId = 99;
const DEBT: u32 = 10;
const ASSET: u32 = 11;
const HOME: AgentId = 10000;

fn fixture(principal: i32, term: u32, funded: bool) -> Simulation {
    fixture_for(principal, term, funded, false)
}
fn fixture_for(principal: i32, term: u32, funded: bool, household: bool) -> Simulation {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, Agreement},
    };
    let (mut w, mut s) = scenario::baseline();
    w.participants[0].needs.clear();
    w.participants[0].capacity.quantity = 0;
    if !household {
        w.participants.clear();
    }
    w.definitions.clear();
    w.assets.clear();
    w.rights.clear();
    w.agreements.clear();
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
    for id in [BORROWER, ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    let debtor = if household {
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
        HOME
    } else {
        PERSON
    };
    let loan = |id, creditor, debtor, principal, term| Advance {
        id,
        debtor,
        month: 1,
        principal,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor,
            denomination: TOKEN,
            max_principal: principal,
            monthly_rate_bps: 0,
            term_months: term,
            grace_months: 10,
        },
    };
    w.lending = vec![
        loan(DEBT, STATE_AGENT, debtor, 10, 1),
        loan(ASSET, debtor, BORROWER, principal, term),
    ];
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor,
        estate: ESTATE,
        authority: STATE_AGENT,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 3,
        assets: vec![],
        discharge_deficiency: true,
    });
    s.balances.clear();
    s.balances.insert((STATE_AGENT, TOKEN), 10);
    s.balances.insert((debtor, TOKEN), principal);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    // A disclosed distressed opening snapshot. Reporting begins here; the
    // depletion is not claimed as a transaction or loss produced by this run.
    sim.state.balances.insert((debtor, TOKEN), 0);
    if !funded {
        sim.state.balances.insert((BORROWER, TOKEN), 0);
    }
    if household {
        households::dissolution::request(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
    }
    Simulation::new(sim.world, sim.state, Backend::Reference).unwrap()
}

#[test]
fn household_wind_down_retains_financial_assets_until_collection_and_estate_release() {
    use economics_compute_smoke::households::dissolution;
    let opening = fixture_for(3, 4, true, true);
    let run = |backend| {
        let mut sim = opening.clone();
        sim.backend = backend;
        let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
        until(&mut sim, &mut audit, 4, Phase::Open);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Active
        );
        assert!(dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).is_err());
        assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
        assert!(
            dissolution::blockers(&sim.world, &sim.state, HOME)
                .contains(&dissolution::Blocker::Loan)
        );
        let checkpoint = (sim.clone(), audit.clone());
        until(&mut sim, &mut audit, 8, Phase::Open);
        assert!(dissolution::blockers(&sim.world, &sim.state, HOME).is_empty());
        dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
        until(&mut sim, &mut audit, 9, Phase::Open);
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 3);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Closed
        );
        let (mut resumed, mut resumed_audit) = checkpoint;
        until(&mut resumed, &mut resumed_audit, 8, Phase::Open);
        dissolution::finish(&mut resumed.world, &resumed.state, HOME, PERSON).unwrap();
        until(&mut resumed, &mut resumed_audit, 9, Phase::Open);
        assert_eq!(
            (&sim.state, &sim.ledger, &audit),
            (&resumed.state, &resumed.ledger, &resumed_audit)
        );
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
fn until(sim: &mut Simulation, audit: &mut Audit, month: u32, phase: Phase) {
    while (sim.state.month, sim.state.phase) != (month, phase) {
        audit.step(sim).unwrap();
    }
}

#[test]
fn estate_waits_for_receivable_collection_and_new_cash_without_rehypothecation() {
    for (principal, term) in [(1, 2), (3, 4)] {
        let opening = fixture(principal, term, true);
        let run = |backend| {
            let mut sim = opening.clone();
            sim.backend = backend;
            let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
            until(&mut sim, &mut audit, 3, Phase::Acquire);
            let case = &sim.state.credit.recovery.proceedings[&1];
            assert_eq!(case.stage, Stage::Active);
            assert!(sim.state.credit.loans[&DEBT].principal > 0);
            let b = sim.ledger.last().unwrap().credit.as_ref().unwrap();
            assert!(
                b.recovery
                    .iter()
                    .any(|r| matches!(r, Receipt::AssetsPending { proceeding: 1, .. }))
            );
            if principal == 1 {
                // The asset is fully collected this Due, but its coin cannot be
                // swept or distributed until subsequent opening windows.
                assert_eq!(
                    sim.state.credit.loans[&ASSET].status,
                    credit::Status::Repaid
                );
                assert_eq!(sim.state.balance(PERSON, TOKEN), 1);
                assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
                assert!(b.recovery.iter().any(|r| matches!(r, Receipt::AssetsPending { receivables, uncollected_cash: 1, .. } if receivables.is_empty())));
            } else {
                assert_eq!(
                    recovery_claims::receivables(&sim.world, &sim.state, PERSON).unwrap()[0]
                        .remaining
                        .quantity,
                    2
                );
                let mut forged = sim.state.clone();
                let case = forged.credit.recovery.proceedings.get_mut(&1).unwrap();
                case.stage = Stage::Closed;
                case.closed = Some(3);
                let loan = forged.credit.loans.get_mut(&DEBT).unwrap();
                loan.principal = 0;
                loan.status = credit::Status::Discharged;
                assert!(Simulation::new(sim.world.clone(), forged, Backend::Reference).is_err());
            }
            let checkpoint = (sim.clone(), audit.clone());
            until(&mut sim, &mut audit, 8, Phase::Open);
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Closed
            );
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), principal);
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
            assert_eq!(
                sim.state.credit.loans[&DEBT].status,
                credit::Status::Discharged
            );
            let written_off: i32 = sim
                .ledger
                .iter()
                .filter_map(|b| b.credit.as_ref())
                .flat_map(|b| &b.recovery)
                .filter_map(|r| match r {
                    Receipt::WrittenOff {
                        loan: DEBT,
                        principal,
                        interest,
                        ..
                    } => Some(principal + interest),
                    _ => None,
                })
                .sum();
            assert_eq!(written_off, 10 - principal);
            let (mut resumed, mut resumed_audit) = checkpoint;
            until(&mut resumed, &mut resumed_audit, 8, Phase::Open);
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &resumed_audit)
            );
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn unperformed_receivable_defers_closure_without_inventing_estate_cash() {
    let mut sim = fixture(3, 4, false);
    let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
    until(&mut sim, &mut audit, 8, Phase::Open);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
    assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
    assert_eq!(sim.state.credit.loans[&DEBT].principal, 10);
    assert_eq!(
        recovery_claims::receivables(&sim.world, &sim.state, PERSON).unwrap()[0]
            .remaining
            .quantity,
        3
    );
    assert!(sim.ledger.iter().filter_map(|b| b.credit.as_ref()).flat_map(|b| &b.recovery)
        .any(|r| matches!(r, recovery::Receipt::AssetsPending { uncollected_cash: 0, receivables, .. } if !receivables.is_empty())));
}

#[test]
fn counterparty_discharge_resolves_the_asset_before_the_creditor_estate_closes() {
    let mut opening = fixture(3, 4, false);
    opening.world.agents.push(Agent {
        id: 100,
        name: "counterparty estate".into(),
    });
    opening.world.recovery.proceedings.push(ProceedingTerms {
        id: 2,
        debtor: BORROWER,
        estate: 100,
        authority: STATE_AGENT,
        denomination: TOKEN,
        opening_month: 4,
        earliest_close: 4,
        assets: vec![],
        discharge_deficiency: true,
    });
    let run = |backend| {
        let mut sim = opening.clone();
        sim.backend = backend;
        if matches!(backend, Backend::CubeCpu) {
            sim.world.recovery.proceedings.reverse();
        }
        let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
        until(&mut sim, &mut audit, 4, Phase::Acquire);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&2].stage,
            Stage::Closed
        );
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Active
        );
        assert_eq!(
            sim.state.credit.loans[&ASSET].status,
            credit::Status::Discharged
        );
        assert!(
            recovery_claims::receivables(&sim.world, &sim.state, PERSON)
                .unwrap()
                .is_empty()
        );
        until(&mut sim, &mut audit, 5, Phase::Acquire);
        assert_eq!(
            sim.state.credit.recovery.proceedings[&1].stage,
            Stage::Closed
        );
        assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 0);
        assert_eq!(
            sim.state.credit.loans[&DEBT].status,
            credit::Status::Discharged
        );
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn receivable_counterparty_filter_explains_deferred_estate_closure() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let mut sim = fixture(3, 4, false);
    let mut observer = Observer::new(
        vec![],
        "receivable-estate",
        Config {
            settlement: true,
            agents: [BORROWER].into(),
            ..Config::default()
        },
    )
    .unwrap();
    observer.run_months(&mut sim, 3).unwrap();
    let log = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert!(
        log.lines()
            .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .any(|r| r["kind"] == "estate_recovery"
                && r["detail"]["event"] == "AssetsPending"
                && r["detail"]["receivables"][0]["debtor"] == BORROWER
                && r["detail"]["receivables"][0]["outstanding"] == 3)
    );
}

#[test]
fn earned_wage_receivables_wait_for_real_payment_but_post_closure_work_is_not_backdated() {
    use economics_compute_smoke::{
        employment::{ArrearsPolicy, Terms},
        finance::ContractId,
    };
    let mut sim = fixture(1, 2, false);
    sim.world.lending.retain(|a| a.id != ASSET);
    sim.state.credit.loans.remove(&ASSET);
    let (baseline, _) = scenario::baseline();
    sim.world.participants = baseline.participants;
    sim.world.participants[0].needs.clear();
    sim.world.participants[0].capacity.quantity = 1;
    let labor = sim.world.participants[0].capacity.resource;
    for (id, month) in [(20, 2), (21, 6)] {
        sim.world.employment.push(Terms {
            id,
            employer: BORROWER,
            worker: PERSON,
            from: month,
            through: month,
            capacity: Amount::new(labor, 1),
            wage_per_unit: Amount::new(TOKEN, 2),
            on_arrears: ArrearsPolicy::Continue,
            rank: 0,
        });
    }
    sim.state.balances.insert((STATE_AGENT, TOKEN), 2);
    sim.world.lending.push(Advance {
        id: 12,
        debtor: BORROWER,
        month: 4,
        principal: 2,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: TOKEN,
            max_principal: 2,
            monthly_rate_bps: 0,
            term_months: 1,
            grace_months: 10,
        },
    });
    let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
    until(&mut sim, &mut audit, 3, Phase::Acquire);
    let assets = recovery_claims::receivables(&sim.world, &sim.state, PERSON).unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].contract, ContractId::Wages(20));
    assert_eq!(assets[0].remaining.quantity, 2);
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    until(&mut sim, &mut audit, 5, Phase::Open);
    assert_eq!(sim.state.balance(PERSON, TOKEN), 2);
    assert_eq!(sim.state.employment.earned[&(20, 2)].claim.outstanding(), 0);
    until(&mut sim, &mut audit, 7, Phase::Open);
    assert_eq!(sim.state.credit.recovery.proceedings[&1].closed, Some(6));
    assert_eq!(
        sim.state.credit.loans[&DEBT].status,
        credit::Status::Discharged
    );
    assert_eq!(sim.state.employment.earned[&(21, 6)].claim.outstanding(), 2);
    // Work earned after Due closure is a new asset, not evidence of premature closure.
    Simulation::new(sim.world, sim.state, Backend::CubeCpu).unwrap();
}

#[test]
fn receivable_sale_transfers_existing_claim_and_later_collections_into_separate_books() {
    use economics_compute_smoke::{
        offers,
        recovery::receivables::{Bid, Listing},
        settlement,
    };
    const BUYER: AgentId = 98;
    for household in [false, true] {
        for (funded, price) in [(true, 2), (false, 2), (true, 1)] {
            let mut opening = fixture_for(3, 6, true, household);
            opening.world.agents.push(Agent {
                id: BUYER,
                name: "claim investor".into(),
            });
            opening
                .state
                .balances
                .insert((BUYER, TOKEN), if funded { 2 } else { 0 });
            if household {
                opening.state.balances.insert((PERSON, TOKEN), 5);
            }
            opening.world.recovery.receivable_listings.push(Listing {
                coins_per_unit: 1,
                id: 1,
                proceeding: 1,
                loan: ASSET,
            });
            opening.world.recovery.receivable_bids.push(Bid {
                id: 1,
                listing: 1,
                buyer: BUYER,
                month: 3,
                price,
            });
            let sold = funded && price == 2;
            let run = |backend| {
                let mut sim =
                    Simulation::new(opening.world.clone(), opening.state.clone(), backend).unwrap();
                let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
                until(&mut sim, &mut audit, 3, Phase::Acquire);
                let owner = if household { HOME } else { PERSON };
                assert_eq!(sim.state.credit.loans[&ASSET].principal, 2);
                assert!(
                    offers::discover(&sim.world, &sim.state, BUYER)
                        .iter()
                        .any(|o| o.id == offers::Id::ReceivableLiquidationBid(1))
                );
                let request = offers::Request::new(offers::Id::ReceivableLiquidationBid(1), BUYER);
                let prepared = offers::prepare(&sim, std::slice::from_ref(&request));
                assert_eq!(prepared.is_ok(), sold);
                assert!(offers::prepare(&sim, &[request.clone(), request]).is_err());
                let checkpoint = (sim.clone(), audit.clone());
                audit.step(&mut sim).unwrap();
                assert_eq!(
                    sim.state.credit.loans[&ASSET].creditor,
                    if sold { BUYER } else { owner }
                );
                assert_eq!(sim.state.credit.loans[&ASSET].principal, 2);
                assert_eq!(sim.state.balance(ESTATE, TOKEN), if sold { 2 } else { 0 });
                assert_eq!(sim.state.credit.loans[&DEBT].principal, 10);
                if let Ok(batch) = prepared {
                    assert_eq!(&batch, sim.ledger.last().unwrap());
                    let mut forged = batch.clone();
                    forged
                        .credit
                        .as_mut()
                        .unwrap()
                        .after
                        .loans
                        .get_mut(&ASSET)
                        .unwrap()
                        .creditor = owner;
                    let mut unchanged = checkpoint.0.state.clone();
                    assert!(
                        settlement::commit(
                            &sim.world,
                            &mut unchanged,
                            &forged,
                            backend,
                            sim.effect_limit
                        )
                        .is_err()
                    );
                    assert_eq!(unchanged, checkpoint.0.state);
                }
                until(&mut sim, &mut audit, 10, Phase::Open);
                let (mut resumed, mut rb) = checkpoint;
                until(&mut resumed, &mut rb, 10, Phase::Open);
                assert_eq!(
                    (&sim.state, &sim.ledger, &audit),
                    (&resumed.state, &resumed.ledger, &rb)
                );
                assert_eq!(sim.state.credit.loans[&ASSET].principal, 0);
                assert_eq!(sim.state.balance(BUYER, TOKEN), if funded { 2 } else { 0 });
                assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 3);
                assert_eq!(
                    sim.state.credit.recovery.proceedings[&1].stage,
                    Stage::Closed
                );
                if household {
                    assert_eq!(sim.state.balance(PERSON, TOKEN), 5);
                }
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

#[test]
fn receivable_and_inventory_lots_compete_for_one_opening_cash_budget() {
    use economics_compute_smoke::{
        financial_reporting::Opening,
        recovery::{inventory, receivables},
        scenario::GRAIN,
    };
    const BUYER: AgentId = 98;
    for coins in [2, 3] {
        let mut opening = fixture(3, 6, true);
        opening.world.agents.push(Agent {
            id: BUYER,
            name: "mixed asset buyer".into(),
        });
        opening.state.balances.insert((BUYER, TOKEN), coins);
        opening.state.balances.insert((PERSON, GRAIN), 1);
        opening
            .world
            .recovery
            .receivable_listings
            .push(receivables::Listing {
                coins_per_unit: 1,
                id: 1,
                proceeding: 1,
                loan: ASSET,
            });
        opening
            .world
            .recovery
            .receivable_bids
            .push(receivables::Bid {
                id: 1,
                listing: 1,
                buyer: BUYER,
                month: 3,
                price: 2,
            });
        opening
            .world
            .recovery
            .inventory_listings
            .push(inventory::Listing {
                id: 1,
                proceeding: 1,
                goods: Amount::new(GRAIN, 1),
                minimum_price: 1,
            });
        opening.world.recovery.inventory_bids.push(inventory::Bid {
            id: 1,
            listing: 1,
            buyer: BUYER,
            month: 3,
            price: 1,
        });
        let run = |backend| {
            let mut sim =
                Simulation::new(opening.world.clone(), opening.state.clone(), backend).unwrap();
            let mut audit = Audit::with_opening(
                &sim.world,
                &sim.state,
                TOKEN,
                Opening {
                    inventory: [((PERSON, GRAIN), 1)].into(),
                    ..Default::default()
                },
            )
            .unwrap();
            until(&mut sim, &mut audit, 3, Phase::Acquire);
            let (mut resumed, mut rb) = (sim.clone(), audit.clone());
            audit.step(&mut sim).unwrap();
            rb.step(&mut resumed).unwrap();
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &rb)
            );
            assert_eq!(sim.state.credit.loans[&ASSET].creditor, BUYER);
            assert_eq!(sim.state.balance(BUYER, TOKEN), 0);
            assert_eq!(sim.state.balance(BUYER, GRAIN), coins - 2);
            assert_eq!(sim.state.balance(ESTATE, TOKEN), coins);
            let sold = sim
                .ledger
                .last()
                .unwrap()
                .credit
                .as_ref()
                .unwrap()
                .recovery
                .iter()
                .any(|r| matches!(r, Receipt::InventorySold { .. }));
            assert_eq!(sold, coins == 3);
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn assigned_secured_claim_keeps_reserved_proceeds_across_two_household_and_person_estates() {
    secured_assignment(false, false);
}

#[test]
fn household_claim_assignment_and_guarantee_subrogation_share_one_collateral_recovery() {
    secured_assignment(true, false);
}

#[test]
fn shared_custodian_separates_assigned_household_claims_and_guarantor_liens() {
    secured_assignment(false, true);
    secured_assignment(true, true);
}

fn secured_assignment(guaranteed: bool, shared_custody: bool) {
    use economics_compute_smoke::{
        financial_reporting::Opening,
        household_governance::Governance,
        households::{self, Agreement, dissolution},
        recovery::receivables,
        scenario::PLOT,
    };
    const INVESTOR: AgentId = 98;
    const PROPERTY_BUYER: AgentId = 100;
    const BORROWER_ESTATE: AgentId = 101;
    const GUARANTOR: AgentId = 102;
    for funded in [false, true] {
        for discharge in [false, true] {
            for sale_month in [3, 4] {
                let (mut w, mut s) = scenario::baseline();
                w.participants[0].needs.clear();
                w.participants[0].capacity.quantity = 0;
                w.definitions.clear();
                w.rights.clear();
                w.agreements.clear();
                w.resources.push(Resource {
                    id: TOKEN,
                    name: "coin".into(),
                    kind: ResourceKind::Stock,
                });
                for id in [BORROWER, INVESTOR, ESTATE, PROPERTY_BUYER, BORROWER_ESTATE] {
                    w.agents.push(Agent {
                        id,
                        name: format!("participant {id}"),
                    });
                }
                w.assets.iter_mut().find(|a| a.id == PLOT).unwrap().owner = BORROWER;
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
                let loan = |id, creditor, debtor| Advance {
                    id,
                    debtor,
                    month: 1,
                    principal: 10,
                    collateral: None,
                    priority: 0,
                    terms: LoanOffer {
                        creditor,
                        denomination: TOKEN,
                        max_principal: 10,
                        monthly_rate_bps: 0,
                        term_months: 1,
                        grace_months: 10,
                    },
                };
                w.lending = vec![loan(DEBT, STATE_AGENT, HOME), loan(ASSET, HOME, BORROWER)];
                w.lending[1].collateral = Some(credit::Collateral {
                    asset: PLOT,
                    pledged: true,
                    priority: 5,
                    settlement: credit::CollateralSettlement::AuthorizedLiquidation,
                });
                if guaranteed {
                    w.agents.push(Agent {
                        id: GUARANTOR,
                        name: "collateral guarantor".into(),
                    });
                    w.recovery.guarantees.push(recovery::Guarantee {
                        id: 1,
                        follows_assignment: true,
                        tender: economics_compute_smoke::recovery::GuaranteeTender::Native,
                        security: recovery::RecourseSecurity::InheritLiquidationLien,
                        claim: recovery::GuaranteedClaim::Loan(ASSET),
                        guarantor: GUARANTOR,
                        cap: 6,
                        from: 4,
                        through: 4,
                        delay_months: 0,
                        recourse: 200,
                        priority: 0,
                    });
                }
                for (id, debtor, estate) in [
                    (1, HOME, ESTATE),
                    (
                        2,
                        BORROWER,
                        if shared_custody {
                            ESTATE
                        } else {
                            BORROWER_ESTATE
                        },
                    ),
                ] {
                    w.recovery.proceedings.push(ProceedingTerms {
                        id,
                        debtor,
                        estate,
                        authority: STATE_AGENT,
                        denomination: TOKEN,
                        opening_month: 3,
                        earliest_close: 4,
                        assets: if id == 2 {
                            vec![recovery::Listing {
                                asset: PLOT,
                                minimum_price: 4,
                            }]
                        } else {
                            vec![]
                        },
                        discharge_deficiency: id == 1 || discharge,
                    });
                }
                w.recovery.bids.push(recovery::Bid {
                    id: 1,
                    proceeding: 2,
                    buyer: PROPERTY_BUYER,
                    asset: PLOT,
                    month: sale_month,
                    price: 4,
                });
                w.recovery.receivable_listings.push(receivables::Listing {
                    coins_per_unit: 1,
                    id: 1,
                    proceeding: 1,
                    loan: ASSET,
                });
                w.recovery.receivable_bids.push(receivables::Bid {
                    id: 1,
                    listing: 1,
                    buyer: INVESTOR,
                    month: 3,
                    price: 10,
                });
                s.balances = [
                    ((STATE_AGENT, TOKEN), 10),
                    ((HOME, TOKEN), 10),
                    ((PERSON, TOKEN), 5),
                    ((INVESTOR, TOKEN), if funded { 10 } else { 9 }),
                    ((PROPERTY_BUYER, TOKEN), 4),
                ]
                .into();
                if guaranteed {
                    s.balances.insert((GUARANTOR, TOKEN), 6);
                }
                let run = |backend| {
                    let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                    sim.run_months(1).unwrap();
                    // Both observed losses predate the reporting interval.
                    sim.state.balances.insert((HOME, TOKEN), 0);
                    sim.state.balances.insert((BORROWER, TOKEN), 0);
                    dissolution::request(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
                    let mut audit = Audit::with_opening(
                        &sim.world,
                        &sim.state,
                        TOKEN,
                        Opening {
                            assets: [(PLOT, 6)].into(),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                    until(&mut sim, &mut audit, 3, Phase::Acquire);
                    let (mut resumed, mut rb) = (sim.clone(), audit.clone());
                    audit.step(&mut sim).unwrap();
                    assert_eq!(
                        sim.state.credit.loans[&ASSET].creditor,
                        if funded { INVESTOR } else { HOME }
                    );
                    assert_eq!(
                        sim.state.credit.recovery.proceedings[&2]
                            .secured
                            .get(&ASSET)
                            .copied()
                            .unwrap_or(0),
                        if sale_month == 3 { 4 } else { 0 }
                    );
                    assert_eq!(
                        sim.state.balance(ESTATE, TOKEN),
                        (if funded { 10 } else { 0 })
                            + if shared_custody && sale_month == 3 {
                                4
                            } else {
                                0
                            }
                    );
                    assert_eq!(
                        sim.state.credit.recovery.proceedings[&1].cash,
                        if funded { 10 } else { 0 }
                    );
                    assert_eq!(
                        sim.state.credit.recovery.proceedings[&2].cash,
                        if sale_month == 3 { 4 } else { 0 }
                    );
                    assert_eq!(
                        credit::owner(&sim.world, &sim.state, PLOT),
                        Some(if sale_month == 3 {
                            PROPERTY_BUYER
                        } else {
                            BORROWER
                        })
                    );
                    until(&mut sim, &mut audit, 8, Phase::Open);
                    until(&mut resumed, &mut rb, 8, Phase::Open);
                    assert_eq!(
                        (&sim.state, &sim.ledger, &audit),
                        (&resumed.state, &resumed.ledger, &rb)
                    );
                    let recovered = if guaranteed {
                        if sale_month == 3 { 6 } else { 10 }
                    } else {
                        4
                    };
                    assert_eq!(
                        sim.state.balance(INVESTOR, TOKEN),
                        if funded { recovered } else { 9 }
                    );
                    assert_eq!(
                        sim.state.balance(STATE_AGENT, TOKEN),
                        if funded { 10 } else { recovered }
                    );
                    assert_eq!(sim.state.balance(PERSON, TOKEN), 5);
                    assert_eq!(
                        sim.state.credit.loans[&ASSET].principal,
                        if discharge { 0 } else { 10 - recovered }
                    );
                    if guaranteed {
                        let recourse = &sim.state.credit.loans[&200];
                        assert_eq!(recourse.creditor, GUARANTOR);
                        assert_eq!(recourse.debtor, BORROWER);
                        assert_eq!(recourse.collateral.as_ref().unwrap().asset, PLOT);
                        assert_eq!(recourse.collateral.as_ref().unwrap().priority, 5);
                        let recovered_recourse = if sale_month == 3 { 4 } else { 0 };
                        assert_eq!(sim.state.balance(GUARANTOR, TOKEN), recovered_recourse);
                        assert_eq!(
                            recourse.principal,
                            if discharge { 0 } else { 6 - recovered_recourse }
                        );
                    }
                    let cleared = funded || discharge || recovered == 10;
                    assert_eq!(
                        dissolution::finish(&mut sim.world, &sim.state, HOME, PERSON).is_ok(),
                        cleared
                    );
                    assert_eq!(
                        dissolution::finish(&mut resumed.world, &resumed.state, HOME, PERSON)
                            .is_ok(),
                        cleared
                    );
                    until(&mut sim, &mut audit, 9, Phase::Open);
                    until(&mut resumed, &mut rb, 9, Phase::Open);
                    assert_eq!(
                        (&sim.state, &sim.ledger, &audit),
                        (&resumed.state, &resumed.ledger, &rb)
                    );
                    assert_eq!(
                        households::membership::current(&sim.world.households[0]).is_empty(),
                        cleared
                    );
                    (sim.state, sim.ledger, audit)
                };
                assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
            }
        }
    }
}

#[test]
fn explicit_guarantee_benefit_follows_assignment_and_pays_the_current_holder() {
    use economics_compute_smoke::{
        agreements,
        recovery::{Guarantee, GuaranteedClaim, RecourseSecurity, receivables},
    };
    const BUYER: AgentId = 98;
    const GUARANTOR: AgentId = 100;
    for household in [false, true] {
        let mut opening = fixture_for(3, 6, false, household);
        for id in [BUYER, GUARANTOR] {
            opening.world.agents.push(Agent {
                id,
                name: format!("claim party {id}"),
            });
            opening.state.balances.insert((id, TOKEN), 3);
        }
        opening
            .world
            .recovery
            .receivable_listings
            .push(receivables::Listing {
                coins_per_unit: 1,
                id: 1,
                proceeding: 1,
                loan: ASSET,
            });
        opening
            .world
            .recovery
            .receivable_bids
            .push(receivables::Bid {
                id: 1,
                listing: 1,
                buyer: BUYER,
                month: 3,
                price: 3,
            });
        opening.world.recovery.guarantees.push(Guarantee {
            follows_assignment: true,
            tender: economics_compute_smoke::recovery::GuaranteeTender::Native,
            security: RecourseSecurity::Unsecured,
            id: 1,
            claim: GuaranteedClaim::Loan(ASSET),
            guarantor: GUARANTOR,
            cap: 3,
            from: 4,
            through: 8,
            delay_months: 0,
            recourse: 200,
            priority: 0,
        });
        let mut invalid = opening.world.clone();
        invalid.recovery.guarantees[0].follows_assignment = false;
        assert!(Simulation::new(invalid, opening.state.clone(), Backend::Reference).is_err());
        let mut invalid = opening.world.clone();
        invalid.recovery.receivable_bids[0].buyer = GUARANTOR;
        assert!(Simulation::new(invalid, opening.state.clone(), Backend::Reference).is_err());
        let run = |backend| {
            let mut sim =
                Simulation::new(opening.world.clone(), opening.state.clone(), backend).unwrap();
            let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
            until(&mut sim, &mut audit, 3, Phase::Acquire);
            let offers = receivables::discover(&sim.world, &sim.state, BUYER);
            assert_eq!(
                offers[0].guarantees[0].terms,
                sim.world.recovery.guarantees[0]
            );
            assert_eq!(offers[0].guarantees[0].remaining_cap, 3);
            assert!(receivables::discover(&sim.world, &sim.state, GUARANTOR).is_empty());
            audit.step(&mut sim).unwrap();
            until(&mut sim, &mut audit, 4, Phase::Open);
            let views = agreements::for_agent(&sim.world, &sim.state, BUYER).unwrap();
            assert!(
                views
                    .iter()
                    .any(|v| matches!(v, agreements::View::Guarantee(g) if g.creditor == BUYER))
            );
            let (mut resumed, mut rb) = (sim.clone(), audit.clone());
            until(&mut sim, &mut audit, 9, Phase::Open);
            until(&mut resumed, &mut rb, 9, Phase::Open);
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &rb)
            );
            assert_eq!(sim.state.balance(BUYER, TOKEN), 3);
            assert_eq!(sim.state.balance(GUARANTOR, TOKEN), 0);
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 3);
            assert_eq!(sim.state.credit.loans[&ASSET].principal, 0);
            assert_eq!(sim.state.credit.loans[&200].principal, 3);
            assert_eq!(sim.state.credit.loans[&200].creditor, GUARANTOR);
            assert_eq!(sim.state.credit.loans[&200].debtor, BORROWER);
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn receivable_discovery_distinguishes_posted_guarantees_from_accepted_remaining_cover() {
    use economics_compute_smoke::recovery::{
        Guarantee, GuaranteeTender, GuaranteedClaim, RecourseSecurity, admission, receivables,
    };
    const BUYER: AgentId = 98;
    const GUARANTOR: AgentId = 100;
    for (posted, accepted, funds) in [
        (false, true, 1),
        (false, true, 2),
        (true, false, 1),
        (true, true, 1),
        (true, true, 2),
    ] {
        let mut opening = fixture_for(3, 3, false, true);
        for id in [BUYER, GUARANTOR] {
            opening.world.agents.push(Agent {
                id,
                name: format!("claim party {id}"),
            });
            opening.state.balances.insert((id, TOKEN), funds);
        }
        opening
            .world
            .recovery
            .receivable_listings
            .push(receivables::Listing {
                coins_per_unit: 1,
                id: 1,
                proceeding: 1,
                loan: ASSET,
            });
        opening.world.recovery.guarantees.push(Guarantee {
            follows_assignment: true,
            tender: GuaranteeTender::Native,
            security: RecourseSecurity::Unsecured,
            id: 1,
            claim: GuaranteedClaim::Loan(ASSET),
            guarantor: GUARANTOR,
            cap: 2,
            from: 2,
            through: 3,
            delay_months: 0,
            recourse: 200,
            priority: 0,
        });
        if posted {
            opening.world.recovery.posted_guarantees.insert(1);
            if accepted {
                opening
                    .world
                    .recovery
                    .guarantee_applications
                    .push(admission::Application {
                        guarantee: 1,
                        month: 2,
                    });
            }
        }
        let run = |backend| {
            let mut sim =
                Simulation::new(opening.world.clone(), opening.state.clone(), backend).unwrap();
            let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
            until(&mut sim, &mut audit, 3, Phase::Acquire);
            let before = sim.state.clone();
            let offers = receivables::discover(&sim.world, &sim.state, BUYER);
            assert_eq!(offers.len(), 1);
            assert_eq!(
                !offers[0].guarantees.is_empty(),
                accepted && funds < 2,
                "posted={posted} accepted={accepted} funds={funds} paid={:?} loans={:?}",
                sim.state.credit.recovery.paid_guarantees,
                sim.state.credit.loans
            );
            if accepted && funds < 2 {
                let coverage = &offers[0].guarantees[0];
                assert_eq!(coverage.terms, sim.world.recovery.guarantees[0]);
                assert_eq!(coverage.accepted_month, 2);
                assert_eq!(coverage.remaining_cap, 1);
                assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], 1);
                assert_eq!(sim.state.balance(GUARANTOR, TOKEN), 0);
            }
            if accepted {
                assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], funds);
            }
            assert_eq!(sim.state, before);
            let prefix = sim.ledger.len();
            let mut resumed =
                Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
            let mut ra = audit.clone();
            until(&mut sim, &mut audit, 4, Phase::Acquire);
            until(&mut resumed, &mut ra, 4, Phase::Acquire);
            let expired = receivables::discover(&sim.world, &sim.state, BUYER);
            assert_eq!(expired.len(), 1);
            assert!(expired[0].guarantees.is_empty());
            assert_eq!(
                (&sim.state, &sim.ledger[prefix..], &audit),
                (&resumed.state, &resumed.ledger[..], &ra)
            );
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn assignment_retains_prior_partial_loss_and_requires_the_new_creditors_relief_consent() {
    relieved_assignment(false);
}

#[test]
fn household_wind_down_preserves_its_partial_loss_after_selling_the_surviving_receivable() {
    relieved_assignment(true);
}

fn relieved_assignment(household: bool) {
    use economics_compute_smoke::{
        accounting::Account,
        claim_relief::{Action, Terms},
        finance::ContractId,
        recovery::receivables,
    };
    const BUYER: AgentId = 98;
    const BORROWER_ESTATE: AgentId = 96;
    for current_creditor in [false, true] {
        let mut opening = fixture_for(4, 1, false, household);
        let seller = if household { HOME } else { PERSON };
        opening.world.agents.extend([
            Agent {
                id: BUYER,
                name: "claim buyer".into(),
            },
            Agent {
                id: BORROWER_ESTATE,
                name: "borrower custodian".into(),
            },
        ]);
        opening.state.balances.insert((BUYER, TOKEN), 3);
        opening.world.recovery.proceedings.push(ProceedingTerms {
            id: 2,
            debtor: BORROWER,
            estate: BORROWER_ESTATE,
            authority: STATE_AGENT,
            denomination: TOKEN,
            opening_month: 3,
            earliest_close: 6,
            assets: vec![],
            discharge_deficiency: false,
        });
        for (id, month, creditor, expected_remaining, quantity) in [
            (1, 4, seller, 4, 1),
            (2, 5, if current_creditor { BUYER } else { seller }, 3, 3),
        ] {
            opening.world.recovery.claim_relief.push(Terms {
                id,
                proceeding: 2,
                contract: ContractId::Loan(ASSET),
                original_due: 2,
                debtor: BORROWER,
                creditor,
                month,
                expected_due: 2,
                expected_remaining,
                action: Action::WriteOff { quantity },
            });
        }
        opening
            .world
            .recovery
            .receivable_listings
            .push(receivables::Listing {
                id: 1,
                proceeding: 1,
                loan: ASSET,
                coins_per_unit: 1,
            });
        opening
            .world
            .recovery
            .receivable_bids
            .push(receivables::Bid {
                id: 1,
                listing: 1,
                buyer: BUYER,
                month: 4,
                price: 3,
            });
        let run = |backend| {
            let mut sim =
                Simulation::new(opening.world.clone(), opening.state.clone(), backend).unwrap();
            let mut a = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
            until(&mut sim, &mut a, 4, Phase::Acquire);
            assert_eq!(sim.state.credit.loans[&ASSET].principal, 3);
            assert_eq!(a.book().balances()[&(seller, Account::CreditLoss)], 1);
            if household {
                assert!(
                    economics_compute_smoke::households::dissolution::finish(
                        &mut sim.world,
                        &sim.state,
                        HOME,
                        PERSON
                    )
                    .is_err()
                );
            }
            a.step(&mut sim).unwrap();
            assert_eq!(sim.state.credit.loans[&ASSET].creditor, BUYER);
            let first = sim.state.credit.recovery.loan_writeoffs[&ASSET][0].clone();
            assert_eq!(first.terms.creditor, seller);
            let mut forged_world = sim.world.clone();
            let mut forged_state = sim.state.clone();
            forged_world.recovery.claim_relief[0].creditor = BUYER;
            forged_state
                .credit
                .recovery
                .loan_writeoffs
                .get_mut(&ASSET)
                .unwrap()[0]
                .terms
                .creditor = BUYER;
            assert!(Simulation::new(forged_world, forged_state, backend).is_err());
            let (saved, mut ra) = (sim.clone(), a.clone());
            until(&mut sim, &mut a, 7, Phase::Open);
            assert_eq!(sim.state.credit.recovery.loan_writeoffs[&ASSET][0], first);
            assert_eq!(
                sim.state.credit.loans[&ASSET].principal,
                if current_creditor { 0 } else { 3 }
            );
            assert_eq!(
                sim.state.credit.recovery.proceedings[&2].stage,
                Stage::Closed
            );
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Closed
            );
            let b = a.book().balances();
            assert_eq!(b[&(seller, Account::CreditLoss)], 1);
            assert_eq!(
                b.get(&(BUYER, Account::CreditLoss)).copied().unwrap_or(0),
                if current_creditor { 3 } else { 0 }
            );
            assert_eq!(
                b[&(BORROWER, Account::DebtRelief)],
                if current_creditor { -4 } else { -1 }
            );
            assert_eq!(sim.state.balance(STATE_AGENT, TOKEN), 3);
            assert_eq!(sim.state.balance(BUYER, TOKEN), 0);
            let mut resumed = Simulation::new(saved.world, saved.state, backend).unwrap();
            until(&mut resumed, &mut ra, 7, Phase::Open);
            assert_eq!((&sim.state, &a), (&resumed.state, &ra));
            if household {
                use economics_compute_smoke::households::dissolution as d;
                assert!(d::blockers(&sim.world, &sim.state, HOME).is_empty());
                assert_eq!(sim.state.balance(PERSON, TOKEN), 0);
                assert_eq!(
                    a.book()
                        .balances()
                        .get(&(PERSON, Account::CreditLoss))
                        .copied()
                        .unwrap_or(0),
                    0
                );
                assert_eq!(a.book().balances()[&(HOME, Account::DebtRelief)], -7);
                d::finish(&mut sim.world, &sim.state, HOME, PERSON).unwrap();
                d::finish(&mut resumed.world, &resumed.state, HOME, PERSON).unwrap();
                until(&mut sim, &mut a, 8, Phase::Open);
                until(&mut resumed, &mut ra, 8, Phase::Open);
                assert_eq!(
                    (&sim.world, &sim.state, &a),
                    (&resumed.world, &resumed.state, &ra)
                );
                assert_eq!(sim.state.credit.loans[&ASSET].creditor, BUYER);
                assert_eq!(
                    sim.state.credit.loans[&ASSET].principal,
                    if current_creditor { 0 } else { 3 }
                );
            }
            (sim.world, sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn priced_receivable_sales_keep_face_claims_and_release_acquisition_cost_on_collection_or_loss() {
    priced_receivables(false);
}

#[test]
fn household_priced_claim_sales_keep_losses_separate_through_wind_down_and_counterparty_relief() {
    priced_receivables(true);
}

fn priced_receivables(household: bool) {
    use economics_compute_smoke::{
        accounting::Account,
        claim_relief::{Action, Terms},
        finance::ContractId,
        recovery::receivables::{Bid, Listing},
    };
    const BUYER: AgentId = 98;
    const BORROWER_ESTATE: AgentId = 100;
    for writeoff in [false, true] {
        for price in [1, 2, 3] {
            let mut opening = fixture_for(3, 6, !writeoff, household);
            let seller = if household { HOME } else { PERSON };
            opening.world.agents.push(Agent {
                id: BUYER,
                name: "claim investor".into(),
            });
            opening.state.balances.insert((BUYER, TOKEN), price);
            opening.world.recovery.receivable_listings.push(Listing {
                id: 1,
                proceeding: 1,
                loan: ASSET,
                coins_per_unit: 1,
            });
            opening.world.recovery.receivable_price_floors.insert(1, 1);
            opening.world.recovery.receivable_bids.push(Bid {
                id: 1,
                listing: 1,
                buyer: BUYER,
                month: 3,
                price,
            });
            let face = if writeoff { 3 } else { 2 };
            if writeoff {
                opening.world.agents.push(Agent {
                    id: BORROWER_ESTATE,
                    name: "borrower estate".into(),
                });
                opening.world.recovery.proceedings.push(ProceedingTerms {
                    id: 2,
                    debtor: BORROWER,
                    estate: BORROWER_ESTATE,
                    authority: STATE_AGENT,
                    denomination: TOKEN,
                    opening_month: 4,
                    earliest_close: 4,
                    assets: vec![],
                    discharge_deficiency: false,
                });
                opening.world.recovery.claim_relief.push(Terms {
                    id: 1,
                    proceeding: 2,
                    contract: ContractId::Loan(ASSET),
                    original_due: 2,
                    debtor: BORROWER,
                    creditor: BUYER,
                    month: 4,
                    expected_due: 2,
                    expected_remaining: face,
                    action: Action::WriteOff { quantity: face },
                });
            }
            let run = |backend| {
                let mut sim =
                    Simulation::new(opening.world.clone(), opening.state.clone(), backend).unwrap();
                let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
                until(&mut sim, &mut audit, 3, Phase::Acquire);
                audit.step(&mut sim).unwrap();
                assert_eq!(sim.state.credit.loans[&ASSET].principal, face);
                assert_eq!(sim.state.credit.loans[&ASSET].creditor, BUYER);
                let balance = |a: &Audit, who, account| {
                    a.book()
                        .balances()
                        .get(&(who, account))
                        .copied()
                        .unwrap_or(0)
                };
                assert_eq!(
                    balance(&audit, BUYER, Account::LoanReceivable(ASSET)),
                    i128::from(face)
                );
                assert_eq!(
                    balance(&audit, BUYER, Account::LoanBasisAdjustment(ASSET)),
                    i128::from(price - face)
                );
                assert_eq!(
                    balance(&audit, BORROWER, Account::LoanPayable(ASSET)),
                    -i128::from(face)
                );
                assert_eq!(sim.state.balance(ESTATE, TOKEN), price);
                let (saved, mut ra) = (sim.clone(), audit.clone());
                let mut forged = sim.state.clone();
                forged
                    .credit
                    .recovery
                    .assignments
                    .get_mut(&ASSET)
                    .unwrap()
                    .price += 1;
                assert!(Simulation::new(sim.world.clone(), forged, backend).is_err());
                if household {
                    until(&mut sim, &mut audit, 6, Phase::Open);
                    economics_compute_smoke::households::dissolution::finish(
                        &mut sim.world,
                        &sim.state,
                        HOME,
                        PERSON,
                    )
                    .unwrap();
                    assert_eq!(sim.state.credit.loans[&ASSET].creditor, BUYER);
                    assert_eq!(
                        sim.state.credit.loans[&ASSET].principal,
                        if writeoff { 0 } else { 1 }
                    );
                }
                until(&mut sim, &mut audit, 10, Phase::Open);
                assert_eq!(sim.state.credit.loans[&ASSET].principal, 0);
                assert_eq!(
                    balance(&audit, BUYER, Account::LoanBasisAdjustment(ASSET)),
                    0
                );
                assert_eq!(
                    balance(&audit, BUYER, Account::CreditLoss),
                    if writeoff { i128::from(price) } else { 0 }
                );
                assert_eq!(
                    sim.state.balance(BUYER, TOKEN),
                    if writeoff { 0 } else { face }
                );
                if !writeoff {
                    assert_eq!(
                        balance(&audit, BUYER, Account::SettlementGain)
                            + balance(&audit, BUYER, Account::SettlementLoss),
                        i128::from(price - face)
                    );
                }
                assert_eq!(
                    balance(&audit, seller, Account::DisposalGain)
                        + balance(&audit, seller, Account::DisposalLoss),
                    i128::from(face - price)
                );
                let mut resumed = Simulation::new(saved.world, saved.state, backend).unwrap();
                if household {
                    until(&mut resumed, &mut ra, 6, Phase::Open);
                    economics_compute_smoke::households::dissolution::finish(
                        &mut resumed.world,
                        &resumed.state,
                        HOME,
                        PERSON,
                    )
                    .unwrap();
                }
                until(&mut resumed, &mut ra, 10, Phase::Open);
                assert_eq!((&sim.state, &audit), (&resumed.state, &ra));
                if household {
                    for (branch, book) in [(&mut sim, &mut audit), (&mut resumed, &mut ra)] {
                        assert_eq!(branch.state.balance(PERSON, TOKEN), 0);
                        assert_eq!(balance(book, PERSON, Account::DisposalLoss), 0);
                        assert_eq!(balance(book, PERSON, Account::DisposalGain), 0);
                        until(branch, book, 11, Phase::Open);
                    }
                    assert_eq!((&sim.state, &audit), (&resumed.state, &ra));
                }
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

#[test]
fn estate_claim_bids_use_the_highest_funded_price_above_the_agreed_floor() {
    use economics_compute_smoke::recovery::receivables::{Bid, Listing};
    const LOW: AgentId = 98;
    const HIGH: AgentId = 96;
    for (high_funds, floor, winner, price) in [(3, 1, HIGH, 3), (2, 1, LOW, 1), (2, 2, PERSON, 0)] {
        let mut opening = fixture(3, 6, true);
        for (id, funds) in [(LOW, 1), (HIGH, high_funds)] {
            opening.world.agents.push(Agent {
                id,
                name: format!("bidder {id}"),
            });
            opening.state.balances.insert((id, TOKEN), funds);
        }
        opening.world.recovery.receivable_listings.push(Listing {
            id: 1,
            proceeding: 1,
            loan: ASSET,
            coins_per_unit: 1,
        });
        opening
            .world
            .recovery
            .receivable_price_floors
            .insert(1, floor);
        opening.world.recovery.receivable_bids = vec![
            Bid {
                id: 1,
                listing: 1,
                buyer: LOW,
                month: 3,
                price: 1,
            },
            Bid {
                id: 2,
                listing: 1,
                buyer: HIGH,
                month: 3,
                price: 3,
            },
        ];
        let run = |backend, reverse| {
            let mut world = opening.world.clone();
            if reverse {
                world.recovery.receivable_bids.reverse();
            }
            let mut sim = Simulation::new(world, opening.state.clone(), backend).unwrap();
            let mut audit = Audit::new(&sim.world, &sim.state, TOKEN).unwrap();
            until(&mut sim, &mut audit, 3, Phase::Acquire);
            audit.step(&mut sim).unwrap();
            assert_eq!(sim.state.credit.loans[&ASSET].creditor, winner);
            assert_eq!(sim.state.balance(ESTATE, TOKEN), price);
            assert_eq!(
                sim.state.balance(LOW, TOKEN),
                if winner == LOW { 0 } else { 1 }
            );
            assert_eq!(
                sim.state.balance(HIGH, TOKEN),
                if winner == HIGH { 0 } else { high_funds }
            );
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference, false), run(Backend::CubeCpu, true));
    }
}

#[test]
fn loan_cost_adjustments_cannot_hide_negative_carrying_value_or_exist_without_a_claim() {
    use economics_compute_smoke::accounting::{Account, Book};
    let positions = |face, adjustment| {
        [
            ((PERSON, Account::LoanReceivable(ASSET)), face),
            ((PERSON, Account::LoanBasisAdjustment(ASSET)), adjustment),
        ]
        .into()
    };
    assert!(Book::open(TOKEN, positions(4, -3)).is_ok());
    assert!(Book::open(TOKEN, positions(4, -5)).is_err());
    assert!(Book::open(TOKEN, positions(0, 1)).is_err());
    assert!(Book::open(TOKEN, positions(0, -1)).is_err());
    assert!(Book::open(TOKEN, [((PERSON, Account::Cash), -1)].into()).is_err());
}
