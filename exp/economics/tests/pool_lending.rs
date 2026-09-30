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

#[test]
fn estate_cash_control_preserves_native_collection_and_household_membership() {
    use economics_compute_smoke::{
        employment::{ArrearsPolicy, Terms},
        household_governance::{Governance, Policy as HouseholdPolicy},
        households::{self, Agreement},
        recovery::{ProceedingTerms, Stage},
    };
    const ESTATE: AgentId = 20000;
    const HOME: AgentId = 10000;
    for household in [false, true] {
        let (mut w, mut s) = fixture(true);
        w.horizon = 3;
        let mut worker = w.participants[1].clone();
        worker.agent = PERSON + 2;
        worker.needs.clear();
        w.participants.push(worker);
        w.agents.push(Agent {
            id: PERSON + 2,
            name: "external worker".into(),
        });
        w.lending[0].principal = 12;
        w.lending[0].terms.max_principal = 12;
        w.lending[0].terms.term_months = 6;
        s.balances.insert((STATE_AGENT, FUEL), 12);
        s.balances.insert((STATE_AGENT, TOKEN), 4);
        w.lending.push(Advance {
            id: 11,
            debtor: PERSON,
            principal: 4,
            month: 2,
            collateral: None,
            priority: 0,
            terms: LoanOffer {
                creditor: STATE_AGENT,
                denomination: TOKEN,
                max_principal: 4,
                monthly_rate_bps: 0,
                term_months: 1,
                grace_months: 12,
            },
        });
        // Borrowed coins buy actual work at Close; the missed installment in
        // month 3 authorizes the configured proceeding at Open in month 4.
        w.employment.push(Terms {
            id: 1,
            employer: PERSON,
            worker: PERSON + 2,
            from: 2,
            through: 2,
            capacity: Amount::new(LABOR, 1),
            wage_per_unit: Amount::new(TOKEN, 4),
            on_arrears: ArrearsPolicy::Continue,
            rank: 0,
        });
        w.agents.push(Agent {
            id: ESTATE,
            name: "coin estate".into(),
        });
        w.recovery.proceedings.push(ProceedingTerms {
            id: 1,
            debtor: PERSON,
            authority: STATE_AGENT,
            estate: ESTATE,
            denomination: TOKEN,
            opening_month: 4,
            earliest_close: 8,
            assets: vec![],
            discharge_deficiency: true,
        });
        if household {
            s.balances.insert((HOME, TOKEN), 2);
            let mut governance = Governance::contributed(PERSON);
            governance.charter.initial_policy = HouseholdPolicy::NeedsFirst;
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
        }
        let mut invalid = w.clone();
        invalid.pool_market.as_mut().unwrap().account.0 = ESTATE;
        for pool in &mut invalid.pool_inputs {
            pool.account.0 = ESTATE;
        }
        assert!(Simulation::new(invalid, s.clone(), Backend::Reference).is_err());
        let run = |backend| {
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    inventory: [((STATE_AGENT, FUEL), 12)].into(),
                    exchange_values: [(FUEL, 1)].into(),
                    processes: Some(Default::default()),
                    ..Opening::default()
                },
            )
            .unwrap();
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.month <= 4 {
                audit.step(&mut sim).unwrap();
            }
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Active
            );
            assert_eq!(
                sim.state.credit.loans[&11].status,
                economics_compute_smoke::credit::Status::Stayed
            );
            let native_at_open = sim.state.credit.loans[&10].principal;
            let (mut resumed, mut ra) = (sim.clone(), audit.clone());
            while sim.state.month <= 12 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month <= 12 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &ra)
            );
            assert!(native_at_open > 0);
            assert_eq!(sim.state.credit.loans[&10].principal, 0);
            assert_eq!(
                sim.state.credit.recovery.proceedings[&1].stage,
                Stage::Closed
            );
            assert_eq!(
                sim.state.credit.loans[&11].status,
                economics_compute_smoke::credit::Status::Discharged
            );
            assert!(
                sim.ledger
                    .iter()
                    .any(|b| b.month >= 4 && b.month < 8 && b.pool_market.is_some())
            );
            assert_eq!(sim.state.balance(ESTATE, FUEL), 0);
            if household {
                assert_eq!(
                    households::parent(&sim.world, &sim.state, PERSON),
                    Some(HOME)
                );
                assert_eq!(sim.state.balance(HOME, TOKEN), 2);
            }
            for a in &sim.world.agents {
                let report = audit.book().statements(a.id, 2, 12).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn household_hiring_requires_both_useful_labor_and_available_environmental_stock() {
    use economics_compute_smoke::{
        employment::{ArrearsPolicy, Terms},
        household_governance::{Governance, Policy as HouseholdPolicy},
        households::{self, Agreement},
        offers::{self, Id, Request},
    };
    const HOME: AgentId = 10000;
    const WORKER: AgentId = PERSON + 2;
    for (wood, cash, expected_hours) in [(4, 6, 3), (0, 6, 0), (4, 0, 0), (1, 6, 0)] {
        let (mut w, mut s) = fixture(false);
        w.lending.clear();
        for pool in &mut w.pools {
            pool.monthly_regeneration = wood;
        }
        let mut worker = w.participants[1].clone();
        worker.agent = WORKER;
        worker.needs.clear();
        worker.capacity.quantity = 3;
        for p in &mut w.participants {
            p.capacity.quantity = 0;
        }
        w.participants.push(worker);
        w.agents.push(Agent {
            id: WORKER,
            name: "outside collector".into(),
        });
        w.definitions
            .iter_mut()
            .find(|d| d.id == PREPARE_FUEL)
            .unwrap()
            .stages[0]
            .monthly_services[0]
            .quantity = 3;
        let mut governance = Governance::contributed(PERSON);
        governance.charter.initial_policy = HouseholdPolicy::NeedsFirst;
        governance.charter.hiring_budget = Some(Amount::new(TOKEN, 6));
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
        s.balances.insert((HOME, TOKEN), cash);
        w.employment.push(Terms {
            id: 1,
            employer: HOME,
            worker: WORKER,
            from: 2,
            through: 2,
            capacity: Amount::new(LABOR, 3),
            wage_per_unit: Amount::new(TOKEN, 2),
            on_arrears: ArrearsPolicy::SuspendDelivery,
            rank: 0,
        });
        w.employment_offers.insert(1);
        let run = |backend| {
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    services: Some(Default::default()),
                    processes: Some(Default::default()),
                    ..Opening::default()
                },
            )
            .unwrap();
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.phase != Phase::Acquire {
                audit.step(&mut sim).unwrap();
            }
            let prepared = offers::prepare(&sim, &[Request::new(Id::Employment(1), HOME)]);
            assert_eq!(prepared.is_ok(), expected_hours > 0);
            let (mut resumed, mut ra) = (sim.clone(), audit.clone());
            while sim.state.month == 2 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month == 2 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &ra)
            );
            assert_eq!(sim.state.balance(WORKER, TOKEN), expected_hours * 2);
            let completed = sim
                .state
                .processes
                .values()
                .filter(|p| p.definition == PREPARE_FUEL && p.status == Status::Completed)
                .count();
            assert_eq!(completed, usize::from(expected_hours > 0));
            assert_eq!(
                sim.state.balance(STATE_AGENT, RAW_WOOD),
                wood - if expected_hours > 0 { 2 } else { 0 }
            );
            for agent in &sim.world.agents {
                let report = audit.book().statements(agent.id, 2, 2).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn financed_need_orders_buy_collection_output_without_spending_same_window_advances() {
    use economics_compute_smoke::{
        marketplace::{MARKETPLACE_TYPE, Market, Marketplace},
        need_orders,
        negotiation::{Outcome, QuotePolicy, Session, Trader},
        opportunities::{self, Action, PERSON_TYPE, STATE_TYPE},
    };
    const VENUE: AgentId = 10001;
    let (mut w, mut s) = fixture(false);
    w.horizon = 3;
    w.lending[0].terms.denomination = TOKEN;
    w.lending[0].terms.term_months = 8;
    w.participants
        .iter_mut()
        .find(|p| p.agent == PERSON)
        .unwrap()
        .capacity
        .quantity = 0;
    s.balances.insert((STATE_AGENT, TOKEN), 4);
    s.balances.insert((PERSON + 1, FUEL), 4);
    w.agents.push(Agent {
        id: VENUE,
        name: "fuel market".into(),
    });
    w.transaction_policy = Some(opportunities::Policy {
        authority: STATE_AGENT,
        laws: vec![],
        agreement_forms: None,
        agreement_limits: Default::default(),
        membership_offers: vec![],
        membership_permissions: Default::default(),
        agent_types: [
            (PERSON, PERSON_TYPE),
            (PERSON + 1, PERSON_TYPE),
            (STATE_AGENT, STATE_TYPE),
            (VENUE, MARKETPLACE_TYPE),
        ]
        .into(),
        permissions: [
            (PERSON_TYPE, Action::StockTrade),
            (PERSON_TYPE, Action::Borrow),
            (STATE_TYPE, Action::Lend),
            (PERSON_TYPE, Action::Process(PREPARE_FUEL)),
            (PERSON_TYPE, Action::Process(USE_FUEL)),
        ]
        .into(),
    });
    w.marketplaces.push(Marketplace {
        agent: VENUE,
        allowed_types: [PERSON_TYPE].into(),
        markets: vec![Market {
            id: 1,
            goods: Amount::new(FUEL, 1),
            payment: TOKEN,
            price_tick: 1,
        }],
    });
    w.negotiation = Some(Session {
        marketplace: VENUE,
        market: 1,
        month: 2,
        max_rounds: 1,
        buyer: Trader {
            agent: PERSON,
            limit: 1,
            opening_quote: 1,
            policy: QuotePolicy::Fixed,
        },
        seller: Trader {
            agent: PERSON + 1,
            limit: 1,
            opening_quote: 1,
            policy: QuotePolicy::Fixed,
        },
        goods: Amount::new(FUEL, 1),
        payment: TOKEN,
    });
    w.need_orders = Some(need_orders::Policy { reserve_months: 1 });
    let run = |backend| {
        let mut audit = Audit::with_opening(
            &w,
            &s,
            TOKEN,
            Opening {
                inventory: [((PERSON + 1, FUEL), 4)].into(),
                processes: Some(Default::default()),
                ..Opening::default()
            },
        )
        .unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while sim.state.month == 2 {
            audit.step(&mut sim).unwrap();
        }
        let first = sim
            .ledger
            .iter()
            .find_map(|b| b.negotiation.as_ref())
            .unwrap();
        assert_eq!(first.outcome, Outcome::InsufficientPayment);
        assert_eq!(sim.state.balance(PERSON, TOKEN), 4);
        let (mut resumed, mut ra) = (sim.clone(), audit.clone());
        while sim.state.month <= 6 {
            audit.step(&mut sim).unwrap();
        }
        while resumed.state.month <= 6 {
            ra.step(&mut resumed).unwrap();
        }
        assert_eq!(
            (&sim.state, &sim.ledger, &audit),
            (&resumed.state, &resumed.ledger, &ra)
        );
        assert!(
            sim.ledger
                .iter()
                .filter_map(|b| b.negotiation.as_ref())
                .any(|r| matches!(r.outcome, Outcome::Traded { price: 1 }))
        );
        assert!(
            sim.state
                .processes
                .values()
                .any(|p| p.operator == PERSON + 1
                    && p.definition == PREPARE_FUEL
                    && p.status == Status::Completed)
        );
        assert!(
            sim.reports
                .iter()
                .any(|r| r.agent == PERSON && r.month > 2 && r.deficit(WARMTH) == 0)
        );
        assert!(sim.state.credit.loans[&10].principal < 4);
        assert_eq!(sim.state.balance(PERSON, FUEL), 0);
        assert!(sim.state.balance(PERSON + 1, TOKEN) > 0);
        for a in &sim.world.agents {
            let report = audit.book().statements(a.id, 2, 6).unwrap();
            assert_eq!(report.assets, report.liabilities + report.equity);
        }
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn mortgage_default_does_not_confiscate_independent_environmental_work() {
    use economics_compute_smoke::{
        credit::{self, CollateralSettlement},
        household_governance::Governance,
        households::{self, Agreement},
    };
    const HOME: AgentId = 10000;
    for household in [false, true] {
        for downpayment in [1, 2] {
            let (mut w, mut s) = fixture(false);
            w.lending.clear();
            let borrower = if household { HOME } else { PERSON };
            if household {
                households::form(
                    &mut w,
                    &s,
                    Agreement {
                        id: 1,
                        agent: HOME,
                        adults: vec![PERSON, PERSON + 1],
                        governance: Governance::contributed(PERSON),
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
            }
            let (mortgage, _) = credit::scenario("default").unwrap();
            w.assets = mortgage.assets;
            let mut config = mortgage.credit.unwrap();
            config.endowments.clear();
            config.application.buyer = borrower;
            config.application.month = 2;
            config.application.downpayment = 2;
            let offer = &mut config.offers[0];
            offer.sale.price.quantity = 8;
            offer.minimum_downpayment = 2;
            offer.loan.max_principal = 6;
            offer.loan.monthly_rate_bps = 0;
            offer.loan.term_months = 2;
            offer.loan.grace_months = 0;
            offer.collateral.settlement = CollateralSettlement::FixedValue { value: 4 };
            w.credit = Some(config);
            s.balances.insert((borrower, TOKEN), downpayment);
            s.balances.insert((STATE_AGENT, TOKEN), 6);
            let run = |backend| {
                let mut audit = Audit::with_opening(
                    &w,
                    &s,
                    TOKEN,
                    Opening {
                        assets: [(PLOT, 8)].into(),
                        processes: Some(Default::default()),
                        ..Opening::default()
                    },
                )
                .unwrap();
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while sim.state.month == 2 {
                    audit.step(&mut sim).unwrap();
                }
                assert_eq!(
                    credit::owner(&sim.world, &sim.state, PLOT),
                    Some(if downpayment == 2 {
                        borrower
                    } else {
                        STATE_AGENT
                    })
                );
                let (mut resumed, mut ra) = (sim.clone(), audit.clone());
                while sim.state.month <= 6 {
                    audit.step(&mut sim).unwrap();
                }
                while resumed.state.month <= 6 {
                    ra.step(&mut resumed).unwrap();
                }
                assert_eq!(
                    (&sim.state, &sim.ledger, &audit),
                    (&resumed.state, &resumed.ledger, &ra)
                );
                assert_eq!(
                    credit::owner(&sim.world, &sim.state, PLOT),
                    Some(STATE_AGENT)
                );
                if downpayment == 2 {
                    assert_eq!(sim.state.credit.loans[&1].status, credit::Status::Enforced);
                    assert_eq!(sim.state.credit.loans[&1].principal, 2);
                } else {
                    assert!(sim.state.credit.loans.is_empty());
                }
                assert!(
                    sim.ledger
                        .iter()
                        .filter(|b| b.month >= 3)
                        .flat_map(|b| &b.transactions)
                        .filter_map(|t| t.process.as_ref())
                        .any(|change| change.after.definition == PREPARE_FUEL
                            && change.after.status == Status::Completed)
                );
                for a in &sim.world.agents {
                    let report = audit.book().statements(a.id, 2, 6).unwrap();
                    assert_eq!(report.assets, report.liabilities + report.equity);
                }
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

#[test]
fn native_guarantees_and_household_recourse_share_real_collection_inventory() {
    use economics_compute_smoke::{
        household_governance::Governance,
        households::{self, Agreement},
        recovery::{Guarantee, GuaranteedClaim},
    };
    const HOME: AgentId = 10000;
    for household in [false, true] {
        let (mut w, mut s) = fixture(true);
        w.lending[0].terms.term_months = 1;
        let guarantor = if household { HOME } else { PERSON + 1 };
        if household {
            households::form(
                &mut w,
                &s,
                Agreement {
                    id: 1,
                    agent: HOME,
                    adults: vec![PERSON + 1],
                    governance: Governance::contributed(PERSON + 1),
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
        }
        s.balances.insert((guarantor, FUEL), 4);
        w.recovery.guarantees.push(Guarantee {
            follows_assignment: false,
            security: economics_compute_smoke::recovery::RecourseSecurity::Unsecured,
            id: 1,
            claim: GuaranteedClaim::Loan(10),
            guarantor,
            cap: 4,
            from: 2,
            through: 8,
            delay_months: 0,
            recourse: 101,
            priority: 0,
        });
        let run = |backend| {
            let mut audit = Audit::with_opening(
                &w,
                &s,
                TOKEN,
                Opening {
                    inventory: [((STATE_AGENT, FUEL), 4), ((guarantor, FUEL), 4)].into(),
                    exchange_values: [(FUEL, 1)].into(),
                    processes: Some(Default::default()),
                    ..Opening::default()
                },
            )
            .unwrap();
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            while sim.state.month < 3 || sim.state.phase != Phase::Acquire {
                audit.step(&mut sim).unwrap();
            }
            assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], 1);
            assert_eq!(sim.state.credit.loans[&10].principal, 0);
            let recourse = &sim.state.credit.loans[&101];
            assert_eq!(
                (recourse.principal, recourse.creditor, recourse.opened),
                (1, guarantor, 3)
            );
            assert!(
                economics_compute_smoke::credit::projection::dues(
                    &sim.world, &sim.state, PERSON, FUEL, 1
                )
                .unwrap()
                .is_empty()
            );
            let (mut resumed, mut ra) = (sim.clone(), audit.clone());
            while sim.state.month <= 6 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month <= 6 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &ra)
            );
            assert_eq!(sim.state.credit.loans[&101].principal, 0);
            assert_eq!(sim.state.credit.recovery.paid_guarantees[&1], 1);
            assert!(
                sim.ledger
                    .iter()
                    .filter(|b| b.month >= 3)
                    .flat_map(|b| &b.transactions)
                    .filter_map(|t| t.process.as_ref())
                    .any(|c| c.after.definition == PREPARE_FUEL
                        && c.after.status == Status::Completed)
            );
            assert_eq!(sim.state.balance(guarantor, TOKEN), 0);
            for a in &sim.world.agents {
                let report = audit.book().statements(a.id, 2, 6).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

#[test]
fn negotiated_household_sales_fund_later_useful_hiring_without_reusing_incoming_coins() {
    use economics_compute_smoke::{
        employment::{ArrearsPolicy, Terms},
        household_governance::{Governance, Policy as HouseholdPolicy},
        households::{self, Agreement},
        marketplace::{MARKETPLACE_TYPE, Market, Marketplace},
        negotiation::{QuotePolicy, Session, Trader},
        opportunities::{self, Action, HOUSEHOLD_TYPE, PERSON_TYPE, STATE_TYPE},
    };
    const HOME: AgentId = 10000;
    const VENUE: AgentId = 10001;
    const WORKER: AgentId = PERSON + 2;
    let (mut w, mut s) = fixture(false);
    w.lending.clear();
    let mut worker = w.participants[0].clone();
    worker.agent = WORKER;
    worker.needs.clear();
    worker.capacity.quantity = 3;
    for p in &mut w.participants {
        p.capacity.quantity = 0;
    }
    w.participants.push(worker);
    for (id, name) in [(WORKER, "outside worker"), (VENUE, "fuel market")] {
        w.agents.push(Agent {
            id,
            name: name.into(),
        });
    }
    w.definitions
        .iter_mut()
        .find(|d| d.id == PREPARE_FUEL)
        .unwrap()
        .stages[0]
        .monthly_services[0]
        .quantity = 3;
    let mut governance = Governance::contributed(PERSON);
    governance.charter.initial_policy = HouseholdPolicy::NeedsFirst;
    governance.charter.hiring_budget = Some(Amount::new(TOKEN, 6));
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
    s.balances.insert((HOME, FUEL), 4);
    s.balances.insert((WORKER, TOKEN), 6);
    w.employment.push(Terms {
        id: 1,
        employer: HOME,
        worker: WORKER,
        from: 2,
        through: 3,
        capacity: Amount::new(LABOR, 3),
        wage_per_unit: Amount::new(TOKEN, 2),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    w.employment_offers.insert(1);
    // Existing formation predates this permissive operational policy.
    w.transaction_policy = Some(opportunities::Policy {
        authority: STATE_AGENT,
        laws: vec![],
        agreement_forms: None,
        agreement_limits: Default::default(),
        membership_offers: vec![],
        membership_permissions: Default::default(),
        agent_types: [
            (PERSON, PERSON_TYPE),
            (PERSON + 1, PERSON_TYPE),
            (WORKER, PERSON_TYPE),
            (STATE_AGENT, STATE_TYPE),
            (VENUE, MARKETPLACE_TYPE),
            (HOME, HOUSEHOLD_TYPE),
        ]
        .into(),
        permissions: [
            (PERSON_TYPE, Action::StockTrade),
            (HOUSEHOLD_TYPE, Action::StockTrade),
            (PERSON_TYPE, Action::CapacityTrade),
            (HOUSEHOLD_TYPE, Action::CapacityTrade),
            (PERSON_TYPE, Action::Process(PREPARE_FUEL)),
            (PERSON_TYPE, Action::Process(USE_FUEL)),
        ]
        .into(),
    });
    w.marketplaces.push(Marketplace {
        agent: VENUE,
        allowed_types: [PERSON_TYPE, HOUSEHOLD_TYPE].into(),
        markets: vec![Market {
            id: 1,
            goods: Amount::new(FUEL, 2),
            payment: TOKEN,
            price_tick: 1,
        }],
    });
    w.negotiation = Some(Session {
        marketplace: VENUE,
        market: 1,
        month: 2,
        max_rounds: 1,
        buyer: Trader {
            agent: WORKER,
            limit: 6,
            opening_quote: 6,
            policy: QuotePolicy::Fixed,
        },
        seller: Trader {
            agent: HOME,
            limit: 6,
            opening_quote: 6,
            policy: QuotePolicy::Fixed,
        },
        goods: Amount::new(FUEL, 2),
        payment: TOKEN,
    });
    let run = |backend| {
        let mut audit = Audit::with_opening(
            &w,
            &s,
            TOKEN,
            Opening {
                inventory: [((HOME, FUEL), 4)].into(),
                services: Some(Default::default()),
                processes: Some(Default::default()),
                ..Opening::default()
            },
        )
        .unwrap();
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        while sim.state.month == 2 {
            audit.step(&mut sim).unwrap();
        }
        assert_eq!(sim.state.balance(HOME, TOKEN), 6);
        assert!(!sim.state.employment.earned.contains_key(&(1, 2)));
        let (mut resumed, mut ra) = (sim.clone(), audit.clone());
        while sim.state.month == 3 {
            audit.step(&mut sim).unwrap();
        }
        while resumed.state.month == 3 {
            ra.step(&mut resumed).unwrap();
        }
        assert_eq!(
            (&sim.state, &sim.ledger, &audit),
            (&resumed.state, &resumed.ledger, &ra)
        );
        assert_eq!(sim.state.employment.earned[&(1, 3)].claim.outstanding(), 0);
        assert_eq!(sim.state.balance(WORKER, TOKEN), 6);
        assert_eq!(sim.state.balance(HOME, TOKEN), 0);
        assert_eq!(sim.state.balance(WORKER, FUEL), 2);
        assert!(
            sim.state
                .processes
                .values()
                .any(|p| p.definition == PREPARE_FUEL && p.status == Status::Completed)
        );
        assert!(sim.reports.iter().all(|r| r.deficit(WARMTH) == 0));
        for a in &sim.world.agents {
            let report = audit.book().statements(a.id, 2, 3).unwrap();
            assert_eq!(report.assets, report.liabilities + report.equity);
        }
        (sim.state, sim.ledger, audit)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}

#[test]
fn collection_requests_use_eligible_techniques_without_multiplying_exclusive_tools() {
    use economics_compute_smoke::{
        activities::DurableKind,
        equipment::{DurableAsset, Technique},
    };
    const KIND: u32 = 99;
    for hours in [1, 3] {
        for usable in [false, true] {
            let (mut w, mut s) = fixture(false);
            w.lending.clear();
            w.horizon = 4;
            for p in &mut w.participants {
                p.capacity.quantity = if p.agent == PERSON { hours } else { 0 };
            }
            w.definitions
                .iter_mut()
                .find(|d| d.id == PREPARE_FUEL)
                .unwrap()
                .stages[0]
                .monthly_services[0]
                .quantity = 3;
            w.activities.kinds.insert(
                KIND,
                DurableKind {
                    name: "collection tool".into(),
                    lifetime: 4,
                    monthly_decay: 0,
                    attached: false,
                },
            );
            w.techniques.push(Technique {
                id: 1,
                definition: PREPARE_FUEL,
                stage: 0,
                equipment_kind: Some(KIND),
                competency: None,
                wear: 1,
                output_multiplier: 1,
                services: vec![Amount::new(LABOR, 1)],
            });
            s.equipment.insert(
                TOOL,
                DurableAsset {
                    id: TOOL,
                    owner: PERSON,
                    kind: KIND,
                    attached_to: None,
                    remaining_uses: if usable { 4 } else { 0 },
                    last_used_month: None,
                },
            );
            let run = |backend| {
                let mut audit = Audit::with_opening(
                    &w,
                    &s,
                    TOKEN,
                    Opening {
                        assets: [(TOOL, if usable { 4 } else { 0 })].into(),
                        processes: Some(Default::default()),
                        ..Opening::default()
                    },
                )
                .unwrap();
                let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
                while sim.state.phase != Phase::Productive {
                    audit.step(&mut sim).unwrap();
                }
                let expected_request = if hours == 1 {
                    u32::from(usable)
                } else {
                    1 + u32::from(usable)
                };
                assert_eq!(
                    pool_market::demand(&sim.world, &sim.state, PERSON)
                        .unwrap()
                        .0,
                    expected_request
                );
                let (mut resumed, mut ra) = (sim.clone(), audit.clone());
                audit.step(&mut sim).unwrap();
                let completed = sim
                    .ledger
                    .last()
                    .unwrap()
                    .transactions
                    .iter()
                    .filter_map(|t| t.process.as_ref())
                    .filter(|c| {
                        c.after.definition == PREPARE_FUEL
                            && c.after.operator == PERSON
                            && c.after.status == Status::Completed
                    })
                    .count();
                assert_eq!(completed, usize::from(usable || hours == 3));
                assert_eq!(
                    sim.state.balance(STATE_AGENT, RAW_WOOD),
                    4 - 2 * completed as i32
                );
                assert_eq!(
                    sim.state.equipment[&TOOL].remaining_uses,
                    if usable { 3 } else { 0 }
                );
                if expected_request == 2 {
                    let demand = sim
                        .ledger
                        .last()
                        .unwrap()
                        .pool_market
                        .as_ref()
                        .unwrap()
                        .demands
                        .iter()
                        .find(|d| d.agent == PERSON)
                        .unwrap();
                    assert_eq!((demand.requested, demand.feasible), (2, 1));
                }
                while sim.state.month == 2 {
                    audit.step(&mut sim).unwrap();
                }
                while resumed.state.month == 2 {
                    ra.step(&mut resumed).unwrap();
                }
                assert_eq!(
                    (&sim.state, &sim.ledger, &audit),
                    (&resumed.state, &resumed.ledger, &ra)
                );
                (sim.state, sim.ledger, audit)
            };
            assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
        }
    }
}

#[test]
fn household_hiring_observes_funded_estate_tool_acquisition_in_the_same_boundary() {
    use economics_compute_smoke::{
        activities::DurableKind,
        employment::{ArrearsPolicy, Terms},
        equipment::{DurableAsset, Technique},
        household_governance::{Governance, Policy as HouseholdPolicy},
        households::{self, Agreement},
        recovery::{Bid, Listing, ProceedingTerms},
    };
    const HOME: AgentId = 10000;
    const WORKER: AgentId = 96;
    const DEBTOR: AgentId = 97;
    const ESTATE: AgentId = 98;
    const KIND: u32 = 99;
    for funded in [false, true] {
        let (mut w, mut s) = fixture(false);
        w.lending[0].debtor = DEBTOR;
        w.lending[0].terms.denomination = TOKEN;
        w.lending[0].terms.term_months = 1;
        s.balances.insert((STATE_AGENT, TOKEN), 4);
        s.balances
            .insert((PERSON, TOKEN), if funded { 8 } else { 0 });
        for id in [WORKER, DEBTOR, ESTATE] {
            w.agents.push(Agent {
                id,
                name: format!("agent {id}"),
            });
        }
        let mut worker = w.participants[0].clone();
        worker.agent = WORKER;
        worker.needs.clear();
        worker.capacity.quantity = 1;
        for p in &mut w.participants {
            p.capacity.quantity = 0;
        }
        w.participants.push(worker);
        w.definitions
            .iter_mut()
            .find(|d| d.id == PREPARE_FUEL)
            .unwrap()
            .stages[0]
            .monthly_services[0]
            .quantity = 3;
        w.activities.kinds.insert(
            KIND,
            DurableKind {
                name: "estate collection tool".into(),
                lifetime: 8,
                monthly_decay: 0,
                attached: false,
            },
        );
        w.techniques.push(Technique {
            id: 1,
            definition: PREPARE_FUEL,
            stage: 0,
            equipment_kind: Some(KIND),
            competency: None,
            wear: 1,
            output_multiplier: 1,
            services: vec![Amount::new(LABOR, 1)],
        });
        s.equipment.insert(
            TOOL,
            DurableAsset {
                id: TOOL,
                owner: DEBTOR,
                kind: KIND,
                attached_to: None,
                remaining_uses: 8,
                last_used_month: None,
            },
        );
        let mut governance = Governance::contributed(PERSON);
        governance.charter.initial_policy = HouseholdPolicy::NeedsFirst;
        governance.charter.hiring_budget = Some(Amount::new(TOKEN, 2));
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
        s.balances.insert((HOME, TOKEN), 2);
        w.employment.push(Terms {
            id: 1,
            employer: HOME,
            worker: WORKER,
            from: 4,
            through: 4,
            capacity: Amount::new(LABOR, 1),
            wage_per_unit: Amount::new(TOKEN, 2),
            on_arrears: ArrearsPolicy::SuspendDelivery,
            rank: 0,
        });
        w.employment_offers.insert(1);
        w.recovery.proceedings.push(ProceedingTerms {
            id: 1,
            debtor: DEBTOR,
            authority: STATE_AGENT,
            estate: ESTATE,
            denomination: TOKEN,
            opening_month: 4,
            earliest_close: 5,
            assets: vec![Listing {
                asset: TOOL,
                minimum_price: 8,
            }],
            discharge_deficiency: true,
        });
        w.recovery.bids.push(Bid {
            id: 1,
            proceeding: 1,
            buyer: PERSON,
            asset: TOOL,
            month: 4,
            price: 8,
        });
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            sim.run_months(1).unwrap();
            // An identical controlled loss before the reporting opening creates
            // a default; the measured sale/work interval has no injected funds.
            sim.state.balances.insert((DEBTOR, TOKEN), 0);
            let mut audit = Audit::with_opening(
                &w,
                &sim.state,
                TOKEN,
                Opening {
                    inventory: sim
                        .state
                        .balances
                        .iter()
                        .filter(|((_, r), q)| {
                            **q > 0
                                && *r != TOKEN
                                && w.resources
                                    .iter()
                                    .any(|x| x.id == *r && x.kind == ResourceKind::Stock)
                        })
                        .map(|(a, q)| (*a, i128::from(*q)))
                        .collect(),
                    exchange_values: [(RAW_WOOD, 1), (FUEL, 1)].into(),
                    assets: [(TOOL, 8)].into(),
                    services: Some(Default::default()),
                    processes: Some(Default::default()),
                    ..Opening::default()
                },
            )
            .unwrap();
            while sim.state.month < 4 || sim.state.phase != Phase::Acquire {
                audit.step(&mut sim).unwrap();
            }
            let (mut resumed, mut ra) = (sim.clone(), audit.clone());
            audit.step(&mut sim).unwrap();
            assert_eq!(
                sim.state.equipment[&TOOL].owner,
                if funded { PERSON } else { DEBTOR }
            );
            assert_eq!(sim.state.employment.earned.contains_key(&(1, 4)), funded);
            while sim.state.month < 5 {
                audit.step(&mut sim).unwrap();
            }
            while resumed.state.month < 5 {
                ra.step(&mut resumed).unwrap();
            }
            assert_eq!(
                (&sim.state, &sim.ledger, &audit),
                (&resumed.state, &resumed.ledger, &ra)
            );
            assert_eq!(sim.state.balance(WORKER, TOKEN), if funded { 2 } else { 0 });
            assert_eq!(
                sim.state.equipment[&TOOL].remaining_uses,
                if funded { 7 } else { 8 }
            );
            assert_eq!(
                sim.state
                    .processes
                    .values()
                    .filter(|p| p.definition == PREPARE_FUEL && p.status == Status::Completed)
                    .count(),
                usize::from(funded)
            );
            for a in &sim.world.agents {
                let report = audit.book().statements(a.id, 3, 4).unwrap();
                assert_eq!(report.assets, report.liabilities + report.equity);
            }
            (sim.state, sim.ledger, audit)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
