use economics_compute_smoke::{
    compute::Backend,
    financial_reporting::{Audit, Opening},
    households::{self, market::EXAMPLE_HOUSEHOLD as HOME, support::Mandate},
    model::*,
    scenario::{GRAIN, NUTRITION, PERSON, TOKEN},
    settlement::{DEFAULT_EFFECT_LIMIT, commit},
    simulation::Simulation,
};
const STORED: ResourceId = 100;
fn audit(w: &World, s: &State) -> Audit {
    Audit::with_opening(
        w,
        s,
        TOKEN,
        Opening {
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
                .map(|(key, q)| (*key, i128::from(*q)))
                .collect(),
            processes: Some(Default::default()),
            ..Default::default()
        },
    )
    .unwrap()
}
fn stock(w: &mut World, id: ResourceId) {
    w.resources.push(Resource {
        id,
        name: format!("stored {id}"),
        kind: ResourceKind::Stock,
    });
    w.storage.weights.insert(id, 1);
}
fn support_fixture(blocked: bool) -> (World, State) {
    let (mut w, mut s) = households::market::scenario().unwrap();
    w.town_market = None;
    s.town_market = Default::default();
    w.activities.orders.clear();
    w.participants
        .iter_mut()
        .find(|p| p.agent == 91)
        .unwrap()
        .needs[0]
        .quantity = 2;
    stock(&mut w, STORED);
    w.storage.capacities.insert(PERSON, 10);
    w.storage.capacities.insert(91, 2);
    s.balances
        .insert((PERSON, GRAIN), if blocked { 5 } else { 6 });
    s.balances
        .insert((HOME, STORED), if blocked { 6 } else { 5 });
    w.households[0].support.push(Mandate {
        member: PERSON,
        resource: GRAIN,
        from: 1,
        through: 12,
        revoked_from: None,
        reserve_months: 1,
        private_reserve: 2,
        household_target: 4,
        monthly_limit: 4,
    });
    (w, s)
}
#[test]
fn partial_support_uses_shared_storage_and_retains_private_needs() {
    for blocked in [false, true] {
        let (w, s) = support_fixture(blocked);
        let run = |backend| {
            let mut sim = Simulation::new(w.clone(), s.clone(), backend).unwrap();
            let mut a = audit(&w, &s);
            while sim.state.month == 1 {
                a.step(&mut sim).unwrap();
            }
            let r = sim
                .ledger
                .iter()
                .filter_map(|b| b.household.as_ref())
                .flat_map(|b| &b.support)
                .next()
                .unwrap();
            assert_eq!(r.accepted, if blocked { 0 } else { 1 });
            assert!(r.offered > r.accepted);
            let food = sim
                .reports
                .iter()
                .find(|r| r.agent == 91)
                .unwrap()
                .deficit(NUTRITION);
            assert_eq!(food, if blocked { 2 } else { 1 });
            let mut replay = s.clone();
            for b in &sim.ledger {
                commit(&w, &mut replay, b, backend, DEFAULT_EFFECT_LIMIT).unwrap();
            }
            assert_eq!(replay, sim.state);
            (sim.state, sim.ledger, a)
        };
        assert_eq!(run(Backend::Reference), run(Backend::CubeCpu));
    }
}
