use economics_compute_smoke::{
    borrowing,
    compute::Backend,
    credit,
    model::*,
    scenario::{GRAIN, NUTRITION, PERSON},
    settlement::{self, DEFAULT_EFFECT_LIMIT},
    simulation::Simulation,
    stock_sale,
};
fn sim() -> Simulation {
    let (w, s) = stock_sale::forecast_scenario("funded").unwrap();
    Simulation::new(w, s, Backend::Reference).unwrap()
}
fn acquire(s: &mut Simulation) {
    s.step().unwrap();
    s.step().unwrap();
    assert_eq!(s.state.phase, Phase::Acquire);
}
fn batch(s: &Simulation) -> Batch {
    let mut b = Batch::empty(&s.state);
    b.credit = credit::evaluate(&s.world, &s.state).unwrap();
    b.transactions = b.credit.as_ref().unwrap().transactions.clone();
    b
}
#[test]
fn forecasts_protect_opening_food_and_preserve_funded_repayment() {
    let mut s = sim();
    s.backend = Backend::CubeCpu;
    s.run_months(18).unwrap();
    let first = s
        .ledger
        .iter()
        .filter_map(|b| b.credit.as_ref())
        .find_map(|c| c.stock_sale.as_ref())
        .unwrap();
    assert_eq!(first.reserve, 0);
    assert_eq!(first.goods, 0);
    let d = first.decision.as_ref().unwrap();
    assert!(d.feasible);
    assert_eq!(
        d.alternatives
            .iter()
            .map(|a| a.deficits[&NUTRITION])
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert!(
        s.reports
            .iter()
            .filter(|r| r.agent == PERSON)
            .all(|r| r.deficit(NUTRITION) == 0)
    );
    assert_eq!(s.state.credit.loans[&1].status, credit::Status::Repaid);
    assert_eq!(s.state.credit.stock_spent, 9600);
    let (mut w, state) = stock_sale::scenario("funded").unwrap();
    w.credit
        .as_mut()
        .unwrap()
        .stock_sales
        .as_mut()
        .unwrap()
        .reserve_months = 0;
    let mut unsafe_sale = Simulation::new(w, state, Backend::Reference).unwrap();
    unsafe_sale.run_months(1).unwrap();
    assert!(unsafe_sale.state.credit.loans.is_empty());
}
#[test]
fn imminent_harvest_allows_sale_below_fixed_reserve_but_missing_permission_does_not() {
    let mut s = sim();
    s.run_months(4).unwrap();
    acquire(&mut s);
    s.state.balances.insert((PERSON, GRAIN), 3);
    let b = batch(&s);
    assert_eq!(
        b.credit
            .as_ref()
            .unwrap()
            .stock_sale
            .as_ref()
            .unwrap()
            .goods,
        2
    );
    let mut fixed = s.clone();
    let p = fixed
        .world
        .credit
        .as_mut()
        .unwrap()
        .stock_sales
        .as_mut()
        .unwrap();
    p.forecast = None;
    p.reserve_months = 6;
    assert_eq!(batch(&fixed).credit.unwrap().stock_sale.unwrap().goods, 0);
    // A fresh applicant with no cultivation permission cannot rely on a crop.
    let mut no_land = sim();
    no_land.world.rights.clear();
    no_land
        .world
        .credit
        .as_mut()
        .unwrap()
        .attached_rights
        .clear();
    no_land.world.credit.as_mut().unwrap().purchase_policy = borrowing::Policy::Decline;
    no_land.state.balances.insert((PERSON, GRAIN), 1);
    acquire(&mut no_land);
    let d = batch(&no_land)
        .credit
        .unwrap()
        .stock_sale
        .unwrap()
        .decision
        .unwrap();
    assert!(!d.feasible);
    assert_eq!(d.selected_lots, 0);
    assert!(d.alternatives.iter().all(|a| !a.admissible));
}
#[test]
fn hidden_future_shocks_do_not_change_decision_and_forged_projection_is_atomic() {
    let mut s = sim();
    acquire(&mut s);
    let valid = batch(&s);
    let before = s.state.clone();
    s.world.capacity_overrides.insert((2, PERSON), 0);
    assert_eq!(batch(&s), valid);
    let mut forged = valid;
    forged
        .credit
        .as_mut()
        .unwrap()
        .stock_sale
        .as_mut()
        .unwrap()
        .decision
        .as_mut()
        .unwrap()
        .feasible = false;
    assert!(
        settlement::commit(
            &s.world,
            &mut s.state,
            &forged,
            Backend::Reference,
            DEFAULT_EFFECT_LIMIT
        )
        .is_err()
    );
    assert_eq!(s.state, before);
}
#[test]
fn bounded_horizon_and_need_limits_are_validated() {
    for horizon in [0, 5, 25] {
        let mut s = sim();
        s.world
            .credit
            .as_mut()
            .unwrap()
            .stock_sales
            .as_mut()
            .unwrap()
            .forecast
            .as_mut()
            .unwrap()
            .horizon_months = horizon;
        assert!(Simulation::new(s.world, s.state, Backend::Reference).is_err());
    }
    let mut s = sim();
    s.world
        .credit
        .as_mut()
        .unwrap()
        .stock_sales
        .as_mut()
        .unwrap()
        .forecast
        .as_mut()
        .unwrap()
        .need_limits
        .clear();
    assert!(Simulation::new(s.world, s.state, Backend::Reference).is_err());
}
#[test]
fn cpu_monthly_checkpoint_and_reordered_reference_match() {
    let mut cpu = sim();
    cpu.backend = Backend::CubeCpu;
    cpu.run_months(7).unwrap();
    let mut reference = cpu.clone();
    reference.backend = Backend::Reference;
    reference.world.definitions.reverse();
    reference.world.resources.reverse();
    let mut monthly = cpu.clone();
    cpu.run_months(11).unwrap();
    reference.run_months(11).unwrap();
    for _ in 0..11 {
        monthly.run_months(1).unwrap();
    }
    assert_eq!(cpu.state, reference.state);
    assert_eq!(cpu.ledger, reference.ledger);
    assert_eq!(cpu.state, monthly.state);
    assert_eq!(cpu.ledger, monthly.ledger);
}
