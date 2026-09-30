use economics_compute_smoke::{
    calibration::{self, CROP_PERSON, WOOD_PERSON},
    compute::Backend,
    cooperation::{Contract, Delivery},
    credit::{Advance, LoanOffer},
    finance::Transfer,
    financial_reporting::{Audit, Opening},
    model::*,
    negotiation::GRAIN_MARKET,
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    production_market::{Choice, Policy, Purchases, Work},
    scenario::*,
    settlement,
    simulation::Simulation,
};

fn fixture(buyer_lends: bool, funded: bool, delivery_month: u32) -> (World, State) {
    let (mut w, mut s) = calibration::scenario(true);
    // Retain real needs; isolate acquisition from production profitability.
    w.production_market.as_mut().unwrap().policy = Policy::Agreement(Box::new(Contract {
        start: 1,
        through: 6,
        choices: [CROP_PERSON, WOOD_PERSON]
            .into_iter()
            .map(|a| {
                (
                    a,
                    Choice {
                        work: Work::Wait,
                        buy: Purchases::None,
                    },
                )
            })
            .collect(),
        deliveries: vec![Delivery {
            month: delivery_month,
            market: GRAIN_MARKET,
            goods: Transfer {
                from: CROP_PERSON,
                to: WOOD_PERSON,
                amount: Amount::new(GRAIN, 2),
            },
            payment: Transfer {
                from: WOOD_PERSON,
                to: CROP_PERSON,
                amount: Amount::new(TOKEN, 4),
            },
        }],
    }));
    let (creditor, debtor) = if buyer_lends {
        (WOOD_PERSON, STATE_AGENT)
    } else {
        (STATE_AGENT, WOOD_PERSON)
    };
    s.balances.insert(
        (WOOD_PERSON, TOKEN),
        if buyer_lends {
            if funded { 8 } else { 4 }
        } else {
            0
        },
    );
    s.balances
        .insert((STATE_AGENT, TOKEN), if buyer_lends { 0 } else { 4 });
    w.transaction_policy.as_mut().unwrap().permissions.extend([
        (PERSON_TYPE, Action::Borrow),
        (PERSON_TYPE, Action::Lend),
        (STATE_TYPE, Action::Borrow),
        (STATE_TYPE, Action::Lend),
    ]);
    w.lending.push(Advance {
        id: 500,
        debtor,
        principal: 4,
        month: 1,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor,
            denomination: TOKEN,
            max_principal: 4,
            monthly_rate_bps: 0,
            term_months: 6,
            grace_months: 12,
        },
    });
    (w, s)
}

