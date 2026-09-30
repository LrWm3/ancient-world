use economics_compute_smoke::{
    accounting::Flow,
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::Audit,
    issuance_accounting,
    minting::{self, *},
    model::*,
    opportunities::{Action, PERSON_TYPE, STATE_TYPE},
    simulation::Simulation,
};

fn fixture() -> (World, State) {
    let (mut w, s) = minting::scenario("normal").unwrap();
    let c = w.minting.as_mut().unwrap();
    c.deals.retain(|d| d.month == 2);
    let early: Vec<_> = c
        .deals
        .iter()
        .cloned()
        .map(|mut d| {
            d.id += 10;
            d.month = 1;
            d
        })
        .collect();
    c.deals.extend(early);
    w.lending.push(Advance {
        id: 10,
        debtor: ISSUER,
        terms: LoanOffer {
            creditor: SUPPLIER,
            denomination: COIN,
            max_principal: 6,
            monthly_rate_bps: 0,
            term_months: 12,
            grace_months: 1,
        },
        principal: 6,
        month: 1,
        collateral: None,
        priority: 0,
    });
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .extend([(PERSON_TYPE, Action::Lend), (STATE_TYPE, Action::Borrow)]);
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    let costs = s
        .balances
        .iter()
        .filter(|((_, r), q)| {
            *r != COIN
                && **q > 0
                && w.resources
                    .iter()
                    .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
        })
        .map(|(k, q)| (*k, i128::from(*q)))
        .collect();
    Audit::with_opening(
        w,
        s,
        COIN,
        economics_compute_smoke::financial_reporting::Opening {
            assets: w.assets.iter().map(|a| (a.id, 10)).collect(),
            inventory: costs,
            processes: Some(Default::default()),
            dues: Some(economics_compute_smoke::dues_accounting::Valuation(
                w.agreements.iter().map(|a| (a.id, 1)).collect(),
            )),
            issuance: Some(issuance_accounting::Policy::NonRedeemableEquity),
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn borrowed_coins_fund_later_physical_issuance_and_repayment_with_reconciled_books() {
    let (w, s) = fixture();
    let mut a = audit(&w, &s);
    let mut b = a.clone();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while reference.state.month <= 1 {
        a.step(&mut reference).unwrap();
    }
    assert_eq!(reference.state.balance(ISSUER, COIN), 6);
    assert_eq!(reference.state.balance(ISSUER, METAL), 0);
    let mint = reference
        .ledger
        .iter()
        .find_map(|b| b.minting.as_ref())
        .unwrap();
    assert!(!mint.receipts[0].accepted);
    let mut resumed = Simulation::new(
        reference.world.clone(),
        reference.state.clone(),
        Backend::Reference,
    )
    .unwrap();
    let mut resumed_audit = a.clone();
    while reference.state.month <= 3 {
        a.step(&mut reference).unwrap();
    }
    while resumed.state.month <= 3 {
        resumed_audit.step(&mut resumed).unwrap();
    }
    cpu.world.agents.reverse();
    cpu.world.minting.as_mut().unwrap().deals.reverse();
    while cpu.state.month <= 3 {
        b.step(&mut cpu).unwrap();
    }
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.state, resumed.state);
    assert_eq!(reference.ledger, cpu.ledger);
    assert_eq!(a, b);
    assert_eq!(a, resumed_audit);
    assert_eq!(reference.state.credit.loans[&10].principal, 5);
    assert_eq!(reference.state.balance(ISSUER, COIN), 9);
    let statement = a.book().statements(ISSUER, 1, 3).unwrap();
    assert_eq!(statement.issuance_change, 10);
    assert_eq!(statement.cash_flows[&Flow::Financing], 5);
    assert_eq!(statement.closing_cash, 9);
}

#[test]
fn lending_reserves_the_same_opening_money_as_mint_purchases() {
    let (mut w, mut s) = fixture();
    w.lending[0].month = 2;
    w.lending[0].debtor = SUPPLIER;
    w.lending[0].terms.creditor = ISSUER;
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .extend([(STATE_TYPE, Action::Lend), (PERSON_TYPE, Action::Borrow)]);
    s.balances.insert((ISSUER, COIN), 6);
    w.minting.as_mut().unwrap().deals.retain(|d| d.month == 2);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(2).unwrap();
    assert_eq!(sim.state.credit.loans[&10].principal, 6);
    assert_eq!(sim.state.balance(ISSUER, COIN), 0);
    assert_eq!(sim.state.balance(ISSUER, METAL), 0);
    assert!(
        !sim.state
            .processes
            .values()
            .any(|p| p.definition == MINT && p.status == Status::Completed)
    );
}

#[test]
fn prepaid_wheat_funds_minting_only_after_admission_and_delivers_once() {
    use economics_compute_smoke::forward::direct::Terms;
    let (mut w, s) = fixture();
    w.lending.clear();
    w.prepaid_deliveries.push(Terms {
        id: 20,
        seller: ISSUER,
        buyer: WORKER,
        month: 1,
        due: 2,
        goods: Amount::new(WHEAT, 2),
        prepayment: Amount::new(COIN, 6),
    });
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while sim.state.month <= 1 {
        a.step(&mut sim).unwrap();
    }
    assert_eq!(sim.state.balance(ISSUER, COIN), 6);
    assert_eq!(sim.state.balance(WORKER, WHEAT), 0);
    assert_eq!(sim.state.balance(ISSUER, METAL), 0);
    let mut checkpoint =
        Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
    let mut report = a.clone();
    while sim.state.month <= 3 {
        a.step(&mut sim).unwrap();
    }
    while checkpoint.state.month <= 3 {
        report.step(&mut checkpoint).unwrap();
    }
    assert_eq!(sim.state, checkpoint.state);
    assert_eq!(a, report);
    assert_eq!(sim.state.exchange.forwards[&20].delivered, 2);
    assert_eq!(sim.state.balance(WORKER, WHEAT), 2);
    assert_eq!(sim.state.balance(ISSUER, COIN), 10);
    assert_eq!(
        a.book().statements(ISSUER, 1, 3).unwrap().issuance_change,
        10
    );
}

#[test]
fn due_forward_and_mint_package_cannot_sell_the_same_metal() {
    use economics_compute_smoke::forward::direct::Terms;
    let (mut w, mut s) = fixture();
    w.lending.clear();
    s.balances.insert((ISSUER, COIN), 6);
    w.minting.as_mut().unwrap().deals.retain(|d| d.month == 2);
    w.prepaid_deliveries.push(Terms {
        id: 20,
        seller: SUPPLIER,
        buyer: WORKER,
        month: 1,
        due: 2,
        goods: Amount::new(METAL, 2),
        prepayment: Amount::new(COIN, 2),
    });
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    sim.run_months(3).unwrap();
    assert_eq!(sim.state.exchange.forwards[&20].delivered, 2);
    assert_eq!(sim.state.balance(WORKER, METAL), 2);
    assert_eq!(sim.state.balance(ISSUER, METAL), 0);
    assert_eq!(sim.state.balance(ISSUER, COIN), 6);
    let b = sim
        .ledger
        .iter()
        .find(|b| b.month == 2 && b.phase == Phase::Acquire)
        .unwrap();
    assert!(!b.minting.as_ref().unwrap().receipts[0].accepted);
    assert_eq!(sim.state.balance(WORKER, FIREWOOD), 1);
}

#[test]
fn generated_mint_orders_use_spendable_budget_after_financing() {
    for outgoing in [false, true] {
        let (mut w, mut s) = minting::order_scenario("normal").unwrap();
        let (loan_world, _) = fixture();
        w.lending = loan_world.lending;
        let p = w.transaction_policy.as_mut().unwrap();
        p.permissions.extend([
            (PERSON_TYPE, Action::Borrow),
            (PERSON_TYPE, Action::Lend),
            (STATE_TYPE, Action::Borrow),
            (STATE_TYPE, Action::Lend),
        ]);
        if outgoing {
            s.balances.insert((ISSUER, COIN), 6);
            w.lending[0].debtor = SUPPLIER;
            w.lending[0].terms.creditor = ISSUER;
        }
        w.minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap()
            .month = 1;
        for start in &mut w.scheduled_starts {
            if start.definition == MINT {
                start.month = 1;
            }
        }
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        sim.run_months(1).unwrap();
        let plan = sim
            .ledger
            .iter()
            .find_map(|b| b.minting.as_ref())
            .unwrap()
            .plan
            .as_ref()
            .unwrap();
        assert_eq!(plan.reason, "insufficient opening funds at target date");
        assert!(!plan.orders.iter().any(|o| o.agent == ISSUER));
        assert!(plan.deals.is_empty());
        assert_eq!(sim.state.credit.loans.len(), 1);
        assert_eq!(sim.state.balance(ISSUER, METAL), 0);
    }
}

#[test]
fn food_provision_cannot_spend_a_loan_received_at_the_same_acquisition_boundary() {
    let (mut w, mut s) = minting::provision_scenario("adequate").unwrap();
    let (loan_world, _) = fixture();
    w.lending = loan_world.lending;
    w.lending[0].debtor = WORKER;
    w.lending[0].terms.creditor = ISSUER;
    s.balances.insert((WORKER, COIN), 0);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .extend([(PERSON_TYPE, Action::Borrow), (STATE_TYPE, Action::Lend)]);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut a = audit(&sim.world, &sim.state);
    while sim.state.month <= 1 {
        a.step(&mut sim).unwrap();
    }
    let b = sim.ledger.iter().find_map(|b| b.minting.as_ref()).unwrap();
    assert!(
        !b.deals
            .iter()
            .any(|d| d.buyer == WORKER && d.market == WHEAT)
    );
    assert_eq!(sim.state.credit.loans[&10].principal, 6);
    assert_eq!(sim.state.balance(WORKER, COIN), 6);
    while sim.state.month <= 2 {
        a.step(&mut sim).unwrap();
    }
    let b = sim
        .ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .find(|b| b.month == 2)
        .unwrap();
    assert!(
        b.deals
            .iter()
            .any(|d| d.buyer == WORKER && d.market == WHEAT)
    );
}

#[test]
fn authorized_recovery_blocks_mint_counterparty_and_preserves_estate_custody() {
    use economics_compute_smoke::recovery::{ProceedingTerms, Stage};
    let (mut w, mut s) = minting::scenario("normal").unwrap();
    let (loan_world, _) = fixture();
    w.lending = loan_world.lending;
    let loan = &mut w.lending[0];
    loan.debtor = SUPPLIER;
    loan.terms.creditor = WORKER;
    loan.terms.max_principal = 8;
    loan.principal = 8;
    s.balances.insert((WORKER, COIN), 8);
    s.balances.insert((SUPPLIER, COIN), 0);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .extend([(PERSON_TYPE, Action::Borrow), (PERSON_TYPE, Action::Lend)]);
    let deals = &mut w.minting.as_mut().unwrap().deals;
    deals.retain(|d| d.market != WHEAT || d.buyer == SUPPLIER);
    for d in deals {
        if d.market == WHEAT {
            d.month = 2;
            d.price = 8;
        } else {
            d.month = 4;
        }
    }
    for start in &mut w.scheduled_starts {
        start.month = 4;
    }
    w.agents.push(Agent {
        id: 999,
        name: "custody".into(),
    });
    w.recovery.proceedings.push(ProceedingTerms {
        id: 1,
        debtor: SUPPLIER,
        authority: ISSUER,
        estate: 999,
        denomination: COIN,
        opening_month: 4,
        earliest_close: 5,
        assets: vec![],
        discharge_deficiency: true,
    });
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while sim.state.month <= 4 {
        a.step(&mut sim).unwrap();
    }
    assert_eq!(
        sim.state.credit.recovery.proceedings[&1].stage,
        Stage::Active
    );
    assert_eq!(sim.state.balance(ISSUER, COIN), 8);
    assert_eq!(sim.state.balance(SUPPLIER, METAL), 2);
    let b = sim
        .ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .find(|b| b.month == 4)
        .unwrap();
    assert!(!b.receipts[0].accepted);
    assert_eq!(
        a.book().statements(ISSUER, 1, 4).unwrap().issuance_change,
        0
    );
    let mut invalid = sim.world.clone();
    invalid.minting.as_mut().unwrap().deals[0].buyer = 999;
    assert!(Simulation::new(invalid, sim.state.clone(), Backend::Reference).is_err());
}

#[test]
fn annual_coin_and_native_land_dues_compose_without_unbacked_issuance() {
    use economics_compute_smoke::commitments::Agreement;
    for denomination in [COIN, WHEAT] {
        let (mut w, s) = minting::scenario("normal").unwrap();
        w.assets.push(Asset {
            id: 777,
            owner: ISSUER,
            kind: 1,
        });
        w.rights.push(UseRight {
            id: 77,
            holder: SUPPLIER,
            asset: 777,
            from: 1,
            through: 24,
            output_owner: SUPPLIER,
        });
        w.agreements.push(Agreement {
            id: 77,
            right: 77,
            creditor: ISSUER,
            debtor: SUPPLIER,
            activated: 1,
            payment: Amount::new(denomination, 2),
        });
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        while sim.state.month <= 13 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.obligations[&(77, 13)].paid, 2);
        assert_eq!(sim.state.obligations[&(77, 13)].in_kind_paid, 2);
        assert_eq!(
            sim.state.balance(SUPPLIER, denomination),
            if denomination == COIN { 3 } else { 1 }
        );
        let supply: i32 = sim
            .state
            .balances
            .iter()
            .filter(|((_, r), _)| *r == COIN)
            .map(|(_, q)| *q)
            .sum();
        assert_eq!(supply, 22);
        let statement = a.book().statements(ISSUER, 1, 13).unwrap();
        assert_eq!(statement.issuance_change, 10);
    }
}

#[test]
fn employment_and_mint_market_share_hours_and_pay_only_for_actual_work() {
    use economics_compute_smoke::employment::{ArrearsPolicy, Terms};
    for market_hours in [false, true] {
        let (mut w, s) = minting::scenario("normal").unwrap();
        if !market_hours {
            w.minting
                .as_mut()
                .unwrap()
                .deals
                .retain(|d| d.market != HOURS);
        }
        w.employment.push(Terms {
            id: 30,
            employer: ISSUER,
            worker: WORKER,
            from: 2,
            through: 2,
            capacity: Amount::new(HOURS, 2),
            wage_per_unit: Amount::new(COIN, 2),
            on_arrears: ArrearsPolicy::SuspendDelivery,
            rank: 0,
        });
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.balance(ISSUER, COIN), 10);
        assert_eq!(sim.state.balance(WORKER, COIN), 7);
        assert_eq!(sim.state.balance(WORKER, FIREWOOD), 0);
        assert_eq!(sim.state.balance(ISSUER, HOURS), 0);
        if market_hours {
            assert!(sim.state.employment.earned.is_empty());
        } else {
            let earned = &sim.state.employment.earned[&(30, 2)];
            assert_eq!(earned.delivered, 2);
            assert_eq!(earned.claim.settled, 4);
            assert_eq!(earned.claim.outstanding(), 0);
        }
        assert_eq!(
            a.book().statements(ISSUER, 1, 2).unwrap().issuance_change,
            10
        );
    }
}

