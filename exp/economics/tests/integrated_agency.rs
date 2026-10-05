use economics_compute_smoke::{
    agency::{self, integration as economy},
    compute::Backend,
    credit,
    financial_reporting::Audit,
    household_governance as h,
    minting::*,
    model::*,
    opportunities::{Action, STATE_TYPE},
    simulation::Simulation,
    state_governance as g,
    telemetry::{Config, Observer},
};

fn run(w: World, s: State, backend: Backend) -> (Simulation, Audit) {
    let mut a = economy::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, backend).unwrap();
    through(&mut sim, &mut a, economy::RUN_MONTHS);
    (sim, a)
}
fn through(sim: &mut Simulation, a: &mut Audit, month: u32) {
    while sim.state.month <= month {
        a.step(sim).unwrap();
    }
}
fn coins(s: &State) -> i32 {
    s.balances
        .iter()
        .filter(|((_, r), _)| *r == COIN)
        .map(|(_, q)| q)
        .sum()
}
fn completed(sim: &Simulation, definition: DefinitionId) -> usize {
    sim.state
        .processes
        .values()
        .filter(|p| p.definition == definition && p.status == Status::Completed)
        .count()
}
fn losses(sim: &Simulation) -> i32 {
    sim.reports.iter().map(|r| r.deficit(NUTRITION)).sum()
}

#[test]
fn state_household_people_trade_and_finance_settle_in_one_audited_economy() {
    let (w, s) = economy::scenario().unwrap();
    let opening_coins = coins(&s);
    let (reference, a) = run(w.clone(), s.clone(), Backend::Reference);
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    let mut b = economy::audit(&sim.world, &sim.state).unwrap();
    let mut observer = Observer::new(
        vec![],
        "integrated",
        Config {
            settlement: true,
            ..Default::default()
        },
    )
    .unwrap();
    while sim.state.month <= economy::RUN_MONTHS {
        observer.step_audited(&mut sim, &mut b).unwrap();
        // Every phase is a possible in-memory restart, including pending policies,
        // forward delivery, loan repayment, elections and the annual bill.
        let mut resumed =
            Simulation::new(sim.world.clone(), sim.state.clone(), Backend::CubeCpu).unwrap();
        resumed.ledger = sim.ledger;
        resumed.reports = sim.reports;
        sim = resumed;
    }
    assert_eq!(sim.world, reference.world);
    assert_eq!(sim.state, reference.state);
    assert_eq!(sim.ledger, reference.ledger);
    assert_eq!(sim.reports, reference.reports);
    assert_eq!(a, b);
    assert_eq!(losses(&sim), 2);
    assert!(
        sim.reports
            .iter()
            .filter(|r| r.month > 1)
            .all(|r| r.deficit(NUTRITION) == 0)
    );
    assert!(completed(&sim, economy::GROW) > 1);
    assert_eq!(sim.state.credit.loans[&10].status, credit::Status::Repaid);
    assert_eq!(sim.state.exchange.forwards[&20].delivered, 2);
    assert_eq!(sim.state.obligations[&(77, 13)].outstanding(), 0);
    let issuance = b
        .book()
        .statements(ISSUER, 1, economy::RUN_MONTHS)
        .unwrap()
        .issuance_change;
    assert_eq!(issuance, 40);
    assert_eq!(i128::from(coins(&sim.state) - opening_coins), issuance);
    assert_eq!(completed(&sim, MINT), 4);
    assert_eq!(sim.state.balance(ISSUER, COIN), 18);
    assert_eq!(sim.state.balance(WORKER, COIN), 19);
    assert!(sim.state.balances.values().all(|q| *q >= 0));
    assert!(sim.world.governance_observation.is_none());
    for id in [ISSUER, economy::HOUSEHOLD] {
        let c = &sim.world.agency[&id];
        assert!(c.history.iter().any(|d| d.chosen.is_some()));
        assert!(c.history.iter().any(|d| !d.ballots.is_empty()));
        assert!(
            c.history
                .iter()
                .all(|d| d.alternatives.iter().any(|a| a.losses.is_some()))
        );
        assert!(c.history.last().unwrap().chosen.is_none());
    }
    for agent in &sim.world.agents {
        let statement = b
            .book()
            .statements(agent.id, 1, economy::RUN_MONTHS)
            .unwrap();
        assert_eq!(statement.assets, statement.liabilities + statement.equity);
        assert_eq!(
            statement.closing_cash,
            i128::from(sim.state.balance(agent.id, COIN))
        );
    }
    let logs = String::from_utf8(observer.finish().unwrap()).unwrap();
    assert_eq!(
        logs.lines()
            .filter(|l| l.contains("organization_decision"))
            .count(),
        (economy::RUN_MONTHS * 2) as usize
    );
}

#[test]
fn changing_household_priorities_changes_real_food_outcomes_in_the_same_economy() {
    let (w, s) = economy::scenario().unwrap();
    let mut fixed = w.clone();
    fixed
        .agency
        .get_mut(&economy::HOUSEHOLD)
        .unwrap()
        .config
        .programs
        .get_mut(&1)
        .unwrap()
        .commands = vec![agency::Command::HouseholdPolicy(h::Policy::NetOutput)];
    let (normal, _) = run(w, s.clone(), Backend::Reference);
    let (control, _) = run(fixed, s, Backend::Reference);
    assert!(losses(&control) > losses(&normal));
    // Both still generate elections: this compares allocation, not absence of a governor.
    assert!(
        h::authority(&control.world.households[0], &control.state)
            .leader
            .is_some()
    );
    assert_eq!(losses(&control), 8);
}

