use economics_compute_smoke::{
    borrowing, commitments,
    compute::Backend,
    credit,
    dues_accounting::Valuation,
    financial_reporting::{Audit, Opening},
    forward::direct::Terms,
    household_governance::Governance,
    households::{self, Agreement},
    model::*,
    process_accounting::{Costs, Output},
    scenario::*,
    simulation::Simulation,
    stock_sale,
};
const HOME: AgentId = 10000;
const RENTED: AssetId = 999;

fn fixture(household: bool, sales: bool) -> (World, State) {
    let (mut w, s) = stock_sale::scenario("funded").unwrap();
    // This test concerns performance of accepted commitments, not underwriting.
    let c = w.credit.as_mut().unwrap();
    c.purchase_policy = borrowing::Policy::Scripted;
    c.stock_sales.as_mut().unwrap().reserve_months = 0;
    c.stock_sales.as_mut().unwrap().forecast = Some(economics_compute_smoke::sale_plan::Policy {
        horizon_months: 6,
        need_limits: [(NUTRITION, 0)].into(),
    });
    if !sales {
        c.stock_sales.as_mut().unwrap().purchase_budget = 0;
    }
    w.definitions
        .iter_mut()
        .find(|d| d.id == GROW)
        .unwrap()
        .outputs
        .iter_mut()
        .find(|a| a.resource == GRAIN)
        .unwrap()
        .quantity = 20;
    if household {
        households::form(
            &mut w,
            &s,
            Agreement {
                id: 1,
                agent: HOME,
                adults: vec![PERSON],
                governance: Governance::contributed(PERSON),
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
    }
    w.assets.push(Asset {
        id: RENTED,
        owner: STATE_AGENT,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: 99,
        holder: PERSON,
        asset: RENTED,
        from: 1,
        through: 36,
        output_owner: PERSON,
    });
    w.agreements.push(commitments::Agreement {
        id: 99,
        right: 99,
        creditor: STATE_AGENT,
        debtor: PERSON,
        activated: 1,
        payment: Amount::new(GRAIN, 2),
    });
    for (id, month) in [(5, 7), (6, 15)] {
        w.prepaid_deliveries.push(Terms {
            id,
            seller: PERSON,
            buyer: STATE_AGENT,
            month,
            due: month + 1,
            goods: Amount::new(GRAIN, 2),
            prepayment: Amount::new(TOKEN, 600),
        });
    }
    (w, s)
}

#[test]
fn repeated_household_harvests_service_mortgage_rent_and_forwards_through_actual_sales() {
    for household in [false, true] {
        for sales in [false, true] {
            let (w, s) = fixture(household, sales);
            let run = |backend| {
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                let mut audit = Audit::with_opening(&w,&s,TOKEN,Opening {
                    assets: [(PLOT,10000),(RENTED,10000)].into(),
                    inventory: [((PERSON,SEED),i128::from(s.balance(PERSON,SEED))),
                        ((PERSON,GRAIN),i128::from(s.balance(PERSON,GRAIN)))].into(),
                    exchange_values: [(SEED,1),(GRAIN,1)].into(),
                    dues: Some(Valuation([(99,1)].into())),
                    processes: Some(Costs {
                        beneficiary_policy: Some(economics_compute_smoke::process_accounting::BeneficiaryPolicy::TransferAtCost),
                        output_weights: [(GROW,[(Output::Stock(GRAIN),1),(Output::Stock(SEED),1)].into())].into(),
                        ..Costs::default()
                    }), ..Opening::default()
                }).unwrap();
                while sim.state.month < 8 {
                    audit.step(&mut sim).unwrap();
                }
                let mut resumed =
                    Simulation::new(sim.world.clone(), sim.state.clone(), backend).unwrap();
                let mut ra = audit.clone();
                let prefix = sim.ledger.len();
                while sim.state.month <= 24 {
                    audit.step(&mut sim).unwrap();
                }
                while resumed.state.month <= 24 {
                    ra.step(&mut resumed).unwrap();
                }
                assert_eq!(
                    (&sim.state, &sim.ledger[prefix..], &audit),
                    (&resumed.state, &resumed.ledger[..], &ra)
                );
                if sales {
                    assert_eq!(credit::owner(&w, &sim.state, PLOT), Some(PERSON));
                    assert_eq!(sim.state.credit.loans[&1].status, credit::Status::Repaid);
                    assert_eq!(sim.state.obligations[&(99, 13)].paid, 2);
                    for id in [5, 6] {
                        assert_eq!(sim.state.exchange.forwards[&id].delivered, 2);
                    }
                    let seeds: i32 = sim
                        .state
                        .balances
                        .iter()
                        .filter(|((_, r), _)| *r == SEED)
                        .map(|(_, q)| *q)
                        .sum();
                    let planted = sim
                        .state
                        .processes
                        .values()
                        .filter(|p| p.definition == GROW && p.status == Status::Active)
                        .count() as i32;
                    assert_eq!(seeds + planted, 1);
                    assert_eq!(sim.state.credit.loans[&1].debtor, PERSON);
                    // Collective balances never silently assume the member's loan.
                    assert!(
                        sim.ledger
                            .iter()
                            .filter_map(|b| b.household.as_ref())
                            .flat_map(|h| &h.reservations)
                            .all(|r| r.allocated == 0
                                || !matches!(
                                    r.request.purpose,
                                    households::Purpose::LoanSupport { .. }
                                ))
                    );
                    assert!(
                        sim.reports
                            .iter()
                            .filter(|r| r.agent == PERSON)
                            .all(|r| r.deficit(NUTRITION) == 0)
                    );
                    assert!(
                        sim.state
                            .processes
                            .values()
                            .filter(|p| p.definition == GROW && p.status == Status::Completed)
                            .count()
                            >= 2
                    );
                } else {
                    assert_eq!(sim.state.credit.stock_spent, 0);
                    assert_ne!(credit::owner(&w, &sim.state, PLOT), Some(PERSON));
                }
                for agent in &w.agents {
                    let report = audit.book().statements(agent.id, 1, 24).unwrap();
                    assert_eq!(report.assets, report.liabilities + report.equity);
                }
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

#[test]
fn fixed_food_buffer_can_pay_financial_claims_while_missing_a_meal() {
    let (mut w, s) = fixture(false, true);
    let policy = w.credit.as_mut().unwrap().stock_sales.as_mut().unwrap();
    policy.forecast = None;
    policy.reserve_months = 6;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(24).unwrap();
    assert_eq!(sim.state.credit.loans[&1].status, credit::Status::Repaid);
    assert_eq!(sim.state.obligations[&(99, 13)].paid, 2);
    for id in [5, 6] {
        assert_eq!(sim.state.exchange.forwards[&id].delivered, 2);
    }
    assert!(
        sim.reports
            .iter()
            .filter(|r| r.agent == PERSON)
            .any(|r| r.deficit(NUTRITION) > 0)
    );
}
