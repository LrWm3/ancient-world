use economics_compute_smoke::{
    compute::Backend,
    credit::{self, Event},
    credit_stress::{self, Case, OBSERVATION_MONTHS},
    model::{Phase, Status},
    scenario::{GRAIN, GROW, NUTRITION, PERSON, PLOT, STATE_AGENT, TOKEN},
    simulation::Simulation,
    telemetry::{Config, Observer},
};

fn sim(case: Case, backend: Backend) -> Simulation {
    let (w, s) = credit_stress::scenario(case).unwrap();
    Simulation::new(w, s, backend).unwrap()
}

#[test]
fn harvest_losses_separate_cash_repayment_from_collateral_settlement() {
    for (case, payoff, owner, deficit, cash_interest) in [
        (Case::Normal, 13, PERSON, 0, 520),
        (Case::Temporary, 15, PERSON, 6, 672),
        (Case::Persistent, 16, STATE_AGENT, 16, 420),
    ] {
        let mut sim = sim(case, Backend::CubeCpu);
        let mut closed = None;
        for month in 1..=OBSERVATION_MONTHS {
            sim.run_months(1).unwrap();
            let borrower = credit::balance_sheet(&sim.world, &sim.state, PERSON, TOKEN);
            let lender = credit::balance_sheet(&sim.world, &sim.state, STATE_AGENT, TOKEN);
            assert_eq!(borrower.principal_payable, lender.principal_receivable);
            assert_eq!(borrower.interest_payable, lender.interest_receivable);
            assert_eq!(borrower.coins + lender.coins, 106_500);
            assert!(borrower.coins >= 0 && lender.coins >= 0);
            assert_eq!(borrower.assets > 0, lender.assets == 0);
            let loan = &sim.state.credit.loans[&1];
            if loan.debt().unwrap() == 0 {
                closed.get_or_insert(month);
                assert!(!loan.collateral.pledged);
            } else {
                assert!(loan.collateral.pledged);
            }
        }
        assert_eq!(closed, Some(payoff));
        assert_eq!(credit::owner(&sim.world, &sim.state, PLOT), Some(owner));
        assert_eq!(
            sim.reports
                .iter()
                .filter(|r| r.agent == PERSON)
                .map(|r| r.deficit(NUTRITION))
                .sum::<i32>(),
            deficit
        );
        let events: Vec<_> = sim
            .ledger
            .iter()
            .filter_map(|b| b.credit.as_ref())
            .flat_map(|b| &b.events)
            .collect();
        assert_eq!(
            events
                .iter()
                .filter_map(|e| match e {
                    Event::Paid { interest, .. } => Some(*interest),
                    _ => None,
                })
                .sum::<i32>(),
            cash_interest
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Enforced { .. }))
                .count(),
            usize::from(case == Case::Persistent)
        );
        assert_eq!(
            events
                .iter()
                .any(|e| matches!(e, Event::Arrears { since: 8, .. })),
            case != Case::Normal
        );
        assert!(events.iter().all(|e| !matches!(e, Event::Cashflow { .. })));
        for batch in &sim.ledger {
            if batch.phase != Phase::Open {
                assert_eq!(
                    batch
                        .transactions
                        .iter()
                        .flat_map(|t| &t.effects)
                        .filter(|e| e.account.1 == TOKEN)
                        .map(|e| i64::from(e.delta))
                        .sum::<i64>(),
                    0
                );
            }
        }
    }
}

