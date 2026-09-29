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
