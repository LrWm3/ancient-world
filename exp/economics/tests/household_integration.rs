use economics_compute_smoke::{
    compute::Backend,
    household_governance::Policy,
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME, support::Mandate},
    model::*,
    scenario::{GRAIN, NUTRITION, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

fn support_fixture() -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    w.households[0].governance.charter.initial_policy = Policy::NeedsFirst;
    w.activities.orders.clear();
    s.balances.insert((HOME, TOKEN), 0);
    s.balances.insert((PERSON, GRAIN), 6);
    w.households[0].support.push(Mandate {
        member: PERSON,
        resource: GRAIN,
        from: 1,
        through: 12,
        revoked_from: None,
        reserve_months: 1,
        private_reserve: 2,
        household_target: 2,
        monthly_limit: 2,
    });
    (w, s)
}

#[test]
fn voluntary_food_support_meets_collective_needs_without_a_market() {
    let (w, s) = support_fixture();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        sim.run_months(1).unwrap();
        let receipts: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|b| &b.support)
            .collect();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].accepted, 2);
        assert!(receipts[0].baseline_income.is_none());
        assert!(
            sim.reports
                .iter()
                .filter(|r| [PERSON, 91].contains(&r.agent))
                .all(|r| r.deficit(NUTRITION) == 0)
        );
        let mut replay = s.clone();
        for b in &sim.ledger {
            commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
        }
        assert_eq!(replay, sim.state);
        (sim.state, sim.ledger)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn support_protects_private_needs_and_rejects_forged_acceptance() {
    let (w, mut s) = support_fixture();
    s.balances.insert((PERSON, GRAIN), 2);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    let opening = sim.state.clone();
    sim.step().unwrap();
    let mut forged = sim.ledger.last().unwrap().clone();
    let r = &mut forged.household.as_mut().unwrap().support[0];
    assert_eq!(r.accepted, 0);
    r.accepted = 1;
    let mut unchanged = opening.clone();
    assert!(
        commit(
            &sim.world,
            &mut unchanged,
            &forged,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(unchanged, opening);
}

fn wage_fixture(cash: i32) -> (World, State) {
    use economics_compute_smoke::{
        employment::{ArrearsPolicy, Terms},
        opportunities::{Action, PERSON_TYPE},
        scenario::LABOR,
    };
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.activities.orders.clear();
    for p in &mut w.participants {
        p.capacity.quantity = 5;
    }
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::CapacityTrade));
    s.balances.insert((HOME, TOKEN), 0);
    s.balances.insert((89, TOKEN), cash);
    for member in [PERSON, 91] {
        s.balances.insert((member, GRAIN), 2);
    }
    w.employment.push(Terms {
        id: 1,
        employer: 89,
        worker: PERSON,
        from: 1,
        through: 3,
        capacity: Amount::new(LABOR, 5),
        wage_per_unit: Amount::new(TOKEN, 20),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    (w, s)
}
fn inventory_audit(w: &World, s: &State) -> economics_compute_smoke::financial_reporting::Audit {
    use economics_compute_smoke::financial_reporting::{Audit, Opening};
    Audit::with_opening(
        w,
        s,
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
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            processes: Some(Default::default()),
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn wages_pool_only_when_paid_and_finance_next_months_collective_food() {
    let (w, s) = wage_fixture(160);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut audit = inventory_audit(&w, &s);
        while sim.state.phase != Phase::Productive {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.employment.earned[&(1, 1)].delivered, 4);
        assert_eq!(sim.state.balance(HOME, TOKEN), 0);
        audit.step(&mut sim).unwrap();
        let c = sim.ledger.last().unwrap().household.as_ref().unwrap().labor[0]
            .contributions
            .iter()
            .find(|c| c.member == PERSON)
            .unwrap();
        assert_eq!(c.reserved, 1);
        while sim.state.month == 1 {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.balance(HOME, TOKEN), 40);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 40);
        while sim.state.month == 2 {
            audit.step(&mut sim).unwrap();
        }
        assert!(
            sim.reports
                .iter()
                .filter(|r| r.month == 2 && [PERSON, 91].contains(&r.agent))
                .all(|r| r.deficit(NUTRITION) == 0)
        );
        for a in &w.agents {
            let f = audit.book().statements(a.id, 1, 2).unwrap();
            assert_eq!(f.assets, f.liabilities + f.equity);
        }
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn unpaid_member_wages_do_not_fund_household_orders_or_resell_promised_labor() {
    let (mut w, s) = wage_fixture(3);
    // Remove the employer's sales income so this is a genuine cash shortage.
    w.town_market = None;
    let mut s = s;
    s.town_market = Default::default();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut audit = inventory_audit(&w, &s);
    while sim.state.month <= 2 {
        audit.step(&mut sim).unwrap();
    }
    assert_eq!(sim.state.balance(HOME, TOKEN), 1);
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 77);
    assert!(!sim.state.employment.earned.contains_key(&(1, 2)));
}

fn private_sales_fixture() -> (World, State) {
    use economics_compute_smoke::marketplace::Side;
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.activities.orders.clear();
    for cap in w.storage.capacities.values_mut() {
        *cap = 64;
    }
    let c = w.town_market.as_mut().unwrap();
    let mut seller = c
        .traders
        .iter()
        .find(|e| e.side == Side::Sell)
        .unwrap()
        .clone();
    seller.trader.agent = PERSON;
    seller.trader.limit = 10;
    seller.trader.opening_quote = 10;
    c.traders.push(seller);
    let buyer = c.traders.iter_mut().find(|e| e.trader.agent == 89).unwrap();
    buyer.side = Side::Buy;
    buyer.trader.limit = 40;
    buyer.trader.opening_quote = 40;
    let needs = w
        .participants
        .iter()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .needs
        .clone();
    w.participants
        .iter_mut()
        .find(|p| p.agent == 89)
        .unwrap()
        .needs = needs;
    s.balances.insert((PERSON, GRAIN), 10);
    s.balances.insert((89, GRAIN), 0);
    s.balances.insert((89, TOKEN), 100);
    s.balances.insert((HOME, TOKEN), 0);
    (w, s)
}

#[test]
fn private_sales_and_collective_orders_share_a_book_without_reusing_pooled_receipts() {
    let (w, s) = private_sales_fixture();
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut audit = inventory_audit(&w, &s);
        while sim.state.phase != Phase::Acquire {
            audit.step(&mut sim).unwrap();
        }
        audit.step(&mut sim).unwrap();
        assert!(sim.state.balance(PERSON, TOKEN) > 0);
        assert_eq!(
            sim.state.balance(PERSON, TOKEN),
            sim.state.balance(HOME, TOKEN)
        );
        assert_eq!(sim.state.balance(HOME, GRAIN), 0); // incoming sale receipts wait for a later book
        assert!(sim.state.balance(PERSON, GRAIN) >= 2);
        assert!(sim.state.balance(89, GRAIN) > 0);
        while sim.state.month <= 2 {
            audit.step(&mut sim).unwrap();
        }
        for a in &w.agents {
            let f = audit.book().statements(a.id, 1, 2).unwrap();
            assert_eq!(f.assets, f.liabilities + f.equity);
        }
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn private_sales_do_not_enable_duplicate_buying_or_unreserved_physical_payment() {
    let (mut w, s) = private_sales_fixture();
    w.town_market.as_mut().unwrap().adaptive = true;
    assert!(Simulation::new(w, s, Backend::Reference).is_err());
    let (mut w, s) = private_sales_fixture();
    w.storage.weights.insert(TOKEN, 1);
    assert!(
        Simulation::new(w, s, Backend::Reference)
            .err()
            .unwrap()
            .contains("storage-free payment")
    );
}
