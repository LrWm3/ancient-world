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
    assert_eq!(e.people_food_deficit, 2);
    assert_eq!(e.household_delivered, 2);
    assert_eq!(e.land_dues_paid, 1);
    assert_eq!(e.completed_mints, 4);
    assert_eq!(e.issuance, 40);
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
    assert_eq!(run.sim.state.balance(SMITH, COIN), 8);
    assert_eq!(run.sim.state.balance(WORKSHOP, COIN), 8);
    assert_eq!(e.reconciled_agents, 8);
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