fn worker_household(w: &mut World, s: &State, percent: u8) {
    use economics_compute_smoke::{
        household_governance::{Contribution, Governance},
        households,
    };
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::FoundHousehold));
    let mut governance = Governance::contributed(WORKER);
    governance.charter.contribution = Contribution::Percent(percent.into());
    households::form(
        w,
        s,
        households::Agreement {
            id: 1,
            agent: 800,
            governance,
            adults: vec![WORKER],
            membership: vec![],
            asset_sales: vec![],
            equipment_retirements: vec![],
            support: vec![],
            formed: 1,
            dwelling_process: None,
            admission: None,
        },
    )
    .unwrap();
}

#[test]
fn household_member_market_receipts_pool_once_and_own_labor_remains_reserved() {
    for (capacity, percent, expected_mints) in [(5, 20, 1), (2, 50, 0)] {
        let (mut w, s) = minting::scenario("normal").unwrap();
        w.participants
            .iter_mut()
            .find(|p| p.agent == WORKER)
            .unwrap()
            .capacity
            .quantity = capacity;
        worker_household(&mut w, &s, percent);
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        let completed = sim
            .state
            .processes
            .values()
            .filter(|p| p.definition == MINT && p.status == Status::Completed)
            .count();
        assert_eq!(completed, expected_mints);
        let labor = sim
            .ledger
            .iter()
            .filter(|b| b.month == 2)
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .next()
            .unwrap();
        let contribution = &labor.contributions[0];
        assert_eq!(contribution.reserved, 1);
        assert_eq!(sim.state.balance(800, WHEAT), 1);
        assert_eq!(sim.state.balance(WORKER, WHEAT), 2);
        assert_eq!(
            sim.state.balance(800, COIN),
            if expected_mints == 1 { 2 } else { 0 }
        );
        assert_eq!(
            sim.state.balance(WORKER, COIN),
            if expected_mints == 1 { 5 } else { 3 }
        );
        let mut resumed =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::Reference).unwrap();
        let mut report = a.clone();
        while sim.state.month <= 3 {
            a.step(&mut sim).unwrap();
        }
        while resumed.state.month <= 3 {
            report.step(&mut resumed).unwrap();
        }
        assert_eq!(sim.state, resumed.state);
        assert_eq!(a, report);
    }
}

