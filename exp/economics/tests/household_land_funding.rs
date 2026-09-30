use economics_compute_smoke::{
    accounting::Account as A,
    commitments,
    compute::Backend,
    financial_reporting::{Audit, Opening},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
    model::*,
    negotiation::QuotePolicy,
    scenario::{GRAIN, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};

const SELLER: AgentId = 92;
const CREDITOR: AgentId = 89;
const LAND: u32 = 55000;

fn fixture(enabled: bool, cash: i32, month: u32) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    s.month = month;
    s.balances.clear();
    s.balances.insert((HOME, TOKEN), cash);
    s.balances.insert((SELLER, GRAIN), 4);
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = 0;
        w.storage.capacities.insert(p.agent, 100);
    }
    w.households[0].governance.charter.fund_land_dues = enabled;
    w.assets.push(Asset {
        id: LAND,
        owner: CREDITOR,
        kind: 1,
    });
    w.rights.push(UseRight {
        id: LAND,
        holder: HOME,
        asset: LAND,
        from: 1,
        through: 25,
        output_owner: HOME,
    });
    w.agreements.push(commitments::Agreement {
        id: LAND,
        right: LAND,
        creditor: CREDITOR,
        debtor: HOME,
        activated: 1,
        payment: Amount::new(GRAIN, 4),
    });
    let c = w.town_market.as_mut().unwrap();
    c.traders
        .retain(|e| [HOME, SELLER].contains(&e.trader.agent));
    for e in &mut c.traders {
        e.trader.limit = 2;
        e.trader.opening_quote = 2;
        e.trader.policy = QuotePolicy::Fixed;
    }
    let m = &mut w
        .marketplaces
        .iter_mut()
        .find(|m| m.agent == c.venue)
        .unwrap()
        .markets[0];
    m.goods = Amount::new(GRAIN, 4);
    m.payment = TOKEN;
    m.price_tick = 1;
    (w, s)
}
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            assets: w.assets.iter().map(|a| (a.id, 0)).collect(),
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| *r != TOKEN && **q > 0)
                .map(|(a, q)| (*a, i128::from(*q)))
                .collect(),
            exchange_values: [(GRAIN, 1)].into(),
            dues: Some(economics_compute_smoke::dues_accounting::Valuation(
                [(LAND, 1)].into(),
            )),
            ..Default::default()
        },
    )
    .unwrap()
}
fn through(sim: &mut Simulation, a: &mut Audit, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
fn replay(w: &World, s: &State, sim: &Simulation) {
    let mut state = s.clone();
    for b in &sim.ledger {
        commit(w, &mut state, b, sim.backend, DEFAULT_EFFECT_LIMIT).unwrap();
    }
    assert_eq!(state, sim.state);
}
fn balance(a: &Audit, who: AgentId, account: A) -> i128 {
    a.book()
        .balances()
        .get(&(who, account))
        .copied()
        .unwrap_or(0)
}

#[test]
fn current_land_bills_generate_funded_orders_and_keep_collection_at_clear_arrears() {
    for (enabled, cash, month, expected) in [
        (false, 2, 13, 0),
        (true, 2, 13, 4),
        (true, 0, 13, 0),
        (true, 2, 12, 0),
    ] {
        let (w, s) = fixture(enabled, cash, month);
        let run = |mut w: World, backend| {
            if matches!(backend, Backend::CubeCpu) {
                w.households[0].adults.reverse();
                w.town_market.as_mut().unwrap().traders.reverse();
            }
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            while sim.state.phase != Phase::Productive {
                a.step(&mut sim).unwrap();
            }
            assert_eq!(sim.state.balance(HOME, GRAIN), expected);
            assert_eq!(
                sim.state
                    .obligations
                    .get(&(LAND, 13))
                    .map(|o| o.paid)
                    .unwrap_or(0),
                0
            );
            if month == 13 {
                assert_eq!(balance(&a, HOME, A::DuesPayable(LAND, 13)), -4);
            }
            let (mut resumed, mut ra) = (sim.clone(), a.clone());
            through(&mut sim, &mut a, month);
            through(&mut resumed, &mut ra, month);
            assert_eq!(sim.state.balance(CREDITOR, GRAIN), expected);
            assert_eq!(sim.state.balance(HOME, GRAIN), 0);
            assert_eq!(
                (sim.state.clone(), sim.ledger.clone(), a.clone()),
                (resumed.state, resumed.ledger, ra)
            );
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(w.clone(), Backend::Reference), run(w, Backend::CubeCpu));
    }
}

#[test]
fn acquired_rent_stock_is_retained_without_duplicate_buying_when_delivery_is_blocked() {
    let (mut w, mut s) = fixture(true, 4, 13);
    s.balances.insert((SELLER, GRAIN), 8);
    w.storage.capacities.insert(CREDITOR, 0);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = audit(&w, &s);
        through(&mut sim, &mut a, 14);
        assert_eq!(sim.state.balance(HOME, GRAIN), 4);
        assert_eq!(sim.state.balance(HOME, TOKEN), 2);
        assert_eq!(sim.state.obligations[&(LAND, 13)].paid, 0);
        assert_eq!(balance(&a, HOME, A::DuesPayable(LAND, 13)), -4);
        replay(&w, &s, &sim);
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

fn alternative(w: &mut World) {
    w.households[0].governance.charter.land_tender =
        commitments::TenderPreference::AcceptedAlternativeFirst;
    w.activities.coin_payments.insert(
        LAND,
        economics_compute_smoke::activities::CoinPayment {
            resource: TOKEN,
            coins_per_unit: 2,
        },
    );
}

#[test]
fn tender_preference_changes_delivery_not_the_native_bill_and_uses_whole_units() {
    for (preferred, cash, accepted, native, coins) in [
        (false, 8, true, 4, 0),
        (true, 8, true, 0, 8),
        (true, 3, true, 3, 2),
        (true, 8, false, 4, 0),
    ] {
        let (mut w, mut s) = fixture(false, cash, 13);
        w.town_market = None;
        s.town_market = Default::default();
        s.balances.insert((HOME, GRAIN), 4);
        alternative(&mut w);
        if !preferred {
            w.households[0].governance.charter.land_tender = Default::default();
        }
        if !accepted {
            w.activities.coin_payments.clear();
        }
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut sim, &mut a, 13);
            assert_eq!(sim.state.balance(CREDITOR, GRAIN), native);
            assert_eq!(sim.state.balance(CREDITOR, TOKEN), coins);
            let bill = &sim.state.obligations[&(LAND, 13)];
            assert_eq!((bill.owed, bill.paid, bill.in_kind_paid), (4, 4, native));
            assert_eq!(balance(&a, HOME, A::DuesPayable(LAND, 13)), 0);
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn market_can_buy_accepted_tender_for_rent_without_creating_a_second_claim() {
    const BARTER: ResourceId = 900;
    for (enabled, accepted, cash, paid) in [
        (true, true, 4, 4),
        (false, true, 2, 0),
        (true, false, 2, 0),
        (true, true, 0, 0),
    ] {
        let (mut w, mut s) = fixture(enabled, 0, 13);
        alternative(&mut w);
        if !accepted {
            w.activities.coin_payments.clear();
        }
        w.resources.push(Resource {
            id: BARTER,
            name: "barter stock".into(),
            kind: ResourceKind::Stock,
        });
        w.storage.weights.insert(BARTER, 1);
        s.balances.insert((HOME, BARTER), cash);
        s.balances.insert((SELLER, TOKEN), 16);
        let venue = w.town_market.as_ref().unwrap().venue;
        let m = &mut w
            .marketplaces
            .iter_mut()
            .find(|m| m.agent == venue)
            .unwrap()
            .markets[0];
        m.goods = Amount::new(TOKEN, 8);
        m.payment = BARTER;
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            while sim.state.phase != Phase::Productive {
                a.step(&mut sim).unwrap();
            }
            assert_eq!(sim.state.balance(HOME, TOKEN), paid * 2);
            assert_eq!(sim.state.obligations[&(LAND, 13)].paid, 0);
            let (mut resumed, mut ra) = (sim.clone(), a.clone());
            through(&mut sim, &mut a, 14);
            through(&mut resumed, &mut ra, 14);
            assert_eq!(sim.state.balance(CREDITOR, TOKEN), paid * 2);
            assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            assert_eq!(sim.state.obligations[&(LAND, 13)].paid, paid);
            assert_eq!(sim.state.obligations[&(LAND, 13)].in_kind_paid, 0);
            assert_eq!(
                (sim.state.clone(), sim.ledger.clone(), a.clone()),
                (resumed.state, resumed.ledger, ra)
            );
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn preferred_coin_rent_shares_finite_currency_with_loans_before_native_fallback() {
    use economics_compute_smoke::{
        credit::{Advance, LoanOffer},
        finance::CollectionPolicy,
        opportunities::{Action, HOUSEHOLD_TYPE, PERSON_TYPE},
    };
    for (proportional, priority) in [(false, 0), (true, 0), (true, 1)] {
        let (mut w, mut s) = fixture(false, 0, 12);
        w.town_market = None;
        s.town_market = Default::default();
        alternative(&mut w);
        w.transaction_policy.as_mut().unwrap().permissions.extend([
            (HOUSEHOLD_TYPE, Action::Borrow),
            (PERSON_TYPE, Action::Lend),
        ]);
        s.balances.insert((SELLER, TOKEN), 4);
        w.lending.push(Advance {
            id: 1,
            debtor: HOME,
            principal: 4,
            month: 12,
            priority,
            collateral: None,
            terms: LoanOffer {
                creditor: SELLER,
                denomination: TOKEN,
                max_principal: 4,
                monthly_rate_bps: 0,
                term_months: 1,
                grace_months: 12,
            },
        });
        if proportional {
            w.collection_policy = CollectionPolicy::Proportional;
        }
        let mut origin = Simulation::new(w, s, Backend::Reference).unwrap();
        origin.run_months(1).unwrap();
        let (w, mut s) = (origin.world, origin.state);
        assert_eq!(s.credit.loans[&1].principal, 4);
        // Explicit opening endowment: eight coins face twelve coins of demand.
        s.balances.insert((HOME, TOKEN), 8);
        s.balances.insert((HOME, GRAIN), 4);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            while sim.state.phase != Phase::Acquire {
                a.step(&mut sim).unwrap();
            }
            let rent = &sim.state.obligations[&(LAND, 13)];
            assert_eq!(rent.paid, 4);
            assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            let rent_coins = sim.state.balance(CREDITOR, TOKEN);
            let loan_coins = sim.state.balance(SELLER, TOKEN);
            assert_eq!(rent_coins + loan_coins, 8);
            assert_eq!(rent.in_kind_paid * 2 + rent_coins, 8);
            if proportional && priority == 0 {
                assert!(rent_coins > 0 && loan_coins > 0);
                assert!(rent.in_kind_paid > 0);
            } else if priority > 0 {
                assert_eq!((rent_coins, loan_coins, rent.in_kind_paid), (8, 0, 0));
            }
            through(&mut sim, &mut a, 13);
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn voluntary_support_funds_only_the_selected_accepted_tender() {
    use economics_compute_smoke::{households::support, scenario::PERSON};
    for (preferred, paid) in [(false, 0), (true, 4)] {
        let (mut w, mut s) = fixture(false, 0, 13);
        w.town_market = None;
        s.town_market = Default::default();
        alternative(&mut w);
        if !preferred {
            w.households[0].governance.charter.land_tender = Default::default();
        }
        w.households[0].governance.charter.accept_payment_support = true;
        s.balances.insert((PERSON, TOKEN), 20);
        support::authorize(
            &mut w,
            &s,
            HOME,
            PERSON,
            support::Mandate {
                member: PERSON,
                resource: TOKEN,
                from: 13,
                through: 14,
                revoked_from: None,
                reserve_months: 1,
                private_reserve: 2,
                household_target: 20,
                monthly_limit: 20,
            },
        )
        .unwrap();
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            through(&mut sim, &mut a, 14);
            assert_eq!(sim.state.balance(PERSON, TOKEN), 20 - paid * 2);
            assert_eq!(sim.state.balance(CREDITOR, TOKEN), paid * 2);
            assert_eq!(sim.state.balance(HOME, TOKEN), 0);
            assert_eq!(sim.state.obligations[&(LAND, 13)].paid, paid);
            assert_eq!(
                balance(&a, PERSON, A::TransferExpense),
                i128::from(paid * 2)
            );
            assert_eq!(balance(&a, HOME, A::TransferIncome), -i128::from(paid * 2));
            replay(&w, &s, &sim);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
