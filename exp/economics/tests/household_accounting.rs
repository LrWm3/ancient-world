use economics_compute_smoke::{
    accounting::{Account as A, Flow},
    activities::CoinPayment,
    compute::Backend,
    financial_reporting::Audit,
    household_governance::Governance,
    households::{self, Agreement},
    model::*,
    scenario::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;
const HOME: AgentId = 10000;
fn form(w: &mut World, s: &State, adults: Vec<AgentId>) {
    households::form(
        w,
        s,
        Agreement {
            id: 1,
            agent: HOME,
            governance: Governance::contributed(adults[0]),
            adults,
            formed: s.month,
            dwelling_process: None,
            admission: None,
            membership: vec![],
            asset_sales: vec![],
            equipment_retirements: vec![],
        },
    )
    .unwrap();
}
fn coin(w: &mut World) {
    w.resources.push(Resource {
        id: TOKEN,
        name: "coin".into(),
        kind: ResourceKind::Stock,
    });
}
fn audit(w: &World, s: &State) -> Audit {
    let costs = s
        .balances
        .iter()
        .filter(|((_, r), q)| {
            *r != TOKEN
                && **q > 0
                && w.resources
                    .iter()
                    .any(|x| x.id == *r && x.kind == ResourceKind::Stock)
        })
        .map(|(k, q)| (*k, i128::from(*q) * 3))
        .collect();
    Audit::with_dues(
        w,
        s,
        TOKEN,
        BTreeMap::from([(PLOT, 0)]),
        costs,
        w.agreements.iter().map(|a| (a.id, 3)).collect(),
    )
    .unwrap()
    .with_process_policy(
        w,
        w.definitions
            .iter()
            .filter(|d| d.id == GROW && d.outputs.iter().any(|a| a.resource == SEED))
            .map(|d| (d.id, BTreeMap::from([(GRAIN, 1), (SEED, 1)])))
            .collect(),
    )
    .unwrap()
}
fn through(a: &mut Audit, s: &mut Simulation, month: u32) {
    while s.state.month <= month {
        a.step(s)
            .unwrap_or_else(|e| panic!("{e}, month {} phase {:?}", s.state.month, s.state.phase));
    }
}
#[test]
fn household_pooling_preserves_cost_through_production_consumption_and_cpu_continuation() {
    let (mut w, mut s) = repeated();
    coin(&mut w);
    w.agents.push(Agent {
        id: PERSON + 1,
        name: "second adult".into(),
    });
    let mut p = w.participants[0].clone();
    p.agent += 1;
    w.participants.push(p);
    form(&mut w, &s, vec![PERSON, PERSON + 1]);
    // Pool funds seed before planting and food before consumption.
    s.balances.insert((PERSON, SEED), 0);
    s.balances.insert((HOME, SEED), 1);
    s.balances.insert((HOME, GRAIN), 30);
    let mut a = audit(&w, &s);
    let mut b = a.clone();
    let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut a, &mut reference, 6);
    through(&mut b, &mut cpu, 6);
    assert_eq!(a, b);
    assert_eq!(reference.state, cpu.state);
    assert!(reference.ledger.iter().any(|b| {
        b.household
            .as_ref()
            .unwrap()
            .after
            .iter()
            .any(|e| e.account == (HOME, GRAIN) && e.delta > 0)
    }));
    assert!(reference.ledger.iter().any(|b| {
        b.household
            .as_ref()
            .unwrap()
            .before
            .iter()
            .any(|e| e.account == (HOME, SEED) && e.delta < 0)
    }));
    let mut resumed = reference.clone();
    let mut c = a.clone();
    through(&mut a, &mut reference, 12);
    through(&mut b, &mut cpu, 12);
    through(&mut c, &mut resumed, 12);
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(reference.state, cpu.state);
    let mut incoming = 0;
    let mut outgoing = 0;
    for agent in [STATE_AGENT, PERSON, PERSON + 1, HOME] {
        let r = a.book().statements(agent, 1, 12).unwrap();
        incoming += r.income.get(&A::TransferIncome).copied().unwrap_or(0);
        outgoing += r.expenses.get(&A::TransferExpense).copied().unwrap_or(0);
        assert_eq!(r.cash_flows.values().sum::<i128>(), 0);
    }
    assert!(incoming > 0);
    assert_eq!(incoming, outgoing);
}
#[test]
fn household_support_funds_member_dues_in_native_goods_or_coins_once() {
    for cash in [false, true] {
        let (mut w, mut s) = named("annual-access").unwrap();
        coin(&mut w);
        w.priority = Priority::NeedFirst;
        w.participants[0].needs.clear();
        w.condition_rules.clear();
        s.conditions.clear();
        w.definitions.clear();
        s.month = 13;
        s.balances.clear();
        if cash {
            w.activities.coin_payments.insert(
                1,
                CoinPayment {
                    resource: TOKEN,
                    coins_per_unit: 3,
                },
            );
        }
        form(&mut w, &s, vec![PERSON]);
        s.balances.insert(
            (HOME, if cash { TOKEN } else { GRAIN }),
            if cash { 3 } else { 1 },
        );
        let mut a = audit(&w, &s);
        let mut b = a.clone();
        let mut reference = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
        let mut cpu = Simulation::new(w, s, Backend::CubeCpu).unwrap();
        through(&mut a, &mut reference, 13);
        through(&mut b, &mut cpu, 13);
        assert_eq!(a, b);
        assert_eq!(reference.state, cpu.state);
        let member = a.book().statements(PERSON, 13, 13).unwrap();
        let home = a.book().statements(HOME, 13, 13).unwrap();
        assert_eq!(member.income[&A::TransferIncome], 3);
        assert_eq!(member.expenses[&A::DuesExpense], 3);
        assert_eq!(home.expenses[&A::TransferExpense], 3);
        assert_eq!(reference.state.obligations[&(1, 13)].paid, 1);
        if cash {
            assert_eq!(home.cash_flows[&Flow::Operating], -3);
            assert_eq!(member.cash_flows.values().sum::<i128>(), 0);
        }
    }
}
#[test]
fn forged_household_receipt_does_not_publish_financial_changes() {
    let (mut w, mut s) = baseline();
    coin(&mut w);
    form(&mut w, &s, vec![PERSON]);
    s.balances.insert((HOME, GRAIN), 20);
    let mut a = audit(&w, &s);
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    a.step(&mut sim).unwrap();
    let before = sim.state.clone();
    let old = a.clone();
    sim.step().unwrap();
    let mut batch = sim.ledger.last().unwrap().clone();
    batch.household.as_mut().unwrap().before.push(Effect {
        account: (PERSON, GRAIN),
        delta: 1,
    });
    assert!(a.record(&sim.world, &before, &batch, &sim.state).is_err());
    assert_eq!(a, old);
}

