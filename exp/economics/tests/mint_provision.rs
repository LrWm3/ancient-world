use economics_compute_smoke::{
    compute::Backend,
    marketplace::Side,
    minting::{self, provisioning::Choice, *},
    model::*,
    simulation::Simulation,
};
fn sim(case: &str, b: Backend) -> Simulation {
    let (w, s) = minting::provision_scenario(case).unwrap();
    Simulation::new(w, s, b).unwrap()
}
fn deficit(s: &Simulation) -> i32 {
    s.reports.iter().map(|r| r.deficit(NUTRITION)).sum()
}
fn completed(s: &Simulation, definition: DefinitionId) -> usize {
    s.state
        .processes
        .values()
        .filter(|p| p.definition == definition && p.status == Status::Completed)
        .count()
}
fn boundary(s: &Simulation, month: u32) -> &minting::Boundary {
    s.ledger
        .iter()
        .filter_map(|b| b.minting.as_ref())
        .find(|b| b.month == month)
        .unwrap()
}
#[test]
fn cpu_controls_distinguish_usable_money_food_scarcity_and_leisure() {
    for (case, shortfall, mints, leisure, supply) in [
        ("adequate", 0, 3, 3, 48),
        ("scarce", 6, 1, 1, 28),
        ("empty", 12, 0, 0, 18),
        ("endowed", 0, 1, 11, 28),
    ] {
        let mut s = sim(case, Backend::CubeCpu);
        s.run_months(6).unwrap();
        assert_eq!(deficit(&s), shortfall, "{case}");
        assert_eq!(completed(&s, MINT), mints, "{case}");
        assert_eq!(completed(&s, REST), leisure, "{case}");
        assert_eq!(
            s.state
                .balances
                .iter()
                .filter(|((_, r), _)| *r == COIN)
                .map(|(_, v)| *v)
                .sum::<i32>(),
            supply
        );
        // Food sold by the state is consumed through real process legs.
        let consumed = s
            .reports
            .iter()
            .map(|r| r.fulfilled(NUTRITION))
            .sum::<i32>();
        assert_eq!(consumed + shortfall, 12);
    }
}
#[test]
fn food_clears_even_when_procurement_fails_and_after_last_mint_target() {
    let mut s = sim("adequate", Backend::CubeCpu);
    s.run_months(2).unwrap();
    let b = boundary(&s, 2);
    assert!(b.plan.as_ref().unwrap().reason.contains("unmatched"));
    assert_eq!(b.deals.len(), 2);
    assert!(b.deals.iter().all(|d| d.market == WHEAT));
    assert!(b.receipts.iter().all(|r| r.accepted));
    assert_eq!(deficit(&s), 0);
    s.world
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .additional_months
        .clear();
    s.world.scheduled_starts.retain(|p| p.month == 1);
    s.run_months(1).unwrap();
    assert_eq!(boundary(&s, 3).plan.as_ref().unwrap().target_month, None);
    assert!(boundary(&s, 3).deals.iter().any(|d| d.market == WHEAT));
}
#[test]
fn food_price_changes_reservation_asks_and_can_stop_issuance() {
    for (price, ask, success) in [(3, 6, true), (4, 8, false)] {
        let mut s = sim("adequate", Backend::CubeCpu);
        let p = s
            .world
            .minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap();
        p.sale_limit = price;
        for q in p.quotes.iter_mut().filter(|q| q.market == WHEAT) {
            q.limit = price;
        }
        for a in [SUPPLIER, WORKER] {
            s.state.balances.insert((a, COIN), price);
        }
        s.run_months(1).unwrap();
        let plan = boundary(&s, 1).plan.as_ref().unwrap();
        for agent in [SUPPLIER, WORKER] {
            assert_eq!(
                plan.orders
                    .iter()
                    .find(|o| o.agent == agent && o.side == Side::Sell)
                    .unwrap()
                    .limit,
                ask
            );
        }
        assert_eq!(completed(&s, MINT) > 0, success);
        assert_eq!(deficit(&s), 0);
    }
}
#[test]
fn own_provision_generates_real_leisure_and_sold_hours_cannot_be_reused() {
    let mut endowed = sim("endowed", Backend::CubeCpu);
    endowed.run_months(1).unwrap();
    assert!(
        boundary(&endowed, 1)
            .plan
            .as_ref()
            .unwrap()
            .provision
            .iter()
            .all(|d| d.choice == Choice::Covered)
    );
    assert_eq!(completed(&endowed, REST), 2);
    assert_eq!(completed(&endowed, REFINE), 0);
    for agent in [SUPPLIER, WORKER] {
        assert_eq!(endowed.state.balance(agent, LEISURE_TIME), 2);
        assert_eq!(endowed.state.balance(agent, HOURS), 0);
    }
    endowed.step().unwrap();
    assert_eq!(endowed.state.balance(WORKER, LEISURE_TIME), 0);
    let mut working = sim("adequate", Backend::CubeCpu);
    working.run_months(1).unwrap();
    assert!(
        boundary(&working, 1)
            .deals
            .iter()
            .any(|d| d.seller == WORKER && d.market == HOURS)
    );
    assert_eq!(working.state.balance(WORKER, LEISURE_TIME), 0);
    assert_eq!(working.state.balance(SUPPLIER, LEISURE_TIME), 2);
}
#[test]
fn coins_without_food_access_do_not_count_as_provision() {
    let mut s = sim("empty", Backend::CubeCpu);
    for a in [SUPPLIER, WORKER] {
        s.state.balances.insert((a, COIN), 100);
    }
    s.run_months(1).unwrap();
    assert!(
        boundary(&s, 1)
            .plan
            .as_ref()
            .unwrap()
            .provision
            .iter()
            .all(|d| d.choice == Choice::NoFoodAccess)
    );
    assert_eq!(completed(&s, REST), 0);
    assert_eq!(completed(&s, REFINE), 0);
    assert_eq!(completed(&s, MINT), 0);
    assert_eq!(deficit(&s), 2);
}
#[test]
fn current_wages_do_not_purchase_current_food() {
    let mut s = sim("adequate", Backend::CubeCpu);
    for a in [SUPPLIER, WORKER] {
        s.state.balances.insert((a, COIN), 0);
    }
    let p = s
        .world
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap();
    p.input_limits.values_mut().for_each(|v| *v = 9);
    s.state.balances.insert((ISSUER, COIN), 18);
    s.run_months(1).unwrap();
    assert_eq!(completed(&s, MINT), 1);
    assert_eq!(deficit(&s), 2);
    assert!(boundary(&s, 1).deals.iter().all(|d| d.market != WHEAT));
    assert_eq!(s.state.balance(WORKER, COIN), 9);
    assert_eq!(completed(&s, REST), 0);
}
#[test]
fn cpu_reference_and_phase_restart_preserve_choices_and_outcomes() {
    for case in ["adequate", "scarce", "empty", "endowed"] {
        let mut reference = sim(case, Backend::Reference);
        reference.run_months(6).unwrap();
        let mut cpu = sim(case, Backend::CubeCpu);
        cpu.world.agents.reverse();
        cpu.world.participants.reverse();
        cpu.world.activities.orders.reverse();
        cpu.world
            .minting
            .as_mut()
            .unwrap()
            .order_policy
            .as_mut()
            .unwrap()
            .quotes
            .reverse();
        while cpu.state.month <= 6 {
            cpu.step().unwrap();
            cpu.state = Simulation::new(cpu.world.clone(), cpu.state.clone(), Backend::CubeCpu)
                .unwrap()
                .state;
        }
        assert_eq!(cpu.state, reference.state);
        assert_eq!(cpu.ledger, reference.ledger);
        assert_eq!(cpu.reports, reference.reports);
    }
}
#[test]
fn invalid_provisioning_and_forged_reasoning_are_rejected() {
    use economics_compute_smoke::settlement::{self, DEFAULT_EFFECT_LIMIT};
    let (mut w, state) = minting::provision_scenario("adequate").unwrap();
    w.minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .provisioning
        .as_mut()
        .unwrap()
        .horizon_months = 0;
    assert!(Simulation::new(w, state, Backend::Reference).is_err());
    let mut s = sim("adequate", Backend::CubeCpu);
    s.step().unwrap();
    let before = s.state.clone();
    let mut b = Batch::empty(&s.state);
    b.minting = minting::evaluate(&s.world, &s.state).unwrap();
    b.transactions = b.minting.as_ref().unwrap().transactions.clone();
    b.minting.as_mut().unwrap().plan.as_mut().unwrap().provision[0].choice = Choice::Covered;
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &b,
            Backend::CubeCpu,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, before);
}
#[test]
fn observations_include_choices_without_changing_the_run() {
    use economics_compute_smoke::telemetry::{Config, Observer};
    let mut s = sim("endowed", Backend::CubeCpu);
    let mut o = Observer::new(
        Vec::new(),
        "provision",
        Config {
            settlement: true,
            ..Config::default()
        },
    )
    .unwrap();
    o.run_months(&mut s, 6).unwrap();
    let bytes = o.finish().unwrap();
    let rows: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(rows.iter().any(
        |r| r["kind"] == "physical_minting_orders" && r["provision"][0]["choice"] == "Covered"
    ));
    let mut plain = sim("endowed", Backend::CubeCpu);
    plain.run_months(6).unwrap();
    assert_eq!(s.state, plain.state);
    assert_eq!(s.ledger, plain.ledger);
}

#[test]
fn food_sales_do_not_require_mint_authority_and_unusable_food_is_not_cover() {
    use economics_compute_smoke::opportunities::{self, Action};
    let mut s = sim("adequate", Backend::CubeCpu);
    s.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(opportunities::STATE_TYPE, Action::Process(MINT)));
    s.run_months(1).unwrap();
    assert_eq!(deficit(&s), 0);
    assert_eq!(completed(&s, MINT), 0);
    assert_eq!(boundary(&s, 1).deals.len(), 2);
    let mut s = sim("endowed", Backend::CubeCpu);
    s.world
        .transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .remove(&(opportunities::PERSON_TYPE, Action::Process(EAT)));
    s.run_months(1).unwrap();
    assert_eq!(completed(&s, REST), 0);
    assert!(
        boundary(&s, 1)
            .plan
            .as_ref()
            .unwrap()
            .provision
            .iter()
            .all(|d| d.choice == Choice::NoFoodAccess)
    );
}
