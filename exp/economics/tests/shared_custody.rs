use economics_compute_smoke::{
    accounting::Account as LedgerAccount,
    commitments::Agreement as Land,
    compute::Backend,
    credit::{Advance, LoanOffer},
    dues_accounting::Valuation,
    employment::{ArrearsPolicy, Terms},
    financial_reporting::{Audit, Opening},
    household_governance::Governance,
    households::{self, Agreement},
    model::*,
    recovery::{ProceedingTerms, Stage, inventory},
    scenario::*,
    settlement,
    simulation::Simulation,
};
const SECOND: AgentId = 98;
const BUYER: AgentId = 97;
const CUSTODIAN: AgentId = 99;
const HOME: AgentId = 10000;

#[test]
fn shared_custodian_preserves_each_estates_opening_cash_and_beneficial_owner() {
    for household in [false, true] {
        let mut canonical = None;
        for reversed in [false, true] {
            let (mut w, mut s) = baseline();
            s.month = 12;
            s.balances.clear();
            w.definitions.clear();
            w.assets.clear();
            w.rights.clear();
            w.agreements.clear();
            w.participants[0].needs.clear();
            w.participants[0].capacity.quantity = 1;
            w.resources.push(Resource {
                id: TOKEN,
                name: "coin".into(),
                kind: ResourceKind::Stock,
            });
            for id in [SECOND, BUYER, CUSTODIAN] {
                w.agents.push(Agent {
                    id,
                    name: format!("agent {id}"),
                });
            }
            w.participants.push(Participant {
                agent: SECOND,
                capacity: Amount::new(LABOR, 0),
                needs: vec![],
            });
            let seller = if household {
                let mut governance = Governance::contributed(SECOND);
                governance.constitution.allow_dissolution = true;
                households::form(
                    &mut w,
                    &s,
                    Agreement {
                        id: 1,
                        agent: HOME,
                        adults: vec![SECOND],
                        governance,
                        formed: 12,
                        dwelling_process: None,
                        admission: None,
                        membership: vec![],
                        asset_sales: vec![],
                        equipment_retirements: vec![],
                        support: vec![],
                    },
                )
                .unwrap();
                s.balances.insert((SECOND, TOKEN), 5);
                HOME
            } else {
                SECOND
            };
            for (id, debtor) in [(1, PERSON), (2, seller)] {
                w.lending.push(Advance {
                    id,
                    debtor,
                    principal: 10,
                    month: 12,
                    collateral: None,
                    priority: 0,
                    terms: LoanOffer {
                        creditor: STATE_AGENT,
                        denomination: TOKEN,
                        max_principal: 10,
                        monthly_rate_bps: 10000,
                        term_months: 1,
                        grace_months: 12,
                    },
                });
                w.assets.push(Asset {
                    id,
                    owner: STATE_AGENT,
                    kind: 1,
                });
                w.rights.push(UseRight {
                    id,
                    holder: debtor,
                    asset: id,
                    from: 1,
                    through: 24,
                    output_owner: debtor,
                });
                w.agreements.push(Land {
                    id,
                    right: id,
                    creditor: STATE_AGENT,
                    debtor,
                    activated: 1,
                    payment: Amount::new(TOKEN, 2),
                });
                w.recovery.proceedings.push(ProceedingTerms {
                    id,
                    debtor,
                    authority: STATE_AGENT,
                    estate: CUSTODIAN,
                    denomination: TOKEN,
                    opening_month: 14,
                    earliest_close: 15,
                    assets: vec![],
                    discharge_deficiency: false,
                });
            }
            s.balances.insert((STATE_AGENT, TOKEN), 20);
            s.balances.insert((BUYER, TOKEN), 26);
            s.balances.insert((seller, SEED), 1);
            w.recovery.inventory_listings.push(inventory::Listing {
                id: 1,
                proceeding: 2,
                goods: Amount::new(SEED, 1),
                minimum_price: 14,
            });
            w.recovery.inventory_bids.push(inventory::Bid {
                id: 1,
                listing: 1,
                buyer: BUYER,
                month: 14,
                price: 14,
            });
            w.employment.push(Terms {
                id: 1,
                employer: BUYER,
                worker: PERSON,
                from: 14,
                through: 16,
                capacity: Amount::new(LABOR, 1),
                wage_per_unit: Amount::new(TOKEN, 4),
                on_arrears: ArrearsPolicy::Continue,
                rank: 0,
            });
            if reversed {
                w.recovery.proceedings.reverse();
                w.lending.reverse();
                w.agreements.reverse();
            }
            let run = |backend| {
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                let mut audit = Audit::with_opening(
                    &w,
                    &s,
                    TOKEN,
                    Opening {
                        assets: [(1, 0), (2, 0)].into(),
                        inventory: [((seller, SEED), 1)].into(),
                        dues: Some(Valuation::default()),
                        ..Opening::default()
                    },
                )
                .unwrap();
                while sim.state.month < 13 {
                    audit.step(&mut sim).unwrap();
                }
                if household {
                    households::dissolution::request(&mut sim.world, &sim.state, HOME, SECOND)
                        .unwrap();
                }
                while (sim.state.month, sim.state.phase) != (15, Phase::Due) {
                    audit.step(&mut sim).unwrap();
                }
                assert_eq!(sim.state.credit.recovery.proceedings[&1].cash, 0);
                assert_eq!(sim.state.credit.recovery.proceedings[&2].cash, 14);
                assert_eq!(sim.state.balance(CUSTODIAN, TOKEN), 14);
                assert_eq!(sim.state.balance(PERSON, TOKEN), 4);
                let mut resumed =
                    Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
                let mut resumed_audit = audit.clone();
                let mut preview = sim.clone();
                preview.step().unwrap();
                let mut altered = preview.ledger.last().unwrap().clone();
                let cases = &mut altered.credit.as_mut().unwrap().after.recovery.proceedings;
                cases.get_mut(&1).unwrap().cash -= 1;
                cases.get_mut(&2).unwrap().cash += 1;
                let before = sim.state.clone();
                assert!(
                    settlement::commit(
                        &sim.world,
                        &mut sim.state,
                        &altered,
                        backend,
                        sim.effect_limit
                    )
                    .is_err()
                );
                assert_eq!(sim.state, before);
                audit.step(&mut sim).unwrap();
                // The first case sweeps newly earned wages but cannot spend the
                // second estate's fourteen opening coins. That estate pays its
                // own ten-coin loan, two-coin rent, then returns two surplus coins.
                assert_eq!(sim.state.credit.loans[&1].principal, 10);
                assert_eq!(sim.state.credit.loans[&2].principal, 0);
                assert_eq!(sim.state.obligations[&(1, 13)].paid, 0);
                assert_eq!(sim.state.obligations[&(2, 13)].paid, 2);
                assert_eq!(sim.state.credit.recovery.proceedings[&1].cash, 4);
                assert_eq!(sim.state.credit.recovery.proceedings[&2].cash, 0);
                assert_eq!(sim.state.balance(CUSTODIAN, TOKEN), 4);
                assert_eq!(sim.state.balance(seller, TOKEN), 2);
                if household {
                    assert_eq!(sim.state.balance(SECOND, TOKEN), 5);
                }
                let custody = audit.book().statements(CUSTODIAN, 12, 15).unwrap();
                assert_eq!(custody.assets, 4);
                assert_eq!(custody.liabilities, 4);
                assert_eq!(custody.equity, 0);
                assert_eq!(
                    audit
                        .book()
                        .balances()
                        .get(&(CUSTODIAN, LedgerAccount::CustodyCash(1)))
                        .copied()
                        .unwrap_or(0),
                    4
                );
                assert_eq!(
                    audit
                        .book()
                        .balances()
                        .get(&(CUSTODIAN, LedgerAccount::CustodyCash(2)))
                        .copied()
                        .unwrap_or(0),
                    0
                );
                while sim.state.month < 19 {
                    audit.step(&mut sim).unwrap();
                }
                while resumed.state.month < 19 {
                    resumed_audit.step(&mut resumed).unwrap();
                }
                assert_eq!((&sim.state, &audit), (&resumed.state, &resumed_audit));
                assert_eq!(sim.state.balance(CUSTODIAN, TOKEN), 0);
                assert_eq!(sim.state.obligations[&(1, 13)].paid, 2);
                assert!(
                    sim.state
                        .credit
                        .recovery
                        .proceedings
                        .values()
                        .all(|p| p.stage == Stage::Closed)
                );
                assert!(
                    sim.state
                        .credit
                        .loans
                        .values()
                        .all(|l| l.debt().unwrap() == 0)
                );
                (sim.state, sim.ledger, audit)
            };
            let reference = run(Backend::Reference);
            assert_eq!(reference, run(Backend::CubeCpu));
            if let Some(expected) = &canonical {
                assert_eq!(&reference, expected);
            }
            canonical = Some(reference);
        }
    }
}
