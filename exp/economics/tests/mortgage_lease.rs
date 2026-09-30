use economics_compute_smoke::{
    activities::CoinPayment,
    commitments::{Agreement as Land, PaymentPolicy},
    compute::Backend,
    credit,
    dues_accounting::Valuation,
    finance::CollectionPolicy,
    financial_reporting::{Audit, Opening},
    household_governance::Governance,
    households::{self, Agreement},
    model::*,
    scenario::*,
    settlement,
    simulation::Simulation,
};
const RENTED: AssetId = 999;
const HOME: AgentId = 10000;
fn fixture(household: bool, alternate: bool, proportional: bool) -> (World, State) {
    let (mut w, mut s) = credit::scenario("default").unwrap();
    w.resources.extend(baseline().0.resources);
    let borrower = if household { HOME } else { PERSON };
    if household {
        let mut member = baseline().0.participants[0].clone();
        member.needs.clear();
        member.capacity.quantity = 0;
        w.participants.push(member);
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
        id: 1,
        holder: borrower,
        asset: RENTED,
        from: 1,
        through: 36,
        output_owner: borrower,
    });
    w.agreements.push(Land {
        id: 1,
        right: 1,
        creditor: STATE_AGENT,
        debtor: borrower,
        activated: 1,
        payment: Amount::new(
            if alternate { GRAIN } else { TOKEN },
            if alternate { 2 } else { 4 },
        ),
    });
    if alternate {
        w.activities.coin_payments.insert(
            1,
            CoinPayment {
                resource: TOKEN,
                coins_per_unit: 2,
            },
        );
    }
    w.payment_policy = PaymentPolicy::DebtFirst;
    w.collection_policy = if proportional {
        CollectionPolicy::Proportional
    } else {
        CollectionPolicy::Stable
    };
    let c = w.credit.as_mut().unwrap();
    c.endowments.clear();
    c.application.buyer = borrower;
    c.application.month = 12;
    c.application.downpayment = 2;
    let offer = &mut c.offers[0];
    offer.sale.price.quantity = 10;
    offer.minimum_downpayment = 2;
    offer.loan.max_principal = 8;
    offer.loan.monthly_rate_bps = 0;
    offer.loan.term_months = 4;
    offer.loan.grace_months = 12;
    offer.collateral.priority = 0;
    offer.collateral.settlement = credit::CollateralSettlement::FixedValue { value: 8 };
    s.month = 12;
    s.balances.insert((borrower, TOKEN), 5);
    s.balances.insert((STATE_AGENT, TOKEN), 20);
    (w, s)
}
#[test]
fn mortgage_and_independent_lease_share_due_cash_with_person_and_household_books() {
    for household in [false, true] {
        for alternate in [false, true] {
            for proportional in [false, true] {
                let (w, s) = fixture(household, alternate, proportional);
                let borrower = if household { HOME } else { PERSON };
                let run = |backend| {
                    let mut audit = Audit::with_opening(
                        &w,
                        &s,
                        TOKEN,
                        Opening {
                            assets: [(PLOT, 10), (RENTED, 10)].into(),
                            dues: Some(Valuation([(1, 2)].into())),
                            ..Opening::default()
                        },
                    )
                    .unwrap();
                    let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                    while sim.state.month <= 12 {
                        audit.step(&mut sim).unwrap();
                    }
                    assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(borrower));
                    assert_eq!(
                        credit::owner(&sim.world, &sim.state, RENTED),
                        Some(STATE_AGENT)
                    );
                    assert_eq!(sim.state.balance(borrower, TOKEN), 3);
                    audit.step(&mut sim).unwrap(); // Open
                    let (mut resumed, mut ra) = (sim.clone(), audit.clone());
                    audit.step(&mut sim).unwrap(); // Due
                    let loan_paid = if proportional { 1 } else { 2 };
                    assert_eq!(sim.state.credit.loans[&1].principal, 8 - loan_paid);
                    let bill = &sim.state.obligations[&(1, 13)];
                    let rent_paid = if alternate {
                        if proportional { 1 } else { 0 }
                    } else {
                        3 - loan_paid
                    };
                    assert_eq!(bill.paid, rent_paid);
                    assert_eq!(bill.in_kind_paid, if alternate { 0 } else { rent_paid });
                    assert_eq!(
                        sim.state.balance(borrower, TOKEN),
                        if alternate && !proportional { 1 } else { 0 }
                    );
                    while sim.state.month <= 14 {
                        audit.step(&mut sim).unwrap();
                    }
                    while resumed.state.month <= 14 {
                        ra.step(&mut resumed).unwrap();
                    }
                    assert_eq!(
                        (&sim.state, &sim.ledger, &audit),
                        (&resumed.state, &resumed.ledger, &ra)
                    );
                    for who in [PERSON, STATE_AGENT]
                        .into_iter()
                        .chain(household.then_some(HOME))
                    {
                        let report = audit.book().statements(who, 12, 14).unwrap();
                        assert_eq!(report.assets, report.liabilities + report.equity);
                    }
                    (sim.state, sim.ledger, audit)
                };
                assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
            }
        }
    }
}
#[test]
fn financing_cannot_erase_a_lease_on_the_same_property() {
    let (mut w, s) = fixture(false, false, false);
    w.rights[0].asset = PLOT;
    assert!(settlement::validate_world(&w, &s).is_err());
    w.credit.as_mut().unwrap().attached_rights.insert(1);
    assert!(
        settlement::validate_world(&w, &s)
            .unwrap_err()
            .contains("existing land lease")
    );
}