#[test]
#[ignore = "slow 32-person CPU accounting integration; run explicitly"]
fn specialist_households_reconcile_production_trading_and_annual_dues() {
    use economics_compute_smoke::{activities::Outcome, process_accounting::Output};
    let (w, s) = households::scenario().unwrap();
    let stocks = s
        .balances
        .iter()
        .filter(|((_, r), q)| {
            *r != TOKEN
                && **q > 0
                && w.resources
                    .iter()
                    .any(|x| x.id == *r && x.kind == ResourceKind::Stock)
        })
        .map(|(k, q)| (*k, i128::from(*q)))
        .collect();
    let assets = w
        .assets
        .iter()
        .map(|a| (a.id, 1))
        .chain(s.equipment.keys().map(|id| (*id, 1)))
        .collect();
    // Explicit test convention: equal total cost shares for all joint outputs.
    let shares = w
        .definitions
        .iter()
        .filter(|d| d.execution == Execution::Productive)
        .filter_map(|d| {
            let mut outputs: BTreeMap<_, u32> = d
                .outputs
                .iter()
                .filter(|a| {
                    w.resources
                        .iter()
                        .any(|r| r.id == a.resource && r.kind == ResourceKind::Stock)
                })
                .map(|a| (Output::Stock(a.resource), 1))
                .collect();
            if let Some(Outcome::Create(kind)) = w.activities.outcomes.get(&d.id) {
                outputs.insert(Output::Durable(*kind), 1);
            }
            (!outputs.is_empty()).then_some((d.id, outputs))
        })
        .collect();
    let a = Audit::with_dues(
        &w,
        &s,
        TOKEN,
        assets,
        stocks,
        w.agreements.iter().map(|a| (a.id, 1)).collect(),
    )
    .unwrap()
    .with_output_cost_policy(&w, shares)
    .unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut a = a
        .with_issuance_policy(
            economics_compute_smoke::issuance_accounting::Policy::NonRedeemableEquity,
        )
        .unwrap();
    while sim.state.month <= 13 {
        eprintln!(
            "household financial stress: month {} {:?}",
            sim.state.month, sim.state.phase
        );
        a.step(&mut sim).unwrap();
    }
    a.finalize_through(13).unwrap();
    assert_eq!(sim.world.households.len(), 8);
    for agent in &sim.world.agents {
        let r = a.book().finalized_statements(agent.id, 1, 13).unwrap();
        assert_eq!(r.assets, r.liabilities + r.equity);
    }
}