#[test]
fn due_transfers_crop_intact_and_next_acquire_income_cannot_pay_previous_due() {
    let mut temporary = sim(Case::Temporary, Backend::CubeCpu);
    temporary.run_months(12).unwrap();
    temporary.step().unwrap(); // Open 13
    temporary.step().unwrap(); // Due 13
    assert_eq!(temporary.state.credit.loans[&1].debt().unwrap(), 4116);
    assert_eq!(temporary.state.balance(PERSON, TOKEN), 0);
    temporary.step().unwrap(); // Acquire 13
    assert_eq!(temporary.state.credit.loans[&1].debt().unwrap(), 4116);
    assert_eq!(temporary.state.balance(PERSON, TOKEN), 2400);

    let mut persistent = sim(Case::Persistent, Backend::CubeCpu);
    persistent.run_months(15).unwrap();
    persistent.step().unwrap(); // Open 16
    let before = persistent
        .state
        .processes
        .values()
        .find(|p| p.definition == GROW && p.status == Status::Active)
        .unwrap()
        .clone();
    persistent.step().unwrap(); // Due 16
    let mut expected = before;
    expected.operator = STATE_AGENT;
    expected.beneficiary = STATE_AGENT;
    expected.goal = None;
    assert_eq!(persistent.state.processes[&expected.id], expected);
    assert_eq!(persistent.state.balance(STATE_AGENT, GRAIN), 0);
    let events = &persistent
        .ledger
        .last()
        .unwrap()
        .credit
        .as_ref()
        .unwrap()
        .events;
    assert!(events.contains(&Event::Enforced {
        loan: 1,
        value: 6000,
        debt_credit: 4233,
        surplus: 1767,
        remaining_debt: 0
    }));
    persistent.run_months(3).unwrap();
    assert_eq!(
        persistent.state.processes[&expected.id].status,
        Status::Completed
    );
    assert_eq!(persistent.state.balance(STATE_AGENT, GRAIN), 12);
}

#[test]
fn cpu_monthly_checkpoint_continuation_matches_reference_batch() {
    for case in [Case::Normal, Case::Temporary, Case::Persistent] {
        let mut reference = sim(case, Backend::Reference);
        reference.run_months(OBSERVATION_MONTHS).unwrap();
        let mut cpu = sim(case, Backend::CubeCpu);
        cpu.world.participants.reverse();
        cpu.world.definitions.reverse();
        for _ in 0..OBSERVATION_MONTHS {
            cpu.step().unwrap(); // checkpoint after Open, before Due
            cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                .unwrap()
                .state;
            cpu.run_months(1).unwrap();
        }
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
    }
}

#[test]
fn settlement_observer_explains_enforcement_without_changing_execution() {
    let mut observed = sim(Case::Persistent, Backend::CubeCpu);
    let mut observer = Observer::new(
        Vec::new(),
        "credit",
        Config {
            settlement: true,
            agents: [PERSON].into_iter().collect(),
            ..Config::default()
        },
    )
    .unwrap();
    observer
        .run_months(&mut observed, OBSERVATION_MONTHS)
        .unwrap();
    let bytes = observer.finish().unwrap();
    let rows: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let enforcement = rows
        .iter()
        .find(|r| r["kind"] == "loan_event" && r["detail"]["event"] == "Enforced")
        .unwrap();
    assert_eq!(enforcement["month"], 16);
    assert_eq!(enforcement["phase"], "Due");
    assert_eq!(enforcement["detail"]["debt_credit"], 4233);
    assert!(
        rows.iter()
            .any(|r| r["kind"] == "collateral_process_transfer"
                && r["from"] == PERSON
                && r["to"] == STATE_AGENT)
    );
    assert!(
        rows.iter()
            .any(|r| r["kind"] == "credit_stock_sale" && r["sold_lots"] == 0)
    );
    assert!(rows.iter().any(|r| r["kind"] == "loan_state"
        && r["month"] == 16
        && r["status"] == "Repaid"
        && r["owner"] == STATE_AGENT));
    let mut plain = sim(Case::Persistent, Backend::CubeCpu);
    plain.run_months(OBSERVATION_MONTHS).unwrap();
    assert_eq!(observed.state, plain.state);
    assert_eq!(observed.ledger, plain.ledger);
    assert_eq!(observed.reports, plain.reports);
}

#[test]
fn credit_observation_honors_settlement_toggle_and_counterparty_filters() {
    for (settlement, agent, expected) in [
        (false, PERSON, false),
        (true, 999, false),
        (true, STATE_AGENT, true),
    ] {
        let mut sim = sim(Case::Normal, Backend::Reference);
        let mut observer = Observer::new(
            Vec::new(),
            "filtered",
            Config {
                settlement,
                agents: [agent].into_iter().collect(),
                ..Config::default()
            },
        )
        .unwrap();
        observer.run_months(&mut sim, 2).unwrap();
        let bytes = observer.finish().unwrap();
        let rows: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        for kind in ["loan_state", "loan_event", "credit_stock_sale"] {
            assert_eq!(rows.iter().any(|r| r["kind"] == kind), expected);
        }
    }
}
