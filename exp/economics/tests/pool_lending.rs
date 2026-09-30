use economics_compute_smoke::{
    allocation::Policy,
    compute::Backend,
    credit::{Advance, LoanOffer},
    financial_reporting::{Audit, Opening},
    model::*,
    pool_market,
    scenario::*,
    simulation::Simulation,
};

fn fixture(funded: bool) -> (World, State) {
    let (mut w, mut s) = pool_market::scenario("ample", Policy::PriorityLottery).unwrap();
    w.competition = None;
    w.access_offers.clear();
    w.open_access_offers.clear();
    w.agreements.clear();
    w.rights.clear();
    w.assets.clear();
    w.transaction_policy = None;
    w.agent_search.clear();
    w.decision_horizon = None;
    w.priority = Priority::ContinuingFirst;
    w.horizon = 1;
    w.resources.push(Resource {
        id: TOKEN,
        name: "reporting coin".into(),
        kind: ResourceKind::Stock,
    });
    w.condition_rules.clear();
    w.definitions
        .retain(|d| [PREPARE_FUEL, USE_FUEL].contains(&d.id));
    for p in &mut w.participants {
        p.needs.retain(|n| n.resource == WARMTH);
    }
    s.processes.clear();
    s.accepted_agreements.clear();
    s.obligations.clear();
    s.memberships.clear();
    s.conditions.clear();
    s.balances.clear();
    s.balances
        .insert((STATE_AGENT, FUEL), if funded { 4 } else { 0 });
    w.lending.push(Advance {
        id: 10,
        debtor: PERSON,
        principal: 4,
        month: 2,
        collateral: None,
        priority: 0,
        terms: LoanOffer {
            creditor: STATE_AGENT,
            denomination: FUEL,
            max_principal: 4,
            monthly_rate_bps: 0,
            term_months: 2,
            grace_months: 12,
        },
    });
    (w, s)
}

