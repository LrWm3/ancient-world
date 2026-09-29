use economics_compute_smoke::{
    compute::Backend,
    employment::{ArrearsPolicy, Terms},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME},
    model::*,
    opportunities::{Action, PERSON_TYPE},
    scenario::{LABOR, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const PAY: ResourceId = 100;
const STORED: ResourceId = 101;
fn wages(room: i32) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    for (id, name) in [(PAY, "wage stock"), (STORED, "stored stock")] {
        w.resources.push(Resource {
            id,
            name: name.into(),
            kind: ResourceKind::Stock,
        });
        w.storage.weights.insert(id, 1);
    }
    for p in &mut w.participants {
        p.needs.clear();
        p.capacity.quantity = if p.agent == PERSON { 5 } else { 0 };
    }
    w.storage.capacities.insert(PERSON, 100);
    w.storage.capacities.insert(91, 2);
    w.storage.capacities.insert(89, 100);
    s.balances.insert((HOME, STORED), 51 - room);
    s.balances.insert((89, PAY), 4);
    w.transaction_policy
        .as_mut()
        .unwrap()
        .permissions
        .insert((PERSON_TYPE, Action::CapacityTrade));
    w.employment.push(Terms {
        id: 1,
        employer: 89,
        worker: PERSON,
        from: 1,
        through: 3,
        capacity: Amount::new(LABOR, 4),
        wage_per_unit: Amount::new(PAY, 1),
        on_arrears: ArrearsPolicy::SuspendDelivery,
        rank: 0,
    });
    (w, s)
}
#[test]
fn physical_wages_fit_both_private_and_collective_storage_and_preserve_arrears() {
    for (room, paid) in [(0, 1), (1, 3), (2, 4)] {
        let (w, s) = wages(room);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            sim.run_months(1).unwrap();
            assert_eq!(sim.state.employment.earned[&(1, 1)].claim.settled, paid);
            assert_eq!(sim.state.balance(HOME, PAY), paid / 2);
            assert_eq!(sim.state.balance(PERSON, PAY), paid - paid / 2);
            assert_eq!(sim.state.balance(89, PAY), 4 - paid);
            let mut replay = s.clone();
            for b in &sim.ledger {
                commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(replay, sim.state);
            if paid < 4 {
                sim.run_months(1).unwrap();
                assert!(!sim.state.employment.earned.contains_key(&(1, 2)));
                assert_eq!(sim.state.employment.earned[&(1, 1)].claim.settled, paid);
            }
            (sim.state, sim.ledger)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}

fn wage_audit(
    w: &World,
    s: &State,
    value: Option<i128>,
) -> Result<economics_compute_smoke::financial_reporting::Audit, String> {
    use economics_compute_smoke::financial_reporting::{Audit, Opening};
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
            exchange_values: value.map(|v| [(PAY, v)].into()).unwrap_or_default(),
            inventory: s
                .balances
                .iter()
                .filter(|((_, r), q)| {
                    *r != TOKEN
                        && **q > 0
                        && w.resources
                            .iter()
                            .any(|v| v.id == *r && v.kind == ResourceKind::Stock)
                })
                .map(|(key, q)| (*key, i128::from(*q) * if key.1 == PAY { 2 } else { 1 }))
                .collect(),
            processes: Some(Default::default()),
            services: Some(Default::default()),
            ..Default::default()
        },
    )
}
#[test]
fn physical_wages_value_claims_once_pool_costed_inventory_and_clear_after_room_returns() {
    use economics_compute_smoke::accounting::Account as A;
    let (w, s) = wages(1);
    let run = |backend| {
        let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
        let mut a = wage_audit(&w, &s, Some(3)).unwrap();
        while sim.state.month <= 2 {
            a.step(&mut sim).unwrap();
        }
        let balance = |id, account| {
            a.book()
                .balances()
                .get(&(id, account))
                .copied()
                .unwrap_or(0)
        };
        assert_eq!(balance(PERSON, A::WagesReceivable(1, 1)), 3);
        assert_eq!(balance(89, A::WagesPayable(1, 1)), -3);
        assert_eq!(balance(PERSON, A::ServiceIncome), -12);
        assert_eq!(balance(89, A::ServiceExpense), 12);
        assert_eq!(balance(89, A::Sales), -9);
        assert_eq!(balance(89, A::CostOfSales), 6);
        assert_eq!(balance(HOME, A::Inventory(PAY)), 3);
        assert!(
            a.book()
                .statements(PERSON, 1, 2)
                .unwrap()
                .cash_flows
                .is_empty()
        );
        sim.world.storage.capacities.insert(91, 4);
        let (mut resumed, mut saved) = (sim.clone(), a.clone());
        while sim.state.month == 3 {
            a.step(&mut sim).unwrap();
            saved.step(&mut resumed).unwrap();
        }
        assert_eq!((&sim.state, &a), (&resumed.state, &saved));
        assert_eq!(sim.state.employment.earned[&(1, 1)].claim.outstanding(), 0);
        assert_eq!(sim.state.balance(HOME, PAY), 2);
        assert_eq!(sim.state.balance(PERSON, PAY), 2);
        for id in [PERSON, HOME, 89] {
            let f = a.book().statements(id, 1, 3).unwrap();
            assert_eq!(f.assets, f.liabilities + f.equity);
        }
        (sim.state, sim.ledger, a)
    };
    assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
}
#[test]
fn missing_or_overflowing_wage_values_cannot_publish_financial_or_simulation_state() {
    let (w, s) = wages(2);
    assert!(wage_audit(&w, &s, None).is_err());
    let mut a = wage_audit(&w, &s, Some(i128::MAX)).unwrap();
    let mut sim = Simulation::new(w, s, Backend::Reference).unwrap();
    while sim.state.phase != Phase::Acquire {
        a.step(&mut sim).unwrap();
    }
    let before = (sim.state.clone(), a.clone());
    assert!(a.step(&mut sim).unwrap_err().contains("overflow"));
    assert_eq!(before, (sim.state, a));
}
