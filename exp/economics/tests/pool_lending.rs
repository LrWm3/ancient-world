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

#[test]
fn household_governance_pooling_and_member_credit_share_the_collection_boundary() {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, Agreement},
    };
    const HOME: AgentId = 10000;
    let (mut w, s) = fixture(true);
    for p in &mut w.participants {
        p.capacity.quantity = if p.agent == PERSON { 10 } else { 1 };
    }
    w.definitions
        .iter_mut()
        .find(|d| d.id == PREPARE_FUEL)
        .unwrap()
        .stages[0]
        .monthly_services[0]
        .quantity = 3;
    let mut governance = Governance::contributed(PERSON);
    governance.charter.initial_policy =
        economics_compute_smoke::household_governance::Policy::NeedsFirst;
    households::form(
        &mut w,
        &s,
        Agreement {
            id: 1,
            agent: HOME,
            adults: vec![PERSON, PERSON + 1],
            governance,
            formed: s.month,
            dwelling_process: None,
            admission: None,
            membership: vec![],
            asset_sales: vec![],
            equipment_retirements: vec![],
            support: vec![],
        },
    )
    .unwrap();
    let mut output_world = w.clone();
    output_world.households[0].governance.charter.initial_policy =
        economics_compute_smoke::household_governance::Policy::PreserveCommittedWork;
    let mut output_only = Simulation::new(output_world, s.clone(), Backend::Reference).unwrap();
    output_only.run_months(1).unwrap();
    assert!(
        output_only
            .ledger
            .iter()
            .filter_map(|b| b.household.as_ref())
            .flat_map(|h| &h.labor)
            .all(|d| d.granted == 0)
    );
    assert!(
        !output_only
            .state
            .processes
            .values()
            .any(|p| p.definition == PREPARE_FUEL && p.status == Status::Completed)
    );
    let run = |backend| {
        let mut audit = Audit::with_opening(
            &w,
            &s,
            TOKEN,
            Opening {
                inventory: [((STATE_AGENT, FUEL), 4)].into(),
                exchange_values: [(FUEL, 1)].into(),
                processes: Some(Default::default()),
                ..Opening::default()
            },
        )
        .unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while sim.state.month <= 3 {
            audit.step(&mut sim).unwrap();
        }
        let (mut resumed, mut ra) = (sim.clone(), audit.clone());
        while sim.state.month <= 7 {
            audit.step(&mut sim).unwrap();
        }
        while resumed.state.month <= 7 {
            ra.step(&mut resumed).unwrap();
        }
        assert_eq!(
            (&sim.state, &sim.ledger, &audit),
            (&resumed.state, &resumed.ledger, &ra)
        );
        assert_eq!(sim.state.credit.loans[&10].debtor, PERSON);
        assert!(
            sim.ledger
                .iter()
                .filter_map(|b| b.household.as_ref())
                .flat_map(|h| &h.labor)
                .any(|d| d.granted > 0)
        );
        assert!(
            sim.ledger
                .iter()
                .filter_map(|b| b.household.as_ref())
                .flat_map(|h| &h.after)
                .any(|e| e.account == (HOME, FUEL) && e.delta > 0)
        );
        assert!(
            sim.ledger
                .iter()
                .any(|b| b.pool_market.is_some() && b.household.is_some())
        );
        for b in &sim.ledger {
            if let Some(round) = &b.pool_market {
                let spent: i32 = b
                    .transactions
                    .iter()
                    .flat_map(|t| &t.effects)
                    .filter(|e| e.account == (STATE_AGENT, RAW_WOOD) && e.delta < 0)
                    .map(|e| -e.delta)
                    .sum();
                assert!(spent <= round.available_stock);
            }
        }
        for who in [PERSON, PERSON + 1, HOME, STATE_AGENT] {
            let report = audit.book().statements(who, 2, 7).unwrap();
            assert_eq!(report.assets, report.liabilities + report.equity);
        }
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn accepted_prepaid_deliveries_generate_collection_work_without_selling_the_need_buffer() {
    use economics_compute_smoke::{forward::direct::Terms, offers};
    for loan in [false, true] {
        for funded in [false, true] {
            let (mut w, mut s) = fixture(loan);
            if !loan {
                w.lending.clear();
            }
            w.horizon = 2;
            w.prepaid_deliveries.push(Terms {
                id: 20,
                seller: PERSON,
                buyer: STATE_AGENT,
                month: 2,
                due: 3,
                goods: Amount::new(FUEL, 2),
                prepayment: Amount::new(TOKEN, 2),
            });
            s.balances.insert((PERSON, FUEL), if loan { 0 } else { 2 });
            s.balances
                .insert((STATE_AGENT, TOKEN), if funded { 2 } else { 0 });
            let run = |backend| {
                let mut audit = Audit::with_opening(
                    &w,
                    &s,
                    TOKEN,
                    Opening {
                        inventory: if loan {
                            [((STATE_AGENT, FUEL), 4)].into()
                        } else {
                            [((PERSON, FUEL), 2)].into()
                        },
                        exchange_values: [(FUEL, 1)].into(),
                        processes: Some(Default::default()),
                        ..Opening::default()
                    },
                )
                .unwrap();
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while sim.state.phase != Phase::Acquire {
                    audit.step(&mut sim).unwrap();
                }
                let preview = offers::prepare(
                    &sim,
                    &[offers::Request::new(
                        offers::Id::PrepaidDelivery(20),
                        PERSON,
                    )],
                );
                assert_eq!(preview.is_ok(), funded);
                audit.step(&mut sim).unwrap();
                assert_eq!(sim.state.phase, Phase::Productive);
                assert_eq!(sim.state.exchange.forwards.contains_key(&20), funded);
                let (lots, _) = pool_market::demand(&sim.world, &sim.state, PERSON).unwrap();
                assert_eq!(lots > 0, funded);
                let (mut resumed, mut ra) = (sim.clone(), audit.clone());
                while sim.state.month <= 3 {
                    audit.step(&mut sim).unwrap();
                }
                while resumed.state.month <= 3 {
                    ra.step(&mut resumed).unwrap();
                }
                assert_eq!(
                    (&sim.state, &sim.ledger, &audit),
                    (&resumed.state, &resumed.ledger, &ra)
                );
                if funded {
                    assert_eq!(sim.state.exchange.forwards[&20].delivered, 2);
                    assert_eq!(
                        sim.state.balance(STATE_AGENT, FUEL),
                        if loan { 4 } else { 2 }
                    );
                }
                assert_eq!(
                    sim.state
                        .processes
                        .values()
                        .filter(|p| p.operator == PERSON
                            && p.definition == USE_FUEL
                            && p.status == Status::Completed)
                        .count(),
                    2
                );
                for who in [PERSON, PERSON + 1, STATE_AGENT] {
                    let report = audit.book().statements(who, 2, 3).unwrap();
                    assert_eq!(report.assets, report.liabilities + report.equity);
                }
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}