#[test]
fn accepted_forward_changes_state_funding_but_final_delivery_does_not_hide_lateness() {
    let (w, s) = economy::scenario().unwrap();
    let mut a = economy::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    through(&mut sim, &mut a, 4);
    assert_eq!(sim.state.exchange.forwards[&20].claim().outstanding(), 2);
    through(&mut sim, &mut a, economy::RUN_MONTHS);
    let deliveries: Vec<_> = sim
        .ledger
        .iter()
        .flat_map(|b| {
            b.transactions.iter().filter_map(move |t| match t.forward {
                Some(economics_compute_smoke::forward::Event::Delivery {
                    contract: 20,
                    quantity,
                }) => Some((b.month, quantity)),
                _ => None,
            })
        })
        .collect();
    assert_eq!(deliveries, [(10, 1), (11, 1)]);
    let mut no_forward = w;
    no_forward.prepaid_deliveries.clear();
    let (control, _) = run(no_forward, s, Backend::Reference);
    assert_eq!(completed(&control, MINT), 3);
    assert_eq!(completed(&sim, MINT), 4);
}

#[test]
fn household_trade_permission_is_required_and_late_claim_blocks_new_advance() {
    let (mut w, s) = economy::scenario().unwrap();
    let mut prohibited = w.clone();
    prohibited
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(
            economics_compute_smoke::opportunities::HOUSEHOLD_TYPE,
            Action::StockTrade,
        ));
    let (control, _) = run(prohibited, s.clone(), Backend::Reference);
    assert!(!control.state.exchange.forwards.contains_key(&20));
    let mut second = w.prepaid_deliveries[0].clone();
    second.id = 21;
    second.month = 5;
    second.due = 6;
    w.prepaid_deliveries.push(second);
    let (sim, _) = run(w, s, Backend::Reference);
    assert_eq!(sim.state.exchange.forwards[&20].delivered, 2);
    assert!(!sim.state.exchange.forwards.contains_key(&21));
}

#[test]
fn missing_paid_hours_prevent_issuance_but_do_not_erase_contracts() {
    let (w, mut s) = economy::scenario().unwrap();
    let mut w = w;
    w.participants
        .iter_mut()
        .find(|p| p.agent == WORKER)
        .unwrap()
        .capacity
        .quantity = 0;
    let supply = coins(&s);
    // No extra initial treasury or gift can hide the unfunded mint in this control.
    s.balances.insert((ISSUER, COIN), 0);
    let (sim, a) = run(w, s, Backend::Reference);
    assert_eq!(completed(&sim, MINT), 0);
    assert_eq!(coins(&sim.state), supply);
    assert_eq!(
        a.book()
            .statements(ISSUER, 1, economy::RUN_MONTHS)
            .unwrap()
            .issuance_change,
        0
    );
    assert!(sim.state.credit.loans.contains_key(&10));
    assert!(sim.state.exchange.forwards.contains_key(&20));
    assert_eq!(sim.state.exchange.forwards[&20].delivered, 2);
    assert!(sim.state.balances.values().all(|q| *q >= 0));
}

#[test]
fn dated_law_interrupts_minting_and_both_governments_continue_after_reopening() {
    let (mut w, s) = economy::scenario().unwrap();
    w.state_governance
        .as_mut()
        .unwrap()
        .constitution
        .policies
        .insert(
            1,
            g::LegalPolicy {
                name: "pause minting".into(),
                prohibited: [(Some(STATE_TYPE), Action::Process(MINT))].into(),
            },
        );
    let mut a = economy::audit(&w, &s).unwrap();
    let mut sim = Simulation::new(w, s, Backend::CubeCpu).unwrap();
    through(&mut sim, &mut a, 2);
    let governor = g::authority(&sim.world, &sim.state)
        .unwrap()
        .governor
        .unwrap();
    g::schedule(
        &mut sim.world,
        &sim.state,
        g::PolicyChange {
            month: 4,
            authorized_by: governor,
            policy: 1,
        },
    )
    .unwrap();
    through(&mut sim, &mut a, 5);
    let paused = completed(&sim, MINT);
    assert_eq!(paused, 2);
    assert!(
        sim.world.agency[&ISSUER]
            .history
            .iter()
            .filter(|d| [4, 5].contains(&d.month))
            .all(|d| d.chosen.is_none())
    );
    let governor = g::authority(&sim.world, &sim.state)
        .unwrap()
        .governor
        .unwrap();
    g::schedule(
        &mut sim.world,
        &sim.state,
        g::PolicyChange {
            month: 7,
            authorized_by: governor,
            policy: 0,
        },
    )
    .unwrap();
    let (mut resumed, mut ra) = (sim.clone(), a.clone());
    resumed.backend = Backend::Reference;
    // Reverse input catalogs; priority remains the explicit configured policy.
    resumed.world.agents.reverse();
    resumed.world.participants.reverse();
    through(&mut sim, &mut a, economy::RUN_MONTHS);
    through(&mut resumed, &mut ra, economy::RUN_MONTHS);
    assert_eq!(sim.state, resumed.state);
    assert_eq!(sim.ledger, resumed.ledger);
    assert_eq!(sim.reports, resumed.reports);
    assert_eq!(a, ra);
    assert!(completed(&sim, MINT) > paused);
    assert_eq!(sim.state.credit.loans[&10].status, credit::Status::Repaid);
    assert_eq!(sim.state.exchange.forwards[&20].delivered, 2);
    assert_eq!(sim.state.obligations[&(77, 13)].outstanding(), 0);
    assert_eq!(losses(&sim), 2);
}