#[test]
fn shared_dwelling_entitlements_do_not_duplicate_asset_wear() {
    use economics_compute_smoke::{
        activities::{DurableKind, Target, WorkOrder},
        equipment::DurableAsset,
    };
    let (mut w, mut s) = baseline();
    coin(&mut w);
    let ticket = 990;
    let occupy = 991;
    let kind = 992;
    let home = 993;
    w.resources.push(Resource {
        id: ticket,
        name: "occupancy entitlement".into(),
        kind: ResourceKind::Stock,
    });
    w.agents.push(Agent {
        id: PERSON + 1,
        name: "second adult".into(),
    });
    let mut p = w.participants[0].clone();
    p.agent += 1;
    w.participants.push(p);
    for p in &mut w.participants {
        p.needs[0].quantity = 1;
    }
    w.definitions.retain(|d| d.id == CONSUME);
    w.definitions[0].stages[0].entry_inputs = vec![Amount::new(ticket, 1)];
    w.definitions[0].outputs = vec![Amount::new(NUTRITION, 1)];
    w.definitions.push(ProcessDefinition {
        id: occupy,
        name: "occupy".into(),
        execution: Execution::Productive,
        enabled: true,
        asset_kind: None,
        stages: vec![Stage {
            name: "use".into(),
            months: 1,
            entry_inputs: vec![],
            monthly_services: vec![],
        }],
        outputs: vec![Amount::new(ticket, 1)],
    });
    w.activities.perishable.insert(ticket);
    w.activities.required.insert(occupy, kind);
    w.activities.kinds.insert(
        kind,
        DurableKind {
            name: "dwelling".into(),
            lifetime: 4,
            attached: false,
            monthly_decay: 0,
        },
    );
    w.activities.orders.push(WorkOrder {
        agent: PERSON,
        definition: occupy,
        priority: 0,
        target: Target::Stock(Amount::new(ticket, 1)),
    });
    s.balances.clear();
    s.equipment.insert(
        home,
        DurableAsset {
            id: home,
            owner: PERSON,
            kind,
            remaining_uses: 4,
            last_used_month: None,
            attached_to: None,
        },
    );
    form(&mut w, &s, vec![PERSON, PERSON + 1]);
    w.households[0].dwelling_process = Some(occupy);
    let mut a = Audit::with_processes(
        &w,
        &s,
        TOKEN,
        BTreeMap::from([(PLOT, 0), (home, 8)]),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    let mut b = a.clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut a, &mut reference, 2);
    through(&mut b, &mut cpu, 2);
    assert_eq!(a, b);
    assert_eq!(reference.state, cpu.state);
    assert_eq!(cpu.state.equipment[&home].remaining_uses, 2);
    assert!(cpu.reports.iter().all(|r| r.deficit(NUTRITION) == 0));
    let user = a.book().statements(PERSON, 1, 2).unwrap();
    assert_eq!(user.expenses[&A::ConsumptionExpense], 4);
    assert_eq!(user.trial_balance[&A::Tangible(home)], 4);
    assert_eq!(a.book().statements(PERSON + 1, 1, 2).unwrap().net_income, 0);
}

