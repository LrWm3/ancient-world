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
        let proceeds = sim.state.balance(PERSON, TOKEN) + sim.state.balance(HOME, TOKEN);
        assert_eq!(
            sim.state.balance(HOME, TOKEN),
            proceeds / households::POOL_DIVISOR
        );
        assert_eq!(
            sim.state.household_remainders[&(HOME, PERSON, TOKEN)],
            proceeds % households::POOL_DIVISOR
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
fn private_sales_do_not_enable_duplicate_buying() {
    let (mut w, s) = private_sales_fixture();
    w.town_market.as_mut().unwrap().adaptive = true;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Acquire {
        sim.step().unwrap();
    }
    sim.step().unwrap();
    let round = &sim.state.town_market.history[0];
    assert!(!round.orders.iter().any(|o|o.agent==PERSON && o.side==economics_compute_smoke::marketplace::Side::Buy));
    assert!(
        round.orders.iter().any(
            |o| o.agent == PERSON && o.side == economics_compute_smoke::marketplace::Side::Sell
        )
    );
}

#[test]
fn cash_target_stops_extra_income_work_and_resumes_below_the_buffer() {
    use economics_compute_smoke::households::income::scenario;
    let run = |target: Option<i32>| {
        let (mut w, s) = scenario::scenario().unwrap();
        w.households[0].governance.charter.cash_target = target.map(|q| Amount::new(TOKEN, q));
        let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
        while sim.state.phase != Phase::Productive {
            sim.step().unwrap();
        }
        sim.step().unwrap();
        sim.ledger.last().unwrap().household.as_ref().unwrap().labor[0].clone()
    };
    assert_eq!(run(None).granted, 2);
    let rested = run(Some(20));
    assert_eq!(rested.granted, 0);
    assert!(
        rested
            .contributions
            .iter()
            .all(|c| c.directed == 0 && c.returned == c.reserved)
    );
    assert_eq!(run(Some(60)).granted, 2);
}

fn lend(w: &mut World, debtor: AgentId, creditor: AgentId, principal: i32) {
    use economics_compute_smoke::{
        credit::{Advance, LoanOffer},
        opportunities::{Action, HOUSEHOLD_TYPE, PERSON_TYPE},
    };
    let law = w.transaction_policy.as_mut().unwrap();
    for kind in [PERSON_TYPE, HOUSEHOLD_TYPE] {
        for action in [Action::Borrow, Action::Lend] {
            law.permissions.insert((kind, action));
        }
    }
    w.lending.push(Advance {
        id: 1,
        debtor,
        principal,
        month: 1,
        priority: 0,
        collateral: None,
        terms: LoanOffer {
            creditor,
            denomination: TOKEN,
            max_principal: principal,
            monthly_rate_bps: 0,
            term_months: 2,
            grace_months: 3,
        },
    });
}

#[test]
fn household_loan_and_town_purchase_share_budget_and_repay_on_separate_books() {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.activities.orders.clear();
    s.balances.insert((HOME, TOKEN), 0);
    s.balances.insert((89, TOKEN), 80);
    lend(&mut w, HOME, 89, 80);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut audit = inventory_audit(&w, &s);
        while sim.state.phase != Phase::Acquire {
            audit.step(&mut sim).unwrap();
        }
        let opening = sim.state.clone();
        audit.step(&mut sim).unwrap();
        assert_eq!(sim.state.balance(HOME, TOKEN), 80);
        assert_eq!(sim.state.balance(HOME, GRAIN), 0);
        assert_eq!(sim.state.credit.loans[&1].principal, 80);
        let mut forged = sim.ledger.last().unwrap().clone();
        forged.transactions.pop();
        let mut unchanged = opening.clone();
        assert!(commit(&w, &mut unchanged, &forged, backend, DEFAULT_EFFECT_LIMIT).is_err());
        assert_eq!(unchanged, opening);
        while sim.state.month <= 2 {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.credit.loans[&1].principal, 40);
        assert!(
            sim.state.town_market.history[1]
                .markets
                .values()
                .any(|m| m.volume > 0)
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
fn lender_cannot_spend_the_same_coins_on_a_town_purchase() {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.activities.orders.clear();
    s.balances.insert((HOME, TOKEN), 40);
    lend(&mut w, 89, HOME, 40);
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut audit = inventory_audit(&w, &s);
    while sim.state.phase != Phase::Productive {
        audit.step(&mut sim).unwrap();
    }
    assert_eq!(sim.state.balance(HOME, TOKEN), 0);
    assert_eq!(sim.state.balance(HOME, GRAIN), 0);
    assert_eq!(sim.state.credit.loans[&1].principal, 40);
}

fn combined_fixture() -> (World, State) {
    use economics_compute_smoke::{
        employment::{ArrearsPolicy, Terms},
        marketplace::Side,
        opportunities::{Action, PERSON_TYPE},
        scenario::{FUEL, LABOR},
    };
    let (mut w, mut s) = households::income::scenario::coordinated().unwrap();
    w.households[0].governance.charter.cash_target = Some(Amount::new(TOKEN, 60));
    let c = w.town_market.as_mut().unwrap();
    let add = |entries: &mut Vec<economics_compute_smoke::town_market::Entry>| {
        let mut entry = entries
            .iter()
            .find(|e| e.side == Side::Sell)
            .unwrap()
            .clone();
        entry.trader.agent = PERSON;
        entries.push(entry);
    };
    add(&mut c.traders);
    for l in &mut c.additional {
        add(&mut l.traders);
        // This member's private fuel ask does not cross current demand; the
        // collective can still accept voluntary stock under its own price policy.
        let private = l
            .traders
            .iter_mut()
            .find(|e| e.trader.agent == PERSON)
            .unwrap();
        private.trader.limit = 80;
        private.trader.opening_quote = 80;
    }
    s.balances.insert((PERSON, GRAIN), 4);
    s.balances.insert((PERSON, FUEL), 4);
    s.balances.insert((92, TOKEN), 50);
    lend(&mut w, HOME, 92, 20);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::CapacityTrade));
    w.employment.push(Terms {
        id: 1,
        employer: 89,
        worker: 91,
        from: 1,
        through: 12,
        capacity: Amount::new(LABOR, 5),
        wage_per_unit: Amount::new(TOKEN, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    w.capacity_overrides.insert((3, 91), 0);
    (w, s)
}

fn advance_combined(
    sim: &mut Simulation,
    audit: &mut economics_compute_smoke::financial_reporting::Audit,
    through: u32,
) {
    while sim.state.month <= through {
        if sim.state.phase == Phase::Open {
            if sim.state.month == 5 {
                households::membership::leave(&mut sim.world, &sim.state, HOME, 91).unwrap();
            }
            if sim.state.month == 7 {
                households::membership::join(
                    &mut sim.world,
                    &sim.state,
                    HOME,
                    91,
                    vec![PERSON, 91],
                )
                .unwrap();
            }
        }
        audit
            .step(sim)
            .unwrap_or_else(|e| panic!("month {} {:?}: {e}", sim.state.month, sim.state.phase));
    }
}

#[test]
fn household_loop_combines_wages_private_trade_loans_support_and_changing_membership() {
    use economics_compute_smoke::financial_reporting::{Audit, Opening};
    let (w, s) = combined_fixture();
    let coins = |state: &State| {
        state
            .balances
            .iter()
            .filter(|((_, r), _)| *r == TOKEN)
            .map(|(_, q)| i64::from(*q))
            .sum::<i64>()
    };
    let opening_coins = coins(&s);
    let initial = Audit::with_opening(
        &w,
        &s,
        TOKEN,
        Opening {
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| *r != TOKEN && **q > 0)
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            processes: Some(households::income::scenario::costs()),
            ..Default::default()
        },
    )
    .unwrap();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut audit = initial.clone();
    advance_combined(&mut reference, &mut audit, 4);
    let mut resumed = reference.clone();
    let mut batched = reference.clone();
    households::membership::leave(&mut batched.world, &batched.state, HOME, 91).unwrap();
    batched.run_months(2).unwrap();
    households::membership::join(
        &mut batched.world,
        &batched.state,
        HOME,
        91,
        vec![PERSON, 91],
    )
    .unwrap();
    batched.run_months(2).unwrap();
    let mut resumed_audit = audit.clone();
    advance_combined(&mut reference, &mut audit, 8);
    advance_combined(&mut resumed, &mut resumed_audit, 8);
    assert_eq!(reference.state, resumed.state);
    assert_eq!(reference.state, batched.state);
    assert_eq!(reference.ledger, batched.ledger);
    assert_eq!(reference.ledger, resumed.ledger);
    assert_eq!(audit, resumed_audit);
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut cpu_audit = initial;
    advance_combined(&mut cpu, &mut cpu_audit, 8);
    assert_eq!(reference.state, cpu.state);
    assert_eq!(reference.ledger, cpu.ledger);
    assert_eq!(audit, cpu_audit);
    assert_eq!(coins(&cpu.state), opening_coins);
    assert_eq!(
        cpu.state.credit.loans[&1].status,
        economics_compute_smoke::credit::Status::Repaid
    );
    assert!(!cpu.state.employment.earned.contains_key(&(1, 3))); // actual capacity shock
    assert!(cpu.state.employment.earned.contains_key(&(1, 4))); // capacity recovers
    assert_eq!(cpu.state.employment.earned[&(1, 5)].delivered, 5); // no household commitment after exit
    assert_eq!(cpu.state.employment.earned[&(1, 7)].delivered, 4); // entitlement resumes on accession
    assert!(
        cpu.state
            .town_market
            .history
            .iter()
            .flat_map(|r| &r.attempts)
            .any(|a| a.session.seller.agent == PERSON
                && matches!(
                    a.round.outcome,
                    economics_compute_smoke::negotiation::Outcome::Traded { .. }
                ))
    );
    let decisions: Vec<_> = cpu
        .ledger
        .iter()
        .filter_map(|b| b.household.as_ref())
        .flat_map(|h| &h.labor)
        .collect();
    assert!(decisions.iter().any(|d| d.granted > 0));
    assert!(decisions.iter().all(|d| {
        d.contributions
            .iter()
            .all(|c| c.directed + c.returned == c.reserved)
    }));
    assert!(
        cpu.ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.support)
            .any(|r| r.accepted > 0)
    );
    for batch in cpu
        .ledger
        .iter()
        .filter(|b| b.phase == Phase::Close && [5, 6, 7].contains(&b.month))
    {
        let pooled = batch
            .household
            .as_ref()
            .unwrap()
            .after
            .iter()
            .any(|e| e.account == (91, TOKEN) && e.delta < 0);
        assert_eq!(
            pooled,
            batch.month == 7,
            "income pooling follows current membership"
        );
    }
    for a in &cpu.world.agents {
        let f = audit.book().statements(a.id, 1, 8).unwrap();
        assert_eq!(f.assets, f.liabilities + f.equity);
    }
}

#[test]
fn employment_and_household_collection_share_one_effect_limit() {
    let (w, s) = wage_fixture(160);
    let mut sim = Simulation::new(w.clone(), s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Close {
        sim.step().unwrap();
    }
    let before = sim.state.clone();
    sim.step().unwrap();
    let batch = sim.ledger.last().unwrap();
    let core = batch
        .all_transactions()
        .map(|t| t.effects.len())
        .sum::<usize>();
    assert!(!batch.household.as_ref().unwrap().after.is_empty());
    let mut unchanged = before.clone();
    assert!(
        commit(&w, &mut unchanged, batch, Backend::Reference, core)
            .unwrap_err()
            .contains("effect buffer")
    );
    assert_eq!(unchanged, before);
}

#[test]
fn earned_wage_arrears_are_protected_before_discretionary_town_purchases() {
    use economics_compute_smoke::marketplace::Side;
    let (w, s) = wage_fixture(3);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    sim.run_months(1).unwrap();
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 77);
    // A new opening-state control: the employer has received 40 coins, but
    // still owes 77. Give it food demand competing with the earned wage claim.
    sim.state.balances.insert((89, TOKEN), 40);
    sim.state.balances.insert((89, GRAIN), 0);
    let needs = sim
        .world
        .participants
        .iter()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .needs
        .clone();
    sim.world
        .participants
        .iter_mut()
        .find(|p| p.agent == 89)
        .unwrap()
        .needs = needs;
    let e = sim
        .world
        .town_market
        .as_mut()
        .unwrap()
        .traders
        .iter_mut()
        .find(|e| e.trader.agent == 89)
        .unwrap();
    e.side = Side::Buy;
    e.trader.limit = 40;
    e.trader.opening_quote = 40;
    let mut audit = inventory_audit(&sim.world, &sim.state);
    while sim.state.phase != Phase::Productive {
        audit.step(&mut sim).unwrap();
    }
    assert_eq!(sim.state.balance(89, TOKEN), 40);
    assert_eq!(sim.state.balance(89, GRAIN), 0);
    while sim.state.month == 2 {
        audit.step(&mut sim).unwrap();
    }
    assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 37);
}
