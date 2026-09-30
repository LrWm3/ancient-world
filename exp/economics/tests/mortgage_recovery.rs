use economics_compute_smoke::{
    compute::Backend,
    credit::{self, CollateralSettlement},
    financial_reporting::{Audit, Opening},
    model::*,
    process_accounting::{Costs, Output},
    recovery::{Bid, Listing, ProceedingTerms},
    scenario::*,
    simulation::Simulation,
};
const BUYER: AgentId = 98;
const ESTATE: AgentId = 97;

fn fixture(price: i32, funded: bool, maintain: bool) -> (World, State) {
    let (mut w, mut s) = credit::crop_scenario(false).unwrap();
    let c = w.credit.as_mut().unwrap();
    c.endowments.clear();
    c.application.downpayment = 2;
    let offer = &mut c.offers[0];
    offer.sale.price.quantity = 8;
    offer.minimum_downpayment = 2;
    offer.loan.max_principal = 6;
    offer.loan.monthly_rate_bps = 0;
    offer.loan.term_months = 1;
    offer.loan.grace_months = 0;
    offer.collateral.settlement = CollateralSettlement::AuthorizedLiquidation;
    for id in [BUYER, ESTATE] {
        w.agents.push(Agent {
            id,
            name: format!("agent {id}"),
        });
    }
    w.participants.push(Participant {
        agent: BUYER,
        capacity: Amount::new(LABOR, if maintain { 2 } else { 0 }),
        needs: vec![],
    });
    s.balances.insert((PERSON, TOKEN), 2);
    s.balances.insert((STATE_AGENT, TOKEN), 20);
    s.balances
        .insert((BUYER, TOKEN), if funded { price } else { 0 });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: PERSON,
        authority: STATE_AGENT,
        estate: ESTATE,
        denomination: TOKEN,
        opening_month: 3,
        earliest_close: 4,
        assets: vec![Listing {
            asset: PLOT,
            minimum_price: 4,
        }],
        discharge_deficiency: false,
    });
    w.recovery.bids.push(Bid {
        id: 1,
        proceeding: 1,
        buyer: BUYER,
        asset: PLOT,
        month: 3,
        price,
    });
    (w, s)
}

#[test]
fn financed_land_and_attached_crop_use_the_authorized_estate_lifecycle() {
    for (price, funded, maintain) in [
        (4, true, true),
        (9, true, true),
        (4, true, false),
        (4, false, true),
    ] {
        let (w, s) = fixture(price, funded, maintain);
        let run = |backend| {
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
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
                audit.step(&mut sim).unwrap();
            }
            let original = sim
                .state
                .processes
                .values()
                .find(|p| p.definition == GROW)
                .unwrap()
                .clone();
            assert_eq!(original.operator, PERSON);
            assert_eq!(credit::owner(&w, &sim.state, PLOT), Some(PERSON));
            assert_eq!(sim.state.credit.loans[&1].principal, 6);
            let (mut resumed, mut ra) = (sim.clone(), audit.clone());
            audit.step(&mut sim).unwrap();
            let mut expected = original.clone();
            if funded {
                expected.operator = BUYER;
                expected.beneficiary = BUYER;
                expected.goal = None;
            }
            assert_eq!(sim.state.processes[&original.id], expected);
            assert_eq!(
                credit::owner(&w, &sim.state, PLOT),
                Some(if funded { BUYER } else { PERSON })
            );
            assert_eq!(
                sim.state.balance(ESTATE, TOKEN),
                if funded { price } else { 0 }
            );
            assert_eq!(sim.state.credit.loans[&1].principal, 6); // sale receipt is not Due payment
            while sim.state.month < 8 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month < 8 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &ra)
            );
            assert_eq!(
                sim.state.credit.loans[&1].principal,
                if funded { (6 - price).max(0) } else { 6 }
            );
            assert_eq!(
                sim.state.balance(PERSON, TOKEN),
                if funded { (price - 6).max(0) } else { 0 }
            );
            let crop = &sim.state.processes[&original.id];
            assert_eq!(
                crop.status,
                if !funded || maintain {
                    Status::Completed
                } else {
                    Status::Aborted
                }
            );
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
            for agent in &w.agents {
                let report = audit.book().statements(agent.id, 1, 7).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn mortgage_recovery_keeps_custody_nonoperating_and_legacy_enforcement_explicit() {
    let (w, s) = fixture(4, true, true);
    for mode in 0..3 {
        let mut invalid = w.clone();
        let c = invalid.credit.as_mut().unwrap();
        match mode {
            0 => c.application.buyer = ESTATE,
            1 => c.endowments.push(credit::Endowment {
                agent: ESTATE,
                amount: Amount::new(TOKEN, 1),
            }),
            _ => c.offers[0].collateral.settlement = CollateralSettlement::FixedValue { value: 6 },
        }
        assert!(Simulation::new(invalid, s.clone(), Backend::Reference).is_err());
    }
}

#[test]
fn buying_estate_land_can_reserve_new_cultivation_in_the_same_request() {
    use economics_compute_smoke::offers::{self, Id, Request};
    for funded in [false, true] {
        let (mut w, mut s) = fixture(4, funded, true);
        w.scheduled_starts.clear();
        s.balances.insert((BUYER, SEED), 1);
        let run = |backend| {
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    assets: [(PLOT, 8)].into(),
                    inventory: [((PERSON, SEED), 1), ((BUYER, SEED), 1)].into(),
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
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while (sim.state.month, sim.state.phase) != (3, Phase::Acquire) {
                audit.step(&mut sim).unwrap();
            }
            let before = sim.state.clone();
            let requests = [
                Request::new(Id::LiquidationBid(1), BUYER),
                Request::new(Id::Process(GROW), BUYER),
            ];
            let prepared = offers::prepare(&sim, &requests);
            assert_eq!(sim.state, before);
            if !funded {
                assert!(prepared.is_err());
                assert!(offers::accept(&mut sim, &requests).is_err());
                assert_eq!(sim.state, before);
                return (sim.state, sim.ledger, audit);
            }
            let prepared = prepared.unwrap();
            offers::accept(&mut sim, &requests).unwrap();
            audit
                .record(&sim.world, &before, &prepared, &sim.state)
                .unwrap();
            assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(BUYER));
            assert_eq!(sim.state.balance(BUYER, SEED), 1);
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 4);
            assert_eq!(sim.state.credit.loans[&1].principal, 6);
            audit.step(&mut sim).unwrap();
            assert_eq!(sim.state.balance(BUYER, SEED), 0);
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
            assert_eq!(sim.state.balance(BUYER, GRAIN), 8);
            assert_eq!(sim.state.credit.loans[&1].principal, 2);
            assert_eq!(sim.state.balance(ESTATE, TOKEN), 0);
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