#[test]
fn mint_market_reserves_pooled_storage_before_accepting_a_member_purchase() {
    for generated in [false, true] {
        let (mut w, mut s) = if generated {
            minting::order_scenario("normal")
        } else {
            minting::scenario("normal")
        }
        .unwrap();
        w.storage.capacities.insert(WORKER, 8);
        worker_household(&mut w, &s, 20);
        // Four shared slots are full. A three-unit private receipt fits before
        // pooling, but its mandatory one-unit contribution does not.
        s.balances.insert((800, WHEAT), 4);
        let mut a = audit(&w, &s);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        while sim.state.month <= 1 {
            a.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.balance(WORKER, WHEAT), 0);
        assert_eq!(sim.state.balance(WORKER, COIN), 6);
        assert_eq!(sim.state.balance(800, WHEAT), 4);
        assert_eq!(sim.state.balance(SUPPLIER, WHEAT), 3);
        assert_eq!(sim.state.balance(ISSUER, COIN), 3);
        assert!(
            !sim.state
                .household_remainders
                .contains_key(&(800, WORKER, WHEAT))
        );
        assert_eq!(a.book().statements(ISSUER, 1, 1).unwrap().closing_cash, 3);
        let mint = sim.ledger.iter().find_map(|b| b.minting.as_ref()).unwrap();
        if generated {
            assert!(!mint.deals.iter().any(|d| d.buyer == WORKER));
        } else {
            assert!(!mint.receipts[1].accepted);
        }
    }
}
