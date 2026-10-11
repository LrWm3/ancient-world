use economics_compute_smoke::{
    acceptance::{self, Run},
    compute::Backend,
};

#[test]
fn acceptance_baseline_reconciles_actual_cross_agent_outcomes() {
    let (w, s) = acceptance::scenario().unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    assert!(run.evidence().is_err());
    run.through(acceptance::RUN_MONTHS).unwrap();
    let e = run.evidence().unwrap();
    assert_eq!(e.months, 14);
    assert_eq!(e.household_on_time_delivery, 2);
    assert!(e.state_loan_repaid);
    assert_eq!(e.state_loan_outstanding, Some(0));
    assert_eq!(e.household_delivered, 2);
    assert_eq!(e.land_dues_paid, 1);
    assert!(e.completed_mints > 0);
    assert_eq!(e.issuance, 10 * e.completed_mints as i128);
}

#[test]
fn workshop_requires_real_inputs_and_hours_to_supply_the_state() {
    use economics_compute_smoke::{
        acceptance::{SMITH, WORKSHOP},
        minting::{COIN, ORE},
    };
    let (w, s) = acceptance::scenario().unwrap();
    let mut run = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    run.through(acceptance::RUN_MONTHS).unwrap();
    let e = run.evidence().unwrap();
    assert_eq!(e.workshop_batches, 4);
    assert_eq!(e.workshop_wages_paid, 8);
    assert_eq!(
        run.sim.state.balance(SMITH, COIN),
        1 + e.workshop_wages_paid + e.municipal_wages_paid - 3 * e.smith_food_purchases as i32
    );
    assert_eq!(
        run.sim.state.balance(WORKSHOP, COIN),
        2 * e.workshop_metal_sales as i32
    );
    assert_eq!(e.reconciled_agents, 9);
    for missing_labor in [false, true] {
        let (mut w, mut s) = (w.clone(), s.clone());
        if missing_labor {
            w.participants
                .iter_mut()
                .find(|p| p.agent == SMITH)
                .unwrap()
                .capacity
                .quantity = 0;
        } else {
            s.balances.insert((WORKSHOP, ORE), 0);
        }
        let mut control = Run::new(w, s, Backend::Reference).unwrap();
        control.through(acceptance::RUN_MONTHS).unwrap();
        let c = control.evidence().unwrap();
        assert_eq!(c.workshop_batches, 0);
        assert_eq!(c.completed_mints, 0);
        assert_eq!(c.issuance, 0);
    }
}

#[test]
fn municipality_uses_only_residual_worker_hours_and_receives_real_land_dues() {
    use economics_compute_smoke::{
        acceptance::{SMITH, TOWN, TOWN_EMPLOYMENT, WORKSHOP_EMPLOYMENT},
        agency::integration,
        minting::{COIN, FIREWOOD},
    };
    let (w, s) = acceptance::scenario().unwrap();
    assert_eq!(
        w.assets
            .iter()
            .find(|a| a.id == integration::PLOT)
            .unwrap()
            .owner,
        TOWN
    );
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(4).unwrap();
    assert_eq!(run.evidence().unwrap().municipal_batches, 0);
    assert_eq!(run.sim.state.balance(TOWN, COIN), 10);
    run.through(acceptance::RUN_MONTHS).unwrap();
    let e = run.evidence().unwrap();
    assert_eq!(e.municipal_batches, 10);
    assert_eq!(e.municipal_wages_paid, 10);
    assert_eq!(e.municipal_cash, 1);
    assert_eq!(run.sim.state.balance(TOWN, FIREWOOD), 10);
    assert_eq!(
        run.sim.state.balance(SMITH, COIN),
        1 + e.workshop_wages_paid + e.municipal_wages_paid - 3 * e.smith_food_purchases as i32
    );
    for month in 1..=acceptance::RUN_MONTHS {
        let delivered: i32 = run
            .sim
            .state
            .employment
            .earned
            .iter()
            .filter(|((id, m), _)| {
                *m == month && [WORKSHOP_EMPLOYMENT, TOWN_EMPLOYMENT].contains(id)
            })
            .map(|(_, e)| e.delivered)
            .sum();
        assert!(delivered <= 2, "duplicated worker hours in month {month}");
    }
}

#[test]
fn worker_wages_purchase_market_food_and_missing_asks_expose_shortages() {
    use economics_compute_smoke::{agency::integration, minting::WHEAT};
    let (w, s) = acceptance::scenario().unwrap();
    let mut run = Run::new(w.clone(), s.clone(), Backend::Reference).unwrap();
    run.through(acceptance::RUN_MONTHS).unwrap();
    let e = run.evidence().unwrap();
    assert!(e.household_food_sales > 0);
    assert!(e.smith_food_purchases > 0);
    assert_eq!(e.smith_food_deficit, 4);
    assert!(acceptance::accepted_deals(&run.sim).any(|d| {
        [integration::HOUSEHOLD, integration::GROWER].contains(&d.seller)
            && d.buyer == acceptance::SMITH
            && d.market == WHEAT
    }));
    let mut unavailable = w;
    unavailable
        .minting
        .as_mut()
        .unwrap()
        .order_policy
        .as_mut()
        .unwrap()
        .quotes
        .retain(|q| ![integration::HOUSEHOLD, integration::GROWER].contains(&q.agent));
    let mut control = Run::new(unavailable, s, Backend::Reference).unwrap();
    control.through(acceptance::RUN_MONTHS).unwrap();
    let c = control.evidence().unwrap();
    assert_eq!(c.household_food_sales, 0);
    assert!(c.smith_food_deficit > e.smith_food_deficit);
}

#[test]
fn accepted_deal_evidence_does_not_count_uncommitted_packages() {
    let (w, s) = acceptance::scenario().unwrap();
    let mut run = Run::new(w, s, Backend::Reference).unwrap();
    run.through(acceptance::RUN_MONTHS).unwrap();
    assert!(acceptance::accepted_deals(&run.sim).count() > 0);
    // Corrupt only a diagnostic clone: forecasts/deal proposals must not count as fills.
    let mut proposed_only = run.sim.clone();
    for b in &mut proposed_only.ledger {
        if let Some(m) = &mut b.minting {
            for r in &mut m.receipts {
                r.accepted = false;
            }
        }
    }
    assert_eq!(acceptance::accepted_deals(&proposed_only).count(), 0);
}