#[test]
fn lawful_elections_policy_changes_allocation_and_separate_books_compose() {
    use economics_compute_smoke::{
        accounting::ReportingScope,
        household_governance::{self as g, scenario as fixture},
    };
    let (w, s) = fixture::pair().unwrap();
    let make_audit = || {
        Audit::with_inventory(
            &w,
            &s,
            TOKEN,
            BTreeMap::new(),
            [((PERSON + 1, SEED), 3)].into(),
        )
        .unwrap()
        .with_process_policy(
            &w,
            [(fixture::FOOD_PROCESS, [(GRAIN, 1), (SEED, 1)].into())].into(),
        )
        .unwrap()
    };
    let mut cpu_audit = make_audit();
    let mut reference_audit = make_audit();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    through(&mut cpu_audit, &mut cpu, fixture::TERM_MONTHS);
    through(&mut reference_audit, &mut reference, fixture::TERM_MONTHS);
    let mut resumed = cpu.clone();
    let mut resumed_audit = cpu_audit.clone();
    let charter = cpu.world.households[0].governance.charter.clone();
    for sim in [&mut cpu, &mut reference, &mut resumed] {
        fixture::incoming_policy(&mut sim.world, &sim.state).unwrap();
        assert_eq!(
            sim.world.households[0].governance.policy(3),
            g::Policy::NetOutput
        );
        assert_eq!(
            sim.world.households[0].governance.policy(4),
            g::Policy::NeedsFirst
        );
    }
    through(&mut cpu_audit, &mut cpu, fixture::RUN_MONTHS);
    for month in 3..=fixture::RUN_MONTHS {
        through(&mut reference_audit, &mut reference, month);
    }
    through(&mut resumed_audit, &mut resumed, fixture::RUN_MONTHS);
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.reports, reference.reports);
    assert_eq!(cpu_audit, reference_audit);
    assert_eq!(cpu.state, resumed.state);
    assert_eq!(cpu.ledger, resumed.ledger);
    assert_eq!(cpu_audit, resumed_audit);
    assert_eq!(cpu.world.households[0].governance.charter, charter);
    for report in &cpu.reports {
        assert_eq!(report.deficit(NUTRITION), i32::from(report.month < 4));
    }
    let opening = cpu
        .ledger
        .iter()
        .find(|b| b.month == 3 && b.phase == Phase::Open)
        .unwrap();
    let authority = &opening.household.as_ref().unwrap().governance[0];
    assert_eq!(authority.leader, Some(PERSON + 1));
    assert_eq!(authority.election.as_ref().unwrap().turnout, 2);
    cpu_audit.finalize_through(fixture::RUN_MONTHS).unwrap();
    for agent in [PERSON, PERSON + 1, fixture::HOUSEHOLD] {
        let statement = cpu_audit
            .book()
            .finalized_statements(agent, 1, fixture::RUN_MONTHS)
            .unwrap();
        assert_eq!(statement.scope, ReportingScope::Separate { agent });
        assert_eq!(statement.assets, statement.liabilities + statement.equity);
    }
}

#[test]
fn adult_join_and_exit_preserve_separate_books_through_actual_settlement() {
    use economics_compute_smoke::{accounting::ReportingScope, households::membership as m};
    let (mut w, s) = repeated();
    coin(&mut w);
    for offset in 1..3 {
        w.agents.push(Agent {
            id: PERSON + offset,
            name: format!("member {offset}"),
        });
        let mut p = w.participants[0].clone();
        p.agent += offset;
        w.participants.push(p);
    }
    form(&mut w, &s, vec![PERSON, PERSON + 1]);
    let mut cpu_audit = audit(&w, &s);
    let mut reference_audit = cpu_audit.clone();
    let mut cpu = Simulation::new(w.clone(), s.clone(), Backend::CubeCpu).unwrap();
    let mut reference = Simulation::new(w, s, Backend::Reference).unwrap();
    for (sim, audit) in [
        (&mut cpu, &mut cpu_audit),
        (&mut reference, &mut reference_audit),
    ] {
        through(audit, sim, 1);
        let books_before = audit.clone();
        let state_before = sim.state.clone();
        m::join(
            &mut sim.world,
            &sim.state,
            HOME,
            PERSON + 2,
            (PERSON..PERSON + 3).collect(),
        )
        .unwrap();
        assert_eq!(*audit, books_before);
        assert_eq!(sim.state, state_before);
        through(audit, sim, 2);
        m::leave(&mut sim.world, &sim.state, HOME, PERSON + 1).unwrap();
        through(audit, sim, 6);
        audit.finalize_through(6).unwrap();
        for agent in [PERSON, PERSON + 1, PERSON + 2, HOME] {
            let report = audit.book().finalized_statements(agent, 1, 6).unwrap();
            assert_eq!(report.scope, ReportingScope::Separate { agent });
            assert_eq!(report.assets, report.liabilities + report.equity);
        }
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu_audit, reference_audit);
}