#[test]
fn commodity_credit_changes_collection_demand_but_not_the_shared_environment_budget() {
    for funded in [false, true] {
        let (w, s) = fixture(funded);
        let run = |backend| {
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    inventory: s
                        .balances
                        .iter()
                        .filter(|(_, q)| **q > 0)
                        .map(|(k, q)| (*k, i128::from(*q)))
                        .collect(),
                    exchange_values: [(FUEL, 1)].into(),
                    processes: Some(Default::default()),
                    ..Opening::default()
                },
            )
            .unwrap();
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.phase != Phase::Productive {
                audit.step(&mut sim).unwrap();
            }
            assert_eq!(sim.state.balance(PERSON, FUEL), if funded { 4 } else { 0 });
            assert_eq!(sim.state.balance(STATE_AGENT, RAW_WOOD), 4);
            let (before, mut resumed_audit) = (sim.clone(), audit.clone());
            audit.step(&mut sim).unwrap();
            let round = sim.ledger.last().unwrap().pool_market.as_ref().unwrap();
            let demand = round.demands.iter().find(|d| d.agent == PERSON);
            if funded {
                assert!(demand.is_none_or(|d| d.requested == 0));
            } else {
                assert!(demand.is_some_and(|d| d.requested > 0));
            }
            assert!(sim.state.balance(STATE_AGENT, RAW_WOOD) >= 0);
            assert_eq!(round.available_stock, 4);
            let mut resumed = before;
            let end = sim.state.month + 4;
            while sim.state.month < end {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month < end {
                resumed_audit.step(&mut resumed).unwrap();
            }
            assert_eq!(audit, resumed_audit);
            for agent in &w.agents {
                let report = audit.book().statements(agent.id, 2, end - 1).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            assert_eq!(sim.state, resumed.state);
            assert_eq!(sim.ledger, resumed.ledger);
            if funded {
                let loan = &sim.state.credit.loans[&10];
                assert_eq!(loan.denomination, FUEL);
                assert!(loan.principal < 4);
            } else {
                assert!(sim.state.credit.loans.is_empty());
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn refused_storage_and_same_window_relending_do_not_expand_collection_resources() {
    for blocked_storage in [false, true] {
        let (mut w, mut s) = fixture(true);
        w.storage.weights.insert(FUEL, 1);
        w.storage
            .capacities
            .insert(PERSON, if blocked_storage { 0 } else { 10 });
        w.storage.capacities.insert(STATE_AGENT, 20);
        let mut onward = w.lending[0].clone();
        onward.id = 11;
        onward.terms.creditor = PERSON;
        onward.debtor = PERSON + 1;
        w.lending.push(onward);
        s.balances.insert((PERSON, FUEL), 0);
        let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        while sim.state.phase != Phase::Productive {
            sim.step().unwrap();
        }
        assert_eq!(sim.state.credit.loans.contains_key(&10), !blocked_storage);
        assert!(!sim.state.credit.loans.contains_key(&11));
        assert_eq!(sim.state.balance(PERSON + 1, FUEL), 0);
        assert_eq!(sim.state.balance(STATE_AGENT, RAW_WOOD), 4);
        sim.step().unwrap();
        assert!(sim.state.balance(STATE_AGENT, RAW_WOOD) >= 0);
    }
}

#[test]
fn collection_reserves_for_accepted_repayments_inside_its_need_horizon() {
    use economics_compute_smoke::credit;
    let (mut w, s) = fixture(true);
    w.horizon = 3;
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    let before = sim.clone();
    assert_eq!(
        credit::projection::dues(&sim.world, &sim.state, PERSON, FUEL, 1).unwrap(),
        Default::default()
    );
    assert_eq!(
        credit::projection::dues(&sim.world, &sim.state, PERSON, FUEL, 3).unwrap(),
        [(3, 2), (4, 2)].into()
    );
    let with_debt = pool_market::demand(&sim.world, &sim.state, PERSON)
        .unwrap()
        .0;
    let mut unencumbered = sim.state.clone();
    unencumbered.credit.loans.clear();
    let without_debt = pool_market::demand(&sim.world, &unencumbered, PERSON)
        .unwrap()
        .0;
    assert!(with_debt > without_debt);
    assert_eq!(sim.state, before.state);
    assert_eq!(sim.ledger, before.ledger);
    sim.step().unwrap();
    assert!(
        sim.ledger
            .last()
            .unwrap()
            .pool_market
            .as_ref()
            .unwrap()
            .demands
            .iter()
            .any(|d| d.agent == PERSON && d.requested > 0)
    );
}

#[test]
fn repayment_projection_accrues_once_reduces_future_interest_and_ignores_offers() {
    use economics_compute_smoke::credit;
    let (mut w, s) = fixture(true);
    w.lending[0].terms.monthly_rate_bps = 2500;
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    assert!(
        credit::projection::dues(&sim.world, &sim.state, PERSON, FUEL, 4)
            .unwrap()
            .is_empty()
    );
    while sim.state.phase != Phase::Productive {
        sim.step().unwrap();
    }
    assert_eq!(
        credit::projection::dues(&sim.world, &sim.state, PERSON, FUEL, 4).unwrap(),
        [(3, 3), (4, 2)].into()
    );
    assert!(
        credit::projection::dues(&sim.world, &sim.state, STATE_AGENT, FUEL, 4)
            .unwrap()
            .is_empty()
    );
    sim.run_months(1).unwrap();
    sim.step().unwrap(); // Open
    let before_due = credit::projection::dues(&sim.world, &sim.state, PERSON, FUEL, 2).unwrap();
    sim.step().unwrap(); // Due accrues once, actual funding may leave arrears.
    let after_due = credit::projection::dues(&sim.world, &sim.state, PERSON, FUEL, 2).unwrap();
    let payment = sim
        .ledger
        .last()
        .unwrap()
        .credit
        .as_ref()
        .unwrap()
        .collections[0]
        .paid;
    assert_eq!(
        before_due.values().sum::<i128>(),
        after_due.values().sum::<i128>() + i128::from(payment)
    );
}