#[test]
fn cooperative_deliveries_share_loan_reservations_and_wait_for_receipts() {
    for (buyer_lends, funded, month, trades) in [
        (false, true, 1, false), // incoming loan unavailable in this batch
        (false, true, 2, true),  // retained cash available next month
        (true, false, 1, false), // cannot both lend and spend the same coins
        (true, true, 1, true),   // separate funding for loan and delivery
    ] {
        let (w, s) = fixture(buyer_lends, funded, month);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    inventory: s
                        .balances
                        .iter()
                        .filter(|((_, r), q)| {
                            *r != TOKEN
                                && **q > 0
                                && w.resources
                                    .iter()
                                    .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
                        })
                        .map(|(a, q)| (*a, i128::from(*q)))
                        .collect(),
                    exchange_values: [(GRAIN, 1), (FUEL, 1), (SEED, 1), (RAW_WOOD, 1)].into(),
                    assets: w.assets.iter().map(|a| (a.id, 1)).collect(),
                    processes: Some(Default::default()),
                    ..Opening::default()
                },
            )
            .unwrap();
            while (sim.state.month, sim.state.phase) != (month, Phase::Acquire) {
                audit.step(&mut sim).unwrap();
            }
            let before = sim.state.clone();
            let mut resumed = Simulation::new(w.clone(), before.clone(), backend).unwrap();
            let mut resumed_audit = audit.clone();
            audit.step(&mut sim).unwrap();
            resumed_audit.step(&mut resumed).unwrap();
            assert_eq!((&sim.state, &audit), (&resumed.state, &resumed_audit));
            assert_eq!(sim.ledger.last(), resumed.ledger.last());
            assert_eq!(sim.state.credit.loans.len(), 1);
            let round = sim.state.town_market.history.last().unwrap();
            let receipt = round.cooperation.as_ref().unwrap();
            assert_eq!(!receipt.completed.is_empty(), trades);
            assert_eq!(receipt.failure.is_some(), !trades);
            assert_eq!(round.transactions.len(), usize::from(trades));
            if !trades {
                assert!(
                    receipt
                        .failure
                        .as_ref()
                        .unwrap()
                        .contains("opening budget 0")
                );
            }
            let mut forged = sim.ledger.last().unwrap().clone();
            if let Some(economics_compute_smoke::town_market::Boundary::Market(r)) =
                &mut forged.town_market
            {
                r.cooperation.as_mut().unwrap().completed.clear();
                r.cooperation.as_mut().unwrap().event = "invented acceptance".into();
            }
            let mut unchanged = before.clone();
            assert!(
                settlement::commit(&w, &mut unchanged, &forged, backend, sim.effect_limit).is_err()
            );
            assert_eq!(unchanged, before);
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn a_failed_configured_delivery_does_not_restart_when_loan_money_becomes_available() {
    let (mut w, s) = fixture(false, true, 1);
    let Policy::Agreement(c) = &mut w.production_market.as_mut().unwrap().policy else {
        unreachable!()
    };
    let mut later = c.deliveries[0].clone();
    later.month = 2;
    c.deliveries.push(later);
    for backend in [Backend::Reference, Backend::CubeCpu] {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        sim.run_months(1).unwrap();
        assert!(
            sim.state.town_market.history[0]
                .cooperation
                .as_ref()
                .unwrap()
                .failure
                .is_some()
        );
        let mut resumed = Simulation::new(w.clone(), sim.state.clone(), backend).unwrap();
        sim.run_months(1).unwrap();
        resumed.run_months(1).unwrap();
        assert_eq!(sim.state, resumed.state);
        let round = sim.state.town_market.history.last().unwrap();
        assert_eq!(round.cooperation.as_ref().unwrap().event, "Cancelled");
        assert!(round.transactions.is_empty());
        assert!(round.cooperation.as_ref().unwrap().active.is_none());
        assert!(sim.state.balance(WOOD_PERSON, TOKEN) > 0);
        assert_eq!(sim.state.credit.loans.len(), 1);
    }
}

#[test]
fn household_contribution_storage_is_reserved_before_accepting_a_delivery() {
    use economics_compute_smoke::{
        household_governance::{Governance, Purchasing},
        households::{self, Agreement},
    };
    const HOME: AgentId = 10_000;
    for room in [false, true] {
        let (mut w, mut s) = fixture(true, true, 1);
        for resource in [GRAIN, FUEL, SEED, RAW_WOOD] {
            s.balances.insert((WOOD_PERSON, resource), 0);
        }
        w.storage.weights.insert(GRAIN, 1);
        w.storage.weights.insert(SEED, 1);
        w.storage.capacities.insert(WOOD_PERSON, 4);
        w.transaction_policy
            .as_mut()
            .unwrap()
            .permissions
            .insert((PERSON_TYPE, Action::FoundHousehold));
        let mut governance = Governance::contributed(WOOD_PERSON);
        governance.charter.purchasing = Purchasing::Members;
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: vec![WOOD_PERSON],
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
        s.balances.insert((HOME, SEED), if room { 1 } else { 2 });
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.phase != Phase::Acquire {
                sim.step().unwrap();
            }
            let before = sim.state.clone();
            // The raw two-unit purchase fits the member's private room in both
            // cases. Its one-unit household contribution only fits in one.
            assert!(economics_compute_smoke::storage::fits(
                &w,
                &economics_compute_smoke::storage::usage(&w, &before.balances),
                &Transfer {
                    from: CROP_PERSON,
                    to: WOOD_PERSON,
                    amount: Amount::new(GRAIN, 2)
                }
                .effects()
                .unwrap()
            ));
            sim.step().unwrap();
            let round = sim.state.town_market.history.last().unwrap();
            let boundary = round.cooperation.as_ref().unwrap();
            assert_eq!(boundary.failure.is_none(), room);
            assert_eq!(boundary.completed.len(), usize::from(room));
            assert_eq!(sim.state.balance(WOOD_PERSON, GRAIN), i32::from(room));
            assert_eq!(sim.state.balance(HOME, GRAIN), i32::from(room));
            assert_eq!(sim.state.balance(HOME, SEED), if room { 1 } else { 2 });
            assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            assert_eq!(sim.state.credit.loans[&500].creditor, WOOD_PERSON);
            let mut resumed = Simulation::new(w.clone(), before, backend).unwrap();
            resumed.step().unwrap();
            assert_eq!(sim.state, resumed.state);
            assert_eq!(sim.ledger.last(), resumed.ledger.last());
            (sim.state, sim.ledger)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
